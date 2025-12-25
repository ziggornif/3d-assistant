# Production Security Checklist

Complete security checklist for deploying the 3D Quote Service in production.

## Authentication & Authorization

### Tokens and Secrets

- [ ] **ADMIN_TOKEN changed from default**
  - Generate: `openssl rand -base64 32`
  - Minimum 32 characters
  - Never commit to version control

- [ ] **MCP_TOKEN changed from default**
  - Generate: `openssl rand -base64 32`
  - Different from ADMIN_TOKEN
  - Rotate quarterly

- [ ] **Database password is strong**
  - Minimum 16 characters
  - Mix of letters, numbers, symbols
  - Unique to this deployment

- [ ] **Environment variables secured**
  - .env file has `chmod 600` permissions
  - .env is in .gitignore
  - No secrets in git history

### Access Control

- [ ] **Admin endpoints protected**
  - `/admin/*` requires ADMIN_TOKEN
  - Token validation on every request
  - No bypass mechanisms

- [ ] **MCP endpoints protected**
  - `/mcp/*` requires MCP_TOKEN
  - Separate from admin authentication
  - Rate limiting enabled

- [ ] **Session management**
  - Sessions expire after 24 hours (configurable)
  - Session cleanup runs regularly
  - Old session files deleted

## Network Security

### HTTPS/TLS

- [ ] **HTTPS enabled in production**
  - Valid SSL/TLS certificate
  - HTTP redirects to HTTPS
  - HSTS header enabled

- [ ] **TLS configuration hardened**
  - TLS 1.2+ only (disable TLS 1.0/1.1)
  - Strong cipher suites
  - Perfect forward secrecy

- [ ] **Certificate auto-renewal configured**
  - Let's Encrypt certbot timer active
  - Renewal notifications enabled
  - Backup certificates stored securely

### Firewall

- [ ] **Firewall enabled and configured**
  - Only necessary ports open (80, 443, 22)
  - PostgreSQL not exposed publicly (5432 blocked)
  - OTEL Collector not exposed (4317 blocked)

- [ ] **SSH hardened**
  - Key-based authentication only
  - Root login disabled
  - fail2ban configured
  - Non-default SSH port (optional)

### Reverse Proxy

- [ ] **Nginx security headers configured**
  ```nginx
  add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
  add_header X-Content-Type-Options "nosniff" always;
  add_header X-Frame-Options "DENY" always;
  add_header X-XSS-Protection "1; mode=block" always;
  add_header Referrer-Policy "strict-origin-when-cross-origin" always;
  ```

- [ ] **Request rate limiting enabled**
  ```nginx
  limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
  limit_req zone=api burst=20 nodelay;
  ```

- [ ] **File upload size limited**
  ```nginx
  client_max_body_size 50M;
  ```

## Application Security

### Input Validation

- [ ] **File upload validation**
  - File type whitelist (STL, OBJ, 3MF only)
  - File size limit enforced (50MB default)
  - Filename sanitization
  - MIME type verification

- [ ] **SQL injection prevention**
  - Using prepared statements (sqlx)
  - No dynamic SQL construction
  - Input validation on all parameters

- [ ] **Path traversal prevention**
  - Upload directory isolated
  - Filename validation (no ../)
  - Session ID validation

### Data Protection

- [ ] **Database security**
  - PostgreSQL user has minimal privileges
  - Database not exposed publicly
  - Connection uses password authentication
  - Regular backups enabled

- [ ] **File storage security**
  - Uploads directory has restricted permissions
  - Files served with correct MIME types
  - No directory listing enabled
  - Old files cleaned up regularly

- [ ] **Sensitive data handling**
  - Secrets not logged (sanitize_secret() used)
  - No sensitive data in error messages
  - Database credentials not in logs

## Dependency Security

### Rust Dependencies

- [ ] **Dependencies up to date**
  ```bash
  cargo update
  cargo audit
  ```

- [ ] **Security advisories reviewed**
  - Check RustSec advisory database
  - Subscribe to security mailing lists
  - Regular dependency updates

- [ ] **Minimal dependency surface**
  - Only necessary crates included
  - Features flags used to minimize code
  - Transitive dependencies reviewed

### System Packages

- [ ] **OS packages updated**
  ```bash
  sudo apt update && sudo apt upgrade
  ```

- [ ] **Unattended upgrades configured**
  ```bash
  sudo apt install unattended-upgrades
  sudo dpkg-reconfigure -plow unattended-upgrades
  ```

- [ ] **Security patches auto-applied**
  - Critical security updates automatic
  - Restart policy configured
  - Update notifications enabled

## Operational Security

### Logging and Monitoring

- [ ] **Logging configured**
  - Structured JSON logs in production
  - No sensitive data in logs
  - Log rotation enabled
  - Logs retained for 30+ days

- [ ] **Monitoring enabled**
  - OpenTelemetry traces collected
  - Metrics exported to observability platform
  - Error rates monitored
  - Alerts configured

- [ ] **Audit trail**
  - Admin actions logged
  - Authentication failures logged
  - File uploads logged with session ID
  - Database changes tracked

### Backup and Recovery

- [ ] **Automated backups configured**
  - Daily database backups
  - Upload directory backups
  - Backups stored off-site
  - Retention policy: 7 days minimum

- [ ] **Backup encryption**
  - Backups encrypted at rest
  - Encryption keys secured separately
  - Access to backups restricted

- [ ] **Recovery tested**
  - Restore procedure documented
  - Recovery tested quarterly
  - RTO/RPO defined
  - Disaster recovery plan exists

