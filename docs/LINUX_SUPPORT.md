# Linux Support Matrix

## Interactive readiness repair

Failed first-run checks can be selected for a reviewed recovery flow. On verified local-system
connections, missing UEFI firmware or TPM emulation may offer a fixed package repair. The review
shows the detected distribution, connection, effects, and privilege requirement. Authentication
is handled by the desktop privilege agent; KVM Manager never receives an administrator password.

Secure Boot templates, BIOS settings, login renewal, storage choices, best-effort distributions,
and non-local connections remain guided manual or navigation actions. Readiness is refreshed after
every outcome rather than inferred from process completion.

KVM Manager verifies the following host families: Arch/CachyOS, Debian/Ubuntu LTS,
Fedora/RHEL-compatible, and openSUSE. Other Linux distributions receive non-destructive,
best-effort diagnostics only.

| Family | Package manager | Virtualization prerequisites | Service guidance |
|---|---|---|---|
| Arch/CachyOS | `pacman` | `libvirt`, `qemu-full`, `edk2-ovmf`, `swtpm` | Enable `libvirtd.socket` |
| Debian/Ubuntu LTS | `apt` | `libvirt-daemon-system`, `qemu-system-x86`, `ovmf`, `swtpm-tools` | Enable the distribution's libvirt service or socket |
| Fedora/RHEL-compatible | `dnf` | `libvirt-daemon-kvm`, `qemu-kvm`, `edk2-ovmf`, `swtpm` | Enable the distribution's libvirt service or socket |
| openSUSE | `zypper` | `libvirt`, `qemu-kvm`, `qemu-ovmf-x86_64`, `swtpm` | Enable the distribution's libvirt service or socket |

Use the connection-keyed Host Readiness panel on first run, the Dashboard, or Settings as the
source of package, service, firmware, storage, and recovery guidance. A local-system connection
may use the detected desktop distribution profile. Remote, test, and local-session connections
show only libvirt-owned facts and never reuse setup commands from the desktop host.

## First VM workflow

1. Select and connect to the intended libvirt host.
2. Review Host Readiness. Inspection is read-only and never installs packages, enables services,
   creates directories, changes permissions, or starts storage pools.
3. In the VM wizard, select an active storage pool by its connection-owned UUID. KVM Manager does
   not assume a pool named `default`.
4. If there is no eligible pool, cancel or open Storage. Pool creation previews the connection,
   target, autostart behavior, and activation effect before confirmation.
5. For an ISO in Downloads, use **Import into selected storage pool**. The app reads the source,
   streams a new volume through libvirt, preserves the source, and never overwrites implicitly.
6. Review firmware, Secure Boot, TPM, network, and storage preflight before Create is enabled.

## Permissions

The account running KVM Manager needs access to the selected libvirt connection. Group names,
QEMU service accounts, MAC policy, and image locations differ by distribution. Use your
distribution's libvirt documentation and the diagnostic shown by the application; do not run the
desktop application as root. Do not make a private home directory traversable to the QEMU service
account merely to attach a download; import that media into a libvirt-managed pool instead.

## NVIDIA Wayland graphics

On Linux hosts with NVIDIA detected, KVM Manager disables WebKitGTK's DMA-BUF renderer before
creating the webview. This avoids the known Wayland `Gdk-Message: Error 71 (Protocol error)` crash.
Set `WEBKIT_DISABLE_DMABUF_RENDERER` yourself before launch to retain full control of that setting.

## Forwarding and release evidence

Native Arch, Debian, and RPM packages will carry the authorized forwarding component. The AppImage
intentionally reports forwarding as unavailable. Each release must record the runner, artifact,
commit, install result, startup result, and forwarding result for every affected family.
