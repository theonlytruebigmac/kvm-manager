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

export interface StoragePoolConfig {
  name: string
  poolType: 'dir' | 'logical' | 'netfs'
  targetPath: string
  autostart: boolean
  sourceDevices?: string[]
  sourceHost?: string
  sourcePath?: string
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

// Historical Metrics types
export interface VmMetrics {
  vmId: string
  timestamp: number
  cpuUsage: number
  memoryUsageMb: number
  memoryTotalMb: number
  diskReadBytesPerSec: number
  diskWriteBytesPerSec: number
  networkRxBytesPerSec: number
  networkTxBytesPerSec: number
}

export interface MetricDataPoint {
  timestamp: number
  value: number
}

export interface HistoricalMetrics {
  vmId: string
  cpuUsage: MetricDataPoint[]
  memoryUsage: MetricDataPoint[]
  diskRead: MetricDataPoint[]
  diskWrite: MetricDataPoint[]
  networkRx: MetricDataPoint[]
  networkTx: MetricDataPoint[]
}

// Template types
export interface VmTemplate {
  id: string
  name: string
  description: string
  config: VmConfig
  createdAt: number
  updatedAt: number
}

export interface CreateTemplateRequest {
  name: string
  description: string
  config: VmConfig
}

// Scheduler types
export type ScheduleFrequency = 'once' | 'daily' | 'weekly' | 'monthly'
export type OperationType = 'start' | 'stop' | 'reboot' | 'snapshot'

export interface ScheduledOperation {
  id: string
  name: string
  vmId: string
  operation: OperationType
  frequency: ScheduleFrequency
  scheduledTime: string // Format: "HH:MM"
  dayOfWeek?: number // 0-6 for weekly (0=Sunday)
  dayOfMonth?: number // 1-31 for monthly
  enabled: boolean
  lastRun?: number
  nextRun: number
  createdAt: number
}

export interface CreateScheduleRequest {
  name: string
  vmId: string
  operation: OperationType
  frequency: ScheduleFrequency
  scheduledTime: string
  dayOfWeek?: number
  dayOfMonth?: number
}

// Alert types
export type ThresholdType = 'cpu' | 'memory' | 'disk' | 'network'
export type AlertSeverity = 'info' | 'warning' | 'critical'

export interface ResourceAlert {
  id: string
  name: string
  vmId: string
  thresholdType: ThresholdType
  thresholdValue: number
  severity: AlertSeverity
  enabled: boolean
  consecutiveChecks: number
  currentTriggerCount: number
  lastTriggered?: number
  createdAt: number
}

export interface CreateAlertRequest {
  name: string
  vmId: string
  thresholdType: ThresholdType
  thresholdValue: number
  severity: AlertSeverity
  consecutiveChecks: number
}

export interface AlertEvent {
  alertId: string
  alertName: string
  vmId: string
  thresholdType: ThresholdType
  thresholdValue: number
  currentValue: number
  severity: AlertSeverity
  timestamp: number
}

// Backup types
export interface BackupConfig {
  id: string
  name: string
  vmId: string
  scheduleId: string
  retentionCount: number
  enabled: boolean
  lastBackup?: number
  backupCount: number
  createdAt: number
}

export interface CreateBackupRequest {
  name: string
  vmId: string
  frequency: ScheduleFrequency
  scheduledTime: string
  dayOfWeek?: number
  dayOfMonth?: number
  retentionCount: number
}

// Batch operations types
export interface BatchOperationResult {
  vmId: string
  vmName: string
  success: boolean
  error?: string
}

// Optimization types
export type OptimizationCategory = 'cpu' | 'memory' | 'disk' | 'network'
export type OptimizationSeverity = 'info' | 'warning' | 'critical'

export interface OptimizationSuggestion {
  vmId: string
  vmName: string
  category: OptimizationCategory
  severity: OptimizationSeverity
  title: string
  description: string
  recommendation: string
  currentValue: number
  threshold: number
}

// Retention policy types
export interface RetentionPolicy {
  enabled: boolean
  retentionDays: number
  cleanupHour: number
  lastCleanup?: number
}
