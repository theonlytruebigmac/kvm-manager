# Quick Deployment Guide - Guest Agent ISO

## TL;DR

Run this one command to deploy the guest agent ISO:

```bash
sudo ./DEPLOY_ISO.sh
```

Or manually:

```bash
sudo cp guest-agent/kvmmanager-guest-agent.iso /var/lib/libvirt/images/
sudo chmod 644 /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

---

## What This Does

Copies the KVM Manager guest agent ISO to `/var/lib/libvirt/images/` so the application can mount it to VMs for agent installation.

---

## Prerequisites

- Guest agent ISO built (already done - located at `guest-agent/kvmmanager-guest-agent.iso`)
- libvirtd service running (already verified)
- sudo access (required for this step)

---

## Option 1: Automated Script (Recommended)

```bash
sudo ./DEPLOY_ISO.sh
```

**What it does**:
- Verifies source ISO exists
- Checks target directory
- Copies ISO with progress
- Sets correct permissions (644)
- Verifies deployment integrity
- Shows next steps

**Time**: < 10 seconds

---

## Option 2: Manual Commands

```bash
# Copy ISO
sudo cp guest-agent/kvmmanager-guest-agent.iso /var/lib/libvirt/images/

# Set permissions
sudo chmod 644 /var/lib/libvirt/images/kvmmanager-guest-agent.iso

# Verify
ls -lh /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

**Time**: < 1 minute

---

## Verification

After deployment, verify the ISO is in place:

```bash
ls -lh /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

Expected output:
```
-rw-r--r-- 1 root root 1.9M Dec 10 20:00 /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

---

## Next Steps

### 1. Create Test VM

Use KVM Manager UI to create a new Ubuntu 22.04 or 24.04 VM.

The UI automatically adds the required virtio-serial channel.

### 2. Mount ISO

In the VM details page:
1. Find the "Guest Information" card
2. Click "Mount Agent ISO"
3. The ISO will be attached to the VM

### 3. Install Agent

Inside the VM (via console or SSH):

```bash
# Mount the CD
sudo mkdir -p /media/cdrom
sudo mount /dev/cdrom /media/cdrom

# Run the appropriate installer
# For Ubuntu/Debian:
sudo bash /media/cdrom/install-debian.sh

# For RHEL/Fedora/Rocky:
sudo bash /media/cdrom/install-rhel.sh

# For Alpine:
sudo sh /media/cdrom/install-alpine.sh
```

### 4. Verify Agent

```bash
# Check service status
systemctl status kvmmanager-agent

# View logs
journalctl -u kvmmanager-agent -f
```

### 5. Check UI

Refresh the VM details page in KVM Manager. You should see:
- Agent status: "Available"
- OS information (actual from guest)
- Network interfaces with IPs
- Disk usage

---

## Troubleshooting

### "Permission denied" when copying

Make sure you're using `sudo`:
```bash
sudo cp guest-agent/kvmmanager-guest-agent.iso /var/lib/libvirt/images/
```

### "ISO not found" in KVM Manager

Verify the ISO is at the exact path:
```bash
ls -lh /var/lib/libvirt/images/kvmmanager-guest-agent.iso
```

The application expects it at this exact location (hardcoded in backend).

### Agent won't connect

1. Check VM has virtio-serial channel:
   ```bash
   virsh dumpxml <vm-name> | grep -A5 "org.kvmmanager.agent"
   ```

2. Check agent is running in guest:
   ```bash
   systemctl status kvmmanager-agent
   ```

3. Check device exists in guest:
   ```bash
   ls -la /dev/vport0p1
   # or
   ls -la /dev/virtio-ports/org.kvmmanager.agent.0
   ```

---

## Documentation

- **DEPLOYMENT_REPORT.md** - Comprehensive deployment analysis
- **DEVOPS_STATUS.md** - DevOps agent status and findings
- **guest-agent/INSTALL.md** - Full installation guide
- **guest-agent/DEPLOYMENT_CHECKLIST.md** - Complete testing checklist

---

## Status

- ✅ Guest agent binary built (1.6 MB)
- ✅ ISO created (1.9 MB)
- ✅ Backend integration complete
- ✅ Frontend UI complete
- ✅ Documentation complete
- ✅ libvirtd running and healthy
- ⏸️ **ISO deployment** - Awaiting this step
- ⏸️ Testing - Ready to begin after deployment

---

## Questions?

See the comprehensive documentation:
- Technical details: `DEPLOYMENT_REPORT.md`
- Installation guide: `guest-agent/INSTALL.md`
- Testing plan: `guest-agent/DEPLOYMENT_CHECKLIST.md`

---

**Ready to deploy?**

```bash
sudo ./DEPLOY_ISO.sh
```
