import { fireEvent, render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { HostReadinessPanel } from './HostReadinessPanel'
import type { VmCreationReadiness } from '@/lib/types'

const readyReport: VmCreationReadiness = {
  checkedAt: '2026-09-06T12:00:00Z',
  connectionId: 'local',
  connectionLabel: 'QEMU/KVM (Local)',
  connectionScope: 'local_system',
  overallState: 'ready',
  distribution: {
    family: 'arch_cachyos',
    displayName: 'CachyOS Linux',
    packageManager: 'pacman',
    supported: true,
    packages: ['libvirt', 'qemu-full'],
    service: 'Enable libvirtd.socket.',
    permissionGuidance: 'Start a new login session after adding the user to the libvirt group.',
    firmwareGuidance: 'Firmware is discovered from libvirt capabilities.',
    limitations: [],
  },
  capabilities: [
    {
      kind: 'qemu_emulator',
      state: 'available',
      summary: 'A QEMU system emulator is available.',
    },
  ],
  storage: {
    connectionId: 'local',
    state: 'selection_required',
    pools: [{ id: 'pool-a', name: 'VM storage', state: 'active', poolType: 'dir', capacityBytes: 100, allocationBytes: 10, availableBytes: 90, autostart: true, eligible: true }],
  },
}

describe('HostReadinessPanel', () => {
  const renderPanel = (report: VmCreationReadiness) => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return render(
      <QueryClientProvider client={queryClient}>
        <HostReadinessPanel report={report} />
      </QueryClientProvider>,
    )
  }

  it('shows supported distribution guidance and ready state', () => {
    renderPanel(readyReport)

    expect(screen.getByText('Host Readiness')).toBeInTheDocument()
    expect(screen.getByText('Ready')).toBeInTheDocument()
    expect(screen.getByText('CachyOS Linux', { exact: false })).toBeInTheDocument()
    expect(screen.getByText(/pacman/)).toBeInTheDocument()
  })

  it('shows best-effort status and capability remediation', () => {
    renderPanel({
          ...readyReport,
          overallState: 'degraded',
          distribution: {
            ...readyReport.distribution,
            supported: false,
            displayName: 'Example Linux',
            packageManager: 'unknown',
            limitations: ['This distribution is not in the verified support matrix.'],
          },
          capabilities: [
            {
              kind: 'qemu_emulator',
              state: 'unavailable',
              summary: 'qemu emulator is unavailable.',
              remediation: 'Install the QEMU system emulator package for this distribution.',
            },
          ],
    })

    expect(screen.getByText('Best-effort support', { exact: false })).toBeInTheDocument()
    expect(screen.getByText('Degraded')).toBeInTheDocument()
    expect(screen.getByText(/Install the QEMU system emulator/)).toBeInTheDocument()
  })

  it('opens actionable manual guidance with a command and readiness recheck', () => {
    renderPanel({
      ...readyReport,
      overallState: 'degraded',
      capabilities: [{
        kind: 'secure_boot',
        state: 'unavailable',
        summary: 'Secure Boot is unavailable.',
        repairAction: {
          id: 'secure_boot_guidance',
          mode: 'manual',
          title: 'Configure Secure Boot firmware',
          effect: 'Secure Boot firmware is installed, but an enrolled-key store is not available.',
          requiresPrivilege: false,
          requiresConfirmation: false,
          expectedConnectionId: 'local',
          steps: [
            'Option 1 — continue now: Select UEFI in the VM wizard if Secure Boot enforcement is not required.',
            'Option 2 — configure enrolled keys: Install the key-enrollment utility for VM-specific NVRAM.',
            'Run: sudo pacman -S --needed virt-firmware',
            'Reference: https://wiki.archlinux.org/title/KVM#Enabling_Secure_Boot',
          ],
        },
      }],
    })

    fireEvent.click(screen.getByRole('button', { name: 'Show guided steps' }))
    expect(screen.getByRole('alertdialog')).toHaveTextContent('Configure Secure Boot firmware')
    expect(screen.getByText(/Option 1 — continue now/)).toBeInTheDocument()
    expect(screen.getByText(/Option 2 — configure enrolled keys/)).toBeInTheDocument()
    expect(screen.getByText('sudo pacman -S --needed virt-firmware')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Copy command' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Open the Arch Linux Secure Boot guide' })).toHaveAttribute(
      'href',
      'https://wiki.archlinux.org/title/KVM#Enabling_Secure_Boot',
    )
    expect(screen.getByRole('button', { name: 'Recheck readiness' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Confirm and repair' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Recheck readiness' }))
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
  })

  it('previews privilege and effects before an automated repair', () => {
    renderPanel({
      ...readyReport,
      overallState: 'degraded',
      capabilities: [{
        kind: 'tpm',
        state: 'unavailable',
        summary: 'TPM is unavailable.',
        repairAction: {
          id: 'install_tpm',
          mode: 'automated',
          title: 'Install TPM emulation support',
          effect: 'Install the verified TPM package.',
          requiresPrivilege: true,
          requiresConfirmation: true,
          expectedConnectionId: 'local',
          steps: ['Authorize package installation.'],
        },
      }],
    })

    fireEvent.click(screen.getByRole('button', { name: 'Fix this requirement' }))
    expect(screen.getByText('Install the verified TPM package.')).toBeInTheDocument()
    expect(screen.getByText(/will not receive your password/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Confirm and repair' })).toBeInTheDocument()
  })
})
