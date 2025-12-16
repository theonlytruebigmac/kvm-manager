import { ConnectionBar, type Connection } from './ConnectionBar'

// TODO: Replace with actual connection management from store/context
const mockConnections: Connection[] = [
  { id: 'qemu-system', label: 'QEMU/KVM (qemu:///system)', status: 'connected' },
  { id: 'qemu-session', label: 'QEMU/KVM (qemu:///session)', status: 'disconnected' },
]

export function ConnectionBarWrapper() {
  const handleConnectionChange = (connectionId: string) => {
    console.log('Connection changed to:', connectionId)
    // TODO: Implement connection switching logic
  }

  const handleAddConnection = () => {
    console.log('Add new connection')
    // TODO: Implement add connection dialog
  }

  return (
    <ConnectionBar
      currentConnection="qemu-system"
      connections={mockConnections}
      onConnectionChange={handleConnectionChange}
      onAddConnection={handleAddConnection}
    />
  )
}
