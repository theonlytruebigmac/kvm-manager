import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import {
  Play,
  Square,
  Pause,
  RotateCw,
  Monitor,
  Settings,
  Copy,
  Trash2,
  Camera,
  Edit3,
  Power,
  Download,
  ArrowRightLeft,
  Moon,
} from 'lucide-react'
import type { VM } from '@/lib/types'

interface VmContextMenuProps {
  vm: VM
  children: React.ReactNode
  onOpenDetails?: () => void
  onOpenRename?: () => void
  onOpenDelete?: () => void
  onOpenMigrate?: () => void
}

export function VmContextMenu({ vm, children, onOpenDetails, onOpenRename, onOpenDelete, onOpenMigrate }: VmContextMenuProps) {
  const queryClient = useQueryClient()

  const isRunning = vm.state === 'running'
  const isPaused = vm.state === 'paused'
  const isStopped = vm.state === 'stopped'

  // Mutations
  const startMutation = useMutation({
    mutationFn: () => api.startVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} started`)
    },
    onError: (err) => toast.error(`Failed to start: ${err}`),
  })

  const stopMutation = useMutation({
    mutationFn: () => api.forceStopVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} stopped`)
    },
    onError: (err) => toast.error(`Failed to stop: ${err}`),
  })

  const pauseMutation = useMutation({
    mutationFn: () => api.pauseVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} paused`)
    },
    onError: (err) => toast.error(`Failed to pause: ${err}`),
  })

  const resumeMutation = useMutation({
    mutationFn: () => api.resumeVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} resumed`)
    },
    onError: (err) => toast.error(`Failed to resume: ${err}`),
  })

  const rebootMutation = useMutation({
    mutationFn: () => api.rebootVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} rebooted`)
    },
    onError: (err) => toast.error(`Failed to reboot: ${err}`),
  })

  const hibernateMutation = useMutation({
    mutationFn: () => api.hibernateVm(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} hibernated - will resume on next start`)
    },
    onError: (err) => toast.error(`Failed to hibernate: ${err}`),
  })

  const cloneMutation = useMutation({
    mutationFn: () => api.cloneVm(vm.id, `${vm.name}-clone`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`${vm.name} cloned`)
    },
    onError: (err) => toast.error(`Failed to clone: ${err}`),
  })

  const createSnapshotMutation = useMutation({
    mutationFn: () => api.createSnapshot(vm.id, {
      name: `snapshot-${Date.now()}`,
      description: 'Quick snapshot from context menu',
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['snapshots', vm.id] })
      toast.success('Snapshot created')
    },
    onError: (err) => toast.error(`Failed to create snapshot: ${err}`),
  })

  const handleOpenConsole = async () => {
    try {
      await api.openConsoleWindow(vm.id, vm.name)
    } catch (err) {
      toast.error(`Failed to open console: ${err}`)
    }
  }

  const handleExportXml = async () => {
    try {
      const xml = await api.exportVm(vm.id)
      // Copy to clipboard
      await navigator.clipboard.writeText(xml)
      toast.success('VM XML copied to clipboard')
    } catch (err) {
      toast.error(`Failed to export: ${err}`)
    }
  }

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent className="w-56">
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
            <ContextMenuItem onClick={() => rebootMutation.mutate()}>
              <RotateCw className="mr-2 h-4 w-4" />
              Reboot
            </ContextMenuItem>
            <ContextMenuItem onClick={() => hibernateMutation.mutate()}>
              <Moon className="mr-2 h-4 w-4 text-blue-500" />
              Hibernate
            </ContextMenuItem>
            <ContextMenuSub>
              <ContextMenuSubTrigger>
                <Power className="mr-2 h-4 w-4 text-red-500" />
                Shut Down
              </ContextMenuSubTrigger>
              <ContextMenuSubContent>
                <ContextMenuItem onClick={() => stopMutation.mutate()}>
                  <Square className="mr-2 h-4 w-4" />
                  Force Stop
                </ContextMenuItem>
              </ContextMenuSubContent>
            </ContextMenuSub>
          </>
        )}

        <ContextMenuSeparator />

        {/* Console */}
        <ContextMenuItem onClick={handleOpenConsole} disabled={!isRunning}>
          <Monitor className="mr-2 h-4 w-4" />
          Open Console
        </ContextMenuItem>

        <ContextMenuItem onClick={onOpenDetails}>
          <Settings className="mr-2 h-4 w-4" />
          Settings
        </ContextMenuItem>

        <ContextMenuSeparator />

        {/* Snapshots */}
        <ContextMenuItem onClick={() => createSnapshotMutation.mutate()}>
          <Camera className="mr-2 h-4 w-4" />
          Take Snapshot
        </ContextMenuItem>

        <ContextMenuSeparator />

        {/* VM Management */}
        <ContextMenuItem onClick={() => cloneMutation.mutate()}>
          <Copy className="mr-2 h-4 w-4" />
          Clone
        </ContextMenuItem>

        <ContextMenuItem onClick={onOpenMigrate}>
          <ArrowRightLeft className="mr-2 h-4 w-4" />
          Migrate
        </ContextMenuItem>

        <ContextMenuItem onClick={onOpenRename}>
          <Edit3 className="mr-2 h-4 w-4" />
          Rename
        </ContextMenuItem>

        <ContextMenuItem onClick={handleExportXml}>
          <Download className="mr-2 h-4 w-4" />
          Export XML
        </ContextMenuItem>

        <ContextMenuSeparator />

        {/* Delete */}
        <ContextMenuItem
          onClick={onOpenDelete}
          className="text-red-600 focus:text-red-600 focus:bg-red-50 dark:focus:bg-red-950"
        >
          <Trash2 className="mr-2 h-4 w-4" />
          Delete
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
