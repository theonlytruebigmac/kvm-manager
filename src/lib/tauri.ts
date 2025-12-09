import { invoke } from '@tauri-apps/api/core'
import type { VM, HostInfo, ConnectionStatus, VmConfig, VncInfo, VmStats, Network, NetworkConfig, StoragePool, Volume, VolumeConfig, StoragePoolConfig, Snapshot, SnapshotConfig, VmMetrics, HistoricalMetrics, VmTemplate, CreateTemplateRequest, ScheduledOperation, CreateScheduleRequest, ResourceAlert, CreateAlertRequest, AlertEvent, BackupConfig, CreateBackupRequest, BatchOperationResult, OptimizationSuggestion, RetentionPolicy, GuestAgentStatus, GuestSystemInfo, GuestNetworkInfo, GuestDiskUsage, GuestCommandResult } from './types'

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
  deleteVm: (vmId: string, deleteDisks: boolean, deleteSnapshots: boolean) => invoke<void>('delete_vm', { vmId, deleteDisks, deleteSnapshots }),
  cloneVm: (sourceVmId: string, newName: string) => invoke<string>('clone_vm', { sourceVmId, newName }),
  createVm: (config: VmConfig) => invoke<string>('create_vm', { config }),
  addVmTags: (vmId: string, tags: string[]) => invoke<void>('add_vm_tags', { vmId, tags }),
  removeVmTags: (vmId: string, tags: string[]) => invoke<void>('remove_vm_tags', { vmId, tags }),
  exportVm: (vmId: string) => invoke<string>('export_vm', { vmId }),
  importVm: (xml: string) => invoke<string>('import_vm', { xml }),
  attachDisk: (vmId: string, diskPath: string, deviceTarget: string, busType: string) => invoke<void>('attach_disk', { vmId, diskPath, deviceTarget, busType }),
  detachDisk: (vmId: string, deviceTarget: string) => invoke<void>('detach_disk', { vmId, deviceTarget }),

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
  addPortForward: (hostPort: number, guestIp: string, guestPort: number, protocol: string) => invoke<void>('add_port_forward', { hostPort, guestIp, guestPort, protocol }),
  removePortForward: (hostPort: number, guestIp: string, guestPort: number, protocol: string) => invoke<void>('remove_port_forward', { hostPort, guestIp, guestPort, protocol }),

  // Storage Operations
  getStoragePools: () => invoke<StoragePool[]>('get_storage_pools'),
  getVolumes: (poolId: string) => invoke<Volume[]>('get_volumes', { poolId }),
  createVolume: (poolId: string, config: VolumeConfig) => invoke<string>('create_volume', { poolId, config }),
  deleteVolume: (poolId: string, volumeName: string) => invoke<void>('delete_volume', { poolId, volumeName }),
  createStoragePool: (config: StoragePoolConfig) => invoke<string>('create_storage_pool', { config }),
  resizeVolume: (poolId: string, volumeName: string, newCapacityGb: number) => invoke<void>('resize_volume', { poolId, volumeName, newCapacityGb }),

  // Snapshot Operations
  getSnapshots: (vmId: string) => invoke<Snapshot[]>('get_snapshots', { vmId }),
  createSnapshot: (vmId: string, config: SnapshotConfig) => invoke<string>('create_snapshot', { vmId, config }),
  deleteSnapshot: (vmId: string, snapshotName: string) => invoke<void>('delete_snapshot', { vmId, snapshotName }),
  revertSnapshot: (vmId: string, snapshotName: string) => invoke<void>('revert_snapshot', { vmId, snapshotName }),

  // Metrics Operations
  storeVmMetrics: (metrics: VmMetrics) => invoke<void>('store_vm_metrics', { metrics }),
  getHistoricalMetrics: (vmId: string, startTime: number, endTime: number, maxPoints?: number) => invoke<HistoricalMetrics>('get_historical_metrics', { vmId, startTime, endTime, maxPoints }),
  cleanupOldMetrics: (beforeTimestamp: number) => invoke<void>('cleanup_old_metrics', { beforeTimestamp }),
  getMetricsCount: (vmId?: string) => invoke<number>('get_metrics_count', { vmId }),

  // Template Operations
  createTemplate: (request: CreateTemplateRequest) => invoke<VmTemplate>('create_template', { request }),
  listTemplates: () => invoke<VmTemplate[]>('list_templates'),
  getTemplate: (id: string) => invoke<VmTemplate>('get_template', { id }),
  updateTemplate: (id: string, request: CreateTemplateRequest) => invoke<VmTemplate>('update_template', { id, request }),
  deleteTemplate: (id: string) => invoke<void>('delete_template', { id }),

  // Scheduler Operations
  createSchedule: (request: CreateScheduleRequest) => invoke<ScheduledOperation>('create_schedule', { request }),
  listSchedules: () => invoke<ScheduledOperation[]>('list_schedules'),
  getSchedule: (id: string) => invoke<ScheduledOperation>('get_schedule', { id }),
  updateScheduleStatus: (id: string, enabled: boolean) => invoke<ScheduledOperation>('update_schedule_status', { id, enabled }),
  deleteSchedule: (id: string) => invoke<void>('delete_schedule', { id }),
  getVmSchedules: (vmId: string) => invoke<ScheduledOperation[]>('get_vm_schedules', { vmId }),

  // Alert Operations
  createAlert: (request: CreateAlertRequest) => invoke<ResourceAlert>('create_alert', { request }),
  listAlerts: () => invoke<ResourceAlert[]>('list_alerts'),
  getAlert: (id: string) => invoke<ResourceAlert>('get_alert', { id }),
  updateAlertStatus: (id: string, enabled: boolean) => invoke<ResourceAlert>('update_alert_status', { id, enabled }),
  deleteAlert: (id: string) => invoke<void>('delete_alert', { id }),
  getVmAlerts: (vmId: string) => invoke<ResourceAlert[]>('get_vm_alerts', { vmId }),
  checkAlertThreshold: (alertId: string, currentValue: number) => invoke<AlertEvent | null>('check_alert_threshold', { alertId, currentValue }),

  // Backup Operations
  createBackupConfig: (request: CreateBackupRequest) => invoke<BackupConfig>('create_backup_config', { request }),
  listBackupConfigs: () => invoke<BackupConfig[]>('list_backup_configs'),
  getBackupConfig: (id: string) => invoke<BackupConfig>('get_backup_config', { id }),
  updateBackupStatus: (id: string, enabled: boolean) => invoke<BackupConfig>('update_backup_status', { id, enabled }),
  deleteBackupConfig: (id: string) => invoke<void>('delete_backup_config', { id }),
  getVmBackupConfigs: (vmId: string) => invoke<BackupConfig[]>('get_vm_backup_configs', { vmId }),
  recordBackup: (id: string) => invoke<BackupConfig>('record_backup', { id }),

  // Batch Operations
  batchStartVms: (vmIds: string[]) => invoke<BatchOperationResult[]>('batch_start_vms', { vmIds }),
  batchStopVms: (vmIds: string[], force: boolean) => invoke<BatchOperationResult[]>('batch_stop_vms', { vmIds, force }),
  batchRebootVms: (vmIds: string[]) => invoke<BatchOperationResult[]>('batch_reboot_vms', { vmIds }),

  // Optimization Operations
  analyzeVmPerformance: (vmId: string, vmName: string, timeRangeHours?: number) => invoke<OptimizationSuggestion[]>('analyze_vm_performance', { vmId, vmName, timeRangeHours }),
  analyzeAllVms: (timeRangeHours?: number) => invoke<OptimizationSuggestion[]>('analyze_all_vms', { timeRangeHours }),

  // Retention Policy Operations
  getRetentionPolicy: () => invoke<RetentionPolicy>('get_retention_policy'),
  updateRetentionPolicy: (policy: RetentionPolicy) => invoke<RetentionPolicy>('update_retention_policy', { policy }),
  executeRetentionCleanup: () => invoke<number>('execute_retention_cleanup'),

  // Guest Agent Operations
  checkGuestAgentStatus: (vmName: string) => invoke<GuestAgentStatus>('check_guest_agent_status', { vmName }),
  getGuestSystemInfo: (vmName: string) => invoke<GuestSystemInfo>('get_guest_system_info', { vmName }),
  getGuestNetworkInfo: (vmName: string) => invoke<GuestNetworkInfo>('get_guest_network_info', { vmName }),
  getGuestDiskUsage: (vmName: string) => invoke<GuestDiskUsage>('get_guest_disk_usage', { vmName }),
  executeGuestCommand: (vmName: string, command: string, args: string[], timeoutSeconds?: number) => invoke<GuestCommandResult>('execute_guest_command', { vmName, command, args, timeoutSeconds }),
  readGuestFile: (vmName: string, path: string) => invoke<string>('read_guest_file', { vmName, path }),
  writeGuestFile: (vmName: string, path: string, content: string, createDirs?: boolean) => invoke<void>('write_guest_file', { vmName, path, content, createDirs }),
  guestAgentShutdown: (vmName: string, force?: boolean) => invoke<void>('guest_agent_shutdown', { vmName, force }),
  guestAgentReboot: (vmName: string, force?: boolean) => invoke<void>('guest_agent_reboot', { vmName, force }),

  // Guest Agent Installation
  mountGuestAgentIso: (vmId: string) => invoke<void>('mount_guest_agent_iso', { vmId }),
  ejectCdrom: (vmId: string) => invoke<void>('eject_cdrom', { vmId }),
}
