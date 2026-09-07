import type { ConnectionCapability, SafeFailure } from '@/lib/types'

export const remoteResourceCapability: ConnectionCapability = {
  kind: 'resource_management',
  state: 'available',
  checkedAt: '2026-09-06T00:00:00Z',
}

export const unavailableHostDeviceCapability: ConnectionCapability = {
  kind: 'host_device',
  state: 'unavailable',
  reasonCode: 'requires_local_host',
  checkedAt: '2026-09-06T00:00:00Z',
}

export const unavailableConnectionFailure: SafeFailure = {
  code: 'unavailable',
  summary: 'The selected connection is unavailable.',
  outcome: 'rejected',
  retryable: true,
  connectionId: 'lab-remote',
  operationId: 'test-operation',
  recoveryAction: {
    kind: 'reconnect',
    label: 'Reconnect and retry',
    requiresConfirmation: false,
    expectedConnectionId: 'lab-remote',
  },
}
