import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { Layout } from './components/layout/Layout'
import { Dashboard } from './pages/Dashboard'
import { VmList } from './pages/VmList'
import { VmDetails } from './pages/VmDetails'
import { StorageManager } from './pages/StorageManager'
import { NetworkManager } from './pages/NetworkManager'
import { Templates } from './pages/Templates'
import { Schedules } from './pages/Schedules'
import { Alerts } from './pages/Alerts'
import Backups from './pages/Backups'
import Settings from './pages/Settings'
import { Toaster } from './components/ui/sonner'

function App() {
  return (
    <>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<Navigate to="/dashboard" replace />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/vms" element={<VmList />} />
            <Route path="/vms/:vmId" element={<VmDetails />} />
            <Route path="/storage" element={<StorageManager />} />
            <Route path="/networks" element={<NetworkManager />} />
            <Route path="/templates" element={<Templates />} />
            <Route path="/schedules" element={<Schedules />} />
            <Route path="/alerts" element={<Alerts />} />
            <Route path="/backups" element={<Backups />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Layout>
      </BrowserRouter>
      <Toaster />
    </>
  )
}

export default App
