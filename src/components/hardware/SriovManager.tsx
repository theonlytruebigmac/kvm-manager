import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { SriovPf, SriovVf, SriovVfConfig, VM } from '@/lib/types'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { toast } from 'sonner'
import {
  Network,
  ChevronDown,
  ChevronRight,
  Plus,
  Minus,
  Settings,
  Check,
  X,
  Loader2,
  AlertTriangle,
} from 'lucide-react'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

interface SriovManagerProps {
  vm: VM
  children?: React.ReactNode
}

export function SriovManager({ vm, children }: SriovManagerProps) {
  const [open, setOpen] = useState(false)
  const [expandedPfs, setExpandedPfs] = useState<Set<string>>(new Set())
  const [configVf, setConfigVf] = useState<{
    pf: SriovPf
    vf: SriovVf
  } | null>(null)
  const [vfConfig, setVfConfig] = useState<SriovVfConfig>({
    pfInterface: '',
    vfIndex: 0,
    macAddress: '',
    vlanId: undefined,
    spoofCheck: true,
    trust: false,
  })

  const queryClient = useQueryClient()

  // Fetch all SR-IOV PFs
  const { data: pfs = [], isLoading: loadingPfs } = useQuery({
    queryKey: ['sriov-pfs'],
    queryFn: () => api.listSriovPfs(),
    enabled: open,
    refetchInterval: open ? 5000 : false,
  })

  // Fetch VFs for each expanded PF
  const vfQueries = useQuery({
    queryKey: ['sriov-vfs', Array.from(expandedPfs)],
    queryFn: async () => {
      const results: Record<string, SriovVf[]> = {}
      for (const pfAddress of expandedPfs) {
        try {
          results[pfAddress] = await api.listSriovVfs(pfAddress)
        } catch (err) {
          console.error(`Failed to fetch VFs for ${pfAddress}:`, err)
          results[pfAddress] = []
        }
      }
      return results
    },
    enabled: open && expandedPfs.size > 0,
    refetchInterval: open ? 5000 : false,
  })

  const vfsByPf = vfQueries.data || {}

  // Enable VFs mutation
  const enableVfsMutation = useMutation({
    mutationFn: ({ pfAddress, numVfs }: { pfAddress: string; numVfs: number }) =>
      api.enableSriovVfs(pfAddress, numVfs),
    onSuccess: (_, { pfAddress, numVfs }) => {
      toast.success(`Enabled ${numVfs} VFs on ${pfAddress}`)
      queryClient.invalidateQueries({ queryKey: ['sriov-pfs'] })
      queryClient.invalidateQueries({ queryKey: ['sriov-vfs'] })
    },
    onError: (err: Error) => {
      toast.error(`Failed to enable VFs: ${err.message}`)
    },
  })

  // Configure VF mutation
  const configureVfMutation = useMutation({
    mutationFn: (config: SriovVfConfig) => api.configureSriovVf(config),
    onSuccess: () => {
      toast.success('VF configured successfully')
      setConfigVf(null)
      queryClient.invalidateQueries({ queryKey: ['sriov-vfs'] })
    },
    onError: (err: Error) => {
      toast.error(`Failed to configure VF: ${err.message}`)
    },
  })

  // Attach VF to VM mutation
  const attachVfMutation = useMutation({
    mutationFn: (vfAddress: string) => api.attachSriovVf(vm.id, vfAddress),
    onSuccess: () => {
      toast.success('VF attached to VM')
      queryClient.invalidateQueries({ queryKey: ['sriov-vfs'] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
    },
    onError: (err: Error) => {
      toast.error(`Failed to attach VF: ${err.message}`)
    },
  })

  // Detach VF from VM mutation
  const detachVfMutation = useMutation({
    mutationFn: (vfAddress: string) => api.detachSriovVf(vm.id, vfAddress),
    onSuccess: () => {
      toast.success('VF detached from VM')
      queryClient.invalidateQueries({ queryKey: ['sriov-vfs'] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
    },
    onError: (err: Error) => {
      toast.error(`Failed to detach VF: ${err.message}`)
    },
  })

  const togglePfExpanded = (pfAddress: string) => {
    setExpandedPfs(prev => {
      const next = new Set(prev)
      if (next.has(pfAddress)) {
        next.delete(pfAddress)
      } else {
        next.add(pfAddress)
      }
      return next
    })
  }

  const openVfConfig = (pf: SriovPf, vf: SriovVf) => {
    setConfigVf({ pf, vf })
    setVfConfig({
      pfInterface: pf.interfaceName || '',
      vfIndex: vf.vfIndex,
      macAddress: vf.macAddress || '',
      vlanId: vf.vlanId,
      spoofCheck: true,
      trust: false,
    })
  }

  const isVmRunning = vm.state === 'running'

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {children || (
          <Button variant="outline" size="sm">
            <Network className="w-4 h-4 mr-2" />
            SR-IOV Manager
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Network className="w-5 h-5" />
            SR-IOV Network Manager - {vm.name}
          </DialogTitle>
          <DialogDescription>
            Manage SR-IOV Virtual Functions for high-performance network passthrough.
            {!isVmRunning && (
              <span className="text-yellow-600 dark:text-yellow-400 ml-2">
                (VM must be running to attach VFs)
              </span>
            )}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 mt-4">
          {loadingPfs ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin mr-2" />
              <span>Scanning for SR-IOV devices...</span>
            </div>
          ) : pfs.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <AlertTriangle className="w-12 h-12 mx-auto mb-4 text-yellow-500" />
              <p className="font-medium">No SR-IOV capable network adapters found</p>
              <p className="text-sm mt-2">
                SR-IOV requires compatible network hardware and IOMMU (VT-d/AMD-Vi) enabled in BIOS.
              </p>
            </div>
          ) : (
            <Tabs defaultValue="pfs">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="pfs">Physical Functions</TabsTrigger>
                <TabsTrigger value="attached">Attached to VM</TabsTrigger>
              </TabsList>

              <TabsContent value="pfs" className="space-y-4 mt-4">
                {pfs.map(pf => (
                  <div key={pf.address} className="border rounded-lg">
                    <Collapsible
                      open={expandedPfs.has(pf.address)}
                      onOpenChange={() => togglePfExpanded(pf.address)}
                    >
                      <CollapsibleTrigger asChild>
                        <div className="flex items-center justify-between p-4 cursor-pointer hover:bg-muted/50">
                          <div className="flex items-center gap-4">
                            {expandedPfs.has(pf.address) ? (
                              <ChevronDown className="w-4 h-4" />
                            ) : (
                              <ChevronRight className="w-4 h-4" />
                            )}
                            <div>
                              <div className="font-medium">
                                {pf.deviceName || pf.address}
                              </div>
                              <div className="text-sm text-muted-foreground flex gap-4">
                                <span>{pf.vendor}</span>
                                {pf.interfaceName && (
                                  <span className="font-mono">{pf.interfaceName}</span>
                                )}
                                {pf.linkSpeed && <span>{pf.linkSpeed}</span>}
                              </div>
                            </div>
                          </div>
                          <div className="flex items-center gap-4">
                            <div className="text-right text-sm">
                              <div>
                                <span className="text-muted-foreground">VFs: </span>
                                <span className="font-medium">{pf.numVfs}</span>
                                <span className="text-muted-foreground"> / {pf.maxVfs}</span>
                              </div>
                              <div className="text-muted-foreground">
                                {pf.availableVfs} available
                              </div>
                            </div>
                            <div className="flex gap-1">
                              <TooltipProvider>
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button
                                      variant="outline"
                                      size="icon"
                                      onClick={e => {
                                        e.stopPropagation()
                                        if (pf.numVfs < pf.maxVfs) {
                                          enableVfsMutation.mutate({
                                            pfAddress: pf.address,
                                            numVfs: pf.numVfs + 1,
                                          })
                                        }
                                      }}
                                      disabled={
                                        pf.numVfs >= pf.maxVfs ||
                                        enableVfsMutation.isPending
                                      }
                                    >
                                      <Plus className="w-4 h-4" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>Add VF</TooltipContent>
                                </Tooltip>
                              </TooltipProvider>
                              <TooltipProvider>
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button
                                      variant="outline"
                                      size="icon"
                                      onClick={e => {
                                        e.stopPropagation()
                                        if (pf.numVfs > 0) {
                                          enableVfsMutation.mutate({
                                            pfAddress: pf.address,
                                            numVfs: pf.numVfs - 1,
                                          })
                                        }
                                      }}
                                      disabled={
                                        pf.numVfs <= 0 || enableVfsMutation.isPending
                                      }
                                    >
                                      <Minus className="w-4 h-4" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>Remove VF</TooltipContent>
                                </Tooltip>
                              </TooltipProvider>
                            </div>
                          </div>
                        </div>
                      </CollapsibleTrigger>

                      <CollapsibleContent>
                        <div className="px-4 pb-4 border-t">
                          {vfsByPf[pf.address]?.length === 0 ? (
                            <div className="text-center py-4 text-muted-foreground text-sm">
                              No VFs enabled. Click + to add Virtual Functions.
                            </div>
                          ) : (
                            <div className="space-y-2 mt-4">
                              {vfsByPf[pf.address]?.map(vf => (
                                <div
                                  key={vf.address}
                                  className="flex items-center justify-between p-3 bg-muted/50 rounded-lg"
                                >
                                  <div className="flex items-center gap-4">
                                    <div>
                                      <div className="font-mono text-sm">
                                        VF {vf.vfIndex}: {vf.address}
                                      </div>
                                      <div className="text-xs text-muted-foreground flex gap-3">
                                        {vf.macAddress && (
                                          <span>MAC: {vf.macAddress}</span>
                                        )}
                                        {vf.vlanId !== undefined && vf.vlanId > 0 && (
                                          <span>VLAN: {vf.vlanId}</span>
                                        )}
                                        {vf.iommuGroup !== undefined && (
                                          <span>IOMMU: {vf.iommuGroup}</span>
                                        )}
                                      </div>
                                    </div>
                                  </div>
                                  <div className="flex items-center gap-2">
                                    {vf.inUse ? (
                                      <Badge variant="secondary">
                                        {vf.attachedToVm === vm.id
                                          ? 'Attached to this VM'
                                          : `Used by ${vf.attachedToVm}`}
                                      </Badge>
                                    ) : (
                                      <Badge variant="outline" className="text-green-600">
                                        Available
                                      </Badge>
                                    )}
                                    <TooltipProvider>
                                      <Tooltip>
                                        <TooltipTrigger asChild>
                                          <Button
                                            variant="ghost"
                                            size="icon"
                                            onClick={() => openVfConfig(pf, vf)}
                                          >
                                            <Settings className="w-4 h-4" />
                                          </Button>
                                        </TooltipTrigger>
                                        <TooltipContent>Configure VF</TooltipContent>
                                      </Tooltip>
                                    </TooltipProvider>
                                    {vf.attachedToVm === vm.id ? (
                                      <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={() => detachVfMutation.mutate(vf.address)}
                                        disabled={
                                          detachVfMutation.isPending || !isVmRunning
                                        }
                                      >
                                        {detachVfMutation.isPending ? (
                                          <Loader2 className="w-4 h-4 animate-spin" />
                                        ) : (
                                          'Detach'
                                        )}
                                      </Button>
                                    ) : (
                                      <Button
                                        variant="default"
                                        size="sm"
                                        onClick={() => attachVfMutation.mutate(vf.address)}
                                        disabled={
                                          vf.inUse ||
                                          attachVfMutation.isPending ||
                                          !isVmRunning
                                        }
                                      >
                                        {attachVfMutation.isPending ? (
                                          <Loader2 className="w-4 h-4 animate-spin" />
                                        ) : (
                                          'Attach'
                                        )}
                                      </Button>
                                    )}
                                  </div>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      </CollapsibleContent>
                    </Collapsible>
                  </div>
                ))}
              </TabsContent>

              <TabsContent value="attached" className="mt-4">
                {(() => {
                  const attachedVfs = Object.values(vfsByPf)
                    .flat()
                    .filter(vf => vf.attachedToVm === vm.id)

                  if (attachedVfs.length === 0) {
                    return (
                      <div className="text-center py-8 text-muted-foreground">
                        <Network className="w-12 h-12 mx-auto mb-4 opacity-50" />
                        <p>No SR-IOV VFs attached to this VM</p>
                        <p className="text-sm mt-2">
                          Expand a Physical Function above and attach VFs to enable SR-IOV
                          networking.
                        </p>
                      </div>
                    )
                  }

                  return (
                    <div className="space-y-2">
                      {attachedVfs.map(vf => (
                        <div
                          key={vf.address}
                          className="flex items-center justify-between p-4 border rounded-lg"
                        >
                          <div>
                            <div className="font-medium font-mono">{vf.address}</div>
                            <div className="text-sm text-muted-foreground">
                              VF {vf.vfIndex} of {vf.parentPf}
                              {vf.macAddress && ` • MAC: ${vf.macAddress}`}
                            </div>
                          </div>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => detachVfMutation.mutate(vf.address)}
                            disabled={detachVfMutation.isPending || !isVmRunning}
                          >
                            {detachVfMutation.isPending ? (
                              <Loader2 className="w-4 h-4 animate-spin mr-2" />
                            ) : (
                              <X className="w-4 h-4 mr-2" />
                            )}
                            Detach
                          </Button>
                        </div>
                      ))}
                    </div>
                  )
                })()}
              </TabsContent>
            </Tabs>
          )}
        </div>

        {/* VF Configuration Dialog */}
        <Dialog open={!!configVf} onOpenChange={open => !open && setConfigVf(null)}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Configure Virtual Function</DialogTitle>
              <DialogDescription>
                Configure VF {configVf?.vf.vfIndex} on {configVf?.pf.interfaceName}
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="vf-mac">MAC Address</Label>
                <Input
                  id="vf-mac"
                  placeholder="00:00:00:00:00:00"
                  value={vfConfig.macAddress || ''}
                  onChange={e =>
                    setVfConfig({ ...vfConfig, macAddress: e.target.value })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  Leave empty for auto-generated MAC
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="vf-vlan">VLAN ID</Label>
                <Input
                  id="vf-vlan"
                  type="number"
                  min={0}
                  max={4095}
                  placeholder="0 (no VLAN)"
                  value={vfConfig.vlanId ?? ''}
                  onChange={e =>
                    setVfConfig({
                      ...vfConfig,
                      vlanId: e.target.value ? parseInt(e.target.value) : undefined,
                    })
                  }
                />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="vf-spoofcheck">Spoof Check</Label>
                  <p className="text-xs text-muted-foreground">
                    Prevent MAC/VLAN spoofing
                  </p>
                </div>
                <Switch
                  id="vf-spoofcheck"
                  checked={vfConfig.spoofCheck}
                  onCheckedChange={checked =>
                    setVfConfig({ ...vfConfig, spoofCheck: checked })
                  }
                />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="vf-trust">Trust Mode</Label>
                  <p className="text-xs text-muted-foreground">
                    Allow VF to set promiscuous mode
                  </p>
                </div>
                <Switch
                  id="vf-trust"
                  checked={vfConfig.trust}
                  onCheckedChange={checked =>
                    setVfConfig({ ...vfConfig, trust: checked })
                  }
                />
              </div>
            </div>

            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setConfigVf(null)}>
                Cancel
              </Button>
              <Button
                onClick={() => configureVfMutation.mutate(vfConfig)}
                disabled={configureVfMutation.isPending || !configVf?.pf.interfaceName}
              >
                {configureVfMutation.isPending ? (
                  <Loader2 className="w-4 h-4 animate-spin mr-2" />
                ) : (
                  <Check className="w-4 h-4 mr-2" />
                )}
                Apply
              </Button>
            </div>
          </DialogContent>
        </Dialog>
      </DialogContent>
    </Dialog>
  )
}
