# Guest Agent Testing - Known Issues and Risks
**Phase 4 Testing Phase**
**Date**: December 10, 2025

---

## Purpose

This document identifies potential issues, risks, and edge cases to specifically test during Phase 4 validation. Based on the implementation review, these areas require focused attention.

---

## High-Risk Areas

### 1. Virtio-Serial Communication

#### Potential Issues
- **Connection timing**: Agent may start before virtio-serial device is ready
- **Reconnection logic**: Retry intervals and backoff strategy
- **Message framing**: Newline delimiter handling with large messages
- **Buffer overflow**: Large responses near 64 KB limit
- **Concurrent access**: Multiple requests on same channel

#### Tests Required
- [ ] Start agent before VM fully booted
- [ ] Send messages > 60 KB
- [ ] Send malformed JSON (no newline)
- [ ] Send rapid-fire requests (100+ in 1 second)
- [ ] Test with slow consumer (delayed reads)
- [ ] Verify no message mixing

#### Risk Level: **HIGH**
- Impact: Total communication failure
- Mitigation: Extensive testing with edge cases

---

### 2. JSON-RPC Request/Response Matching

#### Potential Issues
- **ID collision**: Request IDs not properly tracked
- **Orphaned responses**: Response without matching request
- **Timeout handling**: Requests timing out while response in flight
- **Concurrent request IDs**: Multiple clients with same ID

#### Tests Required
- [ ] Send requests with duplicate IDs
- [ ] Send response without request
- [ ] Test timeout boundary conditions
- [ ] Verify ID tracking cleanup
- [ ] Test with out-of-order responses

#### Risk Level: **MEDIUM**
- Impact: Wrong data returned to frontend
- Mitigation: Request ID tracking audit

---

### 3. File Operations Security

#### Potential Issues
- **Path traversal**: `/../` sequences, symlinks
- **Race conditions**: TOCTOU (Time-of-check Time-of-use)
- **Privilege escalation**: Writing to protected paths
- **Symlink following**: Following symlinks outside allowed paths
- **Binary vs text confusion**: Encoding issues with binary files

#### Tests Required
- [ ] Test: `file_write("/tmp/../../etc/passwd")`
- [ ] Test: `file_read("/tmp/link")` where link → `/etc/shadow`
- [ ] Create symlink in allowed dir pointing to restricted dir
- [ ] Write binary file, read as UTF-8
- [ ] Write to `/tmp/test`, race to create `/tmp` as symlink
- [ ] Test null bytes in paths: `/tmp/test\0.txt`
- [ ] Test very long paths (> 4096 chars)

#### Risk Level: **CRITICAL**
- Impact: Security breach, privilege escalation
- Mitigation: Thorough security audit and fuzzing

---

### 4. Command Execution

#### Potential Issues
- **Shell injection**: Despite no shell, unexpected behavior
- **Resource exhaustion**: Fork bombs, memory hogs
- **Zombie processes**: Commands not reaped
- **Signal handling**: SIGTERM, SIGKILL during command execution
- **Working directory traversal**: Using command to change restrictions

#### Tests Required
- [ ] Execute: `sh -c "rm -rf /"`
- [ ] Execute: `:(){ :|:& };:` (fork bomb - in isolated VM!)
- [ ] Execute long-running command, kill agent
- [ ] Execute command that spawns children
- [ ] Test whitelist bypass: `cat /bin/sh | sh`
- [ ] Test command with special chars in args: `echo $HOME`
- [ ] Test absolute vs relative command paths

#### Risk Level: **CRITICAL**
- Impact: Code execution, DoS, system compromise
- Mitigation: Sandboxing, resource limits, whitelist testing

---

### 5. Memory Safety and Leaks

#### Potential Issues
- **Connection leak**: Connections not closed properly
- **Request tracking leak**: Old requests not garbage collected
- **Buffer growth**: Buffers growing without bounds
- **serde deserialization**: Large JSON causing memory spike

#### Tests Required
- [ ] Connect/disconnect 1000 times, check memory
- [ ] Send 10,000 requests, monitor memory growth
- [ ] Send very large JSON payloads (MB size)
- [ ] Check file descriptors: `lsof -p <agent_pid>`
- [ ] Monitor with `valgrind` or `heaptrack`
- [ ] 24-hour stress test with memory tracking

#### Risk Level: **HIGH**
- Impact: Memory exhaustion, OOM killer, service failure
- Mitigation: Load testing and profiling

---

### 6. Distribution-Specific Issues

#### Potential Issues
- **SELinux denials** (RHEL, Fedora): virtio-serial access blocked
- **AppArmor denials** (Ubuntu): File operations blocked
- **musl libc compatibility** (Alpine): Different behavior than glibc
- **systemd vs OpenRC**: Service management differences
- **Old kernel versions**: virtio-serial bugs or missing features

#### Tests Required
- [ ] RHEL/Fedora: Check `ausearch -m avc -ts recent`
- [ ] Ubuntu: Check `aa-status` and `/var/log/syslog`
- [ ] Alpine: Verify musl compatibility, no glibc deps
- [ ] Each distro: Verify service auto-start after reboot
- [ ] Old kernel (3.x): Test basic virtio-serial functionality

