import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Video, Trash2, AlertCircle, Loader2, CheckCircle } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useQueryClient } from '@tanstack/react-query'
import type { VM } from '@/lib/types'

interface VideoEditorProps {
  vm: VM
  compact?: boolean
}

export function VideoEditor({ vm, compact }: VideoEditorProps) {
  const queryClient = useQueryClient()
  const originalModel = vm.video?.model || 'virtio'

  const [videoModel, setVideoModel] = useState(originalModel)
  const [vram, setVram] = useState(65536)
  const [heads, setHeads] = useState(vm.video?.heads || 1)
  const [accel3d] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const isRunning = vm.state === 'running'
  const hasChanges = videoModel !== originalModel || heads !== (vm.video?.heads || 1)

  useEffect(() => {
    setVideoModel(vm.video?.model || 'virtio')
    setHeads(vm.video?.heads || 1)
  }, [vm])

  const handleApply = async () => {
    setLoading(true)
    setError(null)
    setSuccess(false)

    try {
      await api.attachVideo(vm.id, videoModel, vram, heads, accel3d)
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
    setVideoModel(originalModel)
    setError(null)
  }

  // Compact mode shows inline summary
  if (compact) {
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Video className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">Video</span>
          </div>
          <Badge variant={videoModel === 'virtio' ? 'default' : 'secondary'} className="text-xs">
            {videoModel.toUpperCase()}
          </Badge>
        </div>

        <Select value={videoModel} onValueChange={setVideoModel} disabled={isRunning}>
          <SelectTrigger className="h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="virtio">VirtIO (recommended)</SelectItem>
            <SelectItem value="qxl">QXL (SPICE)</SelectItem>
            <SelectItem value="vga">VGA (Legacy)</SelectItem>
            <SelectItem value="cirrus">Cirrus (Legacy)</SelectItem>
          </SelectContent>
        </Select>

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
            <Video className="w-6 h-6" />
            Video Device
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure virtual graphics card for this VM
          </p>
        </div>
        <Badge variant={videoModel === 'virtio' ? 'default' : 'secondary'}>
          {videoModel.toUpperCase()}
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
              <CardTitle>Video Model</CardTitle>
              <CardDescription>
                Select the virtual graphics adapter type
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {isRunning && (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Video model changes require the VM to be stopped.
                  </AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label htmlFor="model">Model</Label>
                <Select
                  value={videoModel}
                  onValueChange={setVideoModel}
                  disabled={isRunning}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select model" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="virtio">VirtIO (Recommended)</SelectItem>
                    <SelectItem value="qxl">QXL (SPICE optimized)</SelectItem>
                    <SelectItem value="vga">VGA (Legacy)</SelectItem>
                    <SelectItem value="bochs">Bochs</SelectItem>
                    <SelectItem value="ramfb">Ramfb (UEFI only)</SelectItem>
                    <SelectItem value="cirrus">Cirrus (Legacy)</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  VirtIO offers best performance with modern guest drivers
                </p>
              </div>

              <div className="p-3 bg-muted rounded-lg">
                <h4 className="font-medium text-sm mb-2">Model Comparison</h4>
                <ul className="text-xs text-muted-foreground space-y-1">
                  <li><strong>VirtIO</strong> - Best performance, requires guest drivers</li>
                  <li><strong>QXL</strong> - Optimized for SPICE, multi-monitor support</li>
                  <li><strong>VGA</strong> - Maximum compatibility, no special drivers</li>
                  <li><strong>Bochs</strong> - Simple, good for servers/headless</li>
                  <li><strong>Ramfb</strong> - Minimal, UEFI boot only</li>
                </ul>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Video Memory</CardTitle>
              <CardDescription>
                Configure video RAM allocation
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="vram">Video RAM (KB)</Label>
                  <Select
                    value={vram.toString()}
                    onValueChange={(v) => setVram(parseInt(v))}
                    disabled={isRunning}
                  >
                    <SelectTrigger id="vram">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="16384">16 MB (basic)</SelectItem>
                      <SelectItem value="32768">32 MB</SelectItem>
                      <SelectItem value="65536">64 MB (recommended)</SelectItem>
                      <SelectItem value="131072">128 MB (multi-monitor)</SelectItem>
                      <SelectItem value="262144">256 MB (high-res)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Increase for higher resolutions or multi-monitor
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="heads">Display Heads (Monitors)</Label>
                  <Select
                    value={heads.toString()}
                    onValueChange={(v) => setHeads(parseInt(v))}
                    disabled={isRunning}
                  >
                    <SelectTrigger id="heads">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">1 monitor</SelectItem>
                      <SelectItem value="2">2 monitors</SelectItem>
                      <SelectItem value="3">3 monitors</SelectItem>
                      <SelectItem value="4">4 monitors</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Number of virtual displays for multi-monitor setups
                  </p>
                </div>
              </div>

              {heads > 1 && (
                <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                  <p className="text-sm text-blue-600 dark:text-blue-400 font-medium">
                    Multi-Monitor Configuration
                  </p>
                  <p className="text-xs text-blue-600/80 dark:text-blue-400/80 mt-1">
                    {heads} monitors configured. For best results:
                    <ul className="list-disc list-inside mt-1">
                      <li>Use QXL or VirtIO video model</li>
                      <li>Allocate more VRAM (128MB+ recommended)</li>
                      <li>SPICE clients support multi-monitor better than VNC</li>
                      <li>Install SPICE guest tools in the VM</li>
                    </ul>
                  </p>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>3D Acceleration</CardTitle>
              <CardDescription>
                Hardware-accelerated 3D graphics
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label>3D Acceleration</Label>
                  <p className="text-xs text-muted-foreground">
                    Enable GPU acceleration for 3D graphics (VirtIO only)
                  </p>
                </div>
                <Switch checked={false} disabled />
              </div>

              {videoModel === 'virtio' && (
                <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                  <p className="text-xs text-yellow-600 dark:text-yellow-400">
                    <strong>Note:</strong> 3D acceleration requires:
                    <ul className="list-disc list-inside mt-1">
                      <li>SPICE graphics with OpenGL enabled</li>
                      <li>VirtIO-GPU guest drivers</li>
                      <li>Host GPU with OpenGL 4.3+ support</li>
                    </ul>
                  </p>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Video XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for this video device
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {`<video>
  <model type='${videoModel}' heads='${heads}' primary='yes'${videoModel === 'virtio' ? ` vram='${vram}'` : ''}>
    <acceleration accel3d='${accel3d ? 'yes' : 'no'}'/>
  </model>
</video>`}
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
          <AlertDescription>Video configuration updated successfully!</AlertDescription>
        </Alert>
      )}

      <div className="flex justify-between">
        <Button variant="destructive" size="sm" className="gap-1.5" disabled>
          <Trash2 className="w-4 h-4" />
          Remove Video
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
