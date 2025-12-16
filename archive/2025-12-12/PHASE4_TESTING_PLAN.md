# Phase 4 Guest Agent - Testing & Validation Plan

**Date**: December 10, 2025
**Status**: Testing Phase - Development Complete
**Coordinator**: Project Coordinator Agent

---

## Executive Summary

Phase 4 development is **COMPLETE** and significantly ahead of schedule (3-4x faster than planned). All components are built, integrated, and ready for systematic testing. The ISO is deployed and we're ready to begin validation.

**Current State**:
- ✅ Guest Agent binary built (1.6 MB, production-ready)
- ✅ ISO created and deployed to `/var/lib/libvirt/images/`
- ✅ Backend integration complete (GuestAgentService + Tauri commands)
- ✅ Frontend UI complete (GuestInfo component)
- ✅ Installation scripts for all major distributions
- ✅ Documentation complete (INSTALL.md, PROTOCOL.md, DEPLOYMENT_CHECKLIST.md)

**Next Phase**: Systematic testing and validation

---

## Testing Strategy

### Phase 1: Initial Validation (Priority: CRITICAL)
**Timeline**: 2-4 hours
**Goal**: Verify core functionality on a single test VM

### Phase 2: Multi-Distribution Testing (Priority: HIGH)
**Timeline**: 2-3 days
**Goal**: Validate across all major Linux distributions

### Phase 3: Performance & Reliability (Priority: MEDIUM)
**Timeline**: 1-2 days
**Goal**: Test edge cases, performance, and multi-VM scenarios

### Phase 4: Packaging & Distribution (Priority: MEDIUM)
**Timeline**: 2-3 days
**Goal**: Create distributable packages (.deb, .rpm, .apk)

---

## Phase 1: Initial Validation

### Test VM Specifications

**Recommended Test Configuration**:
```
Name: guest-agent-test-ubuntu
OS: Ubuntu 24.04 LTS
Memory: 2048 MB
CPU: 2 cores
Disk: 20 GB
Firmware: UEFI (Q35 chipset)
Network: Default NAT
virtio-serial: Enabled (automatic in new VMs)
```

### Test Procedures

#### 1.1 VM Creation & Agent Installation
**Estimated Time**: 30 minutes

**Steps**:
1. Create test VM through KVM Manager UI
2. Install Ubuntu 24.04 LTS
3. Verify virtio-serial channel in VM XML:
   ```bash
   virsh dumpxml guest-agent-test-ubuntu | grep -A5 "org.kvmmanager.agent"
   ```
4. Mount guest agent ISO from VM details page
5. SSH into VM and run installer:
   ```bash
   sudo bash /media/cdrom/install-debian.sh
   ```
6. Verify agent service is running:
   ```bash
   systemctl status kvmmanager-agent
   journalctl -u kvmmanager-agent -f
   ```
7. Eject ISO from UI
8. Refresh VM details page

**Expected Result**: Guest information card shows "Agent Available" with green indicator

**Acceptance Criteria**:
- [ ] Agent service starts without errors
- [ ] Agent connects to virtio-serial device
- [ ] UI shows agent as available
- [ ] No error messages in logs

#### 1.2 System Information Accuracy
**Estimated Time**: 15 minutes

**Test Cases**:
| Information | Verification Method | Pass/Fail |
|-------------|---------------------|-----------|
| OS Type | `cat /etc/os-release` | ☐ |
| Hostname | `hostname` | ☐ |
| Kernel Version | `uname -r` | ☐ |
| CPU Count | `nproc` | ☐ |
| Memory Amount | `free -h` | ☐ |
| Uptime | `uptime` | ☐ |

**Acceptance Criteria**:
- [ ] All fields match actual VM values
- [ ] Data updates on page refresh
- [ ] No timeout errors

#### 1.3 Network Information Accuracy
**Estimated Time**: 15 minutes

**Test Cases**:
1. Verify all network interfaces are listed
2. Verify IP addresses match `ip addr show`
3. Verify MAC addresses match
4. Verify interface state (up/down) is correct
5. Test with multiple network interfaces (if available)

