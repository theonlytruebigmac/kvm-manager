import { useParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { useEffect, useRef, useState } from 'react'
import { Loader2, Monitor, Terminal } from 'lucide-react'
import { useWindowState } from '@/hooks/useWindowState'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { VncViewer, type VncViewerRef, type ScaleMode } from '@/components/console/VncViewer'
import { SpiceViewer, type SpiceViewerRef } from '@/components/console/SpiceViewer'
import { SerialConsole } from '@/components/console/SerialConsole'
import { ConsoleToolbar } from '@/components/console/ConsoleToolbar'
import { toast } from 'sonner'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'

type ConsoleType = 'graphical' | 'serial'

/**
 * Console Window - Dedicated window for VM console (VNC or SPICE)
 *
 * This component renders in a separate Tauri window and provides
 * a clean, focused interface for interacting with the VM console.
 */

export function ConsoleWindow() {
  const { vmId } = useParams<{ vmId: string }>()
  const vncViewerRef = useRef<VncViewerRef | null>(null)
  const spiceViewerRef = useRef<SpiceViewerRef | null>(null)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [isConnected, setIsConnected] = useState(false)
  const [scaleMode, setScaleMode] = useState<ScaleMode>('scale')
  const [consoleType, setConsoleType] = useState<ConsoleType>('graphical')

  // Enable window state persistence
  useWindowState()

  const { data: vm, isLoading: vmLoading } = useQuery({
    queryKey: ['vm', vmId],
    queryFn: () => api.getVm(vmId!),
    enabled: !!vmId,
  })

  const { data: vncInfo, isLoading: vncLoading } = useQuery({
    queryKey: ['vnc-info', vmId],
    queryFn: () => api.getVncInfo(vmId!),
    enabled: !!vmId,
    refetchInterval: false, // Don't auto-refresh - websockify proxy stays active
  })

  const isLoading = vmLoading || vncLoading

  // Cleanup websockify proxy when window closes - ONLY on actual window close
  useEffect(() => {
    if (!vmId) return

    // Listen for window close event
    const currentWindow = getCurrentWindow()
    let unlistenFn: (() => void) | null = null

    const setupListener = async () => {
      unlistenFn = await currentWindow.onCloseRequested(async () => {
        try {
          await api.stopVncProxy(vmId)
          console.log('Stopped websockify proxy on window close for VM:', vmId)
        } catch (e) {
          console.warn('Failed to stop websockify proxy:', e)
        }
      })
    }

    setupListener()

    // Only unsubscribe from event listener on unmount, don't stop the proxy
    // The proxy will be stopped via the onCloseRequested handler when window actually closes
    return () => {
      if (unlistenFn) {
        unlistenFn()
      }
    }
  }, [vmId])

  const handleToggleFullscreen = async () => {
    const window = getCurrentWindow()
    const isCurrentlyFullscreen = await window.isFullscreen()
    await window.setFullscreen(!isCurrentlyFullscreen)
    setIsFullscreen(!isCurrentlyFullscreen)
  }

  // Get the active viewer ref based on graphics type
  const getActiveViewer = () => {
    const graphicsType = vncInfo?.graphics_type || 'vnc'
    return graphicsType === 'spice' ? spiceViewerRef.current : vncViewerRef.current
  }

  const handleScreenshot = async () => {
    const canvas = getActiveViewer()?.getCanvas()
    if (!canvas) {
      toast.error('Console not connected')
      return
    }

    try {
      // Convert canvas to blob
      canvas.toBlob(async (blob: Blob | null) => {
        if (!blob) {
          toast.error('Failed to create screenshot')
          return
        }

        try {
          // Save file using Tauri dialog
          const { save } = await import('@tauri-apps/plugin-dialog')
          const filePath = await save({
            defaultPath: `${vm?.name || 'vm'}-screenshot-${Date.now()}.png`,
            filters: [{
              name: 'PNG Image',
              extensions: ['png']
            }]
          })

          if (filePath) {
            // Create a download link as fallback (Tauri v2 file write will be added later)
            const url = URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `${vm?.name || 'vm'}-screenshot-${Date.now()}.png`
            document.body.appendChild(a)
            a.click()
            document.body.removeChild(a)
            URL.revokeObjectURL(url)

            toast.success('Screenshot saved')
          }
        } catch (error) {
          console.error('Save error:', error)
          toast.error(`Failed to save screenshot: ${error}`)
        }
      }, 'image/png')
    } catch (error) {
      console.error('Screenshot error:', error)
      toast.error(`Failed to capture screenshot: ${error}`)
    }
  }

  const handleConnected = () => {
    setIsConnected(true)
    toast.success('Console connected')
  }

  const handleDisconnected = () => {
    setIsConnected(false)
    toast.error('Console disconnected')
  }

  const handleError = (error: string) => {
    toast.error(`Console error: ${error}`)
  }

  const handleScaleModeChange = (mode: ScaleMode) => {
    setScaleMode(mode)
    getActiveViewer()?.setScaleMode(mode)
    toast.success(`Display mode: ${mode === 'scale' ? 'Scale to Window' : mode === 'fit' ? '1:1 Pixel Mapping' : 'Stretch to Fill'}`)
  }

  // Keyboard shortcuts for window management and fullscreen
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Escape or Ctrl+W to close window (only if not in fullscreen)
      if (!isFullscreen && (event.key === 'Escape' || (event.ctrlKey && event.key === 'w'))) {
        event.preventDefault()
        getCurrentWindow().close()
      }

      // F11 for fullscreen toggle
      if (event.key === 'F11') {
        event.preventDefault()
        handleToggleFullscreen()
      }

      // F10 for screenshot
      if (event.key === 'F10') {
        event.preventDefault()
        handleScreenshot()
      }

      // Escape to exit fullscreen
      if (isFullscreen && event.key === 'Escape') {
        event.preventDefault()
        handleToggleFullscreen()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isFullscreen])

  if (isLoading) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="w-8 h-8 animate-spin text-primary" />
          <p className="text-sm text-muted-foreground">Connecting to console...</p>
        </div>
      </div>
    )
  }

  if (!vm || !vncInfo) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <p className="text-lg font-medium text-foreground">Failed to connect to console</p>
          <p className="text-sm text-muted-foreground mt-2">
            VM may not be running or console is not configured
          </p>
        </div>
      </div>
    )
  }

  const graphicsType = vncInfo?.graphics_type || 'vnc'

  return (
    <div className="h-screen w-screen flex flex-col bg-black">
      {/* Console Type Tabs & Toolbar */}
      <div className="flex items-center gap-4 border-b border-gray-700 bg-gray-900 px-4">
        <Tabs value={consoleType} onValueChange={(v) => setConsoleType(v as ConsoleType)}>
          <TabsList className="h-8 bg-gray-800">
            <TabsTrigger value="graphical" className="text-xs gap-1.5 data-[state=active]:bg-gray-700">
              <Monitor className="h-3 w-3" />
              Graphical
            </TabsTrigger>
            <TabsTrigger value="serial" className="text-xs gap-1.5 data-[state=active]:bg-gray-700">
              <Terminal className="h-3 w-3" />
              Serial
            </TabsTrigger>
          </TabsList>
        </Tabs>

        {consoleType === 'graphical' && (
          <ConsoleToolbar
            isFullscreen={isFullscreen}
            onToggleFullscreen={handleToggleFullscreen}
            onScreenshot={handleScreenshot}
            vncViewerRef={graphicsType === 'vnc' ? vncViewerRef : spiceViewerRef}
            vmName={vm.name}
            scaleMode={scaleMode}
            onScaleModeChange={handleScaleModeChange}
          />
        )}
      </div>

      {/* Console Display */}
      <div className="flex-1 relative min-h-0">
        {consoleType === 'graphical' ? (
          // Graphical Console (VNC/SPICE)
          vm.state === 'running' && vncInfo ? (
            graphicsType === 'spice' ? (
              <SpiceViewer
                ref={spiceViewerRef}
                host={vncInfo.host}
                port={vncInfo.port}
                password={vncInfo.password}
                scaleMode={scaleMode}
                onConnected={handleConnected}
                onDisconnected={handleDisconnected}
                onError={handleError}
              />
            ) : (
              <VncViewer
                ref={vncViewerRef}
                host={vncInfo.host}
                port={vncInfo.port}
                password={vncInfo.password}
                scaleMode={scaleMode}
                onConnected={handleConnected}
                onDisconnected={handleDisconnected}
                onError={handleError}
              />
            )
          ) : (
            <div className="h-full flex flex-col items-center justify-center bg-gray-900">
              <p className="text-white text-lg mb-2">VM is {vm.state}</p>
              <p className="text-gray-400 text-sm">Start the VM to access the console</p>
            </div>
          )
        ) : (
          // Serial Console
          <div className="h-full p-4">
            <SerialConsole vmId={vmId!} vmName={vm.name} />
          </div>
        )}
      </div>

      {/* Status Bar */}
      <div className="h-6 border-t border-gray-700 bg-gray-800 flex items-center px-4 text-xs text-gray-400">
        <div className={`w-2 h-2 rounded-full mr-2 ${isConnected ? 'bg-green-500' : 'bg-gray-600'}`} />
        <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
        <span className="mx-2">•</span>
        <span className="uppercase">{consoleType === 'graphical' ? graphicsType : 'Serial'}</span>
        {consoleType === 'graphical' && (
          <>
            <span className="mx-2">•</span>
            <span className="capitalize">{scaleMode === 'scale' ? 'Scaled' : scaleMode === 'fit' ? '1:1' : 'Stretched'}</span>
          </>
        )}
        {isConnected && consoleType === 'graphical' && (
          <>
            <span className="mx-2">•</span>
            <span>F11: Fullscreen, F10: Screenshot</span>
          </>
        )}
      </div>
    </div>
  )
}
