import { Plus, Play, Square, Pause, Monitor, Trash2, Settings, RefreshCw } from "lucide-react"
import { Toolbar, ToolbarButton, ToolbarSeparator, ToolbarSpacer } from "./Toolbar"

/**
 * Example usage of the Toolbar component
 *
 * This demonstrates how to use the Toolbar components in a typical VM management context.
 */

export function ToolbarExample() {
  const handleNew = () => console.log("New")
  const handleStart = () => console.log("Start VM")
  const handlePause = () => console.log("Pause VM")
  const handleStop = () => console.log("Stop VM")
  const handleConsole = () => console.log("Open Console")
  const handleDelete = () => console.log("Delete VM")
  const handleRefresh = () => console.log("Refresh")
  const handleSettings = () => console.log("Settings")

  return (
    <div className="space-y-4 p-4">
      <div>
        <h3 className="text-lg font-semibold mb-2">Basic Toolbar</h3>
        <Toolbar>
          <ToolbarButton icon={Plus} onClick={handleNew}>
            New VM
          </ToolbarButton>
          <ToolbarButton icon={Play} onClick={handleStart}>
            Start
          </ToolbarButton>
          <ToolbarButton icon={Pause} onClick={handlePause}>
            Pause
          </ToolbarButton>
          <ToolbarButton icon={Square} onClick={handleStop}>
            Stop
          </ToolbarButton>
        </Toolbar>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-2">With Separators</h3>
        <Toolbar>
          <ToolbarButton icon={Plus} onClick={handleNew}>
            New VM
          </ToolbarButton>
          <ToolbarSeparator />
          <ToolbarButton icon={Play} onClick={handleStart}>
            Start
          </ToolbarButton>
          <ToolbarButton icon={Pause} onClick={handlePause}>
            Pause
          </ToolbarButton>
          <ToolbarButton icon={Square} onClick={handleStop}>
            Stop
          </ToolbarButton>
          <ToolbarSeparator />
          <ToolbarButton icon={Monitor} onClick={handleConsole}>
            Console
          </ToolbarButton>
          <ToolbarButton icon={Trash2} onClick={handleDelete}>
            Delete
          </ToolbarButton>
        </Toolbar>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-2">With Spacer (Right-aligned items)</h3>
        <Toolbar>
          <ToolbarButton icon={Plus} onClick={handleNew}>
            New VM
          </ToolbarButton>
          <ToolbarSeparator />
          <ToolbarButton icon={Play} onClick={handleStart}>
            Start
          </ToolbarButton>
          <ToolbarButton icon={Square} onClick={handleStop}>
            Stop
          </ToolbarButton>
          <ToolbarSpacer />
          <ToolbarButton icon={RefreshCw} onClick={handleRefresh} tooltip="Refresh list" />
          <ToolbarButton icon={Settings} onClick={handleSettings} tooltip="Settings" />
        </Toolbar>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-2">Icon-only Buttons with Tooltips</h3>
        <Toolbar>
          <ToolbarButton icon={Plus} onClick={handleNew} tooltip="Create new VM" />
          <ToolbarSeparator />
          <ToolbarButton icon={Play} onClick={handleStart} tooltip="Start VM" />
          <ToolbarButton icon={Pause} onClick={handlePause} tooltip="Pause VM" />
          <ToolbarButton icon={Square} onClick={handleStop} tooltip="Stop VM" />
          <ToolbarSeparator />
          <ToolbarButton icon={Monitor} onClick={handleConsole} tooltip="Open console" />
          <ToolbarButton icon={Trash2} onClick={handleDelete} tooltip="Delete VM" />
          <ToolbarSpacer />
          <ToolbarButton icon={RefreshCw} onClick={handleRefresh} tooltip="Refresh" />
          <ToolbarButton icon={Settings} onClick={handleSettings} tooltip="Settings" />
        </Toolbar>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-2">With Disabled State</h3>
        <Toolbar>
          <ToolbarButton icon={Plus} onClick={handleNew}>
            New VM
          </ToolbarButton>
          <ToolbarSeparator />
          <ToolbarButton icon={Play} onClick={handleStart} disabled>
            Start
          </ToolbarButton>
          <ToolbarButton icon={Pause} onClick={handlePause} disabled>
            Pause
          </ToolbarButton>
          <ToolbarButton icon={Square} onClick={handleStop}>
            Stop
          </ToolbarButton>
          <ToolbarSeparator />
          <ToolbarButton icon={Monitor} onClick={handleConsole}>
            Console
          </ToolbarButton>
        </Toolbar>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-2">Compact (Icon-only)</h3>
        <Toolbar>
          <ToolbarButton icon={Plus} onClick={handleNew} tooltip="New" />
          <ToolbarButton icon={Play} onClick={handleStart} tooltip="Start" />
          <ToolbarButton icon={Pause} onClick={handlePause} tooltip="Pause" />
          <ToolbarButton icon={Square} onClick={handleStop} tooltip="Stop" />
          <ToolbarSeparator />
          <ToolbarButton icon={Monitor} onClick={handleConsole} tooltip="Console" />
          <ToolbarButton icon={Trash2} onClick={handleDelete} tooltip="Delete" />
        </Toolbar>
      </div>
    </div>
  )
}
