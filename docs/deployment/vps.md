# VPS/VM Deployment Guide

This guide covers deploying the 3D Quote Service on a Virtual Private Server (VPS) or Virtual Machine using systemd and nginx.

**Target platforms**: Ubuntu 22.04 LTS, Debian 12, or similar Linux distributions.

## Prerequisites

- Ubuntu 22.04 LTS (or Debian 12+)
- 2 CPU cores minimum (4 recommended)
- 2GB RAM minimum (4GB recommended)
- 20GB disk space
- Root or sudo access
- Domain name (optional, for HTTPS)

## Installation Steps

### 1. Initial Server Setup

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install required packages
sudo apt install -y \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    postgresql \
    postgresql-contrib \
    nginx \
    certbot \
    python3-certbot-nginx

# Create application user
sudo useradd -m -s /bin/bash quoteapp
sudo usermod -aG sudo quoteapp  # Optional: if app user needs sudo
```

### 2. Install Rust

```bash
# Switch to app user
sudo su - quoteapp

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
```

### 3. Configure PostgreSQL

```bash
# Switch to postgres user
sudo su - postgres

# Create database and user
psql <<EOF
CREATE USER quoteuser WITH PASSWORD 'your_secure_password_here';
CREATE DATABASE quotes OWNER quoteuser;
GRANT ALL PRIVILEGES ON DATABASE quotes TO quoteuser;
\q
EOF

# Exit postgres user
exit

# Configure PostgreSQL to allow password authentication
sudo nano /etc/postgresql/15/main/pg_hba.conf
```

Add or modify this line:

```
# IPv4 local connections:
host    quotes    quoteuser    127.0.0.1/32    md5
```

Restart PostgreSQL:

```bash
sudo systemctl restart postgresql
sudo systemctl enable postgresql
```

### 4. Deploy Application

```bash
# Switch to app user
sudo su - quoteapp

# Clone repository
cd /home/quoteapp
git clone https://github.com/your-org/3d-assistant.git
cd 3d-assistant

# Build release binary
cargo build --release

# Create necessary directories
mkdir -p /home/quoteapp/3d-assistant/uploads
mkdir -p /home/quoteapp/3d-assistant/logs

# Create .env file
cp .env.example .env
nano .env
```

**Configure .env**:

```bash
# Database
DATABASE_URL=postgres://quoteuser:your_secure_password_here@localhost:5432/quotes

# Server
HOST=127.0.0.1
PORT=8000
ENVIRONMENT=production

# File Upload
MAX_FILE_SIZE_MB=50
UPLOAD_DIR=/home/quoteapp/3d-assistant/uploads

# Session
SESSION_EXPIRY_HOURS=24

# Security (CHANGE THESE!)
ADMIN_TOKEN=$(openssl rand -base64 32)
MCP_TOKEN=$(openssl rand -base64 32)

# OpenTelemetry (optional, for external collector)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=quote-service

# Directories
STATIC_DIR=/home/quoteapp/3d-assistant/static
TEMPLATE_DIR=/home/quoteapp/3d-assistant/templates
```

Secure the .env file:

```bash
chmod 600 .env
```

### 5. Create Systemd Service

```bash
# Exit app user
exit

# Create systemd service file
sudo nano /etc/systemd/system/quote-service.service
```

**Service configuration**:

```ini
[Unit]
Description=3D Quote Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=quoteapp
Group=quoteapp
WorkingDirectory=/home/quoteapp/3d-assistant
EnvironmentFile=/home/quoteapp/3d-assistant/.env
ExecStart=/home/quoteapp/3d-assistant/target/release/quote-service
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/home/quoteapp/3d-assistant/uploads /home/quoteapp/3d-assistant/logs

# Resource limits
LimitNOFILE=65536
MemoryMax=1G
CPUQuota=200%

# Logging
StandardOutput=append:/home/quoteapp/3d-assistant/logs/stdout.log
StandardError=append:/home/quoteapp/3d-assistant/logs/stderr.log

[Install]
WantedBy=multi-user.target
```

Enable and start service:

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable quote-service

# Start service
sudo systemctl start quote-service

# Check status
sudo systemctl status quote-service

# View logs
sudo journalctl -u quote-service -f
```

### 6. Configure Nginx Reverse Proxy

```bash
# Create nginx configuration
sudo nano /etc/nginx/sites-available/quote-service
```

**HTTP configuration** (for testing):

```nginx
server {
    listen 80;
    server_name quote.example.com;  # Replace with your domain

    # Security headers
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Logging
    access_log /var/log/nginx/quote-service-access.log;
    error_log /var/log/nginx/quote-service-error.log;

    # Proxy to application
    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # File upload size limit
    client_max_body_size 50M;
}
```

Enable site:

```bash
# Create symlink
sudo ln -s /etc/nginx/sites-available/quote-service /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

### 7. Configure HTTPS with Let's Encrypt

```bash
# Obtain SSL certificate
sudo certbot --nginx -d quote.example.com

# Certbot will automatically update nginx config for HTTPS
# Certificate auto-renewal is configured via systemd timer
```

**Verify HTTPS configuration**:

```nginx
# /etc/nginx/sites-available/quote-service (after certbot)
server {
    listen 80;
    server_name quote.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name quote.example.com;

    # SSL certificates (managed by Certbot)
    ssl_certificate /etc/letsencrypt/live/quote.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/quote.example.com/privkey.pem;
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Rest of configuration...
    location / {
        proxy_pass http://127.0.0.1:8000;
        # ... (same as HTTP config)
    }

    client_max_body_size 50M;
}
```

### 8. Configure Firewall

```bash
# Install ufw
sudo apt install ufw

