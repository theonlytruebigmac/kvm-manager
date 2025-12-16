import { ReactNode, useState, useEffect } from 'react'
import { ToolbarContent } from '@/components/desktop/ToolbarContent'
import { StatusBar, StatusItem, StatusSpacer } from '@/components/desktop/StatusBar'
import { CommandPalette } from '@/components/desktop/CommandPalette'
import { KeyboardShortcutsDialog } from '@/components/ui/keyboard-shortcuts-dialog'
import { Breadcrumbs } from '@/components/layout/Breadcrumbs'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Wifi, WifiOff, Cpu, MemoryStick } from 'lucide-react'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const [showShortcuts, setShowShortcuts] = useState(false)

  // Listen for keyboard shortcuts dialog event
  useEffect(() => {
    const handler = () => setShowShortcuts(true)
    window.addEventListener('show-keyboard-shortcuts', handler)
    return () => window.removeEventListener('show-keyboard-shortcuts', handler)
  }, [])

  // Get VM stats for status bar
  const { data: vms } = useQuery({
    queryKey: ['vms'],
    queryFn: api.getVms,
    refetchInterval: 5000,
  })

  // Get connection status
  const { data: connectionStatus } = useQuery({
    queryKey: ['connectionStatus'],
    queryFn: api.getConnectionStatus,
    refetchInterval: 10000,
  })

  // Get host info for resource usage
  const { data: hostInfo } = useQuery({
    queryKey: ['hostInfo'],
    queryFn: api.getHostInfo,
    refetchInterval: 5000,
  })

  const vmCount = vms?.length || 0
  const runningCount = vms?.filter(vm => vm.state === 'running').length || 0
  const stoppedCount = vms?.filter(vm => vm.state === 'stopped').length || 0
  const pausedCount = vms?.filter(vm => vm.state === 'paused').length || 0

  // Calculate host memory usage
  const memoryUsedMb = hostInfo ? hostInfo.memoryTotalMb - hostInfo.memoryFreeMb : 0
  const memoryUsedPercent = hostInfo ? Math.round((memoryUsedMb / hostInfo.memoryTotalMb) * 100) : 0

  return (
    <div className="h-screen flex flex-col bg-[var(--window-bg)]">
      {/* Command Palette (Ctrl+K) */}
      <CommandPalette open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} />

      {/* Keyboard Shortcuts Dialog */}
      <KeyboardShortcutsDialog open={showShortcuts} onOpenChange={setShowShortcuts} />

      {/* Unified Toolbar with Connection Info */}
      <ToolbarContent onOpenCommandPalette={() => setCommandPaletteOpen(true)} />

      {/* Breadcrumb Navigation */}
      <Breadcrumbs />

      {/* Main Content Area */}
      <main className="flex-1 overflow-auto bg-[var(--window-bg)]">
        {children}
      </main>

      {/* Status Bar */}
      <StatusBar>
        {/* Connection Status */}
        <StatusItem variant={connectionStatus?.connected ? 'success' : 'error'}>
          {connectionStatus?.connected ? (
            <span className="flex items-center gap-1">
              <Wifi className="w-3 h-3" />
              Connected
            </span>
          ) : (
            <span className="flex items-center gap-1">
              <WifiOff className="w-3 h-3" />
              Disconnected
            </span>
          )}
        </StatusItem>

        {/* VM Counts */}
        <StatusItem>{vmCount} VMs</StatusItem>
        {runningCount > 0 && <StatusItem variant="success">{runningCount} Running</StatusItem>}
        {stoppedCount > 0 && <StatusItem variant="muted">{stoppedCount} Stopped</StatusItem>}
        {pausedCount > 0 && <StatusItem variant="warning">{pausedCount} Paused</StatusItem>}

        <StatusSpacer />

        {/* Host Resources */}
        {hostInfo && (
          <>
            <StatusItem variant={memoryUsedPercent > 90 ? 'error' : memoryUsedPercent > 75 ? 'warning' : 'muted'}>
              <span className="flex items-center gap-1">
                <MemoryStick className="w-3 h-3" />
                {(memoryUsedMb / 1024).toFixed(1)}/{(hostInfo.memoryTotalMb / 1024).toFixed(0)} GB
              </span>
            </StatusItem>
            <StatusItem variant="muted">
              <span className="flex items-center gap-1">
                <Cpu className="w-3 h-3" />
                {hostInfo.cpuCount} cores
              </span>
            </StatusItem>
          </>
        )}

        <StatusItem className="text-muted-foreground text-[10px]">⌘K for commands</StatusItem>
      </StatusBar>
    </div>
  )
}
