import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { PageContainer, PageHeader, PageContent } from '@/components/layout/PageContainer'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { ErrorState, EmptyState } from '@/components/ui/error-state'
import { AlertCircle, Server, Cpu, HardDrive, Activity, Box } from 'lucide-react'

export function Dashboard() {
  const { data: hostInfo, isLoading, error, refetch } = useQuery({
    queryKey: ['hostInfo'],
    queryFn: api.getHostInfo,
    refetchInterval: 10000, // Poll every 10 seconds
  })

  if (isLoading) {
    return (
      <PageContainer>
        <PageHeader
          title="Dashboard"
          description="System overview and resource monitoring"
        />
        <PageContent>
          <div className="space-y-6">
            {/* System Overview Skeleton */}
            <Card className="border-[var(--panel-border)] shadow-sm bg-[var(--panel-bg)]">
              <CardHeader className="py-3 px-4">
                <div className="flex items-center gap-2">
                  <Skeleton className="h-4 w-4 rounded" />
                  <Skeleton className="h-5 w-32" />
                </div>
              </CardHeader>
              <CardContent className="px-4 pb-3">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  {[1, 2, 3].map(i => (
                    <div key={i} className="space-y-2">
                      <Skeleton className="h-3 w-24" />
                      <Skeleton className="h-4 w-32" />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
            {/* Resource Usage Skeletons */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {[1, 2, 3, 4].map(i => (
                <Card key={i} className="border-[var(--panel-border)] shadow-sm bg-[var(--panel-bg)]">
                  <CardHeader className="py-3 px-4">
                    <div className="flex items-center gap-2">
                      <Skeleton className="h-4 w-4 rounded" />
                      <Skeleton className="h-5 w-24" />
                    </div>
                  </CardHeader>
                  <CardContent className="px-4 pb-3 space-y-3">
                    <Skeleton className="h-4 w-full" />
                    <Skeleton className="h-2 w-full" />
                    <div className="flex justify-between">
                      <Skeleton className="h-3 w-16" />
                      <Skeleton className="h-3 w-16" />
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </PageContent>
      </PageContainer>
    )
  }

  if (error) {
    const errorMsg = String(error)
    let suggestion = 'Make sure libvirt daemon is running and you have proper permissions.'

    if (errorMsg.includes('Permission denied')) {
      suggestion = 'Check that your user is in the "libvirt" group. Run: sudo usermod -aG libvirt $USER, then log out and back in.'
    } else if (errorMsg.includes('Connection refused')) {
      suggestion = 'The libvirt daemon may not be running. Try: sudo systemctl start libvirtd'
    }

    return (
      <PageContainer>
        <PageHeader
          title="Dashboard"
          description="System overview and resource monitoring"
        />
        <PageContent>
          <ErrorState
            title="Cannot Connect to Libvirt"
            message={errorMsg}
            suggestion={suggestion}
            onRetry={() => refetch()}
          />
        </PageContent>
      </PageContainer>
    )
  }

  if (!hostInfo) {
    return (
      <PageContainer>
        <PageHeader
          title="Dashboard"
          description="System overview and resource monitoring"
        />
        <PageContent>
          <EmptyState
            icon={<AlertCircle className="w-12 h-12" />}
            title="No Host Information"
            description="Unable to retrieve host information from libvirt."
          />
        </PageContent>
      </PageContainer>
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
    <PageContainer>
      <PageHeader
        title="Dashboard"
        description="System overview and resource monitoring"
      />
      <PageContent>
        <div className="space-y-6">
          {/* System Overview Card */}
          <Card className="border-[var(--panel-border)] shadow-sm bg-[var(--panel-bg)]">
        <CardHeader className="py-3 px-4">
          <CardTitle className="flex items-center gap-2 text-desktop-lg font-semibold">
            <Server className="w-4 h-4 text-muted-foreground" />
            System Overview
          </CardTitle>
        </CardHeader>
        <CardContent className="px-4 pb-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2.5">
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Hostname:</span>
                <span className="text-desktop-sm">{hostInfo.hostname}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Hypervisor:</span>
                <Badge variant="outline" className="border-[var(--panel-border)] h-5 text-desktop-xs">{hostInfo.hypervisor}</Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Libvirt Version:</span>
                <span className="text-desktop-sm">{hostInfo.libvirtVersion}</span>
              </div>
            </div>
            <div className="space-y-2.5">
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">QEMU Version:</span>
                <span className="text-desktop-sm">{hostInfo.qemuVersion}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">CPU Model:</span>
                <span className="text-desktop-sm truncate max-w-[200px]" title={hostInfo.cpuModel}>
                  {hostInfo.cpuModel}
                </span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Resource Usage Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* CPU Card */}
        <Card className="border-[var(--panel-border)] shadow-sm bg-[var(--panel-bg)]">
          <CardHeader className="py-3 px-4">
            <CardTitle className="flex items-center gap-2 text-desktop-lg font-semibold">
              <Cpu className="w-4 h-4 text-muted-foreground" />
              CPU Resources
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3 space-y-3">
            <div className="space-y-2.5">
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Physical Cores:</span>
                <span className="text-2xl font-light">{hostInfo.cpuCount}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Total Threads:</span>
                <span className="text-2xl font-light">{hostInfo.cpuThreads}</span>
              </div>
            </div>
            <div className="pt-3 border-t border-[var(--panel-border)]">
              <div className="text-desktop-xs text-muted-foreground">
                {hostInfo.cpuModel}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Memory Card */}
        <Card className="border-[var(--panel-border)] shadow-sm bg-[var(--panel-bg)]">
          <CardHeader className="py-3 px-4">
            <CardTitle className="flex items-center gap-2 text-desktop-lg font-semibold">
              <HardDrive className="w-4 h-4 text-muted-foreground" />
              Memory Resources
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3 space-y-3">
            <div className="space-y-2.5">
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Total Memory:</span>
                <span className="text-2xl font-light">{formatMemory(hostInfo.memoryTotalMb)}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Used:</span>
                <span className="text-desktop-sm">{formatMemory(memoryUsedMb)}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-desktop-sm text-muted-foreground">Free:</span>
                <span className="text-desktop-sm">{formatMemory(hostInfo.memoryFreeMb)}</span>
              </div>
            </div>
            <div className="space-y-2 pt-2">
              <div className="flex justify-between text-desktop-sm">
                <span className="text-muted-foreground">Usage:</span>
                <span className="text-desktop-sm">{memoryUsagePercent.toFixed(1)}%</span>
              </div>
              <Progress value={memoryUsagePercent} className="h-2" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* VM Statistics Card */}
      <Card className="border-[var(--panel-border)] shadow-sm bg-[var(--panel-bg)]">
        <CardHeader className="py-3 px-4">
          <CardTitle className="flex items-center gap-2 text-desktop-lg font-semibold">
            <Activity className="w-4 h-4 text-muted-foreground" />
            Virtual Machine Statistics
          </CardTitle>
        </CardHeader>
        <CardContent className="px-4 pb-3">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="flex flex-col items-center justify-center py-6">
              <Box className="w-6 h-6 mb-2 text-muted-foreground" />
              <div className="text-3xl font-light mb-1">{hostInfo.totalVms}</div>
              <div className="text-desktop-sm text-muted-foreground">Total VMs</div>
            </div>
            <div className="flex flex-col items-center justify-center py-6 border-x border-[var(--panel-border)]">
              <Activity className="w-6 h-6 mb-2 text-green-500" />
              <div className="text-3xl font-light text-green-500 mb-1">{hostInfo.activeVms}</div>
              <div className="text-desktop-sm text-muted-foreground">Active VMs</div>
            </div>
            <div className="flex flex-col items-center justify-center py-6">
              <Box className="w-6 h-6 mb-2 text-muted-foreground" />
              <div className="text-3xl font-light mb-1">{hostInfo.totalVms - hostInfo.activeVms}</div>
              <div className="text-desktop-sm text-muted-foreground">Inactive VMs</div>
            </div>
          </div>
        </CardContent>
      </Card>
        </div>
      </PageContent>
    </PageContainer>
  )
}