**Commands for Verification**:
```bash
ip addr show
ip link show
```

**Acceptance Criteria**:
- [ ] All interfaces displayed
- [ ] IPv4 addresses correct
- [ ] IPv6 addresses correct (if configured)
- [ ] MAC addresses match
- [ ] Interface states accurate

#### 1.4 Disk Usage Reporting
**Estimated Time**: 15 minutes

**Test Cases**:
1. Verify all mounted filesystems are shown
2. Verify disk usage percentages match `df -h`
3. Test with multiple filesystems (if available)

**Commands for Verification**:
```bash
df -h
df -B1  # For exact byte counts
```

**Acceptance Criteria**:
- [ ] All filesystems listed
- [ ] Total/used/available bytes accurate
- [ ] Percentages correct
- [ ] Mount points shown

#### 1.5 Command Execution (Optional Feature)
**Estimated Time**: 20 minutes

**Test Cases**:
```javascript
// Basic command
executeGuestCommand("guest-agent-test-ubuntu", "ls", ["-la", "/tmp"], 30)

// Command with output
executeGuestCommand("guest-agent-test-ubuntu", "cat", ["/etc/hostname"], 30)

// Command with stderr
executeGuestCommand("guest-agent-test-ubuntu", "ls", ["/nonexistent"], 30)

// Long-running command (test timeout)
executeGuestCommand("guest-agent-test-ubuntu", "sleep", ["60"], 5)
```

**Acceptance Criteria**:
- [ ] Stdout captured correctly
- [ ] Stderr captured for errors
- [ ] Exit codes returned properly
- [ ] Timeout enforced
- [ ] No shell injection possible

#### 1.6 File Operations
**Estimated Time**: 20 minutes

**Test Cases**:

**File Read**:
```javascript
// Read allowed file
readGuestFile("guest-agent-test-ubuntu", "/etc/hostname")

// Read from allowed directory
readGuestFile("guest-agent-test-ubuntu", "/tmp/test.txt")

// Attempt directory traversal (should fail)
readGuestFile("guest-agent-test-ubuntu", "/tmp/../../etc/shadow")
```

**File Write**:
```javascript
// Write to allowed directory
writeGuestFile("guest-agent-test-ubuntu", "/tmp/test.txt", "Hello World", false)

// Verify with read
readGuestFile("guest-agent-test-ubuntu", "/tmp/test.txt")

// Attempt write outside allowed paths (should fail)
writeGuestFile("guest-agent-test-ubuntu", "/etc/test.txt", "Bad", false)
```

**Acceptance Criteria**:
- [ ] Can read files in allowed paths
- [ ] Can write files in allowed paths
- [ ] Path restrictions enforced
- [ ] Directory traversal blocked
- [ ] File size limits enforced (test with >10MB file)

#### 1.7 Power Operations
**Estimated Time**: 20 minutes

**Test Cases**:
1. **Graceful Reboot**:
   - Execute `guestAgentReboot("guest-agent-test-ubuntu", false)`
   - VM should reboot cleanly
   - Agent should reconnect after reboot

2. **Graceful Shutdown**:
   - Execute `guestAgentShutdown("guest-agent-test-ubuntu", false)`
   - VM should shutdown cleanly

**Acceptance Criteria**:
- [ ] Graceful shutdown works
- [ ] Graceful reboot works
- [ ] Agent reconnects after reboot
- [ ] No data loss or corruption

#### 1.8 Reconnection & Reliability
**Estimated Time**: 30 minutes

**Test Cases**:
1. **VM Pause/Resume**:
   - Pause VM via KVM Manager
   - Resume VM
   - Verify agent reconnects

2. **VM Reboot**:
   - Reboot VM via guest command
   - Verify agent reconnects
   - Verify data is fresh (uptime should reset)

3. **Service Restart**:
   - In VM: `systemctl restart kvmmanager-agent`
   - Verify UI updates status
   - Verify reconnection

