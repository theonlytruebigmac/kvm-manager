import { useState, useEffect, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { VM, VmStats } from '@/lib/types'
import { PageContainer, PageHeader, PageContent } from '@/components/layout/PageContainer'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, Legend, AreaChart, Area
} from 'recharts'
import {
  Activity, Cpu, HardDrive, Network, AlertTriangle, CheckCircle,
  Settings2, Box
} from 'lucide-react'
import {
  Collapsible,
  CollapsibleContent,
} from "@/components/ui/collapsible"

interface AggregatedMetric {
  timestamp: number
  time: string
  totalCpu: number
  totalMemoryMb: number
  totalMemoryPercent: number
  totalDiskRead: number
  totalDiskWrite: number
  totalNetworkRx: number
  totalNetworkTx: number
  vmCount: number
}

interface VmPerformanceData {
  vm: VM
  stats: VmStats | null
  cpuAlert: boolean
  memoryAlert: boolean
}

interface AlertThresholds {
  cpuWarning: number
  cpuCritical: number
  memoryWarning: number
  memoryCritical: number
}

const DEFAULT_THRESHOLDS: AlertThresholds = {
  cpuWarning: 70,
  cpuCritical: 90,
  memoryWarning: 80,
  memoryCritical: 95,
}

const MAX_HISTORY_POINTS = 60

