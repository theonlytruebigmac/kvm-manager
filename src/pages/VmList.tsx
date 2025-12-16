import { useState, useMemo, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { fuzzySearch } from '@/lib/fuzzySearch'
import { VmTable } from '@/components/vm/VmTable'
import { VmCardView } from '@/components/vm/VmCardView'
import { CreateVmWizard } from '@/components/vm/CreateVmWizard'
import { RenameVmDialog } from '@/components/vm/RenameVmDialog'
import { DeleteVmDialog } from '@/components/vm/DeleteVmDialog'
import { MigrationDialog } from '@/components/vm/MigrationDialog'
import { KeyboardShortcutsDialog } from '@/components/ui/keyboard-shortcuts-dialog'
import { PageContainer, PageContent } from '@/components/layout/PageContainer'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Label } from '@/components/ui/label'
import { SkeletonVmList } from '@/components/ui/skeleton'
import { ErrorState } from '@/components/ui/error-state'
import { Plus, Search, X, Monitor } from 'lucide-react'
import { useVmEvents } from '@/hooks/useVmEvents'
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts'
import { useMenuEvents } from '@/hooks/useMenuEvents'
import { useToolbarStore } from '@/hooks/useToolbarActions'
import type { VM } from '@/lib/types'

export function VmList() {
  // Listen for VM state change events
  useVmEvents()

  const navigate = useNavigate()
  const queryClient = useQueryClient()

  // Use toolbar store for dialogs and selection
  const {
    showCreateVm,
    setShowCreateVm,
    showImportVm,
    setShowImportVm,
    selectedVmIds,
    setSelectedVmIds,
    focusedVmId,
    stateFilter,
    viewMode,
    sortField,
    sortDirection,
  } = useToolbarStore()

  const [showShortcuts, setShowShortcuts] = useState(false)
  const [importXml, setImportXml] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedTag, setSelectedTag] = useState<string | null>(null)
  const [renameVm, setRenameVm] = useState<VM | null>(null)
  const [deleteVm, setDeleteVm] = useState<VM | null>(null)
  const [migrateVm, setMigrateVm] = useState<VM | null>(null)
  const searchInputRef = useRef<HTMLInputElement>(null)

  const { data: vms, isLoading, error } = useQuery({
    queryKey: ['vms'],
    queryFn: api.getVms,
    refetchInterval: 5000, // Poll every 5 seconds
  })

  console.log('VmList render:', { vms, isLoading, error, vmsLength: vms?.length })

  // Bulk operations mutations
  const startMutation = useMutation({
    mutationFn: (vmId: string) => api.startVm(vmId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['vms'] }),
    onError: (error) => {
      const errorMsg = String(error)
      if (errorMsg.includes('Permission denied') || errorMsg.includes('Could not open')) {
        toast.error('Failed to start: Permission denied. Check that libvirt can access all disk/ISO files. See VM details for help.')
      } else {
        toast.error(`Failed to start: ${error}`)
      }
    }
  })

  const stopMutation = useMutation({
    mutationFn: (vmId: string) => api.forceStopVm(vmId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['vms'] }),
  })

  const importMutation = useMutation({
    mutationFn: (xml: string) => api.importVm(xml),
    onSuccess: (vmId) => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM imported successfully (ID: ${vmId})`)
      setShowImportVm(false)
      setImportXml('')
    },
    onError: (error) => {
      toast.error(`Failed to import VM: ${error}`)
    },
  })

  // Get all unique tags across all VMs
  const allTags = useMemo(() => {
    if (!vms) return []
    const tagSet = new Set<string>()
    vms.forEach((vm: VM) => {
      vm.tags?.forEach((tag: string) => tagSet.add(tag))
    })
    return Array.from(tagSet).sort()
  }, [vms])

  // Filter VMs based on search, tag, and state filter with fuzzy search
  const filteredVms = useMemo(() => {
    if (!vms) return []

    // First apply state and tag filters
    let filtered = vms.filter((vm: VM) => {
      // Filter by state
      if (stateFilter !== 'all' && vm.state !== stateFilter) {
        return false
      }
      // Filter by selected tag
      if (selectedTag && !vm.tags?.includes(selectedTag)) {
        return false
      }
      return true
    })

    // Apply fuzzy search if there's a query
    if (searchQuery.trim()) {
      const results = fuzzySearch(filtered, searchQuery, (vm: VM) => [
        vm.name,
        vm.osType || '',
        ...(vm.tags || []),
        ...(vm.networkInterfaces?.map(n => n.ipAddress || '') || [])
      ])
      filtered = results.map(r => r.item)
    }

    // Apply sorting
    const sortMultiplier = sortDirection === 'asc' ? 1 : -1
    filtered.sort((a: VM, b: VM) => {
      let comparison = 0
      switch (sortField) {
        case 'name':
          comparison = a.name.localeCompare(b.name)
          break
        case 'state':
          // Order: running > paused > stopped
          const stateOrder: Record<string, number> = { running: 3, paused: 2, stopped: 1 }
          comparison = (stateOrder[a.state] || 0) - (stateOrder[b.state] || 0)
          break
        case 'cpu':
          // Sort by CPU count (we don't have live CPU usage at list level)
          comparison = (a.cpuCount || 0) - (b.cpuCount || 0)
          break
        case 'memory':
          // Sort by memory allocation
          comparison = (a.memoryMb || 0) - (b.memoryMb || 0)
          break
        case 'disk':
          // Sort by disk size
          comparison = (a.diskSizeGb || 0) - (b.diskSizeGb || 0)
          break
        default:
          comparison = a.name.localeCompare(b.name)
      }
      return comparison * sortMultiplier
    })

    return filtered
  }, [vms, searchQuery, selectedTag, stateFilter, sortField, sortDirection])

  // Selection helpers
  const clearSelection = () => setSelectedVmIds([])

  // Get the focused VM (first selected, or first in list)
  const selectedVms = filteredVms.filter((vm: VM) => selectedVmIds.includes(vm.id))
  const focusedVm = focusedVmId
    ? filteredVms.find((vm: VM) => vm.id === focusedVmId)
    : selectedVms[0] || filteredVms[0]

  // Handle double-click to open VM details in new window
  const handleVmDoubleClick = async (vmId: string) => {
    const vm = vms?.find(v => v.id === vmId)
    if (!vm) return

    try {
      await api.openVmDetailsWindow(vm.id, vm.name)
    } catch (error) {
      toast.error(`Failed to open VM details: ${(error as Error).message}`)
    }
  }

  // Menu event handlers
  useMenuEvents({
    // File menu
    onNewVm: () => setShowCreateVm(true),
    onImportVm: () => setShowImportVm(true),

    // Edit menu (context-aware - operate on focused VM)
    onVmDetails: () => {
      if (focusedVm) handleVmDoubleClick(focusedVm.id)
    },
    onCloneVm: async () => {
      if (!focusedVm) return
      try {
        const newName = `${focusedVm.name}-clone`
        await api.cloneVm(focusedVm.id, newName)
        toast.success(`VM cloned as ${newName}`)
        queryClient.invalidateQueries({ queryKey: ['vms'] })
      } catch (error) {
        toast.error(`Failed to clone VM: ${(error as Error).message}`)
      }
    },

    // View menu
    onRefresh: () => queryClient.invalidateQueries({ queryKey: ['vms'] }),

    // Actions menu (context-aware - operate on focused VM)
    onStartVm: () => {
      if (focusedVm && focusedVm.state !== 'running') {
        startMutation.mutate(focusedVm.id)
      }
    },
    onStopVm: () => {
      if (focusedVm && focusedVm.state === 'running') {
        stopMutation.mutate(focusedVm.id)
      }
    },
    onOpenConsole: async () => {
      if (!focusedVm) return
      try {
        await api.openConsoleWindow(focusedVm.id, focusedVm.name)
      } catch (error) {
        toast.error(`Failed to open console: ${(error as Error).message}`)
      }
    },

    // Tools menu
    onStorageManager: () => navigate('/storage'),
    onNetworkManager: () => navigate('/networks'),
    onTemplates: () => navigate('/templates'),
    onSchedules: () => navigate('/schedules'),
    onAlerts: () => navigate('/alerts'),
    onBackups: () => navigate('/backups'),

    // Help menu
    onKeyboardShortcuts: () => setShowShortcuts(true),
  })

  // Keyboard shortcuts
  useKeyboardShortcuts([
    {
      key: 'n',
      ctrlKey: true,
      handler: () => setShowCreateVm(true),
      description: 'Create new VM'
    },
    {
      key: 'p',
      ctrlKey: true,
      handler: () => {
        if (focusedVm && focusedVm.state === 'stopped') {
          startMutation.mutate(focusedVm.id)
        }
      },
      description: 'Start focused VM'
    },
    {
      key: 's',
      ctrlKey: true,
      handler: () => {
        if (focusedVm && focusedVm.state === 'running') {
          stopMutation.mutate(focusedVm.id)
        }
      },
      description: 'Stop focused VM'
    },
    {
      key: 'o',
      ctrlKey: true,
      handler: () => {
        if (focusedVm) {
          handleVmDoubleClick(focusedVm.id)
        }
      },
      description: 'Open focused VM details'
    },
    {
      key: 'Escape',
      handler: () => clearSelection(),
      description: 'Clear selection'
    },
    {
      key: '?',
      handler: () => setShowShortcuts(true),
      description: 'Show keyboard shortcuts'
    },
    {
      key: 'f',
      ctrlKey: true,
      handler: () => {
        searchInputRef.current?.focus()
        searchInputRef.current?.select()
      },
      description: 'Focus search field'
    }
  ])

  // Early returns AFTER all hooks
  if (isLoading) {
    return (
      <PageContainer>
        <PageContent>
          <SkeletonVmList count={6} />
        </PageContent>
      </PageContainer>
    )
  }

  if (error) {
    const errorMsg = String(error)
    let suggestion = 'Ensure libvirt daemon is running and accessible.'

    if (errorMsg.includes('Permission denied')) {
      suggestion = 'Add your user to the libvirt group: sudo usermod -aG libvirt $USER'
    } else if (errorMsg.includes('Connection refused')) {
      suggestion = 'Start libvirt daemon: sudo systemctl start libvirtd'
    }

    return (
      <PageContainer>
        <PageContent>
          <ErrorState
            title="Cannot Load Virtual Machines"
            message={errorMsg}
            suggestion={suggestion}
            onRetry={() => queryClient.invalidateQueries({ queryKey: ['vms'] })}
          />
        </PageContent>
      </PageContainer>
    )
  }

  if (!vms || vms.length === 0) {
    return (
      <>
        <div className="h-full flex items-center justify-center p-6">
          <div className="text-center space-y-8 max-w-2xl">
            {/* Hero section */}
            <div className="space-y-3">
              <div className="w-16 h-16 mx-auto rounded-full bg-primary/10 flex items-center justify-center">
                <Monitor className="w-8 h-8 text-primary" />
              </div>
              <h2 className="text-2xl font-semibold">Welcome to KVM Manager</h2>
              <p className="text-muted-foreground">
                Create and manage virtual machines with a modern, intuitive interface
              </p>
            </div>

            {/* Quick actions */}
            <div className="flex flex-col sm:flex-row items-center justify-center gap-3">
              <Button size="lg" onClick={() => setShowCreateVm(true)}>
                <Plus className="w-4 h-4 mr-2" />
                Create Your First VM
              </Button>
              <Button size="lg" variant="outline" onClick={() => setShowImportVm(true)}>
                Import Existing VM
              </Button>
            </div>

            {/* Quick start guide */}
            <div className="grid sm:grid-cols-3 gap-4 pt-4">
              <div className="p-4 rounded-lg border bg-card text-left space-y-2">
                <div className="w-8 h-8 rounded-full bg-blue-500/10 flex items-center justify-center">
                  <span className="text-blue-500 font-semibold text-sm">1</span>
                </div>
                <h3 className="font-medium">Prepare an ISO</h3>
                <p className="text-sm text-muted-foreground">
                  Download an installation ISO for your preferred operating system (Linux, Windows, etc.)
                </p>
              </div>
              <div className="p-4 rounded-lg border bg-card text-left space-y-2">
                <div className="w-8 h-8 rounded-full bg-green-500/10 flex items-center justify-center">
                  <span className="text-green-500 font-semibold text-sm">2</span>
                </div>
                <h3 className="font-medium">Create a VM</h3>
                <p className="text-sm text-muted-foreground">
                  Use the wizard to configure CPU, memory, storage, and attach your ISO
                </p>
              </div>
              <div className="p-4 rounded-lg border bg-card text-left space-y-2">
                <div className="w-8 h-8 rounded-full bg-purple-500/10 flex items-center justify-center">
                  <span className="text-purple-500 font-semibold text-sm">3</span>
                </div>
                <h3 className="font-medium">Start & Connect</h3>
                <p className="text-sm text-muted-foreground">
                  Boot the VM and use the built-in console to complete the installation
                </p>
              </div>
            </div>

            {/* Tips */}
            <div className="text-xs text-muted-foreground space-y-1 pt-2">
              <p>💡 Tip: Press <kbd className="px-1.5 py-0.5 rounded bg-muted font-mono">Ctrl+K</kbd> to open the Command Palette</p>
              <p>💡 Tip: Drag & drop an ISO file onto a VM card to mount it quickly</p>
            </div>
          </div>
        </div>
        {showCreateVm && <CreateVmWizard onClose={() => setShowCreateVm(false)} />}
      </>
    )
  }

  return (
    <>
      <PageContainer>
        {/* Unified toolbar: Search + Filters + Selection Info */}
        <div className="flex-shrink-0 px-4 py-2.5 border-b border-[var(--panel-border)] bg-[var(--window-bg)]">
          <div className="flex items-center gap-3">
            {/* Search */}
            <div className="relative flex-1 max-w-sm">
              <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
              <Input
                ref={searchInputRef}
                placeholder="Search VMs by name or OS..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-8 h-8 text-desktop-sm"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  className="absolute right-2.5 top-1/2 -translate-y-1/2"
                >
                  <X className="h-3.5 w-3.5 text-muted-foreground hover:text-foreground" />
                </button>
              )}
            </div>

            {/* Tag filter pills */}
            {allTags.length > 0 && (
              <div className="flex items-center gap-1.5">
                {allTags.slice(0, 4).map(tag => (
                  <Badge
                    key={tag}
                    variant={selectedTag === tag ? "default" : "outline"}
                    className="cursor-pointer h-6 text-desktop-xs px-2"
                    onClick={() => setSelectedTag(selectedTag === tag ? null : tag)}
                  >
                    {tag}
                  </Badge>
                ))}
                {selectedTag && (
                  <button
                    onClick={() => setSelectedTag(null)}
                    className="text-xs text-muted-foreground hover:text-foreground ml-1"
                  >
                    Clear
                  </button>
                )}
              </div>
            )}

            {/* Stats & Selection info */}
            <div className="flex items-center gap-3 ml-auto text-desktop-sm text-muted-foreground">
              {selectedVmIds.length > 0 ? (
                <>
                  <span className="font-medium text-foreground">{selectedVmIds.length} selected</span>
                  <button
                    onClick={clearSelection}
                    className="text-primary hover:underline"
                  >
                    Clear
                  </button>
                </>
              ) : (
                <span>{filteredVms.length} VMs</span>
              )}
            </div>
          </div>
        </div>

        <PageContent noPadding>
        {filteredVms.length === 0 ? (
          <div className="flex flex-col items-center justify-center min-h-[300px] gap-3">
            <p className="text-muted-foreground">
              {searchQuery || selectedTag ? 'No VMs match your filters' : 'No VMs found'}
            </p>
            {(searchQuery || selectedTag) && (
              <Button
                variant="outline"
                onClick={() => {
                  setSearchQuery('')
                  setSelectedTag(null)
                }}
              >
                Clear Filters
              </Button>
            )}
          </div>
        ) : viewMode === 'cards' ? (
          <VmCardView
            vms={filteredVms}
            selectedIds={selectedVmIds}
            onSelectionChange={setSelectedVmIds}
            onVmDoubleClick={handleVmDoubleClick}
            onOpenRename={(vmId) => {
              const vm = vms?.find(v => v.id === vmId)
              if (vm) setRenameVm(vm)
            }}
            onOpenDelete={(vmId) => {
              const vm = vms?.find(v => v.id === vmId)
              if (vm) setDeleteVm(vm)
            }}
            onOpenMigrate={(vmId) => {
              const vm = vms?.find(v => v.id === vmId)
              if (vm) setMigrateVm(vm)
            }}
          />
        ) : (
          <VmTable
            vms={filteredVms}
            selectedIds={selectedVmIds}
            onSelectionChange={setSelectedVmIds}
            onVmDoubleClick={handleVmDoubleClick}
            onOpenRename={(vmId) => {
              const vm = vms?.find(v => v.id === vmId)
              if (vm) setRenameVm(vm)
            }}
            onOpenDelete={(vmId) => {
              const vm = vms?.find(v => v.id === vmId)
              if (vm) setDeleteVm(vm)
            }}
            onOpenMigrate={(vmId) => {
              const vm = vms?.find(v => v.id === vmId)
              if (vm) setMigrateVm(vm)
            }}
          />
        )}
        </PageContent>
      </PageContainer>

      {showCreateVm && <CreateVmWizard onClose={() => setShowCreateVm(false)} />}
      <KeyboardShortcutsDialog open={showShortcuts} onOpenChange={setShowShortcuts} />

      {/* Import VM Dialog */}
      <AlertDialog open={showImportVm} onOpenChange={setShowImportVm}>
        <AlertDialogContent className="max-w-2xl">
          <AlertDialogHeader>
            <AlertDialogTitle>Import Virtual Machine</AlertDialogTitle>
            <AlertDialogDescription>
              Import a VM from its XML configuration. Paste the exported configuration below.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="import-xml">VM Configuration (XML)</Label>
              <textarea
                id="import-xml"
                className="w-full h-64 px-3 py-2 text-sm border rounded-md font-mono resize-none"
                placeholder="Paste VM XML configuration here..."
                value={importXml}
                onChange={(e) => setImportXml(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                You can also import from file by pasting its contents here. Make sure disk images are accessible at the paths specified in the XML.
              </p>
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setImportXml('')}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => importMutation.mutate(importXml)}
              disabled={!importXml.trim() || importMutation.isPending}
            >
              {importMutation.isPending ? 'Importing...' : 'Import VM'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Rename VM Dialog */}
      <RenameVmDialog
        vm={renameVm}
        open={!!renameVm}
        onOpenChange={(open) => !open && setRenameVm(null)}
      />

      {/* Delete VM Dialog */}
      <DeleteVmDialog
        vm={deleteVm}
        open={!!deleteVm}
        onOpenChange={(open) => !open && setDeleteVm(null)}
      />

      {/* Migration Dialog */}
      {migrateVm && (
        <MigrationDialog
          vmId={migrateVm.id}
          vmName={migrateVm.name}
          isRunning={migrateVm.state === 'running'}
          open={!!migrateVm}
          onOpenChange={(open) => !open && setMigrateVm(null)}
        />
      )}
    </>
  )
}
