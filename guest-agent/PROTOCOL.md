# KVM Manager Guest Agent Protocol

**Version**: 1.0.0
**Transport**: virtio-serial
**Format**: JSON-RPC 2.0
**Last Updated**: December 9, 2025

---

## Overview

The KVM Manager Guest Agent enables bidirectional communication between the hypervisor host and guest VMs. The protocol uses JSON-RPC 2.0 over a virtio-serial channel for efficient, low-latency communication without network dependencies.

### Design Goals

- **Zero Network Dependency**: Works without guest network configuration
- **Lightweight**: Minimal resource footprint in guest
- **Secure**: Command execution with privilege separation
- **Cross-Platform**: Support Linux and Windows guests
- **Reliable**: Connection recovery and error handling
- **Extensible**: Easy to add new commands and capabilities

---

## Transport Layer

### Virtio-Serial Configuration

**Channel Name**: `org.kvmmanager.agent.0`

**Host Configuration** (libvirt domain XML):
```xml
<channel type='unix'>
  <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/DOMAIN_NAME.org.kvmmanager.agent.0'/>
  <target type='virtio' name='org.kvmmanager.agent.0'/>
  <address type='virtio-serial' controller='0' bus='0' port='1'/>
</channel>
```

**Guest Device Path**:
- Linux: `/dev/virtio-ports/org.kvmmanager.agent.0`
- Windows: `\\.\Global\org.kvmmanager.agent.0`

### Message Framing

Messages are newline-delimited JSON (`\n` separator). Each message is a complete JSON-RPC 2.0 request or response.

**Maximum Message Size**: 64 KB (configurable)

**Example Frame**:
```
{"jsonrpc":"2.0","method":"system_info","id":1}\n
{"jsonrpc":"2.0","result":{...},"id":1}\n
```

---

## JSON-RPC 2.0 Protocol

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": {
    // Method-specific parameters
  },
  "id": 1  // Unique request ID (integer or string)
}
```

### Success Response Format

```json
{
  "jsonrpc": "2.0",
  "result": {
    // Method-specific result
  },
  "id": 1  // Matches request ID
}
```

### Error Response Format

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Error description",
    "data": {
      // Optional additional error information
    }
  },
  "id": 1  // Matches request ID (null if parse error)
}
```

### Standard Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Missing required fields |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Parameter validation failed |
| -32603 | Internal error | Agent internal error |
| -32000 | Command failed | Command execution failed |
| -32001 | Permission denied | Insufficient privileges |
| -32002 | Not found | Resource not found |
| -32003 | Timeout | Operation timed out |

---

## Core Methods

### 1. ping

**Purpose**: Verify agent is alive and responsive

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "ping",
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "pong": true,
    "timestamp": 1702123456
  },
  "id": 1
}
```

---

### 2. get_agent_info

**Purpose**: Get agent version and capabilities

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_agent_info",
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "version": "1.0.0",
    "protocol_version": "1.0.0",
    "platform": "linux",
    "capabilities": [
      "system_info",
      "exec_command",
      "file_read",
      "file_write",
      "shutdown",
      "reboot"
    ]
  },
  "id": 2
}
```

---

### 3. get_system_info

**Purpose**: Retrieve guest OS information

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_system_info",
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "os_type": "linux",
    "os_name": "Ubuntu",
    "os_version": "22.04 LTS",
    "kernel_version": "5.15.0-89-generic",
    "hostname": "ubuntu-vm",
    "architecture": "x86_64",
    "cpu_count": 4,
    "total_memory_kb": 4194304,
    "uptime_seconds": 3600
  },
  "id": 3
}
```

---

### 4. get_network_info

**Purpose**: Get network interface information

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_network_info",
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "interfaces": [
      {
        "name": "eth0",
        "mac_address": "52:54:00:12:34:56",
        "ipv4_addresses": ["192.168.122.10/24"],
        "ipv6_addresses": ["fe80::5054:ff:fe12:3456/64"],
        "state": "up",
        "mtu": 1500
      },
      {
        "name": "lo",
        "mac_address": "00:00:00:00:00:00",
        "ipv4_addresses": ["127.0.0.1/8"],
        "ipv6_addresses": ["::1/128"],
        "state": "up",
        "mtu": 65536
      }
    ]
  },
  "id": 4
}
```

