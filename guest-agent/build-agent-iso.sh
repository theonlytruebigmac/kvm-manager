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

# Build the agent
echo "Building guest agent..."
cd "$SCRIPT_DIR"
cargo build --release --bin kvmmanager-agent

# Copy agent binary
echo "Copying agent binary..."
mkdir -p "$ISO_DIR/bin"
cp target/release/kvmmanager-agent "$ISO_DIR/bin/"

# Create install scripts for different distros
echo "Creating installation scripts..."

# Alpine Linux installer
cat > "$ISO_DIR/install-alpine.sh" << 'EOF'
#!/bin/sh
# KVM Manager Guest Agent installer for Alpine Linux
set -e

if [ "$(id -u)" -ne 0 ]; then
    echo "Error: Must run as root. Use: sudo sh /mnt/cdrom/install-alpine.sh"
    exit 1
fi

echo "Installing KVM Manager Guest Agent for Alpine Linux..."

# Install dependencies
apk add --no-cache openrc

# Copy binary
cp /mnt/cdrom/bin/kvmmanager-agent /usr/local/bin/
chmod +x /usr/local/bin/kvmmanager-agent

# Create log directory
mkdir -p /var/log/kvmmanager-agent

# Create OpenRC service
cat > /etc/init.d/kvmmanager-agent << 'EOFSERVICE'
#!/sbin/openrc-run

name="KVM Manager Guest Agent"
description="Guest agent for KVM Manager"
command="/usr/local/bin/kvmmanager-agent"
command_args="--device /dev/vport0p1"
command_background=true
pidfile="/run/kvmmanager-agent.pid"
output_log="/var/log/kvmmanager-agent/agent.log"
error_log="/var/log/kvmmanager-agent/agent.log"

depend() {
    need net
    after firewall
}

start_pre() {
    checkpath --directory --mode 0755 /var/log/kvmmanager-agent
    if [ ! -c /dev/vport0p1 ]; then
        eerror "Virtio-serial device /dev/vport0p1 not found"
        return 1
    fi
}
EOFSERVICE

chmod +x /etc/init.d/kvmmanager-agent

# Enable and start
rc-update add kvmmanager-agent default
rc-service kvmmanager-agent start

echo "✓ KVM Manager Guest Agent installed and started"
echo "  Check status: rc-service kvmmanager-agent status"
echo "  View logs: tail -f /var/log/kvmmanager-agent/agent.log"
EOF

# Debian/Ubuntu installer
cat > "$ISO_DIR/install-debian.sh" << 'EOF'
#!/bin/bash
# KVM Manager Guest Agent installer for Debian/Ubuntu
set -e

if [ "$(id -u)" -ne 0 ]; then
    echo "Error: Must run as root. Use: sudo bash /media/cdrom/install-debian.sh"
    exit 1
fi

echo "Installing KVM Manager Guest Agent for Debian/Ubuntu..."

# Copy binary
cp /media/cdrom/bin/kvmmanager-agent /usr/local/bin/
chmod +x /usr/local/bin/kvmmanager-agent

# Create systemd service
cat > /etc/systemd/system/kvmmanager-agent.service << 'EOFSERVICE'
[Unit]
Description=KVM Manager Guest Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/kvmmanager-agent --device /dev/vport0p1
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOFSERVICE

# Enable and start
systemctl daemon-reload
systemctl enable kvmmanager-agent
systemctl start kvmmanager-agent

echo "✓ KVM Manager Guest Agent installed and started"
echo "  Check status: systemctl status kvmmanager-agent"
echo "  View logs: journalctl -u kvmmanager-agent -f"
EOF

# RHEL/Fedora/Rocky installer
cat > "$ISO_DIR/install-rhel.sh" << 'EOF'
#!/bin/bash
# KVM Manager Guest Agent installer for RHEL/Fedora/Rocky
set -e

