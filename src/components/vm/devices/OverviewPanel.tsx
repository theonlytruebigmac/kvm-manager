import type { VM } from '@/lib/types'

interface OverviewPanelProps {
  vm: VM
}

export function OverviewPanel({ vm }: OverviewPanelProps) {
  return (
    <div className="space-y-6 max-w-4xl">
      <div>
        <h2 className="text-2xl font-semibold mb-4">{vm.name}</h2>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium text-muted-foreground">State</label>
              <p className="text-base capitalize">{vm.state}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">UUID</label>
              <p className="text-base font-mono text-xs">{vm.id}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">CPUs</label>
              <p className="text-base">{vm.cpus} vCPUs</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">Memory</label>
              <p className="text-base">{(vm.memoryMb / 1024).toFixed(1)} GB</p>
            </div>
          </div>
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium text-muted-foreground">Disk</label>
              <p className="text-base">{vm.diskSizeGb} GB</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">OS Type</label>
              <p className="text-base capitalize">{vm.osType || 'Unknown'}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">Architecture</label>
              <p className="text-base">{vm.arch || 'x86_64'}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">Chipset</label>
              <p className="text-base">{vm.machine?.includes('q35') ? 'Q35' : 'i440FX'}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
