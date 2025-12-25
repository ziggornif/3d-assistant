# Troubleshooting Guide

Common issues and solutions for the 3D Quote Service.

## Application Issues

### Service won't start

**Symptoms**: Application crashes immediately or won't start

**Diagnosis**:
```bash
# Check service status
sudo systemctl status quote-service

# View recent logs
sudo journalctl -u quote-service -n 100 --no-pager

# Check if port is already in use
sudo netstat -tulpn | grep 8000
# or
sudo lsof -i :8000

# Test binary directly
/path/to/quote-service
```

**Common causes**:

1. **Missing environment variables**
   ```
   Error: environment variable not found: DATABASE_URL
   ```
   **Solution**: Check `.env` file exists and contains all required variables

2. **Port already in use**
   ```
   Error: Address already in use (os error 98)
   ```
   **Solution**:
   ```bash
   # Find process using port 8000
   sudo lsof -i :8000
   # Kill the process or change PORT in .env
   ```

3. **Permission denied**
   ```
   Error: Permission denied (os error 13)
   ```
   **Solution**: Check file permissions and ownership
   ```bash
   sudo chown -R quoteapp:quoteapp /app
   chmod 755 /app/quote-service
   ```

### Database connection errors

**Symptoms**: Can't connect to PostgreSQL

**Diagnosis**:
```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Test connection manually
psql $DATABASE_URL
# or
psql -h localhost -U quoteuser -d quotes

# Check PostgreSQL logs
sudo journalctl -u postgresql -n 50
```

**Common causes**:

1. **Wrong connection string**
   ```
   Error: password authentication failed for user "quoteuser"
   ```
   **Solution**: Verify `DATABASE_URL` in `.env`
   ```bash
   # Format: postgres://username:password@host:port/database
   DATABASE_URL=postgres://quoteuser:correct_password@localhost:5432/quotes
   ```

2. **PostgreSQL not accepting connections**
   ```
   Error: could not connect to server: Connection refused
   ```
   **Solution**: Check `pg_hba.conf` and `postgresql.conf`
   ```bash
   sudo nano /etc/postgresql/15/main/pg_hba.conf
   # Add: host quotes quoteuser 127.0.0.1/32 md5

   sudo systemctl restart postgresql
   ```

3. **Database doesn't exist**
   ```
   Error: database "quotes" does not exist
   ```
   **Solution**: Create database
   ```bash
   sudo -u postgres psql
   CREATE DATABASE quotes OWNER quoteuser;
   \q
   ```

### Migrations fail

**Symptoms**: Application starts but migrations error

**Diagnosis**:
```bash
# Check logs for migration errors
sudo journalctl -u quote-service | grep -i migration

# Manually test migrations
psql $DATABASE_URL -f src/db/migrations/001_initial.sql
```

**Solution**:
```bash
# Reset database (WARNING: deletes all data)
sudo -u postgres psql
DROP DATABASE quotes;
CREATE DATABASE quotes OWNER quoteuser;
\q

# Restart application (migrations run automatically)
sudo systemctl restart quote-service
```

### File upload failures

**Symptoms**: File uploads return 500 error

**Diagnosis**:
```bash
# Check upload directory exists and is writable
ls -ld /app/uploads
# Should be: drwxr-xr-x quoteapp quoteapp

# Check disk space
df -h /app/uploads

# Check logs for file upload errors
sudo journalctl -u quote-service | grep -i upload
```

**Common causes**:

1. **Directory doesn't exist**
   **Solution**:
   ```bash
   mkdir -p /app/uploads
   sudo chown quoteapp:quoteapp /app/uploads
   ```

2. **Permission denied**
   **Solution**:
   ```bash
   sudo chown -R quoteapp:quoteapp /app/uploads
   chmod 755 /app/uploads
   ```

3. **Disk full**
   **Solution**: Free up disk space or increase volume size
   ```bash
   # Find large files
   du -sh /app/uploads/*
   # Clean up old sessions (run cleanup endpoint)
   curl -X POST http://localhost:8000/admin/cleanup \
     -H "Authorization: Bearer $ADMIN_TOKEN"
   ```

## Observability Issues

### No traces in SigNoz

**Symptoms**: SigNoz shows no data despite application running

**Diagnosis**:
```bash
# Check OTEL Collector is running
docker ps | grep otel-collector

# Check OTEL Collector logs
docker logs signoz-otel-collector

# Verify OTLP endpoint
echo $OTEL_EXPORTER_OTLP_ENDPOINT
# Should be: http://otel-collector:4317 (Docker) or http://localhost:4317 (local)

# Test connectivity from app container
docker exec quote-service-app nc -zv otel-collector 4317
```

**Common causes**:

1. **Wrong OTLP endpoint**
   **Solution**: Update `.env` or environment variables
   ```bash
   # Docker Compose:
   OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317

   # Local development:
   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
   ```

