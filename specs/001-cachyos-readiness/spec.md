# Feature Specification: Modern Linux Readiness

**Feature Branch**: `001-cachyos-readiness`

**Created**: 2026-09-06

**Status**: Draft

**Input**: Prepare KVM Manager for supported modern Linux distributions by detecting host
prerequisites and firmware accurately, safely managing host networking, documenting setup, and
establishing repeatable validation and release gates.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Receive Distro-Accurate Setup Guidance (Priority: P1)

As a Linux user, I can identify whether my host is in the supported distribution matrix and receive
setup and recovery guidance using my distribution's current package, service, permission, and
firmware conventions.

**Why this priority**: Accurate setup guidance is the entry point for every supported host and
prevents users from applying another distribution's commands or filesystem assumptions.

**Independent Test**: Supply representative distribution identity fixtures for Arch/CachyOS,
Debian/Ubuntu LTS, Fedora/RHEL-compatible, openSUSE, and an unsupported distribution; verify that
each produces the intended support status and guidance.

**Acceptance Scenarios**:

1. **Given** a supported Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible, or openSUSE
   host, **When** the user views setup guidance, **Then** it uses the correct package manager,
   service convention, package names, and known firmware layout for that family.
2. **Given** a Linux distribution outside the support matrix, **When** the user views setup
   guidance, **Then** the application clearly identifies best-effort support and presents safe,
   generic diagnostics without claiming verified compatibility.

---

### User Story 2 - Diagnose Host Readiness (Priority: P2)

As a Linux user, I can open KVM Manager on a host that is not fully configured and receive a
clear readiness result with the missing capability, its effect, and the steps needed to proceed.

**Why this priority**: A user cannot safely create or manage a VM until the application can
distinguish an unavailable hypervisor from a usable host.

**Independent Test**: On a disposable supported host with QEMU, firmware, or libvirt access removed
one at a time, launch the application and verify that it remains usable enough to show
distribution-specific remediation rather than terminating unexpectedly.

**Acceptance Scenarios**:

1. **Given** QEMU is not installed, **When** the user opens the application, **Then** the user sees
   that VM creation is unavailable and receives distribution-appropriate installation guidance.
2. **Given** the user cannot access the selected libvirt connection, **When** the user opens the
   application, **Then** the user receives a connection diagnosis and can review the recovery steps.
3. **Given** the host meets all required capabilities, **When** the user opens the application,
   **Then** the readiness check reports it as ready without blocking normal VM management.

---

### User Story 3 - Create a UEFI Guest (Priority: P3)

As a Linux user, I can create a UEFI or Secure Boot VM using installed firmware without manually
finding or editing firmware paths.

**Why this priority**: Modern Linux and Windows guests require UEFI, and firmware layouts differ
across supported distributions.

**Independent Test**: On one host from each supported distribution family with its standard UEFI
firmware package installed, create a regular-UEFI VM and a Secure-Boot VM, then inspect the
resulting guest definitions and boot each to its installer.

**Acceptance Scenarios**:

1. **Given** compatible UEFI firmware is installed, **When** the user selects UEFI during VM
   creation, **Then** the application selects a valid code and variable-store template.
2. **Given** compatible Secure Boot firmware is installed, **When** the user selects Secure Boot,
   **Then** the VM definition uses compatible Secure Boot firmware.
3. **Given** no compatible firmware is installed, **When** the user selects a UEFI option, **Then**
   creation is prevented with distribution-appropriate recovery guidance.

---

### User Story 4 - Safely Publish a Guest Port (Priority: P4)

As a host administrator, I can create and remove a guest port-forwarding rule without the app
executing unintended host commands or leaving ambiguous network state.

**Why this priority**: Port forwarding changes host network exposure and is currently handled by
shell command construction.

**Independent Test**: On an isolated default virtual network, add and remove valid TCP and UDP
rules, attempt malformed destination input, and verify that only expected forwarding state changes.

**Acceptance Scenarios**:

1. **Given** valid protocol, port, and guest address values, **When** the administrator adds a
   forwarding rule, **Then** the rule is applied and its effective state is reported.
2. **Given** malformed or malicious address input, **When** the administrator submits the rule,
   **Then** the application rejects it before a host command or firewall change occurs.
3. **Given** a previously created rule, **When** the administrator removes it, **Then** the matching
   forwarding state is removed without affecting unrelated rules.
4. **Given** a native package installation with forwarding support, **When** the administrator
   requests a forwarding change, **Then** the operating system authorizes only that scoped action
   before host network state changes.
5. **Given** a portable installation without the authorized forwarding component, **When** the
   administrator opens forwarding, **Then** the application clearly states that this feature needs
   a supported native package and does not request broad administrator execution.

### Edge Cases

- A host has KVM kernel support but no QEMU emulator, or a running libvirt daemon with no usable
  QEMU capability.
- The detected `/etc/os-release` is incomplete, spoofed, or belongs to a distribution outside the
  support matrix.
- The same package is named, split, or activated differently across supported distribution families.
- Libvirt access is denied because the current user has not refreshed group membership or selected
  an unavailable connection.
- Multiple installed firmware layouts are available; the application must select a compatible,
  existing pair deterministically.
- The configured firewall backend cannot support the requested forwarding behavior or the caller
  lacks the required privilege.
- A requested forwarding rule conflicts with an existing host port or an identical rule.
- A requested guest address does not belong to the selected virtual network or to a guest interface
  known to libvirt.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST assess and report the availability of libvirt access, a usable QEMU
  emulator, KVM support, and firmware required for the user-selected VM type before a dependent
  operation is attempted.
