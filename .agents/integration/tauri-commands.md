# Tauri Commands - Backend/Frontend Contract

**Status**: Approved - Ready for Implementation
**Last Updated**: 2025-12-07
**Owners**: Backend Agent, Frontend Agent
**Reviewer**: Architecture Agent (Approved)

---

## Overview

This document defines the contract between the Tauri backend (Rust) and frontend (TypeScript). All commands must be implemented exactly as specified to ensure frontend/backend compatibility.

**Protocol**: Tauri IPC (JSON over WebSocket)
**Serialization**: serde_json (Rust) ↔ JSON (TypeScript)

---

## Command Naming Convention

- **Rust**: `snake_case` function names with `#[tauri::command]`
- **TypeScript**: `camelCase` when calling via `invoke()`
- **Tauri**: Automatically converts `snake_case` → `camelCase`

Example:
```rust
// Rust
#[tauri::command]
async fn get_vms(state: State<'_, AppState>) -> Result<Vec<VM>, String>
```

```typescript
// TypeScript
const vms = await invoke<VM[]>('getVms')
```

---

## Type Mapping

| Rust | TypeScript |
|------|------------|
| `String` | `string` |
| `u64`, `i64` | `number` |
| `bool` | `boolean` |
| `Vec<T>` | `T[]` |
| `Option<T>` | `T \| null` |
| `HashMap<K, V>` | `Record<K, V>` |
| `struct` with `#[derive(Serialize)]` | `interface` |

---

## Error Handling

All commands return `Result<T, String>`:
- **Success**: Frontend receives `T`
- **Error**: Frontend receives thrown exception with error message

**Backend Pattern**:
```rust
#[tauri::command]
async fn some_command() -> Result<Output, String> {
    something().map_err(|e| e.to_string())?;
    Ok(output)
}
```

**Frontend Pattern**:
```typescript
try {
    const result = await invoke<Output>('someCommand')
    // Handle success
} catch (error) {
    // error is the String from Rust
    toast.error(`Failed: ${error}`)
}
```

---

## VM Commands

### `get_vms`

**Description**: List all VMs (active and inactive)

**Rust**:
```rust
#[tauri::command]
async fn get_vms(state: State<'_, AppState>) -> Result<Vec<VM>, String>
```

**TypeScript**:
```typescript
function getVms(): Promise<VM[]>

// Usage
const vms = await invoke<VM[]>('getVms')
```

**Response**:
```json
[
  {
    "id": "vm-uuid-1234",
    "name": "ubuntu-dev",
    "state": "running",
    "cpuCount": 4,
    "memoryMb": 8192,
    "diskSizeGb": 50
  }
]
```

---

### `get_vm`

**Description**: Get detailed information about a single VM

**Rust**:
```rust
#[tauri::command]
async fn get_vm(state: State<'_, AppState>, vm_id: String) -> Result<VM, String>
```

**TypeScript**:
```typescript
function getVm(vmId: string): Promise<VM>

// Usage
const vm = await invoke<VM>('getVm', { vmId: 'vm-uuid-1234' })
```

---

### `start_vm`

**Description**: Start a stopped VM

**Rust**:
```rust
#[tauri::command]
async fn start_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>
```

**TypeScript**:
```typescript
function startVm(vmId: string): Promise<void>

// Usage
await invoke('startVm', { vmId: 'vm-uuid-1234' })
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"VM is already running"`
- `"libvirtd is not running"`
- `"Permission denied"`

---

### `stop_vm`

**Description**: Stop a running VM

**Rust**:
```rust
#[tauri::command]
async fn stop_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>
```

**TypeScript**:
```typescript
function stopVm(vmId: string): Promise<void>
```

---

### `pause_vm`

**Description**: Pause a running VM (suspend to RAM)

**Rust**:
```rust
#[tauri::command]
async fn pause_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>
```

**TypeScript**:
```typescript
function pauseVm(vmId: string): Promise<void>
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"VM is not running"`
- `"Failed to pause VM: {libvirt_error}"`

---

### `resume_vm`

**Description**: Resume a paused VM

**Rust**:
```rust
#[tauri::command]
async fn resume_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>
```

**TypeScript**:
```typescript
function resumeVm(vmId: string): Promise<void>
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"VM is not paused"`
- `"Failed to resume VM: {libvirt_error}"`

