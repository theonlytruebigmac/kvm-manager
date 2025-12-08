# Event Schema - Backend → Frontend Communication

**Status**: Complete - Ready for Implementation
**Last Updated**: 2025-12-07
**Owner**: Architecture Agent
**Implementers**: Backend Agent (emit), Frontend Agent (listen)

---

## Overview

This document defines all events emitted by the Tauri backend to the frontend. Events enable real-time updates without polling, keeping the UI synchronized with backend state changes.

**Protocol**: Tauri Event System
**Transport**: WebSocket (managed by Tauri)
**Serialization**: JSON

---

## Event Emission Pattern

### Backend (Rust)
```rust
use tauri::Manager;

// Emit to all windows
app_handle.emit_all("event-name", payload)?;

// Emit to specific window
window.emit("event-name", payload)?;
```

### Frontend (TypeScript)
```typescript
import { listen, UnlistenFn } from '@tauri-apps/api/event'

// Listen for event
const unlisten: UnlistenFn = await listen<PayloadType>('event-name', (event) => {
  console.log('Received:', event.payload)
  // Handle event
})

// Cleanup on unmount
unlisten()
```

---

## Core Events (Phase 1)

### `vm-state-changed`

**When Emitted**: When a VM's state changes (running → stopped, stopped → running, etc.)

**Trigger Sources**:
- User action (start, stop, pause, resume via UI)
- External change (virsh command, libvirt event)
- VM crashes or becomes unresponsive
- Scheduled operation completes

**Rust Payload**:
```rust
#[derive(Serialize, Clone)]
pub struct VmStateChangedPayload {
    pub vm_id: String,
    pub vm_name: String,
    pub old_state: VmState,
    pub new_state: VmState,
    pub timestamp: i64,  // Unix timestamp in milliseconds
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VmState {
    Running,
    Stopped,
    Paused,
    Suspended,
}

// Emit example
app_handle.emit_all("vm-state-changed", VmStateChangedPayload {
    vm_id: "vm-uuid-1234".to_string(),
    vm_name: "ubuntu-dev".to_string(),
    old_state: VmState::Stopped,
    new_state: VmState::Running,
    timestamp: chrono::Utc::now().timestamp_millis(),
})?;
```

**TypeScript Payload**:
```typescript
interface VmStateChangedPayload {
  vmId: string
  vmName: string
  oldState: 'running' | 'stopped' | 'paused' | 'suspended'
  newState: 'running' | 'stopped' | 'paused' | 'suspended'
  timestamp: number
}

// Listen example
listen<VmStateChangedPayload>('vm-state-changed', (event) => {
  const { vmId, newState } = event.payload

  // Update React Query cache
  queryClient.setQueryData(['vm', vmId], (old) => ({
    ...old,
    state: newState
  }))

  // Invalidate list query to refresh
  queryClient.invalidateQueries({ queryKey: ['vms'] })
})
```

**Frontend Handling**:
1. Update local cache (React Query or store)
2. Show toast notification for user-initiated changes
3. Update UI immediately (optimistic update)
4. Log state transition for debugging

---

### `vm-created`

**When Emitted**: After a new VM is successfully created

**Trigger Sources**:
- User creates VM via wizard
- VM imported from XML/OVA
- External creation detected

**Rust Payload**:
```rust
#[derive(Serialize, Clone)]
pub struct VmCreatedPayload {
    pub vm_id: String,
    pub vm_name: String,
    pub timestamp: i64,
}
```

**TypeScript Payload**:
```typescript
interface VmCreatedPayload {
  vmId: string
  vmName: string
  timestamp: number
}
```

**Frontend Handling**:
1. Invalidate VM list query (refetch to include new VM)
2. Show success toast: "VM '{vmName}' created successfully"
3. Optionally navigate to VM details page
4. Update dashboard statistics

---

### `vm-deleted`

**When Emitted**: After a VM is successfully deleted

**Trigger Sources**:
- User deletes VM via UI
- External deletion detected

**Rust Payload**:
```rust
#[derive(Serialize, Clone)]
pub struct VmDeletedPayload {
    pub vm_id: String,
    pub vm_name: String,
    pub timestamp: i64,
}
```

**TypeScript Payload**:
```typescript
interface VmDeletedPayload {
  vmId: string
  vmName: string
  timestamp: number
}
```

