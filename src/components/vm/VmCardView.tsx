import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Button } from '@/components/ui/button'
import { VmContextMenu } from '@/components/vm/VmContextMenu'
import { IsoDropZone } from '@/components/vm/IsoDropZone'
import { cn } from '@/lib/utils'
import { Monitor, Cpu, MemoryStick, HardDrive, Network, Play, Square, Pause, MonitorPlay } from 'lucide-react'
import { toast } from 'sonner'
import type { VM } from '@/lib/types'

interface VmCardViewProps {
  vms: VM[]
  selectedIds: string[]
  onSelectionChange: (selectedIds: string[]) => void
  onVmDoubleClick: (vmId: string) => void
  onOpenRename?: (vmId: string) => void
  onOpenDelete?: (vmId: string) => void
  onOpenMigrate?: (vmId: string) => void
}

interface VmCardProps {
  vm: VM
  isSelected: boolean
  onToggleSelect: () => void
  onDoubleClick: () => void
  onOpenRename?: () => void
  onOpenDelete?: () => void
  onOpenMigrate?: () => void
}

function VmCard({ vm, isSelected, onToggleSelect, onDoubleClick, onOpenRename, onOpenDelete, onOpenMigrate }: VmCardProps) {
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

  const openConsoleMutation = useMutation({
    mutationFn: () => api.openConsoleWindow(vm.id, vm.name),
    onError: (error) => toast.error(`Failed to open console: ${error}`),
  })

  const cpuUsage = vmStats ? Math.round(vmStats.cpuUsagePercent) : null
  const memoryUsed = vmStats ? Math.round(vmStats.memoryUsedMb) : null

  const isActionPending = startMutation.isPending || stopMutation.isPending || pauseMutation.isPending

  const handleQuickAction = (e: React.MouseEvent, action: () => void) => {
    e.stopPropagation()
    action()
  }

  const getStateColor = () => {
    switch (vm.state) {
      case 'running':
        return 'bg-green-500'
      case 'paused':
        return 'bg-yellow-500'
      case 'stopped':
        return 'bg-gray-400'
      default:
        return 'bg-gray-400'
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

  const getNetworkAddress = () => {
    if (!vm.networkInterfaces || vm.networkInterfaces.length === 0) {
      return null
    }
    return vm.networkInterfaces[0]?.ipAddress || null
  }

  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation()
  }

  const cardContent = (
    <Card
      className={cn(
        'cursor-pointer transition-all hover:shadow-md hover:border-primary/50',
        isSelected && 'ring-2 ring-primary border-primary'
      )}
      onDoubleClick={onDoubleClick}
    >
      <CardContent className="p-4">
        {/* Header with checkbox, name, and state */}
        <div className="flex items-start gap-3 mb-3">
          <div onClick={handleCheckboxClick} className="pt-0.5">
            <Checkbox
              checked={isSelected}
              onCheckedChange={onToggleSelect}
              className="data-[state=checked]:bg-primary"
            />
          </div>

          {/* VM Icon with state indicator */}
          <div className="relative">
            <div className="w-10 h-10 rounded-lg bg-muted flex items-center justify-center">
              <Monitor className="w-5 h-5 text-muted-foreground" />
            </div>
            <div className={cn('absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-background', getStateColor())} />
          </div>

          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-sm truncate" title={vm.name}>{vm.name}</h3>
            <Badge
              variant="outline"
              className={cn('text-[10px] mt-1 border', getStateBadgeColor())}
            >
              {vm.state.toUpperCase()}
            </Badge>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 gap-2 text-xs">
          {/* CPU */}
          <div className="flex items-center gap-1.5 text-muted-foreground">
            <Cpu className="w-3 h-3" />
            <span>
              {vm.state === 'running' && cpuUsage !== null ? (
                <span className={cn(
                  cpuUsage > 80 && 'text-red-600 dark:text-red-400 font-medium',
                  cpuUsage > 60 && cpuUsage <= 80 && 'text-yellow-600 dark:text-yellow-400'
                )}>
                  {cpuUsage}%
                </span>
              ) : (
                `${vm.cpuCount} vCPU`
              )}
            </span>
          </div>

          {/* Memory */}
          <div className="flex items-center gap-1.5 text-muted-foreground">
            <MemoryStick className="w-3 h-3" />
            <span>
              {vm.state === 'running' && memoryUsed !== null
                ? `${(memoryUsed / 1024).toFixed(1)} GB`
                : `${(vm.memoryMb / 1024).toFixed(1)} GB`
              }
            </span>
          </div>

          {/* Disk */}
          <div className="flex items-center gap-1.5 text-muted-foreground">
            <HardDrive className="w-3 h-3" />
            <span>{vm.diskSizeGb} GB</span>
          </div>

          {/* Network */}
          <div className="flex items-center gap-1.5 text-muted-foreground">
            <Network className="w-3 h-3" />
            <span className="font-mono text-[10px] truncate">
              {getNetworkAddress() || '-'}
            </span>
          </div>
        </div>

        {/* Tags */}
        {vm.tags && vm.tags.length > 0 && (
          <div className="flex gap-1 mt-3 flex-wrap">
            {vm.tags.slice(0, 3).map(tag => (
              <Badge key={tag} variant="secondary" className="text-[10px] px-1.5 py-0 h-4">
                {tag}
              </Badge>
            ))}
            {vm.tags.length > 3 && (
              <Badge variant="secondary" className="text-[10px] px-1.5 py-0 h-4">
                +{vm.tags.length - 3}
              </Badge>
            )}
          </div>
        )}

        {/* Quick Actions */}
        <div className="flex gap-1 mt-3 pt-3 border-t border-border/50">
          {vm.state === 'stopped' && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 px-2 text-xs flex-1"
              onClick={(e) => handleQuickAction(e, () => startMutation.mutate())}
              disabled={isActionPending}
            >
              <Play className="w-3 h-3 mr-1" />
              Start
            </Button>
          )}
          {vm.state === 'running' && (
            <>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 px-2 text-xs flex-1"
                onClick={(e) => handleQuickAction(e, () => pauseMutation.mutate())}
                disabled={isActionPending}
              >
                <Pause className="w-3 h-3 mr-1" />
                Pause
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 px-2 text-xs flex-1"
                onClick={(e) => handleQuickAction(e, () => openConsoleMutation.mutate())}
              >
                <MonitorPlay className="w-3 h-3 mr-1" />
                Console
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 px-2 text-xs text-destructive hover:text-destructive"
                onClick={(e) => handleQuickAction(e, () => stopMutation.mutate())}
                disabled={isActionPending}
              >
                <Square className="w-3 h-3" />
              </Button>
            </>
          )}
          {vm.state === 'paused' && (
            <>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 px-2 text-xs flex-1"
                onClick={(e) => handleQuickAction(e, () => startMutation.mutate())}
                disabled={isActionPending}
              >
                <Play className="w-3 h-3 mr-1" />
                Resume
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 px-2 text-xs text-destructive hover:text-destructive"
                onClick={(e) => handleQuickAction(e, () => stopMutation.mutate())}
                disabled={isActionPending}
              >
                <Square className="w-3 h-3" />
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  )

  return (
    <IsoDropZone vmId={vm.id} vmName={vm.name} vmState={vm.state}>
      <VmContextMenu
        vm={vm}
        onOpenDetails={onDoubleClick}
        onOpenRename={onOpenRename}
        onOpenDelete={onOpenDelete}
        onOpenMigrate={onOpenMigrate}
      >
        {cardContent}
      </VmContextMenu>
    </IsoDropZone>
  )
}

export function VmCardView({ vms, selectedIds, onSelectionChange, onVmDoubleClick, onOpenRename, onOpenDelete, onOpenMigrate }: VmCardViewProps) {
  const handleToggleSelect = (vmId: string) => {
    if (selectedIds.includes(vmId)) {
      onSelectionChange(selectedIds.filter(id => id !== vmId))
    } else {
      onSelectionChange([...selectedIds, vmId])
    }
  }

  if (vms.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground gap-2">
        <Monitor className="w-8 h-8 opacity-50" />
        <p>No VMs match your current filters</p>
        <p className="text-sm">Try adjusting your search or filter criteria</p>
      </div>
    )
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4 p-4">
      {vms.map(vm => (
        <VmCard
          key={vm.id}
          vm={vm}
          isSelected={selectedIds.includes(vm.id)}
          onToggleSelect={() => handleToggleSelect(vm.id)}
          onDoubleClick={() => onVmDoubleClick(vm.id)}
          onOpenRename={onOpenRename ? () => onOpenRename(vm.id) : undefined}
          onOpenDelete={onOpenDelete ? () => onOpenDelete(vm.id) : undefined}
          onOpenMigrate={onOpenMigrate ? () => onOpenMigrate(vm.id) : undefined}
        />
      ))}
    </div>
  )
}
