# Deployment Documentation

Complete deployment guides for the 3D Quote Service across different platforms.

## Quick Links

- **[Configuration Reference](./configuration.md)** - Environment variables and settings
- **[Docker Compose Deployment](./docker-compose.md)** - Production deployment with Docker
- **[VPS/VM Deployment](./vps.md)** - Ubuntu/Debian server deployment
- **[Security Checklist](./security.md)** - Production security hardening
- **[Troubleshooting Guide](./troubleshooting.md)** - Common issues and solutions

## Deployment Options

### 1. Docker Compose (Recommended)

**Best for**: Production deployments, staging environments, easy scaling

**Pros**:
- One-command deployment
- Includes database and observability stack
- Easy to update and rollback
- Resource limits configured
- Health checks built-in

**Cons**:
- Requires Docker knowledge
- Slightly higher resource usage

**[Read Docker Compose Guide →](./docker-compose.md)**

### 2. VPS/VM with Systemd

**Best for**: Traditional server deployments, dedicated VPS, full control

**Pros**:
- Full system control
- Lower overhead (no Docker)
- Traditional deployment model
- Easy integration with existing infrastructure

**Cons**:
- More manual setup
- System-specific configuration
- Manual dependency management

**[Read VPS Deployment Guide →](./vps.md)**

### 3. CleverCloud (PaaS)

**Best for**: Minimal ops overhead, automatic scaling, managed infrastructure

**Pros**:
- Zero ops (fully managed)
- Auto-scaling
- Built-in PostgreSQL
- Automatic HTTPS
- Git-based deployment

**Cons**:
- Platform lock-in
- Limited observability options (use external SigNoz)
- Cost at scale

**Status**: Documentation TODO

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Internet                              │
└────────────────────────────┬────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  Nginx (HTTPS)  │
                    │  Reverse Proxy  │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Quote Service  │
                    │    (Rust/Axum)  │
                    └────┬─────┬──────┘
                         │     │
         ┌───────────────┘     └──────────────┐
         │                                     │
    ┌────▼────────┐                  ┌────────▼─────────┐
    │ PostgreSQL  │                  │ OTEL Collector   │
    │  Database   │                  │  (Telemetry)     │
    └─────────────┘                  └────────┬─────────┘
                                              │
                                     ┌────────▼─────────┐
                                     │     SigNoz       │
                                     │  (Observability) │
                                     └──────────────────┘
```

## Minimum Requirements

### Production (with observability)

- **CPU**: 4 cores
- **RAM**: 6GB
- **Disk**: 20GB SSD
- **OS**: Ubuntu 22.04 LTS or Debian 12

### Production (app + database only)

- **CPU**: 2 cores
- **RAM**: 2GB
- **Disk**: 10GB SSD
- **OS**: Ubuntu 22.04 LTS or Debian 12

### Development

- **CPU**: 2 cores
- **RAM**: 4GB (with SigNoz) or 2GB (without)
- **Disk**: 5GB
- **OS**: Any Docker-compatible OS

## Environment Variables

See **[Configuration Reference](./configuration.md)** for complete list.

**Required in production**:
```bash
DATABASE_URL=postgres://user:password@host:5432/quotes
ADMIN_TOKEN=$(openssl rand -base64 32)
MCP_TOKEN=$(openssl rand -base64 32)
ENVIRONMENT=production
```

**Optional**:
```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
OTEL_SERVICE_NAME=quote-service
MAX_FILE_SIZE_MB=50
SESSION_EXPIRY_HOURS=24
```

## Deployment Checklist

Use this checklist for any production deployment:

### Pre-Deployment

- [ ] Choose deployment platform (Docker Compose / VPS / CleverCloud)
- [ ] Provision server with minimum requirements
- [ ] Acquire domain name (optional but recommended)
- [ ] Plan backup strategy
- [ ] Review security checklist

### Deployment

- [ ] Clone repository
- [ ] Configure environment variables (.env)
- [ ] Change all default secrets (ADMIN_TOKEN, MCP_TOKEN, passwords)
- [ ] Set up database (PostgreSQL)
- [ ] Build application or pull Docker image
- [ ] Start application
- [ ] Verify health checks pass

### Post-Deployment

- [ ] Configure HTTPS/SSL
- [ ] Set up firewall rules
- [ ] Enable automated backups
- [ ] Configure monitoring and alerts
- [ ] Test disaster recovery
- [ ] Document deployment details
- [ ] Complete [security checklist](./security.md)

## Observability

The application includes comprehensive observability via OpenTelemetry:

### Traces

Distributed tracing for all HTTP requests and key operations:
- File uploads
- Quote calculations
- Database queries
- Session cleanup

**Access**: SigNoz UI → Services → quote-service → Traces

### Metrics

Business and technical metrics:
- `quotes_generated_total` - Business KPI
- `models_uploaded_total` - Upload activity
- `http_request_duration_ms` - Latency
- `db_connections_active` - Database health

**Access**: SigNoz UI → Dashboards

### Logs

Structured logs with trace correlation:
- JSON format in production (machine-readable)
- Pretty format in development (human-readable)
- Automatic trace_id/span_id correlation

**Access**: SigNoz UI → Logs

### Optional Observability

If you prefer not to run the observability stack:

```bash
# Docker Compose: Run without --profile observability
docker-compose -f deployment/docker-compose.prod.yml up -d postgres app

