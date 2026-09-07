import { fireEvent, render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { CreateVmWizard, findPoolIsoByName, GuestCapabilityReviewPanel, poolIsoVolumes } from './CreateVmWizard'

const readiness = vi.hoisted(() => vi.fn())

vi.mock('@/lib/tauri', () => ({
  api: {
    getVmCreationReadiness: readiness,
    getNetworks: vi.fn().mockResolvedValue([]),
    getVolumes: vi.fn().mockResolvedValue([]),
    preflightVmCreation: vi.fn(),
    createVm: vi.fn(),
    importIsoToPool: vi.fn(),
    openVmDetailsWindow: vi.fn(),
  },
}))

vi.mock('@/hooks/useActiveConnection', () => ({
  useActiveConnection: () => ({
    data: { id: 'fixture-a', name: 'Fixture connection' },
    connectionId: 'fixture-a',
    resourceQueryKey: (...resource: string[]) => ['connection', 'fixture-a', ...resource],
  }),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

function renderWizard(onClose = vi.fn()) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  render(
    <QueryClientProvider client={queryClient}>
      <CreateVmWizard onClose={onClose} />
    </QueryClientProvider>,
  )
  fireEvent.change(screen.getByLabelText('VM Name *'), { target: { value: 'fixture-vm' } })
  fireEvent.click(screen.getByRole('button', { name: /Next/ }))
  return onClose
}

describe('CreateVmWizard storage onboarding', () => {
  beforeEach(() => {
    readiness.mockResolvedValue({
      checkedAt: '2026-09-06T00:00:00Z',
      connectionId: 'fixture-a',
      connectionLabel: 'Fixture connection',
      connectionScope: 'test',
      overallState: 'degraded',
      distribution: { family: 'best_effort', displayName: 'Test', packageManager: 'unknown', supported: false, packages: [], service: '', permissionGuidance: '', firmwareGuidance: '', limitations: [] },
      capabilities: [],
      storage: { connectionId: 'fixture-a', state: 'unavailable', pools: [], recoveryAction: { kind: 'inspect', label: 'Inspect or create storage for this connection.', requiresConfirmation: false, expectedConnectionId: 'fixture-a' } },
    })
  })

  it('shows connection-owned storage recovery instead of assuming a default pool', async () => {
    renderWizard()
    expect(await screen.findByText('Fixture connection', { exact: false })).toBeInTheDocument()
    expect(await screen.findByText('Inspect or create storage for this connection.')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Open Storage' })).toBeInTheDocument()
  })

  it('cancels without issuing a creation mutation', () => {
    const onClose = renderWizard()
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('describes the safe import step for a recently downloaded ISO', () => {
    renderWizard()
    expect(screen.getByText(/ISO attached from the selected libvirt storage pool/)).toBeInTheDocument()
    expect(screen.getByLabelText('ISO path or existing pool ISO')).toHaveAttribute('list', 'existing-pool-isos')
  })

  it('offers only ISO media discovered in the selected storage pool', () => {
    const volumes = [
      { name: 'windows.iso', path: '/pool/windows.iso', poolName: 'default', capacityBytes: 10, allocationBytes: 10, format: 'iso' },
      { name: 'installer.bin', path: '/pool/installer.bin', poolName: 'default', capacityBytes: 10, allocationBytes: 10, format: 'iso' },
      { name: 'guest.qcow2', path: '/pool/guest.qcow2', poolName: 'default', capacityBytes: 10, allocationBytes: 1, format: 'qcow2' },
    ]
    expect(poolIsoVolumes(volumes)).toEqual([
      { name: 'windows.iso', path: '/pool/windows.iso', poolName: 'default', capacityBytes: 10, allocationBytes: 10, format: 'iso' },
      { name: 'installer.bin', path: '/pool/installer.bin', poolName: 'default', capacityBytes: 10, allocationBytes: 10, format: 'iso' },
    ])
    expect(findPoolIsoByName(volumes, 'windows.iso')?.path).toBe('/pool/windows.iso')
    expect(findPoolIsoByName(volumes, 'guest.qcow2')).toBeUndefined()
  })

  it('renders a specific guest capability blocker', () => {
    render(<GuestCapabilityReviewPanel
      isLoading={false}
      connectionLabel="Fixture connection"
      expectedConnectionId="fixture-a"
      review={{
        checkedAt: '2026-09-06T00:00:00Z',
        connectionId: 'fixture-a',
        requirements: { firmware: 'uefi-secure', tpmEnabled: true, network: 'active-net' },
        canCreate: false,
        storage: { connectionId: 'fixture-a', state: 'ready', pools: [] },
        capabilities: [{ kind: 'secure_boot', state: 'unavailable', summary: 'Secure Boot is unavailable.', remediation: 'Configure secure firmware on this connection.' }],
      }}
    />)
    expect(screen.getByText('Secure Boot is unavailable.')).toBeInTheDocument()
    expect(screen.getByText('Configure secure firmware on this connection.')).toBeInTheDocument()
  })

  it('does not render a review owned by a stale connection', () => {
    render(<GuestCapabilityReviewPanel
      isLoading={false}
      connectionLabel="New connection"
      expectedConnectionId="fixture-b"
      review={{
        checkedAt: '2026-09-06T00:00:00Z',
        connectionId: 'fixture-a',
        requirements: { firmware: 'bios', tpmEnabled: false },
        canCreate: true,
        storage: { connectionId: 'fixture-a', state: 'ready', pools: [] },
        capabilities: [{ kind: 'network', state: 'available', summary: 'STALE CAPABILITY' }],
      }}
    />)
    expect(screen.getByText('Connection changed. Refreshing prerequisites…')).toBeInTheDocument()
    expect(screen.queryByText('STALE CAPABILITY')).not.toBeInTheDocument()
  })
})
