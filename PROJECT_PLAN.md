# KVM/QEMU Manager - Project Plan

## Project Overview

A modern, fast, and user-friendly GUI manager for KVM/QEMU/libvirtd built with Rust. The goal is to provide a VMware Workstation or Hyper-V Manager quality experience for Linux virtualization that is opensource, performant, and feature-rich.

---

## 1. Technology Stack

### 1.1 GUI Framework - Tauri

**SELECTED: Tauri + Web Frontend**

Tauri provides the best of both worlds: Rust backend for performance and libvirt integration, with modern web technologies for the UI.

**Architecture Benefits:**
- **Rust Backend**: All libvirt operations, system calls, and business logic in Rust
- **Web Frontend**: Modern, flexible UI using web frameworks (React/Vue/Svelte/Solid)
- **IPC Communication**: Type-safe commands and events between frontend and backend
- **Small Bundle**: Uses system webview, no bundled Chromium (~3-5MB vs 100MB+ with Electron)
- **Security**: Built-in security with allowlist for IPC commands
- **Cross-Platform**: Potential to support macOS/Windows managing remote libvirt hosts
- **Developer Experience**: Hot reload for frontend, fast Rust compilation for backend

**Why Tauri for this project:**
1. Modern, polished UI achievable with web tech (matching VMware/Hyper-V quality)
2. Rust handles all performance-critical libvirt operations
3. Clear separation between presentation and business logic
4. Easier to iterate on UI/UX with web frameworks
5. Access to rich web ecosystem (charting libraries, UI components, animations)

### 1.2 Backend Libraries (Rust)

**Tauri Core:**
- **tauri**: Main framework for building the desktop app
- **tauri-plugin-***: Official plugins (store, shell, dialog, fs, etc.)

**Libvirt & System:**
- **virt** (rust-libvirt): Rust bindings for libvirt
- **tokio**: Async runtime for non-blocking operations
- **serde**: Serialization for IPC and configuration
- **tracing**: Logging and diagnostics
- **anyhow/thiserror**: Error handling

**Monitoring & Events:**
- **notify**: File system watching for libvirt changes
- **sysinfo**: Host resource monitoring (CPU, memory, disk)
- **chrono**: Time/date handling for scheduling and logs

### 1.3 Frontend Framework (To Be Decided)

**Options to consider:**

**React + TypeScript**
- Most popular, mature ecosystem, excellent libraries (TanStack Query, Recharts, shadcn/ui)
- Good for complex state management with Redux/Zustand/Jotai
- Best choice if team is familiar with React

**Svelte + TypeScript**
- Less boilerplate, reactive by default, smaller bundle size
- Growing ecosystem, good DX
- Excellent for performance-critical UIs

**Solid + TypeScript**
- Fine-grained reactivity, excellent performance
- Similar API to React but faster
- Smaller community but growing

**Vue 3 + TypeScript**
- Good balance of features and simplicity
- Composition API is clean and powerful
- Strong ecosystem

**Recommendation**: React or Svelte, both have excellent Tauri support and rich UI ecosystems

### 1.4 Additional Components

**Backend (Rust):**
- **XML handling**: quick-xml or roxmltree for libvirt domain XML manipulation
- **VNC/SPICE protocol**:
  - noVNC (web-based VNC client) for frontend integration
  - OR rust-vnc for native backend VNC proxy
  - Spice-HTML5 for web-based SPICE
- **Image handling**: image-rs for VM thumbnails/screenshots

**Frontend:**
- **UI Component Library**:
  - shadcn/ui (React), or Melt UI (Svelte), or similar
  - Tailwind CSS for styling
- **Data Visualization**: Recharts, Chart.js, or Apache ECharts for performance graphs
- **State Management**: TanStack Query for server state, Zustand/Jotai for client state (if React)
- **Icons**: Lucide Icons or Heroicons
- **Notifications**: sonner or react-hot-toast

---

## 2. Core Features (Prioritized)

### 2.1 Phase 1: MVP (Minimum Viable Product)

**VM Lifecycle Management**
- List all VMs with status (running, stopped, paused)
- Start, stop, pause, resume, reboot, force-off VMs
- Delete VMs
- View VM details (CPU, memory, disk, network config)
- Real-time status updates

**Console Access**
- Integrated VNC/SPICE console viewer
- Fullscreen mode
- Basic keyboard/mouse input handling

**Host Overview**
- Connection to local libvirtd
- Host resource summary (CPU, memory, storage)
- Active VM count and resource usage

**Basic VM Creation**
- Wizard-based VM creation
- OS selection (Linux, Windows, Other)
- Resource allocation (CPU cores, memory)
- Disk creation (qcow2, raw)
- Network selection (existing networks)
- ISO mounting for installation

### 2.2 Phase 2: Enhanced Features

**Advanced VM Management**
- Clone VMs
- Snapshot creation, deletion, revert
- Snapshot tree visualization
- VM export/import (OVA/OVF or libvirt XML)
- Live migration between hosts

**Storage Management**
- Storage pool listing and management
- Volume creation/deletion/resize
- Support for multiple storage backends (dir, LVM, NFS, iSCSI)
- Storage pool creation wizard
- Disk attachment/detachment for VMs

**Network Management**
- Virtual network listing and management
- Network creation/deletion
- DHCP range configuration
- NAT and bridged networking
- Port forwarding rules

**Enhanced UI**
- Dashboard with resource graphs
- VM grouping/tagging
- Search and filtering
- Multi-VM operations (bulk start/stop)
- Customizable layouts

### 2.3 Phase 3: Advanced Features

**Performance & Monitoring**
- Real-time CPU/memory/disk/network graphs per VM
- Historical performance data
- Resource alerts and notifications
- Performance optimization suggestions

