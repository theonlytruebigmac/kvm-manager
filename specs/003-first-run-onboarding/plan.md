# Implementation Plan: First-Run Virtualization Onboarding

**Branch**: `003-first-run-onboarding` | **Date**: 2026-09-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-first-run-onboarding/spec.md`

## Summary

Make readiness, storage selection, and guest capability checks connection-scoped prerequisites for
VM creation. The React application will show a non-destructive onboarding assessment before the
first creation attempt and add a storage-pool choice plus review to the VM wizard. Rust commands
will inspect the connection through libvirt, validate a selected pool and guest requirements, and
repeat that validation in `create_vm` before any volume or domain mutation. Existing storage-pool
creation remains an explicit reviewed action; it is never inferred from a distribution or a pool
called `default`. Failed checks also expose connection-bound guided actions: verified local package
and service repairs use a fixed backend allowlist and desktop privilege authorization, while all
other cases remain manual or navigation guidance.

## Technical Context

**Language/Version**: Rust 2021; TypeScript 5.8; React 19

**Primary Dependencies**: Tauri 2, `virt`/libvirt 0.3, React Query 5, Radix UI, Vite 7

**Storage**: libvirt connections, storage pools, volumes, networks, and domains; existing SQLite
application state only (no feature schema change)

**Testing**: Rust integration/unit tests with libvirt-independent fixtures; Vitest + Testing
Library component tests; documented isolated-host libvirt smoke validation

**Target Platform**: Linux desktop on Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible,
and openSUSE; other distributions get safe best-effort diagnostics

**Project Type**: Tauri desktop application (React frontend + Rust host integration)

**Performance Goals**: Non-destructive assessment completes without blocking UI interaction; cached
results are partitioned by selected connection and refreshed before accepting a mutation

**Constraints**: Libvirt remains authoritative; no process input from the UI; no mutation during
inspection; repair is allowlisted, connection-bound, previewed, confirmed, and rechecked; no
password handling; no fixed host assumption; remote/session scopes remain truthful

**Scale/Scope**: One connection-scoped onboarding entry point, interactive readiness repair, VM wizard storage/review additions,
an explicit local-ISO import-copy path, typed Tauri contracts, libvirt preflight enforcement,
support documentation, and focused tests

## Constitution Check

| Principle | Plan response | Gate |
|---|---|---|
| Safe Host Control | Inspection is read-only. Host repair uses named, reviewed, confirmed actions on the selected connection with backend-owned arguments. | Pass |
| Libvirt Is the Source of Truth | Pool state, capacity, activation, connection availability, and guest-compatible firmware are queried through the captured libvirt connection. | Pass |
| Modern Linux Compatibility | Guidance uses the Feature 001 distribution profile matrix. No package, service, account, path, or `default` pool is generalized across distributions. | Pass |
| Test the Risk | Fixture coverage includes unavailable/inactive/multiple/undersized pools, capability failures, cancelled setup, connection changes, and no-mutation command rejections; quickstart requires isolated-host checks. | Pass |
| Clear Boundaries | Rust models/services/Tauri commands expose typed assessment/preflight results; React Query keys retain connection ownership; safe failures redact paths and credentials. | Pass |

No constitution exceptions or new projects are required.

## Project Structure

### Documentation (this feature)

```text
specs/003-first-run-onboarding/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── onboarding-and-vm-preflight.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── components/system/       # first-run/readiness presentation and component tests
├── components/storage/      # reviewed storage setup entry point
├── components/vm/           # create wizard and preflight review
├── hooks/                   # active-connection-owned React Query state
└── lib/                     # typed IPC wrapper and frontend contracts

src-tauri/src/
├── commands/                # typed Tauri assessment, preflight, storage, and VM commands
├── models/                  # serializable readiness and VM request/result models
├── services/                # connection-scoped libvirt inspection and mutation enforcement
└── state/                   # captured operation connection and capability context

src-tauri/tests/             # Rust fixture, IPC, and safety tests
docs/                        # support matrix, permission, first-run, and validation guidance
```

**Structure Decision**: Preserve the existing Tauri React/Rust split. Reuse the active connection
hook, storage service, and distribution-profile service rather than creating a parallel setup
subsystem. Add only narrowly scoped assessment/preflight models and commands at the existing IPC
boundary.

## Complexity Tracking

No constitution violations require justification.
