# ADR-002: State Management Patterns

Date: 2025-12-07
Status: Accepted
Decision Makers: Architecture Agent

## Context

KVM Manager needs to manage two distinct types of state:

1. **Server State**: Data from the backend (VMs, networks, storage, host info)
   - Originates from libvirt via Rust backend
   - Needs synchronization with backend
   - Updates in real-time via Tauri events
   - Requires caching and invalidation strategies

2. **Local UI State**: Frontend-only state (selected VM, view modes, filters, UI preferences)
   - Lives entirely in the browser
   - No backend involvement
   - Needs to be reactive and performant

### Requirements

1. **Real-time Synchronization**: UI must reflect backend state changes immediately
2. **Efficient Caching**: Avoid redundant backend calls
3. **Event-Driven Updates**: Handle backend events to update cached data
4. **Optimistic Updates**: Show immediate feedback for user actions
5. **Error Handling**: Gracefully handle failed operations and rollback
6. **Type Safety**: Strong typing for all state
7. **Developer Experience**: Easy to use, minimal boilerplate

### Constraints

- React 18+ chosen as frontend framework (from ADR-001)
- Tauri IPC for backend communication
- Must support real-time event updates
- Need to handle 100+ VMs without performance degradation

## Decision

### Server State: TanStack Query (React Query)

Use **TanStack Query v5** for all server state management.

### Local UI State: Zustand

Use **Zustand** for global UI state (selected items, filters, view modes, preferences).

## Rationale

### Why TanStack Query for Server State?

1. **Industry Standard**: De facto solution for server state in React
2. **Built-in Caching**: Intelligent caching with automatic garbage collection
3. **Real-time Support**: Easy integration with Tauri events
4. **Optimistic Updates**: First-class support for optimistic UI updates
5. **Automatic Refetching**: Configurable refetch strategies (on focus, interval, etc.)
6. **DevTools**: Excellent debugging with React Query DevTools
7. **Type Safety**: Excellent TypeScript support
8. **Error Handling**: Built-in retry logic and error boundaries

### Why Zustand for Local State?

1. **Simplicity**: Minimal boilerplate, no providers needed
2. **Performance**: No unnecessary re-renders
3. **Developer Experience**: Clean API, easy to use
4. **Bundle Size**: Tiny (~3KB)
5. **TypeScript**: Excellent type inference
6. **Middleware**: Built-in persist, devtools, immer support
7. **No Context Hell**: Direct store access without prop drilling

### Alternatives Considered

**For Server State:**
- **Redux Toolkit + RTK Query**: Too heavy, more boilerplate than needed
- **SWR**: Good but less feature-rich than TanStack Query
- **Apollo Client**: Overkill (designed for GraphQL)

**For Local State:**
- **Jotai**: Excellent but atomic approach not needed for our use case
- **Redux Toolkit**: Too heavy for simple UI state
- **React Context**: Performance concerns with frequent updates
- **Valtio**: Proxy-based, but Zustand's API is cleaner

## Implementation Details

### TanStack Query Setup

**Query Client Configuration**:
```typescript
// src/lib/query-client.ts
import { QueryClient } from '@tanstack/react-query'

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 10_000,        // Data fresh for 10 seconds
      gcTime: 5 * 60 * 1000,    // Cache for 5 minutes (formerly cacheTime)
      retry: 1,                  // Retry failed queries once
      refetchOnWindowFocus: true,
      refetchOnReconnect: true,
    },
    mutations: {
      retry: 0,                  // Don't retry mutations
    },
  },
})
```

**App Wrapper**:
```typescript
// src/App.tsx
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { queryClient } from './lib/query-client'

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <MainContent />
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  )
}
```

### Query Key Structure

**Standard Pattern**:
```typescript
// Query keys are hierarchical arrays
// This enables targeted invalidation

// All VMs
['vms']

// Single VM
['vm', vmId]

// VM stats
['vm', vmId, 'stats']

// Host info
['host-info']

// Connection status
['connection-status']

// Networks (Phase 2)
['networks']
['network', networkId]

// Storage (Phase 2)
['storage-pools']
['storage-pool', poolId]
['storage-pool', poolId, 'volumes']
```

