# Docker Compose Deployment Guide

This guide covers deploying the 3D Quote Service using Docker Compose for production or staging environments.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- 4 CPU cores minimum (with observability stack)
- 6GB RAM minimum (with observability stack)
- 20GB disk space minimum

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/your-org/3d-assistant.git
cd 3d-assistant
```

### 2. Configure Environment

```bash
# Copy production environment template
cp deployment/.env.production.example deployment/.env

# Edit with your values
nano deployment/.env
```

**Required changes**:
```bash
POSTGRES_PASSWORD=your_secure_password_here
ADMIN_TOKEN=$(openssl rand -base64 32)
MCP_TOKEN=$(openssl rand -base64 32)
```

### 3. Start Services

**Option A: Full stack with observability**

```bash
docker-compose -f deployment/docker-compose.prod.yml --profile observability up -d
```

**Option B: App + Database only (no observability)**

```bash
docker-compose -f deployment/docker-compose.prod.yml up -d postgres app
```

### 4. Verify Deployment

```bash
# Check all services are running
docker-compose -f deployment/docker-compose.prod.yml ps

# Check application health
curl http://localhost:8000/health

# View logs
docker-compose -f deployment/docker-compose.prod.yml logs -f app
```

### 5. Access Services

- **Application**: http://localhost:8000
- **SigNoz UI**: http://localhost:3301 (if observability enabled)
- **API Docs**: http://localhost:8000/api (TODO: add Swagger)

## Configuration

### Environment Variables

See [configuration.md](./configuration.md) for complete reference.

**Critical variables**:

| Variable          | Description                    | Example                         |
|-------------------|--------------------------------|---------------------------------|
| DATABASE_URL      | PostgreSQL connection          | postgres://user:pass@postgres:5432/quotes |
| ADMIN_TOKEN       | Admin API authentication       | Generate with `openssl rand -base64 32` |
| MCP_TOKEN         | MCP authentication             | Generate with `openssl rand -base64 32` |
| ENVIRONMENT       | Environment name               | production                      |
| OTEL_EXPORTER_OTLP_ENDPOINT | OTLP collector | http://otel-collector:4317 |

### Resource Limits

Edit `deployment/docker-compose.prod.yml` to adjust:

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
```

### Volumes

**Persistent data volumes**:

| Volume                      | Purpose                  | Backup Priority |
|-----------------------------|--------------------------|-----------------|
| quote-service-postgres-data | Database                 | Critical        |
| quote-service-uploads       | User-uploaded files      | High            |
| signoz-clickhouse-data      | Telemetry data           | Medium          |

**Backup example**:

```bash
# Backup database
docker exec quote-service-postgres pg_dump -U quoteuser quotes > backup_$(date +%Y%m%d).sql

# Backup uploads
docker run --rm -v quote-service-uploads:/data -v $(pwd):/backup alpine tar czf /backup/uploads_$(date +%Y%m%d).tar.gz /data
```

## Production Deployment

### SSL/TLS with Nginx

Add nginx reverse proxy for HTTPS:

```yaml
# docker-compose.prod.yml (add this service)
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /etc/letsencrypt:/etc/letsencrypt:ro
    depends_on:
      - app
    networks:
      - app-network
    restart: unless-stopped
```

**nginx.conf**:

```nginx
server {
    listen 80;
    server_name quote.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name quote.example.com;

    ssl_certificate /etc/letsencrypt/live/quote.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/quote.example.com/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;

    # Proxy to app
    location / {
        proxy_pass http://app:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support (if needed)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # File upload size limit
    client_max_body_size 50M;
}
```

### Automated Backups

Create backup script `backup.sh`:

```bash
#!/bin/bash
set -e

BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Backup database
docker exec quote-service-postgres pg_dump -U quoteuser quotes | gzip > "$BACKUP_DIR/db_$DATE.sql.gz"

# Backup uploads
docker run --rm -v quote-service-uploads:/data -v $BACKUP_DIR:/backup alpine \
  tar czf /backup/uploads_$DATE.tar.gz /data

# Keep only last 7 days
find $BACKUP_DIR -name "db_*.sql.gz" -mtime +7 -delete
find $BACKUP_DIR -name "uploads_*.tar.gz" -mtime +7 -delete

echo "Backup completed: $DATE"
```

Add to crontab:

```bash
# Run daily at 2 AM
0 2 * * * /path/to/backup.sh >> /var/log/quote-backup.log 2>&1
```

