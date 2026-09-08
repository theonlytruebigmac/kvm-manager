import { act, createRef } from 'react'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { sendCtrlAltDel, VncViewer, type VncViewerRef } from './VncViewer'

class FakeRfb {
  static instance: FakeRfb | undefined
  private listeners = new Map<string, (event: { detail?: { clean?: boolean } }) => void>()
  viewOnly = true
  focusOnClick = false
  scaleViewport = false
  resizeSession = false
  showDotCursor = false
  qualityLevel = 0
  compressionLevel = 0
  focus = vi.fn()
  blur = vi.fn()
  disconnect = vi.fn()
  sendCtrlAltDel = vi.fn()
  sendKey = vi.fn()

  constructor(target: HTMLElement) {
    const canvas = document.createElement('canvas')
    target.appendChild(canvas)
    FakeRfb.instance = this
  }

  addEventListener(name: string, listener: (event: { detail?: { clean?: boolean } }) => void) {
    this.listeners.set(name, listener)
  }

  emit(name: string) {
    this.listeners.get(name)?.({ detail: { clean: true } })
  }
}

describe('VncViewer input capture', () => {
  it('focuses the supported RFB input target and sends special keys through the live instance', async () => {
    ;(window as typeof window & { __noVNC_RFB__?: typeof FakeRfb }).__noVNC_RFB__ = FakeRfb
    ;(window as typeof window & { __noVNC_getKeysym__?: () => number }).__noVNC_getKeysym__ = () => 0x61
    const inputFocus = vi.fn()
    const keySent = vi.fn()
    const ref = createRef<VncViewerRef>()
    render(
      <VncViewer
        ref={ref}
        host="127.0.0.1"
        port={5901}
        onInputFocusChange={inputFocus}
        onKeySent={keySent}
      />,
    )

    await waitFor(() => expect(FakeRfb.instance).toBeDefined())
    act(() => FakeRfb.instance?.emit('connect'))

    await waitFor(() => expect(FakeRfb.instance?.focus).toHaveBeenCalled())
    expect(FakeRfb.instance?.viewOnly).toBe(false)
    expect(FakeRfb.instance?.focusOnClick).toBe(true)
    expect(inputFocus).toHaveBeenCalledWith(true)

    fireEvent.keyDown(window, { key: 'a', code: 'KeyA' })
    expect(FakeRfb.instance?.sendKey).toHaveBeenCalledWith(0x61, 'KeyA', true)
    expect(keySent).toHaveBeenCalledOnce()

    fireEvent.mouseDown(screen.getByRole('application', { name: 'Interactive VM console' }))
    expect(FakeRfb.instance?.focus).toHaveBeenCalledTimes(2)
    expect(sendCtrlAltDel(ref)).toBe(true)
    expect(FakeRfb.instance?.sendCtrlAltDel).toHaveBeenCalledOnce()
  })
})
