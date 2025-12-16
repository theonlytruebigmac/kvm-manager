import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Monitor, Trash2, Eye, EyeOff, ExternalLink, AlertCircle, Loader2, CheckCircle, Usb } from 'lucide-react'
import { useState, useEffect } from 'react'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import type { VM } from '@/lib/types'

interface GraphicsEditorProps {
  vm: VM
  compact?: boolean
}

export function GraphicsEditor({ vm, compact }: GraphicsEditorProps) {
  const queryClient = useQueryClient()
  const [showPassword, setShowPassword] = useState(false)
  const originalType = vm.graphics?.type || 'vnc'

  // Editable state
  const [graphicsType, setGraphicsType] = useState(originalType)
  const [listenAddress, setListenAddress] = useState('127.0.0.1')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  // USB Redirection state
  const [usbRedirCount, setUsbRedirCount] = useState(4)
  const [usbRedirLoading, setUsbRedirLoading] = useState(false)

  // Query USB redirection status
  const { data: usbRedir, refetch: refetchUsbRedir } = useQuery({
    queryKey: ['usbRedirection', vm.id],
    queryFn: () => api.getUsbRedirection(vm.id),
    staleTime: 5000,
  })

  const isRunning = vm.state === 'running'
  const hasChanges = graphicsType !== originalType

  useEffect(() => {
    setGraphicsType(vm.graphics?.type || 'vnc')
  }, [vm])

  const handleOpenConsole = async () => {
    try {
      await api.openConsoleWindow(vm.id, vm.name)
    } catch (error) {
      toast.error(`Failed to open console: ${(error as Error).message}`)
    }
  }

  const handleApply = async () => {
    setLoading(true)
    setError(null)
    setSuccess(false)

    try {
      await api.attachGraphics(vm.id, graphicsType, listenAddress)
      setSuccess(true)
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      setTimeout(() => setSuccess(false), 3000)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleRevert = () => {
    setGraphicsType(originalType)
    setError(null)
  }

  // Compact mode shows inline summary
  if (compact) {
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Monitor className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">Graphics</span>
            <Badge variant={graphicsType === 'spice' ? 'default' : 'secondary'} className="text-xs">
              {graphicsType.toUpperCase()}
            </Badge>
          </div>
          {isRunning && (
            <Button size="sm" variant="outline" className="h-7 text-xs gap-1" onClick={handleOpenConsole}>
              <ExternalLink className="w-3 h-3" />
              Console
            </Button>
          )}
        </div>

        <div className="flex items-center gap-2">
          <Select value={graphicsType} onValueChange={setGraphicsType} disabled={isRunning}>
            <SelectTrigger className="h-8 w-32 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="vnc">VNC</SelectItem>
              <SelectItem value="spice">SPICE</SelectItem>
            </SelectContent>
          </Select>
          <span className="text-xs text-muted-foreground">Listen: {listenAddress}</span>
        </div>

        {hasChanges && !isRunning && (
          <div className="flex gap-2">
            <Button size="sm" onClick={handleApply} disabled={loading}>
              {loading && <Loader2 className="mr-1 h-3 w-3 animate-spin" />}
              Apply
            </Button>
            <Button size="sm" variant="ghost" onClick={handleRevert}>
              Revert
            </Button>
          </div>
        )}

        {error && <p className="text-sm text-red-500">{error}</p>}
        {success && <p className="text-sm text-green-500">Updated successfully</p>}
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold mb-2 flex items-center gap-2">
            <Monitor className="w-6 h-6" />
            Graphics Device
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure remote display access for this VM
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant={graphicsType === 'spice' ? 'default' : 'secondary'}>
            {graphicsType.toUpperCase()}
          </Badge>
          {vm.state === 'running' && (
            <Button size="sm" className="gap-1.5" onClick={handleOpenConsole}>
              <ExternalLink className="w-4 h-4" />
              Open Console
            </Button>
          )}
        </div>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Display Type</CardTitle>
              <CardDescription>
                Choose between VNC and SPICE display protocols
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {isRunning && (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Graphics type changes require the VM to be stopped.
                  </AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label htmlFor="type">Graphics Type</Label>
                <Select
                  value={graphicsType}
                  onValueChange={setGraphicsType}
                  disabled={isRunning}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="vnc">VNC</SelectItem>
                    <SelectItem value="spice">SPICE (Recommended)</SelectItem>
                    <SelectItem value="none">None</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  SPICE offers better performance and features. VNC is more compatible.
                </p>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="listen">Listen Type</Label>
                  <Select value="address" disabled>
                    <SelectTrigger>
                      <SelectValue placeholder="Select listen type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="address">Address</SelectItem>
                      <SelectItem value="socket">Unix Socket</SelectItem>
                      <SelectItem value="none">None (Local only)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="address">Listen Address</Label>
                  <Select
                    value={listenAddress}
                    onValueChange={setListenAddress}
                    disabled={isRunning}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select address" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="127.0.0.1">Localhost only (127.0.0.1)</SelectItem>
                      <SelectItem value="0.0.0.0">All interfaces (0.0.0.0)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Use 0.0.0.0 for remote access (requires password)
                  </p>
                </div>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="port">Port</Label>
                  <Input
                    id="port"
                    value={vm.vncPort || 'Auto'}
                    readOnly
                    className="font-mono"
                  />
                  <p className="text-xs text-muted-foreground">
                    Port is auto-assigned when VM starts
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>Autoport</Label>
                  <div className="flex items-center h-10">
                    <Switch checked={true} disabled />
                    <span className="ml-2 text-sm text-muted-foreground">Enabled</span>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {graphicsType === 'spice' && (
            <Card>
              <CardHeader>
                <CardTitle>SPICE Features</CardTitle>
                <CardDescription>
                  Advanced SPICE protocol settings
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <Label>OpenGL Acceleration</Label>
                    <p className="text-xs text-muted-foreground">
                      Enable GPU rendering in SPICE
                    </p>
                  </div>
                  <Switch checked={false} disabled />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label>File Transfer</Label>
                    <p className="text-xs text-muted-foreground">
                      Allow drag-and-drop file transfer to VM
                    </p>
                  </div>
                  <Switch checked={true} disabled />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label>Copy/Paste</Label>
                    <p className="text-xs text-muted-foreground">
                      Share clipboard between host and guest
                    </p>
                  </div>
                  <Switch checked={true} disabled />
                </div>
              </CardContent>
            </Card>
          )}

          {graphicsType === 'spice' && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Usb className="w-5 h-5" />
                  USB Redirection
                </CardTitle>
                <CardDescription>
                  Redirect USB devices from SPICE client to VM
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>USB Redirection Channels</Label>
                    <p className="text-xs text-muted-foreground">
                      {usbRedir?.enabled
                        ? `${usbRedir.channelCount} channel(s) configured`
                        : 'USB redirection not enabled'}
                    </p>
                  </div>
                  <Badge variant={usbRedir?.enabled ? 'default' : 'secondary'}>
                    {usbRedir?.enabled ? 'Enabled' : 'Disabled'}
                  </Badge>
                </div>

                {!usbRedir?.enabled && (
                  <div className="space-y-3">
                    <div className="space-y-2">
                      <Label htmlFor="usb-redir-count">Number of Channels</Label>
                      <Select
                        value={usbRedirCount.toString()}
                        onValueChange={(v) => setUsbRedirCount(parseInt(v))}
                        disabled={isRunning || usbRedirLoading}
                      >
                        <SelectTrigger id="usb-redir-count">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="1">1 channel</SelectItem>
                          <SelectItem value="2">2 channels</SelectItem>
                          <SelectItem value="3">3 channels</SelectItem>
                          <SelectItem value="4">4 channels (recommended)</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-xs text-muted-foreground">
                        Each channel allows one USB device to be redirected
                      </p>
                    </div>

                    {isRunning && (
                      <Alert>
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>
                          Stop the VM to enable USB redirection.
                        </AlertDescription>
                      </Alert>
                    )}

                    <Button
                      className="w-full"
                      disabled={isRunning || usbRedirLoading}
                      onClick={async () => {
                        setUsbRedirLoading(true)
                        try {
                          await api.attachUsbRedirection(vm.id, usbRedirCount)
                          toast.success(`Added ${usbRedirCount} USB redirection channel(s)`)
                          refetchUsbRedir()
                        } catch (err) {
                          toast.error(`Failed: ${err instanceof Error ? err.message : String(err)}`)
                        } finally {
                          setUsbRedirLoading(false)
                        }
                      }}
                    >
                      {usbRedirLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                      Enable USB Redirection
                    </Button>
                  </div>
                )}

                {usbRedir?.enabled && (
                  <div className="space-y-3">
                    <div className="bg-muted/50 p-3 rounded-md text-sm">
                      <p className="font-medium">How to use USB redirection:</p>
                      <ol className="list-decimal list-inside mt-2 text-xs text-muted-foreground space-y-1">
                        <li>Start the VM</li>
                        <li>Connect with a SPICE client (virt-viewer or remote-viewer)</li>
                        <li>Use the client's USB menu to redirect devices</li>
                      </ol>
                    </div>

                    <Button
                      variant="destructive"
                      className="w-full"
                      disabled={isRunning || usbRedirLoading}
                      onClick={async () => {
                        setUsbRedirLoading(true)
                        try {
                          await api.removeUsbRedirection(vm.id)
                          toast.success('USB redirection disabled')
                          refetchUsbRedir()
                        } catch (err) {
                          toast.error(`Failed: ${err instanceof Error ? err.message : String(err)}`)
                        } finally {
                          setUsbRedirLoading(false)
                        }
                      }}
                    >
                      {usbRedirLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                      Disable USB Redirection
                    </Button>
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="security" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Authentication</CardTitle>
              <CardDescription>
                Password protection for remote display access
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label>Password Protection</Label>
                  <p className="text-xs text-muted-foreground">
                    Require password to connect to this display
                  </p>
                </div>
                <Switch checked={false} disabled />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <div className="flex gap-2">
                  <Input
                    id="password"
                    type={showPassword ? 'text' : 'password'}
                    placeholder="Enter password"
                    disabled
                    className="flex-1"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() => setShowPassword(!showPassword)}
                  >
                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>

          {graphicsType === 'spice' && (
            <Card>
              <CardHeader>
                <CardTitle>TLS Encryption</CardTitle>
                <CardDescription>
                  Encrypt SPICE traffic with TLS
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <Label>Enable TLS</Label>
                    <p className="text-xs text-muted-foreground">
                      Encrypt all display traffic (recommended for remote access)
                    </p>
                  </div>
                  <Switch checked={false} disabled />
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Graphics XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for this graphics device
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {graphicsType === 'spice' ?
`<graphics type='spice' autoport='yes'>
  <listen type='address' address='127.0.0.1'/>
  <image compression='auto_glz'/>
  <streaming mode='filter'/>
  <clipboard copypaste='yes'/>
  <filetransfer enable='yes'/>
</graphics>` :
`<graphics type='vnc' port='${vm.vncPort || -1}' autoport='yes' listen='127.0.0.1'>
  <listen type='address' address='127.0.0.1'/>
</graphics>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {success && (
        <Alert>
          <CheckCircle className="h-4 w-4 text-green-500" />
          <AlertDescription>Graphics configuration updated successfully!</AlertDescription>
        </Alert>
      )}

      <div className="flex justify-between">
        <Button variant="destructive" size="sm" className="gap-1.5" disabled>
          <Trash2 className="w-4 h-4" />
          Remove Graphics
        </Button>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={handleRevert}
            disabled={!hasChanges || loading}
          >
            Revert
          </Button>
          <Button
            onClick={handleApply}
            disabled={!hasChanges || loading || isRunning}
          >
            {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Apply
          </Button>
        </div>
      </div>
    </div>
  )
}
