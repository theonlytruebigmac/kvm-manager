import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { RetentionPolicy } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { Database, Play, Settings as SettingsIcon } from 'lucide-react'

export default function Settings() {
  const queryClient = useQueryClient()
  const [editedPolicy, setEditedPolicy] = useState<RetentionPolicy | null>(null)

  // Query for retention policy
  const { data: policy, isLoading } = useQuery<RetentionPolicy>({
    queryKey: ['retentionPolicy'],
    queryFn: () => api.getRetentionPolicy(),
  })

  // Update policy mutation
  const updateMutation = useMutation({
    mutationFn: (policy: RetentionPolicy) => api.updateRetentionPolicy(policy),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['retentionPolicy'] })
      toast.success('Retention policy updated successfully')
      setEditedPolicy(null)
    },
    onError: (error: Error) => {
      toast.error(`Failed to update retention policy: ${error.message}`)
    },
  })

  // Execute cleanup mutation
  const cleanupMutation = useMutation({
    mutationFn: () => api.executeRetentionCleanup(),
    onSuccess: (deletedCount) => {
      queryClient.invalidateQueries({ queryKey: ['retentionPolicy'] })
      toast.success(`Cleanup completed: ${deletedCount} records deleted`)
    },
    onError: (error: Error) => {
      toast.error(`Cleanup failed: ${error.message}`)
    },
  })

  const handleSave = () => {
    if (editedPolicy) {
      updateMutation.mutate(editedPolicy)
    }
  }

  const handleReset = () => {
    setEditedPolicy(null)
  }

  const handleExecuteCleanup = () => {
    if (confirm('Are you sure you want to execute cleanup now? This will delete old metrics according to the retention policy.')) {
      cleanupMutation.mutate()
    }
  }

  const currentPolicy = editedPolicy || policy

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-muted-foreground">Loading settings...</div>
      </div>
    )
  }

  if (!currentPolicy) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-muted-foreground">Failed to load retention policy</div>
      </div>
    )
  }

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center gap-3">
        <SettingsIcon className="h-8 w-8 text-primary" />
        <div>
          <h1 className="text-3xl font-bold">Settings</h1>
          <p className="text-muted-foreground">Configure KVM Manager settings and policies</p>
        </div>
      </div>

      {/* Metrics Retention Policy */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              <CardTitle>Metrics Retention Policy</CardTitle>
            </div>
            <Badge variant={currentPolicy.enabled ? 'default' : 'secondary'}>
              {currentPolicy.enabled ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>
          <CardDescription>
            Automatically clean up old performance metrics to manage disk space
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Policy Status */}
          <div className="grid grid-cols-3 gap-4 p-4 bg-muted rounded-lg">
            <div>
              <div className="text-sm text-muted-foreground">Status</div>
              <div className="font-medium">
                {currentPolicy.enabled ? 'Active' : 'Inactive'}
              </div>
            </div>
            <div>
              <div className="text-sm text-muted-foreground">Retention Period</div>
              <div className="font-medium">{currentPolicy.retentionDays} days</div>
            </div>
            <div>
              <div className="text-sm text-muted-foreground">Last Cleanup</div>
              <div className="font-medium">
                {currentPolicy.lastCleanup
                  ? new Date(currentPolicy.lastCleanup * 1000).toLocaleString()
                  : 'Never'}
              </div>
            </div>
          </div>

          {/* Configuration Form */}
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="enabled"
                  checked={currentPolicy.enabled}
                  onChange={(e) =>
                    setEditedPolicy({
                      ...currentPolicy,
                      enabled: e.target.checked,
                    })
                  }
                  className="w-4 h-4"
                />
                <Label htmlFor="enabled">Enable automatic cleanup</Label>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="retention-days">Retention Period (days)</Label>
                <Input
                  id="retention-days"
                  type="number"
                  min="1"
                  max="365"
                  value={currentPolicy.retentionDays}
                  onChange={(e) =>
                    setEditedPolicy({
                      ...currentPolicy,
                      retentionDays: parseInt(e.target.value),
                    })
                  }
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Metrics older than this will be deleted
                </p>
              </div>

              <div>
                <Label htmlFor="cleanup-hour">Daily Cleanup Time (hour)</Label>
                <Input
                  id="cleanup-hour"
                  type="number"
                  min="0"
                  max="23"
                  value={currentPolicy.cleanupHour}
                  onChange={(e) =>
                    setEditedPolicy({
                      ...currentPolicy,
                      cleanupHour: parseInt(e.target.value),
                    })
                  }
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Hour of day to run cleanup (0-23, 24-hour format)
                </p>
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2 pt-4 border-t">
            {editedPolicy && (
              <>
                <Button onClick={handleSave} disabled={updateMutation.isPending}>
                  <Database className="mr-2 h-4 w-4" />
                  Save Policy
                </Button>
                <Button variant="outline" onClick={handleReset}>
                  Cancel
                </Button>
              </>
            )}
            <Button
              variant="outline"
              onClick={handleExecuteCleanup}
              disabled={cleanupMutation.isPending || !currentPolicy.enabled}
              className="ml-auto"
            >
              <Play className="mr-2 h-4 w-4" />
              Execute Cleanup Now
            </Button>
          </div>

          {/* Information */}
          <div className="bg-muted/50 border border-border rounded-lg p-4 text-sm">
            <div className="font-medium mb-2 text-foreground">How it works:</div>
            <ul className="list-disc list-inside space-y-1.5 text-muted-foreground">
              <li>
                Cleanup runs automatically every day at {currentPolicy.cleanupHour}:00
              </li>
              <li>
                Metrics older than {currentPolicy.retentionDays} days are permanently deleted
              </li>
              <li>You can manually trigger cleanup at any time using the button above</li>
              <li>
                The policy is stored at: ~/.config/kvm-manager/retention_policy.json
              </li>
            </ul>
          </div>
        </CardContent>
      </Card>

      {/* Additional Settings Placeholder */}
      <Card>
        <CardHeader>
          <CardTitle>Application Settings</CardTitle>
          <CardDescription>Additional configuration options (coming soon)</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-muted-foreground text-sm">
            More settings will be available in future updates
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
