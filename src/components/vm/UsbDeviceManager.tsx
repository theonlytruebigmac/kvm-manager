import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs'
import { Usb, Plus, Minus, RefreshCw, Check, AlertCircle } from 'lucide-react'
import { toast } from 'sonner'
import type { UsbDevice, VM } from '@/lib/types'

interface UsbDeviceManagerProps {
  vm: VM
  trigger?: React.ReactNode
}

export function UsbDeviceManager({ vm, trigger }: UsbDeviceManagerProps) {
  const [open, setOpen] = useState(false)
  const queryClient = useQueryClient()
  const isRunning = vm.state === 'running'

  // Fetch all host USB devices
  const { data: hostDevices = [], isLoading: hostLoading, refetch: refetchHost } = useQuery<UsbDevice[]>({
    queryKey: ['usb-devices'],
    queryFn: () => api.listUsbDevices(),
    enabled: open,
    refetchInterval: open ? 5000 : false,
  })

  // Fetch USB devices attached to this VM
  const { data: vmDevices = [], isLoading: vmLoading, refetch: refetchVm } = useQuery<UsbDevice[]>({
    queryKey: ['vm-usb-devices', vm.id],
    queryFn: () => api.getVmUsbDevices(vm.id),
    enabled: open,
    refetchInterval: open ? 5000 : false,
  })

  // Attach USB device mutation
  const attachMutation = useMutation({
    mutationFn: ({ vendorId, productId }: { vendorId: string; productId: string }) =>
      api.attachUsbDevice(vm.id, vendorId, productId),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['usb-devices'] })
      queryClient.invalidateQueries({ queryKey: ['vm-usb-devices', vm.id] })
      toast.success('USB Device Attached', {
        description: `Device ${variables.vendorId}:${variables.productId} attached to ${vm.name}`,
      })
    },
    onError: (error: Error) => {
      toast.error('Failed to attach USB device', {
        description: error.message,
      })
    },
  })

  // Detach USB device mutation
  const detachMutation = useMutation({
    mutationFn: ({ vendorId, productId }: { vendorId: string; productId: string }) =>
      api.detachUsbDevice(vm.id, vendorId, productId),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['usb-devices'] })
      queryClient.invalidateQueries({ queryKey: ['vm-usb-devices', vm.id] })
      toast.success('USB Device Detached', {
        description: `Device ${variables.vendorId}:${variables.productId} detached from ${vm.name}`,
      })
    },
    onError: (error: Error) => {
      toast.error('Failed to detach USB device', {
        description: error.message,
      })
    },
  })

  const handleRefresh = () => {
    refetchHost()
    refetchVm()
  }

  // Filter available devices (not in use or in use by other VMs)
  const availableDevices = hostDevices.filter(d => !d.inUse || d.usedByVm !== vm.name)

  // Check if a device is attached to this VM
  const isAttached = (device: UsbDevice) => {
    return vmDevices.some(vd =>
      vd.vendorId === device.vendorId && vd.productId === device.productId
    )
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || (
          <Button variant="outline" size="sm">
            <Usb className="mr-2 h-4 w-4" />
            USB Devices
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-3xl max-h-[80vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Usb className="h-5 w-5" />
            USB Device Manager - {vm.name}
          </DialogTitle>
          <DialogDescription>
            {isRunning
              ? "Attach or detach USB devices from the running virtual machine."
              : "USB hot-plug requires the VM to be running."}
          </DialogDescription>
        </DialogHeader>

        {!isRunning ? (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <AlertCircle className="h-12 w-12 mb-4" />
            <p>Start the VM to manage USB devices</p>
          </div>
        ) : (
          <Tabs defaultValue="attached" className="w-full">
            <div className="flex items-center justify-between mb-4">
              <TabsList>
                <TabsTrigger value="attached">
                  Attached ({vmDevices.length})
                </TabsTrigger>
                <TabsTrigger value="available">
                  Available ({availableDevices.length})
                </TabsTrigger>
              </TabsList>
              <Button variant="outline" size="sm" onClick={handleRefresh}>
                <RefreshCw className="h-4 w-4 mr-2" />
                Refresh
              </Button>
            </div>

            <TabsContent value="attached">
              <ScrollArea className="h-[400px]">
                {vmLoading ? (
                  <p className="text-center text-muted-foreground py-8">Loading attached devices...</p>
                ) : vmDevices.length === 0 ? (
                  <p className="text-center text-muted-foreground py-8">
                    No USB devices attached to this VM
                  </p>
                ) : (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Device</TableHead>
                        <TableHead>Vendor:Product</TableHead>
                        <TableHead className="text-right">Actions</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {vmDevices.map((device) => (
                        <TableRow key={`${device.vendorId}:${device.productId}`}>
                          <TableCell>
                            <div>
                              <p className="font-medium">
                                {device.productName || device.description || 'USB Device'}
                              </p>
                              <p className="text-sm text-muted-foreground">
                                {device.vendorName || 'Unknown Vendor'}
                              </p>
                            </div>
                          </TableCell>
                          <TableCell>
                            <code className="text-sm">{device.vendorId}:{device.productId}</code>
                          </TableCell>
                          <TableCell className="text-right">
                            <Button
                              variant="destructive"
                              size="sm"
                              onClick={() => detachMutation.mutate({
                                vendorId: device.vendorId,
                                productId: device.productId,
                              })}
                              disabled={detachMutation.isPending}
                            >
                              <Minus className="h-4 w-4 mr-1" />
                              Detach
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                )}
              </ScrollArea>
            </TabsContent>

            <TabsContent value="available">
              <ScrollArea className="h-[400px]">
                {hostLoading ? (
                  <p className="text-center text-muted-foreground py-8">Loading host USB devices...</p>
                ) : availableDevices.length === 0 ? (
                  <p className="text-center text-muted-foreground py-8">
                    No available USB devices found
                  </p>
                ) : (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Device</TableHead>
                        <TableHead>Vendor:Product</TableHead>
                        <TableHead>Status</TableHead>
                        <TableHead className="text-right">Actions</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {availableDevices.map((device) => {
                        const attached = isAttached(device)
                        return (
                          <TableRow key={`${device.vendorId}:${device.productId}`}>
                            <TableCell>
                              <div>
                                <p className="font-medium">
                                  {device.productName || device.description || 'USB Device'}
                                </p>
                                <p className="text-sm text-muted-foreground">
                                  {device.vendorName || 'Unknown Vendor'}
                                </p>
                                {device.speed && (
                                  <p className="text-xs text-muted-foreground">{device.speed}</p>
                                )}
                              </div>
                            </TableCell>
                            <TableCell>
                              <code className="text-sm">{device.vendorId}:{device.productId}</code>
                            </TableCell>
                            <TableCell>
                              {device.inUse ? (
                                <Badge variant="secondary">
                                  In use by {device.usedByVm || 'VM'}
                                </Badge>
                              ) : attached ? (
                                <Badge variant="default">
                                  <Check className="h-3 w-3 mr-1" />
                                  Attached
                                </Badge>
                              ) : (
                                <Badge variant="outline">Available</Badge>
                              )}
                            </TableCell>
                            <TableCell className="text-right">
                              {attached ? (
                                <Button
                                  variant="destructive"
                                  size="sm"
                                  onClick={() => detachMutation.mutate({
                                    vendorId: device.vendorId,
                                    productId: device.productId,
                                  })}
                                  disabled={detachMutation.isPending}
                                >
                                  <Minus className="h-4 w-4 mr-1" />
                                  Detach
                                </Button>
                              ) : (
                                <Button
                                  variant="default"
                                  size="sm"
                                  onClick={() => attachMutation.mutate({
                                    vendorId: device.vendorId,
                                    productId: device.productId,
                                  })}
                                  disabled={device.inUse || attachMutation.isPending}
                                >
                                  <Plus className="h-4 w-4 mr-1" />
                                  Attach
                                </Button>
                              )}
                            </TableCell>
                          </TableRow>
                        )
                      })}
                    </TableBody>
                  </Table>
                )}
              </ScrollArea>
            </TabsContent>
          </Tabs>
        )}
      </DialogContent>
    </Dialog>
  )
}