4. **Host Service Restart**:
   - On host: Restart KVM Manager application
   - Verify agent connection restored

**Acceptance Criteria**:
- [ ] Agent reconnects after pause/resume
- [ ] Agent reconnects after VM reboot
- [ ] Agent reconnects after service restart
- [ ] UI updates status correctly
- [ ] No manual intervention required

### Phase 1 Success Criteria

**All of the following must pass**:
- ✅ Agent installs and starts successfully
- ✅ All system information displays correctly
- ✅ Network interfaces shown accurately
- ✅ Disk usage reports match actual values
- ✅ Agent reconnects after VM operations
- ✅ Security restrictions enforced (path validation)
- ✅ No critical bugs or errors

**If Phase 1 passes**: Proceed to Phase 2 (Multi-distribution testing)
**If Phase 1 fails**: Document issues, fix, and re-test

---

## Phase 2: Multi-Distribution Testing

### Test Matrix

| Distribution | Version | Priority | Installer Script | Service Manager | Status |
|--------------|---------|----------|------------------|-----------------|--------|
| Ubuntu | 24.04 LTS | CRITICAL | install-debian.sh | systemd | ☐ |
| Ubuntu | 22.04 LTS | HIGH | install-debian.sh | systemd | ☐ |
| Debian | 12 (Bookworm) | HIGH | install-debian.sh | systemd | ☐ |
| Fedora | 40 | MEDIUM | install-rhel.sh | systemd | ☐ |
| Rocky Linux | 9 | MEDIUM | install-rhel.sh | systemd | ☐ |
| Alpine Linux | 3.19 | LOW | install-alpine.sh | OpenRC | ☐ |

### Test Procedure Per Distribution

**For each distribution in the matrix**:

1. **VM Creation**:
   - Name: `guest-agent-test-{distro}`
   - Specs: 2GB RAM, 2 CPU, 20GB disk
   - Install OS from official ISO

2. **Agent Installation**:
   - Mount guest agent ISO
   - Run appropriate installer script
   - Verify service starts

3. **Functionality Testing** (abbreviated):
   - ✅ System info displays
   - ✅ Network info accurate
   - ✅ Disk usage correct
   - ✅ Agent reconnects after reboot

4. **Distribution-Specific Checks**:
   - Verify service manager integration (systemd/OpenRC)
   - Check log locations
   - Test distro-specific features

5. **Documentation**:
   - Note any distribution-specific issues
   - Update INSTALL.md if needed
   - Record compatibility notes

### Distribution-Specific Notes

#### Ubuntu/Debian
- Package manager: apt
- Service manager: systemd
- Expected challenges: None (baseline)
- Test both LTS versions

#### Fedora/RHEL/Rocky
- Package manager: dnf/yum
- Service manager: systemd
- Expected challenges: SELinux contexts
- RHEL requires subscription (use Rocky)

#### Alpine Linux
- Package manager: apk
- Service manager: OpenRC
- Expected challenges: musl libc compatibility, OpenRC service
- Critical for minimal footprint deployments

### Phase 2 Success Criteria

**For each distribution**:
- ✅ Installation script completes without errors
- ✅ Service starts and enables on boot
- ✅ Agent connects and provides data
- ✅ Basic functionality verified
- ✅ Distribution documented in INSTALL.md

**Overall**:
- ✅ At least 4 out of 6 distributions pass
- ✅ All CRITICAL and HIGH priority distros pass
- ✅ Any failures documented with workarounds

---

## Phase 3: Performance & Reliability Testing

### Test Scenarios

#### 3.1 Multi-VM Scenario
**Goal**: Verify agent works with multiple VMs simultaneously

**Test**:
1. Create 5 VMs with guest agent installed
2. Start all VMs
3. Verify all agents connect
4. Refresh VM list multiple times
5. Monitor host system resources

**Metrics**:
- [ ] All agents connect successfully
- [ ] No connection conflicts
- [ ] Response time < 100ms per VM
- [ ] Host CPU usage reasonable
- [ ] No memory leaks

