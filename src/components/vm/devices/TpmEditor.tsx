import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { ShieldCheck, Trash2 } from 'lucide-react'
import type { VM } from '@/lib/types'

interface TpmEditorProps {
  vm: VM
  compact?: boolean
}

export function TpmEditor({ vm, compact }: TpmEditorProps) {
  const hasTpm = vm.tpm || vm.tpmEnabled

  // Compact mode shows inline summary
  if (compact) {
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <ShieldCheck className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">TPM</span>
            <Badge variant={hasTpm ? 'default' : 'secondary'} className="text-xs">
              {hasTpm ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>
        </div>

        {hasTpm && (
          <div className="flex gap-4 text-xs text-muted-foreground pl-7">
            <span>Model: TPM CRB</span>
            <span>Version: 2.0</span>
          </div>
        )}
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold mb-2 flex items-center gap-2">
            <ShieldCheck className="w-6 h-6" />
            TPM Device
          </h2>
          <p className="text-sm text-muted-foreground">
            Trusted Platform Module for enhanced security
          </p>
        </div>
        <Badge variant={hasTpm ? 'default' : 'secondary'}>
          {hasTpm ? 'Enabled' : 'Disabled'}
        </Badge>
      </div>

      <Tabs defaultValue="details" className="w-full">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="xml">XML</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-6 pt-4">
          {hasTpm ? (
            <>
              <Card>
                <CardHeader>
                  <CardTitle>TPM Configuration</CardTitle>
                  <CardDescription>
                    Virtual TPM device settings
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label htmlFor="model">TPM Model</Label>
                      <Select value="tpm-crb" disabled>
                        <SelectTrigger>
                          <SelectValue placeholder="Select model" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="tpm-crb">TPM CRB (Recommended)</SelectItem>
                          <SelectItem value="tpm-tis">TPM TIS (Legacy)</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-xs text-muted-foreground">
                        CRB (Command Response Buffer) is the modern interface
                      </p>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="version">TPM Version</Label>
                      <Select value="2.0" disabled>
                        <SelectTrigger>
                          <SelectValue placeholder="Select version" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="2.0">TPM 2.0 (Recommended)</SelectItem>
                          <SelectItem value="1.2">TPM 1.2 (Legacy)</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-xs text-muted-foreground">
                        TPM 2.0 is required for Windows 11
                      </p>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="backend">Backend Type</Label>
                    <Select value="emulator" disabled>
                      <SelectTrigger>
                        <SelectValue placeholder="Select backend" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="emulator">Emulator (swtpm)</SelectItem>
                        <SelectItem value="passthrough">Passthrough (Host TPM)</SelectItem>
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">
                      Emulator uses software TPM (swtpm). Passthrough shares the host TPM.
                    </p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Security Information</CardTitle>
                  <CardDescription>
                    TPM capabilities and use cases
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-3">
                    <div className="p-3 bg-muted rounded-lg">
                      <h4 className="font-medium text-sm mb-1">BitLocker / LUKS</h4>
                      <p className="text-xs text-muted-foreground">
                        Full disk encryption with TPM-backed key storage
                      </p>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <h4 className="font-medium text-sm mb-1">Secure Boot</h4>
                      <p className="text-xs text-muted-foreground">
                        TPM can store measurements for verified boot
                      </p>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <h4 className="font-medium text-sm mb-1">Windows 11</h4>
                      <p className="text-xs text-muted-foreground">
                        TPM 2.0 is a system requirement for Windows 11
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </>
          ) : (
            <Card>
              <CardHeader>
                <CardTitle>TPM Not Configured</CardTitle>
                <CardDescription>
                  This VM does not have a TPM device
                </CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-4">
                  TPM (Trusted Platform Module) provides hardware-based security features
                  like disk encryption and secure boot verification.
                </p>
                <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                  <p className="text-xs text-yellow-600 dark:text-yellow-400">
                    <strong>Note:</strong> Adding a TPM requires:
                    <ul className="list-disc list-inside mt-1">
                      <li>UEFI firmware (not BIOS)</li>
                      <li>swtpm package installed on host</li>
                      <li>VM must be stopped to add TPM</li>
                    </ul>
                  </p>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="xml" className="pt-4">
          <Card>
            <CardHeader>
              <CardTitle>TPM XML Configuration</CardTitle>
              <CardDescription>
                Raw libvirt XML for this TPM device
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto">
                {hasTpm ?
`<tpm model='tpm-crb'>
  <backend type='emulator' version='2.0'/>
</tpm>` :
'<!-- No TPM device configured -->'}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <div className="flex justify-between">
        {hasTpm && (
          <Button variant="destructive" size="sm" className="gap-1.5" disabled>
            <Trash2 className="w-4 h-4" />
            Remove TPM
          </Button>
        )}
        <div className="flex gap-2 ml-auto">
          <Button variant="outline" disabled>Revert</Button>
          <Button disabled>Apply</Button>
        </div>
      </div>
    </div>
  )
}