# VPS: Don't install OTEL Collector

# Application will continue to work, but telemetry won't be exported
```

## Security

Security is critical for production deployments. Follow the **[Security Checklist](./security.md)**.

**Key security measures**:

1. **Authentication**: All admin endpoints require token
2. **HTTPS**: Always use HTTPS in production
3. **Secrets**: Never commit secrets, use strong tokens
4. **Firewall**: Only expose necessary ports (80, 443)
5. **Updates**: Regular security updates for dependencies
6. **Backups**: Automated, encrypted, off-site backups
7. **Monitoring**: Alert on suspicious activity

## Maintenance

### Regular Tasks

**Daily**: Automated backups
**Weekly**: Review logs and metrics
**Monthly**: Security updates, backup restore test
**Quarterly**: Dependency updates, security audit
**Annually**: Penetration testing, disaster recovery drill

### Updates

**Application updates**:

```bash
# Docker Compose
docker-compose -f deployment/docker-compose.prod.yml pull
docker-compose -f deployment/docker-compose.prod.yml up -d

# VPS
cd /home/quoteapp/3d-assistant
git pull origin main
cargo build --release
sudo systemctl restart quote-service
```

**System updates**:

```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade -y
sudo reboot  # If kernel updated
```

### Backups

**What to backup**:
- PostgreSQL database (critical)
- User uploads directory (high priority)
- Configuration files (.env) (high priority)
- Telemetry data (optional, can be regenerated)

**Backup frequency**:
- Database: Daily, retain 7 days
- Uploads: Daily, retain 7 days
- Configuration: On change

**Backup storage**:
- Off-site (S3, B2, rsync to remote server)
- Encrypted at rest
- Tested quarterly

See deployment guides for backup scripts.

## Monitoring and Alerts

### Health Checks

All services include health checks:

```bash
# Application
curl http://localhost:8000/health

# Database
pg_isready -U quoteuser

# SigNoz
curl http://localhost:8080/api/v1/health
```

### Alerts (Recommended)

Configure alerts in SigNoz or external monitoring:

- HTTP 5xx errors > 1% of requests
- Response time p95 > 1 second
- Database connections > 80% of pool
- Disk usage > 80%
- Failed authentication attempts > 10/minute

## Scaling

### Vertical Scaling

Increase resources for single instance:

**Docker Compose**:
```yaml
app:
  deploy:
    resources:
      limits:
        cpus: '4'    # Increase
        memory: 2G   # Increase
```

**VPS**:
- Resize VPS to larger instance
- Adjust PostgreSQL shared_buffers (25% of RAM)
- Increase systemd resource limits

### Horizontal Scaling (Future)

For high-traffic scenarios:
- Load balancer (nginx, HAProxy)
- Multiple app instances (stateless)
- Shared PostgreSQL (managed service)
- Shared upload storage (S3, NFS)
- Redis for session storage (replace file-based)

**Status**: Not yet implemented, application is currently stateful (file-based sessions)

## Cost Estimation

### Docker Compose VPS

**2 CPU / 4GB RAM** (without observability):
- DigitalOcean: ~$24/month
- Hetzner: ~€8/month
- Linode: ~$18/month

**4 CPU / 8GB RAM** (with observability):
- DigitalOcean: ~$48/month
- Hetzner: ~€16/month
- Linode: ~$36/month

### CleverCloud (PaaS)

- App (S instance): €10/month
- PostgreSQL (S): €7.50/month
- **Total**: ~€17.50/month

*Prices approximate, check vendor websites*

## Support

### Documentation

- **Configuration**: [configuration.md](./configuration.md)
- **Docker Compose**: [docker-compose.md](./docker-compose.md)
- **VPS**: [vps.md](./vps.md)
- **Security**: [security.md](./security.md)
- **Troubleshooting**: [troubleshooting.md](./troubleshooting.md)

### Community

- **GitHub Issues**: https://github.com/your-org/3d-assistant/issues
- **Discussions**: https://github.com/your-org/3d-assistant/discussions

### Commercial Support

For commercial support, consulting, or custom deployments:
- Email: support@example.com

### Security Issues

Report security vulnerabilities privately:
- Email: security@example.com
- See: SECURITY.md (TODO)

## Next Steps

1. **Choose deployment platform**: Docker Compose, VPS, or CleverCloud
2. **Follow deployment guide**: [docker-compose.md](./docker-compose.md) or [vps.md](./vps.md)
3. **Complete security checklist**: [security.md](./security.md)
4. **Set up monitoring**: Configure alerts in SigNoz
5. **Test disaster recovery**: Restore from backup
6. **Document your deployment**: Keep deployment notes for your team

## Contributing

Found an issue with deployment docs? Want to add a new platform guide?

- Open an issue: https://github.com/your-org/3d-assistant/issues
- Submit a PR: https://github.com/your-org/3d-assistant/pulls

We especially welcome:
- Platform-specific guides (AWS, GCP, Azure, etc.)
- Kubernetes/Helm charts
- Terraform/Ansible automation
- Performance tuning guides
