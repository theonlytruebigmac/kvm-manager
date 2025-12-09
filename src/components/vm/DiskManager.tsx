import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { HardDrive, Plus, Trash2 } from 'lucide-react'

interface DiskManagerProps {
  vmId: string
  vmName: string
  disks?: Array<{
    device: string
    path: string
    type: string
  }>
}

export function DiskManager({ vmId, vmName, disks = [] }: DiskManagerProps) {
  const queryClient = useQueryClient()
  const [showAttachDialog, setShowAttachDialog] = useState(false)
  const [diskPath, setDiskPath] = useState('')
  const [deviceTarget, setDeviceTarget] = useState('vdb')
  const [busType, setBusType] = useState('virtio')
  const [detachTarget, setDetachTarget] = useState<string | null>(null)

  // Attach disk mutation
  const attachMutation = useMutation({
    mutationFn: ({ diskPath, deviceTarget, busType }: { diskPath: string; deviceTarget: string; busType: string }) =>
      api.attachDisk(vmId, diskPath, deviceTarget, busType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success(`Disk attached to ${vmName}`)
      setShowAttachDialog(false)
      setDiskPath('')
      setDeviceTarget('vdb')
      setBusType('virtio')
    },
    onError: (error) => {
      toast.error(`Failed to attach disk: ${error}`)
    },
  })

  // Detach disk mutation
  const detachMutation = useMutation({
    mutationFn: (deviceTarget: string) => api.detachDisk(vmId, deviceTarget),
    onSuccess: (_data, deviceTarget) => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success(`Disk ${deviceTarget} detached from ${vmName}`)
      setDetachTarget(null)
    },
    onError: (error, deviceTarget) => {
      toast.error(`Failed to detach disk ${deviceTarget}: ${error}`)
    },
  })

  // Generate next available device target
  const getNextDeviceTarget = () => {
    const usedTargets = disks.map(d => d.device)
    const prefix = busType === 'virtio' ? 'vd' : busType === 'scsi' ? 'sd' : 'hd'

    for (let i = 98; i <= 122; i++) { // ASCII codes for 'b' to 'z'
      const target = `${prefix}${String.fromCharCode(i)}`
      if (!usedTargets.includes(target)) {
        return target
      }
    }
    return 'vdb'
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <HardDrive className="h-5 w-5" />
              Disk Management
            </CardTitle>
            <CardDescription>Attach and detach disks from this VM</CardDescription>
          </div>
          <Button
            size="sm"
            onClick={() => {
              setDeviceTarget(getNextDeviceTarget())
              setShowAttachDialog(true)
            }}
          >
            <Plus className="mr-2 h-4 w-4" />
            Attach Disk
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {disks.length === 0 ? (
          <div className="text-center py-8">
            <HardDrive className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <p className="text-sm text-muted-foreground">No additional disks attached</p>
            <p className="text-xs text-muted-foreground mt-1">
              The primary boot disk is managed automatically
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {disks.map((disk) => (
              <div
                key={disk.device}
                className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors group"
              >
                <div className="flex-1">
                  <p className="font-medium">{disk.device}</p>
                  <p className="text-xs text-muted-foreground truncate max-w-md">
                    {disk.path}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">{disk.type}</span>
                  <AlertDialog
                    open={detachTarget === disk.device}
                    onOpenChange={(open) => !open && setDetachTarget(null)}
                  >
                    <AlertDialogTrigger asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="opacity-0 group-hover:opacity-100 transition-opacity"
                        onClick={() => setDetachTarget(disk.device)}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>Detach Disk?</AlertDialogTitle>
                        <AlertDialogDescription>
                          Are you sure you want to detach disk "{disk.device}" from {vmName}?
                          The disk file will not be deleted, but the VM will lose access to it.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                          onClick={() => detachMutation.mutate(disk.device)}
                          className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                        >
                          Detach Disk
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>

      {/* Attach Disk Dialog */}
      <AlertDialog open={showAttachDialog} onOpenChange={setShowAttachDialog}>
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Attach Disk to {vmName}</AlertDialogTitle>
            <AlertDialogDescription>
              Attach an existing disk image to this virtual machine.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="disk-path">Disk Image Path</Label>
              <Input
                id="disk-path"
                value={diskPath}
                onChange={(e) => setDiskPath(e.target.value)}
                placeholder="/var/lib/libvirt/images/disk.qcow2"
              />
              <p className="text-xs text-muted-foreground">
                Full path to an existing disk image file
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="bus-type">Bus Type</Label>
              <Select value={busType} onValueChange={(v) => {
                setBusType(v)
                // Update device target prefix when bus type changes
                const prefix = v === 'virtio' ? 'vd' : v === 'scsi' ? 'sd' : 'hd'
                const suffix = deviceTarget.slice(-1)
                setDeviceTarget(`${prefix}${suffix}`)
              }}>
                <SelectTrigger id="bus-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="virtio">VirtIO (Recommended)</SelectItem>
                  <SelectItem value="scsi">SCSI</SelectItem>
                  <SelectItem value="sata">SATA</SelectItem>
                  <SelectItem value="ide">IDE</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="device-target">Device Target</Label>
              <Input
                id="device-target"
                value={deviceTarget}
                onChange={(e) => setDeviceTarget(e.target.value)}
                placeholder="vdb"
              />
              <p className="text-xs text-muted-foreground">
                Device name in the guest (e.g., vdb, vdc, sdb)
              </p>
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => attachMutation.mutate({ diskPath, deviceTarget, busType })}
              disabled={!diskPath.trim() || !deviceTarget.trim() || attachMutation.isPending}
            >
              {attachMutation.isPending ? 'Attaching...' : 'Attach Disk'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  )
}
