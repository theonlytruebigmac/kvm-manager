import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { open } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { ChevronLeft, ChevronRight, Check, FolderOpen, ChevronDown, ChevronUp } from 'lucide-react'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { CloudInitConfig, defaultCloudInitConfig } from './CloudInitConfig'
import type { CloudInitConfig as CloudInitConfigType } from '@/lib/types'

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
  firmware: 'bios' | 'uefi' | 'uefi-secure'
  tpmEnabled: boolean
  chipset: 'pc' | 'q35'
  cpuSockets: number
  cpuCores: number
  cpuThreads: number
  bootOrder: string[]
  cloudInit?: CloudInitConfigType
  // New: Import existing disk support
  installationType: 'iso' | 'import' | 'network' | 'manual'
  existingDiskPath?: string
  // Network install URL (HTTP/HTTPS/FTP)
  networkInstallUrl?: string
}

const steps = [
  { id: 1, name: 'Basic Info', description: 'VM name and resources' },
  { id: 2, name: 'Configuration', description: 'OS, network, and disk settings' },
  { id: 3, name: 'Advanced', description: 'Firmware and hardware options' },
  { id: 4, name: 'Review', description: 'Confirm settings' },
]

interface CreateVmWizardProps {
  onClose: () => void
}

export function CreateVmWizard({ onClose }: CreateVmWizardProps) {
  const queryClient = useQueryClient()
  const [currentStep, setCurrentStep] = useState(1)
  const [showCpuTopology, setShowCpuTopology] = useState(false)
  const [customizeBeforeInstall, setCustomizeBeforeInstall] = useState(false)
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
    firmware: 'bios',
    tpmEnabled: false,
    chipset: 'pc',
    cpuSockets: 1,
    cpuCores: 2,
    cpuThreads: 1,
    bootOrder: ['cdrom', 'hd'],
    cloudInit: defaultCloudInitConfig,
    installationType: 'iso',
    existingDiskPath: undefined,
    networkInstallUrl: undefined,
  })
  const createMutation = useMutation({
    mutationFn: () => api.createVm(formData),
    onSuccess: async (vmId) => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })

      if (customizeBeforeInstall && vmId) {
        // Open the VM details window for customization before starting
        try {
          await api.openVmDetailsWindow(vmId, formData.name)
          toast.success(`VM "${formData.name}" created. Customize it now before starting.`)
        } catch (err) {
          toast.success(`VM "${formData.name}" created successfully`)
          console.error('Failed to open VM details window:', err)
        }
      } else {
        toast.success(`VM "${formData.name}" created successfully`)
      }

      onClose()
    },
    onError: (error) => {
      toast.error(`Failed to create VM: ${error}`)
    },
  })

  // Helper function to update CPU topology and keep it in sync
  const updateCpuTopology = (field: 'sockets' | 'cores' | 'threads', value: number) => {
    const newData = { ...formData }

    if (field === 'sockets') {
      newData.cpuSockets = value
    } else if (field === 'cores') {
      newData.cpuCores = value
    } else if (field === 'threads') {
      newData.cpuThreads = value
    }

    // Recalculate total vCPUs
    newData.cpuCount = newData.cpuSockets * newData.cpuCores * newData.cpuThreads
    setFormData(newData)
  }

  // Helper function to update total CPU count and adjust topology
  const updateCpuCount = (count: number) => {
    // Keep 1 socket and 1 thread, adjust cores
    setFormData({
      ...formData,
      cpuCount: count,
      cpuCores: count,
      cpuSockets: 1,
      cpuThreads: 1,
    })
  }

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
      // Validate CPU topology
      const calculatedCpus = formData.cpuSockets * formData.cpuCores * formData.cpuThreads
      if (calculatedCpus !== formData.cpuCount) {
        toast.error(`CPU topology mismatch: ${formData.cpuSockets} × ${formData.cpuCores} × ${formData.cpuThreads} = ${calculatedCpus}, but total is ${formData.cpuCount}`)
        return
      }
      if (formData.memoryMb < 512 || formData.memoryMb > 32768) {
        toast.error('Memory must be between 512 MB and 32 GB')
        return
      }
      // Only validate disk size if NOT importing existing disk
      if (formData.installationType !== 'import') {
        if (formData.diskSizeGb < 1 || formData.diskSizeGb > 500) {
          toast.error('Disk size must be between 1 GB and 500 GB')
          return
        }
      }
    }
    if (currentStep === 2) {
      // Validate step 2
      if (!formData.network.trim()) {
        toast.error('Please select a network')
        return
      }
      // Validate import disk path if importing
      if (formData.installationType === 'import' && !formData.existingDiskPath?.trim()) {
        toast.error('Please specify the path to the existing disk image')
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
                  autoFocus
                />
              </div>

              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="cpu">Total vCPUs *</Label>
                  <Input
                    id="cpu"
                    type="number"
                    min="1"
                    max="16"
                    value={formData.cpuCount}
                    onChange={(e) => updateCpuCount(parseInt(e.target.value) || 1)}
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

              {/* Advanced CPU Topology */}
              <Collapsible open={showCpuTopology} onOpenChange={setShowCpuTopology}>
                <CollapsibleTrigger asChild>
                  <Button variant="ghost" size="sm" className="w-full justify-between">
                    <span className="text-sm font-medium">Advanced CPU Configuration</span>
                    {showCpuTopology ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </Button>
                </CollapsibleTrigger>
                <CollapsibleContent className="space-y-4 pt-4">
                  <div className="bg-muted p-4 rounded-lg space-y-4">
                    <p className="text-sm text-muted-foreground">
                      Configure CPU topology for performance tuning. Total vCPUs = Sockets × Cores × Threads
                    </p>

                    <div className="grid grid-cols-3 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="sockets">Sockets</Label>
                        <Input
                          id="sockets"
                          type="number"
                          min="1"
                          max="4"
                          value={formData.cpuSockets}
                          onChange={(e) => updateCpuTopology('sockets', parseInt(e.target.value) || 1)}
                        />
                        <p className="text-xs text-muted-foreground">Physical CPUs</p>
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="cores">Cores per Socket</Label>
                        <Input
                          id="cores"
                          type="number"
                          min="1"
                          max="8"
                          value={formData.cpuCores}
                          onChange={(e) => updateCpuTopology('cores', parseInt(e.target.value) || 1)}
                        />
                        <p className="text-xs text-muted-foreground">Cores per CPU</p>
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="threads">Threads per Core</Label>
                        <Input
                          id="threads"
                          type="number"
                          min="1"
                          max="2"
                          value={formData.cpuThreads}
                          onChange={(e) => updateCpuTopology('threads', parseInt(e.target.value) || 1)}
                        />
                        <p className="text-xs text-muted-foreground">SMT/HT threads</p>
                      </div>
                    </div>

                    <div className="bg-blue-50 dark:bg-blue-950 p-3 rounded text-sm">
                      <p className="text-blue-900 dark:text-blue-100 font-medium mb-1">
                        Current Configuration:
                      </p>
                      <p className="text-blue-800 dark:text-blue-200">
                        {formData.cpuSockets} socket{formData.cpuSockets > 1 ? 's' : ''} × {formData.cpuCores} core{formData.cpuCores > 1 ? 's' : ''} × {formData.cpuThreads} thread{formData.cpuThreads > 1 ? 's' : ''} = <strong>{formData.cpuCount} total vCPUs</strong>
                      </p>
                    </div>

                    <div className="space-y-2">
                      <p className="text-sm font-medium">Quick Presets:</p>
                      <div className="grid grid-cols-2 gap-2">
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            setFormData({ ...formData, cpuSockets: 1, cpuCores: 2, cpuThreads: 1, cpuCount: 2 })
                          }}
                        >
                          1×2×1 (2 cores)
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            setFormData({ ...formData, cpuSockets: 1, cpuCores: 4, cpuThreads: 1, cpuCount: 4 })
                          }}
                        >
                          1×4×1 (4 cores)
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            setFormData({ ...formData, cpuSockets: 2, cpuCores: 2, cpuThreads: 1, cpuCount: 4 })
                          }}
                        >
                          2×2×1 (2 CPUs)
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            setFormData({ ...formData, cpuSockets: 1, cpuCores: 4, cpuThreads: 2, cpuCount: 8 })
                          }}
                        >
                          1×4×2 (SMT)
                        </Button>
                      </div>
                    </div>
                  </div>
                </CollapsibleContent>
              </Collapsible>

              <div className="bg-muted p-4 rounded-lg">
                <h4 className="text-sm font-medium mb-2">Quick Presets</h4>
                <div className="grid grid-cols-3 gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({ ...formData, cpuCount: 1, cpuSockets: 1, cpuCores: 1, cpuThreads: 1, memoryMb: 512, diskSizeGb: 10 })}
                  >
                    Minimal
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({ ...formData, cpuCount: 2, cpuSockets: 1, cpuCores: 2, cpuThreads: 1, memoryMb: 2048, diskSizeGb: 20 })}
                  >
                    Standard
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({ ...formData, cpuCount: 4, cpuSockets: 1, cpuCores: 4, cpuThreads: 1, memoryMb: 4096, diskSizeGb: 50 })}
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

              {/* Installation Type Selection */}
              <div className="space-y-3">
                <Label>Installation Source</Label>
                <div className="grid grid-cols-4 gap-2">
                  <Button
                    type="button"
                    variant={formData.installationType === 'iso' ? 'default' : 'outline'}
                    className="h-auto py-3 flex flex-col items-center gap-1"
                    onClick={() => setFormData({
                      ...formData,
                      installationType: 'iso',
                      existingDiskPath: undefined,
                      networkInstallUrl: undefined,
                      bootOrder: ['cdrom', 'hd']
                    })}
                  >
                    <span className="text-sm font-medium">ISO Image</span>
                    <span className="text-xs text-muted-foreground">Boot from ISO</span>
                  </Button>
                  <Button
                    type="button"
                    variant={formData.installationType === 'network' ? 'default' : 'outline'}
                    className="h-auto py-3 flex flex-col items-center gap-1"
                    onClick={() => setFormData({
                      ...formData,
                      installationType: 'network',
                      isoPath: undefined,
                      existingDiskPath: undefined,
                      bootOrder: ['network', 'hd']
                    })}
                  >
                    <span className="text-sm font-medium">Network</span>
                    <span className="text-xs text-muted-foreground">HTTP/FTP URL</span>
                  </Button>
                  <Button
                    type="button"
                    variant={formData.installationType === 'import' ? 'default' : 'outline'}
                    className="h-auto py-3 flex flex-col items-center gap-1"
                    onClick={() => setFormData({
                      ...formData,
                      installationType: 'import',
                      isoPath: undefined,
                      networkInstallUrl: undefined,
                      bootOrder: ['hd']
                    })}
                  >
                    <span className="text-sm font-medium">Import Disk</span>
                    <span className="text-xs text-muted-foreground">Existing image</span>
                  </Button>
                  <Button
                    type="button"
                    variant={formData.installationType === 'manual' ? 'default' : 'outline'}
                    className="h-auto py-3 flex flex-col items-center gap-1"
                    onClick={() => setFormData({
                      ...formData,
                      installationType: 'manual',
                      isoPath: undefined,
                      existingDiskPath: undefined,
                      networkInstallUrl: undefined,
                      bootOrder: ['hd', 'cdrom']
                    })}
                  >
                    <span className="text-sm font-medium">Manual</span>
                    <span className="text-xs text-muted-foreground">Configure later</span>
                  </Button>
                </div>
              </div>

              {/* ISO Path - only shown for ISO installation */}
              {formData.installationType === 'iso' && (
                <div className="space-y-2">
                  <Label htmlFor="isoPath">ISO Path</Label>
                  <div className="flex gap-2">
                    <Input
                      id="isoPath"
                      placeholder="/var/lib/libvirt/images/alpine.iso"
                      value={formData.isoPath || ''}
                      onChange={(e) => setFormData({ ...formData, isoPath: e.target.value || undefined })}
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={async () => {
                        try {
                          const selected = await open({
                            multiple: false,
                            directory: false,
                            defaultPath: await (async () => {
                              try {
                                const homeDir = await import('@tauri-apps/api/path').then(m => m.downloadDir())
                                return await homeDir
                              } catch {
                                return '/var/lib/libvirt/images'
                              }
                            })(),
                            filters: [{
                              name: 'ISO Images',
                              extensions: ['iso']
                            }]
                          })
                          if (selected) {
                            setFormData({ ...formData, isoPath: selected as string })
                          }
                        } catch (error) {
                          toast.error('Failed to open file browser')
                        }
                      }}
                      title="Browse for ISO file"
                    >
                      <FolderOpen className="h-4 w-4" />
                    </Button>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Full path to an ISO image for OS installation.
                  </p>
                </div>
              )}

              {/* Existing Disk Path - only shown for Import installation */}
              {formData.installationType === 'import' && (
                <div className="space-y-2">
                  <Label htmlFor="existingDiskPath">Existing Disk Image Path *</Label>
                  <div className="flex gap-2">
                    <Input
                      id="existingDiskPath"
                      placeholder="/var/lib/libvirt/images/my-disk.qcow2"
                      value={formData.existingDiskPath || ''}
                      onChange={(e) => setFormData({ ...formData, existingDiskPath: e.target.value || undefined })}
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={async () => {
                        try {
                          const selected = await open({
                            multiple: false,
                            directory: false,
                            defaultPath: '/var/lib/libvirt/images',
                            filters: [{
                              name: 'Disk Images',
                              extensions: ['qcow2', 'raw', 'img', 'vmdk', 'vdi', 'vhd', 'vhdx']
                            }]
                          })
                          if (selected) {
                            setFormData({ ...formData, existingDiskPath: selected as string })
                          }
                        } catch (error) {
                          toast.error('Failed to open file browser')
                        }
                      }}
                      title="Browse for disk image"
                    >
                      <FolderOpen className="h-4 w-4" />
                    </Button>
                  </div>
                  <div className="space-y-1">
                    <p className="text-xs text-muted-foreground">
                      Select an existing disk image (QCOW2, RAW, VMDK, VDI, VHD).
                    </p>
                    <p className="text-xs text-blue-600 dark:text-blue-400">
                      💡 The disk will be used as-is. A new disk will NOT be created.
                    </p>
                  </div>
                </div>
              )}

              {/* Network Install URL - only shown for Network installation */}
              {formData.installationType === 'network' && (
                <div className="space-y-2">
                  <Label htmlFor="networkInstallUrl">Network Install URL *</Label>
                  <Input
                    id="networkInstallUrl"
                    placeholder="http://archive.ubuntu.com/ubuntu/dists/jammy/main/installer-amd64/"
                    value={formData.networkInstallUrl || ''}
                    onChange={(e) => setFormData({ ...formData, networkInstallUrl: e.target.value || undefined })}
                  />
                  <div className="space-y-1">
                    <p className="text-xs text-muted-foreground">
                      HTTP, HTTPS, or FTP URL to an installation tree or netboot image.
                    </p>
                    <p className="text-xs text-blue-600 dark:text-blue-400">
                      💡 Examples: Debian/Ubuntu netboot, CentOS/Fedora mirrors, or custom PXE server.
                    </p>
                  </div>
                </div>
              )}

              {/* Manual installation info */}
              {formData.installationType === 'manual' && (
                <div className="p-4 bg-muted rounded-lg">
                  <p className="text-sm text-muted-foreground">
                    A new empty disk will be created. You can attach an ISO or PXE boot after VM creation.
                  </p>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4">
                {/* Only show disk format for non-import installations */}
                {formData.installationType !== 'import' && (
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
                )}

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

          {/* Step 3: Advanced Configuration */}
          {currentStep === 3 && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="firmware">Firmware</Label>
                <Select value={formData.firmware} onValueChange={(value: any) => setFormData({ ...formData, firmware: value })}>
                  <SelectTrigger id="firmware">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="bios">BIOS (Legacy)</SelectItem>
                    <SelectItem value="uefi">UEFI</SelectItem>
                    <SelectItem value="uefi-secure">UEFI with Secure Boot</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {formData.firmware === 'bios' && 'Traditional BIOS boot (compatible with older systems)'}
                  {formData.firmware === 'uefi' && 'Modern UEFI boot (recommended for new installations)'}
                  {formData.firmware === 'uefi-secure' && 'UEFI with Secure Boot (required for Windows 11, recommended for security)'}
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="chipset">Chipset</Label>
                <Select value={formData.chipset} onValueChange={(value: any) => setFormData({ ...formData, chipset: value })}>
                  <SelectTrigger id="chipset">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="pc">i440FX (PC)</SelectItem>
                    <SelectItem value="q35">Q35 (PCIe)</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {formData.chipset === 'pc' && 'Legacy chipset, good compatibility but limited features'}
                  {formData.chipset === 'q35' && 'Modern PCIe chipset (required for PCIe passthrough, recommended for Windows)'}
                </p>
              </div>

              <div className="space-y-2">
                <Label>Security Features</Label>
                <div className="space-y-3 p-3 border rounded-lg">
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id="tpm"
                      checked={formData.tpmEnabled}
                      onCheckedChange={(checked) => setFormData({ ...formData, tpmEnabled: checked === true })}
                    />
                    <label
                      htmlFor="tpm"
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      Enable TPM 2.0
                    </label>
                  </div>
                  <p className="text-xs text-muted-foreground ml-6">
                    Virtual Trusted Platform Module (required for Windows 11, BitLocker encryption)
                  </p>
                </div>
              </div>

              <div className="bg-amber-50 dark:bg-amber-950 p-4 rounded-lg text-sm space-y-2">
                <p className="text-amber-900 dark:text-amber-100 font-medium">
                  ℹ️ Configuration Notes:
                </p>
                <ul className="text-amber-800 dark:text-amber-200 space-y-1 ml-4 list-disc">
                  <li>Windows 11 requires: UEFI with Secure Boot + TPM 2.0 + Q35 chipset</li>
                  <li>UEFI/Secure Boot requires OVMF firmware (install via: sudo apt install ovmf)</li>
                  <li>Secure Boot requires compatible OS (Windows 8+, recent Linux with signed kernel)</li>
                  <li>Changing firmware after OS installation will prevent boot</li>
                  <li>Q35 chipset is required for GPU passthrough and modern features</li>
                </ul>
              </div>

              {/* Quick Presets */}
              <div className="space-y-2">
                <Label>Quick Presets</Label>
                <div className="grid grid-cols-2 gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({
                      ...formData,
                      firmware: 'uefi-secure',
                      tpmEnabled: true,
                      chipset: 'q35',
                      osType: 'windows'
                    })}
                  >
                    Windows 11 Setup
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({
                      ...formData,
                      firmware: 'uefi',
                      tpmEnabled: false,
                      chipset: 'q35',
                      osType: 'linux'
                    })}
                  >
                    Modern Linux
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({
                      ...formData,
                      firmware: 'bios',
                      tpmEnabled: false,
                      chipset: 'pc',
                      osType: 'linux'
                    })}
                  >
                    Legacy/Compatible
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setFormData({
                      ...formData,
                      firmware: 'uefi-secure',
                      tpmEnabled: true,
                      chipset: 'q35',
                      osType: 'windows'
                    })}
                  >
                    Maximum Security
                  </Button>
                </div>
              </div>

              {/* Cloud-Init Section */}
              <div className="border-t pt-4 mt-4">
                <CloudInitConfig
                  config={formData.cloudInit || defaultCloudInitConfig}
                  onChange={(cloudInit) => setFormData({ ...formData, cloudInit })}
                />
              </div>
            </div>
          )}

          {/* Step 4: Review */}
          {currentStep === 4 && (
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
                    <span className="font-medium">{formData.cpuCount} vCPUs ({formData.cpuSockets}×{formData.cpuCores}×{formData.cpuThreads})</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Memory:</span>
                    <span className="font-medium">{formData.memoryMb} MB ({(formData.memoryMb / 1024).toFixed(1)} GB)</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Installation:</span>
                    <span className="font-medium capitalize">
                      {formData.installationType === 'iso' && 'ISO Image'}
                      {formData.installationType === 'import' && 'Import Existing Disk'}
                      {formData.installationType === 'manual' && 'Manual (No Media)'}
                    </span>
                  </div>
                  {formData.installationType === 'import' ? (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Disk Image:</span>
                      <span className="font-medium text-xs truncate max-w-[200px]" title={formData.existingDiskPath}>
                        {formData.existingDiskPath}
                      </span>
                    </div>
                  ) : (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Disk:</span>
                      <span className="font-medium">{formData.diskSizeGb} GB ({formData.diskFormat.toUpperCase()})</span>
                    </div>
                  )}
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
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Firmware:</span>
                    <span className="font-medium">
                      {formData.firmware === 'bios' && 'BIOS'}
                      {formData.firmware === 'uefi' && 'UEFI'}
                      {formData.firmware === 'uefi-secure' && 'UEFI + Secure Boot'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Chipset:</span>
                    <span className="font-medium">{formData.chipset === 'q35' ? 'Q35 (PCIe)' : 'i440FX (PC)'}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">TPM 2.0:</span>
                    <span className="font-medium">{formData.tpmEnabled ? 'Enabled' : 'Disabled'}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Cloud-Init:</span>
                    <span className="font-medium">{formData.cloudInit?.enabled ? 'Enabled' : 'Disabled'}</span>
                  </div>
                  {formData.cloudInit?.enabled && (
                    <>
                      {formData.cloudInit.hostname && (
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">Hostname:</span>
                          <span className="font-medium">{formData.cloudInit.hostname}</span>
                        </div>
                      )}
                      {formData.cloudInit.username && (
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">User:</span>
                          <span className="font-medium">{formData.cloudInit.username}</span>
                        </div>
                      )}
                      {formData.cloudInit.sshAuthorizedKeys.length > 0 && (
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">SSH Keys:</span>
                          <span className="font-medium">{formData.cloudInit.sshAuthorizedKeys.length} key(s)</span>
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>

              <div className="bg-blue-50 dark:bg-blue-950 p-4 rounded-lg text-sm">
                <p className="text-blue-900 dark:text-blue-100">
                  <strong>Note:</strong> The VM will be created in a stopped state.
                  {formData.isoPath ? ' You can start it to begin the installation.' : ' Attach an ISO image to install an operating system.'}
                </p>
              </div>

              {/* Customize before install option */}
              <div className="flex items-center space-x-2 p-3 bg-muted/50 rounded-lg">
                <Checkbox
                  id="customizeBeforeInstall"
                  checked={customizeBeforeInstall}
                  onCheckedChange={(checked) => setCustomizeBeforeInstall(checked === true)}
                />
                <div>
                  <Label htmlFor="customizeBeforeInstall" className="cursor-pointer font-medium">
                    Customize configuration before install
                  </Label>
                  <p className="text-xs text-muted-foreground">
                    Opens the VM details window to add hardware, adjust settings, etc.
                  </p>
                </div>
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