# CIM Person Domain Service Deployment

This directory contains deployment configurations for running the Person Domain Service as a long-running daemon that responds to NATS commands.

## Overview

The **person-service** is a standalone binary that:
- Connects to NATS and JetStream
- Subscribes to `person.commands.>` subject
- Processes Person domain commands (CreatePerson, RecordAttribute, etc.)
- Publishes events to `person.events.>` subject
- Maintains durable event store via JetStream
- Runs continuously as a system service

## Architecture

```
┌─────────────────────────────────────────────┐
│         NATS Cluster (10.0.0.41:4222)       │
│  ┌──────────────────────────────────────┐  │
│  │      JetStream: PERSON_EVENTS        │  │
│  │  • person.events.{id}.created        │  │
│  │  • person.events.{id}.updated        │  │
│  │  • person.events.{id}.attribute_*    │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
                    ▲ ▼
                    │ │ NATS Protocol
                    │ │
         ┌──────────┴─┴───────────┐
         │   person-service       │
         │                        │
         │  • Command Handler     │
         │  • Event Publisher     │
         │  • Event Store         │
         │  • Snapshot Store      │
         └────────────────────────┘
```

## Deployment Options

### 1. NixOS Module (Recommended for NixOS)

See [nix/README.md](nix/README.md) for full documentation.

**Quick Start:**

```nix
# In configuration.nix
{
  imports = [ /path/to/cim-domain-person/deployment/nix/module.nix ];

  services.cim-domain-person = {
    enable = true;
    natsUrl = "nats://10.0.0.41:4222";
    streamName = "PERSON_EVENTS";
    logLevel = "info";
  };
}
```

```bash
sudo nixos-rebuild switch
```

**Advantages:**
- Declarative configuration
- Automatic dependency management
- Security hardening built-in
- Easy rollback
- Integration with NixOS secrets management

### 2. Systemd Service (Traditional Linux)

See [systemd/](systemd/) directory.

**Installation:**

```bash
cd /path/to/cim-domain-person
sudo ./deployment/systemd/install.sh
```

**Configuration:**

Edit `/etc/cim-person/environment`:
```bash
NATS_URL=nats://10.0.0.41:4222
STREAM_NAME=PERSON_EVENTS
LOG_LEVEL=info
SNAPSHOT_FREQ=100
```

**Start Service:**

```bash
sudo systemctl start cim-domain-person
sudo systemctl status cim-domain-person
sudo journalctl -u cim-domain-person -f
```

**Advantages:**
- Works on any Linux distribution
- Standard systemd service
- Familiar management tools

### 3. Docker Container (Planned)

Docker deployment is planned for future release.

### 4. Kubernetes (Planned)

Kubernetes deployment is planned for future release.

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NATS_URL` | `nats://localhost:4222` | NATS server URL |
| `STREAM_NAME` | `PERSON_EVENTS` | JetStream stream name |
| `LOG_LEVEL` | `info` | Logging level (trace, debug, info, warn, error) |
| `SNAPSHOT_FREQ` | `100` | Snapshot frequency in events |

### NATS Authentication (Optional)

```bash
# Username/Password
NATS_USER=person-service
NATS_PASSWORD=secret

# Token-based
NATS_TOKEN=your-jwt-token

# Credentials file
NATS_CREDS_FILE=/path/to/nats.creds
```

## Testing the Service

### 1. Build and Run Locally

```bash
cargo build --bin person-service

NATS_URL=nats://10.0.0.41:4222 ./target/debug/person-service
```

### 2. Send Test Commands

Use the NATS CLI or create a client:

```bash
# Install NATS CLI
nix-shell -p natscli

# Send a command
nats pub person.commands.test '{"CreatePerson": {"person_id": "...", "name": {...}, "source": "test"}}'
```

### 3. Monitor Events

```bash
# Subscribe to all person events
nats sub "person.events.>"
```

## Service Management

### NixOS

```bash
# Status
systemctl status cim-domain-person

# Logs
journalctl -u cim-domain-person -f

# Restart
systemctl restart cim-domain-person

# Stop
systemctl stop cim-domain-person

# Disable
systemctl disable cim-domain-person
```

