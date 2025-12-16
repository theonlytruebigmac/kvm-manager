# Guest Agent Testing Checklist
**Quick Reference for Phase 4 Testing**
**Date**: December 10, 2025

---

## Pre-Flight Checklist

### Environment Setup
- [ ] ISO deployed: `sudo cp guest-agent/kvmmanager-guest-agent.iso /var/lib/libvirt/images/`
- [ ] ISO permissions: `sudo chmod 644 /var/lib/libvirt/images/kvmmanager-guest-agent.iso`
- [ ] KVM Manager running
- [ ] Test VM created with virtio-serial channel
- [ ] Tools installed: `socat`, `jq`

---

## Day 1: Basic Functionality Testing

### Morning: Setup and Protocol Methods

#### Setup Test VM (30 min)
- [ ] Create Ubuntu 22.04 test VM
- [ ] Verify virtio-serial in VM XML: `virsh dumpxml <vm> | grep kvmmanager`
- [ ] Boot VM and verify device: `ls -l /dev/virtio-ports/org.kvmmanager.agent.0`
- [ ] Mount ISO via KVM Manager UI
- [ ] Install agent: `sudo bash /media/cdrom/install-debian.sh`
- [ ] Verify service: `systemctl status kvmmanager-agent`
- [ ] Check logs: `journalctl -u kvmmanager-agent -n 20`

#### Protocol Testing (2 hours)
Use the test script or manual testing with socat.

**Test each method**:
- [ ] **ping**: Basic connectivity
  ```bash
  echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:/var/lib/libvirt/qemu/channel/target/<vm>.org.kvmmanager.agent.0
  ```
  Expected: `{"jsonrpc":"2.0","result":{"pong":true,...},"id":1}`

- [ ] **get_agent_info**: Version and capabilities
  - Verify version matches binary
  - Check all 10 methods in capabilities

- [ ] **get_system_info**: OS details
  - Compare with `cat /etc/os-release`
  - Verify hostname: `hostname`
  - Check CPU count: `nproc`
  - Verify memory: `free -k`

- [ ] **get_network_info**: Network interfaces
  - Compare with `ip addr`
  - Verify all interfaces present
  - Check IP addresses match

- [ ] **get_disk_usage**: Filesystem usage
  - Compare with `df -h`
  - Verify all mount points
  - Check percentages accurate

- [ ] **exec_command**: Command execution
  - Simple: `echo "test"`
  - With args: `ls -la /tmp`
  - With error: `ls /nonexistent` (check stderr)
  - Verify exit codes

- [ ] **file_read**: Read files
  - Read `/etc/hostname`
  - Verify content matches actual file
  - Test base64 encoding

- [ ] **file_write**: Write files
  - Write to `/tmp/test.txt`
  - Verify file created
  - Read back and compare

- [ ] **shutdown**: Graceful shutdown (careful!)
  - Test on disposable VM only
  - Verify graceful shutdown

- [ ] **reboot**: Graceful reboot
  - Verify agent reconnects after reboot
  - Check all methods still work

### Afternoon: UI Integration Testing (2 hours)

#### KVM Manager UI
- [ ] Open VM Details page
- [ ] Verify "Guest Information" section visible
- [ ] Check agent status shows "Available"
- [ ] System info displays correctly:
  - [ ] OS name and version
  - [ ] Hostname
  - [ ] Kernel version
  - [ ] CPU count
  - [ ] Memory
  - [ ] Uptime

- [ ] Network interfaces display:
  - [ ] All interfaces listed
  - [ ] IP addresses correct
  - [ ] MAC addresses match
  - [ ] State indicators accurate

- [ ] Disk usage displays:
  - [ ] All filesystems shown
  - [ ] Usage percentages correct
  - [ ] Progress bars color-coded
  - [ ] GB values accurate

- [ ] Auto-refresh working:
  - [ ] Create large file in VM
  - [ ] Watch disk usage update (30s interval)
  - [ ] Verify uptime increments (5s interval)

- [ ] ISO management:
  - [ ] Mount ISO button works
  - [ ] Eject ISO button works
  - [ ] ISO state reflected in UI

---

## Day 2: Multi-Distribution Testing

### Distribution Test Matrix

For each distribution, perform quick validation:

#### Ubuntu 24.04 (1 hour)
- [ ] Create VM
- [ ] Install agent from ISO
- [ ] Verify all 10 methods work
- [ ] Check UI displays correctly
- [ ] Test reboot and reconnection

#### Debian 12 (1 hour)
- [ ] Create VM
- [ ] Install agent from ISO
- [ ] Verify all 10 methods work
- [ ] Check UI displays correctly
- [ ] Test reboot and reconnection

#### Fedora 40 (1 hour)
- [ ] Create VM
- [ ] Install agent from ISO
- [ ] Verify all 10 methods work
- [ ] Check SELinux compatibility (no AVCs)
- [ ] Test reboot and reconnection

#### RHEL 9 / Rocky Linux 9 (1 hour)
- [ ] Create VM
- [ ] Install agent from ISO
- [ ] Verify all 10 methods work
- [ ] Check SELinux compatibility
- [ ] Test reboot and reconnection

#### Alpine Linux 3.19 (1 hour)
- [ ] Create VM
- [ ] Install agent from ISO (OpenRC script)
- [ ] Verify all 10 methods work
- [ ] Check resource usage (should be minimal)
- [ ] Test reboot and reconnection

### Afternoon: Security Testing (2 hours)

#### Path Restrictions
- [ ] Configure restrictive paths in `/etc/kvmmanager-agent/config.json`:
  ```json
  {
    "allowed_read_paths": ["/tmp"],
    "allowed_write_paths": ["/tmp/kvmmanager"]
  }
  ```
- [ ] Restart agent
- [ ] Attempt to read `/etc/passwd` → Should fail
- [ ] Attempt to read `/tmp/../etc/passwd` → Should fail
- [ ] Attempt to write `/etc/test` → Should fail
- [ ] Verify errors logged

#### Command Whitelist
- [ ] Enable whitelist:
  ```json
  {
    "command_whitelist_enabled": true,
    "command_whitelist": ["ls", "cat"]
  }
  ```
- [ ] Restart agent
- [ ] Execute `ls` → Should work
- [ ] Execute `rm` → Should fail
- [ ] Verify rejection message clear

#### Shell Injection Prevention
- [ ] Execute: `echo "test; whoami"`
- [ ] Verify output is literal string
- [ ] No command chaining occurs

#### Timeouts and Limits
- [ ] Set `command_timeout_seconds: 5`
- [ ] Execute `sleep 60`
- [ ] Verify timeout at 5 seconds
- [ ] Agent still responsive

---

## Day 3: Reliability and Performance

### Morning: Reliability Testing (2 hours)

#### Reconnection Scenarios
- [ ] **Pause/Resume**:
  - `virsh suspend <vm>`
  - Wait 10 seconds
  - `virsh resume <vm>`
  - Verify agent reconnects
  - Test methods work

- [ ] **VM Reboot**:
  - `virsh reboot <vm>`
  - Verify agent auto-starts
  - Connection restored
  - All methods functional

- [ ] **Agent Restart**:
  - In VM: `systemctl restart kvmmanager-agent`
  - Verify reconnection
  - No data loss

#### Multiple VMs
- [ ] Start 3 VMs with agents
- [ ] Send requests to each
- [ ] Verify no cross-contamination
- [ ] Check all respond correctly

#### Stress Test
- [ ] Send 100 ping requests rapidly
- [ ] Verify all responses received
- [ ] Check for memory leaks
- [ ] Monitor CPU usage

### Afternoon: Performance Testing (2 hours)

#### Latency Measurement
- [ ] Run benchmark script (see Appendix A in test plan)
- [ ] Record results:
  - Average ping latency: _____ ms (target: < 10ms)
  - get_system_info latency: _____ ms (target: < 50ms)
  - Throughput: _____ req/sec (target: > 50)

#### Resource Usage
- [ ] Idle CPU usage: _____ % (target: < 0.5%)
- [ ] Idle memory: _____ MB (target: < 10 MB)
- [ ] Under load CPU: _____ % (target: < 10%)
- [ ] Under load memory: _____ MB (target: < 20 MB)

#### Startup Time
- [ ] Restart service 5 times
- [ ] Measure time to first successful ping
- [ ] Average: _____ ms (target: < 500ms)

