import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { EnhancedVmRow } from '@/components/vm/EnhancedVmRow'
import { CreateVmWizard } from '@/components/vm/CreateVmWizard'
import { KeyboardShortcutsDialog } from '@/components/ui/keyboard-shortcuts-dialog'
import { BatchOperations } from '@/components/vm/BatchOperations'
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
import { AlertCircle, Loader2, Plus, Search, X, Keyboard, Upload } from 'lucide-react'
import { useVmEvents } from '@/hooks/useVmEvents'
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts'
import type { VM } from '@/lib/types'

export function VmList() {
  // Listen for VM state change events
  useVmEvents()

  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [showCreateWizard, setShowCreateWizard] = useState(false)
  const [showShortcuts, setShowShortcuts] = useState(false)
  const [showImportDialog, setShowImportDialog] = useState(false)
  const [importXml, setImportXml] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedTag, setSelectedTag] = useState<string | null>(null)
  const [selectedVmIds, setSelectedVmIds] = useState<Set<string>>(new Set())
  const [focusedVmId, setFocusedVmId] = useState<string | null>(null)

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
  })

  const stopMutation = useMutation({
    mutationFn: (vmId: string) => api.forceStopVm(vmId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['vms'] }),
  })

  const deleteMutation = useMutation({
    mutationFn: (vmId: string) => api.deleteVm(vmId, true),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
    },
  })

  const importMutation = useMutation({
    mutationFn: (xml: string) => api.importVm(xml),
    onSuccess: (vmId) => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`VM imported successfully (ID: ${vmId})`)
      setShowImportDialog(false)
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

  // Filter VMs based on search and tag
  const filteredVms = useMemo(() => {
    if (!vms) return []
    return vms.filter((vm: VM) => {
      // Filter by search query
      if (searchQuery) {
        const query = searchQuery.toLowerCase()
        if (!vm.name.toLowerCase().includes(query) &&
            !vm.osType?.toLowerCase().includes(query)) {
          return false
        }
      }
      // Filter by selected tag
      if (selectedTag && !vm.tags?.includes(selectedTag)) {
        return false
      }
      return true
    })
  }, [vms, searchQuery, selectedTag])

  // Selection helpers
  const toggleVmSelection = (vmId: string) => {
    setSelectedVmIds(prev => {
      const next = new Set(prev)
      if (next.has(vmId)) {
        next.delete(vmId)
      } else {
        next.add(vmId)
      }
      return next
    })
  }

  const toggleAllSelection = () => {
    if (selectedVmIds.size === filteredVms.length) {
      setSelectedVmIds(new Set())
    } else {
      setSelectedVmIds(new Set(filteredVms.map((vm: VM) => vm.id)))
    }
  }

  const clearSelection = () => setSelectedVmIds(new Set())

  // Get the focused VM (first selected, or first in list)
  const selectedVms = filteredVms.filter((vm: VM) => selectedVmIds.has(vm.id))

  // Get the focused VM (first selected, or first in list)
  const focusedVm = focusedVmId
    ? filteredVms.find((vm: VM) => vm.id === focusedVmId)
    : selectedVms[0] || filteredVms[0]

  // Keyboard shortcuts
  useKeyboardShortcuts([
    {
      key: 'n',
      ctrlKey: true,
      handler: () => setShowCreateWizard(true),
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
          navigate(`/vms/${focusedVm.id}`)
        }
      },
      description: 'Open focused VM details'
    },
    {
      key: 'd',
      ctrlKey: true,
      handler: () => {
        if (focusedVm && confirm(`Delete ${focusedVm.name}?`)) {
          deleteMutation.mutate(focusedVm.id)
        }
      },
      description: 'Delete focused VM'
    },
    {
      key: 'a',
      ctrlKey: true,
      handler: () => toggleAllSelection(),
      description: 'Select all VMs'
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
    }
  ])

  // Early returns AFTER all hooks
  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="w-8 h-8 animate-spin text-primary" />
          <p className="text-sm text-muted-foreground">Loading virtual machines...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="flex flex-col items-center gap-3">
          <AlertCircle className="w-8 h-8 text-destructive" />
          <div className="text-center">
            <p className="font-medium">Error loading VMs</p>
            <p className="text-sm text-muted-foreground mt-1">{String(error)}</p>
          </div>
        </div>
      </div>
    )
  }

  if (!vms || vms.length === 0) {
    return (
      <>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center space-y-4">
            <div>
              <p className="font-medium">No virtual machines found</p>
              <p className="text-sm text-muted-foreground mt-1">
                Create your first VM to get started
              </p>
            </div>
            <Button onClick={() => setShowCreateWizard(true)}>
              <Plus className="w-4 h-4 mr-1" />
              Create VM
            </Button>
          </div>
        </div>
        {showCreateWizard && <CreateVmWizard onClose={() => setShowCreateWizard(false)} />}
      </>
    )
  }

  return (
    <>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Virtual Machines</h1>
            <p className="text-muted-foreground mt-1">
              Manage and monitor your VMs
            </p>
          </div>
          <div className="flex items-center gap-3">
            <div className="text-sm text-muted-foreground">
              {filteredVms.length} of {vms.length} {vms.length === 1 ? 'VM' : 'VMs'}
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setShowShortcuts(true)}
              title="Keyboard shortcuts (?)"
            >
              <Keyboard className="w-4 h-4" />
            </Button>
            <Button variant="outline" onClick={() => setShowImportDialog(true)}>
              <Upload className="w-4 h-4 mr-1" />
              Import VM
            </Button>
            <Button onClick={() => setShowCreateWizard(true)}>
              <Plus className="w-4 h-4 mr-1" />
              Create VM
            </Button>
          </div>
        </div>

        {/* Search and Filter */}
        <div className="flex flex-col gap-3">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search VMs by name or OS..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                className="absolute right-3 top-1/2 -translate-y-1/2"
              >
                <X className="h-4 w-4 text-muted-foreground hover:text-foreground" />
              </button>
            )}
          </div>

          {allTags.length > 0 && (
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-sm text-muted-foreground">Filter by tag:</span>
              <Badge
                variant={selectedTag === null ? "default" : "outline"}
                className="cursor-pointer"
                onClick={() => setSelectedTag(null)}
              >
                All
              </Badge>
              {allTags.map(tag => (
                <Badge
                  key={tag}
                  variant={selectedTag === tag ? "default" : "outline"}
                  className="cursor-pointer"
                  onClick={() => setSelectedTag(selectedTag === tag ? null : tag)}
                >
                  {tag}
                </Badge>
              ))}
            </div>
          )}
        </div>

        {/* Batch Operations Toolbar */}
        <BatchOperations
          selectedVmIds={Array.from(selectedVmIds)}
          onClearSelection={clearSelection}
        />

        {/* VM List */}
        <div>
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
        ) : (
          <div className="space-y-2">
            {/* Select All Header */}
            <div className="flex items-center gap-3 px-4 py-2 bg-muted/30 rounded-lg">
              <input
                type="checkbox"
                checked={selectedVmIds.size === filteredVms.length}
                onChange={toggleAllSelection}
                className="w-4 h-4 rounded border-gray-300 cursor-pointer"
              />
              <span className="text-sm font-medium text-muted-foreground">
                Select All ({filteredVms.length})
              </span>
            </div>

            {/* VM Rows */}
            {filteredVms.map((vm: VM) => (
              <EnhancedVmRow
                key={vm.id}
                vm={vm}
                isSelected={selectedVmIds.has(vm.id)}
                onToggleSelect={() => toggleVmSelection(vm.id)}
                isFocused={focusedVmId === vm.id}
                onFocus={() => setFocusedVmId(vm.id)}
              />
            ))}
          </div>
        )}
        </div>
      </div>

      {showCreateWizard && <CreateVmWizard onClose={() => setShowCreateWizard(false)} />}
      <KeyboardShortcutsDialog open={showShortcuts} onOpenChange={setShowShortcuts} />

      {/* Import VM Dialog */}
      <AlertDialog open={showImportDialog} onOpenChange={setShowImportDialog}>
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
    </>
  )
}
