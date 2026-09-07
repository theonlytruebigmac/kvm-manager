# Tasks: First-Run Virtualization Onboarding

**Input**: Design documents from `specs/003-first-run-onboarding/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), and
[onboarding-and-vm-preflight.md](contracts/onboarding-and-vm-preflight.md)

**Tests**: Required by FR-013 and the constitution. Write focused tests before the corresponding
implementation and use libvirt-independent fixtures for automated coverage.

**Organization**: Tasks are grouped by user story so each increment has a demonstrable outcome.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish feature fixtures and test boundaries before changing host behavior.

- [X] T001 [P] Add zero, inactive, multiple, and insufficient-capacity pool fixtures in `src-tauri/tests/fixtures/first-run-onboarding/`.
- [X] T002 [P] Add firmware, TPM, network, connection-scope, and supported-distribution fixture inputs in `src-tauri/tests/fixtures/first-run-onboarding/`.
- [X] T003 Record the contract and isolated-host scenarios in `specs/003-first-run-onboarding/quickstart.md` before implementation.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the typed, connection-scoped primitives required by every story.

**⚠️ CRITICAL**: No wizard or host mutation change begins until these tasks are complete.

- [X] T004 Define serializable onboarding assessment, storage readiness/choice, and guest review models in `src-tauri/src/models/host.rs`.
- [X] T005 Extend the shared TypeScript contracts and `VmConfig.storagePoolId` in `src/lib/types.ts`.
- [X] T006 Create pure pool-eligibility and disk-byte validation helpers in `src-tauri/src/services/storage_service.rs`.
- [X] T007 Extract connection-compatible firmware/TPM discovery from fixed VM-creation paths into `src-tauri/src/services/host_readiness_service.rs` and `src-tauri/src/services/vm_service.rs`.
- [X] T008 Add connection-scoped readiness and VM-preflight Tauri commands in `src-tauri/src/commands/system.rs` and register them in `src-tauri/src/lib.rs`.
- [X] T009 Add typed frontend IPC wrappers for readiness and preflight in `src/lib/tauri.ts`.
- [X] T010 Add Rust model, eligibility, and safe-failure contract coverage in `src-tauri/tests/host_readiness.rs` and `src-tauri/tests/ipc_contracts.rs`.

**Checkpoint**: Assessment and preflight contracts compile, use the captured connection, and expose no
raw path, credential, or libvirt error data.

---

## Phase 3: User Story 1 - Understand Host Readiness Before Creating a VM (Priority: P1) 🎯 MVP

**Goal**: First-time users see connection-owned readiness and safe, distribution-appropriate
recovery guidance before entering a failing VM creation flow.

**Independent Test**: On supported-profile fixtures and local/remote/session/test connection
fixtures, the assessment displays the selected connection, marks the exact missing capability, and
does not show local setup commands for a remote or session connection.

### Tests for User Story 1

- [X] T011 [P] [US1] Add distribution and connection-scope assessment tests in `src-tauri/tests/host_readiness.rs`.
- [X] T012 [P] [US1] Add onboarding ready/degraded/best-effort rendering tests in `src/components/system/HostReadinessPanel.test.tsx`.
- [X] T013 [P] [US1] Add connection-change query-ownership tests in `src/components/connections/ConnectionRouting.test.tsx`.

### Implementation for User Story 1

- [X] T014 [US1] Make `HostReadinessService` inspect the captured operation connection and return scope-truthful distribution guidance in `src-tauri/src/services/host_readiness_service.rs`.
- [X] T015 [US1] Update `get_host_readiness` to use `AppState` and safe operation context in `src-tauri/src/commands/system.rs`.
- [X] T016 [US1] Expand the readiness presentation with connection scope, impact, remediation, and safe recovery actions in `src/components/system/HostReadinessPanel.tsx`.
- [X] T017 [US1] Add a dismissible, connection-keyed first-run readiness entry point before VM creation in `src/App.tsx` and `src/pages/Dashboard.tsx`.
- [X] T018 [US1] Invalidate and refetch onboarding data on active-connection changes in `src/hooks/useActiveConnection.ts` and `src/lib/tauri.ts`.

**Checkpoint**: A missing virtualization prerequisite is explainable before the VM wizard, with
guidance correct for the detected supported distribution and honest for other scopes.

---

## Phase 4: User Story 2 - Prepare Storage Without Hidden Defaults (Priority: P1)

**Goal**: A user explicitly selects an eligible connection-owned pool, or deliberately enters
reviewed storage setup, before a new disk can be created.

**Independent Test**: Fixtures for zero, inactive, multiple, and undersized pools require an
explicit valid UUID selection; the server rejects invalid selection before creating a volume or VM.

### Tests for User Story 2

- [X] T019 [P] [US2] Add storage readiness state and pool UUID selection tests in `src-tauri/tests/host_readiness.rs`.
- [X] T020 [P] [US2] Add no-mutation VM-creation rejection coverage for missing, stale, inactive, and undersized pools in `src-tauri/tests/mutation_outcomes.rs`.
- [X] T021 [P] [US2] Add pool selector, capacity message, cancellation, and connection-switch component tests in `src/components/vm/CreateVmWizard.test.tsx`.

### Implementation for User Story 2

- [X] T022 [US2] Implement a connection-scoped storage readiness assessment using libvirt pool UUID/state/capacity in `src-tauri/src/services/storage_service.rs`.
- [X] T023 [US2] Implement `preflight_vm_creation` with read-only storage validation in `src-tauri/src/commands/system.rs`.
- [X] T024 [US2] Require and validate `storage_pool_id` by UUID before `StorageVol::create_xml`, remove the named `default` lookup, and preserve cleanup semantics in `src-tauri/src/services/vm_service.rs`.
- [X] T025 [US2] Keep the captured connection attached to the VM mutation and map rejected preflight results to safe failures in `src-tauri/src/commands/vm.rs`.
- [X] T026 [US2] Add pool selection, capacity/readiness display, and preflight gating for new-disk installation modes in `src/components/vm/CreateVmWizard.tsx`.
- [X] T027 [US2] Show selected connection/target/effects and an explicit final confirmation before storage creation or activation in `src/components/storage/CreateStoragePoolWizard.tsx`.
- [X] T028 [US2] Refresh connection-owned storage, readiness, and wizard preflight queries after storage setup outcomes in `src/components/storage/CreateStoragePoolWizard.tsx` and `src/lib/tauri.ts`.

**Checkpoint**: The reported missing-`default` error is replaced by a selectable storage workflow;
no new-disk request can create a partial resource without an eligible selected pool.

---

## Phase 5: User Story 3 - Validate a Chosen Guest Profile Before Submission (Priority: P2)

**Goal**: Firmware, Secure Boot, TPM, and applicable network requirements are evaluated against
the selected connection before the user submits a Windows 11 or other constrained profile.

**Independent Test**: Capability fixtures produce a complete review on a capable connection and a
specific blocker with no resource mutation when firmware, Secure Boot, TPM, or network is missing.

### Tests for User Story 3

- [X] T029 [P] [US3] Add firmware, Secure Boot, TPM, and network preflight fixture coverage in `src-tauri/tests/host_readiness.rs`.
- [X] T030 [P] [US3] Add backend no-mutation preflight enforcement tests in `src-tauri/tests/mutation_outcomes.rs`.
- [X] T031 [P] [US3] Add guest capability review and stale-connection UI tests in `src/components/vm/CreateVmWizard.test.tsx`.

### Implementation for User Story 3

- [X] T032 [US3] Complete libvirt-backed firmware/Secure Boot/TPM/network capability evaluation in `src-tauri/src/services/host_readiness_service.rs`.
- [X] T033 [US3] Reuse the shared capability evaluator before volume allocation and domain definition in `src-tauri/src/services/vm_service.rs`.
- [X] T034 [US3] Add a guest capability review to the wizard's final step and invalidate it when profile or connection changes in `src/components/vm/CreateVmWizard.tsx`.
- [X] T035 [US3] Remove remaining fixed firmware and storage-path wording from user-facing recovery output in `src-tauri/src/services/vm_service.rs` and `src/components/vm/CreateVmWizard.tsx`.

**Checkpoint**: Windows 11 requirements are confirmed or specifically blocked before any disk or VM
mutation, for the active connection only.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Document supported behavior and validate the complete safety boundary.

- [X] T036 [P] Document first-run, storage selection, connection scope, and distro-specific recovery in `docs/LINUX_SUPPORT.md`.
- [X] T037 [P] Replace unsafe universal permission/path examples with scoped guidance in `docs/LIBVIRT_PERMISSIONS.md`.
- [X] T038 [P] Add a first-run and Windows 11 troubleshooting path in `docs/UEFI_SETUP.md`.
- [X] T039 Run Rust formatting, focused Rust tests, frontend tests, lint, and build from `specs/003-first-run-onboarding/quickstart.md`.
- [ ] T040 Run the isolated-host validation matrix and record safe release evidence in `specs/003-first-run-onboarding/quickstart.md`.
- [X] T041 [P] [US2] Add private-home-directory ISO import and source-preservation coverage in `src-tauri/tests/mutation_outcomes.rs` and `src-tauri/tests/fixtures/first-run-onboarding/`.
- [X] T042 [US2] Replace direct filesystem copying with a libvirt-managed volume upload stream in `src-tauri/src/services/storage_service.rs` so imported media receives the pool's policy.
- [X] T043 [US2] Add a confirmed local ISO import-copy command with connection-scope, capacity, and overwrite validation in `src-tauri/src/commands/storage.rs` and `src/lib/tauri.ts`.
- [X] T044 [US2] Offer “Import into selected storage pool” after local ISO selection, then attach the imported volume and preserve the source file in `src/components/vm/CreateVmWizard.tsx`.

---

## Phase 7: User Story 4 - Interactive Readiness Repair (Priority: P1)

**Goal**: Failed checks offer reviewed automation where supported and focused manual guidance elsewhere.

- [X] T045 [P] [US4] Add repair contracts in `src-tauri/src/models/host.rs` and `src/lib/types.ts`.
- [X] T046 [P] [US4] Add repair security tests in `src-tauri/tests/readiness_repair.rs`.
- [X] T047 [P] [US4] Add interactive panel tests in `src/components/system/HostReadinessPanel.test.tsx`.
- [X] T048 [US4] Derive repair descriptors in `src-tauri/src/services/host_readiness_service.rs`.
- [X] T049 [US4] Implement the fixed distro repair allowlist and privilege execution in `src-tauri/src/services/readiness_repair_service.rs` and `src-tauri/src/services/mod.rs`.
- [X] T050 [US4] Add confirmed repair execution in `src-tauri/src/commands/system.rs` and `src-tauri/src/lib.rs`.
- [X] T051 [US4] Add typed repair IPC in `src/lib/tauri.ts`.
- [X] T052 [US4] Add clickable review/manual/progress/outcome/refresh UI in `src/components/system/HostReadinessPanel.tsx`.
- [X] T053 [P] [US4] Document privilege and manual fallback boundaries in `docs/LINUX_SUPPORT.md` and `docs/LIBVIRT_PERMISSIONS.md`.
- [X] T054 [US4] Run all automated gates and isolated-host guidance in `specs/003-first-run-onboarding/quickstart.md`.

### Guided Secure Boot follow-up

- [X] T055 [P] [US4] Add regression coverage for copyable distro commands, explicit Secure Boot choices, and readiness recheck in `src/components/system/HostReadinessPanel.test.tsx` and `src-tauri/tests/host_readiness.rs`.
- [X] T056 [US4] Distinguish Secure Boot-capable firmware from enrolled-key availability and return detected Arch/CachyOS-specific guidance in `src-tauri/src/services/host_readiness_service.rs`.
- [X] T057 [US4] Present command steps, explicit alternatives, and a manual-action recheck control in `src/components/system/HostReadinessPanel.tsx` and document the limitation in `docs/UEFI_SETUP.md`.
- [X] T058 [US4] Run focused Rust/frontend tests plus formatting, lint, and build validation.

### Existing installation media follow-up

- [X] T059 [P] [US2] Add wizard regression coverage for discovering ISO volumes in the selected pool and accepting an existing pool path in `src/components/vm/CreateVmWizard.test.tsx`.
- [X] T060 [US2] Query connection-owned pool volumes and filter existing ISO media in `src/components/vm/CreateVmWizard.tsx`.
- [X] T061 [US2] Replace the import-only source field with an existing-ISO dropdown/text input and offer the existing volume when an import name conflicts in `src/components/vm/CreateVmWizard.tsx`.
- [X] T062 [US2] Run frontend tests, lint, production build, Rust regression tests, formatting, and diff validation.

---

## Dependencies & Execution Order

- Phase 1 establishes fixtures and validation evidence.
- Phase 2 provides shared types and safe libvirt inspection; it blocks all stories.
- US1 delivers the first-run/connection-scoped readiness MVP after Phase 2.
- US2 depends on the Phase 2 models and is integrated with US1's assessment UI.
- US3 depends on shared preflight and extends the same wizard review; it may start after Phase 2
  but should merge after US2's preflight flow.
- Polish follows the desired story increments.

## Parallel Opportunities

- T001/T002/T003 can run in parallel.
- T004/T005 and T006/T007 can be developed in parallel once their model interfaces are agreed.
- T011/T012/T013, T019/T020/T021, and T029/T030/T031 are independent test-file work.
- T036/T037/T038 are independent documentation work.

## Parallel Example: User Story 2

```text
Task: "T019 storage readiness tests in src-tauri/tests/host_readiness.rs"
Task: "T020 no-mutation rejection tests in src-tauri/tests/mutation_outcomes.rs"
Task: "T021 selector UI tests in src/components/vm/CreateVmWizard.test.tsx"
```

## Implementation Strategy

### MVP First

1. Complete phases 1 and 2.
2. Complete US1 and verify a connection change refreshes truthful readiness.
3. Complete US2 next to remove the hard-coded `default` storage-pool dependency.
4. Validate the no-mutation cases before proceeding to guest-profile enhancements.

### Incremental Delivery

1. Deliver connection-scoped first-run readiness.
2. Deliver explicit storage selection and server-side enforcement.
3. Deliver Windows 11/firmware/TPM review.
4. Complete support documentation and isolated-host validation.

All tasks use the required checkbox, ID, story label where applicable, and exact file paths.