#### Risk Level: **MEDIUM**
- Impact: Agent non-functional on specific distributions
- Mitigation: Multi-distribution testing matrix

---

### 7. Race Conditions and Timing Issues

#### Potential Issues
- **Agent starts before VM network ready**: Network info empty
- **Disk mount during get_disk_usage**: Concurrent filesystem operations
- **File modified during read**: Partial or corrupt data
- **VM shutdown during request**: Request in flight
- **Concurrent file operations**: Multiple writes to same file

#### Tests Required
- [ ] Request network info immediately after boot
- [ ] Mount filesystem during disk_usage call
- [ ] Write to file while reading it
- [ ] Send request, immediately shutdown VM
- [ ] Concurrent writes to same path from different requests
- [ ] Agent restart during active request

#### Risk Level: **MEDIUM**
- Impact: Incorrect data, partial responses, crashes
- Mitigation: Locking, retry logic, error handling

---

### 8. Error Handling and Edge Cases

#### Potential Issues
- **Malformed JSON**: Invalid UTF-8, truncated JSON
- **Missing required fields**: Request without `id` or `method`
- **Unknown methods**: Graceful rejection
- **Null values in unexpected places**: `null` path, `null` args
- **Empty arrays**: Empty command args, empty file content
- **Very large numbers**: Integer overflow in sizes, timeouts

#### Tests Required
- [ ] Send invalid JSON: `{invalid`
- [ ] Send request without `id`: `{"jsonrpc":"2.0","method":"ping"}`
- [ ] Send unknown method: `{"jsonrpc":"2.0","method":"unknown","id":1}`
- [ ] Send `null` for required fields
- [ ] Send empty strings: `{"path":""}`
- [ ] Send negative numbers: `{"timeout_seconds":-1}`
- [ ] Send huge numbers: `{"timeout_seconds":999999999999}`

#### Risk Level: **MEDIUM**
- Impact: Crashes, panics, undefined behavior
- Mitigation: Comprehensive input validation

---

### 9. Performance Degradation

#### Potential Issues
- **O(n) lookups**: Request tracking with linear search
- **Unbounded queues**: Request queue growing without limit
- **Blocking I/O**: Synchronous operations blocking event loop
- **CPU spikes**: Inefficient algorithms for large datasets
- **Disk I/O saturation**: Too many concurrent file operations

#### Tests Required
- [ ] 10,000 in-flight requests, measure response time
- [ ] 100 VMs with agents, measure backend CPU
- [ ] Large directory listing (1M+ files)
- [ ] Very large file read (near size limit)
- [ ] Measure time complexity: 1, 10, 100, 1000 requests

#### Risk Level: **MEDIUM**
- Impact: Slow responses, UI lag, poor user experience
- Mitigation: Performance profiling and optimization

---

### 10. Backend Integration Issues

#### Potential Issues
- **Socket path construction**: Incorrect VM name escaping
- **Connection pooling**: Stale connections not cleaned
- **Tauri state management**: State not properly synchronized
- **Error propagation**: Backend errors not surfaced to UI
- **Timeout mismatches**: Backend timeout < agent timeout

#### Tests Required
- [ ] VM name with special chars: `test-vm_123.domain`
- [ ] Stop agent, check backend error handling
- [ ] Multiple rapid requests from UI
- [ ] Backend timeout during long operation
- [ ] Check Tauri command error messages in UI

#### Risk Level: **MEDIUM**
- Impact: UI errors, confusing user experience
- Mitigation: Integration testing with real UI

---

## Edge Cases to Test

### Unusual VM Configurations

- [ ] **VM with 100+ network interfaces**: Performance and correctness
- [ ] **VM with 50+ filesystems**: get_disk_usage response size
- [ ] **VM with no network**: Network info returns empty gracefully
- [ ] **VM with read-only root**: File operations fail gracefully
- [ ] **VM with very long hostname** (> 255 chars): Truncation or error

### Unusual System States

- [ ] **Out of disk space**: File write operations
- [ ] **Out of memory**: Agent behavior under memory pressure
- [ ] **High CPU load**: Agent responsiveness during stress
- [ ] **Network namespace isolation**: Agent in container/namespace
- [ ] **Chroot environment**: Agent running in chroot

### Unusual Operations

- [ ] **Send 1 MB JSON request**: Message framing
- [ ] **Execute command that outputs 10 MB**: Output capture
- [ ] **Read /dev/zero**: Infinite file handling
- [ ] **Execute `cat` without input**: Hanging command
- [ ] **Concurrent shutdown/reboot**: Multiple power commands

---

## Automated Testing Opportunities

### Unit Tests
- [ ] JSON-RPC request/response parsing
- [ ] Path validation logic
- [ ] Command whitelist checking
- [ ] Message framing (newline delimiter)
- [ ] Configuration parsing

### Integration Tests
- [ ] Full protocol test suite (all 10 methods)
- [ ] Security test suite (path traversal, injection)
- [ ] Error handling test suite (malformed input)
- [ ] Performance benchmark suite

