import { describe, expect, it } from 'vitest'
import { connectionQueryKey, isConnectionQueryKey, isSafeFailure } from '@/lib/tauri'
import { hasConnectionCapability } from '@/hooks/useActiveConnection'
import { unavailableConnectionFailure, unavailableHostDeviceCapability } from '@/test/hardeningFixtures'

describe('connection routing boundaries', () => {
  it('partitions resource query keys by the selected connection', () => {
    expect(connectionQueryKey('fixture-a', 'vms')).not.toEqual(
      connectionQueryKey('fixture-b', 'vms'),
    )
    expect(connectionQueryKey('fixture-a', 'vm', 'same-name')).toEqual([
      'connection',
      'fixture-a',
      'vm',
      'same-name',
    ])
    expect(connectionQueryKey('fixture-a', 'vm-creation-readiness')).not.toEqual(
      connectionQueryKey('fixture-b', 'vm-creation-readiness'),
    )
  })

  it('identifies all connection-owned resources for reconnection cache eviction', () => {
    expect(isConnectionQueryKey(connectionQueryKey('fixture-a', 'vms'))).toBe(true)
    expect(isConnectionQueryKey(['vms'])).toBe(false)
    expect(isConnectionQueryKey(['active-connection'])).toBe(false)
  })

  it('keeps stale-connection failures and unavailable capabilities explicit', () => {
    expect(isSafeFailure(unavailableConnectionFailure)).toBe(true)
    expect(unavailableConnectionFailure.recoveryAction?.expectedConnectionId).toBe('lab-remote')
    expect(unavailableHostDeviceCapability.state).toBe('unavailable')
    expect(unavailableHostDeviceCapability.reasonCode).toBe('requires_local_host')
    expect(hasConnectionCapability({ capabilities: [unavailableHostDeviceCapability] } as never, 'hostDevice')).toBe(false)
  })
})
