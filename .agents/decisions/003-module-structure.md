# ADR-003: Module Structure and Organization

Date: 2025-12-07
Status: Accepted
Decision Makers: Architecture Agent

## Context

With React + shadcn/ui chosen for the frontend, we need to finalize the module structure for both the Tauri backend and React frontend. The structure should:

1. Follow React and Rust best practices
2. Support shadcn/ui component organization
3. Enable clear separation between layers
4. Scale well as features are added
5. Make code easy to navigate and maintain

## Decision

The module structure from PROJECT_PLAN.md Section 4.4 is **approved with minor refinements** for React + shadcn/ui integration.

## Refined Module Structure

```
kvm-manager/
├── src-tauri/                          # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs                     # Tauri app entry point
│       ├── lib.rs                      # Library root
│       │
│       ├── commands/                   # Tauri command handlers
│       │   ├── mod.rs
│       │   ├── vm.rs                   # VM lifecycle commands
│       │   ├── network.rs              # Network commands (Phase 2)
│       │   ├── storage.rs              # Storage commands (Phase 2)
│       │   ├── guest.rs                # Guest agent commands (Phase 3)
│       │   └── system.rs               # System/host commands
│       │
│       ├── services/                   # Service layer (business logic)
│       │   ├── mod.rs
│       │   ├── libvirt.rs              # Core libvirt connection
│       │   ├── vm_service.rs           # VM operations
│       │   ├── network_service.rs      # Network operations
│       │   ├── storage_service.rs      # Storage operations
│       │   ├── guest_agent_service.rs  # Guest agent communication
│       │   ├── monitoring.rs           # Monitoring/stats
│       │   └── events.rs               # Event handling & emission
│       │
│       ├── models/                     # Data models (serializable)
│       │   ├── mod.rs
│       │   ├── vm.rs                   # VM, VmState, VmConfig
│       │   ├── network.rs              # Network models
│       │   ├── storage.rs              # Storage models
│       │   ├── host.rs                 # HostInfo, ConnectionStatus
│       │   └── events.rs               # Event payload types
│       │
│       ├── state/                      # Application state
│       │   ├── mod.rs
│       │   └── app_state.rs            # AppState struct
│       │
│       ├── config/                     # Configuration
│       │   ├── mod.rs
│       │   └── settings.rs             # App settings
│       │
│       └── utils/                      # Utilities
│           ├── mod.rs
│           ├── error.rs                # Error types
│           ├── xml.rs                  # XML helpers for libvirt
│           └── validation.rs           # Input validation
│
├── src/                                # React frontend
│   ├── main.tsx                        # Frontend entry point
│   ├── App.tsx                         # Main app component
│   ├── index.css                       # Global styles
│   │
│   ├── components/                     # React components
│   │   ├── ui/                         # shadcn/ui components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── table.tsx
│   │   │   ├── badge.tsx
│   │   │   ├── dropdown-menu.tsx
│   │   │   ├── input.tsx
│   │   │   ├── label.tsx
│   │   │   ├── toast.tsx
│   │   │   └── ...                     # Other shadcn components
│   │   │
│   │   ├── layout/                     # Layout components
│   │   │   ├── AppLayout.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── Footer.tsx
│   │   │
│   │   ├── vm/                         # VM-specific components
│   │   │   ├── VmCard.tsx
│   │   │   ├── VmList.tsx
│   │   │   ├── VmGrid.tsx
│   │   │   ├── VmDetails.tsx
│   │   │   ├── VmStateBadge.tsx
│   │   │   ├── VmActionMenu.tsx
│   │   │   └── VmConsole.tsx
│   │   │
│   │   ├── wizards/                    # Multi-step wizards
│   │   │   ├── VmCreationWizard/
│   │   │   │   ├── index.tsx
│   │   │   │   ├── BasicInfo.tsx
│   │   │   │   ├── Resources.tsx
│   │   │   │   ├── Storage.tsx
│   │   │   │   ├── Network.tsx
│   │   │   │   └── Review.tsx
│   │   │   └── ...
│   │   │
│   │   ├── charts/                     # Chart components
│   │   │   ├── CpuUsageChart.tsx
│   │   │   ├── MemoryUsageChart.tsx
│   │   │   ├── NetworkChart.tsx
│   │   │   └── ResourceGraph.tsx
│   │   │
│   │   └── common/                     # Shared/common components
│   │       ├── LoadingSpinner.tsx
│   │       ├── ErrorBoundary.tsx
│   │       ├── EmptyState.tsx
│   │       └── ConfirmDialog.tsx
│   │
│   ├── pages/                          # Page components (routes)
│   │   ├── Dashboard.tsx
│   │   ├── VmListPage.tsx
│   │   ├── VmDetailsPage.tsx
│   │   ├── NetworkManager.tsx          # Phase 2
│   │   ├── StorageManager.tsx          # Phase 2
│   │   └── Settings.tsx
│   │
│   ├── hooks/                          # Custom React hooks
│   │   ├── useVms.ts                   # VM queries
│   │   ├── useVmMutations.ts           # VM mutations
│   │   ├── useHost.ts                  # Host queries
│   │   ├── useVmEvents.ts              # Event listeners
│   │   ├── useConnectionStatus.ts      # Connection monitoring
│   │   └── useDebounce.ts              # Utility hooks
│   │
│   ├── lib/                            # Utilities and core logic
│   │   ├── tauri.ts                    # Tauri invoke wrappers
│   │   ├── query-client.ts             # TanStack Query config
│   │   ├── utils.ts                    # Utility functions (cn, etc.)
│   │   └── constants.ts                # App constants
│   │
│   ├── stores/                         # Zustand stores
│   │   ├── ui-store.ts                 # UI state (selected VM, filters, etc.)
│   │   └── settings-store.ts           # User preferences (persisted)
│   │
│   ├── types/                          # TypeScript types
│   │   ├── index.ts                    # Re-exports
│   │   ├── vm.ts                       # VM types (matches Rust models)
│   │   ├── host.ts                     # Host types
│   │   ├── network.ts                  # Network types
│   │   ├── storage.ts                  # Storage types
│   │   └── events.ts                   # Event payload types
│   │
│   └── styles/                         # Styles
│       └── globals.css                 # Global styles + Tailwind imports
│
├── guest-agent/                        # Guest agent (separate workspace)
│   ├── Cargo.toml
│   ├── agent-common/                   # Shared protocol
│   │   └── src/
│   │       ├── protocol.rs
│   │       └── types.rs
│   ├── agent-linux/                    # Linux agent
│   │   └── src/
│   │       ├── main.rs
│   │       ├── transport.rs
│   │       └── handlers/
│   └── agent-windows/                  # Windows agent
│       └── src/
│           ├── main.rs
│           └── ...
│
├── public/                             # Public assets
│   └── vite.svg
│
├── .github/                            # GitHub workflows
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── components.json                     # shadcn/ui config
├── Cargo.toml                          # Workspace root
└── README.md
```

