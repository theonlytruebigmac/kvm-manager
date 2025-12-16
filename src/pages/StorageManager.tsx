import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { PageContainer, PageHeader, PageContent } from '@/components/layout/PageContainer'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { CreateStoragePoolWizard } from '@/components/storage/CreateStoragePoolWizard'
import { ImportOvaDialog } from '@/components/storage/ImportOvaDialog'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Database, HardDrive, Plus, FolderOpen, Trash2, Maximize2, Upload, Download, Lock, Eye, EyeOff, FileArchive } from 'lucide-react'
import type { StoragePool, VolumeConfig } from '@/lib/types'

export function StorageManager() {
  const queryClient = useQueryClient()
  const [selectedPool, setSelectedPool] = useState<string | null>(null)
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showCreatePoolWizard, setShowCreatePoolWizard] = useState(false)
  const [showImportOvaDialog, setShowImportOvaDialog] = useState(false)
  const [volumeName, setVolumeName] = useState('')
  const [volumeSize, setVolumeSize] = useState('10')
  const [volumeFormat, setVolumeFormat] = useState<'qcow2' | 'raw'>('qcow2')
  const [volumeEncrypted, setVolumeEncrypted] = useState(false)
  const [volumePassphrase, setVolumePassphrase] = useState('')
  const [volumePassphraseConfirm, setVolumePassphraseConfirm] = useState('')
  const [showPassphrase, setShowPassphrase] = useState(false)
  const [deleteConfirmVolume, setDeleteConfirmVolume] = useState<string | null>(null)
  const [resizeVolume, setResizeVolume] = useState<{ name: string; currentGb: number } | null>(null)
  const [newVolumeSize, setNewVolumeSize] = useState('')

  // Query storage pools
  const { data: pools = [], isLoading: poolsLoading, error: poolsError } = useQuery({
    queryKey: ['storage-pools'],
    queryFn: api.getStoragePools,
    refetchInterval: 10000,
  })

  // Query volumes for selected pool
  const { data: volumes = [], isLoading: volumesLoading } = useQuery({
    queryKey: ['volumes', selectedPool],
    queryFn: () => api.getVolumes(selectedPool!),
    enabled: !!selectedPool,
    refetchInterval: 10000,
  })

  // Create volume mutation
  const createVolumeMutation = useMutation({
    mutationFn: (config: VolumeConfig) => api.createVolume(selectedPool!, config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['volumes', selectedPool] })
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      toast.success(`Volume "${volumeName}" created successfully${volumeEncrypted ? ' (encrypted)' : ''}`)
      setShowCreateDialog(false)
      setVolumeName('')
      setVolumeSize('10')
      setVolumeFormat('qcow2')
      setVolumeEncrypted(false)
      setVolumePassphrase('')
      setVolumePassphraseConfirm('')
    },
    onError: (error) => {
      toast.error(`Failed to create volume: ${error}`)
    },
  })

  // Delete volume mutation
  const deleteVolumeMutation = useMutation({
    mutationFn: (volumeName: string) => api.deleteVolume(selectedPool!, volumeName),
    onSuccess: (_data, volumeName) => {
      queryClient.invalidateQueries({ queryKey: ['volumes', selectedPool] })
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      toast.success(`Volume "${volumeName}" deleted successfully`)
      setDeleteConfirmVolume(null)
    },
    onError: (error, volumeName) => {
      toast.error(`Failed to delete volume "${volumeName}": ${error}`)
      setDeleteConfirmVolume(null)
    },
  })

  // Resize volume mutation
  const resizeVolumeMutation = useMutation({
    mutationFn: ({ volumeName, newCapacityGb }: { volumeName: string; newCapacityGb: number }) =>
      api.resizeVolume(selectedPool!, volumeName, newCapacityGb),
    onSuccess: (_data, { volumeName }) => {
      queryClient.invalidateQueries({ queryKey: ['volumes', selectedPool] })
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      toast.success(`Volume "${volumeName}" resized successfully`)
      setResizeVolume(null)
      setNewVolumeSize('')
    },
    onError: (error, { volumeName }) => {
      toast.error(`Failed to resize volume "${volumeName}": ${error}`)
    },
  })

  // Upload volume mutation
  const uploadVolumeMutation = useMutation({
    mutationFn: ({ volumeName, sourcePath, format }: { volumeName: string; sourcePath: string; format?: string }) =>
      api.uploadVolume(selectedPool!, volumeName, sourcePath, format),
    onSuccess: (_data, { volumeName }) => {
      queryClient.invalidateQueries({ queryKey: ['volumes', selectedPool] })
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      toast.success(`Volume "${volumeName}" uploaded successfully`)
    },
    onError: (error, { volumeName }) => {
      toast.error(`Failed to upload volume "${volumeName}": ${error}`)
    },
  })

  // Download volume mutation
  const downloadVolumeMutation = useMutation({
    mutationFn: ({ volumeName, destPath }: { volumeName: string; destPath: string }) =>
      api.downloadVolume(selectedPool!, volumeName, destPath),
    onSuccess: (bytes, { volumeName, destPath }) => {
      toast.success(`Volume "${volumeName}" downloaded successfully (${formatBytes(bytes)}) to ${destPath}`)
    },
    onError: (error, { volumeName }) => {
      toast.error(`Failed to download volume "${volumeName}": ${error}`)
    },
  })

  // Handle upload click
  const handleUploadClick = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        title: 'Select disk image to upload',
        filters: [
          { name: 'Disk Images', extensions: ['qcow2', 'raw', 'img', 'vmdk', 'vdi', 'vhd', 'iso'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      })
      if (selected && typeof selected === 'string') {
        // Extract filename from path
        const filename = selected.split('/').pop() || 'uploaded-disk'
        uploadVolumeMutation.mutate({ volumeName: filename, sourcePath: selected })
      }
    } catch (error) {
      toast.error(`Failed to open file picker: ${error}`)
    }
  }

  // Handle download click
  const handleDownloadClick = async (volumeName: string) => {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      const destPath = await save({
        title: 'Save volume as',
        defaultPath: volumeName,
        filters: [
          { name: 'Disk Images', extensions: ['qcow2', 'raw', 'img'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      })
      if (destPath) {
        downloadVolumeMutation.mutate({ volumeName, destPath })
      }
    } catch (error) {
      toast.error(`Failed to open save dialog: ${error}`)
    }
  }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const getStateColor = (state: string) => {
    switch (state) {
      case 'active':
        return 'default'
      case 'inactive':
        return 'secondary'
      case 'degraded':
        return 'destructive'
      default:
        return 'outline'
    }
  }

  const calculateUsagePercent = (pool: StoragePool) => {
    if (pool.capacityBytes === 0) return 0
    return ((pool.allocationBytes / pool.capacityBytes) * 100).toFixed(1)
  }

  if (poolsError) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Card className="max-w-md">
          <CardHeader>
            <CardTitle>Storage Management</CardTitle>
            <CardDescription>Error loading storage pools: {String(poolsError)}</CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  return (
    <PageContainer>
      <PageHeader
        title="Storage Management"
        description="Manage storage pools and volumes"
        actions={
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setShowImportOvaDialog(true)}>
              <FileArchive className="w-4 h-4 mr-2" />
              Import OVA/OVF
            </Button>
            <Button onClick={() => setShowCreatePoolWizard(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Create Pool
            </Button>
          </div>
        }
      />
      <PageContent>
        <div className="space-y-8">
          {/* Storage Pools */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {poolsLoading ? (
          <Card>
            <CardContent className="p-6">
              <p className="text-sm text-muted-foreground">Loading storage pools...</p>
            </CardContent>
          </Card>
        ) : pools.length === 0 ? (
          <Card>
            <CardContent className="p-6">
              <p className="text-sm text-muted-foreground">No storage pools found</p>
            </CardContent>
          </Card>
        ) : (
          pools.map((pool) => (
            <Card
              key={pool.id}
              className={`cursor-pointer transition-all border-border/40 shadow-sm hover:shadow-md ${
                selectedPool === pool.id ? 'border-primary ring-2 ring-primary/20' : ''
              }`}
              onClick={() => setSelectedPool(pool.id)}
              onDoubleClick={() => {
                // Double-click to select pool and scroll to volumes
                setSelectedPool(pool.id)
                setTimeout(() => {
                  const volumesSection = document.querySelector('[data-volumes-section]')
                  volumesSection?.scrollIntoView({ behavior: 'smooth' })
                }, 100)
              }}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2 text-lg font-semibold">
                    <Database className="h-5 w-5 text-muted-foreground" />
                    {pool.name}
                  </CardTitle>
                  <Badge variant={getStateColor(pool.state)} className="border-border/40">
                    {pool.state.toUpperCase()}
                  </Badge>
                </div>
                <CardDescription className="capitalize">{pool.poolType}</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Capacity:</span>
                    <span className="text-sm">{formatBytes(pool.capacityBytes)}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Used:</span>
                    <span className="text-sm">{formatBytes(pool.allocationBytes)}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Available:</span>
                    <span className="text-sm">{formatBytes(pool.availableBytes)}</span>
                  </div>
                  <div className="mt-4 pt-3 border-t border-border/40">
                    <div className="flex justify-between text-xs mb-2">
                      <span className="text-muted-foreground">Usage</span>
                      <span className="text-sm">{calculateUsagePercent(pool)}%</span>
                    </div>
                    <div className="h-1.5 bg-secondary rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${calculateUsagePercent(pool)}%` }}
                      />
                    </div>
                  </div>
                  <div className="pt-2 text-xs text-muted-foreground truncate">
                    <FolderOpen className="inline h-3 w-3 mr-1" />
                    {pool.path}
                  </div>
                </div>
              </CardContent>
            </Card>
          ))
        )}
      </div>

      {/* Volumes Section */}
      {selectedPool && (
        <Card className="border-border/40 shadow-sm" data-volumes-section>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2 text-lg font-semibold">
                  <HardDrive className="h-5 w-5 text-muted-foreground" />
                  Volumes in {pools.find((p) => p.id === selectedPool)?.name}
                </CardTitle>
                <CardDescription>Disk images and storage volumes</CardDescription>
              </div>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleUploadClick}
                  disabled={uploadVolumeMutation.isPending}
                >
                  <Upload className="mr-2 h-4 w-4" />
                  {uploadVolumeMutation.isPending ? 'Uploading...' : 'Upload'}
                </Button>
                <AlertDialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                  <AlertDialogTrigger asChild>
                    <Button size="sm">
                      <Plus className="mr-2 h-4 w-4" />
                      Create Volume
                    </Button>
                  </AlertDialogTrigger>
                <AlertDialogContent className="max-w-md">
                  <AlertDialogHeader>
                    <AlertDialogTitle>Create Storage Volume</AlertDialogTitle>
                    <AlertDialogDescription>
                      Create a new disk image in this storage pool.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <div className="space-y-4 py-4">
                    <div className="space-y-2">
                      <Label htmlFor="volume-name">Volume Name</Label>
                      <Input
                        id="volume-name"
                        value={volumeName}
                        onChange={(e) => setVolumeName(e.target.value)}
                        placeholder="e.g., disk-01.qcow2"
                        autoFocus
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="volume-size">Size (GB)</Label>
                      <Input
                        id="volume-size"
                        type="number"
                        min="1"
                        max="1000"
                        value={volumeSize}
                        onChange={(e) => setVolumeSize(e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="volume-format">Format</Label>
                      <Select value={volumeFormat} onValueChange={(v) => setVolumeFormat(v as 'qcow2' | 'raw')}>
                        <SelectTrigger id="volume-format">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="qcow2">QCOW2 (Recommended)</SelectItem>
                          <SelectItem value="raw">RAW</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-xs text-muted-foreground">
                        QCOW2 supports compression and snapshots. RAW provides better performance.
                      </p>
                    </div>
                    {/* Encryption Section */}
                    <div className="border-t pt-4 mt-2">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <Lock className="h-4 w-4 text-muted-foreground" />
                          <Label htmlFor="volume-encrypted">LUKS Encryption</Label>
                        </div>
                        <input
                          type="checkbox"
                          id="volume-encrypted"
                          checked={volumeEncrypted}
                          onChange={(e) => setVolumeEncrypted(e.target.checked)}
                          className="h-4 w-4 rounded border-gray-300"
                        />
                      </div>
                      {volumeEncrypted && (
                        <div className="space-y-3 mt-3">
                          <div className="space-y-2">
                            <Label htmlFor="passphrase">Passphrase</Label>
                            <div className="relative">
                              <Input
                                id="passphrase"
                                type={showPassphrase ? 'text' : 'password'}
                                value={volumePassphrase}
                                onChange={(e) => setVolumePassphrase(e.target.value)}
                                placeholder="Enter encryption passphrase"
                                className="pr-10"
                              />
                              <button
                                type="button"
                                onClick={() => setShowPassphrase(!showPassphrase)}
                                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                              >
                                {showPassphrase ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                              </button>
                            </div>
                          </div>
                          <div className="space-y-2">
                            <Label htmlFor="passphrase-confirm">Confirm Passphrase</Label>
                            <Input
                              id="passphrase-confirm"
                              type={showPassphrase ? 'text' : 'password'}
                              value={volumePassphraseConfirm}
                              onChange={(e) => setVolumePassphraseConfirm(e.target.value)}
                              placeholder="Confirm passphrase"
                            />
                          </div>
                          {volumePassphrase && volumePassphrase.length < 8 && (
                            <p className="text-xs text-yellow-600">Passphrase must be at least 8 characters</p>
                          )}
                          {volumePassphrase && volumePassphraseConfirm && volumePassphrase !== volumePassphraseConfirm && (
                            <p className="text-xs text-red-600">Passphrases do not match</p>
                          )}
                          <div className="p-2 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                            <p className="text-xs text-yellow-700">
                              <strong>Important:</strong> Store your passphrase securely. If lost, data cannot be recovered.
                            </p>
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction
                      onClick={() => createVolumeMutation.mutate({
                        name: volumeName,
                        capacityGb: parseInt(volumeSize),
                        format: volumeFormat,
                        encrypted: volumeEncrypted,
                        passphrase: volumeEncrypted ? volumePassphrase : undefined,
                      })}
                      disabled={
                        !volumeName.trim() ||
                        !volumeSize ||
                        createVolumeMutation.isPending ||
                        (volumeEncrypted && (volumePassphrase.length < 8 || volumePassphrase !== volumePassphraseConfirm))
                      }
                    >
                      {createVolumeMutation.isPending ? 'Creating...' : 'Create Volume'}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            {volumesLoading ? (
              <p className="text-sm text-muted-foreground">Loading volumes...</p>
            ) : volumes.length === 0 ? (
              <div className="text-center py-8">
                <HardDrive className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                <p className="text-sm text-muted-foreground">No volumes in this pool</p>
                <Button
                  size="sm"
                  variant="outline"
                  className="mt-4"
                  onClick={() => setShowCreateDialog(true)}
                >
                  <Plus className="mr-2 h-4 w-4" />
                  Create First Volume
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                {volumes.map((volume) => (
                  <div
                    key={volume.path}
                    className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors group cursor-pointer"
                    onDoubleClick={() => {
                      // Open resize dialog on double-click
                      setResizeVolume({
                        name: volume.name,
                        currentGb: Math.ceil(volume.capacityBytes / (1024 ** 3))
                      })
                      setNewVolumeSize((Math.ceil(volume.capacityBytes / (1024 ** 3)) + 10).toString())
                    }}
                  >
                    <div className="flex-1">
                      <p className="font-medium">{volume.name}</p>
                      <p className="text-xs text-muted-foreground truncate max-w-md">
                        {volume.path}
                      </p>
                    </div>
                    <div className="flex items-center gap-4">
                      <div className="text-right">
                        <p className="text-sm font-medium">{formatBytes(volume.capacityBytes)}</p>
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <span>Used: {formatBytes(volume.allocationBytes)}</span>
                          <Badge variant="outline" className="text-xs">
                            {volume.format.toUpperCase()}
                          </Badge>
                        </div>
                      </div>
                      <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Button
                          variant="ghost"
                          size="sm"
                          title="Download volume"
                          onClick={() => handleDownloadClick(volume.name)}
                          disabled={downloadVolumeMutation.isPending}
                        >
                          <Download className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          title="Resize volume"
                          onClick={() => {
                            setResizeVolume({
                              name: volume.name,
                              currentGb: Math.ceil(volume.capacityBytes / (1024 ** 3))
                            })
                            setNewVolumeSize((Math.ceil(volume.capacityBytes / (1024 ** 3)) + 10).toString())
                          }}
                        >
                          <Maximize2 className="h-4 w-4" />
                        </Button>
                        <AlertDialog
                          open={deleteConfirmVolume === volume.name}
                          onOpenChange={(open) => !open && setDeleteConfirmVolume(null)}
                        >
                          <AlertDialogTrigger asChild>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => setDeleteConfirmVolume(volume.name)}
                            >
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          </AlertDialogTrigger>
                        <AlertDialogContent>
                          <AlertDialogHeader>
                            <AlertDialogTitle>Delete Volume?</AlertDialogTitle>
                            <AlertDialogDescription>
                              Are you sure you want to delete volume "{volume.name}"? This action cannot be undone.
                              Any VMs using this volume will lose access to it.
                            </AlertDialogDescription>
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel>Cancel</AlertDialogCancel>
                            <AlertDialogAction
                              onClick={() => deleteVolumeMutation.mutate(volume.name)}
                              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                            >
                              Delete Volume
                            </AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {!selectedPool && pools.length > 0 && (
        <Card>
          <CardContent className="p-12 text-center">
            <Database className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <p className="text-muted-foreground">Select a storage pool to view its volumes</p>
          </CardContent>
        </Card>
      )}

      {/* Create Storage Pool Wizard */}
      {showCreatePoolWizard && (
        <CreateStoragePoolWizard onClose={() => setShowCreatePoolWizard(false)} />
      )}

      {/* Import OVA/OVF Dialog */}
      {showImportOvaDialog && (
        <ImportOvaDialog onClose={() => setShowImportOvaDialog(false)} />
      )}

      {/* Resize Volume Dialog */}
      <AlertDialog
        open={!!resizeVolume}
        onOpenChange={(open) => !open && setResizeVolume(null)}
      >
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Resize Volume</AlertDialogTitle>
            <AlertDialogDescription>
              Increase the capacity of volume "{resizeVolume?.name}". The new size must be larger than the current size.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>Current Size</Label>
              <Input
                value={`${resizeVolume?.currentGb || 0} GB`}
                disabled
                className="bg-muted"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new-volume-size">New Size (GB)</Label>
              <Input
                id="new-volume-size"
                type="number"
                min={(resizeVolume?.currentGb || 0) + 1}
                max="10000"
                value={newVolumeSize}
                onChange={(e) => setNewVolumeSize(e.target.value)}
                placeholder={`Minimum: ${(resizeVolume?.currentGb || 0) + 1} GB`}
              />
              <p className="text-xs text-muted-foreground">
                Note: Volumes can only be expanded, not shrunk. Make sure the VM's filesystem is also expanded after resizing.
              </p>
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (resizeVolume && newVolumeSize) {
                  const newSize = parseInt(newVolumeSize)
                  if (newSize > resizeVolume.currentGb) {
                    resizeVolumeMutation.mutate({
                      volumeName: resizeVolume.name,
                      newCapacityGb: newSize,
                    })
                  } else {
                    toast.error('New size must be larger than current size')
                  }
                }
              }}
              disabled={
                !newVolumeSize ||
                parseInt(newVolumeSize) <= (resizeVolume?.currentGb || 0) ||
                resizeVolumeMutation.isPending
              }
            >
              {resizeVolumeMutation.isPending ? 'Resizing...' : 'Resize Volume'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
        </div>
      </PageContent>
    </PageContainer>
  )
}