---

### 5. exec_command

**Purpose**: Execute a command in the guest

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "exec_command",
  "params": {
    "command": "ls",
    "args": ["-la", "/tmp"],
    "timeout_seconds": 30,
    "capture_output": true,
    "working_directory": "/tmp"
  },
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "exit_code": 0,
    "stdout": "total 12\ndrwxrwxrwt 10 root root 4096 Dec  9 10:30 .\n...",
    "stderr": "",
    "execution_time_ms": 45
  },
  "id": 5
}
```

**Security Notes**:
- Commands run with agent user privileges (typically non-root)
- Shell metacharacters are NOT interpreted (direct exec)
- Command whitelist can be configured
- Timeout enforced to prevent hanging

---

### 6. file_read

**Purpose**: Read file contents from guest

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "file_read",
  "params": {
    "path": "/etc/hostname",
    "encoding": "utf8"  // or "base64"
  },
  "id": 6
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": "ubuntu-vm\n",
    "size_bytes": 11,
    "encoding": "utf8"
  },
  "id": 6
}
```

**Security Notes**:
- Path must be within allowed directories (configurable)
- Default allowed paths: `/tmp`, `/var/log` (read-only)
- Symbolic links are NOT followed

---

### 7. file_write

**Purpose**: Write file contents to guest

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "file_write",
  "params": {
    "path": "/tmp/kvmmanager-data.txt",
    "content": "Hello from host!",
    "encoding": "utf8",
    "create_dirs": true,
    "permissions": "0644"
  },
  "id": 7
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "bytes_written": 16,
    "path": "/tmp/kvmmanager-data.txt"
  },
  "id": 7
}
```

**Security Notes**:
- Path must be within allowed directories (configurable)
- Default allowed paths: `/tmp` (read-write)
- File ownership: agent user

---

### 8. shutdown

**Purpose**: Gracefully shutdown the guest

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "shutdown",
  "params": {
    "timeout_seconds": 60,
    "force": false
  },
  "id": 8
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "initiated": true,
    "message": "Shutdown initiated"
  },
  "id": 8
}
```

**Note**: Agent responds immediately, then initiates shutdown

---

### 9. reboot

**Purpose**: Gracefully reboot the guest

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "reboot",
  "params": {
    "timeout_seconds": 60,
    "force": false
  },
  "id": 9
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "initiated": true,
    "message": "Reboot initiated"
  },
  "id": 9
}
```

---

### 10. get_disk_usage

**Purpose**: Get filesystem disk usage information

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_disk_usage",
  "id": 10
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "filesystems": [
      {
        "mount_point": "/",
        "device": "/dev/vda1",
        "fs_type": "ext4",
        "total_bytes": 21003583488,
        "used_bytes": 8403345408,
        "available_bytes": 11523674112,
        "used_percent": 40.0
      }
    ]
  },
  "id": 10
}
```

---

## Agent Lifecycle

### Initialization Sequence

1. **Agent Startup**
   - Load configuration from `/etc/kvmmanager-agent/config.json` (Linux) or registry (Windows)
   - Open virtio-serial channel `/dev/virtio-ports/org.kvmmanager.agent.0`
   - Send initial `agent_ready` notification

2. **Agent Ready Notification** (host → guest, not implemented yet)
```json
{
  "jsonrpc": "2.0",
  "method": "agent_ready",
  "params": {
    "version": "1.0.0",
    "platform": "linux"
  }
}
```

3. **Host Connection**
   - Host connects to Unix socket `/var/lib/libvirt/qemu/channel/target/DOMAIN_NAME.org.kvmmanager.agent.0`
   - Host sends `ping` to verify connectivity
   - Host calls `get_agent_info` to check capabilities

### Reconnection Handling