- **FR-002**: The system MUST keep the application available when a required local virtualization
  capability is missing and MUST provide an actionable diagnostic rather than an unexplained exit.
- **FR-003**: The system MUST recognize supported UEFI and Secure Boot firmware through verified
  libvirt capabilities or current distribution layouts and use only verified existing firmware files.
- **FR-004**: The system MUST prevent UEFI VM creation when it cannot identify a compatible firmware
  and variable-store template, and MUST describe the prerequisite needed to continue.
- **FR-005**: The system MUST validate every port-forwarding input, selected virtual network, and
  guest destination before changing host networking.
- **FR-006**: The system MUST invoke host networking through structured arguments or an equivalent
  non-shell interface and MUST NOT interpolate user-controlled input into a shell command.
- **FR-007**: The system MUST report whether adding or removing a forwarding rule succeeded and
  MUST leave unrelated forwarding state unchanged on failure.
- **FR-008**: The system MUST document setup, service activation, permissions, UEFI, networking
  prerequisites, and known capability limits using current per-distribution package and path names.
- **FR-009**: The project MUST provide automated coverage for readiness decisions, firmware
  selection, and forwarding input validation, plus a documented isolated validation flow for every
  support-matrix family.
- **FR-010**: The release workflow MUST fail when required formatting, static analysis, or the
  feature's automated validation or artifact smoke test fails.
- **FR-011**: The system MUST identify the host distribution family using operating-system metadata
  and classify it as verified support or best-effort support.
- **FR-012**: The system MUST provide current, distribution-accurate virtualization prerequisites
  for Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible, and openSUSE hosts.
- **FR-013**: The project MUST validate documentation and host-readiness behavior on every supported
  distribution family affected by a release.
- **FR-014**: The system MUST require operating-system authorization through a narrowly scoped
  native component before it changes forwarding state; the desktop application itself MUST NOT run
  with broad administrator privileges.
- **FR-015**: The authorized forwarding component MUST manage only application-owned rules and MUST
  reject destinations that cannot be associated with the selected local virtual network.
- **FR-016**: A release MUST not be published until its native and portable artifacts pass their
  applicable installation and startup smoke tests on designated supported Linux runners.
- **FR-017**: Each release MUST provide a native install artifact for every verified distribution
  family: `.pkg.tar.zst` for Arch/CachyOS, `.deb` for Debian/Ubuntu, and `.rpm` for
  Fedora/RHEL-compatible and openSUSE. Each native artifact MUST include the authorized forwarding
  component; the AppImage MUST state that forwarding is unavailable.

### Key Entities *(include if feature involves data)*

- **Host Readiness Report**: The evaluated host capabilities, their readiness state, diagnostic
  messages, and recovery guidance.
- **Firmware Candidate**: A verified firmware code image and matching variable-store template with
  its boot-security capability.
- **Port-Forward Rule**: A requested and validated protocol, host port, guest address, guest port,
  ownership information, and observed application state.
- **Forwarding Authorization Request**: A network-scoped, user-approved request for the limited
  system operation needed to add or remove one Port-Forward Rule.
- **Distribution Profile**: A supported or best-effort Linux distribution family, its package and
  service conventions, firmware-discovery strategy, and guidance text.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On each of four intentionally incomplete CachyOS configurations (no emulator, no
  firmware, unavailable libvirt, insufficient network privilege), the application presents a
  capability-specific recovery result and does not terminate during startup.
- **SC-002**: A regular UEFI VM and a Secure Boot VM created on a current CachyOS host with standard
  firmware both reach their installer boot menu without manually supplied firmware paths.
- **SC-003**: The validation suite accepts valid TCP and UDP forwarding inputs and rejects a corpus
  of malformed address, port, and protocol inputs before any host networking operation is called.
- **SC-004**: Removing a forwarding rule leaves at least one unrelated forwarding rule functional in
  the isolated network validation scenario.
- **SC-005**: Each documented support-matrix quickstart can be completed by a new tester without
  relying on another distribution family's package names, paths, or ownership assumptions.
- **SC-006**: A supported-host fixture for each of the four distribution families resolves to its own
  correct setup guidance, while an unsupported-host fixture resolves to best-effort guidance.
- **SC-007**: Before a release that changes virtualization behavior or packaging, an isolated
  validation result is recorded for each affected supported distribution family.
- **SC-008**: A forwarding request with an unassociated guest destination is rejected before an
  authorization request or firewall mutation occurs.
- **SC-009**: On a supported native package install, an administrator can authorize one forwarding
  action while the desktop app continues to run without broad administrator privileges.
- **SC-010**: Each release artifact completes its designated install-and-start smoke test before the
  release publication step is permitted.
- **SC-011**: On each designated family runner, the matching native artifact installs, starts, and
  exposes the authorized forwarding workflow; the AppImage starts and reports the documented
  forwarding limitation.

## Assumptions

- The verified support matrix is Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible, and
  openSUSE. Other distributions receive safe best-effort diagnostics until explicitly added.
- Validation hosts are disposable or isolated; no automated test modifies a production VM, storage
  pool, physical interface, or firewall.
- The initial release target remains local libvirt management; complete remote-connection routing is
  outside this feature except for accurately reporting its present support level.
- Existing VNC and SPICE behavior is regression-tested but is not redesigned by this feature.
- Forwarding is supported only for local system connections and only in native packages that install
  the operating-system-authorized forwarding component; portable builds report this limitation.