**Frontend Handling**:
1. Remove from React Query cache
2. Invalidate VM list query
3. Show toast: "VM '{vmName}' deleted"
4. Navigate away if currently viewing deleted VM
5. Update dashboard statistics

---

### `vm-stats-updated`

**When Emitted**: Periodically (every 2-5 seconds) with real-time VM performance stats

**Trigger Sources**:
- Background monitoring service in backend
- Only emitted for running VMs
- Stops when VM is stopped

**Rust Payload**:
```rust
#[derive(Serialize, Clone)]
pub struct VmStatsPayload {
    pub vm_id: String,
    pub cpu_usage_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub disk_read_bytes_sec: u64,
    pub disk_write_bytes_sec: u64,
    pub network_rx_bytes_sec: u64,
    pub network_tx_bytes_sec: u64,
    pub timestamp: i64,
}
```

**TypeScript Payload**:
```typescript
interface VmStatsPayload {
  vmId: string
  cpuUsagePercent: number
  memoryUsedMb: number
  memoryTotalMb: number
  diskReadBytesSec: number
  diskWriteBytesSec: number
  networkRxBytesSec: number
  networkTxBytesSec: number
  timestamp: number
}
```

**Frontend Handling**:
1. Update performance graphs (if VM details page is open)
2. Store recent stats for historical view (keep last 60 data points)
3. DO NOT update on every event if page is not visible (performance)
4. Use for dashboard aggregate statistics

**Performance Note**: Frontend should throttle/debounce updates if necessary. Consider using `requestAnimationFrame` for smooth graph updates.

---

### `connection-status-changed`

**When Emitted**: When libvirt connection status changes

**Trigger Sources**:
- Initial connection established
- Connection lost (libvirtd stopped)
- Connection restored
- Connection error

**Rust Payload**:
```rust
#[derive(Serialize, Clone)]
pub struct ConnectionStatusPayload {
    pub connected: bool,
    pub uri: String,
    pub error: Option<String>,
    pub timestamp: i64,
}
```

**TypeScript Payload**:
```typescript
interface ConnectionStatusPayload {
  connected: boolean
  uri: string
  error: string | null
  timestamp: number
}
```

**Frontend Handling**:
1. Show connection status in header/sidebar
2. If disconnected: show error banner with reconnect action
3. Pause polling/queries when disconnected
4. Auto-retry connection on restore
5. Show toast: "Connected to libvirt" or "Connection lost"

---

### `error`

**When Emitted**: When a background operation fails or critical error occurs

**Trigger Sources**:
- Background task failure (monitoring, event listener)
- Libvirt error during event processing
- System errors that don't map to specific operations

**Rust Payload**:
```rust
#[derive(Serialize, Clone)]
pub struct ErrorPayload {
    pub message: String,
    pub severity: ErrorSeverity,
    pub context: Option<String>,  // e.g., "VM start failed: ubuntu-dev"
    pub timestamp: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
```

**TypeScript Payload**:
```typescript
interface ErrorPayload {
  message: string
  severity: 'info' | 'warning' | 'error' | 'critical'
  context: string | null
  timestamp: number
}
```

**Frontend Handling**:
1. Show toast with appropriate severity styling
2. Log to console for debugging
3. For critical errors: show modal with details and retry option
4. Store in error log (accessible from settings/debug page)

---

## Phase 2 Events (Storage & Network)

### `storage-pool-changed`

**When Emitted**: Storage pool created, deleted, or state changed

```typescript
interface StoragePoolChangedPayload {
  poolId: string
  poolName: string
  action: 'created' | 'deleted' | 'started' | 'stopped'
  timestamp: number
}
```

### `network-changed`

**When Emitted**: Virtual network created, deleted, or state changed

```typescript
interface NetworkChangedPayload {
  networkId: string
  networkName: string
  action: 'created' | 'deleted' | 'started' | 'stopped'
  timestamp: number
}
```

---

## Phase 3 Events (Guest Agent)

### `guest-agent-connected`

**When Emitted**: Guest agent connection established or lost

```typescript
interface GuestAgentConnectionPayload {
  vmId: string
  connected: boolean
  agentVersion: string | null
  timestamp: number
}
```

### `guest-info-updated`

**When Emitted**: Guest OS information updated (IP address changed, etc.)

```typescript
interface GuestInfoUpdatedPayload {
  vmId: string
  ipAddresses: string[]
  hostname: string
  osVersion: string
  timestamp: number
}
```

