import { describe, expect, it } from 'vitest'
import config from '../../src-tauri/tauri.conf.json'

const csp = config.app.security.csp as string

describe('production content policy', () => {
  it('uses an explicit, narrow policy for IPC and loopback consoles', () => {
    expect(csp).toContain("default-src 'self'")
    expect(csp).toContain("connect-src 'self' ipc:")
    expect(csp).toContain('ws://localhost:*')
    expect(csp).toContain('ws://127.0.0.1:*')
    expect(csp).not.toContain('connect-src *')
    expect(csp).not.toContain('script-src *')
    expect(csp).not.toContain("'unsafe-eval'")
  })
})
