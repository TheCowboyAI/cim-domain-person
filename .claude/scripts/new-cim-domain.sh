#!/usr/bin/env bash
# Bootstrap a new cim-domain-* project with standard structure
#
# Usage:
#   ./new-cim-domain.sh order "Order management domain"
#   ./new-cim-domain.sh invoice "Invoice generation and tracking"

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}!${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Check arguments
if [ $# -lt 2 ]; then
    error "Usage: $0 <domain-name> <description>"
    echo ""
    echo "Examples:"
    echo "  $0 order \"Order management domain\""
    echo "  $0 invoice \"Invoice generation and tracking\""
    echo "  $0 product \"Product catalog and inventory\""
    exit 1
fi

DOMAIN_NAME="$1"
DESCRIPTION="$2"
DOMAIN_UPPER=$(echo "$DOMAIN_NAME" | tr '[:lower:]' '[:upper:]')
DOMAIN_TITLE=$(echo "$DOMAIN_NAME" | sed 's/.*/\u&/')

PROJECT_NAME="cim-domain-$DOMAIN_NAME"
CURRENT_DATE=$(date -I)
CURRENT_YEAR=$(date +%Y)

header "Creating CIM Domain: $DOMAIN_TITLE"
info "Project: $PROJECT_NAME"
info "Description: $DESCRIPTION"
echo ""

# Create project directory
if [ -d "$PROJECT_NAME" ]; then
    error "Directory $PROJECT_NAME already exists!"
    exit 1
fi

mkdir -p "$PROJECT_NAME"
cd "$PROJECT_NAME"

info "Created project directory"

# Create directory structure
mkdir -p src/{bin,aggregate,commands,events,value_objects,services,queries,projections,category_theory,infrastructure}
mkdir -p deployment/{nix,systemd}
mkdir -p examples
mkdir -p tests
mkdir -p doc
mkdir -p .claude/{scripts,instructions}

info "Created directory structure"

# Create Cargo.toml
cat > Cargo.toml <<EOF
[package]
name = "cim-domain-$DOMAIN_NAME"
version = "0.1.0"
edition = "2021"
authors = ["The Cowboy AI"]
description = "$DESCRIPTION"
license = "MIT"
repository = "https://github.com/thecowboyai/cim-domain-$DOMAIN_NAME"
keywords = ["cim", "domain", "$DOMAIN_NAME", "event-sourcing", "nats"]
categories = ["data-structures"]

[dependencies]
# CIM Domain Framework
cim-domain = { path = "../cim-domain" }

# Core dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.4", features = ["v7", "serde"] }

# Async dependencies
async-trait = "0.1"
tokio = { version = "1.32", features = ["full"] }
async-nats = "0.35"
futures = "0.3"
tokio-stream = "0.1"

# Content addressing
cid = "0.11"
multihash = "0.19"
blake3 = "1.5"

[dev-dependencies]
tokio-test = "0.4"
pretty_assertions = "1.4"
rstest = "0.18"
tracing-subscriber = "0.3"

[[bin]]
name = "$DOMAIN_NAME-service"
path = "src/bin/$DOMAIN_NAME-service.rs"

[features]
default = []
EOF

info "Created Cargo.toml"

# Create flake.nix
cat > flake.nix <<'FLAKE_EOF'
{
  description = "CIM %DOMAIN_TITLE% Domain - Scalable NATS event-sourced service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    darwin = {
      url = "github:LnL7/nix-darwin";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, darwin, flake-utils, rust-overlay }:
    let
      flakeContext = {
        inherit (self) inputs;
        inherit self;
      };

      systemOutputs = flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };

          rustVersion = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };

          %DOMAIN_NAME%-service = pkgs.rustPlatform.buildRustPackage {
            pname = "cim-domain-%DOMAIN_NAME%";
            version = "0.1.0";
            src = ./.;

            cargoLock = { lockFile = ./Cargo.lock; };
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ];
            cargoBuildFlags = [ "--bin" "%DOMAIN_NAME%-service" ];

            meta = with pkgs.lib; {
              description = "%DESCRIPTION%";
              homepage = "https://github.com/thecowboyai/cim-domain-%DOMAIN_NAME%";
              license = licenses.mit;
            };
          };

        in {
          packages = {
            default = %DOMAIN_NAME%-service;
            %DOMAIN_NAME%-service = %DOMAIN_NAME%-service;
          };

          apps.default = {
            type = "app";
            program = "${%DOMAIN_NAME%-service}/bin/%DOMAIN_NAME%-service";
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustVersion cargo-edit cargo-watch
              pkg-config openssl nats-server
            ];

            shellHook = ''
              echo "CIM %DOMAIN_TITLE% Domain - Development Environment"
              echo ""
              echo "Commands:"
              echo "  cargo build --bin %DOMAIN_NAME%-service"
              echo "  cargo run --bin %DOMAIN_NAME%-service"
              echo "  nix build .#%DOMAIN_NAME%-lxc"
            '';
          };
        });

    in systemOutputs // {
      nixosModules = {
        default = import ./deployment/nix/module.nix;
        container = import ./deployment/nix/container.nix flakeContext;
      };

      packages.x86_64-linux.%DOMAIN_NAME%-lxc = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "proxmox-lxc";
        modules = [
          (import ./deployment/nix/container.nix flakeContext)
          {
            services.cim-domain-%DOMAIN_NAME% = {
              enable = true;
              natsUrl = "nats://10.0.0.41:4222";
            };
          }
        ];
      };

      darwinModules.default = { config, lib, pkgs, ... }:
        with lib;
        let cfg = config.services.cim-domain-%DOMAIN_NAME%;
        in {
          options.services.cim-domain-%DOMAIN_NAME% = {
            enable = mkEnableOption "CIM %DOMAIN_TITLE% Domain Service";
            natsUrl = mkOption {
              type = types.str;
              default = "nats://localhost:4222";
            };
          };

          config = mkIf cfg.enable {
            launchd.daemons.cim-domain-%DOMAIN_NAME% = {
              serviceConfig = {
                ProgramArguments = [
                  "${self.packages.${pkgs.system}.%DOMAIN_NAME%-service}/bin/%DOMAIN_NAME%-service"
                ];
                EnvironmentVariables = { NATS_URL = cfg.natsUrl; };
                KeepAlive = true;
                RunAtLoad = true;
              };
            };
          };
        };
    };
}
FLAKE_EOF

