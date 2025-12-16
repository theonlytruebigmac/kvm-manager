import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { VM } from '@/lib/types'

interface RenameVmDialogProps {
  vm: VM | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function RenameVmDialog({ vm, open, onOpenChange }: RenameVmDialogProps) {
  const [newName, setNewName] = useState('')
  const queryClient = useQueryClient()

  useEffect(() => {
    if (vm && open) {
      setNewName(vm.name)
    }
  }, [vm, open])

  const renameMutation = useMutation({
    mutationFn: () => api.renameVm(vm!.id, newName.trim()),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM renamed to ${newName.trim()}`)
      onOpenChange(false)
    },
    onError: (error) => toast.error(`Failed to rename: ${error}`),
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (vm && newName.trim() && newName.trim() !== vm.name) {
      renameMutation.mutate()
    }
  }

  if (!vm) return null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[400px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Rename Virtual Machine</DialogTitle>
            <DialogDescription>
              Enter a new name for "{vm.name}"
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label htmlFor="vm-name" className="sr-only">Name</Label>
            <Input
              id="vm-name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Enter new VM name"
              autoFocus
            />
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={!newName.trim() || newName.trim() === vm.name || renameMutation.isPending}
            >
              {renameMutation.isPending ? 'Renaming...' : 'Rename'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
