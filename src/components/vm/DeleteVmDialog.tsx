import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { useActiveConnection } from '@/hooks/useActiveConnection'
import { toast } from 'sonner'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import type { VM } from '@/lib/types'

interface DeleteVmDialogProps {
  vm: VM | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function DeleteVmDialog({ vm, open, onOpenChange }: DeleteVmDialogProps) {
  const [deleteDisks, setDeleteDisks] = useState(false)
  const [deleteSnapshots, setDeleteSnapshots] = useState(false)
  const queryClient = useQueryClient()
  const { resourceQueryKey } = useActiveConnection()
  const vmsQueryKey = resourceQueryKey('vms') ?? ['connection', 'pending', 'vms']
  const { data: activeConnection } = useQuery({
    queryKey: ['active-connection'],
    queryFn: api.getActiveConnection,
  })

  const deleteMutation = useMutation({
    mutationFn: ({ vmId, deleteDisks, deleteSnapshots }: {
      vmId: string
      deleteDisks: boolean
      deleteSnapshots: boolean
    }) => api.deleteVm(vmId, deleteDisks, deleteSnapshots),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: vmsQueryKey })
      toast.success(`${vm?.name} deleted`)
      onOpenChange(false)
      // Reset options
      setDeleteDisks(false)
      setDeleteSnapshots(false)
    },
    onError: (error) => toast.error(`Failed to delete: ${error}`),
  })

  const handleDelete = () => {
    if (vm) {
      // Capture the target at click time. The dialog can unmount while the request is queued.
      deleteMutation.mutate({
        vmId: vm.id,
        deleteDisks,
        deleteSnapshots,
      })
    }
  }

  if (!vm) return null

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Virtual Machine</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to delete "{vm.name}"? This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="rounded-md border bg-muted/40 p-3 text-sm">
          <div><span className="font-medium">Target:</span> Virtual machine “{vm.name}”</div>
          <div><span className="font-medium">Connection:</span> {activeConnection?.name ?? 'No selected connection'}</div>
        </div>

        <div className="space-y-3 py-4">
          <div className="flex items-center space-x-2">
            <Checkbox
              id="delete-disks"
              checked={deleteDisks}
              onCheckedChange={(checked) => setDeleteDisks(checked === true)}
            />
            <Label htmlFor="delete-disks" className="text-sm font-normal">
              Delete associated disk files
            </Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox
              id="delete-snapshots"
              checked={deleteSnapshots}
              onCheckedChange={(checked) => setDeleteSnapshots(checked === true)}
            />
            <Label htmlFor="delete-snapshots" className="text-sm font-normal">
              Delete all snapshots
            </Label>
          </div>
        </div>

        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            className="bg-red-600 hover:bg-red-700 focus:ring-red-600"
            disabled={deleteMutation.isPending}
          >
            {deleteMutation.isPending ? 'Deleting...' : 'Delete VM'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
