# Modern Linux Readiness Validation Guide

## Safety

Use a disposable host or nested VM from the support matrix. Do not run forwarding, storage, or
guest-creation validation against a production network or VM inventory.

## Prerequisites

1. Select a support-matrix host and use its verified profile. The initial matrix is Arch/CachyOS,
   Debian/Ubuntu LTS, Fedora/RHEL-compatible, and openSUSE. Install libvirt, a QEMU/KVM emulator,
   UEFI firmware, TPM emulation when required, and the Tauri runtime dependencies using that
   profile's package names.

2. Arch/CachyOS reference setup:

   ```bash
   sudo pacman -Syu --needed libvirt qemu-full edk2-ovmf swtpm
   sudo systemctl enable --now libvirtd.socket
   sudo usermod -aG libvirt "$USER"
   ```

3. Start a new login session and verify the host can serve QEMU capabilities:

   ```bash
   virsh -c qemu:///system domcapabilities
   ```

4. Build the application:

   ```bash
   npm ci
   npm run build
   cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test
   ```

## Readiness Scenarios

1. For each support-matrix host, open the application and verify the detected distribution profile,
   package guidance, service guidance, and firmware guidance match that family.
2. On one isolated host per family, temporarily make each of QEMU capability, UEFI firmware, and
   libvirt access unavailable one at a time. Verify the application shows the specific recovery
   action and stays open.
3. Verify an unsupported-distribution fixture reports best-effort support without presenting
   commands intended for a different package manager.
4. Restore each prerequisite before proceeding to the next case.

## Firmware Scenarios

1. On each affected support-matrix family, create a regular UEFI VM and confirm its installer
   reaches a boot menu.
2. Create a Secure Boot VM and confirm its installer reaches a boot menu.
3. Attempt each creation after hiding compatible firmware on an isolated host; verify that creation
   is blocked with the documented recovery guidance.

## Forwarding Scenarios

1. Install the native distribution package that contains the forwarding helper (`.pkg.tar.zst` on
   Arch/CachyOS, `.deb` on Debian/Ubuntu, or `.rpm` on Fedora/RHEL-compatible and openSUSE); do not
   validate this feature from a development build or portable AppImage. Confirm the desktop
   application still runs as the ordinary desktop user.
2. On an isolated default virtual network, select that network and create one TCP and one UDP
   forwarding rule to a test guest. Confirm the operating system asks only to authorize the scoped
   forwarding operation.
3. Verify traffic reaches the intended guest service and inspect only the application-owned
   firewall scope to confirm both rules are present.
4. Submit malformed address, port, protocol, and an address outside the selected network; verify
   no authorization request or firewall state change occurs.
5. Remove one rule and verify the unrelated rule still works and no non-application rule changes.
6. Start the AppImage on an isolated host; confirm it starts but reports forwarding as unavailable
   with the native-package requirement, without requesting broad administrator execution.

## Artifact Release Scenarios

1. Run formatting, static analysis, Rust tests, frontend tests, and production build through the
   reusable quality workflow.
2. On each designated support-matrix runner, install the artifact intended for that runner and
   verify the application starts: `.pkg.tar.zst` on Arch/CachyOS, `.deb` on Debian/Ubuntu, and
   `.rpm` on Fedora/RHEL-compatible and openSUSE. Test native forwarding only with the native
   package; test the documented unavailable result for portable artifacts.
3. Confirm the release publication job remains blocked until every required quality and
   artifact-smoke job has passed, and retain the runner, artifact, commit, and result in the
   support-matrix evidence record.

## Expected Artifacts

Refer to [the Tauri contract](contracts/readiness-and-forwarding.md) and
[the data model](data-model.md) for response and state expectations.
