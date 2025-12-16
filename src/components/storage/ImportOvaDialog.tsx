import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { Progress } from '@/components/ui/progress'
import { FileArchive, HardDrive, Cpu, MemoryStick, Network, Check, Loader2, FolderOpen } from 'lucide-react'
import type { OvfMetadata } from '@/lib/types'

interface ImportOvaDialogProps {
  onClose: () => void
  onSuccess?: (vmName: string) => void
}

export function ImportOvaDialog({ onClose, onSuccess }: ImportOvaDialogProps) {
  const queryClient = useQueryClient()
  const [step, setStep] = useState<'select' | 'preview' | 'importing'>('select')
  const [sourcePath, setSourcePath] = useState('')
  const [metadata, setMetadata] = useState<OvfMetadata | null>(null)
  const [selectedPool, setSelectedPool] = useState('')
  const [vmName, setVmName] = useState('')
  const [convertToQcow2, setConvertToQcow2] = useState(true)
  const [importProgress, setImportProgress] = useState(0)

  // Query storage pools
  const { data: pools = [] } = useQuery({
    queryKey: ['storage-pools'],
    queryFn: api.getStoragePools,
  })

  // Select file mutation
  const selectFileMutation = useMutation({
    mutationFn: async () => {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        title: 'Select OVA or OVF file',
        filters: [
          { name: 'Virtual Appliance', extensions: ['ova', 'ovf'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      })
      if (!selected || typeof selected !== 'string') {
        throw new Error('No file selected')
      }
      return selected
    },
    onSuccess: (path) => {
      setSourcePath(path)
    },
  })

  // Get metadata mutation
  const getMetadataMutation = useMutation({
    mutationFn: (path: string) => api.getOvaMetadata(path),
    onSuccess: (data) => {
      setMetadata(data)
      setVmName(data.name)
      // Auto-select first pool
      if (pools.length > 0 && !selectedPool) {
        setSelectedPool(pools[0].path)
      }
      setStep('preview')
    },
    onError: (error) => {
      toast.error(`Failed to read OVA/OVF: ${error}`)
    },
  })

  // Import mutation
  const importMutation = useMutation({
    mutationFn: async () => {
      setStep('importing')
      setImportProgress(10)

      const result = await api.importOva({
        sourcePath,
        targetPoolPath: selectedPool,
        vmName: vmName !== metadata?.name ? vmName : undefined,
        convertToQcow2,
      })

      setImportProgress(80)
      return result
    },
    onSuccess: (diskPath) => {
      setImportProgress(100)
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`Successfully imported "${vmName}". Disk saved to: ${diskPath}`)

      // Note: The import just extracts and converts the disk.
      // User still needs to create a VM using the imported disk.
      if (onSuccess) {
        onSuccess(vmName)
      }

      setTimeout(() => onClose(), 1500)
    },
    onError: (error) => {
      setStep('preview')
      toast.error(`Import failed: ${error}`)
    },
  })

  const handleSelectFile = async () => {
    await selectFileMutation.mutateAsync()
  }

  const handleLoadMetadata = () => {
    if (sourcePath) {
      getMetadataMutation.mutate(sourcePath)
    }
  }

  const handleImport = () => {
    if (!selectedPool) {
      toast.error('Please select a target storage pool')
      return
    }
    importMutation.mutate()
  }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-2xl max-h-[90vh] overflow-auto">
        <CardHeader>
          <div className="flex items-center gap-2">
            <FileArchive className="h-6 w-6" />
            <div>
              <CardTitle>Import OVA/OVF</CardTitle>
              <CardDescription>
                Import a virtual machine from VMware or VirtualBox format
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Step 1: Select File */}
          {step === 'select' && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Source File</Label>
                <div className="flex gap-2">
                  <Input
                    value={sourcePath}
                    onChange={(e) => setSourcePath(e.target.value)}
                    placeholder="/path/to/vm.ova or vm.ovf"
                    className="flex-1"
                  />
                  <Button
                    variant="outline"
                    onClick={handleSelectFile}
                    disabled={selectFileMutation.isPending}
                  >
                    <FolderOpen className="h-4 w-4 mr-2" />
                    Browse
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                  Select an OVA (single file) or OVF (with accompanying disk files) to import
                </p>
              </div>

              <div className="bg-blue-50 dark:bg-blue-950 p-4 rounded-lg text-sm">
                <p className="text-blue-900 dark:text-blue-100">
                  <strong>Supported formats:</strong> OVA/OVF files exported from VMware Workstation/ESXi,
                  VirtualBox, or other OVF-compliant virtualization platforms. Disk images (VMDK, VHD)
                  will be automatically converted to qcow2 format.
                </p>
              </div>

              <div className="flex justify-between pt-4 border-t">
                <Button variant="outline" onClick={onClose}>
                  Cancel
                </Button>
                <Button
                  onClick={handleLoadMetadata}
                  disabled={!sourcePath || getMetadataMutation.isPending}
                >
                  {getMetadataMutation.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                      Loading...
                    </>
                  ) : (
                    'Next'
                  )}
                </Button>
              </div>
            </div>
          )}

          {/* Step 2: Preview and Configure */}
          {step === 'preview' && metadata && (
            <div className="space-y-4">
              <div className="bg-muted p-4 rounded-lg space-y-3">
                <h4 className="font-medium">Detected Configuration</h4>

                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div className="flex items-center gap-2">
                    <Cpu className="h-4 w-4 text-muted-foreground" />
                    <span className="text-muted-foreground">CPUs:</span>
                    <span className="font-medium">{metadata.cpuCount}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <MemoryStick className="h-4 w-4 text-muted-foreground" />
                    <span className="text-muted-foreground">Memory:</span>
                    <span className="font-medium">{metadata.memoryMb} MB</span>
                  </div>
                </div>

                {metadata.osType && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">OS Type:</span>{' '}
                    <span className="font-medium">{metadata.osType}</span>
                  </div>
                )}

                {metadata.description && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">Description:</span>{' '}
                    <span>{metadata.description}</span>
                  </div>
                )}

                {metadata.disks.length > 0 && (
                  <div className="space-y-1">
                    <span className="text-sm text-muted-foreground">Disks:</span>
                    {metadata.disks.map((disk, i) => (
                      <div key={i} className="flex items-center gap-2 text-sm ml-4">
                        <HardDrive className="h-4 w-4 text-muted-foreground" />
                        <span className="font-mono text-xs">{disk.fileName}</span>
                        <span className="text-muted-foreground">
                          ({disk.format}, {formatBytes(disk.capacityBytes)})
                        </span>
                      </div>
                    ))}
                  </div>
                )}

                {metadata.networks.length > 0 && (
                  <div className="space-y-1">
                    <span className="text-sm text-muted-foreground">Networks:</span>
                    {metadata.networks.map((net, i) => (
                      <div key={i} className="flex items-center gap-2 text-sm ml-4">
                        <Network className="h-4 w-4 text-muted-foreground" />
                        <span>{net.name}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="vmName">VM Name</Label>
                  <Input
                    id="vmName"
                    value={vmName}
                    onChange={(e) => setVmName(e.target.value)}
                    placeholder="Name for the imported VM"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="targetPool">Target Storage Pool *</Label>
                  <Select value={selectedPool} onValueChange={setSelectedPool}>
                    <SelectTrigger id="targetPool">
                      <SelectValue placeholder="Select storage pool" />
                    </SelectTrigger>
                    <SelectContent>
                      {pools.map((pool) => (
                        <SelectItem key={pool.id} value={pool.path}>
                          {pool.name} ({pool.path})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    Disk images will be extracted/converted to this location
                  </p>
                </div>

                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="convert"
                    checked={convertToQcow2}
                    onCheckedChange={(checked) => setConvertToQcow2(checked === true)}
                  />
                  <label
                    htmlFor="convert"
                    className="text-sm font-medium leading-none"
                  >
                    Convert disks to qcow2 format (recommended)
                  </label>
                </div>
              </div>

              <div className="bg-yellow-50 dark:bg-yellow-950 p-4 rounded-lg text-sm">
                <p className="text-yellow-900 dark:text-yellow-100">
                  <strong>Note:</strong> This will extract and convert disk images.
                  After import, you'll need to create a new VM using the converted disk.
                </p>
              </div>

              <div className="flex justify-between pt-4 border-t">
                <Button variant="outline" onClick={() => setStep('select')}>
                  Back
                </Button>
                <Button onClick={handleImport} disabled={!selectedPool}>
                  Import
                </Button>
              </div>
            </div>
          )}

          {/* Step 3: Importing */}
          {step === 'importing' && (
            <div className="space-y-6 py-8">
              <div className="text-center space-y-2">
                <Loader2 className="h-12 w-12 animate-spin mx-auto text-primary" />
                <h4 className="font-medium text-lg">Importing Virtual Machine</h4>
                <p className="text-sm text-muted-foreground">
                  Extracting and converting disk images...
                </p>
              </div>

              <Progress value={importProgress} className="w-full" />

              <p className="text-xs text-center text-muted-foreground">
                This may take several minutes for large disk images
              </p>
            </div>
          )}

          {/* Success state */}
          {importProgress === 100 && (
            <div className="text-center space-y-4 py-8">
              <div className="h-12 w-12 rounded-full bg-green-100 dark:bg-green-900 flex items-center justify-center mx-auto">
                <Check className="h-6 w-6 text-green-600 dark:text-green-400" />
              </div>
              <h4 className="font-medium text-lg">Import Complete!</h4>
              <p className="text-sm text-muted-foreground">
                Disk images have been imported. You can now create a VM using these disks.
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
