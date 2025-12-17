import { useEffect, useState, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
} from '@/components/ui/command'
import { api } from '@/lib/tauri'
import { useToolbarStore } from '@/hooks/useToolbarActions'
import { toast } from 'sonner'
import {
  Plus,
  Upload,
  Play,
  Square,
  Pause,
  Monitor,
  Settings,
  HardDrive,
  Network,
  LayoutDashboard,
  FileText,
  Clock,
  Bell,
  Database,
  Lightbulb,
  RotateCw,
  Power,
  Keyboard,
  Gauge,
} from 'lucide-react'
import type { VM } from '@/lib/types'

interface CommandPaletteProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export function CommandPalette({ open: controlledOpen, onOpenChange }: CommandPaletteProps) {
  const [internalOpen, setInternalOpen] = useState(false)
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { setShowCreateVm, setShowImportVm, selectedVmIds, focusedVmId } = useToolbarStore()

  // Support both controlled and uncontrolled modes
  const open = controlledOpen ?? internalOpen
  const setOpen = onOpenChange ?? setInternalOpen

  // Fetch VMs for VM-specific commands
  const { data: vms } = useQuery({
    queryKey: ['vms'],
    queryFn: api.getVms,
  })

  // Get currently selected/focused VM
  const activeVm = vms?.find(vm => vm.id === focusedVmId) ??
                   vms?.find(vm => selectedVmIds.includes(vm.id))

