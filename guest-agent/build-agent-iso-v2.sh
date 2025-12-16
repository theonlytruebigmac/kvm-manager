#!/bin/bash
# Build KVM Manager Guest Agent ISO
# This creates an ISO image that can be mounted to VMs for easy agent installation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ISO_DIR="$SCRIPT_DIR/iso-build"
ISO_OUTPUT="$SCRIPT_DIR/kvmmanager-guest-agent.iso"

echo "=== Building KVM Manager Guest Agent ISO ==="
echo ""

# Clean previous build
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR"

# Build the agent in release mode
echo "Building guest agent..."
cd "$SCRIPT_DIR"
cargo build --release --bin kvmmanager-agent

# Copy all files needed for installation
echo "Copying installation files..."
cp target/release/kvmmanager-agent "$ISO_DIR/"
cp packaging/config.json "$ISO_DIR/"
cp packaging/kvmmanager-agent.service "$ISO_DIR/"
cp packaging/install-alpine.sh "$ISO_DIR/"
cp packaging/install-debian.sh "$ISO_DIR/"
cp packaging/install-rhel.sh "$ISO_DIR/"

# Make scripts executable
chmod +x "$ISO_DIR"/install-*.sh

# Create README for the ISO
cat > "$ISO_DIR/README.txt" << 'EOF'
================================================================================
                   KVM Manager Guest Agent Installation
================================================================================

This ISO contains the KVM Manager guest agent and installation scripts
for various Linux distributions.

INSTALLATION
------------

1. This ISO is mounted in your VM. Run the appropriate script:

   Debian/Ubuntu:    sudo bash /media/cdrom/install-debian.sh
   RHEL/Fedora:      sudo bash /media/cdrom/install-rhel.sh
   Alpine Linux:     sudo sh /media/cdrom/install-alpine.sh

2. The agent will be installed and started automatically.

3. Eject the ISO from the KVM Manager UI.

VERIFICATION
------------

Check that the agent is running:

   systemd-based:    systemctl status kvmmanager-agent
   Alpine Linux:     rc-service kvmmanager-agent status

WHAT THE AGENT DOES
-------------------

The guest agent enables enhanced VM management:
- Display actual OS information (type, version, hostname)
- Show network interfaces with IP addresses
- Report disk usage from inside the guest
- Execute commands remotely (optional, configurable)
- Graceful shutdown and reboot

SECURITY & CONFIGURATION
------------------------

Default configuration: /etc/kvmmanager-agent/config.json

Security features:
- Path-based file access control
- Optional command whitelist
- Timeout enforcement
- File size limits

View logs:
   systemd:  journalctl -u kvmmanager-agent -f
   Alpine:   tail -f /var/log/kvmmanager-agent/agent.log

================================================================================
More info: https://github.com/theonlytruebigmac/kvm-manager
================================================================================
EOF

# Check if genisoimage or mkisofs is available
if command -v genisoimage &> /dev/null; then
    ISO_CMD="genisoimage"
elif command -v mkisofs &> /dev/null; then
    ISO_CMD="mkisofs"
else
    echo "Error: Neither genisoimage nor mkisofs found. Please install one of them:"
    echo "  Debian/Ubuntu: sudo apt install genisoimage"
    echo "  RHEL/Fedora:   sudo dnf install genisoimage"
    exit 1
fi

# Build the ISO
echo "Creating ISO image..."
$ISO_CMD \
    -o "$ISO_OUTPUT" \
    -V "KVMMANAGER_AGENT" \
    -R -J \
    -input-charset utf-8 \
    "$ISO_DIR"

# Show results
echo ""
echo "✓ ISO created successfully!"
echo ""
echo "  Location: $ISO_OUTPUT"
echo "  Size: $(du -h "$ISO_OUTPUT" | cut -f1)"
echo ""
echo "The ISO is ready to be mounted to VMs for guest agent installation."
echo "Use the 'Mount Agent ISO' button in the VM details page."