### Log Rotation

Configure log rotation in `/etc/docker/daemon.json`:

```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
```

Restart Docker:

```bash
sudo systemctl restart docker
```

## Monitoring

### Health Checks

All services include health checks. View status:

```bash
# Check service health
docker-compose -f deployment/docker-compose.prod.yml ps

# Test application endpoint
curl http://localhost:8000/health
# Expected: {"status":"ok"}

# Test database
docker exec quote-service-postgres pg_isready -U quoteuser
# Expected: /var/run/postgresql:5432 - accepting connections
```

### Observability

With observability stack enabled:

1. **Access SigNoz**: http://localhost:3301
2. **View traces**: Services → quote-service → Traces
3. **Check metrics**: Dashboards → Create custom dashboard
4. **Query logs**: Logs → Filter by service

**Key metrics to monitor**:
- `quotes_generated_total`: Business metric
- `http_request_duration_ms`: Latency
- `http_requests_total`: Traffic
- `db_connections_active`: Database health

### Resource Monitoring

```bash
# Monitor container resources
docker stats

# Check disk usage
docker system df

# Check volume sizes
docker system df -v | grep quote-service
```

## Maintenance

### Updates

Update to new version:

```bash
# Pull latest images
docker-compose -f deployment/docker-compose.prod.yml pull

# Recreate containers with new images
docker-compose -f deployment/docker-compose.prod.yml up -d

# Check logs for errors
docker-compose -f deployment/docker-compose.prod.yml logs -f app
```

### Database Migrations

Migrations run automatically on startup. Manual trigger:

```bash
# Access app container
docker exec -it quote-service-app /bin/bash

# Run migrations manually (if needed)
# Migrations are embedded in binary, auto-run on startup
```

### Cleanup

```bash
# Remove stopped containers
docker-compose -f deployment/docker-compose.prod.yml down

# Remove volumes (WARNING: deletes all data)
docker-compose -f deployment/docker-compose.prod.yml down -v

# Clean unused images
docker image prune -a
```

## Troubleshooting

### Application won't start

**Check logs**:
```bash
docker-compose -f deployment/docker-compose.prod.yml logs app
```

**Common issues**:
- Missing environment variables → Check `.env` file
- Database connection failed → Verify `DATABASE_URL`
- Port already in use → Change port in `docker-compose.prod.yml`

### Database connection errors

```bash
# Check PostgreSQL is running
docker-compose -f deployment/docker-compose.prod.yml ps postgres

# Check PostgreSQL logs
docker-compose -f deployment/docker-compose.prod.yml logs postgres

# Test connection from app container
docker exec quote-service-app ping postgres
```

### High memory usage

```bash
# Check resource usage
docker stats

# If ClickHouse is using too much memory, reduce limits:
# Edit docker-compose.prod.yml → clickhouse → deploy.resources.limits.memory
```

### Observability not working

```bash
# Check OTEL Collector logs
docker logs signoz-otel-collector

# Verify OTLP endpoint
docker exec quote-service-app env | grep OTEL

# Test connectivity
docker exec quote-service-app nc -zv otel-collector 4317
```

## Security Checklist

- [ ] Changed default ADMIN_TOKEN
- [ ] Changed default MCP_TOKEN
- [ ] Set strong POSTGRES_PASSWORD
- [ ] Configured firewall (only expose 80/443)
- [ ] Enabled HTTPS with valid SSL certificate
- [ ] Restricted file permissions on .env (chmod 600)
- [ ] Enabled automated backups
- [ ] Configured log rotation
- [ ] Reviewed resource limits
- [ ] Tested disaster recovery

## Performance Tuning

### PostgreSQL

Edit PostgreSQL settings via environment:

```yaml
services:
  postgres:
    environment:
      # Increase shared buffers
      - POSTGRES_SHARED_BUFFERS=256MB
      # Increase work memory
      - POSTGRES_WORK_MEM=16MB
```

### Application

Increase worker threads:

```yaml
services:
  app:
    environment:
      - RUST_LOG=info
      # Add custom runtime config if needed
```

## Support

- **Documentation**: [docs/deployment/](./README.md)
- **Issues**: https://github.com/your-org/3d-assistant/issues
- **Security**: security@example.com

## Next Steps

- [ ] Configure monitoring alerts
- [ ] Set up CI/CD pipeline
- [ ] Review [security.md](./security.md)
- [ ] Read [troubleshooting.md](./troubleshooting.md)