2. **OTEL Collector not running**
   **Solution**:
   ```bash
   docker-compose -f deployment/docker-compose.prod.yml up -d otel-collector
   ```

3. **Network not connected**
   **Solution**: Ensure app container is on signoz network
   ```bash
   docker inspect quote-service-app | grep -A 10 Networks
   # Should show "signoz-network"
   ```

### ClickHouse high memory usage

**Symptoms**: System running out of memory

**Diagnosis**:
```bash
# Check memory usage
docker stats clickhouse

# Check ClickHouse logs
docker logs signoz-clickhouse | tail -50
```

**Solution**:

1. **Reduce memory limit**
   Edit `docker-compose.prod.yml`:
   ```yaml
   clickhouse:
     deploy:
       resources:
         limits:
           memory: 1G  # Reduce from 2G
   ```

2. **Configure data retention**
   - Access SigNoz UI → Settings → Data Retention
   - Reduce from 7 days to 3 days

3. **Run without observability stack**
   ```bash
   docker-compose -f deployment/docker-compose.prod.yml stop clickhouse query-service frontend otel-collector
   ```

### Metrics not showing up

**Symptoms**: Metrics are zero or missing in SigNoz

**Diagnosis**:
```bash
# Check application logs for metric errors
docker logs quote-service-app | grep -i metric

# Verify metrics are being recorded
# Add temporary log in code or check OTEL debug logs
```

**Solution**:
- Metrics export to OTLP is currently TODO (in-memory only)
- Metrics are collected but not exported
- Update `src/observability/metrics.rs` to add OTLP exporter once API stabilizes

## Deployment Issues

### Docker build fails

**Symptoms**: Docker build errors

**Diagnosis**:
```bash
# Build with full output
docker build -t 3d-quote-service:latest . --progress=plain
```

**Common causes**:

1. **Cargo dependency resolution failure**
   **Solution**: Clear cache and rebuild
   ```bash
   docker build --no-cache -t 3d-quote-service:latest .
   ```

2. **Out of disk space**
   **Solution**: Clean Docker system
   ```bash
   docker system prune -a --volumes
   ```

### Container crashes in production

**Symptoms**: Container restarts repeatedly

**Diagnosis**:
```bash
# Check container status
docker-compose -f deployment/docker-compose.prod.yml ps

# Check container logs
docker logs quote-service-app --tail 200

# Check resource usage
docker stats quote-service-app
```

**Common causes**:

1. **Out of memory**
   **Solution**: Increase memory limit
   ```yaml
   app:
     deploy:
       resources:
         limits:
           memory: 2G  # Increase from 1G
   ```

2. **Health check failing**
   **Solution**: Check health endpoint
   ```bash
   docker exec quote-service-app curl -f http://localhost:8000/health
   ```

## Network Issues

### Can't access application from outside

**Symptoms**: Application works on localhost but not from external IP

**Diagnosis**:
```bash
# Check application is listening on 0.0.0.0
sudo netstat -tulpn | grep 8000
# Should show: 0.0.0.0:8000 (not 127.0.0.1:8000)

# Check firewall
sudo ufw status

# Check nginx is proxying correctly
curl -I http://localhost:8000  # Direct to app
curl -I http://localhost        # Through nginx
```

**Solution**:

1. **Application binding to localhost only**
   Update `.env`:
   ```bash
   HOST=0.0.0.0  # Not 127.0.0.1
   ```

2. **Firewall blocking**
   ```bash
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   ```

3. **Nginx not running**
   ```bash
   sudo systemctl start nginx
   sudo nginx -t  # Test config
   ```

### HTTPS not working

**Symptoms**: SSL/TLS errors or certificate warnings

**Diagnosis**:
```bash
# Check certificate
openssl s_client -connect quote.example.com:443 -servername quote.example.com

# Check certbot status
sudo certbot certificates

# Check nginx SSL configuration
sudo nginx -t
```

**Solution**:

1. **Certificate expired**
   ```bash
   sudo certbot renew
   sudo systemctl reload nginx
   ```

2. **Certificate not found**
   ```bash
   # Obtain certificate
   sudo certbot --nginx -d quote.example.com
   ```

3. **Wrong domain in certificate**
   - Certificate must match exactly: `quote.example.com`
   - Wildcard certificates: `*.example.com`

## Performance Issues

### Slow response times

**Symptoms**: Requests take > 1 second

**Diagnosis**:
```bash
# Check response time
time curl http://localhost:8000/api/materials

# Check system resources
htop

# Check database performance
psql $DATABASE_URL
EXPLAIN ANALYZE SELECT * FROM quotes;
```

**Common causes**:

1. **Database slow queries**
   **Solution**: Add indexes
   ```sql
   CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
   CREATE INDEX idx_quotes_created_at ON quotes(created_at);
   ```