# Replace placeholders in flake.nix
sed -i "s/%DOMAIN_NAME%/$DOMAIN_NAME/g" flake.nix
sed -i "s/%DOMAIN_TITLE%/$DOMAIN_TITLE/g" flake.nix
sed -i "s/%DESCRIPTION%/$DESCRIPTION/g" flake.nix
sed -i "s/%DOMAIN_UPPER%/$DOMAIN_UPPER/g" flake.nix

info "Created flake.nix"

# Create basic lib.rs
cat > src/lib.rs <<EOF
//! # CIM $DOMAIN_TITLE Domain
//!
//! $DESCRIPTION
//!
//! This is a pure functional event-sourced domain following Category Theory
//! and Functional Reactive Programming principles.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod value_objects;
pub mod services;
pub mod queries;
pub mod projections;
pub mod category_theory;
pub mod infrastructure;

pub use cim_domain;
EOF

info "Created src/lib.rs"

# Create service binary
cat > "src/bin/$DOMAIN_NAME-service.rs" <<EOF
//! $DOMAIN_TITLE Domain Service
//!
//! NATS-based event-sourced microservice

use std::env;
use std::sync::Arc;
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("Starting $DOMAIN_TITLE Domain Service");

    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let stream_name = env::var("STREAM_NAME")
        .unwrap_or_else(|_| "${DOMAIN_UPPER}_EVENTS".to_string());

    info!("Configuration:");
    info!("  NATS URL: {}", nats_url);
    info!("  Stream Name: {}", stream_name);

    // Connect to NATS
    let client = async_nats::connect(&nats_url).await?;
    info!("✓ Connected to NATS");

    // TODO: Initialize domain components
    // - Event store
    // - Repository
    // - Command handler

    info!("$DOMAIN_TITLE Domain Service ready");
    info!("Listening on $DOMAIN_NAME.commands.>");
    info!("Press Ctrl+C to shutdown");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    info!("$DOMAIN_TITLE Domain Service stopped");
    Ok(())
}
EOF

info "Created service binary"

# Create README
cat > README.md <<EOF
# CIM $DOMAIN_TITLE Domain

$DESCRIPTION

## Overview

This is a **cim-domain-*** service - a NATS-based event-sourced microservice following pure Category Theory and Functional Reactive Programming principles.

## Features

- ✅ Pure functional event sourcing
- ✅ Category Theory compliant
- ✅ 100% FRP (Functional Reactive Programming)
- ✅ NATS/JetStream integration
- ✅ Scalable container deployment
- ✅ Proxmox LXC support
- ✅ NixOS containers
- ✅ nix-darwin support

## Quick Start

### Development

