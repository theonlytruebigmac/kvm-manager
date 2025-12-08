import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts'
import { Activity, HardDrive, Network, Cpu } from 'lucide-react'


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

const MAX_POINTS = 60 // Keep last 60 data points (1 minute at 1s interval)

export function ResourceGraphs({ vmId }: ResourceGraphsProps) {
  const [metrics, setMetrics] = useState<MetricPoint[]>([])

  // Poll VM stats every second
  const { data: stats } = useQuery({
    queryKey: ['vm-stats', vmId],
    queryFn: () => api.getVmStats(vmId),
    refetchInterval: 1000,
    enabled: true,
  })

  // Update metrics history when new stats arrive
  useEffect(() => {
    if (!stats) return

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
  }, [stats])

  const formatBytes = (mb: number) => {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`
    return `${mb.toFixed(0)} MB`
  }

  return (
    <div className="space-y-4">
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
    </div>
  )
}
