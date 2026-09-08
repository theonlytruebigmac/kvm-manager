import { useEffect, useRef, useState, useCallback, forwardRef, useImperativeHandle } from 'react'
import { Loader2, WifiOff, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { localConsoleWebSocketUrl } from '@/lib/consoleSecurity'

// noVNC RFB class - loaded dynamically at runtime
let RFBClass: any = null
let noVNCLoadPromise: Promise<any> | null = null

async function loadNoVNC(): Promise<any> {
  // Already loaded
  if (RFBClass) return RFBClass

  // Already loading
  if (noVNCLoadPromise) return noVNCLoadPromise

  noVNCLoadPromise = new Promise((resolve, reject) => {
    // Check if already loaded (e.g., from a previous console window)
    if ((window as any).__noVNC_RFB__) {
      RFBClass = (window as any).__noVNC_RFB__
      console.log('noVNC RFB already available')
      resolve(RFBClass)
      return
    }

    // Load noVNC via external script (works better with Tauri CSP)
    const script = document.createElement('script')
    script.type = 'module'
    script.src = '/novnc-loader.js'

    const handleLoaded = () => {
      window.removeEventListener('novnc-loaded', handleLoaded)
      RFBClass = (window as any).__noVNC_RFB__
      if (RFBClass) {
        console.log('noVNC RFB loaded successfully')
        resolve(RFBClass)
      } else {
        reject(new Error('noVNC RFB not found after load'))
      }
    }

    window.addEventListener('novnc-loaded', handleLoaded)

    script.onerror = () => {
      window.removeEventListener('novnc-loaded', handleLoaded)
      reject(new Error('Failed to load noVNC script'))
    }

    document.head.appendChild(script)

    // Timeout after 10 seconds
    setTimeout(() => {
      if (!RFBClass) {
        window.removeEventListener('novnc-loaded', handleLoaded)
        reject(new Error('noVNC load timeout'))
      }
    }, 10000)
  })

  return noVNCLoadPromise
}

export type ScaleMode = 'scale' | 'fit' | 'stretch'

interface VncViewerProps {
  host: string
  port: number
  password?: string
  scaleMode?: ScaleMode
  onConnected?: () => void
  onDisconnected?: () => void
  onError?: (error: string) => void
  onInputFocusChange?: (focused: boolean) => void
  onKeySent?: () => void
}

export interface ConsoleViewerRef {
  reconnect: () => void
  setScaleMode: (mode: ScaleMode) => void
  getCanvas: () => HTMLCanvasElement | null
  focus: () => void
  blur: () => void
  sendCtrlAltDel?: () => void
  sendCtrlAltFn?: (fnNum: number) => void
  sendCtrlAltBackspace?: () => void
  sendKey?: (keysym: number, code: string) => void
}

export type VncViewerRef = ConsoleViewerRef

export const VncViewer = forwardRef<VncViewerRef, VncViewerProps>(({
  host,
  port,
  password,
  scaleMode = 'scale',
  onConnected,
  onDisconnected,
  onError,
  onInputFocusChange,
  onKeySent,
}, ref) => {
  const canvasRef = useRef<HTMLDivElement>(null)
  const rfbRef = useRef<any>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)
  const inputCaptureRequestedRef = useRef(false)
  const forwardedKeysRef = useRef(new Map<string, number>())

  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('connecting')
  const [errorMessage, setErrorMessage] = useState<string>('')
  const [reconnectAttempts, setReconnectAttempts] = useState(0)
  const [isReconnecting, setIsReconnecting] = useState(false)

  // Maximum reconnection attempts before giving up
  const MAX_RECONNECT_ATTEMPTS = 5

  // Calculate backoff delay (exponential: 1s, 2s, 4s, 8s, 16s)
  const getReconnectDelay = (attempt: number) => {
    return Math.min(1000 * Math.pow(2, attempt), 16000)
  }

  const connectToVnc = useCallback(async () => {
    if (!canvasRef.current) return

    // Dynamically load noVNC if not already loaded
    let RFB: any
    try {
      RFB = await loadNoVNC()
    } catch (_error) {
      setStatus('error')
      setErrorMessage('Failed to load VNC library.')
      onError?.('Failed to load VNC library')
      return
    }

    // Clean up existing connection
    if (rfbRef.current) {
      try {
        rfbRef.current.disconnect()
      } catch (e) {
        console.warn('Error disconnecting existing RFB:', e)
      }
      rfbRef.current = null
    }

    // Construct WebSocket URL for websockify proxy
    // Always use ws:// for local connections (websockify listens on localhost)
    let url: string
    try {
      url = localConsoleWebSocketUrl(host, port)
    } catch {
      setStatus('error')
      setErrorMessage('The local console proxy is unavailable.')
      onError?.('The local console proxy is unavailable.')
      return
    }

    console.log('Connecting to VNC via websockify:', url, `(attempt ${reconnectAttempts + 1})`)
    setStatus('connecting')

    try {
      // Create RFB instance using dynamically loaded noVNC
      const rfb = new RFB(canvasRef.current, url, {
        credentials: password ? { password } : undefined,
        wsProtocols: ['binary'],
      })

      rfbRef.current = rfb

      // Set up event handlers
      rfb.addEventListener('connect', () => {
        console.log('VNC connected')
        setStatus('connected')
        setReconnectAttempts(0)
        setIsReconnecting(false)
        onConnected?.()
        requestAnimationFrame(() => {
          if (rfbRef.current === rfb) {
            inputCaptureRequestedRef.current = true
            rfb.focus({ preventScroll: true })
            onInputFocusChange?.(true)
          }
        })
      })

      rfb.addEventListener('disconnect', (e: any) => {
        console.log('VNC disconnected:', e.detail)
        setStatus('disconnected')

        // Auto-reconnect if not intentional disconnect and under max attempts
        if (!e.detail.clean && reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
          const delay = getReconnectDelay(reconnectAttempts)
          console.log(`Will attempt reconnect in ${delay}ms...`)
          setIsReconnecting(true)
          setReconnectAttempts(prev => prev + 1)

          reconnectTimeoutRef.current = setTimeout(() => {
            connectToVnc()
          }, delay)
        } else {
          setIsReconnecting(false)
          onDisconnected?.()
        }
      })

      rfb.addEventListener('credentialsrequired', () => {
        console.error('VNC credentials required but not provided')
        setStatus('error')
        setErrorMessage('VNC password required but not provided')
        setIsReconnecting(false)
        onError?.('Credentials required')
      })

      rfb.addEventListener('securityfailure', (e: any) => {
        console.error('VNC security failure:', e.detail)
        setStatus('error')
        setErrorMessage(`Security failure: ${e.detail.reason}`)
        setIsReconnecting(false)
        onError?.(e.detail.reason)
      })

      // Set quality and compression
      rfb.qualityLevel = 6 // 0-9, higher is better quality
      rfb.compressionLevel = 2 // 0-9, higher is more compression

      // Set scale mode
      applyScaleMode(rfb, scaleMode)

      // Enable local cursor
      rfb.showDotCursor = false
      rfb.viewOnly = false
      rfb.focusOnClick = true

    } catch (error) {
      console.error('Failed to create VNC connection:', error)
      setStatus('error')
      setErrorMessage(error instanceof Error ? error.message : 'Connection failed')
      setIsReconnecting(false)
      onError?.(error instanceof Error ? error.message : 'Connection failed')
    }
  }, [host, port, password, scaleMode, reconnectAttempts, onConnected, onDisconnected, onError, onInputFocusChange])

  const applyScaleMode = (rfb: any, mode: ScaleMode) => {
    if (!rfb) return

    switch (mode) {
      case 'scale':
        rfb.scaleViewport = true
        rfb.resizeSession = false
        break
      case 'fit':
        rfb.scaleViewport = false
        rfb.resizeSession = false
        break
      case 'stretch':
        rfb.scaleViewport = true
        rfb.resizeSession = true
        break
    }
  }

  const handleManualReconnect = () => {
    setReconnectAttempts(0)
    setIsReconnecting(false)
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    connectToVnc()
  }

  // Expose methods to parent via ref
  useImperativeHandle(ref, () => ({
    reconnect: handleManualReconnect,
    setScaleMode: (mode: ScaleMode) => {
      if (rfbRef.current) {
        applyScaleMode(rfbRef.current, mode)
      }
    },
    getCanvas: () => {
      return canvasRef.current?.querySelector('canvas') || null
    },
    focus: () => {
      inputCaptureRequestedRef.current = true
      rfbRef.current?.focus({ preventScroll: true })
      onInputFocusChange?.(true)
    },
    blur: () => {
      inputCaptureRequestedRef.current = false
      rfbRef.current?.blur()
      onInputFocusChange?.(false)
    },
    sendCtrlAltDel: () => rfbRef.current?.sendCtrlAltDel(),
    sendCtrlAltFn: (fnNum: number) => sendCtrlAltFnToRfb(rfbRef.current, fnNum),
    sendCtrlAltBackspace: () => sendCtrlAltBackspaceToRfb(rfbRef.current),
    sendKey: (keysym: number, code: string) => rfbRef.current?.sendKey(keysym, code),
  }))

  // Initial connection
  useEffect(() => {
    connectToVnc()

    // Cleanup on unmount
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      if (rfbRef.current) {
        try {
          rfbRef.current.disconnect()
        } catch (e) {
          console.warn('Error disconnecting RFB on unmount:', e)
        }
        rfbRef.current = null
      }
    }
  }, [])

  // Update scale mode when prop changes
  useEffect(() => {
    if (rfbRef.current && status === 'connected') {
      applyScaleMode(rfbRef.current, scaleMode)
    }
  }, [scaleMode, status])

  // WebKitGTK can leave document focus on the webview even after canvas.focus(). When capture was
  // explicitly requested, translate the original event with noVNC's mapper and send it directly.
  useEffect(() => {
    if (status !== 'connected') return

    const releaseForwardedKeys = () => {
      for (const [code, keysym] of forwardedKeysRef.current) {
        rfbRef.current?.sendKey(keysym, code, false)
      }
      forwardedKeysRef.current.clear()
    }

    const forwardToRfb = (event: KeyboardEvent) => {
      if (!inputCaptureRequestedRef.current) return
      const canvas = canvasRef.current?.querySelector('canvas')
      if (!canvas) return
      if (event.target === canvas) {
        onKeySent?.()
        return
      }

      const mapper = (window as typeof window & {
        __noVNC_getKeysym__?: (keyboardEvent: KeyboardEvent) => number | null
      }).__noVNC_getKeysym__
      const code = event.code || `Platform${event.keyCode}`
      const down = event.type === 'keydown'
      const keysym = down ? mapper?.(event) : forwardedKeysRef.current.get(code) ?? mapper?.(event)
      if (keysym === undefined || keysym === null) return

      rfbRef.current?.sendKey(keysym, code, down)
      if (down) forwardedKeysRef.current.set(code, keysym)
      else forwardedKeysRef.current.delete(code)
      onKeySent?.()
      event.preventDefault()
      event.stopImmediatePropagation()
    }

    window.addEventListener('keydown', forwardToRfb, true)
    window.addEventListener('keyup', forwardToRfb, true)
    window.addEventListener('blur', releaseForwardedKeys)
    return () => {
      window.removeEventListener('keydown', forwardToRfb, true)
      window.removeEventListener('keyup', forwardToRfb, true)
      window.removeEventListener('blur', releaseForwardedKeys)
      releaseForwardedKeys()
    }
  }, [status, onKeySent])

  return (
    <div
      className="relative w-full h-full bg-gray-900 focus-within:ring-2 focus-within:ring-inset focus-within:ring-blue-500"
      role="application"
      aria-label="Interactive VM console"
      onMouseDown={() => {
        inputCaptureRequestedRef.current = true
        rfbRef.current?.focus({ preventScroll: true })
        onInputFocusChange?.(true)
      }}
      onFocusCapture={() => onInputFocusChange?.(true)}
      onBlurCapture={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
          inputCaptureRequestedRef.current = false
          onInputFocusChange?.(false)
        }
      }}
    >
      {/* VNC Canvas Container */}
      <div
        ref={canvasRef}
        className="w-full h-full"
        style={{ overflow: 'hidden' }}
      />

      {/* Connecting Overlay */}
      {status === 'connecting' && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-900 bg-opacity-90">
          <Loader2 className="w-12 h-12 text-blue-500 animate-spin mb-4" />
          <p className="text-white text-lg">Connecting to VM console...</p>
          <p className="text-gray-400 text-sm mt-2">{host}:{port}</p>
        </div>
      )}

      {/* Disconnected Overlay */}
      {status === 'disconnected' && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-900 bg-opacity-90">
          <WifiOff className="w-12 h-12 text-gray-500 mb-4" />
          <p className="text-white text-lg">Console disconnected</p>
          {isReconnecting ? (
            <>
              <p className="text-gray-400 text-sm mt-2">
                Reconnecting... (attempt {reconnectAttempts}/{MAX_RECONNECT_ATTEMPTS})
              </p>
              <Loader2 className="w-6 h-6 text-blue-500 animate-spin mt-4" />
            </>
          ) : (
            <>
              <p className="text-gray-400 text-sm mt-2">
                The VM console connection was closed
              </p>
              {reconnectAttempts >= MAX_RECONNECT_ATTEMPTS && (
                <p className="text-yellow-500 text-sm mt-2">
                  Maximum reconnection attempts reached
                </p>
              )}
              <Button
                onClick={handleManualReconnect}
                className="mt-4"
                variant="outline"
              >
                Reconnect
              </Button>
            </>
          )}
        </div>
      )}

      {/* Error Overlay */}
      {status === 'error' && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-900 bg-opacity-90">
          <AlertCircle className="w-12 h-12 text-red-500 mb-4" />
          <p className="text-white text-lg">Connection failed</p>
          <p className="text-gray-400 text-sm mt-2 max-w-md text-center">
            {errorMessage || 'Unable to connect to VM console'}
          </p>
          <div className="mt-6 space-y-2 text-sm text-gray-400 max-w-md">
            <p className="font-semibold text-white">Troubleshooting:</p>
            <ul className="list-disc list-inside space-y-1">
              <li>Ensure the VM is running</li>
              <li>Check that VNC graphics are enabled for this VM</li>
              <li>Verify firewall settings allow VNC connections</li>
              <li>Try restarting the VM</li>
            </ul>
          </div>
        </div>
      )}
    </div>
  )
})

