# Container Deployment for CIM Person Domain

**cim-domain-* services are NATS microservices designed to run in containers and scale horizontally.**

This document covers deploying the Person Domain Service in containers using:
- **Proxmox LXC** (via nixos-generators)
- **NixOS containers** (native NixOS containerization)
- **nix-darwin** (macOS launchd services)

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│         NATS Cluster (10.0.0.41:4222)              │
│                  JetStream                          │
└─────────────────────────────────────────────────────┘
                       ▲ ▲ ▲
                       │ │ │
         ┌─────────────┴─┴─┴─────────────┐
         │                                │
    ┌────▼────┐    ┌────▼────┐    ┌─────▼───┐
    │ Person  │    │ Person  │    │ Person  │
    │ Service │    │ Service │    │ Service │
    │ (LXC 1) │    │ (LXC 2) │    │ (LXC 3) │
    └─────────┘    └─────────┘    └─────────┘
     10.0.64.140    10.0.64.141    10.0.64.142

    Multiple replicas for:
    - High availability
    - Load distribution
    - Geographic distribution
```

Each container:
- Runs `person-service` binary
- Connects to same NATS cluster
- Subscribes to `person.commands.>`
- Publishes to `person.events.>`
- Shares JetStream event store
- Can be replicated infinitely

## Deployment Methods

### 1. Proxmox LXC (Production)

#### Build LXC Container

```bash
# Build Proxmox LXC container image
nix build .#person-lxc

