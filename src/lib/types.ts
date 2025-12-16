// TypeScript types matching the Rust backend models
// Based on .agents/integration/tauri-commands.md

export type VmState = 'running' | 'stopped' | 'paused' | 'suspended' | 'crashed'

export interface NetworkInterface {
  name: string
  macAddress: string
  mac: string  // Alias for macAddress for backwards compatibility
  network: string
  ipAddress?: string
  type?: string  // NIC model type (virtio, e1000, etc.)
  // Bandwidth/QoS settings (in KB/s for average, peak; KB for burst)
  inboundAverage?: number
  inboundPeak?: number
  inboundBurst?: number
  outboundAverage?: number
  outboundPeak?: number
  outboundBurst?: number
}

export interface DiskDevice {
  device: string
  path: string
  diskType: string
  bus: string
  // I/O tuning options
  cache?: string  // none, writeback, writethrough, directsync, unsafe
  io?: string  // native, threads, io_uring
  discard?: string  // unmap, ignore
  detectZeroes?: string  // off, on, unmap
  // I/O throttling
  readIopsSec?: number
  writeIopsSec?: number
  readBytesSec?: number
  writeBytesSec?: number
}

export interface VM {
  id: string
  name: string
  state: VmState
  cpuCount: number
  cpus: number  // Alias for cpuCount
  memoryMb: number
  maxMemoryMb?: number  // Maximum memory for ballooning
  osType?: string
  osVariant?: string
  diskSizeGb: number
  networkInterfaces: NetworkInterface[]
  disks: DiskDevice[]
  vncPort?: number
  createdAt?: number
  description?: string
  tags: string[]
  firmware: 'bios' | 'uefi' | 'uefi-secure'
  tpmEnabled: boolean
  tpm?: boolean  // Alias for tpmEnabled
  chipset: 'pc' | 'q35'
  machine?: string  // Machine type (e.g., 'pc-q35-7.2')
  arch?: string  // Architecture (e.g., 'x86_64')
  cpuSockets: number
  cpuCores: number
  cpuThreads: number
  topology?: {
    sockets: number
    cores: number
    threads: number
  }
  cdrom?: boolean | string  // CDROM device
  bootMenu?: boolean
  bootOrder?: string[]  // Boot device priority order
  graphics?: {
    type: string
  }
  video?: {
    model: string
    heads?: number
    vram?: number
    accel3d?: boolean
  }
  sound?: boolean
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

// Saved Connection Configuration
export type ConnectionType = 'local' | 'ssh' | 'tls' | 'tcp'

export interface SavedConnection {
  id: string
  name: string
  connectionType: ConnectionType
  hypervisor: string
  host?: string
  username?: string
  sshPort?: number
  tlsPort?: number
  autoConnect: boolean
  path: string
}

// Cloud-Init Configuration
export interface CloudInitNetworkConfig {
  version: number
  configYaml?: string
}

export interface CloudInitConfig {
  enabled: boolean
  username?: string
  password?: string
  sshAuthorizedKeys: string[]
  hostname?: string
  packages: string[]
  runcmd: string[]
  customUserData?: string
  networkConfig?: CloudInitNetworkConfig
  autoEject: boolean
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
  bootOrder: string[]
  firmware: 'bios' | 'uefi' | 'uefi-secure'
  tpmEnabled: boolean
  chipset: 'pc' | 'q35'
  cpuSockets: number
  cpuCores: number
  cpuThreads: number
  cloudInit?: CloudInitConfig
  // Installation type: iso, import (existing disk), network (PXE/URL), or manual (no media)
  installationType?: 'iso' | 'import' | 'network' | 'manual'
  // Path to existing disk image when installationType is 'import'
  existingDiskPath?: string
  // URL for network installation (HTTP/HTTPS/FTP)
  networkInstallUrl?: string
  // Direct kernel boot options
  directKernelBoot?: boolean
  kernelPath?: string
  initrdPath?: string
  kernelArgs?: string
  dtbPath?: string
}

export interface KernelBootSettings {
  enabled: boolean
  kernelPath?: string
  initrdPath?: string
  kernelArgs?: string
  dtbPath?: string
}

export interface HugepagesSettings {
  enabled: boolean
  /** Page size in KiB (2048 = 2MB, 1048576 = 1GB) */
  size?: number
}

export interface HugepageInfo {
  /** Size in KiB */
  sizeKb: number
  /** Total number of hugepages allocated */
  total: number
  /** Number of free hugepages */
  free: number
  /** Human-readable size (e.g., "2 MB", "1 GB") */
  sizeHuman: string
}

export interface CpuModelConfig {
  /** CPU mode: host-passthrough, host-model, custom */
  mode: string
  /** CPU model name (only used when mode is "custom") */
  model?: string
}

export interface UsbRedirectionInfo {
  enabled: boolean
  channelCount: number
  hasSpiceGraphics: boolean
}

export interface EvdevDevice {
  id: string
  path: string
  name: string
  deviceType: string  // keyboard, mouse, joystick, other
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
  websocket_port?: number
  graphics_type?: 'vnc' | 'spice'
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

// Migration types
export interface MigrationInfo {
  canLiveMigrate: boolean
  memoryMb: number
  diskSizeGb: number
  estimatedDowntimeSeconds: number
}

// NUMA configuration types
export interface HostNumaNode {
  id: number
  cpus: number[]
  memoryMb: number
}

export interface VmNumaCell {
  id: number
  cpus: string
  memoryMb: number
  hostNodes?: string
}

export interface VmNumaConfig {
  mode: 'strict' | 'preferred' | 'interleave'
  nodeset?: string
  cells: VmNumaCell[]
}

export interface Network {
  name: string
  uuid: string
  bridge: string
  active: boolean
  autostart: boolean
  ipRange: string | null
}

export interface DhcpLease {
  mac: string
  ipAddress: string
  hostname: string | null
  clientId: string | null
  expiryTime: number
}

export interface NetworkDetails {
  name: string
  uuid: string
  bridge: string
  active: boolean
  autostart: boolean
  ipRange: string | null
  forwardMode: string
  ipAddress: string | null
  netmask: string | null
  dhcpStart: string | null
  dhcpEnd: string | null
  dhcpLeases: DhcpLease[]
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

// Network Filter types
export interface NwFilter {
  uuid: string
  name: string
  ruleCount: number
  chain: string | null
  priority: number | null
}

export type RuleDirection = 'in' | 'out' | 'inout'
export type RuleAction = 'accept' | 'drop' | 'reject' | 'return' | 'continue'

export interface NwFilterRule {
  direction: RuleDirection
  action: RuleAction
  priority?: number
  protocol?: string
  srcIp?: string
  srcMac?: string
  destIp?: string
  destMac?: string
  srcPort?: string
  destPort?: string
  comment?: string
}

export interface NwFilterConfig {
  name: string
  chain?: string
  priority?: number
  rules: NwFilterRule[]
  filterRefs: string[]
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
  /** Enable LUKS encryption */
  encrypted?: boolean
  /** Encryption passphrase (required if encrypted = true) */
  passphrase?: string
}

export interface VolumeEncryptionInfo {
  encrypted: boolean
  format?: string  // "luks", "qcow"
  secretUuid?: string
}

export interface StoragePoolConfig {
  name: string
  poolType: 'dir' | 'logical' | 'netfs' | 'iscsi' | 'gluster' | 'rbd'
  targetPath: string
  autostart: boolean
  sourceDevices?: string[]
  sourceHost?: string
  sourcePath?: string
  // iSCSI specific
  iscsiTarget?: string
  initiatorIqn?: string
  // Gluster specific
  glusterVolume?: string
  // Ceph RBD specific
  rbdPool?: string
  cephMonitors?: string[]
  cephAuthUser?: string
  cephAuthSecret?: string
}

// OVA/OVF Import types
export interface OvfMetadata {
  name: string
  description?: string
  osType?: string
  cpuCount: number
  memoryMb: number
  disks: OvfDisk[]
  networks: OvfNetwork[]
}

export interface OvfDisk {
  id: string
  fileName: string
  capacityBytes: number
  format: string
}

export interface OvfNetwork {
  name: string
  description?: string
}

export interface OvaImportConfig {
  sourcePath: string
  targetPoolPath: string
  vmName?: string
  memoryMb?: number
  cpuCount?: number
  convertToQcow2?: boolean
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

// ===== PCI Passthrough Types =====

export type PciDeviceType = 'gpu' | 'audio' | 'usbcontroller' | 'networkadapter' | 'storagecontroller' | 'other'

export interface PciDevice {
  address: string
  vendor: string
  vendorId: string
  deviceName: string
  deviceId: string
  deviceClass: string
  deviceType: PciDeviceType
  iommuGroup?: number
  inUse: boolean
  attachedToVm?: string
  driver?: string
  passthroughSafe: boolean
}

export interface IommuGroup {
  groupId: number
  devices: string[]
  passthroughSafe: boolean
}

export interface IommuStatus {
  enabled: boolean
  iommuType?: string
  warning?: string
}

// ===== USB Device Types =====

export interface UsbDevice {
  bus: string
  device: string
  vendorId: string
  productId: string
  vendorName: string
  productName: string
  description: string
  speed?: string
  deviceClass?: string
  inUse: boolean
  usedByVm?: string
}

// ===== MDEV (vGPU) Types =====

export interface MdevType {
  name: string
  description?: string
  availableInstances: number
  maxInstances: number
  deviceApi: string
  parentDevice: string
  parentName: string
}

export interface MdevDevice {
  uuid: string
  mdevType: string
  parentDevice: string
  inUse: boolean
  usedByVm?: string
}

export interface MdevStatus {
  supported: boolean
  message: string
  supportedVendors: string[]
}

// ===== Guest Agent Types =====

export interface GuestAgentStatus {
  available: boolean
  agentInfo?: AgentInfo
}

export interface AgentInfo {
  version: string
  protocolVersion: string
  platform: string
  capabilities: string[]
}

export interface GuestSystemInfo {
  osType: string
  osName: string
  osVersion: string
  kernelVersion: string
  hostname: string
  architecture: string
  cpuCount: number
  totalMemoryKb: number
  uptimeSeconds: number
}

export interface GuestNetworkInterface {
  name: string
  macAddress: string
  ipv4Addresses: string[]
  ipv6Addresses: string[]
  state: string
  mtu: number
}

export interface GuestNetworkInfo {
  interfaces: GuestNetworkInterface[]
}

export interface GuestFilesystemInfo {
  mountPoint: string
  device: string
  fsType: string
  totalBytes: number
  usedBytes: number
  availableBytes: number
  usedPercent: number
}

export interface GuestDiskUsage {
  filesystems: GuestFilesystemInfo[]
}

export interface GuestCommandResult {
  exitCode: number
  stdout: string
  stderr: string
  executionTimeMs: number
}

// Extended Guest Agent Types for Performance Monitoring

export interface CpuStats {
  cpu: number
  user: number
  system: number
  idle: number
  iowait: number
  steal: number
  nice: number
  irq: number
  softirq: number
  guest: number
}

export interface GuestCpuStats {
  cpus: CpuStats[]
  totalUser: number
  totalSystem: number
  totalIdle: number
  usagePercent: number
}

export interface DiskStats {
  name: string
  readBytes: number
  writeBytes: number
  readIos: number
  writeIos: number
}

export interface GuestDiskStats {
  disks: DiskStats[]
}

export interface GuestUser {
  username: string
  loginTime: number
}

export interface GuestTimezone {
  zone: string
  offset: number
}

export interface GuestFullInfo {
  agentVersion: string
  osName: string
  osVersion: string
  kernelVersion: string
  hostname: string
  architecture: string
  timezone?: GuestTimezone
  users: GuestUser[]
  cpuCount: number
  filesystems: GuestFilesystemInfo[]
  networkInterfaces: GuestNetworkInterface[]
}

// Serial Console types
export interface SerialConsoleInfo {
  vmId: string
  vmName: string
  ptyPath: string
  active: boolean
}