- **Guest Side**: If virtio-serial channel closes, retry open every 5 seconds
- **Host Side**: If connection lost, retry connection every 10 seconds
- **Timeout**: Mark agent as "unavailable" after 30 seconds without response

### Graceful Shutdown

1. Host sends `shutdown` or closes connection
2. Agent completes in-flight requests (up to 5 second grace period)
3. Agent closes virtio-serial channel
4. Agent exits cleanly

---

## Security Considerations

### Command Execution

- **User Context**: Agent runs as dedicated `kvmmanager-agent` user (not root)
- **Command Whitelist**: Optional configuration to allow only specific commands
- **Timeout Enforcement**: All commands have maximum execution time
- **No Shell Expansion**: Commands executed directly without shell

### File Operations

- **Path Restrictions**: Only allow access to configured directories
- **Symlink Protection**: Do not follow symbolic links outside allowed paths
- **Size Limits**: Maximum file read/write size: 10 MB
- **Permissions**: Files created with agent user ownership

### Resource Limits

- **CPU**: Agent process CPU usage limited via cgroups (Linux) or job objects (Windows)
- **Memory**: Maximum 50 MB resident memory
- **Concurrent Requests**: Maximum 10 concurrent operations
- **Rate Limiting**: Maximum 100 requests per second

---

## Configuration

### Agent Configuration File

**Linux**: `/etc/kvmmanager-agent/config.json`
**Windows**: Registry `HKLM\Software\KVMManager\Agent` or `C:\ProgramData\KVMManager\agent-config.json`

```json
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
    "command_whitelist": null,  // null = all commands allowed
    "max_file_size_mb": 10,
    "max_command_timeout_seconds": 300
  }
}
```

---

## Implementation Notes

### Linux Agent

- **Language**: Rust
- **Dependencies**:
  - `tokio` - async runtime
  - `serde_json` - JSON serialization
  - `nix` - Unix system calls
  - `sysinfo` - system information
- **Service**: systemd unit `kvmmanager-agent.service`
- **Installation**: `.deb` and `.rpm` packages

### Windows Agent

- **Language**: Rust (same codebase with platform abstractions)
- **Dependencies**:
  - `tokio` - async runtime
  - `serde_json` - JSON serialization
  - `windows-sys` - Windows API
  - `sysinfo` - system information
- **Service**: Windows Service
- **Installation**: MSI installer

### Backend Integration

- **Service**: `src-tauri/src/services/guest_agent_service.rs`
- **Connection Pool**: Maintain connections to all VMs with agents
- **Request Tracking**: Track request IDs and timeouts
- **Event Handling**: Emit events when agent status changes

---

## Testing Strategy

### Unit Tests

- Message parsing and serialization
- Request/response validation
- Error handling
- Timeout enforcement

### Integration Tests

- Full guest agent running in test VM
- Execute all methods and verify responses
- Test connection recovery
- Test security restrictions

### Manual Testing

- Install agent in Ubuntu 22.04, Debian 12, Alpine Linux
- Install agent in Windows 10, Windows 11
- Test all methods through KVM Manager UI
- Test behavior with network disabled
- Test behavior during host/guest crashes

---

## Future Extensions

### Planned Features (Phase 4+)

- **File Transfer**: Efficient large file transfer
- **Process Management**: List/kill processes
- **Service Management**: Start/stop systemd services (Linux) or Windows Services
- **User Management**: Create/delete users
- **Package Management**: Install/update packages (apt, yum, chocolatey)
- **Log Streaming**: Real-time log file streaming
- **Metrics Collection**: CPU, memory, disk I/O metrics
- **SSH Key Injection**: Inject SSH keys for remote access
- **Cloud-init Integration**: Execute cloud-init directives

---

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [QEMU Guest Agent Protocol](https://wiki.qemu.org/Features/GuestAgent)
- [virtio-serial Documentation](https://www.linux-kvm.org/page/Virtio)
- [libvirt Domain XML Format](https://libvirt.org/formatdomain.html#channel)

---

**Protocol Version**: 1.0.0
**Status**: Draft
**Next Review**: After Sprint 1 implementation