### Suggested Test Framework
```bash
# Structure
guest-agent/tests/
  ├── unit/              # Rust unit tests
  │   ├── protocol.rs
  │   ├── security.rs
  │   └── config.rs
  ├── integration/       # Shell scripts
  │   ├── test-protocol.sh
  │   ├── test-security.sh
  │   └── test-performance.sh
  └── fixtures/          # Test data
      ├── valid-requests.json
      ├── invalid-requests.json
      └── test-configs/
```

### CI/CD Integration
- [ ] Run unit tests on every commit
- [ ] Run integration tests on PR
- [ ] Performance regression tests weekly
- [ ] Multi-distribution tests before release

---

## Monitoring and Observability

### Metrics to Track

#### Agent Metrics
- Request count by method
- Request latency (p50, p95, p99)
- Error rate by error code
- Memory usage over time
- File descriptor count
- CPU usage

#### Transport Metrics
- Connection establishment time
- Reconnection attempts
- Message send/receive count
- Message size distribution
- Socket errors

#### Security Metrics
- Path restriction violations
- Command whitelist rejections
- Timeout events
- File size limit exceeded
- Authentication failures (if added)

### Logging Best Practices
- [ ] Structured logging (JSON format)
- [ ] Log levels properly used (trace/debug/info/warn/error)
- [ ] Sensitive data redacted (file content, command output)
- [ ] Request ID in all logs for correlation
- [ ] Performance metrics logged

---

## Test Data Sets

### Valid Test Data
```json
{
  "valid_file_paths": [
    "/tmp/test.txt",
    "/var/log/syslog",
    "/etc/hostname"
  ],
  "valid_commands": [
    {"command": "echo", "args": ["test"]},
    {"command": "ls", "args": ["-la", "/tmp"]},
    {"command": "cat", "args": ["/etc/hostname"]}
  ]
}
```

### Invalid Test Data
```json
{
  "invalid_file_paths": [
    "/tmp/../etc/passwd",
    "/tmp/link_to_etc_shadow",
    "/etc/shadow",
    "../../../../root/.ssh/id_rsa",
    "/tmp/test\0.txt",
    "A" * 10000
  ],
  "malformed_json": [
    "{invalid",
    "{'single': 'quotes'}",
    "{\"truncated\":",
    "not json at all"
  ]
}
```

---

## Issue Severity Classification

### Critical (P0)
- Security vulnerabilities
- Data corruption
- Service crashes/hangs
- Privilege escalation

### High (P1)
- Incorrect data returned
- Performance degradation > 2x
- Memory leaks
- Connection failures

### Medium (P2)
- Poor error messages
- Minor performance issues
- UI inconsistencies
- Documentation errors

### Low (P3)
- Cosmetic issues
- Nice-to-have features
- Code style issues
- Optimization opportunities

---

## Pre-Release Checklist

Before declaring testing complete:

### Functionality
- [ ] All 10 protocol methods tested on 5+ distributions
- [ ] Security tests passed (no bypasses found)
- [ ] Performance benchmarks meet targets
- [ ] 24-hour stability test passed

### Quality
- [ ] No P0/P1 issues open
- [ ] All P2 issues documented
- [ ] Code reviewed
- [ ] Documentation complete and accurate

### Integration
- [ ] UI integration tested
- [ ] Multi-VM scenarios tested
- [ ] Reconnection scenarios tested
- [ ] Error handling verified

### Operations
- [ ] Installation tested on all platforms
- [ ] Upgrade path tested
- [ ] Monitoring/logging verified
- [ ] Troubleshooting guide validated

---

## Post-Testing Analysis

After testing, document:

### What Worked Well
- Areas that exceeded expectations
- Particularly robust components
- Good design decisions

### What Needs Improvement
- Fragile areas
- Performance bottlenecks
- Confusing error messages
- Missing features

### Lessons Learned
- Testing approaches that were effective
- Issues found late that should have been caught earlier
- Process improvements for future phases

### Technical Debt
- Code that needs refactoring
- Tests that need to be automated
- Documentation that needs updating

---

## Resources

### Testing Tools
- `socat`: Socket communication
- `jq`: JSON parsing and formatting
- `valgrind`: Memory leak detection
- `strace`: System call tracing
- `ltrace`: Library call tracing
- `heaptrack`: Memory profiling
- `perf`: Performance profiling

### Reference Documentation
- `/home/fraziersystems/Documents/projects/kvm-manager/guest-agent/PROTOCOL.md`
- `/home/fraziersystems/Documents/projects/kvm-manager/guest-agent/INSTALL.md`
- `/home/fraziersystems/Documents/projects/kvm-manager/guest-agent/TESTING.md`
- `/home/fraziersystems/Documents/projects/kvm-manager/guest-agent/COMPREHENSIVE_TEST_PLAN.md`

### Support Channels
- Project Coordinator: Overall testing coordination
- DevOps Agent: ISO deployment and infrastructure
- Backend Agent: Integration issues
- Architecture Agent: Protocol and design questions

---

**This document should be referenced throughout testing to ensure comprehensive coverage of all risk areas.**