#### 3.2 Long-Running Stability
**Goal**: Verify agent stability over time

**Test**:
1. Leave test VM running for 24 hours
2. Monitor agent logs
3. Check for memory leaks
4. Verify continuous connectivity

**Metrics**:
- [ ] Agent runs continuously
- [ ] Memory usage stable (< 10MB)
- [ ] CPU usage minimal (< 0.5%)
- [ ] No connection drops
- [ ] Logs show no errors

#### 3.3 Stress Testing
**Goal**: Test agent under load

**Test**:
1. Execute 100 rapid `ping` requests
2. Read large files (near 10MB limit)
3. Execute multiple concurrent commands
4. Monitor performance degradation

**Metrics**:
- [ ] No crashes or hangs
- [ ] Timeouts enforced correctly
- [ ] Resource limits respected
- [ ] Error handling graceful

#### 3.4 Edge Cases
**Goal**: Test unusual scenarios

**Test Cases**:
- [ ] VM with no network interfaces
- [ ] VM with many network interfaces (10+)
- [ ] VM with very high disk usage (>95%)
- [ ] VM with special characters in hostname
- [ ] VM during high load (stress -c 4)
- [ ] Rapid VM start/stop cycles

### Phase 3 Success Criteria

- ✅ Agent stable under normal conditions
- ✅ Performance acceptable with multiple VMs
- ✅ Edge cases handled gracefully
- ✅ No memory leaks or resource exhaustion
- ✅ Error handling comprehensive

---

## Phase 4: Packaging & Distribution

### 4.1 Debian/Ubuntu Package (.deb)

**Tasks**:
1. Create package structure:
   ```
   kvmmanager-agent_0.4.0_amd64/
   ├── DEBIAN/
   │   ├── control
   │   ├── postinst
   │   └── prerm
   ├── usr/bin/
   │   └── kvmmanager-agent
   ├── etc/kvmmanager-agent/
   │   └── config.json
   └── etc/systemd/system/
       └── kvmmanager-agent.service
   ```

2. Build package:
   ```bash
   dpkg-deb --build kvmmanager-agent_0.4.0_amd64
   ```

3. Test installation:
   ```bash
   sudo dpkg -i kvmmanager-agent_0.4.0_amd64.deb
   systemctl status kvmmanager-agent
   ```

**Acceptance Criteria**:
- [ ] Package installs cleanly
- [ ] Service auto-starts
- [ ] Configuration preserved on upgrade
- [ ] Uninstall removes all files

### 4.2 Fedora/RHEL Package (.rpm)

**Tasks**:
1. Create spec file: `kvmmanager-agent.spec`
2. Build package:
   ```bash
   rpmbuild -ba kvmmanager-agent.spec
   ```
3. Test installation:
   ```bash
   sudo dnf install kvmmanager-agent-0.4.0-1.x86_64.rpm
   ```

**Acceptance Criteria**:
- [ ] Package installs cleanly
- [ ] Service auto-starts
- [ ] SELinux contexts correct
- [ ] Uninstall removes all files

### 4.3 Alpine Package (.apk)

**Tasks**:
1. Create APKBUILD file
2. Build package:
   ```bash
   abuild -r
   ```
3. Test installation:
   ```bash
   sudo apk add kvmmanager-agent-0.4.0-r0.apk
   ```

**Acceptance Criteria**:
- [ ] Package installs cleanly
- [ ] OpenRC service auto-starts
- [ ] Minimal dependencies
- [ ] Uninstall removes all files

### Phase 4 Success Criteria

- ✅ All package formats build successfully
- ✅ Packages install without errors
- ✅ Services auto-start on installation
- ✅ Packages follow distribution standards
- ✅ Packages uploaded to releases

---

## Test Execution Schedule

### Week 1 (Current)
**Days 1-2** (Dec 10-11):
- ☐ Phase 1: Initial validation on Ubuntu 24.04
- ☐ Fix any critical issues found
- ☐ Document results

