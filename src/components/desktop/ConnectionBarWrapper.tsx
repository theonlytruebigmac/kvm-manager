import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api, isConnectionQueryKey } from '@/lib/tauri'
import { activeConnectionQueryKey, activeOperationContextQueryKey } from '@/hooks/useActiveConnection'
import { ConnectionBar, type Connection } from './ConnectionBar'

export function ConnectionBarWrapper() {
  const queryClient = useQueryClient()
  const { data: savedConnections = [] } = useQuery({
    queryKey: ['saved-connections'],
    queryFn: api.getSavedConnections,
  })
  const { data: activeConnection } = useQuery({
    queryKey: activeConnectionQueryKey,
    queryFn: api.getActiveConnection,
  })
  const connectMutation = useMutation({
    mutationFn: (connectionId: string) => api.connectTo(connectionId),
    onSuccess: (_data, connectionId) => {
      // Cancel and evict resource data that was fetched under the prior connection.
      // This also protects legacy consumers while they are migrated to scoped keys.
      queryClient.cancelQueries({ predicate: (query) => isConnectionQueryKey(query.queryKey) || query.queryKey[0] === 'vms' || query.queryKey[0] === 'host-info' || query.queryKey[0] === 'hostInfo' })
      queryClient.removeQueries({ predicate: (query) => isConnectionQueryKey(query.queryKey) || query.queryKey[0] === 'vms' || query.queryKey[0] === 'host-info' || query.queryKey[0] === 'hostInfo' })
      queryClient.invalidateQueries({ queryKey: activeConnectionQueryKey })
      queryClient.removeQueries({ queryKey: activeOperationContextQueryKey })
      queryClient.invalidateQueries({ queryKey: ['saved-connections'] })
      queryClient.setQueryData(activeConnectionQueryKey, savedConnections.find((connection) => connection.id === connectionId) ?? null)
    },
  })

  const connections: Connection[] = savedConnections.map((connection) => ({
    id: connection.id,
    label: connection.name,
    status: activeConnection?.id === connection.id
      ? 'connected'
      : 'disconnected',
  }))

  const handleConnectionChange = (connectionId: string) => {
    if (connectionId !== activeConnection?.id) connectMutation.mutate(connectionId)
  }

  const handleAddConnection = () => {
    window.dispatchEvent(new CustomEvent('kvm-manager:open-connection-manager'))
  }

  return (
    <ConnectionBar
      currentConnection={activeConnection?.id ?? connections[0]?.id ?? ''}
      connections={connections}
      onConnectionChange={handleConnectionChange}
      onAddConnection={handleAddConnection}
    />
  )
}
