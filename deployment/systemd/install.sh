#!/usr/bin/env bash
# Installation script for CIM Person Domain Service (systemd)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    error "Please run as root (use sudo)"
    exit 1
fi

info "Installing CIM Person Domain Service"

# Build the service binary
info "Building person-service binary..."
cd "$(dirname "$0")/../.."
cargo build --release --bin person-service

# Create user and group
if ! id -u cim-person >/dev/null 2>&1; then
    info "Creating cim-person user..."
    useradd --system --no-create-home --shell /usr/sbin/nologin cim-person
else
    warn "User cim-person already exists"
fi

# Create directories
info "Creating service directories..."
mkdir -p /var/lib/cim-person
mkdir -p /etc/cim-person
mkdir -p /var/log/cim-person

# Set permissions
chown -R cim-person:cim-person /var/lib/cim-person
chown -R cim-person:cim-person /var/log/cim-person
chmod 755 /var/lib/cim-person
chmod 755 /etc/cim-person

# Install binary
info "Installing person-service binary..."
install -m 755 target/release/person-service /usr/local/bin/person-service

# Install systemd service
info "Installing systemd service..."
install -m 644 deployment/systemd/cim-domain-person.service /etc/systemd/system/

# Install environment file if it doesn't exist
if [ ! -f /etc/cim-person/environment ]; then
    info "Installing environment configuration..."
    install -m 640 deployment/systemd/environment.example /etc/cim-person/environment
    chown root:cim-person /etc/cim-person/environment
    warn "Please edit /etc/cim-person/environment to configure NATS connection"
else
    warn "Environment file already exists at /etc/cim-person/environment"
fi

# Reload systemd
info "Reloading systemd..."
systemctl daemon-reload

# Enable service
info "Enabling cim-domain-person service..."
systemctl enable cim-domain-person.service

info "Installation complete!"
echo ""
info "Next steps:"
echo "  1. Edit /etc/cim-person/environment to configure NATS URL"
echo "  2. Start the service: sudo systemctl start cim-domain-person"
echo "  3. Check status: sudo systemctl status cim-domain-person"
echo "  4. View logs: sudo journalctl -u cim-domain-person -f"
