# Feature Specification: First-Run Virtualization Onboarding

**Feature Branch**: `003-first-run-onboarding`

**Created**: 2026-09-06

**Status**: Draft

**Input**: Help first-time users configure KVM Manager safely before they enter the VM creation
wizard, with guidance that respects supported Linux distribution differences instead of assuming
one host layout or a storage pool named `default`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Understand Host Readiness Before Creating a VM (Priority: P1)

As a first-time user, I can see whether my selected connection is ready for VM creation and receive
clear, distribution-appropriate next steps before investing time in a creation wizard.

**Why this priority**: A missing virtualization prerequisite currently appears only after the user
submits a VM definition, where the cause is hard to understand and may look like an OS-specific
failure.

**Independent Test**: Use supported-distribution fixtures with one prerequisite missing at a time;
verify that the onboarding view identifies the affected capability, explains the effect on VM
creation, and gives only the matching distribution family's setup guidance.

**Acceptance Scenarios**:

1. **Given** a supported host without usable virtualization access, **When** the user first opens
   the application or changes the selected connection, **Then** the user sees a readiness summary
   before opening the VM creation wizard and can review the appropriate recovery guidance.
2. **Given** a host where the user is missing required access or packages, **When** the user views
   the guidance, **Then** it identifies the detected distribution family and does not display
   commands, package names, or filesystem assumptions belonging to another family.
3. **Given** a best-effort distribution, **When** the user views onboarding, **Then** the
   application clearly distinguishes unverified guidance from verified support without preventing
   safe inspection of the selected connection.

---

### User Story 2 - Prepare Storage Without Hidden Defaults (Priority: P1)

As a user creating my first VM, I can select an existing usable storage location or explicitly
start a guided storage setup, so VM creation never silently assumes a pool named `default`.

**Why this priority**: Storage is required for ordinary ISO, network, and manual VM installations;
a missing or inactive pool should be resolved before a VM creation request is submitted.

**Independent Test**: Use selected connections with zero pools, inactive pools, multiple usable
pools, and insufficient free space; verify that the user must resolve storage selection before
creation and that no default-named pool is assumed.

**Acceptance Scenarios**:

1. **Given** the selected connection has no usable storage pool, **When** the user starts a VM
   creation flow requiring a new disk, **Then** the application explains that a storage location is
   required and offers an explicit path to inspect or create one before continuing.
2. **Given** the selected connection has multiple usable storage pools, **When** the user creates
   a VM, **Then** the user selects the intended pool and sees its safe name, state, available
   capacity, and connection ownership.
3. **Given** a storage setup action would change the host, **When** the user elects to create or
   activate storage, **Then** the application previews the affected connection and storage target,
   requires confirmation, and reports a clear outcome.
4. **Given** no storage pool can satisfy the requested disk size, **When** the user attempts to
   continue, **Then** the application blocks the request before creating a partial VM or disk.

---

### User Story 3 - Validate a Chosen Guest Profile Before Submission (Priority: P2)

As a user choosing Windows 11, UEFI, Secure Boot, or TPM options, I can see the selected host's
compatible capabilities and safe recovery guidance before submitting the VM.

**Why this priority**: Guest requirements such as secure firmware and TPM support vary by host and
distribution; finding a missing capability after disk creation wastes time and can leave cleanup
work.

**Independent Test**: Use fixtures with missing regular firmware, secure firmware, TPM support,
storage, and network availability; verify that each incompatible guest choice is identified before
a VM or disk mutation begins.

**Acceptance Scenarios**:

1. **Given** the user selects Windows 11 requirements on a capable host, **When** the user reviews
   the VM configuration, **Then** the application confirms compatible storage, firmware, TPM, and
   connection capabilities before allowing submission.
2. **Given** one required capability is unavailable, **When** the user reviews the configuration,
   **Then** the application identifies the specific unavailable capability and presents safe,
   distribution-appropriate recovery guidance without creating a disk or VM.
3. **Given** the selected connection changes while a setup or VM review is open, **When** the user
   continues, **Then** the application refreshes the prerequisites for the newly selected
   connection and invalidates the prior selection where needed.

---

### User Story 4 - Resolve Failed Readiness Checks Interactively (Priority: P1)

As a first-time user, I can select a failed readiness check and either run a reviewed supported
repair or follow focused manual steps, then see the check refreshed without leaving onboarding.

**Why this priority**: Plain remediation text still requires users to translate distro-specific
requirements into privileged terminal work and makes it unclear whether the repair succeeded.

