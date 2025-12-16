import { useState, useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { AlertTriangle, Info } from 'lucide-react'
import { toast } from 'sonner'
import type { SavedConnection } from '@/lib/types'

interface MigrationDialogProps {
  vmId: string
  vmName: string
  isRunning: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
  trigger?: React.ReactNode
}

export function MigrationDialog({ vmId, vmName, isRunning, open: controlledOpen, onOpenChange, trigger }: MigrationDialogProps) {
  const [internalOpen, setInternalOpen] = useState(false)
  const open = controlledOpen !== undefined ? controlledOpen : internalOpen
  const setOpen = (value: boolean) => {
    if (onOpenChange) {
      onOpenChange(value)
    } else {
      setInternalOpen(value)
    }
  }
  const [destUri, setDestUri] = useState('')
  const [useCustomUri, setUseCustomUri] = useState(false)
  const [selectedConnection, setSelectedConnection] = useState<string>('')
  const [liveMigration, setLiveMigration] = useState(isRunning)
  const [unsafeMigration, setUnsafeMigration] = useState(false)
  const queryClient = useQueryClient()

  // Fetch saved connections for destination selection
  const { data: connections = [] } = useQuery({
    queryKey: ['savedConnections'],
    queryFn: () => api.getSavedConnections(),
    enabled: open,
  })

  // Fetch migration info for the VM
  const { data: migrationInfo } = useQuery({
    queryKey: ['migrationInfo', vmId],
    queryFn: () => api.getMigrationInfo(vmId),
    enabled: open,
  })

  // Update live migration default when running state changes
  useEffect(() => {
    setLiveMigration(isRunning && (migrationInfo?.canLiveMigrate ?? true))
  }, [isRunning, migrationInfo?.canLiveMigrate])

  const migrateMutation = useMutation({
    mutationFn: () => {
      const uri = useCustomUri ? destUri : getConnectionUri(selectedConnection)
      return api.migrateVm(vmId, uri, liveMigration, unsafeMigration)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success('Migration Started', {
        description: `${vmName} is being migrated to the destination host`,
      })
      setOpen(false)
      resetForm()
    },
    onError: (error: Error) => {
      toast.error('Migration Failed', {
        description: error.message,
      })
    },
  })

  const getConnectionUri = (connectionId: string): string => {
    const conn = connections.find((c: SavedConnection) => c.id === connectionId)
    if (!conn) return ''

    // Build libvirt connection URI from saved connection
    switch (conn.connectionType) {
      case 'ssh':
        return `qemu+ssh://${conn.username || 'root'}@${conn.host}:${conn.sshPort || 22}/system`
      case 'tcp':
        return `qemu+tcp://${conn.host}:${conn.tlsPort || 16509}/system`
      case 'tls':
        return `qemu+tls://${conn.host}:${conn.tlsPort || 16514}/system`
      default:
        return `qemu:///system`
    }
  }

  const resetForm = () => {
    setDestUri('')
    setSelectedConnection('')
    setUseCustomUri(false)
    setLiveMigration(isRunning)
    setUnsafeMigration(false)
  }

  const handleMigrate = () => {
    const uri = useCustomUri ? destUri : getConnectionUri(selectedConnection)
    if (!uri.trim()) {
      toast.error('Destination Required', {
        description: 'Please select a destination or enter a custom URI',
      })
      return
    }
    migrateMutation.mutate()
  }

  const canMigrate = useCustomUri ? destUri.trim() : selectedConnection

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      {trigger && (
        <DialogTrigger asChild>
          {trigger}
        </DialogTrigger>
      )}
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Migrate Virtual Machine</DialogTitle>
          <DialogDescription>
            Migrate "{vmName}" to another host. {isRunning ? 'The VM is currently running.' : 'The VM is currently stopped.'}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* Migration Info */}
          {migrationInfo && (
            <div className="rounded-lg border bg-muted/50 p-3 text-sm">
              <div className="flex items-center gap-2 mb-2">
                <Info className="h-4 w-4 text-muted-foreground" />
                <span className="font-medium">Migration Details</span>
              </div>
              <div className="grid grid-cols-2 gap-2 text-muted-foreground">
                <span>Memory:</span>
                <span>{migrationInfo.memoryMb} MB</span>
                <span>Disk Size:</span>
                <span>{migrationInfo.diskSizeGb.toFixed(1)} GB</span>
                {isRunning && (
                  <>
                    <span>Est. Downtime:</span>
                    <span>{migrationInfo.estimatedDowntimeSeconds}s</span>
                  </>
                )}
              </div>
            </div>
          )}

          {/* Destination Selection */}
          <div className="grid gap-2">
            <div className="flex items-center justify-between">
              <Label>Destination</Label>
              <div className="flex items-center gap-2">
                <Label htmlFor="custom-uri" className="text-sm text-muted-foreground">
                  Custom URI
                </Label>
                <Switch
                  id="custom-uri"
                  checked={useCustomUri}
                  onCheckedChange={setUseCustomUri}
                />
              </div>
            </div>

            {useCustomUri ? (
              <Input
                value={destUri}
                onChange={(e) => setDestUri(e.target.value)}
                placeholder="qemu+ssh://user@host/system"
                disabled={migrateMutation.isPending}
              />
            ) : (
              <Select
                value={selectedConnection}
                onValueChange={setSelectedConnection}
                disabled={migrateMutation.isPending}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select destination host" />
                </SelectTrigger>
                <SelectContent>
                  {connections
                    .filter((c: SavedConnection) => c.connectionType !== 'local')
                    .map((conn: SavedConnection) => (
                      <SelectItem key={conn.id} value={conn.id}>
                        {conn.name} ({conn.host})
                      </SelectItem>
                    ))}
                  {connections.filter((c: SavedConnection) => c.connectionType !== 'local').length === 0 && (
                    <SelectItem value="_none" disabled>
                      No remote connections configured
                    </SelectItem>
                  )}
                </SelectContent>
              </Select>
            )}
            <p className="text-sm text-muted-foreground">
              {useCustomUri
                ? 'Enter a libvirt connection URI (e.g., qemu+ssh://user@host/system)'
                : 'Select from saved connections or use a custom URI'}
            </p>
          </div>

          {/* Migration Options */}
          <div className="space-y-3">
            <Label>Migration Options</Label>

            <div className="flex items-center justify-between rounded-lg border p-3">
              <div className="space-y-0.5">
                <Label htmlFor="live-migration" className="text-sm font-medium">
                  Live Migration
                </Label>
                <p className="text-xs text-muted-foreground">
                  Migrate while VM continues running (minimal downtime)
                </p>
              </div>
              <Switch
                id="live-migration"
                checked={liveMigration}
                onCheckedChange={setLiveMigration}
                disabled={!isRunning || migrateMutation.isPending}
              />
            </div>

            <div className="flex items-center justify-between rounded-lg border p-3">
              <div className="space-y-0.5">
                <div className="flex items-center gap-2">
                  <Label htmlFor="unsafe-migration" className="text-sm font-medium">
                    Allow Unsafe Migration
                  </Label>
                  <AlertTriangle className="h-4 w-4 text-yellow-500" />
                </div>
                <p className="text-xs text-muted-foreground">
                  Skip safety checks (may cause issues if disk storage is not shared)
                </p>
              </div>
              <Switch
                id="unsafe-migration"
                checked={unsafeMigration}
                onCheckedChange={setUnsafeMigration}
                disabled={migrateMutation.isPending}
              />
            </div>
          </div>

          {/* Warning for non-shared storage */}
          {unsafeMigration && (
            <div className="flex items-start gap-2 rounded-lg border border-yellow-500/50 bg-yellow-500/10 p-3">
              <AlertTriangle className="h-5 w-5 text-yellow-500 mt-0.5" />
              <div className="text-sm">
                <p className="font-medium text-yellow-600 dark:text-yellow-400">
                  Warning: Unsafe Migration
                </p>
                <p className="text-muted-foreground mt-1">
                  If the VM uses local storage, migration may fail or corrupt data.
                  Ensure storage is shared between source and destination.
                </p>
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => {
              setOpen(false)
              resetForm()
            }}
            disabled={migrateMutation.isPending}
          >
            Cancel
          </Button>
          <Button
            onClick={handleMigrate}
            disabled={migrateMutation.isPending || !canMigrate}
          >
            {migrateMutation.isPending ? 'Migrating...' : 'Migrate VM'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