**Days 3-5** (Dec 12-14):
- ☐ Phase 2: Multi-distribution testing
- ☐ Test on all 6 distributions
- ☐ Document distribution-specific notes

### Week 2
**Days 1-2** (Dec 15-16):
- ☐ Phase 3: Performance and reliability testing
- ☐ Multi-VM testing
- ☐ Long-running stability test

**Days 3-5** (Dec 17-19):
- ☐ Phase 4: Package creation
- ☐ Build .deb, .rpm, .apk packages
- ☐ Test package installations

---

## Resource Requirements

### Test VMs Required
- 1x Ubuntu 24.04 (primary test)
- 1x Ubuntu 22.04
- 1x Debian 12
- 1x Fedora 40
- 1x Rocky Linux 9
- 1x Alpine Linux 3.19
- 5x Additional VMs (multi-VM testing)

**Total**: ~11 VMs (can be created/destroyed as needed)

### Disk Space
- OS ISOs: ~6 GB
- VMs: ~200 GB (11 VMs × ~20 GB each)
- Build artifacts: ~100 MB

**Total**: ~210 GB

### Time Allocation
- **Phase 1**: 4 hours (1 person)
- **Phase 2**: 2-3 days (can parallelize)
- **Phase 3**: 2 days
- **Phase 4**: 2-3 days

**Total**: 6-8 days of testing work

---

## Issue Tracking

### Issue Template

```markdown
## Issue: [Brief Description]

**Severity**: Critical / High / Medium / Low
**Component**: Guest Agent / Backend / Frontend / Installer / Documentation
**Distribution**: Ubuntu 24.04 / Debian 12 / etc.

**Description**:
[Detailed description of the issue]

**Steps to Reproduce**:
1. Step 1
2. Step 2
3. Step 3

**Expected Behavior**:
[What should happen]

**Actual Behavior**:
[What actually happens]

**Logs**:
```
[Relevant log excerpts]
```

**Screenshots**:
[If applicable]

**Workaround**:
[If available]

**Fix Status**: Not Started / In Progress / Fixed / Won't Fix
```

### Issue Severity Definitions

**Critical**:
- Agent doesn't start
- Complete loss of functionality
- Security vulnerability
- Data corruption

**High**:
- Major feature not working
- Incorrect data displayed
- Installation fails
- Service doesn't auto-start

**Medium**:
- Minor feature issue
- UI inconsistency
- Performance degradation
- Non-critical error messages

**Low**:
- Cosmetic issues
- Documentation errors
- Enhancement requests

---

## Success Metrics

### Overall Phase 4 Success

**Must Have (MVP)**:
- ✅ Works on Ubuntu 22.04/24.04 (CRITICAL)
- ✅ Works on Debian 12 (HIGH)
- ✅ All core features functional
- ✅ No critical bugs
- ✅ Documentation complete

**Should Have**:
- ✅ Works on Fedora/RHEL (MEDIUM)
- ✅ Works on Alpine (MEDIUM)
- ✅ .deb package available
- ✅ Performance acceptable

**Nice to Have**:
- ✅ .rpm and .apk packages
- ✅ Extensive testing on all distributions
- ✅ Perfect test coverage

### Quality Gates

**Gate 1: Phase 1 Complete**
- All initial validation tests pass
- No critical bugs
- Ready for broader testing

**Gate 2: Phase 2 Complete**
- At least 4/6 distributions working
- All CRITICAL/HIGH priority distros pass
- Installation documentation verified

**Gate 3: Phase 3 Complete**
- Performance acceptable
- No stability issues
- Edge cases handled

**Gate 4: Production Ready**
- All phases complete
- Packages available
- Documentation finalized
- No open critical/high bugs

---

## Coordination Points

### With DevOps Agent
**Responsibilities**:
- VM creation and management
- ISO deployment verification
- libvirt configuration
- Service monitoring
- Performance testing

**Deliverables**:
- Test VMs created
- VM configurations documented
- Performance metrics collected