if [ "$(id -u)" -ne 0 ]; then
    echo "Error: Must run as root. Use: sudo bash /media/cdrom/install-rhel.sh"
    exit 1
fi

echo "Installing KVM Manager Guest Agent for RHEL/Fedora/Rocky..."

# Copy binary
cp /media/cdrom/bin/kvmmanager-agent /usr/local/bin/
chmod +x /usr/local/bin/kvmmanager-agent

# Create systemd service
cat > /etc/systemd/system/kvmmanager-agent.service << 'EOFSERVICE'
[Unit]
Description=KVM Manager Guest Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/kvmmanager-agent --device /dev/vport0p1
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOFSERVICE

# Enable and start
systemctl daemon-reload
systemctl enable kvmmanager-agent
systemctl start kvmmanager-agent

echo "✓ KVM Manager Guest Agent installed and started"
echo "  Check status: systemctl status kvmmanager-agent"
echo "  View logs: journalctl -u kvmmanager-agent -f"
EOF

# Create README
cat > "$ISO_DIR/README.txt" << 'EOF'
KVM Manager Guest Agent Installation ISO
=========================================

This ISO contains the KVM Manager Guest Agent and installation scripts
for various Linux distributions.

Installation Instructions:
--------------------------

1. This ISO should be automatically mounted by your VM as /dev/cdrom
2. Mount it if not already mounted:
   - Alpine: mount /dev/cdrom /mnt/cdrom
   - Others: mount /dev/cdrom /media/cdrom

3. Run the appropriate installer for your distribution:

   Alpine Linux:
   -------------
   cd /mnt/cdrom
   sudo sh install-alpine.sh

   Debian/Ubuntu:
   --------------
   cd /media/cdrom
   sudo bash install-debian.sh

   RHEL/Fedora/Rocky:
   ------------------
   cd /media/cdrom
   sudo bash install-rhel.sh

4. After installation, the agent will start automatically

5. You can now eject the ISO from KVM Manager

Verification:
-------------
After installation, verify the agent is running:

Alpine:       rc-service kvmmanager-agent status
Others:       systemctl status kvmmanager-agent

The agent communicates through the virtio-serial device (/dev/vport0p1).
If you see errors about this device, ensure your VM has a virtio-serial
channel configured in KVM Manager.

For more information, visit: https://github.com/yourusername/kvm-manager
EOF

# Create autorun for Windows (future)
cat > "$ISO_DIR/autorun.inf" << 'EOF'
[autorun]
open=setup.exe
icon=setup.exe,0
label=KVM Manager Guest Agent
EOF

# Make scripts executable
chmod +x "$ISO_DIR"/*.sh

# Create the ISO
echo "Creating ISO image..."
if command -v genisoimage &> /dev/null; then
    genisoimage -o "$ISO_OUTPUT" \
        -V "KVMMANAGER_AGENT" \
        -r -J \
        -input-charset utf-8 \
        "$ISO_DIR"
elif command -v mkisofs &> /dev/null; then
    mkisofs -o "$ISO_OUTPUT" \
        -V "KVMMANAGER_AGENT" \
        -r -J \
        -input-charset utf-8 \
        "$ISO_DIR"
else
    echo "Error: Neither genisoimage nor mkisofs found"
    echo "Install with: sudo apt-get install genisoimage"
    exit 1
fi

# Cleanup
rm -rf "$ISO_DIR"

# Show results
ISO_SIZE=$(du -h "$ISO_OUTPUT" | cut -f1)
echo ""
echo "✓ ISO created successfully!"
echo "  Location: $ISO_OUTPUT"
echo "  Size: $ISO_SIZE"
echo ""
echo "Next steps:"
echo "  1. Copy ISO to libvirt images: sudo cp $ISO_OUTPUT /var/lib/libvirt/images/"
echo "  2. Mount to VM from KVM Manager UI"
echo "  3. Install agent from within the VM"
echo ""