VncViewer.displayName = 'VncViewer'

// Export helper function to send special keys
export function sendCtrlAltDel(vncViewerRef: React.RefObject<ConsoleViewerRef | null>) {
  if (!vncViewerRef.current?.sendCtrlAltDel) return false
  vncViewerRef.current.sendCtrlAltDel()
  vncViewerRef.current.focus()
  return true
}

// Keysym constants for special keys
const KEYSYMS = {
  XK_BackSpace: 0xff08,
  XK_F1: 0xffbe,
  XK_F2: 0xffbf,
  XK_F3: 0xffc0,
  XK_F4: 0xffc1,
  XK_F5: 0xffc2,
  XK_F6: 0xffc3,
  XK_F7: 0xffc4,
  XK_F8: 0xffc5,
  XK_F9: 0xffc6,
  XK_F10: 0xffc7,
  XK_F11: 0xffc8,
  XK_F12: 0xffc9,
}

function sendCtrlAltFnToRfb(rfb: any, fnNum: number) {
  if (!rfb || fnNum < 1 || fnNum > 12) return
  const keysym = KEYSYMS[`XK_F${fnNum}` as keyof typeof KEYSYMS]

  // Send Ctrl+Alt+Fn combination
  rfb.sendKey(0xffe3, 'ControlLeft', true)   // Ctrl down
  rfb.sendKey(0xffe9, 'AltLeft', true)       // Alt down
  rfb.sendKey(keysym, `F${fnNum}`, true)     // Fn down
  rfb.sendKey(keysym, `F${fnNum}`, false)    // Fn up
  rfb.sendKey(0xffe9, 'AltLeft', false)      // Alt up
  rfb.sendKey(0xffe3, 'ControlLeft', false)  // Ctrl up
}

