# Libvirt Permissions Troubleshooting

## Common Permission Issues

### Problem: "Permission denied" when starting VM with ISO/disk attached

**Error message:**
```
Could not open '/path/to/file.iso': Permission denied
```

### Solutions

#### Option 1: Copy files to libvirt's image directory (Recommended)
```bash
# Copy ISO to libvirt directory
sudo cp /home/user/Downloads/alpine.iso /var/lib/libvirt/images/

# Set proper ownership
sudo chown libvirt-qemu:kvm /var/lib/libvirt/images/alpine.iso

# Set proper permissions
sudo chmod 644 /var/lib/libvirt/images/alpine.iso
```

#### Option 2: Fix permissions on existing file location
```bash
# Set read permissions for all
sudo chmod 644 /path/to/file.iso

# Change ownership to libvirt user
sudo chown libvirt-qemu:kvm /path/to/file.iso

# Also ensure parent directories are accessible
sudo chmod +x /home/user
sudo chmod +x /home/user/Downloads
```

#### Option 3: Add libvirt-qemu to your user group (Less secure)
```bash
# Add libvirt-qemu user to your group
sudo usermod -a -G $(id -gn) libvirt-qemu

# Restart libvirt
sudo systemctl restart libvirtd

# Make sure your home directory allows group access
chmod g+x ~
```

### Why This Happens

Libvirt runs QEMU/KVM VMs as the `libvirt-qemu` user (or `qemu` on some systems) for security reasons. This user needs read access to:
- Disk images (.qcow2, .raw, etc.)
- ISO files for installation
- Any other files the VM needs to access

Files in user home directories (like `/home/user/Downloads/`) are typically not readable by other users, causing permission errors.

### Best Practices

1. **Store VM-related files in `/var/lib/libvirt/images/`** - This is the standard location with correct permissions
2. **Never run libvirt as root** - Keep the security isolation
3. **Use proper permissions** - 644 for files, 755 for directories
4. **Check ownership** - Files should be owned by `libvirt-qemu:kvm`

### Quick Permission Check

```bash
# Check file permissions
ls -la /path/to/file.iso

# Check if libvirt-qemu can read it
sudo -u libvirt-qemu cat /path/to/file.iso > /dev/null
# If this fails with "Permission denied", libvirt can't access it
```

### Debugging

```bash
# Check libvirt logs
sudo journalctl -u libvirtd -f

# Check VM-specific logs
sudo tail -f /var/log/libvirt/qemu/<vm-name>.log

# Check SELinux (if enabled)
sudo ausearch -m avc -ts recent
```
