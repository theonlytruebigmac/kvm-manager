import { useNavigate } from 'react-router-dom'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { Play, Square, Pause, Trash2, Monitor, RotateCcw, Info, Copy, XCircle, Edit, Loader2 } from 'lucide-react'
import type { VM } from '@/lib/types'
import { api } from '@/lib/tauri'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useState } from 'react'
import { CloneVmDialog } from './CloneVmDialog'

interface VmCardProps {
  vm: VM
}

export function VmCard({ vm }: VmCardProps) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [deleteDisks, setDeleteDisks] = useState(false)

  const startMutation = useMutation({
    mutationFn: () => api.startVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM "${vm.name}" started successfully`)
    },
    onError: (error) => {
      toast.error(`Failed to start VM: ${error}`)
    },
  })

  const stopMutation = useMutation({
    mutationFn: () => api.stopVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM "${vm.name}" stopped successfully`)
    },
    onError: (error) => {
      toast.error(`Failed to stop VM: ${error}`)
    },
  })

  const pauseMutation = useMutation({
    mutationFn: () => api.pauseVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM "${vm.name}" paused successfully`)
    },
    onError: (error) => {
      toast.error(`Failed to pause VM: ${error}`)
    },
  })

  const resumeMutation = useMutation({
    mutationFn: () => api.resumeVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM "${vm.name}" resumed successfully`)
    },
    onError: (error) => {
      toast.error(`Failed to resume VM: ${error}`)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => api.deleteVm(vm.id, deleteDisks, false),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      const diskMsg = deleteDisks ? ' (including disks)' : ''
      toast.success(`VM "${vm.name}" deleted successfully${diskMsg}`)
      setDeleteDisks(false)
    },
    onError: (error) => {
      toast.error(`Failed to delete VM: ${error}`)
    },
  })

  const openConsoleMutation = useMutation({
    mutationFn: () => api.openVncConsole(vm.id),
    onSuccess: () => {
      toast.success('VNC console opened')
    },
    onError: (error) => {
      toast.error(`Failed to open console: ${error}`)
    },
  })

  const rebootMutation = useMutation({
    mutationFn: () => api.rebootVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM "${vm.name}" rebooted successfully`)
    },
    onError: (error) => {
      toast.error(`Failed to reboot VM: ${error}`)
    },
  })

  const stateColors: Record<VM['state'], string> = {
    running: 'bg-green-500',
    stopped: 'bg-gray-500',
    paused: 'bg-yellow-500',
    suspended: 'bg-blue-500',
    crashed: 'bg-red-500',
  }

  const stateLabels: Record<VM['state'], string> = {
    running: 'Running',
    stopped: 'Stopped',
    paused: 'Paused',
    suspended: 'Suspended',
    crashed: 'Crashed',
  }

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
    <Card
      className="bg-[var(--panel-bg)] cursor-pointer transition-shadow hover:shadow-md"
      onDoubleClick={() => navigate(`/vms/${vm.id}`)}
    >
      <CardHeader className="py-3 px-4">
        <CardTitle className="flex items-center justify-between text-desktop-lg font-semibold">
          {vm.name}
          <Badge className={stateColors[vm.state]}>
            {stateLabels[vm.state]}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="px-4 pb-3">
        <div className="space-y-2">
          <div className="space-y-1.5 text-desktop-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">CPU:</span>
              <span className="font-medium">{vm.cpuCount} cores</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Memory:</span>
              <span className="font-medium">{vm.memoryMb} MB</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Disk:</span>
              <span className="font-medium">{vm.diskSizeGb} GB</span>
            </div>
          </div>

          <div className="flex gap-1.5 pt-2 border-t border-[var(--panel-border)]">
            {vm.state === 'stopped' && (
              <Button
                size="sm"
                onClick={() => startMutation.mutate()}
                disabled={startMutation.isPending}
                className="h-7 text-desktop-xs"
              >
                {startMutation.isPending ? (
                  <Loader2 className="w-3.5 h-3.5 mr-1 animate-spin" />
                ) : (
                  <Play className="w-3.5 h-3.5 mr-1" />
                )}
                {startMutation.isPending ? 'Starting...' : 'Start'}
              </Button>
            )}
            {vm.state === 'running' && (
              <>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => rebootMutation.mutate()}
                  disabled={rebootMutation.isPending}
                  className="h-7 text-desktop-xs"
                >
                  <RotateCcw className="w-3.5 h-3.5 mr-1" />
                  Reboot
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => pauseMutation.mutate()}
                  disabled={pauseMutation.isPending}
                  className="h-7 text-desktop-xs"
                >
                  <Pause className="w-3.5 h-3.5 mr-1" />
                  Pause
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => stopMutation.mutate()}
                  disabled={stopMutation.isPending}
                  className="h-7 text-desktop-xs"
                >
                  {stopMutation.isPending ? (
                    <Loader2 className="w-3.5 h-3.5 mr-1 animate-spin" />
                  ) : (
                    <Square className="w-3.5 h-3.5 mr-1" />
                  )}
                  {stopMutation.isPending ? 'Stopping...' : 'Stop'}
                </Button>
              </>
            )}
            {vm.state === 'paused' && (
              <Button
                size="sm"
                onClick={() => resumeMutation.mutate()}
                disabled={resumeMutation.isPending}
                className="h-7 text-desktop-xs"
              >
                <Play className="w-3.5 h-3.5 mr-1" />
                Resume
              </Button>
            )}
          </div>

          {/* View Details button */}
          <div className="pt-1.5 border-t border-[var(--panel-border)]">
            <Button
              size="sm"
              variant="outline"
              className="w-full h-7 text-desktop-xs"
              onClick={() => navigate(`/vms/${vm.id}`)}
            >
              <Info className="w-3.5 h-3.5 mr-1" />
              View Details & Snapshots
            </Button>
          </div>

          {/* Clone button - only show when VM is stopped */}
          {vm.state === 'stopped' && (
            <div className="pt-1.5 border-t border-[var(--panel-border)]">
              <CloneVmDialog vmId={vm.id} vmName={vm.name} />
            </div>
          )}

          {/* Console button - show when VM is running */}
          {vm.state === 'running' && (
            <div className="pt-1.5 border-t border-[var(--panel-border)]">
              <Button
                size="sm"
                variant="outline"
                className="w-full h-7 text-desktop-xs"
                onClick={() => openConsoleMutation.mutate()}
                disabled={openConsoleMutation.isPending}
              >
                <Monitor className="w-3.5 h-3.5 mr-1" />
                Open Console
              </Button>
            </div>
          )}

          {/* Delete button - only show when VM is stopped */}
          {vm.state === 'stopped' && (
            <div className="pt-1.5 border-t border-[var(--panel-border)]">
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button
                    size="sm"
                    variant="outline"
                    className="w-full h-7 text-desktop-xs text-destructive hover:text-destructive"
                    disabled={deleteMutation.isPending}
                  >
                    <Trash2 className="w-3.5 h-3.5 mr-1" />
                    Delete VM
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                    <AlertDialogDescription>
                      This will permanently delete the VM "{vm.name}". This action cannot be undone.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <div className="flex items-center space-x-2 py-4">
                    <Checkbox
                      id="delete-disks"
                      checked={deleteDisks}
                      onCheckedChange={(checked) => setDeleteDisks(checked as boolean)}
                    />
                    <Label
                      htmlFor="delete-disks"
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      Also delete disk images (cannot be undone)
                    </Label>
                  </div>
                  <AlertDialogFooter>
                    <AlertDialogCancel onClick={() => setDeleteDisks(false)}>Cancel</AlertDialogCancel>
                    <AlertDialogAction
                      className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                      onClick={() => deleteMutation.mutate()}
                    >
                      Delete VM{deleteDisks ? ' & Disks' : ''}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
      </ContextMenuTrigger>

      <ContextMenuContent>
        {/* State controls */}
        {vm.state === 'stopped' && (
          <ContextMenuItem icon={<Play className="w-3.5 h-3.5" />} onClick={() => startMutation.mutate()}>
            Start
          </ContextMenuItem>
        )}
        {vm.state === 'running' && (
          <>
            <ContextMenuItem icon={<Pause className="w-3.5 h-3.5" />} onClick={() => pauseMutation.mutate()}>
              Pause
            </ContextMenuItem>
            <ContextMenuItem icon={<Square className="w-3.5 h-3.5" />} onClick={() => stopMutation.mutate()}>
              Stop
            </ContextMenuItem>
            <ContextMenuItem icon={<XCircle className="w-3.5 h-3.5" />} onClick={() => api.forceStopVm(vm.id)}>
              Force Stop
            </ContextMenuItem>
            <ContextMenuItem icon={<RotateCcw className="w-3.5 h-3.5" />} onClick={() => rebootMutation.mutate()}>
              Reboot
            </ContextMenuItem>
          </>
        )}
        {vm.state === 'paused' && (
          <ContextMenuItem icon={<Play className="w-3.5 h-3.5" />} onClick={() => resumeMutation.mutate()}>
            Resume
          </ContextMenuItem>
        )}

        <ContextMenuSeparator />

        {/* Console and Details */}
        <ContextMenuItem
          icon={<Monitor className="w-3.5 h-3.5" />}
          onClick={() => openConsoleMutation.mutate()}
          disabled={vm.state !== 'running'}
          shortcut="Ctrl+Shift+C"
        >
          Open Console
        </ContextMenuItem>
        <ContextMenuItem
          icon={<Edit className="w-3.5 h-3.5" />}
          onClick={() => navigate(`/vms/${vm.id}`)}
          shortcut="Enter"
        >
          Open Details
        </ContextMenuItem>

        <ContextMenuSeparator />

        {/* Management actions */}
        <ContextMenuItem
          icon={<Copy className="w-3.5 h-3.5" />}
          disabled={vm.state !== 'stopped'}
        >
          Clone VM
        </ContextMenuItem>
        <ContextMenuItem
          icon={<Trash2 className="w-3.5 h-3.5" />}
          disabled={vm.state !== 'stopped'}
          shortcut="Del"
        >
          Delete VM
        </ContextMenuItem>

        <ContextMenuSeparator />

        <ContextMenuItem icon={<Info className="w-3.5 h-3.5" />}>
          Properties
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
