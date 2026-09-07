import { useEffect } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useActiveConnection } from './useActiveConnection'

interface VmStateChangedPayload {
  vmId: string
  vmName: string
  oldState: 'running' | 'stopped' | 'paused' | 'suspended'
  newState: 'running' | 'stopped' | 'paused' | 'suspended'
  timestamp: number
}

/**
 * Hook to listen for VM state change events from the backend
 * Automatically invalidates VM queries when state changes occur
 */
export function useVmEvents() {
  const queryClient = useQueryClient()
  const { connectionId, resourceQueryKey } = useActiveConnection()

  useEffect(() => {
    let unlisten: UnlistenFn | undefined

    // Set up event listener
    const setupListener = async () => {
      unlisten = await listen<VmStateChangedPayload>('vm-state-changed', (event) => {
        const { vmName, oldState, newState } = event.payload

        console.log(`VM ${vmName} state changed: ${oldState} → ${newState}`)

        // Invalidate queries to refresh UI
        if (!connectionId) return
        queryClient.invalidateQueries({ queryKey: resourceQueryKey('vms') })
        queryClient.invalidateQueries({ queryKey: resourceQueryKey('vm', event.payload.vmId) })

        // Show notification
        const stateLabels = {
          running: 'Running',
          stopped: 'Stopped',
          paused: 'Paused',
          suspended: 'Suspended',
        }

        toast.info(`${vmName} is now ${stateLabels[newState]}`, {
          duration: 3000,
        })
      })
    }

    setupListener()

    // Cleanup
    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [connectionId, queryClient, resourceQueryKey])
}
