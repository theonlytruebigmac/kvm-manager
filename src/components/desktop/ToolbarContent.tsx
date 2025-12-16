import { Plus, Play, Square, Pause, Monitor, Settings, Upload, Search, Circle, LayoutGrid, List, ArrowUpDown, ArrowUp, ArrowDown, Server } from 'lucide-react'
import { Toolbar, ToolbarButton, ToolbarSeparator, ToolbarSpacer } from './Toolbar'
import { useToolbarStore } from '@/hooks/useToolbarActions'
import type { VmSortField } from '@/hooks/useToolbarActions'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { ThemeToggle } from '@/components/ui/theme-toggle'
import { cn } from '@/lib/utils'
import { useState } from 'react'
import { ConnectionManager } from '@/components/connections/ConnectionManager'

interface ToolbarContentProps {
  onOpenCommandPalette?: () => void
}

// Filter button component
function FilterButton({
  active,
  onClick,
  tooltip,
  dotColor,
  children,
}: {
  active: boolean
  onClick: () => void
  tooltip: string
  dotColor?: string
  children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      title={tooltip}
      className={cn(
        'px-2 py-0.5 text-xs rounded transition-colors flex items-center gap-1.5',
        active
          ? 'bg-background text-foreground shadow-sm'
          : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
      )}
    >
      {dotColor && <span className={cn('w-1.5 h-1.5 rounded-full', dotColor)} />}
      {children}
    </button>
  )
}

// View toggle component
function ViewToggle() {
  const { viewMode, setViewMode } = useToolbarStore()

  return (
    <div className="flex items-center bg-muted/50 rounded p-0.5">
      <button
        onClick={() => setViewMode('table')}
        title="Table view"
        className={cn(
          'p-1 rounded transition-colors',
          viewMode === 'table'
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
        )}
      >
        <List className="w-3.5 h-3.5" />
      </button>
      <button
        onClick={() => setViewMode('cards')}
        title="Card view"
        className={cn(
          'p-1 rounded transition-colors',
          viewMode === 'cards'
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
        )}
      >
        <LayoutGrid className="w-3.5 h-3.5" />
      </button>
    </div>
  )
}

