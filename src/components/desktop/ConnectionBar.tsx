import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Plus, Circle } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface Connection {
  id: string
  label: string
  status: 'connected' | 'disconnected' | 'connecting'
}

interface ConnectionBarProps {
  currentConnection: string
  connections: Connection[]
  onConnectionChange?: (connectionId: string) => void
  onAddConnection?: () => void
  className?: string
}

export function ConnectionBar({
  currentConnection,
  connections,
  onConnectionChange,
  onAddConnection,
  className,
}: ConnectionBarProps) {
  const activeConnection = connections.find((conn) => conn.id === currentConnection)

  const getStatusColor = (status: Connection['status']) => {
    switch (status) {
      case 'connected':
        return 'text-green-500'
      case 'disconnected':
        return 'text-red-500'
      case 'connecting':
        return 'text-yellow-500'
      default:
        return 'text-gray-500'
    }
  }

  const getStatusText = (status: Connection['status']) => {
    switch (status) {
      case 'connected':
        return 'Connected'
      case 'disconnected':
        return 'Disconnected'
      case 'connecting':
        return 'Connecting...'
      default:
        return 'Unknown'
    }
  }

  return (
    <div
      className={cn(
        'flex items-center justify-between border-b border-[var(--toolbar-border)] bg-[var(--toolbar-bg)] px-3 py-1.5',
        className
      )}
    >
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <span className="text-desktop-sm font-medium text-muted-foreground">
            Connection:
          </span>
          <Select
            value={currentConnection}
            onValueChange={onConnectionChange}
          >
            <SelectTrigger className="h-8 w-[280px] bg-background">
              <SelectValue placeholder="Select a connection" />
            </SelectTrigger>
            <SelectContent>
              {connections.map((connection) => (
                <SelectItem key={connection.id} value={connection.id}>
                  <div className="flex items-center gap-2">
                    <Circle
                      className={cn(
                        'h-2 w-2 fill-current',
                        getStatusColor(connection.status)
                      )}
                    />
                    <span>{connection.label}</span>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {activeConnection && (
          <Badge
            variant={activeConnection.status === 'connected' ? 'default' : 'secondary'}
            className={cn(
              'text-xs',
              activeConnection.status === 'connected' && 'bg-green-500 hover:bg-green-600',
              activeConnection.status === 'disconnected' && 'bg-red-500 hover:bg-red-600',
              activeConnection.status === 'connecting' && 'bg-yellow-500 hover:bg-yellow-600'
            )}
          >
            <Circle
              className={cn(
                'mr-1 h-2 w-2 fill-current',
                activeConnection.status === 'connected' && 'text-white',
                activeConnection.status === 'disconnected' && 'text-white',
                activeConnection.status === 'connecting' && 'text-white animate-pulse'
              )}
            />
            {getStatusText(activeConnection.status)}
          </Badge>
        )}
      </div>

      <Button
        size="sm"
        variant="outline"
        onClick={onAddConnection}
        className="h-8 gap-1.5"
      >
        <Plus className="h-3.5 w-3.5" />
        Add Connection
      </Button>
    </div>
  )
}
