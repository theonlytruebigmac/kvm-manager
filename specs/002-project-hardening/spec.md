# Feature Specification: Trustworthy Core Operations

**Feature Branch**: `002-project-hardening`

**Created**: 2026-09-06

**Status**: Draft

**Input**: User description: "Perform a full project review, identify bugs and improvements, translate
the findings into specifications, and continue improving the project."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Keep Management Data Safe (Priority: P1)

As a host administrator, I can create and change virtual machines, networks, storage, and guest
configuration without user-supplied values escaping their intended field or sensitive data appearing
in application diagnostics.

**Why this priority**: The application manages privileged host resources and guest credentials. A
malformed value or diagnostic leak can affect the host or expose access material even when the UI
appears to work normally.

**Independent Test**: Submit valid values plus a corpus containing markup delimiters, control
characters, shell metacharacters, oversized values, credentials, guest initialization content, and
host paths. Verify that valid requests produce one well-formed target definition, invalid requests
are rejected before mutation, and captured diagnostics contain none of the protected values.

**Acceptance Scenarios**:

1. **Given** a user-supplied name, address, path, description, or device value contains characters
   meaningful to the target configuration format, **When** the value is submitted, **Then** it is
   either safely represented as data or rejected before any host or guest state changes.
2. **Given** a request includes a password, encryption material, SSH key, guest initialization
   content, console credential, or local host path, **When** the operation succeeds or fails, **Then**
   application logs and user-visible diagnostic details do not disclose that protected value.
3. **Given** a user supplies a raw configuration document through an explicitly identified advanced
   workflow, **When** the document is submitted, **Then** it is parsed and validated against the
   allowed resource type before definition, and parse failures leave existing state unchanged.
4. **Given** a destructive or host-wide operation is requested, **When** the user proceeds, **Then**
   the application identifies the exact target and requires intentional confirmation before the
   mutation begins.

---

### User Story 2 - Manage the Connection Actually Selected (Priority: P2)

As a user with local or saved remote libvirt connections, I can trust that every supported action,
status indicator, console launch, and refresh applies to the connection currently identified by the
application.

**Why this priority**: Acting on a different host than the one shown can damage the wrong virtual
machine. Presenting a remote connection as active while management calls still use the local host is
a correctness and safety failure.

**Independent Test**: Connect two isolated libvirt hosts containing distinct, same-named test
resources. Select each connection in turn and verify that discovery, mutation, refresh, migration
targeting, and supported console actions affect only the selected host; verify unsupported actions
are visibly unavailable before invocation.

**Acceptance Scenarios**:

1. **Given** a local connection is selected, **When** the user lists or changes a resource, **Then**
   the request and subsequent refresh use that local connection.
2. **Given** a remote connection is selected, **When** the user performs a supported action, **Then**
   the request, result, and refresh use that same remote connection and never silently fall back to
   the local system connection.
3. **Given** a selected connection cannot support a feature such as a graphical console or a
   host-local device operation, **When** the user views that feature, **Then** the application marks
   it unavailable with a reason instead of claiming support or targeting another host.
4. **Given** an active connection is lost during an operation, **When** the failure is reported,
   **Then** the UI identifies the affected connection, preserves other saved connections, and offers
   a non-destructive retry or reselection path.

---

### User Story 3 - Recover from Failures Without Crashing (Priority: P3)

As a user, I receive a concise, actionable error when a host integration, malformed response,
window operation, or background task fails, while the rest of the application remains usable where
safe.

**Why this priority**: Host services and virtual machines change independently of the UI. Expected
failures must not become process crashes, frozen views, or accidental exposure of internal details.

**Independent Test**: Inject unavailable connections, poisoned synchronization state, malformed
configuration responses, missing commands, non-UTF-8 paths, and failed window creation. Verify each
public operation returns a classified error, no process or command handler panics, and unaffected
views remain usable.

**Acceptance Scenarios**:

1. **Given** a required connection or host service is unavailable, **When** a dependent command is
   invoked, **Then** it returns an unavailable or degraded result without terminating the process.
2. **Given** a host response is missing expected fields or uses an unfamiliar valid ordering or
   quoting style, **When** it is read, **Then** the application parses it structurally or returns a
   classified compatibility error without corrupting the definition.
