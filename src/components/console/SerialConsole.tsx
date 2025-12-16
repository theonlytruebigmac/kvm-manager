import { useEffect, useRef, useState, useCallback } from 'react'
import { useQuery, useMutation } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { Terminal, Power, PowerOff, RefreshCw } from 'lucide-react'

interface SerialConsoleProps {
  vmId: string
  vmName: string
}

export function SerialConsole({ vmId, vmName }: SerialConsoleProps) {
  const [output, setOutput] = useState<string>('')
  const [inputBuffer, setInputBuffer] = useState<string>('')
  const outputRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null)

  // Get serial console info
  const { data: consoleInfo, isLoading: infoLoading } = useQuery({
    queryKey: ['serialConsoleInfo', vmId],
    queryFn: () => api.getSerialConsoleInfo(vmId),
    enabled: !!vmId,
  })

  // Check if connected
  const { data: isConnected = false, refetch: refetchConnected } = useQuery({
    queryKey: ['serialConsoleConnected', vmId],
    queryFn: () => api.isSerialConsoleConnected(vmId),
    enabled: !!vmId,
    refetchInterval: 2000,
  })

  // Open console mutation
  const openMutation = useMutation({
    mutationFn: () => api.openSerialConsole(vmId),
    onSuccess: () => {
      toast.success('Serial console connected')
      refetchConnected()
      startPolling()
    },
    onError: (error) => {
      toast.error(`Failed to connect: ${error}`)
    },
  })

  // Close console mutation
  const closeMutation = useMutation({
    mutationFn: () => api.closeSerialConsole(vmId),
    onSuccess: () => {
      toast.success('Serial console disconnected')
      stopPolling()
      refetchConnected()
    },
    onError: (error) => {
      toast.error(`Failed to disconnect: ${error}`)
    },
  })

  // Write to console mutation
  const writeMutation = useMutation({
    mutationFn: (input: string) => api.writeSerialConsole(vmId, input),
    onError: (error) => {
      toast.error(`Failed to write: ${error}`)
    },
  })

  // Read output
  const readOutput = useCallback(async () => {
    try {
      const data = await api.readSerialConsole(vmId)
      if (data) {
        setOutput((prev) => prev + data)
        // Auto-scroll
        if (outputRef.current) {
          outputRef.current.scrollTop = outputRef.current.scrollHeight
        }
      }
    } catch (error) {
      // Ignore read errors during polling
    }
  }, [vmId])

  // Start polling for output
  const startPolling = useCallback(() => {
    if (pollingRef.current) {
      clearInterval(pollingRef.current)
    }
    pollingRef.current = setInterval(readOutput, 100)
  }, [readOutput])

  // Stop polling
  const stopPolling = useCallback(() => {
    if (pollingRef.current) {
      clearInterval(pollingRef.current)
      pollingRef.current = null
    }
  }, [])

  // Handle key press
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isConnected) return

    // Send special keys
    if (e.key === 'Enter') {
      e.preventDefault()
      writeMutation.mutate(inputBuffer + '\r')
      setInputBuffer('')
    } else if (e.key === 'Backspace' && inputBuffer.length > 0) {
      e.preventDefault()
      setInputBuffer(inputBuffer.slice(0, -1))
      writeMutation.mutate('\b \b') // Backspace, space, backspace
    } else if (e.key === 'Tab') {
      e.preventDefault()
      writeMutation.mutate('\t')
    } else if (e.key === 'Escape') {
      e.preventDefault()
      writeMutation.mutate('\x1b')
    } else if (e.ctrlKey && e.key === 'c') {
      e.preventDefault()
      writeMutation.mutate('\x03') // Ctrl+C
      setInputBuffer('')
    } else if (e.ctrlKey && e.key === 'd') {
      e.preventDefault()
      writeMutation.mutate('\x04') // Ctrl+D
    } else if (e.ctrlKey && e.key === 'l') {
      e.preventDefault()
      writeMutation.mutate('\x0c') // Ctrl+L (clear screen)
      setOutput('')
    }
  }

  const handleInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value
    if (newValue.length > inputBuffer.length) {
      const newChars = newValue.slice(inputBuffer.length)
      writeMutation.mutate(newChars)
    }
    setInputBuffer(newValue)
  }

  // Clean up on unmount
  useEffect(() => {
    return () => {
      stopPolling()
    }
  }, [stopPolling])

  // Start polling if already connected
  useEffect(() => {
    if (isConnected && !pollingRef.current) {
      startPolling()
    }
  }, [isConnected, startPolling])

  // Focus input when connected
  useEffect(() => {
    if (isConnected && inputRef.current) {
      inputRef.current.focus()
    }
  }, [isConnected])

  const clearOutput = () => {
    setOutput('')
  }

  return (
    <Card className="flex flex-col h-full">
      <CardHeader className="py-3 px-4 border-b flex-shrink-0">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Terminal className="h-5 w-5 text-muted-foreground" />
            <div>
              <CardTitle className="text-sm font-medium">Serial Console</CardTitle>
              <p className="text-xs text-muted-foreground">{vmName}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={isConnected ? 'default' : 'secondary'}>
              {isConnected ? 'Connected' : 'Disconnected'}
            </Badge>
            {consoleInfo && (
              <Badge variant="outline" className="font-mono text-xs">
                {consoleInfo.ptyPath}
              </Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="flex-1 p-0 flex flex-col min-h-0">
        {/* Terminal Output */}
        <div
          ref={outputRef}
          className="flex-1 bg-black text-green-400 font-mono text-sm p-4 overflow-auto whitespace-pre-wrap"
          style={{ minHeight: '300px' }}
          onClick={() => inputRef.current?.focus()}
        >
          {infoLoading ? (
            <div className="text-gray-500">Loading...</div>
          ) : !consoleInfo?.active ? (
            <div className="text-yellow-500">VM is not running. Start the VM to use serial console.</div>
          ) : !isConnected ? (
            <div className="text-gray-500">Click 'Connect' to open the serial console.</div>
          ) : output.length === 0 ? (
            <div className="text-gray-500">Connected. Press Enter to get a prompt...</div>
          ) : (
            output
          )}
          {isConnected && (
            <span className="inline-block bg-green-400 w-2 h-4 animate-pulse ml-0.5" />
          )}
        </div>

        {/* Hidden input for capturing keystrokes */}
        {isConnected && (
          <input
            ref={inputRef}
            type="text"
            value={inputBuffer}
            onChange={handleInput}
            onKeyDown={handleKeyDown}
            className="sr-only"
            autoComplete="off"
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
          />
        )}

        {/* Controls */}
        <div className="flex items-center gap-2 p-3 border-t bg-muted/50">
          {!isConnected ? (
            <Button
              onClick={() => openMutation.mutate()}
              disabled={openMutation.isPending || !consoleInfo?.active}
              size="sm"
            >
              <Power className="h-4 w-4 mr-2" />
              Connect
            </Button>
          ) : (
            <Button
              onClick={() => closeMutation.mutate()}
              disabled={closeMutation.isPending}
              variant="destructive"
              size="sm"
            >
              <PowerOff className="h-4 w-4 mr-2" />
              Disconnect
            </Button>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={clearOutput}
            disabled={!isConnected}
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            Clear
          </Button>
          <div className="flex-1" />
          <p className="text-xs text-muted-foreground">
            Click terminal area to focus • Ctrl+C to interrupt • Ctrl+L to clear
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
