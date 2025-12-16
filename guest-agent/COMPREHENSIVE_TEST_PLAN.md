# Guest Agent Comprehensive Test Plan
**Phase 4 Testing Phase**
**Date**: December 10, 2025
**Version**: 1.0
**Status**: Ready for Execution

---

## Executive Summary

This comprehensive test plan covers all aspects of the KVM Manager Guest Agent implementation, including protocol methods, virtio-serial communication, installation procedures, security controls, and multi-distribution compatibility.

**Scope**: Phase 4 Guest Agent MVP (Linux only)
**Test Duration**: 2-3 days
**Distributions**: Ubuntu 22.04/24.04, Debian 12, Fedora 40, RHEL 9/Rocky 9, Alpine 3.19

---

## Table of Contents

1. [Test Environment Setup](#test-environment-setup)
2. [Protocol Method Testing](#protocol-method-testing)
3. [Transport Layer Testing](#transport-layer-testing)
4. [Installation Testing](#installation-testing)
5. [Security Testing](#security-testing)
6. [Multi-Distribution Testing](#multi-distribution-testing)
7. [Performance Testing](#performance-testing)
8. [Integration Testing](#integration-testing)
9. [Reliability Testing](#reliability-testing)
10. [Issue Documentation](#issue-documentation)

---

## Test Environment Setup

### Prerequisites

- [ ] KVM Manager application built and running
- [ ] Access to libvirt (qemu:///system)
- [ ] ISO deployed to `/var/lib/libvirt/images/`
- [ ] Test VMs prepared for each distribution
- [ ] Testing tools installed: `socat`, `jq`, `curl`
- [ ] Documentation reviewed: PROTOCOL.md, INSTALL.md

### ISO Deployment

```bash
# Deploy the guest agent ISO
sudo cp /home/fraziersystems/Documents/projects/kvm-manager/guest-agent/kvmmanager-guest-agent.iso \
        /var/lib/libvirt/images/
sudo chmod 644 /var/lib/libvirt/images/kvmmanager-guest-agent.iso

# Verify
ls -lh /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

**Expected**: ISO file (~1.9 MB) readable by libvirt

### Test VM Requirements

Each test VM must have:
- virtio-serial channel configured (`org.kvmmanager.agent.0`)
- CD-ROM device for ISO mounting
- Network connectivity (for initial setup)
- Minimum 512 MB RAM, 1 vCPU

---

## Protocol Method Testing

### Objective
Verify all 10 JSON-RPC 2.0 protocol methods work correctly.

### Test Cases

#### TC-P01: ping
**Purpose**: Verify connectivity and agent responsiveness

**Test Steps**:
1. Connect to virtio-serial socket
2. Send ping request: `{"jsonrpc":"2.0","method":"ping","id":1}`
3. Verify response format

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "pong": true,
    "timestamp": <unix_timestamp>
  },
  "id": 1
}
```

**Pass Criteria**:
- [ ] Response received within 100ms
- [ ] `pong` is `true`
- [ ] `timestamp` is valid Unix timestamp
- [ ] No errors in agent logs

---

#### TC-P02: get_agent_info
**Purpose**: Verify agent version and capabilities reporting

**Test Steps**:
1. Send: `{"jsonrpc":"2.0","method":"get_agent_info","id":2}`
2. Parse response

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "version": "0.1.0",
    "protocol_version": "1.0.0",
    "platform": "linux",
    "capabilities": [...]
  },
  "id": 2
}
```

**Pass Criteria**:
- [ ] Version matches binary version
- [ ] Platform is "linux"
- [ ] Capabilities list includes all 10 methods
- [ ] Response time < 50ms

---

#### TC-P03: get_system_info
**Purpose**: Verify system information accuracy

**Test Steps**:
1. Record actual VM info (OS, hostname, CPU, memory)
2. Send: `{"jsonrpc":"2.0","method":"get_system_info","id":3}`
3. Compare response with actual values

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "os_type": "linux",
    "os_name": "Ubuntu",
    "os_version": "22.04 LTS",
    "kernel_version": "5.15.0-89-generic",
    "hostname": "<actual_hostname>",
    "architecture": "x86_64",
    "cpu_count": <actual_cpus>,
    "total_memory_kb": <actual_memory>,
    "uptime_seconds": <actual_uptime>
  },
  "id": 3
}
```

**Pass Criteria**:
- [ ] OS name matches `/etc/os-release`
- [ ] Hostname matches `hostname` command
- [ ] Kernel version matches `uname -r`
- [ ] CPU count matches `nproc`
- [ ] Memory within 5% of `free -k`
- [ ] Uptime within 60 seconds of `uptime -s`

---

#### TC-P04: get_network_info
**Purpose**: Verify network interface detection

**Test Steps**:
1. List actual interfaces with `ip addr`
2. Send: `{"jsonrpc":"2.0","method":"get_network_info","id":4}`
3. Compare all interfaces

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "interfaces": [
      {
        "name": "eth0",
        "mac_address": "52:54:00:...",
        "ipv4_addresses": ["192.168.122.10/24"],
        "ipv6_addresses": ["fe80::..."],
        "state": "up",
        "mtu": 1500
      }
    ]
  },
  "id": 4
}
```

**Pass Criteria**:
- [ ] All interfaces present (lo, eth0, etc.)
- [ ] MAC addresses correct
- [ ] IPv4 addresses match
- [ ] IPv6 addresses match (if configured)
- [ ] Interface state correct (up/down)
- [ ] MTU values correct

---

#### TC-P05: get_disk_usage
**Purpose**: Verify filesystem usage reporting

**Test Steps**:
1. Check actual disk usage: `df -k`
2. Send: `{"jsonrpc":"2.0","method":"get_disk_usage","id":5}`
3. Compare values

**Expected Result**:
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
  "id": 5
}
```

**Pass Criteria**:
- [ ] All mounted filesystems reported
- [ ] Mount points correct
- [ ] Filesystem types correct
- [ ] Byte values within 1% of `df`
- [ ] Percentages accurate
- [ ] Excludes tmpfs/devtmpfs (unless configured otherwise)

---

#### TC-P06: exec_command
**Purpose**: Verify command execution with security controls

**Test Steps**:

1. **Simple command**:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "exec_command",
     "params": {
       "command": "echo",
       "args": ["Hello World"],
       "timeout_seconds": 10
     },
     "id": 6
   }
   ```

2. **Command with output**:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "exec_command",
     "params": {
       "command": "ls",
       "args": ["-la", "/tmp"],
       "timeout_seconds": 10
     },
     "id": 7
   }
   ```

3. **Command with stderr**:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "exec_command",
     "params": {
       "command": "ls",
       "args": ["/nonexistent"],
       "timeout_seconds": 10
     },
     "id": 8
   }
   ```

**Pass Criteria**:
- [ ] Stdout captured correctly
- [ ] Stderr captured separately
- [ ] Exit code accurate (0 for success, non-zero for errors)
- [ ] No shell expansion (`echo $HOME` returns literal "$HOME")
- [ ] Execution time reported
- [ ] Timeout enforced (test with `sleep 60`)

---

#### TC-P07: file_read
**Purpose**: Verify file reading with path restrictions

**Test Steps**:

1. **Read allowed file**:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "file_read",
     "params": {
       "path": "/etc/hostname"
     },
     "id": 9
   }
   ```

2. **Read with base64**:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "file_read",
     "params": {
       "path": "/etc/hostname",
       "encoding": "base64"
     },
     "id": 10
   }
   ```

**Pass Criteria**:
- [ ] File content matches actual content
- [ ] Size reported accurately
- [ ] UTF-8 encoding works
- [ ] Base64 encoding works
- [ ] Binary files handled correctly
- [ ] Path restrictions enforced (see Security Testing)

---

#### TC-P08: file_write
**Purpose**: Verify file writing with path restrictions

**Test Steps**:

1. **Write text file**:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "file_write",
     "params": {
       "path": "/tmp/test-agent-write.txt",
       "content": "Hello from host!",
       "create_dirs": false,
       "permissions": "0644"
     },
     "id": 11
   }
   ```

2. **Verify written file**:
   - Use file_read to read back
   - Check content matches
   - Verify permissions: `ls -l /tmp/test-agent-write.txt`

**Pass Criteria**:
- [ ] File created successfully
- [ ] Content matches
- [ ] Permissions set correctly
- [ ] Bytes written reported
- [ ] create_dirs option works
- [ ] Path restrictions enforced (see Security Testing)

---

#### TC-P09: shutdown
**Purpose**: Verify graceful shutdown

**Test Steps**:
1. Create disposable test VM
2. Send shutdown request:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "shutdown",
     "params": {
       "timeout_seconds": 60,
       "force": false
     },
     "id": 12
   }
   ```
3. Verify VM shuts down gracefully

**Pass Criteria**:
- [ ] Response received before shutdown
- [ ] VM shuts down within timeout
- [ ] Graceful shutdown (not forced)
- [ ] No data corruption

**Note**: Use with caution - only on test VMs!

---

#### TC-P10: reboot
**Purpose**: Verify graceful reboot

**Test Steps**:
1. Send reboot request:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "reboot",
     "params": {
       "timeout_seconds": 60,
       "force": false
     },
     "id": 13
   }
   ```
2. Verify VM reboots
3. Verify agent reconnects after reboot

**Pass Criteria**:
- [ ] Response received before reboot
- [ ] VM reboots within timeout
- [ ] Agent starts on reboot
- [ ] Agent reconnects to host
- [ ] All methods work after reboot

---

## Transport Layer Testing

### Objective
Verify virtio-serial communication reliability and error handling.

### TC-T01: Connection Establishment
**Test Steps**:
1. Start VM with virtio-serial
2. Start agent
3. Verify connection established

**Pass Criteria**:
- [ ] Agent logs "Connected to virtio-serial device"
- [ ] Socket exists on host
- [ ] Can send/receive messages

### TC-T02: Message Framing
**Test Steps**:
1. Send multiple messages in rapid succession
2. Verify all responses received in order

**Pass Criteria**:
- [ ] All messages processed
- [ ] Responses match request IDs
- [ ] No message corruption
- [ ] Newline delimiters working correctly

### TC-T03: Large Message Handling
**Test Steps**:
1. Send large file_read request (near 64 KB limit)
2. Verify response handling

**Pass Criteria**:
- [ ] Large messages handled correctly
- [ ] No truncation
- [ ] Performance acceptable

### TC-T04: Reconnection After Disconnect
**Test Steps**:
1. Pause VM: `virsh suspend <vm>`
2. Resume VM: `virsh resume <vm>`
3. Verify agent reconnects

**Pass Criteria**:
- [ ] Agent detects disconnect
- [ ] Agent retries connection
- [ ] Communication restored
- [ ] No data loss

### TC-T05: Multiple VM Support
**Test Steps**:
1. Start 3+ VMs with agents
2. Send requests to each simultaneously

**Pass Criteria**:
- [ ] All agents respond correctly
- [ ] No cross-VM contamination
- [ ] Backend tracks connections properly

---

## Installation Testing

### Objective
Verify installation scripts work correctly on all supported distributions.

### TC-I01: Debian/Ubuntu Installation
**Distributions**: Ubuntu 22.04, Ubuntu 24.04, Debian 12

**Test Steps**:
1. Create fresh VM
2. Mount guest agent ISO
3. Run: `sudo bash /media/cdrom/install-debian.sh`
4. Verify service starts

**Pass Criteria**:
- [ ] Binary copied to `/usr/bin/kvmmanager-agent`
- [ ] Config created in `/etc/kvmmanager-agent/config.json`
- [ ] systemd service installed and enabled
- [ ] Service running and functional
- [ ] No errors in installation

### TC-I02: RHEL/Fedora Installation
**Distributions**: Fedora 40, RHEL 9, Rocky Linux 9

**Test Steps**:
1. Create fresh VM
2. Mount guest agent ISO
3. Run: `sudo bash /media/cdrom/install-rhel.sh`
4. Verify service starts

**Pass Criteria**:
- [ ] Binary copied correctly
- [ ] Config created
- [ ] systemd service working
- [ ] SELinux compatibility (no AVCs)
- [ ] Service functional

### TC-I03: Alpine Linux Installation
**Distribution**: Alpine 3.19

**Test Steps**:
1. Create fresh VM
2. Mount guest agent ISO
3. Run: `sudo sh /media/cdrom/install-alpine.sh`
4. Verify service starts

**Pass Criteria**:
- [ ] Binary copied correctly
- [ ] Config created
- [ ] OpenRC service installed
- [ ] Service running
- [ ] Lightweight operation (< 10 MB total)

### TC-I04: ISO Mounting/Ejection
**Test Steps**:
1. Use KVM Manager UI to mount ISO
2. Verify ISO accessible in VM
3. Install agent
4. Eject ISO via UI
5. Verify ISO ejected

**Pass Criteria**:
- [ ] Mount button works
- [ ] ISO appears in VM
- [ ] Installation proceeds
- [ ] Eject button works
- [ ] ISO cleanly removed

### TC-I05: Upgrade Installation
**Test Steps**:
1. Install version 0.1.0
2. Build new version 0.1.1
3. Re-run installer
4. Verify upgrade

**Pass Criteria**:
- [ ] Binary replaced
- [ ] Config preserved
- [ ] Service restarted
- [ ] Agent reports new version
- [ ] No data loss

---

## Security Testing

### Objective
Verify security controls prevent unauthorized access and attacks.

### TC-S01: Path Traversal Prevention (Read)
**Test Steps**:
1. Configure `allowed_read_paths: ["/tmp"]`
2. Attempt to read outside allowed paths:
   - `/etc/passwd`
   - `../../../etc/shadow`
   - `/tmp/../etc/hostname`

**Pass Criteria**:
- [ ] All attempts blocked with error
- [ ] Error code -32001 (Permission denied)
- [ ] Attempts logged
- [ ] No actual file access

### TC-S02: Path Traversal Prevention (Write)
**Test Steps**:
1. Configure `allowed_write_paths: ["/tmp/kvmmanager"]`
2. Attempt to write outside allowed paths:
   - `/etc/passwd`
   - `/tmp/../etc/malicious`
   - `/tmp/kvmmanager/../../etc/test`

**Pass Criteria**:
- [ ] All attempts blocked
- [ ] Permission denied errors
- [ ] No files created outside allowed paths
- [ ] Attempts logged

### TC-S03: Command Whitelist
**Test Steps**:
1. Enable command whitelist:
   ```json
   {
     "command_whitelist_enabled": true,
     "command_whitelist": ["ls", "cat"]
   }
   ```
2. Restart agent
3. Attempt allowed command: `ls /tmp`
4. Attempt blocked command: `rm -rf /tmp/test`

**Pass Criteria**:
- [ ] Allowed commands execute
- [ ] Blocked commands rejected
- [ ] Error message clear
- [ ] No security bypass possible

### TC-S04: Shell Injection Prevention
**Test Steps**:
1. Attempt command with shell metacharacters:
   ```json
   {
     "method": "exec_command",
     "params": {
       "command": "echo",
       "args": ["test; rm -rf /"]
     }
   }
   ```

**Pass Criteria**:
- [ ] Output is literal "test; rm -rf /"
- [ ] No command execution beyond echo
- [ ] Semicolons not interpreted
- [ ] Pipes, redirects not processed

### TC-S05: File Size Limits
**Test Steps**:
1. Configure `max_file_size_mb: 10`
2. Attempt to read/write file > 10 MB

**Pass Criteria**:
- [ ] Large reads rejected
- [ ] Large writes rejected
- [ ] Error message indicates size limit
- [ ] No memory exhaustion

### TC-S06: Command Timeout Enforcement
**Test Steps**:
1. Configure `command_timeout_seconds: 5`
2. Execute long-running command: `sleep 60`

**Pass Criteria**:
- [ ] Command terminated at 5 seconds
- [ ] Timeout error returned
- [ ] Process killed
- [ ] Agent remains responsive

### TC-S07: Resource Limits
**Test Steps**:
1. Send 100 requests simultaneously
2. Monitor agent resource usage

**Pass Criteria**:
- [ ] Agent handles load gracefully
- [ ] CPU usage < 50%
- [ ] Memory usage < 50 MB
- [ ] No crashes or hangs
- [ ] All requests processed

### TC-S08: Concurrent Request Limit
**Test Steps**:
1. Configure `max_concurrent_requests: 10`
2. Send 20 simultaneous requests

**Pass Criteria**:
- [ ] First 10 processed
- [ ] Remaining queued or rejected gracefully
- [ ] No deadlocks
- [ ] Throughput maintained

---

## Multi-Distribution Testing

### Objective
Verify compatibility across different Linux distributions.

### Distribution Matrix

| Distribution | Version | Kernel | Init System | Status |
|--------------|---------|--------|-------------|--------|
| Ubuntu       | 22.04   | 5.15   | systemd     | [ ]    |
| Ubuntu       | 24.04   | 6.8    | systemd     | [ ]    |
| Debian       | 12      | 6.1    | systemd     | [ ]    |
| Fedora       | 40      | 6.8    | systemd     | [ ]    |
| RHEL         | 9       | 5.14   | systemd     | [ ]    |
| Rocky Linux  | 9       | 5.14   | systemd     | [ ]    |
| Alpine       | 3.19    | 6.6    | OpenRC      | [ ]    |

### Per-Distribution Tests

For each distribution:

#### Installation
- [ ] ISO mounts successfully
- [ ] Installer runs without errors
- [ ] Binary compatible (no missing libs)
- [ ] Service starts automatically
- [ ] Agent functional after boot

#### OS Detection
- [ ] OS name correct in `/etc/os-release`
- [ ] Version string accurate
- [ ] Distribution-specific details captured

#### Package Dependencies
- [ ] No missing system libraries
- [ ] Minimal dependencies required
- [ ] Static linking works

#### Service Management
- [ ] systemd/OpenRC integration
- [ ] Auto-start on boot
- [ ] Restart on failure
- [ ] Logging to journal/file

#### Special Cases
- [ ] SELinux (RHEL/Fedora): No policy violations
- [ ] AppArmor (Ubuntu): No denials
- [ ] Alpine musl libc: Binary compatible

---

## Performance Testing

### Objective
Verify agent meets performance requirements.

### TC-PERF01: Response Latency
**Test Steps**:
1. Send 100 ping requests
2. Measure response time for each

**Pass Criteria**:
- [ ] Average latency < 10ms
- [ ] 95th percentile < 20ms
- [ ] 99th percentile < 50ms
- [ ] No timeouts

### TC-PERF02: Throughput
**Test Steps**:
1. Send 1000 system_info requests over 60 seconds
2. Measure requests/second

**Pass Criteria**:
- [ ] Throughput > 50 req/sec
- [ ] No degradation over time
- [ ] No memory leaks
- [ ] Consistent response times

### TC-PERF03: Resource Usage (Idle)
**Test Steps**:
1. Install agent
2. Let run for 1 hour idle
3. Measure resources

**Pass Criteria**:
- [ ] CPU usage < 0.5%
- [ ] Memory < 10 MB RSS
- [ ] No memory growth
- [ ] No CPU spikes

### TC-PERF04: Resource Usage (Active)
**Test Steps**:
1. Generate continuous requests
2. Measure peak resource usage

**Pass Criteria**:
- [ ] CPU usage < 10% under load
- [ ] Memory < 20 MB RSS
- [ ] No resource leaks
- [ ] Graceful degradation

### TC-PERF05: Startup Time
**Test Steps**:
1. Restart agent service
2. Measure time to first successful request

**Pass Criteria**:
- [ ] Startup < 500ms
- [ ] Agent ready immediately
- [ ] No initialization delays

### TC-PERF06: Large Dataset Handling
**Test Steps**:
1. VM with 100+ network interfaces
2. Request network_info
3. Measure performance

**Pass Criteria**:
- [ ] All interfaces reported
- [ ] Response time < 1 second
- [ ] No truncation
- [ ] Memory efficient

---

## Integration Testing

### Objective
Verify end-to-end integration with KVM Manager.

### TC-INT01: UI Display
**Test Steps**:
1. Open KVM Manager
2. Navigate to VM details
3. Verify guest information displayed

**Pass Criteria**:
- [ ] Agent status shown correctly
- [ ] System info displays
- [ ] Network info displays
- [ ] Disk usage displays
- [ ] UI updates automatically

### TC-INT02: Auto-Refresh
**Test Steps**:
1. Open VM details page
2. Create large file in VM
3. Wait for auto-refresh

**Pass Criteria**:
- [ ] Disk usage updates without manual refresh
- [ ] Network changes detected
- [ ] Uptime increments
- [ ] Refresh intervals correct (5s, 10s, 30s)

### TC-INT03: Multiple VMs
**Test Steps**:
1. Run 5 VMs with agents
2. Open each VM details page
3. Verify independent operation

**Pass Criteria**:
- [ ] All VMs show correct data
- [ ] No data mixing between VMs
- [ ] Performance acceptable
- [ ] UI responsive

### TC-INT04: Agent Unavailable Handling
**Test Steps**:
1. Stop agent in VM
2. Check UI response

**Pass Criteria**:
- [ ] UI shows "Agent not available"
- [ ] Installation instructions displayed
- [ ] No crashes or errors
- [ ] Graceful degradation

### TC-INT05: Backend Error Handling
**Test Steps**:
1. Inject errors (network issues, timeouts)
2. Verify backend handling

**Pass Criteria**:
- [ ] Errors logged appropriately
- [ ] User-friendly error messages
- [ ] No backend crashes
- [ ] Automatic retry logic

---

## Reliability Testing

### Objective
Verify agent stability and fault tolerance.

### TC-REL01: 24-Hour Stability
**Test Steps**:
1. Install agent
2. Run for 24 hours with periodic requests
3. Monitor for issues

**Pass Criteria**:
- [ ] No crashes
- [ ] No memory leaks
- [ ] No performance degradation
- [ ] All requests successful

### TC-REL02: Reconnection After VM Operations
**Test Steps**:

1. **Pause/Resume**:
   - `virsh suspend <vm>`
   - `virsh resume <vm>`
   - Verify agent reconnects

2. **Save/Restore**:
   - `virsh save <vm> <file>`
   - `virsh restore <file>`
   - Verify agent reconnects

3. **Reboot**:
   - `virsh reboot <vm>`
   - Verify agent auto-starts

**Pass Criteria**:
- [ ] Agent reconnects within 10 seconds
- [ ] All methods functional after reconnection
- [ ] No data corruption
- [ ] Logs show clean reconnection

### TC-REL03: Host Restart
**Test Steps**:
1. Configure VM for auto-start
2. Restart host system
3. Verify agent reconnects

**Pass Criteria**:
- [ ] VM auto-starts
- [ ] Agent auto-starts
- [ ] Connection established
- [ ] Fully functional

### TC-REL04: Virtio-Serial Failure Recovery
**Test Steps**:
1. Simulate virtio-serial device removal
2. Re-add device
3. Verify recovery

**Pass Criteria**:
- [ ] Agent logs error
- [ ] Agent retries connection
- [ ] Reconnects when available
- [ ] No manual intervention needed

### TC-REL05: Concurrent Operations
**Test Steps**:
1. Execute multiple commands simultaneously
2. Perform file operations concurrently
3. Verify no race conditions

**Pass Criteria**:
- [ ] All operations complete successfully
- [ ] Correct results for each
- [ ] No deadlocks
- [ ] No corruption

---

## Issue Documentation

### Issue Template

For any issues found during testing, document using this template:

```markdown
### Issue #XXX: [Short Description]

**Severity**: Critical / High / Medium / Low
**Test Case**: TC-XXX
**Distribution**: Ubuntu 22.04 / etc.
**Agent Version**: 0.1.0

**Description**:
Clear description of the issue.

**Steps to Reproduce**:
1. Step one
2. Step two
3. Step three

**Expected Behavior**:
What should happen

**Actual Behavior**:
What actually happened

**Logs**:
```
Relevant log excerpts
```

**Screenshots**:
If applicable

**Workaround**:
If any temporary fix exists

**Fix Required**:
What needs to be changed to fix this issue
```

### Issue Tracking

| ID | Severity | Description | Status | Fixed In |
|----|----------|-------------|--------|----------|
|    |          |             |        |          |

---

## Test Execution Log

### Test Session Information

- **Tester**: [Name]
- **Date**: [Date]
- **Environment**: [Host OS, libvirt version]
- **Agent Version**: [Version]
- **Duration**: [Time spent]

### Summary Results

| Category | Total Tests | Passed | Failed | Blocked | Pass Rate |
|----------|-------------|--------|--------|---------|-----------|
| Protocol Methods | 10 | | | | |
| Transport Layer | 5 | | | | |
| Installation | 5 | | | | |
| Security | 8 | | | | |
| Multi-Distribution | 7 | | | | |
| Performance | 6 | | | | |
| Integration | 5 | | | | |
| Reliability | 5 | | | | |
| **TOTAL** | **51** | | | | |

### Critical Findings

List any critical issues that block release:

1.
2.
3.

### Recommendations

Based on testing results:

1. **Ship as-is**: All tests passed, ready for production
2. **Ship with known issues**: Document limitations
3. **Needs fixes**: Critical issues must be resolved
4. **Major rework needed**: Fundamental problems found

---

## Sign-Off

### Test Completion

- [ ] All test cases executed
- [ ] Issues documented
- [ ] Results reviewed
- [ ] Recommendations provided

### Approvals

**Guest Agent Specialist**: ________________________ Date: __________

**Project Coordinator**: ________________________ Date: __________

**QA Lead**: ________________________ Date: __________

---

## Appendix A: Quick Test Scripts

### Automated Protocol Test Script

Save as `test-protocol-methods.sh`:

```bash
#!/bin/bash
# Automated protocol method testing script

set -e

VM_NAME="${1:-test-vm}"
SOCKET="/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0"

echo "=== Guest Agent Protocol Test Suite ==="
echo "VM: $VM_NAME"
echo "Socket: $SOCKET"
echo ""

if [ ! -S "$SOCKET" ]; then
    echo "❌ Socket not found!"
    exit 1
fi

# Test 1: ping
echo "Test 1: ping"
RESULT=$(echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.pong == true' > /dev/null; then
    echo "✅ PASS"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 2: get_agent_info
echo "Test 2: get_agent_info"
RESULT=$(echo '{"jsonrpc":"2.0","method":"get_agent_info","id":2}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.version' > /dev/null; then
    echo "✅ PASS - Version: $(echo $RESULT | jq -r .result.version)"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 3: get_system_info
echo "Test 3: get_system_info"
RESULT=$(echo '{"jsonrpc":"2.0","method":"get_system_info","id":3}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.hostname' > /dev/null; then
    echo "✅ PASS"
    echo "$RESULT" | jq '.result | {os_name, hostname, cpu_count}'
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 4: get_network_info
echo "Test 4: get_network_info"
RESULT=$(echo '{"jsonrpc":"2.0","method":"get_network_info","id":4}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.interfaces' > /dev/null; then
    echo "✅ PASS - Interfaces: $(echo $RESULT | jq '.result.interfaces | length')"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 5: get_disk_usage
echo "Test 5: get_disk_usage"
RESULT=$(echo '{"jsonrpc":"2.0","method":"get_disk_usage","id":5}' | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.filesystems' > /dev/null; then
    echo "✅ PASS - Filesystems: $(echo $RESULT | jq '.result.filesystems | length')"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 6: exec_command
echo "Test 6: exec_command (echo test)"
PAYLOAD='{"jsonrpc":"2.0","method":"exec_command","params":{"command":"echo","args":["Hello from test"],"timeout_seconds":10},"id":6}'
RESULT=$(echo "$PAYLOAD" | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.stdout' | grep -q "Hello from test"; then
    echo "✅ PASS"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 7: file_write
echo "Test 7: file_write"
PAYLOAD='{"jsonrpc":"2.0","method":"file_write","params":{"path":"/tmp/agent-test.txt","content":"Test from host","create_dirs":false},"id":7}'
RESULT=$(echo "$PAYLOAD" | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.bytes_written' > /dev/null; then
    echo "✅ PASS"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

# Test 8: file_read
echo "Test 8: file_read"
PAYLOAD='{"jsonrpc":"2.0","method":"file_read","params":{"path":"/tmp/agent-test.txt"},"id":8}'
RESULT=$(echo "$PAYLOAD" | socat - UNIX-CONNECT:$SOCKET)
if echo "$RESULT" | jq -e '.result.content' | grep -q "Test from host"; then
    echo "✅ PASS"
else
    echo "❌ FAIL: $RESULT"
fi
echo ""

echo "=== Test Suite Complete ==="
```

### Performance Benchmark Script

Save as `benchmark-agent.sh`:

```bash
#!/bin/bash
# Guest agent performance benchmark

VM_NAME="${1:-test-vm}"
SOCKET="/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0"
ITERATIONS=100

echo "=== Guest Agent Performance Benchmark ==="
echo "Iterations: $ITERATIONS"
echo ""

# Benchmark ping latency
echo "Benchmarking ping latency..."
START=$(date +%s%N)
for i in $(seq 1 $ITERATIONS); do
    echo '{"jsonrpc":"2.0","method":"ping","id":'$i'}' | socat - UNIX-CONNECT:$SOCKET > /dev/null
done
END=$(date +%s%N)
DURATION=$(( (END - START) / 1000000 ))
AVG=$(( DURATION / ITERATIONS ))
echo "Total: ${DURATION}ms"
echo "Average: ${AVG}ms per request"
echo "Throughput: $(( 1000 / AVG )) req/sec"
echo ""

# Benchmark system_info
echo "Benchmarking get_system_info..."
START=$(date +%s%N)
for i in $(seq 1 50); do
    echo '{"jsonrpc":"2.0","method":"get_system_info","id":'$i'}' | socat - UNIX-CONNECT:$SOCKET > /dev/null
done
END=$(date +%s%N)
DURATION=$(( (END - START) / 1000000 ))
AVG=$(( DURATION / 50 ))
echo "Average: ${AVG}ms per request"
echo ""

echo "=== Benchmark Complete ==="
```

---

## Appendix B: Configuration Templates

### Development Config (Permissive)

```json
{
  "allowed_read_paths": [
    "/",
    "/etc",
    "/proc",
    "/sys",
    "/var",
    "/tmp",
    "/home"
  ],
  "allowed_write_paths": [
    "/tmp",
    "/var/tmp"
  ],
  "command_whitelist_enabled": false,
  "max_file_size_bytes": 10485760,
  "command_timeout_seconds": 30,
  "log_level": "debug"
}
```

### Production Config (Restrictive)

```json
{
  "allowed_read_paths": [
    "/etc/hostname",
    "/etc/os-release",
    "/proc/cpuinfo",
    "/proc/meminfo",
    "/proc/uptime",
    "/var/log",
    "/tmp"
  ],
  "allowed_write_paths": [
    "/tmp/kvmmanager"
  ],
  "command_whitelist_enabled": true,
  "command_whitelist": [
    "ls",
    "cat",
    "ps",
    "df",
    "free",
    "uptime"
  ],
  "max_file_size_bytes": 1048576,
  "command_timeout_seconds": 10,
  "log_level": "info"
}
```

---

## Appendix C: Common Issues and Solutions

### Issue: Socket Permission Denied

**Symptom**: Backend cannot connect to virtio-serial socket

**Solution**:
```bash
# Check socket permissions
ls -l /var/lib/libvirt/qemu/channel/target/

# Fix permissions if needed
sudo chown libvirt-qemu:kvm /var/lib/libvirt/qemu/channel/target/*.agent.0
sudo chmod 660 /var/lib/libvirt/qemu/channel/target/*.agent.0
```

### Issue: Agent Not Starting

**Symptom**: Service fails to start

**Diagnosis**:
```bash
# Check service status
sudo systemctl status kvmmanager-agent

# View logs
sudo journalctl -u kvmmanager-agent -n 100 --no-pager

# Common causes:
# - virtio-serial device missing
# - Config file invalid JSON
# - Binary permissions incorrect
```

### Issue: Methods Timing Out

**Symptom**: Requests timeout without response

**Diagnosis**:
- Check agent is running and responsive
- Verify socket connection
- Check for agent deadlock in logs
- Test with simple ping request

---

**End of Comprehensive Test Plan**
