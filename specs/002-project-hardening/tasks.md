# Tasks: Trustworthy Core Operations

**Input**: Design documents from `/specs/002-project-hardening/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, and
`quickstart.md`

**Tests**: Required by FR-019 and the project constitution. Write each story's tests first and verify
that they fail for the intended reason before implementation.

**Cross-feature boundary**: Feature 001 owns host readiness, firmware discovery, privileged port
forwarding, Linux packaging, and release smoke tests. Rebase this work after overlapping feature 001
command-guard changes; do not duplicate or remove its tasks.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and has no incomplete dependency.
- **[Story]**: Maps the task to its independently testable user story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add the reviewed dependencies, test directories, and explicit static-check entry point.

- [X] T001 Add the selected event-based XML dependency in `src-tauri/Cargo.toml`, remove the application lock-file exclusion from `.gitignore`, and update `src-tauri/Cargo.lock` with a reviewed locked graph.
- [X] T002 [P] Add an explicit frontend lint script and configuration in `package.json` and `eslint.config.js`; do not use an optional or no-op lint command.
- [X] T003 [P] Create sanitized XML, protected-value, and two-connection test fixtures under `src-tauri/tests/fixtures/hardening/`.
- [X] T004 [P] Create shared frontend safe-failure and connection-capability fixtures in `src/test/hardeningFixtures.ts`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish contracts used by every story before migrating resource behavior.

**⚠️ CRITICAL**: User-story implementation begins only after these types and helpers pass their own
unit tests.

- [X] T005 Define `OperationContext`, `TargetIdentity`, `ConnectionCapability`, `MutationOutcome`, `RecoveryAction`, and confirmation models from `data-model.md` in `src-tauri/src/models/operation.rs` and export them from `src-tauri/src/models/mod.rs`.
- [X] T006 [P] Replace arbitrary public display errors with the serializable `SafeFailure` categories from `contracts/ipc.md` in `src-tauri/src/utils/error.rs`, retaining source errors only as backend-internal context.
- [X] T007 [P] Implement bounded field validation, safe XML text/attribute emission, structural root validation, and unknown-event-preserving transforms in `src-tauri/src/utils/xml.rs` with colocated unit tests.
- [X] T008 [P] Implement the allowlisted operation-event API from `contracts/diagnostics.md` in `src-tauri/src/utils/diagnostics.rs` and prohibit arbitrary payload fields at its public boundary.
- [X] T009 Add Rust/TypeScript contract parity tests for safe failures, capabilities, mutation outcomes, and recovery actions in `src-tauri/tests/ipc_contracts.rs` and `src/lib/types.test.ts`.
- [X] T010 Add matching frontend types plus one safe invocation/error-normalization boundary in `src/lib/types.ts` and `src/lib/tauri.ts`; retain temporary legacy-string normalization only for command groups not yet migrated.

**Checkpoint**: Shared safe contracts compile and their unit/contract tests pass without changing
resource behavior.

---

## Phase 3: User Story 1 - Keep Management Data Safe (Priority: P1) 🎯 MVP

**Goal**: User-controlled configuration remains data, protected values never reach diagnostics, and
high-impact mutations require exact-target confirmation.

**Independent Test**: Run the 100-value configuration corpus and protected-sentinel capture suite;
verify valid definitions, zero unintended structure, zero protected output, and no mutation on
rejection.

### Tests for User Story 1

- [X] T011 [P] [US1] Add a deterministic corpus of at least 100 XML-sensitive, Unicode, control, oversized, and malicious field values with no-mutation assertions in `src-tauri/tests/configuration_safety.rs`.
- [X] T012 [P] [US1] Add captured-output success/failure tests for every protected category in `src-tauri/tests/diagnostic_redaction.rs`.
- [X] T013 [P] [US1] Add raw-domain, network-filter, wrong-root, malformed-document, namespace, quote-style, and unknown-element preservation cases in `src-tauri/tests/xml_import_validation.rs`.
- [X] T014 [P] [US1] Add stale-selection, target-change, expiry, and single-use confirmation tests in `src-tauri/tests/destructive_confirmation.rs`.

### Implementation for User Story 1

- [X] T015 [US1] Replace interpolated network-definition generation and substring parsing with safe structural helpers in `src-tauri/src/services/network_service.rs`, excluding forwarding work owned by feature 001.
- [X] T016 [P] [US1] Replace interpolated filter-definition generation and parsing with safe structural helpers in `src-tauri/src/services/nwfilter_service.rs`.
- [X] T017 [P] [US1] Replace storage-pool, volume, and secret definition interpolation with safe structural helpers in `src-tauri/src/services/storage_service.rs`.
- [X] T018 [US1] Replace VM creation, clone metadata, cloud-init attachment, and import definition interpolation with safe structural helpers in `src-tauri/src/services/vm_service.rs` and `src-tauri/src/services/cloud_init_service.rs`.
- [X] T019 [US1] Replace substring-based VM disk, boot, CPU, memory, interface, and device edits with event-based targeted transforms in `src-tauri/src/services/vm_service.rs`, preserving unowned XML and namespaces.
- [X] T020 [P] [US1] Validate resource root, structure, size, and allowed operation before raw VM or filter definition in `src-tauri/src/commands/vm.rs` and `src-tauri/src/commands/nwfilter.rs`.
- [X] T021 [US1] Remove raw XML, cloud-init contents, guest contents, credentials, secret identifiers, and host paths from logging and nested public errors across `src-tauri/src/services/` and `src-tauri/src/commands/`; replace them with the allowlisted diagnostic API.
- [X] T022 [US1] Add confirmation-token issuance and verification for destructive VM, snapshot, network, filter, storage, host-device, and host-wide commands in `src-tauri/src/commands/`.
- [X] T023 [US1] Integrate connection-and-target confirmation previews into destructive controls in `src/pages/VmList.tsx`, `src/pages/NetworkManager.tsx`, `src/pages/StorageManager.tsx`, and the affected `src/components/` dialogs.
- [X] T024 [US1] Run and record the User Story 1 corpus, redaction, raw-import, and confirmation evidence in `specs/002-project-hardening/quickstart.md` without recording protected values or host paths.

**Checkpoint**: P1 is independently deployable; configuration and diagnostic boundaries pass their
attack corpora even if later connection and UI hardening stories remain open.

---

## Phase 4: User Story 2 - Manage the Connection Actually Selected (Priority: P2)

**Goal**: Every enabled action, refresh, status, and console route uses one selected connection, with
unsupported connection-specific features blocked before invocation.

**Independent Test**: Use two isolated fixture connections containing same-named resources and prove
that all enabled operations affect only the connection captured at command entry.

### Tests for User Story 2

- [X] T025 [P] [US2] Add two same-name libvirt test-driver host definitions and lifecycle setup under `src-tauri/tests/fixtures/hardening/connections/`.
- [X] T026 [US2] Add query, mutation, refresh, selection-race, disconnect, duplicate-name, and no-local-fallback cases in `src-tauri/tests/connection_routing.rs`.
- [X] T027 [P] [US2] Add frontend query-key, stale-confirmation, capability-gate, and reconnection regression tests in `src/components/connections/ConnectionRouting.test.tsx`.

### Implementation for User Story 2

- [X] T028 [US2] Make `ConnectionService` resolve an immutable operation context and live handle atomically in `src-tauri/src/services/connection_service.rs` and `src-tauri/src/state/app_state.rs`.
- [X] T029 [US2] Remove the parallel fixed-local connection authority and route VM, snapshot, backup, template, metrics, alert, scheduler, guest-agent, and optimization services through operation context in `src-tauri/src/services/`.
- [X] T030 [P] [US2] Route network, filter, storage, OVA, and migration services through operation context in `src-tauri/src/services/`.
- [X] T031 [P] [US2] Classify PCI, USB, mediated-device, SR-IOV, evdev, and other host-local operations as connection capabilities and enforce them in `src-tauri/src/services/`.
- [X] T032 [US2] Pass the selected connection URI to justified `virsh`, serial-console, and viewer invocations—or mark the feature unavailable—in `src-tauri/src/services/guest_agent_service.rs`, `src-tauri/src/services/serial_console_service.rs`, `src-tauri/src/services/vm_service.rs`, and `src-tauri/src/commands/system.rs`.
- [X] T033 [US2] Include `connectionId` in resource query keys, mutation contexts, invalidation, and refresh wrappers in `src/lib/tauri.ts`, `src/hooks/`, and affected `src/pages/`.
- [X] T034 [US2] Replace hard-coded desktop connection status and inert switching with live saved/active connection state in `src/components/desktop/ConnectionBarWrapper.tsx` and `src/components/desktop/ConnectionBar.tsx`.
- [X] T035 [US2] Render connection-scoped availability reasons and block unsupported console, migration, and host-device actions in `src/components/connections/ConnectionManager.tsx`, `src/components/console/`, `src/components/hardware/`, and `src/components/vm/MigrationDialog.tsx`.
- [ ] T036 [US2] Execute test-driver and isolated two-host console scenarios and record connection IDs, capabilities, and outcomes in `specs/002-project-hardening/quickstart.md` without host paths or credentials.

**Checkpoint**: P1 and P2 both work independently; selecting another host cannot silently operate on
the local system connection.

---

## Phase 5: User Story 3 - Recover from Failures Without Crashing (Priority: P3)

**Goal**: Expected host, parse, synchronization, window, and multi-step failures return classified
safe outcomes while unaffected application areas remain usable.

**Independent Test**: Inject each specified failure at every command group and verify zero panics,
safe error envelopes, an explicit mutation outcome, and usable unaffected views.

### Tests for User Story 3

- [X] T037 [P] [US3] Add unavailable-connection, malformed-response, non-UTF-8, missing-command, poisoned-lock, and failed-window fault cases in `src-tauri/tests/failure_recovery.rs`.
- [X] T038 [P] [US3] Add applied, rejected, rolled-back, partial, and unknown multi-step outcome cases in `src-tauri/tests/mutation_outcomes.rs`.
- [X] T039 [P] [US3] Add production ErrorBoundary disclosure and safe-retry tests in `src/components/ErrorBoundary.test.tsx`.

### Implementation for User Story 3

- [X] T040 [US3] Replace the panicking libvirt accessor with a result-bearing availability contract in `src-tauri/src/services/libvirt.rs` and update every remaining caller under `src-tauri/src/services/`.
- [X] T041 [US3] Migrate VM, snapshot, guest-agent, backup, console, and migration commands from string errors to `SafeFailure` in `src-tauri/src/commands/` and their `src/lib/tauri.ts` wrappers.
- [X] T042 [P] [US3] Migrate network, filter, storage, and OVA commands from string errors to `SafeFailure` in `src-tauri/src/commands/` and their `src/lib/tauri.ts` wrappers.
- [X] T043 [P] [US3] Migrate PCI, USB, mediated-device, SR-IOV, system, and optimization commands from string errors to `SafeFailure` in `src-tauri/src/commands/` and their `src/lib/tauri.ts` wrappers.
- [X] T044 [P] [US3] Migrate scheduler, alert, retention, metrics, and template commands from string errors to `SafeFailure` in `src-tauri/src/commands/` and their `src/lib/tauri.ts` wrappers.
- [X] T045 [US3] Replace synchronization and window/menu `unwrap` calls with classified recovery paths in `src-tauri/src/commands/window.rs`, `src-tauri/src/menu.rs`, and `src-tauri/src/window_state.rs`.
- [X] T046 [US3] Add compensation or explicit residual-state inspection for multi-step clone, import, encrypted-volume, device, and related mutations in `src-tauri/src/services/`.
- [X] T047 [US3] Replace production stack/raw-error display with safe classified recovery actions in `src/components/ErrorBoundary.tsx`, toast helpers, and affected page-level error views under `src/`.
- [X] T048 [US3] Run fault-injection and partial-outcome scenarios and record only safe operation IDs, classifications, and outcomes in `specs/002-project-hardening/quickstart.md`.

**Checkpoint**: All public command groups return safe classified errors for expected faults and no
known command path can panic because a connection, lock, host response, or window is unavailable.

---

## Phase 6: User Story 4 - Trust Product Claims and Quality Evidence (Priority: P4)

**Goal**: Visible state is live, incomplete features cannot masquerade as working, the desktop
boundary is explicit, and clean-checkout quality validation is locked and fail-closed.

**Independent Test**: Run product-claim and security-policy checks plus the entire clean-checkout
workflow; verify no placeholder facts, unrestricted policy, unlocked graph, skipped check, or empty
required suite can pass.

### Tests for User Story 4

- [X] T049 [P] [US4] Add repository tests for placeholder owners/links, fabricated versions/status, incomplete actionable controls, committed locks, and required non-empty suites in `scripts/test-project-truthfulness.mjs`.
- [X] T050 [P] [US4] Add a production content-policy assertion and local VNC/SPICE connectivity regression tests in `src/test/securityPolicy.test.ts` and `src/components/console/ConsoleSecurity.test.tsx`.

### Implementation for User Story 4

- [X] T051 [US4] Define the narrowest tested production content policy for bundled assets, Tauri IPC, and enabled local console traffic in `src-tauri/tauri.conf.json`.
- [X] T052 [P] [US4] Replace placeholder QEMU/version and other fabricated runtime facts with queried state in `src-tauri/src/commands/system.rs` and affected system views; desktop connection state is owned by T034.
- [X] T053 [P] [US4] Disable or accurately label currently actionable placeholder controls in `src/pages/Settings.tsx`, `src/components/vm/AddHardwareDialog.tsx`, and `src/components/vm/devices/InputEditor.tsx`.
- [X] T054 [P] [US4] Replace template author, description, repository, release, issue, and discussion metadata with the canonical `theonlytruebigmac/kvm-manager` project values in `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`, and `README.md`.
- [X] T055 [US4] Update `.github/workflows/quality.yml` to run locked Rust resolution, explicit frontend lint, project-truthfulness checks, and expected-suite-count guards alongside existing strict checks.
- [X] T056 [US4] Add reviewed JavaScript and Rust vulnerability-policy checks with locked tool versions in `.github/workflows/quality.yml` and document local equivalents in `CONTRIBUTING.md`.
- [X] T057 [US4] Run the production policy, console, truthfulness, metadata, and clean-checkout scenarios and record non-sensitive evidence in `specs/002-project-hardening/quickstart.md`.

**Checkpoint**: Product claims reflect live state and the locked, fail-closed quality workflow proves
all four stories' automated boundaries.

---

## Phase 7: Polish and Cross-Cutting Validation

**Purpose**: Close documentation and full-system evidence after all independent stories pass.

- [X] T058 [P] Document safe diagnostics, connection-specific capability limits, destructive confirmation, and recovery behavior in `docs/SECURITY.md` and `docs/CONNECTIONS.md`.
- [X] T059 [P] Update developer workflow and required checks in `CONTRIBUTING.md` and `README.md`, keeping feature 001 distro and packaging guidance authoritative.
- [X] T060 Run every command in `specs/002-project-hardening/quickstart.md`, the feature 001 regression suite, and the production Tauri build; record clean-worktree evidence and resolve every unexplained warning.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts after feature 001 overlapping command-guard work is rebased.
- **Foundational (Phase 2)**: Depends on Phase 1 and blocks every story.
- **US1 (Phase 3)**: Starts after Phase 2 and is the MVP.
- **US2 (Phase 4)**: Starts after Phase 2; final command integration must retain US1 safe errors and diagnostics.
- **US3 (Phase 5)**: Starts after Phase 2; command migrations are easiest after US2 establishes operation context.
- **US4 (Phase 6)**: Tests can start after Phase 2; final quality wiring depends on completed US1–US3 suites.
- **Polish (Phase 7)**: Depends on all selected stories and feature 001 regression compatibility.

### User Story Dependencies

- **US1**: Independent after shared foundations; delivers the primary safety MVP.
- **US2**: Independent routing value after shared foundations, with integration dependency on US1's
  safe failure/diagnostic contracts.
- **US3**: Independently testable fault recovery; command-wide migration follows US2 to avoid
  touching every command twice.
- **US4**: Independently testable product truthfulness; its final workflow aggregates all prior
  boundary suites.

### Within Each User Story

- Add tests and confirm their intended failure before changing runtime behavior.
- Implement backend boundary and contract behavior before frontend integration.
- Refresh and post-mutation inspection must use the same operation context.
- Record isolated evidence only after automated cases pass.

### Parallel Opportunities

- T002–T004 can proceed in parallel after T001's dependency selection is known.
- T006–T008 can proceed in parallel after T005's common names are fixed.
- US1 generator migrations T016–T017 can proceed in parallel; VM migration remains serialized because
  it shares `vm_service.rs`.
- US2 resource groups T029–T031 can proceed in parallel after T028.
- US3 command groups T042–T044 can proceed in parallel after T040 establishes result-bearing access.
- US4 truthfulness, policy, and metadata tasks T051–T054 can proceed in parallel.

## Parallel Examples

### User Story 1

```text
T011 configuration attack corpus
T012 protected diagnostic capture
T013 structural import validation
T014 destructive confirmation races
```

### User Story 2

```text
T029 VM and automation connection routing
T030 network and storage connection routing
T031 host-local capability classification
```

### User Story 3

```text
T042 network/storage safe command errors
T043 host-device/system safe command errors
T044 background-service safe command errors
```

### User Story 4

```text
T051 production content policy
T052 live version and connection state
T053 incomplete-feature truthfulness
T054 canonical project metadata
```

## Implementation Strategy

### MVP First

1. Complete T001–T010.
2. Complete T011–T024 for User Story 1.
3. Stop and run the configuration-safety, redaction, import, and confirmation suites.
4. Merge only if feature 001 regressions remain green.

### Incremental Delivery

1. Ship safe configuration and diagnostics (US1).
2. Route all enabled behavior through the selected connection (US2).
3. Replace remaining panic/string-error paths and reconcile partial changes (US3).
4. Enforce truthful product claims and fail-closed evidence (US4).
5. Finish documentation and the full quickstart without combining unvalidated stories into one
   release-sized rewrite.

## Task Summary

- Setup: 4 tasks
- Foundational: 6 tasks
- User Story 1: 14 tasks
- User Story 2: 12 tasks
- User Story 3: 12 tasks
- User Story 4: 9 tasks
- Polish: 3 tasks
- **Total: 60 tasks**