### With Guest Agent Specialist
**Responsibilities**:
- Agent functionality verification
- Bug investigation
- Protocol testing
- Security validation
- Performance optimization

**Deliverables**:
- Functionality test results
- Bug fixes (if needed)
- Security audit
- Performance report

### With Project Owner
**Communication**:
- Daily: Brief status updates during active testing
- Weekly: Comprehensive test report
- As-needed: Blocker escalation

**Decisions Needed**:
- Severity prioritization
- Distribution support scope
- Release timeline
- Package distribution strategy

---

## Risk Management

### Identified Risks

**Risk 1: Distribution Compatibility**
- **Probability**: Medium
- **Impact**: Medium
- **Mitigation**: Test on all major distributions early
- **Contingency**: Document unsupported distros, provide manual install

**Risk 2: virtio-serial Configuration**
- **Probability**: Low
- **Impact**: High
- **Mitigation**: Automatic configuration for new VMs, clear docs for existing
- **Contingency**: Provide XML examples and troubleshooting guide

**Risk 3: Performance at Scale**
- **Probability**: Low
- **Impact**: Medium
- **Mitigation**: Performance testing with 50+ VMs
- **Contingency**: Optimize if needed, document limits

**Risk 4: Security Issues**
- **Probability**: Low
- **Impact**: Critical
- **Mitigation**: Security review, path validation testing
- **Contingency**: Immediate fix and patch release

### Risk Monitoring

**Daily**:
- Check for new issues during testing
- Monitor test results
- Track blocker count

**Weekly**:
- Review risk register
- Update mitigation status
- Report to project owner

---

## Next Actions (Immediate)

### For DevOps Agent
1. **Create initial test VM**:
   - Name: guest-agent-test-ubuntu
   - OS: Ubuntu 24.04
   - Specs: 2GB RAM, 2 CPU, 20GB disk
   - Verify virtio-serial configuration

2. **Verify ISO deployment**:
   - Confirm ISO at `/var/lib/libvirt/images/kvmmanager-guest-agent.iso`
   - Verify permissions (644, owner: libvirt-qemu)

### For Guest Agent Specialist
1. **Prepare test procedures**:
   - Review Phase 1 test cases
   - Set up test scripts/automation
   - Prepare verification commands

2. **Monitor first installation**:
   - Watch for any errors
   - Verify all logs
   - Test all functionality

### For Project Coordinator (Current)
1. **Launch specialist agents**:
   - Activate DevOps Agent
   - Activate Guest Agent Specialist
   - Provide this testing plan

2. **Track progress**:
   - Monitor test execution
   - Document results
   - Escalate blockers

3. **Daily updates**:
   - Collect status from specialists
   - Report to project owner
   - Update testing plan as needed

---

## Appendices

### Appendix A: Quick Reference Commands

**VM Management**:
```bash
# List VMs
virsh list --all

# VM XML
virsh dumpxml <vm-name>

# Start/Stop
virsh start <vm-name>
virsh shutdown <vm-name>
```

**Agent Management (Guest)**:
```bash
# Status
systemctl status kvmmanager-agent

# Logs
journalctl -u kvmmanager-agent -f

# Restart
systemctl restart kvmmanager-agent
```

**Debugging**:
```bash
# Check virtio-serial device
ls -la /dev/vport0p1
ls -la /dev/virtio-ports/

# Test connectivity
echo '{"jsonrpc":"2.0","method":"ping","params":{},"id":1}' | sudo tee /dev/vport0p1
```

### Appendix B: Test Data

**Sample test files**:
- `/tmp/test-small.txt` - Small file for read/write tests
- `/tmp/test-large.bin` - Large file (9MB) for size limit tests
- `/tmp/test-unicode.txt` - File with unicode characters

**Sample commands**:
- `ls -la /tmp` - Basic command
- `cat /etc/hostname` - Read file
- `df -h` - Disk usage
- `ip addr show` - Network info

---

**Version**: 1.0
**Last Updated**: December 10, 2025
**Owner**: Project Coordinator Agent
**Status**: Active - Testing Phase Initiated
