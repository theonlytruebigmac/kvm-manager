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
  { keys: ['Ctrl', 'N'], description: 'Create new VM', category: 'General' },
  { keys: ['Ctrl', 'A'], description: 'Select all VMs', category: 'Selection' },
  { keys: ['Esc'], description: 'Clear selection', category: 'Selection' },
  { keys: ['Ctrl', 'P'], description: 'Start focused/selected VM', category: 'VM Control' },
  { keys: ['Ctrl', 'S'], description: 'Stop focused/selected VM', category: 'VM Control' },
  { keys: ['Ctrl', 'Z'], description: 'Pause focused VM', category: 'VM Control' },
  { keys: ['Ctrl', 'O'], description: 'Open VM details', category: 'Navigation' },
  { keys: ['Ctrl', 'D'], description: 'Delete focused VM', category: 'VM Control' },
  { keys: ['?'], description: 'Show keyboard shortcuts', category: 'Help' },
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
              <h3 className="font-semibold text-sm mb-3 text-muted-foreground">
                {category}
              </h3>
              <div className="space-y-2">
                {shortcuts
                  .filter(s => s.category === category)
                  .map((shortcut, idx) => (
                    <div
                      key={idx}
                      className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-muted/50"
                    >
                      <span className="text-sm">{shortcut.description}</span>
                      <div className="flex gap-1">
                        {shortcut.keys.map((key, i) => (
                          <span key={i} className="flex items-center gap-1">
                            <Badge variant="secondary" className="font-mono text-xs px-2">
                              {key}
                            </Badge>
                            {i < shortcut.keys.length - 1 && (
                              <span className="text-muted-foreground">+</span>
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
      </DialogContent>
    </Dialog>
  )
}
