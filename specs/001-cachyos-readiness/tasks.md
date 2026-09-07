---

description: "Task list for Modern Linux Readiness implementation"
---

# Tasks: Modern Linux Readiness

**Input**: Design documents from `/specs/001-cachyos-readiness/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), and [quickstart.md](quickstart.md)

**Tests**: Required by the project constitution and FR-009. Write each listed automated test before
its implementation task and confirm it fails for the intended reason.

**Organization**: Tasks are grouped by user story so every story is independently testable.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish enforceable quality gates and test structure.

- [X] T001 Repair existing formatting violations in `src-tauri/src/` so `cargo fmt --check` passes.
- [X] T002 Resolve existing Clippy violations in `src-tauri/src/` so `cargo clippy -- -D warnings` passes.
- [X] T003 [P] Add frontend test commands and dependencies in `package.json` and a test configuration file at `vite.config.ts` or a dedicated test config.
- [X] T004 [P] Create Rust test fixtures for OS metadata and host capabilities in `src-tauri/tests/fixtures/`.
- [X] T005 Create a reusable quality workflow in `.github/workflows/quality.yml` for formatting, Clippy, Rust tests, frontend tests, and production build; call it from `.github/workflows/ci.yml`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create typed, non-destructive host-probing boundaries before any user workflow.

**⚠️ CRITICAL**: No user-story implementation begins until this phase is complete.

- [X] T006 Define distribution, readiness, capability, firmware, and forwarding data types in `src-tauri/src/models/host.rs`.
- [X] T007 [P] Add TypeScript equivalents and IPC result types in `src/lib/types.ts`.
- [X] T008 [P] Add unit tests for OS-release parsing and support-matrix classification in `src-tauri/tests/distribution_profiles.rs`.
- [X] T009 Implement pure `/etc/os-release` parsing and profile selection in `src-tauri/src/services/distribution_profile_service.rs`.
- [X] T010 [P] Add unit tests for non-sensitive readiness-result mapping in `src-tauri/tests/host_readiness.rs`.
- [X] T011 Implement a host-readiness service that returns independent capability results without mutating host state in `src-tauri/src/services/host_readiness_service.rs`.
- [X] T012 Refactor startup state handling in `src-tauri/src/state/app_state.rs` and `src-tauri/src/services/libvirt.rs` so unavailable local libvirt produces a degraded state instead of terminating the app.
- [X] T013 Register non-destructive readiness IPC commands in `src-tauri/src/commands/system.rs` and `src-tauri/src/lib.rs`.
- [X] T014 Add typed readiness IPC wrappers in `src/lib/tauri.ts`.

**Checkpoint**: The application can start and present a typed readiness result even when local
libvirt or QEMU is unavailable.

---

## Phase 3: User Story 1 - Receive Distro-Accurate Setup Guidance (Priority: P1) 🎯 MVP

**Goal**: Present the correct support status and setup guidance for each verified distribution
family without applying any host changes.

**Independent Test**: Feed Arch/CachyOS, Debian/Ubuntu, Fedora/RHEL-compatible, openSUSE, and
unsupported fixtures through the service and UI; verify the selected profile and guidance.

- [X] T015 [P] [US1] Add frontend tests for supported and best-effort guidance states in `src/components/system/HostReadinessPanel.test.tsx`.
- [X] T016 [P] [US1] Add profile-specific prerequisite, service, permission, firmware, and limitation content in `src-tauri/src/services/distribution_profile_service.rs`.
- [X] T017 [US1] Implement the Host Readiness Report presentation in `src/components/system/HostReadinessPanel.tsx`.
- [X] T018 [US1] Integrate the readiness panel into `src/pages/Settings.tsx` and provide an entry point from the existing application layout.
- [X] T019 [P] [US1] Create the support-matrix policy and profile-specific setup documentation in `docs/LINUX_SUPPORT.md`.
- [X] T020 [P] [US1] Update `README.md` and `docs/LIBVIRT_PERMISSIONS.md` to remove cross-distro package, path, and ownership assumptions.

**Checkpoint**: Every profile fixture produces correct guidance and an unsupported profile clearly
states best-effort status without showing another distribution’s commands.

---

## Phase 4: User Story 2 - Diagnose Host Readiness (Priority: P2)

**Goal**: Show capability-specific diagnostics while preserving a usable application shell.

**Independent Test**: Run readiness tests with mocked missing emulator, firmware, connection, and
privilege conditions; launch the app with an unavailable local connection and confirm it stays open.

- [ ] T021 [P] [US2] Add unit tests for QEMU emulator, KVM, libvirt-access, firmware, and privilege capability probes in `src-tauri/tests/host_readiness.rs`.
- [ ] T022 [US2] Implement capability probes and safe remediation mapping in `src-tauri/src/services/host_readiness_service.rs`.
- [ ] T023 [US2] Guard VM, storage, network, and console command entry points against a degraded connection in `src-tauri/src/commands/`.
- [ ] T024 [US2] Render unavailable and degraded capability states with recovery actions in `src/components/system/HostReadinessPanel.tsx`.
- [ ] T025 [US2] Add startup/degraded-state UI regression coverage in `src/components/system/HostReadinessPanel.test.tsx`.

**Checkpoint**: Missing prerequisites never cause an unexplained startup exit, and each missing
capability identifies its effect and family-specific recovery action.

---

## Phase 5: User Story 3 - Create a UEFI Guest (Priority: P3)

**Goal**: Create UEFI and Secure Boot guests through verified firmware selection across the support
matrix.

**Independent Test**: Use fixture capability data for each family plus real isolated-host smoke tests
to create regular and Secure Boot UEFI VM definitions without manually supplied paths.

- [ ] T026 [P] [US3] Add firmware selection and no-firmware regression cases in `src-tauri/tests/firmware_selection.rs`.
- [ ] T027 [US3] Implement capability-driven firmware discovery with verified existing fallback candidates in `src-tauri/src/services/firmware_service.rs`.
- [ ] T028 [US3] Add firmware-candidate IPC and frontend wrappers in `src-tauri/src/commands/system.rs` and `src/lib/tauri.ts`.
- [ ] T029 [US3] Replace hard-coded UEFI and Secure Boot path generation in `src-tauri/src/services/vm_service.rs` with selected candidates and blocking error states.
- [ ] T030 [US3] Update UEFI selection and recovery presentation in `src/components/vm/CreateVmWizard.tsx` and `src/components/vm/devices/BootEditor.tsx`.
- [ ] T031 [P] [US3] Rewrite firmware instructions for every supported family in `docs/UEFI_SETUP.md`.

**Checkpoint**: Valid UEFI and Secure Boot candidates are used automatically; an absent candidate
blocks creation with the matching distribution profile guidance.

---

## Phase 6: User Story 4 - Safely Publish a Guest Port (Priority: P4)

**Goal**: Create and remove exact forwarding rules through a narrowly scoped, OS-authorized native
helper without shell interpolation or ambiguous state.

**Independent Test**: Exercise a fake executor for malformed data and an isolated virtual network
for valid TCP/UDP add/remove operations, including one unrelated retained rule.

- [ ] T032 [P] [US4] Add acceptance, malicious-input, network-membership, exact-removal, helper-ownership, denied-authorization, and portable-unavailable cases in `src-tauri/tests/port_forwarding.rs`.
- [ ] T033 [US4] Define canonical network-scoped forwarding and authorization-request models, and validate selected libvirt-network membership before authorization, in `src-tauri/src/models/host.rs` and `src-tauri/src/services/network_service.rs`.
- [ ] T034 [US4] Implement and test the root-owned `kvm-manager-network-helper` binary in `src-tauri/binaries/kvm-manager-network-helper.rs`; it must repeat validation, use direct `nft` argument vectors, and add, remove, or inspect only application-owned rules.
- [ ] T035 [US4] Add explicit Arch (`src-tauri/packaging/arch/PKGBUILD`), Debian, and RPM native-package staging and installation for the helper and its narrowly scoped Polkit action. Verify the installed helper is root-owned at the policy's fixed path and test the `.pkg.tar.zst`, `.deb`, and `.rpm` artifacts; update `src-tauri/tauri.conf.json` and `src-tauri/packaging/polkit/com.fraziersystems.kvm-manager.network.policy` as needed. Mark AppImage artifacts as forwarding-unavailable rather than providing a privilege workaround.
- [ ] T036 [US4] Replace direct forwarding mutation in `src-tauri/src/services/network_service.rs`, `src-tauri/src/commands/network.rs`, and `src/lib/tauri.ts` with typed authorization-helper requests and present, absent, denied, failed, and unavailable results.
- [ ] T037 [US4] Update `src/components/network/PortForwardingManager.tsx`, `docs/LINUX_SUPPORT.md`, and `docs/CONSOLE_USER_GUIDE.md` with network selection, native-package authorization, portable limitation, firewall ownership, and isolated-validation guidance.

**Checkpoint**: Invalid data cannot reach an executor, successful operations report exact state, and
removing one rule does not affect an unrelated rule.

---

## Phase 7: Polish and Release Validation

**Purpose**: Enforce the support contract and complete release evidence.

- [ ] T038 [P] Configure `.github/workflows/ci.yml` to invoke the reusable quality workflow for each change and expose its required checks.
- [ ] T039 [P] Add a support-matrix evidence template, runner labels, artifact types, and expected portable-forwarding limitation to `docs/LINUX_SUPPORT.md`.
- [ ] T040 Add an isolated Arch/CachyOS runner job in `.github/workflows/ci.yml` that performs libvirt/readiness integration validation and install-and-start smoke tests for both the native `.pkg.tar.zst` artifact (including forwarding authorization) and the AppImage (including its expected forwarding-unavailable state).
- [ ] T041 [P] Add designated Debian/Ubuntu, Fedora/RHEL-compatible, and openSUSE runner procedures and native-package/AppImage smoke-test commands to `docs/LINUX_SUPPORT.md`.
- [ ] T042 Configure `.github/workflows/release.yml` to invoke `.github/workflows/quality.yml`, build the `.pkg.tar.zst`, `.deb`, `.rpm`, and AppImage release artifacts, and run each artifact's designated package-install and application-start smoke job before a separate publication job.
- [ ] T043 Run the full quickstart and CI/release artifact smoke validation on the designated isolated runners, then record support-matrix evidence in `specs/001-cachyos-readiness/quickstart.md` and `docs/LINUX_SUPPORT.md`.

---

## Dependencies and Execution Order

- Phase 1 unblocks all implementation by restoring enforceable quality gates.
- Phase 2 blocks every user story because it creates typed host and distro boundaries.
- US1 is the MVP and may ship after Phase 2.
- US2 depends on the readiness presentation delivered by US1.
- US3 depends on profile and readiness outputs from US1 and US2.
- US4 shares the typed error/result boundary from Phase 2 but is otherwise independent of UEFI work.
- Phase 7 follows all desired user-story phases.

## Parallel Opportunities

- T003–T005 can proceed in parallel after agreeing on the test and CI approach.
- T007, T008, and T010 can proceed in parallel with T006 once model shapes are agreed.
- Within US1, T015, T016, T019, and T020 are parallelizable.
- Within US3, T026 and T031 are parallelizable; within US4, T032 can begin before the executor.
- T038, T039, and T041 can proceed in parallel during release hardening.

## Implementation Strategy

### MVP First

1. Complete Phases 1 and 2.
2. Complete US1 and validate every distribution fixture.
3. Stop and review the guidance on at least one actual host in each family before advancing.

### Incremental Delivery

1. Add diagnostics without host mutation (US1 and US2).
2. Add capability-driven firmware selection (US3).
3. Add security-hardened forwarding (US4).
4. Enforce the cross-distro release gate (Phase 7).

## Notes

- All tasks use the required checkbox, ID, story label (where applicable), and exact file paths.
- The `001-cachyos-readiness` directory name is retained from initial feature creation; its current
  scope and title are Modern Linux Readiness.
- Tests must be written before their corresponding implementation task and run in isolated hosts.