### Custom Hooks for Queries

**VM Queries**:
```typescript
// src/hooks/useVms.ts
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api'
import type { VM } from '@/types'

export function useVms() {
  return useQuery({
    queryKey: ['vms'],
    queryFn: () => invoke<VM[]>('getVms'),
    staleTime: 5_000,  // VMs list stale after 5 seconds
  })
}

export function useVm(vmId: string) {
  return useQuery({
    queryKey: ['vm', vmId],
    queryFn: () => invoke<VM>('getVm', { vmId }),
    enabled: !!vmId,  // Don't query if no vmId
  })
}

export function useVmStats(vmId: string) {
  return useQuery({
    queryKey: ['vm', vmId, 'stats'],
    queryFn: () => invoke<VmStats>('getVmStats', { vmId }),
    refetchInterval: 2_000,  // Poll every 2 seconds
    enabled: !!vmId,
  })
}
```

**Host Queries**:
```typescript
// src/hooks/useHost.ts
export function useHostInfo() {
  return useQuery({
    queryKey: ['host-info'],
    queryFn: () => invoke<HostInfo>('getHostInfo'),
    staleTime: 30_000,  // Host info changes rarely
  })
}

export function useConnectionStatus() {
  return useQuery({
    queryKey: ['connection-status'],
    queryFn: () => invoke<ConnectionStatus>('getConnectionStatus'),
    refetchInterval: 5_000,  // Check connection every 5 seconds
  })
}
```

### Mutations

**VM Operations**:
```typescript
// src/hooks/useVmMutations.ts
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api'
import { toast } from 'sonner'

export function useStartVm() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (vmId: string) => invoke('startVm', { vmId }),
    onMutate: async (vmId) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['vm', vmId] })

      // Snapshot previous value
      const previous = queryClient.getQueryData(['vm', vmId])

      // Optimistically update
      queryClient.setQueryData(['vm', vmId], (old: VM) => ({
        ...old,
        state: 'running'
      }))

      return { previous, vmId }
    },
    onError: (err, vmId, context) => {
      // Rollback on error
      if (context?.previous) {
        queryClient.setQueryData(['vm', context.vmId], context.previous)
      }
      toast.error(`Failed to start VM: ${err}`)
    },
    onSuccess: (_, vmId) => {
      toast.success('VM started successfully')
      queryClient.invalidateQueries({ queryKey: ['vms'] })
    },
  })
}

export function useCreateVm() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (config: VmConfig) => invoke<string>('createVm', { config }),
    onSuccess: (vmId, config) => {
      toast.success(`VM "${config.name}" created successfully`)
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['host-info'] })
    },
    onError: (err) => {
      toast.error(`Failed to create VM: ${err}`)
    },
  })
}

export function useDeleteVm() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ vmId, options }: { vmId: string, options: DeleteVmOptions }) =>
      invoke('deleteVm', { vmId, options }),
    onSuccess: (_, { vmId }) => {
      toast.success('VM deleted')
      queryClient.removeQueries({ queryKey: ['vm', vmId] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['host-info'] })
    },
  })
}
```

### Event Integration

