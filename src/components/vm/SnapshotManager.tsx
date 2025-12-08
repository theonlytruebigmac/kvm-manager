import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import { Checkbox } from '@/components/ui/checkbox'
import { Camera, Trash2, RotateCcw, Clock } from 'lucide-react'
import type { Snapshot } from '@/lib/types'

interface SnapshotManagerProps {
  vmId: string
  vmName: string
}

export function SnapshotManager({ vmId, vmName }: SnapshotManagerProps) {
  const queryClient = useQueryClient()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [showRevertDialog, setShowRevertDialog] = useState(false)
  const [selectedSnapshot, setSelectedSnapshot] = useState<Snapshot | null>(null)

  const [snapshotName, setSnapshotName] = useState('')
  const [snapshotDescription, setSnapshotDescription] = useState('')
  const [includeMemory, setIncludeMemory] = useState(false)

  // Query snapshots
  const { data: snapshots = [], isLoading, error } = useQuery({
    queryKey: ['snapshots', vmId],
    queryFn: () => api.getSnapshots(vmId),
    refetchInterval: 30000, // Refresh every 30 seconds
  })

  // Create snapshot mutation
  const createMutation = useMutation({
    mutationFn: () => api.createSnapshot(vmId, {
      name: snapshotName,
      description: snapshotDescription || undefined,
      includeMemory,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['snapshots', vmId] })
      toast.success(`Snapshot "${snapshotName}" created successfully`)
      setShowCreateDialog(false)
      setSnapshotName('')
      setSnapshotDescription('')
      setIncludeMemory(false)
    },
    onError: (error) => {
      toast.error(`Failed to create snapshot: ${error}`)
    },
  })

  // Delete snapshot mutation
  const deleteMutation = useMutation({
    mutationFn: (snapshotName: string) => api.deleteSnapshot(vmId, snapshotName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['snapshots', vmId] })
      toast.success(`Snapshot deleted successfully`)
      setShowDeleteDialog(false)
      setSelectedSnapshot(null)
    },
    onError: (error) => {
      toast.error(`Failed to delete snapshot: ${error}`)
    },
  })

  // Revert snapshot mutation
  const revertMutation = useMutation({
    mutationFn: (snapshotName: string) => api.revertSnapshot(vmId, snapshotName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['snapshots', vmId] })
      toast.success(`VM reverted to snapshot successfully`)
      setShowRevertDialog(false)
      setSelectedSnapshot(null)
    },
    onError: (error) => {
      toast.error(`Failed to revert snapshot: ${error}`)
    },
  })

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString()
  }

  const getStateIcon = (state: Snapshot['state']) => {
    switch (state) {
      case 'running':
        return '🟢'
      case 'paused':
        return '🟡'
      case 'shutoff':
        return '🔴'
      case 'disksnapshot':
        return '💾'
      default:
        return '❓'
    }
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Snapshots</CardTitle>
          <CardDescription>Error loading snapshots: {String(error)}</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Snapshots</CardTitle>
              <CardDescription>Manage VM snapshots for {vmName}</CardDescription>
            </div>
            <Button
              onClick={() => setShowCreateDialog(true)}
              size="sm"
            >
              <Camera className="mr-2 h-4 w-4" />
              Create Snapshot
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-sm text-muted-foreground">Loading snapshots...</p>
          ) : snapshots.length === 0 ? (
            <p className="text-sm text-muted-foreground">No snapshots yet. Create one to save the current VM state.</p>
          ) : (
            <div className="space-y-3">
              {snapshots.map((snapshot) => (
                <div
                  key={snapshot.name}
                  className={`flex items-center justify-between p-3 border rounded-lg ${
                    snapshot.isCurrent ? 'border-primary bg-primary/5' : ''
                  }`}
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="text-lg">{getStateIcon(snapshot.state)}</span>
                      <div>
                        <p className="font-medium flex items-center gap-2">
                          {snapshot.name}
                          {snapshot.isCurrent && (
                            <span className="text-xs bg-primary text-primary-foreground px-2 py-0.5 rounded">
                              Current
                            </span>
                          )}
                        </p>
                        {snapshot.description && (
                          <p className="text-sm text-muted-foreground">{snapshot.description}</p>
                        )}
                        <div className="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {formatDate(snapshot.creationTime)}
                          </span>
                          <span className="capitalize">{snapshot.state}</span>
                          {snapshot.parent && (
                            <span className="flex items-center gap-1">
                              Parent: {snapshot.parent}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setSelectedSnapshot(snapshot)
                        setShowRevertDialog(true)
                      }}
                      disabled={revertMutation.isPending}
                    >
                      <RotateCcw className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setSelectedSnapshot(snapshot)
                        setShowDeleteDialog(true)
                      }}
                      disabled={deleteMutation.isPending}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Create Snapshot Dialog */}
      <AlertDialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Create Snapshot</AlertDialogTitle>
            <AlertDialogDescription>
              Create a snapshot of the current VM state. You can restore to this point later.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="snapshot-name">Snapshot Name</Label>
              <Input
                id="snapshot-name"
                value={snapshotName}
                onChange={(e) => setSnapshotName(e.target.value)}
                placeholder="e.g., before-update"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="snapshot-description">Description (Optional)</Label>
              <Input
                id="snapshot-description"
                value={snapshotDescription}
                onChange={(e) => setSnapshotDescription(e.target.value)}
                placeholder="e.g., Before installing new software"
              />
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="include-memory"
                checked={includeMemory}
                onCheckedChange={(checked) => setIncludeMemory(checked as boolean)}
              />
              <Label htmlFor="include-memory" className="text-sm cursor-pointer">
                Include memory state (for running VMs)
              </Label>
            </div>
            <p className="text-xs text-muted-foreground">
              {includeMemory ? (
                <>Memory snapshots are larger but allow restoring running VMs to their exact state.</>
              ) : (
                <>Disk-only snapshots are faster and smaller, but VMs will restart from shutdown state.</>
              )}
            </p>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => createMutation.mutate()}
              disabled={!snapshotName.trim() || createMutation.isPending}
            >
              {createMutation.isPending ? 'Creating...' : 'Create Snapshot'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Delete Snapshot Confirmation */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Snapshot</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete snapshot "{selectedSnapshot?.name}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setSelectedSnapshot(null)}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => selectedSnapshot && deleteMutation.mutate(selectedSnapshot.name)}
              disabled={deleteMutation.isPending}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleteMutation.isPending ? 'Deleting...' : 'Delete Snapshot'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Revert Snapshot Confirmation */}
      <AlertDialog open={showRevertDialog} onOpenChange={setShowRevertDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Revert to Snapshot</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to revert VM "{vmName}" to snapshot "{selectedSnapshot?.name}"?
              All changes made after this snapshot will be lost.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setSelectedSnapshot(null)}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => selectedSnapshot && revertMutation.mutate(selectedSnapshot.name)}
              disabled={revertMutation.isPending}
            >
              {revertMutation.isPending ? 'Reverting...' : 'Revert to Snapshot'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
