import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'

export interface MenuEventHandlers {
  // File menu
  onNewVm?: () => void
  onImportVm?: () => void
  onNewConnection?: () => void
  onCloseConnection?: () => void
  onPreferences?: () => void

  // Edit menu
  onVmDetails?: () => void
  onCloneVm?: () => void
  onRenameVm?: () => void
  onDeleteVm?: () => void
  onTakeSnapshot?: () => void
  onManageSnapshots?: () => void

  // View menu
  onRefresh?: () => void
  onToggleToolbar?: () => void
  onToggleStatusBar?: () => void

  // Actions menu
  onStartVm?: () => void
  onStopVm?: () => void
  onForceStopVm?: () => void
  onPauseVm?: () => void
  onResumeVm?: () => void
  onRebootVm?: () => void
  onOpenConsole?: () => void

  // Tools menu
  onStorageManager?: () => void
  onNetworkManager?: () => void
  onTemplates?: () => void
  onSchedules?: () => void
  onAlerts?: () => void
  onBackups?: () => void
  onPerformance?: () => void
  onOptimization?: () => void

  // Help menu
  onDocumentation?: () => void
  onKeyboardShortcuts?: () => void
  onCheckUpdates?: () => void
  onReportIssue?: () => void
  onAbout?: () => void
}

/**
 * Hook to listen for menu events from Tauri
 *
 * @param handlers - Object containing handler functions for menu events
 *
 * @example
 * ```tsx
 * useMenuEvents({
 *   onNewVm: () => setShowCreateWizard(true),
 *   onRefresh: () => queryClient.invalidateQueries(['vms']),
 * })
 * ```
 */
export function useMenuEvents(handlers: MenuEventHandlers) {
  useEffect(() => {
    const unlisteners: (() => void)[] = []

    // File menu
    if (handlers.onNewVm) {
      listen('menu-new-vm', handlers.onNewVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onImportVm) {
      listen('menu-import-vm', handlers.onImportVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onNewConnection) {
      listen('menu-new-connection', handlers.onNewConnection).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onCloseConnection) {
      listen('menu-close-connection', handlers.onCloseConnection).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onPreferences) {
      listen('menu-preferences', handlers.onPreferences).then(unlisten => unlisteners.push(unlisten))
    }

    // Edit menu
    if (handlers.onVmDetails) {
      listen('menu-vm-details', handlers.onVmDetails).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onCloneVm) {
      listen('menu-clone-vm', handlers.onCloneVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onRenameVm) {
      listen('menu-rename-vm', handlers.onRenameVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onDeleteVm) {
      listen('menu-delete-vm', handlers.onDeleteVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onTakeSnapshot) {
      listen('menu-take-snapshot', handlers.onTakeSnapshot).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onManageSnapshots) {
      listen('menu-manage-snapshots', handlers.onManageSnapshots).then(unlisten => unlisteners.push(unlisten))
    }

    // View menu
    if (handlers.onRefresh) {
      listen('menu-refresh', handlers.onRefresh).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onToggleToolbar) {
      listen('menu-toggle-toolbar', handlers.onToggleToolbar).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onToggleStatusBar) {
      listen('menu-toggle-statusbar', handlers.onToggleStatusBar).then(unlisten => unlisteners.push(unlisten))
    }

    // Actions menu
    if (handlers.onStartVm) {
      listen('menu-start-vm', handlers.onStartVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onStopVm) {
      listen('menu-stop-vm', handlers.onStopVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onForceStopVm) {
      listen('menu-force-stop-vm', handlers.onForceStopVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onPauseVm) {
      listen('menu-pause-vm', handlers.onPauseVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onResumeVm) {
      listen('menu-resume-vm', handlers.onResumeVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onRebootVm) {
      listen('menu-reboot-vm', handlers.onRebootVm).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onOpenConsole) {
      listen('menu-open-console', handlers.onOpenConsole).then(unlisten => unlisteners.push(unlisten))
    }

    // Tools menu
    if (handlers.onStorageManager) {
      listen('menu-storage-manager', handlers.onStorageManager).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onNetworkManager) {
      listen('menu-network-manager', handlers.onNetworkManager).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onTemplates) {
      listen('menu-templates', handlers.onTemplates).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onSchedules) {
      listen('menu-schedules', handlers.onSchedules).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onAlerts) {
      listen('menu-alerts', handlers.onAlerts).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onBackups) {
      listen('menu-backups', handlers.onBackups).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onPerformance) {
      listen('menu-performance', handlers.onPerformance).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onOptimization) {
      listen('menu-optimization', handlers.onOptimization).then(unlisten => unlisteners.push(unlisten))
    }

    // Help menu
    if (handlers.onDocumentation) {
      listen('menu-documentation', handlers.onDocumentation).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onKeyboardShortcuts) {
      listen('menu-keyboard-shortcuts', handlers.onKeyboardShortcuts).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onCheckUpdates) {
      listen('menu-check-updates', handlers.onCheckUpdates).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onReportIssue) {
      listen('menu-report-issue', handlers.onReportIssue).then(unlisten => unlisteners.push(unlisten))
    }
    if (handlers.onAbout) {
      listen('menu-about', handlers.onAbout).then(unlisten => unlisteners.push(unlisten))
    }

    // Cleanup
    return () => {
      unlisteners.forEach(unlisten => unlisten())
    }
  }, [handlers])
}