---

### `reboot_vm`

**Description**: Reboot a running VM

**Rust**:
```rust
#[tauri::command]
async fn reboot_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>
```

**TypeScript**:
```typescript
function rebootVm(vmId: string): Promise<void>
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"VM is not running"`
- `"Failed to reboot VM: {libvirt_error}"`

**Note**: Uses ACPI reboot (graceful). For force reboot, use `stop_vm` then `start_vm`.

---

### `force_stop_vm`

**Description**: Forcefully stop a VM (equivalent to pulling power plug)

**Rust**:
```rust
#[tauri::command]
async fn force_stop_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>
```

**TypeScript**:
```typescript
function forceStopVm(vmId: string): Promise<void>
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"Failed to force stop VM: {libvirt_error}"`

**Warning**: Should only be used when graceful shutdown fails. UI should show confirmation dialog.

---

### `delete_vm`

**Description**: Delete a VM and optionally its disk images

**Rust**:
```rust
#[derive(Deserialize)]
pub struct DeleteVmOptions {
    pub delete_disks: bool,  // If true, delete associated disk images
}

#[tauri::command]
async fn delete_vm(
    state: State<'_, AppState>,
    vm_id: String,
    options: DeleteVmOptions
) -> Result<(), String>
```

**TypeScript**:
```typescript
interface DeleteVmOptions {
    deleteDisks: boolean
}

function deleteVm(vmId: string, options: DeleteVmOptions): Promise<void>

// Usage
await invoke('deleteVm', {
    vmId: 'vm-uuid-1234',
    options: { deleteDisks: true }
})
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"Cannot delete running VM. Stop it first."`
- `"Failed to delete VM: {libvirt_error}"`
- `"Failed to delete disk: {path} - {error}"`

**Validation**:
- VM must be stopped before deletion
- UI should show confirmation dialog with disk deletion checkbox

---

### `create_vm`

**Description**: Create a new VM with specified configuration

**Rust**:
```rust
#[derive(Deserialize)]
pub struct VmConfig {
    pub name: String,
    pub cpu_count: u32,
    pub memory_mb: u64,
    pub disk_size_gb: u64,
    pub os_type: String,           // "linux", "windows", "other"
    pub os_variant: Option<String>, // e.g., "ubuntu22.04", "win11"
    pub iso_path: Option<String>,
    pub network: String,            // Network name or "default"
    pub disk_format: String,        // "qcow2", "raw"
    pub boot_menu: bool,
}

#[tauri::command]
async fn create_vm(
    state: State<'_, AppState>,
    config: VmConfig
) -> Result<String, String>  // Returns VM ID/UUID
```

**TypeScript**:
```typescript
interface VmConfig {
    name: string
    cpuCount: number
    memoryMb: number
    diskSizeGb: number
    osType: 'linux' | 'windows' | 'other'
    osVariant?: string  // For optimized settings
    isoPath?: string    // Path to installation ISO
    network: string
    diskFormat: 'qcow2' | 'raw'
    bootMenu: boolean
}

function createVm(config: VmConfig): Promise<string>

// Usage
const vmId = await invoke<string>('createVm', {
    config: {
        name: 'ubuntu-dev',
        cpuCount: 4,
        memoryMb: 8192,
        diskSizeGb: 50,
        osType: 'linux',
        osVariant: 'ubuntu22.04',
        isoPath: '/path/to/ubuntu-22.04.iso',
        network: 'default',
        diskFormat: 'qcow2',
        bootMenu: true
    }
})
```

**Errors**:
- `"VM name already exists: {name}"`
- `"Invalid CPU count: {cpu_count} (must be 1-256)"`
- `"Invalid memory size: {memory_mb}MB (minimum 128MB)"`
- `"Invalid disk size: {disk_size_gb}GB (minimum 1GB)"`
- `"ISO file not found: {iso_path}"`
- `"Network not found: {network}"`
- `"Failed to create disk image: {error}"`
- `"Failed to create VM: {libvirt_error}"`

---

## Data Types

### `VM`

