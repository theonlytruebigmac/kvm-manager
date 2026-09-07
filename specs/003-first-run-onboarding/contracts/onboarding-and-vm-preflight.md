# IPC Contract: Onboarding and VM Preflight

This is an internal Tauri command contract. All returned and rejected payloads use camelCase and
safe failures; raw libvirt errors, paths, command output, and credentials are not exposed.

## `get_vm_creation_readiness`

Reads the active captured connection and performs no host mutation.

**Request**

```ts
type GetVmCreationReadinessRequest = {
  requestedDiskBytes?: number
  guestRequirements?: {
    firmware: 'bios' | 'uefi' | 'uefi-secure'
    tpmEnabled: boolean
    network?: string
  }
}
```

**Response**

```ts
type VmCreationReadiness = {
  checkedAt: string
  connectionId: string
  connectionLabel: string
  connectionScope: 'local_system' | 'local_session' | 'remote' | 'test'
  distribution: DistributionProfile
  overallState: 'ready' | 'degraded' | 'unavailable'
  capabilities: CapabilityResult[]
  storage: StorageReadiness
}
```

For remote, test, and local-session scope, `distribution` must state its limitation and no local
package/service/permission action may be offered. A disconnected or inaccessible connection
returns the existing safe failure envelope.

## `preflight_vm_creation`

Performs the same read-only check using the complete prospective `VmConfig` before the review
step. It returns a `GuestCapabilityReview` with `canCreate: false` and capability-specific recovery
information for unavailable prerequisites. It never creates a volume, activates a pool, or defines
a domain.

## `create_vm`

The existing command accepts the extended `VmConfig`.

```ts
type VmConfig = ExistingVmConfig & {
  storagePoolId?: string
}
```

For `iso`, `network`, and `manual`, `storagePoolId` is mandatory. Before `StorageVol::create_xml`,
the command must capture the active mutation connection, re-run the relevant preflight, look up the
selected pool by UUID, and reject with a safe `invalid_input`, `unavailable`, or `conflict` failure
if it is missing, inactive, belongs to a stale selection, or lacks capacity. It must not look up a
pool by the name `default`. `import` must not create a storage volume and may omit the field.

## Guided storage setup

The existing storage-pool creation/activation command remains a mutation command. Its UI contract
must show the captured connection label/scope, target pool name/type, requested autostart behavior,
and an explicit final confirmation before invocation. On success it invalidates connection-owned
pool and readiness queries. On cancellation it invokes no command.

## Interactive readiness repair

Capability results may contain a safe `repairAction`. Automated execution uses
`execute_readiness_repair(actionId, confirmationToken)`. It accepts no executable, arguments,
packages, services, paths, environment, or command text. The backend re-resolves a local-system
mutation, consumes confirmation bound to the action and canonical `execute` effect, re-detects the
verified distribution, and derives the process plan from a closed mapping. It returns only a safe
terminal result. Manual/navigation actions never invoke it; the frontend refreshes readiness after
every terminal automated outcome.
