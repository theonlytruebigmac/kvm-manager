import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Activity, HardDrive, Network, Server, Cpu, MemoryStick, Clock, Users, Globe, BarChart3, AlertCircle, ChevronDown, Copy, Terminal } from 'lucide-react'
import { toast } from 'sonner'
import { useState } from 'react'

interface GuestInfoProps {
  vmId?: string  // Kept for backwards compatibility, not currently used
  vmName: string
  vmState?: string
  osType?: string  // Kept for backwards compatibility, not currently used
  compact?: boolean
}

export function GuestInfo({ vmName, vmState, compact }: GuestInfoProps) {
  const [showInstallHelp, setShowInstallHelp] = useState(false)
  const isRunning = vmState === 'running'

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text)
    toast.success(`Copied ${label} to clipboard`)
  }

  // Only check agent status if VM is running
  const { data: agentStatus, isLoading: statusLoading } = useQuery({
    queryKey: ['guestAgentStatus', vmName],
    queryFn: () => api.checkGuestAgentStatus(vmName),
    refetchInterval: 10000, // Check every 10 seconds
    retry: false,
    enabled: isRunning,
  })

  // Get system info if agent is available
  const { data: systemInfo, isLoading: systemLoading } = useQuery({
    queryKey: ['guestSystemInfo', vmName],
    queryFn: () => api.getGuestSystemInfo(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 5000, // Refresh every 5 seconds
    retry: false,
  })

  // Get network info if agent is available
  const { data: networkInfo } = useQuery({
    queryKey: ['guestNetworkInfo', vmName],
    queryFn: () => api.getGuestNetworkInfo(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 10000,
    retry: false,
  })

  // Get disk usage if agent is available
  const { data: diskUsage } = useQuery({
    queryKey: ['guestDiskUsage', vmName],
    queryFn: () => api.getGuestDiskUsage(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 30000, // Refresh every 30 seconds
    retry: false,
  })

  // Get CPU stats for performance monitoring
  const { data: cpuStats } = useQuery({
    queryKey: ['guestCpuStats', vmName],
    queryFn: () => api.getGuestCpuStats(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 3000, // Refresh every 3 seconds for realtime feel
    retry: false,
  })

  // Get disk I/O stats
  const { data: diskStats } = useQuery({
    queryKey: ['guestDiskStats', vmName],
    queryFn: () => api.getGuestDiskStats(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 5000,
    retry: false,
  })

  // Get logged-in users
  const { data: users } = useQuery({
    queryKey: ['guestUsers', vmName],
    queryFn: () => api.getGuestUsers(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 30000,
    retry: false,
  })

  // Get timezone
  const { data: timezone } = useQuery({
    queryKey: ['guestTimezone', vmName],
    queryFn: () => api.getGuestTimezone(vmName),
    enabled: agentStatus?.available === true,
    refetchInterval: 60000, // Timezone rarely changes
    retry: false,
  })

  if (statusLoading) {
    return (
      <div className="flex items-center justify-center py-4 text-muted-foreground text-xs">
        <Activity className="h-4 w-4 animate-pulse mr-2" />
        Checking agent...
      </div>
    )
  }

  // VM is not running - show install option
  if (!isRunning) {
    return (
      <div className="space-y-3">
        <div className="text-xs text-muted-foreground text-center flex items-center justify-center gap-1.5">
          <AlertCircle className="w-3.5 h-3.5" />
          <span>VM is not running</span>
        </div>
        <p className="text-[10px] text-muted-foreground text-center">
          Start the VM to check QEMU Guest Agent status
        </p>
      </div>
    )
  }

  if (!agentStatus?.available) {
    // VM is running but agent not responding - show install instructions
    const installCommands = [
      { distro: 'Debian/Ubuntu', cmd: 'sudo apt install qemu-guest-agent && sudo systemctl enable --now qemu-guest-agent' },
      { distro: 'Fedora/RHEL', cmd: 'sudo dnf install qemu-guest-agent && sudo systemctl enable --now qemu-guest-agent' },
      { distro: 'Arch Linux', cmd: 'sudo pacman -S qemu-guest-agent && sudo systemctl enable --now qemu-guest-agent' },
      { distro: 'Alpine', cmd: 'apk add qemu-guest-agent && rc-update add qemu-guest-agent && service qemu-guest-agent start' },
    ]

    return (
      <div className="space-y-3">
        <div className="text-xs text-amber-500 text-center flex items-center justify-center gap-1.5">
          <AlertCircle className="w-3.5 h-3.5" />
          <span>Agent not responding</span>
        </div>

        <Collapsible open={showInstallHelp} onOpenChange={setShowInstallHelp}>
          <CollapsibleTrigger asChild>
            <Button
              variant="outline"
              size="sm"
              className="w-full h-7 text-xs justify-between"
            >
              <span className="flex items-center gap-1.5">
                <Terminal className="w-3 h-3" />
                Install QEMU Guest Agent
              </span>
              <ChevronDown className={`w-3 h-3 transition-transform ${showInstallHelp ? 'rotate-180' : ''}`} />
            </Button>
          </CollapsibleTrigger>
          <CollapsibleContent className="mt-2 space-y-2">
            <p className="text-[10px] text-muted-foreground">
              Install in the guest VM using your package manager:
            </p>
            {installCommands.map(({ distro, cmd }) => (
              <div key={distro} className="space-y-1">
                <div className="text-[10px] text-muted-foreground font-medium">{distro}</div>
                <div className="flex items-center gap-1">
                  <code className="flex-1 text-[9px] bg-muted/50 px-1.5 py-1 rounded font-mono overflow-x-auto whitespace-nowrap">
                    {cmd}
                  </code>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 shrink-0"
                    onClick={() => copyToClipboard(cmd, distro)}
                  >
                    <Copy className="w-3 h-3" />
                  </Button>
                </div>
              </div>
            ))}
            <p className="text-[10px] text-muted-foreground pt-1">
              After installing, the agent should connect within 10 seconds.
            </p>
          </CollapsibleContent>
        </Collapsible>
      </div>
    )
  }

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)

    const parts = []
    if (days > 0) parts.push(`${days}d`)
    if (hours > 0) parts.push(`${hours}h`)
    if (minutes > 0) parts.push(`${minutes}m`)

    return parts.join(' ') || '< 1m'
  }

  const formatBytes = (bytes: number) => {
    const gb = bytes / (1024 ** 3)
    if (gb >= 1) return `${gb.toFixed(1)} GB`
    const mb = bytes / (1024 ** 2)
    return `${mb.toFixed(0)} MB`
  }

  const formatMemory = (kb: number) => {
    const gb = kb / (1024 ** 2)
    if (gb >= 1) return `${gb.toFixed(1)} GB`
    const mb = kb / 1024
    return `${mb.toFixed(0)} MB`
  }

  // Compact mode for Overview tab
  if (compact) {
    const primaryIp = networkInfo?.interfaces
      ?.flatMap(i => i.ipv4Addresses || [])
      ?.filter(ip => ip && typeof ip === 'string')
      ?.find(ip => !ip.startsWith('127.'))

    return (
      <div className="space-y-3">
        {/* Status indicator */}
        <div className="flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
          <span className="text-xs text-green-500 font-medium">Agent Connected</span>
        </div>

        {/* System info grid */}
        {systemInfo ? (
          <div className="space-y-2 text-xs">
            <div className="flex items-baseline justify-between">
              <span className="text-muted-foreground">Hostname</span>
              <span className="font-medium">{systemInfo.hostname}</span>
            </div>
            {systemInfo.osName && (
              <div className="flex items-baseline justify-between">
                <span className="text-muted-foreground">OS</span>
                <span className="text-muted-foreground truncate max-w-[120px]">{systemInfo.osName}</span>
              </div>
            )}
            <div className="flex items-baseline justify-between">
              <span className="text-muted-foreground">Uptime</span>
              <span>{formatUptime(systemInfo.uptimeSeconds)}</span>
            </div>
            {primaryIp && (
              <div className="flex items-baseline justify-between">
                <span className="text-muted-foreground">IP</span>
                <span className="font-mono text-[11px]">{primaryIp}</span>
              </div>
            )}
            {cpuStats && typeof cpuStats.usagePercent === 'number' && (
              <div className="flex items-baseline justify-between">
                <span className="text-muted-foreground">CPU Usage</span>
                <span className={cpuStats.usagePercent > 80 ? 'text-red-500' : cpuStats.usagePercent > 50 ? 'text-yellow-500' : 'text-green-500'}>
                  {cpuStats.usagePercent.toFixed(1)}%
                </span>
              </div>
            )}
          </div>
        ) : systemLoading ? (
          <p className="text-xs text-muted-foreground">Loading...</p>
        ) : null}
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Agent Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5 text-green-500" />
            Guest Information
            <Badge variant="outline" className="ml-auto">
              Agent v{agentStatus.agentInfo?.version || 'unknown'}
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {systemLoading ? (
            <div className="text-sm text-muted-foreground">Loading system information...</div>
          ) : systemInfo ? (
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1">
                <div className="text-sm font-medium flex items-center gap-2">
                  <Server className="h-4 w-4" />
                  Operating System
                </div>
                <div className="text-sm text-muted-foreground">
                  {systemInfo.osName} {systemInfo.osVersion}
                </div>
                <div className="text-xs text-muted-foreground">
                  Kernel: {systemInfo.kernelVersion}
                </div>
              </div>

              <div className="space-y-1">
                <div className="text-sm font-medium flex items-center gap-2">
                  <Server className="h-4 w-4" />
                  Hostname
                </div>
                <div className="text-sm text-muted-foreground font-mono">
                  {systemInfo.hostname}
                </div>
              </div>

              <div className="space-y-1">
                <div className="text-sm font-medium flex items-center gap-2">
                  <Cpu className="h-4 w-4" />
                  CPU & Architecture
                </div>
                <div className="text-sm text-muted-foreground">
                  {systemInfo.cpuCount} vCPU{systemInfo.cpuCount > 1 ? 's' : ''} ({systemInfo.architecture})
                </div>
              </div>

              <div className="space-y-1">
                <div className="text-sm font-medium flex items-center gap-2">
                  <MemoryStick className="h-4 w-4" />
                  Total Memory
                </div>
                <div className="text-sm text-muted-foreground">
                  {formatMemory(systemInfo.totalMemoryKb)}
                </div>
              </div>

              <div className="space-y-1">
                <div className="text-sm font-medium flex items-center gap-2">
                  <Clock className="h-4 w-4" />
                  Uptime
                </div>
                <div className="text-sm text-muted-foreground">
                  {formatUptime(systemInfo.uptimeSeconds)}
                </div>
              </div>

              {timezone && (
                <div className="space-y-1">
                  <div className="text-sm font-medium flex items-center gap-2">
                    <Globe className="h-4 w-4" />
                    Timezone
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {timezone.zone} (UTC{timezone.offset >= 0 ? '+' : ''}{timezone.offset / 3600}h)
                  </div>
                </div>
              )}
            </div>
          ) : null}
        </CardContent>
      </Card>

      {/* CPU & Performance Stats */}
      {cpuStats && typeof cpuStats.usagePercent === 'number' && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="h-5 w-5" />
              CPU Performance
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {/* Overall CPU Usage */}
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="font-medium">Overall CPU Usage</span>
                  <span className={cpuStats.usagePercent > 80 ? 'text-red-500 font-medium' : cpuStats.usagePercent > 50 ? 'text-yellow-500' : 'text-green-500'}>
                    {cpuStats.usagePercent.toFixed(1)}%
                  </span>
                </div>
                <div className="w-full h-3 bg-secondary rounded-full overflow-hidden">
                  <div
                    className={`h-full transition-all ${
                      cpuStats.usagePercent > 80
                        ? 'bg-red-500'
                        : cpuStats.usagePercent > 50
                        ? 'bg-yellow-500'
                        : 'bg-green-500'
                    }`}
                    style={{ width: `${Math.min(cpuStats.usagePercent, 100)}%` }}
                  />
                </div>
              </div>

              {/* Per-CPU Stats */}
              {cpuStats.cpus.length > 1 && (
                <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                  {cpuStats.cpus.map((cpu) => {
                    const total = cpu.user + cpu.system + cpu.idle + cpu.iowait + cpu.nice + cpu.irq + cpu.softirq + cpu.steal
                    const usage = total > 0 ? ((total - cpu.idle) / total) * 100 : 0
                    return (
                      <div key={cpu.cpu} className="text-xs p-2 bg-muted rounded-md">
                        <div className="flex justify-between mb-1">
                          <span className="font-medium">CPU {cpu.cpu}</span>
                          <span className={usage > 80 ? 'text-red-500' : usage > 50 ? 'text-yellow-500' : 'text-green-500'}>
                            {usage.toFixed(0)}%
                          </span>
                        </div>
                        <div className="w-full h-1.5 bg-secondary rounded-full overflow-hidden">
                          <div
                            className={`h-full ${usage > 80 ? 'bg-red-500' : usage > 50 ? 'bg-yellow-500' : 'bg-green-500'}`}
                            style={{ width: `${Math.min(usage, 100)}%` }}
                          />
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Logged-in Users */}
      {users && users.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Users className="h-5 w-5" />
              Logged-in Users
              <Badge variant="outline" className="ml-auto">
                {users.length} active
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {users.map((user, idx) => (
                <div key={idx} className="flex items-center justify-between text-sm border-l-2 border-muted pl-3 py-1">
                  <span className="font-mono font-medium">{user.username}</span>
                  <span className="text-muted-foreground text-xs">
                    since {new Date(user.loginTime * 1000).toLocaleString()}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Disk I/O Stats */}
      {diskStats && diskStats.disks.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <HardDrive className="h-5 w-5" />
              Disk I/O Statistics
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {diskStats.disks.map((disk) => (
                <div key={disk.name} className="border-l-2 border-muted pl-4 py-2">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-mono font-medium">{disk.name}</span>
                  </div>
                  <div className="grid grid-cols-2 gap-4 text-sm text-muted-foreground">
                    <div>
                      <span className="font-medium text-foreground">Read:</span>{' '}
                      {formatBytes(disk.readBytes)} ({disk.readIos.toLocaleString()} ops)
                    </div>
                    <div>
                      <span className="font-medium text-foreground">Write:</span>{' '}
                      {formatBytes(disk.writeBytes)} ({disk.writeIos.toLocaleString()} ops)
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Network Information */}
      {networkInfo && networkInfo.interfaces.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Network className="h-5 w-5" />
              Network Interfaces
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {networkInfo.interfaces.map((iface) => (
                <div key={iface.name} className="border-l-2 border-muted pl-4 space-y-2">
                  <div className="flex items-center gap-2">
                    <span className="font-mono font-medium">{iface.name}</span>
                    <Badge variant="outline">{iface.state}</Badge>
                  </div>

                  <div className="text-sm text-muted-foreground space-y-1">
                    {iface.macAddress && (
                      <div>
                        <span className="font-medium">MAC:</span> {iface.macAddress}
                      </div>
                    )}

                    {(iface.ipv4Addresses?.length ?? 0) > 0 && (
                      <div>
                        <span className="font-medium">IPv4:</span>{' '}
                        {iface.ipv4Addresses?.filter(ip => ip).map((ip) => (
                          <Badge key={ip} variant="secondary" className="ml-1 font-mono text-xs">
                            {ip}
                          </Badge>
                        ))}
                      </div>
                    )}

                    {(iface.ipv6Addresses?.length ?? 0) > 0 && (
                      <div>
                        <span className="font-medium">IPv6:</span>{' '}
                        {iface.ipv6Addresses?.filter(ip => ip).slice(0, 2).map((ip) => (
                          <Badge key={ip} variant="secondary" className="ml-1 font-mono text-xs">
                            {ip}
                          </Badge>
                        ))}
                        {iface.ipv6Addresses.length > 2 && (
                          <span className="text-xs ml-2">
                            +{iface.ipv6Addresses.length - 2} more
                          </span>
                        )}
                      </div>
                    )}

                    <div className="text-xs">
                      MTU: {iface.mtu}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Disk Usage */}
      {diskUsage && diskUsage.filesystems.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <HardDrive className="h-5 w-5" />
              Disk Usage
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {diskUsage.filesystems.map((fs) => (
                <div key={fs.mountPoint} className="space-y-2">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-mono text-sm font-medium">{fs.mountPoint}</div>
                      <div className="text-xs text-muted-foreground">
                        {fs.device} ({fs.fsType})
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm font-medium">
                        {formatBytes(fs.usedBytes)} / {formatBytes(fs.totalBytes)}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {fs.usedPercent.toFixed(1)}% used
                      </div>
                    </div>
                  </div>

                  {/* Progress bar */}
                  <div className="w-full h-2 bg-secondary rounded-full overflow-hidden">
                    <div
                      className={`h-full transition-all ${
                        fs.usedPercent > 90
                          ? 'bg-red-500'
                          : fs.usedPercent > 75
                          ? 'bg-yellow-500'
                          : 'bg-green-500'
                      }`}
                      style={{ width: `${Math.min(fs.usedPercent, 100)}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
