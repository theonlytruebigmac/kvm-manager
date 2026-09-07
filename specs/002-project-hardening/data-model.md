# Data Model: Trustworthy Core Operations

## OperationContext

Immutable context captured once before an operation begins.

| Field | Type | Rules |
|---|---|---|
| `operation_id` | opaque identifier | Unique per invocation; safe to log and display for support. |
| `operation_kind` | enum | Query, mutation, console, host-local, migration, or background. |
| `connection_id` | opaque identifier | Required; identifies one saved/active connection. |
| `connection_label` | string | User-approved display value; bounded and safe for diagnostics. |
| `connection_scope` | enum | Local-system, local-session, remote, or test. |
| `capabilities` | set of ConnectionCapability | Snapshot used for preflight and confirmation. |
| `target` | TargetIdentity | Exact resource kind and stable identity. |
| `captured_at` | timestamp | Used to detect stale confirmation and selection changes. |

The connection handle is runtime-only and is never serialized to the frontend or persisted.

## TargetIdentity

| Field | Type | Rules |
|---|---|---|
| `resource_kind` | enum | VM, network, filter, storage pool, volume, device, host rule, or window. |
| `stable_id` | string | Prefer UUID; bounded validated name only where no UUID exists. |
| `display_name` | optional string | Safe display value, never a raw path or definition. |

Targets are meaningful only together with `connection_id`; same-named resources on two connections
are distinct.

## ConnectionCapability

| Field | Type | Rules |
|---|---|---|
| `kind` | enum | Resource management, graphical console, serial console, host device, migration, or other explicit feature. |
| `state` | enum | Available, unavailable, degraded, or unknown. |
| `reason_code` | optional enum | Non-sensitive stable reason. |
| `recovery_action` | optional RecoveryAction | Must not contain credentials, paths, or raw host output. |
| `checked_at` | timestamp | Capability results expire or refresh after connection changes. |

State transitions: `unknown -> available|unavailable|degraded`; any connected state may return to
`unknown` after disconnect or reconnect.

## SafeFailure

| Field | Type | Rules |
|---|---|---|
| `code` | enum | Unavailable, invalid-input, conflict, unauthorized, integration, unsupported, partial, or internal. |
| `summary` | string | User-safe, bounded, and free of protected values. |
| `operation_id` | identifier | Correlates with safe diagnostics. |
| `connection_id` | optional identifier | Included when resolution reached a connection context. |
| `target` | optional TargetIdentity | Included only when safe and resolved. |
| `outcome` | MutationOutcome | Required for mutations. |
| `retryable` | boolean | True only when repeating without changed user intent is safe. |
| `recovery_action` | optional RecoveryAction | Explicit next step or reconciliation action. |

## MutationOutcome

| State | Meaning | Required behavior |
|---|---|---|
| `rejected` | No mutation began. | Safe correction or capability guidance may be offered. |
| `applied` | Intended state is confirmed. | Refresh using the same operation connection. |
| `rolled_back` | Mutation began but pre-operation state was restored. | Report failure and confirmed rollback. |
| `partial` | Residual state differs from both initial and intended state. | Identify safe residual facts and require reconciliation. |
| `unknown` | State could not be inspected safely. | Never report success; provide inspection guidance. |

## RecoveryAction

| Field | Type | Rules |
|---|---|---|
| `kind` | enum | Retry, reconnect, reselect, inspect, reconcile, open-settings, or none. |
| `label` | string | Concise user-facing action. |
| `requires_confirmation` | boolean | True for any follow-up mutation. |
| `expected_connection_id` | optional identifier | Prevents retry on a changed host. |

## ProtectedValue

Protected values are a classification, not a persistable record. Categories are credential,
encryption material, SSH key, guest content, console secret, raw resource definition, local host
path, and transport-sensitive detail. No IPC error or diagnostic event may accept a field of this
type.

## ConfirmationToken

| Field | Type | Rules |
|---|---|---|
| `operation_kind` | enum | Must match the subsequent mutation. |
| `connection_id` | identifier | Must equal the captured operation context. |
| `target` | TargetIdentity | Must equal the exact mutation target. |
| `effect_digest` | opaque digest | Represents the reviewed material effect without storing secrets. |
| `expires_at` | timestamp | Short-lived; rejected after connection or selection changes. |

State transitions: `issued -> consumed|expired|invalidated`. A token is single-use.
