import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { Switch } from '@/components/ui/switch'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, CheckCircle, ChevronUp, ChevronDown, HardDrive, Disc, Network, Terminal, FolderOpen } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useQueryClient } from '@tanstack/react-query'
import type { VM, KernelBootSettings } from '@/lib/types'
import { open } from '@tauri-apps/plugin-dialog'

interface BootEditorProps {
  vm: VM
  compact?: boolean
}

type BootDevice = 'hd' | 'cdrom' | 'network'

const BOOT_DEVICE_INFO: Record<BootDevice, { label: string; icon: React.ComponentType<{ className?: string }> }> = {
  hd: { label: 'Hard Disk', icon: HardDrive },
  cdrom: { label: 'CDROM', icon: Disc },
  network: { label: 'Network (PXE)', icon: Network },
}

export function BootEditor({ vm, compact }: BootEditorProps) {
  const queryClient = useQueryClient()
  const [autostart, setAutostart] = useState<boolean>(false)
  const [autostartLoading, setAutostartLoading] = useState<boolean>(true)

  // Boot order state
  const getInitialBootOrder = (): BootDevice[] => {
    if (vm.bootOrder && vm.bootOrder.length > 0) {
      return vm.bootOrder as BootDevice[]
    }
    // Default boot order
    const order: BootDevice[] = ['hd']
    if (vm.cdrom) order.push('cdrom')
    order.push('network')
    return order
  }

  const [bootOrder, setBootOrder] = useState<BootDevice[]>(getInitialBootOrder)
  const [bootOrderLoading, setBootOrderLoading] = useState(false)
  const [bootOrderError, setBootOrderError] = useState<string | null>(null)
  const [bootOrderSuccess, setBootOrderSuccess] = useState(false)

  const originalBootOrder = vm.bootOrder || getInitialBootOrder()
  const hasBootOrderChanges = JSON.stringify(bootOrder) !== JSON.stringify(originalBootOrder)

  // Direct kernel boot state
  const [kernelBoot, setKernelBoot] = useState<KernelBootSettings>({
    enabled: false,
    kernelPath: '',
    initrdPath: '',
    kernelArgs: '',
    dtbPath: '',
  })
  const [originalKernelBoot, setOriginalKernelBoot] = useState<KernelBootSettings | null>(null)
  const [kernelBootLoading, setKernelBootLoading] = useState(false)
  const [kernelBootSaving, setKernelBootSaving] = useState(false)
  const [kernelBootError, setKernelBootError] = useState<string | null>(null)
  const [kernelBootSuccess, setKernelBootSuccess] = useState(false)

  const hasKernelBootChanges = originalKernelBoot !== null &&
    JSON.stringify(kernelBoot) !== JSON.stringify(originalKernelBoot)

  // Load autostart status on mount
  useEffect(() => {
    api.getVmAutostart(vm.id)
      .then((enabled) => {
        setAutostart(enabled)
      })
      .catch((error) => {
        console.error('Failed to get autostart status:', error)
      })
      .finally(() => {
        setAutostartLoading(false)
      })
  }, [vm.id])

  // Load kernel boot settings on mount
  useEffect(() => {
    setKernelBootLoading(true)
    api.getKernelBootSettings(vm.id)
      .then((settings) => {
        setKernelBoot(settings)
        setOriginalKernelBoot(settings)
      })
      .catch((error) => {
        console.error('Failed to get kernel boot settings:', error)
      })
      .finally(() => {
        setKernelBootLoading(false)
      })
  }, [vm.id])

  // Sync boot order when VM data changes
  useEffect(() => {
    setBootOrder(getInitialBootOrder())
  }, [vm])

  // Handle autostart toggle
  const handleAutostartChange = async (enabled: boolean) => {
    try {
      await api.setVmAutostart(vm.id, enabled)
      setAutostart(enabled)
    } catch (error) {
      console.error('Failed to set autostart:', error)
    }
  }

  // Direct kernel boot file pickers
  const browseKernel = async () => {
    try {
      const selected = await open({
        title: 'Select Kernel',
        filters: [{ name: 'Kernel', extensions: ['vmlinuz', 'vmlinux', 'bzImage', 'Image', '*'] }],
      })
      if (selected) {
        setKernelBoot({ ...kernelBoot, kernelPath: selected as string })
      }
    } catch (err) {
      console.error('Failed to open file dialog:', err)
    }
  }

  const browseInitrd = async () => {
    try {
      const selected = await open({
        title: 'Select Initrd/Initramfs',
        filters: [{ name: 'Initrd', extensions: ['img', 'gz', 'lz4', 'xz', 'cpio', '*'] }],
      })
      if (selected) {
        setKernelBoot({ ...kernelBoot, initrdPath: selected as string })
      }
    } catch (err) {
      console.error('Failed to open file dialog:', err)
    }
  }

  const browseDtb = async () => {
    try {
      const selected = await open({
        title: 'Select Device Tree Blob',
        filters: [{ name: 'DTB', extensions: ['dtb', '*'] }],
      })
      if (selected) {
        setKernelBoot({ ...kernelBoot, dtbPath: selected as string })
      }
    } catch (err) {
      console.error('Failed to open file dialog:', err)
    }
  }

  const handleApplyKernelBoot = async () => {
    setKernelBootSaving(true)
    setKernelBootError(null)
    setKernelBootSuccess(false)

    try {
      await api.setKernelBootSettings(vm.id, kernelBoot)
      setOriginalKernelBoot(kernelBoot)
      setKernelBootSuccess(true)
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      setTimeout(() => setKernelBootSuccess(false), 3000)
    } catch (err) {
      setKernelBootError(err instanceof Error ? err.message : String(err))
    } finally {
      setKernelBootSaving(false)
    }
  }

  const handleRevertKernelBoot = () => {
    if (originalKernelBoot) {
      setKernelBoot(originalKernelBoot)
    }
    setKernelBootError(null)
  }

  const moveDevice = (index: number, direction: 'up' | 'down') => {
    const newOrder = [...bootOrder]
    const newIndex = direction === 'up' ? index - 1 : index + 1
    if (newIndex < 0 || newIndex >= newOrder.length) return
    ;[newOrder[index], newOrder[newIndex]] = [newOrder[newIndex], newOrder[index]]
    setBootOrder(newOrder)
  }

  const handleApplyBootOrder = async () => {
    setBootOrderLoading(true)
    setBootOrderError(null)
    setBootOrderSuccess(false)

    try {
      await api.updateVmBootOrder(vm.id, bootOrder)
      setBootOrderSuccess(true)
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      setTimeout(() => setBootOrderSuccess(false), 3000)
    } catch (err) {
      setBootOrderError(err instanceof Error ? err.message : String(err))
    } finally {
      setBootOrderLoading(false)
    }
  }

  const handleRevertBootOrder = () => {
    setBootOrder(getInitialBootOrder())
    setBootOrderError(null)
  }

  // Compact mode - inline boot order pills and autostart toggle
  if (compact) {
    return (
      <div className="space-y-2">
        <div className="flex items-center flex-wrap gap-1.5">
          {bootOrder.map((device, index) => {
            const info = BOOT_DEVICE_INFO[device]
            const Icon = info.icon
            return (
              <div key={device} className="flex items-center gap-1 px-2 py-0.5 border rounded bg-muted/50 text-xs">
                <span className="text-muted-foreground">{index + 1}.</span>
                <Icon className="h-3 w-3" />
                <span>{info.label}</span>
                <Button size="icon" variant="ghost" className="h-4 w-4 ml-0.5" onClick={() => moveDevice(index, 'up')} disabled={index === 0}>
                  <ChevronUp className="h-2.5 w-2.5" />
                </Button>
                <Button size="icon" variant="ghost" className="h-4 w-4" onClick={() => moveDevice(index, 'down')} disabled={index === bootOrder.length - 1}>
                  <ChevronDown className="h-2.5 w-2.5" />
                </Button>
              </div>
            )
          })}
          {hasBootOrderChanges && (
            <>
              <Button size="sm" variant="default" onClick={handleApplyBootOrder} disabled={bootOrderLoading} className="h-5 px-2 text-xs">
                {bootOrderLoading ? <Loader2 className="h-3 w-3 animate-spin" /> : 'Apply'}
              </Button>
              <Button size="sm" variant="ghost" onClick={handleRevertBootOrder} className="h-5 px-2 text-xs">Revert</Button>
            </>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Switch checked={autostart} onCheckedChange={handleAutostartChange} disabled={autostartLoading} className="scale-75" />
          <span className="text-xs text-muted-foreground">Autostart with host</span>
        </div>
        {bootOrderError && <p className="text-xs text-red-500">{bootOrderError}</p>}
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div>
        <h2 className="text-2xl font-semibold mb-2">Boot Options</h2>
        <p className="text-sm text-muted-foreground">
          Configure boot order and firmware settings
        </p>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="kernel">Direct Kernel Boot</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle>Firmware</CardTitle>
              <CardDescription>
                Firmware type and loader configuration
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="firmware">Firmware Type</Label>
                <Select defaultValue={vm.firmware || 'bios'} disabled>
                  <SelectTrigger id="firmware">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="bios">BIOS</SelectItem>
                    <SelectItem value="uefi">UEFI</SelectItem>
                    <SelectItem value="uefi-secure">UEFI with Secure Boot</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Current firmware: {vm.firmware?.toUpperCase() || 'BIOS'}
                </p>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Boot Order</CardTitle>
              <CardDescription>
                Device boot priority order - use arrows to reorder
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                {bootOrder.map((device, index) => {
                  const info = BOOT_DEVICE_INFO[device]
                  const Icon = info.icon
                  return (
                    <div key={device} className="flex items-center justify-between p-3 border rounded-md">
                      <div className="flex items-center gap-3">
                        <span className="font-mono text-sm font-medium w-6">{index + 1}</span>
                        <Icon className="h-4 w-4 text-muted-foreground" />
                        <span>{info.label}</span>
                      </div>
                      <div className="flex gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={() => moveDevice(index, 'up')}
                          disabled={index === 0}
                        >
                          <ChevronUp className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={() => moveDevice(index, 'down')}
                          disabled={index === bootOrder.length - 1}
                        >
                          <ChevronDown className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  )
                })}
              </div>

              {bootOrderError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{bootOrderError}</AlertDescription>
                </Alert>
              )}

              {bootOrderSuccess && (
                <Alert>
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <AlertDescription>Boot order updated successfully!</AlertDescription>
                </Alert>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Boot Options</CardTitle>
              <CardDescription>
                Additional boot configuration
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="autostart" className="text-sm font-medium">
                    Start on host boot
                  </Label>
                  <p className="text-xs text-muted-foreground">
                    Automatically start this VM when the host system boots
                  </p>
                </div>
                <Switch
                  id="autostart"
                  checked={autostart}
                  onCheckedChange={handleAutostartChange}
                  disabled={autostartLoading}
                />
              </div>

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="boot-menu"
                  defaultChecked={vm.bootMenu}
                  disabled
                />
                <Label htmlFor="boot-menu" className="text-sm font-normal">
                  Enable boot menu
                </Label>
              </div>
              <p className="text-xs text-muted-foreground">
                Press a key during boot to select boot device
              </p>
            </CardContent>
          </Card>

          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={handleRevertBootOrder}
              disabled={!hasBootOrderChanges || bootOrderLoading}
            >
              Revert
            </Button>
            <Button
              onClick={handleApplyBootOrder}
              disabled={!hasBootOrderChanges || bootOrderLoading}
            >
              {bootOrderLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Apply
            </Button>
          </div>
        </TabsContent>

        <TabsContent value="kernel" className="space-y-6 pt-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Terminal className="h-5 w-5" />
                Direct Kernel Boot
              </CardTitle>
              <CardDescription>
                Boot directly from a kernel and initrd on the host system, bypassing the normal boot process.
                Useful for kernel development, debugging, or custom boot scenarios.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {vm.state === 'running' && (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    VM must be shut down to modify kernel boot settings.
                  </AlertDescription>
                </Alert>
              )}

              {kernelBootLoading ? (
                <div className="flex items-center justify-center p-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <>
                  <div className="flex items-center justify-between">
                    <div className="space-y-0.5">
                      <Label htmlFor="kernel-boot-enabled" className="text-sm font-medium">
                        Enable Direct Kernel Boot
                      </Label>
                      <p className="text-xs text-muted-foreground">
                        When enabled, the VM will boot using the specified kernel instead of the disk bootloader
                      </p>
                    </div>
                    <Switch
                      id="kernel-boot-enabled"
                      checked={kernelBoot.enabled}
                      onCheckedChange={(checked) => setKernelBoot({ ...kernelBoot, enabled: checked })}
                      disabled={vm.state === 'running'}
                    />
                  </div>

                  <div className="space-y-4 pt-4">
                    <div className="space-y-2">
                      <Label htmlFor="kernel-path">Kernel Path</Label>
                      <div className="flex gap-2">
                        <Input
                          id="kernel-path"
                          placeholder="/boot/vmlinuz-linux or /path/to/custom-kernel"
                          value={kernelBoot.kernelPath || ''}
                          onChange={(e) => setKernelBoot({ ...kernelBoot, kernelPath: e.target.value })}
                          disabled={!kernelBoot.enabled || vm.state === 'running'}
                        />
                        <Button
                          variant="outline"
                          size="icon"
                          onClick={browseKernel}
                          disabled={!kernelBoot.enabled || vm.state === 'running'}
                        >
                          <FolderOpen className="h-4 w-4" />
                        </Button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Path to the kernel image on the host (e.g., vmlinuz, bzImage)
                      </p>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="initrd-path">Initrd/Initramfs Path</Label>
                      <div className="flex gap-2">
                        <Input
                          id="initrd-path"
                          placeholder="/boot/initramfs-linux.img (optional)"
                          value={kernelBoot.initrdPath || ''}
                          onChange={(e) => setKernelBoot({ ...kernelBoot, initrdPath: e.target.value })}
                          disabled={!kernelBoot.enabled || vm.state === 'running'}
                        />
                        <Button
                          variant="outline"
                          size="icon"
                          onClick={browseInitrd}
                          disabled={!kernelBoot.enabled || vm.state === 'running'}
                        >
                          <FolderOpen className="h-4 w-4" />
                        </Button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Path to the initial ramdisk (optional but usually required for booting)
                      </p>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="kernel-args">Kernel Command Line Arguments</Label>
                      <Textarea
                        id="kernel-args"
                        placeholder="root=/dev/vda1 console=ttyS0 quiet"
                        value={kernelBoot.kernelArgs || ''}
                        onChange={(e) => setKernelBoot({ ...kernelBoot, kernelArgs: e.target.value })}
                        disabled={!kernelBoot.enabled || vm.state === 'running'}
                        className="font-mono text-sm"
                        rows={3}
                      />
                      <p className="text-xs text-muted-foreground">
                        Kernel parameters passed at boot time (e.g., root device, console settings)
                      </p>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="dtb-path">Device Tree Blob (ARM only)</Label>
                      <div className="flex gap-2">
                        <Input
                          id="dtb-path"
                          placeholder="/path/to/device-tree.dtb (optional, ARM systems only)"
                          value={kernelBoot.dtbPath || ''}
                          onChange={(e) => setKernelBoot({ ...kernelBoot, dtbPath: e.target.value })}
                          disabled={!kernelBoot.enabled || vm.state === 'running'}
                        />
                        <Button
                          variant="outline"
                          size="icon"
                          onClick={browseDtb}
                          disabled={!kernelBoot.enabled || vm.state === 'running'}
                        >
                          <FolderOpen className="h-4 w-4" />
                        </Button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Device tree blob for ARM/AArch64 systems (not needed for x86)
                      </p>
                    </div>
                  </div>

                  {kernelBootError && (
                    <Alert variant="destructive">
                      <AlertCircle className="h-4 w-4" />
                      <AlertDescription>{kernelBootError}</AlertDescription>
                    </Alert>
                  )}

                  {kernelBootSuccess && (
                    <Alert>
                      <CheckCircle className="h-4 w-4 text-green-500" />
                      <AlertDescription>Kernel boot settings updated successfully!</AlertDescription>
                    </Alert>
                  )}
                </>
              )}
            </CardContent>
          </Card>

          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={handleRevertKernelBoot}
              disabled={!hasKernelBootChanges || kernelBootSaving || vm.state === 'running'}
            >
              Revert
            </Button>
            <Button
              onClick={handleApplyKernelBoot}
              disabled={!hasKernelBootChanges || kernelBootSaving || vm.state === 'running'}
            >
              {kernelBootSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Apply
            </Button>
          </div>
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardContent className="p-4">
              <pre className="text-xs font-mono bg-muted p-4 rounded-md overflow-auto max-h-96">
                {`<os>
  <type arch='x86_64' machine='${vm.machine || 'pc-q35-7.2'}'>hvm</type>
  <loader readonly='yes' type='${vm.firmware === 'uefi' ? 'pflash' : 'rom'}'>/usr/share/OVMF/OVMF_CODE.fd</loader>
${kernelBoot.enabled && kernelBoot.kernelPath ? `  <kernel>${kernelBoot.kernelPath}</kernel>` : ''}
${kernelBoot.enabled && kernelBoot.initrdPath ? `  <initrd>${kernelBoot.initrdPath}</initrd>` : ''}
${kernelBoot.enabled && kernelBoot.kernelArgs ? `  <cmdline>${kernelBoot.kernelArgs}</cmdline>` : ''}
${kernelBoot.enabled && kernelBoot.dtbPath ? `  <dtb>${kernelBoot.dtbPath}</dtb>` : ''}
  <boot dev='hd'/>
  ${vm.cdrom ? "<boot dev='cdrom'/>" : ''}
  <boot dev='network'/>
  <bootmenu enable='${vm.bootMenu ? 'yes' : 'no'}'/>
</os>`}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
