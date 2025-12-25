# Health Check API

The 3D Quote Service provides Kubernetes-style health check endpoints for monitoring and orchestration.

## Endpoints

### GET /health - Liveness Probe

**Purpose**: Indicates if the service is running and responsive.

**Use case**: Kubernetes liveness probe, Docker healthcheck, load balancer health check

**Behavior**:
- Always returns `200 OK` if the service is running
- Never performs expensive checks
- Should complete in milliseconds

**Response**:

```http
GET /health HTTP/1.1
Host: localhost:8000

HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "ok"
}
```

**When to use**:
- Kubernetes `livenessProbe`
- Docker `HEALTHCHECK`
- Load balancer health check (for basic availability)
- Uptime monitoring (pingdom, UptimeRobot, etc.)

**Example Kubernetes configuration**:

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8000
  initialDelaySeconds: 10
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
```

---

### GET /ready - Readiness Probe

**Purpose**: Indicates if the service is ready to handle requests.

**Use case**: Kubernetes readiness probe, load balancer traffic routing

**Behavior**:
- Returns `200 OK` if all dependencies are healthy
- Returns `503 Service Unavailable` if any dependency fails
- Checks:
  - **Database**: Can execute queries (`SELECT 1`)
  - **Filesystem**: Can write to upload directory

**Response (healthy)**:

```http
GET /ready HTTP/1.1
Host: localhost:8000

HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "ready",
  "checks": {
    "database": {
      "status": "ok"
    },
    "filesystem": {
      "status": "ok"
    }
  }
}
```

**Response (unhealthy)**:

```http
GET /ready HTTP/1.1
Host: localhost:8000

HTTP/1.1 503 Service Unavailable
Content-Type: application/json

{
  "status": "not_ready",
  "checks": {
    "database": {
      "status": "error",
      "message": "Database connection failed: connection refused"
    },
    "filesystem": {
      "status": "ok"
    }
  }
}
```

**When to use**:
- Kubernetes `readinessProbe`
- Load balancer health check (for traffic routing)
- Pre-deployment verification
- Dependency monitoring

**Example Kubernetes configuration**:

```yaml
readinessProbe:
  httpGet:
    path: /ready
    port: 8000
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
  successThreshold: 1
```

---

## Health Check Details

### Database Check

**What it checks**: PostgreSQL connection and query execution

**Implementation**:
```sql
SELECT 1
```

**Failure scenarios**:
- Database server is down
- Connection pool exhausted
- Network partition between app and database
- Database credentials invalid

**Recovery**:
- Returns to healthy once database connection is restored
- No manual intervention required

### Filesystem Check

**What it checks**: Write access to upload directory

**Implementation**:
- Writes test file: `{UPLOAD_DIR}/.health_check`
- Reads back the file
- Deletes test file

**Failure scenarios**:
- Upload directory doesn't exist
- Insufficient disk space
- Permission denied
- Filesystem mounted read-only

**Recovery**:
- Fix filesystem issue (create directory, free disk space, fix permissions)
- Returns to healthy automatically

---

## Integration Examples

### Docker Compose

```yaml
services:
  app:
    image: 3d-quote-service:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/ready"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: quote-service
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: app
        image: 3d-quote-service:latest
        ports:
        - containerPort: 8000
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
```

### Nginx Upstream Health Check

```nginx
upstream quote_service {
    server 127.0.0.1:8000 max_fails=3 fail_timeout=30s;

    # Nginx Plus (commercial) supports active health checks
    health_check interval=10s fails=3 passes=2 uri=/ready;
}
```

### Load Balancer (HAProxy)

```haproxy
backend quote_service
    option httpchk GET /ready
    http-check expect status 200
    server app1 127.0.0.1:8000 check inter 10s fall 3 rise 2
```

### Systemd Service Monitor

Create a systemd timer to monitor health:

```ini
# /etc/systemd/system/quote-health.service
[Unit]
Description=Quote Service Health Check
After=quote-service.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/check-quote-health.sh

# /usr/local/bin/check-quote-health.sh
#!/bin/bash
curl -f http://localhost:8000/ready || systemctl restart quote-service
```

```ini
# /etc/systemd/system/quote-health.timer
[Unit]
Description=Quote Service Health Check Timer

[Timer]
OnBootSec=5min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
```

---

## Monitoring and Alerts

### Prometheus Exporter (Future)

The `/ready` endpoint can be monitored with Prometheus:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'quote-service-health'
    metrics_path: /ready
    scrape_interval: 30s
    static_configs:
      - targets: ['localhost:8000']
```

**Alert rule**:

```yaml
groups:
  - name: quote-service
    rules:
      - alert: QuoteServiceNotReady
        expr: up{job="quote-service-health"} == 0
        for: 2m
        annotations:
          summary: "Quote service is not ready"
          description: "Quote service has been unhealthy for 2 minutes"
```

### SigNoz / OpenTelemetry

Health check status is automatically included in traces and metrics:

```rust
// Metric: health_check_status (1 = healthy, 0 = unhealthy)
// Trace: Automatic span for each health check request
```

View in SigNoz:
- Navigate to **Services** → **quote-service** → **Metrics**
- Filter by `http.route="/ready"`

### Simple Uptime Monitoring

Use services like UptimeRobot, Pingdom, or StatusCake:

- **URL**: `https://quote.example.com/health`
- **Interval**: Every 5 minutes
- **Expected**: HTTP 200
- **Alert**: Email/SMS on downtime

---

## Troubleshooting

### Health check always fails in Docker

**Symptom**:
```bash
docker ps
# STATUS: unhealthy
```

**Diagnosis**:
```bash
docker exec quote-service-app curl http://localhost:8000/ready
# Check response
```

**Common causes**:
1. Wrong port in `HEALTHCHECK` (should match `PORT` env var)
2. App not binding to `0.0.0.0` (check `HOST` env var)
3. Database not ready (increase `start_period`)

**Solution**:
```dockerfile
# Increase start period if database takes long to initialize
HEALTHCHECK --start-period=60s ...
```

### Readiness probe flapping (ready → not ready → ready)

**Symptom**: Service alternates between ready and not ready

**Diagnosis**:
```bash
# Check logs for database connection errors
journalctl -u quote-service | grep -i "database\|ready"

# Check database connection pool
psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity WHERE usename='quoteuser';"
```

**Common causes**:
1. Database connection pool exhausted
2. Filesystem full (upload directory)
3. Slow database queries (increase timeout)

**Solution**:
- Increase database connection pool size
- Clean up old files (run cleanup job)
- Increase readiness probe timeout

### Kubernetes pod restarting frequently

**Symptom**: Pod shows many restarts

**Diagnosis**:
```bash
kubectl describe pod quote-service-xxx
# Check Events for liveness probe failures

kubectl logs quote-service-xxx --previous
# Check logs from previous pod
```

**Solution**:
```yaml
# Adjust probe parameters
livenessProbe:
  initialDelaySeconds: 30  # Increase if slow startup
  periodSeconds: 15        # Increase if probe is too frequent
  timeoutSeconds: 10       # Increase if responses are slow
  failureThreshold: 5      # Increase to tolerate transient failures
```

---

## Performance Impact

### /health Endpoint

- **Latency**: < 1ms (no I/O operations)
- **Resource usage**: Negligible (single JSON serialization)
- **Rate limit**: Not rate limited (critical infrastructure)

**Safe polling interval**: Every 1 second

### /ready Endpoint

- **Latency**: 10-50ms (database query + filesystem write)
- **Resource usage**:
  - 1 database connection from pool
  - 1 small file write + delete
  - Negligible CPU/memory

**Safe polling interval**: Every 10 seconds

**Recommended Kubernetes configuration**:

```yaml
readinessProbe:
  periodSeconds: 10  # Not more frequent than 10s
  timeoutSeconds: 5
```

---

## Future Enhancements

Planned improvements for health checks:

1. **Metrics export**: Expose health status as Prometheus metrics
2. **Custom checks**: Allow plugins for custom health checks
3. **Degraded state**: Support "degraded but operational" status
4. **Dependency details**: Version info, latency stats per dependency
5. **Historical data**: Track health check history in observability platform

---

## API Reference

### GET /health

**Response Schema**:

```json
{
  "status": "ok"
}
```

**Status codes**:
- `200 OK`: Service is running

### GET /ready

**Response Schema**:

```json
{
  "status": "ready" | "not_ready",
  "checks": {
    "database": {
      "status": "ok" | "error",
      "message"?: "Error message if status is error"
    },
    "filesystem": {
      "status": "ok" | "error",
      "message"?: "Error message if status is error"
    }
  }
}
```

**Status codes**:
- `200 OK`: All checks passed, service is ready
- `503 Service Unavailable`: One or more checks failed

---

## Support

For health check issues:
- **Documentation**: [docs/deployment/troubleshooting.md](../deployment/troubleshooting.md)
- **GitHub Issues**: https://github.com/your-org/3d-assistant/issues
- **Monitoring**: SigNoz dashboard → Services → quote-service → Health
