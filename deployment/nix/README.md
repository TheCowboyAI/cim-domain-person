# NixOS Deployment for CIM Person Domain Service

This directory contains NixOS configuration for deploying the Person Domain Service.

## Quick Start

### Option 1: Using the NixOS Module

Add to your `configuration.nix`:

```nix
{
  imports = [
    /path/to/cim-domain-person/deployment/nix/module.nix
  ];

  services.cim-domain-person = {
    enable = true;
    natsUrl = "nats://10.0.0.41:4222";
    streamName = "PERSON_EVENTS";
    logLevel = "info";
    snapshotFrequency = 100;
  };
}
```

Then rebuild:
```bash
sudo nixos-rebuild switch
```

### Option 2: Using the Flake

Add to your `flake.nix`:

```nix
{
  inputs = {
    cim-domain-person.url = "path:/path/to/cim-domain-person/deployment/nix";
  };

  outputs = { self, nixpkgs, cim-domain-person }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      modules = [
        cim-domain-person.nixosModules.default
        {
          services.cim-domain-person = {
            enable = true;
            natsUrl = "nats://10.0.0.41:4222";
          };
        }
      ];
    };
  };
}
```

### Option 3: Standalone Binary

Build and run the service directly:

```bash
# Build
nix build .#person-service

# Run
./result/bin/person-service
```

## Configuration Options

### `services.cim-domain-person.enable`
- **Type**: boolean
- **Default**: false
- **Description**: Enable the CIM Person Domain Service

### `services.cim-domain-person.natsUrl`
- **Type**: string
- **Default**: "nats://localhost:4222"
- **Description**: NATS server URL
- **Example**: "nats://10.0.0.41:4222"

### `services.cim-domain-person.streamName`
- **Type**: string
- **Default**: "PERSON_EVENTS"
- **Description**: JetStream stream name for person events

### `services.cim-domain-person.logLevel`
- **Type**: enum (trace, debug, info, warn, error)
- **Default**: "info"
- **Description**: Logging verbosity level

### `services.cim-domain-person.snapshotFrequency`
- **Type**: integer
- **Default**: 100
- **Description**: Take snapshot every N events

### `services.cim-domain-person.user`
- **Type**: string
- **Default**: "cim-person"
- **Description**: User account for the service

### `services.cim-domain-person.group`
- **Type**: string
- **Default**: "cim-person"
- **Description**: Group account for the service

### `services.cim-domain-person.dataDir`
- **Type**: path
- **Default**: "/var/lib/cim-person"
- **Description**: Data directory for the service

### `services.cim-domain-person.environmentFile`
- **Type**: null or path
- **Default**: null
- **Description**: Environment file with secrets (NATS credentials, etc.)
- **Example**: "/run/secrets/cim-person-env"

## Examples

### Basic Local Setup
```nix
services.cim-domain-person = {
  enable = true;
  # Uses default localhost NATS
};
```

### Production with Remote NATS
```nix
services.cim-domain-person = {
  enable = true;
  natsUrl = "nats://nats.production.example.com:4222";
  streamName = "PRODUCTION_PERSON_EVENTS";
  logLevel = "warn";
  snapshotFrequency = 500;
};
```

### With Secrets Management
```nix
services.cim-domain-person = {
  enable = true;
  natsUrl = "nats://nats.example.com:4222";
  environmentFile = "/run/secrets/cim-person-env";
  # Environment file should contain:
  # NATS_USER=person-service
  # NATS_PASSWORD=secret
  # or
  # NATS_TOKEN=your-jwt-token
};
```

## Service Management

### Check Status
```bash
systemctl status cim-domain-person
```

### View Logs
```bash
journalctl -u cim-domain-person -f
```

### Restart Service
```bash
systemctl restart cim-domain-person
```

### Stop Service
```bash
systemctl stop cim-domain-person
```

## Security

The service runs with extensive security hardening:
- Runs as unprivileged user
- Private /tmp
- Read-only file system (except data directory)
- Restricted system calls
- No new privileges
- Memory execution protection
- And more (see module.nix for full list)

## Dependencies

The service requires:
- NATS server with JetStream enabled
- Network connectivity to NATS

The module declares a dependency on `nats.service`, so if you have NATS configured as a NixOS service, it will automatically start before this service.

## Troubleshooting

### Service won't start
1. Check NATS is running: `systemctl status nats`
2. Check NATS connectivity: `nc -zv <nats-host> 4222`
3. Check logs: `journalctl -u cim-domain-person -n 50`
4. Verify JetStream is enabled in NATS

### Cannot connect to NATS
- Verify `natsUrl` is correct
- Check firewall rules
- Check NATS server logs
- Verify NATS authentication if required

### High memory usage
- Reduce `snapshotFrequency` (more frequent snapshots = less memory)
- Monitor JetStream storage usage
- Check for memory leaks in logs

## Development

Build for development:
```bash
nix develop
cargo build --bin person-service
```

Run locally:
```bash
NATS_URL=nats://localhost:4222 cargo run --bin person-service
```
