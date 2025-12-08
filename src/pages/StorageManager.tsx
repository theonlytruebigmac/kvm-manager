import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import { Database, HardDrive, Plus, FolderOpen, Trash2 } from 'lucide-react'
import type { StoragePool, VolumeConfig } from '@/lib/types'

export function StorageManager() {
  const queryClient = useQueryClient()
  const [selectedPool, setSelectedPool] = useState<string | null>(null)
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [volumeName, setVolumeName] = useState('')
  const [volumeSize, setVolumeSize] = useState('10')
  const [volumeFormat, setVolumeFormat] = useState<'qcow2' | 'raw'>('qcow2')
  const [deleteConfirmVolume, setDeleteConfirmVolume] = useState<string | null>(null)

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
      toast.success(`Volume "${volumeName}" created successfully`)
      setShowCreateDialog(false)
      setVolumeName('')
      setVolumeSize('10')
      setVolumeFormat('qcow2')
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
      <Card>
        <CardHeader>
          <CardTitle>Storage Management</CardTitle>
          <CardDescription>Error loading storage pools: {String(poolsError)}</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <div className="h-full overflow-y-auto">
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Storage Management</h1>
          <p className="text-muted-foreground">Manage storage pools and volumes</p>
        </div>
      </div>

      {/* Storage Pools */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
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
              className={`cursor-pointer transition-colors ${
                selectedPool === pool.id ? 'border-primary' : ''
              }`}
              onClick={() => setSelectedPool(pool.id)}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2">
                    <Database className="h-5 w-5" />
                    {pool.name}
                  </CardTitle>
                  <Badge variant={getStateColor(pool.state)}>
                    {pool.state.toUpperCase()}
                  </Badge>
                </div>
                <CardDescription className="capitalize">{pool.poolType}</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Capacity:</span>
                    <span className="font-medium">{formatBytes(pool.capacityBytes)}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Used:</span>
                    <span className="font-medium">{formatBytes(pool.allocationBytes)}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Available:</span>
                    <span className="font-medium">{formatBytes(pool.availableBytes)}</span>
                  </div>
                  <div className="mt-2">
                    <div className="flex justify-between text-xs mb-1">
                      <span className="text-muted-foreground">Usage</span>
                      <span className="font-medium">{calculateUsagePercent(pool)}%</span>
                    </div>
                    <div className="h-2 bg-secondary rounded-full overflow-hidden">
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
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <HardDrive className="h-5 w-5" />
                  Volumes in {pools.find((p) => p.id === selectedPool)?.name}
                </CardTitle>
                <CardDescription>Disk images and storage volumes</CardDescription>
              </div>
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
                  </div>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction
                      onClick={() => createVolumeMutation.mutate({
                        name: volumeName,
                        capacityGb: parseInt(volumeSize),
                        format: volumeFormat,
                      })}
                      disabled={!volumeName.trim() || !volumeSize || createVolumeMutation.isPending}
                    >
                      {createVolumeMutation.isPending ? 'Creating...' : 'Create Volume'}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
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
                    className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors group"
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
                      <AlertDialog
                        open={deleteConfirmVolume === volume.name}
                        onOpenChange={(open) => !open && setDeleteConfirmVolume(null)}
                      >
                        <AlertDialogTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="opacity-0 group-hover:opacity-100 transition-opacity"
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
    </div>
    </div>
  )
}
