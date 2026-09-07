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
import { PerformanceMonitor } from './pages/PerformanceMonitor'
import { Toaster } from './components/ui/sonner'
import { ErrorBoundary } from './components/ErrorBoundary'
import { HostReadinessPanel } from './components/system/HostReadinessPanel'

function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          {/* Main window routes (with Layout) */}
          <Route path="/" element={<Layout><VmList /></Layout>} />
          <Route path="/dashboard" element={<Layout><Dashboard /></Layout>} />
          <Route path="/performance" element={<Layout><PerformanceMonitor /></Layout>} />
          <Route path="/storage" element={<Layout><StorageManager /></Layout>} />
          <Route path="/networks" element={<Layout><NetworkManager /></Layout>} />
          <Route path="/insights" element={<Layout><Insights /></Layout>} />
          <Route path="/templates" element={<Layout><Templates /></Layout>} />
          <Route path="/schedules" element={<Layout><Schedules /></Layout>} />
          <Route path="/alerts" element={<Layout><Alerts /></Layout>} />
          <Route path="/backups" element={<Layout><Backups /></Layout>} />
          <Route path="/settings" element={<Layout><Settings /></Layout>} />
          <Route path="/onboarding" element={<Layout><div className="p-4"><HostReadinessPanel /></div></Layout>} />

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
