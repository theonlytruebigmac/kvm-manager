# KVM Manager Guest Agent

The KVM Manager Guest Agent runs inside Linux (and Windows, planned) guest VMs to enable deep integration with the host. It communicates via virtio-serial using JSON-RPC 2.0 protocol, requiring no network configuration.

## Features

- ✅ **System Information**: OS details, hostname, network interfaces, disk usage
- ✅ **Command Execution**: Run commands inside guest with security controls
- ✅ **File Operations**: Read/write files with path restrictions
- ✅ **Power Management**: Graceful shutdown and reboot
- ✅ **Zero Network Dependency**: Works without guest network via virtio-serial
- ✅ **Cross-Platform**: Linux (implemented), Windows (planned)

## Architecture

```
┌─────────────────────────────────────────┐
│          KVM Manager Host               │
│  ┌──────────────────────────────────┐   │
│  │  Backend (Rust/Tauri)            │   │
│  │  GuestAgentService               │   │
│  └────────────┬─────────────────────┘   │
│               │ Unix Socket              │
│  ┌────────────▼─────────────────────┐   │
│  │  libvirt virtio-serial channel   │   │
│  │  /var/lib/libvirt/qemu/channel/  │   │
│  └────────────┬─────────────────────┘   │
└───────────────┼─────────────────────────┘
                │ virtio-serial
┌───────────────▼─────────────────────────┐
│          Guest VM (Linux)               │
│  ┌────────────┴─────────────────────┐   │
│  │  /dev/virtio-ports/              │   │
│  │  org.kvmmanager.agent.0          │   │
│  └────────────┬─────────────────────┘   │
│               │                          │
│  ┌────────────▼─────────────────────┐   │
│  │  kvmmanager-agent (daemon)       │   │
│  │  - JSON-RPC 2.0 handler          │   │
│  │  - System info collection        │   │
│  │  - Command execution             │   │
│  │  - File operations               │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## Building

### Prerequisites

- Rust 1.70+ toolchain
- Linux build environment

### Build Release Binary

```bash
cd guest-agent
cargo build --release --bin kvmmanager-agent

# Binary location
ls -lh target/release/kvmmanager-agent
```

### Build for Minimal Size

The workspace is configured for small binaries:

```bash
cargo build --release --bin kvmmanager-agent
strip target/release/kvmmanager-agent

# Expected size: ~2-3 MB (optimized for embedded deployment)
```

## Installation

### Manual Installation

1. **Copy binary to guest VM**:
```bash
scp target/release/kvmmanager-agent user@guest-vm:/tmp/
```

2. **Install on guest**:
```bash
ssh user@guest-vm
sudo mv /tmp/kvmmanager-agent /usr/local/bin/
sudo chmod +x /usr/local/bin/kvmmanager-agent
```

3. **Create config directory**:
```bash
sudo mkdir -p /etc/kvmmanager-agent
```

4. **Generate default config**:
```bash
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
    "allowed_read_paths": ["/tmp", "/var/log", "/etc/hostname", "/etc/os-release"],
    "allowed_write_paths": ["/tmp/kvmmanager"],
    "command_whitelist": null,
    "max_file_size_mb": 10,
    "max_command_timeout_seconds": 300
  }
}
EOF
```

5. **Create systemd service**:
```bash
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
```

6. **Enable and start service**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable kvmmanager-agent
sudo systemctl start kvmmanager-agent
sudo systemctl status kvmmanager-agent
```

### Package Installation (Planned)

```bash
# Debian/Ubuntu
sudo dpkg -i kvmmanager-agent_0.1.0_amd64.deb

# RHEL/CentOS/Fedora
sudo rpm -i kvmmanager-agent-0.1.0.x86_64.rpm
```

## Configuration

**Config File**: `/etc/kvmmanager-agent/config.json`

### Security Settings

```json
{
  "security": {
    "allowed_read_paths": [
      "/tmp",
      "/var/log",
      "/etc/hostname",
      "/etc/os-release"
    ],
    "allowed_write_paths": [
      "/tmp/kvmmanager"
    ],
    "command_whitelist": null,
    "max_file_size_mb": 10,
    "max_command_timeout_seconds": 300
  }
}
```

**Command Whitelist**: To restrict commands, set:
```json
{
  "security": {
    "command_whitelist": ["ls", "cat", "df", "free"]
  }
}
```

## Testing

### Check Agent Status

```bash
sudo systemctl status kvmmanager-agent
sudo journalctl -u kvmmanager-agent -f
```

### Verify virtio-serial Device