**Event Listener Hook**:
```typescript
// src/hooks/useVmEvents.ts
import { useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { listen } from '@tauri-apps/api/event'
import type {
  VmStateChangedPayload,
  VmCreatedPayload,
  VmDeletedPayload,
  VmStatsPayload
} from '@/types/events'

export function useVmEvents() {
  const queryClient = useQueryClient()

  useEffect(() => {
    const listeners: (() => void)[] = []

    // VM state changed
    listen<VmStateChangedPayload>('vm-state-changed', (event) => {
      const { vmId, newState } = event.payload

      // Update specific VM in cache
      queryClient.setQueryData(['vm', vmId], (old: VM | undefined) =>
        old ? { ...old, state: newState } : undefined
      )

      // Invalidate list (will refetch on next access)
      queryClient.invalidateQueries({ queryKey: ['vms'] })
    }).then((unlisten) => listeners.push(unlisten))

    // VM created
    listen<VmCreatedPayload>('vm-created', () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['host-info'] })
    }).then((unlisten) => listeners.push(unlisten))

    // VM deleted
    listen<VmDeletedPayload>('vm-deleted', (event) => {
      const { vmId } = event.payload
      queryClient.removeQueries({ queryKey: ['vm', vmId] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['host-info'] })
    }).then((unlisten) => listeners.push(unlisten))

    // VM stats updated (high frequency)
    listen<VmStatsPayload>('vm-stats-updated', (event) => {
      const { vmId, ...stats } = event.payload
      queryClient.setQueryData(['vm', vmId, 'stats'], stats)
    }).then((unlisten) => listeners.push(unlisten))

    // Cleanup
    return () => {
      listeners.forEach((unlisten) => unlisten())
    }
  }, [queryClient])
}

// Use in App.tsx
function App() {
  useVmEvents()  // Set up global event listeners
  // ... rest of app
}
```

### Zustand Store

**UI Store**:
```typescript
// src/stores/ui-store.ts
import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface UiState {
  // Selected items
  selectedVmId: string | null
  setSelectedVmId: (id: string | null) => void

  // View modes
  vmListView: 'grid' | 'list'
  setVmListView: (view: 'grid' | 'list') => void

  // Filters
  vmFilter: string
  setVmFilter: (filter: string) => void

  vmStateFilter: 'all' | 'running' | 'stopped'
  setVmStateFilter: (state: 'all' | 'running' | 'stopped') => void

  // UI preferences (persisted)
  sidebarCollapsed: boolean
  toggleSidebar: () => void

  theme: 'light' | 'dark' | 'system'
  setTheme: (theme: 'light' | 'dark' | 'system') => void
}

export const useUiStore = create<UiState>()(
  persist(
    (set) => ({
      // Selected items
      selectedVmId: null,
      setSelectedVmId: (id) => set({ selectedVmId: id }),

      // View modes
      vmListView: 'grid',
      setVmListView: (view) => set({ vmListView: view }),

      // Filters (not persisted)
      vmFilter: '',
      setVmFilter: (filter) => set({ vmFilter: filter }),

      vmStateFilter: 'all',
      setVmStateFilter: (state) => set({ vmStateFilter: state }),

      // UI preferences
      sidebarCollapsed: false,
      toggleSidebar: () => set((state) => ({
        sidebarCollapsed: !state.sidebarCollapsed
      })),

      theme: 'system',
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: 'kvm-manager-ui',
      partialize: (state) => ({
        // Only persist these fields
        sidebarCollapsed: state.sidebarCollapsed,
        theme: state.theme,
        vmListView: state.vmListView,
      }),
    }
  )
)
```

**Usage in Components**:
```typescript
// src/components/VmList.tsx
import { useUiStore } from '@/stores/ui-store'
import { useVms } from '@/hooks/useVms'

function VmList() {
  // Server state
  const { data: vms, isLoading } = useVms()

  // Local UI state
  const vmListView = useUiStore((state) => state.vmListView)
  const vmFilter = useUiStore((state) => state.vmFilter)
  const vmStateFilter = useUiStore((state) => state.vmStateFilter)

  // Derived state
  const filteredVms = vms?.filter((vm) => {
    if (vmStateFilter !== 'all' && vm.state !== vmStateFilter) return false
    if (vmFilter && !vm.name.toLowerCase().includes(vmFilter.toLowerCase())) return false
    return true
  })

  return (
    <div>
      {vmListView === 'grid' ? (
        <GridView vms={filteredVms} />
      ) : (
        <ListView vms={filteredVms} />
      )}
    </div>
  )
}
```

## Cache Invalidation Strategy

### Automatic Invalidation

1. **On Mutation Success**: Invalidate affected queries
   - Create VM → invalidate `['vms']` and `['host-info']`
   - Delete VM → remove `['vm', vmId]`, invalidate `['vms']`
   - Update VM → invalidate `['vm', vmId]` and `['vms']`

