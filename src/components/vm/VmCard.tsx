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
import { Play, Square, Pause, Trash2, Monitor, RotateCcw, Info } from 'lucide-react'
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
    mutationFn: () => api.deleteVm(vm.id, deleteDisks),
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
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between text-lg">
          {vm.name}
          <Badge className={stateColors[vm.state]}>
            {stateLabels[vm.state]}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <div className="space-y-2 text-sm">
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

          <div className="flex gap-2 pt-2 border-t">
            {vm.state === 'stopped' && (
              <Button
                size="sm"
                onClick={() => startMutation.mutate()}
                disabled={startMutation.isPending}
              >
                <Play className="w-4 h-4 mr-1" />
                Start
              </Button>
            )}
            {vm.state === 'running' && (
              <>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => rebootMutation.mutate()}
                  disabled={rebootMutation.isPending}
                >
                  <RotateCcw className="w-4 h-4 mr-1" />
                  Reboot
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => pauseMutation.mutate()}
                  disabled={pauseMutation.isPending}
                >
                  <Pause className="w-4 h-4 mr-1" />
                  Pause
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => stopMutation.mutate()}
                  disabled={stopMutation.isPending}
                >
                  <Square className="w-4 h-4 mr-1" />
                  Stop
                </Button>
              </>
            )}
            {vm.state === 'paused' && (
              <Button
                size="sm"
                onClick={() => resumeMutation.mutate()}
                disabled={resumeMutation.isPending}
              >
                <Play className="w-4 h-4 mr-1" />
                Resume
              </Button>
            )}
          </div>

          {/* View Details button */}
          <div className="pt-2 border-t">
            <Button
              size="sm"
              variant="outline"
              className="w-full"
              onClick={() => navigate(`/vms/${vm.id}`)}
            >
              <Info className="w-4 h-4 mr-1" />
              View Details & Snapshots
            </Button>
          </div>

          {/* Clone button - only show when VM is stopped */}
          {vm.state === 'stopped' && (
            <div className="pt-2 border-t">
              <CloneVmDialog vmId={vm.id} vmName={vm.name} />
            </div>
          )}

          {/* Console button - show when VM is running */}
          {vm.state === 'running' && (
            <div className="pt-2 border-t">
              <Button
                size="sm"
                variant="outline"
                className="w-full"
                onClick={() => openConsoleMutation.mutate()}
                disabled={openConsoleMutation.isPending}
              >
                <Monitor className="w-4 h-4 mr-1" />
                Open Console
              </Button>
            </div>
          )}

          {/* Delete button - only show when VM is stopped */}
          {vm.state === 'stopped' && (
            <div className="pt-2 border-t">
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button
                    size="sm"
                    variant="outline"
                    className="w-full text-destructive hover:text-destructive"
                    disabled={deleteMutation.isPending}
                  >
                    <Trash2 className="w-4 h-4 mr-1" />
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
  )
}
