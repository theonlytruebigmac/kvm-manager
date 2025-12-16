#!/bin/sh
# Installation script for Alpine Linux

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

# Create OpenRC service script
cat > /etc/init.d/kvmmanager-agent << 'EOF'
#!/sbin/openrc-run

name="kvmmanager-agent"
description="KVM Manager Guest Agent"
command="/usr/bin/kvmmanager-agent"
command_background=true
pidfile="/run/${RC_SVCNAME}.pid"
output_log="/var/log/kvmmanager-agent/agent.log"
error_log="/var/log/kvmmanager-agent/agent.err"

depend() {
    need net
}
EOF

chmod +x /etc/init.d/kvmmanager-agent

# Enable and start service
rc-update add kvmmanager-agent default
rc-service kvmmanager-agent start

echo "✓ KVM Manager Guest Agent installed successfully!"
echo ""
echo "Status: rc-service kvmmanager-agent status"
echo "Logs:   tail -f /var/log/kvmmanager-agent/agent.log"
echo "Config: /etc/kvmmanager-agent/config.json"