**Automation & Scripting**
- Template system for VM creation
- Scheduled operations (auto-start, auto-stop)
- Backup scheduling
- CLI interface for automation
- REST API for external integration

**Advanced Networking**
- Open vSwitch integration
- VLAN configuration
- SR-IOV support
- Network topology visualization

**Cloud Integration**
- Cloud-init support
- Template library (popular OS images)
- Auto-download OS installation media
- Integration with cloud image repositories

**Security & Access Control**
- Remote libvirtd connection (SSH, TLS)
- Multi-host management
- User roles and permissions (if implementing custom auth)
- Secure credential storage

### 2.4 Phase 4: Polish & Extras

- Dark mode support
- Internationalization (i18n)
- Accessibility features
- Plugin/extension system
- Advanced console features (clipboard sharing, USB redirection)
- GPU passthrough configuration UI
- PCI device passthrough
- TPM and UEFI support configuration

---

## 3. Guest Agent Architecture

### 3.1 Overview

A lightweight agent that runs inside guest VMs to provide enhanced management capabilities, similar to:
- **qemu-guest-agent** (Proxmox/QEMU)
- **VMware Tools**
- **Hyper-V Integration Services**

The guest agent enables deep integration between the hypervisor and guest OS, providing capabilities that cannot be achieved through libvirt alone.

### 3.2 Guest Agent Features

**Phase 1: Core Agent (MVP)**
- Graceful shutdown/reboot (vs. force power off)
- Guest OS information:
  - OS type, version, kernel version
  - Hostname
  - IP addresses (all network interfaces)
  - MAC addresses
- Agent version and status reporting
- Heartbeat/health check

**Phase 2: Enhanced Management**
- File operations:
  - Copy files to guest (host → guest)
  - Copy files from guest (guest → host)
  - Create/delete directories
  - Read file contents
- Command execution:
  - Execute commands in guest
  - Return stdout/stderr/exit code
  - Run scripts
- User management:
  - List logged-in users
  - User session information

**Phase 3: Advanced Features**
- Filesystem quiescing for snapshots
  - Freeze/thaw filesystem before snapshot
  - Ensures consistent snapshots
- Time synchronization (guest ↔ host)
- Guest performance metrics:
  - CPU usage (per-core)
  - Memory usage (detailed)
  - Disk I/O statistics
  - Network statistics per interface
  - Process list and resource usage
- Volume resizing coordination
- Custom metadata exchange

**Phase 4: Platform-Specific**
- Windows:
  - Registry access
  - Windows service management
  - MSI package installation
  - Windows Update status
- Linux:
  - Package manager integration (apt, dnf, pacman)
  - Systemd service management
  - SELinux/AppArmor status

### 3.3 Architecture & Communication

```
┌─────────────────────────────────────────────────┐
│             KVM Manager (Host)                  │
│                                                 │
│  ┌──────────────────────────────────────────┐   │
│  │         Tauri Backend                    │   │
│  │  ┌────────────────────────────────────┐  │   │
│  │  │  Guest Agent Service               │  │   │
│  │  │  - Manages agent connections       │  │   │
│  │  │  - Routes commands to agents       │  │   │
│  │  │  - Collects agent data             │  │   │
│  │  └────────────────┬───────────────────┘  │   │
│  └───────────────────┼──────────────────────┘   │
│                      │                          │
└──────────────────────┼──────────────────────────┘
                       │ virtio-serial / VSOCK
                       │ (JSON-RPC protocol)
┌──────────────────────▼──────────────────────────┐
│            Guest VM (Linux/Windows)             │
│                                                 │
│  ┌──────────────────────────────────────────┐   │
│  │       KVM Manager Guest Agent            │   │
│  │                                          │   │
│  │  ┌────────────────────────────────────┐  │   │
│  │  │  Communication Module              │  │   │
│  │  │  - virtio-serial reader/writer     │  │   │
│  │  │  - JSON-RPC handler                │  │   │
│  │  └────────────┬───────────────────────┘  │   │
│  │               │                          │   │
│  │  ┌────────────▼───────────────────────┐  │   │
│  │  │  Command Handlers                  │  │   │
│  │  │  - shutdown, reboot, info          │  │   │
│  │  │  - file operations                 │  │   │
│  │  │  - exec, metrics                   │  │   │
│  │  └────────────────────────────────────┘  │   │
│  │                                          │   │
│  │  ┌────────────────────────────────────┐  │   │
│  │  │  OS Integration Layer              │  │   │
│  │  │  - Platform-specific APIs          │  │   │
│  │  │  - (Windows API, Linux syscalls)   │  │   │
│  │  └────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### 3.4 Communication Protocol

**Transport Layer Options:**

1. **virtio-serial** (Recommended)
   - Virtual serial port device
   - Bidirectional communication
   - No network dependency
   - Fast and reliable

2. **VSOCK** (Alternative)
   - Virtual socket (AF_VSOCK)
   - More socket-like API
   - Better for high-throughput scenarios

3. **Network socket** (Fallback)
   - TCP connection
   - Requires network configuration
   - Less secure, but universal

**Protocol: JSON-RPC 2.0**

Request from host:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "guest.get_info",
  "params": {}
}
```

Response from guest:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "os_type": "linux",
    "os_version": "Ubuntu 22.04",
    "hostname": "my-vm",
    "ip_addresses": ["192.168.122.100", "fe80::..."],
    "agent_version": "0.1.0"
  }
}
```

### 3.5 Guest Agent Implementation

**Technology Stack:**

**For Linux Guest Agent:**
- **Language**: Rust (cross-compile for different architectures)
- **Binary size**: ~2-5MB statically linked
- **Communication**: virtio-serial via `/dev/virtio-ports/org.kvmmanager.agent.0`
- **Installation**: .deb, .rpm, binary tarball
- **Service**: systemd unit file for auto-start

**For Windows Guest Agent:**
- **Language**: Rust (compile for Windows)
- **Binary size**: ~3-6MB
- **Communication**: virtio-serial via COM port
- **Installation**: MSI installer, auto-update support
- **Service**: Windows service (auto-start)

**Shared Core (Rust):**
```rust
// Shared protocol definitions
pub enum AgentCommand {
    GetInfo,
    Shutdown { force: bool },
    Reboot { force: bool },
    ExecuteCommand { command: String, args: Vec<String> },
    CopyFileToGuest { path: String, content: Vec<u8> },
    CopyFileFromGuest { path: String },
    GetMetrics,
}

