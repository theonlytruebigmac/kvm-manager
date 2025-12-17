import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { InterfaceType, DirectMode, VM } from '@/lib/types'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from 'sonner'
import { Network, Plus, Loader2 } from 'lucide-react'

interface AddNetworkInterfaceDialogProps {
  vm: VM
  children?: React.ReactNode
}

const INTERFACE_TYPES: { value: InterfaceType; label: string }[] = [
  { value: 'network', label: 'Virtual Network (NAT)' },
  { value: 'bridge', label: 'Bridge' },
  { value: 'direct', label: 'Macvtap' },
  { value: 'ovs', label: 'Open vSwitch' },
]

const DIRECT_MODES: { value: DirectMode; label: string }[] = [
  { value: 'bridge', label: 'Bridge' },
  { value: 'vepa', label: 'VEPA' },
  { value: 'private', label: 'Private' },
  { value: 'passthrough', label: 'Passthrough' },
]

const NIC_MODELS = [
  { value: 'virtio', label: 'VirtIO (recommended)' },
  { value: 'e1000e', label: 'Intel e1000e' },
  { value: 'e1000', label: 'Intel e1000' },
  { value: 'rtl8139', label: 'RTL8139' },
]

export function AddNetworkInterfaceDialog({ vm, children }: AddNetworkInterfaceDialogProps) {
  const [open, setOpen] = useState(false)
  const [interfaceType, setInterfaceType] = useState<InterfaceType>('network')
  const [source, setSource] = useState('default')
  const [model, setModel] = useState('virtio')
  const [macAddress, setMacAddress] = useState('')
  const [directMode, setDirectMode] = useState<DirectMode>('bridge')
  const [vlanId, setVlanId] = useState('')
  const [mtu, setMtu] = useState('')
  const [showAdvanced, setShowAdvanced] = useState(false)

  const queryClient = useQueryClient()

  // Fetch networks for 'network' type
  const { data: networks = [] } = useQuery({
    queryKey: ['networks'],
    queryFn: () => api.getNetworks(),
    enabled: open && interfaceType === 'network',
  })

  // Fetch host interfaces for 'direct' type
  const { data: hostInterfaces = [], isLoading: loadingInterfaces } = useQuery({
    queryKey: ['host-interfaces'],
    queryFn: () => api.listHostInterfaces(),
    enabled: open && (interfaceType === 'direct' || interfaceType === 'bridge'),
  })

  const attachMutation = useMutation({
    mutationFn: () => {
      if (interfaceType === 'network') {
        return api.attachInterface(vm.id, source, model, macAddress || undefined)
      }
      return api.attachInterfaceAdvanced(vm.id, interfaceType, source, model, {
        macAddress: macAddress || undefined,
        sourceMode: interfaceType === 'direct' ? directMode : undefined,
        vlanId: vlanId ? parseInt(vlanId) : undefined,
        mtu: mtu ? parseInt(mtu) : undefined,
      })
    },
    onSuccess: (mac) => {
      toast.success(`Network interface attached with MAC ${mac}`)
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      setOpen(false)
      resetForm()
    },
    onError: (err: Error) => {
      toast.error(`Failed to attach interface: ${err.message}`)
    },
  })

  const resetForm = () => {
    setInterfaceType('network')
    setSource('default')
    setModel('virtio')
    setMacAddress('')
    setDirectMode('bridge')
    setVlanId('')
    setMtu('')
    setShowAdvanced(false)
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {children || (
          <Button variant="outline" size="sm">
            <Plus className="w-4 h-4 mr-2" />
            Add Network Interface
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Network className="w-5 h-5" />
            Add Network Interface
          </DialogTitle>
          <DialogDescription>
            Attach a new network interface to {vm.name}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 mt-4">
          {/* Interface Type */}
          <div className="space-y-1.5">
            <Label>Type</Label>
            <Select value={interfaceType} onValueChange={(v) => {
              setInterfaceType(v as InterfaceType)
              setSource('')
            }}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {INTERFACE_TYPES.map(t => (
                  <SelectItem key={t.value} value={t.value}>{t.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Source Selection */}
          <div className="space-y-1.5">
            <Label>
              {interfaceType === 'network' ? 'Network' : interfaceType === 'ovs' ? 'Bridge Name' : 'Interface'}
            </Label>

            {interfaceType === 'network' && (
              <Select value={source} onValueChange={setSource}>
                <SelectTrigger>
                  <SelectValue placeholder="Select network" />
                </SelectTrigger>
                <SelectContent>
                  {networks.map(n => (
                    <SelectItem key={n.name} value={n.name}>
                      {n.name} {!n.active && <span className="text-muted-foreground">(inactive)</span>}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}

            {(interfaceType === 'direct' || interfaceType === 'bridge') && (
              loadingInterfaces ? (
                <div className="flex items-center gap-2 py-2 text-sm text-muted-foreground">
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Loading interfaces...
                </div>
              ) : hostInterfaces.length === 0 ? (
                <p className="text-sm text-yellow-600">No host interfaces found</p>
              ) : (
                <Select value={source} onValueChange={setSource}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select interface" />
                  </SelectTrigger>
                  <SelectContent>
                    {hostInterfaces.map(iface => (
                      <SelectItem key={iface.name} value={iface.name}>
                        <span className="font-mono">{iface.name}</span>
                        {iface.speed && <span className="text-muted-foreground ml-2">{iface.speed >= 1000 ? `${iface.speed / 1000}G` : `${iface.speed}M`}</span>}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )
            )}

            {interfaceType === 'ovs' && (
              <Input placeholder="ovsbr0" value={source} onChange={e => setSource(e.target.value)} />
            )}
          </div>

          {/* Direct Mode (for macvtap) */}
          {interfaceType === 'direct' && (
            <div className="space-y-1.5">
              <Label>Mode</Label>
              <Select value={directMode} onValueChange={(v) => setDirectMode(v as DirectMode)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {DIRECT_MODES.map(m => (
                    <SelectItem key={m.value} value={m.value}>{m.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {directMode === 'passthrough' && (
                <p className="text-xs text-yellow-600">⚠️ Passthrough gives VM exclusive NIC access</p>
              )}
            </div>
          )}

          {/* NIC Model */}
          <div className="space-y-1.5">
            <Label>Model</Label>
            <Select value={model} onValueChange={setModel}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {NIC_MODELS.map(m => (
                  <SelectItem key={m.value} value={m.value}>{m.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Advanced Options Toggle */}
          <button
            type="button"
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="text-sm text-muted-foreground hover:text-foreground"
          >
            {showAdvanced ? '▾ Hide' : '▸ Show'} advanced options
          </button>

          {showAdvanced && (
            <div className="space-y-3 pl-4 border-l-2 border-muted">
              <div className="space-y-1">
                <Label className="text-xs">MAC Address</Label>
                <Input
                  placeholder="Auto-generated if empty"
                  value={macAddress}
                  onChange={e => setMacAddress(e.target.value)}
                  className="h-8"
                />
              </div>
              {interfaceType === 'ovs' && (
                <div className="space-y-1">
                  <Label className="text-xs">VLAN ID</Label>
                  <Input type="number" min={0} max={4095} placeholder="Optional" value={vlanId} onChange={e => setVlanId(e.target.value)} className="h-8" />
                </div>
              )}
              <div className="space-y-1">
                <Label className="text-xs">MTU</Label>
                <Input type="number" min={576} max={65535} placeholder="1500" value={mtu} onChange={e => setMtu(e.target.value)} className="h-8" />
              </div>
            </div>
          )}

        </div>

        <div className="flex justify-end gap-2 mt-4">
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => attachMutation.mutate()}
            disabled={!source || attachMutation.isPending}
          >
            {attachMutation.isPending ? (
              <Loader2 className="w-4 h-4 animate-spin mr-2" />
            ) : (
              <Plus className="w-4 h-4 mr-2" />
            )}
            Add Interface
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