3. **Given** an unexpected UI rendering error occurs in a production build, **When** the recovery
   view is shown, **Then** it offers a safe retry path without displaying a stack trace, credentials,
   or local filesystem details.
4. **Given** a partial multi-step mutation fails, **When** recovery completes, **Then** the result
   identifies whether the change was applied, rolled back, or requires explicit reconciliation.

---

### User Story 4 - Trust Product Claims and Quality Evidence (Priority: P4)

As a maintainer or user, I can distinguish implemented capabilities from placeholders and can rely
on repeatable quality checks to catch regressions at privileged boundaries.

**Why this priority**: Hard-coded status, placeholder version data, dormant controls, and shallow
tests make a successful build look more complete than the runtime behavior actually is.

**Independent Test**: Run the documented quality workflow from a clean checkout, inspect every
visible feature/status claim against live application state, and introduce representative failures
at each privileged boundary to confirm a required check fails.

**Acceptance Scenarios**:

1. **Given** a capability is incomplete or unavailable, **When** it is shown in the product, **Then**
   it is disabled or explicitly labeled with its real status and does not display fabricated runtime
   data.
2. **Given** the project is built from a clean checkout, **When** the documented quality workflow
   runs, **Then** dependency resolution is repeatable and all required static checks, tests, and
   builds execute rather than silently skipping an absent check.
3. **Given** code changes configuration generation, connection routing, destructive confirmation,
   protected-data handling, or error recovery, **When** the change is proposed, **Then** automated
   tests for both success and failure behavior are required before it can pass quality checks.
4. **Given** product metadata or project links are displayed, **When** a user follows or inspects
   them, **Then** they identify the actual project and maintainer-owned location rather than template
   placeholders.

### Edge Cases

- A resource name is valid to libvirt but contains XML-sensitive characters, Unicode, leading or
  trailing whitespace, or resembles a different resource element.
- A credential or host path is embedded inside a nested error returned by an external integration.
- A connection changes or disconnects between initial discovery and a destructive confirmation.
- Two connections contain resources with the same name or identifier.
- A multi-step operation succeeds on its first mutation and fails on a later mutation.
- A valid host definition uses namespaces, optional elements, reordered attributes, or either quote
  style.
- A synchronization primitive is poisoned or a window closes while an asynchronous callback is
  updating its state.
- A quality tool is missing, a lock file is stale, or a required test suite discovers zero tests.
- A production page encounters an exception before logging is fully initialized.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST validate all user-controlled values at the boundary appropriate to
  their target resource before performing a host or guest mutation.
- **FR-002**: The system MUST represent accepted user-controlled values as data in generated or
  modified configuration documents so they cannot introduce sibling elements, attributes, or
  commands.
- **FR-003**: The system MUST structurally parse configuration documents received from users or host
  integrations and MUST reject malformed or wrong-resource documents before mutation.
- **FR-004**: The system MUST NOT record credentials, encryption material, SSH keys, guest
  initialization contents, console secrets, raw resource definitions, or local host paths in logs or
  user-visible internal diagnostics.
- **FR-005**: Diagnostic events for protected operations MUST retain a non-sensitive operation type,
  connection identity, target identifier suitable for display, outcome, and correlation context.
- **FR-006**: Destructive and host-wide operations MUST show the exact target, expected effect, and
  connection identity and MUST require an intentional confirmation that cannot be satisfied by a
  stale selection.
- **FR-007**: Every resource query, mutation, refresh, and supported console action MUST use the
  connection currently identified as active for that operation.
- **FR-008**: The system MUST NOT silently fall back to a local connection after a remote or alternate
  connection has been selected.
- **FR-009**: The system MUST expose connection-specific capability availability and MUST prevent
  invocation of operations that the selected connection cannot support.
- **FR-010**: Loss of an active connection MUST produce a recoverable state that identifies the
  affected connection without removing unrelated saved connections.
- **FR-011**: Public command and service boundaries MUST return classified failures for expected
  unavailable, invalid-input, conflict, authorization, integration, and partial-application outcomes
  rather than panic.
- **FR-012**: Multi-step mutations MUST either restore the pre-operation state after failure or report
  the exact residual state and a safe reconciliation action.
- **FR-013**: Production error views MUST not disclose stack traces, credentials, raw host responses,
  or local filesystem details and MUST offer a safe recovery action where possible.
