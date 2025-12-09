import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { ResourceAlert, CreateAlertRequest, ThresholdType, AlertSeverity, VM } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { Bell, Plus, Trash2, Cpu, HardDrive, Network as NetworkIcon, Activity, PlayCircle, PauseCircle, AlertTriangle } from 'lucide-react'

interface AlertManagerProps {
  vmId?: string // Optional: show alerts for specific VM
}

export function AlertManager({ vmId }: AlertManagerProps) {
  const queryClient = useQueryClient()
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [alertForm, setAlertForm] = useState<CreateAlertRequest>({
    name: '',
    vmId: vmId || '',
    thresholdType: 'cpu',
    thresholdValue: 80,
    severity: 'warning',
    consecutiveChecks: 3,
  })

  // Query for VMs (to populate dropdown)
  const { data: vms } = useQuery<VM[]>({
    queryKey: ['vms'],
    queryFn: () => api.getVms(),
  })

  // Query for alerts
  const { data: alerts, isLoading } = useQuery<ResourceAlert[]>({
    queryKey: vmId ? ['alerts', vmId] : ['alerts'],
    queryFn: () => vmId ? api.getVmAlerts(vmId) : api.listAlerts(),
  })

  // Create alert mutation
  const createMutation = useMutation({
    mutationFn: (request: CreateAlertRequest) => api.createAlert(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['alerts'] })
      toast.success('Alert created successfully')
      setIsCreateDialogOpen(false)
      resetForm()
    },
    onError: (error: Error) => {
      toast.error(`Failed to create alert: ${error.message}`)
    },
  })

  // Update alert status mutation
  const updateStatusMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      api.updateAlertStatus(id, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['alerts'] })
      toast.success('Alert status updated')
    },
    onError: (error: Error) => {
      toast.error(`Failed to update alert: ${error.message}`)
    },
  })

  // Delete alert mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteAlert(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['alerts'] })
      toast.success('Alert deleted successfully')
    },
    onError: (error: Error) => {
      toast.error(`Failed to delete alert: ${error.message}`)
    },
  })

  const resetForm = () => {
    setAlertForm({
      name: '',
      vmId: vmId || '',
      thresholdType: 'cpu',
      thresholdValue: 80,
      severity: 'warning',
      consecutiveChecks: 3,
    })
  }

  const handleCreate = () => {
    if (!alertForm.vmId) {
      toast.error('Please select a VM')
      return
    }
    if (alertForm.thresholdValue < 0 || alertForm.thresholdValue > 100) {
      toast.error('Threshold value must be between 0 and 100')
      return
    }
    createMutation.mutate(alertForm)
  }

  const handleToggleEnabled = (id: string, currentlyEnabled: boolean) => {
    updateStatusMutation.mutate({ id, enabled: !currentlyEnabled })
  }

  const handleDelete = (id: string, name: string) => {
    if (confirm(`Are you sure you want to delete alert "${name}"?`)) {
      deleteMutation.mutate(id)
    }
  }

  const getThresholdIcon = (type: ThresholdType) => {
    switch (type) {
      case 'cpu':
        return <Cpu className="h-4 w-4" />
      case 'memory':
        return <Activity className="h-4 w-4" />
      case 'disk':
        return <HardDrive className="h-4 w-4" />
      case 'network':
        return <NetworkIcon className="h-4 w-4" />
    }
  }

  const getThresholdLabel = (type: ThresholdType) => {
    const labels: Record<ThresholdType, string> = {
      cpu: 'CPU Usage',
      memory: 'Memory Usage',
      disk: 'Disk Usage',
      network: 'Network Usage',
    }
    return labels[type]
  }

  const getSeverityColor = (severity: AlertSeverity) => {
    switch (severity) {
      case 'info':
        return 'default'
      case 'warning':
        return 'secondary'
      case 'critical':
        return 'destructive'
    }
  }

  const getSeverityLabel = (severity: AlertSeverity) => {
    return severity.charAt(0).toUpperCase() + severity.slice(1)
  }

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString()
  }

  const getVmName = (vmId: string) => {
    const vm = vms?.find(v => v.id === vmId)
    return vm?.name || vmId
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-muted-foreground">Loading alerts...</div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Resource Alerts</h2>
          <p className="text-muted-foreground">Set thresholds and get notified when resources exceed limits</p>
        </div>
        <Button onClick={() => setIsCreateDialogOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Alert
        </Button>
      </div>

      {!alerts || alerts.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center p-8">
            <Bell className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-center">
              No resource alerts configured. Create an alert to monitor VM resource usage.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {alerts.map((alert) => (
            <Card key={alert.id}>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <CardTitle className="text-lg flex items-center gap-2">
                      {getThresholdIcon(alert.thresholdType)}
                      {alert.name}
                    </CardTitle>
                    <CardDescription className="mt-1">
                      {!vmId && `VM: ${getVmName(alert.vmId)} • `}
                      {getThresholdLabel(alert.thresholdType)} threshold: {alert.thresholdValue}%
                    </CardDescription>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={getSeverityColor(alert.severity)}>
                      {getSeverityLabel(alert.severity)}
                    </Badge>
                    <Badge variant={alert.enabled ? 'default' : 'secondary'}>
                      {alert.enabled ? 'Enabled' : 'Disabled'}
                    </Badge>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="grid grid-cols-3 gap-4 text-sm">
                    <div>
                      <div className="text-muted-foreground">Threshold</div>
                      <div className="font-medium">{alert.thresholdValue}%</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Consecutive Checks</div>
                      <div className="font-medium">{alert.consecutiveChecks}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Current Triggers</div>
                      <div className="font-medium">{alert.currentTriggerCount}</div>
                    </div>
                  </div>

                  {alert.lastTriggered && (
                    <div className="flex items-center text-sm text-muted-foreground">
                      <AlertTriangle className="mr-2 h-4 w-4" />
                      Last triggered: {formatTimestamp(alert.lastTriggered)}
                    </div>
                  )}

                  {alert.currentTriggerCount > 0 && alert.enabled && (
                    <div className="flex items-center text-sm text-orange-600 dark:text-orange-400">
                      <AlertTriangle className="mr-2 h-4 w-4" />
                      Warning: {alert.currentTriggerCount} of {alert.consecutiveChecks} checks exceeded
                    </div>
                  )}

                  <div className="flex gap-2 pt-2">
                    <Button
                      size="sm"
                      variant={alert.enabled ? 'outline' : 'default'}
                      onClick={() => handleToggleEnabled(alert.id, alert.enabled)}
                    >
                      {alert.enabled ? (
                        <>
                          <PauseCircle className="mr-1 h-3 w-3" />
                          Disable
                        </>
                      ) : (
                        <>
                          <PlayCircle className="mr-1 h-3 w-3" />
                          Enable
                        </>
                      )}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleDelete(alert.id, alert.name)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Create Alert Dialog */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Create Resource Alert</DialogTitle>
            <DialogDescription>
              Set up threshold-based alerts for VM resource monitoring
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="alert-name">Alert Name</Label>
              <Input
                id="alert-name"
                value={alertForm.name}
                onChange={(e) =>
                  setAlertForm({ ...alertForm, name: e.target.value })
                }
                placeholder="e.g., High CPU Usage Alert"
              />
            </div>

            {!vmId && (
              <div>
                <Label htmlFor="vm-select">Virtual Machine</Label>
                <select
                  id="vm-select"
                  value={alertForm.vmId}
                  onChange={(e) =>
                    setAlertForm({ ...alertForm, vmId: e.target.value })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="">Select a VM...</option>
                  {vms?.map((vm) => (
                    <option key={vm.id} value={vm.id}>
                      {vm.name}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="threshold-type">Resource Type</Label>
                <select
                  id="threshold-type"
                  value={alertForm.thresholdType}
                  onChange={(e) =>
                    setAlertForm({
                      ...alertForm,
                      thresholdType: e.target.value as ThresholdType,
                    })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="cpu">CPU Usage</option>
                  <option value="memory">Memory Usage</option>
                  <option value="disk">Disk Usage</option>
                  <option value="network">Network Usage</option>
                </select>
              </div>

              <div>
                <Label htmlFor="threshold-value">Threshold (%)</Label>
                <Input
                  id="threshold-value"
                  type="number"
                  min="0"
                  max="100"
                  step="5"
                  value={alertForm.thresholdValue}
                  onChange={(e) =>
                    setAlertForm({
                      ...alertForm,
                      thresholdValue: parseFloat(e.target.value),
                    })
                  }
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="severity">Severity Level</Label>
                <select
                  id="severity"
                  value={alertForm.severity}
                  onChange={(e) =>
                    setAlertForm({
                      ...alertForm,
                      severity: e.target.value as AlertSeverity,
                    })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="info">Info</option>
                  <option value="warning">Warning</option>
                  <option value="critical">Critical</option>
                </select>
              </div>

              <div>
                <Label htmlFor="consecutive-checks">Consecutive Checks</Label>
                <Input
                  id="consecutive-checks"
                  type="number"
                  min="1"
                  max="10"
                  value={alertForm.consecutiveChecks}
                  onChange={(e) =>
                    setAlertForm({
                      ...alertForm,
                      consecutiveChecks: parseInt(e.target.value),
                    })
                  }
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Number of consecutive threshold violations before alerting
                </p>
              </div>
            </div>

            <div className="bg-muted p-3 rounded-md text-sm">
              <p className="font-medium mb-1">Alert Preview:</p>
              <p>
                Alert will trigger when <strong>{getThresholdLabel(alertForm.thresholdType)}</strong> exceeds{' '}
                <strong>{alertForm.thresholdValue}%</strong> for{' '}
                <strong>{alertForm.consecutiveChecks}</strong> consecutive check(s).
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={!alertForm.name || !alertForm.vmId}>
              Create Alert
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
