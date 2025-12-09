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
import { ChevronLeft, ChevronRight, Check } from 'lucide-react'

interface VmFormData {
  name: string
  cpuCount: number
  memoryMb: number
  diskSizeGb: number
  osType: 'linux' | 'windows' | 'other'
  network: string
  diskFormat: 'qcow2' | 'raw'
  bootMenu: boolean
  isoPath?: string
}

const steps = [
  { id: 1, name: 'Basic Info', description: 'VM name and resources' },
  { id: 2, name: 'Configuration', description: 'OS, network, and disk settings' },
  { id: 3, name: 'Review', description: 'Confirm settings' },
]

interface CreateVmWizardProps {
  onClose: () => void
}

export function CreateVmWizard({ onClose }: CreateVmWizardProps) {
  const queryClient = useQueryClient()
  const [currentStep, setCurrentStep] = useState(1)
  const [formData, setFormData] = useState<VmFormData>({
    name: '',
    cpuCount: 2,
    memoryMb: 2048,
    diskSizeGb: 20,
    osType: 'linux',
    network: 'default',
    diskFormat: 'qcow2',
    bootMenu: false,
    isoPath: undefined,
  })

  const createMutation = useMutation({
    mutationFn: () => api.createVm(formData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM "${formData.name}" created successfully`)
      onClose()
    },
    onError: (error) => {
      toast.error(`Failed to create VM: ${error}`)
    },
  })

  const handleNext = () => {
    if (currentStep === 1) {
      // Validate step 1
      if (!formData.name.trim()) {
        toast.error('Please enter a VM name')
        return
      }
      if (formData.cpuCount < 1 || formData.cpuCount > 16) {
        toast.error('CPU count must be between 1 and 16')
        return
      }
      if (formData.memoryMb < 512 || formData.memoryMb > 32768) {
        toast.error('Memory must be between 512 MB and 32 GB')
        return
      }
      if (formData.diskSizeGb < 1 || formData.diskSizeGb > 500) {
        toast.error('Disk size must be between 1 GB and 500 GB')
        return
      }
    }
    if (currentStep === 2) {
      // Validate step 2
      if (!formData.network.trim()) {
        toast.error('Please select a network')
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

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-2xl max-h-[90vh] overflow-auto">
        <CardHeader>
          <CardTitle>Create New Virtual Machine</CardTitle>
          <CardDescription>
            Step {currentStep} of {steps.length}: {steps[currentStep - 1].description}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Step 1: Basic Info */}
          {currentStep === 1 && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">VM Name *</Label>
                <Input
                  id="name"
                  placeholder="my-vm"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                />
              </div>

              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="cpu">CPU Cores *</Label>
                  <Input
                    id="cpu"
                    type="number"
                    min="1"
                    max="16"
                    value={formData.cpuCount}
                    onChange={(e) => setFormData({ ...formData, cpuCount: parseInt(e.target.value) || 1 })}
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="memory">Memory (MB) *</Label>
                  <Input
                    id="memory"
                    type="number"
                    min="512"
                    max="32768"
                    step="512"
                    value={formData.memoryMb}
                    onChange={(e) => setFormData({ ...formData, memoryMb: parseInt(e.target.value) || 512 })}
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="disk">Disk Size (GB) *</Label>
                  <Input
                    id="disk"
                    type="number"
                    min="1"
                    max="500"
                    value={formData.diskSizeGb}
                    onChange={(e) => setFormData({ ...formData, diskSizeGb: parseInt(e.target.value) || 1 })}
                  />
                </div>
              </div>

              <div className="bg-muted p-4 rounded-lg">
                <h4 className="text-sm font-medium mb-2">Quick Presets</h4>
                <div className="grid grid-cols-3 gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({ ...formData, cpuCount: 1, memoryMb: 512, diskSizeGb: 10 })}
                  >
                    Minimal
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({ ...formData, cpuCount: 2, memoryMb: 2048, diskSizeGb: 20 })}
                  >
                    Standard
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({ ...formData, cpuCount: 4, memoryMb: 4096, diskSizeGb: 50 })}
                  >
                    Performance
                  </Button>
                </div>
              </div>
            </div>
          )}

          {/* Step 2: Configuration */}
          {currentStep === 2 && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="osType">Operating System Type *</Label>
                  <Select value={formData.osType} onValueChange={(value: any) => setFormData({ ...formData, osType: value })}>
                    <SelectTrigger id="osType">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="linux">Linux</SelectItem>
                      <SelectItem value="windows">Windows</SelectItem>
                      <SelectItem value="other">Other</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="network">Network *</Label>
                  <Input
                    id="network"
                    placeholder="default"
                    value={formData.network}
                    onChange={(e) => setFormData({ ...formData, network: e.target.value })}
                  />
                  <p className="text-xs text-muted-foreground">Libvirt network name (e.g., 'default')</p>
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="isoPath">ISO Path (optional)</Label>
                <Input
                  id="isoPath"
                  placeholder="/var/lib/libvirt/images/alpine.iso"
                  value={formData.isoPath || ''}
                  onChange={(e) => setFormData({ ...formData, isoPath: e.target.value || undefined })}
                />
                <div className="space-y-1">
                  <p className="text-xs text-muted-foreground">
                    Full path to an ISO image for OS installation. Leave empty to attach later.
                  </p>
                  <p className="text-xs text-amber-600 dark:text-amber-500">
                    ⚠️ ISO must be accessible by libvirt (copy to /var/lib/libvirt/images/ or fix permissions)
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="diskFormat">Disk Format *</Label>
                  <Select value={formData.diskFormat} onValueChange={(value: any) => setFormData({ ...formData, diskFormat: value })}>
                    <SelectTrigger id="diskFormat">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="qcow2">QCOW2 (Recommended)</SelectItem>
                      <SelectItem value="raw">RAW</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    QCOW2 supports snapshots and compression
                  </p>
                </div>

                <div className="space-y-2">
                  <Label>Boot Options</Label>
                  <div className="flex items-center space-x-2 pt-2">
                    <Checkbox
                      id="bootMenu"
                      checked={formData.bootMenu}
                      onCheckedChange={(checked) => setFormData({ ...formData, bootMenu: checked === true })}
                    />
                    <label
                      htmlFor="bootMenu"
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      Enable boot menu
                    </label>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Show BIOS boot menu on startup
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Step 3: Review */}
          {currentStep === 3 && (
            <div className="space-y-4">
              <div className="bg-muted p-4 rounded-lg space-y-3">
                <h4 className="font-medium">VM Configuration Summary</h4>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Name:</span>
                    <span className="font-medium">{formData.name}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">OS Type:</span>
                    <span className="font-medium capitalize">{formData.osType}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">CPU:</span>
                    <span className="font-medium">{formData.cpuCount} cores</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Memory:</span>
                    <span className="font-medium">{formData.memoryMb} MB ({(formData.memoryMb / 1024).toFixed(1)} GB)</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Disk:</span>
                    <span className="font-medium">{formData.diskSizeGb} GB ({formData.diskFormat.toUpperCase()})</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Network:</span>
                    <span className="font-medium">{formData.network}</span>
                  </div>
                  {formData.isoPath && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">ISO:</span>
                      <span className="font-medium text-xs truncate max-w-[200px]" title={formData.isoPath}>
                        {formData.isoPath}
                      </span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Boot Menu:</span>
                    <span className="font-medium">{formData.bootMenu ? 'Enabled' : 'Disabled'}</span>
                  </div>
                </div>
              </div>

              <div className="bg-blue-50 dark:bg-blue-950 p-4 rounded-lg text-sm">
                <p className="text-blue-900 dark:text-blue-100">
                  <strong>Note:</strong> The VM will be created in a stopped state.
                  {formData.isoPath ? ' You can start it to begin the installation.' : ' Attach an ISO image to install an operating system.'}
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
                      Create VM
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
