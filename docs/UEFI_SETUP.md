# UEFI, Secure Boot, TPM, and Windows 11

KVM Manager discovers guest capabilities from the selected libvirt connection. It does not assume
an OVMF path, NVRAM directory, QEMU binary, storage directory, or package layout. During creation,
libvirt selects firmware from the descriptors advertised by that connection.

## Windows 11 workflow

1. Select the intended connection and review Host Readiness.
2. Create or select an active storage pool with sufficient free space.
3. Select the Windows ISO. If it is in Downloads or another private location, confirm **Import
   into selected storage pool**; the source is preserved.
4. Choose the Windows 11 preset: Q35, UEFI with Secure Boot, and TPM 2.0.
5. On Review, confirm storage, firmware, Secure Boot, TPM, and network are available. Create remains
   disabled while a required capability is missing or while results belong to an old connection.

## Distribution packages

The Host Readiness panel shows guidance only for the detected local-system family. The verified
profiles identify `edk2-ovmf` and `swtpm` on Arch/CachyOS and Fedora/RHEL-compatible hosts, `ovmf`
and `swtpm-tools` on Debian/Ubuntu LTS, and `qemu-ovmf-x86_64` and `swtpm` on openSUSE. Consult the
host administrator for remote/session/test connections and best-effort distributions.

### Arch and CachyOS enrolled keys

On Arch-family hosts, `edk2-ovmf` provides Secure Boot-capable firmware but does not provide a
Microsoft-key-enrolled variable-store template. Reinstalling `edk2-ovmf` does not change that. The
Host Readiness dialog therefore offers two explicit paths:

1. Select plain **UEFI** in the wizard when Secure Boot enforcement is not required. This does not
   satisfy the Windows 11 Secure Boot requirement.
2. For enforced Secure Boot, install the VM firmware utility with
   `sudo pacman -S --needed virt-firmware`. A host administrator must then install an enrolled-key
   firmware template and descriptor that libvirt advertises, or create the VM externally with its
   own enrolled NVRAM and import it. Never modify the shared system `OVMF_VARS` template.

The second path is deliberately not automated during first-run checks: overwriting a shared
variable-store template can affect future VMs, while a safe per-VM NVRAM target does not exist until
a particular VM has been defined. Recheck readiness only after libvirt advertises an enrolled-key
template; changing one existing VM's NVRAM does not change the host-wide capability result.

## Troubleshooting

- **UEFI unavailable:** refresh readiness after installing/configuring firmware on the selected
  guest host. Do not copy a firmware path from another distribution.
- **Secure Boot unavailable:** UEFI alone is insufficient; the connection must advertise a secure
  firmware option and enrolled-key support appropriate to the guest.
- **TPM unavailable:** install/configure a libvirt-supported TPM emulator on the selected guest
  host, then reconnect and refresh.
- **Network unavailable:** select or activate a libvirt network on the same connection.
- **Storage unavailable:** select, activate, or explicitly create a pool. A pool named `default`
  has no special meaning.
- **ISO permission denied:** import it into the selected pool; do not weaken home-directory access.

Readiness and preflight checks are non-mutating. Storage creation/activation and ISO import are
separate reviewed actions with explicit confirmation and observable outcomes.
