import { useParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { useState, useEffect } from 'react'
import {
  Play,
  Square,
  Pause,
  RotateCw,
  Monitor,
  Loader2,
  Copy,
  Cpu,
  MemoryStick,
  HardDrive,
  Network,
  Activity,
  ChevronDown,
  Plus,
  Power,
  Settings2,
  Disc,
  Tv,
  Keyboard,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { toast } from 'sonner'
import { useWindowState } from '@/hooks/useWindowState'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'

// Import components
import { ResourceGraphs } from '@/components/vm/ResourceGraphs'
import { SnapshotManager } from '@/components/vm/SnapshotManager'
import { GuestInfo } from '@/components/vm/GuestInfo'
import { AddHardwareDialog } from '@/components/vm/AddHardwareDialog'
import {
  CpuEditor,
  MemoryEditor,
  BootEditor,
  DiskEditor,
  CdromEditor,
  NetworkEditor,
  GraphicsEditor,
  VideoEditor,
  SoundEditor,
  InputEditor,
  TpmEditor,
  NumaEditor,
} from '@/components/vm/devices'

/**
 * VM Details Window - Redesigned with modern UX
 *
 * Design principles:
 * - Clear visual hierarchy with hero header
 * - Tab-based navigation (not tree sidebar)
 * - Card-based content organization
 * - Contextual actions per section
 */

export function VmDetailsWindow() {
  const { vmId } = useParams<{ vmId: string }>()
  const queryClient = useQueryClient()
  const [activeTab, setActiveTab] = useState('overview')
  const [showAddHardware, setShowAddHardware] = useState(false)

  // Debug logging
  console.log('VmDetailsWindow rendering, vmId:', vmId)

  useWindowState()

  const { data: vm, isLoading, error } = useQuery({
    queryKey: ['vm', vmId],
    queryFn: () => api.getVm(vmId!),
    enabled: !!vmId,
    refetchInterval: 2000,
  })

  // Debug logging for query results
  console.log('VM Query state:', { vm, isLoading, error, vmId })

  // Mutations
  const startMutation = useMutation({
    mutationFn: () => api.startVm(vmId!),
    onSuccess: () => {
      toast.success('VM started')
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
    },
    onError: (error: Error) => toast.error(`Failed to start: ${error.message}`),
  })

  const stopMutation = useMutation({
    mutationFn: () => api.stopVm(vmId!),
    onSuccess: () => {
      toast.success('VM stopped')
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
    },
    onError: (error: Error) => toast.error(`Failed to stop: ${error.message}`),
  })

  const pauseMutation = useMutation({
    mutationFn: () => api.pauseVm(vmId!),
    onSuccess: () => {
      toast.success('VM paused')
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
    },
    onError: (error: Error) => toast.error(`Failed to pause: ${error.message}`),
  })

  const resumeMutation = useMutation({
    mutationFn: () => api.resumeVm(vmId!),
    onSuccess: () => {
      toast.success('VM resumed')
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
    },
    onError: (error: Error) => toast.error(`Failed to resume: ${error.message}`),
  })

  const rebootMutation = useMutation({
    mutationFn: () => api.rebootVm(vmId!),
    onSuccess: () => {
      toast.success('VM rebooted')
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
    },
    onError: (error: Error) => toast.error(`Failed to reboot: ${error.message}`),
  })

  const handleOpenConsole = async () => {
    if (!vm) return
    try {
      await api.openConsoleWindow(vm.id, vm.name)
    } catch (error) {
      toast.error(`Failed to open console: ${(error as Error).message}`)
    }
  }

  const handleClone = async () => {
    if (!vm) return
    try {
      const newName = `${vm.name}-clone`
      await api.cloneVm(vm.id, newName)
      toast.success(`Cloned as ${newName}`)
    } catch (error) {
      toast.error(`Clone failed: ${(error as Error).message}`)
    }
  }

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return

      if (event.key === 'Escape' || (event.ctrlKey && event.key === 'w')) {
        event.preventDefault()
        getCurrentWindow().close()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  if (isLoading) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-background">
        <Loader2 className="w-8 h-8 animate-spin text-primary" />
      </div>
    )
  }

  if (!vm) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <p className="text-lg font-medium">VM not found</p>
          <p className="text-sm text-muted-foreground mt-1">
            The requested virtual machine could not be loaded
          </p>
        </div>
      </div>
    )
  }

  const isRunning = vm.state === 'running'
  const isPaused = vm.state === 'paused'
  const isStopped = vm.state === 'stopped'

  const stateColors: Record<string, string> = {
    running: 'bg-green-500/15 text-green-600 border-green-500/30',
    stopped: 'bg-zinc-500/15 text-zinc-500 border-zinc-500/30',
    paused: 'bg-yellow-500/15 text-yellow-600 border-yellow-500/30',
  }

  return (
    <div className="h-screen w-screen flex flex-col bg-background overflow-hidden">
      {/* Compact Header Bar */}
      <div className="shrink-0 border-b bg-card/50 px-4 py-2 flex items-center justify-between">
        {/* Left: VM Identity */}
        <div className="flex items-center gap-3">
          <h1 className="text-base font-semibold">{vm.name}</h1>
          <Badge
            variant="outline"
            className={cn('capitalize text-xs h-5', stateColors[vm.state] || stateColors.stopped)}
          >
            {vm.state}
          </Badge>
          <div className="hidden sm:flex items-center gap-3 text-xs text-muted-foreground ml-2 pl-3 border-l">
            <span className="flex items-center gap-1">
              <Cpu className="w-3 h-3" />
              {vm.cpus}
            </span>
            <span className="flex items-center gap-1">
              <MemoryStick className="w-3 h-3" />
              {(vm.memoryMb / 1024).toFixed(1)}G
            </span>
            <span className="flex items-center gap-1">
              <HardDrive className="w-3 h-3" />
              {vm.diskSizeGb}G
            </span>
          </div>
        </div>

        {/* Right: Actions */}
        <div className="flex items-center gap-1">
          {/* Power dropdown */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant={isRunning ? 'default' : 'outline'} size="sm" className="h-7 gap-1.5 px-2">
                <Power className="w-3.5 h-3.5" />
                <span className="hidden sm:inline text-xs">Power</span>
                <ChevronDown className="w-3 h-3 opacity-60" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {isStopped && (
                <DropdownMenuItem onClick={() => startMutation.mutate()}>
                  <Play className="w-4 h-4 mr-2" />Start
                </DropdownMenuItem>
              )}
              {isPaused && (
                <DropdownMenuItem onClick={() => resumeMutation.mutate()}>
                  <Play className="w-4 h-4 mr-2" />Resume
                </DropdownMenuItem>
              )}
              {isRunning && (
                <>
                  <DropdownMenuItem onClick={() => pauseMutation.mutate()}>
                    <Pause className="w-4 h-4 mr-2" />Pause
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => rebootMutation.mutate()}>
                    <RotateCw className="w-4 h-4 mr-2" />Reboot
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={() => stopMutation.mutate()} className="text-red-600">
                    <Square className="w-4 h-4 mr-2" />Force Stop
                  </DropdownMenuItem>
                </>
              )}
              {(isRunning || isPaused) && (
                <>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={() => stopMutation.mutate()}>
                    <Square className="w-4 h-4 mr-2" />Shut Down
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>

          <Button variant="outline" size="sm" onClick={handleOpenConsole} disabled={!isRunning} className="h-7 gap-1.5 px-2">
            <Monitor className="w-3.5 h-3.5" />
            <span className="hidden sm:inline text-xs">Console</span>
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-7 w-7 p-0">
                <Settings2 className="w-3.5 h-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={handleClone}>
                <Copy className="w-4 h-4 mr-2" />Clone VM
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* Tab Navigation - Compact */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col overflow-hidden">
        <div className="shrink-0 border-b px-4">
          <TabsList className="h-8 bg-transparent gap-0">
            <TabsTrigger value="overview" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              Overview
            </TabsTrigger>
            <TabsTrigger value="network" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              Network
            </TabsTrigger>
            <TabsTrigger value="storage" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              Storage
            </TabsTrigger>
            <TabsTrigger value="graphics" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              Graphics
            </TabsTrigger>
            <TabsTrigger value="cdrom" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              CD/DVD
            </TabsTrigger>
            <TabsTrigger value="additional" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              Additional
            </TabsTrigger>
            <TabsTrigger value="snapshots" className="text-xs h-8 px-3 data-[state=active]:bg-muted rounded-none data-[state=active]:border-b-2 data-[state=active]:border-primary">
              Snapshots
            </TabsTrigger>
          </TabsList>
        </div>

        {/* Tab Content */}
        <div className="flex-1 overflow-auto">
          {/* Overview Tab - Clean, modern design */}
          <TabsContent value="overview" className="m-0 h-full">
            <div className="p-4 space-y-4">
              {/* Top Row: System Config + Guest Agent side by side */}
              <div className="grid md:grid-cols-2 gap-4">
                {/* System Configuration Card */}
                <div className="border rounded-lg overflow-hidden">
                  <div className="bg-muted/50 px-3 py-1.5 border-b flex items-center justify-between">
                    <span className="text-xs font-medium">System</span>
                    <span className="text-[10px] text-muted-foreground">
                      {vm.machine?.includes('q35') ? 'Q35' : 'i440FX'} • {vm.firmware?.toUpperCase() || 'BIOS'}
                    </span>
                  </div>
                  <div className="p-3 space-y-3">
                    {/* CPU */}
                    <div className="flex items-center gap-2">
                      <Cpu className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                      <CpuEditor vm={vm} compact />
                    </div>
                    {/* Memory */}
                    <div className="flex items-center gap-2">
                      <MemoryStick className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                      <MemoryEditor vm={vm} compact />
                    </div>
                    {/* Boot */}
                    <div className="flex items-start gap-2">
                      <HardDrive className="w-3.5 h-3.5 text-muted-foreground shrink-0 mt-0.5" />
                      <BootEditor vm={vm} compact />
                    </div>
                  </div>
                </div>

                {/* Guest Agent Card */}
                <div className="border rounded-lg overflow-hidden">
                  <div className="bg-muted/50 px-4 py-2 border-b">
                    <span className="text-sm font-medium">Guest Agent</span>
                  </div>
                  <div className="p-4">
                    <GuestInfo vmId={vm.id} vmName={vm.name} vmState={vm.state} osType={vm.osType} compact />
                  </div>
                </div>
              </div>

              {/* Performance Section - Full width at bottom with graphs */}
              <div className="border rounded-lg overflow-hidden">
                <div className="bg-muted/50 px-4 py-2 border-b">
                  <span className="text-sm font-medium flex items-center gap-2">
                    <Activity className="w-4 h-4" />
                    Performance
                  </span>
                </div>
                <div className="p-4">
                  {isRunning ? (
                    <ResourceGraphs vmId={vm.id} vmName={vm.name} compact />
                  ) : (
                    <div className="text-center py-4 text-muted-foreground text-sm">
                      Start the VM to view performance metrics
                    </div>
                  )}
                </div>
              </div>
            </div>
          </TabsContent>

          {/* Network Tab */}
          <TabsContent value="network" className="m-0 h-full">
            <div className="p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium flex items-center gap-2">
                  <Network className="w-4 h-4" />
                  Network Interfaces
                </h3>
                <Button variant="outline" size="sm" onClick={() => setShowAddHardware(true)} className="h-7 text-xs gap-1.5">
                  <Plus className="w-3 h-3" />Add NIC
                </Button>
              </div>

              <div className="space-y-2">
                {vm.networkInterfaces?.map((_, index) => (
                  <div key={index} className="p-3 bg-muted/30 rounded-lg">
                    <h4 className="text-xs font-medium mb-2">NIC {index + 1}</h4>
                    <NetworkEditor vm={vm} nicIndex={index} compact />
                  </div>
                )) || (
                  <div className="p-3 bg-muted/30 rounded-lg">
                    <h4 className="text-xs font-medium mb-2">NIC 1</h4>
                    <NetworkEditor vm={vm} nicIndex={0} compact />
                  </div>
                )}
              </div>
            </div>
          </TabsContent>

          {/* Storage Tab */}
          <TabsContent value="storage" className="m-0 h-full">
            <div className="p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium flex items-center gap-2">
                  <HardDrive className="w-4 h-4" />
                  Storage Devices
                </h3>
                <Button variant="outline" size="sm" onClick={() => setShowAddHardware(true)} className="h-7 text-xs gap-1.5">
                  <Plus className="w-3 h-3" />Add Disk
                </Button>
              </div>

              <div className="space-y-2">
                {vm.disks?.map((disk, index) => (
                  <div key={index} className="p-3 bg-muted/30 rounded-lg">
                    <h4 className="text-xs font-medium mb-2">
                      Disk {index + 1} {disk.device && `(${disk.device})`}
                    </h4>
                    <DiskEditor vm={vm} diskIndex={index} compact />
                  </div>
                )) || (
                  <div className="p-3 bg-muted/30 rounded-lg">
                    <h4 className="text-xs font-medium mb-2">Disk 1</h4>
                    <DiskEditor vm={vm} diskIndex={0} compact />
                  </div>
                )}
              </div>
            </div>
          </TabsContent>

          {/* Graphics Tab */}
          <TabsContent value="graphics" className="m-0 h-full">
            <div className="p-4 space-y-3">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Tv className="w-4 h-4" />
                Display Configuration
              </h3>

              <div className="grid md:grid-cols-2 gap-4">
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">Graphics Server</h4>
                  <GraphicsEditor vm={vm} compact />
                </div>
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">Video Device</h4>
                  <VideoEditor vm={vm} compact />
                </div>
              </div>
            </div>
          </TabsContent>

          {/* CD/DVD Tab */}
          <TabsContent value="cdrom" className="m-0 h-full">
            <div className="p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium flex items-center gap-2">
                  <Disc className="w-4 h-4" />
                  CD/DVD Drives
                </h3>
                <Button variant="outline" size="sm" onClick={() => setShowAddHardware(true)} className="h-7 text-xs gap-1.5">
                  <Plus className="w-3 h-3" />Add CD/DVD
                </Button>
              </div>

              {vm.cdrom ? (
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">Optical Drive</h4>
                  <CdromEditor vm={vm} compact />
                </div>
              ) : (
                <div className="text-center py-8 text-muted-foreground text-sm border rounded-lg border-dashed">
                  No CD/DVD drive configured
                </div>
              )}
            </div>
          </TabsContent>

          {/* Additional Hardware Tab */}
          <TabsContent value="additional" className="m-0 h-full">
            <div className="p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium flex items-center gap-2">
                  <Keyboard className="w-4 h-4" />
                  Additional Hardware
                </h3>
                <Button variant="outline" size="sm" onClick={() => setShowAddHardware(true)} className="h-7 text-xs gap-1.5">
                  <Plus className="w-3 h-3" />Add Device
                </Button>
              </div>

              <div className="grid md:grid-cols-2 gap-4">
                {/* Sound */}
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">Sound</h4>
                  {vm.sound ? (
                    <SoundEditor vm={vm} compact />
                  ) : (
                    <p className="text-xs text-muted-foreground">Not configured</p>
                  )}
                </div>

                {/* Input Devices */}
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">Input Devices</h4>
                  <InputEditor vm={vm} compact />
                </div>

                {/* TPM */}
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">TPM</h4>
                  {vm.tpm ? (
                    <TpmEditor vm={vm} compact />
                  ) : (
                    <p className="text-xs text-muted-foreground">Not configured</p>
                  )}
                </div>

                {/* NUMA Configuration */}
                <div className="p-3 bg-muted/30 rounded-lg">
                  <h4 className="text-xs font-medium mb-2">NUMA</h4>
                  <NumaEditor vm={vm} />
                </div>
              </div>
            </div>
          </TabsContent>

          {/* Snapshots Tab */}
          <TabsContent value="snapshots" className="m-0 h-full">
            <div className="p-4">
              <SnapshotManager vmId={vm.id} vmName={vm.name} />
            </div>
          </TabsContent>
        </div>
      </Tabs>

      {/* Add Hardware Dialog */}
      <AddHardwareDialog
        vm={vm}
        open={showAddHardware}
        onOpenChange={setShowAddHardware}
      />
    </div>
  )
}