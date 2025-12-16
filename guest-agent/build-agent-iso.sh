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
echo "Building guest agent for Linux..."
cd "$SCRIPT_DIR"
cargo build --release --bin kvmmanager-agent

# Check if we can build Windows version
WINDOWS_BINARY=""
if rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "Building guest agent for Windows..."
    cargo build --release --bin kvmmanager-agent --target x86_64-pc-windows-gnu
    WINDOWS_BINARY="target/x86_64-pc-windows-gnu/release/kvmmanager-agent.exe"
else
    echo "Note: Windows cross-compilation not available."
    echo "  To enable: rustup target add x86_64-pc-windows-gnu"
    echo "  And install: sudo apt-get install mingw-w64"
    echo ""
fi

# Copy agent binary and packaging files
echo "Copying agent binary and installation files..."
mkdir -p "$ISO_DIR"
cp target/release/kvmmanager-agent "$ISO_DIR/"
cp packaging/config.json "$ISO_DIR/"
cp packaging/kvmmanager-agent.service "$ISO_DIR/"
cp packaging/install-*.sh "$ISO_DIR/"
chmod +x "$ISO_DIR"/install-*.sh

# Copy Windows binary if available
if [ -n "$WINDOWS_BINARY" ] && [ -f "$WINDOWS_BINARY" ]; then
    echo "Including Windows binary in ISO..."
    cp "$WINDOWS_BINARY" "$ISO_DIR/kvmmanager-agent.exe"
fi

# Create README for the ISO
cat > "$ISO_DIR/README.txt" << 'EOF'
KVM Manager Guest Agent Installation

This ISO contains the KVM Manager guest agent and installation scripts
for various Linux distributions.

INSTALLATION:
1. Mount this ISO in your VM
2. Run the appropriate installation script for your distribution:

   Debian/Ubuntu:    sudo bash /media/cdrom/install-debian.sh
   RHEL/Fedora:      sudo bash /media/cdrom/install-rhel.sh
   Alpine Linux:     sudo sh /media/cdrom/install-alpine.sh

VERIFICATION:
After installation, verify the agent is running:
   systemctl status kvmmanager-agent     (systemd-based systems)
   rc-service kvmmanager-agent status    (Alpine Linux)

CONFIGURATION:
Edit /etc/kvmmanager-agent/config.json to customize:
- Allowed file read/write paths
- Command whitelist
- Timeout settings

LOGS:
View logs with:
   journalctl -u kvmmanager-agent -f    (systemd-based systems)
   tail -f /var/log/kvmmanager-agent/agent.log  (Alpine Linux)

For more information, visit:
https://github.com/theonlytruebigmac/kvm-manager

EOF

# Note: Actual installation scripts are already copied from packaging/
echo "Creating Alpine installer..."
cat > "$ISO_DIR/install-alpine-generated.sh" << 'EOF'
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
for various operating systems.

Installation Instructions:
--------------------------

LINUX:
------

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

WINDOWS:
--------

1. The ISO should auto-mount as drive D: (or similar)
2. Open PowerShell as Administrator
3. Run: powershell -ExecutionPolicy Bypass -File D:\install-windows.ps1

   Or for manual installation:
   - Copy kvmmanager-agent.exe to C:\Program Files\KVMManager Agent\
   - Copy config.json to the same directory
   - Run: sc create kvmmanager-agent binPath="C:\Program Files\KVMManager Agent\kvmmanager-agent.exe"
   - Run: sc start kvmmanager-agent

Verification:
-------------
After installation, verify the agent is running:

Linux (Alpine):      rc-service kvmmanager-agent status
Linux (Others):      systemctl status kvmmanager-agent
Windows:             sc query kvmmanager-agent

The agent communicates through the virtio-serial device.
- Linux: /dev/vport0p1
- Windows: \\.\COM3 (or similar virtio-serial port)

For more information, visit: https://github.com/yourusername/kvm-manager
EOF

# Create Windows installer script
cat > "$ISO_DIR/install-windows.ps1" << 'EOFPS'
# KVM Manager Guest Agent Installer for Windows
# Run as Administrator: powershell -ExecutionPolicy Bypass -File D:\install-windows.ps1

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

Write-Host "Installing KVM Manager Guest Agent for Windows..." -ForegroundColor Cyan
Write-Host ""

# Configuration
$InstallDir = "$env:ProgramFiles\KVMManager Agent"
$ServiceName = "kvmmanager-agent"
$IsoDrive = (Get-Volume | Where-Object { $_.FileSystemLabel -eq "KVMMANAGER_AGENT" }).DriveLetter
if (-not $IsoDrive) {
    $IsoDrive = "D"
}
$SourcePath = "${IsoDrive}:\"

try {
    # Create installation directory
    Write-Host "Creating installation directory..." -ForegroundColor Yellow
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Copy agent binary
    Write-Host "Copying agent binary..." -ForegroundColor Yellow
    Copy-Item "${SourcePath}kvmmanager-agent.exe" -Destination "$InstallDir\kvmmanager-agent.exe" -Force

    # Copy config file
    Write-Host "Copying configuration..." -ForegroundColor Yellow
    Copy-Item "${SourcePath}config.json" -Destination "$InstallDir\config.json" -Force

    # Stop existing service if running
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($existingService) {
        Write-Host "Stopping existing service..." -ForegroundColor Yellow
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
        Start-Sleep -Seconds 2
    }

    # Create Windows service
    Write-Host "Creating Windows service..." -ForegroundColor Yellow
    $binPath = "`"$InstallDir\kvmmanager-agent.exe`""
    sc.exe create $ServiceName binPath= $binPath start= auto displayname= "KVM Manager Guest Agent" | Out-Null
    sc.exe description $ServiceName "Guest agent for KVM Manager - provides VM information and management capabilities" | Out-Null

    # Start the service
    Write-Host "Starting service..." -ForegroundColor Yellow
    Start-Service -Name $ServiceName

    # Verify service is running
    $service = Get-Service -Name $ServiceName
    if ($service.Status -eq "Running") {
        Write-Host ""
        Write-Host "SUCCESS: KVM Manager Guest Agent installed and running!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Installation path: $InstallDir" -ForegroundColor Gray
        Write-Host "Service name: $ServiceName" -ForegroundColor Gray
        Write-Host ""
        Write-Host "Commands:" -ForegroundColor Cyan
        Write-Host "  Check status:  sc query $ServiceName" -ForegroundColor Gray
        Write-Host "  Stop service:  Stop-Service $ServiceName" -ForegroundColor Gray
        Write-Host "  Start service: Start-Service $ServiceName" -ForegroundColor Gray
    } else {
        throw "Service failed to start. Status: $($service.Status)"
    }
}
catch {
    Write-Host ""
    Write-Host "ERROR: Installation failed!" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host ""
    exit 1
}
EOFPS

# Create autorun for Windows
cat > "$ISO_DIR/autorun.inf" << 'EOF'
[autorun]
open=install-windows.ps1
icon=kvmmanager-agent.exe,0
label=KVM Manager Guest Agent
action=Install KVM Manager Guest Agent
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
