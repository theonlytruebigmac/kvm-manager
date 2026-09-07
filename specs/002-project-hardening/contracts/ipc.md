# IPC Contract: Safe Operations and Connection Capabilities

## Contract rules

- Every resource operation resolves one `OperationContext` before it queries or mutates state.
- Mutations and their refresh use the same `connectionId`.
- Failures cross IPC only as `SafeFailure`; raw error chains are backend-only.
- Responses use camelCase field names to match existing Tauri serialization conventions.
- Adding a failure code is backward-compatible; changing the meaning of an existing code is not.

## Safe failure envelope

```json
{
  "code": "unavailable",
  "summary": "The selected connection is unavailable.",
  "operationId": "opaque-correlation-id",
  "connectionId": "connection-id",
  "target": {
    "resourceKind": "vm",
    "stableId": "resource-uuid",
    "displayName": "test-vm"
  },
  "outcome": "rejected",
  "retryable": true,
  "recoveryAction": {
    "kind": "reconnect",
    "label": "Reconnect and retry",
    "requiresConfirmation": false,
    "expectedConnectionId": "connection-id"
  }
}
```

The envelope MUST NOT contain a URI with credentials, host filesystem path, raw XML, command output,
guest initialization content, key, password, token, or stack trace.

## Connection capability response

```json
{
  "connectionId": "connection-id",
  "connectionLabel": "Lab host",
  "scope": "remote",
  "state": "connected",
  "capabilities": [
    {
      "kind": "resourceManagement",
      "state": "available",
      "checkedAt": "2026-09-06T19:00:00Z"
    },
    {
      "kind": "hostDevice",
      "state": "unavailable",
      "reasonCode": "requiresLocalHost",
      "checkedAt": "2026-09-06T19:00:00Z"
    }
  ]
}
```

## Mutation result

Successful and reconciled mutations return an explicit outcome rather than relying on an empty
response:

```json
{
  "operationId": "opaque-correlation-id",
  "connectionId": "connection-id",
  "target": {
    "resourceKind": "network",
    "stableId": "resource-uuid",
    "displayName": "isolated-lab"
  },
  "outcome": "applied"
}
```

`partial` and `unknown` are failure results and require a recovery action.

## Destructive confirmation

The confirmation preview returns connection identity, exact target, material effects, and an opaque
single-use token. The execute request supplies that token. Execution rejects it if the active
connection, target, reviewed effects, or expiry no longer match.

## Compatibility transition

Existing frontend wrappers that expect rejected strings may temporarily normalize either shape into
`SafeFailure`. A command group is complete only after its Rust commands emit typed failures directly,
its frontend wrappers no longer parse strings, and success/failure contract tests pass.
