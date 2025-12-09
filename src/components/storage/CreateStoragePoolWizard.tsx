import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { ChevronLeft, ChevronRight, Check, Info } from 'lucide-react'

interface StoragePoolFormData {
  name: string
  poolType: 'dir' | 'logical' | 'netfs'
  targetPath: string
  autostart: boolean
  // For logical pools
  sourceDevices: string[]
  deviceInput: string
  // For netfs pools
  sourceHost: string
  sourcePath: string
}

const steps = [
  { id: 1, name: 'Pool Type', description: 'Select storage pool type' },
  { id: 2, name: 'Configuration', description: 'Configure pool settings' },
  { id: 3, name: 'Review', description: 'Confirm settings' },
]

interface CreateStoragePoolWizardProps {
  onClose: () => void
}

export function CreateStoragePoolWizard({ onClose }: CreateStoragePoolWizardProps) {
  const queryClient = useQueryClient()
  const [currentStep, setCurrentStep] = useState(1)
  const [formData, setFormData] = useState<StoragePoolFormData>({
    name: '',
    poolType: 'dir',
    targetPath: '/var/lib/libvirt/images',
    autostart: true,
    sourceDevices: [],
    deviceInput: '',
    sourceHost: '',
    sourcePath: '',
  })

  const createMutation = useMutation({
    mutationFn: () => {
      const config: any = {
        name: formData.name,
        poolType: formData.poolType,
        targetPath: formData.targetPath,
        autostart: formData.autostart,
      }

      if (formData.poolType === 'logical') {
        config.sourceDevices = formData.sourceDevices
      } else if (formData.poolType === 'netfs') {
        config.sourceHost = formData.sourceHost
        config.sourcePath = formData.sourcePath
      }

      return api.createStoragePool(config)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      toast.success(`Storage pool "${formData.name}" created successfully`)
      onClose()
    },
    onError: (error) => {
      toast.error(`Failed to create storage pool: ${error}`)
    },
  })

  const handleNext = () => {
    if (currentStep === 1) {
      if (!formData.name.trim()) {
        toast.error('Please enter a pool name')
        return
      }
    }
    if (currentStep === 2) {
      if (!formData.targetPath.trim()) {
        toast.error('Please enter a target path')
        return
      }
      if (formData.poolType === 'logical' && formData.sourceDevices.length === 0) {
        toast.error('Please add at least one source device for logical pool')
        return
      }
      if (formData.poolType === 'netfs' && (!formData.sourceHost || !formData.sourcePath)) {
        toast.error('Please enter source host and path for network filesystem')
        return
      }
    }
    setCurrentStep(currentStep + 1)
  }

  const handleBack = () => {
    setCurrentStep(currentStep - 1)
  }

  const handleSubmit = () => {
    createMutation.mutate()
  }

  const addDevice = () => {
    if (formData.deviceInput.trim()) {
      setFormData({
        ...formData,
        sourceDevices: [...formData.sourceDevices, formData.deviceInput.trim()],
        deviceInput: '',
      })
    }
  }

  const removeDevice = (index: number) => {
    setFormData({
      ...formData,
      sourceDevices: formData.sourceDevices.filter((_, i) => i !== index),
    })
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-2xl max-h-[90vh] overflow-auto">
        <CardHeader>
          <CardTitle>Create Storage Pool</CardTitle>
          <CardDescription>
            Step {currentStep} of {steps.length}: {steps[currentStep - 1].description}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Step 1: Pool Type */}
          {currentStep === 1 && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Pool Name *</Label>
                <Input
                  id="name"
                  placeholder="my-storage-pool"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="poolType">Pool Type *</Label>
                <Select value={formData.poolType} onValueChange={(value: any) => setFormData({ ...formData, poolType: value })}>
                  <SelectTrigger id="poolType">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="dir">Directory (Filesystem)</SelectItem>
                    <SelectItem value="logical">LVM Logical Volume</SelectItem>
                    <SelectItem value="netfs">Network Filesystem (NFS)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="bg-blue-50 dark:bg-blue-950 p-4 rounded-lg space-y-2">
                <div className="flex items-start gap-2">
                  <Info className="w-4 h-4 mt-0.5 text-blue-600 dark:text-blue-400" />
                  <div className="text-sm text-blue-900 dark:text-blue-100">
                    <p className="font-medium mb-1">Pool Types:</p>
                    <ul className="list-disc list-inside space-y-1 text-xs">
                      <li><strong>Directory:</strong> Simple filesystem directory storage</li>
                      <li><strong>LVM:</strong> Linux Logical Volume Manager for advanced features</li>
                      <li><strong>NFS:</strong> Network-attached storage over NFS protocol</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Step 2: Configuration */}
          {currentStep === 2 && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="targetPath">Target Path *</Label>
                <Input
                  id="targetPath"
                  placeholder="/var/lib/libvirt/images"
                  value={formData.targetPath}
                  onChange={(e) => setFormData({ ...formData, targetPath: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  {formData.poolType === 'dir' && 'Directory where VM disk images will be stored'}
                  {formData.poolType === 'logical' && 'Mount point for the LVM volume group'}
                  {formData.poolType === 'netfs' && 'Local mount point for the network filesystem'}
                </p>
              </div>

              {/* Logical Pool - Source Devices */}
              {formData.poolType === 'logical' && (
                <div className="space-y-2">
                  <Label>Source Devices *</Label>
                  <div className="flex gap-2">
                    <Input
                      placeholder="/dev/sdb"
                      value={formData.deviceInput}
                      onChange={(e) => setFormData({ ...formData, deviceInput: e.target.value })}
                      onKeyPress={(e) => e.key === 'Enter' && addDevice()}
                    />
                    <Button type="button" onClick={addDevice}>Add</Button>
                  </div>
                  {formData.sourceDevices.length > 0 && (
                    <div className="bg-muted p-3 rounded-lg space-y-1">
                      {formData.sourceDevices.map((device, index) => (
                        <div key={index} className="flex items-center justify-between text-sm">
                          <span className="font-mono">{device}</span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => removeDevice(index)}
                          >
                            Remove
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                  <p className="text-xs text-muted-foreground">
                    Physical devices or partitions for the volume group (e.g., /dev/sdb, /dev/sdc1)
                  </p>
                </div>
              )}

              {/* Network Filesystem - Source Configuration */}
              {formData.poolType === 'netfs' && (
                <>
                  <div className="space-y-2">
                    <Label htmlFor="sourceHost">NFS Server Host *</Label>
                    <Input
                      id="sourceHost"
                      placeholder="nfs.example.com or 192.168.1.100"
                      value={formData.sourceHost}
                      onChange={(e) => setFormData({ ...formData, sourceHost: e.target.value })}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="sourcePath">NFS Export Path *</Label>
                    <Input
                      id="sourcePath"
                      placeholder="/export/vms"
                      value={formData.sourcePath}
                      onChange={(e) => setFormData({ ...formData, sourcePath: e.target.value })}
                    />
                  </div>
                </>
              )}

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="autostart"
                  checked={formData.autostart}
                  onCheckedChange={(checked) => setFormData({ ...formData, autostart: checked === true })}
                />
                <label
                  htmlFor="autostart"
                  className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                  Start automatically on host boot
                </label>
              </div>
            </div>
          )}

          {/* Step 3: Review */}
          {currentStep === 3 && (
            <div className="space-y-4">
              <div className="bg-muted p-4 rounded-lg space-y-3">
                <h4 className="font-medium">Storage Pool Configuration Summary</h4>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Name:</span>
                    <span className="font-medium">{formData.name}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Type:</span>
                    <span className="font-medium capitalize">{formData.poolType === 'netfs' ? 'NFS' : formData.poolType}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Target Path:</span>
                    <span className="font-medium font-mono text-xs">{formData.targetPath}</span>
                  </div>
                  {formData.poolType === 'logical' && formData.sourceDevices.length > 0 && (
                    <div>
                      <span className="text-muted-foreground">Source Devices:</span>
                      <ul className="mt-1 space-y-1">
                        {formData.sourceDevices.map((device, index) => (
                          <li key={index} className="font-mono text-xs ml-4">• {device}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                  {formData.poolType === 'netfs' && (
                    <>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">NFS Host:</span>
                        <span className="font-medium">{formData.sourceHost}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">NFS Path:</span>
                        <span className="font-medium font-mono text-xs">{formData.sourcePath}</span>
                      </div>
                    </>
                  )}
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Autostart:</span>
                    <span className="font-medium">{formData.autostart ? 'Enabled' : 'Disabled'}</span>
                  </div>
                </div>
              </div>

              <div className="bg-yellow-50 dark:bg-yellow-950 p-4 rounded-lg text-sm">
                <p className="text-yellow-900 dark:text-yellow-100">
                  <strong>Warning:</strong> Ensure the target path exists and has proper permissions.
                  {formData.poolType === 'logical' && ' Make sure the specified devices are available and not in use.'}
                  {formData.poolType === 'netfs' && ' Verify that the NFS server is accessible and the export is configured.'}
                </p>
              </div>
            </div>
          )}

          {/* Navigation Buttons */}
          <div className="flex justify-between pt-4 border-t">
            <div>
              {currentStep > 1 && (
                <Button variant="outline" onClick={handleBack} disabled={createMutation.isPending}>
                  <ChevronLeft className="w-4 h-4 mr-1" />
                  Back
                </Button>
              )}
            </div>
            <div className="flex gap-2">
              <Button variant="outline" onClick={onClose} disabled={createMutation.isPending}>
                Cancel
              </Button>
              {currentStep < steps.length ? (
                <Button onClick={handleNext}>
                  Next
                  <ChevronRight className="w-4 h-4 ml-1" />
                </Button>
              ) : (
                <Button onClick={handleSubmit} disabled={createMutation.isPending}>
                  {createMutation.isPending ? (
                    'Creating...'
                  ) : (
                    <>
                      <Check className="w-4 h-4 mr-1" />
                      Create Pool
                    </>
                  )}
                </Button>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