// Sort dropdown component
function SortDropdown() {
  const { sortField, sortDirection, setSortField, toggleSortDirection } = useToolbarStore()

  const sortOptions: { value: VmSortField; label: string }[] = [
    { value: 'name', label: 'Name' },
    { value: 'state', label: 'State' },
    { value: 'cpu', label: 'CPU Usage' },
    { value: 'memory', label: 'Memory Usage' },
    { value: 'disk', label: 'Disk Usage' },
  ]

  const currentLabel = sortOptions.find(o => o.value === sortField)?.label || 'Name'

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="h-7 gap-1.5 px-2 text-xs">
          <ArrowUpDown className="w-3.5 h-3.5" />
          <span className="hidden sm:inline">Sort: {currentLabel}</span>
          {sortDirection === 'asc' ? (
            <ArrowUp className="w-3 h-3 text-muted-foreground" />
          ) : (
            <ArrowDown className="w-3 h-3 text-muted-foreground" />
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-40">
        {sortOptions.map((option) => (
          <DropdownMenuItem
            key={option.value}
            onClick={() => {
              if (sortField === option.value) {
                toggleSortDirection()
              } else {
                setSortField(option.value)
              }
            }}
            className="flex items-center justify-between"
          >
            <span>{option.label}</span>
            {sortField === option.value && (
              sortDirection === 'asc' ? (
                <ArrowUp className="w-3.5 h-3.5" />
              ) : (
                <ArrowDown className="w-3.5 h-3.5" />
              )
            )}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

export function ToolbarContent({ onOpenCommandPalette }: ToolbarContentProps) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { selectedVmIds, focusedVmId, setShowCreateVm, setShowImportVm, stateFilter, setStateFilter } = useToolbarStore()
  const [showConnectionManager, setShowConnectionManager] = useState(false)

  // Get active connection
  const { data: activeConnection } = useQuery({
    queryKey: ['active-connection'],
    queryFn: api.getActiveConnection,
  })

  // Check if we have any selection
  const hasSelection = selectedVmIds.length > 0
  const isBatchMode = selectedVmIds.length > 1

  // Get VMs data to check focused VM state
  const { data: vms } = useQuery({
    queryKey: ['vms'],
    queryFn: api.getVms,
  })

  // Get selected VMs
  const selectedVms = vms?.filter(vm => selectedVmIds.includes(vm.id)) || []

  // For single selection, use the first selected VM (or focused if available)
  const activeVm = selectedVms.length === 1
    ? selectedVms[0]
    : vms?.find(vm => vm.id === focusedVmId)

  // Check what actions are available
  const anyCanStart = selectedVms.some(vm => vm.state === 'stopped')
  const anyCanStop = selectedVms.some(vm => vm.state === 'running')

  // Button states depend on selection
  const canStart = hasSelection ? anyCanStart : false
  const canStop = hasSelection ? anyCanStop : false
  const canPause = hasSelection && !isBatchMode ? (activeVm?.state === 'running') : false
  const canConsole = hasSelection && !isBatchMode && activeVm?.state === 'running'

  // Single VM mutations
  const startMutation = useMutation({
    mutationFn: (vmId: string) => api.startVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM started successfully')
    },
    onError: (error) => toast.error(`Failed to start VM: ${error}`)
  })

  const stopMutation = useMutation({
    mutationFn: (vmId: string) => api.forceStopVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM stopped successfully')
    },
    onError: (error) => toast.error(`Failed to stop VM: ${error}`)
  })

  const pauseMutation = useMutation({
    mutationFn: (vmId: string) => api.pauseVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM paused successfully')
    },
    onError: (error) => toast.error(`Failed to pause VM: ${error}`)
  })

  // Batch mutations
  const batchStartMutation = useMutation({
    mutationFn: (vmIds: string[]) => api.batchStartVms(vmIds),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      const successCount = data.filter(r => r.success).length
      toast.success(`Started ${successCount} of ${data.length} VMs`)
    },
    onError: (error) => toast.error(`Batch start failed: ${error}`)
  })

  const batchStopMutation = useMutation({
    mutationFn: (vmIds: string[]) => api.batchStopVms(vmIds, true),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      const successCount = data.filter(r => r.success).length
      toast.success(`Stopped ${successCount} of ${data.length} VMs`)
    },
    onError: (error) => toast.error(`Batch stop failed: ${error}`)
  })

  const openConsoleMutation = useMutation({
    mutationFn: ({ vmId, vmName }: { vmId: string; vmName: string }) => api.openConsoleWindow(vmId, vmName),
    onSuccess: () => toast.success('Console window opened'),
    onError: (error) => toast.error(`Failed to open console: ${error}`)
  })

  const handleStart = () => {
    if (isBatchMode) {
      const stoppedIds = selectedVms.filter(vm => vm.state === 'stopped').map(vm => vm.id)
      if (stoppedIds.length > 0) {
        batchStartMutation.mutate(stoppedIds)
      }
    } else if (selectedVmIds.length === 1 && canStart) {
      startMutation.mutate(selectedVmIds[0])
    }
  }

  const handleStop = () => {
    if (isBatchMode) {
      const runningIds = selectedVms.filter(vm => vm.state === 'running').map(vm => vm.id)
      if (runningIds.length > 0) {
        batchStopMutation.mutate(runningIds)
      }
    } else if (selectedVmIds.length === 1 && canStop) {
      stopMutation.mutate(selectedVmIds[0])
    }
  }

  const handlePause = () => {
    if (selectedVmIds.length === 1 && canPause) {
      pauseMutation.mutate(selectedVmIds[0])
    }
  }

  const handleConsole = () => {
    if (selectedVmIds.length === 1 && canConsole && activeVm) {
      openConsoleMutation.mutate({ vmId: selectedVmIds[0], vmName: activeVm.name })
    }
  }

  const isLoading = startMutation.isPending || stopMutation.isPending ||
                    pauseMutation.isPending || batchStartMutation.isPending ||
                    batchStopMutation.isPending

  return (
    <Toolbar>
      {/* Connection Status - clickable to open Connection Manager */}
      <Button
        variant="ghost"
        size="sm"
        className="h-7 gap-1.5 px-2 text-xs font-normal hover:bg-accent"
        onClick={() => setShowConnectionManager(true)}
        title="Click to manage connections"
      >
        <Circle className="h-2 w-2 fill-green-500 text-green-500" />
        <Server className="h-3.5 w-3.5" />
        <span className="max-w-[120px] truncate">
          {activeConnection?.name || 'QEMU/KVM'}
        </span>
      </Button>

      {/* Connection Manager Dialog */}
      <ConnectionManager
        open={showConnectionManager}
        onOpenChange={setShowConnectionManager}
      />

      <ToolbarSeparator />

      {/* VM Actions */}
      <ToolbarButton
        icon={Plus}
        tooltip="Create New Virtual Machine (Ctrl+N)"
        onClick={() => setShowCreateVm(true)}
      >
        New
      </ToolbarButton>

      <ToolbarButton
        icon={Upload}
        tooltip="Import Virtual Machine"
        onClick={() => setShowImportVm(true)}
      >
        Import
      </ToolbarButton>

      <ToolbarSeparator />

      <ToolbarButton
        icon={Play}
        tooltip={isBatchMode ? `Start ${selectedVmIds.length} VMs` : "Start Virtual Machine (Ctrl+P)"}
        disabled={!canStart || isLoading}
        onClick={handleStart}
      >
        Start
      </ToolbarButton>

      <ToolbarButton
        icon={Square}
        tooltip={isBatchMode ? `Stop ${selectedVmIds.length} VMs` : "Stop Virtual Machine (Ctrl+S)"}
        disabled={!canStop || isLoading}
        onClick={handleStop}
      >
        Stop
      </ToolbarButton>

      <ToolbarButton
        icon={Pause}
        tooltip="Pause Virtual Machine"
        disabled={!canPause || isLoading || isBatchMode}
        onClick={handlePause}
      >
        Pause
      </ToolbarButton>

      <ToolbarSeparator />

      <ToolbarButton
        icon={Monitor}
        tooltip="Open Console (Ctrl+O)"
        disabled={!canConsole || isBatchMode}
        onClick={handleConsole}
      >
        Console
      </ToolbarButton>

      <ToolbarSeparator />

      {/* State Filters */}
      <div className="flex items-center gap-0.5 bg-muted/50 rounded-md p-0.5">
        <FilterButton
          active={stateFilter === 'all'}
          onClick={() => setStateFilter('all')}
          tooltip="Show all VMs"
        >
          All
        </FilterButton>
        <FilterButton
          active={stateFilter === 'running'}
          onClick={() => setStateFilter('running')}
          tooltip="Show running VMs"
          dotColor="bg-green-500"
        >
          Running
        </FilterButton>
        <FilterButton
          active={stateFilter === 'stopped'}
          onClick={() => setStateFilter('stopped')}
          tooltip="Show stopped VMs"
          dotColor="bg-gray-400"
        >
          Stopped
        </FilterButton>
        <FilterButton
          active={stateFilter === 'paused'}
          onClick={() => setStateFilter('paused')}
          tooltip="Show paused VMs"
          dotColor="bg-yellow-500"
        >
          Paused
        </FilterButton>
      </div>

      {/* View Toggle */}
      <ViewToggle />

      {/* Sort Dropdown */}
      <SortDropdown />

      {/* Spacer to push settings to the right */}
      <ToolbarSpacer />

      {/* Selection indicator */}
      {isBatchMode && (
        <span className="text-xs text-muted-foreground mr-2">
          {selectedVmIds.length} VMs selected
        </span>
      )}

      {/* Command Palette / Search */}
      <ToolbarButton
        icon={Search}
        tooltip="Command Palette (Ctrl+K)"
        onClick={onOpenCommandPalette}
      />

      {/* Theme Toggle */}
      <ThemeToggle />

      <ToolbarButton
        icon={Settings}
        tooltip="Settings (Ctrl+,)"
        onClick={() => navigate('/settings')}
      >
        Settings
      </ToolbarButton>
    </Toolbar>
  )
}
