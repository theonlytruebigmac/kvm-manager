import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CircleAlert, CircleCheck, CircleX, Copy, LoaderCircle, RefreshCw } from 'lucide-react'
import { api } from '@/lib/tauri'
import { useActiveConnection } from '@/hooks/useActiveConnection'
import type { CapabilityResult, VmCreationReadiness } from '@/lib/types'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, buttonVariants } from '@/components/ui/button'
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
  AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { toast } from 'sonner'
import type { ReadinessRepairAction } from '@/lib/types'

interface HostReadinessPanelProps {
  report?: VmCreationReadiness
}

function stateIcon(capability: CapabilityResult) {
  if (capability.state === 'available') {
    return <CircleCheck aria-hidden="true" className="h-4 w-4 text-green-600" />
  }
  if (capability.state === 'unavailable') {
    return <CircleX aria-hidden="true" className="h-4 w-4 text-destructive" />
  }
  return <CircleAlert aria-hidden="true" className="h-4 w-4 text-amber-600" />
}

export function HostReadinessPanel({ report: providedReport }: HostReadinessPanelProps) {
  const { connectionId, resourceQueryKey } = useActiveConnection()
  const queryClient = useQueryClient()
  const [selectedAction, setSelectedAction] = useState<ReadinessRepairAction | null>(null)
  const readiness = useQuery({
    queryKey: resourceQueryKey('host-readiness') ?? ['connection', 'pending', 'host-readiness'],
    queryFn: api.getHostReadiness,
    enabled: !!connectionId && providedReport === undefined,
    retry: false,
  })
  const report = providedReport ?? readiness.data
  useEffect(() => setSelectedAction(null), [connectionId])
  const repair = useMutation({
    mutationFn: (action: ReadinessRepairAction) => api.executeReadinessRepair(action.id),
    onSuccess: (result) => {
      toast[result.outcome === 'applied' ? 'success' : 'error'](result.summary)
      setSelectedAction(null)
      queryClient.invalidateQueries({ queryKey: resourceQueryKey('host-readiness') })
      queryClient.invalidateQueries({ queryKey: resourceQueryKey('vm-creation-readiness') })
      queryClient.invalidateQueries({ queryKey: resourceQueryKey('storage-pools') })
    },
    onError: () => toast.error('The repair could not be completed. Review the guided steps and retry.'),
  })

  const recheckReadiness = () => {
    setSelectedAction(null)
    queryClient.invalidateQueries({ queryKey: resourceQueryKey('host-readiness') })
    queryClient.invalidateQueries({ queryKey: resourceQueryKey('vm-creation-readiness') })
    toast.success('Readiness is being checked again.')
  }

  const copyCommand = async (command: string) => {
    try {
      await navigator.clipboard.writeText(command)
      toast.success('Command copied.')
    } catch {
      toast.error('Could not copy the command. Select the command text and copy it manually.')
    }
  }

  if (!report && readiness.isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center gap-2 p-6 text-sm text-muted-foreground">
          <LoaderCircle className="h-4 w-4 animate-spin" /> Checking host readiness…
        </CardContent>
      </Card>
    )
  }

  if (!report) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Host Readiness</CardTitle>
          <CardDescription>
            The readiness check could not be completed. Reconnect the selected connection and retry.
          </CardDescription>
        </CardHeader>
      </Card>
    )
  }

  const isReady = report.overallState === 'ready'
  return (
    <>
    <Card>
      <CardHeader>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div>
            <CardTitle>Host Readiness</CardTitle>
            <CardDescription>{report.connectionLabel} · {report.connectionScope.replace('_', ' ')}</CardDescription>
          </div>
          <Badge variant={isReady ? 'default' : 'secondary'}>{isReady ? 'Ready' : 'Degraded'}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-md border p-3 text-sm">
          <p className="font-medium">
            {report.distribution.displayName} · {report.distribution.supported ? 'Verified support' : 'Best-effort support'} · {report.distribution.packageManager}
          </p>
          {report.distribution.packages.length > 0 && (
            <p className="mt-1 text-muted-foreground">
              Packages: {report.distribution.packages.join(', ')}
            </p>
          )}
          <p className="mt-1 text-muted-foreground">{report.distribution.service}</p>
          {report.distribution.limitations.map((limitation) => (
            <p key={limitation} className="mt-1 text-muted-foreground">{limitation}</p>
          ))}
        </div>
        <div className="rounded-md border p-3 text-sm">
          <p className="font-medium">Storage · {report.storage.state.replace('_', ' ')}</p>
          <p className="mt-1 text-muted-foreground">
            {report.storage.pools.length === 0
              ? 'No storage pools were reported by this connection.'
              : `${report.storage.pools.filter((pool) => pool.eligible).length} of ${report.storage.pools.length} pools are usable.`}
          </p>
          {report.storage.recoveryAction && (
            <p className="mt-1 text-muted-foreground">{report.storage.recoveryAction.label}</p>
          )}
        </div>
        <ul className="space-y-2" aria-label="Host capabilities">
          {report.capabilities.map((capability) => (
            <li key={capability.kind} className="rounded-md border p-3 text-sm">
              <div className="flex items-start gap-2">
                {stateIcon(capability)}
                <div>
                  <p className="font-medium">{capability.summary}</p>
                  {capability.remediation && (
                    <p className="mt-1 text-muted-foreground">{capability.remediation}</p>
                  )}
                  {capability.repairAction && (
                    capability.repairAction.mode === 'navigate' ? (
                      <a href="/storage" className={buttonVariants({ variant: 'outline', size: 'sm', className: 'mt-2' })}>
                        {capability.repairAction.title}
                      </a>
                    ) : (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="mt-2"
                        onClick={() => setSelectedAction(capability.repairAction ?? null)}
                      >
                        {capability.repairAction.mode === 'automated' ? 'Fix this requirement' : 'Show guided steps'}
                      </Button>
                    )
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
    <AlertDialog open={selectedAction !== null} onOpenChange={(open) => !open && setSelectedAction(null)}>
      <AlertDialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-xl">
        <AlertDialogHeader>
          <AlertDialogTitle>{selectedAction?.title}</AlertDialogTitle>
          <AlertDialogDescription>
            {report.connectionLabel} · {report.distribution.displayName}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <div className="space-y-3 text-sm">
          <div className="space-y-1">
            <p className="font-medium">What KVM Manager found</p>
            <p className="text-muted-foreground">{selectedAction?.effect}</p>
          </div>
          {selectedAction?.requiresPrivilege && (
            <p className="text-amber-600">Desktop administrator authorization will be requested. KVM Manager will not receive your password.</p>
          )}
          <p className="font-medium">What you can do</p>
          <ol className="list-decimal space-y-3 pl-5 text-muted-foreground">
            {selectedAction?.steps.map((step) => {
              const command = step.startsWith('Run: ') ? step.slice('Run: '.length) : null
              const reference = step.startsWith('Reference: ') ? step.slice('Reference: '.length) : null
              return (
                <li key={step}>
                  {command ? (
                    <div className="space-y-2">
                      <span>Run this command on the virtualization host:</span>
                      <div className="flex items-center gap-2 rounded-md border bg-muted/40 p-2">
                        <code className="min-w-0 flex-1 select-all overflow-x-auto text-xs text-foreground">
                          {command}
                        </code>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          aria-label="Copy command"
                          onClick={() => void copyCommand(command)}
                        >
                          <Copy aria-hidden="true" className="h-3.5 w-3.5" />
                          Copy
                        </Button>
                      </div>
                    </div>
                  ) : reference ? (
                    <a
                      href={reference}
                      target="_blank"
                      rel="noreferrer"
                      className="text-primary underline underline-offset-4"
                    >
                      Open the Arch Linux Secure Boot guide
                    </a>
                  ) : step}
                </li>
              )
            })}
          </ol>
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          {selectedAction?.mode === 'manual' && (
            <Button type="button" variant="outline" onClick={recheckReadiness}>
              <RefreshCw aria-hidden="true" className="h-4 w-4" />
              Recheck readiness
            </Button>
          )}
          {selectedAction?.mode === 'automated' && (
            <AlertDialogAction
              disabled={repair.isPending}
              onClick={(event) => {
                event.preventDefault()
                if (selectedAction) repair.mutate(selectedAction)
              }}
            >
              {repair.isPending ? 'Authorizing…' : 'Confirm and repair'}
            </AlertDialogAction>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
    </>
  )
}
