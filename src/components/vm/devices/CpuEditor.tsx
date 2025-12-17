import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, CheckCircle, Cpu, Pin, X, Info } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useQuery, useQueryClient, useMutation } from '@tanstack/react-query'
import { toast } from 'sonner'
import type { VM } from '@/lib/types'

// CPU Model Section Component
function CpuModelSection({ vm }: { vm: VM }) {
  const queryClient = useQueryClient()
  const isRunning = vm.state === 'running'

  // Get current CPU model
  const { data: cpuConfig, refetch: refetchCpuModel } = useQuery({
    queryKey: ['cpuModel', vm.id],
    queryFn: () => api.getCpuModel(vm.id),
  })

  // Get available CPU models
  const { data: availableModels = [] } = useQuery({
    queryKey: ['availableCpuModels'],
    queryFn: () => api.getAvailableCpuModels(),
  })

  // Local state
  const [mode, setMode] = useState(cpuConfig?.mode || 'host-passthrough')
  const [customModel, setCustomModel] = useState(cpuConfig?.model || 'qemu64')
  const [hasChanges, setHasChanges] = useState(false)

  // Update local state when data loads
  useEffect(() => {
    if (cpuConfig) {
      setMode(cpuConfig.mode)
      setCustomModel(cpuConfig.model || 'qemu64')
    }
  }, [cpuConfig])

  // Track changes
  useEffect(() => {
    if (cpuConfig) {
      const modeChanged = mode !== cpuConfig.mode
      const modelChanged = mode === 'custom' && customModel !== cpuConfig.model
      setHasChanges(modeChanged || modelChanged)
    }
  }, [mode, customModel, cpuConfig])

  // Apply mutation
  const applyMutation = useMutation({
    mutationFn: () => api.setCpuModel(vm.id, mode, mode === 'custom' ? customModel : undefined),
    onSuccess: () => {
      toast.success('CPU model updated')
      refetchCpuModel()
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      setHasChanges(false)
    },
    onError: (error) => {
      toast.error(`Failed to update CPU model: ${error}`)
    },
  })

  const handleRevert = () => {
    if (cpuConfig) {
      setMode(cpuConfig.mode)
      setCustomModel(cpuConfig.model || 'qemu64')
      setHasChanges(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>CPU Model</CardTitle>
        <CardDescription>
          CPU model and features exposed to the guest
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="cpu-mode">CPU Mode</Label>
          <Select value={mode} onValueChange={setMode} disabled={isRunning}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="host-passthrough">Host Passthrough (Best Performance)</SelectItem>
              <SelectItem value="host-model">Host Model (Migratable)</SelectItem>
              <SelectItem value="custom">Custom CPU Model</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            {mode === 'host-passthrough' && 'Exposes all host CPU features directly to the guest for maximum performance.'}
            {mode === 'host-model' && 'Uses a CPU model based on the host but allows live migration to similar hosts.'}
            {mode === 'custom' && 'Select a specific CPU model for maximum compatibility or migration flexibility.'}
          </p>
        </div>

        {mode === 'custom' && (
          <div className="space-y-2">
            <Label htmlFor="cpu-model">CPU Model</Label>
            <Select value={customModel} onValueChange={setCustomModel} disabled={isRunning}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="max-h-60">
                {availableModels.map((model) => (
                  <SelectItem key={model} value={model}>{model}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              Choose a CPU model that matches your migration target hosts.
            </p>
          </div>
        )}

        {isRunning && (
          <div className="flex items-center gap-2 p-2 bg-amber-500/10 border border-amber-500/30 rounded-lg">
            <Info className="w-4 h-4 text-amber-500" />
            <p className="text-xs text-muted-foreground">
              Stop the VM to change CPU model settings.
            </p>
          </div>
        )}

        {hasChanges && !isRunning && (
          <div className="flex gap-2">
            <Button
              onClick={() => applyMutation.mutate()}
              disabled={applyMutation.isPending}
            >
              {applyMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Apply
            </Button>
            <Button variant="outline" onClick={handleRevert}>
              Revert
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

// CPU Pinning Section Component
function CpuPinningSection({ vm }: { vm: VM }) {
  const queryClient = useQueryClient()
  const isRunning = vm.state === 'running'
  const vcpuCount = vm.cpuCount || vm.cpus || 1

  // Get host CPU count
  const { data: hostInfo } = useQuery({
    queryKey: ['hostInfo'],
    queryFn: () => api.getHostInfo(),
  })

  // Get current pinning configuration
  const { data: pinnings = [], refetch: refetchPinning } = useQuery({
    queryKey: ['cpuPinning', vm.id],
    queryFn: () => api.getCpuPinning(vm.id),
  })

  const hostCpuCount = hostInfo?.cpuCount || 8

  // Local state for editing
  const [editingVcpu, setEditingVcpu] = useState<number | null>(null)
  const [selectedCpus, setSelectedCpus] = useState<number[]>([])
  const [isPending, setIsPending] = useState(false)

  // Get pinned CPUs for a vCPU
  const getPinnedCpus = (vcpu: number): number[] => {
    const pin = pinnings.find(([v]) => v === vcpu)
    return pin ? pin[1] : []
  }

  const handleStartEdit = (vcpu: number) => {
    setEditingVcpu(vcpu)
    setSelectedCpus(getPinnedCpus(vcpu))
  }

  const handleCancelEdit = () => {
    setEditingVcpu(null)
    setSelectedCpus([])
  }

  const handleToggleCpu = (cpu: number) => {
    setSelectedCpus(prev =>
      prev.includes(cpu) ? prev.filter(c => c !== cpu) : [...prev, cpu].sort((a, b) => a - b)
    )
  }

  const handleApplyPin = async () => {
    if (editingVcpu === null) return

    setIsPending(true)
    try {
      if (selectedCpus.length === 0) {
        await api.clearCpuPin(vm.id, editingVcpu)
        toast.success(`CPU pinning cleared for vCPU ${editingVcpu}`)
      } else {
        await api.setCpuPin(vm.id, editingVcpu, selectedCpus)
        toast.success(`vCPU ${editingVcpu} pinned to CPU(s) ${selectedCpus.join(', ')}`)
      }
      refetchPinning()
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      handleCancelEdit()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setIsPending(false)
    }
  }

  const handleClearPin = async (vcpu: number) => {
    setIsPending(true)
    try {
      await api.clearCpuPin(vm.id, vcpu)
      toast.success(`CPU pinning cleared for vCPU ${vcpu}`)
      refetchPinning()
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setIsPending(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Pin className="h-5 w-5" />
          CPU Pinning
        </CardTitle>
        <CardDescription>
          Pin virtual CPUs to specific host physical CPUs for improved performance
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {isRunning && (
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              CPU pinning changes require the VM to be stopped.
            </AlertDescription>
          </Alert>
        )}

        <div className="text-sm text-muted-foreground mb-4">
          Host has {hostCpuCount} physical CPU(s). This VM has {vcpuCount} vCPU(s).
        </div>

        <div className="space-y-3">
          {Array.from({ length: vcpuCount }, (_, i) => i).map(vcpu => {
            const pinnedCpus = getPinnedCpus(vcpu)
            const isEditing = editingVcpu === vcpu

            return (
              <div
                key={vcpu}
                className="flex items-center gap-3 p-3 border rounded-lg"
              >
                <div className="flex items-center gap-2 min-w-[80px]">
                  <Cpu className="h-4 w-4 text-muted-foreground" />
                  <span className="font-medium">vCPU {vcpu}</span>
                </div>

                {isEditing ? (
                  <div className="flex-1 space-y-2">
                    <div className="flex flex-wrap gap-1">
                      {Array.from({ length: hostCpuCount }, (_, i) => i).map(cpu => (
                        <Button
                          key={cpu}
                          variant={selectedCpus.includes(cpu) ? 'default' : 'outline'}
                          size="sm"
                          className="h-7 w-7 p-0 text-xs"
                          onClick={() => handleToggleCpu(cpu)}
                        >
                          {cpu}
                        </Button>
                      ))}
                    </div>
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        onClick={handleApplyPin}
                        disabled={isPending}
                      >
                        {isPending ? <Loader2 className="h-3 w-3 animate-spin mr-1" /> : null}
                        {selectedCpus.length === 0 ? 'Clear' : 'Apply'}
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={handleCancelEdit}
                        disabled={isPending}
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                ) : (
                  <>
                    <div className="flex-1">
                      {pinnedCpus.length > 0 ? (
                        <span className="text-sm">
                          → CPU {pinnedCpus.join(', ')}
                        </span>
                      ) : (
                        <span className="text-sm text-muted-foreground italic">
                          Not pinned (any CPU)
                        </span>
                      )}
                    </div>
                    <div className="flex gap-1">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleStartEdit(vcpu)}
                        disabled={isRunning || isPending}
                      >
                        Edit
                      </Button>
                      {pinnedCpus.length > 0 && (
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => handleClearPin(vcpu)}
                          disabled={isRunning || isPending}
                        >
                          <X className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </>
                )}
              </div>
            )
          })}
        </div>

        <div className="p-3 bg-muted rounded-lg">
          <p className="text-xs text-muted-foreground">
            <strong>Tips:</strong>
            <br />• Pin vCPUs to physical CPUs that share the same NUMA node for best performance
            <br />• Avoid pinning to CPUs used by other critical VMs
            <br />• For hyperthreaded hosts, consider pinning to sibling threads
          </p>
        </div>
      </CardContent>
    </Card>
  )
}

interface CpuEditorProps {
  vm: VM
  compact?: boolean
}

export function CpuEditor({ vm, compact }: CpuEditorProps) {
  const queryClient = useQueryClient()
  const isRunning = vm.state === 'running'

  // Get actual vCPU count - backend provides cpuCount
  const actualVcpus = vm.cpuCount || vm.cpus || 1

  // State for vCPU count - ensure we have a valid default
  const [vcpus, setVcpus] = useState(actualVcpus)
  const [vcpusLoading, setVcpusLoading] = useState(false)
  const [vcpusError, setVcpusError] = useState<string | null>(null)
  const [vcpusSuccess, setVcpusSuccess] = useState(false)

  // State for topology - use cpuSockets/cpuCores/cpuThreads from backend
  const [sockets, setSockets] = useState(vm.cpuSockets || vm.topology?.sockets || 1)
  const [cores, setCores] = useState(vm.cpuCores || vm.topology?.cores || actualVcpus)
  const [threads, setThreads] = useState(vm.cpuThreads || vm.topology?.threads || 1)
  const [topologyLoading, setTopologyLoading] = useState(false)
  const [topologyError, setTopologyError] = useState<string | null>(null)
  const [topologySuccess, setTopologySuccess] = useState(false)

  // Sync state when VM data changes
  useEffect(() => {
    const newVcpus = vm.cpuCount || vm.cpus || 1
    setVcpus(newVcpus)
    setSockets(vm.cpuSockets || vm.topology?.sockets || 1)
    setCores(vm.cpuCores || vm.topology?.cores || newVcpus)
    setThreads(vm.cpuThreads || vm.topology?.threads || 1)
  }, [vm])

  const hasVcpuChanges = vcpus !== actualVcpus
  const hasTopologyChanges =
    sockets !== (vm.topology?.sockets || 1) ||
    cores !== (vm.topology?.cores || vm.cpus) ||
    threads !== (vm.topology?.threads || 1)

  const handleApplyVcpus = async () => {
    setVcpusLoading(true)
    setVcpusError(null)
    setVcpusSuccess(false)

    try {
      await api.setVmVcpus(vm.id, vcpus)
      setVcpusSuccess(true)
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      setTimeout(() => setVcpusSuccess(false), 3000)
    } catch (err) {
      setVcpusError(err instanceof Error ? err.message : String(err))
    } finally {
      setVcpusLoading(false)
    }
  }

  const handleApplyTopology = async () => {
    setTopologyLoading(true)
    setTopologyError(null)
    setTopologySuccess(false)

    try {
      await api.setVmCpuTopology(vm.id, sockets, cores, threads)
      setTopologySuccess(true)
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      setTimeout(() => setTopologySuccess(false), 3000)
    } catch (err) {
      setTopologyError(err instanceof Error ? err.message : String(err))
    } finally {
      setTopologyLoading(false)
    }
  }

  const revertVcpus = () => {
    setVcpus(vm.cpuCount || vm.cpus || 1)
    setVcpusError(null)
  }

  const revertTopology = () => {
    setSockets(vm.cpuSockets || vm.topology?.sockets || 1)
    setCores(vm.cpuCores || vm.topology?.cores || vm.cpuCount || vm.cpus || 1)
    setThreads(vm.cpuThreads || vm.topology?.threads || 1)
    setTopologyError(null)
  }

  // Compact mode - clean display with inline edit on click
  if (compact) {
    const [editing, setEditing] = useState(false)

    if (editing || hasVcpuChanges) {
      return (
        <div className="flex items-center gap-2">
          <Input
            type="number"
            min={1}
            max={64}
            value={vcpus}
            onChange={(e) => setVcpus(parseInt(e.target.value) || 1)}
            className="w-14 h-6 text-xs px-2"
            autoFocus
            onBlur={() => !hasVcpuChanges && setEditing(false)}
          />
          <span className="text-[11px] text-muted-foreground">vCPUs</span>
          {hasVcpuChanges && (
            <>
              <Button size="sm" variant="default" onClick={() => { handleApplyVcpus(); setEditing(false) }} disabled={vcpusLoading} className="h-5 px-2 text-[10px]">
                {vcpusLoading ? <Loader2 className="h-3 w-3 animate-spin" /> : 'Save'}
              </Button>
              <Button size="sm" variant="ghost" onClick={() => { revertVcpus(); setEditing(false) }} className="h-5 px-1 text-[10px]">✕</Button>
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
        <span className="font-medium text-sm">{vcpus}</span>
        <span className="text-[11px] text-muted-foreground">vCPUs ({sockets}×{cores}×{threads})</span>
      </button>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div>
        <h2 className="text-2xl font-semibold mb-2">CPU Configuration</h2>
        <p className="text-sm text-muted-foreground">
          Configure virtual CPU allocation and topology for this VM
        </p>
      </div>

      {isRunning && (
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            Some CPU changes require the VM to be stopped. vCPU count changes may require hot-plug support.
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
              <CardTitle>vCPU Allocation</CardTitle>
              <CardDescription>
                Number of virtual CPUs allocated to this VM
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="vcpus">vCPUs</Label>
                  <Input
                    id="vcpus"
                    type="number"
                    min={1}
                    max={64}
                    value={vcpus}
                    onChange={(e) => setVcpus(parseInt(e.target.value) || 1)}
                  />
                  <p className="text-xs text-muted-foreground">
                    Total number of virtual CPUs
                  </p>
                </div>
              </div>

              {vcpusError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{vcpusError}</AlertDescription>
                </Alert>
              )}

              {vcpusSuccess && (
                <Alert>
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <AlertDescription>vCPU count updated successfully!</AlertDescription>
                </Alert>
              )}

              <div className="flex justify-end gap-2">
                <Button
                  variant="outline"
                  onClick={revertVcpus}
                  disabled={!hasVcpuChanges || vcpusLoading}
                >
                  Revert
                </Button>
                <Button
                  onClick={handleApplyVcpus}
                  disabled={!hasVcpuChanges || vcpusLoading}
                >
                  {vcpusLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Apply
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>CPU Topology</CardTitle>
              <CardDescription>
                Configure how vCPUs are presented to the guest OS
                {isRunning && " (VM must be stopped to change topology)"}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-3">
                <div className="space-y-2">
                  <Label htmlFor="sockets">Sockets</Label>
                  <Input
                    id="sockets"
                    type="number"
                    min={1}
                    value={sockets}
                    onChange={(e) => setSockets(parseInt(e.target.value) || 1)}
                    disabled={isRunning}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="cores">Cores per socket</Label>
                  <Input
                    id="cores"
                    type="number"
                    min={1}
                    value={cores}
                    onChange={(e) => setCores(parseInt(e.target.value) || 1)}
                    disabled={isRunning}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="threads">Threads per core</Label>
                  <Input
                    id="threads"
                    type="number"
                    min={1}
                    value={threads}
                    onChange={(e) => setThreads(parseInt(e.target.value) || 1)}
                    disabled={isRunning}
                  />
                </div>
              </div>
              <p className="text-xs text-muted-foreground">
                Topology: {sockets} × {cores} × {threads} = {sockets * cores * threads} vCPUs
              </p>

              {topologyError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{topologyError}</AlertDescription>
                </Alert>
              )}

              {topologySuccess && (
                <Alert>
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <AlertDescription>CPU topology updated successfully!</AlertDescription>
                </Alert>
              )}

              <div className="flex justify-end gap-2">
                <Button
                  variant="outline"
                  onClick={revertTopology}
                  disabled={!hasTopologyChanges || topologyLoading || isRunning}
                >
                  Revert
                </Button>
                <Button
                  onClick={handleApplyTopology}
                  disabled={!hasTopologyChanges || topologyLoading || isRunning}
                >
                  {topologyLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Apply
                </Button>
              </div>
            </CardContent>
          </Card>

          <CpuModelSection vm={vm} />

          <CpuPinningSection vm={vm} />
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardContent className="p-4">
              <pre className="text-xs font-mono bg-muted p-4 rounded-md overflow-auto max-h-96">
                {`<vcpu placement='static'>${vm.cpus}</vcpu>
<cpu mode='host-passthrough'>
  <topology sockets='${vm.topology?.sockets || 1}' cores='${vm.topology?.cores || vm.cpus}' threads='${vm.topology?.threads || 1}'/>
</cpu>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
