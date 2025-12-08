import { invoke } from '@tauri-apps/api/core'
import type { VM, HostInfo, ConnectionStatus, VmConfig, VncInfo, VmStats, Network, NetworkConfig, StoragePool, Volume, VolumeConfig, Snapshot, SnapshotConfig } from './types'

/**
 * Tauri API wrapper for KVM Manager
 * All commands follow the contract defined in .agents/integration/tauri-commands.md
 */

export const api = {
  // VM Operations
  getVms: () => invoke<VM[]>('get_vms'),
  getVm: (vmId: string) => invoke<VM>('get_vm', { vmId }),
  startVm: (vmId: string) => invoke<void>('start_vm', { vmId }),
  stopVm: (vmId: string) => invoke<void>('stop_vm', { vmId }),
  forceStopVm: (vmId: string) => invoke<void>('force_stop_vm', { vmId }),
  pauseVm: (vmId: string) => invoke<void>('pause_vm', { vmId }),
  resumeVm: (vmId: string) => invoke<void>('resume_vm', { vmId }),
  rebootVm: (vmId: string) => invoke<void>('reboot_vm', { vmId }),
  deleteVm: (vmId: string, deleteDisks: boolean) => invoke<void>('delete_vm', { vmId, deleteDisks }),
  cloneVm: (sourceVmId: string, newName: string) => invoke<string>('clone_vm', { sourceVmId, newName }),
  createVm: (config: VmConfig) => invoke<string>('create_vm', { config }),
  addVmTags: (vmId: string, tags: string[]) => invoke<void>('add_vm_tags', { vmId, tags }),
  removeVmTags: (vmId: string, tags: string[]) => invoke<void>('remove_vm_tags', { vmId, tags }),

  // Console Operations
  getVncInfo: (vmId: string) => invoke<VncInfo>('get_vnc_info', { vmId }),
  openVncConsole: (vmId: string) => invoke<void>('open_vnc_console', { vmId }),

  // Performance Operations
  getVmStats: (vmId: string) => invoke<VmStats>('get_vm_stats', { vmId }),

  // System Operations
  getHostInfo: () => invoke<HostInfo>('get_host_info'),
  getConnectionStatus: () => invoke<ConnectionStatus>('get_connection_status'),

  // Network Operations
  getNetworks: () => invoke<Network[]>('get_networks'),
  getNetwork: (networkName: string) => invoke<Network>('get_network', { networkName }),
  createNetwork: (config: NetworkConfig) => invoke<string>('create_network', { config }),
  deleteNetwork: (networkName: string) => invoke<void>('delete_network', { networkName }),
  startNetwork: (networkName: string) => invoke<void>('start_network', { networkName }),
  stopNetwork: (networkName: string) => invoke<void>('stop_network', { networkName }),

  // Storage Operations
  getStoragePools: () => invoke<StoragePool[]>('get_storage_pools'),
  getVolumes: (poolId: string) => invoke<Volume[]>('get_volumes', { poolId }),
  createVolume: (poolId: string, config: VolumeConfig) => invoke<string>('create_volume', { poolId, config }),
  deleteVolume: (poolId: string, volumeName: string) => invoke<void>('delete_volume', { poolId, volumeName }),

  // Snapshot Operations
  getSnapshots: (vmId: string) => invoke<Snapshot[]>('get_snapshots', { vmId }),
  createSnapshot: (vmId: string, config: SnapshotConfig) => invoke<string>('create_snapshot', { vmId, config }),
  deleteSnapshot: (vmId: string, snapshotName: string) => invoke<void>('delete_snapshot', { vmId, snapshotName }),
  revertSnapshot: (vmId: string, snapshotName: string) => invoke<void>('revert_snapshot', { vmId, snapshotName }),
}
