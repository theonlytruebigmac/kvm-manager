import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Volume2, Trash2, AlertCircle, Loader2, CheckCircle } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useQueryClient } from '@tanstack/react-query'
import type { VM } from '@/lib/types'

interface SoundEditorProps {
  vm: VM
  compact?: boolean
}

export function SoundEditor({ vm, compact }: SoundEditorProps) {
  const queryClient = useQueryClient()
  const originalModel = 'ich9' // Default - would come from VM config

  const [soundModel, setSoundModel] = useState(originalModel)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const isRunning = vm.state === 'running'
  const hasChanges = soundModel !== originalModel

  const handleApply = async () => {
    setLoading(true)
    setError(null)
    setSuccess(false)

    try {
      await api.attachSound(vm.id, soundModel)
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
    setSoundModel(originalModel)
    setError(null)
  }

  const handleRemove = async () => {
    setLoading(true)
    setError(null)

    try {
      await api.detachSound(vm.id)
      setSuccess(true)
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  // Compact mode shows inline summary
  if (compact) {
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Volume2 className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">Sound</span>
            <Badge variant="secondary" className="text-xs">
              {soundModel.toUpperCase()}
            </Badge>
          </div>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 text-red-600 hover:text-red-700 hover:bg-red-50"
            onClick={handleRemove}
            disabled={loading || isRunning}
          >
            <Trash2 className="w-3.5 h-3.5" />
          </Button>
        </div>

        <Select value={soundModel} onValueChange={setSoundModel} disabled={isRunning}>
          <SelectTrigger className="h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="ich9">ICH9 (HDA)</SelectItem>
            <SelectItem value="ich6">ICH6 (AC97)</SelectItem>
            <SelectItem value="ac97">AC97</SelectItem>
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
            <Volume2 className="w-6 h-6" />
            Sound Device
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure virtual audio device for this VM
          </p>
        </div>
        <Badge variant="secondary">
          {soundModel.toUpperCase()}
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
              <CardTitle>Sound Card Model</CardTitle>
              <CardDescription>
                Select the virtual sound card type
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {isRunning && (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Sound device changes require the VM to be stopped.
                  </AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label htmlFor="model">Model</Label>
                <Select
                  value={soundModel}
                  onValueChange={setSoundModel}
                  disabled={isRunning}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select model" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="ich9">Intel ICH9 (Recommended)</SelectItem>
                    <SelectItem value="ich6">Intel ICH6</SelectItem>
                    <SelectItem value="ac97">AC97</SelectItem>
                    <SelectItem value="es1370">Ensoniq ES1370</SelectItem>
                    <SelectItem value="sb16">Sound Blaster 16 (Legacy)</SelectItem>
                    <SelectItem value="usb">USB Audio</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  ICH9 offers best compatibility with modern operating systems
                </p>
              </div>

              <div className="p-3 bg-muted rounded-lg">
                <h4 className="font-medium text-sm mb-2">Model Comparison</h4>
                <ul className="text-xs text-muted-foreground space-y-1">
                  <li><strong>ICH9</strong> - Intel HD Audio, best for modern guests</li>
                  <li><strong>ICH6</strong> - Intel AC'97, good compatibility</li>
                  <li><strong>AC97</strong> - Realtek AC'97, wide compatibility</li>
                  <li><strong>SB16</strong> - Legacy DOS/Windows 9x support</li>
                  <li><strong>USB</strong> - USB audio, for special cases</li>
                </ul>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Audio Backend</CardTitle>
              <CardDescription>
                Configure host audio system integration
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="backend">Audio Backend</Label>
                <Select value="spice" disabled>
                  <SelectTrigger>
                    <SelectValue placeholder="Select backend" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="spice">SPICE (Network audio)</SelectItem>
                    <SelectItem value="pulseaudio">PulseAudio</SelectItem>
                    <SelectItem value="pipewire">PipeWire</SelectItem>
                    <SelectItem value="none">None</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  SPICE audio is routed through the display connection.
                  Use PulseAudio/PipeWire for local audio output.
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Sound XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for this sound device
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {`<sound model='${soundModel}'>
  <audio id='1'/>
</sound>
<audio id='1' type='spice'/>`}
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
          <AlertDescription>Sound configuration updated successfully!</AlertDescription>
        </Alert>
      )}

      <div className="flex justify-between">
        <Button
          variant="destructive"
          size="sm"
          className="gap-1.5"
          onClick={handleRemove}
          disabled={loading || isRunning}
        >
          <Trash2 className="w-4 h-4" />
          Remove Sound
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