**Independent Test**: For each connection scope and supported distribution fixture, open every
failed check and verify that local supported repairs show exact effects and confirmation, manual
requirements show guided steps, and remote/session/test connections never offer local execution.

**Acceptance Scenarios**:

1. **Given** a failed check with a supported local repair, **When** the user selects it, **Then**
   the application previews the detected distribution, selected connection, privilege requirement,
   and fixed effects before asking for confirmation.
2. **Given** the user confirms a supported repair, **When** authorization succeeds, **Then** the
   application executes only the predefined repair, reports its terminal outcome, and refreshes all
   readiness checks from the selected connection.
3. **Given** authorization is cancelled or the repair fails, **When** control returns to onboarding,
   **Then** no success is assumed and the user receives a safe retry or manual recovery route.
4. **Given** a check requires firmware setup, a new login session, best-effort distro knowledge, or
   remote administration, **When** the user selects it, **Then** the application provides guided
   steps without offering an unsafe automated action.

### Edge Cases

- A host has libvirt installed but no active storage pools, or pools exist but are inactive,
  inaccessible, or too small for the requested disk.
- A user selects a remote, test, local-session, or disconnected connection where local host setup
  cannot safely be offered.
- A distribution has a different package split, service activation convention, firmware location,
  permission model, or storage policy from another supported family.
- Storage creation succeeds but activation, capacity refresh, or later guest validation fails.
- A user dismisses onboarding, returns later, or changes a connection after completing a prior
  readiness review.
- A guest profile is valid on one selected connection but unavailable on another.
- A desktop privilege agent is unavailable, authorization is cancelled, or a package manager is busy.
- A repair completes but a new login, reboot, or firmware configuration is still required.
- Readiness changes between preview and confirmation, or the selected connection changes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST assess the selected connection's VM-creation prerequisites before a
  first-time user submits a VM creation request, including usable virtualization access, storage,
  requested guest capabilities, and applicable network availability.
- **FR-002**: The system MUST present first-run and connection-change onboarding as a clear
  readiness state with the impact of each unavailable prerequisite and a safe route to recover.
- **FR-003**: The system MUST use the detected distribution profile to present verified package,
  service, permission, firmware, and storage guidance for Arch/CachyOS, Debian/Ubuntu LTS,
  Fedora/RHEL-compatible, and openSUSE; other distributions MUST be explicitly marked
  best-effort.
- **FR-004**: The system MUST not automatically install packages, enable services, alter
  permissions, create directories, create or activate storage, or otherwise mutate a host during
  readiness inspection.
- **FR-005**: The system MUST never assume a storage pool named `default` or another fixed host
  layout for VM disk creation.
- **FR-006**: For VM installations requiring a new disk, the system MUST require a selected,
  usable storage pool belonging to the captured operation connection and validate available
  capacity before any disk or VM mutation begins.
- **FR-007**: When no usable storage pool exists, the system MUST provide an explicit guided path
  to inspect, create, or activate storage, while preserving a path to cancel and return to the
  application without host changes.
- **FR-008**: Every guided action that creates or activates storage MUST identify the selected
  connection and exact storage target, require intentional confirmation, and return a clear
  outcome and recovery action when unsuccessful.
- **FR-009**: Before a user submits a guest profile that requires UEFI, Secure Boot, TPM, or other
  host capability, the system MUST validate that capability on the selected connection and block
  an incompatible request before any VM or disk mutation begins.
- **FR-010**: The system MUST refresh setup and guest-capability results when the selected
  connection changes and MUST not reuse a prior connection's storage or readiness result.
- **FR-011**: Remote, test, disconnected, and local-session connections MUST show only setup
  guidance that is safe and truthful for their scope; the system MUST not offer a local-host setup
  action for a connection it cannot safely manage.
- **FR-012**: The system MUST preserve safe diagnostics: onboarding, setup outcomes, and errors
  may include safe connection and resource identity but MUST NOT include credentials, raw host
  output, guest content, or local paths.
- **FR-013**: The project MUST include automated coverage for supported distribution guidance,
  zero/multiple/inactive/insufficient storage-pool states, connection changes, guest-profile
  preflight failures, cancellation, and no-mutation rejection paths.
- **FR-014**: The project MUST document the first-run, storage, permissions, and guest-capability
  workflow for every supported distribution family and provide isolated validation guidance.
- **FR-015**: When a user selects a local ISO outside a libvirt-managed storage pool, the system
  MUST offer an explicit, confirmed import-copy into a selected eligible pool, preserve the source
  file by default, and attach the imported volume rather than assuming QEMU can read Downloads.
