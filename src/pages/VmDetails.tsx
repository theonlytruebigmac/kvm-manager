import { useParams, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { SnapshotManager } from '@/components/vm/SnapshotManager'
import { ResourceGraphs } from '@/components/vm/ResourceGraphs'
import { ArrowLeft, Cpu, HardDrive, Network, Settings } from 'lucide-react'

export function VmDetails() {
  const { vmId } = useParams<{ vmId: string }>()
  const navigate = useNavigate()

  const { data: vm, isLoading, error } = useQuery({
    queryKey: ['vm', vmId],
    queryFn: () => api.getVm(vmId!),
    enabled: !!vmId,
    refetchInterval: 5000,
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-muted-foreground">Loading VM details...</p>
      </div>
    )
  }

  if (error || !vm) {
    return (
      <div className="space-y-4">
        <Button variant="ghost" onClick={() => navigate('/vms')}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to VMs
        </Button>
        <Card>
          <CardHeader>
            <CardTitle>Error</CardTitle>
            <CardDescription>Failed to load VM details: {String(error)}</CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

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

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" onClick={() => navigate('/vms')}>
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to VMs
          </Button>
          <div>
            <h1 className="text-3xl font-bold">{vm.name}</h1>
            <div className="flex items-center gap-2 mt-1">
              <Badge variant={getStateBadgeVariant(vm.state)}>
                {vm.state.toUpperCase()}
              </Badge>
              {vm.osType && (
                <span className="text-sm text-muted-foreground">
                  {vm.osType}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* VM Info Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{vm.cpuCount}</div>
            <p className="text-xs text-muted-foreground">vCPUs</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory</CardTitle>
            <Settings className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{(vm.memoryMb / 1024).toFixed(1)} GB</div>
            <p className="text-xs text-muted-foreground">{vm.memoryMb} MB</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Disk</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{vm.diskSizeGb} GB</div>
            <p className="text-xs text-muted-foreground">Storage</p>
          </CardContent>
        </Card>
      </div>

      {/* Network Interfaces */}
      {vm.networkInterfaces && vm.networkInterfaces.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Network className="h-5 w-5" />
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

      {/* Resource Monitoring - Only show for running VMs */}
      {vm.state === 'running' && (
        <ResourceGraphs vmId={vm.id} vmName={vm.name} />
      )}

      {/* Snapshot Manager */}
      <SnapshotManager vmId={vm.id} vmName={vm.name} />

      {/* Additional Info */}
      {vm.description && (
        <Card>
          <CardHeader>
            <CardTitle>Description</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">{vm.description}</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