# Allow SSH (IMPORTANT: before enabling firewall!)
sudo ufw allow 22/tcp

# Allow HTTP and HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status
```

## Monitoring Setup (Optional)

### Install OpenTelemetry Collector

```bash
# Download OTEL Collector
wget https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.88.0/otelcol_0.88.0_linux_amd64.tar.gz

# Extract
tar xzf otelcol_0.88.0_linux_amd64.tar.gz
sudo mv otelcol /usr/local/bin/

# Create config directory
sudo mkdir -p /etc/otelcol

# Create config file
sudo nano /etc/otelcol/config.yaml
```

**OTEL Collector config** (export to external SigNoz/Grafana):

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: localhost:4317

exporters:
  otlp:
    endpoint: https://your-signoz-instance.com:4317
    headers:
      signoz-access-token: "your-token-here"

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [otlp]
    metrics:
      receivers: [otlp]
      exporters: [otlp]
```

**Create systemd service**:

```bash
sudo nano /etc/systemd/system/otelcol.service
```

```ini
[Unit]
Description=OpenTelemetry Collector
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/otelcol --config=/etc/otelcol/config.yaml
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable otelcol
sudo systemctl start otelcol
```

## Backup and Restore

### Automated Backups

Create backup script:

```bash
sudo nano /usr/local/bin/quote-backup.sh
```

```bash
#!/bin/bash
set -e

BACKUP_DIR="/backups/quote-service"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup database
sudo -u postgres pg_dump quotes | gzip > "$BACKUP_DIR/db_$DATE.sql.gz"

# Backup uploads
tar czf "$BACKUP_DIR/uploads_$DATE.tar.gz" /home/quoteapp/3d-assistant/uploads/

# Keep only last 7 days
find "$BACKUP_DIR" -name "db_*.sql.gz" -mtime +7 -delete
find "$BACKUP_DIR" -name "uploads_*.tar.gz" -mtime +7 -delete

echo "Backup completed: $DATE"
```

```bash
sudo chmod +x /usr/local/bin/quote-backup.sh

# Add to crontab
sudo crontab -e
```

```
# Backup daily at 2 AM
0 2 * * * /usr/local/bin/quote-backup.sh >> /var/log/quote-backup.log 2>&1
```

### Restore from Backup

```bash
# Stop service
sudo systemctl stop quote-service

# Restore database
gunzip -c /backups/quote-service/db_20240115_020000.sql.gz | sudo -u postgres psql quotes

# Restore uploads
sudo tar xzf /backups/quote-service/uploads_20240115_020000.tar.gz -C /

# Fix permissions
sudo chown -R quoteapp:quoteapp /home/quoteapp/3d-assistant/uploads

# Start service
sudo systemctl start quote-service
```

## Maintenance

### Update Application

```bash
# Switch to app user
sudo su - quoteapp
cd /home/quoteapp/3d-assistant

# Pull latest code
git pull origin main

# Rebuild
cargo build --release

# Exit app user
exit

# Restart service
sudo systemctl restart quote-service

# Check logs
sudo journalctl -u quote-service -f
```

### Log Rotation

Configure logrotate:

```bash
sudo nano /etc/logrotate.d/quote-service
```

```
/home/quoteapp/3d-assistant/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 quoteapp quoteapp
    postrotate
        systemctl reload quote-service > /dev/null 2>&1 || true
    endscript
}
```

## Troubleshooting

### Service won't start

```bash
# Check service status
sudo systemctl status quote-service

# View logs
sudo journalctl -u quote-service -n 50

# Check if port is in use
sudo netstat -tulpn | grep 8000

# Test configuration
sudo -u quoteapp /home/quoteapp/3d-assistant/target/release/quote-service --help
```

### Database connection errors

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Test connection
sudo -u postgres psql -U quoteuser -d quotes -h localhost
```

### High resource usage

```bash
# Monitor resources
htop

# Check disk usage
df -h
du -sh /home/quoteapp/3d-assistant/*

# Check service limits
systemctl show quote-service | grep -i memory
```

## Security Hardening

- [ ] Changed default passwords and tokens
- [ ] Configured firewall (ufw)
- [ ] Enabled HTTPS with valid certificate
- [ ] Restricted SSH access (key-based only)
- [ ] Configured fail2ban for SSH protection
- [ ] Regular security updates (unattended-upgrades)
- [ ] File permissions locked down (chmod 600 .env)
- [ ] Disabled root login
- [ ] Configured automated backups
- [ ] Monitoring enabled

## Performance Tuning

### PostgreSQL

```bash
sudo nano /etc/postgresql/15/main/postgresql.conf
```

```ini
# Increase shared buffers (25% of RAM)
shared_buffers = 512MB

# Increase work memory
work_mem = 16MB

# Increase maintenance work memory
maintenance_work_mem = 128MB

# Effective cache size (50-75% of RAM)
effective_cache_size = 1536MB
```

```bash
sudo systemctl restart postgresql
```

### System

```bash
# Increase file descriptor limit
sudo nano /etc/security/limits.conf
```

```
quoteapp soft nofile 65536
quoteapp hard nofile 65536
```

## Next Steps

- Review [security.md](./security.md) checklist
- Configure monitoring and alerts
- Set up CI/CD pipeline
- Read [troubleshooting.md](./troubleshooting.md)