#### 24-Hour Stability (Optional)
- [ ] Install agent on test VM
- [ ] Run continuous requests (1 per minute)
- [ ] Check after 24 hours:
  - [ ] Still running
  - [ ] No crashes in logs
  - [ ] Memory usage stable
  - [ ] All methods working

---

## Issue Tracking

### Critical Issues (Block Release)
| # | Description | Found In | Status |
|---|-------------|----------|--------|
|   |             |          |        |

### High Priority Issues
| # | Description | Found In | Status |
|---|-------------|----------|--------|
|   |             |          |        |

### Medium/Low Issues
| # | Description | Found In | Status |
|---|-------------|----------|--------|
|   |             |          |        |

---

## Test Results Summary

### Overall Progress
- [ ] Day 1 Complete (Basic Functionality)
- [ ] Day 2 Complete (Multi-Distribution)
- [ ] Day 3 Complete (Reliability & Performance)

### Test Statistics
- Total Test Cases: 51
- Passed: _____
- Failed: _____
- Blocked: _____
- Pass Rate: _____%

### Distribution Compatibility
| Distribution | Install | Protocol | UI | Reboot | Status |
|--------------|---------|----------|----|---------| ------ |
| Ubuntu 22.04 | [ ] | [ ] | [ ] | [ ] | |
| Ubuntu 24.04 | [ ] | [ ] | [ ] | [ ] | |
| Debian 12    | [ ] | [ ] | [ ] | [ ] | |
| Fedora 40    | [ ] | [ ] | [ ] | [ ] | |
| RHEL 9       | [ ] | [ ] | [ ] | [ ] | |
| Alpine 3.19  | [ ] | [ ] | [ ] | [ ] | |

### Performance Metrics
- Average Latency: _____ ms
- Throughput: _____ req/sec
- Idle Memory: _____ MB
- Startup Time: _____ ms

### Security Validation
- [ ] Path restrictions enforced
- [ ] Command whitelist working
- [ ] Shell injection prevented
- [ ] Timeouts enforced
- [ ] File size limits working

---

## Next Steps Based on Results

### ✅ All Tests Passed
- [ ] Update DEPLOYMENT_CHECKLIST.md
- [ ] Update main README.md
- [ ] Create release notes
- [ ] Tag release version
- [ ] Begin packaging (.deb, .rpm, .apk)

### ⚠️ Minor Issues Found
- [ ] Document issues
- [ ] Create GitHub issues
- [ ] Evaluate severity
- [ ] Plan fixes for next iteration
- [ ] Can ship with known limitations

### ❌ Critical Issues Found
- [ ] Document all issues in detail
- [ ] Prioritize fixes
- [ ] Fix critical blockers
- [ ] Re-test after fixes
- [ ] Delay release if necessary

---

## Sign-Off

**Testing Completed By**: ________________________

**Date**: ________________________

**Recommendation**:
- [ ] **APPROVED** - Ready for production
- [ ] **APPROVED WITH RESERVATIONS** - Minor issues documented
- [ ] **NOT APPROVED** - Critical issues must be fixed

**Notes**:
```



```

---

## Quick Command Reference

### Test VM Setup
```bash
# Create test VM with virtio-serial
# Add to VM XML in <devices> section:
<channel type='unix'>
  <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/VM_NAME.org.kvmmanager.agent.0'/>
  <target type='virtio' name='org.kvmmanager.agent.0'/>
</channel>
```

### Quick Protocol Test
```bash
VM="your-vm-name"
SOCK="/var/lib/libvirt/qemu/channel/target/${VM}.org.kvmmanager.agent.0"

# Ping
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | socat - UNIX-CONNECT:$SOCK

# System info
echo '{"jsonrpc":"2.0","method":"get_system_info","id":2}' | socat - UNIX-CONNECT:$SOCK | jq
```

### Agent Logs
```bash
# View logs
sudo journalctl -u kvmmanager-agent -f

# Service status
sudo systemctl status kvmmanager-agent

# Restart
sudo systemctl restart kvmmanager-agent
```

### Performance Test
```bash
# Quick latency test
time for i in {1..100}; do
  echo '{"jsonrpc":"2.0","method":"ping","id":'$i'}' | socat - UNIX-CONNECT:$SOCK > /dev/null
done
```

---

**Use this checklist daily during testing. Check off items as completed.**
