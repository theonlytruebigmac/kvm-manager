import { useQuery } from '@tanstack/react-query'
import { api, connectionQueryKey } from '@/lib/tauri'
import type { OperationContext } from '@/lib/types'

export const activeConnectionQueryKey = ['active-connection'] as const
export const activeOperationContextQueryKey = ['active-operation-context'] as const

/**
 * Supplies the connection identity that must partition every resource cache entry.
 * Callers should disable resource queries until an active connection is known.
 */
export function useActiveConnection() {
  const activeConnection = useQuery({
    queryKey: activeConnectionQueryKey,
    queryFn: api.getActiveConnection,
  })

  return {
    ...activeConnection,
    connectionId: activeConnection.data?.id,
    resourceQueryKey: (...resource: string[]) => activeConnection.data
      ? connectionQueryKey(activeConnection.data.id, ...resource)
      : undefined,
  }
}

export function useActiveOperationContext(enabled = true) {
  return useQuery({
    queryKey: activeOperationContextQueryKey,
    queryFn: api.getActiveOperationContext,
    enabled,
  })
}

export function hasConnectionCapability(
  context: OperationContext | undefined,
  kind: string,
): boolean {
  return context?.capabilities.some(
    (capability) => capability.kind === kind && capability.state === 'available',
  ) ?? false
}
