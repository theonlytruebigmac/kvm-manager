import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { BackupConfig, CreateBackupRequest, ScheduleFrequency, VM } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { HardDrive, Plus, Trash2, PlayCircle, PauseCircle, Calendar, Camera } from 'lucide-react'

interface BackupManagerProps {
  vmId?: string // Optional: show backups for specific VM
}

export function BackupManager({ vmId }: BackupManagerProps) {
  const queryClient = useQueryClient()
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [backupForm, setBackupForm] = useState<CreateBackupRequest>({
    name: '',
    vmId: vmId || '',
    frequency: 'daily',
    scheduledTime: '02:00',
    dayOfWeek: undefined,
    dayOfMonth: undefined,
    retentionCount: 7,
  })

  // Query for VMs (to populate dropdown)
  const { data: vms } = useQuery<VM[]>({
    queryKey: ['vms'],
    queryFn: () => api.getVms(),
  })

  // Query for backup configs
  const { data: backupConfigs, isLoading } = useQuery<BackupConfig[]>({
    queryKey: vmId ? ['backupConfigs', vmId] : ['backupConfigs'],
    queryFn: () => vmId ? api.getVmBackupConfigs(vmId) : api.listBackupConfigs(),
  })

  // Create backup config mutation
  const createMutation = useMutation({
    mutationFn: (request: CreateBackupRequest) => api.createBackupConfig(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backupConfigs'] })
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
      toast.success('Backup schedule created successfully')
      setIsCreateDialogOpen(false)
      resetForm()
    },
    onError: (error: Error) => {
      toast.error(`Failed to create backup schedule: ${error.message}`)
    },
  })

  // Update backup status mutation
  const updateStatusMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      api.updateBackupStatus(id, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backupConfigs'] })
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
      toast.success('Backup status updated')
    },
    onError: (error: Error) => {
      toast.error(`Failed to update backup: ${error.message}`)
    },
  })

  // Delete backup config mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteBackupConfig(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backupConfigs'] })
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
      toast.success('Backup schedule deleted successfully')
    },
    onError: (error: Error) => {
      toast.error(`Failed to delete backup schedule: ${error.message}`)
    },
  })

  const resetForm = () => {
    setBackupForm({
      name: '',
      vmId: vmId || '',
      frequency: 'daily',
      scheduledTime: '02:00',
      dayOfWeek: undefined,
      dayOfMonth: undefined,
      retentionCount: 7,
    })
  }

  const handleCreate = () => {
    if (!backupForm.vmId) {
      toast.error('Please select a VM')
      return
    }
    if (backupForm.retentionCount < 1) {
      toast.error('Retention count must be at least 1')
      return
    }
    createMutation.mutate(backupForm)
  }

  const handleToggleEnabled = (id: string, currentlyEnabled: boolean) => {
    updateStatusMutation.mutate({ id, enabled: !currentlyEnabled })
  }

  const handleDelete = (id: string, name: string) => {
    if (confirm(`Are you sure you want to delete backup schedule "${name}"?`)) {
      deleteMutation.mutate(id)
    }
  }

  const getFrequencyLabel = (frequency: ScheduleFrequency) => {
    return frequency.charAt(0).toUpperCase() + frequency.slice(1)
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
        <div className="text-muted-foreground">Loading backup schedules...</div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Backup Schedules</h2>
          <p className="text-muted-foreground">Automated snapshot backups with retention management</p>
        </div>
        <Button onClick={() => setIsCreateDialogOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Backup Schedule
        </Button>
      </div>

      {!backupConfigs || backupConfigs.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center p-8">
            <HardDrive className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-center">
              No backup schedules configured. Create a schedule to automate VM snapshots.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {backupConfigs.map((backup) => (
            <Card key={backup.id}>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <CardTitle className="text-lg flex items-center gap-2">
                      <Camera className="h-4 w-4" />
                      {backup.name}
                    </CardTitle>
                    <CardDescription className="mt-1">
                      {!vmId && `VM: ${getVmName(backup.vmId)} • `}
                      Automated snapshot backups
                    </CardDescription>
                  </div>
                  <Badge variant={backup.enabled ? 'default' : 'secondary'}>
                    {backup.enabled ? 'Enabled' : 'Disabled'}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="grid grid-cols-3 gap-4 text-sm">
                    <div>
                      <div className="text-muted-foreground">Retention</div>
                      <div className="font-medium">{backup.retentionCount} backups</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Total Backups</div>
                      <div className="font-medium">{backup.backupCount}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Created</div>
                      <div className="font-medium">{formatTimestamp(backup.createdAt)}</div>
                    </div>
                  </div>

                  {backup.lastBackup && (
                    <div className="flex items-center text-sm text-muted-foreground">
                      <Calendar className="mr-2 h-4 w-4" />
                      Last backup: {formatTimestamp(backup.lastBackup)}
                    </div>
                  )}

                  <div className="bg-muted p-3 rounded-md text-sm">
                    <p className="text-muted-foreground">
                      This backup is scheduled via the Schedules system. View the full schedule details in the Schedules page.
                    </p>
                  </div>

                  <div className="flex gap-2 pt-2">
                    <Button
                      size="sm"
                      variant={backup.enabled ? 'outline' : 'default'}
                      onClick={() => handleToggleEnabled(backup.id, backup.enabled)}
                    >
                      {backup.enabled ? (
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
                      onClick={() => handleDelete(backup.id, backup.name)}
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

      {/* Create Backup Schedule Dialog */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Create Backup Schedule</DialogTitle>
            <DialogDescription>
              Set up automated snapshot backups with retention management
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="backup-name">Backup Name</Label>
              <Input
                id="backup-name"
                value={backupForm.name}
                onChange={(e) =>
                  setBackupForm({ ...backupForm, name: e.target.value })
                }
                placeholder="e.g., Daily Production Backup"
              />
            </div>

            {!vmId && (
              <div>
                <Label htmlFor="vm-select">Virtual Machine</Label>
                <select
                  id="vm-select"
                  value={backupForm.vmId}
                  onChange={(e) =>
                    setBackupForm({ ...backupForm, vmId: e.target.value })
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
                <Label htmlFor="frequency">Backup Frequency</Label>
                <select
                  id="frequency"
                  value={backupForm.frequency}
                  onChange={(e) => {
                    const freq = e.target.value as ScheduleFrequency
                    setBackupForm({
                      ...backupForm,
                      frequency: freq,
                      dayOfWeek: freq === 'weekly' ? 0 : undefined,
                      dayOfMonth: freq === 'monthly' ? 1 : undefined,
                    })
                  }}
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="daily">Daily</option>
                  <option value="weekly">Weekly</option>
                  <option value="monthly">Monthly</option>
                </select>
              </div>

              <div>
                <Label htmlFor="scheduled-time">Time</Label>
                <Input
                  id="scheduled-time"
                  type="time"
                  value={backupForm.scheduledTime}
                  onChange={(e) =>
                    setBackupForm({ ...backupForm, scheduledTime: e.target.value })
                  }
                />
              </div>
            </div>

            {backupForm.frequency === 'weekly' && (
              <div>
                <Label htmlFor="day-of-week">Day of Week</Label>
                <select
                  id="day-of-week"
                  value={backupForm.dayOfWeek || 0}
                  onChange={(e) =>
                    setBackupForm({
                      ...backupForm,
                      dayOfWeek: parseInt(e.target.value),
                    })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="0">Sunday</option>
                  <option value="1">Monday</option>
                  <option value="2">Tuesday</option>
                  <option value="3">Wednesday</option>
                  <option value="4">Thursday</option>
                  <option value="5">Friday</option>
                  <option value="6">Saturday</option>
                </select>
              </div>
            )}

            {backupForm.frequency === 'monthly' && (
              <div>
                <Label htmlFor="day-of-month">Day of Month</Label>
                <Input
                  id="day-of-month"
                  type="number"
                  min="1"
                  max="28"
                  value={backupForm.dayOfMonth || 1}
                  onChange={(e) =>
                    setBackupForm({
                      ...backupForm,
                      dayOfMonth: parseInt(e.target.value),
                    })
                  }
                />
              </div>
            )}

            <div>
              <Label htmlFor="retention-count">Retention Count</Label>
              <Input
                id="retention-count"
                type="number"
                min="1"
                max="100"
                value={backupForm.retentionCount}
                onChange={(e) =>
                  setBackupForm({
                    ...backupForm,
                    retentionCount: parseInt(e.target.value),
                  })
                }
              />
              <p className="text-xs text-muted-foreground mt-1">
                Number of backup snapshots to retain (older backups will be automatically removed)
              </p>
            </div>

            <div className="bg-muted p-3 rounded-md text-sm">
              <p className="font-medium mb-1">Backup Schedule Preview:</p>
              <p>
                Snapshots will be created <strong>{getFrequencyLabel(backupForm.frequency)}</strong> at{' '}
                <strong>{backupForm.scheduledTime}</strong>, keeping the most recent{' '}
                <strong>{backupForm.retentionCount}</strong> backup(s).
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={!backupForm.name || !backupForm.vmId}>
              Create Backup Schedule
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