**Rust**:
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct VM {
    pub id: String,                    // UUID
    pub name: String,
    pub state: VmState,
    pub cpu_count: u32,
    pub memory_mb: u64,
    pub os_type: Option<String>,       // "linux", "windows", "other"
    pub os_variant: Option<String>,    // e.g., "ubuntu22.04"
    pub disk_size_gb: u64,
    pub network_interfaces: Vec<NetworkInterface>,
    pub vnc_port: Option<u16>,         // VNC port if available
    pub created_at: Option<i64>,       // Unix timestamp
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VmState {
    Running,
    Stopped,
    Paused,
    Suspended,
    Crashed,  // Added for error state
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    pub mac: String,
    pub network: String,
    pub ip_address: Option<String>,  // Requires guest agent or DHCP lookup
}
```

**TypeScript**:
```typescript
interface VM {
    id: string
    name: string
    state: 'running' | 'stopped' | 'paused' | 'suspended' | 'crashed'
    cpuCount: number
    memoryMb: number
    osType?: string
    osVariant?: string
    diskSizeGb: number
    networkInterfaces: NetworkInterface[]
    vncPort?: number
    createdAt?: number
    description?: string
}

interface NetworkInterface {
    mac: string
    network: string
    ipAddress?: string
}
```

### `HostInfo`

**Rust**:
```rust
#[derive(Serialize, Clone)]
pub struct HostInfo {
    pub hostname: String,
    pub cpu_model: String,
    pub cpu_count: u32,
    pub cpu_threads: u32,
    pub memory_total_mb: u64,
    pub memory_free_mb: u64,
    pub libvirt_version: String,
    pub qemu_version: String,
    pub hypervisor: String,        // "QEMU/KVM"
    pub active_vms: u32,           // Currently running VMs
    pub total_vms: u32,            // Total VMs (running + stopped)
}
```

**TypeScript**:
```typescript
interface HostInfo {
    hostname: string
    cpuModel: string
    cpuCount: number
    cpuThreads: number
    memoryTotalMb: number
    memoryFreeMb: number
    libvirtVersion: string
    qemuVersion: string
    hypervisor: string
    activeVms: number
    totalVms: number
}
```

---

## System Commands

### `get_host_info`

**Rust**:
```rust
#[tauri::command]
async fn get_host_info(state: State<'_, AppState>) -> Result<HostInfo, String>
```

**TypeScript**:
```typescript
function getHostInfo(): Promise<HostInfo>
```

---

### `get_connection_status`

**Rust**:
```rust
#[derive(Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub uri: String,
    pub error: Option<String>,
}

#[tauri::command]
async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String>
```

**TypeScript**:
```typescript
interface ConnectionStatus {
    connected: boolean
    uri: string
    error: string | null
}

function getConnectionStatus(): Promise<ConnectionStatus>
```

---

## Console/Display Commands

### `get_vnc_info`

**Description**: Get VNC connection information for a running VM

**Rust**:
```rust
#[derive(Serialize)]
pub struct VncInfo {
    pub host: String,       // Usually "localhost" or "127.0.0.1"
    pub port: u16,          // VNC port (usually 5900 + display number)
    pub password: Option<String>,
    pub websocket_port: Option<u16>,  // For web-based VNC (noVNC)
}

#[tauri::command]
async fn get_vnc_info(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<VncInfo, String>
```

**TypeScript**:
```typescript
interface VncInfo {
    host: string
    port: number
    password?: string
    websocketPort?: number
}

function getVncInfo(vmId: string): Promise<VncInfo>
```

**Errors**:
- `"VM not found: {vm_id}"`
- `"VM is not running"`
- `"VNC not configured for this VM"`

**Note**: For Phase 1, may return direct VNC port. Phase 2 will add WebSocket proxy for noVNC.

---

## Guest Agent Commands

*(To be added in Phase 3 - Weeks 8+)*

### Future Commands (Placeholder)
- `guest_agent_status(vm_id)` - Check if guest agent is available
- `guest_get_info(vm_id)` - Get OS info, IP addresses
- `guest_shutdown(vm_id, force)` - Graceful shutdown via agent
- `guest_exec(vm_id, command, args)` - Execute command in guest

---

## Storage & Network Commands

*(To be added in Phase 2 - Weeks 5-8)*

### Future Commands (Placeholder)
- `get_storage_pools()` - List storage pools
- `get_volumes(pool_id)` - List volumes in a pool
- `create_volume(pool_id, config)` - Create new disk volume
- `get_networks()` - List virtual networks
- `create_network(config)` - Create virtual network

---

## Performance Commands

### `get_vm_stats`

**Description**: Get current performance statistics for a VM

**Rust**:
```rust
#[derive(Serialize, Clone)]
pub struct VmStats {
    pub vm_id: String,
    pub cpu_usage_percent: f64,
    pub memory_used_mb: u64,
    pub memory_available_mb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub timestamp: i64,
}

#[tauri::command]
async fn get_vm_stats(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<VmStats, String>
```

**TypeScript**:
```typescript
interface VmStats {
    vmId: string
    cpuUsagePercent: number
    memoryUsedMb: number
    memoryAvailableMb: number
    diskReadBytes: number
    diskWriteBytes: number
    networkRxBytes: number
    networkTxBytes: number
    timestamp: number
}

function getVmStats(vmId: string): Promise<VmStats>
```

**Note**:
- For stopped VMs, returns zeros
- Stats are cumulative (total bytes read/written since VM start)
- Use events for real-time updates (see event-schema.md)

---

## Validation Rules

**Backend must validate**:
- **VM names**: 1-64 chars, alphanumeric + dash/underscore, cannot start with number
- **CPU count**: 1-256 (warn if > host CPU count)
- **Memory**: 128MB minimum, cannot exceed 90% of host memory
- **Disk size**: 1GB minimum, cannot exceed available storage pool space
- **Network names**: Must reference existing libvirt network
- **ISO paths**: Must exist and be readable
- **Disk format**: Only "qcow2" or "raw" allowed

**Frontend should validate**:
- Same rules for immediate user feedback
- Backend is authoritative and will reject invalid requests

**Example Validation Errors**:
```
"Invalid VM name: must be 1-64 characters, alphanumeric with dash/underscore only"
"CPU count exceeds host capacity (requested: 16, available: 8)"
"Memory exceeds available host memory (requested: 32GB, available: 16GB)"
"Disk size exceeds storage pool capacity (requested: 500GB, available: 250GB)"
```

---

## Command Summary Table

### Phase 1 Commands (MVP - Weeks 2-4)

| Command | Purpose | Priority | Complexity |
|---------|---------|----------|------------|
| `get_vms` | List all VMs | Critical | Low |
| `get_vm` | Get VM details | Critical | Low |
| `start_vm` | Start VM | Critical | Medium |
| `stop_vm` | Stop VM gracefully | Critical | Medium |
| `force_stop_vm` | Force stop VM | High | Low |
| `pause_vm` | Pause VM | High | Low |
| `resume_vm` | Resume VM | High | Low |
| `reboot_vm` | Reboot VM | High | Low |
| `delete_vm` | Delete VM | High | Medium |
| `create_vm` | Create new VM | Critical | High |
| `get_host_info` | Get host information | Critical | Medium |
| `get_connection_status` | Check libvirt connection | Critical | Low |
| `get_vnc_info` | Get VNC connection info | High | Medium |
| `get_vm_stats` | Get VM performance stats | Medium | Medium |

**Total Phase 1 Commands**: 14

### Phase 2 Commands (Weeks 5-8)
- Storage pool management (4 commands)
- Network management (4 commands)
- Snapshot operations (3 commands)

**Total Phase 2 Commands**: ~11

### Phase 3 Commands (Weeks 8+)
- Guest agent operations (6 commands)

**Total Phase 3 Commands**: ~6

---

## Testing Contract

Both agents must ensure:
- All types match exactly between Rust and TypeScript
- All commands work as specified
- Error messages are user-friendly and actionable
- Performance meets targets:
  - List operations: <100ms for <100 VMs
  - State changes: <500ms
  - VM creation: <5 seconds (excluding disk allocation)
  - Stats queries: <50ms

---

## Change Process

**To modify this contract**:
1. Backend or Frontend Agent proposes change
2. Document in GitHub issue
3. Architecture Agent reviews and approves
4. Both agents implement simultaneously
5. Update this document
6. Notify other agent when complete

---

## TypeScript Type Generation

**Future**: Consider using rust-typegen or similar to auto-generate TypeScript types from Rust structs.

For now: Backend Agent provides type definitions, Frontend Agent implements matching interfaces.

---

*This is a living document. Keep it updated as commands are added or changed.*
