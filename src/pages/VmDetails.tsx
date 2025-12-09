import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo } from 'react'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { PageContainer, PageHeader, PageContent } from '@/components/layout/PageContainer'
import { Badge } from '@/components/ui/badge'
import { SnapshotManager } from '@/components/vm/SnapshotManager'
import { ResourceGraphs } from '@/components/vm/ResourceGraphs'
import { DiskManager } from '@/components/vm/DiskManager'
import { GuestInfo } from '@/components/vm/GuestInfo'
import { VncConsole } from '@/components/vm/VncConsole'
import { OptimizationSuggestions } from '@/components/vm/OptimizationSuggestions'
import { ArrowLeft, Cpu, HardDrive, Network, Settings, Download, Play, Square, Pause, PlayCircle, RotateCcw, Monitor } from 'lucide-react'

export function VmDetails() {
  const { vmId } = useParams<{ vmId: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const { data: vm, isLoading, error } = useQuery({
    queryKey: ['vm', vmId],
    queryFn: () => api.getVm(vmId!),
    enabled: !!vmId,
    refetchInterval: 5000,
  })

  // Poll live stats for running VMs
  const { data: vmStats } = useQuery({
    queryKey: ['vm-stats', vmId],
    queryFn: () => api.getVmStats(vmId!),
    enabled: !!vmId && vm?.state === 'running',
    refetchInterval: 2000, // Update every 2 seconds
  })

  // VM control mutations
  const startMutation = useMutation({
    mutationFn: () => api.startVm(vmId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success('VM started successfully')
    },
    onError: (error) => {
      const errorMsg = String(error)
      if (errorMsg.includes('Permission denied') || errorMsg.includes('Could not open')) {
        toast.error('Failed to start VM: Permission denied on disk/ISO file. Copy to /var/lib/libvirt/images/ or fix permissions.', { duration: 6000 })
      } else {
        toast.error(`Failed to start VM: ${error}`)
      }
    },
  })

  const stopMutation = useMutation({
    mutationFn: () => api.forceStopVm(vmId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success('VM stopped successfully')
    },
    onError: (error) => toast.error(`Failed to stop VM: ${error}`),
  })

  const pauseMutation = useMutation({
    mutationFn: () => api.pauseVm(vmId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success('VM paused successfully')
    },
    onError: (error) => toast.error(`Failed to pause VM: ${error}`),
  })

  const resumeMutation = useMutation({
    mutationFn: () => api.resumeVm(vmId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success('VM resumed successfully')
    },
    onError: (error) => toast.error(`Failed to resume VM: ${error}`),
  })

  const rebootMutation = useMutation({
    mutationFn: () => api.rebootVm(vmId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success('VM rebooting...')
    },
    onError: (error) => toast.error(`Failed to reboot VM: ${error}`),
  })

  const openVncMutation = useMutation({
    mutationFn: () => api.openVncConsole(vmId!),
    onSuccess: () => {
      toast.success('Opening VNC console...')
    },
    onError: (error) => toast.error(`Failed to open VNC console: ${error}`),
  })

  const handleBack = useCallback(() => {
    navigate('/vms')
  }, [navigate])

  const handleExport = useCallback(async () => {
    if (!vm) return
    try {
      const xml = await api.exportVm(vm.id)
      const blob = new Blob([xml], { type: 'application/xml' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${vm.name}.xml`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
      toast.success(`Exported ${vm.name} configuration`)
    } catch (error) {
      toast.error(`Failed to export VM: ${error}`)
    }
  }, [vm])

  // Helper function - defined early so useMemo can use it
  const getStateBadgeVariant = (state: string) => {
    switch (state) {
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

  const backButton = useMemo(() => (
    <Button variant="ghost" size="sm" onClick={handleBack}>
      <ArrowLeft className="mr-2 h-4 w-4" />
      Back
    </Button>
  ), [handleBack])

  // Define vmActions early (before conditionals) to satisfy Rules of Hooks
  // We'll use vm state when available, otherwise show just back button
  const vmActions = useMemo(() => {
    if (!vm) {
      return (
        <Button variant="ghost" size="sm" onClick={handleBack}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back
        </Button>
      )
    }

    const isRunning = vm.state === 'running'
    const isPaused = vm.state === 'paused'
    const isStopped = vm.state === 'stopped' || vm.state === 'shut off'

    return (
      <div className="flex items-center gap-2 flex-wrap">
        <Button variant="ghost" size="sm" onClick={handleBack}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back
        </Button>
        <Badge variant={getStateBadgeVariant(vm.state)}>
          {vm.state.toUpperCase()}
        </Badge>
        <div className="h-6 w-px bg-border mx-1" />

        {/* Start button - only when stopped */}
        {isStopped && (
          <Button
            size="sm"
            onClick={() => startMutation.mutate()}
            disabled={startMutation.isPending}
          >
            <Play className="mr-2 h-4 w-4" />
            Start
          </Button>
        )}

        {/* Pause/Resume buttons - only when running or paused */}
        {isRunning && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => pauseMutation.mutate()}
            disabled={pauseMutation.isPending}
          >
            <Pause className="mr-2 h-4 w-4" />
            Pause
          </Button>
        )}

        {isPaused && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => resumeMutation.mutate()}
            disabled={resumeMutation.isPending}
          >
            <PlayCircle className="mr-2 h-4 w-4" />
            Resume
          </Button>
        )}

        {/* Reboot button - only when running */}
        {isRunning && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => rebootMutation.mutate()}
            disabled={rebootMutation.isPending}
          >
            <RotateCcw className="mr-2 h-4 w-4" />
            Reboot
          </Button>
        )}

        {/* Stop button - only when running or paused */}
        {(isRunning || isPaused) && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => stopMutation.mutate()}
            disabled={stopMutation.isPending}
          >
            <Square className="mr-2 h-4 w-4" />
            Stop
          </Button>
        )}

        {/* VNC Console - only when running */}
        {isRunning && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => openVncMutation.mutate()}
            disabled={openVncMutation.isPending}
          >
            <Monitor className="mr-2 h-4 w-4" />
            Remote Control
          </Button>
        )}

        <div className="h-6 w-px bg-border mx-1" />

        <Button variant="outline" size="sm" onClick={handleExport}>
          <Download className="mr-2 h-4 w-4" />
          Export
        </Button>
      </div>
    )
  }, [vm, handleBack, handleExport, startMutation, stopMutation, pauseMutation, resumeMutation, rebootMutation, openVncMutation])

  if (isLoading) {
    return (
      <PageContainer>
        <PageHeader
          title="Loading..."
          actions={backButton}
        />
        <PageContent>
          <div className="flex items-center justify-center py-20">
            <p className="text-muted-foreground">Loading VM details...</p>
          </div>
        </PageContent>
      </PageContainer>
    )
  }

  if (error || !vm) {
    return (
      <PageContainer>
        <PageHeader
          title="Error"
          actions={backButton}
        />
        <PageContent>
          <Card className="border-border/40 shadow-sm">
            <CardHeader>
              <CardTitle>Failed to Load VM</CardTitle>
              <CardDescription>
                {error ? String(error) : 'VM not found'}
              </CardDescription>
            </CardHeader>
          </Card>
        </PageContent>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <PageHeader
        title={vm.name}
        description={vm.osType}
        actions={vmActions}
      />
      <PageContent>
        <div className="space-y-8">
          {/* VM Info Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <Card className="border-border/40 shadow-sm">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">CPU</CardTitle>
                <Cpu className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-light">{vm.cpuCount}</div>
                {vmStats && vm.state === 'running' ? (
                  <p className="text-xs text-muted-foreground mt-1">
                    {Math.round(vmStats.cpuUsagePercent)}% Usage
                  </p>
                ) : (
                  <p className="text-xs text-muted-foreground mt-1">vCPUs</p>
                )}
              </CardContent>
            </Card>

            <Card className="border-border/40 shadow-sm">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Memory</CardTitle>
                <Settings className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                {vmStats && vm.state === 'running' ? (
                  <>
                    <div className="text-3xl font-light">
                      {(vmStats.memoryUsedMb / 1024).toFixed(1)} GB
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      {Math.round((vmStats.memoryUsedMb / vmStats.memoryAvailableMb) * 100)}% of {(vmStats.memoryAvailableMb / 1024).toFixed(1)} GB
                    </p>
                  </>
                ) : (
                  <>
                    <div className="text-3xl font-light">{(vm.memoryMb / 1024).toFixed(1)} GB</div>
                    <p className="text-xs text-muted-foreground mt-1">{vm.memoryMb} MB</p>
                  </>
                )}
              </CardContent>
            </Card>

            <Card className="border-border/40 shadow-sm">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Disk</CardTitle>
                <HardDrive className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-light">{vm.diskSizeGb} GB</div>
                {vmStats && vm.state === 'running' ? (
                  <p className="text-xs text-muted-foreground mt-1">
                    R: {(vmStats.diskReadBytes / 1024 / 1024).toFixed(1)} MB/s | W: {(vmStats.diskWriteBytes / 1024 / 1024).toFixed(1)} MB/s
                  </p>
                ) : (
                  <p className="text-xs text-muted-foreground mt-1">Storage</p>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Network Interfaces */}
          {vm.networkInterfaces && vm.networkInterfaces.length > 0 && (
            <Card className="border-border/40 shadow-sm">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-lg font-semibold">
                  <Network className="h-5 w-5 text-muted-foreground" />
                  Network Interfaces
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {vm.networkInterfaces.map((nic, index) => (
                    <div key={index} className="flex items-center justify-between p-2 border rounded">
                      <div>
                        <p className="font-medium">{nic.network}</p>
                        <p className="text-sm text-muted-foreground">MAC: {nic.mac}</p>
                      </div>
                      {nic.ipAddress && (
                        <Badge variant="outline">{nic.ipAddress}</Badge>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Snapshot Manager */}
          <SnapshotManager vmId={vm.id} vmName={vm.name} />

          {/* Disk Manager */}
          <DiskManager vmId={vm.id} vmName={vm.name} disks={vm.disks} />

          {/* Guest Agent Information - Show for running VMs */}
          {vm.state === 'running' && (
            <GuestInfo vmId={vm.id} vmName={vm.name} />
          )}

          {/* VNC Console - Only show for running VMs */}
          {vm.state === 'running' && (
            <VncConsole vmId={vm.id} vmName={vm.name} />
          )}

          {/* Resource Monitoring - Only show for running VMs */}
          {vm.state === 'running' && (
            <ResourceGraphs vmId={vm.id} vmName={vm.name} />
          )}

          {/* Optimization Suggestions */}
          <OptimizationSuggestions vmId={vm.id} vmName={vm.name} />

          {/* Additional Info */}
          {vm.description && (
            <Card className="border-border/40 shadow-sm">
              <CardHeader>
                <CardTitle className="text-lg font-semibold">Description</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">{vm.description}</p>
              </CardContent>
            </Card>
          )}
        </div>
      </PageContent>
    </PageContainer>
  )
}
