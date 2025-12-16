import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Network, Trash2, RefreshCw, Copy, Check, Loader2, Unplug, Plug } from 'lucide-react'
import { useState, useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { api } from '@/lib/tauri'
import type { VM } from '@/lib/types'

interface NetworkEditorProps {
  vm: VM
  nicIndex: number
  compact?: boolean
}

// Generate random MAC address with QEMU OUI prefix
function generateMacAddress(): string {
  const hexDigit = () => Math.floor(Math.random() * 16).toString(16)
  const hexPair = () => hexDigit() + hexDigit()
  return `52:54:00:${hexPair()}:${hexPair()}:${hexPair()}`
}

export function NetworkEditor({ vm, nicIndex, compact }: NetworkEditorProps) {
  const [copied, setCopied] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const queryClient = useQueryClient()
  const nic = vm.networkInterfaces?.[nicIndex]

  // Local editing state
  const [selectedNetwork, setSelectedNetwork] = useState(nic?.network || 'default')
  const [selectedModel, setSelectedModel] = useState(nic?.type || 'virtio')
  const [hasChanges, setHasChanges] = useState(false)

  // QoS / Bandwidth state
  const [inboundAverage, setInboundAverage] = useState<string>(nic?.inboundAverage?.toString() || '')
  const [inboundPeak, setInboundPeak] = useState<string>(nic?.inboundPeak?.toString() || '')
  const [inboundBurst, setInboundBurst] = useState<string>(nic?.inboundBurst?.toString() || '')
  const [outboundAverage, setOutboundAverage] = useState<string>(nic?.outboundAverage?.toString() || '')
  const [outboundPeak, setOutboundPeak] = useState<string>(nic?.outboundPeak?.toString() || '')
  const [outboundBurst, setOutboundBurst] = useState<string>(nic?.outboundBurst?.toString() || '')
  const [hasBandwidthChanges, setHasBandwidthChanges] = useState(false)
  const [bandwidthSaving, setBandwidthSaving] = useState(false)

  // Fetch available networks
  const { data: networks = [] } = useQuery({
    queryKey: ['networks'],
    queryFn: () => api.getNetworks(),
  })

  // Fetch link state for this interface
  const { data: linkUp = true, refetch: refetchLinkState } = useQuery({
    queryKey: ['interface-link-state', vm.id, nic?.mac],
    queryFn: () => api.getInterfaceLinkState(vm.id, nic?.mac || ''),
    enabled: !!nic?.mac,
  })

  // Link state mutation
  const linkStateMutation = useMutation({
    mutationFn: (newLinkUp: boolean) => api.setInterfaceLinkState(vm.id, nic?.mac || '', newLinkUp),
    onSuccess: (_, newLinkUp) => {
      queryClient.invalidateQueries({ queryKey: ['interface-link-state', vm.id, nic?.mac] })
      refetchLinkState()
      toast.success(newLinkUp ? 'Network cable connected' : 'Network cable disconnected')
    },
    onError: (error) => {
      toast.error(`Failed to change link state: ${error}`)
    },
  })

  // Update hasChanges when selections change
  useEffect(() => {
    if (nic) {
      const changed = selectedNetwork !== (nic.network || 'default') ||
                      selectedModel !== (nic.type || 'virtio')
      setHasChanges(changed)
    }
  }, [selectedNetwork, selectedModel, nic])

  // Track bandwidth changes
  useEffect(() => {
    if (nic) {
      const origInAvg = nic.inboundAverage?.toString() || ''
      const origInPeak = nic.inboundPeak?.toString() || ''
      const origInBurst = nic.inboundBurst?.toString() || ''
      const origOutAvg = nic.outboundAverage?.toString() || ''
      const origOutPeak = nic.outboundPeak?.toString() || ''
      const origOutBurst = nic.outboundBurst?.toString() || ''

      const changed = inboundAverage !== origInAvg ||
                      inboundPeak !== origInPeak ||
                      inboundBurst !== origInBurst ||
                      outboundAverage !== origOutAvg ||
                      outboundPeak !== origOutPeak ||
                      outboundBurst !== origOutBurst
      setHasBandwidthChanges(changed)
    }
  }, [inboundAverage, inboundPeak, inboundBurst, outboundAverage, outboundPeak, outboundBurst, nic])

  // Handle bandwidth apply
  const handleApplyBandwidth = async () => {
    if (!nic) return
    setBandwidthSaving(true)
    try {
      await api.updateInterfaceBandwidth(vm.id, nic.mac, {
        inboundAverage: inboundAverage ? parseInt(inboundAverage) : undefined,
        inboundPeak: inboundPeak ? parseInt(inboundPeak) : undefined,
        inboundBurst: inboundBurst ? parseInt(inboundBurst) : undefined,
        outboundAverage: outboundAverage ? parseInt(outboundAverage) : undefined,
        outboundPeak: outboundPeak ? parseInt(outboundPeak) : undefined,
        outboundBurst: outboundBurst ? parseInt(outboundBurst) : undefined,
      })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('Bandwidth limits updated')
      setHasBandwidthChanges(false)
    } catch (error) {
      toast.error(`Failed to update bandwidth: ${error}`)
    } finally {
      setBandwidthSaving(false)
    }
  }

  const handleRevertBandwidth = () => {
    setInboundAverage(nic?.inboundAverage?.toString() || '')
    setInboundPeak(nic?.inboundPeak?.toString() || '')
    setInboundBurst(nic?.inboundBurst?.toString() || '')
    setOutboundAverage(nic?.outboundAverage?.toString() || '')
    setOutboundPeak(nic?.outboundPeak?.toString() || '')
    setOutboundBurst(nic?.outboundBurst?.toString() || '')
    setHasBandwidthChanges(false)
  }

  const detachMutation = useMutation({
    mutationFn: () => api.detachInterface(vm.id, nic?.mac || ''),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('Network interface removed')
      setShowDeleteDialog(false)
    },
    onError: (error) => {
      toast.error(`Failed to remove interface: ${error}`)
    },
  })

  // Update interface by detaching and re-attaching
  const updateMutation = useMutation({
    mutationFn: async () => {
      // First detach the existing interface
      await api.detachInterface(vm.id, nic?.mac || '')
      // Then attach a new one with new settings
      await api.attachInterface(vm.id, selectedNetwork, selectedModel, nic?.mac)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('Network interface updated')
      setHasChanges(false)
    },
    onError: (error) => {
      toast.error(`Failed to update interface: ${error}`)
    },
  })

  // Generate new MAC address
  const regenerateMacMutation = useMutation({
    mutationFn: async () => {
      const newMac = generateMacAddress()
      await api.detachInterface(vm.id, nic?.mac || '')
      await api.attachInterface(vm.id, selectedNetwork, selectedModel, newMac)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('MAC address regenerated')
    },
    onError: (error) => {
      toast.error(`Failed to regenerate MAC: ${error}`)
    },
  })

  const isPending = detachMutation.isPending || updateMutation.isPending || regenerateMacMutation.isPending

  if (!nic) {
    return (
      <div className={compact ? 'p-2' : 'p-6'}>
        <div className="flex items-center gap-2 text-muted-foreground">
          <Network className="w-4 h-4" />
          <span className="text-sm">No network interface found</span>
        </div>
      </div>
    )
  }

  const copyMac = async () => {
    try {
      await navigator.clipboard.writeText(nic.mac)
      setCopied(true)
      toast.success('MAC address copied')
      setTimeout(() => setCopied(false), 2000)
    } catch {
      toast.error('Failed to copy')
    }
  }

  const handleRevert = () => {
    setSelectedNetwork(nic.network || 'default')
    setSelectedModel(nic.type || 'virtio')
    setHasChanges(false)
  }

  // Compact mode shows inline summary with key controls
  if (compact) {
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Badge variant={nic.type === 'virtio' ? 'default' : 'secondary'} className="text-xs">
              {nic.type?.toUpperCase() || 'VIRTIO'}
            </Badge>
            <span className="text-sm">{nic.network || 'default'}</span>
          </div>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 text-red-600 hover:text-red-700 hover:bg-red-50"
            onClick={() => setShowDeleteDialog(true)}
          >
            <Trash2 className="w-3.5 h-3.5" />
          </Button>
        </div>

        <div className="flex items-center gap-2">
          <code className="text-xs font-mono bg-muted px-2 py-0.5 rounded">{nic.mac}</code>
          <Button size="icon" variant="ghost" className="h-6 w-6" onClick={copyMac}>
            {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
          </Button>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <Select value={selectedNetwork} onValueChange={setSelectedNetwork}>
            <SelectTrigger className="h-8 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {networks.map((net) => (
                <SelectItem key={net.name} value={net.name}>{net.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={selectedModel} onValueChange={setSelectedModel}>
            <SelectTrigger className="h-8 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="virtio">VirtIO</SelectItem>
              <SelectItem value="e1000e">E1000e</SelectItem>
              <SelectItem value="rtl8139">RTL8139</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {hasChanges && (
          <div className="flex gap-2">
            <Button size="sm" onClick={() => updateMutation.mutate()} disabled={isPending}>
              {updateMutation.isPending && <Loader2 className="mr-1 h-3 w-3 animate-spin" />}
              Apply
            </Button>
            <Button size="sm" variant="ghost" onClick={handleRevert}>
              Revert
            </Button>
          </div>
        )}

        <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Remove Network Interface?</AlertDialogTitle>
              <AlertDialogDescription>
                This will remove the network interface from the VM.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction
                onClick={() => detachMutation.mutate()}
                className="bg-red-600 hover:bg-red-700"
              >
                Remove
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold mb-2 flex items-center gap-2">
            <Network className="w-6 h-6" />
            Network Interface {nicIndex + 1}
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure virtual network interface settings
          </p>
        </div>
        <Badge variant={nic.type === 'virtio' ? 'default' : 'secondary'}>
          {nic.type?.toUpperCase() || 'VIRTIO'}
        </Badge>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="advanced">Advanced</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Network Source</CardTitle>
              <CardDescription>
                The virtual network or bridge this interface is connected to
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="network">Virtual Network</Label>
                <Select
                  value={selectedNetwork}
                  onValueChange={setSelectedNetwork}
                  disabled={isPending}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select network" />
                  </SelectTrigger>
                  <SelectContent>
                    {networks.map((net) => (
                      <SelectItem key={net.name} value={net.name}>
                        {net.name} {net.bridge ? `(${net.bridge})` : ''}
                      </SelectItem>
                    ))}
                    {networks.length === 0 && (
                      <SelectItem value="default">default (NAT)</SelectItem>
                    )}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Virtual network providing connectivity for this interface
                </p>
              </div>

              {nic.ipAddress && (
                <div className="p-3 bg-muted rounded-lg">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium">IP Address</p>
                      <p className="text-lg font-mono">{nic.ipAddress}</p>
                    </div>
                    <Badge variant="outline" className="text-green-600 border-green-600">
                      Connected
                    </Badge>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Device Configuration</CardTitle>
              <CardDescription>
                Network interface hardware settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="model">Device Model</Label>
                  <Select
                    value={selectedModel}
                    onValueChange={setSelectedModel}
                    disabled={isPending}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select model" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="virtio">VirtIO (Recommended)</SelectItem>
                      <SelectItem value="e1000">Intel e1000</SelectItem>
                      <SelectItem value="e1000e">Intel e1000e</SelectItem>
                      <SelectItem value="rtl8139">Realtek RTL8139</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    VirtIO provides best performance for modern guests
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="mac">MAC Address</Label>
                  <div className="flex gap-2">
                    <Input
                      id="mac"
                      value={nic.mac || '52:54:00:xx:xx:xx'}
                      readOnly
                      className="flex-1 font-mono"
                    />
                    <Button variant="outline" size="icon" onClick={copyMac}>
                      {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </Button>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => regenerateMacMutation.mutate()}
                      disabled={isPending}
                      title="Generate new MAC"
                    >
                      {regenerateMacMutation.isPending ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <RefreshCw className="w-4 h-4" />
                      )}
                    </Button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="advanced" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Link State</CardTitle>
              <CardDescription>
                Control the virtual network cable state
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between p-4 bg-muted rounded-lg">
                <div className="flex items-center gap-3">
                  {linkUp ? (
                    <Plug className="w-5 h-5 text-green-500" />
                  ) : (
                    <Unplug className="w-5 h-5 text-red-500" />
                  )}
                  <div>
                    <Label className="font-medium">Virtual Network Cable</Label>
                    <p className="text-xs text-muted-foreground">
                      {linkUp ? 'Connected - Network traffic is allowed' : 'Disconnected - No network traffic'}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <Badge variant={linkUp ? 'default' : 'destructive'}>
                    {linkUp ? 'UP' : 'DOWN'}
                  </Badge>
                  <Switch
                    checked={linkUp}
                    onCheckedChange={(checked) => linkStateMutation.mutate(checked)}
                    disabled={linkStateMutation.isPending}
                  />
                </div>
              </div>
              <p className="text-xs text-muted-foreground">
                Toggling this simulates plugging or unplugging the network cable.
                Useful for testing network failover, DHCP renewal, or diagnosing connectivity issues.
                {vm.state !== 'running' && ' (Changes take effect when VM is running)'}
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Bandwidth Limits</CardTitle>
              <CardDescription>
                Network quality of service (QoS) settings - limit traffic rates
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div>
                <h4 className="text-sm font-medium mb-3">Inbound Traffic (to VM)</h4>
                <div className="grid gap-4 md:grid-cols-3">
                  <div className="space-y-2">
                    <Label>Average (KB/s)</Label>
                    <Input
                      type="number"
                      placeholder="Unlimited"
                      value={inboundAverage}
                      onChange={(e) => setInboundAverage(e.target.value)}
                      min={0}
                    />
                    <p className="text-xs text-muted-foreground">Guaranteed rate</p>
                  </div>
                  <div className="space-y-2">
                    <Label>Peak (KB/s)</Label>
                    <Input
                      type="number"
                      placeholder="Unlimited"
                      value={inboundPeak}
                      onChange={(e) => setInboundPeak(e.target.value)}
                      min={0}
                    />
                    <p className="text-xs text-muted-foreground">Maximum rate</p>
                  </div>
                  <div className="space-y-2">
                    <Label>Burst (KB)</Label>
                    <Input
                      type="number"
                      placeholder="Auto"
                      value={inboundBurst}
                      onChange={(e) => setInboundBurst(e.target.value)}
                      min={0}
                    />
                    <p className="text-xs text-muted-foreground">Burst buffer</p>
                  </div>
                </div>
              </div>

              <div>
                <h4 className="text-sm font-medium mb-3">Outbound Traffic (from VM)</h4>
                <div className="grid gap-4 md:grid-cols-3">
                  <div className="space-y-2">
                    <Label>Average (KB/s)</Label>
                    <Input
                      type="number"
                      placeholder="Unlimited"
                      value={outboundAverage}
                      onChange={(e) => setOutboundAverage(e.target.value)}
                      min={0}
                    />
                    <p className="text-xs text-muted-foreground">Guaranteed rate</p>
                  </div>
                  <div className="space-y-2">
                    <Label>Peak (KB/s)</Label>
                    <Input
                      type="number"
                      placeholder="Unlimited"
                      value={outboundPeak}
                      onChange={(e) => setOutboundPeak(e.target.value)}
                      min={0}
                    />
                    <p className="text-xs text-muted-foreground">Maximum rate</p>
                  </div>
                  <div className="space-y-2">
                    <Label>Burst (KB)</Label>
                    <Input
                      type="number"
                      placeholder="Auto"
                      value={outboundBurst}
                      onChange={(e) => setOutboundBurst(e.target.value)}
                      min={0}
                    />
                    <p className="text-xs text-muted-foreground">Burst buffer</p>
                  </div>
                </div>
              </div>

              {hasBandwidthChanges && (
                <div className="flex justify-end gap-2 pt-2 border-t">
                  <Button variant="outline" size="sm" onClick={handleRevertBandwidth}>
                    Revert
                  </Button>
                  <Button size="sm" onClick={handleApplyBandwidth} disabled={bandwidthSaving}>
                    {bandwidthSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                    Apply Bandwidth Limits
                  </Button>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Port Isolation</CardTitle>
              <CardDescription>
                Prevent communication between VMs on the same network
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label>Port Isolation</Label>
                  <p className="text-xs text-muted-foreground">
                    When enabled, this VM cannot communicate with other VMs on the same bridge
                  </p>
                </div>
                <Switch checked={false} disabled />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Network Interface XML</CardTitle>
              <CardDescription>
                Raw libvirt XML for this network interface
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {`<interface type='network'>
  <mac address='${nic.mac || '52:54:00:00:00:00'}'/>
  <source network='${nic.network || 'default'}'/>
  <model type='${nic.type || 'virtio'}'/>
${(inboundAverage || outboundAverage) ? `  <bandwidth>
${inboundAverage ? `    <inbound average='${inboundAverage}'${inboundPeak ? ` peak='${inboundPeak}'` : ''}${inboundBurst ? ` burst='${inboundBurst}'` : ''}/>` : ''}
${outboundAverage ? `    <outbound average='${outboundAverage}'${outboundPeak ? ` peak='${outboundPeak}'` : ''}${outboundBurst ? ` burst='${outboundBurst}'` : ''}/>` : ''}
  </bandwidth>` : ''}</interface>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <div className="flex justify-between">
        <Button
          variant="destructive"
          size="sm"
          className="gap-1.5"
          onClick={() => setShowDeleteDialog(true)}
          disabled={detachMutation.isPending}
        >
          <Trash2 className="w-4 h-4" />
          Remove Interface
        </Button>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => {
              setSelectedNetwork(nic.network || 'default')
              setSelectedModel(nic.type || 'virtio')
              setHasChanges(false)
            }}
            disabled={!hasChanges || isPending}
          >
            Revert
          </Button>
          <Button
            onClick={() => updateMutation.mutate()}
            disabled={!hasChanges || isPending}
          >
            {updateMutation.isPending ? (
              <>
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                Applying...
              </>
            ) : (
              'Apply'
            )}
          </Button>
        </div>
      </div>

      {/* Remove Confirmation Dialog */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove Network Interface?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to remove this network interface with MAC address <strong className="font-mono">{nic.mac}</strong>?
              The VM may lose network connectivity.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => detachMutation.mutate()}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={detachMutation.isPending}
            >
              {detachMutation.isPending ? 'Removing...' : 'Remove Interface'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
