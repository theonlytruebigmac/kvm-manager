# Implementation Plan: Modern Linux Readiness

**Branch**: `001-cachyos-readiness` | **Date**: 2026-09-06 | **Spec**:
[spec.md](spec.md)

## Summary

Make the application safe and usable across a defined modern-Linux matrix by decoupling startup
from a local libvirt connection, reporting verified host capabilities, delegating UEFI selection to
libvirt, removing shell-based port forwarding, and making per-family validation a release gate.

## Technical Context

**Language/Version**: Rust 2021 backend; TypeScript 5.8 and React 19 frontend

**Primary Dependencies**: Tauri 2, rust-libvirt, Tokio, Serde, Vite, React Query, operating-system
policy authorization, and nftables-compatible host networking for the native forwarding component

**Storage**: Existing SQLite application data; no new persistent data is needed for the readiness
report. Port-forward ownership is derived from firewall rules during this feature.

**Testing**: Rust unit tests for pure capability, firmware, and validation logic; frontend component
tests for readiness presentation; isolated support-matrix/libvirt integration validation; `cargo fmt`,
`cargo clippy -- -D warnings`, `cargo test`, and `npm run build`

**Target Platform**: Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible, and openSUSE for
functional validation; Linux desktop for best-effort unsupported-distribution diagnostics

**Project Type**: Tauri desktop application with React frontend and Rust backend

**Performance Goals**: Host readiness completes within 2 seconds on a local ready host; individual
capability failures return a result without delaying the UI by more than 5 seconds.

**Constraints**: No startup exit when libvirt or QEMU is unavailable; no user-controlled shell
interpolation; the desktop app MUST NOT run with broad administrator privileges; forwarding is
available only through a network-scoped, policy-authorized native helper; no test may change
non-disposable VM, storage, physical network, or host firewall; all firmware paths must be verified
before use; guidance is selected from OS metadata, not guesses.

**Scale/Scope**: One local host readiness workflow, a four-family distribution profile matrix, UEFI
and Secure Boot VM creation, authorized local-host forwarding, per-family documentation, Arch
(`.pkg.tar.zst`), Debian (`.deb`), and RPM (`.rpm`) native-package plus AppImage validation, and
CI/release gates. Remote command routing is explicitly out of scope; the forwarding authorization
path is intentionally limited to one local network operation.

## Constitution Check

| Principle | Plan response | Status |
|-----------|---------------|--------|
| Safe Host Control | Typed validation, network ownership checks, and a scoped policy-authorized helper replace shell construction and broad elevation. | PASS |
| Libvirt Is the Source of Truth | Readiness and firmware discovery query libvirt capabilities; direct calls are limited to validated host checks/firewall execution. | PASS |
| Modern Linux Compatibility | Current package/service guidance and firmware behavior are validated across every support-matrix family. | PASS |
| Test the Risk | Unit, UI, integration, and release-gate work are planned before feature tasks. | PASS |
| Clear Boundaries | New service models and Tauri contracts isolate host probing from UI rendering. | PASS |

## Project Structure

### Documentation (this feature)

```text
specs/001-cachyos-readiness/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── readiness-and-forwarding.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── components/
│   └── system/HostReadinessPanel.tsx
├── lib/
│   ├── tauri.ts
│   └── types.ts
└── pages/
    └── Settings.tsx

src-tauri/src/
├── commands/
│   ├── system.rs
│   └── network.rs
├── models/
│   └── host.rs
├── services/
│   ├── host_readiness_service.rs
│   ├── distribution_profile_service.rs
│   ├── firmware_service.rs
│   ├── network_service.rs
│   └── libvirt.rs
└── state/
    └── app_state.rs

src-tauri/tests/
├── distribution_profiles.rs
├── host_readiness.rs
├── firmware_selection.rs
└── port_forwarding.rs

src-tauri/
├── binaries/
│   └── kvm-manager-network-helper.rs
├── packaging/
│   ├── arch/PKGBUILD
│   └── polkit/com.fraziersystems.kvm-manager.network.policy
└── tauri.conf.json

docs/
├── LINUX_SUPPORT.md
├── CACHYOS_SETUP.md
├── UEFI_SETUP.md
└── LIBVIRT_PERMISSIONS.md

.github/workflows/
├── ci.yml
├── quality.yml
└── release.yml
```

**Structure Decision**: Retain the existing Tauri architecture. Add small domain services and
models rather than embedding host probing in commands or React components. Centralize verified
distribution profiles in the backend; the UI renders its returned guidance. Put host-specific tests
beside the Rust crate and retain a documented manual integration flow for real libvirt/QEMU. Package
a root-owned, single-purpose forwarding helper and its Polkit action only in native distribution
packages through explicit Arch, Debian, and RPM package-install steps; it validates an operation
again before altering its dedicated firewall rules. Do not add a long-running privileged service.
Use a reusable quality workflow from both CI and release workflows,
then make publication depend on artifact-specific smoke-test jobs.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Host-readiness service | Startup must report partial host failures without crashing unrelated UI. | A startup-only connection attempt cannot distinguish capabilities or keep the UI available. |
| Port-forward executor boundary | Firewall mutations require validation, exact state inspection, and privilege-aware errors. | Calling a shell string directly is unsafe and cannot be unit-tested in isolation. |
| Native forwarding helper | The desktop app needs a least-privilege authorization boundary for firewall changes. | Running the whole desktop app as root would expose every Tauri command to broad elevation. |
| Artifact smoke-test matrix | Native packages and AppImages have different install and privilege behavior that a build alone cannot prove. | Publishing after a successful bundle can release an artifact that cannot install or start. |

## Post-Design Constitution Check

All feature design artifacts preserve the five principles. The forwarding path is a narrowly scoped
native helper authorized by the operating system. Portable artifacts explicitly expose it as
unavailable rather than elevating the desktop app. Release publication is gated by the quality and
smoke-test evidence for every artifact.