# Result will be in ./result
ls -lh result/
# Output: nixos-system-person-service-24.05.NNNN.tar.xz
```

#### Deploy to Proxmox

1. **Copy to Proxmox host**:
   ```bash
   scp result/*.tar.xz root@proxmox:/var/lib/vz/template/cache/
   ```

2. **Create container** (via Proxmox web UI or CLI):
   ```bash
   pct create 140 \
     /var/lib/vz/template/cache/nixos-system-person-service-*.tar.xz \
     --hostname person-service-1 \
     --cores 2 \
     --memory 2048 \
     --net0 name=eth0,bridge=vmbr0,ip=10.0.64.140/19,gw=10.0.64.1 \
     --storage local-lvm \
     --unprivileged 1 \
     --features nesting=1
   ```

3. **Start container**:
   ```bash
   pct start 140
   ```

4. **Verify service**:
   ```bash
   pct enter 140
   systemctl status cim-domain-person
   journalctl -u cim-domain-person -f
   ```

#### Scale Horizontally

Create multiple replicas with different IPs:

```bash
# Replica 2
pct create 141 /var/lib/vz/template/cache/nixos-system-*.tar.xz \
  --hostname person-service-2 \
  --cores 2 --memory 2048 \
  --net0 name=eth0,bridge=vmbr0,ip=10.0.64.141/19,gw=10.0.64.1 \
  --storage local-lvm --unprivileged 1 --features nesting=1

# Replica 3
pct create 142 /var/lib/vz/template/cache/nixos-system-*.tar.xz \
  --hostname person-service-3 \
  --cores 2 --memory 2048 \
  --net0 name=eth0,bridge=vmbr0,ip=10.0.64.142/19,gw=10.0.64.1 \
  --storage local-lvm --unprivileged 1 --features nesting=1
```

All replicas connect to the same NATS cluster and share the event store.

### 2. NixOS Containers

#### System Configuration

Add to your NixOS `configuration.nix`:

```nix
{
  # Import the module
  imports = [
    /path/to/cim-domain-person/deployment/nix/container.nix
  ];

  # Define containers
  containers.person-service = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.0.64.1";
    localAddress = "10.0.64.140";

    config = { config, pkgs, ... }: {
      services.cim-domain-person = {
        enable = true;
        natsUrl = "nats://10.0.0.41:4222";
        streamName = "PERSON_EVENTS";
        sshKeys = [
          "ssh-ed25519 AAAAC3Nz... user@host"
        ];
      };
    };
  };
}
```

#### Multiple Replicas

```nix
{
  containers = {
    person-service-1 = {
      autoStart = true;
      privateNetwork = true;
      hostAddress = "10.0.64.1";
      localAddress = "10.0.64.140";
      config = { ... }; # Same as above
    };

    person-service-2 = {
      autoStart = true;
      privateNetwork = true;
      hostAddress = "10.0.64.1";
      localAddress = "10.0.64.141";
      config = { ... }; # Same configuration
    };

    person-service-3 = {
      autoStart = true;
      privateNetwork = true;
      hostAddress = "10.0.64.1";
      localAddress = "10.0.64.142";
      config = { ... }; # Same configuration
    };
  };
}
```

#### Container Management

```bash
# Start/stop containers
nixos-container start person-service
nixos-container stop person-service

# Enter container
nixos-container root-login person-service

# Check status
systemctl status container@person-service
```

### 3. nix-darwin (macOS Development)

#### Darwin Configuration

Add to your nix-darwin `flake.nix`:

```nix
{
  inputs = {
    cim-domain-person.url = "github:thecowboyai/cim-domain-person";
  };

  outputs = { self, darwin, cim-domain-person }: {
    darwinConfigurations.myMac = darwin.lib.darwinSystem {
      modules = [
        cim-domain-person.darwinModules.default
        {
          services.cim-domain-person = {
            enable = true;
            natsUrl = "nats://10.0.0.41:4222";
            streamName = "PERSON_EVENTS";
            logLevel = "debug";
          };
        }
      ];
    };
  };
}
```

#### Standalone Binary (Development)

```bash
# Build for macOS
nix build .#person-service-darwin

# Run manually
NATS_URL=nats://10.0.0.41:4222 ./result/bin/person-service
```

## Configuration Options

### Container Configuration

All deployment methods support these configuration options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | false | Enable the service |
| `natsUrl` | string | `nats://10.0.0.41:4222` | NATS server URL |
| `streamName` | string | `PERSON_EVENTS` | JetStream stream name |
| `logLevel` | enum | `info` | Log level (trace/debug/info/warn/error) |
| `snapshotFrequency` | int | `100` | Snapshot every N events |
| `containerIp` | string? | null | Static IP (Proxmox/LXC) |
| `gateway` | string | `10.0.64.1` | Default gateway |
| `prefixLength` | int | `19` | Network prefix |
| `nameservers` | list | `["10.0.0.254", "1.1.1.1"]` | DNS servers |
| `sshKeys` | list | `[]` | SSH authorized keys |

### Example Configurations

#### Development (Local NATS)
```nix
services.cim-domain-person = {
  enable = true;
  natsUrl = "nats://localhost:4222";
  logLevel = "debug";
};
```

#### Production (Remote NATS Cluster)
```nix
services.cim-domain-person = {
  enable = true;
  natsUrl = "nats://nats.production.example.com:4222";
  streamName = "PRODUCTION_PERSON_EVENTS";
  logLevel = "warn";
  snapshotFrequency = 500;
};
```

#### Multi-region (Geographic Distribution)
```nix
# US East
services.cim-domain-person = {
  enable = true;
  natsUrl = "nats://nats-us-east.example.com:4222";
  streamName = "PERSON_EVENTS_US_EAST";
};

# EU West
services.cim-domain-person = {
  enable = true;
  natsUrl = "nats://nats-eu-west.example.com:4222";
  streamName = "PERSON_EVENTS_EU_WEST";
};
```

## Scaling Strategies

### Horizontal Scaling

Deploy multiple identical containers:
- All connect to same NATS cluster
- NATS handles load distribution
- No coordination required between instances
- Add/remove replicas as needed

**Benefits:**
- High availability (if one fails, others continue)
- Load distribution across replicas
- Easy to scale up/down
- Geographic distribution possible

### Vertical Scaling

Adjust container resources:

**Proxmox:**
```bash
pct set 140 --cores 4 --memory 4096
pct reboot 140
```

**NixOS containers:**
```nix
containers.person-service.config = {
  # Resource limits via systemd
  systemd.services.cim-domain-person.serviceConfig = {
    MemoryMax = "4G";
    CPUQuota = "400%";
  };
};
```

### Geographic Distribution

Deploy across regions:

```
NATS Leaf Nodes:
  ├── US-East (10.0.64.140-149)
  ├── US-West (10.0.65.140-149)
  ├── EU-West (10.0.66.140-149)
  └── APAC (10.0.67.140-149)

Each region:
- 3-5 person-service replicas
- Local NATS leaf node
- Connected to global super-cluster
```

## Monitoring and Operations

### Health Checks

```bash
# SSH into container
ssh root@10.0.64.140

# Check service status
systemctl status cim-domain-person

# View logs
journalctl -u cim-domain-person -f

# Check NATS connectivity
nats context select
nats sub "person.events.>"
```

### Performance Monitoring

```bash
# Container resource usage (Proxmox)
pct exec 140 -- htop

# Service metrics (inside container)
systemctl status cim-domain-person
journalctl -u cim-domain-person --since "1 hour ago"
```

### Backup and Recovery

**JetStream data is centralized**:
- Backup NATS JetStream storage (on NATS server)
- Containers are stateless (can be recreated)
- No container-specific backups needed

**To rebuild a container:**
```bash
# Stop old
pct stop 140
pct destroy 140

# Deploy new
nix build .#person-lxc
scp result/*.tar.xz root@proxmox:/var/lib/vz/template/cache/
pct create 140 ... # Same as before

# Service continues from JetStream
```

## Migration Path: Proxmox → Pure NixOS

Current approach: **Proxmox with NixOS LXC containers**

Future approach: **Pure NixOS with declarative containers**

### Phase 1: Proxmox + NixOS LXC (Current)
```
Proxmox Host
  ├── LXC 140 (NixOS with person-service)
  ├── LXC 141 (NixOS with person-service)
  └── LXC 142 (NixOS with person-service)
```

### Phase 2: Pure NixOS (Future)
```nix
# Single NixOS host with declarative containers
{
  containers = {
    person-1 = { ... };
    person-2 = { ... };
    person-3 = { ... };
  };
}
```

**Migration steps:**
1. Continue using Proxmox LXC during development
2. Test with NixOS containers in parallel
3. Gradually migrate workloads
4. Eventually retire Proxmox for pure NixOS

**Benefits of pure NixOS:**
- Fully declarative infrastructure
- Atomic rollbacks
- Reproducible deployments
- Simplified operations

## Troubleshooting

### Container Won't Start

1. **Check NATS connectivity:**
   ```bash
   pct enter 140
   nc -zv 10.0.0.41 4222
   ```

2. **Check service logs:**
   ```bash
   journalctl -u cim-domain-person -n 50
   ```

3. **Verify configuration:**
   ```bash
   systemctl cat cim-domain-person
   ```

### High Memory Usage

- Reduce `snapshotFrequency` (more frequent snapshots = less memory)
- Increase container memory allocation
- Check for event replay loops in logs

### Network Issues

- Verify IP configuration matches Proxmox network
- Check firewall rules (port 4222 outbound to NATS)
- Verify gateway and DNS settings

## Best Practices

1. **Use identical configurations** for all replicas
2. **Monitor NATS JetStream** storage and performance
3. **Test failover** by stopping containers randomly
4. **Use static IPs** for easier troubleshooting
5. **Keep containers minimal** (single service per container)
6. **Automate deployment** with CI/CD pipelines
7. **Document network topology** for your infrastructure

## Next Steps

1. Deploy first LXC container to Proxmox
2. Verify NATS connectivity and service operation
3. Scale to 3 replicas for high availability
4. Set up monitoring and alerting
5. Plan migration to pure NixOS
6. Deploy additional cim-domain-* services in containers

## Support

- **Issues**: https://github.com/thecowboyai/cim-domain-person/issues
- **Documentation**: See `/deployment` directory
- **Examples**: See container configurations in `flake.nix`
