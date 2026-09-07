import { describe, expect, it } from 'vitest'
import { localConsoleWebSocketUrl } from '@/lib/consoleSecurity'

describe('local console transport', () => {
  it('preserves VNC and SPICE loopback proxy connectivity', () => {
    expect(localConsoleWebSocketUrl('127.0.0.1', 5901)).toBe('ws://127.0.0.1:5901')
    expect(localConsoleWebSocketUrl('localhost', 6080)).toBe('ws://localhost:6080')
  })

  it('rejects remote hosts and invalid proxy ports before connection', () => {
    expect(() => localConsoleWebSocketUrl('192.0.2.15', 5901)).toThrow()
    expect(() => localConsoleWebSocketUrl('127.0.0.1', 0)).toThrow()
    expect(() => localConsoleWebSocketUrl('127.0.0.1', 65_536)).toThrow()
  })
})
