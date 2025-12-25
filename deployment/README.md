# Deployment Guide

This directory contains deployment configurations for the 3D Quote Service.

## Quick Start

### Development Observability Stack

Run the observability stack locally for development:

```bash
# Start SigNoz stack
docker-compose -f deployment/docker-compose.observability.yml up -d

# Access SigNoz UI
open http://localhost:3301

# Run your application with OTEL enabled
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run
```

### Production Deployment

Full production stack with application, database, and observability:

```bash
# Create .env file with required secrets
cp .env.example .env
# Edit .env and set POSTGRES_PASSWORD, ADMIN_TOKEN, MCP_TOKEN

# Start all services (app + db + observability)
docker-compose -f deployment/docker-compose.prod.yml --profile observability up -d

# Or start just app + db (without observability)
docker-compose -f deployment/docker-compose.prod.yml up -d postgres app
```

## Files

- **docker-compose.observability.yml**: Local development observability stack (SigNoz only)
- **docker-compose.prod.yml**: Production stack with app, database, and optional observability
- **otel-collector-config.yml**: OpenTelemetry Collector configuration
- **signoz-query-service-config.yml**: SigNoz query service configuration
- **README.md**: This file

## Observability Stack Components

### SigNoz (All-in-One)

SigNoz provides:
- **Traces**: Distributed tracing with OpenTelemetry
- **Metrics**: Business and technical metrics
- **Logs**: Structured logs with trace correlation

**Access**: http://localhost:3301

### Architecture

```
┌──────────────────┐
│  Quote Service   │
│   (Rust/Axum)    │
└────────┬─────────┘
         │ OTLP (gRPC)
         │ Port 4317
         ▼
┌──────────────────┐
│ OTEL Collector   │
│  (Processing)    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   ClickHouse     │ ◄──┐
│   (Storage)      │    │
└──────────────────┘    │
         │              │
         ▼              │
┌──────────────────┐    │
│ Query Service    │────┘
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  SigNoz UI       │
│  (Frontend)      │
└──────────────────┘
```

## Ports

| Service            | Port  | Description                    |
|--------------------|-------|--------------------------------|
| App                | 8000  | Quote service HTTP API         |
| PostgreSQL         | 5432  | Database                       |
| OTEL Collector     | 4317  | OTLP gRPC receiver             |
| OTEL Collector     | 4318  | OTLP HTTP receiver             |
| SigNoz UI          | 3301  | Observability dashboard        |
| SigNoz Query       | 8080  | Query service API              |
| ClickHouse Native  | 9000  | ClickHouse native protocol     |
| ClickHouse HTTP    | 8123  | ClickHouse HTTP interface      |

## Resource Limits

Production deployment uses resource limits:

| Service        | CPU Limit | Memory Limit | CPU Reserve | Memory Reserve |
|----------------|-----------|--------------|-------------|----------------|
| App            | 2 cores   | 1GB          | 0.5 cores   | 512MB          |
| PostgreSQL     | -         | -            | -           | -              |
| ClickHouse     | 2 cores   | 2GB          | 1 core      | 1GB            |
| OTEL Collector | 1 core    | 512MB        | 0.25 cores  | 256MB          |
| Query Service  | 1 core    | 1GB          | 0.5 cores   | 512MB          |
| Frontend       | 0.5 cores | 512MB        | 0.25 cores  | 256MB          |

**Minimum requirements**: 4 CPU cores, 6GB RAM for full stack with observability.

## Volumes

| Volume Name                    | Purpose                        |
|--------------------------------|--------------------------------|
| quote-service-postgres-data    | PostgreSQL database files      |
| quote-service-uploads          | User-uploaded 3D model files   |
| signoz-clickhouse-data         | Telemetry data (traces/metrics)|

## Environment Variables

See [../docs/deployment/configuration.md](../docs/deployment/configuration.md) for complete environment variable reference.

**Required in production**:
- `POSTGRES_PASSWORD`: PostgreSQL password
- `ADMIN_TOKEN`: Admin API token
- `MCP_TOKEN`: MCP authentication token

**Optional**:
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP collector endpoint (default: http://otel-collector:4317)
- `OTEL_SERVICE_NAME`: Service name for telemetry (default: quote-service)
- `MAX_FILE_SIZE_MB`: Max upload size (default: 50)

## Health Checks

All services include health checks:

```bash
# Check app health
curl http://localhost:8000/health

# Check PostgreSQL
docker exec quote-service-postgres pg_isready

# Check SigNoz Query Service
curl http://localhost:8080/api/v1/health

# Check ClickHouse
curl http://localhost:8123/ping
```

## Troubleshooting

### Observability stack not receiving data

1. Check OTEL Collector logs:
   ```bash
   docker logs signoz-otel-collector
   ```

2. Verify endpoint configuration:
   ```bash
   echo $OTEL_EXPORTER_OTLP_ENDPOINT
   # Should be: http://localhost:4317 (dev) or http://otel-collector:4317 (docker)
   ```

3. Test OTLP endpoint connectivity:
   ```bash
   nc -zv localhost 4317
   ```

### High resource usage

ClickHouse can be resource-intensive. For development:

```bash
# Use observability stack only when needed
docker-compose -f deployment/docker-compose.observability.yml stop

# Or run without observability
docker-compose -f deployment/docker-compose.prod.yml up -d postgres app
```

### Container fails to start

Check logs:
```bash
docker-compose -f deployment/docker-compose.prod.yml logs app
docker-compose -f deployment/docker-compose.prod.yml logs postgres
```

Common issues:
- Missing environment variables (POSTGRES_PASSWORD, ADMIN_TOKEN)
- Port conflicts (8000, 5432 already in use)
- Insufficient disk space for volumes

## Production Deployment

See [../docs/deployment/](../docs/deployment/) for detailed guides:

- **VPS Deployment**: Ubuntu with systemd
- **CleverCloud**: PaaS deployment
- **Security Checklist**: Production hardening
- **Troubleshooting**: Common issues and solutions

## Monitoring Best Practices

1. **Enable observability in production**: Use `--profile observability`
2. **Set up alerts**: Configure SigNoz alerts for critical metrics
3. **Monitor disk usage**: ClickHouse data grows over time
4. **Retention policy**: Configure data retention in SigNoz (default: 7 days)
5. **Backup volumes**: Regular backups of postgres-data and clickhouse-data

## Alternative: Grafana Stack

For a more customizable stack, see `docker-compose.grafana.yml` (TODO) which uses:
- Tempo (traces)
- Loki (logs)
- Prometheus (metrics)
- Grafana (visualization)

SigNoz is recommended for simplicity and out-of-box experience.
