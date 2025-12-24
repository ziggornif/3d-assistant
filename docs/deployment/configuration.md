# Configuration Guide

## Overview

The 3D Print Quote Service is configured entirely through environment variables, following the [12-factor app](https://12factor.net/config) methodology. This allows the same codebase to run in different environments (development, staging, production) with different configurations.

## Environment Variables Reference

### Required Variables

These variables **MUST** be set in production environments:

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `DATABASE_URL` | String | PostgreSQL connection string | `postgres://user:pass@host:5432/dbname` |
| `ADMIN_TOKEN` | String | Admin interface authentication token | `secure-random-token-32-chars` |
| `MCP_TOKEN` | String | MCP API authentication token | `mcp-auth-token-32-chars` |

**Security Notes**:
- `ADMIN_TOKEN` and `MCP_TOKEN` should be generated using a cryptographically secure random generator
- Generate tokens with: `openssl rand -base64 32`
- Never commit these values to version control
- Rotate tokens regularly (recommended: every 90 days)

### Optional Variables (with defaults)

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `ENVIRONMENT` | String | `development` | Environment mode: `development` or `production` |
| `HOST` | String | `127.0.0.1` | Server bind address |
| `PORT` | Integer | `3000` | Server listen port |
| `MAX_FILE_SIZE_MB` | Integer | `50` | Maximum file upload size in megabytes |
| `UPLOAD_DIR` | String | `./uploads` | Directory for uploaded files |
| `STATIC_DIR` | String | `./static` | Static assets directory |
| `TEMPLATE_DIR` | String | `./templates` | HTML templates directory |
| `SESSION_EXPIRY_HOURS` | Integer | `24` | Quote session expiry time in hours |

### OpenTelemetry Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | String | `http://localhost:4317` | OTLP collector endpoint for traces/metrics/logs |
| `OTEL_SERVICE_NAME` | String | `quote-service` | Service name in telemetry data |

## Platform-Specific Configuration

### Local Development

Create a `.env.local` file in the repository root:

```bash
# Copy from example
cp .env.example .env.local

# Edit with your values
vim .env.local

# The application will auto-load .env.local if it exists
cargo run
```

### Docker Compose

**Option 1: Environment file**

```yaml
# docker-compose.yml
services:
  app:
    image: quote-service:latest
    env_file:
      - .env.production
```

**Option 2: Inline environment**

```yaml
services:
  app:
    image: quote-service:latest
    environment:
      - DATABASE_URL=postgres://user:pass@db:5432/quotes
      - ADMIN_TOKEN=${ADMIN_TOKEN}
      - MCP_TOKEN=${MCP_TOKEN}
      - ENVIRONMENT=production
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
```

### Ubuntu VPS / Systemd

Create `/etc/quote-service/config`:

```bash
# /etc/quote-service/config
DATABASE_URL=postgres://user:pass@localhost:5432/quotes
ADMIN_TOKEN=your-secure-token
MCP_TOKEN=your-mcp-token
ENVIRONMENT=production
HOST=127.0.0.1
PORT=3000
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

Reference in systemd service file:

```ini
# /etc/systemd/system/quote-service.service
[Service]
EnvironmentFile=/etc/quote-service/config
ExecStart=/usr/local/bin/quote-service
```

**File permissions**:
```bash
sudo chmod 600 /etc/quote-service/config
sudo chown quote-service:quote-service /etc/quote-service/config
```

### CleverCloud

Set environment variables via the CleverCloud dashboard:

1. Navigate to your application
2. Go to **Environment variables** section
3. Add each required variable
4. Redeploy the application

Or use the CleverCloud CLI:

```bash
clever env set DATABASE_URL "postgres://..."
clever env set ADMIN_TOKEN "your-token"
clever env set MCP_TOKEN "your-mcp-token"
clever env set ENVIRONMENT "production"
```

### Kubernetes

Create a Secret for sensitive data:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: quote-service-secrets
type: Opaque
stringData:
  database-url: postgres://user:pass@db:5432/quotes
  admin-token: your-secure-token
  mcp-token: your-mcp-token
```

Reference in Deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: quote-service
spec:
  template:
    spec:
      containers:
      - name: app
        image: quote-service:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: quote-service-secrets
              key: database-url
        - name: ADMIN_TOKEN
          valueFrom:
            secretKeyRef:
              name: quote-service-secrets
              key: admin-token
        - name: MCP_TOKEN
          valueFrom:
            secretKeyRef:
              name: quote-service-secrets
              key: mcp-token
        - name: ENVIRONMENT
          value: "production"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://otel-collector:4317"
```

## Environment-Specific Behavior

### Development Mode (`ENVIRONMENT=development`)

- Logs in human-readable pretty format
- Verbose debug logging enabled
- Less strict validation (allows default tokens)
- CORS permissive for local frontend development
- File uploads accept both STL and 3MF formats

### Production Mode (`ENVIRONMENT=production`)

- Logs in structured JSON format
- Production-level logging (INFO and above)
- Strict validation (rejects default secrets)
- Requires secure ADMIN_TOKEN and DATABASE_URL
- CORS configured for specific domains only
- All telemetry data exported to OpenTelemetry collector

## Validation

The service validates configuration at startup:

1. **Fail Fast**: Invalid configuration causes immediate exit with clear error message
2. **Production Checks**: In production mode, validates that secrets are not default values
3. **Type Safety**: Port numbers, file sizes parsed with error handling

Example validation error:

```
ERROR: DATABASE_URL must be set in production.
Please set DATABASE_URL to a valid PostgreSQL connection string.
```

## Best Practices

1. **Use `.env.example` as template**: Never modify `.env.example` with real secrets
2. **Different secrets per environment**: Development, staging, and production should have unique tokens
3. **Rotate secrets regularly**: Update `ADMIN_TOKEN` and `MCP_TOKEN` every 90 days
4. **Restrict file permissions**: Configuration files should be `600` (owner read/write only)
5. **Use secret management**: For production, consider HashiCorp Vault, AWS Secrets Manager, or similar
6. **Never log secrets**: The application automatically sanitizes secrets in logs (uses `[REDACTED]`)

## Troubleshooting

### "Missing required environment variable"

**Symptom**: Application fails to start with `DATABASE_URL must be set`

**Solution**: Ensure all required variables are set in your environment or `.env` file

### "ADMIN_TOKEN must be changed from default value"

**Symptom**: Application refuses to start in production with default token

**Solution**: Generate a secure token:
```bash
openssl rand -base64 32
```
Set it as `ADMIN_TOKEN` environment variable

### Configuration not loading from .env file

**Symptom**: Application uses defaults instead of values from `.env.local`

**Solution**:
- Ensure file is named exactly `.env.local` (or `.env`)
- File must be in the same directory as `Cargo.toml`
- Check file permissions are readable
- Verify no syntax errors in .env file (no quotes needed for values)

### Database connection fails

**Symptom**: `Failed to connect to database`

**Solution**:
- Verify `DATABASE_URL` format: `postgres://user:password@host:port/database`
- Test connection manually: `psql $DATABASE_URL`
- Check firewall rules allow connection
- Ensure PostgreSQL is running

## See Also

- [Deployment Guide](./docker-compose.md) - Full deployment instructions
- [Security Checklist](./security-checklist.md) - Production security requirements
- [Troubleshooting](./troubleshooting.md) - Common deployment issues