## Key Refinements for React + shadcn/ui

### 1. Component Organization

**shadcn/ui Pattern**:
- All shadcn/ui components live in `src/components/ui/`
- These are "primitive" components copied from shadcn/ui
- Application-specific components compose these primitives

**Domain Components**:
- Organized by domain: `vm/`, `network/`, `storage/`
- Each domain has its own folder under `components/`

**Layout Components**:
- Separate `layout/` folder for app shell components

### 2. Hooks Organization

**Query Hooks**: All TanStack Query hooks in `hooks/`
- `useVms.ts` - All VM queries
- `useVmMutations.ts` - All VM mutations
- Pattern: Separate files for queries vs mutations

**Event Hooks**: Tauri event listeners
- `useVmEvents.ts` - Sets up global event listeners

**Utility Hooks**: Reusable logic
- `useDebounce.ts`, `useLocalStorage.ts`, etc.

### 3. Type Organization

**Mirrored Rust Types**:
- TypeScript types in `src/types/` mirror Rust models exactly
- Naming convention: `snake_case` in Rust → `camelCase` in TypeScript
- Same structure, just different casing

**Example**:
```rust
// Rust: src-tauri/src/models/vm.rs
#[derive(Serialize, Deserialize)]
pub struct VM {
    pub id: String,
    pub name: String,
    pub cpu_count: u32,
}
```

```typescript
// TypeScript: src/types/vm.ts
export interface VM {
  id: string
  name: string
  cpuCount: number
}
```

### 4. Wizard Components

**Multi-Step Wizards**:
- Each wizard gets its own folder under `components/wizards/`
- Contains `index.tsx` (main wizard logic) + step components
- Keeps wizard code organized and maintainable

### 5. shadcn/ui Configuration

**components.json**:
```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "default",
  "rsc": false,
  "tsx": true,
  "tailwind": {
    "config": "tailwind.config.js",
    "css": "src/styles/globals.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}
```

**Path Aliases** (tsconfig.json):
```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

This enables clean imports:
```typescript
import { Button } from '@/components/ui/button'
import { useVms } from '@/hooks/useVms'
import type { VM } from '@/types/vm'
```

## Backend Module Guidelines

### Command Handlers (`commands/`)

**Purpose**: Thin layer that validates input and delegates to services

**Pattern**:
```rust
#[tauri::command]
async fn start_vm(
    state: State<'_, AppState>,
    vm_id: String
) -> Result<(), String> {
    // Validate input
    if vm_id.is_empty() {
        return Err("VM ID cannot be empty".to_string());
    }

    // Delegate to service
    state.vm_service
        .start_vm(&vm_id)
        .await
        .map_err(|e| e.to_string())
}
```

### Service Layer (`services/`)

**Purpose**: Business logic, libvirt interactions, state management

**Pattern**:
```rust
pub struct VmService {
    connection: Arc<Mutex<LibvirtConnection>>,
    event_service: Arc<EventService>,
}

