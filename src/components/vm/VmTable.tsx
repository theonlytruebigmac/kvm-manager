import { useState, useEffect, useRef, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { cn } from '@/lib/utils'
import { Search, Play, Square, Pause, Monitor, Copy, Trash2, Edit3, ArrowRightLeft, Moon } from 'lucide-react'
import { toast } from 'sonner'
import type { VM } from '@/lib/types'

interface VmTableProps {
  vms: VM[]
  selectedIds: string[]
  onSelectionChange: (selectedIds: string[]) => void
  onVmDoubleClick: (vmId: string) => void
  onOpenRename?: (vmId: string) => void
  onOpenDelete?: (vmId: string) => void
  onOpenMigrate?: (vmId: string) => void
}

interface VmRowProps {
  vm: VM
  isSelected: boolean
  onToggleSelect: () => void
  onDoubleClick: () => void
  onOpenRename?: () => void
  onOpenDelete?: () => void
  onOpenMigrate?: () => void
  isFocused: boolean
}

function VmRow({ vm, isSelected, onToggleSelect, onDoubleClick, onOpenRename, onOpenDelete, onOpenMigrate, isFocused }: VmRowProps) {
  const queryClient = useQueryClient()

  // Poll stats for running VMs
  const { data: vmStats } = useQuery({
    queryKey: ['vm-stats', vm.id],
    queryFn: () => api.getVmStats(vm.id),
    enabled: vm.state === 'running',
    refetchInterval: 2000,
  })

  // VM action mutations
  const startMutation = useMutation({
    mutationFn: () => api.startVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} started`)
    },
    onError: (error) => toast.error(`Failed to start: ${error}`),
  })

  const stopMutation = useMutation({
    mutationFn: () => api.forceStopVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} stopped`)
    },
    onError: (error) => toast.error(`Failed to stop: ${error}`),
  })

  const pauseMutation = useMutation({
    mutationFn: () => api.pauseVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} paused`)
    },
    onError: (error) => toast.error(`Failed to pause: ${error}`),
  })

  const resumeMutation = useMutation({
    mutationFn: () => api.resumeVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} resumed`)
    },
    onError: (error) => toast.error(`Failed to resume: ${error}`),
  })

  const hibernateMutation = useMutation({
    mutationFn: () => api.hibernateVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} hibernated`)
    },
    onError: (error) => toast.error(`Failed to hibernate: ${error}`),
  })

  const openConsoleMutation = useMutation({
    mutationFn: () => api.openConsoleWindow(vm.id, vm.name),
    onError: (error) => toast.error(`Failed to open console: ${error}`),
  })

  const cpuUsage = vmStats ? Math.round(vmStats.cpuUsagePercent) : null
  const memoryUsed = vmStats ? Math.round(vmStats.memoryUsedMb) : null
  const memoryTotal = vm.memoryMb

  const getStateBadgeVariant = () => {
    switch (vm.state) {
      case 'running':
        return 'default'
      case 'stopped':
        return 'secondary'
      case 'paused':
        return 'outline'
      default:
        return 'secondary'
    }
  }

  const getStateBadgeColor = () => {
    switch (vm.state) {
      case 'running':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 border-green-200 dark:border-green-800'
      case 'paused':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 border-yellow-200 dark:border-yellow-800'
      case 'stopped':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200 border-gray-200 dark:border-gray-800'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200 border-gray-200 dark:border-gray-800'
    }
  }

  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation()
  }

  const getNetworkAddress = () => {
    if (!vm.networkInterfaces || vm.networkInterfaces.length === 0) {
      return '-'
    }
    const firstInterface = vm.networkInterfaces[0]
    return firstInterface.ipAddress || '-'
  }

  const isRunning = vm.state === 'running'
  const isPaused = vm.state === 'paused'
  const isStopped = vm.state === 'stopped'

  const rowContent = (
    <tr
      className={cn(
        'h-8 border-b border-border/50 transition-colors cursor-pointer',
        'hover:bg-accent/50',
        isFocused && 'bg-accent',
        isSelected && 'bg-primary/5'
      )}
      onDoubleClick={onDoubleClick}
    >
      {/* Checkbox */}
      <td className="w-12 px-4 text-center" onClick={handleCheckboxClick}>
        <Checkbox
          checked={isSelected}
          onCheckedChange={onToggleSelect}
          className="data-[state=checked]:bg-primary"
        />
      </td>

      {/* Name */}
      <td className="px-4 py-1">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm truncate max-w-xs">{vm.name}</span>
          {vm.tags && vm.tags.length > 0 && (
            <div className="flex gap-1">
              {vm.tags.slice(0, 1).map(tag => (
                <Badge key={tag} variant="outline" className="text-xs px-1.5 py-0 h-4">
                  {tag}
                </Badge>
              ))}
              {vm.tags.length > 1 && (
                <Badge variant="outline" className="text-xs px-1.5 py-0 h-4">
                  +{vm.tags.length - 1}
                </Badge>
              )}
            </div>
          )}
        </div>
      </td>

      {/* State */}
      <td className="px-4 py-1 w-32">
        <Badge
          variant={getStateBadgeVariant()}
          className={cn('text-xs border', getStateBadgeColor())}
        >
          {vm.state.toUpperCase()}
        </Badge>
      </td>

      {/* CPU */}
      <td className="px-4 py-1 w-24 text-sm text-muted-foreground">
        {vm.state === 'running' && cpuUsage !== null ? (
          <span className={cn(
            cpuUsage > 80 && 'text-red-600 dark:text-red-400 font-medium',
            cpuUsage > 60 && cpuUsage <= 80 && 'text-yellow-600 dark:text-yellow-400'
          )}>
            {cpuUsage}%
          </span>
        ) : (
          <span className="text-muted-foreground/50">-</span>
        )}
      </td>

      {/* Memory */}
      <td className="px-4 py-1 w-32 text-sm text-muted-foreground">
        {vm.state === 'running' && memoryUsed !== null ? (
          <span>
            {(memoryUsed / 1024).toFixed(1)} / {(memoryTotal / 1024).toFixed(1)} GB
          </span>
        ) : (
          <span>{(memoryTotal / 1024).toFixed(1)} GB</span>
        )}
      </td>

      {/* Disk */}
      <td className="px-4 py-1 w-24 text-sm text-muted-foreground">
        {vm.diskSizeGb} GB
      </td>

      {/* Network */}
      <td className="px-4 py-1 w-32 text-sm text-muted-foreground font-mono text-xs">
        {getNetworkAddress()}
      </td>
    </tr>
  )

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        {rowContent}
      </ContextMenuTrigger>
      <ContextMenuContent className="w-48">
        {/* Power Actions */}
        {isStopped && (
          <ContextMenuItem onClick={() => startMutation.mutate()}>
            <Play className="mr-2 h-4 w-4 text-green-500" />
            Start
          </ContextMenuItem>
        )}
        {isPaused && (
          <ContextMenuItem onClick={() => resumeMutation.mutate()}>
            <Play className="mr-2 h-4 w-4 text-green-500" />
            Resume
          </ContextMenuItem>
        )}
        {isRunning && (
          <>
            <ContextMenuItem onClick={() => pauseMutation.mutate()}>
              <Pause className="mr-2 h-4 w-4 text-yellow-500" />
              Pause
            </ContextMenuItem>
            <ContextMenuItem onClick={() => hibernateMutation.mutate()}>
              <Moon className="mr-2 h-4 w-4 text-blue-500" />
              Hibernate
            </ContextMenuItem>
            <ContextMenuItem onClick={() => openConsoleMutation.mutate()}>
              <Monitor className="mr-2 h-4 w-4" />
              Open Console
            </ContextMenuItem>
          </>
        )}
        {(isRunning || isPaused) && (
          <ContextMenuItem onClick={() => stopMutation.mutate()} className="text-red-500">
            <Square className="mr-2 h-4 w-4" />
            Force Stop
          </ContextMenuItem>
        )}

        <ContextMenuSeparator />

        {/* Management Actions */}
        <ContextMenuItem onClick={onDoubleClick}>
          <Monitor className="mr-2 h-4 w-4" />
          View Details
        </ContextMenuItem>
        {onOpenRename && (
          <ContextMenuItem onClick={onOpenRename}>
            <Edit3 className="mr-2 h-4 w-4" />
            Rename
          </ContextMenuItem>
        )}
        <ContextMenuItem onClick={async () => {
          try {
            await api.cloneVm(vm.id, `${vm.name}-clone`)
            queryClient.invalidateQueries({ queryKey: ['vms'] })
            toast.success('VM cloned successfully')
          } catch (error) {
            toast.error(`Failed to clone: ${error}`)
          }
        }}>
          <Copy className="mr-2 h-4 w-4" />
          Clone
        </ContextMenuItem>
        {onOpenMigrate && (
          <ContextMenuItem onClick={onOpenMigrate}>
            <ArrowRightLeft className="mr-2 h-4 w-4" />
            Migrate
          </ContextMenuItem>
        )}

        <ContextMenuSeparator />

        {onOpenDelete && (
          <ContextMenuItem onClick={onOpenDelete} className="text-red-500">
            <Trash2 className="mr-2 h-4 w-4" />
            Delete
          </ContextMenuItem>
        )}
      </ContextMenuContent>
    </ContextMenu>
  )
}

export function VmTable({ vms, selectedIds, onSelectionChange, onVmDoubleClick, onOpenRename, onOpenDelete, onOpenMigrate }: VmTableProps) {
  const [focusedIndex, setFocusedIndex] = useState<number>(0)
  const tableRef = useRef<HTMLTableElement>(null)

  const isAllSelected = vms.length > 0 && selectedIds.length === vms.length
  const isSomeSelected = selectedIds.length > 0 && selectedIds.length < vms.length

  const handleSelectAll = () => {
    if (isAllSelected) {
      onSelectionChange([])
    } else {
      onSelectionChange(vms.map(vm => vm.id))
    }
  }

  const handleToggleSelect = (vmId: string) => {
    if (selectedIds.includes(vmId)) {
      onSelectionChange(selectedIds.filter(id => id !== vmId))
    } else {
      onSelectionChange([...selectedIds, vmId])
    }
  }

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!vms.length) return

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setFocusedIndex(prev => Math.min(prev + 1, vms.length - 1))
        break
      case 'ArrowUp':
        e.preventDefault()
        setFocusedIndex(prev => Math.max(prev - 1, 0))
        break
      case ' ':
        e.preventDefault()
        if (vms[focusedIndex]) {
          handleToggleSelect(vms[focusedIndex].id)
        }
        break
      case 'Enter':
        e.preventDefault()
        if (vms[focusedIndex]) {
          onVmDoubleClick(vms[focusedIndex].id)
        }
        break
      case 'a':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          handleSelectAll()
        }
        break
    }
  }, [vms, focusedIndex, selectedIds])

  useEffect(() => {
    const table = tableRef.current
    if (!table) return

    table.addEventListener('keydown', handleKeyDown as any)
    return () => {
      table.removeEventListener('keydown', handleKeyDown as any)
    }
  }, [handleKeyDown])

  // Ensure focused index is valid
  useEffect(() => {
    if (focusedIndex >= vms.length && vms.length > 0) {
      setFocusedIndex(vms.length - 1)
    }
  }, [vms.length, focusedIndex])

  if (vms.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground gap-2">
        <Search className="w-8 h-8 opacity-50" />
        <p>No VMs match your current filters</p>
        <p className="text-sm">Try adjusting your search or filter criteria</p>
      </div>
    )
  }

  return (
    <div className="w-full border border-border rounded-lg overflow-hidden">
      <table
        ref={tableRef}
        className="w-full table-fixed"
        tabIndex={0}
        style={{ outline: 'none' }}
      >
        <thead className="bg-muted/50 border-b border-border">
          <tr className="h-8">
            {/* Select All Checkbox */}
            <th className="w-12 px-4 text-center">
              <Checkbox
                checked={isAllSelected}
                ref={(el) => {
                  if (el) {
                    (el as any).indeterminate = isSomeSelected
                  }
                }}
                onCheckedChange={handleSelectAll}
                className="data-[state=checked]:bg-primary"
              />
            </th>
            <th className="px-4 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">
              Name
            </th>
            <th className="px-4 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider w-32">
              State
            </th>
            <th className="px-4 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider w-24">
              CPU
            </th>
            <th className="px-4 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider w-32">
              Memory
            </th>
            <th className="px-4 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider w-24">
              Disk
            </th>
            <th className="px-4 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider w-32">
              Network
            </th>
          </tr>
        </thead>
        <tbody className="bg-background">
          {vms.map((vm, index) => (
            <VmRow
              key={vm.id}
              vm={vm}
              isSelected={selectedIds.includes(vm.id)}
              onToggleSelect={() => handleToggleSelect(vm.id)}
              onDoubleClick={() => onVmDoubleClick(vm.id)}
              onOpenRename={onOpenRename ? () => onOpenRename(vm.id) : undefined}
              onOpenDelete={onOpenDelete ? () => onOpenDelete(vm.id) : undefined}
              onOpenMigrate={onOpenMigrate ? () => onOpenMigrate(vm.id) : undefined}
              isFocused={index === focusedIndex}
            />
          ))}
        </tbody>
      </table>
    </div>
  )
}
