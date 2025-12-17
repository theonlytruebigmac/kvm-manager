import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { AlertCircle, Loader2, CheckCircle, Zap } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import type { VM } from '@/lib/types'

interface MemoryEditorProps {
  vm: VM
  compact?: boolean
}

export function MemoryEditor({ vm, compact }: MemoryEditorProps) {
  const queryClient = useQueryClient()
  const isRunning = vm.state === 'running'

  const [memoryMb, setMemoryMb] = useState(vm.memoryMb)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  // Hugepages state
  const [hugepagesEnabled, setHugepagesEnabled] = useState(false)
  const [hugepageSize, setHugepageSize] = useState<number | undefined>(undefined)
  const [hugepagesLoading, setHugepagesLoading] = useState(false)
  const [hugepagesError, setHugepagesError] = useState<string | null>(null)
  const [hugepagesSuccess, setHugepagesSuccess] = useState(false)

  // Query host hugepage info
  const { data: hostHugepages = [] } = useQuery({
    queryKey: ['hostHugepages'],
    queryFn: () => api.getHostHugepageInfo(),
    staleTime: 30000,
  })

  // Query VM hugepages settings
  const { data: hugepagesSettings, refetch: refetchHugepages } = useQuery({
    queryKey: ['hugepagesSettings', vm.id],
    queryFn: () => api.getHugepagesSettings(vm.id),
    staleTime: 5000,
  })

  // Sync hugepages state when settings load
  useEffect(() => {
    if (hugepagesSettings) {
      setHugepagesEnabled(hugepagesSettings.enabled)
      setHugepageSize(hugepagesSettings.size)
    }
  }, [hugepagesSettings])

  // Sync state when VM data changes
  useEffect(() => {
    setMemoryMb(vm.memoryMb)
  }, [vm])

  const hasChanges = memoryMb !== vm.memoryMb
  const memoryGB = (memoryMb / 1024).toFixed(1)

  const handleApply = async () => {
    setLoading(true)
    setError(null)
    setSuccess(false)

    try {
      await api.setVmMemory(vm.id, memoryMb)
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
    setMemoryMb(vm.memoryMb)
    setError(null)
  }

  // Compact mode - clean display with inline edit on click
  if (compact) {
    const [editing, setEditing] = useState(false)

    if (editing || hasChanges) {
      return (
        <div className="flex items-center gap-2">
          <Input
            type="number"
            min={512}
            max={65536}
            step={512}
            value={memoryMb}
            onChange={(e) => setMemoryMb(parseInt(e.target.value) || 512)}
            className="w-16 h-6 text-xs px-2"
            autoFocus
            onBlur={() => !hasChanges && setEditing(false)}
          />
          <span className="text-[11px] text-muted-foreground">MB</span>
          {hasChanges && (
            <>
              <Button size="sm" variant="default" onClick={() => { handleApply(); setEditing(false) }} disabled={loading} className="h-5 px-2 text-[10px]">
                {loading ? <Loader2 className="h-3 w-3 animate-spin" /> : 'Save'}
              </Button>
              <Button size="sm" variant="ghost" onClick={() => { handleRevert(); setEditing(false) }} className="h-5 px-1 text-[10px]">✕</Button>
            </>
          )}
        </div>
      )
    }

    return (
      <button
        onClick={() => setEditing(true)}
        className="flex items-center gap-1.5 hover:bg-muted/50 rounded px-1 -mx-1 transition-colors"
      >
        <span className="font-medium text-sm">{memoryGB}</span>
        <span className="text-[11px] text-muted-foreground">GB</span>
      </button>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div>
        <h2 className="text-2xl font-semibold mb-2">Memory Configuration</h2>
        <p className="text-sm text-muted-foreground">
          Configure RAM allocation for this VM
        </p>
      </div>

      {isRunning && (
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            Memory changes on a running VM require memory ballooning support. For best results, stop the VM before changing memory.
          </AlertDescription>
        </Alert>
      )}

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Memory Allocation</CardTitle>
              <CardDescription>
                Amount of RAM allocated to this VM
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="memory">Memory (MB)</Label>
                <Input
                  id="memory"
                  type="number"
                  min={512}
                  max={65536}
                  step={512}
                  value={memoryMb}
                  onChange={(e) => setMemoryMb(parseInt(e.target.value) || 512)}
                />
                <p className="text-xs text-muted-foreground">
                  Current allocation: {memoryGB} GB ({memoryMb} MB)
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="max-memory">Maximum Memory (MB)</Label>
                <Input
                  id="max-memory"
                  type="number"
                  min={memoryMb}
                  max={131072}
                  step={512}
                  value={vm.maxMemoryMb || memoryMb}
                  disabled
                />
                <p className="text-xs text-muted-foreground">
                  Maximum memory for ballooning: {((vm.maxMemoryMb || memoryMb) / 1024).toFixed(1)} GB
                  {vm.maxMemoryMb && vm.maxMemoryMb > memoryMb && (
                    <span className="text-green-600 ml-1">(Ballooning enabled)</span>
                  )}
                </p>
              </div>

              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              {success && (
                <Alert>
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <AlertDescription>Memory updated successfully!</AlertDescription>
                </Alert>
              )}

              <div className="flex justify-end gap-2">
                <Button
                  variant="outline"
                  onClick={handleRevert}
                  disabled={!hasChanges || loading}
                >
                  Revert
                </Button>
                <Button
                  onClick={handleApply}
                  disabled={!hasChanges || loading}
                >
                  {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Apply
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Zap className="h-5 w-5" />
                Hugepages Memory Backing
              </CardTitle>
              <CardDescription>
                Use hugepages for better memory performance (requires VM to be stopped and hugepages pre-allocated on host)
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Host Hugepages Info */}
              {hostHugepages.length > 0 && (
                <div className="bg-muted/50 p-3 rounded-md text-sm">
                  <p className="font-medium mb-1">Host Hugepages Available:</p>
                  <div className="grid grid-cols-3 gap-2 text-xs">
                    {hostHugepages.map((hp) => (
                      <div key={hp.sizeKb} className="flex justify-between">
                        <span>{hp.sizeHuman}:</span>
                        <span className={hp.free > 0 ? 'text-green-600' : 'text-muted-foreground'}>
                          {hp.free}/{hp.total} free
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {hostHugepages.length === 0 && (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    No hugepages detected on host. You need to configure hugepages in your system before enabling this feature.
                    Use <code className="bg-muted px-1 rounded">sudo sysctl vm.nr_hugepages=1024</code> to allocate 2MB pages.
                  </AlertDescription>
                </Alert>
              )}

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Enable Hugepages</Label>
                  <p className="text-xs text-muted-foreground">
                    Use hugepages for VM memory (improves performance for large VMs)
                  </p>
                </div>
                <Switch
                  checked={hugepagesEnabled}
                  onCheckedChange={setHugepagesEnabled}
                  disabled={isRunning || hugepagesLoading}
                />
              </div>

              {hugepagesEnabled && (
                <div className="space-y-2">
                  <Label htmlFor="hugepage-size">Page Size</Label>
                  <Select
                    value={hugepageSize?.toString() || 'auto'}
                    onValueChange={(v) => setHugepageSize(v === 'auto' ? undefined : parseInt(v))}
                    disabled={isRunning || hugepagesLoading}
                  >
                    <SelectTrigger id="hugepage-size">
                      <SelectValue placeholder="Select page size" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="auto">Auto (use default)</SelectItem>
                      {hostHugepages.map((hp) => (
                        <SelectItem key={hp.sizeKb} value={hp.sizeKb.toString()}>
                          {hp.sizeHuman} ({hp.free} free)
                        </SelectItem>
                      ))}
                      {/* Always offer 2MB and 1GB options even if not detected */}
                      {!hostHugepages.some(h => h.sizeKb === 2048) && (
                        <SelectItem value="2048">2 MB</SelectItem>
                      )}
                      {!hostHugepages.some(h => h.sizeKb === 1048576) && (
                        <SelectItem value="1048576">1 GB</SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Larger pages reduce TLB misses but require more contiguous memory
                  </p>
                </div>
              )}

              {hugepagesError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{hugepagesError}</AlertDescription>
                </Alert>
              )}

              {hugepagesSuccess && (
                <Alert>
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <AlertDescription>Hugepages settings updated!</AlertDescription>
                </Alert>
              )}

              {isRunning && hugepagesEnabled !== hugepagesSettings?.enabled && (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Stop the VM to apply hugepages changes.
                  </AlertDescription>
                </Alert>
              )}

              <div className="flex justify-end gap-2">
                <Button
                  variant="outline"
                  onClick={() => {
                    setHugepagesEnabled(hugepagesSettings?.enabled || false)
                    setHugepageSize(hugepagesSettings?.size)
                    setHugepagesError(null)
                  }}
                  disabled={
                    hugepagesLoading ||
                    (hugepagesEnabled === hugepagesSettings?.enabled && hugepageSize === hugepagesSettings?.size)
                  }
                >
                  Revert
                </Button>
                <Button
                  onClick={async () => {
                    setHugepagesLoading(true)
                    setHugepagesError(null)
                    setHugepagesSuccess(false)
                    try {
                      await api.setHugepages(vm.id, hugepagesEnabled, hugepageSize)
                      setHugepagesSuccess(true)
                      refetchHugepages()
                      setTimeout(() => setHugepagesSuccess(false), 3000)
                    } catch (err) {
                      setHugepagesError(err instanceof Error ? err.message : String(err))
                    } finally {
                      setHugepagesLoading(false)
                    }
                  }}
                  disabled={
                    hugepagesLoading ||
                    isRunning ||
                    (hugepagesEnabled === hugepagesSettings?.enabled && hugepageSize === hugepagesSettings?.size)
                  }
                >
                  {hugepagesLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Apply Hugepages
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardContent className="p-4">
              <pre className="text-xs font-mono bg-muted p-4 rounded-md overflow-auto max-h-96">
                {`<memory unit='KiB'>${(vm.maxMemoryMb || vm.memoryMb) * 1024}</memory>
<currentMemory unit='KiB'>${vm.memoryMb * 1024}</currentMemory>${hugepagesSettings?.enabled ? `
<memoryBacking>
  <hugepages${hugepagesSettings.size ? `>
    <page size='${hugepagesSettings.size}' unit='KiB'/>
  </hugepages>` : '/>'}
</memoryBacking>` : ''}
<memballoon model='virtio'>
  <address type='pci' domain='0x0000' bus='0x00' slot='0x07' function='0x0'/>
</memballoon>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
