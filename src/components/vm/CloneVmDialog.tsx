import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Copy } from 'lucide-react'
import { toast } from 'sonner'

interface CloneVmDialogProps {
  vmId: string
  vmName: string
  trigger?: React.ReactNode
}

export function CloneVmDialog({ vmId, vmName, trigger }: CloneVmDialogProps) {
  const [open, setOpen] = useState(false)
  const [newName, setNewName] = useState(`${vmName}-clone`)
  const queryClient = useQueryClient()

  const cloneMutation = useMutation({
    mutationFn: () => api.cloneVm(vmId, newName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM Cloned', {
        description: `Successfully cloned ${vmName} to ${newName}`,
      })
      setOpen(false)
      setNewName(`${vmName}-clone`) // Reset for next time
    },
    onError: (error: Error) => {
      toast.error('Clone Failed', {
        description: error.message,
      })
    },
  })

  const handleClone = () => {
    if (!newName.trim()) {
      toast.error('Name Required', {
        description: 'Please enter a name for the cloned VM',
      })
      return
    }
    cloneMutation.mutate()
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || (
          <Button variant="outline" size="sm">
            <Copy className="mr-2 h-4 w-4" />
            Clone
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Clone Virtual Machine</DialogTitle>
          <DialogDescription>
            Create a copy of "{vmName}". The clone will have a new UUID and MAC addresses.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="clone-name">New VM Name</Label>
            <Input
              id="clone-name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Enter name for cloned VM"
              disabled={cloneMutation.isPending}
            />
            <p className="text-sm text-muted-foreground">
              The new VM will be created in a shutdown state.
            </p>
          </div>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setOpen(false)}
            disabled={cloneMutation.isPending}
          >
            Cancel
          </Button>
          <Button
            onClick={handleClone}
            disabled={cloneMutation.isPending || !newName.trim()}
          >
            {cloneMutation.isPending ? 'Cloning...' : 'Clone VM'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
