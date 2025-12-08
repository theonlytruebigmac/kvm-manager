import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Network, Plus, Play, Square, Trash2, Wifi } from 'lucide-react'
import type { Network as NetworkType, NetworkConfig } from '@/lib/types'

export function NetworkManager() {
  const queryClient = useQueryClient()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [selectedNetwork, setSelectedNetwork] = useState<NetworkType | null>(null)

  const [networkName, setNetworkName] = useState('')
  const [bridgeName, setBridgeName] = useState('virbr')
  const [forwardMode, setForwardMode] = useState<'nat' | 'route' | 'bridge' | 'isolated'>('nat')
  const [ipAddress, setIpAddress] = useState('192.168.100.1')
  const [netmask, setNetmask] = useState('255.255.255.0')
  const [dhcpStart, setDhcpStart] = useState('192.168.100.2')
  const [dhcpEnd, setDhcpEnd] = useState('192.168.100.254')
  const [autostart, setAutostart] = useState(true)

  // Query networks
  const { data: networks = [], isLoading, error } = useQuery({
    queryKey: ['networks'],
    queryFn: api.getNetworks,
    refetchInterval: 10000,
  })

  // Create network mutation
  const createMutation = useMutation({
    mutationFn: (config: NetworkConfig) => api.createNetwork(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['networks'] })
      toast.success(`Network "${networkName}" created successfully`)
      setShowCreateDialog(false)
      resetForm()
    },
    onError: (error) => {
      toast.error(`Failed to create network: ${error}`)
    },
  })

  // Delete network mutation
  const deleteMutation = useMutation({
    mutationFn: (networkName: string) => api.deleteNetwork(networkName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['networks'] })
      toast.success('Network deleted successfully')
      setShowDeleteDialog(false)
      setSelectedNetwork(null)
    },
    onError: (error) => {
      toast.error(`Failed to delete network: ${error}`)
    },
  })

  // Start network mutation
  const startMutation = useMutation({
    mutationFn: (networkName: string) => api.startNetwork(networkName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['networks'] })
      toast.success('Network started successfully')
    },
    onError: (error) => {
      toast.error(`Failed to start network: ${error}`)
    },
  })

  // Stop network mutation
  const stopMutation = useMutation({
    mutationFn: (networkName: string) => api.stopNetwork(networkName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['networks'] })
      toast.success('Network stopped successfully')
    },
    onError: (error) => {
      toast.error(`Failed to stop network: ${error}`)
    },
  })

  const resetForm = () => {
    setNetworkName('')
    setBridgeName('virbr')
    setForwardMode('nat')
    setIpAddress('192.168.100.1')
    setNetmask('255.255.255.0')
    setDhcpStart('192.168.100.2')
    setDhcpEnd('192.168.100.254')
    setAutostart(true)
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Card className="max-w-md">
          <CardHeader>
            <CardTitle>Network Management</CardTitle>
            <CardDescription>Error loading networks: {String(error)}</CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Network Management</h1>
          <p className="text-muted-foreground">Manage virtual networks</p>
        </div>
        <AlertDialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
          <AlertDialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Create Network
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
            <AlertDialogHeader>
              <AlertDialogTitle>Create Virtual Network</AlertDialogTitle>
              <AlertDialogDescription>
                Create a new virtual network for VMs to connect to.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <div className="space-y-4 py-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="network-name">Network Name</Label>
                  <Input
                    id="network-name"
                    value={networkName}
                    onChange={(e) => setNetworkName(e.target.value)}
                    placeholder="e.g., isolated-net"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="bridge-name">Bridge Name</Label>
                  <Input
                    id="bridge-name"
                    value={bridgeName}
                    onChange={(e) => setBridgeName(e.target.value)}
                    placeholder="e.g., virbr1"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="forward-mode">Forward Mode</Label>
                <Select value={forwardMode} onValueChange={(v) => setForwardMode(v as typeof forwardMode)}>
                  <SelectTrigger id="forward-mode">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="nat">NAT (Network Address Translation)</SelectItem>
                    <SelectItem value="route">Route</SelectItem>
                    <SelectItem value="bridge">Bridge</SelectItem>
                    <SelectItem value="isolated">Isolated</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {forwardMode === 'nat' && 'VMs can access external networks via host IP'}
                  {forwardMode === 'route' && 'VMs are routed through the host'}
                  {forwardMode === 'bridge' && 'VMs connect directly to physical network'}
                  {forwardMode === 'isolated' && 'VMs can only communicate with each other'}
                </p>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="ip-address">IP Address</Label>
                  <Input
                    id="ip-address"
                    value={ipAddress}
                    onChange={(e) => setIpAddress(e.target.value)}
                    placeholder="192.168.100.1"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="netmask">Netmask</Label>
                  <Input
                    id="netmask"
                    value={netmask}
                    onChange={(e) => setNetmask(e.target.value)}
                    placeholder="255.255.255.0"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label className="text-base font-medium">DHCP Configuration</Label>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="dhcp-start">DHCP Start</Label>
                    <Input
                      id="dhcp-start"
                      value={dhcpStart}
                      onChange={(e) => setDhcpStart(e.target.value)}
                      placeholder="192.168.100.2"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="dhcp-end">DHCP End</Label>
                    <Input
                      id="dhcp-end"
                      value={dhcpEnd}
                      onChange={(e) => setDhcpEnd(e.target.value)}
                      placeholder="192.168.100.254"
                    />
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="autostart"
                  checked={autostart}
                  onCheckedChange={(checked) => setAutostart(checked as boolean)}
                />
                <Label htmlFor="autostart" className="text-sm cursor-pointer">
                  Start network automatically with host
                </Label>
              </div>
            </div>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction
                onClick={() => createMutation.mutate({
                  name: networkName,
                  bridgeName,
                  forwardMode,
                  ipAddress,
                  netmask,
                  dhcpStart,
                  dhcpEnd,
                  autostart,
                })}
                disabled={!networkName.trim() || createMutation.isPending}
              >
                {createMutation.isPending ? 'Creating...' : 'Create Network'}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>

      {/* Networks List */}
      <div className="grid grid-cols-1 gap-4">
        {isLoading ? (
          <Card>
            <CardContent className="p-6">
              <p className="text-sm text-muted-foreground">Loading networks...</p>
            </CardContent>
          </Card>
        ) : networks.length === 0 ? (
          <Card>
            <CardContent className="p-12 text-center">
              <Wifi className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <p className="text-muted-foreground mb-4">No virtual networks found</p>
              <Button onClick={() => setShowCreateDialog(true)}>
                <Plus className="mr-2 h-4 w-4" />
                Create First Network
              </Button>
            </CardContent>
          </Card>
        ) : (
          networks.map((network) => (
            <Card key={network.uuid}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <Network className="h-6 w-6 text-primary" />
                    <div>
                      <CardTitle>{network.name}</CardTitle>
                      <CardDescription>Bridge: {network.bridge}</CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={network.active ? 'default' : 'secondary'}>
                      {network.active ? 'Active' : 'Inactive'}
                    </Badge>
                    {network.autostart && (
                      <Badge variant="outline">Autostart</Badge>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <p className="text-muted-foreground">UUID</p>
                      <p className="font-mono text-xs">{network.uuid.slice(0, 8)}...</p>
                    </div>
                    {network.ipRange && (
                      <div>
                        <p className="text-muted-foreground">IP Range</p>
                        <p className="font-medium">{network.ipRange}</p>
                      </div>
                    )}
                  </div>

                  <div className="flex gap-2">
                    {network.active ? (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => stopMutation.mutate(network.name)}
                        disabled={stopMutation.isPending}
                      >
                        <Square className="mr-2 h-4 w-4" />
                        Stop
                      </Button>
                    ) : (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => startMutation.mutate(network.name)}
                        disabled={startMutation.isPending}
                      >
                        <Play className="mr-2 h-4 w-4" />
                        Start
                      </Button>
                    )}
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => {
                        setSelectedNetwork(network)
                        setShowDeleteDialog(true)
                      }}
                      disabled={network.name === 'default' || deleteMutation.isPending}
                    >
                      <Trash2 className="mr-2 h-4 w-4" />
                      Delete
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))
        )}
      </div>

      {/* Delete Confirmation */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Network</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete network "{selectedNetwork?.name}"?
              {selectedNetwork?.active && ' The network is currently active and will be stopped first.'}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setSelectedNetwork(null)}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => selectedNetwork && deleteMutation.mutate(selectedNetwork.name)}
              disabled={deleteMutation.isPending}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleteMutation.isPending ? 'Deleting...' : 'Delete Network'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
