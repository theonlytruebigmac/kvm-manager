# Implementation Plan: Trustworthy Core Operations

**Branch**: `002-project-hardening` | **Date**: 2026-09-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-project-hardening/spec.md`

## Summary

Harden KVM Manager's privileged boundaries by introducing safe structured configuration handling,
redacted diagnostic contracts, immutable per-operation connection context, typed recoverable errors,
truthful capability presentation, and enforceable quality checks. Work proceeds boundary-first: add
shared contracts and characterization tests, migrate one independently testable user-story slice at
a time, then remove unsafe compatibility paths only after their callers have moved.

## Technical Context

**Language/Version**: Rust 2021 on stable (validated with 1.94.0); TypeScript 5.8; React 19  
**Primary Dependencies**: Tauri 2, `virt` 0.3/libvirt, Tokio, Serde, tracing, React Query, Zustand;
add one event-based XML reader/writer and focused test support after dependency review  
**Storage**: Existing SQLite metrics database and in-memory/saved connection configuration; no new
persistent domain store  
**Testing**: `cargo test`, Rust unit/integration tests, libvirt test-driver fixtures, Vitest, React
Testing Library, TypeScript checking, production Vite build, strict Clippy  
**Target Platform**: Linux desktop; Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible, and
openSUSE support matrix  
**Project Type**: Tauri desktop application with React frontend and Rust host-integration backend  
**Performance Goals**: No user-visible regression in ordinary list/refresh interactions; validation
and redaction complete before each privileged operation reaches libvirt or another host boundary  
**Constraints**: Never log protected values; no expected failure may panic; no implicit local
fallback; preserve libvirt XML not owned by the edited feature; keep feature 001 ownership intact  
**Scale/Scope**: Approximately 218 Tauri commands, 25 Rust services, 7,000-line VM service, four user
stories, and local plus saved remote libvirt connections

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Gate | Design response |
|---|---|---|
| I. Safe Host Control | PASS | Structured value validation, safe configuration writing, exact-target confirmation, and explicit partial outcomes are first-class contracts. |
| II. Libvirt Is the Source of Truth | PASS | Resource operations route through the selected libvirt connection; direct parsing or commands require a documented narrow adapter and tests. |
| III. Modern Linux Compatibility | PASS | No support-family behavior is changed here; validation continues on the four-family matrix owned by feature 001. |
| IV. Test the Risk | PASS | Boundary characterization, malformed-input corpora, fault injection, and isolated two-connection validation are required before migration. |
| V. Clear Boundaries and Observable Failures | PASS | IPC uses typed safe failures and operation context; diagnostics retain useful identifiers without protected values. |
| Platform and Security Constraints | PASS | Remote features are advertised only when routed end to end, and the desktop content boundary is restricted to justified sources. |
| Development Workflow and Quality Gates | PASS | This spec, plan, research, models, contracts, quickstart, and dependency-ordered task list precede implementation. |

**Post-design re-check**: PASS. The data model makes connection identity immutable for an operation,
the contracts prohibit raw sensitive payloads, and the quickstart includes isolated host validation.
No constitutional exception is requested.

## Phase 0: Research Outcomes

Detailed decisions and rejected alternatives are in [research.md](research.md).

- Use one active-connection authority and capture an immutable operation context at command entry.
- Use structural/event-based XML handling for all generated or modified definitions; do not attempt a
  single all-domain schema rewrite in one change.
- Serialize stable, safe IPC error envelopes rather than returning arbitrary display strings.
- Define allowlisted diagnostic fields and prove redaction with captured-output tests.
- Introduce an explicit production content policy derived from actual local console traffic.
- Commit both dependency lock files and make every configured quality check fail closed.

## Phase 1: Design

### Backend boundaries

1. Add shared safe operation, capability, failure, and mutation-outcome models.
2. Make the connection registry the only source of an operation's `Connect` handle and immutable
   connection identity; transition the fixed local service behind that authority.
3. Add configuration helpers for value validation, safe XML emission, structural root/type checks,
   and event-based targeted transforms.
4. Add a diagnostic facade whose accepted fields cannot contain protected payloads; remove raw XML,
   cloud-init content, host paths, and credentials from logging sites.
5. Return typed failures through every Tauri command group and map them to safe frontend messages.

### Frontend boundaries

1. Carry connection capabilities and operation context through query keys, mutations, confirmations,
   refreshes, and console entry points.
2. Disable unsupported and incomplete operations with an explicit reason.
3. Render safe classified failures and production recovery UI without stack traces or raw host data.
4. Replace hard-coded connection/version/product claims with queried state.

### Quality boundaries

1. Preserve current format, strict static analysis, test, and production-build gates.
2. Add frontend linting as an explicit script rather than an optional command.
3. Commit the Rust application lock file and validate locked clean-checkout builds.
4. Add focused boundary suites before broad implementation migration; use isolated libvirt test
   fixtures for connection-routing verification.

## Project Structure

### Documentation (this feature)

```text
specs/002-project-hardening/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── diagnostics.md
│   └── ipc.md
└── tasks.md
```

### Source Code (repository root)

```text
src-tauri/src/
├── commands/                 # command-entry context and safe IPC errors
├── models/                   # operation/capability/failure contracts
├── services/                 # selected-connection routing and resource operations
├── utils/                    # safe XML, validation, and diagnostics helpers
└── state/                    # connection authority and operation snapshots

src-tauri/tests/
├── fixtures/                 # XML, connection, and protected-value fixtures
├── configuration_safety.rs
├── connection_routing.rs
├── diagnostic_redaction.rs
└── failure_recovery.rs

src/
├── components/               # confirmations, capabilities, and recovery surfaces
├── lib/                      # typed IPC models and query helpers
├── pages/                    # connection-aware page integration
└── test/                     # shared frontend test setup and fixtures

.github/workflows/            # fail-closed quality validation
```

**Structure Decision**: Retain the existing Tauri/React layout. Shared safety primitives live at
the Rust command/service boundary and in typed frontend IPC helpers; story-specific UI remains with
the current resource components. The oversized VM service is migrated through focused adapters,
not rewritten wholesale.

## Complexity Tracking

No constitution violations require justification. The incremental compatibility layer around the
existing local libvirt service is temporary and must be removed by the connection-routing story.
