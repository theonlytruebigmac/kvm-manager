import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
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
} from '@/components/ui/alert-dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Play,
  Square,
  Pause,
  RotateCcw,
  Trash2,
  Monitor,
  Copy,
  MoreVertical,
  Activity,
  Cpu,
  HardDrive,
  Info,
  Pencil,
} from 'lucide-react'
import { Input } from '@/components/ui/input'
import type { VM } from '@/lib/types'

interface EnhancedVmRowProps {
  vm: VM
  isSelected: boolean
  onToggleSelect: () => void
  isFocused?: boolean
  onFocus?: () => void
}

export function EnhancedVmRow({ vm, isSelected, onToggleSelect, isFocused = false, onFocus }: EnhancedVmRowProps) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [stats, setStats] = useState<{ cpu: number; memory: number } | null>(null)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [showRenameDialog, setShowRenameDialog] = useState(false)
  const [newName, setNewName] = useState(vm.name)
  const [deleteDisks, setDeleteDisks] = useState(false)
  const [deleteSnapshots, setDeleteSnapshots] = useState(false)

  // Poll stats for running VMs
  const { data: vmStats } = useQuery({
    queryKey: ['vm-stats', vm.id],
    queryFn: () => api.getVmStats(vm.id),
    enabled: vm.state === 'running',
    refetchInterval: 2000,
  })

  useEffect(() => {
    if (vmStats) {
      setStats({
        cpu: Math.round(vmStats.cpuUsagePercent),
        memory: Math.round((vmStats.memoryUsedMb / vmStats.memoryAvailableMb) * 100),
      })
    }
  }, [vmStats])

  const startMutation = useMutation({
    mutationFn: () => api.startVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} started`)
    },
    onError: (error) => {
      const errorMsg = String(error)
      if (errorMsg.includes('Permission denied') || errorMsg.includes('Could not open')) {
        toast.error(`Failed to start ${vm.name}: Permission denied on disk/ISO file. Copy to /var/lib/libvirt/images/ or fix permissions.`, { duration: 6000 })
      } else {
        toast.error(`Failed to start: ${error}`)
      }
    },
  })

  const stopMutation = useMutation({
    mutationFn: () => api.stopVm(vm.id),
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

  const rebootMutation = useMutation({
    mutationFn: () => api.rebootVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} rebooting`)
    },
    onError: (error) => toast.error(`Failed to reboot: ${error}`),
  })

  const openConsoleMutation = useMutation({
    mutationFn: () => api.openVncConsole(vm.id),
    onSuccess: () => toast.success('Console opened'),
    onError: (error) => toast.error(`Failed to open console: ${error}`),
  })

  const deleteMutation = useMutation({
    mutationFn: ({ deleteDisks, deleteSnapshots }: { deleteDisks: boolean; deleteSnapshots: boolean }) =>
      api.deleteVm(vm.id, deleteDisks, deleteSnapshots),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} deleted`)
      setShowDeleteDialog(false)
      setDeleteDisks(false)
      setDeleteSnapshots(false)
    },
    onError: (error) => toast.error(`Failed to delete: ${error}`),
  })

  const renameMutation = useMutation({
    mutationFn: (newName: string) => api.renameVm(vm.id, newName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM renamed to ${newName}`)
      setShowRenameDialog(false)
    },
    onError: (error) => toast.error(`Failed to rename: ${error}`),
  })

  const handleRename = () => {
    if (newName.trim() && newName !== vm.name) {
      renameMutation.mutate(newName.trim())
    } else {
      setShowRenameDialog(false)
    }
  }

  const handleDelete = () => {
    deleteMutation.mutate({ deleteDisks, deleteSnapshots })
  }

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

  const getStateIndicator = () => {
    switch (vm.state) {
      case 'running':
        return 'bg-green-500'
      case 'paused':
        return 'bg-yellow-500'
      default:
        return 'bg-gray-400'
    }
  }

  return (
    <Card
      className={`hover:bg-accent/50 transition-colors ${isFocused ? 'ring-2 ring-primary' : ''}`}
      onClick={onFocus}
    >
      <div className="p-4 flex items-center gap-4">
        {/* Checkbox */}
        <Checkbox
          checked={isSelected}
          onCheckedChange={onToggleSelect}
          className="data-[state=checked]:bg-primary"
        />

        {/* Status Indicator */}
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${getStateIndicator()} animate-pulse`} />
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <button
                onClick={() => navigate(`/vms/${vm.id}`)}
                className="font-semibold hover:text-primary transition-colors text-left truncate"
              >
                {vm.name}
              </button>
              {vm.tags && vm.tags.length > 0 && (
                <div className="flex gap-1">
                  {vm.tags.slice(0, 2).map(tag => (
                    <Badge key={tag} variant="outline" className="text-xs px-1.5 py-0">
                      {tag}
                    </Badge>
                  ))}
                  {vm.tags.length > 2 && (
                    <Badge variant="outline" className="text-xs px-1.5 py-0">
                      +{vm.tags.length - 2}
                    </Badge>
                  )}
                </div>
              )}
            </div>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Badge variant={getStateBadgeVariant()} className="text-xs">
                {vm.state.toUpperCase()}
              </Badge>
              {vm.osType && <span className="text-xs">{vm.osType}</span>}
              {vm.firmware !== 'bios' && (
                <Badge variant="outline" className="text-xs px-1.5 py-0 bg-blue-50 dark:bg-blue-950 border-blue-200 dark:border-blue-800">
                  {vm.firmware === 'uefi-secure' ? '🔒 UEFI' : 'UEFI'}
                </Badge>
              )}
              {vm.tpmEnabled && (
                <Badge variant="outline" className="text-xs px-1.5 py-0 bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800">
                  TPM
                </Badge>
              )}
            </div>
          </div>
        </div>

        {/* VM Specs */}
        <div className="hidden lg:flex items-center gap-6 ml-auto">
          <div className="flex items-center gap-2 text-sm">
            <Cpu className="h-4 w-4 text-muted-foreground" />
            <span>{vm.cpuCount} vCPU</span>
            {stats && (
              <span className="text-xs text-muted-foreground">({stats.cpu}%)</span>
            )}
          </div>
          <div className="flex items-center gap-2 text-sm">
            <Activity className="h-4 w-4 text-muted-foreground" />
            <span>{(vm.memoryMb / 1024).toFixed(1)} GB</span>
            {stats && (
              <span className="text-xs text-muted-foreground">({stats.memory}%)</span>
            )}
          </div>
          <div className="flex items-center gap-2 text-sm">
            <HardDrive className="h-4 w-4 text-muted-foreground" />
            <span>{vm.diskSizeGb} GB</span>
          </div>
        </div>

        {/* Quick Actions */}
        <div className="flex items-center gap-1">
          {vm.state === 'stopped' && (
            <Button
              variant="ghost"
              size="icon"
              onClick={() => startMutation.mutate()}
              disabled={startMutation.isPending}
              title="Start VM (Ctrl+P)"
            >
              <Play className="h-4 w-4 text-green-600" />
            </Button>
          )}

          {vm.state === 'running' && (
            <>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => pauseMutation.mutate()}
                disabled={pauseMutation.isPending}
                title="Pause VM (Ctrl+Z)"
              >
                <Pause className="h-4 w-4 text-yellow-600" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => openConsoleMutation.mutate()}
                disabled={openConsoleMutation.isPending}
                title="Open Console (Ctrl+O)"
              >
                <Monitor className="h-4 w-4 text-blue-600" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => stopMutation.mutate()}
                disabled={stopMutation.isPending}
                title="Stop VM (Ctrl+S)"
              >
                <Square className="h-4 w-4 text-red-600" />
              </Button>
            </>
          )}

          {vm.state === 'paused' && (
            <Button
              variant="ghost"
              size="icon"
              onClick={() => resumeMutation.mutate()}
              disabled={resumeMutation.isPending}
              title="Resume VM"
            >
              <Play className="h-4 w-4 text-green-600" />
            </Button>
          )}

          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate(`/vms/${vm.id}`)}
            title="VM Details (Ctrl+D)"
          >
            <Info className="h-4 w-4" />
          </Button>

          {/* More Actions Menu */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" title="More actions">
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48">
              {vm.state === 'running' && (
                <>
                  <DropdownMenuItem onClick={() => rebootMutation.mutate()}>
                    <RotateCcw className="mr-2 h-4 w-4" />
                    Reboot
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                </>
              )}
              {vm.state === 'stopped' && (
                <>
                  <DropdownMenuItem onClick={() => {
                    setNewName(vm.name)
                    setShowRenameDialog(true)
                  }}>
                    <Pencil className="mr-2 h-4 w-4" />
                    Rename VM
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => navigate(`/vms/${vm.id}/clone`)}>
                    <Copy className="mr-2 h-4 w-4" />
                    Clone VM
                  </DropdownMenuItem>
                </>
              )}
              <DropdownMenuItem onClick={() => navigate(`/vms/${vm.id}`)}>
                <Info className="mr-2 h-4 w-4" />
                View Details
              </DropdownMenuItem>
              {vm.state === 'stopped' && (
                <>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    className="text-destructive"
                    onClick={() => setShowDeleteDialog(true)}
                  >
                    <Trash2 className="mr-2 h-4 w-4" />
                    Delete VM
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Virtual Machine?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete <strong>{vm.name}</strong>? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-4 py-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="delete-disks"
                checked={deleteDisks}
                onCheckedChange={(checked) => setDeleteDisks(checked as boolean)}
              />
              <Label htmlFor="delete-disks" className="text-sm font-normal cursor-pointer">
                Delete attached storage disks (cannot be recovered)
              </Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="delete-snapshots"
                checked={deleteSnapshots}
                onCheckedChange={(checked) => setDeleteSnapshots(checked as boolean)}
              />
              <Label htmlFor="delete-snapshots" className="text-sm font-normal cursor-pointer">
                Delete all snapshots
              </Label>
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? 'Deleting...' : 'Delete VM'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Rename Dialog */}
      <AlertDialog open={showRenameDialog} onOpenChange={setShowRenameDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Rename Virtual Machine</AlertDialogTitle>
            <AlertDialogDescription>
              Enter a new name for this virtual machine. The VM must be stopped to rename.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="py-4">
            <Input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Enter new VM name"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  handleRename()
                }
              }}
            />
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleRename}
              disabled={renameMutation.isPending || !newName.trim() || newName === vm.name}
            >
              {renameMutation.isPending ? 'Renaming...' : 'Rename'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  )
}
