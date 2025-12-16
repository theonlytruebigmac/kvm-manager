import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Keyboard, Mouse, Tablet, Plus, Trash2, Gamepad2, MonitorUp, RefreshCw, AlertTriangle } from 'lucide-react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import type { VM } from '@/lib/types'
import { useState } from 'react'

interface InputEditorProps {
  vm: VM
  compact?: boolean
}

interface InputDevice {
  type: 'tablet' | 'mouse' | 'keyboard'
  bus: 'usb' | 'ps2' | 'virtio'
}

export function InputEditor({ vm, compact }: InputEditorProps) {
  const queryClient = useQueryClient()
  const [grabAll, setGrabAll] = useState(true)

  // Default input devices (typical QEMU defaults)
  // TODO: Parse actual input devices from VM configuration
  const inputDevices: InputDevice[] = [
    { type: 'tablet', bus: 'usb' },
    { type: 'keyboard', bus: 'ps2' },
    { type: 'mouse', bus: 'ps2' },
  ]

  // Query available evdev devices on the host
  const { data: availableEvdevDevices = [], isLoading: loadingAvailable, refetch: refetchAvailable } = useQuery({
    queryKey: ['evdev-devices'],
    queryFn: () => api.listEvdevDevices(),
  })

  // Query evdev devices attached to this VM
  const { data: attachedEvdevDevices = [], isLoading: loadingAttached, refetch: refetchAttached } = useQuery({
    queryKey: ['vm-evdev-devices', vm.id],
    queryFn: () => api.getVmEvdevDevices(vm.id),
  })

  // Attach evdev device mutation
  const attachEvdevMutation = useMutation({
    mutationFn: (devicePath: string) => api.attachEvdev(vm.id, devicePath, grabAll),
    onSuccess: () => {
      toast.success('Evdev device attached')
      queryClient.invalidateQueries({ queryKey: ['vm-evdev-devices', vm.id] })
      refetchAttached()
    },
    onError: (err) => {
      toast.error(`Failed to attach evdev device: ${err}`)
    },
  })

  // Detach evdev device mutation
  const detachEvdevMutation = useMutation({
    mutationFn: (devicePath: string) => api.detachEvdev(vm.id, devicePath),
    onSuccess: () => {
      toast.success('Evdev device detached')
      queryClient.invalidateQueries({ queryKey: ['vm-evdev-devices', vm.id] })
      refetchAttached()
    },
    onError: (err) => {
      toast.error(`Failed to detach evdev device: ${err}`)
    },
  })

  // Filter out already attached devices from available list
  const attachedPaths = new Set(attachedEvdevDevices.map(d => d.path))
  const unattachedDevices = availableEvdevDevices.filter(d => !attachedPaths.has(d.path))

  const getIcon = (type: string) => {
    switch (type) {
      case 'keyboard':
        return Keyboard
      case 'mouse':
        return Mouse
      case 'tablet':
        return Tablet
      case 'joystick':
        return Gamepad2
      default:
        return Keyboard
    }
  }

  const getEvdevIcon = (deviceType: string) => {
    switch (deviceType) {
      case 'keyboard':
        return Keyboard
      case 'mouse':
        return Mouse
      case 'joystick':
        return Gamepad2
      default:
        return MonitorUp
    }
  }

  // Compact mode shows inline summary
  if (compact) {
    const evdevCount = attachedEvdevDevices.length
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Keyboard className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">Input Devices</span>
          </div>
          <div className="flex gap-2">
            <Badge variant="outline" className="text-xs">
              {inputDevices.length} virtual
            </Badge>
            {evdevCount > 0 && (
              <Badge variant="secondary" className="text-xs">
                {evdevCount} passthrough
              </Badge>
            )}
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          {inputDevices.map((device, index) => {
            const Icon = getIcon(device.type)
            return (
              <div
                key={index}
                className="flex items-center gap-1.5 px-2 py-1 border rounded-md bg-muted/50 text-xs"
              >
                <Icon className="w-3.5 h-3.5 text-muted-foreground" />
                <span className="capitalize">{device.type}</span>
                <span className="text-muted-foreground">({device.bus})</span>
              </div>
            )
          })}
          {attachedEvdevDevices.map((device, index) => {
            const Icon = getEvdevIcon(device.deviceType)
            return (
              <div
                key={`evdev-${index}`}
                className="flex items-center gap-1.5 px-2 py-1 border rounded-md bg-orange-500/10 border-orange-500/30 text-xs"
              >
                <Icon className="w-3.5 h-3.5 text-orange-500" />
                <span>{device.name}</span>
                <span className="text-orange-500">(evdev)</span>
              </div>
            )
          })}
        </div>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold mb-2 flex items-center gap-2">
            <Keyboard className="w-6 h-6" />
            Input Devices
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure virtual input devices for this VM
          </p>
        </div>
        <Badge variant="outline">
          {inputDevices.length} Devices
        </Badge>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Input Device List</CardTitle>
              <CardDescription>
                Virtual input devices attached to this VM
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {inputDevices.map((device, index) => {
                const Icon = getIcon(device.type)
                return (
                  <div
                    key={index}
                    className="flex items-center justify-between p-3 rounded-lg border bg-card hover:bg-muted/50 transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <Icon className="w-5 h-5 text-muted-foreground" />
                      <div>
                        <p className="font-medium capitalize">{device.type}</p>
                        <p className="text-xs text-muted-foreground uppercase">
                          {device.bus} bus
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant="secondary">{device.bus.toUpperCase()}</Badge>
                      <Button variant="ghost" size="icon" className="h-8 w-8" disabled>
                        <Trash2 className="w-4 h-4 text-destructive" />
                      </Button>
                    </div>
                  </div>
                )
              })}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>About Input Devices</CardTitle>
              <CardDescription>
                Understanding virtual input device types
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="p-3 bg-muted rounded-lg">
                  <div className="flex items-center gap-2 mb-1">
                    <Tablet className="w-4 h-4" />
                    <h4 className="font-medium text-sm">USB Tablet</h4>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Provides absolute pointer positioning. Recommended for graphical guests
                    to prevent mouse grabbing issues.
                  </p>
                </div>

                <div className="p-3 bg-muted rounded-lg">
                  <div className="flex items-center gap-2 mb-1">
                    <Mouse className="w-4 h-4" />
                    <h4 className="font-medium text-sm">PS/2 Mouse</h4>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Standard relative pointer device. Compatible with all operating systems
                    but may require mouse grabbing.
                  </p>
                </div>

                <div className="p-3 bg-muted rounded-lg">
                  <div className="flex items-center gap-2 mb-1">
                    <Keyboard className="w-4 h-4" />
                    <h4 className="font-medium text-sm">PS/2 Keyboard</h4>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Standard keyboard input. Compatible with all operating systems
                    including BIOS/UEFI firmware.
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div>
                <CardTitle>Add Input Device</CardTitle>
                <CardDescription>
                  Add additional input devices
                </CardDescription>
              </div>
              <Button variant="outline" size="sm" className="gap-1.5" disabled>
                <Plus className="w-4 h-4" />
                Add Device
              </Button>
            </CardHeader>
          </Card>

          {/* Evdev Passthrough Section */}
          <Card className="border-orange-500/30">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <MonitorUp className="w-5 h-5 text-orange-500" />
                  <div>
                    <CardTitle>Evdev Input Passthrough</CardTitle>
                    <CardDescription>
                      Pass physical input devices directly to the VM for low-latency input
                    </CardDescription>
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => {
                    refetchAvailable()
                    refetchAttached()
                  }}
                  disabled={loadingAvailable || loadingAttached}
                >
                  <RefreshCw className={`w-4 h-4 ${(loadingAvailable || loadingAttached) ? 'animate-spin' : ''}`} />
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Warning banner */}
              <div className="flex items-start gap-2 p-3 bg-orange-500/10 border border-orange-500/30 rounded-lg">
                <AlertTriangle className="w-4 h-4 text-orange-500 mt-0.5 flex-shrink-0" />
                <div className="text-xs text-muted-foreground">
                  <p className="font-medium text-orange-500 mb-1">Important Notes:</p>
                  <ul className="list-disc list-inside space-y-0.5">
                    <li>Evdev passthrough requires appropriate permissions (input group)</li>
                    <li>Devices will be grabbed and unavailable to the host while attached</li>
                    <li>VM must be running to attach/detach devices</li>
                    <li>Best for gaming controllers, specialized keyboards, or GPU passthrough setups</li>
                  </ul>
                </div>
              </div>

              {/* Grab All Toggle */}
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <div>
                  <Label htmlFor="grab-all" className="font-medium">Exclusive Grab</Label>
                  <p className="text-xs text-muted-foreground">
                    Grab all input from device (prevents host from receiving input)
                  </p>
                </div>
                <Switch
                  id="grab-all"
                  checked={grabAll}
                  onCheckedChange={setGrabAll}
                />
              </div>

              {/* Attached Devices */}
              {attachedEvdevDevices.length > 0 && (
                <div className="space-y-2">
                  <h4 className="text-sm font-medium">Attached Devices</h4>
                  {attachedEvdevDevices.map((device) => {
                    const Icon = getEvdevIcon(device.deviceType)
                    return (
                      <div
                        key={device.id}
                        className="flex items-center justify-between p-3 rounded-lg border border-orange-500/30 bg-orange-500/5"
                      >
                        <div className="flex items-center gap-3">
                          <Icon className="w-5 h-5 text-orange-500" />
                          <div>
                            <p className="font-medium text-sm">{device.name}</p>
                            <p className="text-xs text-muted-foreground">{device.path}</p>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <Badge variant="outline" className="text-orange-500 border-orange-500/50">
                            {device.deviceType}
                          </Badge>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => detachEvdevMutation.mutate(device.path)}
                            disabled={detachEvdevMutation.isPending || vm.state !== 'running'}
                          >
                            <Trash2 className="w-4 h-4 text-destructive" />
                          </Button>
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}

              {/* Available Devices */}
              <div className="space-y-2">
                <h4 className="text-sm font-medium">Available Devices</h4>
                {loadingAvailable ? (
                  <p className="text-sm text-muted-foreground">Loading available devices...</p>
                ) : unattachedDevices.length === 0 ? (
                  <p className="text-sm text-muted-foreground">
                    No evdev devices available. Make sure you have permission to access /dev/input/ devices.
                  </p>
                ) : (
                  <div className="space-y-2 max-h-60 overflow-y-auto">
                    {unattachedDevices.map((device) => {
                      const Icon = getEvdevIcon(device.deviceType)
                      return (
                        <div
                          key={device.id}
                          className="flex items-center justify-between p-3 rounded-lg border bg-card hover:bg-muted/50 transition-colors"
                        >
                          <div className="flex items-center gap-3">
                            <Icon className="w-5 h-5 text-muted-foreground" />
                            <div>
                              <p className="font-medium text-sm">{device.name}</p>
                              <p className="text-xs text-muted-foreground">{device.path}</p>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            <Badge variant="secondary">{device.deviceType}</Badge>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => attachEvdevMutation.mutate(device.path)}
                              disabled={attachEvdevMutation.isPending || vm.state !== 'running'}
                            >
                              <Plus className="w-4 h-4 mr-1" />
                              Attach
                            </Button>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                )}
              </div>

              {vm.state !== 'running' && (
                <p className="text-xs text-amber-500 text-center">
                  VM must be running to attach or detach evdev devices
                </p>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Input Devices XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for input devices
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {[
                  ...inputDevices.map(device =>
                    `<input type='${device.type}' bus='${device.bus}'/>`
                  ),
                  ...attachedEvdevDevices.map(device =>
                    `<input type='evdev'>\n  <source dev='${device.path}' grab='all' grabToggle='ctrl-ctrl' repeat='on'/>\n</input>`
                  )
                ].join('\n')}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <div className="flex justify-end gap-2">
        <Button variant="outline" disabled>Revert</Button>
        <Button disabled>Apply</Button>
      </div>
    </div>
  )
}