2. **High CPU usage**
   **Solution**: Increase CPU limit or scale horizontally
   ```yaml
   app:
     deploy:
       resources:
         limits:
           cpus: '4'  # Increase from 2
   ```

3. **Many expired sessions**
   **Solution**: Run cleanup
   ```bash
   curl -X POST http://localhost:8000/admin/cleanup \
     -H "Authorization: Bearer $ADMIN_TOKEN"
   ```

### High memory usage

**Diagnosis**:
```bash
# Monitor memory
docker stats

# Check for memory leaks
# Review application logs for unusual patterns
```

**Solution**:
- Restart application (temporary)
- Increase memory limit (if needed)
- Review code for memory leaks (check for large allocations)

## Backup and Restore Issues

### Backup fails

**Diagnosis**:
```bash
# Check backup script logs
tail -50 /var/log/quote-backup.log

# Test backup manually
sudo /usr/local/bin/quote-backup.sh
```

**Common causes**:

1. **Out of disk space**
   ```bash
   df -h /backups
   # Clean old backups if needed
   ```

2. **PostgreSQL not accessible**
   ```bash
   sudo -u postgres pg_dump quotes > /tmp/test.sql
   ```

### Restore fails

**Diagnosis**:
```bash
# Check backup file integrity
gunzip -t backup.sql.gz

# Test restore to temporary database
createdb test_restore
gunzip -c backup.sql.gz | psql test_restore
```

**Solution**:
```bash
# Full restore procedure
sudo systemctl stop quote-service
sudo -u postgres dropdb quotes
sudo -u postgres createdb quotes -O quoteuser
gunzip -c backup.sql.gz | sudo -u postgres psql quotes
sudo systemctl start quote-service
```

## Monitoring and Debugging

### Enable debug logging

Temporarily enable debug logging:

```bash
# Set environment variable
export RUST_LOG=debug

# Or in .env file
RUST_LOG=debug,tower_http=debug,sqlx=debug

# Restart application
sudo systemctl restart quote-service
```

### Capture request/response

```bash
# Use tcpdump to capture HTTP traffic
sudo tcpdump -i any -A -s 0 'tcp port 8000 and (((ip[2:2] - ((ip[0]&0xf)<<2)) - ((tcp[12]&0xf0)>>2)) != 0)' -w capture.pcap

# Or use nginx access logs
tail -f /var/log/nginx/quote-service-access.log
```

### Profile performance

```bash
# Use cargo flamegraph (development only)
cargo install flamegraph
cargo flamegraph

# Or use perf (Linux)
perf record -F 99 -p $(pgrep quote-service) -g -- sleep 30
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg
```

## Getting Help

### Information to include in bug reports

```bash
# System information
uname -a
cat /etc/os-release

# Application version
/path/to/quote-service --version

# Service status
sudo systemctl status quote-service

# Recent logs (last 100 lines)
sudo journalctl -u quote-service -n 100 --no-pager

# Configuration (sanitized - remove secrets!)
cat .env | sed 's/PASSWORD=.*/PASSWORD=REDACTED/' | sed 's/TOKEN=.*/TOKEN=REDACTED/'

# Docker info (if using Docker)
docker --version
docker-compose --version
docker-compose -f deployment/docker-compose.prod.yml ps
```

### Where to get help

- **Documentation**: [docs/deployment/](./README.md)
- **Issues**: https://github.com/your-org/3d-assistant/issues
- **Discussions**: https://github.com/your-org/3d-assistant/discussions
- **Security**: security@example.com

## Quick Diagnostic Script

Save this as `diagnose.sh`:

```bash
#!/bin/bash
echo "=== 3D Quote Service Diagnostics ==="
echo ""

echo "1. Service Status:"
systemctl status quote-service --no-pager | head -10

echo ""
echo "2. Port Check:"
netstat -tulpn | grep 8000 || echo "Port 8000 not listening"

echo ""
echo "3. Database Check:"
sudo -u postgres psql -c "SELECT version();" quotes 2>&1 | head -1

echo ""
echo "4. Disk Space:"
df -h | grep -E "Filesystem|/app|/home"

echo ""
echo "5. Memory Usage:"
free -h

echo ""
echo "6. Recent Errors:"
journalctl -u quote-service --since "5 minutes ago" | grep -i error | tail -5

echo ""
echo "7. Configuration Check:"
[ -f .env ] && echo "✅ .env exists" || echo "❌ .env missing"
[ "$(stat -c %a .env 2>/dev/null)" = "600" ] && echo "✅ .env permissions OK" || echo "⚠️  .env permissions should be 600"

echo ""
echo "8. Network Check:"
curl -s -o /dev/null -w "HTTP %{http_code}" http://localhost:8000/health
echo ""
```

Run with:
```bash
chmod +x diagnose.sh
./diagnose.sh
```
