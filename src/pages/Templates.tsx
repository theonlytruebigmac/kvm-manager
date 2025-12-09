import { TemplateManager } from '@/components/vm/TemplateManager'
import { useNavigate } from 'react-router-dom'
import type { VmConfig } from '@/lib/types'

export function Templates() {
  const navigate = useNavigate()

  const handleCreateFromTemplate = (_config: VmConfig) => {
    // Navigate to VM creation page with template config
    // For now, we'll just show a message
    // In a future update, we can enhance CreateVmWizard to accept initial config
    navigate('/vms')
  }

  return (
    <div className="container mx-auto p-6">
      <TemplateManager onCreateFromTemplate={handleCreateFromTemplate} />
    </div>
  )
}
