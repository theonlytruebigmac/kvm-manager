import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { ScheduledOperation, CreateScheduleRequest, OperationType, ScheduleFrequency, VM } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { Clock, Plus, Trash2, Power, PowerOff, RotateCw, Camera, Calendar, PlayCircle, PauseCircle } from 'lucide-react'

interface ScheduleManagerProps {
  vmId?: string // Optional: show schedules for specific VM
}

export function ScheduleManager({ vmId }: ScheduleManagerProps) {
  const queryClient = useQueryClient()
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [scheduleForm, setScheduleForm] = useState<CreateScheduleRequest>({
    name: '',
    vmId: vmId || '',
    operation: 'start',
    frequency: 'daily',
    scheduledTime: '09:00',
    dayOfWeek: undefined,
    dayOfMonth: undefined,
  })

  // Query for VMs (to populate dropdown)
  const { data: vms } = useQuery<VM[]>({
    queryKey: ['vms'],
    queryFn: () => api.getVms(),
  })

  // Query for schedules
  const { data: schedules, isLoading } = useQuery<ScheduledOperation[]>({
    queryKey: vmId ? ['schedules', vmId] : ['schedules'],
    queryFn: () => vmId ? api.getVmSchedules(vmId) : api.listSchedules(),
  })

  // Create schedule mutation
  const createMutation = useMutation({
    mutationFn: (request: CreateScheduleRequest) => api.createSchedule(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
      toast.success('Schedule created successfully')
      setIsCreateDialogOpen(false)
      resetForm()
    },
    onError: (error: Error) => {
      toast.error(`Failed to create schedule: ${error.message}`)
    },
  })

  // Update schedule status mutation
  const updateStatusMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      api.updateScheduleStatus(id, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
      toast.success('Schedule status updated')
    },
    onError: (error: Error) => {
      toast.error(`Failed to update schedule: ${error.message}`)
    },
  })

  // Delete schedule mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteSchedule(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
      toast.success('Schedule deleted successfully')
    },
    onError: (error: Error) => {
      toast.error(`Failed to delete schedule: ${error.message}`)
    },
  })

  const resetForm = () => {
    setScheduleForm({
      name: '',
      vmId: vmId || '',
      operation: 'start',
      frequency: 'daily',
      scheduledTime: '09:00',
      dayOfWeek: undefined,
      dayOfMonth: undefined,
    })
  }

  const handleCreate = () => {
    if (!scheduleForm.vmId) {
      toast.error('Please select a VM')
      return
    }
    createMutation.mutate(scheduleForm)
  }

  const handleToggleEnabled = (id: string, currentlyEnabled: boolean) => {
    updateStatusMutation.mutate({ id, enabled: !currentlyEnabled })
  }

  const handleDelete = (id: string, name: string) => {
    if (confirm(`Are you sure you want to delete schedule "${name}"?`)) {
      deleteMutation.mutate(id)
    }
  }

  const getOperationIcon = (operation: OperationType) => {
    switch (operation) {
      case 'start':
        return <Power className="h-4 w-4" />
      case 'stop':
        return <PowerOff className="h-4 w-4" />
      case 'reboot':
        return <RotateCw className="h-4 w-4" />
      case 'snapshot':
        return <Camera className="h-4 w-4" />
    }
  }

  const getOperationLabel = (operation: OperationType) => {
    return operation.charAt(0).toUpperCase() + operation.slice(1)
  }

  const getFrequencyLabel = (frequency: ScheduleFrequency) => {
    return frequency.charAt(0).toUpperCase() + frequency.slice(1)
  }

  const formatNextRun = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString()
  }

  const getVmName = (vmId: string) => {
    const vm = vms?.find(v => v.id === vmId)
    return vm?.name || vmId
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-muted-foreground">Loading schedules...</div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Scheduled Operations</h2>
          <p className="text-muted-foreground">Automate VM start, stop, reboot, and snapshots</p>
        </div>
        <Button onClick={() => setIsCreateDialogOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Schedule
        </Button>
      </div>

      {!schedules || schedules.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center p-8">
            <Clock className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-center">
              No scheduled operations yet. Create a schedule to automate VM operations.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {schedules.map((schedule) => (
            <Card key={schedule.id}>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <CardTitle className="text-lg flex items-center gap-2">
                      {getOperationIcon(schedule.operation)}
                      {schedule.name}
                    </CardTitle>
                    <CardDescription className="mt-1">
                      {!vmId && `VM: ${getVmName(schedule.vmId)} • `}
                      {getOperationLabel(schedule.operation)} • {getFrequencyLabel(schedule.frequency)}
                    </CardDescription>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={schedule.enabled ? 'default' : 'secondary'}>
                      {schedule.enabled ? 'Enabled' : 'Disabled'}
                    </Badge>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <div className="text-muted-foreground">Time</div>
                      <div className="font-medium">{schedule.scheduledTime}</div>
                    </div>
                    {schedule.frequency === 'weekly' && schedule.dayOfWeek !== undefined && (
                      <div>
                        <div className="text-muted-foreground">Day of Week</div>
                        <div className="font-medium">
                          {['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][schedule.dayOfWeek]}
                        </div>
                      </div>
                    )}
                    {schedule.frequency === 'monthly' && schedule.dayOfMonth !== undefined && (
                      <div>
                        <div className="text-muted-foreground">Day of Month</div>
                        <div className="font-medium">{schedule.dayOfMonth}</div>
                      </div>
                    )}
                  </div>

                  <div className="flex items-center text-sm text-muted-foreground">
                    <Calendar className="mr-2 h-4 w-4" />
                    Next run: {formatNextRun(schedule.nextRun)}
                  </div>

                  {schedule.lastRun && (
                    <div className="text-sm text-muted-foreground">
                      Last run: {formatNextRun(schedule.lastRun)}
                    </div>
                  )}

                  <div className="flex gap-2 pt-2">
                    <Button
                      size="sm"
                      variant={schedule.enabled ? 'outline' : 'default'}
                      onClick={() => handleToggleEnabled(schedule.id, schedule.enabled)}
                    >
                      {schedule.enabled ? (
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
                      onClick={() => handleDelete(schedule.id, schedule.name)}
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

      {/* Create Schedule Dialog */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Create Scheduled Operation</DialogTitle>
            <DialogDescription>
              Schedule automatic VM operations at specific times
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="schedule-name">Schedule Name</Label>
              <Input
                id="schedule-name"
                value={scheduleForm.name}
                onChange={(e) =>
                  setScheduleForm({ ...scheduleForm, name: e.target.value })
                }
                placeholder="e.g., Daily VM Startup"
              />
            </div>

            {!vmId && (
              <div>
                <Label htmlFor="vm-select">Virtual Machine</Label>
                <select
                  id="vm-select"
                  value={scheduleForm.vmId}
                  onChange={(e) =>
                    setScheduleForm({ ...scheduleForm, vmId: e.target.value })
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
                <Label htmlFor="operation">Operation</Label>
                <select
                  id="operation"
                  value={scheduleForm.operation}
                  onChange={(e) =>
                    setScheduleForm({
                      ...scheduleForm,
                      operation: e.target.value as OperationType,
                    })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="start">Start VM</option>
                  <option value="stop">Stop VM</option>
                  <option value="reboot">Reboot VM</option>
                  <option value="snapshot">Create Snapshot</option>
                </select>
              </div>

              <div>
                <Label htmlFor="frequency">Frequency</Label>
                <select
                  id="frequency"
                  value={scheduleForm.frequency}
                  onChange={(e) => {
                    const freq = e.target.value as ScheduleFrequency
                    setScheduleForm({
                      ...scheduleForm,
                      frequency: freq,
                      dayOfWeek: freq === 'weekly' ? 1 : undefined,
                      dayOfMonth: freq === 'monthly' ? 1 : undefined,
                    })
                  }}
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="once">Once</option>
                  <option value="daily">Daily</option>
                  <option value="weekly">Weekly</option>
                  <option value="monthly">Monthly</option>
                </select>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="scheduled-time">Time</Label>
                <Input
                  id="scheduled-time"
                  type="time"
                  value={scheduleForm.scheduledTime}
                  onChange={(e) =>
                    setScheduleForm({ ...scheduleForm, scheduledTime: e.target.value })
                  }
                />
              </div>

              {scheduleForm.frequency === 'weekly' && (
                <div>
                  <Label htmlFor="day-of-week">Day of Week</Label>
                  <select
                    id="day-of-week"
                    value={scheduleForm.dayOfWeek || 0}
                    onChange={(e) =>
                      setScheduleForm({
                        ...scheduleForm,
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

              {scheduleForm.frequency === 'monthly' && (
                <div>
                  <Label htmlFor="day-of-month">Day of Month</Label>
                  <Input
                    id="day-of-month"
                    type="number"
                    min="1"
                    max="28"
                    value={scheduleForm.dayOfMonth || 1}
                    onChange={(e) =>
                      setScheduleForm({
                        ...scheduleForm,
                        dayOfMonth: parseInt(e.target.value),
                      })
                    }
                  />
                </div>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={!scheduleForm.name || !scheduleForm.vmId}>
              Create Schedule
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