export function sendCtrlAltFn(vncViewerRef: React.RefObject<ConsoleViewerRef | null>, fnNum: number) {
  if (!vncViewerRef.current?.sendCtrlAltFn || fnNum < 1 || fnNum > 12) return false
  vncViewerRef.current.sendCtrlAltFn(fnNum)
  vncViewerRef.current.focus()
  return true
}

function sendCtrlAltBackspaceToRfb(rfb: any) {
  if (!rfb) return
  // Send Ctrl+Alt+Backspace combination
  rfb.sendKey(0xffe3, 'ControlLeft', true)
  rfb.sendKey(0xffe9, 'AltLeft', true)
  rfb.sendKey(KEYSYMS.XK_BackSpace, 'Backspace', true)
  rfb.sendKey(KEYSYMS.XK_BackSpace, 'Backspace', false)
  rfb.sendKey(0xffe9, 'AltLeft', false)
  rfb.sendKey(0xffe3, 'ControlLeft', false)
}

export function sendCtrlAltBackspace(vncViewerRef: React.RefObject<ConsoleViewerRef | null>) {
  if (!vncViewerRef.current?.sendCtrlAltBackspace) return false
  vncViewerRef.current.sendCtrlAltBackspace()
  vncViewerRef.current.focus()
  return true
}

export function sendConsoleKey(
  viewerRef: React.RefObject<ConsoleViewerRef | null>,
  key: 'enter' | 'escape',
) {
  if (!viewerRef.current?.sendKey) return false
  const [keysym, code] = key === 'enter' ? [0xff0d, 'Enter'] : [0xff1b, 'Escape']
  viewerRef.current.sendKey(keysym, code)
  viewerRef.current.focus()
  return true
}
