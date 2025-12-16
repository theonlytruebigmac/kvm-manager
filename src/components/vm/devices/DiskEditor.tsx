import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
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
import { HardDrive, Trash2, FolderOpen, Info } from 'lucide-react'
import { toast } from 'sonner'
import { api } from '@/lib/tauri'
import type { VM } from '@/lib/types'

interface DiskEditorProps {
  vm: VM
  diskIndex: number
  compact?: boolean
}

export function DiskEditor({ vm, diskIndex, compact }: DiskEditorProps) {
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const queryClient = useQueryClient()
  const disk = vm.disks?.[diskIndex]

  // I/O tuning state
  const [cache, setCache] = useState(disk?.cache || 'none')
  const [io, setIo] = useState(disk?.io || 'native')
  const [discard, setDiscard] = useState(disk?.discard || 'unmap')
  const [detectZeroes, setDetectZeroes] = useState(disk?.detectZeroes || 'unmap')
  const [readIopsSec, setReadIopsSec] = useState<string>(disk?.readIopsSec?.toString() || '')
  const [writeIopsSec, setWriteIopsSec] = useState<string>(disk?.writeIopsSec?.toString() || '')
  const [readBytesSec, setReadBytesSec] = useState<string>(disk?.readBytesSec?.toString() || '')
  const [writeBytesSec, setWriteBytesSec] = useState<string>(disk?.writeBytesSec?.toString() || '')

  // Track if settings have changed
  const [hasChanges, setHasChanges] = useState(false)

  const isRunning = vm.state === 'running'

  // Reset state when disk changes
  useEffect(() => {
    if (disk) {
      setCache(disk.cache || 'none')
      setIo(disk.io || 'native')
      setDiscard(disk.discard || 'unmap')
      setDetectZeroes(disk.detectZeroes || 'unmap')
      setReadIopsSec(disk.readIopsSec?.toString() || '')
      setWriteIopsSec(disk.writeIopsSec?.toString() || '')
      setReadBytesSec(disk.readBytesSec?.toString() || '')
      setWriteBytesSec(disk.writeBytesSec?.toString() || '')
      setHasChanges(false)
    }
  }, [disk])

  const detachMutation = useMutation({
    mutationFn: () => api.detachDisk(vm.id, disk?.device || ''),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('Disk detached successfully')
      setShowDeleteDialog(false)
    },
    onError: (error) => {
      toast.error(`Failed to detach disk: ${error}`)
    },
  })

  const updateSettingsMutation = useMutation({
    mutationFn: () => api.updateDiskSettings(vm.id, disk?.device || '', {
      cache: cache !== (disk?.cache || 'none') ? cache : undefined,
      io: io !== (disk?.io || 'native') ? io : undefined,
      discard: discard !== (disk?.discard || 'unmap') ? discard : undefined,
      detectZeroes: detectZeroes !== (disk?.detectZeroes || 'unmap') ? detectZeroes : undefined,
      readIopsSec: readIopsSec ? parseInt(readIopsSec) : undefined,
      writeIopsSec: writeIopsSec ? parseInt(writeIopsSec) : undefined,
      readBytesSec: readBytesSec ? parseInt(readBytesSec) : undefined,
      writeBytesSec: writeBytesSec ? parseInt(writeBytesSec) : undefined,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('Disk settings updated successfully')
      setHasChanges(false)
    },
    onError: (error) => {
      toast.error(`Failed to update disk settings: ${error}`)
    },
  })

  const handleRevert = () => {
    if (disk) {
      setCache(disk.cache || 'none')
      setIo(disk.io || 'native')
      setDiscard(disk.discard || 'unmap')
      setDetectZeroes(disk.detectZeroes || 'unmap')
      setReadIopsSec(disk.readIopsSec?.toString() || '')
      setWriteIopsSec(disk.writeIopsSec?.toString() || '')
      setReadBytesSec(disk.readBytesSec?.toString() || '')
      setWriteBytesSec(disk.writeBytesSec?.toString() || '')
      setHasChanges(false)
    }
  }

  const markChanged = () => setHasChanges(true)

  if (!disk) {
    return (
      <div className={compact ? 'p-2' : 'p-6'}>
        <div className="flex items-center gap-2 text-muted-foreground">
          <HardDrive className="w-4 h-4" />
          <span className="text-sm">No disk found</span>
        </div>
      </div>
    )
  }

  // Compact mode shows inline summary
  if (compact) {
    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Badge variant={disk.bus === 'virtio' ? 'default' : 'secondary'} className="text-xs">
              {disk.bus?.toUpperCase() || 'VIRTIO'}
            </Badge>
            <span className="text-sm font-medium">Disk {diskIndex + 1}</span>
          </div>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 text-red-600 hover:text-red-700 hover:bg-red-50"
            onClick={() => setShowDeleteDialog(true)}
          >
            <Trash2 className="w-3.5 h-3.5" />
          </Button>
        </div>
        <div className="text-sm text-muted-foreground truncate">
          {disk.path || 'No path'}
        </div>
        <div className="flex gap-4 text-xs text-muted-foreground">
          <span>Format: {disk.diskType || 'qcow2'}</span>
          <span>Device: {disk.device || 'disk'}</span>
          {disk.cache && <span>Cache: {disk.cache}</span>}
        </div>

        <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Detach Disk?</AlertDialogTitle>
              <AlertDialogDescription>
                This will detach the disk from the VM. The disk image file will not be deleted.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction
                onClick={() => detachMutation.mutate()}
                className="bg-red-600 hover:bg-red-700"
              >
                Detach
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold mb-2 flex items-center gap-2">
            <HardDrive className="w-6 h-6" />
            Disk {diskIndex + 1}
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure storage device settings
          </p>
        </div>
        <Badge variant={disk.bus === 'virtio' ? 'default' : 'secondary'}>
          {disk.bus?.toUpperCase() || 'VIRTIO'}
        </Badge>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="performance">Performance</TabsTrigger>
          <TabsTrigger value="throttling">I/O Throttling</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Disk Source</CardTitle>
              <CardDescription>
                The storage file or device backing this disk
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="path">Disk Image Path</Label>
                <div className="flex gap-2">
                  <Input
                    id="path"
                    value={disk.path || ''}
                    placeholder="/var/lib/libvirt/images/disk.qcow2"
                    readOnly
                    className="flex-1"
                  />
                  <Button variant="outline" size="icon" disabled>
                    <FolderOpen className="w-4 h-4" />
                  </Button>
                </div>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="format">Disk Format</Label>
                  <Select value={disk.diskType || 'qcow2'} disabled>
                    <SelectTrigger>
                      <SelectValue placeholder="Select format" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="qcow2">QCOW2 (Recommended)</SelectItem>
                      <SelectItem value="raw">Raw</SelectItem>
                      <SelectItem value="vmdk">VMDK</SelectItem>
                      <SelectItem value="vdi">VDI</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    QCOW2 supports snapshots and thin provisioning
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="device">Device Type</Label>
                  <Select value={disk.device || 'disk'} disabled>
                    <SelectTrigger>
                      <SelectValue placeholder="Select device type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="disk">Disk</SelectItem>
                      <SelectItem value="cdrom">CD-ROM</SelectItem>
                      <SelectItem value="floppy">Floppy</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Bus Configuration</CardTitle>
              <CardDescription>
                How the disk is connected to the virtual machine
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="bus">Bus Type</Label>
                  <Select value={disk.bus || 'virtio'} disabled>
                    <SelectTrigger>
                      <SelectValue placeholder="Select bus type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="virtio">VirtIO (Recommended)</SelectItem>
                      <SelectItem value="sata">SATA</SelectItem>
                      <SelectItem value="scsi">SCSI</SelectItem>
                      <SelectItem value="ide">IDE (Legacy)</SelectItem>
                      <SelectItem value="usb">USB</SelectItem>
                      <SelectItem value="nvme">NVMe</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    VirtIO provides best performance for Linux/Windows guests
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>Target Device</Label>
                  <Input
                    value={disk.device || 'vda'}
                    disabled
                  />
                  <p className="text-xs text-muted-foreground">
                    Device name as seen by the guest OS
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="performance" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Cache & I/O Mode</CardTitle>
              <CardDescription>
                Configure caching and I/O behavior for this disk
                {isRunning && (
                  <span className="block mt-1 text-yellow-600 dark:text-yellow-500">
                    <Info className="w-3 h-3 inline mr-1" />
                    VM must be stopped to change cache, I/O mode, and discard settings
                  </span>
                )}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label>Cache Mode</Label>
                  <Select
                    value={cache}
                    onValueChange={(v) => { setCache(v); markChanged() }}
                    disabled={isRunning}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select cache mode" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="none">None (Recommended for data safety)</SelectItem>
                      <SelectItem value="writeback">Writeback (Better performance)</SelectItem>
                      <SelectItem value="writethrough">Writethrough (Safe, slower)</SelectItem>
                      <SelectItem value="directsync">Directsync (Safe, O_DIRECT)</SelectItem>
                      <SelectItem value="unsafe">Unsafe (Fastest, risky)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    'None' is safest. 'Writeback' improves performance but may lose data on crash.
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>I/O Mode</Label>
                  <Select
                    value={io}
                    onValueChange={(v) => { setIo(v); markChanged() }}
                    disabled={isRunning}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select I/O mode" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="native">Native (O_DIRECT, recommended)</SelectItem>
                      <SelectItem value="threads">Threads (Thread pool)</SelectItem>
                      <SelectItem value="io_uring">io_uring (Linux 5.1+, fastest)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    'io_uring' is fastest on modern Linux. 'Native' is most compatible.
                  </p>
                </div>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label>Discard Mode</Label>
                  <Select
                    value={discard}
                    onValueChange={(v) => { setDiscard(v); markChanged() }}
                    disabled={isRunning}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select discard mode" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="unmap">Unmap (Enable TRIM)</SelectItem>
                      <SelectItem value="ignore">Ignore (Disable TRIM)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    'Unmap' enables TRIM for SSD optimization and sparse file shrinking
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>Detect Zeroes</Label>
                  <Select
                    value={detectZeroes}
                    onValueChange={(v) => { setDetectZeroes(v); markChanged() }}
                    disabled={isRunning}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select detect zeroes" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="off">Off</SelectItem>
                      <SelectItem value="on">On</SelectItem>
                      <SelectItem value="unmap">Unmap</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Detect and optimize zero-write operations
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="throttling" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>I/O Throttling</CardTitle>
              <CardDescription>
                Limit disk I/O to prevent a single VM from monopolizing storage
                {isRunning && (
                  <span className="block mt-1 text-green-600 dark:text-green-500">
                    <Info className="w-3 h-3 inline mr-1" />
                    I/O throttling can be changed while VM is running
                  </span>
                )}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label>Read IOPS Limit</Label>
                  <Input
                    type="number"
                    min={0}
                    placeholder="Unlimited"
                    value={readIopsSec}
                    onChange={(e) => { setReadIopsSec(e.target.value); markChanged() }}
                  />
                  <p className="text-xs text-muted-foreground">
                    Maximum read operations per second (0 = unlimited)
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>Write IOPS Limit</Label>
                  <Input
                    type="number"
                    min={0}
                    placeholder="Unlimited"
                    value={writeIopsSec}
                    onChange={(e) => { setWriteIopsSec(e.target.value); markChanged() }}
                  />
                  <p className="text-xs text-muted-foreground">
                    Maximum write operations per second (0 = unlimited)
                  </p>
                </div>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label>Read Bandwidth Limit (bytes/sec)</Label>
                  <Input
                    type="number"
                    min={0}
                    placeholder="Unlimited"
                    value={readBytesSec}
                    onChange={(e) => { setReadBytesSec(e.target.value); markChanged() }}
                  />
                  <p className="text-xs text-muted-foreground">
                    Maximum read throughput (e.g., 104857600 = 100 MB/s)
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>Write Bandwidth Limit (bytes/sec)</Label>
                  <Input
                    type="number"
                    min={0}
                    placeholder="Unlimited"
                    value={writeBytesSec}
                    onChange={(e) => { setWriteBytesSec(e.target.value); markChanged() }}
                  />
                  <p className="text-xs text-muted-foreground">
                    Maximum write throughput (e.g., 104857600 = 100 MB/s)
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Disk XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for this disk device
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {`<disk type='file' device='disk'>
  <driver name='qemu' type='${disk.diskType || 'qcow2'}' cache='${cache}' io='${io}' discard='${discard}'${detectZeroes !== 'off' ? ` detect_zeroes='${detectZeroes}'` : ''}/>
  <source file='${disk.path || ''}'/>
  <target dev='${disk.device || 'vda'}' bus='${disk.bus || 'virtio'}'/>${(readIopsSec || writeIopsSec || readBytesSec || writeBytesSec) ? `
  <iotune>${readIopsSec ? `
    <read_iops_sec>${readIopsSec}</read_iops_sec>` : ''}${writeIopsSec ? `
    <write_iops_sec>${writeIopsSec}</write_iops_sec>` : ''}${readBytesSec ? `
    <read_bytes_sec>${readBytesSec}</read_bytes_sec>` : ''}${writeBytesSec ? `
    <write_bytes_sec>${writeBytesSec}</write_bytes_sec>` : ''}
  </iotune>` : ''}
</disk>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <div className="flex justify-between">
        <Button
          variant="destructive"
          size="sm"
          className="gap-1.5"
          onClick={() => setShowDeleteDialog(true)}
          disabled={detachMutation.isPending || diskIndex === 0}
          title={diskIndex === 0 ? 'Cannot remove primary disk' : 'Remove this disk'}
        >
          <Trash2 className="w-4 h-4" />
          Remove Disk
        </Button>
        <div className="flex gap-2">
          <Button
            variant="outline"
            disabled={!hasChanges}
            onClick={handleRevert}
          >
            Revert
          </Button>
          <Button
            disabled={!hasChanges || updateSettingsMutation.isPending}
            onClick={() => updateSettingsMutation.mutate()}
          >
            {updateSettingsMutation.isPending ? 'Applying...' : 'Apply'}
          </Button>
        </div>
      </div>

      {/* Remove Confirmation Dialog */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Detach Disk?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to detach this disk from the VM?
              The disk file will NOT be deleted, it can be reattached later.
              <div className="mt-2 p-2 bg-muted rounded text-xs font-mono">
                {disk?.path || 'Unknown path'}
              </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => detachMutation.mutate()}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={detachMutation.isPending}
            >
              {detachMutation.isPending ? 'Detaching...' : 'Detach Disk'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