```bash
ls -l /dev/virtio-ports/org.kvmmanager.agent.0
# Should show: crw-rw-rw- 1 root root
```

### Manual Test (Host Side)

On the KVM host, connect to the Unix socket:

```bash
# Find VM's channel socket
VM_NAME="your-vm-name"
SOCKET="/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0"

# Send ping request
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:$SOCKET
# Expected response: {"jsonrpc":"2.0","result":{"pong":true,"timestamp":...},"id":1}

# Get agent info
echo '{"jsonrpc":"2.0","method":"get_agent_info","id":2}' | socat - UNIX-CONNECT:$SOCKET
```

## VM Configuration

To enable guest agent, add virtio-serial channel to VM's libvirt XML:

```xml
<devices>
  <!-- Other devices -->

  <channel type='unix'>
    <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/VM_NAME.org.kvmmanager.agent.0'/>
    <target type='virtio' name='org.kvmmanager.agent.0'/>
    <address type='virtio-serial' controller='0' bus='0' port='1'/>
  </channel>

  <controller type='virtio-serial' index='0'>
    <address type='pci' domain='0x0000' bus='0x00' slot='0x05' function='0x0'/>
  </controller>
</devices>
```

Apply configuration:

```bash
virsh define vm.xml
virsh start VM_NAME
```

## Protocol

See [PROTOCOL.md](PROTOCOL.md) for complete JSON-RPC 2.0 protocol specification.

### Supported Methods

| Method | Description |
|--------|-------------|
| `ping` | Verify agent is responsive |
| `get_agent_info` | Get agent version and capabilities |
| `get_system_info` | OS details, hostname, CPU, memory |
| `get_network_info` | Network interfaces and IPs |
| `get_disk_usage` | Filesystem disk usage |
| `exec_command` | Execute command in guest |
| `file_read` | Read file contents |
| `file_write` | Write file contents |
| `shutdown` | Graceful shutdown |
| `reboot` | Graceful reboot |

## Troubleshooting

### Agent Not Starting

1. **Check virtio-serial device**:
```bash
ls -l /dev/virtio-ports/org.kvmmanager.agent.0
```
If missing, VM needs virtio-serial channel configured.

2. **Check logs**:
```bash
sudo journalctl -u kvmmanager-agent -n 50
```

3. **Test manually**:
```bash
/usr/local/bin/kvmmanager-agent --foreground --log-level debug
```

### Permission Denied Errors

Check security configuration in `/etc/kvmmanager-agent/config.json`. Ensure paths are in `allowed_read_paths` or `allowed_write_paths`.

### Connection Lost

Agent automatically reconnects if virtio-serial channel closes. Check VM is running and channel is configured.

## Development

### Run in Development Mode

```bash
cd guest-agent
cargo run --bin kvmmanager-agent -- --foreground --log-level debug --device /dev/virtio-ports/org.kvmmanager.agent.0
```

### Run Tests

```bash
cargo test
cargo test --all-features
```

### Code Structure

```
guest-agent/
├── agent-common/          # Shared protocol types
│   └── src/
│       ├── protocol.rs    # JSON-RPC types
│       └── error.rs       # Error types
├── agent-linux/           # Linux implementation
│   └── src/
│       ├── main.rs        # Entry point
│       ├── config.rs      # Configuration
│       ├── transport.rs   # virtio-serial I/O
│       └── handlers/      # Request handlers
│           ├── mod.rs
│           ├── system.rs  # System info
│           ├── exec.rs    # Command execution
│           ├── files.rs   # File operations
│           └── power.rs   # Shutdown/reboot
└── agent-windows/         # Windows implementation (planned)
```

## Roadmap

- [x] Protocol specification
- [x] Linux agent core
- [x] System information collection
- [x] Command execution
- [x] File operations
- [x] Power management
- [ ] Integration tests with real VM
- [ ] .deb package builder
- [ ] .rpm package builder
- [ ] Windows agent implementation
- [ ] Process management
- [ ] Service management
- [ ] Package management integration

## Security Considerations

1. **No Root Required**: Agent runs as regular user with systemd
2. **Path Restrictions**: File operations limited to configured paths
3. **Command Whitelist**: Optional restriction of allowed commands
4. **Timeout Enforcement**: All operations have maximum execution time
5. **Size Limits**: File operations capped at configured size
6. **No Shell Expansion**: Commands executed directly without shell

## License

MIT OR Apache-2.0

## Contributing

See main project [CONTRIBUTING.md](../CONTRIBUTING.md)
