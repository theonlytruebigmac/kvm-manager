# Guest Agent Testing Guide

**Date**: December 9, 2025
**Status**: Ready for Testing

---

## Prerequisites

- KVM Manager application running
- Access to libvirt (qemu:///system)
- Test VM (Linux - Ubuntu, Debian, or similar)
- Root/sudo access for installation

---

## Step 1: Build the Guest Agent

### Build Release Binary

```bash
cd guest-agent
cargo build --release --bin kvmmanager-agent

# Verify binary was created
ls -lh target/release/kvmmanager-agent
```

The binary should be around 2-3 MB after optimization.

### Strip Binary (Optional, for smaller size)

```bash
strip target/release/kvmmanager-agent
ls -lh target/release/kvmmanager-agent
```

---

## Step 2: Create/Configure Test VM

### Option A: Create New Test VM

You can create a test VM through the KVM Manager UI or via virsh:

```bash
# Create a simple Alpine Linux VM (lightweight for testing)
virt-install \
  --name test-guest-agent \
  --memory 1024 \
  --vcpus 1 \
  --disk size=10 \
  --cdrom /path/to/alpine-linux.iso \
  --network network=default \
  --graphics vnc \
  --noautoconsole
```

### Option B: Add virtio-serial to Existing VM

For an existing VM, you need to add a virtio-serial channel.

#### 1. Shut down the VM

```bash
virsh shutdown YOUR_VM_NAME
```

#### 2. Edit VM XML

```bash
virsh edit YOUR_VM_NAME
```

#### 3. Add virtio-serial controller and channel

Add this inside the `<devices>` section:

```xml
<devices>
  <!-- Existing devices -->

  <!-- Add virtio-serial controller -->
  <controller type='virtio-serial' index='0'>
    <address type='pci' domain='0x0000' bus='0x00' slot='0x05' function='0x0'/>
  </controller>

  <!-- Add guest agent channel -->
  <channel type='unix'>
    <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/YOUR_VM_NAME.org.kvmmanager.agent.0'/>
    <target type='virtio' name='org.kvmmanager.agent.0'/>
    <address type='virtio-serial' controller='0' bus='0' port='1'/>
  </channel>
</devices>
```

**Important**: Replace `YOUR_VM_NAME` with your actual VM name in the `source path` attribute!

#### 4. Start the VM

```bash
virsh start YOUR_VM_NAME
```

#### 5. Verify virtio-serial device exists

Inside the VM, check:

```bash
ls -l /dev/virtio-ports/
# Should show: org.kvmmanager.agent.0
```

If the device doesn't exist, check the VM XML and ensure the virtio-serial controller and channel are properly configured.

---

## Step 3: Install Guest Agent in VM

### Copy Binary to VM

From your host machine:

```bash
# Copy the agent binary to the VM
scp guest-agent/target/release/kvmmanager-agent user@VM_IP:/tmp/

# SSH into the VM
ssh user@VM_IP
```

### Install Agent

Inside the VM:

```bash
# Move binary to system location
sudo mv /tmp/kvmmanager-agent /usr/local/bin/
sudo chmod +x /usr/local/bin/kvmmanager-agent

# Create config directory
sudo mkdir -p /etc/kvmmanager-agent

# Create default config file
sudo tee /etc/kvmmanager-agent/config.json <<EOF
{
  "version": "1.0.0",
  "log_level": "info",
  "log_file": "/var/log/kvmmanager-agent.log",
  "channel_name": "org.kvmmanager.agent.0",
  "max_message_size_kb": 64,
  "connection_retry_seconds": 5,
  "request_timeout_seconds": 30,
  "max_concurrent_requests": 10,
  "security": {
    "allowed_read_paths": ["/tmp", "/var/log", "/etc/hostname", "/etc/os-release", "/proc/cpuinfo", "/proc/meminfo"],
    "allowed_write_paths": ["/tmp/kvmmanager"],
    "command_whitelist": null,
    "max_file_size_mb": 10,
    "max_command_timeout_seconds": 300
  }
}
EOF

# Create systemd service
sudo tee /etc/systemd/system/kvmmanager-agent.service <<EOF
[Unit]
Description=KVM Manager Guest Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/kvmmanager-agent --foreground
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable kvmmanager-agent
sudo systemctl start kvmmanager-agent

# Check status
sudo systemctl status kvmmanager-agent
```

### Verify Agent is Running

```bash
# Check if agent is running
sudo systemctl status kvmmanager-agent

# Check logs
sudo journalctl -u kvmmanager-agent -n 20

# Should see:
# "KVM Manager Guest Agent v0.1.0 starting"
# "Connected to virtio-serial device: /dev/virtio-ports/org.kvmmanager.agent.0"
# "Guest agent ready, listening for requests"
```

---

## Step 4: Test from Host (Command Line)

Before testing through the UI, verify the agent responds via command line.

### Find the Socket Path

```bash
# List channel sockets
ls -l /var/lib/libvirt/qemu/channel/target/

# Look for YOUR_VM_NAME.org.kvmmanager.agent.0
```

### Test with socat

```bash
# Install socat if not available
sudo apt install socat

# Set VM name
VM_NAME="your-vm-name"
SOCKET="/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0"

# Test ping
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:$SOCKET
# Expected: {"jsonrpc":"2.0","result":{"pong":true,"timestamp":...},"id":1}

# Get agent info
echo '{"jsonrpc":"2.0","method":"get_agent_info","id":2}' | socat - UNIX-CONNECT:$SOCKET
# Expected: {"jsonrpc":"2.0","result":{"version":"0.1.0","protocol_version":"1.0.0","platform":"linux",...},"id":2}

# Get system info
echo '{"jsonrpc":"2.0","method":"get_system_info","id":3}' | socat - UNIX-CONNECT:$SOCKET
# Expected: {"jsonrpc":"2.0","result":{"os_type":"linux","os_name":"Ubuntu","hostname":"..."},"id":3}
```

If these work, the agent is functioning correctly!

---

## Step 5: Test Through KVM Manager UI

### 5.1 Check Agent Status

1. Open KVM Manager
2. Navigate to VM List
3. Click on your test VM to open VM Details
4. Scroll down to the "Guest Information" section

**Expected**:
- Should show "Guest Information" with green icon
- Should display "Agent v0.1.0" badge
- If agent not available, shows installation instructions

### 5.2 Verify System Information

In the Guest Information section, verify:

- ✅ **Operating System**: Shows correct OS name and version
- ✅ **Kernel**: Shows kernel version
- ✅ **Hostname**: Shows actual hostname from inside VM
- ✅ **CPU**: Shows correct vCPU count and architecture
- ✅ **Total Memory**: Shows memory from guest perspective
- ✅ **Uptime**: Shows VM uptime

### 5.3 Verify Network Information

Check the "Network Interfaces" card:

- ✅ Shows all network interfaces (eth0, lo, etc.)
- ✅ Displays MAC addresses
- ✅ Shows IPv4 addresses with correct IPs
- ✅ Shows IPv6 addresses (if configured)
- ✅ Interface state (up/down)

### 5.4 Verify Disk Usage

Check the "Disk Usage" card:

- ✅ Shows all mounted filesystems
- ✅ Displays mount points (/, /home, etc.)
- ✅ Shows device names and filesystem types
- ✅ Shows used/total space in GB
- ✅ Shows usage percentage
- ✅ Progress bar color-coded (green < 75%, yellow < 90%, red ≥ 90%)

### 5.5 Test Auto-Refresh

The guest information should auto-refresh:

- System info: Every 5 seconds
- Network info: Every 10 seconds
- Disk usage: Every 30 seconds

Change something in the guest (e.g., create a large file) and verify the UI updates automatically.

---

## Step 6: Test Advanced Features (Optional)

### Test Command Execution

You can test command execution via the browser console:

1. Open browser DevTools (F12)
2. Go to Console tab
3. Run:

```javascript
// Check if API is available
window.__TAURI__

// Execute a simple command
await window.__TAURI__.invoke('execute_guest_command', {
  vmName: 'your-vm-name',
  command: 'echo',
  args: ['Hello from guest!'],
  timeoutSeconds: 10
})
// Expected: { exitCode: 0, stdout: "Hello from guest!\n", stderr: "", executionTimeMs: ... }

// List directory
await window.__TAURI__.invoke('execute_guest_command', {
  vmName: 'your-vm-name',
  command: 'ls',
  args: ['-la', '/tmp'],
  timeoutSeconds: 10
})
```

### Test File Operations

```javascript
// Read a file
await window.__TAURI__.invoke('read_guest_file', {
  vmName: 'your-vm-name',
  path: '/etc/hostname'
})
// Expected: Your VM's hostname

// Write a file
await window.__TAURI__.invoke('write_guest_file', {
  vmName: 'your-vm-name',
  path: '/tmp/test-from-host.txt',
  content: 'Hello from KVM Manager!',
  createDirs: true
})
// Expected: success (no error)

// Verify in VM
await window.__TAURI__.invoke('read_guest_file', {
  vmName: 'your-vm-name',
  path: '/tmp/test-from-host.txt'
})
// Expected: "Hello from KVM Manager!"
```

### Test Power Management

```javascript
// Graceful reboot (careful!)
await window.__TAURI__.invoke('guest_agent_reboot', {
  vmName: 'your-vm-name',
  force: false
})
```

---

## Troubleshooting

### Agent Not Detected

**Symptom**: UI shows "Guest agent not available"

**Checks**:

1. **Verify VM is running**:
   ```bash
   virsh list --all
   ```

2. **Check virtio-serial device in guest**:
   ```bash
   ssh user@VM_IP
   ls -l /dev/virtio-ports/org.kvmmanager.agent.0
   ```
   If missing, VM XML is not configured correctly.

3. **Check agent service in guest**:
   ```bash
   sudo systemctl status kvmmanager-agent
   sudo journalctl -u kvmmanager-agent -n 50
   ```

4. **Check socket on host**:
   ```bash
   ls -l /var/lib/libvirt/qemu/channel/target/YOUR_VM_NAME.org.kvmmanager.agent.0
   ```
   Socket should exist and be a Unix socket.

5. **Test with socat**:
   ```bash
   echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:/var/lib/libvirt/qemu/channel/target/YOUR_VM_NAME.org.kvmmanager.agent.0
   ```

### Agent Crashes on Startup

**Check logs**:
```bash
sudo journalctl -u kvmmanager-agent -n 100
```

**Common issues**:
- virtio-serial device not found → Fix VM XML
- Permission denied → Agent might need elevated permissions
- Config file parse error → Check `/etc/kvmmanager-agent/config.json` syntax

### Connection Timeout Errors

**Symptom**: UI shows connection errors

**Checks**:

1. **Verify agent is responsive**:
   ```bash
   # In guest
   sudo systemctl status kvmmanager-agent
   ```

2. **Check firewall** (shouldn't affect virtio-serial, but check anyway):
   ```bash
   sudo ufw status
   ```

3. **Restart agent**:
   ```bash
   sudo systemctl restart kvmmanager-agent
   ```

4. **Check KVM Manager backend logs**:
   Look for errors in the terminal where you ran `npm run tauri dev`

### Permission Denied Errors

**Symptom**: File operations or commands fail with permission errors

**Solution**: Update agent configuration to allow the paths:

```bash
sudo nano /etc/kvmmanager-agent/config.json
```

Add paths to `allowed_read_paths` or `allowed_write_paths` in the `security` section.

Then restart:
```bash
sudo systemctl restart kvmmanager-agent
```

---

## Success Criteria

✅ **Agent Status**: Shows as available in UI
✅ **System Info**: Displays correct OS, hostname, CPU, memory, uptime
✅ **Network Info**: Shows all interfaces with correct IPs
✅ **Disk Usage**: Shows all filesystems with usage data
✅ **Auto-Refresh**: Data updates automatically without page refresh
✅ **Command Execution**: Can execute commands via console (optional)
✅ **File Operations**: Can read/write files via console (optional)

---

## Next Steps After Successful Testing

1. **Documentation**: Update main README with guest agent features
2. **Packaging**: Create .deb and .rpm packages for easy installation
3. **Windows Agent**: Port to Windows (Phase 4 Week 7-8)
4. **Enhanced Features**:
   - UI for command execution
   - File browser
   - Process management
   - Service management

---

## Quick Test Script

Save this as `test-agent.sh` for quick testing:

```bash
#!/bin/bash
set -e

VM_NAME="${1:-test-guest-agent}"
SOCKET="/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0"

echo "Testing guest agent for VM: $VM_NAME"
echo "Socket: $SOCKET"
echo ""

if [ ! -S "$SOCKET" ]; then
    echo "❌ Socket not found. Is VM running and configured?"
    exit 1
fi

echo "✅ Socket exists"
echo ""

echo "Testing ping..."
RESULT=$(echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | grep -q '"pong":true'; then
    echo "✅ Ping successful"
else
    echo "❌ Ping failed: $RESULT"
    exit 1
fi
echo ""

echo "Testing get_agent_info..."
RESULT=$(echo '{"jsonrpc":"2.0","method":"get_agent_info","id":2}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | grep -q '"version"'; then
    echo "✅ Agent info retrieved"
    echo "$RESULT" | jq .result.version
else
    echo "❌ Failed: $RESULT"
    exit 1
fi
echo ""

echo "Testing get_system_info..."
RESULT=$(echo '{"jsonrpc":"2.0","method":"get_system_info","id":3}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | grep -q '"hostname"'; then
    echo "✅ System info retrieved"
    echo "$RESULT" | jq '.result | {os_name, hostname, cpu_count, total_memory_kb}'
else
    echo "❌ Failed: $RESULT"
    exit 1
fi
echo ""

echo "🎉 All tests passed! Guest agent is working correctly."
```

Usage:
```bash
chmod +x test-agent.sh
./test-agent.sh YOUR_VM_NAME
```

---

**Ready to test!** Follow the steps above and report any issues. 🚀
