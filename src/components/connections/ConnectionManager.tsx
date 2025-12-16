import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { SavedConnection, ConnectionType } from '@/lib/types'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Server,
  Plus,
  Trash2,
  Edit,
  Check,
  X,
  Loader2,
  Wifi,
  WifiOff,
  Globe,
  Lock,
  Terminal,
} from 'lucide-react'

interface ConnectionManagerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ConnectionManager({ open, onOpenChange }: ConnectionManagerProps) {
  const queryClient = useQueryClient()
  const [editingConnection, setEditingConnection] = useState<SavedConnection | null>(null)
  const [isAddingNew, setIsAddingNew] = useState(false)

  // Fetch saved connections
  const { data: connections = [] } = useQuery({
    queryKey: ['saved-connections'],
    queryFn: api.getSavedConnections,
    enabled: open,
  })

  // Fetch active connection
  const { data: activeConnection } = useQuery({
    queryKey: ['active-connection'],
    queryFn: api.getActiveConnection,
    enabled: open,
  })

  // Connect mutation
  const connectMutation = useMutation({
    mutationFn: (connectionId: string) => api.connectTo(connectionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['active-connection'] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['host-info'] })
      toast.success('Connected successfully')
    },
    onError: (error) => {
      toast.error(`Connection failed: ${error}`)
    },
  })

  // Disconnect mutation
  const disconnectMutation = useMutation({
    mutationFn: (connectionId: string) => api.disconnectFrom(connectionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['active-connection'] })
      toast.success('Disconnected')
    },
    onError: (error) => {
      toast.error(`Disconnect failed: ${error}`)
    },
  })

  // Remove connection mutation
  const removeMutation = useMutation({
    mutationFn: (connectionId: string) => api.removeConnection(connectionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['saved-connections'] })
      toast.success('Connection removed')
    },
    onError: (error) => {
      toast.error(`Failed to remove: ${error}`)
    },
  })

  const getConnectionIcon = (type: ConnectionType) => {
    switch (type) {
      case 'local':
        return <Server className="h-4 w-4" />
      case 'ssh':
        return <Terminal className="h-4 w-4" />
      case 'tls':
        return <Lock className="h-4 w-4" />
      case 'tcp':
        return <Globe className="h-4 w-4" />
    }
  }

  const getConnectionDescription = (conn: SavedConnection) => {
    switch (conn.connectionType) {
      case 'local':
        return 'Local hypervisor'
      case 'ssh':
        return `${conn.username || 'root'}@${conn.host || 'localhost'}${conn.sshPort && conn.sshPort !== 22 ? `:${conn.sshPort}` : ''}`
      case 'tls':
        return `${conn.host || 'localhost'}${conn.tlsPort && conn.tlsPort !== 16514 ? `:${conn.tlsPort}` : ''} (TLS)`
      case 'tcp':
        return `${conn.host || 'localhost'} (TCP)`
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Connection Manager</DialogTitle>
          <DialogDescription>
            Manage connections to local and remote hypervisors
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Connection List */}
          <ScrollArea className="h-[300px] border rounded-lg p-2">
            {connections.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                No connections configured
              </div>
            ) : (
              <div className="space-y-2">
                {connections.map((conn) => {
                  const isActive = activeConnection?.id === conn.id
                  const isConnecting = connectMutation.isPending && connectMutation.variables === conn.id

                  return (
                    <div
                      key={conn.id}
                      className={`flex items-center gap-3 p-3 rounded-lg border ${
                        isActive ? 'bg-green-500/10 border-green-500/50' : 'bg-muted/50'
                      }`}
                    >
                      {/* Icon */}
                      <div className={`p-2 rounded-full ${isActive ? 'bg-green-500/20' : 'bg-muted'}`}>
                        {getConnectionIcon(conn.connectionType)}
                      </div>

                      {/* Connection Info */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium truncate">{conn.name}</span>
                          {isActive && (
                            <span className="text-xs bg-green-500 text-white px-2 py-0.5 rounded-full">
                              Active
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground truncate">
                          {getConnectionDescription(conn)}
                        </p>
                      </div>

                      {/* Actions */}
                      <div className="flex items-center gap-1">
                        {isActive ? (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => disconnectMutation.mutate(conn.id)}
                            disabled={disconnectMutation.isPending || conn.id === 'local'}
                          >
                            <WifiOff className="h-4 w-4 mr-1" />
                            Disconnect
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => connectMutation.mutate(conn.id)}
                            disabled={isConnecting}
                          >
                            {isConnecting ? (
                              <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                            ) : (
                              <Wifi className="h-4 w-4 mr-1" />
                            )}
                            Connect
                          </Button>
                        )}

                        {conn.id !== 'local' && (
                          <>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => {
                                setEditingConnection(conn)
                                setIsAddingNew(false)
                              }}
                            >
                              <Edit className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => removeMutation.mutate(conn.id)}
                              disabled={isActive}
                            >
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          </>
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </ScrollArea>

          {/* Add New Connection Button */}
          <Button
            variant="outline"
            className="w-full"
            onClick={() => {
              setIsAddingNew(true)
              setEditingConnection(null)
            }}
          >
            <Plus className="h-4 w-4 mr-2" />
            Add Connection
          </Button>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>

      {/* Edit/Add Connection Dialog */}
      {(editingConnection || isAddingNew) && (
        <ConnectionEditDialog
          connection={editingConnection}
          onClose={() => {
            setEditingConnection(null)
            setIsAddingNew(false)
          }}
          onSave={() => {
            queryClient.invalidateQueries({ queryKey: ['saved-connections'] })
            setEditingConnection(null)
            setIsAddingNew(false)
          }}
        />
      )}
    </Dialog>
  )
}

interface ConnectionEditDialogProps {
  connection: SavedConnection | null
  onClose: () => void
  onSave: () => void
}

function ConnectionEditDialog({ connection, onClose, onSave }: ConnectionEditDialogProps) {
  const isEditing = !!connection

  const [name, setName] = useState(connection?.name || '')
  const [connectionType, setConnectionType] = useState<ConnectionType>(connection?.connectionType || 'ssh')
  const [host, setHost] = useState(connection?.host || '')
  const [username, setUsername] = useState(connection?.username || 'root')
  const [sshPort, setSshPort] = useState(connection?.sshPort || 22)
  const [tlsPort, setTlsPort] = useState(connection?.tlsPort || 16514)
  const [autoConnect, setAutoConnect] = useState(connection?.autoConnect || false)
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null)
  const [isTesting, setIsTesting] = useState(false)

  const addMutation = useMutation({
    mutationFn: () => api.addConnection(
      name,
      connectionType,
      connectionType !== 'local' ? host : undefined,
      connectionType === 'ssh' ? username : undefined,
      connectionType === 'ssh' ? sshPort : undefined,
      connectionType === 'tls' ? tlsPort : undefined,
      autoConnect
    ),
    onSuccess: () => {
      toast.success('Connection added')
      onSave()
    },
    onError: (error) => {
      toast.error(`Failed to add connection: ${error}`)
    },
  })

  const updateMutation = useMutation({
    mutationFn: () => api.updateConnection(
      connection!.id,
      name,
      connectionType,
      connectionType !== 'local' ? host : undefined,
      connectionType === 'ssh' ? username : undefined,
      connectionType === 'ssh' ? sshPort : undefined,
      connectionType === 'tls' ? tlsPort : undefined,
      autoConnect
    ),
    onSuccess: () => {
      toast.success('Connection updated')
      onSave()
    },
    onError: (error) => {
      toast.error(`Failed to update connection: ${error}`)
    },
  })

  const handleTest = async () => {
    setIsTesting(true)
    setTestResult(null)
    try {
      const result = await api.testConnection(
        connectionType,
        connectionType !== 'local' ? host : undefined,
        connectionType === 'ssh' ? username : undefined,
        connectionType === 'ssh' ? sshPort : undefined,
        connectionType === 'tls' ? tlsPort : undefined
      )
      setTestResult({ success: true, message: result })
    } catch (error) {
      setTestResult({ success: false, message: String(error) })
    } finally {
      setIsTesting(false)
    }
  }

  const handleSave = () => {
    if (!name.trim()) {
      toast.error('Connection name is required')
      return
    }
    if (connectionType !== 'local' && !host.trim()) {
      toast.error('Host is required for remote connections')
      return
    }

    if (isEditing) {
      updateMutation.mutate()
    } else {
      addMutation.mutate()
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? 'Edit Connection' : 'Add Connection'}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* Connection Name */}
          <div className="space-y-2">
            <Label>Name</Label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Remote Server"
            />
          </div>

          {/* Connection Type */}
          <div className="space-y-2">
            <Label>Connection Type</Label>
            <Select value={connectionType} onValueChange={(v) => setConnectionType(v as ConnectionType)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="local">Local (qemu:///system)</SelectItem>
                <SelectItem value="ssh">SSH (qemu+ssh://)</SelectItem>
                <SelectItem value="tls">TLS (qemu+tls://)</SelectItem>
                <SelectItem value="tcp">TCP (qemu+tcp://)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Host (for remote connections) */}
          {connectionType !== 'local' && (
            <div className="space-y-2">
              <Label>Host</Label>
              <Input
                value={host}
                onChange={(e) => setHost(e.target.value)}
                placeholder="192.168.1.100 or hostname"
              />
            </div>
          )}

          {/* SSH-specific fields */}
          {connectionType === 'ssh' && (
            <>
              <div className="space-y-2">
                <Label>Username</Label>
                <Input
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="root"
                />
              </div>
              <div className="space-y-2">
                <Label>SSH Port</Label>
                <Input
                  type="number"
                  value={sshPort}
                  onChange={(e) => setSshPort(parseInt(e.target.value) || 22)}
                />
              </div>
            </>
          )}

          {/* TLS-specific fields */}
          {connectionType === 'tls' && (
            <div className="space-y-2">
              <Label>TLS Port</Label>
              <Input
                type="number"
                value={tlsPort}
                onChange={(e) => setTlsPort(parseInt(e.target.value) || 16514)}
              />
            </div>
          )}

          {/* Auto-connect */}
          <div className="flex items-center justify-between">
            <Label htmlFor="auto-connect">Auto-connect on startup</Label>
            <Switch
              id="auto-connect"
              checked={autoConnect}
              onCheckedChange={setAutoConnect}
            />
          </div>

          {/* Test Connection Result */}
          {testResult && (
            <div className={`p-3 rounded-lg text-sm ${testResult.success ? 'bg-green-500/10 text-green-600' : 'bg-red-500/10 text-red-600'}`}>
              {testResult.success ? (
                <div className="flex items-center gap-2">
                  <Check className="h-4 w-4" />
                  {testResult.message}
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <X className="h-4 w-4" />
                  {testResult.message}
                </div>
              )}
            </div>
          )}
        </div>

        <DialogFooter className="flex gap-2">
          <Button variant="outline" onClick={handleTest} disabled={isTesting}>
            {isTesting ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : null}
            Test Connection
          </Button>
          <div className="flex-1" />
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={addMutation.isPending || updateMutation.isPending}>
            {(addMutation.isPending || updateMutation.isPending) && (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            )}
            {isEditing ? 'Save' : 'Add'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
