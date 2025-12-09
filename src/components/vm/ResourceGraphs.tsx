import { useEffect, useState } from 'react'
import { useQuery, useMutation } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { VmMetrics } from '@/lib/types'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts'
import { Activity, HardDrive, Network, Cpu, Clock, Calendar } from 'lucide-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'

interface ResourceGraphsProps {
  vmId: string
  vmName: string
}

interface MetricPoint {
  timestamp: number
  time: string
  cpuUsage: number
  memoryUsage: number
  memoryPercent: number
  diskRead: number
  diskWrite: number
  networkRx: number
  networkTx: number
}

type TimeRange = 'live' | '1h' | '6h' | '24h' | '7d' | '30d'

const MAX_POINTS = 60 // Keep last 60 data points (1 minute at 1s interval)

export function ResourceGraphs({ vmId }: ResourceGraphsProps) {
  const [metrics, setMetrics] = useState<MetricPoint[]>([])
  const [timeRange, setTimeRange] = useState<TimeRange>('live')

  // Poll VM stats every second (only for live view)
  const { data: stats } = useQuery({
    queryKey: ['vm-stats', vmId],
    queryFn: () => api.getVmStats(vmId),
    refetchInterval: 1000,
    enabled: timeRange === 'live',
  })

  // Store metrics mutation
  const storeMetrics = useMutation({
    mutationFn: (metrics: VmMetrics) => api.storeVmMetrics(metrics),
  })

  // Fetch historical metrics
  const { data: historical, isLoading: historyLoading } = useQuery({
    queryKey: ['historical-metrics', vmId, timeRange],
    queryFn: async () => {
      if (timeRange === 'live') return null

      const now = Date.now()
      const ranges = {
        '1h': 60 * 60 * 1000,
        '6h': 6 * 60 * 60 * 1000,
        '24h': 24 * 60 * 60 * 1000,
        '7d': 7 * 24 * 60 * 60 * 1000,
        '30d': 30 * 24 * 60 * 60 * 1000,
      }
      const startTime = now - ranges[timeRange as keyof typeof ranges]

      // Limit points for performance
      const maxPoints = timeRange === '1h' ? 60 : timeRange === '6h' ? 72 : 100

      return await api.getHistoricalMetrics(vmId, startTime, now, maxPoints)
    },
    enabled: timeRange !== 'live',
    staleTime: 30000, // Cache for 30s
  })

  // Update metrics history when new stats arrive (live mode)
  useEffect(() => {
    if (!stats || timeRange !== 'live') return

    const now = Date.now()
    const timeStr = new Date(now).toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })

    const newPoint: MetricPoint = {
      timestamp: now,
      time: timeStr,
      cpuUsage: Math.round(stats.cpuUsagePercent * 10) / 10,
      memoryUsage: Math.round(stats.memoryUsedMb),
      memoryPercent: Math.round((stats.memoryUsedMb / stats.memoryAvailableMb) * 100 * 10) / 10,
      diskRead: Math.round(stats.diskReadBytes / 1024 / 1024), // Convert to MB
      diskWrite: Math.round(stats.diskWriteBytes / 1024 / 1024),
      networkRx: Math.round(stats.networkRxBytes / 1024 / 1024),
      networkTx: Math.round(stats.networkTxBytes / 1024 / 1024),
    }

    setMetrics((prev) => {
      const updated = [...prev, newPoint]
      // Keep only last MAX_POINTS
      return updated.slice(-MAX_POINTS)
    })

    // Store to database every 30 seconds
    if (Math.floor(now / 1000) % 30 === 0) {
      const vmMetrics: VmMetrics = {
        vmId,
        timestamp: now,
        cpuUsage: stats.cpuUsagePercent,
        memoryUsageMb: stats.memoryUsedMb,
        memoryTotalMb: stats.memoryAvailableMb,
        diskReadBytesPerSec: stats.diskReadBytes,
        diskWriteBytesPerSec: stats.diskWriteBytes,
        networkRxBytesPerSec: stats.networkRxBytes,
        networkTxBytesPerSec: stats.networkTxBytes,
      }
      storeMetrics.mutate(vmMetrics)
    }
  }, [stats, vmId, timeRange, storeMetrics])

  // Convert historical data to chart format
  useEffect(() => {
    if (!historical || timeRange === 'live') return

    const formatTime = (timestamp: number) => {
      const date = new Date(timestamp)
      if (timeRange === '7d' || timeRange === '30d') {
        return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
      } else if (timeRange === '24h') {
        return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })
      } else {
        return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
      }
    }

    const points: MetricPoint[] = historical.cpuUsage.map((cpu, idx) => ({
      timestamp: cpu.timestamp,
      time: formatTime(cpu.timestamp),
      cpuUsage: Math.round(cpu.value * 10) / 10,
      memoryUsage: Math.round(historical.memoryUsage[idx]?.value || 0),
      memoryPercent: 0, // Calculate if needed
      diskRead: Math.round((historical.diskRead[idx]?.value || 0) / 1024 / 1024),
      diskWrite: Math.round((historical.diskWrite[idx]?.value || 0) / 1024 / 1024),
      networkRx: Math.round((historical.networkRx[idx]?.value || 0) / 1024 / 1024),
      networkTx: Math.round((historical.networkTx[idx]?.value || 0) / 1024 / 1024),
    }))

    setMetrics(points)
  }, [historical, timeRange])

  const formatBytes = (mb: number) => {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`
    return `${mb.toFixed(0)} MB`
  }

  return (
    <div className="space-y-4">
      {/* Time Range Selector */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {timeRange === 'live' ? (
            <>
              <Clock className="h-5 w-5 text-green-500" />
              <span className="text-sm font-medium">Live Monitoring</span>
              <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
            </>
          ) : (
            <>
              <Calendar className="h-5 w-5 text-blue-500" />
              <span className="text-sm font-medium">Historical Data</span>
            </>
          )}
        </div>

        <div className="flex items-center gap-2">
          <Select value={timeRange} onValueChange={(v) => setTimeRange(v as TimeRange)}>
            <SelectTrigger className="w-[180px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="live">Live (1 minute)</SelectItem>
              <SelectItem value="1h">Last Hour</SelectItem>
              <SelectItem value="6h">Last 6 Hours</SelectItem>
              <SelectItem value="24h">Last 24 Hours</SelectItem>
              <SelectItem value="7d">Last 7 Days</SelectItem>
              <SelectItem value="30d">Last 30 Days</SelectItem>
            </SelectContent>
          </Select>

          {timeRange !== 'live' && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setTimeRange('live')}
            >
              Back to Live
            </Button>
          )}
        </div>
      </div>

      {historyLoading && timeRange !== 'live' ? (
        <div className="text-center py-12 text-muted-foreground">
          Loading historical data...
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* CPU Usage Graph */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Cpu className="h-5 w-5 text-blue-500" />
                <CardTitle className="text-base">CPU Usage</CardTitle>
              </div>
              {stats && (
                <span className="text-2xl font-bold text-blue-500">
                  {stats.cpuUsagePercent.toFixed(1)}%
                </span>
              )}
            </div>
            <CardDescription>Real-time CPU utilization</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                <XAxis
                  dataKey="time"
                  tick={{ fontSize: 12 }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 12 }}
                  domain={[0, 100]}
                  unit="%"
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'hsl(var(--background))',
                    border: '1px solid hsl(var(--border))'
                  }}
                  formatter={(value: number) => [`${value.toFixed(1)}%`, 'CPU']}
                />
                <Line
                  type="monotone"
                  dataKey="cpuUsage"
                  stroke="#3b82f6"
                  strokeWidth={2}
                  dot={false}
                  isAnimationActive={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Memory Usage Graph */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Activity className="h-5 w-5 text-green-500" />
                <CardTitle className="text-base">Memory Usage</CardTitle>
              </div>
              {stats && (
                <span className="text-2xl font-bold text-green-500">
                  {((stats.memoryUsedMb / stats.memoryAvailableMb) * 100).toFixed(1)}%
                </span>
              )}
            </div>
            <CardDescription>
              {stats && `${formatBytes(stats.memoryUsedMb)} / ${formatBytes(stats.memoryAvailableMb)}`}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                <XAxis
                  dataKey="time"
                  tick={{ fontSize: 12 }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 12 }}
                  domain={[0, 100]}
                  unit="%"
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'hsl(var(--background))',
                    border: '1px solid hsl(var(--border))'
                  }}
                  formatter={(value: number) => [`${value.toFixed(1)}%`, 'Memory']}
                />
                <Line
                  type="monotone"
                  dataKey="memoryPercent"
                  stroke="#22c55e"
                  strokeWidth={2}
                  dot={false}
                  isAnimationActive={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Network Usage Graph */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Network className="h-5 w-5 text-purple-500" />
                <CardTitle className="text-base">Network I/O</CardTitle>
              </div>
              {stats && (
                <div className="text-right text-sm">
                  <div className="text-purple-500 font-semibold">
                    ↓ {formatBytes(stats.networkRxBytes / 1024 / 1024)}
                  </div>
                  <div className="text-pink-500 font-semibold">
                    ↑ {formatBytes(stats.networkTxBytes / 1024 / 1024)}
                  </div>
                </div>
              )}
            </div>
            <CardDescription>Received and transmitted data</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                <XAxis
                  dataKey="time"
                  tick={{ fontSize: 12 }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 12 }}
                  unit=" MB"
                />
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
                  dataKey="networkRx"
                  stroke="#a855f7"
                  strokeWidth={2}
                  dot={false}
                  name="Download"
                  isAnimationActive={false}
                />
                <Line
                  type="monotone"
                  dataKey="networkTx"
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
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <HardDrive className="h-5 w-5 text-orange-500" />
                <CardTitle className="text-base">Disk I/O</CardTitle>
              </div>
              {stats && (
                <div className="text-right text-sm">
                  <div className="text-orange-500 font-semibold">
                    R {formatBytes(stats.diskReadBytes / 1024 / 1024)}
                  </div>
                  <div className="text-amber-500 font-semibold">
                    W {formatBytes(stats.diskWriteBytes / 1024 / 1024)}
                  </div>
                </div>
              )}
            </div>
            <CardDescription>Read and write operations</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                <XAxis
                  dataKey="time"
                  tick={{ fontSize: 12 }}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 12 }}
                  unit=" MB"
                />
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
                  dataKey="diskRead"
                  stroke="#f97316"
                  strokeWidth={2}
                  dot={false}
                  name="Read"
                  isAnimationActive={false}
                />
                <Line
                  type="monotone"
                  dataKey="diskWrite"
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
    </div>
  )
}
