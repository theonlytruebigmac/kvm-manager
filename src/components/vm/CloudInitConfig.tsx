import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Cloud,
  User,
  Key,
  Package,
  Terminal,
  Network,
  Plus,
  X,
  Eye,
  EyeOff,
  Copy,
  Check,
} from 'lucide-react'
import { toast } from 'sonner'
import type { CloudInitConfig as CloudInitConfigType } from '@/lib/types'

interface CloudInitConfigProps {
  config: CloudInitConfigType
  onChange: (config: CloudInitConfigType) => void
  disabled?: boolean
}

const defaultConfig: CloudInitConfigType = {
  enabled: false,
  username: '',
  password: '',
  sshAuthorizedKeys: [],
  hostname: '',
  packages: [],
  runcmd: [],
  customUserData: '',
  autoEject: true,
}

export function CloudInitConfig({ config, onChange, disabled = false }: CloudInitConfigProps) {
  const [showPassword, setShowPassword] = useState(false)
  const [newSshKey, setNewSshKey] = useState('')
  const [newPackage, setNewPackage] = useState('')
  const [newCommand, setNewCommand] = useState('')
  const [copied, setCopied] = useState(false)

  const currentConfig = { ...defaultConfig, ...config }

  const updateConfig = (updates: Partial<CloudInitConfigType>) => {
    onChange({ ...currentConfig, ...updates })
  }

  const addSshKey = () => {
    if (!newSshKey.trim()) return
    if (!newSshKey.startsWith('ssh-')) {
      toast.error('SSH key should start with ssh-rsa, ssh-ed25519, etc.')
      return
    }
    updateConfig({
      sshAuthorizedKeys: [...currentConfig.sshAuthorizedKeys, newSshKey.trim()]
    })
    setNewSshKey('')
  }

  const removeSshKey = (index: number) => {
    updateConfig({
      sshAuthorizedKeys: currentConfig.sshAuthorizedKeys.filter((_, i) => i !== index)
    })
  }

  const addPackage = () => {
    if (!newPackage.trim()) return
    updateConfig({
      packages: [...currentConfig.packages, newPackage.trim()]
    })
    setNewPackage('')
  }

  const removePackage = (index: number) => {
    updateConfig({
      packages: currentConfig.packages.filter((_, i) => i !== index)
    })
  }

  const addCommand = () => {
    if (!newCommand.trim()) return
    updateConfig({
      runcmd: [...currentConfig.runcmd, newCommand.trim()]
    })
    setNewCommand('')
  }

  const removeCommand = (index: number) => {
    updateConfig({
      runcmd: currentConfig.runcmd.filter((_, i) => i !== index)
    })
  }

  const generatePreview = () => {
    let yaml = '#cloud-config\n'

    if (currentConfig.hostname) {
      yaml += `hostname: ${currentConfig.hostname}\n`
    }

    if (currentConfig.username) {
      yaml += `\nusers:\n`
      yaml += `  - name: ${currentConfig.username}\n`
      yaml += `    sudo: ALL=(ALL) NOPASSWD:ALL\n`
      yaml += `    groups: users, admin, wheel, sudo\n`
      yaml += `    shell: /bin/bash\n`
      if (currentConfig.password) {
        yaml += `    lock_passwd: false\n`
      }
      if (currentConfig.sshAuthorizedKeys.length > 0) {
        yaml += `    ssh_authorized_keys:\n`
        currentConfig.sshAuthorizedKeys.forEach(key => {
          yaml += `      - ${key}\n`
        })
      }
    }

    if (currentConfig.password && currentConfig.username) {
      yaml += `\nchpasswd:\n`
      yaml += `  list: |\n`
      yaml += `    ${currentConfig.username}:${currentConfig.password}\n`
      yaml += `  expire: false\n`
    }

    if (currentConfig.packages.length > 0) {
      yaml += `\npackages:\n`
      currentConfig.packages.forEach(pkg => {
        yaml += `  - ${pkg}\n`
      })
    }

    if (currentConfig.runcmd.length > 0) {
      yaml += `\nruncmd:\n`
      currentConfig.runcmd.forEach(cmd => {
        yaml += `  - ${cmd}\n`
      })
    }

    return yaml
  }

  const copyPreview = async () => {
    try {
      await navigator.clipboard.writeText(generatePreview())
      setCopied(true)
      toast.success('Cloud-config copied to clipboard')
      setTimeout(() => setCopied(false), 2000)
    } catch {
      toast.error('Failed to copy')
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Cloud className="w-5 h-5 text-blue-500" />
          <div>
            <h3 className="font-medium">Cloud-Init Configuration</h3>
            <p className="text-sm text-muted-foreground">
              Automatic VM provisioning on first boot
            </p>
          </div>
        </div>
        <Switch
          checked={currentConfig.enabled}
          onCheckedChange={(enabled) => updateConfig({ enabled })}
          disabled={disabled}
        />
      </div>

      {currentConfig.enabled && (
        <Tabs defaultValue="basic" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="basic">Basic</TabsTrigger>
            <TabsTrigger value="packages">Packages & Commands</TabsTrigger>
            <TabsTrigger value="preview">Preview</TabsTrigger>
          </TabsList>

          <TabsContent value="basic" className="space-y-4 pt-4">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <User className="w-4 h-4" />
                  User Account
                </CardTitle>
                <CardDescription>
                  Create a user account on first boot
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label htmlFor="hostname">Hostname</Label>
                    <Input
                      id="hostname"
                      placeholder="my-vm"
                      value={currentConfig.hostname || ''}
                      onChange={(e) => updateConfig({ hostname: e.target.value })}
                      disabled={disabled}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="username">Username</Label>
                    <Input
                      id="username"
                      placeholder="ubuntu"
                      value={currentConfig.username || ''}
                      onChange={(e) => updateConfig({ username: e.target.value })}
                      disabled={disabled}
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="password">Password</Label>
                  <div className="flex gap-2">
                    <div className="relative flex-1">
                      <Input
                        id="password"
                        type={showPassword ? 'text' : 'password'}
                        placeholder="Enter password"
                        value={currentConfig.password || ''}
                        onChange={(e) => updateConfig({ password: e.target.value })}
                        disabled={disabled}
                      />
                    </div>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => setShowPassword(!showPassword)}
                      type="button"
                    >
                      {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </Button>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Password will be set via cloud-init on first boot
                  </p>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Key className="w-4 h-4" />
                  SSH Keys
                </CardTitle>
                <CardDescription>
                  Public keys for passwordless SSH access
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Textarea
                    placeholder="ssh-ed25519 AAAA... user@host"
                    value={newSshKey}
                    onChange={(e) => setNewSshKey(e.target.value)}
                    disabled={disabled}
                    className="min-h-[60px] font-mono text-xs"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={addSshKey}
                    disabled={disabled || !newSshKey.trim()}
                    className="shrink-0"
                  >
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>

                {currentConfig.sshAuthorizedKeys.length > 0 && (
                  <div className="space-y-2">
                    {currentConfig.sshAuthorizedKeys.map((key, index) => (
                      <div
                        key={index}
                        className="flex items-center gap-2 p-2 bg-muted rounded-md"
                      >
                        <Key className="w-4 h-4 text-muted-foreground shrink-0" />
                        <code className="flex-1 text-xs font-mono truncate">
                          {key.substring(0, 50)}...
                        </code>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-6 w-6"
                          onClick={() => removeSshKey(index)}
                          disabled={disabled}
                        >
                          <X className="w-3 h-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Network className="w-4 h-4" />
                  Options
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center justify-between">
                  <div>
                    <Label>Auto-eject ISO after first boot</Label>
                    <p className="text-xs text-muted-foreground">
                      Automatically remove cloud-init ISO after provisioning
                    </p>
                  </div>
                  <Switch
                    checked={currentConfig.autoEject}
                    onCheckedChange={(autoEject) => updateConfig({ autoEject })}
                    disabled={disabled}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="packages" className="space-y-4 pt-4">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Package className="w-4 h-4" />
                  Packages
                </CardTitle>
                <CardDescription>
                  Packages to install on first boot
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Input
                    placeholder="Package name (e.g., nginx, docker.io)"
                    value={newPackage}
                    onChange={(e) => setNewPackage(e.target.value)}
                    disabled={disabled}
                    onKeyDown={(e) => e.key === 'Enter' && addPackage()}
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={addPackage}
                    disabled={disabled || !newPackage.trim()}
                  >
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>

                {currentConfig.packages.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {currentConfig.packages.map((pkg, index) => (
                      <Badge
                        key={index}
                        variant="secondary"
                        className="gap-1 pr-1"
                      >
                        {pkg}
                        <button
                          onClick={() => removePackage(index)}
                          disabled={disabled}
                          className="ml-1 hover:text-destructive"
                        >
                          <X className="w-3 h-3" />
                        </button>
                      </Badge>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Terminal className="w-4 h-4" />
                  Run Commands
                </CardTitle>
                <CardDescription>
                  Shell commands to execute on first boot
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Input
                    placeholder="e.g., systemctl enable nginx"
                    value={newCommand}
                    onChange={(e) => setNewCommand(e.target.value)}
                    disabled={disabled}
                    onKeyDown={(e) => e.key === 'Enter' && addCommand()}
                    className="font-mono text-sm"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={addCommand}
                    disabled={disabled || !newCommand.trim()}
                  >
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>

                {currentConfig.runcmd.length > 0 && (
                  <div className="space-y-2">
                    {currentConfig.runcmd.map((cmd, index) => (
                      <div
                        key={index}
                        className="flex items-center gap-2 p-2 bg-muted rounded-md"
                      >
                        <code className="flex-1 text-xs font-mono">{cmd}</code>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-6 w-6"
                          onClick={() => removeCommand(index)}
                          disabled={disabled}
                        >
                          <X className="w-3 h-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="preview" className="pt-4">
            <Card>
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base">Cloud-Config Preview</CardTitle>
                  <Button variant="outline" size="sm" onClick={copyPreview}>
                    {copied ? <Check className="w-4 h-4 mr-1" /> : <Copy className="w-4 h-4 mr-1" />}
                    {copied ? 'Copied!' : 'Copy'}
                  </Button>
                </div>
                <CardDescription>
                  This YAML will be used for the cloud-init ISO
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre className="bg-muted p-4 rounded-lg text-xs font-mono overflow-auto max-h-96 whitespace-pre-wrap">
                  {generatePreview()}
                </pre>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      )}
    </div>
  )
}

export { defaultConfig as defaultCloudInitConfig }
