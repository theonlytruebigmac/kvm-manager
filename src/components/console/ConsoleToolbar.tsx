import { Maximize, Minimize, Camera, Send, ChevronDown, Settings2 } from 'lucide-react'
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
import { sendCtrlAltDel, sendCtrlAltBackspace, sendCtrlAltFn, type VncViewerRef, type ScaleMode } from './VncViewer'
import { toast } from 'sonner'

interface ConsoleToolbarProps {
  isFullscreen: boolean
  onToggleFullscreen: () => void
  onScreenshot: () => void
  vncViewerRef: React.RefObject<VncViewerRef | null>
  vmName: string
  scaleMode: ScaleMode
  onScaleModeChange: (mode: ScaleMode) => void
}

export function ConsoleToolbar({
  isFullscreen,
  onToggleFullscreen,
  onScreenshot,
  vncViewerRef,
  vmName,
  scaleMode,
  onScaleModeChange,
}: ConsoleToolbarProps) {
  const handleSendKey = (action: string) => {
    if (!vncViewerRef.current) {
      toast.error('Console not connected')
      return
    }

    switch (action) {
      case 'ctrl-alt-del':
        sendCtrlAltDel(vncViewerRef)
        toast.success('Sent Ctrl+Alt+Delete')
        break
      case 'ctrl-alt-backspace':
        sendCtrlAltBackspace(vncViewerRef)
        toast.success('Sent Ctrl+Alt+Backspace')
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
        sendCtrlAltFn(vncViewerRef, fnNum)
        toast.success(`Sent Ctrl+Alt+F${fnNum}`)
        break
      default:
        break
    }
  }

  return (
    <div
      className={`
        flex items-center justify-between px-4 py-2
        bg-gray-800 bg-opacity-90 border-b border-gray-700
        ${isFullscreen ? 'absolute top-0 left-0 right-0 z-50' : ''}
      `}
    >
      {/* Left: VM Name */}
      <div className="flex items-center gap-2 text-white">
        <div className="w-2 h-2 bg-green-500 rounded-full" />
        <span className="text-sm font-medium">{vmName}</span>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-2">
        {/* Screenshot */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onScreenshot}
          className="text-white hover:bg-gray-700"
          title="Take Screenshot (F10)"
        >
          <Camera className="w-4 h-4" />
        </Button>

        {/* Send Keys Menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="text-white hover:bg-gray-700"
              title="Send Special Keys"
            >
              <Send className="w-4 h-4 mr-1" />
              <span className="text-xs">Send</span>
              <ChevronDown className="w-3 h-3 ml-1" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-56">
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
              className="text-white hover:bg-gray-700"
              title="Display Scale Mode"
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
          className="text-white hover:bg-gray-700"
          title="Toggle Fullscreen (F11)"
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