impl VmService {
    pub async fn start_vm(&self, vm_id: &str) -> Result<()> {
        // Business logic here
        let conn = self.connection.lock().await;
        let domain = conn.lookup_domain_by_uuid_string(vm_id)?;
        domain.create()?;

        // Emit event
        self.event_service.emit_vm_state_changed(vm_id, VmState::Running).await;

        Ok(())
    }
}
```

### Models (`models/`)

**Purpose**: Data structures that cross the IPC boundary

**Pattern**:
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct VM {
    pub id: String,
    pub name: String,
    pub state: VmState,
    // ... fields
}
```

## Frontend Module Guidelines

### Page Components

**Purpose**: Top-level route components

**Pattern**:
```typescript
// src/pages/VmListPage.tsx
export default function VmListPage() {
  const { data: vms, isLoading } = useVms()
  const selectedVmId = useUiStore((s) => s.selectedVmId)

  if (isLoading) return <LoadingSpinner />

  return (
    <div>
      <VmList vms={vms} selectedVmId={selectedVmId} />
    </div>
  )
}
```

### Domain Components

**Purpose**: Feature-specific, composable components

**Pattern**:
```typescript
// src/components/vm/VmCard.tsx
interface VmCardProps {
  vm: VM
  onSelect?: (vmId: string) => void
}

export function VmCard({ vm, onSelect }: VmCardProps) {
  const { mutate: startVm } = useStartVm()

  return (
    <Card onClick={() => onSelect?.(vm.id)}>
      <CardHeader>
        <CardTitle>{vm.name}</CardTitle>
        <VmStateBadge state={vm.state} />
      </CardHeader>
      <CardContent>
        <p>{vm.cpuCount} CPUs, {vm.memoryMb}MB RAM</p>
      </CardContent>
      <CardFooter>
        <Button onClick={() => startVm(vm.id)}>Start</Button>
      </CardFooter>
    </Card>
  )
}
```

## Rationale

### Why This Structure?

1. **Separation of Concerns**: Clear boundaries between layers
2. **Scalability**: Easy to add new features without reorganizing
3. **React Best Practices**: Follows community conventions
4. **shadcn/ui Integration**: `components/ui/` is the standard pattern
5. **Type Safety**: Mirrored types ensure IPC contract compliance
6. **Testability**: Each layer can be tested independently
7. **Navigation**: Developers can find code intuitively

### Why Domain-Based Components?

Alternative: Organize by type (`containers/`, `presentational/`)

**Rejected because**:
- Harder to navigate as project grows
- Domain-based is more intuitive for feature work
- Modern React trend (Next.js app router uses this)

## Consequences

### Positive

1. **Clear Organization**: New developers can navigate easily
2. **Scalable**: Structure supports growth to 100+ components
3. **Maintainable**: Changes are localized to specific domains
4. **Best Practices**: Aligns with React and Rust conventions
5. **Type Safety**: TypeScript types mirror Rust models exactly

### Negative

1. **More Folders**: Deeper nesting than flat structure
2. **Initial Setup**: Requires more upfront organization

### Trade-offs Accepted

- **Simplicity vs Organization**: Chose organization for long-term maintainability
- **Flat vs Nested**: Nested structure scales better for large projects

## Migration Notes

As features are added:

1. **New Domain**: Create folder under `components/`
2. **New Command**: Add to appropriate file in `commands/`
3. **New Service**: Add to `services/`
4. **New Type**: Add to `types/` (both Rust and TypeScript)

## Implementation Checklist

### Week 1 Setup
- [x] Define structure (this ADR)
- [ ] Initialize Tauri project with React template
- [ ] Set up path aliases in tsconfig.json
- [ ] Initialize shadcn/ui with `npx shadcn-ui@latest init`
- [ ] Create folder structure
- [ ] Set up Tailwind CSS

### Week 2 Implementation
- [ ] Implement basic backend structure (commands, services, models)
- [ ] Implement basic frontend structure (pages, components, hooks)
- [ ] Add first shadcn/ui components (Button, Card, Badge)
- [ ] Create first query hook (useVms)
- [ ] Create first page (VmListPage)

## References

- [React Project Structure Best Practices](https://react.dev/learn/thinking-in-react)
- [shadcn/ui Documentation](https://ui.shadcn.com)
- [Tauri Project Structure](https://tauri.app/v1/guides/features/command)

## Review and Approval

- **Architecture Agent**: Approved (2025-12-07)
- **Backend Agent**: To implement backend structure
- **Frontend Agent**: To implement frontend structure
- **DevOps Agent**: Acknowledged (affects CI/CD setup)

## Related Decisions

- ADR-001: Frontend Technology Stack (React + shadcn/ui)
- ADR-002: State Management (TanStack Query + Zustand)
