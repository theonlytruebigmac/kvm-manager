#!/bin/bash
# Installation script for Debian/Ubuntu systems

set -e

echo "Installing KVM Manager Guest Agent..."

# Create directories
mkdir -p /etc/kvmmanager-agent
mkdir -p /var/log/kvmmanager-agent

# Copy binary
cp kvmmanager-agent /usr/bin/kvmmanager-agent
chmod +x /usr/bin/kvmmanager-agent

# Copy configuration
if [ ! -f /etc/kvmmanager-agent/config.json ]; then
    cp config.json /etc/kvmmanager-agent/config.json
    echo "Created default configuration at /etc/kvmmanager-agent/config.json"
fi

# Copy systemd service
cp kvmmanager-agent.service /etc/systemd/system/kvmmanager-agent.service

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable kvmmanager-agent.service
systemctl start kvmmanager-agent.service

echo "✓ KVM Manager Guest Agent installed successfully!"
echo ""
echo "Status: systemctl status kvmmanager-agent"
echo "Logs:   journalctl -u kvmmanager-agent -f"
echo "Config: /etc/kvmmanager-agent/config.json"
