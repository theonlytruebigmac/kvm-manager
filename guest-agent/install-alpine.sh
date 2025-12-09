#!/bin/sh
# Installation script for KVM Manager Guest Agent on Alpine Linux

set -e

echo "=== KVM Manager Guest Agent - Alpine Linux Installer ==="
echo ""

# Check if running as root
if [ "$(id -u)" -ne 0 ]; then
    echo "Error: This script must be run as root"
    echo "Please run: sudo sh install-alpine.sh"
    exit 1
fi

# Install required packages
echo "Installing dependencies..."
apk add --no-cache openrc

# Create agent directory
echo "Creating agent directory..."
mkdir -p /usr/local/bin
mkdir -p /var/log/kvmmanager-agent

# Copy agent binary (assume it's in current directory)
if [ ! -f "./kvmmanager-agent" ]; then
    echo "Error: kvmmanager-agent binary not found in current directory"
    echo "Please copy the binary here first:"
    echo "  scp target/release/kvmmanager-agent root@alpine-ip:/root/"
    exit 1
fi

echo "Installing agent binary..."
cp ./kvmmanager-agent /usr/local/bin/
chmod +x /usr/local/bin/kvmmanager-agent

# Create OpenRC init script
echo "Creating OpenRC service..."
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
        eerror "Make sure the VM has a virtio-serial channel configured"
        return 1
    fi
}
EOFSERVICE

chmod +x /etc/init.d/kvmmanager-agent

# Enable and start service
echo "Enabling service..."
rc-update add kvmmanager-agent default

echo "Starting service..."
rc-service kvmmanager-agent start

# Wait a moment
sleep 2

# Check status
echo ""
echo "Checking service status..."
rc-service kvmmanager-agent status

echo ""
echo "=== Installation Complete ==="
echo ""
echo "The guest agent should now be running."
echo ""
echo "To check logs:"
echo "  tail -f /var/log/kvmmanager-agent/agent.log"
echo ""
echo "To check status:"
echo "  rc-service kvmmanager-agent status"
echo ""
echo "To test from the host:"
echo "  sudo socat - UNIX-CONNECT:/var/lib/libvirt/qemu/channel/target/alpine.org.kvmmanager.agent.0"
echo "  Then type: {\"jsonrpc\":\"2.0\",\"method\":\"ping\",\"id\":1}"
echo ""
