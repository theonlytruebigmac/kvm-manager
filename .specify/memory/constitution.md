<!--
Sync Impact Report
- Version change: 1.0.0 → 1.1.0
- Modified principles: III. CachyOS and Arch Compatibility Is a First-Class Contract →
  III. Modern Linux Compatibility Is a First-Class Contract
- Added sections: Platform and Security Constraints; Development Workflow and Quality Gates
- Removed sections: none
- Follow-up TODOs: none
-->

# KVM Manager Constitution

## Core Principles

### I. Safe Host Control Is Non-Negotiable

Every action that changes a VM, storage pool, network, host device, firewall, or guest MUST use
validated structured inputs and the least privilege required. The application MUST NOT build shell
commands from untrusted strings. Destructive operations MUST clearly identify their target and
provide an intentional confirmation path. This protects hosts and guests managed by the app.

### II. Libvirt Is the Source of Truth

VM, network, storage, and device state MUST be read from and changed through libvirt whenever
libvirt supports the operation. Direct filesystem access, XML manipulation, and external commands
MUST be narrowly scoped, validated, and justified when no safe libvirt API exists. UI state MUST
refresh from the active connection after mutations.

### III. Modern Linux Compatibility Is a First-Class Contract

Supported Linux behavior MUST be verified across the supported modern-Linux matrix: Arch/CachyOS,
Debian/Ubuntu LTS, Fedora/RHEL-compatible, and openSUSE. Runtime detection MUST use current
distribution package layouts and service conventions; installation, firmware, permissions, and
networking guidance MUST be distribution-accurate. Unsupported distributions or host capabilities
MUST produce actionable, non-destructive diagnostics.

### IV. Test the Risk, Not Just the Build

Every behavior change MUST include automated coverage at the narrowest practical layer. Changes to
input validation, XML generation, command construction, or state transitions MUST have unit tests.
Changes that exercise libvirt, QEMU, graphics consoles, storage, networking, or permissions MUST
also have documented integration validation on an isolated host or disposable VM. A passing build
is not evidence that host-management behavior is safe.

### V. Clear Boundaries and Observable Failures

The React UI, Tauri command layer, Rust services, and host integrations MUST retain explicit,
typed contracts. Failures MUST preserve useful context without exposing secrets and MUST be logged
through the project logging path. Feature documentation MUST describe prerequisites, supported
platforms, capability limits, and recovery actions.

## Platform and Security Constraints

The application is a Linux desktop application built with Tauri, React, TypeScript, Rust, libvirt,
QEMU/KVM, and system WebKit. The support matrix is Arch/CachyOS, Debian/Ubuntu LTS,
Fedora/RHEL-compatible, and openSUSE; other distributions receive best-effort diagnostics unless
explicitly added. Local system and user-session connections are distinct security and capability
contexts; features MUST state which they require. Remote connections MUST not claim support unless
command routing, credential handling, and console connectivity work through that connection.
Secrets, guest contents, and host paths MUST not be logged or committed.

## Development Workflow and Quality Gates

Each non-trivial change MUST begin with a feature specification, implementation plan, and
dependency-ordered task list under `specs/`. Before merge, maintainers MUST review constitution
compliance, run formatting, static analysis, relevant automated tests, and the feature quickstart.
Release candidates MUST be built from a clean worktree and smoke-tested on every support-matrix
family affected by Linux virtualization behavior or packaging changes. Exceptions require a written
rationale in the relevant plan.

## Governance

This constitution supersedes conflicting implementation conventions and planning assumptions.
Amendments require an explicit rationale, a semantic-version update, and a Sync Impact Report at
the top of this document. MAJOR versions redefine or remove principles, MINOR versions add or
materially expand governance, and PATCH versions clarify existing rules. Specifications, plans,
tasks, reviews, and release checklists MUST verify compliance; unresolved violations block
implementation or release until an approved exception is documented.

**Version**: 1.1.0 | **Ratified**: 2026-09-06 | **Last Amended**: 2026-09-06
