import { describe, expect, it } from 'vitest'
import { unavailableConnectionFailure } from '@/test/hardeningFixtures'
import { normalizeInvokeFailure, SafeFailureError } from './tauri'

describe('safe failure contract', () => {
  it('retains recoverable connection context without protected details', () => {
    expect(unavailableConnectionFailure.code).toBe('unavailable')
    expect(unavailableConnectionFailure.outcome).toBe('rejected')
    expect(unavailableConnectionFailure.recoveryAction?.expectedConnectionId).toBe('lab-remote')
    expect(JSON.stringify(unavailableConnectionFailure)).not.toContain('password')
  })

  it('does not expose legacy command rejection text', () => {
    const failure = normalizeInvokeFailure('SENTINEL_PASSWORD_DO_NOT_LOG /private/path')
    expect(failure.code).toBe('internal')
    expect(failure.summary).not.toContain('SENTINEL_PASSWORD_DO_NOT_LOG')
  })

  it('renders only a classified recovery message for a rejected IPC payload', () => {
    const error = new SafeFailureError({
      ...unavailableConnectionFailure,
      summary: 'The selected connection is unavailable.',
    })

    expect(error.message).toContain('The selected connection is unavailable.')
    expect(error.message).toContain('Reconnect and retry')
    expect(error.message).not.toContain('SENTINEL_PASSWORD_DO_NOT_LOG')
    expect(error.failure.code).toBe('unavailable')
  })
})