### Incident Response

- [ ] **Incident response plan**
  - Contact list maintained
  - Escalation procedure defined
  - Communication templates ready
  - Post-mortem process established

- [ ] **Security contact**
  - Security email published
  - Vulnerability disclosure policy
  - Bug bounty program (optional)

## Compliance

### Data Privacy

- [ ] **GDPR compliance** (if applicable)
  - User data deletion capability
  - Data retention policy
  - Privacy policy published
  - Data processing documented

- [ ] **User data protection**
  - User consent for data storage
  - Session data auto-deleted
  - No unnecessary data collected
  - Data minimization principle

### Legal

- [ ] **Terms of service**
  - Terms clearly displayed
  - User acceptance tracked
  - Regular legal review

- [ ] **License compliance**
  - Open source licenses reviewed
  - Attribution requirements met
  - No GPL contamination (if proprietary)

## Deployment Security

### Docker (if applicable)

- [ ] **Base images verified**
  - Official images used
  - Image signatures verified
  - Regular image updates

- [ ] **Container security**
  - Non-root user in containers
  - Read-only root filesystem
  - Capabilities dropped
  - Resource limits enforced

- [ ] **Secrets management**
  - Docker secrets used (not environment variables)
  - No secrets in images
  - Secret rotation possible

### CI/CD

- [ ] **Pipeline security**
  - CI/CD secrets secured
  - Build artifacts signed
  - Deployment requires approval
  - Audit trail of deployments

- [ ] **Code security**
  - Dependency scanning in CI
  - SAST tools configured
  - Code review required
  - Branch protection enabled

## Testing

### Security Testing

- [ ] **Penetration testing**
  - Annual pentest scheduled
  - Findings remediated
  - Re-test after fixes

- [ ] **Vulnerability scanning**
  - Automated scanning enabled
  - Weekly vulnerability scans
  - Critical findings prioritized

- [ ] **Security code review**
  - Authentication code reviewed
  - File handling code reviewed
  - SQL queries reviewed
  - Third-party integrations reviewed

## Post-Deployment

### Ongoing Security

- [ ] **Security updates schedule**
  - Monthly dependency updates
  - Quarterly security review
  - Annual security audit

- [ ] **Monitoring and alerts**
  - Failed authentication alerts
  - Unusual traffic alerts
  - Error rate alerts
  - Resource usage alerts

- [ ] **Regular reviews**
  - Access control review (quarterly)
  - Firewall rules review (quarterly)
  - Backup verification (monthly)
  - Log analysis (weekly)

## Security Tools

### Recommended Tools

**Scanning**:
- `cargo audit` - Rust dependency vulnerabilities
- `trivy` - Container image scanning
- `nmap` - Network scanning
- `nikto` - Web server scanning

**Monitoring**:
- `fail2ban` - SSH brute force protection
- `OSSEC` - Host-based intrusion detection
- `Wazuh` - Security monitoring platform

**Hardening**:
- `lynis` - Security auditing tool
- `CIS benchmarks` - Configuration standards

### Security Audit Commands

```bash
# Check for known vulnerabilities in Rust dependencies
cargo audit

# Check for outdated dependencies
cargo outdated

# Scan Docker image
trivy image 3d-quote-service:latest

# System security audit
sudo lynis audit system

# Check SSL/TLS configuration
testssl.sh quote.example.com

# Check HTTP headers
curl -I https://quote.example.com | grep -i security

# Verify firewall
sudo ufw status verbose

# Check open ports
sudo netstat -tulpn
```

## Quick Security Validation

Run these commands to verify security posture:

```bash
# 1. Check environment variables are not default
grep "changeme\|admin-secret\|mcp-token-placeholder" .env && echo "❌ Change default secrets!" || echo "✅ Secrets changed"

# 2. Check .env permissions
[ "$(stat -c %a .env)" = "600" ] && echo "✅ .env secured" || echo "❌ Run: chmod 600 .env"

# 3. Check HTTPS redirect
curl -I http://quote.example.com | grep -i "location: https" && echo "✅ HTTPS redirect" || echo "❌ Configure HTTPS redirect"

# 4. Check security headers
curl -I https://quote.example.com | grep -i "strict-transport-security" && echo "✅ HSTS enabled" || echo "❌ Add HSTS header"

# 5. Check firewall
sudo ufw status | grep -E "80/tcp|443/tcp" && sudo ufw status | grep -E "5432|4317" | grep -v "ALLOW" && echo "✅ Firewall configured" || echo "❌ Review firewall rules"
```

## Emergency Procedures

### Security Incident Response

**Suspected breach**:
1. Isolate affected system (disconnect network)
2. Preserve evidence (disk snapshots, logs)
3. Notify stakeholders
4. Investigate (analyze logs, check for indicators of compromise)
5. Remediate (patch vulnerabilities, rotate secrets)
6. Restore service (from clean backup if needed)
7. Post-mortem (document incident, improve defenses)

**Token compromise**:
1. Immediately rotate compromised token
2. Review logs for unauthorized access
3. Notify affected users
4. Investigate how token was compromised
5. Implement additional controls

**Data breach**:
1. Assess scope (what data, how many users)
2. Contain breach (stop data exfiltration)
3. Notify authorities (GDPR: within 72 hours)
4. Notify affected users
5. Provide remediation (password reset, credit monitoring)
6. Document incident

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CIS Benchmarks](https://www.cisecurity.org/cis-benchmarks)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)

## Support

Security issues should be reported to: **security@example.com**

For vulnerability disclosure, see: **SECURITY.md** (TODO: create)
