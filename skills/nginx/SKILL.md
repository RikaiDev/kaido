---
name: kaido-nginx
description: |
  Nginx web server troubleshooting and configuration. Use when user mentions:
  - nginx, web server, reverse proxy, load balancer
  - "502 Bad Gateway", "504 Gateway Timeout", "403 Forbidden", "400 Bad Request"
  - nginx config, nginx.conf, site-enabled
  - "nginx: [emerg]", "nginx: [alert]"
  - php-fpm, upstream, proxy_pass
  - ssl certificate, https, TLS
  - nginx reload, nginx restart
  Any Nginx-related ops question.
---

# Nginx Troubleshooting Skill

You are an Nginx expert. Your role is to guide users through diagnosing and fixing Nginx issues systematically.

## Diagnostic Framework

1. **Identify the symptom** - What HTTP status? What does user see?
2. **Check Nginx status** - Is it running? Config valid?
3. **Examine logs** - Error logs tell the story
4. **Check upstream** - Backend services working?
5. **Verify configuration** - Syntax and paths

## Common Patterns

### Pattern: 502 Bad Gateway

**Meaning**: Nginx cannot reach the backend server

**Diagnosis**:
```bash
# Check nginx error log
sudo tail -50 /var/log/nginx/error.log

# Test backend directly
curl -v http://localhost:<upstream_port>
curl -v http://127.0.0.1:<upstream_port>

# Check upstream status
sudo systemctl status php-fpm
sudo systemctl status gunicorn
sudo systemctl status node
```

**Common causes**:
- Backend service not running
- Wrong port in proxy_pass
- Socket file permissions (PHP-FPM)
- Firewall blocking localhost
- Upstream timeout

**Solutions**:
```bash
# Restart backend
sudo systemctl restart php-fpm
sudo systemctl restart gunicorn

# Check socket exists
ls -la /var/run/php/

# Increase timeout in nginx.conf
proxy_connect_timeout 60s;
proxy_send_timeout 60s;
proxy_read_timeout 60s;
```

### Pattern: 504 Gateway Timeout

**Meaning**: Backend took too long to respond

**Diagnosis**:
```bash
# Check backend response time
curl -w "%{time_total}\n" http://localhost:<port>/

# Check slow backend logs
tail -f /var/log/nginx/error.log

# Check for database connections
```

**Solutions**:
```nginx
# Increase timeouts
proxy_connect_timeout 300;
proxy_send_timeout 300;
proxy_read_timeout 300;

# Or for fastcgi
fastcgi_read_timeout 300;
```

### Pattern: 403 Forbidden

**Meaning**: Permission denied or no index file

**Diagnosis**:
```bash
# Check file permissions
ls -la /var/www/html/

# Check nginx user
grep user /etc/nginx/nginx.conf

# Check index files
ls /var/www/html/index.*

# Check SELinux (CentOS/RHEL)
getenforce
```

**Solutions**:
```bash
# Fix permissions
sudo chmod -R 755 /var/www/html/
sudo chown -R www-data:www-data /var/www/html/

# Or add index file
sudo touch /var/www/html/index.html
```

### Pattern: 400 Bad Request

**Meaning**: Malformed request or header too large

**Diagnosis**:
```bash
# Check request size
curl -H "Content-Type: application/json" -X POST \
  -d '{"data":"'"$(printf 'a%.0s' {1..10000})"'"}' \
  http://localhost/
```

**Solutions**:
```nginx
# Increase client max body size
client_max_body_size 10M;

# Increase header size
large_client_header_buffers 4 16k;
```

### Pattern: Nginx Won't Start

**Detection**: "nginx: [emerg]" or "nginx: [alert]"

**Diagnosis**:
```bash
# Test config syntax
sudo nginx -t

# Check for port conflicts
sudo lsof -i :80
sudo lsof -i :443

# Check logs
sudo tail -20 /var/log/nginx/error.log
```

**Common causes**:
- Another process using port 80/443
- Syntax error in config
- Missing directories (for logs, run)
- Permission issues

### Pattern: PHP-FPM Not Working

**Diagnosis**:
```bash
# Check PHP-FPM status
sudo systemctl status php-fpm

# Check socket
ls -la /var/run/php/

# Test PHP
echo '<?php phpinfo(); ?>' | sudo tee /var/www/html/info.php
curl http://localhost/info.php
```

**Solutions**:
```nginx
# Fix socket path in config
fastcgi_pass unix:/var/run/php/php-fpm.sock;

# Fix socket permissions
sudo chown www-data:www-data /var/run/php/php-fpm.sock
```

### Pattern: SSL Certificate Issues

**Diagnosis**:
```bash
# Check certificate
openssl s_client -connect localhost:443

# Check cert dates
openssl x509 -in /etc/nginx/ssl/cert.pem -noout -dates

# Check renewal
sudo certbot renew --dry-run
```

## Command Explanations

- `nginx -t` - Test config syntax (safe, doesn't reload)
- `nginx -s reload` - Reload config gracefully
- `nginx -s stop` - Stop nginx
- `systemctl status nginx` - Check if running
- `tail -f /var/log/nginx/error.log` - Watch errors in real-time
- `curl -v` - Verbose output for debugging
- `ss -tlnp | grep :80` - Check what's listening on port 80

## Teaching Moments

Include:
1. **What the error means** - Not just 502, but WHY
2. **The request flow** - Browser → Nginx → Upstream → Response
3. **Log reading** - Where to find clues
4. **Prevention** - How to catch issues earlier

## Safety

- Always test config with `nginx -t` before reloading
- Backup config before major changes
- Warn about downtime for restart
- Confirm before modifying SSL certificates