### Configuration Updates

#### NixOS
1. Edit `configuration.nix`
2. Run `sudo nixos-rebuild switch`
3. Service automatically restarts

#### Systemd
1. Edit `/etc/cim-person/environment`
2. Run `sudo systemctl restart cim-domain-person`

## Monitoring

### Logs

The service logs to systemd journal:

```bash
# Follow logs
journalctl -u cim-domain-person -f

# Last 100 lines
journalctl -u cim-domain-person -n 100

# Logs since last boot
journalctl -u cim-domain-person -b

# Logs with specific level
journalctl -u cim-domain-person -p err
```

### Metrics

Future releases will include:
- Prometheus metrics endpoint
- Command processing latency
- Event publishing rate
- Error rates
- JetStream storage usage

### Health Checks

Future releases will include:
- HTTP health endpoint
- NATS connectivity check
- JetStream availability check

## Security

### Systemd/NixOS Security Features

The service runs with extensive hardening:

- **User Isolation**: Runs as dedicated `cim-person` user
- **Filesystem**: Read-only root, writable data directory only
- **Network**: Restricted to AF_INET/AF_INET6/AF_UNIX
- **System Calls**: Filtered to @system-service only
- **No New Privileges**: Cannot gain additional privileges
- **Memory Protection**: W^X enforcement
- **Private Temp**: Isolated /tmp directory
- **Resource Limits**: Bounded file descriptors and tasks

### NATS Security

For production deployments:

1. **Enable TLS**:
   ```bash
   NATS_URL=tls://nats.example.com:4222
   ```

2. **Use Authentication**:
   - JWT tokens (recommended)
   - Username/password
   - NKeys

3. **Use Credentials File**:
   ```bash
   NATS_CREDS_FILE=/run/secrets/nats.creds
   ```

## Troubleshooting

### Service Won't Start

1. **Check NATS is running:**
   ```bash
   nc -zv 10.0.0.41 4222
   ```

2. **Check JetStream is enabled:**
   ```bash
   nats stream ls
   ```

3. **Check logs:**
   ```bash
   journalctl -u cim-domain-person -n 50
   ```

### Cannot Connect to NATS

- Verify NATS_URL is correct
- Check firewall rules
- Verify NATS authentication
- Check NATS server logs

### Events Not Publishing

- Check JetStream is enabled
- Verify stream exists: `nats stream info PERSON_EVENTS`
- Check permissions on stream subjects
- Review service logs for errors

### High Resource Usage

- Reduce snapshot frequency for less memory
- Check for event replay (high CPU)
- Monitor JetStream storage usage
- Review error logs for loops

## Production Checklist

- [ ] NATS cluster configured with JetStream
- [ ] TLS enabled for NATS connections
- [ ] Authentication configured (JWT/credentials)
- [ ] Environment variables configured
- [ ] Service enabled and running
- [ ] Logs being collected
- [ ] Monitoring configured
- [ ] Backup strategy for JetStream storage
- [ ] Firewall rules configured
- [ ] Security hardening verified
- [ ] Load testing completed
- [ ] Disaster recovery plan documented

## Performance Tuning

### Snapshot Frequency

Higher frequency = Less memory, more disk I/O
Lower frequency = More memory, less disk I/O

Recommended: 100-500 events

### JetStream Configuration

```nats
# Edit NATS server config
jetstream {
  store_dir: /var/lib/nats/jetstream
  max_memory_store: 1GB
  max_file_store: 10GB
}
```

### Resource Limits

Adjust in systemd configuration:
- `LimitNOFILE`: File descriptor limit
- `TasksMax`: Thread limit

## Next Steps

1. **Deploy to Production**: Follow the deployment guide for your platform
2. **Configure Monitoring**: Set up log aggregation and metrics
3. **Test Failover**: Verify behavior during NATS outages
4. **Load Test**: Validate performance under expected load
5. **Document Runbooks**: Create operational procedures

## Support

- Issues: https://github.com/thecowboyai/cim-domain-person/issues
- Documentation: See `/doc` directory in repository
- Examples: See `/examples` directory
