import { fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ErrorBoundary } from './ErrorBoundary'

let shouldThrow = true

function ConditionalThrow() {
  if (shouldThrow) {
    throw new Error('SENTINEL_HOST_PATH /private/host/path')
  }

  return <p>Recovered view</p>
}

describe('ErrorBoundary', () => {
  afterEach(() => {
    shouldThrow = true
    vi.restoreAllMocks()
  })

  it('discloses only a safe recovery message instead of raw error details', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)

    render(
      <ErrorBoundary>
        <ConditionalThrow />
      </ErrorBoundary>,
    )

    expect(screen.getByRole('heading', { name: 'Unable to display this view' })).toBeTruthy()
    expect(screen.getByText('Retry the page or return to a working view.')).toBeTruthy()
    expect(screen.queryByText(/SENTINEL_HOST_PATH|private\/host\/path/)).toBeNull()
  })

  it('retries safely without leaving the error screen stuck', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)

    render(
      <ErrorBoundary>
        <ConditionalThrow />
      </ErrorBoundary>,
    )

    shouldThrow = false
    fireEvent.click(screen.getByRole('button', { name: 'Retry page' }))

    expect(screen.getByText('Recovered view')).toBeTruthy()
  })
})
