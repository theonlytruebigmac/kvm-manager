# Guest Agent Installation Guide

## Overview

The KVM Manager Guest Agent provides enhanced VM management capabilities by running inside your virtual machines. It enables:

- **Real Guest OS Information**: Display actual OS type, version, hostname, and kernel information
- **Network Details**: Show all network interfaces with IP addresses
- **Disk Usage**: Report filesystem usage from inside the guest
- **Remote Command Execution**: Execute commands inside VMs (optional, with security controls)
- **Graceful Shutdown/Reboot**: Power management via the agent

## Quick Start

### 1. Build the Guest Agent ISO

From the project root:

```bash
cd guest-agent
./build-agent-iso-v2.sh
```

This creates `kvmmanager-guest-agent.iso` (~2 MB) containing:
- The guest agent binary
- Installation scripts for various Linux distributions
- Configuration template
- systemd/OpenRC service files

### 2. Deploy the ISO

Copy the ISO to the libvirt images directory:

```bash
sudo cp kvmmanager-guest-agent.iso /var/lib/libvirt/images/
sudo chmod 644 /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

Alternatively, update `src-tauri/src/commands/vm.rs` to point to your ISO location.

### 3. Install in a VM

1. **Mount the ISO** from the VM details page in KVM Manager
   - Click the "Guest Information" card
   - Click "Mount Agent ISO"

2. **Run the installer** inside the VM:

   ```bash
   # Debian/Ubuntu
   sudo bash /media/cdrom/install-debian.sh

   # RHEL/Fedora/Rocky/CentOS
   sudo bash /media/cdrom/install-rhel.sh

   # Alpine Linux
   sudo sh /media/cdrom/install-alpine.sh
   ```

3. **Verify the agent is running**:

   ```bash
   # systemd-based systems
   systemctl status kvmmanager-agent

   # Alpine Linux
   rc-service kvmmanager-agent status
   ```

4. **Eject the ISO** from KVM Manager

5. **Refresh** the VM details page - you should now see guest information!

## VM Configuration Requirements

For the guest agent to work, the VM must have a **virtio-serial channel** configured.

### Automatic Configuration (Recommended)

When creating VMs through KVM Manager, virtio-serial is automatically added.

### Manual Configuration

If you need to add it manually to an existing VM:

1. Shut down the VM
2. Edit the VM XML:
   ```bash
   virsh edit <vm-name>
   ```

3. Add this inside the `<devices>` section:
   ```xml
   <channel type='unix'>
     <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/<vm-name>.org.kvmmanager.agent.0'/>
     <target type='virtio' name='org.kvmmanager.agent.0'/>
   </channel>
   ```

4. Start the VM

The agent will communicate through `/dev/vport0p1` (or `/dev/virtio-ports/org.kvmmanager.agent.0`) in the guest.

## Configuration

### Agent Configuration File

Location: `/etc/kvmmanager-agent/config.json`

```json
{
  "allowed_read_paths": [
    "/etc",
    "/proc",
    "/sys",
    "/var/log",
    "/tmp"
  ],
  "allowed_write_paths": [
    "/tmp",
    "/var/tmp"
  ],
  "command_whitelist_enabled": false,
  "command_whitelist": [
    "ls", "cat", "grep", "find",
    "ps", "df", "free", "uptime"
  ],
  "max_file_size_bytes": 10485760,
  "command_timeout_seconds": 30,
  "log_level": "info"
}
```

### Security Settings

**File Access Control**:
- `allowed_read_paths`: Directories where the agent can read files
- `allowed_write_paths`: Directories where the agent can write files
- Path traversal attacks are automatically blocked

**Command Execution**:
- `command_whitelist_enabled`: Set to `true` to restrict allowed commands
- `command_whitelist`: Commands that can be executed when whitelist is enabled
- Commands are executed directly (no shell) to prevent injection attacks

**Resource Limits**:
- `max_file_size_bytes`: Maximum file size for read/write operations (default: 10 MB)
- `command_timeout_seconds`: Maximum execution time for commands (default: 30s)

### Restart After Config Changes

```bash
# systemd
sudo systemctl restart kvmmanager-agent

# Alpine
sudo rc-service kvmmanager-agent restart
```

## Monitoring and Logs

### Check Status

```bash
# systemd
systemctl status kvmmanager-agent

# Alpine
rc-service kvmmanager-agent status
```

### View Logs

```bash
# systemd
journalctl -u kvmmanager-agent -f

