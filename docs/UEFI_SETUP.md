# UEFI/Secure Boot Setup Guide

## System Requirements

### Firmware Files
The application automatically detects OVMF firmware in common locations:
- `/usr/share/OVMF/OVMF_CODE_4M.fd` (your system ✓)
- `/usr/share/OVMF/OVMF_CODE_4M.secboot.fd` (your system ✓)
- `/usr/share/edk2/ovmf/` (alternative location)
- `/usr/share/edk2-ovmf/x64/` (alternative location)

### Installation
```bash
# Ubuntu/Debian
sudo apt install ovmf

# Fedora/RHEL
sudo dnf install edk2-ovmf

# Arch Linux
sudo pacman -S edk2-ovmf
```

### NVRAM Directory
The application requires write access to:
```bash
/var/lib/libvirt/qemu/nvram/
```

This directory is now configured with proper permissions (✓).

## Firmware Options

### BIOS (Traditional)
- **Chipset**: i440FX (PC)
- **Use Case**: Legacy systems, older Linux distributions
- **Compatibility**: Maximum
- **Features**: Limited

### UEFI (Modern)
- **Firmware**: `/usr/share/OVMF/OVMF_CODE_4M.fd`
- **NVRAM Template**: `/usr/share/OVMF/OVMF_VARS_4M.fd`
- **Chipset**: Q35 (recommended)
- **Use Case**: Modern Linux, recent Windows versions
- **Features**: GPT partitions, faster boot, modern drivers

### UEFI with Secure Boot
- **Firmware**: `/usr/share/OVMF/OVMF_CODE_4M.secboot.fd`
- **NVRAM Template**: `/usr/share/OVMF/OVMF_VARS_4M.ms.fd`
- **Chipset**: Q35 (required)
- **Use Case**: Windows 11, security-focused deployments
- **Features**: All UEFI features + verified boot chain

## TPM 2.0 Support

### Installation
```bash
# Ubuntu/Debian
sudo apt install swtpm swtpm-tools

# Fedora/RHEL
sudo dnf install swtpm swtpm-tools

# Arch Linux
sudo pacman -S swtpm
```

### Use Cases
- Windows 11 (required)
- BitLocker encryption
- Measured boot
- Security compliance

## Quick Presets

### Windows 11 Compatible
- **Firmware**: UEFI + Secure Boot
- **TPM**: Enabled
- **Chipset**: Q35
- **Memory**: 4096 MB minimum
- **Disk**: 64 GB minimum

### Modern Linux
- **Firmware**: UEFI
- **TPM**: Optional
- **Chipset**: Q35
- **Memory**: 2048 MB minimum
- **Disk**: 20 GB minimum

### Legacy BIOS
- **Firmware**: BIOS
- **TPM**: Not available
- **Chipset**: PC (i440FX)
- **Memory**: 512 MB minimum
- **Disk**: 8 GB minimum

### Maximum Security
- **Firmware**: UEFI + Secure Boot
- **TPM**: Enabled
- **Chipset**: Q35
- **Memory**: 4096 MB minimum
- **Disk**: 32 GB minimum

## Testing UEFI/Secure Boot

### 1. Create Test VM
```
Name: uefi-test
Firmware: UEFI with Secure Boot
TPM: Enabled
Chipset: Q35
Memory: 4096 MB
Disk: 64 GB
```

### 2. Expected Behavior
- VM starts without errors
- UEFI boot menu appears
- Secure Boot shows as "Enabled" in guest OS
- TPM device visible in guest OS (if OS supports it)

### 3. Verification (Linux Guest)
```bash
# Check Secure Boot status
mokutil --sb-state

# Check TPM
ls /dev/tpm*

# Check firmware type
[ -d /sys/firmware/efi ] && echo "UEFI" || echo "BIOS"
```

### 4. Verification (Windows Guest)
```powershell
# Check Secure Boot
Confirm-SecureBootUEFI

# Check TPM
Get-Tpm
```

## Troubleshooting

### Error: "UEFI firmware not found"
**Solution**: Install OVMF package
```bash
sudo apt install ovmf
```

### Error: "Permission denied" on NVRAM directory
**Solution**: Fix permissions
```bash
sudo mkdir -p /var/lib/libvirt/qemu/nvram
sudo chown -R libvirt-qemu:kvm /var/lib/libvirt/qemu/nvram
sudo chmod 755 /var/lib/libvirt/qemu/nvram
```

### VM fails to start with UEFI
**Check**:
1. OVMF firmware files exist
2. NVRAM directory permissions
3. Libvirt logs: `sudo journalctl -u libvirtd -f`

### Secure Boot not working in guest
**Possible causes**:
1. Guest OS doesn't support Secure Boot
2. Unsigned bootloader
3. Wrong OVMF firmware path

### TPM not visible in guest
**Check**:
1. swtpm package installed
2. TPM enabled in VM config
3. Guest OS has TPM drivers

## Platform-Specific Notes

### Ubuntu/Debian
- OVMF packages: `ovmf`
- Default paths work out of the box
- SELinux not an issue

### Fedora/RHEL
- OVMF packages: `edk2-ovmf`
- Paths: `/usr/share/edk2/ovmf/`
- Check SELinux contexts

### Arch Linux
- OVMF packages: `edk2-ovmf`
- Paths: `/usr/share/edk2-ovmf/x64/`
- May need manual symlinks

## References

- [OVMF Documentation](https://github.com/tianocore/tianocore.github.io/wiki/OVMF)
- [Libvirt UEFI Setup](https://wiki.archlinux.org/title/Libvirt#UEFI_support)
- [TPM Emulation](https://www.qemu.org/docs/master/specs/tpm.html)
- [Windows 11 Requirements](https://docs.microsoft.com/en-us/windows/whats-new/windows-11-requirements)
