import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'

interface KeyboardShortcut {
  keys: string[]
  description: string
  category: string
}

const shortcuts: KeyboardShortcut[] = [
  // General
  { keys: ['Ctrl', 'K'], description: 'Open Command Palette', category: 'General' },
  { keys: ['Ctrl', 'N'], description: 'Create new VM', category: 'General' },
  { keys: ['Ctrl', 'R'], description: 'Refresh VM list', category: 'General' },
  { keys: ['Ctrl', ','], description: 'Open Settings', category: 'General' },
  { keys: ['Ctrl', '?'], description: 'Show keyboard shortcuts', category: 'General' },

  // VM Control
  { keys: ['Ctrl', 'P'], description: 'Start/Play selected VM', category: 'VM Control' },
  { keys: ['Ctrl', 'S'], description: 'Stop selected VM', category: 'VM Control' },
  { keys: ['Ctrl', 'Shift', 'S'], description: 'Force stop selected VM', category: 'VM Control' },
  { keys: ['Ctrl', 'Shift', 'P'], description: 'Pause/Resume selected VM', category: 'VM Control' },
  { keys: ['Ctrl', 'Shift', 'R'], description: 'Reboot selected VM', category: 'VM Control' },
  { keys: ['Delete'], description: 'Delete selected VM', category: 'VM Control' },

  // Windows & Navigation
  { keys: ['Ctrl', 'D'], description: 'Open VM Details window', category: 'Windows' },
  { keys: ['Ctrl', 'O'], description: 'Open Console window', category: 'Windows' },
  { keys: ['Ctrl', 'W'], description: 'Close current window', category: 'Windows' },
  { keys: ['Enter'], description: 'Open details for selected VM', category: 'Windows' },

  // Selection
  { keys: ['Ctrl', 'A'], description: 'Select all VMs', category: 'Selection' },
  { keys: ['Esc'], description: 'Clear selection', category: 'Selection' },
  { keys: ['↑'], description: 'Select previous VM', category: 'Selection' },
  { keys: ['↓'], description: 'Select next VM', category: 'Selection' },

  // Search
  { keys: ['Ctrl', 'F'], description: 'Focus search box', category: 'Search' },
]

interface KeyboardShortcutsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function KeyboardShortcutsDialog({ open, onOpenChange }: KeyboardShortcutsDialogProps) {
  const categories = Array.from(new Set(shortcuts.map(s => s.category)))

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Keyboard Shortcuts</DialogTitle>
          <DialogDescription>
            Use these keyboard shortcuts to navigate and control VMs quickly
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 mt-4">
          {categories.map(category => (
            <div key={category}>
              <h3 className="font-semibold text-sm mb-3 text-foreground">
                {category}
              </h3>
              <div className="space-y-1.5">
                {shortcuts
                  .filter(s => s.category === category)
                  .map((shortcut, idx) => (
                    <div
                      key={idx}
                      className="flex items-center justify-between py-2 px-3 rounded hover:bg-muted/30 text-desktop-sm"
                    >
                      <span className="text-foreground">{shortcut.description}</span>
                      <div className="flex gap-1">
                        {shortcut.keys.map((key, i) => (
                          <span key={i} className="flex items-center gap-1">
                            <Badge variant="secondary" className="font-mono text-xs px-2 py-0.5">
                              {key}
                            </Badge>
                            {i < shortcut.keys.length - 1 && (
                              <span className="text-muted-foreground text-xs">+</span>
                            )}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
              </div>
            </div>
          ))}
        </div>

        <div className="mt-6 pt-4 border-t text-xs text-muted-foreground">
          <p>
            💡 <strong>Tip:</strong> Hover over toolbar buttons to see their keyboard shortcuts
          </p>
        </div>
      </DialogContent>
    </Dialog>
  )
}
