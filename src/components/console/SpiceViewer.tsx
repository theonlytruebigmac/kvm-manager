import { useEffect, useRef, useState, useCallback, forwardRef, useImperativeHandle } from 'react'
import { Loader2, WifiOff, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'

// SpiceMainConn class - loaded dynamically at runtime
let SpiceMainConnClass: any = null
let spiceLoadPromise: Promise<any> | null = null

async function loadSpice(): Promise<any> {
  // Already loaded
  if (SpiceMainConnClass) return SpiceMainConnClass

  // Already loading
  if (spiceLoadPromise) return spiceLoadPromise

  spiceLoadPromise = new Promise((resolve, reject) => {
    // Check if already loaded (e.g., from a previous console window)
    if ((window as any).SpiceMainConn) {
      SpiceMainConnClass = (window as any).SpiceMainConn
      console.log('SpiceMainConn already available')
      resolve(SpiceMainConnClass)
      return
    }

    // Load SPICE via external script
    const script = document.createElement('script')
    script.src = '/spice-bundle.js'

    script.onload = () => {
      SpiceMainConnClass = (window as any).SpiceMainConn
      if (SpiceMainConnClass) {
        console.log('SPICE library loaded successfully')
        resolve(SpiceMainConnClass)
      } else {
        reject(new Error('SpiceMainConn not found after load'))
      }
    }

    script.onerror = () => {
      reject(new Error('Failed to load SPICE script'))
    }

    document.head.appendChild(script)

    // Timeout after 10 seconds
    setTimeout(() => {
      if (!SpiceMainConnClass) {
        reject(new Error('SPICE load timeout'))
      }
    }, 10000)
  })

  return spiceLoadPromise
}

export type ScaleMode = 'scale' | 'fit' | 'stretch'

interface SpiceViewerProps {
  host: string
  port: number
  password?: string
  scaleMode?: ScaleMode
  onConnected?: () => void
  onDisconnected?: () => void
  onError?: (error: string) => void
}

export interface SpiceViewerRef {
  reconnect: () => void
  setScaleMode: (mode: ScaleMode) => void
  getCanvas: () => HTMLCanvasElement | null
}

export const SpiceViewer = forwardRef<SpiceViewerRef, SpiceViewerProps>(({
  host,
  port,
  password,
  scaleMode = 'scale',
  onConnected,
  onDisconnected: _onDisconnected,
  onError,
}, ref) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const spiceRef = useRef<any>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('connecting')
  const [errorMessage, setErrorMessage] = useState<string>('')
  const [reconnectAttempts, setReconnectAttempts] = useState(0)
  const [isReconnecting, setIsReconnecting] = useState(false)

  const MAX_RECONNECT_ATTEMPTS = 5

  const getReconnectDelay = (attempt: number) => {
    return Math.min(1000 * Math.pow(2, attempt), 16000)
  }

  const connectToSpice = useCallback(async () => {
    if (!containerRef.current) return

    // Dynamically load SPICE library if not already loaded
    let SpiceMainConn: any
    try {
      SpiceMainConn = await loadSpice()
    } catch (e) {
      console.error('Failed to load SPICE library:', e)
      setStatus('error')
      setErrorMessage('Failed to load SPICE library.')
      onError?.('Failed to load SPICE library')
      return
    }

    // Clean up existing connection
    if (spiceRef.current) {
      try {
        spiceRef.current.stop()
      } catch (e) {
        console.warn('Error stopping existing SPICE connection:', e)
      }
      spiceRef.current = null
    }

    // Clear the container
    containerRef.current.innerHTML = ''

    // Create display container with unique ID
    const displayId = `spice-screen-${Date.now()}`
    const displayDiv = document.createElement('div')
    displayDiv.id = displayId
    displayDiv.style.width = '100%'
    displayDiv.style.height = '100%'
    containerRef.current.appendChild(displayDiv)

    // Construct WebSocket URL
    const uri = `ws://${host}:${port}`

    console.log('Connecting to SPICE via websockify:', uri, `(attempt ${reconnectAttempts + 1})`)
    setStatus('connecting')

    try {
      // Create SpiceMainConn instance
      const sc = new SpiceMainConn({
        uri: uri,
        password: password || '',
        screen_id: displayId,
        onerror: (e: any) => {
          console.error('SPICE error:', e)
          // Only set error if we haven't already connected
          if (spiceRef.current) {
            setStatus('error')
            const errMsg = e?.message || 'SPICE connection error'
            setErrorMessage(errMsg)
            onError?.(errMsg)

            // Attempt reconnection
            if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
              setIsReconnecting(true)
              const delay = getReconnectDelay(reconnectAttempts)
              reconnectTimeoutRef.current = setTimeout(() => {
                setReconnectAttempts(prev => prev + 1)
                connectToSpice()
              }, delay)
            }
          }
        },
        onagent: () => {
          console.log('SPICE agent connected')
          // Agent connected means full connection is established
        },
        onsuccess: () => {
          console.log('SPICE connection established')
          setStatus('connected')
          setReconnectAttempts(0)
          setIsReconnecting(false)
          onConnected?.()
        },
      })

      spiceRef.current = sc

      // SpiceMainConn may not call onsuccess, so we check for canvas creation
      // as a sign of successful connection
      const checkConnection = () => {
        const canvas = containerRef.current?.querySelector('canvas')
        if (canvas && spiceRef.current) {
          console.log('SPICE display canvas detected - connection successful')
          setStatus('connected')
          setReconnectAttempts(0)
          setIsReconnecting(false)
          onConnected?.()
        }
      }

      // Check multiple times for canvas creation
      setTimeout(checkConnection, 500)
      setTimeout(checkConnection, 1500)
      setTimeout(checkConnection, 3000)

    } catch (e: any) {
      console.error('Failed to create SPICE connection:', e)
      setStatus('error')
      setErrorMessage(e?.message || 'Failed to create SPICE connection')
      onError?.(e?.message || 'Failed to create SPICE connection')
    }
  }, [host, port, password, reconnectAttempts, onConnected, onError])

  // Initial connection
  useEffect(() => {
    connectToSpice()

    return () => {
      // Clear any pending reconnection
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }

      // Disconnect SPICE
      if (spiceRef.current) {
        try {
          spiceRef.current.stop()
        } catch (e) {
          console.warn('Error disconnecting SPICE:', e)
        }
        spiceRef.current = null
      }
    }
  }, [host, port, password])

  // Handle scale mode changes
  useEffect(() => {
    if (!containerRef.current) return

    const canvas = containerRef.current.querySelector('canvas')
    if (!canvas) return

    // Apply scaling based on mode
    switch (scaleMode) {
      case 'scale':
        canvas.style.width = '100%'
        canvas.style.height = '100%'
        canvas.style.objectFit = 'contain'
        break
      case 'fit':
        canvas.style.width = 'auto'
        canvas.style.height = 'auto'
        canvas.style.objectFit = 'none'
        break
      case 'stretch':
        canvas.style.width = '100%'
        canvas.style.height = '100%'
        canvas.style.objectFit = 'fill'
        break
    }
  }, [scaleMode, status])

  // Expose methods to parent
  useImperativeHandle(ref, () => ({
    reconnect: () => {
      setReconnectAttempts(0)
      connectToSpice()
    },
    setScaleMode: (_mode: ScaleMode) => {
      // Scale mode is handled by the effect above
    },
    getCanvas: () => {
      return containerRef.current?.querySelector('canvas') || null
    },
  }), [connectToSpice])

  // Render error state
  if (status === 'error' && !isReconnecting) {
    return (
      <div className="h-full flex flex-col items-center justify-center bg-gray-900 p-8">
        <AlertCircle className="w-16 h-16 text-red-500 mb-4" />
        <h3 className="text-xl font-semibold text-white mb-2">Connection failed</h3>
        <p className="text-gray-400 text-center mb-6 max-w-md">
          {errorMessage || 'Failed to connect to SPICE console'}
        </p>

        <div className="bg-gray-800 rounded-lg p-4 mb-6 max-w-md">
          <h4 className="text-sm font-medium text-gray-300 mb-2">Troubleshooting:</h4>
          <ul className="text-sm text-gray-400 space-y-1 list-disc list-inside">
            <li>Ensure the VM is running</li>
            <li>Check that SPICE graphics are enabled for this VM</li>
            <li>Verify firewall settings allow SPICE connections</li>
            <li>Try restarting the VM</li>
          </ul>
        </div>

        <Button
          onClick={() => {
            setReconnectAttempts(0)
            setStatus('connecting')
            connectToSpice()
          }}
          className="bg-blue-600 hover:bg-blue-700"
        >
          Try Again
        </Button>
      </div>
    )
  }

  // Render disconnected state
  if (status === 'disconnected') {
    return (
      <div className="h-full flex flex-col items-center justify-center bg-gray-900 p-8">
        <WifiOff className="w-16 h-16 text-gray-500 mb-4" />
        <h3 className="text-xl font-semibold text-white mb-2">Disconnected</h3>
        <p className="text-gray-400 text-center mb-6">
          The SPICE connection was closed.
        </p>
        <Button
          onClick={() => {
            setReconnectAttempts(0)
            setStatus('connecting')
            connectToSpice()
          }}
          className="bg-blue-600 hover:bg-blue-700"
        >
          Reconnect
        </Button>
      </div>
    )
  }

  return (
    <div className="h-full w-full relative bg-black spice-container">
      {/* SPICE Display Container */}
      <div
        ref={containerRef}
        className="h-full w-full flex items-center justify-center"
        style={{
          overflow: scaleMode === 'fit' ? 'auto' : 'hidden',
        }}
      />

      {/* Loading/Reconnecting overlay */}
      {(status === 'connecting' || isReconnecting) && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-900/80">
          <Loader2 className="w-12 h-12 animate-spin text-blue-500 mb-4" />
          <p className="text-white text-lg">
            {isReconnecting
              ? `Reconnecting... (attempt ${reconnectAttempts + 1}/${MAX_RECONNECT_ATTEMPTS})`
              : 'Connecting to SPICE console...'}
          </p>
          <p className="text-gray-400 text-sm mt-2">
            ws://{host}:{port}
          </p>
        </div>
      )}
    </div>
  )
})

SpiceViewer.displayName = 'SpiceViewer'
