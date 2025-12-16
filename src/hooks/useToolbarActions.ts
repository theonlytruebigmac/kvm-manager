import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type VmStateFilter = 'all' | 'running' | 'stopped' | 'paused'
export type VmViewMode = 'table' | 'cards'
export type VmSortField = 'name' | 'state' | 'cpu' | 'memory' | 'disk'
export type SortDirection = 'asc' | 'desc'

interface ToolbarState {
  // Selected VMs
  selectedVmIds: string[]
  focusedVmId: string | null

  // Actions
  setSelectedVmIds: (ids: string[]) => void
  setFocusedVmId: (id: string | null) => void
  clearSelection: () => void

  // Dialogs
  showCreateVm: boolean
  setShowCreateVm: (show: boolean) => void

  showImportVm: boolean
  setShowImportVm: (show: boolean) => void

  // Filters
  stateFilter: VmStateFilter
  setStateFilter: (filter: VmStateFilter) => void

  // View mode
  viewMode: VmViewMode
  setViewMode: (mode: VmViewMode) => void

  // Sorting
  sortField: VmSortField
  sortDirection: SortDirection
  setSortField: (field: VmSortField) => void
  toggleSortDirection: () => void
}

export const useToolbarStore = create<ToolbarState>()(
  persist(
    (set) => ({
      selectedVmIds: [],
      focusedVmId: null,

      setSelectedVmIds: (ids) => set({ selectedVmIds: ids }),
      setFocusedVmId: (id) => set({ focusedVmId: id }),
      clearSelection: () => set({ selectedVmIds: [], focusedVmId: null }),

      showCreateVm: false,
      setShowCreateVm: (show) => set({ showCreateVm: show }),

      showImportVm: false,
      setShowImportVm: (show) => set({ showImportVm: show }),

      stateFilter: 'all',
      setStateFilter: (filter) => set({ stateFilter: filter }),

      viewMode: 'table',
      setViewMode: (mode) => set({ viewMode: mode }),

      sortField: 'name',
      sortDirection: 'asc',
      setSortField: (field) => set({ sortField: field }),
      toggleSortDirection: () => set((state) => ({
        sortDirection: state.sortDirection === 'asc' ? 'desc' : 'asc'
      })),
    }),
    {
      name: 'kvm-manager-toolbar',
      partialize: (state) => ({
        viewMode: state.viewMode,
        stateFilter: state.stateFilter,
        sortField: state.sortField,
        sortDirection: state.sortDirection,
      }),
    }
  )
)
