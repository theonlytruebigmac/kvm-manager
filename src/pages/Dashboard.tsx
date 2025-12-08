import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { AlertCircle, Loader2, Server, Cpu, HardDrive, Activity, Box } from 'lucide-react'

export function Dashboard() {
  const { data: hostInfo, isLoading, error } = useQuery({
    queryKey: ['hostInfo'],
    queryFn: api.getHostInfo,
    refetchInterval: 10000, // Poll every 10 seconds
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="w-8 h-8 animate-spin text-primary" />
          <p className="text-sm text-muted-foreground">Loading host information...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="flex flex-col items-center gap-3">
          <AlertCircle className="w-8 h-8 text-destructive" />
          <div className="text-center">
            <p className="font-medium">Error loading host information</p>
            <p className="text-sm text-muted-foreground mt-1">{String(error)}</p>
          </div>
        </div>
      </div>
    )
  }

  if (!hostInfo) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="flex flex-col items-center gap-3">
          <AlertCircle className="w-8 h-8 text-muted-foreground" />
          <p className="text-sm text-muted-foreground">No host information available</p>
        </div>
      </div>
    )
  }

  // Calculate memory usage percentage
  const memoryUsedMb = hostInfo.memoryTotalMb - hostInfo.memoryFreeMb
  const memoryUsagePercent = (memoryUsedMb / hostInfo.memoryTotalMb) * 100

  // Format memory values to GB if > 1024 MB
  const formatMemory = (mb: number) => {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)} GB`
    }
    return `${mb} MB`
  }

  return (
    <div className="space-y-6">
      {/* System Overview Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Server className="w-5 h-5" />
            System Overview
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Hostname:</span>
                <span className="font-medium">{hostInfo.hostname}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Hypervisor:</span>
                <Badge variant="outline">{hostInfo.hypervisor}</Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Libvirt Version:</span>
                <span className="font-medium">{hostInfo.libvirtVersion}</span>
              </div>
            </div>
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">QEMU Version:</span>
                <span className="font-medium">{hostInfo.qemuVersion}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">CPU Model:</span>
                <span className="font-medium text-sm truncate max-w-[200px]" title={hostInfo.cpuModel}>
                  {hostInfo.cpuModel}
                </span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Resource Usage Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* CPU Card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Cpu className="w-5 h-5" />
              CPU Resources
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Physical Cores:</span>
                <span className="font-medium text-lg">{hostInfo.cpuCount}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Total Threads:</span>
                <span className="font-medium text-lg">{hostInfo.cpuThreads}</span>
              </div>
            </div>
            <div className="pt-4 border-t">
              <div className="text-xs text-muted-foreground mb-2">
                {hostInfo.cpuModel}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Memory Card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <HardDrive className="w-5 h-5" />
              Memory Resources
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Total Memory:</span>
                <span className="font-medium text-lg">{formatMemory(hostInfo.memoryTotalMb)}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Used:</span>
                <span className="font-medium">{formatMemory(memoryUsedMb)}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Free:</span>
                <span className="font-medium">{formatMemory(hostInfo.memoryFreeMb)}</span>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Usage:</span>
                <span className="font-medium">{memoryUsagePercent.toFixed(1)}%</span>
              </div>
              <Progress value={memoryUsagePercent} className="h-3" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* VM Statistics Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="w-5 h-5" />
            Virtual Machine Statistics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="flex flex-col items-center justify-center p-6 bg-muted/50 rounded-lg">
              <Box className="w-8 h-8 mb-2 text-primary" />
              <div className="text-3xl font-bold">{hostInfo.totalVms}</div>
              <div className="text-sm text-muted-foreground">Total VMs</div>
            </div>
            <div className="flex flex-col items-center justify-center p-6 bg-green-500/10 rounded-lg">
              <Activity className="w-8 h-8 mb-2 text-green-500" />
              <div className="text-3xl font-bold text-green-500">{hostInfo.activeVms}</div>
              <div className="text-sm text-muted-foreground">Active VMs</div>
            </div>
            <div className="flex flex-col items-center justify-center p-6 bg-muted/50 rounded-lg">
              <Box className="w-8 h-8 mb-2 text-muted-foreground" />
              <div className="text-3xl font-bold">{hostInfo.totalVms - hostInfo.activeVms}</div>
              <div className="text-sm text-muted-foreground">Inactive VMs</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