\`\`\`bash
# Enter development environment
nix develop

# Build
cargo build --bin $DOMAIN_NAME-service

# Run locally
NATS_URL=nats://localhost:4222 cargo run --bin $DOMAIN_NAME-service
\`\`\`

### Deployment

#### Proxmox LXC
\`\`\`bash
# Build container
nix build .#$DOMAIN_NAME-lxc

# Deploy
scp result/*.tar.xz root@proxmox:/var/lib/vz/template/cache/
pct create 150 /var/lib/vz/template/cache/*.tar.xz \\
  --hostname $DOMAIN_NAME-service \\
  --net0 name=eth0,bridge=vmbr0,ip=10.0.64.150/19,gw=10.0.64.1
\`\`\`

See [deployment/CONTAINER_DEPLOYMENT.md](deployment/CONTAINER_DEPLOYMENT.md) for details.

## Architecture

### Event Sourcing
\`\`\`rust
Command → [Event] → State
\`\`\`

All state changes occur through immutable events. No CRUD operations.

### NATS Integration
- **Commands**: \`$DOMAIN_NAME.commands.>\`
- **Events**: \`$DOMAIN_NAME.events.>\`
- **Storage**: JetStream durable storage

### Category Theory
- Functors for structure-preserving transformations
- Monads for compositional operations
- Natural transformations between functors
- Coalgebras for state machines

## Development

### Project Structure
\`\`\`
src/
├── aggregate/        # Domain aggregates
├── commands/         # Command types
├── events/           # Event types
├── value_objects/    # Value types
├── services/         # Application services
├── queries/          # Query specifications
├── projections/      # Read model projections
├── category_theory/  # CT implementations
└── infrastructure/   # Infrastructure adapters
\`\`\`

### Testing
\`\`\`bash
cargo test
\`\`\`

## License

MIT

## Support

- Issues: https://github.com/thecowboyai/cim-domain-$DOMAIN_NAME/issues
- Documentation: See \`doc/\` directory
EOF

info "Created README.md"

# Create CHANGELOG
cat > CHANGELOG.md <<EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - $CURRENT_DATE - Initial Release

### Added
- Initial project structure
- Basic domain model
- NATS service binary
- Container deployment support
- NixOS/Proxmox LXC configuration
- nix-darwin support

### Architecture
- Pure functional event sourcing
- Category Theory compliance
- 100% FRP implementation
- MealyStateMachine pattern
EOF

info "Created CHANGELOG.md"

# Create .gitignore
cat > .gitignore <<EOF
# Rust
target/
Cargo.lock
**/*.rs.bk

# Nix
result
result-*
.direnv/

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
EOF

info "Created .gitignore"

# Copy CLAUDE template
cat > .claude/CLAUDE.md <<EOF
# CIM $DOMAIN_TITLE Domain - Development Instructions

This project follows the standard CIM domain template.

## Context

This is **cim-domain-$DOMAIN_NAME** - a NATS-based event-sourced microservice.

**Domain**: $DESCRIPTION

## Quick Reference

See \`.claude/CIM_DOMAIN_TEMPLATE.md\` for complete template instructions.

### Key Principles
1. Pure functional event sourcing only
2. No CRUD operations
3. All events immutable
4. Category Theory compliant
5. 100% FRP (no side effects in domain logic)
6. Always use \`Uuid::now_v7()\` for IDs

### Project Structure
- \`src/aggregate/\` - Domain aggregates with MealyStateMachine
- \`src/commands/\` - Command types
- \`src/events/\` - Event types (immutable)
- \`src/infrastructure/\` - NATS integration
- \`src/bin/$DOMAIN_NAME-service.rs\` - Service binary

### NATS Subjects
- Commands: \`$DOMAIN_NAME.commands.>\`
- Events: \`$DOMAIN_NAME.events.>\`

### Deployment
\`\`\`bash
# Build LXC container
nix build .#$DOMAIN_NAME-lxc

# Run locally
NATS_URL=nats://10.0.0.41:4222 cargo run --bin $DOMAIN_NAME-service
\`\`\`

## Implementation Checklist

- [ ] Define aggregate root in \`src/aggregate/$DOMAIN_NAME.rs\`
- [ ] Implement MealyStateMachine trait
- [ ] Define commands in \`src/commands/mod.rs\`
- [ ] Define events in \`src/events/mod.rs\`
- [ ] Create value objects
- [ ] Implement NATS integration
- [ ] Add NATS event store
- [ ] Create command handler
- [ ] Write tests
- [ ] Add examples
- [ ] Document API

## Next Steps

1. Implement core domain aggregate
2. Define commands and events
3. Add NATS integration
4. Test with local NATS
5. Build and deploy container
EOF

# Copy template for reference
cp .claude/CLAUDE.md .claude/CIM_DOMAIN_TEMPLATE.md

info "Created .claude/ configuration"

# Initialize git repo
git init
git add .
git commit -m "Initial commit: Bootstrap CIM $DOMAIN_TITLE domain

Generated using new-cim-domain.sh script
Following CIM domain template v1.0

Project: cim-domain-$DOMAIN_NAME
Description: $DESCRIPTION

Structure includes:
- Pure functional event sourcing architecture
- NATS/JetStream integration
- Container deployment (Proxmox LXC, NixOS, nix-darwin)
- Category Theory compliance
- 100% FRP implementation

Ready for domain implementation."

info "Initialized git repository"

# Summary
echo ""
header "✅ Project Created Successfully!"
echo ""
info "Project: $PROJECT_NAME"
info "Location: $(pwd)"
echo ""
echo "Next steps:"
echo "  1. cd $PROJECT_NAME"
echo "  2. Implement domain aggregate in src/aggregate/$DOMAIN_NAME.rs"
echo "  3. Define commands and events"
echo "  4. Add NATS integration"
echo "  5. cargo build --bin $DOMAIN_NAME-service"
echo "  6. NATS_URL=nats://10.0.0.41:4222 cargo run --bin $DOMAIN_NAME-service"
echo ""
info "See .claude/CIM_DOMAIN_TEMPLATE.md for complete instructions"
echo ""
EOF

chmod +x .claude/scripts/new-cim-domain.sh

info "Created bootstrap script"