export function PerformanceMonitor() {
  const [thresholds, setThresholds] = useState<AlertThresholds>(DEFAULT_THRESHOLDS)
  const [showSettings, setShowSettings] = useState(false)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const [aggregatedHistory, setAggregatedHistory] = useState<AggregatedMetric[]>([])

  // Fetch all VMs
  const { data: vms, isLoading: vmsLoading } = useQuery({
    queryKey: ['vms'],
    queryFn: api.getVms,
    refetchInterval: autoRefresh ? 5000 : false,
  })

  // Get running VMs
  const runningVms = useMemo(() =>
    vms?.filter((vm: VM) => vm.state === 'running') || [],
    [vms]
  )

  // Fetch stats for all running VMs
  const { data: allVmStats } = useQuery({
    queryKey: ['all-vm-stats', runningVms.map((vm: VM) => vm.id)],
    queryFn: async () => {
      const stats: Record<string, VmStats | null> = {}
      await Promise.all(
        runningVms.map(async (vm: VM) => {
          try {
            stats[vm.id] = await api.getVmStats(vm.id)
          } catch {
            stats[vm.id] = null
          }
        })
      )
      return stats
    },
    refetchInterval: autoRefresh ? 1000 : false,
    enabled: runningVms.length > 0,
  })

  // Calculate per-VM data with alerts
  const vmPerformanceData = useMemo<VmPerformanceData[]>(() => {
    if (!runningVms || !allVmStats) return []

    return runningVms.map((vm: VM) => {
      const stats = allVmStats[vm.id] || null
      const cpuUsage = stats?.cpuUsagePercent || 0
      const memoryPercent = stats ? (stats.memoryUsedMb / stats.memoryAvailableMb) * 100 : 0

      return {
        vm,
        stats,
        cpuAlert: cpuUsage >= thresholds.cpuWarning,
        memoryAlert: memoryPercent >= thresholds.memoryWarning,
      }
    }).sort((a: VmPerformanceData, b: VmPerformanceData) => {
      // Sort by alerts first, then by CPU usage
      const aAlerts = (a.cpuAlert ? 1 : 0) + (a.memoryAlert ? 1 : 0)
      const bAlerts = (b.cpuAlert ? 1 : 0) + (b.memoryAlert ? 1 : 0)
      if (aAlerts !== bAlerts) return bAlerts - aAlerts
      return (b.stats?.cpuUsagePercent || 0) - (a.stats?.cpuUsagePercent || 0)
    })
  }, [runningVms, allVmStats, thresholds])

  // Calculate aggregated stats
  const aggregatedStats = useMemo(() => {
    if (!allVmStats || !runningVms.length) return null

    let totalCpu = 0
    let totalMemoryUsed = 0
    let totalMemoryAvailable = 0
    let totalDiskRead = 0
    let totalDiskWrite = 0
    let totalNetworkRx = 0
    let totalNetworkTx = 0
    let vmWithStats = 0

    for (const vm of runningVms as VM[]) {
      const stats = allVmStats[vm.id]
      if (stats) {
        totalCpu += stats.cpuUsagePercent
        totalMemoryUsed += stats.memoryUsedMb
        totalMemoryAvailable += stats.memoryAvailableMb
        totalDiskRead += stats.diskReadBytes
        totalDiskWrite += stats.diskWriteBytes
        totalNetworkRx += stats.networkRxBytes
        totalNetworkTx += stats.networkTxBytes
        vmWithStats++
      }
    }

    const avgCpu = vmWithStats > 0 ? totalCpu / vmWithStats : 0
    const memoryPercent = totalMemoryAvailable > 0
      ? (totalMemoryUsed / totalMemoryAvailable) * 100
      : 0

    return {
      avgCpu,
      totalCpu,
      totalMemoryUsed,
      totalMemoryAvailable,
      memoryPercent,
      totalDiskRead,
      totalDiskWrite,
      totalNetworkRx,
      totalNetworkTx,
      vmCount: vmWithStats,
    }
  }, [allVmStats, runningVms])

  // Update aggregated history
  useEffect(() => {
    if (!aggregatedStats) return

    const now = Date.now()
    const timeStr = new Date(now).toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })

    const newPoint: AggregatedMetric = {
      timestamp: now,
      time: timeStr,
      totalCpu: Math.round(aggregatedStats.avgCpu * 10) / 10,
      totalMemoryMb: Math.round(aggregatedStats.totalMemoryUsed),
      totalMemoryPercent: Math.round(aggregatedStats.memoryPercent * 10) / 10,
      totalDiskRead: Math.round(aggregatedStats.totalDiskRead / 1024 / 1024),
      totalDiskWrite: Math.round(aggregatedStats.totalDiskWrite / 1024 / 1024),
      totalNetworkRx: Math.round(aggregatedStats.totalNetworkRx / 1024 / 1024),
      totalNetworkTx: Math.round(aggregatedStats.totalNetworkTx / 1024 / 1024),
      vmCount: aggregatedStats.vmCount,
    }

    setAggregatedHistory(prev => [...prev, newPoint].slice(-MAX_HISTORY_POINTS))
  }, [aggregatedStats])

  // Count alerts
  const alertCounts = useMemo(() => {
    let cpuWarnings = 0
    let cpuCritical = 0
    let memWarnings = 0
    let memCritical = 0

    for (const data of vmPerformanceData) {
      if (!data.stats) continue
      const cpuUsage = data.stats.cpuUsagePercent
      const memPercent = (data.stats.memoryUsedMb / data.stats.memoryAvailableMb) * 100

      if (cpuUsage >= thresholds.cpuCritical) cpuCritical++
      else if (cpuUsage >= thresholds.cpuWarning) cpuWarnings++

      if (memPercent >= thresholds.memoryCritical) memCritical++
      else if (memPercent >= thresholds.memoryWarning) memWarnings++
    }

    return { cpuWarnings, cpuCritical, memWarnings, memCritical }
  }, [vmPerformanceData, thresholds])

  const formatBytes = (mb: number) => {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`
    return `${mb.toFixed(0)} MB`
  }

  const getStatusColor = (value: number, warning: number, critical: number) => {
    if (value >= critical) return 'text-red-500'
    if (value >= warning) return 'text-yellow-500'
    return 'text-green-500'
  }

  const getProgressColor = (value: number, warning: number, critical: number) => {
    if (value >= critical) return 'bg-red-500'
    if (value >= warning) return 'bg-yellow-500'
    return 'bg-green-500'
  }

  if (vmsLoading) {
    return (
      <PageContainer>
        <PageHeader
          title="Performance Monitor"
          description="Real-time system performance monitoring"
        />
        <PageContent>
          <div className="space-y-6">
            {[1, 2, 3].map(i => (
              <Skeleton key={i} className="h-48 w-full" />
            ))}
          </div>
        </PageContent>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <PageHeader
        title="Performance Monitor"
        description="Real-time system performance monitoring with alerts"
        actions={
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <Switch
                id="auto-refresh"
                checked={autoRefresh}
                onCheckedChange={setAutoRefresh}
              />
              <Label htmlFor="auto-refresh" className="text-sm">Auto-refresh</Label>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowSettings(!showSettings)}
            >
              <Settings2 className="w-4 h-4 mr-1" />
              Thresholds
            </Button>
          </div>
        }
      />

      <PageContent>
        <div className="space-y-6">
          {/* Alert Threshold Settings */}
          <Collapsible open={showSettings} onOpenChange={setShowSettings}>
            <CollapsibleContent>
              <Card className="mb-6">
                <CardHeader>
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Settings2 className="w-4 h-4" />
                    Alert Thresholds
                  </CardTitle>
                  <CardDescription>Configure warning and critical thresholds for alerts</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-wrap gap-6">
                    <div className="flex items-center gap-2 text-sm">
                      <Cpu className="w-4 h-4 text-blue-500" />
                      <span className="text-muted-foreground">CPU:</span>
                      <Input type="number" value={thresholds.cpuWarning} onChange={(e) => setThresholds((t: AlertThresholds) => ({ ...t, cpuWarning: parseInt(e.target.value) || 0 }))} min={0} max={100} className="w-16 h-7 text-center" />
                      <span className="text-yellow-500">warn</span>
                      <Input type="number" value={thresholds.cpuCritical} onChange={(e) => setThresholds((t: AlertThresholds) => ({ ...t, cpuCritical: parseInt(e.target.value) || 0 }))} min={0} max={100} className="w-16 h-7 text-center" />
                      <span className="text-red-500">crit</span>
                    </div>
                    <div className="flex items-center gap-2 text-sm">
                      <Activity className="w-4 h-4 text-green-500" />
                      <span className="text-muted-foreground">Memory:</span>
                      <Input type="number" value={thresholds.memoryWarning} onChange={(e) => setThresholds((t: AlertThresholds) => ({ ...t, memoryWarning: parseInt(e.target.value) || 0 }))} min={0} max={100} className="w-16 h-7 text-center" />
                      <span className="text-yellow-500">warn</span>
                      <Input type="number" value={thresholds.memoryCritical} onChange={(e) => setThresholds((t: AlertThresholds) => ({ ...t, memoryCritical: parseInt(e.target.value) || 0 }))} min={0} max={100} className="w-16 h-7 text-center" />
                      <span className="text-red-500">crit</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </CollapsibleContent>
          </Collapsible>

          {/* Status Summary - Single compact line */}
          {runningVms.length === 0 ? (
            <div className="text-sm text-muted-foreground py-2">
              No running VMs to monitor. Start a VM to see performance data.
            </div>
          ) : (
            <div className="flex items-center gap-4 text-sm">
              {alertCounts.cpuCritical > 0 || alertCounts.memCritical > 0 ? (
                <div className="flex items-center gap-2 text-red-500">
                  <AlertTriangle className="w-4 h-4" />
                  <span>{alertCounts.cpuCritical + alertCounts.memCritical} critical</span>
                </div>
              ) : alertCounts.cpuWarnings > 0 || alertCounts.memWarnings > 0 ? (
                <div className="flex items-center gap-2 text-yellow-500">
                  <AlertTriangle className="w-4 h-4" />
                  <span>{alertCounts.cpuWarnings + alertCounts.memWarnings} warnings</span>
                </div>
              ) : (
                <div className="flex items-center gap-2 text-green-500">
                  <CheckCircle className="w-4 h-4" />
                  <span>All {runningVms.length} VMs normal</span>
                </div>
              )}
            </div>
          )}

          {/* System Overview Cards */}
          {aggregatedStats && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {/* Total VMs */}
              <Card>
                <CardContent className="pt-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-blue-500/10">
                      <Box className="w-5 h-5 text-blue-500" />
                    </div>
                    <div>
                      <div className="text-2xl font-bold">{runningVms.length}</div>
                      <div className="text-xs text-muted-foreground">Running VMs</div>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Average CPU */}
              <Card>
                <CardContent className="pt-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-blue-500/10">
                      <Cpu className="w-5 h-5 text-blue-500" />
                    </div>
                    <div>
                      <div className={`text-2xl font-bold ${getStatusColor(aggregatedStats.avgCpu, thresholds.cpuWarning, thresholds.cpuCritical)}`}>
                        {aggregatedStats.avgCpu.toFixed(1)}%
                      </div>
                      <div className="text-xs text-muted-foreground">Avg CPU Usage</div>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Total Memory */}
              <Card>
                <CardContent className="pt-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-green-500/10">
                      <Activity className="w-5 h-5 text-green-500" />
                    </div>
                    <div>
                      <div className={`text-2xl font-bold ${getStatusColor(aggregatedStats.memoryPercent, thresholds.memoryWarning, thresholds.memoryCritical)}`}>
                        {formatBytes(aggregatedStats.totalMemoryUsed)}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        of {formatBytes(aggregatedStats.totalMemoryAvailable)} ({aggregatedStats.memoryPercent.toFixed(0)}%)
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Total I/O */}
              <Card>
                <CardContent className="pt-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-purple-500/10">
                      <Network className="w-5 h-5 text-purple-500" />
                    </div>
                    <div>
                      <div className="text-sm">
                        <span className="text-purple-500 font-semibold">↓{formatBytes(aggregatedStats.totalNetworkRx / 1024 / 1024)}</span>
                        {' '}
                        <span className="text-pink-500 font-semibold">↑{formatBytes(aggregatedStats.totalNetworkTx / 1024 / 1024)}</span>
                      </div>
                      <div className="text-xs text-muted-foreground">Network I/O</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Aggregated Performance Graphs */}
          {aggregatedHistory.length > 0 && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              {/* CPU Graph */}
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Cpu className="w-4 h-4 text-blue-500" />
                    Average CPU Usage (All VMs)
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={180}>
                    <AreaChart data={aggregatedHistory}>
                      <defs>
                        <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
                          <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                        </linearGradient>
                      </defs>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                      <XAxis dataKey="time" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
                      <YAxis domain={[0, 100]} tick={{ fontSize: 10 }} unit="%" />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'hsl(var(--background))',
                          border: '1px solid hsl(var(--border))'
                        }}
                        formatter={(value: number) => [`${value.toFixed(1)}%`, 'Avg CPU']}
                      />
                      <Area
                        type="monotone"
                        dataKey="totalCpu"
                        stroke="#3b82f6"
                        strokeWidth={2}
                        fill="url(#cpuGradient)"
                        isAnimationActive={false}
                      />
                    </AreaChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              {/* Memory Graph */}
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Activity className="w-4 h-4 text-green-500" />
                    Total Memory Usage (All VMs)
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={180}>
                    <AreaChart data={aggregatedHistory}>
                      <defs>
                        <linearGradient id="memGradient" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="5%" stopColor="#22c55e" stopOpacity={0.3}/>
                          <stop offset="95%" stopColor="#22c55e" stopOpacity={0}/>
                        </linearGradient>
                      </defs>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                      <XAxis dataKey="time" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
                      <YAxis domain={[0, 100]} tick={{ fontSize: 10 }} unit="%" />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'hsl(var(--background))',
                          border: '1px solid hsl(var(--border))'
                        }}
                        formatter={(value: number) => [`${value.toFixed(1)}%`, 'Memory']}
                      />
                      <Area
                        type="monotone"
                        dataKey="totalMemoryPercent"
                        stroke="#22c55e"
                        strokeWidth={2}
                        fill="url(#memGradient)"
                        isAnimationActive={false}
                      />
                    </AreaChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              {/* Network I/O Graph */}
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Network className="w-4 h-4 text-purple-500" />
                    Network I/O (All VMs)
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={180}>
                    <LineChart data={aggregatedHistory}>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                      <XAxis dataKey="time" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
                      <YAxis tick={{ fontSize: 10 }} unit=" MB" />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'hsl(var(--background))',
                          border: '1px solid hsl(var(--border))'
                        }}
                        formatter={(value: number) => [`${value} MB`]}
                      />
                      <Legend />
                      <Line
                        type="monotone"
                        dataKey="totalNetworkRx"
                        stroke="#a855f7"
                        strokeWidth={2}
                        dot={false}
                        name="Download"
                        isAnimationActive={false}
                      />
                      <Line
                        type="monotone"
                        dataKey="totalNetworkTx"
                        stroke="#ec4899"
                        strokeWidth={2}
                        dot={false}
                        name="Upload"
                        isAnimationActive={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              {/* Disk I/O Graph */}
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <HardDrive className="w-4 h-4 text-orange-500" />
                    Disk I/O (All VMs)
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={180}>
                    <LineChart data={aggregatedHistory}>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                      <XAxis dataKey="time" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
                      <YAxis tick={{ fontSize: 10 }} unit=" MB" />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'hsl(var(--background))',
                          border: '1px solid hsl(var(--border))'
                        }}
                        formatter={(value: number) => [`${value} MB`]}
                      />
                      <Legend />
                      <Line
                        type="monotone"
                        dataKey="totalDiskRead"
                        stroke="#f97316"
                        strokeWidth={2}
                        dot={false}
                        name="Read"
                        isAnimationActive={false}
                      />
                      <Line
                        type="monotone"
                        dataKey="totalDiskWrite"
                        stroke="#fbbf24"
                        strokeWidth={2}
                        dot={false}
                        name="Write"
                        isAnimationActive={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Per-VM Performance Cards */}
          {vmPerformanceData.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-sm font-medium text-muted-foreground">Per-VM Performance</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {vmPerformanceData.map(({ vm, stats, cpuAlert, memoryAlert }) => {
                  const cpuUsage = stats?.cpuUsagePercent || 0
                  const memoryPercent = stats
                    ? (stats.memoryUsedMb / stats.memoryAvailableMb) * 100
                    : 0

                  return (
                    <Card
                      key={vm.id}
                      className={`${
                        cpuUsage >= thresholds.cpuCritical || memoryPercent >= thresholds.memoryCritical
                          ? 'border-red-500/50'
                          : cpuAlert || memoryAlert
                          ? 'border-yellow-500/50'
                          : ''
                      }`}
                    >
                      <CardHeader className="pb-2">
                        <div className="flex items-center justify-between">
                          <CardTitle className="text-sm font-medium">{vm.name}</CardTitle>
                          <div className="flex gap-1">
                            {cpuAlert && (
                              <Badge
                                variant={cpuUsage >= thresholds.cpuCritical ? "destructive" : "outline"}
                                className="text-xs"
                              >
                                CPU
                              </Badge>
                            )}
                            {memoryAlert && (
                              <Badge
                                variant={memoryPercent >= thresholds.memoryCritical ? "destructive" : "outline"}
                                className="text-xs"
                              >
                                MEM
                              </Badge>
                            )}
                          </div>
                        </div>
                        <CardDescription className="text-xs">
                          {vm.cpuCount} vCPU • {formatBytes(vm.memoryMb || 0)}
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="space-y-3">
                        {/* CPU */}
                        <div className="space-y-1">
                          <div className="flex justify-between text-xs">
                            <span className="flex items-center gap-1">
                              <Cpu className="w-3 h-3" /> CPU
                            </span>
                            <span className={getStatusColor(cpuUsage, thresholds.cpuWarning, thresholds.cpuCritical)}>
                              {cpuUsage.toFixed(1)}%
                            </span>
                          </div>
                          <Progress
                            value={cpuUsage}
                            className={`h-1.5 ${getProgressColor(cpuUsage, thresholds.cpuWarning, thresholds.cpuCritical)}`}
                          />
                        </div>

                        {/* Memory */}
                        <div className="space-y-1">
                          <div className="flex justify-between text-xs">
                            <span className="flex items-center gap-1">
                              <Activity className="w-3 h-3" /> Memory
                            </span>
                            <span className={getStatusColor(memoryPercent, thresholds.memoryWarning, thresholds.memoryCritical)}>
                              {stats ? `${formatBytes(stats.memoryUsedMb)} (${memoryPercent.toFixed(0)}%)` : 'N/A'}
                            </span>
                          </div>
                          <Progress
                            value={memoryPercent}
                            className={`h-1.5 ${getProgressColor(memoryPercent, thresholds.memoryWarning, thresholds.memoryCritical)}`}
                          />
                        </div>

                        {/* Network & Disk */}
                        {stats && (
                          <div className="flex justify-between text-xs text-muted-foreground pt-1 border-t">
                            <span className="flex items-center gap-1">
                              <Network className="w-3 h-3" />
                              ↓{formatBytes(stats.networkRxBytes / 1024 / 1024)} ↑{formatBytes(stats.networkTxBytes / 1024 / 1024)}
                            </span>
                            <span className="flex items-center gap-1">
                              <HardDrive className="w-3 h-3" />
                              R{formatBytes(stats.diskReadBytes / 1024 / 1024)} W{formatBytes(stats.diskWriteBytes / 1024 / 1024)}
                            </span>
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  )
                })}
              </div>
            </div>
          )}
        </div>
      </PageContent>
    </PageContainer>
  )
}