- **FR-014**: The desktop content boundary MUST restrict executable and connectable content to the
  minimum local and console sources required by enabled features.
- **FR-015**: Visible connection status, version information, feature availability, and completion
  state MUST be derived from current application or host state and MUST NOT use fabricated
  placeholders.
- **FR-016**: Incomplete functionality MUST be disabled or explicitly marked unavailable and MUST
  not accept an action that can only end in a placeholder response.
- **FR-017**: The project MUST resolve application and build dependencies from committed, reviewable
  version locks during repeatable clean-checkout validation.
- **FR-018**: Required quality checks MUST fail when a configured formatter, static analyzer, test
  suite, or production build fails, is missing, or executes no applicable checks unexpectedly.
- **FR-019**: Changes to configuration generation, connection routing, destructive operations,
  protected-data handling, and failure recovery MUST include automated success, rejection, and
  integration-boundary coverage.
- **FR-020**: Project metadata, ownership fields, support links, and repository links MUST identify
  maintained project values and MUST contain no template placeholders in user-facing artifacts.

### Key Entities *(include if feature involves data)*

- **Operation Context**: The immutable connection identity, operation type, target identity,
  capability snapshot, and correlation identifier associated with one request.
- **Connection Capability**: A connection-scoped feature and its available, unavailable, degraded,
  or unknown state with a non-sensitive reason.
- **Classified Failure**: A stable failure category, safe summary, operation context, application
  state outcome, and optional recovery action.
- **Protected Value**: Credentials, keys, encryption material, guest contents, raw definitions,
  console secrets, and local host paths that must not cross diagnostic boundaries.
- **Mutation Outcome**: Whether an operation was applied, rejected before change, rolled back, or
  partially applied with reconciliation required.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A corpus of at least 100 malformed and format-sensitive values produces zero unintended
  configuration elements, attributes, host commands, or changes outside the selected target.
- **SC-002**: Automated diagnostic-capture tests covering every protected-value category report zero
  protected values in logs, error payloads, and production recovery views.
- **SC-003**: In a two-host isolated test, 100% of resource queries, mutations, refreshes, and enabled
  console actions affect only the connection identified in the operation context.
- **SC-004**: Fault-injection tests at every public command group produce zero process panics and a
  classified failure for every expected unavailable or malformed-input condition.
- **SC-005**: Every tested multi-step mutation reports exactly one of applied, rejected, rolled back,
  or partially applied, with no ambiguous success result.
- **SC-006**: A production security-boundary check reports no unrestricted content policy and no
  network or executable source that lacks an enabled-feature justification.
- **SC-007**: A clean-checkout quality run executes frontend tests, frontend static checks, production
  build, Rust formatting, Rust static analysis, and Rust tests with zero silently skipped required
  checks.
- **SC-008**: Automated boundary coverage includes at least one success and one failure case for every
  configuration generator, connection-routed command group, destructive-operation confirmation,
  and protected-data diagnostic path changed by this feature.
- **SC-009**: A repository scan of user-facing artifacts reports zero template owner, repository, or
  support-link placeholders.

## Assumptions

- Feature 001 remains responsible for Linux distribution readiness, firmware discovery, privileged
  port-forwarding, native packaging, and cross-distribution release smoke tests.
- Raw configuration import remains available as an advanced operation because it is useful to
  experienced libvirt administrators, but it receives structural validation and explicit risk
  labeling.
- Remote libvirt support is retained only for operations that can be routed and tested end to end;
  unsupported connection-specific operations are surfaced as unavailable rather than emulated
  through the local host.
- Existing operating-system credential agents and libvirt authentication mechanisms remain the
  source of connection credentials; adding a new credential vault is outside this feature unless
  later planning finds the application already persists secrets.
- Diagnostic identifiers may include user-approved display names, but raw definitions, guest
  contents, credentials, and local filesystem paths remain protected.
- Performance optimization and broad visual redesign are outside this feature except where needed to
  keep failure recovery usable and truthful.

## Out of Scope

- Adding a new virtualization backend other than libvirt/QEMU.
- Expanding the supported Linux distribution matrix.
- Replacing feature 001's host-readiness, firmware, forwarding-helper, packaging, or release work.
- Designing a cloud account or multi-user authorization system.
- General UI restyling or bundle-size optimization unrelated to a measured usability failure.
