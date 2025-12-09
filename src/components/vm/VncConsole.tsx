import { useQuery, useMutation } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Monitor, RefreshCw, ExternalLink } from 'lucide-react'

interface VncConsoleProps {
  vmId: string
  vmName: string
}

export function VncConsole({ vmId, vmName }: VncConsoleProps) {
  // Get VNC connection info
  const { data: vncInfo, isLoading, error, refetch } = useQuery({
    queryKey: ['vnc-info', vmId],
    queryFn: () => api.getVncInfo(vmId),
    retry: 2,
    refetchOnWindowFocus: false,
  })

  // Open external VNC viewer mutation
  const openVncMutation = useMutation({
    mutationFn: () => api.openVncConsole(vmId),
    onSuccess: () => {
      toast.success('Opening VNC viewer...')
    },
    onError: (error) => {
      toast.error(`Failed to open VNC viewer: ${error}`)
    },
  })

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Monitor className="h-5 w-5" />
            VNC Console Access
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <p className="text-sm text-muted-foreground">Loading VNC connection info...</p>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (error || !vncInfo) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Monitor className="h-5 w-5" />
            VNC Console Access
          </CardTitle>
          <CardDescription>Remote desktop access to {vmName}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-8 space-y-4">
            <Monitor className="h-12 w-12 text-muted-foreground" />
            <div className="text-center">
              <p className="text-sm font-medium">VNC Not Available</p>
              <p className="text-xs text-muted-foreground mt-1">
                {String(error) || 'VM must be running with VNC enabled'}
              </p>
            </div>
            <Button variant="outline" size="sm" onClick={() => refetch()}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Retry
            </Button>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Monitor className="h-5 w-5" />
              VNC Console Access
            </CardTitle>
            <CardDescription>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 bg-green-500 rounded-full" />
                VNC available at {vncInfo.host}:{vncInfo.port}
              </span>
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => refetch()}
              title="Refresh connection info"
            >
              <RefreshCw className="h-4 w-4" />
            </Button>
            <Button
              size="sm"
              onClick={() => openVncMutation.mutate()}
              disabled={openVncMutation.isPending}
            >
              <ExternalLink className="mr-2 h-4 w-4" />
              {openVncMutation.isPending ? 'Opening...' : 'Open VNC Viewer'}
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="p-4 bg-muted/50 rounded-lg space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Host:</span>
              <span className="font-mono">{vncInfo.host}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Port:</span>
              <span className="font-mono">{vncInfo.port}</span>
            </div>
            {vncInfo.password && (
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Password:</span>
                <span className="font-mono">••••••••</span>
              </div>
            )}
          </div>
          <div className="text-xs text-muted-foreground space-y-1">
            <p>• Click "Open VNC Viewer" to launch an external VNC client</p>
            <p>• Make sure you have a VNC viewer installed (e.g., virt-viewer, TigerVNC, RealVNC)</p>
            <p>• You can also connect manually using the connection details above</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
