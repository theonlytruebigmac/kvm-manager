// TypeScript types matching the Rust backend models
// Based on .agents/integration/tauri-commands.md

export type VmState = 'running' | 'stopped' | 'paused' | 'suspended' | 'crashed'

export interface NetworkInterface {
  mac: string
  network: string
  ipAddress?: string
}

export interface VM {
  id: string
  name: string
  state: VmState
  cpuCount: number
  memoryMb: number
  osType?: string
  osVariant?: string
  diskSizeGb: number
  networkInterfaces: NetworkInterface[]
  vncPort?: number
  createdAt?: number
  description?: string
  tags: string[]
}

export interface HostInfo {
  hostname: string
  cpuModel: string
  cpuCount: number
  cpuThreads: number
  memoryTotalMb: number
  memoryFreeMb: number
  libvirtVersion: string
  qemuVersion: string
  hypervisor: string
  activeVms: number
  totalVms: number
}

export interface ConnectionStatus {
  connected: boolean
  uri: string
  error: string | null
}

export interface VmConfig {
  name: string
  cpuCount: number
  memoryMb: number
  diskSizeGb: number
  osType: 'linux' | 'windows' | 'other'
  osVariant?: string
  isoPath?: string
  network: string
  diskFormat: 'qcow2' | 'raw'
  bootMenu: boolean
}

export interface DeleteVmOptions {
  deleteDisks: boolean
}

export interface CloneVmConfig {
  sourceVmId: string
  newName: string
}

export interface VncInfo {
  host: string
  port: number
  password?: string
  websocketPort?: number
}

export interface VmStats {
  vmId: string
  cpuUsagePercent: number
  memoryUsedMb: number
  memoryAvailableMb: number
  diskReadBytes: number
  diskWriteBytes: number
  networkRxBytes: number
  networkTxBytes: number
  timestamp: number
}

export interface Network {
  name: string
  uuid: string
  bridge: string
  active: boolean
  autostart: boolean
  ipRange: string | null
}

export interface NetworkConfig {
  name: string
  bridgeName: string
  forwardMode: 'nat' | 'route' | 'bridge' | 'isolated'
  ipAddress: string
  netmask: string
  dhcpStart: string
  dhcpEnd: string
  autostart: boolean
}

// Storage types
export type PoolState = 'active' | 'inactive' | 'degraded'

export type PoolType =
  | 'dir'
  | 'fs'
  | 'netfs'
  | 'logical'
  | 'disk'
  | 'iscsi'
  | 'scsi'
  | 'mpath'
  | 'rbd'
  | 'sheepdog'
  | 'gluster'
  | 'zfs'

export interface StoragePool {
  id: string
  name: string
  state: PoolState
  poolType: PoolType
  capacityBytes: number
  allocationBytes: number
  availableBytes: number
  path: string
  autostart: boolean
}

export interface Volume {
  name: string
  path: string
  poolName: string
  capacityBytes: number
  allocationBytes: number
  format: string
}

export interface VolumeConfig {
  name: string
  capacityGb: number
  format: 'qcow2' | 'raw'
}

// Snapshot types
export type SnapshotState = 'disksnapshot' | 'running' | 'paused' | 'shutoff'

export interface Snapshot {
  name: string
  description?: string
  creationTime: number
  state: SnapshotState
  parent?: string
  isCurrent: boolean
}

export interface SnapshotConfig {
  name: string
  description?: string
  includeMemory?: boolean
}
