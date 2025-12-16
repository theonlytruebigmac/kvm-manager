# Guest Agent Deployment Checklist

## Pre-Deployment

- [x] Guest agent binary built successfully (1.6 MB)
- [x] ISO created with all installation scripts
- [x] Backend integration complete (GuestAgentService)
- [x] Tauri commands registered and working
- [x] Frontend UI implemented (GuestInfo component)
- [x] TypeScript types defined
- [x] Installation documentation created
- [ ] ISO copied to libvirt images directory
- [ ] Test VM created with virtio-serial channel

## Testing Checklist

### Basic Connectivity
- [ ] Create test VM through KVM Manager UI
- [ ] Verify virtio-serial channel in VM XML
- [ ] Mount guest agent ISO to VM
- [ ] Install agent in VM
- [ ] Verify agent service is running
- [ ] Check agent appears as "available" in UI

### System Information
- [ ] Guest OS type displays correctly
- [ ] Hostname shows actual VM hostname
- [ ] OS version matches VM
- [ ] Kernel version correct
- [ ] CPU count accurate
- [ ] Memory amount correct
- [ ] Uptime displays and updates

### Network Information
- [ ] All network interfaces listed
- [ ] IP addresses shown correctly (IPv4)
- [ ] IPv6 addresses shown if configured
- [ ] MAC addresses match
- [ ] Interface state (up/down) correct

### Disk Usage
- [ ] All mounted filesystems shown
- [ ] Disk usage percentages accurate
- [ ] Total/used/available bytes correct
- [ ] Multiple filesystems handled properly

### Command Execution
- [ ] Simple commands execute successfully (ls, cat, etc.)
- [ ] Output captured correctly
- [ ] Stderr captured for errors
- [ ] Exit codes returned properly
- [ ] Timeout enforced
- [ ] Command whitelist works when enabled

### File Operations
- [ ] File read works for allowed paths
- [ ] File write works for allowed paths
- [ ] Path restrictions enforced (reject ../../../etc/passwd)
- [ ] File size limits enforced
- [ ] Binary file handling correct

### Power Operations
- [ ] Graceful shutdown via agent works
- [ ] Graceful reboot via agent works
- [ ] Force shutdown option works
- [ ] Force reboot option works

### Reconnection & Reliability
- [ ] Agent reconnects after VM reboot
- [ ] Agent reconnects after VM pause/resume
- [ ] Agent handles temporary disconnects
- [ ] Multiple VMs with agents work simultaneously
- [ ] Agent status updates in real-time

### Security
- [ ] Cannot read files outside allowed_read_paths
- [ ] Cannot write files outside allowed_write_paths
- [ ] Command whitelist blocks unlisted commands
- [ ] Large file operations timeout/fail gracefully
- [ ] No shell injection possible

## Distribution Testing

### Debian/Ubuntu
- [ ] Ubuntu 22.04 LTS
- [ ] Ubuntu 24.04 LTS
- [ ] Debian 12 (Bookworm)
- [ ] Install script works
- [ ] systemd service starts
- [ ] Agent functions correctly

### RHEL/Fedora
- [ ] Fedora 40
- [ ] RHEL 9 / Rocky Linux 9
- [ ] Install script works
- [ ] systemd service starts
- [ ] Agent functions correctly

### Alpine Linux
- [ ] Alpine 3.19
- [ ] Install script works
- [ ] OpenRC service starts
- [ ] Agent functions correctly

## Performance Testing
- [ ] Agent CPU usage < 1% idle
- [ ] Agent memory usage < 10 MB
- [ ] API response times < 100ms
- [ ] No memory leaks over 24 hours
- [ ] Works with 50+ VMs simultaneously

## Documentation
- [x] INSTALL.md created with full instructions
- [x] PROTOCOL.md exists with API spec
- [x] README.md in guest-agent/ directory
- [ ] Main README.md updated with agent features
- [ ] Screenshots added to docs
- [ ] Troubleshooting guide tested

## Packaging

### .deb (Debian/Ubuntu)
- [ ] Package structure created
- [ ] Control file configured
- [ ] Pre/post install scripts
- [ ] Build with dpkg-deb
- [ ] Test installation on Ubuntu
- [ ] Test installation on Debian
- [ ] Upload to releases

### .rpm (RHEL/Fedora)
- [ ] Spec file created
- [ ] Build with rpmbuild
- [ ] Test installation on Fedora
- [ ] Test installation on RHEL/Rocky
- [ ] Upload to releases

### .apk (Alpine)
- [ ] APKBUILD created
- [ ] Build with abuild
- [ ] Test installation on Alpine
- [ ] Upload to releases

## Release Preparation
- [ ] Version number updated in Cargo.toml
- [ ] CHANGELOG.md entry created
- [ ] Git tag created (e.g., v0.4.0-agent)
- [ ] Release notes written
- [ ] Binaries uploaded to GitHub releases
- [ ] Announcement prepared

## Post-Deployment
- [ ] Monitor for issues in production
- [ ] Gather user feedback
- [ ] Update documentation based on feedback
- [ ] Plan next iteration improvements

---

## Current Status: 🟡 Development Complete, Testing In Progress

**Completed**: Binary build, ISO creation, backend integration, frontend UI, documentation
**Next**: Deploy ISO and begin systematic testing

**Blocker**: Need to copy ISO to `/var/lib/libvirt/images/` with elevated permissions

**Solution**:
```bash
sudo cp guest-agent/kvmmanager-guest-agent.iso /var/lib/libvirt/images/
sudo chmod 644 /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```