2. **On Backend Events**: Update cache directly or invalidate
   - `vm-state-changed` → update `['vm', vmId]` directly
   - `vm-created` → invalidate `['vms']`
   - `vm-deleted` → remove `['vm', vmId]`

3. **On Reconnect**: Invalidate all queries when connection restored

### Manual Invalidation

Provide a "Refresh" button that invalidates all queries:
```typescript
function RefreshButton() {
  const queryClient = useQueryClient()

  const handleRefresh = () => {
    queryClient.invalidateQueries({ queryKey: ['vms'] })
    queryClient.invalidateQueries({ queryKey: ['host-info'] })
    toast.success('Refreshed')
  }

  return <Button onClick={handleRefresh}>Refresh</Button>
}
```

## Performance Optimizations

1. **Selective Subscription**: Only subscribe to specific store slices in Zustand
2. **Query Deduplication**: TanStack Query automatically dedupes concurrent requests
3. **Background Refetching**: Fresh data without loading states
4. **Stale-While-Revalidate**: Show cached data while fetching fresh data
5. **Garbage Collection**: Old cache entries auto-removed after gcTime

## Testing Strategy

### Testing Queries
```typescript
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClientProvider } from '@tanstack/react-query'
import { queryClient } from '@/lib/query-client'
import { useVms } from '@/hooks/useVms'

test('useVms fetches VM list', async () => {
  const wrapper = ({ children }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  )

  const { result } = renderHook(() => useVms(), { wrapper })

  await waitFor(() => expect(result.current.isSuccess).toBe(true))
  expect(result.current.data).toHaveLength(3)
})
```

### Testing Zustand Store
```typescript
import { renderHook, act } from '@testing-library/react'
import { useUiStore } from '@/stores/ui-store'

test('can select VM', () => {
  const { result } = renderHook(() => useUiStore())

  act(() => {
    result.current.setSelectedVmId('vm-123')
  })

  expect(result.current.selectedVmId).toBe('vm-123')
})
```

## Consequences

### Positive

1. **Separation of Concerns**: Clear distinction between server and UI state
2. **Type Safety**: Full TypeScript support across all state
3. **Developer Experience**: Minimal boilerplate, easy to understand
4. **Performance**: Efficient caching and minimal re-renders
5. **Real-time**: Seamless integration with Tauri events
6. **Debugging**: Excellent DevTools for both libraries
7. **Optimistic Updates**: Great UX with instant feedback
8. **Error Handling**: Built-in retry and error recovery
9. **Code Reuse**: Custom hooks promote DRY principles
10. **Future-Proof**: Both libraries are actively maintained and widely adopted

### Negative

1. **Two Libraries**: Need to learn both TanStack Query and Zustand (mitigated: both are simple)
2. **Bundle Size**: ~40KB combined (acceptable for the value)
3. **Mental Model**: Developers need to understand when to use each
4. **Event Complexity**: Need to coordinate events with cache updates

### Trade-offs Accepted

- **Complexity vs Power**: Two libraries add some complexity, but provide the exact features we need
- **Bundle Size vs DX**: The bundle size increase is worth the developer experience and functionality

## Migration Notes

If state management needs to evolve:

1. **Adding Redux**: Can coexist with TanStack Query + Zustand if complex client-side logic emerges
2. **Switching Local State**: Zustand can be replaced with Jotai without affecting server state
3. **Server State Alternatives**: TanStack Query can be swapped (but unlikely to be needed)

## References

- [TanStack Query Docs](https://tanstack.com/query/latest)
- [Zustand Documentation](https://github.com/pmndrs/zustand)
- [React Query + Tauri Events Pattern](https://github.com/tauri-apps/tauri/discussions/3785)

## Review and Approval

- **Architecture Agent**: Approved (2025-12-07)
- **Frontend Agent**: To implement
- **Backend Agent**: Acknowledged (affects event emission)

## Related Decisions

- ADR-001: Frontend Technology Stack (chose React)
- event-schema.md: Defines events that update React Query cache
- tauri-commands.md: Defines commands that queries invoke
