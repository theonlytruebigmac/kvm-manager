# Research: First-Run Virtualization Onboarding

## Decision: Make readiness connection-scoped and libvirt-backed

**Rationale**: The current `get_host_readiness` command runs local file/command probes and reports
`qemu:///system` regardless of the selected connection. VM creation and storage commands already
capture an operation connection through `AppState::resolve_operation`. A selected remote, test,
system, or session connection can have different pools and capabilities, so the onboarding
assessment must follow the same capture-and-query path.

**Alternatives considered**:

- Retain a local-only panel in Settings: rejected because it cannot explain the selected
  connection or prevent the reported no-pool failure.
- Infer readiness entirely in React from independent API calls: rejected because it permits stale
  selections and has no authoritative mutation guard.

## Decision: Require `storagePoolId` for new-disk creation

**Rationale**: `VmService::create_vm` currently looks up a pool literally named `default` after
validation. `StorageService::list_storage_pools` already returns UUIDs, state, capacity,
allocation, available space, type, and safe display metadata for the active connection. A UUID
selection is stable, can be checked server-side, and works with zero, one, or many pools.

**Alternatives considered**:

- Automatically choose the first active pool: rejected because a default hidden selection can
  send disks to an unintended host location.
- Continue requiring a hard-coded `default` pool: rejected by the feature and by real libvirt
  deployments where no such pool exists.
- Submit a filesystem path instead of a pool: rejected because it bypasses libvirt storage policy
  and varies between distributions.

## Decision: Preflight in the UI and repeat it inside `create_vm`

**Rationale**: The wizard needs prompt, actionable feedback, but UI checks can become stale when
the connection changes or a pool is removed. A preflight command provides an explainable review;
the mutation path repeats selection, pool state/capacity, and guest-capability checks before it
calls `StorageVol::create_xml` or defines a domain.

**Alternatives considered**:

- UI-only checks: rejected because callers can bypass IPC UI and because of time-of-check/time-of-
  use races.
- Command-only checks: rejected because first-time users would still learn requirements only on
  submit rather than during guided setup.

## Decision: Offer storage setup as an explicit handoff, not automatic repair

**Rationale**: The existing `CreateStoragePoolWizard` creates and starts a libvirt pool after a
review step. Onboarding should link into it with the selected connection clearly shown, require a
final intentional confirmation, then refetch readiness. Inspection must not create directories,
enable services, activate pools, or change permissions.

**Alternatives considered**:

- Automatically create `/var/lib/libvirt/images`: rejected because it changes the host and embeds
  a path, policy, and permission assumption that is not portable.
- Provide generic terminal commands: rejected because package, service, MAC policy, and ownership
  differ across the supported matrix.

## Decision: Centralize guest capability discovery in libvirt-compatible services

**Rationale**: Current VM creation checks several distro-specific firmware paths only after it may
have created a volume. The Feature 001 profile correctly says firmware should come from libvirt
capabilities and verified files. The implementation will expose a safe capability result for
firmware modes and TPM, make the UI use it, and make VM creation consume the same discovery logic.

**Alternatives considered**:

- Retain path probing in the wizard: rejected because browser code cannot authoritatively inspect
  the selected libvirt host and would repeat a per-distribution assumption.
- Treat Windows as a special hard-coded host path: rejected because requirements arise from the
  chosen UEFI/Secure Boot/TPM configuration, not the guest label alone.

## Decision: Treat remote, test, and local-session connections conservatively

**Rationale**: Local OS detection cannot describe a remote host. Assessment can still inspect
libvirt-managed resources, but only the local system connection can offer app-managed local-host
guidance. Other scopes receive explicit limitations and safe inspect/reselect/reconnect actions.

**Alternatives considered**:

- Reuse local distro guidance for remote connections: rejected as misleading.
- Disable all remote inspection: rejected because users still need to see connection-owned
  storage and guest compatibility.

## Decision: Use typed repair identifiers and desktop authorization

**Rationale**: Remediation text is not executable input. Failed checks receive a backend-derived
automated, manual, or navigation action. Execution accepts only a closed action identifier and a
connection-bound confirmation, re-derives the verified distro plan, and delegates authentication
to `pkexec`, so the application never receives an administrator password.

**Alternatives considered**:

- Execute remediation text or accept process arguments from React: rejected as arbitrary
  privileged input.
- Prompt for sudo credentials or run the application as root: rejected as unnecessary secret and
  privilege expansion.
- Automate firmware, login-session, remote, or best-effort repairs: rejected because they lack a
  portable verified operation.
