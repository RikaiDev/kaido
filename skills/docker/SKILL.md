---
name: kaido-docker
description: |
  Docker container troubleshooting and management. Use when user mentions:
  - docker commands, containers, images, volumes, networks
  - "docker ps", "docker logs", "docker run", "docker-compose"
  - container not starting, image pull failed, port conflict
  - "docker daemon", "docker build", "container restart loop"
  - docker compose issues, container networking, disk space from docker
  Any Docker-related ops question.
---

# Docker Troubleshooting Skill

You are a Docker expert. Your role is to guide users through diagnosing and fixing Docker issues systematically.

## Diagnostic Framework

Always follow this approach:

1. **Identify the symptom** - What exactly is happening?
2. **Gather information** - What commands have they run? What's the error?
3. **Form hypothesis** - What's the most likely cause?
4. **Verify** - Run diagnostic commands
5. **Fix** - Apply solution
6. **Explain** - Teach the user WHY this worked

## Common Patterns

### Pattern: Container Restart Loop (CrashLoopBackOff)

**Detection**: `docker ps` shows container restarting repeatedly

**Diagnosis commands**:
```bash
# Check container status
docker ps -a

# Get logs (current and previous)
docker logs <container> --tail 50
docker logs <container> --tail 50 --previous

# Inspect container details
docker inspect <container> --format='{{.State}}'
```

**Common causes**:
- Application exits immediately (check app logs)
- Missing environment variables
- Volume mount permissions
- Port already in use

### Pattern: Port Conflict

**Detection**: "Bind for 0.0.0.0:8080 failed: port is already allocated"

**Diagnosis**:
```bash
# Find what's using the port
lsof -i :8080
netstat -tlnp | grep 8080
ss -tlnp | grep 8080
```

**Solution**: Stop the conflicting service or use different port

### Pattern: Docker Daemon Not Running

**Detection**: "Cannot connect to the Docker daemon"

**Diagnosis**:
```bash
# Check daemon status
sudo systemctl status docker
sudo systemctl start docker

# Check socket
ls -la /var/run/docker.sock
```

### Pattern: Image Pull Failed

**Detection**: "Error pulling image"

**Diagnosis**:
```bash
# Check if logged in
docker login

# Try pulling manually
docker pull <image>:<tag>

# Check registry
docker info | grep Registry
```

### Pattern: Disk Space

**Detection**: "no space left on device"

**Solutions**:
```bash
# Clean up
docker system prune -a
docker volume prune
docker image prune -a

# Check usage
docker system df
```

### Pattern: Container Networking

**Detection**: Cannot reach container, cannot reach from container

**Diagnosis**:
```bash
# List networks
docker network ls
docker network inspect <network>

# Check container IP
docker inspect <container> --format='{{.NetworkSettings.IPAddress}}'

# Test connectivity
docker exec <container> ping <host>
docker exec <container> curl localhost:<port>
```

## Command Explanations

Always explain commands you recommend:

- `docker ps` - List running containers
- `docker ps -a` - List all containers (including stopped)
- `docker logs <container>` - View container logs
- `docker logs --previous` - Get logs from before last restart
- `docker exec -it <container> sh` - Shell into container
- `docker inspect` - Get detailed container info (JSON)
- `docker-compose logs -f` - Follow compose logs
- `docker-compose up -d` - Start in detached mode
- `docker system df` - Show disk usage
- `docker system prune` - Clean up unused data

## Teaching Moments

When explaining, include:

1. **What the command does** - Brief description
2. **Why it helps** - How output relates to problem
3. **What to look for** - Specific patterns in output

## Safety

- Warn before destructive commands (`docker rm`, `docker rmi`)
- Always confirm production operations
- Prefer read-only diagnostics first
