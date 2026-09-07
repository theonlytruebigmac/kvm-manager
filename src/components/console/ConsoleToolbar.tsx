import { Maximize, Minimize, Camera, Send, ChevronDown, Settings2, Keyboard, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from '@/components/ui/dropdown-menu'
import { sendConsoleKey, sendCtrlAltDel, sendCtrlAltBackspace, sendCtrlAltFn, type ConsoleViewerRef, type ScaleMode } from './VncViewer'
import { toast } from 'sonner'

interface ConsoleToolbarProps {
  isFullscreen: boolean
  onToggleFullscreen: () => void
  onScreenshot: () => void
  vncViewerRef: React.RefObject<ConsoleViewerRef | null>
  vmName: string
  scaleMode: ScaleMode
  onScaleModeChange: (mode: ScaleMode) => void
  inputFocused: boolean
  onFocusInput: () => void
  onReconnect: () => void
  supportsSpecialKeys: boolean
  isConnected: boolean
}

export function ConsoleToolbar({
  isFullscreen,
  onToggleFullscreen,
  onScreenshot,
  vncViewerRef,
  vmName,
  scaleMode,
  onScaleModeChange,
  inputFocused,
  onFocusInput,
  onReconnect,
  supportsSpecialKeys,
  isConnected,
}: ConsoleToolbarProps) {
  const handleSendKey = (action: string) => {
    if (!vncViewerRef.current) {
      toast.error('Console not connected')
      return
    }

    switch (action) {
      case 'enter':
      case 'escape':
        toast[sendConsoleKey(vncViewerRef, action) ? 'success' : 'error'](
          supportsSpecialKeys ? `Sent ${action === 'enter' ? 'Enter' : 'Escape'}` : 'Key sending is unavailable for this console type',
        )
        break
      case 'ctrl-alt-del':
        toast[sendCtrlAltDel(vncViewerRef) ? 'success' : 'error'](
          supportsSpecialKeys ? 'Sent Ctrl+Alt+Delete' : 'Special-key sending is unavailable for this console type',
        )
        break
      case 'ctrl-alt-backspace':
        toast[sendCtrlAltBackspace(vncViewerRef) ? 'success' : 'error'](
          supportsSpecialKeys ? 'Sent Ctrl+Alt+Backspace' : 'Special-key sending is unavailable for this console type',
        )
        break
      case 'ctrl-alt-f1':
      case 'ctrl-alt-f2':
      case 'ctrl-alt-f3':
      case 'ctrl-alt-f4':
      case 'ctrl-alt-f5':
      case 'ctrl-alt-f6':
      case 'ctrl-alt-f7':
      case 'ctrl-alt-f8':
      case 'ctrl-alt-f9':
      case 'ctrl-alt-f10':
      case 'ctrl-alt-f11':
      case 'ctrl-alt-f12':
        const fnNum = parseInt(action.split('-f')[1])
        toast[sendCtrlAltFn(vncViewerRef, fnNum) ? 'success' : 'error'](
          supportsSpecialKeys ? `Sent Ctrl+Alt+F${fnNum}` : 'Special-key sending is unavailable for this console type',
        )
        break
      default:
        break
    }
  }

  return (
    <div className="flex min-w-0 flex-1 items-center justify-between gap-3">
      {/* Left: VM Name */}
      <div className="flex min-w-0 items-center gap-2 text-white">
        <div className={`h-2 w-2 shrink-0 rounded-full ${isConnected
          ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.65)]'
          : 'bg-slate-500'}`} />
        <span className="truncate text-sm font-medium">{vmName}</span>
        <span className="rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-slate-400">
          {isConnected ? 'Live' : 'Offline'}
        </span>
      </div>

      {/* Right: Actions */}
      <div className="flex shrink-0 items-center gap-1">
        <Button
          variant={inputFocused ? 'secondary' : 'default'}
          size="sm"
          onClick={onFocusInput}
          disabled={!isConnected}
          className={inputFocused ? 'bg-emerald-500/15 text-emerald-300 hover:bg-emerald-500/25' : ''}
          title="Focus the VM display so keyboard input is sent to the guest"
        >
          <Keyboard className="mr-1.5 h-4 w-4" />
          <span className="text-xs">{inputFocused ? 'Keyboard active' : 'Capture keyboard'}</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={onReconnect}
          className="text-gray-200 hover:bg-white/10 hover:text-white"
          title="Reconnect console"
          aria-label="Reconnect console"
        >
          <RefreshCw className="h-4 w-4" />
        </Button>

        {/* Screenshot */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onScreenshot}
          className="text-gray-200 hover:bg-white/10 hover:text-white"
          title="Take Screenshot (F10)"
          aria-label="Take console screenshot"
          disabled={!isConnected}
        >
          <Camera className="w-4 h-4" />
        </Button>

        {/* Send Keys Menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="text-gray-200 hover:bg-white/10 hover:text-white"
              title="Send Special Keys"
              disabled={!supportsSpecialKeys || !isConnected}
            >
              <Send className="w-4 h-4 mr-1" />
              <span className="text-xs">Send</span>
              <ChevronDown className="w-3 h-3 ml-1" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-56">
            <DropdownMenuItem onClick={() => handleSendKey('enter')}>
              <Send className="w-4 h-4 mr-2" />
              Enter
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('escape')}>
              <Send className="w-4 h-4 mr-2" />
              Escape
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-del')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+Delete
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-backspace')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+Backspace
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-f1')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+F1 (TTY1)
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-f2')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+F2 (TTY2)
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-f3')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+F3 (TTY3)
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-f4')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+F4 (TTY4)
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-f5')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+F5 (TTY5)
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleSendKey('ctrl-alt-f6')}>
              <Send className="w-4 h-4 mr-2" />
              Ctrl+Alt+F6 (TTY6)
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Scale Mode */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="text-gray-200 hover:bg-white/10 hover:text-white"
              title="Display Scale Mode"
              aria-label="Change display scaling"
            >
              <Settings2 className="w-4 h-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuRadioGroup value={scaleMode} onValueChange={(value) => onScaleModeChange(value as ScaleMode)}>
              <DropdownMenuRadioItem value="scale">
                Scale to Window
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="fit">
                1:1 Pixel Mapping
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="stretch">
                Stretch to Fill
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Fullscreen Toggle */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onToggleFullscreen}
          className="text-gray-200 hover:bg-white/10 hover:text-white"
          title="Toggle Fullscreen (F11)"
          aria-label={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}
        >
          {isFullscreen ? (
            <Minimize className="w-4 h-4" />
          ) : (
            <Maximize className="w-4 h-4" />
          )}
        </Button>
      </div>
    </div>
  )
}
