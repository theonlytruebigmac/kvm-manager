import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { AlertCircle, Loader2, CheckCircle, Cpu, Trash2 } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import type { VM, VmNumaConfig } from '@/lib/types'

interface NumaEditorProps {
  vm: VM
}

export function NumaEditor({ vm }: NumaEditorProps) {
  const queryClient = useQueryClient()
  const isRunning = vm.state === 'running'

  // Get host NUMA topology
  const { data: hostNodes = [], isLoading: loadingHost } = useQuery({
    queryKey: ['hostNumaTopology'],
    queryFn: () => api.getHostNumaTopology(),
  })

  // Get VM NUMA configuration
  const { data: numaConfig, isLoading: loadingConfig, refetch: refetchConfig } = useQuery({
    queryKey: ['vmNumaConfig', vm.id],
    queryFn: () => api.getVmNumaConfig(vm.id),
  })

  // Local state
  const [mode, setMode] = useState<'strict' | 'preferred' | 'interleave'>('strict')
  const [selectedNodes, setSelectedNodes] = useState<number[]>([])
  const [isPending, setIsPending] = useState(false)
  const [isEditing, setIsEditing] = useState(false)

  // Initialize editing state from current config
  const handleStartEdit = () => {
    if (numaConfig) {
      setMode(numaConfig.mode as 'strict' | 'preferred' | 'interleave')
      // Parse nodeset like "0,1,2" into array
      if (numaConfig.nodeset) {
        const nodes = numaConfig.nodeset.split(',').map(n => parseInt(n.trim())).filter(n => !isNaN(n))
        setSelectedNodes(nodes)
      } else {
        setSelectedNodes([])
      }
    } else {
      setMode('strict')
      setSelectedNodes([])
    }
    setIsEditing(true)
  }

  const handleCancelEdit = () => {
    setIsEditing(false)
    setSelectedNodes([])
  }

  const handleToggleNode = (nodeId: number) => {
    setSelectedNodes(prev =>
      prev.includes(nodeId)
        ? prev.filter(n => n !== nodeId)
        : [...prev, nodeId].sort((a, b) => a - b)
    )
  }

  const handleApplyConfig = async () => {
    if (selectedNodes.length === 0) {
      toast.error('Please select at least one NUMA node')
      return
    }

    setIsPending(true)
    try {
      const config: VmNumaConfig = {
        mode,
        nodeset: selectedNodes.join(','),
        cells: [],
      }
      await api.setVmNumaConfig(vm.id, config)
      toast.success(`NUMA configuration applied: ${mode} mode on node(s) ${selectedNodes.join(', ')}`)
      refetchConfig()
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      setIsEditing(false)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setIsPending(false)
    }
  }

  const handleClearConfig = async () => {
    setIsPending(true)
    try {
      await api.clearVmNumaConfig(vm.id)
      toast.success('NUMA configuration cleared')
      refetchConfig()
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      setIsEditing(false)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setIsPending(false)
    }
  }

  if (loadingHost || loadingConfig) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Cpu className="h-5 w-5" />
            NUMA Configuration
          </CardTitle>
        </CardHeader>
        <CardContent className="flex items-center justify-center py-8">
          <Loader2 className="h-6 w-6 animate-spin" />
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Cpu className="h-5 w-5" />
          NUMA Configuration
        </CardTitle>
        <CardDescription>
          Configure Non-Uniform Memory Access settings for optimal performance on multi-socket systems
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Host NUMA Topology Info */}
        <div className="p-3 bg-muted/50 rounded-lg">
          <h4 className="text-sm font-medium mb-2">Host NUMA Topology</h4>
          {hostNodes.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No NUMA nodes detected (single-socket system or NUMA disabled)
            </p>
          ) : (
            <div className="space-y-2">
              {hostNodes.map(node => (
                <div key={node.id} className="flex items-center gap-3 text-sm">
                  <Badge variant="outline">Node {node.id}</Badge>
                  <span className="text-muted-foreground">
                    {node.cpus.length > 0 ? `CPUs: ${node.cpus.join(', ')}` : 'CPUs: N/A'}
                    {node.memoryMb > 0 && ` | Memory: ${(node.memoryMb / 1024).toFixed(1)} GB`}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Current Configuration */}
        {!isEditing && (
          <div className="space-y-3">
            {numaConfig ? (
              <div className="p-3 border rounded-lg">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <CheckCircle className="h-4 w-4 text-green-500" />
                      <span className="font-medium">NUMA Tuning Active</span>
                    </div>
                    <div className="text-sm text-muted-foreground">
                      Mode: <span className="font-medium">{numaConfig.mode}</span>
                      {numaConfig.nodeset && (
                        <> | Node(s): <span className="font-medium">{numaConfig.nodeset}</span></>
                      )}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleStartEdit}
                      disabled={isRunning}
                    >
                      Edit
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleClearConfig}
                      disabled={isRunning || isPending}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="p-3 border rounded-lg text-center">
                <p className="text-sm text-muted-foreground mb-2">
                  No NUMA configuration set
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleStartEdit}
                  disabled={isRunning || hostNodes.length === 0}
                >
                  Configure NUMA
                </Button>
              </div>
            )}
          </div>
        )}

        {/* Edit Mode */}
        {isEditing && (
          <div className="space-y-4 p-3 border rounded-lg">
            <div className="space-y-2">
              <Label>Memory Mode</Label>
              <Select value={mode} onValueChange={(v) => setMode(v as typeof mode)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="strict">Strict (Fail if NUMA node unavailable)</SelectItem>
                  <SelectItem value="preferred">Preferred (Use others if needed)</SelectItem>
                  <SelectItem value="interleave">Interleave (Spread across nodes)</SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                {mode === 'strict' && 'Memory must be allocated from specified nodes only'}
                {mode === 'preferred' && 'Try specified nodes first, fall back to others if needed'}
                {mode === 'interleave' && 'Distribute memory evenly across specified nodes'}
              </p>
            </div>

            <div className="space-y-2">
              <Label>Select NUMA Node(s)</Label>
              <div className="flex flex-wrap gap-2">
                {hostNodes.map(node => (
                  <Button
                    key={node.id}
                    variant={selectedNodes.includes(node.id) ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => handleToggleNode(node.id)}
                    className="h-8"
                  >
                    Node {node.id}
                    {node.memoryMb > 0 && (
                      <span className="ml-1 text-xs opacity-70">
                        ({(node.memoryMb / 1024).toFixed(0)}GB)
                      </span>
                    )}
                  </Button>
                ))}
              </div>
              {selectedNodes.length > 0 && (
                <p className="text-xs text-muted-foreground">
                  Selected: {selectedNodes.join(', ')}
                </p>
              )}
            </div>

            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={handleApplyConfig}
                disabled={isPending || selectedNodes.length === 0}
              >
                {isPending ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin mr-1" />
                    Applying...
                  </>
                ) : (
                  'Apply'
                )}
              </Button>
              <Button variant="outline" size="sm" onClick={handleCancelEdit}>
                Cancel
              </Button>
            </div>
          </div>
        )}

        {/* Running VM Warning */}
        {isRunning && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              NUMA configuration cannot be changed while the VM is running.
              Stop the VM to modify NUMA settings.
            </AlertDescription>
          </Alert>
        )}

        {/* No NUMA nodes warning */}
        {hostNodes.length === 0 && (
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Your system does not have multiple NUMA nodes or NUMA is disabled.
              NUMA configuration is only beneficial on multi-socket systems.
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  )
}