---

## Event Handling Best Practices

### Backend Guidelines

1. **Emit Sparingly**: Only emit when state actually changes
2. **Include Context**: Always include vm_id, vm_name, timestamp
3. **Error Handling**: Don't crash if emit fails, log and continue
4. **Debounce**: For rapid changes (stats), debounce emissions
5. **Testing**: Mock events in tests

**Example Backend Service**:
```rust
pub struct EventService {
    app_handle: AppHandle,
    last_emitted: Arc<RwLock<HashMap<String, Instant>>>,
}

impl EventService {
    pub async fn emit_vm_state_changed(&self, vm_id: String, old_state: VmState, new_state: VmState) {
        // Only emit if state actually changed
        if old_state == new_state {
            return;
        }

        let payload = VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name: self.get_vm_name(&vm_id).await.unwrap_or_default(),
            old_state,
            new_state,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        if let Err(e) = self.app_handle.emit_all("vm-state-changed", payload) {
            tracing::error!("Failed to emit vm-state-changed: {}", e);
        }
    }
}
```

### Frontend Guidelines

1. **Cleanup Listeners**: Always unlisten in useEffect cleanup
2. **Update Cache**: Use events to update React Query cache
3. **Avoid Refetch**: Prefer optimistic updates over full refetch
4. **User Feedback**: Show toast for important events
5. **Throttle**: Throttle high-frequency events (stats)

**Example React Hook**:
```typescript
function useVmEvents() {
  const queryClient = useQueryClient()

  useEffect(() => {
    let unlisten: UnlistenFn | null = null

    const setupListener = async () => {
      unlisten = await listen<VmStateChangedPayload>('vm-state-changed', (event) => {
        const { vmId, newState, vmName } = event.payload

        // Update specific VM
        queryClient.setQueryData(['vm', vmId], (old: VM | undefined) =>
          old ? { ...old, state: newState } : undefined
        )

        // Invalidate list
        queryClient.invalidateQueries({ queryKey: ['vms'] })

        // Show toast
        toast.success(`${vmName} is now ${newState}`)
      })
    }

    setupListener()

    return () => {
      if (unlisten) unlisten()
    }
  }, [queryClient])
}
```

---

## Testing Events

### Backend Testing
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_event_emission() {
        let app = create_test_app();
        let service = EventService::new(app.handle());

        service.emit_vm_state_changed(
            "test-vm".to_string(),
            VmState::Stopped,
            VmState::Running
        );

        // Assert event was emitted
        // (Requires test helper to capture events)
    }
}
```

### Frontend Testing
```typescript
// Mock Tauri listen
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event, handler) => {
    // Store handler for later invocation
    return () => {} // unlisten
  })
}))

test('handles vm-state-changed event', () => {
  // Trigger mock event
  // Assert UI updated
})
```

---

## Event Flow Diagram

```
┌─────────────────┐
│  Libvirt Event  │
│  or User Action │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│  Backend Service Layer  │
│  - Detects state change │
│  - Validates change     │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│   EventService::emit    │
│   - Builds payload      │
│   - Emits via Tauri     │
└────────┬────────────────┘
         │
         │ Tauri Event System
         │ (WebSocket)
         ▼
┌─────────────────────────┐
│  Frontend Event Listener│
│  - Receives payload     │
│  - Parses JSON          │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  React Query Cache      │
│  - Updates cached data  │
│  - Invalidates queries  │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│    UI Re-renders        │
│  - Shows updated state  │
│  - Displays toast       │
└─────────────────────────┘
```

---

## Migration Plan

### Week 2-3: Core Events
- Implement: `vm-state-changed`, `vm-created`, `vm-deleted`, `connection-status-changed`
- Test with basic VM operations

### Week 4: Performance Events
- Implement: `vm-stats-updated`
- Optimize for performance (throttling, debouncing)

### Week 5+: Phase 2 Events
- Add storage and network events as those features are implemented

---

## Change Log

| Date | Event | Change |
|------|-------|--------|
| 2025-12-07 | All Phase 1 events | Initial specification |

---

**Review and Approval**:
- Architecture Agent: Approved (2025-12-07)
- Backend Agent: To implement emission
- Frontend Agent: To implement listeners

**Related Documents**:
- tauri-commands.md (command signatures)
- ADR-002: State Management Patterns (how events update state)
