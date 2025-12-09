import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Activity, HardDrive, Network, Server, Cpu, MemoryStick, Clock, Disc, Download } from 'lucide-react'
import { toast } from 'sonner'

interface GuestInfoProps {
  vmId: string
  vmName: string
}

export function GuestInfo({ vmId, vmName }: GuestInfoProps) {
  const queryClient = useQueryClient()

  // Mutation to mount guest agent ISO
  const mountIsoMutation = useMutation({
    mutationFn: () => api.mountGuestAgentIso(vmId),
    onSuccess: () => {
      toast.success('Guest agent ISO mounted successfully', {
        description: 'Mount the ISO in your VM and run the installation script.'
      })
    },
    onError: (error: Error) => {
      toast.error('Failed to mount ISO', {
        description: error.message
      })
    },
  })

  // Mutation to eject CDROM
  const ejectMutation = useMutation({
    mutationFn: () => api.ejectCdrom(vmId),
    onSuccess: () => {
      toast.success('CDROM ejected successfully')
    },
    onError: (error: Error) => {
      toast.error('Failed to eject CDROM', {
        description: error.message
      })
    },
  })

  // Check if guest agent is available
  const { data: agentStatus, isLoading: statusLoading } = useQuery({
    queryKey: ['guestAgentStatus', vmName],
    queryFn: () => api.checkGuestAgentStatus(vmName),
    refetchInterval: 10000, // Check every 10 seconds
    retry: false,
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

  if (statusLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Guest Information
          </CardTitle>
          <CardDescription>Loading guest agent status...</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  if (!agentStatus?.available) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5 text-muted-foreground" />
            Guest Information
          </CardTitle>
          <CardDescription>
            Guest agent not available. Install kvmmanager-agent inside the VM for enhanced features.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="text-sm text-muted-foreground space-y-2">
              <p>The guest agent provides:</p>
              <ul className="list-disc list-inside space-y-1 ml-2">
                <li>Real OS information and hostname</li>
                <li>Network interfaces with IP addresses</li>
                <li>Disk usage from inside the guest</li>
                <li>Command execution capabilities</li>
              </ul>
            </div>

            <div className="flex flex-col gap-2 pt-2 border-t">
              <p className="text-sm font-medium">Installation:</p>
              <div className="flex gap-2">
                <Button
                  onClick={() => mountIsoMutation.mutate()}
                  disabled={mountIsoMutation.isPending}
                  size="sm"
                  className="gap-2"
                >
                  <Disc className="h-4 w-4" />
                  {mountIsoMutation.isPending ? 'Mounting...' : 'Mount Agent ISO'}
                </Button>
                <Button
                  onClick={() => ejectMutation.mutate()}
                  disabled={ejectMutation.isPending}
                  variant="outline"
                  size="sm"
                  className="gap-2"
                >
                  <Download className="h-4 w-4" />
                  {ejectMutation.isPending ? 'Ejecting...' : 'Eject CDROM'}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                After mounting, connect to the VM and run the installation script:
                <br />
                <code className="bg-muted px-1 py-0.5 rounded mt-1 inline-block">
                  Alpine: sh /mnt/cdrom/install-alpine.sh
                </code>
                <br />
                <code className="bg-muted px-1 py-0.5 rounded mt-1 inline-block">
                  Debian/Ubuntu: sudo bash /media/cdrom/install-debian.sh
                </code>
                <br />
                <code className="bg-muted px-1 py-0.5 rounded mt-1 inline-block">
                  RHEL/Fedora: sudo bash /media/cdrom/install-rhel.sh
                </code>
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
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
            </div>
          ) : null}
        </CardContent>
      </Card>

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

                    {iface.ipv4Addresses.length > 0 && (
                      <div>
                        <span className="font-medium">IPv4:</span>{' '}
                        {iface.ipv4Addresses.map((ip) => (
                          <Badge key={ip} variant="secondary" className="ml-1 font-mono text-xs">
                            {ip}
                          </Badge>
                        ))}
                      </div>
                    )}

                    {iface.ipv6Addresses.length > 0 && (
                      <div>
                        <span className="font-medium">IPv6:</span>{' '}
                        {iface.ipv6Addresses.slice(0, 2).map((ip) => (
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