pub struct AgentInfo {
    os_type: String,
    os_version: String,
    hostname: String,
    ip_addresses: Vec<String>,
    agent_version: String,
}
```

### 3.6 Host Integration

**Backend Service (Tauri):**

```rust
// src-tauri/src/services/guest_agent_service.rs

pub struct GuestAgentService {
    connections: Arc<RwLock<HashMap<String, AgentConnection>>>,
}

impl GuestAgentService {
    pub async fn connect_to_agent(&self, vm_id: &str) -> Result<()> {
        // Open virtio-serial channel to VM
        // Start message handling loop
    }

    pub async fn send_command(&self, vm_id: &str, cmd: AgentCommand) -> Result<Value> {
        // Send JSON-RPC request
        // Wait for response with timeout
    }

    pub async fn get_guest_info(&self, vm_id: &str) -> Result<GuestInfo> {
        self.send_command(vm_id, AgentCommand::GetInfo).await
    }
}
```

**Tauri Commands:**

```rust
#[tauri::command]
async fn guest_shutdown(
    state: State<'_, AppState>,
    vm_id: String,
    force: bool
) -> Result<(), String> {
    state.guest_agent_service
        .send_command(&vm_id, AgentCommand::Shutdown { force })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn guest_exec(
    state: State<'_, AppState>,
    vm_id: String,
    command: String,
    args: Vec<String>
) -> Result<ExecResult, String> {
    // Execute command in guest, return output
}

#[tauri::command]
async fn guest_get_metrics(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<GuestMetrics, String> {
    // Get real-time metrics from guest
}
```

### 3.7 Security Considerations

1. **Authentication**: Agent only accepts commands from libvirt/QEMU host
2. **Authorization**: Command allowlist (prevent dangerous operations)
3. **Encryption**: Optional TLS over virtio-serial for sensitive environments
4. **Input Validation**: Strict validation of all commands and parameters
5. **Sandboxing**: Agent runs with minimal privileges
6. **Audit Logging**: Log all commands executed via agent

### 3.8 Distribution & Installation

**Linux:**
```bash
# Debian/Ubuntu
curl -sSL https://kvm-manager.io/agent/install.sh | sudo bash
# or
sudo apt install kvm-manager-guest-agent

# Fedora/RHEL
sudo dnf install kvm-manager-guest-agent

# Manual
sudo systemctl enable --now kvm-manager-agent
```

**Windows:**
```powershell
# Download MSI installer
# Run installer (auto-starts service)
# Or via Chocolatey
choco install kvm-manager-guest-agent
```

**Cloud Images:**
- Pre-install agent in custom cloud images
- Cloud-init support for auto-install on first boot

### 3.9 Fallback Behavior

If guest agent is not installed or not responding:
- Application falls back to libvirt-only operations
- UI shows "Guest agent not available" status
- Basic features still work (power on/off, console access)
- Advanced features gracefully disabled with helpful messages

### 3.10 Benefits Over Libvirt Alone

| Capability | Libvirt Only | With Guest Agent |
|------------|--------------|------------------|
| Shutdown VM | Force power-off (risky) | Graceful OS shutdown |
| Get IP address | DHCP lease lookup (unreliable) | Direct from guest OS |
| Copy files | Not possible | Bidirectional file transfer |
| Run commands | Not possible | Full command execution |
| Metrics | Limited (virt-top) | Detailed guest-level metrics |
| Snapshots | Crash-consistent | Filesystem-consistent (quiesced) |
| Time sync | Manual setup | Automatic synchronization |

---

## 4. Application Architecture

### 4.1 Tauri Architecture Pattern

**Frontend-Backend Separation with IPC Communication**

```
┌─────────────────────────────────────────────────────┐
│                 FRONTEND (Web)                      │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │         UI Components (React/Svelte)        │   │
│  │  - VM List, Details, Console, Wizard       │   │
│  │  - Dashboard, Graphs, Settings             │   │
│  └───────────────────┬─────────────────────────┘   │
│                      │                             │
│  ┌───────────────────▼─────────────────────────┐   │
│  │         State Management                    │   │
│  │  - TanStack Query (server state)           │   │
│  │  - Zustand/Jotai (local UI state)          │   │
│  └───────────────────┬─────────────────────────┘   │
│                      │                             │
│  ┌───────────────────▼─────────────────────────┐   │
│  │         Tauri IPC Client                    │   │
│  │  - invoke() for commands                    │   │
│  │  - listen() for events                      │   │
│  └───────────────────┬─────────────────────────┘   │
└──────────────────────┼─────────────────────────────┘
                       │ IPC (JSON over WebSocket)
┌──────────────────────▼─────────────────────────────┐
│                 BACKEND (Rust)                      │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │         Tauri Command Handlers              │   │
│  │  - #[tauri::command]                        │   │
│  │  - Type-safe IPC endpoints                  │   │
│  └───────────────────┬─────────────────────────┘   │
│                      │                             │
│  ┌───────────────────▼─────────────────────────┐   │
│  │      Application State & Logic              │   │
│  │  - Arc<Mutex<AppState>>                     │   │
│  │  - Business logic orchestration             │   │
│  └───────────────────┬─────────────────────────┘   │
│                      │                             │
│  ┌───────────────────▼─────────────────────────┐   │
│  │         Service Layer (Async)               │   │
│  │  - LibvirtService                           │   │
│  │  - StorageService, NetworkService           │   │
│  │  - MonitoringService, EventService          │   │
│  └───────────────────┬─────────────────────────┘   │
│                      │                             │
│  ┌───────────────────▼─────────────────────────┐   │
│  │     Infrastructure Layer                    │   │
│  │  - virt (rust-libvirt)                      │   │
│  │  - File System, System APIs                 │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### 4.2 Key Design Principles

1. **Frontend-Backend Separation**: UI (web) and logic (Rust) are completely decoupled
2. **Async-First**: All libvirt operations are async to prevent blocking
3. **Event-Driven Updates**: Backend pushes real-time updates to frontend via Tauri events
4. **Type Safety**: Serde serialization ensures type-safe IPC communication
5. **Testability**: Backend services can be unit-tested independently of UI
6. **Error Resilience**: Graceful degradation when libvirt operations fail
7. **Security**: Tauri's command allowlist prevents unauthorized backend access

### 4.3 State Management

**Backend State (Rust)**
```rust
pub struct AppState {
    connection: Arc<Mutex<LibvirtConnection>>,
    vms: Arc<RwLock<HashMap<String, VmState>>>,
    networks: Arc<RwLock<HashMap<String, NetworkState>>>,
    storage_pools: Arc<RwLock<HashMap<String, StoragePoolState>>>,
    host_info: Arc<RwLock<HostInfo>>,
    event_tx: broadcast::Sender<AppEvent>,
}
```

**Frontend State (TypeScript/JavaScript)**
```typescript
// Server state (synced with backend)
const { data: vms } = useQuery({
  queryKey: ['vms'],
  queryFn: () => invoke<VM[]>('get_vms')
})

// Local UI state
const [selectedVm, setSelectedVm] = useState<string | null>(null)
const [viewMode, setViewMode] = useAtom(viewModeAtom)
```

**IPC Communication Flow**
1. **Frontend → Backend (Commands)**:
   - User clicks "Start VM" button
   - Frontend calls `invoke('start_vm', { vmId })`
   - Backend command handler executes operation
   - Returns result or error to frontend

2. **Backend → Frontend (Events)**:
   - Backend detects VM state change (libvirt event)
   - Emits event: `emit('vm-state-changed', { vmId, state })`
   - Frontend listener updates local cache
   - React Query refetches or updates optimistically

**Example Tauri Command:**
```rust
#[tauri::command]
async fn start_vm(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<(), String> {
    let connection = state.connection.lock().await;
    // ... libvirt operation
    Ok(())
}
```

### 4.4 Module Structure

```
kvm-manager/
├── src-tauri/                     # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json            # Tauri configuration
│   ├── build.rs
│   ├── icons/                     # App icons
│   └── src/
│       ├── main.rs                # Tauri app entry point
│       ├── lib.rs                 # Library root
│       ├── commands/              # Tauri command handlers
│       │   ├── mod.rs
│       │   ├── vm.rs              # VM commands (start, stop, etc.)
│       │   ├── network.rs         # Network commands
│       │   ├── storage.rs         # Storage commands
│       │   ├── guest.rs           # Guest agent commands
│       │   └── system.rs          # System/host commands
│       ├── services/              # Service layer (business logic)
│       │   ├── mod.rs
│       │   ├── libvirt.rs         # Core libvirt service
│       │   ├── vm_service.rs      # VM operations
│       │   ├── network_service.rs # Network operations
│       │   ├── storage_service.rs # Storage operations
│       │   ├── guest_agent_service.rs # Guest agent communication
│       │   ├── monitoring.rs      # Monitoring/stats
│       │   └── events.rs          # Event handling & emission
│       ├── models/                # Data models (serializable)
│       │   ├── mod.rs
│       │   ├── vm.rs              # VM model
│       │   ├── network.rs         # Network model
│       │   ├── storage.rs         # Storage model
│       │   └── host.rs            # Host info model
│       ├── state/                 # Application state
│       │   ├── mod.rs
│       │   └── app_state.rs       # Shared app state
│       ├── config/                # Configuration
│       │   ├── mod.rs
│       │   └── settings.rs        # App settings
│       └── utils/                 # Utilities
│           ├── mod.rs
│           ├── error.rs           # Error types
│           └── xml.rs             # XML helpers
│
├── src/                           # Frontend (React/Svelte/etc.)
│   ├── main.tsx                   # Frontend entry point
│   ├── App.tsx                    # Main app component
│   ├── pages/                     # Page components
│   │   ├── Dashboard.tsx          # Dashboard/overview page
│   │   ├── VmList.tsx             # VM list page
│   │   ├── VmDetails.tsx          # VM details page
│   │   ├── NetworkManager.tsx     # Network management
│   │   └── StorageManager.tsx     # Storage management
│   ├── components/                # Reusable UI components
│   │   ├── VmCard.tsx             # VM card component
│   │   ├── ResourceGraph.tsx      # Resource usage graph
│   │   ├── StatusBadge.tsx        # Status indicator
│   │   ├── ConsoleViewer.tsx      # VNC/SPICE console
│   │   └── wizards/
│   │       └── VmCreationWizard.tsx
│   ├── hooks/                     # Custom React hooks
│   │   ├── useVms.ts              # VM data hook
│   │   ├── useNetworks.ts         # Network data hook
│   │   └── useEventListener.ts    # Tauri event listener
│   ├── lib/                       # Utilities and helpers
│   │   ├── tauri.ts               # Tauri invoke wrappers
│   │   ├── types.ts               # TypeScript types
│   │   └── utils.ts               # Helper functions
│   ├── stores/                    # State management
│   │   └── ui-store.ts            # UI state (Zustand/Jotai)
│   └── styles/                    # Styles
│       └── globals.css            # Global styles, Tailwind imports
│
├── guest-agent/                   # Guest agent (separate workspace)
│   ├── Cargo.toml                 # Agent dependencies
│   ├── agent-common/              # Shared protocol/types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── protocol.rs        # JSON-RPC protocol definitions
│   │       └── types.rs           # Shared data types
│   ├── agent-linux/               # Linux guest agent
│   │   ├── Cargo.toml
│   │   ├── systemd/               # Systemd service files
│   │   │   └── kvm-manager-agent.service
│   │   └── src/
│   │       ├── main.rs            # Agent entry point
│   │       ├── transport.rs       # virtio-serial communication
│   │       ├── handlers/          # Command handlers
│   │       │   ├── mod.rs
│   │       │   ├── info.rs        # OS info, IP addresses
│   │       │   ├── power.rs       # Shutdown, reboot
│   │       │   ├── files.rs       # File operations
│   │       │   ├── exec.rs        # Command execution
│   │       │   └── metrics.rs     # Performance metrics
│   │       └── platform/          # Linux-specific code
│   │           ├── mod.rs
│   │           └── linux.rs       # Linux syscalls, /proc, /sys
│   └── agent-windows/             # Windows guest agent
│       ├── Cargo.toml
│       ├── installer/             # WiX installer config
│       └── src/
│           ├── main.rs            # Agent entry point
│           ├── transport.rs       # COM port communication
│           ├── handlers/          # Command handlers (shared)
│           ├── platform/          # Windows-specific code
│           │   ├── mod.rs
│           │   └── windows.rs     # Windows API calls
│           └── service.rs         # Windows service wrapper
│
├── package.json                   # Frontend dependencies
├── tsconfig.json                  # TypeScript config
├── vite.config.ts                 # Vite config (or webpack/etc.)
├── tailwind.config.js             # Tailwind CSS config
├── Cargo.toml                     # Workspace root (optional)
└── README.md
```

**Note**: The guest agent is a separate workspace to enable independent building and distribution for different platforms (Linux/Windows guests).

---

## 4. Implementation Roadmap

### 4.1 Phase 1: Foundation (Weeks 1-4)

**Week 1: Project Setup & Spike**
- [x] Initialize Tauri project (`npm create tauri-app`)
- [x] Choose and set up frontend framework (React/Svelte + TypeScript)
- [x] Configure Tailwind CSS and component library
- [x] Add rust-libvirt dependency to backend
- [x] Verify libvirt connectivity (create basic "list VMs" command)
- [x] Set up frontend to call backend command and display results
- [x] Establish build/dev workflow (npm run tauri dev)

**Week 2-3: Core VM Operations**
- [x] Create Tauri commands for VM operations (start, stop, pause, resume, reboot)
- [x] Implement backend services for VM lifecycle management
- [x] Build frontend UI for VM list with cards/table view
- [x] Display VM details (name, state, CPU, memory, disk)
- [x] Set up Tauri event system for real-time VM state updates
- [x] Implement error handling and user notifications (toast messages)

**Week 4: Basic Console & Creation**
- [x] Research and integrate web-based VNC viewer (noVNC)
- [x] Create backend proxy for VNC connections
- [x] Build simple VM creation wizard UI (multi-step form)
- [x] Implement create_vm Tauri command
- [x] Add VM deletion capability with confirmation dialog

**Milestone 1**: ✅ COMPLETE - Can list VMs, start/stop them, view console, and create basic VMs

### 4.2 Phase 2: Storage & Networking (Weeks 5-8)

**Week 5-6: Storage Management**
- [x] List storage pools and volumes
- [x] Create/delete storage volumes
- [x] Attach/detach disks to VMs
- [x] Storage pool creation
- [x] Volume resize functionality

**Week 7-8: Network Management**
- [x] List virtual networks
- [x] Create/delete networks
- [x] Configure DHCP ranges
- [x] Network selection in VM creation wizard
- [x] Network start/stop controls
- [x] Port forwarding rules

**Milestone 2**: ✅ COMPLETE - Full storage and network management capabilities

### 4.3 Phase 3: Advanced VM Features (Weeks 9-12)

**Week 9-10: Snapshots & Cloning**
- [x] Snapshot creation/deletion/revert
- [x] Snapshot tree visualization
- [x] VM cloning
- [x] VM export/import

**Week 11-12: Enhanced Creation & Monitoring**
- [x] Advanced VM creation wizard
- [x] Real-time resource graphs
- [x] Host resource monitoring
- [x] VM grouping/tagging
- [x] Historical performance data (SQLite-based metrics storage)

**Milestone 3**: ✅ COMPLETE - Production-ready VM management with snapshots and monitoring

### 4.4 Phase 4: Polish & Advanced (Weeks 13+)

**Week 13-14: Dashboard & UX**
- [ ] Dashboard with overview graphs
- [ ] Multi-VM operations
- [ ] Search and filtering
- [ ] Performance optimizations

**Week 15-16: Remote & Migration**
- [ ] Remote libvirt connections
- [ ] Multi-host management
- [ ] Live migration UI

**Week 17+: Advanced Features**
- [ ] Templates and automation
- [ ] Cloud-init integration
- [ ] GPU/PCI passthrough UI
- [ ] Plugin system

**Milestone 4**: Feature-complete professional VM manager

### 4.5 Guest Agent Development (Parallel Track)

The guest agent can be developed in parallel with the main application. It's a separate deliverable.

**Phase 1: Linux Agent MVP (Weeks 6-9)**

**Week 6-7: Core Agent Foundation**
- [ ] Set up guest-agent workspace structure
- [ ] Implement agent-common protocol library (JSON-RPC)
- [ ] Implement virtio-serial transport layer for Linux
- [ ] Basic command handling framework
- [ ] Implement `get_info` command (OS type, version, hostname, IPs)
- [ ] Implement `shutdown` and `reboot` commands
- [ ] Systemd service file and auto-start configuration

**Week 8-9: Host Integration & Testing**
- [ ] Implement guest_agent_service.rs in main app backend
- [ ] Tauri commands for guest agent operations
- [ ] UI indicators for agent status (installed/not installed/version)
- [ ] Test with Linux VMs (Ubuntu, Fedora)
- [ ] Package as .deb and .rpm
- [ ] Create installer script

**Milestone**: Linux guest agent working with graceful shutdown and OS info

**Phase 2: Enhanced Agent Features (Weeks 10-13)**

**Week 10-11: File Operations & Execution**
- [ ] Implement file transfer commands (host → guest, guest → host)
- [ ] Implement command execution with output capture
- [ ] Add security controls (command allowlist, path validation)
- [ ] UI for file transfer in VM details page
- [ ] UI for executing commands in guest

**Week 12-13: Metrics & Monitoring**
- [ ] Implement guest metrics collection (CPU, mem, disk, network)
- [ ] Real-time metrics streaming to host
- [ ] Display guest metrics in UI graphs
- [ ] Historical metrics storage (optional)

**Milestone**: Full-featured Linux guest agent with file ops and metrics

**Phase 3: Windows Agent (Weeks 14-18)**

**Week 14-15: Windows Agent Foundation**
- [ ] Port agent core to Windows
- [ ] Implement COM port transport for Windows
- [ ] Windows service wrapper implementation
- [ ] Test on Windows 10/11 VMs
- [ ] WiX MSI installer setup

**Week 16-17: Windows Platform Features**
- [ ] Windows-specific info (registry, services)
- [ ] Windows Update status
- [ ] MSI package installation support
- [ ] Service management commands

**Week 18: Cross-Platform Testing**
- [ ] Test agent on various Linux distros (Ubuntu, Debian, Fedora, Arch)
- [ ] Test agent on Windows 10, 11, Server 2019/2022
- [ ] Performance and stability testing
- [ ] Documentation for installation

**Milestone**: Production-ready guest agent for Linux and Windows

**Phase 4: Advanced Agent Features (Future)**

- [ ] Filesystem quiescing for consistent snapshots
- [ ] Time synchronization
- [ ] Volume resize coordination
- [ ] Custom metadata exchange
- [ ] Agent auto-update mechanism
- [ ] Multi-language support for error messages

---

## 5. Tauri IPC Command Structure

This section defines the key commands that the frontend can invoke on the backend.

### 5.1 VM Commands

```rust
// List all VMs
#[tauri::command]
async fn get_vms(state: State<'_, AppState>) -> Result<Vec<VM>, String>

// Get single VM details
#[tauri::command]
async fn get_vm(state: State<'_, AppState>, vm_id: String) -> Result<VM, String>

// VM lifecycle operations
#[tauri::command]
async fn start_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

#[tauri::command]
async fn stop_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

#[tauri::command]
async fn pause_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

#[tauri::command]
async fn resume_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

#[tauri::command]
async fn reboot_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

#[tauri::command]
async fn delete_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

// VM creation
#[tauri::command]
async fn create_vm(
    state: State<'_, AppState>,
    config: VmConfig
) -> Result<String, String>  // Returns VM ID

// Snapshots
#[tauri::command]
async fn create_snapshot(
    state: State<'_, AppState>,
    vm_id: String,
    snapshot_name: String
) -> Result<(), String>

#[tauri::command]
async fn revert_snapshot(
    state: State<'_, AppState>,
    vm_id: String,
    snapshot_name: String
) -> Result<(), String>
```

### 5.2 Storage Commands

```rust
#[tauri::command]
async fn get_storage_pools(state: State<'_, AppState>) -> Result<Vec<StoragePool>, String>

#[tauri::command]
async fn get_volumes(
    state: State<'_, AppState>,
    pool_id: String
) -> Result<Vec<Volume>, String>

#[tauri::command]
async fn create_volume(
    state: State<'_, AppState>,
    pool_id: String,
    config: VolumeConfig
) -> Result<String, String>
```

### 5.3 Network Commands

```rust
#[tauri::command]
async fn get_networks(state: State<'_, AppState>) -> Result<Vec<Network>, String>

#[tauri::command]
async fn create_network(
    state: State<'_, AppState>,
    config: NetworkConfig
) -> Result<String, String>
```

### 5.4 System Commands

```rust
#[tauri::command]
async fn get_host_info(state: State<'_, AppState>) -> Result<HostInfo, String>

#[tauri::command]
async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String>
```

### 5.5 Guest Agent Commands

```rust
// Check if guest agent is installed and responding
#[tauri::command]
async fn guest_agent_status(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<AgentStatus, String>

// Get guest OS information
#[tauri::command]
async fn guest_get_info(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<GuestInfo, String>

// Graceful shutdown via guest agent
#[tauri::command]
async fn guest_shutdown(
    state: State<'_, AppState>,
    vm_id: String,
    force: bool
) -> Result<(), String>

// Graceful reboot via guest agent
#[tauri::command]
async fn guest_reboot(
    state: State<'_, AppState>,
    vm_id: String,
    force: bool
) -> Result<(), String>

// Execute command in guest
#[tauri::command]
async fn guest_exec(
    state: State<'_, AppState>,
    vm_id: String,
    command: String,
    args: Vec<String>
) -> Result<ExecResult, String>

// Copy file to guest
#[tauri::command]
async fn guest_copy_to(
    state: State<'_, AppState>,
    vm_id: String,
    source_path: String,
    dest_path: String
) -> Result<(), String>

// Copy file from guest
#[tauri::command]
async fn guest_copy_from(
    state: State<'_, AppState>,
    vm_id: String,
    source_path: String,
    dest_path: String
) -> Result<(), String>

// Get guest performance metrics
#[tauri::command]
async fn guest_get_metrics(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<GuestMetrics, String>

// List logged-in users
#[tauri::command]
async fn guest_get_users(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<Vec<UserInfo>, String>
```

**Response Types:**
```rust
pub struct AgentStatus {
    installed: bool,
    version: Option<String>,
    connected: bool,
}

pub struct GuestInfo {
    os_type: String,          // "linux", "windows"
    os_version: String,       // "Ubuntu 22.04", "Windows 11"
    hostname: String,
    ip_addresses: Vec<String>,
    mac_addresses: Vec<String>,
    kernel_version: Option<String>,
}

pub struct ExecResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

pub struct GuestMetrics {
    cpu_usage_percent: f64,
    memory_used_mb: u64,
    memory_total_mb: u64,
    disk_io: Vec<DiskStats>,
    network_io: Vec<NetworkStats>,
}
```

### 5.6 Backend Events

Events emitted from backend to frontend:

```rust
// Event types
pub enum AppEvent {
    VmStateChanged { vm_id: String, state: VmState },
    VmCreated { vm_id: String },
    VmDeleted { vm_id: String },
    VmStatsUpdated { vm_id: String, stats: VmStats },
    ConnectionStatusChanged { status: ConnectionStatus },
    Error { message: String },
}

// Emit example
app_handle.emit_all("vm-state-changed", payload)?;
```

**Frontend Event Listeners:**
```typescript
import { listen } from '@tauri-apps/api/event'

// Listen for VM state changes
listen<{ vm_id: string, state: string }>('vm-state-changed', (event) => {
  queryClient.setQueryData(['vm', event.payload.vm_id], (old) => ({
    ...old,
    state: event.payload.state
  }))
})
```

---

## 6. Technical Considerations

### 6.1 Async Operations & Non-blocking UI

All libvirt operations must be async to avoid blocking the UI. Tauri commands are async by default:

```rust
#[tauri::command]
async fn start_vm(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<(), String> {
    // Long-running operation doesn't block the webview
    let connection = state.connection.lock().await;
    let domain = connection.lookup_domain_by_name(&vm_id)
        .map_err(|e| e.to_string())?;
    domain.create().map_err(|e| e.to_string())?;
    Ok(())
}
```

Frontend shows loading states while commands execute:
```typescript
const startVm = useMutation({
  mutationFn: (vmId: string) => invoke('start_vm', { vmId }),
  onSuccess: () => toast.success('VM started'),
  onError: (err) => toast.error(`Failed: ${err}`)
})
```

### 6.2 Real-time Updates

**Strategy 1: Libvirt Event Listener (Recommended)**
```rust
// Backend: Listen to libvirt events and emit to frontend
async fn setup_event_listener(app_handle: AppHandle, state: AppState) {
    tokio::spawn(async move {
        loop {
            // Poll libvirt for events
            if let Some(event) = check_libvirt_events().await {
                app_handle.emit_all("vm-state-changed", event).ok();
            }
        }
    });
}
```

**Strategy 2: Polling for Stats**
```typescript
// Frontend: Use React Query interval for performance stats
useQuery({
  queryKey: ['vm-stats', vmId],
  queryFn: () => invoke<VmStats>('get_vm_stats', { vmId }),
  refetchInterval: 2000, // Poll every 2 seconds
})
```

**Strategy 3: File Watching** (Optional)
- Use `notify` crate to watch libvirt XML directory
- Emit events when configurations change

### 6.3 Error Handling

**Backend Error Mapping:**
```rust
// Convert libvirt errors to user-friendly messages
fn map_libvirt_error(err: virt::error::Error) -> String {
    match err.code {
        Some(38) => "Libvirtd is not running. Please start the service.".to_string(),
        Some(1) => "Permission denied. Run as root or add user to libvirt group.".to_string(),
        _ => format!("Libvirt error: {}", err.message)
    }
}
```

**Frontend Error Display:**
- Toast notifications for operation failures
- Error boundaries for component failures
- Retry mechanisms with exponential backoff
- Graceful degradation (show cached data if backend unavailable)

### 6.4 Performance

**Backend:**
- Connection pooling for libvirt connections
- Caching frequently accessed data (VM list, host info)
- Efficient XML parsing with quick-xml
- Batch operations where possible

**Frontend:**
- Virtual scrolling for large VM lists (react-window or @tanstack/react-virtual)
- Code splitting for faster initial load
- Image optimization and lazy loading
- Debounced search and filters

### 6.5 Security

**Tauri Security:**
- Command allowlist in tauri.conf.json (only expose necessary commands)
- CSP (Content Security Policy) for webview
- No eval() or dangerous innerHTML usage
- Validate all user inputs before passing to libvirt

**Libvirt Security:**
- Connection security (local socket, SSH, TLS)
- Secure credential storage (use system keyring via tauri-plugin-stronghold)
- Input validation for VM XML generation
- Principle of least privilege

**Example: Command Allowlist**
```json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "scope": ["$APPDATA/*"]
      },
      "shell": {
        "open": false
      }
    }
  }
}
```

---

## 7. Development Workflow

### 7.1 Development Environment

**System Requirements:**
- **OS**: Linux (Fedora, Ubuntu, or Arch recommended)
- **Rust**: 1.70+ (install via rustup)
- **Node.js**: 18+ (install via nvm or system package manager)
- **libvirt/QEMU**: Install and configure libvirtd
- **Test VMs**: Small VMs for testing operations

**Development Tools:**
- **IDE**: VSCode with rust-analyzer, Tauri extension, and ESLint/Prettier
- **Browser DevTools**: Chrome DevTools for debugging webview
- **Rust Tools**: cargo-watch for auto-rebuild, cargo-clippy for linting

**Running Development Server:**
```bash
npm install                    # Install frontend dependencies
npm run tauri dev             # Start dev server with hot reload
```

### 7.2 Testing Strategy

**Backend Tests (Rust):**
```bash
cd src-tauri
cargo test                     # Run all tests
cargo test --lib               # Unit tests only
cargo test --test integration  # Integration tests only
```

1. **Unit Tests**: Service layer functions, XML parsing, error handling
2. **Integration Tests**: Libvirt operations (requires test environment with libvirtd)
3. **Mock Tests**: Mock libvirt responses for CI/CD

**Frontend Tests (TypeScript):**
```bash
npm test                       # Run Vitest/Jest tests
npm run test:e2e              # Playwright E2E tests
```

1. **Component Tests**: Individual React/Svelte components
2. **Hook Tests**: Custom hooks with @testing-library
3. **E2E Tests**: Critical user flows (create VM, start/stop)

**Performance Tests:**
- Load testing with 100+ VMs
- Memory profiling
- IPC latency benchmarks

### 7.3 CI/CD

**GitHub Actions Workflow:**

```yaml
# Backend
- Rust formatting (rustfmt)
- Rust linting (clippy)
- Cargo test
- Security audit (cargo audit)

# Frontend
- TypeScript/ESLint checks
- npm test
- Build verification

# Integration
- Tauri build for Linux
- E2E tests in GitHub-hosted runner
- Release builds for AppImage, .deb, .rpm
```

**Release Process:**
- Semantic versioning
- Automated changelog generation
- GitHub Releases with built artifacts
- Auto-update via Tauri updater

---

## 8. Packaging & Distribution

### 8.1 Target Formats

Tauri provides built-in bundling for multiple formats:

1. **AppImage**: Primary Linux distribution (self-contained, cross-distro)
2. **Debian package (.deb)**: For Ubuntu/Debian-based systems
3. **RPM package**: For Fedora/RHEL-based systems
4. **Flatpak**: Sandboxed distribution (requires additional configuration)
5. **MSI/NSIS** (Windows): If cross-platform remote management is added
6. **DMG** (macOS): If cross-platform remote management is added

**Build Commands:**
```bash
npm run tauri build              # Build for current platform
npm run tauri build -- --target all  # Build all formats
```

### 8.2 Dependencies

**Runtime Dependencies:**
- libvirt (libvirt0, libvirt-clients)
- qemu-kvm
- System webview (webkit2gtk on Linux)

**Build Dependencies:**
- Rust toolchain
- Node.js and npm
- libvirt development headers
- webkit2gtk development headers

**Bundle Configuration:**
- Include libvirt dependencies in package metadata
- Post-install scripts to verify libvirtd is running
- Desktop file with proper categories and icons

---

## 9. Documentation Plan

1. **User Documentation**
   - Installation guide
   - Quick start tutorial
   - Feature documentation
   - Troubleshooting guide

2. **Developer Documentation**
   - Architecture overview
   - Contribution guidelines
   - API documentation (rustdoc)
   - Development setup guide

---

## 10. Success Metrics

- **Performance**: Sub-100ms VM start action, <50MB memory overhead
- **Usability**: Create VM in <5 clicks, intuitive navigation
- **Reliability**: Handle libvirt failures gracefully
- **Features**: Match 80% of VMware Workstation/Hyper-V Manager features
- **Adoption**: Active user base, community contributions

---

## 11. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Libvirt API changes | High | Version detection, compatibility layers |
| rust-libvirt bindings outdated | Medium | Contribute to rust-libvirt, or write FFI bindings |
| Tauri IPC overhead | Medium | Batch operations, optimize serialization |
| Performance with many VMs | Medium | Efficient data structures, lazy loading, virtualization |
| Complex state synchronization | Medium | Use React Query for server state, events for real-time |
| VNC/SPICE web integration | Medium | noVNC/Spice-HTML5 proven solutions |
| Cross-distro compatibility | Low | AppImage and native packages |
| Webview inconsistencies | Low | Test on multiple webkit2gtk versions |

---

## 12. Next Steps

### Immediate Actions

1. **Frontend Framework Decision**: Choose between React, Svelte, Solid, or Vue
   - React recommended for ecosystem and component libraries
   - Svelte if prioritizing bundle size and performance

2. **Project Initialization**:
   ```bash
   npm create tauri-app
   # Choose your frontend framework
   # Add TypeScript support
   ```

3. **Basic Prototype** (Week 1 goal):
   - Set up Tauri with chosen frontend
   - Add rust-libvirt dependency
   - Create one command: `get_vms`
   - Display VM list in frontend
   - Verify build process works

4. **Repository Setup**:
   - Initialize git repository
   - Set up GitHub repo with issue templates
   - Create basic README
   - Add .gitignore for Rust + Node.js

5. **Follow Phase 1 Roadmap**: See Section 4.1

---

## 13. Questions for Consideration

**Please clarify the following to refine the plan:**

1. **Frontend Framework Preference**:
   - Do you have experience with React, Svelte, Vue, or Solid?
   - Any preference based on ecosystem, bundle size, or DX?

2. **Target Audience**:
   - Personal use, enterprise deployment, or both?
   - Single user or multi-user access control needed?

3. **Platform Scope**:
   - Linux-only initially?
   - Future interest in remote management from Windows/macOS?

4. **Remote Management Priority**:
   - Local libvirt only for MVP?
   - Or include remote SSH/TLS connections in Phase 1?

5. **UI Design Style**:
   - Modern web app aesthetic (think Vercel, Linear, Notion)?
   - Or more traditional desktop app look?
   - Dark mode important from the start?

6. **Existing Tools Reference**:
   - Any specific features from virt-manager to prioritize/improve?
   - Features from VMware Workstation you want to emulate?
