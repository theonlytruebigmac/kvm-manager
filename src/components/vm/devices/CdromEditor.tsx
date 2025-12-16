import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Disc, Trash2, FolderOpen, DiscAlbum, Loader2, FileArchive } from 'lucide-react'
import { useState, useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { api } from '@/lib/tauri'
import type { VM } from '@/lib/types'

interface CdromEditorProps {
  vm: VM
  compact?: boolean
}

export function CdromEditor({ vm, compact }: CdromEditorProps) {
  const cdromPath = typeof vm.cdrom === 'string' ? vm.cdrom : ''
  const [showIsoBrowser, setShowIsoBrowser] = useState(false)
  const [selectedPool, setSelectedPool] = useState<string>('')
  const [selectedIso, setSelectedIso] = useState<string>('')
  const queryClient = useQueryClient()

  // Fetch storage pools
  const { data: storagePools = [], isLoading: poolsLoading } = useQuery({
    queryKey: ['storage-pools'],
    queryFn: () => api.getStoragePools(),
  })

  // Fetch volumes for selected pool
  const { data: volumes = [], isLoading: volumesLoading } = useQuery({
    queryKey: ['volumes', selectedPool],
    queryFn: () => api.getVolumes(selectedPool),
    enabled: !!selectedPool,
  })

  // Filter ISO files
  const isoVolumes = volumes.filter(v =>
    v.path?.toLowerCase().endsWith('.iso') ||
    v.name?.toLowerCase().endsWith('.iso')
  )

  // Auto-select first pool
  useEffect(() => {
    if (storagePools.length > 0 && !selectedPool) {
      setSelectedPool(storagePools[0].name)
    }
  }, [storagePools, selectedPool])

  // Mount ISO mutation
  const mountMutation = useMutation({
    mutationFn: (isoPath: string) => api.mountIso(vm.id, isoPath),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('ISO mounted successfully')
      setShowIsoBrowser(false)
      setSelectedIso('')
    },
    onError: (error) => {
      toast.error(`Failed to mount ISO: ${error}`)
    },
  })

  // Eject CDROM mutation
  const ejectMutation = useMutation({
    mutationFn: () => api.ejectCdrom(vm.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('CD-ROM ejected')
    },
    onError: (error) => {
      toast.error(`Failed to eject: ${error}`)
    },
  })

  const handleMountIso = () => {
    if (selectedIso) {
      mountMutation.mutate(selectedIso)
    }
  }

  const isPending = mountMutation.isPending || ejectMutation.isPending

  // ISO Browser Dialog (shared between compact and full modes)
  const isoBrowserDialog = (
    <Dialog open={showIsoBrowser} onOpenChange={setShowIsoBrowser}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Browse ISO Images</DialogTitle>
          <DialogDescription>
            Select an ISO image from your storage pools
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>Storage Pool</Label>
            <Select value={selectedPool} onValueChange={setSelectedPool}>
              <SelectTrigger>
                <SelectValue placeholder="Select storage pool" />
              </SelectTrigger>
              <SelectContent>
                {storagePools.map((pool) => (
                  <SelectItem key={pool.name} value={pool.name}>
                    {pool.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <ScrollArea className="h-64 border rounded-md">
            {volumesLoading ? (
              <div className="flex items-center justify-center h-full">
                <Loader2 className="w-6 h-6 animate-spin" />
              </div>
            ) : isoVolumes.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-muted-foreground p-4">
                <FileArchive className="w-8 h-8 mb-2" />
                <p className="text-sm">No ISO files in this pool</p>
              </div>
            ) : (
              <div className="p-2 space-y-1">
                {isoVolumes.map((vol) => (
                  <button
                    key={vol.path}
                    onClick={() => setSelectedIso(vol.path || '')}
                    className={`w-full text-left px-3 py-2 rounded-md text-sm transition-colors ${
                      selectedIso === vol.path
                        ? 'bg-primary text-primary-foreground'
                        : 'hover:bg-muted'
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <DiscAlbum className="w-4 h-4 flex-shrink-0" />
                      <span className="truncate">{vol.name}</span>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </ScrollArea>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setShowIsoBrowser(false)}>
              Cancel
            </Button>
            <Button onClick={handleMountIso} disabled={!selectedIso || mountMutation.isPending}>
              {mountMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Mount ISO
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )

  // Compact mode shows inline controls
  if (compact) {
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Disc className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">CD-ROM</span>
            <Badge variant={cdromPath ? 'default' : 'secondary'} className="text-xs">
              {cdromPath ? 'Mounted' : 'Empty'}
            </Badge>
          </div>
          <div className="flex gap-1">
            <Button
              size="sm"
              variant="outline"
              onClick={() => setShowIsoBrowser(true)}
              disabled={isPending}
              className="h-7 text-xs"
            >
              {cdromPath ? 'Change' : 'Mount'}
            </Button>
            {cdromPath && (
              <Button
                size="sm"
                variant="ghost"
                onClick={() => ejectMutation.mutate()}
                disabled={isPending}
                className="h-7 text-xs"
              >
                Eject
              </Button>
            )}
          </div>
        </div>
        {cdromPath && (
          <div className="text-xs text-muted-foreground truncate pl-6">
            {cdromPath}
          </div>
        )}
        {isoBrowserDialog}
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold mb-2 flex items-center gap-2">
            <Disc className="w-6 h-6" />
            CD-ROM Drive
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure virtual CD/DVD drive settings
          </p>
        </div>
        <Badge variant={cdromPath ? 'default' : 'secondary'}>
          {cdromPath ? 'Media Inserted' : 'Empty'}
        </Badge>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Media Source</CardTitle>
              <CardDescription>
                ISO image or physical CD/DVD device
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="source">Source Type</Label>
                <Select value={cdromPath ? 'file' : 'empty'} disabled>
                  <SelectTrigger>
                    <SelectValue placeholder="Select source type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="empty">Empty (No media)</SelectItem>
                    <SelectItem value="file">ISO Image File</SelectItem>
                    <SelectItem value="device">Physical CD/DVD Drive</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label htmlFor="path">ISO Image Path</Label>
                <div className="flex gap-2">
                  <Input
                    id="path"
                    value={cdromPath}
                    placeholder="/path/to/image.iso"
                    readOnly
                    className="flex-1"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() => setShowIsoBrowser(true)}
                    disabled={isPending}
                    title="Browse for ISO"
                  >
                    <FolderOpen className="w-4 h-4" />
                  </Button>
                  {cdromPath && (
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => ejectMutation.mutate()}
                      disabled={isPending}
                      title="Eject media"
                    >
                      {ejectMutation.isPending ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <DiscAlbum className="w-4 h-4" />
                      )}
                    </Button>
                  )}
                </div>
                <p className="text-xs text-muted-foreground">
                  {cdromPath
                    ? 'ISO image currently inserted in the virtual drive'
                    : 'No media in drive. Browse to insert an ISO image.'}
                </p>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Drive Configuration</CardTitle>
              <CardDescription>
                Virtual CD-ROM drive settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="bus">Bus Type</Label>
                  <Select value="sata" disabled>
                    <SelectTrigger>
                      <SelectValue placeholder="Select bus type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="sata">SATA (Recommended)</SelectItem>
                      <SelectItem value="ide">IDE (Legacy)</SelectItem>
                      <SelectItem value="scsi">SCSI</SelectItem>
                      <SelectItem value="usb">USB</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label>Target Device</Label>
                  <Input
                    value="sr0"
                    disabled
                  />
                  <p className="text-xs text-muted-foreground">
                    Device name as seen by the guest OS
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
              <CardDescription>
                Common CD-ROM operations
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-1.5"
                  onClick={() => setShowIsoBrowser(true)}
                  disabled={isPending}
                >
                  <FolderOpen className="w-4 h-4" />
                  Insert ISO
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-1.5"
                  onClick={() => ejectMutation.mutate()}
                  disabled={!cdromPath || isPending}
                >
                  {ejectMutation.isPending ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <DiscAlbum className="w-4 h-4" />
                  )}
                  Eject Media
                </Button>
              </div>
              <p className="text-xs text-muted-foreground mt-3">
                <strong>Note:</strong> Hot-plugging ISO images while the VM is running
                requires guest agent support or may require a reboot.
              </p>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>CD-ROM XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for this CD-ROM device
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {`<disk type='file' device='cdrom'>
  <driver name='qemu' type='raw'/>
  ${cdromPath ? `<source file='${cdromPath}'/>` : '<!-- No media -->'}
  <target dev='sr0' bus='sata'/>
  <readonly/>
</disk>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <div className="flex justify-between">
        <Button variant="destructive" size="sm" className="gap-1.5" disabled>
          <Trash2 className="w-4 h-4" />
          Remove Drive
        </Button>
        <div className="flex gap-2">
          {cdromPath && (
            <Button
              variant="outline"
              onClick={() => ejectMutation.mutate()}
              disabled={isPending}
            >
              Eject
            </Button>
          )}
        </div>
      </div>

      {/* ISO Browser Dialog */}
      <Dialog open={showIsoBrowser} onOpenChange={setShowIsoBrowser}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Select ISO Image</DialogTitle>
            <DialogDescription>
              Choose an ISO image from a storage pool to mount in the CD-ROM drive
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label>Storage Pool</Label>
              <Select value={selectedPool} onValueChange={setSelectedPool}>
                <SelectTrigger>
                  <SelectValue placeholder="Select storage pool" />
                </SelectTrigger>
                <SelectContent>
                  {storagePools.map((pool) => (
                    <SelectItem key={pool.name} value={pool.name}>
                      {pool.name} ({pool.path || pool.poolType})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label>ISO Images</Label>
              {poolsLoading || volumesLoading ? (
                <div className="flex items-center justify-center h-48 border rounded-lg">
                  <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
                </div>
              ) : isoVolumes.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-48 border rounded-lg text-muted-foreground">
                  <FileArchive className="w-12 h-12 mb-2" />
                  <p>No ISO images found in this pool</p>
                  <p className="text-xs mt-1">Upload ISOs to your storage pools first</p>
                </div>
              ) : (
                <ScrollArea className="h-48 border rounded-lg">
                  <div className="p-2 space-y-1">
                    {isoVolumes.map((volume) => (
                      <button
                        key={volume.path || volume.name}
                        onClick={() => setSelectedIso(volume.path || '')}
                        className={`w-full flex items-center gap-3 p-2 rounded-md text-left transition-colors ${
                          selectedIso === volume.path
                            ? 'bg-primary text-primary-foreground'
                            : 'hover:bg-muted'
                        }`}
                      >
                        <Disc className="w-4 h-4 flex-shrink-0" />
                        <div className="min-w-0 flex-1">
                          <p className="font-medium truncate">{volume.name}</p>
                          <p className="text-xs opacity-70 truncate">{volume.path}</p>
                        </div>
                        {volume.capacityBytes && (
                          <span className="text-xs opacity-70 flex-shrink-0">
                            {(volume.capacityBytes / (1024 * 1024 * 1024)).toFixed(1)} GB
                          </span>
                        )}
                      </button>
                    ))}
                  </div>
                </ScrollArea>
              )}
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button variant="outline" onClick={() => setShowIsoBrowser(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleMountIso}
                disabled={!selectedIso || mountMutation.isPending}
              >
                {mountMutation.isPending ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Mounting...
                  </>
                ) : (
                  'Mount ISO'
                )}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
