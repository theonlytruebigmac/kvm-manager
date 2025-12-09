import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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
import { ArrowRight, Plus, Trash2, Network } from 'lucide-react'

interface PortForwardRule {
  hostPort: number
  guestIp: string
  guestPort: number
  protocol: string
}

export function PortForwardingManager() {
  const [rules, setRules] = useState<PortForwardRule[]>([])
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [hostPort, setHostPort] = useState('')
  const [guestIp, setGuestIp] = useState('')
  const [guestPort, setGuestPort] = useState('')
  const [protocol, setProtocol] = useState<'tcp' | 'udp'>('tcp')
  const [deleteRule, setDeleteRule] = useState<PortForwardRule | null>(null)

  // Add port forward mutation
  const addMutation = useMutation({
    mutationFn: ({ hostPort, guestIp, guestPort, protocol }: PortForwardRule) =>
      api.addPortForward(hostPort, guestIp, guestPort, protocol),
    onSuccess: (_data, variables) => {
      setRules([...rules, variables])
      toast.success(`Port forwarding rule added: ${variables.hostPort} → ${variables.guestIp}:${variables.guestPort}`)
      setShowAddDialog(false)
      setHostPort('')
      setGuestIp('')
      setGuestPort('')
      setProtocol('tcp')
    },
    onError: (error) => {
      toast.error(`Failed to add port forwarding rule: ${error}`)
    },
  })

  // Remove port forward mutation
  const removeMutation = useMutation({
    mutationFn: ({ hostPort, guestIp, guestPort, protocol }: PortForwardRule) =>
      api.removePortForward(hostPort, guestIp, guestPort, protocol),
    onSuccess: (_data, variables) => {
      setRules(rules.filter(r =>
        !(r.hostPort === variables.hostPort &&
          r.guestIp === variables.guestIp &&
          r.guestPort === variables.guestPort &&
          r.protocol === variables.protocol)
      ))
      toast.success(`Port forwarding rule removed`)
      setDeleteRule(null)
    },
    onError: (error) => {
      toast.error(`Failed to remove port forwarding rule: ${error}`)
    },
  })

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Network className="h-5 w-5" />
              Port Forwarding Rules
            </CardTitle>
            <CardDescription>
              Forward host ports to guest VM ports using iptables NAT
            </CardDescription>
          </div>
          <Button size="sm" onClick={() => setShowAddDialog(true)}>
            <Plus className="mr-2 h-4 w-4" />
            Add Rule
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {rules.length === 0 ? (
          <div className="text-center py-8">
            <Network className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <p className="text-sm text-muted-foreground">No port forwarding rules configured</p>
            <p className="text-xs text-muted-foreground mt-1">
              Add rules to forward host ports to VM guest ports
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {rules.map((rule, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors group"
              >
                <div className="flex items-center gap-4 flex-1">
                  <div className="text-right min-w-20">
                    <p className="font-medium">{rule.protocol.toUpperCase()}</p>
                    <p className="text-sm text-muted-foreground">:{rule.hostPort}</p>
                  </div>
                  <ArrowRight className="h-4 w-4 text-muted-foreground" />
                  <div className="flex-1">
                    <p className="font-medium">{rule.guestIp}</p>
                    <p className="text-sm text-muted-foreground">Port {rule.guestPort}</p>
                  </div>
                </div>
                <AlertDialog
                  open={deleteRule?.hostPort === rule.hostPort &&
                        deleteRule?.guestIp === rule.guestIp &&
                        deleteRule?.guestPort === rule.guestPort}
                  onOpenChange={(open) => !open && setDeleteRule(null)}
                >
                  <AlertDialogTrigger asChild>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="opacity-0 group-hover:opacity-100 transition-opacity"
                      onClick={() => setDeleteRule(rule)}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Remove Port Forwarding Rule?</AlertDialogTitle>
                      <AlertDialogDescription>
                        Are you sure you want to remove the port forwarding rule for host port {rule.hostPort} → {rule.guestIp}:{rule.guestPort}?
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={() => removeMutation.mutate(rule)}
                        className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                      >
                        Remove Rule
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </div>
            ))}
          </div>
        )}
      </CardContent>

      {/* Add Port Forward Dialog */}
      <AlertDialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Add Port Forwarding Rule</AlertDialogTitle>
            <AlertDialogDescription>
              Forward a host port to a VM guest port. Requires root/sudo privileges.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-4 py-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="host-port">Host Port</Label>
                <Input
                  id="host-port"
                  type="number"
                  min="1"
                  max="65535"
                  value={hostPort}
                  onChange={(e) => setHostPort(e.target.value)}
                  placeholder="8080"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="protocol">Protocol</Label>
                <Select value={protocol} onValueChange={(v) => setProtocol(v as 'tcp' | 'udp')}>
                  <SelectTrigger id="protocol">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="tcp">TCP</SelectItem>
                    <SelectItem value="udp">UDP</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="guest-ip">Guest VM IP Address</Label>
              <Input
                id="guest-ip"
                value={guestIp}
                onChange={(e) => setGuestIp(e.target.value)}
                placeholder="192.168.122.100"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="guest-port">Guest Port</Label>
              <Input
                id="guest-port"
                type="number"
                min="1"
                max="65535"
                value={guestPort}
                onChange={(e) => setGuestPort(e.target.value)}
                placeholder="80"
              />
            </div>
            <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-900 rounded-md p-3">
              <p className="text-xs text-amber-800 dark:text-amber-200">
                ⚠️ This feature requires sudo/root privileges to modify iptables.
                Make sure the application has the necessary permissions.
              </p>
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const hp = parseInt(hostPort)
                const gp = parseInt(guestPort)
                if (hp && guestIp && gp) {
                  addMutation.mutate({ hostPort: hp, guestIp, guestPort: gp, protocol })
                }
              }}
              disabled={
                !hostPort || !guestIp || !guestPort ||
                addMutation.isPending
              }
            >
              {addMutation.isPending ? 'Adding...' : 'Add Rule'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  )
}
