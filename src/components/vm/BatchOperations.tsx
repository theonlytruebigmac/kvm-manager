import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { BatchOperationResult } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { PlayCircle, StopCircle, RotateCw, CheckCircle, XCircle } from 'lucide-react'

interface BatchOperationsProps {
  selectedVmIds: string[]
  onClearSelection: () => void
}

export function BatchOperations({ selectedVmIds, onClearSelection }: BatchOperationsProps) {
  const queryClient = useQueryClient()
  const [showResults, setShowResults] = useState(false)
  const [results, setResults] = useState<BatchOperationResult[]>([])
  const [operationType, setOperationType] = useState<string>('')

  // Batch start mutation
  const batchStartMutation = useMutation({
    mutationFn: (vmIds: string[]) => api.batchStartVms(vmIds),
    onSuccess: (data) => {
      setResults(data)
      setOperationType('Start')
      setShowResults(true)
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      const successCount = data.filter(r => r.success).length
      toast.success(`Started ${successCount} of ${data.length} VMs`)
    },
    onError: (error: Error) => {
      toast.error(`Batch start failed: ${error.message}`)
    },
  })

  // Batch stop mutation
  const batchStopMutation = useMutation({
    mutationFn: ({ vmIds, force }: { vmIds: string[]; force: boolean }) =>
      api.batchStopVms(vmIds, force),
    onSuccess: (data) => {
      setResults(data)
      setOperationType('Stop')
      setShowResults(true)
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      const successCount = data.filter(r => r.success).length
      toast.success(`Stopped ${successCount} of ${data.length} VMs`)
    },
    onError: (error: Error) => {
      toast.error(`Batch stop failed: ${error.message}`)
    },
  })

  // Batch reboot mutation
  const batchRebootMutation = useMutation({
    mutationFn: (vmIds: string[]) => api.batchRebootVms(vmIds),
    onSuccess: (data) => {
      setResults(data)
      setOperationType('Reboot')
      setShowResults(true)
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      const successCount = data.filter(r => r.success).length
      toast.success(`Rebooted ${successCount} of ${data.length} VMs`)
    },
    onError: (error: Error) => {
      toast.error(`Batch reboot failed: ${error.message}`)
    },
  })

  const handleBatchStart = () => {
    if (selectedVmIds.length === 0) {
      toast.error('No VMs selected')
      return
    }
    if (confirm(`Are you sure you want to start ${selectedVmIds.length} VMs?`)) {
      batchStartMutation.mutate(selectedVmIds)
    }
  }

  const handleBatchStop = (force: boolean = false) => {
    if (selectedVmIds.length === 0) {
      toast.error('No VMs selected')
      return
    }
    const action = force ? 'force stop' : 'shutdown'
    if (confirm(`Are you sure you want to ${action} ${selectedVmIds.length} VMs?`)) {
      batchStopMutation.mutate({ vmIds: selectedVmIds, force })
    }
  }

  const handleBatchReboot = () => {
    if (selectedVmIds.length === 0) {
      toast.error('No VMs selected')
      return
    }
    if (confirm(`Are you sure you want to reboot ${selectedVmIds.length} VMs?`)) {
      batchRebootMutation.mutate(selectedVmIds)
    }
  }

  const handleCloseResults = () => {
    setShowResults(false)
    setResults([])
    onClearSelection()
  }

  const isLoading =
    batchStartMutation.isPending ||
    batchStopMutation.isPending ||
    batchRebootMutation.isPending

  if (selectedVmIds.length === 0) {
    return null
  }

  return (
    <>
      <div className="flex-shrink-0 px-4 py-2 border-b border-primary/20 bg-primary/5">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium">
            {selectedVmIds.length} VM{selectedVmIds.length !== 1 ? 's' : ''} selected
          </span>
          <button
            onClick={onClearSelection}
            className="text-sm text-muted-foreground hover:text-foreground"
          >
            Clear selection
          </button>
          <div className="h-4 w-px bg-border mx-1" />
          <div className="flex gap-1.5">
            <Button
              size="sm"
              variant="ghost"
              onClick={handleBatchStart}
              disabled={isLoading}
              className="h-7 px-2.5 text-xs"
            >
              <PlayCircle className="mr-1 h-3.5 w-3.5" />
              Start All
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => handleBatchStop(false)}
              disabled={isLoading}
              className="h-7 px-2.5 text-xs"
            >
              <StopCircle className="mr-1 h-3.5 w-3.5" />
              Shutdown All
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => handleBatchStop(true)}
              disabled={isLoading}
              className="h-7 px-2.5 text-xs"
            >
              <StopCircle className="mr-1 h-3.5 w-3.5" />
              Force Stop All
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={handleBatchReboot}
              disabled={isLoading}
              className="h-7 px-2.5 text-xs"
            >
              <RotateCw className="mr-1 h-3.5 w-3.5" />
              Reboot All
            </Button>
          </div>
        </div>
      </div>

      {/* Results Dialog */}
      <Dialog open={showResults} onOpenChange={setShowResults}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>Batch {operationType} Results</DialogTitle>
            <DialogDescription>
              Operation completed for {results.filter(r => r.success).length} of{' '}
              {results.length} VMs
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {results.map((result) => (
              <div
                key={result.vmId}
                className="flex items-center justify-between p-3 border rounded-lg"
              >
                <div className="flex items-center gap-3">
                  {result.success ? (
                    <CheckCircle className="h-5 w-5 text-green-500" />
                  ) : (
                    <XCircle className="h-5 w-5 text-red-500" />
                  )}
                  <div>
                    <div className="font-medium">{result.vmName}</div>
                    {result.error && (
                      <div className="text-xs text-muted-foreground text-red-500">
                        {result.error}
                      </div>
                    )}
                  </div>
                </div>
                <Badge variant={result.success ? 'default' : 'destructive'}>
                  {result.success ? 'Success' : 'Failed'}
                </Badge>
              </div>
            ))}
          </div>
          <DialogFooter>
            <Button onClick={handleCloseResults}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
