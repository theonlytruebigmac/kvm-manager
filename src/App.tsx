import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Layout } from './components/layout/Layout'
import { Dashboard } from './pages/Dashboard'
import { VmList } from './pages/VmList'
import { VmDetailsWindow } from './pages/VmDetailsWindow'
import { ConsoleWindow } from './pages/ConsoleWindow'
import { StorageManager } from './pages/StorageManager'
import { NetworkManager } from './pages/NetworkManager'
import { Templates } from './pages/Templates'
import { Schedules } from './pages/Schedules'
import { Alerts } from './pages/Alerts'
import Backups from './pages/Backups'
import Settings from './pages/Settings'
import Insights from './pages/Insights'
import { Toaster } from './components/ui/sonner'
import React from 'react'

// Error boundary component
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('React error:', error, info)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-8 bg-red-50 text-red-900">
          <h1 className="text-xl font-bold">Something went wrong</h1>
          <pre className="mt-4 text-sm overflow-auto">{this.state.error?.message}</pre>
          <pre className="mt-2 text-xs overflow-auto">{this.state.error?.stack}</pre>
        </div>
      )
    }
    return this.props.children
  }
}

function App() {
  // Debug - log current location
  console.log('App rendering, current location:', window.location.pathname)

  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          {/* Main window routes (with Layout) */}
          <Route path="/" element={<Layout><VmList /></Layout>} />
          <Route path="/dashboard" element={<Layout><Dashboard /></Layout>} />
          <Route path="/storage" element={<Layout><StorageManager /></Layout>} />
          <Route path="/networks" element={<Layout><NetworkManager /></Layout>} />
          <Route path="/insights" element={<Layout><Insights /></Layout>} />
          <Route path="/templates" element={<Layout><Templates /></Layout>} />
          <Route path="/schedules" element={<Layout><Schedules /></Layout>} />
          <Route path="/alerts" element={<Layout><Alerts /></Layout>} />
          <Route path="/backups" element={<Layout><Backups /></Layout>} />
          <Route path="/settings" element={<Layout><Settings /></Layout>} />

          {/* Separate window routes (without Layout) */}
          <Route path="/vms/:vmId" element={<VmDetailsWindow />} />
          <Route path="/console/:vmId" element={<ConsoleWindow />} />

          {/* Catch-all for debugging */}
          <Route path="*" element={<div className="p-8 text-red-500">Route not matched: {window.location.pathname}</div>} />
        </Routes>
      </BrowserRouter>
      <Toaster />
    </ErrorBoundary>
  )
}

export default App