  // VM mutations
  const startMutation = useMutation({
    mutationFn: (vmId: string) => api.startVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM started')
    },
    onError: (err) => toast.error(`Failed to start: ${err}`),
  })

  const stopMutation = useMutation({
    mutationFn: (vmId: string) => api.forceStopVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM stopped')
    },
    onError: (err) => toast.error(`Failed to stop: ${err}`),
  })

  const pauseMutation = useMutation({
    mutationFn: (vmId: string) => api.pauseVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM paused')
    },
    onError: (err) => toast.error(`Failed to pause: ${err}`),
  })

  const resumeMutation = useMutation({
    mutationFn: (vmId: string) => api.resumeVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM resumed')
    },
    onError: (err) => toast.error(`Failed to resume: ${err}`),
  })

  const rebootMutation = useMutation({
    mutationFn: (vmId: string) => api.rebootVm(vmId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM rebooted')
    },
    onError: (err) => toast.error(`Failed to reboot: ${err}`),
  })

  // Global keyboard shortcut (Ctrl+K)
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setOpen(!open)
      }
    }

    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [open, setOpen])

  const runCommand = useCallback((callback: () => void) => {
    setOpen(false)
    callback()
  }, [setOpen])

  // Helper to open VM details
  const openVmDetails = async (vm: VM) => {
    try {
      await api.openVmDetailsWindow(vm.id, vm.name)
    } catch (err) {
      toast.error(`Failed to open VM details: ${err}`)
    }
  }

  // Helper to open console
  const openConsole = async (vm: VM) => {
    try {
      await api.openConsoleWindow(vm.id, vm.name)
    } catch (err) {
      toast.error(`Failed to open console: ${err}`)
    }
  }

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>

        {/* Quick Actions */}
        <CommandGroup heading="Actions">
          <CommandItem onSelect={() => runCommand(() => setShowCreateVm(true))}>
            <Plus className="mr-2 h-4 w-4" />
            <span>Create New VM</span>
            <CommandShortcut>⌃N</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => setShowImportVm(true))}>
            <Upload className="mr-2 h-4 w-4" />
            <span>Import VM from XML</span>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator />

        {/* VM Actions (context-aware) */}
        {activeVm && (
          <>
            <CommandGroup heading={`VM: ${activeVm.name}`}>
              {activeVm.state === 'stopped' && (
                <CommandItem onSelect={() => runCommand(() => startMutation.mutate(activeVm.id))}>
                  <Play className="mr-2 h-4 w-4 text-green-500" />
                  <span>Start VM</span>
                  <CommandShortcut>⌃P</CommandShortcut>
                </CommandItem>
              )}
              {activeVm.state === 'running' && (
                <>
                  <CommandItem onSelect={() => runCommand(() => stopMutation.mutate(activeVm.id))}>
                    <Square className="mr-2 h-4 w-4 text-red-500" />
                    <span>Stop VM</span>
                    <CommandShortcut>⌃S</CommandShortcut>
                  </CommandItem>
                  <CommandItem onSelect={() => runCommand(() => pauseMutation.mutate(activeVm.id))}>
                    <Pause className="mr-2 h-4 w-4 text-yellow-500" />
                    <span>Pause VM</span>
                  </CommandItem>
                  <CommandItem onSelect={() => runCommand(() => rebootMutation.mutate(activeVm.id))}>
                    <RotateCw className="mr-2 h-4 w-4" />
                    <span>Reboot VM</span>
                  </CommandItem>
                  <CommandItem onSelect={() => runCommand(() => openConsole(activeVm))}>
                    <Monitor className="mr-2 h-4 w-4" />
                    <span>Open Console</span>
                    <CommandShortcut>⌃O</CommandShortcut>
                  </CommandItem>
                </>
              )}
              {activeVm.state === 'paused' && (
                <CommandItem onSelect={() => runCommand(() => resumeMutation.mutate(activeVm.id))}>
                  <Play className="mr-2 h-4 w-4 text-green-500" />
                  <span>Resume VM</span>
                </CommandItem>
              )}
              <CommandItem onSelect={() => runCommand(() => openVmDetails(activeVm))}>
                <Settings className="mr-2 h-4 w-4" />
                <span>Open VM Settings</span>
                <CommandShortcut>⌃D</CommandShortcut>
              </CommandItem>
            </CommandGroup>
            <CommandSeparator />
          </>
        )}

        {/* All VMs */}
        {vms && vms.length > 0 && (
          <>
            <CommandGroup heading="Virtual Machines">
              {vms.map((vm) => (
                <CommandItem
                  key={vm.id}
                  value={`vm ${vm.name}`}
                  onSelect={() => runCommand(() => openVmDetails(vm))}
                >
                  <Power className={`mr-2 h-4 w-4 ${
                    vm.state === 'running' ? 'text-green-500' :
                    vm.state === 'paused' ? 'text-yellow-500' : 'text-muted-foreground'
                  }`} />
                  <span>{vm.name}</span>
                  <span className="ml-2 text-xs text-muted-foreground capitalize">
                    {vm.state}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
            <CommandSeparator />
          </>
        )}

        {/* Navigation */}
        <CommandGroup heading="Navigation">
          <CommandItem onSelect={() => runCommand(() => navigate('/'))}>
            <LayoutDashboard className="mr-2 h-4 w-4" />
            <span>Virtual Machines</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/performance'))}>
            <Gauge className="mr-2 h-4 w-4" />
            <span>Performance Monitor</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/storage'))}>
            <HardDrive className="mr-2 h-4 w-4" />
            <span>Storage Manager</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/networks'))}>
            <Network className="mr-2 h-4 w-4" />
            <span>Network Manager</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/templates'))}>
            <FileText className="mr-2 h-4 w-4" />
            <span>Templates</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/schedules'))}>
            <Clock className="mr-2 h-4 w-4" />
            <span>Schedules</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/alerts'))}>
            <Bell className="mr-2 h-4 w-4" />
            <span>Alerts</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/backups'))}>
            <Database className="mr-2 h-4 w-4" />
            <span>Backups</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/insights'))}>
            <Lightbulb className="mr-2 h-4 w-4" />
            <span>Insights</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => navigate('/settings'))}>
            <Settings className="mr-2 h-4 w-4" />
            <span>Settings</span>
            <CommandShortcut>⌃,</CommandShortcut>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator />

        {/* Help */}
        <CommandGroup heading="Help">
          <CommandItem onSelect={() => runCommand(() => {
            // Trigger keyboard shortcuts dialog - this will be handled by a global event
            window.dispatchEvent(new CustomEvent('show-keyboard-shortcuts'))
          })}>
            <Keyboard className="mr-2 h-4 w-4" />
            <span>Keyboard Shortcuts</span>
            <CommandShortcut>⌃?</CommandShortcut>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}

// Hook for using command palette anywhere
export function useCommandPalette() {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setOpen((open) => !open)
      }
    }

    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [])

  return { open, setOpen }
}
