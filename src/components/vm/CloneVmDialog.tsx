import { useState, useEffect } from 'react'
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Textarea } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { Copy, ChevronDown, HardDrive, Camera, Info, AlertTriangle } from 'lucide-react'
import { toast } from 'sonner'
import type { CloneConfig, StoragePool } from '@/lib/types'
import { Alert, AlertDescription } from '@/components/ui/alert'

interface CloneVmDialogProps {
  vmId: string
  vmName: string
  trigger?: React.ReactNode
}

export function CloneVmDialog({ vmId, vmName, trigger }: CloneVmDialogProps) {
  const [open, setOpen] = useState(false)
  const [newName, setNewName] = useState(`${vmName}-clone`)
  const [cloneDisks, setCloneDisks] = useState(true)
  const [cloneSnapshots, setCloneSnapshots] = useState(false)
  const [targetPool, setTargetPool] = useState<string>('')
  const [description, setDescription] = useState('')
  const [showAdvanced, setShowAdvanced] = useState(false)
  const queryClient = useQueryClient()

  // Fetch storage pools for target pool selection
  const { data: storagePools = [] } = useQuery<StoragePool[]>({
    queryKey: ['storage-pools'],
    queryFn: () => api.getStoragePools(),
    enabled: open && showAdvanced,
  })

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      setNewName(`${vmName}-clone`)
      setCloneDisks(true)
      setCloneSnapshots(false)
      setTargetPool('')
      setDescription('')
    }
  }, [open, vmName])

  const cloneMutation = useMutation({
    mutationFn: () => {
      const config: CloneConfig = {
        newName,
        cloneDisks,
        cloneSnapshots,
        targetPool: targetPool || undefined,
        description: description || undefined,
      }
      return api.cloneVmWithOptions(vmId, config)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('VM Cloned', {
        description: `Successfully cloned ${vmName} to ${newName}`,
      })
      setOpen(false)
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

  // Filter active storage pools
  const activePools = storagePools.filter(p => p.state === 'active')

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
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Clone Virtual Machine</DialogTitle>
          <DialogDescription>
            Create a copy of "{vmName}". The clone will have a new UUID and MAC addresses.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* Basic Settings */}
          <div className="grid gap-2">
            <Label htmlFor="clone-name">New VM Name</Label>
            <Input
              id="clone-name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Enter name for cloned VM"
              disabled={cloneMutation.isPending}
            />
          </div>

          {/* Clone Options */}
          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="clone-disks"
                checked={cloneDisks}
                onCheckedChange={(checked) => setCloneDisks(checked === true)}
                disabled={cloneMutation.isPending}
              />
              <Label htmlFor="clone-disks" className="flex items-center gap-2">
                <HardDrive className="h-4 w-4" />
                Clone disk files
              </Label>
            </div>
            <p className="text-xs text-muted-foreground ml-6">
              {cloneDisks
                ? "Create independent copies of all disk files. The clone will be completely independent."
                : "Reference the original disk files. Changes to the clone may affect the original VM."}
            </p>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="clone-snapshots"
                checked={cloneSnapshots}
                onCheckedChange={(checked) => setCloneSnapshots(checked === true)}
                disabled={cloneMutation.isPending}
              />
              <Label htmlFor="clone-snapshots" className="flex items-center gap-2">
                <Camera className="h-4 w-4" />
                Clone snapshots
              </Label>
            </div>
            <p className="text-xs text-muted-foreground ml-6">
              Copy snapshot metadata to the cloned VM (disk-only snapshots).
            </p>
          </div>

          {/* Warning if not cloning disks */}
          {!cloneDisks && (
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                Not cloning disks means the clone will share disk files with the original VM.
                This is useful for creating linked clones but requires care to avoid data corruption.
              </AlertDescription>
            </Alert>
          )}

          {/* Advanced Options */}
          <Collapsible open={showAdvanced} onOpenChange={setShowAdvanced}>
            <CollapsibleTrigger asChild>
              <Button variant="ghost" className="w-full justify-between">
                Advanced Options
                <ChevronDown className={`h-4 w-4 transition-transform ${showAdvanced ? 'rotate-180' : ''}`} />
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="space-y-4 pt-2">
              {/* Target Storage Pool */}
              {cloneDisks && (
                <div className="grid gap-2">
                  <Label htmlFor="target-pool">Target Storage Pool</Label>
                  <Select value={targetPool} onValueChange={setTargetPool}>
                    <SelectTrigger>
                      <SelectValue placeholder="Same as source" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="">Same as source</SelectItem>
                      {activePools.map((pool) => (
                        <SelectItem key={pool.name} value={pool.name}>
                          {pool.name} ({pool.poolType})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Choose where to store the cloned disk files.
                  </p>
                </div>
              )}

              {/* Description */}
              <div className="grid gap-2">
                <Label htmlFor="description">Description (optional)</Label>
                <Textarea
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Enter a description for the cloned VM"
                  disabled={cloneMutation.isPending}
                  rows={2}
                />
              </div>
            </CollapsibleContent>
          </Collapsible>

          {/* Info */}
          <div className="flex items-start gap-2 text-sm text-muted-foreground">
            <Info className="h-4 w-4 mt-0.5 shrink-0" />
            <p>
              The new VM will be created in a shutdown state.
              {cloneDisks && " Disk cloning may take some time depending on disk size."}
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