# Alpine
tail -f /var/log/kvmmanager-agent/agent.log
```

### Common Log Messages

- `Connected to virtio-serial device` - Agent successfully connected
- `Connection lost, retrying...` - Temporary disconnect (normal during VM operations)
- `Virtio-serial device not found` - VM missing virtio-serial channel configuration

## Troubleshooting

### Agent Not Connecting

**Symptom**: "Guest agent not available" in KVM Manager UI

**Checks**:

1. **Is the agent running?**
   ```bash
   systemctl status kvmmanager-agent  # or rc-service
   ```

2. **Does virtio-serial device exist?**
   ```bash
   ls -la /dev/vport0p1
   # or
   ls -la /dev/virtio-ports/org.kvmmanager.agent.0
   ```
   If not found, the VM is missing the virtio-serial channel.

3. **Check VM XML** on the host:
   ```bash
   virsh dumpxml <vm-name> | grep -A5 "org.kvmmanager.agent"
   ```

4. **Check agent logs**:
   ```bash
   journalctl -u kvmmanager-agent -n 50
   ```

### Agent Keeps Restarting

Check for permission issues or config errors:

```bash
journalctl -u kvmmanager-agent -f
```

Common causes:
- Invalid JSON in `/etc/kvmmanager-agent/config.json`
- Missing virtio-serial device
- Permission issues (agent runs as root by default)

### Commands Not Executing

1. **Check if command whitelist is enabled**:
   ```json
   "command_whitelist_enabled": false
   ```

2. **If whitelist is enabled**, add your command:
   ```json
   "command_whitelist": ["ls", "cat", "your-command"]
   ```

3. **Restart the agent** after config changes

## Building from Source

### Prerequisites

- Rust 1.70 or newer
- Standard build tools (gcc, make)

### Build Steps

```bash
cd guest-agent
cargo build --release
```

Binary location: `target/release/kvmmanager-agent`

### Manual Installation

```bash
# Copy binary
sudo cp target/release/kvmmanager-agent /usr/bin/

# Create config directory
sudo mkdir -p /etc/kvmmanager-agent

# Copy config
sudo cp packaging/config.json /etc/kvmmanager-agent/

# Install systemd service
sudo cp packaging/kvmmanager-agent.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now kvmmanager-agent
```

## Distribution Packages

### Debian/Ubuntu (.deb)

Coming soon - will support `apt install ./kvmmanager-agent.deb`

### RHEL/Fedora (.rpm)

Coming soon - will support `dnf install ./kvmmanager-agent.rpm`

### Alpine (.apk)

Coming soon - will support `apk add ./kvmmanager-agent.apk`

## Supported Platforms

**Currently Supported**:
- ✅ Linux (any distribution with systemd or OpenRC)
- ✅ x86_64 architecture

**Tested On**:
- Ubuntu 22.04, 24.04
- Debian 11, 12
- Fedora 39, 40
- RHEL 9, Rocky Linux 9
- Alpine Linux 3.18, 3.19

**Planned**:
- 🔄 Windows 10/11 (guest agent in development)
- 🔄 ARM64 architecture

## Protocol Details

The agent uses JSON-RPC 2.0 over virtio-serial for communication.

### Available Methods

- `ping` - Connectivity check
- `get_agent_info` - Agent version and capabilities
- `get_system_info` - OS information, CPU, memory, uptime
- `get_network_info` - Network interfaces with IPs
- `get_disk_usage` - Filesystem usage
- `exec_command` - Execute command (with security controls)
- `file_read` - Read file contents
- `file_write` - Write file contents
- `shutdown` - Graceful shutdown
- `reboot` - Graceful reboot

See `guest-agent/PROTOCOL.md` for complete protocol specification.

## Security Considerations

### Agent Runs as Root

The agent requires root privileges to:
- Read system information
- Execute commands
- Perform shutdown/reboot

### Mitigation

- Path restrictions prevent directory traversal
- Command whitelist limits execution scope
- No shell expansion (prevents injection)
- Timeouts prevent resource exhaustion
- File size limits prevent DoS

### Network Isolation

The agent uses virtio-serial (not network), so:
- No network exposure
- No firewall configuration needed
- No authentication required (trust model: host controls guest)

### Recommendations

1. **Enable command whitelist** if using command execution in production
2. **Restrict write paths** to only necessary directories
3. **Monitor agent logs** for suspicious activity
4. **Keep agent updated** for security patches

## Contributing

See the main project repository for contribution guidelines:
https://github.com/theonlytruebigmac/kvm-manager

## License

See LICENSE file in the project root.
