# Quickstart: Production Deployment and Observability

**Feature**: 003-production-observability
**Audience**: Developers testing observability locally
**Prerequisites**: Docker, Docker Compose, Rust 1.75+, PostgreSQL

## Quick Start: Local Development with Observability

### 1. Start Observability Stack (SigNoz)

```bash
# From repository root
cd deployment
docker-compose -f docker-compose.observability.yml up -d

# Wait for services to be ready (~30 seconds)
docker-compose -f docker-compose.observability.yml ps
```

**Access SigNoz UI**: http://localhost:3301

### 2. Configure Application

```bash
# Copy environment template
cp .env.example .env.local

# Edit .env.local with observability settings
cat >> .env.local << EOF
# Observability Configuration
ENVIRONMENT=development
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=quote-service-dev
EOF

# Source environment
export $(cat .env.local | xargs)
```

### 3. Run Application with Telemetry

```bash
# Build and run
cargo run

# Or with explicit env file
env $(cat .env.local | xargs) cargo run
```

### 4. Generate Test Traffic

```bash
# Upload a 3D file
curl -X POST http://localhost:3000/api/sessions \
  -H "Content-Type: application/json" \
  | jq -r '.session_id' > session_id.txt

curl -X POST http://localhost:3000/api/sessions/$(cat session_id.txt)/upload \
  -F "file=@tests/fixtures/cube.stl"

# Generate a quote
curl http://localhost:3000/api/sessions/$(cat session_id.txt)/quote \
  | jq '.'
```

### 5. View Telemetry in SigNoz

1. **Open SigNoz**: http://localhost:3301
2. **Navigate to Traces**: See distributed traces of your requests
3. **Navigate to Metrics**: View business and technical metrics
4. **Navigate to Logs**: See structured JSON logs with trace correlation

## Testing Health Endpoints

### Liveness Check

```bash
# Should return 200 OK immediately
curl -v http://localhost:3000/health

# Expected response:
# HTTP/1.1 200 OK
```

### Readiness Check

```bash
# Should return 200 OK with health details
curl http://localhost:3000/ready | jq '.'

# Expected response:
{
  "status": "healthy",
  "checks": [
    {
      "component": "database",
      "status": "up",
      "message": null
    },
    {
      "component": "filesystem",
      "status": "up",
      "message": null
    }
  ]
}
```

### Test Unhealthy State

```bash
# Stop database
docker-compose stop postgres

# Check readiness (should return 503)
curl -v http://localhost:3000/ready

# Expected: HTTP/1.1 503 Service Unavailable
# Response shows which component failed:
{
  "status": "unhealthy",
  "checks": [
    {
      "component": "database",
      "status": "down",
      "message": "Connection refused"
    },
    {
      "component": "filesystem",
      "status": "up",
      "message": null
    }
  ]
}

# Restart database
docker-compose start postgres
```

## Exploring Telemetry Data

### View Request Traces

In SigNoz UI:
1. Go to **Traces** tab
2. Filter by service: `quote-service-dev`
3. Click on a trace to see timeline:
   - HTTP request span
   - File upload span (if applicable)
   - Database query spans
   - Business logic spans (volume calculation, pricing)

### View Metrics

In SigNoz UI:
1. Go to **Metrics** tab
2. Explore available metrics:
   - `quotes_generated_total` - Counter of quotes
   - `http_request_duration_ms` - Request latency
   - `db_connections_active` - Connection pool usage
3. Create custom visualizations (line graphs, histograms)

### View Logs

In SigNoz UI:
1. Go to **Logs** tab
2. Filter by `trace_id` to see logs for a specific request
3. Filter by `level=ERROR` to see only errors
4. Use full-text search on `message` field

## Testing Production Configuration

### Simulate Production Mode

```bash
# Set environment to production
export ENVIRONMENT=production
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Run application
cargo run

# Logs are now in JSON format:
{"timestamp":"2025-12-24T10:30:45.123Z","level":"INFO","message":"Starting server","trace_id":null}
```

### Test Configuration Validation

```bash
# Unset required variable
unset DATABASE_URL

# Try to start (should fail fast)
cargo run

# Expected error:
# ERROR: Missing required environment variable: DATABASE_URL
# Please set DATABASE_URL to a valid PostgreSQL connection string
```

## Load Testing

Generate load to test observability under stress:

```bash
# Install bombardier (or use your preferred load tester)
go install github.com/codesenberg/bombardier@latest

# Generate 100 requests/second for 30 seconds
bombardier -c 10 -d 30s -r 100 http://localhost:3000/health

# View metrics in SigNoz to observe:
# - Request rate increasing
# - Latency distribution (p50, p95, p99)
# - Error rate (should be 0% for /health)
```

## Troubleshooting

### Telemetry Not Appearing in SigNoz

**Check OpenTelemetry Collector**:
```bash
# View collector logs
docker-compose -f deployment/docker-compose.observability.yml logs otel-collector

# Should see: "Trace received" messages
```

**Check Application Logs**:
```bash
# Application should log telemetry initialization
# Look for: "OpenTelemetry initialized" or similar
```

**Verify Endpoint**:
```bash
# Ensure OTLP endpoint is reachable
curl -v http://localhost:4317

# Should connect (even if HTTP method not supported)
```

### Database Connection Failures

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Check DATABASE_URL is correct
echo $DATABASE_URL

# Test connection manually
psql $DATABASE_URL -c "SELECT 1"
```

### Port Conflicts

```bash
# Check if ports are already in use
lsof -i :3000  # Application
lsof -i :3301  # SigNoz UI
lsof -i :4317  # OTLP gRPC
lsof -i :4318  # OTLP HTTP

# Stop conflicting services or change ports in docker-compose
```

## Next Steps

After local testing:

1. **Review Traces**: Understand normal request patterns
2. **Set Baselines**: Note typical latencies and error rates
3. **Test Failure Scenarios**: Simulate DB down, filesystem full, high load
4. **Deploy to Staging**: Follow deployment guides in `docs/deployment/`
5. **Configure Alerts**: Set up SigNoz alerts for error rates, latency spikes

## Cleanup

```bash
# Stop observability stack
docker-compose -f deployment/docker-compose.observability.yml down

# Remove volumes (clears all telemetry data)
docker-compose -f deployment/docker-compose.observability.yml down -v
```

## Additional Resources

- [SigNoz Documentation](https://signoz.io/docs/)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Deployment Guides](../../docs/deployment/)
- [Observability Docs](../../docs/observability/)
