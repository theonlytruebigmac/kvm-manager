import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { Badge } from '@/components/ui/badge'
import { Cpu, MemoryStick, HardDrive, Network, Tag, Server, Disc } from 'lucide-react'
import type { VM } from '@/lib/types'

interface VmInfoTooltipProps {
  vm: VM
  children: React.ReactNode
}

export function VmInfoTooltip({ vm, children }: VmInfoTooltipProps) {
  // Fetch live stats for running VMs
  const { data: vmStats } = useQuery({
    queryKey: ['vm-stats', vm.id],
    queryFn: () => api.getVmStats(vm.id),
    enabled: vm.state === 'running',
    staleTime: 2000,
  })

  const stateColors: Record<string, string> = {
    running: 'bg-green-500',
    stopped: 'bg-gray-400',
    paused: 'bg-yellow-500',
  }

  // Get primary IP address
  const primaryIp = vm.networkInterfaces?.[0]?.ipAddress

  return (
    <TooltipProvider>
      <Tooltip delayDuration={300}>
        <TooltipTrigger asChild>
          {children}
        </TooltipTrigger>
        <TooltipContent side="right" className="p-0 w-72">
          <div className="p-3 space-y-3">
            {/* Header */}
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${stateColors[vm.state] || 'bg-gray-400'}`} />
              <span className="font-medium truncate">{vm.name}</span>
              <Badge variant="outline" className="ml-auto text-xs">
                {vm.state}
              </Badge>
            </div>

            {/* Quick Stats */}
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div className="flex items-center gap-2 text-muted-foreground">
                <Cpu className="w-3.5 h-3.5" />
                <span>
                  {vmStats ? `${Math.round(vmStats.cpuUsagePercent)}%` : `${vm.cpuCount} vCPU`}
                </span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <MemoryStick className="w-3.5 h-3.5" />
                <span>
                  {vmStats
                    ? `${Math.round(vmStats.memoryUsedMb)}/${vm.memoryMb} MB`
                    : `${vm.memoryMb} MB`}
                </span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <HardDrive className="w-3.5 h-3.5" />
                <span>{vm.diskSizeGb} GB</span>
              </div>
              {primaryIp && (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Network className="w-3.5 h-3.5" />
                  <span className="truncate">{primaryIp}</span>
                </div>
              )}
            </div>

            {/* Additional Info */}
            <div className="border-t pt-2 space-y-1.5 text-xs text-muted-foreground">
              {vm.osType && (
                <div className="flex items-center gap-2">
                  <Server className="w-3 h-3" />
                  <span>{vm.osType}</span>
                </div>
              )}
              {vm.disks?.length > 0 && (
                <div className="flex items-center gap-2">
                  <Disc className="w-3 h-3" />
                  <span>{vm.disks.length} disk{vm.disks.length > 1 ? 's' : ''}</span>
                </div>
              )}
              {vm.tags && vm.tags.length > 0 && (
                <div className="flex items-center gap-2">
                  <Tag className="w-3 h-3" />
                  <div className="flex gap-1 flex-wrap">
                    {vm.tags.slice(0, 3).map(tag => (
                      <span key={tag} className="px-1.5 py-0.5 bg-muted rounded text-xs">{tag}</span>
                    ))}
                    {vm.tags.length > 3 && (
                      <span className="text-muted-foreground">+{vm.tags.length - 3}</span>
                    )}
                  </div>
                </div>
              )}
            </div>

            {/* Hint */}
            <div className="text-xs text-muted-foreground/60 border-t pt-2">
              Double-click to view details
            </div>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
