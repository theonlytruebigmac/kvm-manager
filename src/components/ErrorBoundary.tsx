import React from 'react'

interface ErrorBoundaryProps {
  children: React.ReactNode
}

interface ErrorBoundaryState {
  hasError: boolean
}

/**
 * Keeps an unexpected render failure scoped to the affected view. Deliberately never renders or
 * logs the raw React error because it can contain host-specific implementation details.
 */
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false }

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true }
  }

  componentDidCatch(): void {
    console.error('A view failed to render; a safe recovery screen was shown.')
  }

  private retry = () => {
    this.setState({ hasError: false })
  }

  render() {
    if (this.state.hasError) {
      return (
        <main className="p-8 bg-red-50 text-red-900" role="alert">
          <h1 className="text-xl font-bold">Unable to display this view</h1>
          <p className="mt-4">Retry the page or return to a working view.</p>
          <button
            className="mt-4 rounded bg-red-900 px-4 py-2 text-white"
            onClick={this.retry}
            type="button"
          >
            Retry page
          </button>
        </main>
      )
    }

    return this.props.children
  }
}