- **FR-016**: ISO import MUST show the captured local-system connection, source filename, selected
  pool, required capacity, and overwrite behavior before copying. Remote, test, and local-session
  connections MUST not offer a local filesystem import action outside their safe scope.
- **FR-017**: Each unavailable readiness result MUST expose either a selectable guided action or an
  explicit explanation that the result cannot be repaired from the application.
- **FR-018**: Automated host repair MUST be limited to local-system connections on a verified
  distribution and MUST use a predefined action; no user-provided process input is permitted.
- **FR-019**: Automated repair MUST preview connection, distribution, privilege, and complete
  effects and require confirmation bound to the action and connection.
- **FR-020**: Privileged repair MUST use desktop authorization without collecting, transporting,
  storing, or logging an administrator password.
- **FR-021**: Repair MUST report applied, rejected, cancelled, failed, or inspection-required and
  refresh readiness rather than infer success.
- **FR-022**: Package/service repair MUST use only verified distro mappings; best-effort profiles
  receive manual guidance only.
- **FR-023**: Firmware settings, login renewal, remote changes, and unverified repairs MUST remain
  guided manual actions.
- **FR-024**: Preview cancellation MUST cause no mutation and connection changes MUST invalidate it.
- **FR-025**: Tests MUST cover allowlisting, scope/distro gates, confirmation, cancellation,
  redaction, and refresh.

### Key Entities *(include if feature involves data)*

- **Onboarding Assessment**: A connection-scoped, non-destructive summary of prerequisites, their
  current state, user impact, and recovery guidance.
- **Storage Readiness**: The selected connection's usable storage choices, safe capacity and state
  details, and the reason a requested disk can or cannot be created.
- **Guest Capability Review**: The result of comparing the user's selected guest requirements with
  capabilities available on the captured connection.
- **Guided Setup Action**: A user-reviewed request to create or activate a specific storage target,
  with connection identity, effects, confirmation state, outcome, and recovery guidance.
- **Readiness Repair Action**: A connection-bound predefined automated or manual recovery path for
  one failed check, including effects, privilege, confirmation, and terminal outcome.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In supported-host fixtures with a missing package, access, storage, firmware, or TPM
  prerequisite, 100% of VM creation attempts are stopped before a VM or disk mutation and show a
  capability-specific recovery result.
- **SC-002**: In fixtures with zero, one, and multiple usable storage pools, 100% of new-disk VM
  creation flows display the selected connection's actual storage choices and never require a pool
  named `default`.
- **SC-003**: On each supported distribution family, a new tester can reach a ready-to-create
  result using only that family's documented guidance and without applying another family's setup
  instructions.
- **SC-004**: A user selecting Windows 11 requirements on a supported capable host receives a
  complete compatibility review before submission; an unavailable required capability is identified
  before any resource mutation in 100% of test cases.
- **SC-005**: Connection changes invalidate stale readiness and storage selections before a VM
  creation request is accepted in 100% of automated selection-race cases.
- **SC-006**: Automated and documented isolated-host validation covers every supported distribution
  family affected by a release that changes onboarding, storage, firmware, permission, or guest
  capability behavior.
- **SC-007**: In an isolated local-system fixture with a private home directory, 100% of selected
  Downloads ISO flows either attach a confirmed copy from the selected libvirt pool or explain why
  import cannot proceed; none require weakening home-directory permissions.
- **SC-008**: 100% of unknown, altered, stale-connection, non-local, or best-effort automated repair
  requests are rejected before process execution.
- **SC-009**: Every supported repair fixture permits preview, confirmation, outcome review, and a
  refreshed readiness result in one onboarding flow.
- **SC-010**: No repair result or diagnostic contains passwords, raw host output, local paths, or
  user-supplied command content.

## Assumptions

- The existing verified support matrix remains Arch/CachyOS, Debian/Ubuntu LTS,
  Fedora/RHEL-compatible, and openSUSE; other distributions receive safe best-effort guidance.
- First-run readiness is evaluated per selected connection, not solely per application install,
  because storage and guest capabilities can differ between connections.
- Inspection is non-destructive. Host-changing setup remains an explicit, confirmed user action
  that relies on libvirt wherever libvirt supports the operation.
- A user may manage storage outside the application; onboarding must support selecting a discovered
  usable pool rather than requiring application-created storage.
- Feature 001 remains the source of truth for distribution profile, firmware discovery, package,
  service, permission, and release-matrix behavior. This feature extends that work with first-run
  and storage-selection journeys; it does not duplicate forwarding or packaging scope.
- Package installation and service activation are the only automated OS repairs in this extension;
  all other recovery remains guided, and unavailable desktop authorization falls back safely.
