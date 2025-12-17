import { invoke } from '@tauri-apps/api/core'
import type { VM, HostInfo, ConnectionStatus, VmConfig, VncInfo, VmStats, MigrationInfo, Network, NetworkConfig, NetworkDetails, DhcpLease, NwFilter, NwFilterConfig, StoragePool, Volume, VolumeConfig, VolumeEncryptionInfo, StoragePoolConfig, OvfMetadata, OvaImportConfig, Snapshot, SnapshotConfig, VmMetrics, HistoricalMetrics, VmTemplate, CreateTemplateRequest, ScheduledOperation, CreateScheduleRequest, ResourceAlert, CreateAlertRequest, AlertEvent, BackupConfig, CreateBackupRequest, BatchOperationResult, OptimizationSuggestion, RetentionPolicy, GuestAgentStatus, GuestSystemInfo, GuestNetworkInfo, GuestDiskUsage, GuestCommandResult, GuestCpuStats, GuestDiskStats, GuestUser, GuestTimezone, GuestFullInfo, SavedConnection, ConnectionType, KernelBootSettings, CloneConfig } from './types'

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
  hibernateVm: (vmId: string) => invoke<void>('hibernate_vm', { vmId }),
  hasManagedSave: (vmId: string) => invoke<boolean>('has_managed_save', { vmId }),
  removeManagedSave: (vmId: string) => invoke<void>('remove_managed_save', { vmId }),
  rebootVm: (vmId: string) => invoke<void>('reboot_vm', { vmId }),
  deleteVm: (vmId: string, deleteDisks: boolean, deleteSnapshots: boolean) => invoke<void>('delete_vm', { vmId, deleteDisks, deleteSnapshots }),
  cloneVm: (sourceVmId: string, newName: string) => invoke<string>('clone_vm', { sourceVmId, newName }),
  cloneVmWithOptions: (sourceVmId: string, config: CloneConfig) => invoke<string>('clone_vm_with_options', { sourceVmId, config }),
  createVm: (config: VmConfig) => invoke<string>('create_vm', { config }),
  addVmTags: (vmId: string, tags: string[]) => invoke<void>('add_vm_tags', { vmId, tags }),
  removeVmTags: (vmId: string, tags: string[]) => invoke<void>('remove_vm_tags', { vmId, tags }),
  exportVm: (vmId: string) => invoke<string>('export_vm', { vmId }),
  importVm: (xml: string) => invoke<string>('import_vm', { xml }),
  attachDisk: (vmId: string, diskPath: string, deviceTarget: string, busType: string) => invoke<void>('attach_disk', { vmId, diskPath, deviceTarget, busType }),
  detachDisk: (vmId: string, deviceTarget: string) => invoke<void>('detach_disk', { vmId, deviceTarget }),
  updateDiskSettings: (
    vmId: string,
    deviceTarget: string,
    settings: {
      cache?: string
      io?: string
      discard?: string
      detectZeroes?: string
      readIopsSec?: number
      writeIopsSec?: number
      readBytesSec?: number
      writeBytesSec?: number
    }
  ) => invoke<void>('update_disk_settings', {
    vmId,
    deviceTarget,
    cache: settings.cache,
    io: settings.io,
    discard: settings.discard,
    detectZeroes: settings.detectZeroes,
    readIopsSec: settings.readIopsSec,
    writeIopsSec: settings.writeIopsSec,
    readBytesSec: settings.readBytesSec,
    writeBytesSec: settings.writeBytesSec,
  }),

  // Direct Kernel Boot Operations
  getKernelBootSettings: (vmId: string) => invoke<KernelBootSettings>('get_kernel_boot_settings', { vmId }),
  setKernelBootSettings: (
    vmId: string,
    settings: {
      enabled: boolean
      kernelPath?: string
      initrdPath?: string
      kernelArgs?: string
      dtbPath?: string
    }
  ) => invoke<void>('set_kernel_boot_settings', {
    vmId,
    enabled: settings.enabled,
    kernelPath: settings.kernelPath,
    initrdPath: settings.initrdPath,
    kernelArgs: settings.kernelArgs,
    dtbPath: settings.dtbPath,
  }),

  // Hugepages Memory Operations
  getHugepagesSettings: (vmId: string) => invoke<import('./types').HugepagesSettings>('get_hugepages_settings', { vmId }),
  setHugepages: (vmId: string, enabled: boolean, size?: number) =>
    invoke<void>('set_hugepages', { vmId, enabled, size }),
  getHostHugepageInfo: () => invoke<import('./types').HugepageInfo[]>('get_host_hugepage_info'),

  // USB Redirection Operations (for SPICE)
  getUsbRedirection: (vmId: string) => invoke<import('./types').UsbRedirectionInfo>('get_usb_redirection', { vmId }),
  attachUsbRedirection: (vmId: string, count: number) =>
    invoke<void>('attach_usb_redirection', { vmId, count }),
  removeUsbRedirection: (vmId: string) => invoke<void>('remove_usb_redirection', { vmId }),

  // Evdev Input Passthrough Operations
  listEvdevDevices: () => invoke<import('./types').EvdevDevice[]>('list_evdev_devices'),
  attachEvdev: (vmId: string, devicePath: string, grabAll: boolean) =>
    invoke<void>('attach_evdev', { vmId, devicePath, grabAll }),
  getVmEvdevDevices: (vmId: string) => invoke<import('./types').EvdevDevice[]>('get_vm_evdev_devices', { vmId }),
  detachEvdev: (vmId: string, devicePath: string) =>
    invoke<void>('detach_evdev', { vmId, devicePath }),

  // Network Bandwidth/QoS Operations
  updateInterfaceBandwidth: (
    vmId: string,
    macAddress: string,
    settings: {
      inboundAverage?: number
      inboundPeak?: number
      inboundBurst?: number
      outboundAverage?: number
      outboundPeak?: number
      outboundBurst?: number
    }
  ) => invoke<void>('update_interface_bandwidth', {
    vmId,
    macAddress,
    inboundAverage: settings.inboundAverage,
    inboundPeak: settings.inboundPeak,
    inboundBurst: settings.inboundBurst,
    outboundAverage: settings.outboundAverage,
    outboundPeak: settings.outboundPeak,
    outboundBurst: settings.outboundBurst,
  }),

  // Network Interface Link State Operations
  setInterfaceLinkState: (vmId: string, macAddress: string, linkUp: boolean) =>
    invoke<void>('set_interface_link_state', { vmId, macAddress, linkUp }),
  getInterfaceLinkState: (vmId: string, macAddress: string) =>
    invoke<boolean>('get_interface_link_state', { vmId, macAddress }),

  // Migration Operations
  migrateVm: (vmId: string, destUri: string, live: boolean, unsafeMigration: boolean) =>
    invoke<void>('migrate_vm', { vmId, destUri, live, unsafeMigration }),
  getMigrationInfo: (vmId: string) => invoke<MigrationInfo>('get_migration_info', { vmId }),
  checkMigrationCompatibility: (vmId: string) =>
    invoke<[boolean, string[]]>('check_migration_compatibility', { vmId }),
  getMigrationTargets: () => invoke<[string, string][]>('get_migration_targets'),

  // Console Operations
  getVncInfo: (vmId: string) => invoke<VncInfo>('get_vnc_info', { vmId }),
  openVncConsole: (vmId: string) => invoke<void>('open_vnc_console', { vmId }),
  stopVncProxy: (vmId: string) => invoke<void>('stop_vnc_proxy', { vmId }),

  // Performance Operations
  getVmStats: (vmId: string) => invoke<VmStats>('get_vm_stats', { vmId }),

  // System Operations
  getHostInfo: () => invoke<HostInfo>('get_host_info'),
  getConnectionStatus: () => invoke<ConnectionStatus>('get_connection_status'),

  // Connection Management
  getSavedConnections: () => invoke<SavedConnection[]>('get_saved_connections'),
  getActiveConnection: () => invoke<SavedConnection | null>('get_active_connection'),
  connectTo: (connectionId: string) => invoke<void>('connect_to', { connectionId }),
  disconnectFrom: (connectionId: string) => invoke<void>('disconnect_from', { connectionId }),
  addConnection: (
    name: string,
    connectionType: ConnectionType,
    host?: string,
    username?: string,
    sshPort?: number,
    tlsPort?: number,
    autoConnect?: boolean
  ) => invoke<SavedConnection>('add_connection', {
    name,
    connectionType,
    host,
    username,
    sshPort,
    tlsPort,
    autoConnect: autoConnect ?? false,
  }),
  updateConnection: (
    id: string,
    name: string,
    connectionType: ConnectionType,
    host?: string,
    username?: string,
    sshPort?: number,
    tlsPort?: number,
    autoConnect?: boolean
  ) => invoke<void>('update_connection', {
    id,
    name,
    connectionType,
    host,
    username,
    sshPort,
    tlsPort,
    autoConnect: autoConnect ?? false,
  }),
  removeConnection: (connectionId: string) => invoke<void>('remove_connection', { connectionId }),
  testConnection: (
    connectionType: ConnectionType,
    host?: string,
    username?: string,
    sshPort?: number,
    tlsPort?: number
  ) => invoke<string>('test_connection', { connectionType, host, username, sshPort, tlsPort }),

  // Network Operations
  getNetworks: () => invoke<Network[]>('get_networks'),
  getNetwork: (networkName: string) => invoke<Network>('get_network', { networkName }),
  createNetwork: (config: NetworkConfig) => invoke<string>('create_network', { config }),
  deleteNetwork: (networkName: string) => invoke<void>('delete_network', { networkName }),
  startNetwork: (networkName: string) => invoke<void>('start_network', { networkName }),
  stopNetwork: (networkName: string) => invoke<void>('stop_network', { networkName }),
  addPortForward: (hostPort: number, guestIp: string, guestPort: number, protocol: string) => invoke<void>('add_port_forward', { hostPort, guestIp, guestPort, protocol }),
  removePortForward: (hostPort: number, guestIp: string, guestPort: number, protocol: string) => invoke<void>('remove_port_forward', { hostPort, guestIp, guestPort, protocol }),
  setNetworkAutostart: (networkName: string, autostart: boolean) => invoke<void>('set_network_autostart', { networkName, autostart }),
  getNetworkDetails: (networkName: string) => invoke<NetworkDetails>('get_network_details', { networkName }),
  getDhcpLeases: (networkName: string) => invoke<DhcpLease[]>('get_dhcp_leases', { networkName }),

  // Network Filter Operations
  getNwfilters: () => invoke<NwFilter[]>('get_nwfilters'),
  getNwfilter: (name: string) => invoke<NwFilter>('get_nwfilter', { name }),
  getNwfilterXml: (name: string) => invoke<string>('get_nwfilter_xml', { name }),
  createNwfilter: (config: NwFilterConfig) => invoke<NwFilter>('create_nwfilter', { config }),
  createNwfilterFromXml: (xml: string) => invoke<NwFilter>('create_nwfilter_from_xml', { xml }),
  deleteNwfilter: (name: string) => invoke<void>('delete_nwfilter', { name }),

  // Storage Operations
  getStoragePools: () => invoke<StoragePool[]>('get_storage_pools'),
  getVolumes: (poolId: string) => invoke<Volume[]>('get_volumes', { poolId }),
  createVolume: (poolId: string, config: VolumeConfig) => invoke<string>('create_volume', { poolId, config }),
  deleteVolume: (poolId: string, volumeName: string) => invoke<void>('delete_volume', { poolId, volumeName }),
  createStoragePool: (config: StoragePoolConfig) => invoke<string>('create_storage_pool', { config }),
  resizeVolume: (poolId: string, volumeName: string, newCapacityGb: number) => invoke<void>('resize_volume', { poolId, volumeName, newCapacityGb }),
  uploadVolume: (poolId: string, volumeName: string, sourcePath: string, format?: string) =>
    invoke<Volume>('upload_volume', { poolId, volumeName, sourcePath, format }),
  downloadVolume: (poolId: string, volumeName: string, destPath: string) =>
    invoke<number>('download_volume', { poolId, volumeName, destPath }),
  getVolumePath: (poolId: string, volumeName: string) =>
    invoke<string>('get_volume_path', { poolId, volumeName }),
  getVolumeEncryptionInfo: (poolId: string, volumeName: string) =>
    invoke<VolumeEncryptionInfo>('get_volume_encryption_info', { poolId, volumeName }),

  // OVA/OVF Import
  getOvaMetadata: (sourcePath: string) =>
    invoke<OvfMetadata>('get_ova_metadata', { sourcePath }),
  importOva: (config: OvaImportConfig) =>
    invoke<string>('import_ova', { config }),

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
  // Extended Guest Agent Operations
  getGuestCpuStats: (vmName: string) => invoke<GuestCpuStats>('get_guest_cpu_stats', { vmName }),
  getGuestDiskStats: (vmName: string) => invoke<GuestDiskStats>('get_guest_disk_stats', { vmName }),
  getGuestUsers: (vmName: string) => invoke<GuestUser[]>('get_guest_users', { vmName }),
  getGuestTimezone: (vmName: string) => invoke<GuestTimezone>('get_guest_timezone', { vmName }),
  getGuestFullInfo: (vmName: string) => invoke<GuestFullInfo>('get_guest_full_info', { vmName }),

  // CD-ROM/ISO Management
  mountIso: (vmId: string, isoPath: string) => invoke<void>('mount_iso', { vmId, isoPath }),
  mountGuestAgentIso: (vmId: string) => invoke<void>('mount_guest_agent_iso', { vmId }),
  ejectCdrom: (vmId: string) => invoke<void>('eject_cdrom', { vmId }),

  // Boot Order
  updateVmBootOrder: (vmId: string, bootOrder: string[]) => invoke<void>('update_vm_boot_order', { vmId, bootOrder }),

  // VM Rename
  renameVm: (vmId: string, newName: string) => invoke<void>('rename_vm', { vmId, newName }),

  // VM Autostart
  getVmAutostart: (vmId: string) => invoke<boolean>('get_vm_autostart', { vmId }),
  setVmAutostart: (vmId: string, enable: boolean) => invoke<void>('set_vm_autostart', { vmId, enable }),

  // VM CPU/Memory Configuration
  setVmVcpus: (vmId: string, vcpus: number) => invoke<void>('set_vm_vcpus', { vmId, vcpus }),
  setVmMemory: (vmId: string, memoryMb: number) => invoke<void>('set_vm_memory', { vmId, memoryMb }),
  setVmCpuTopology: (vmId: string, sockets: number, cores: number, threads: number) =>
    invoke<void>('set_vm_cpu_topology', { vmId, sockets, cores, threads }),

  // CPU Model Configuration
  getCpuModel: (vmId: string) => invoke<import('./types').CpuModelConfig>('get_cpu_model', { vmId }),
  setCpuModel: (vmId: string, mode: string, model?: string) =>
    invoke<void>('set_cpu_model', { vmId, mode, model }),
  getAvailableCpuModels: () => invoke<string[]>('get_available_cpu_models'),

  // CPU Pinning
  getCpuPinning: (vmId: string) => invoke<[number, number[]][]>('get_cpu_pinning', { vmId }),
  setCpuPin: (vmId: string, vcpu: number, hostCpus: number[]) =>
    invoke<void>('set_cpu_pin', { vmId, vcpu, hostCpus }),
  clearCpuPin: (vmId: string, vcpu: number) =>
    invoke<void>('clear_cpu_pin', { vmId, vcpu }),

  // VM Network Interfaces
  attachInterface: (vmId: string, network: string, model: string, macAddress?: string) =>
    invoke<string>('attach_interface', { vmId, network, model, macAddress }),
  attachInterfaceAdvanced: (
    vmId: string,
    interfaceType: import('./types').InterfaceType,
    source: string,
    model: string,
    options?: {
      macAddress?: string,
      sourceMode?: import('./types').DirectMode,
      vlanId?: number,
      portgroup?: string,
      mtu?: number,
    }
  ) => invoke<string>('attach_interface_advanced', {
    vmId,
    interfaceType,
    source,
    model,
    macAddress: options?.macAddress,
    sourceMode: options?.sourceMode,
    vlanId: options?.vlanId,
    portgroup: options?.portgroup,
    mtu: options?.mtu,
  }),
  listHostInterfaces: () => invoke<import('./types').HostNetworkInterface[]>('list_host_interfaces'),
  detachInterface: (vmId: string, macAddress: string) =>
    invoke<void>('detach_interface', { vmId, macAddress }),

  // VM Sound Devices
  attachSound: (vmId: string, model: string) =>
    invoke<void>('attach_sound', { vmId, model }),
  detachSound: (vmId: string) =>
    invoke<void>('detach_sound', { vmId }),

  // VM Input Devices
  attachInput: (vmId: string, deviceType: string, bus: string) =>
    invoke<void>('attach_input', { vmId, deviceType, bus }),

  // VM RNG Device
  attachRng: (vmId: string, backend: string = '/dev/urandom') =>
    invoke<void>('attach_rng', { vmId, backend }),

  // VM Watchdog Device
  attachWatchdog: (vmId: string, model: string, action: string = 'reset') =>
    invoke<void>('attach_watchdog', { vmId, model, action }),

  // VM Channel Device
  attachChannel: (vmId: string, channelType: string) =>
    invoke<void>('attach_channel', { vmId, channelType }),

  // VM Filesystem Sharing
  attachFilesystem: (vmId: string, sourcePath: string, targetMount: string, fsType: string, readonly: boolean = false) =>
    invoke<void>('attach_filesystem', { vmId, sourcePath, targetMount, fsType, readonly }),

  // PCI Passthrough
  listPciDevices: () => invoke<import('./types').PciDevice[]>('list_pci_devices'),
  checkIommuStatus: () => invoke<import('./types').IommuStatus>('check_iommu_status'),
  getIommuGroups: () => invoke<import('./types').IommuGroup[]>('get_iommu_groups'),
  attachPciDevice: (vmId: string, pciAddress: string, managed: boolean = true) =>
    invoke<void>('attach_pci_device', { vmId, pciAddress, managed }),
  detachPciDevice: (vmId: string, pciAddress: string) =>
    invoke<void>('detach_pci_device', { vmId, pciAddress }),
  getVfioStatus: (pciAddress: string) =>
    invoke<import('./types').VfioStatus>('get_vfio_status', { pciAddress }),
  bindToVfio: (pciAddress: string) =>
    invoke<void>('bind_to_vfio', { pciAddress }),
  unbindFromVfio: (pciAddress: string) =>
    invoke<void>('unbind_from_vfio', { pciAddress }),

  // USB Passthrough
  listUsbDevices: () => invoke<import('./types').UsbDevice[]>('list_usb_devices'),
  attachUsbDevice: (vmId: string, vendorId: string, productId: string) =>
    invoke<void>('attach_usb_device', { vmId, vendorId, productId }),
  detachUsbDevice: (vmId: string, vendorId: string, productId: string) =>
    invoke<void>('detach_usb_device', { vmId, vendorId, productId }),
  getVmUsbDevices: (vmId: string) => invoke<import('./types').UsbDevice[]>('get_vm_usb_devices', { vmId }),

  // Graphics Device
  attachGraphics: (vmId: string, graphicsType: string, listenAddress?: string, port?: number) =>
    invoke<void>('attach_graphics', { vmId, graphicsType, listenAddress, port }),

  // Video Device
  attachVideo: (vmId: string, model: string, vram?: number, heads?: number, acceleration3d?: boolean) =>
    invoke<void>('attach_video', { vmId, model, vram, heads, acceleration3d }),

  // MDEV (vGPU) Passthrough
  checkMdevStatus: () => invoke<import('./types').MdevStatus>('check_mdev_status'),
  listMdevTypes: () => invoke<import('./types').MdevType[]>('list_mdev_types'),
  listMdevDevices: () => invoke<import('./types').MdevDevice[]>('list_mdev_devices'),
  attachMdev: (vmId: string, mdevUuid: string) =>
    invoke<void>('attach_mdev', { vmId, mdevUuid }),
  detachMdev: (vmId: string, mdevUuid: string) =>
    invoke<void>('detach_mdev', { vmId, mdevUuid }),
  createMdev: (parentDevice: string, mdevType: string) =>
    invoke<string>('create_mdev', { parentDevice, mdevType }),
  removeMdev: (mdevUuid: string) =>
    invoke<void>('remove_mdev', { mdevUuid }),

  // SR-IOV Network Passthrough
  listSriovPfs: () => invoke<import('./types').SriovPf[]>('list_sriov_pfs'),
  listSriovVfs: (pfAddress: string) =>
    invoke<import('./types').SriovVf[]>('list_sriov_vfs', { pfAddress }),
  enableSriovVfs: (pfAddress: string, numVfs: number) =>
    invoke<void>('enable_sriov_vfs', { pfAddress, numVfs }),
  configureSriovVf: (config: import('./types').SriovVfConfig) =>
    invoke<void>('configure_sriov_vf', { config }),
  attachSriovVf: (vmId: string, vfAddress: string) =>
    invoke<void>('attach_sriov_vf', { vmId, vfAddress }),
  detachSriovVf: (vmId: string, vfAddress: string) =>
    invoke<void>('detach_sriov_vf', { vmId, vfAddress }),

  // Serial Port Device
  attachSerial: (vmId: string, portType: string, targetPort: number = 0) =>
    invoke<void>('attach_serial', { vmId, portType, targetPort }),

  // Console Device
  attachConsole: (vmId: string, targetPort: number = 0, targetType: string = 'virtio') =>
    invoke<void>('attach_console', { vmId, targetPort, targetType }),

  // TPM Device
  attachTpm: (vmId: string, model: string = 'tpm-crb', version: string = '2.0') =>
    invoke<void>('attach_tpm', { vmId, model, version }),

  // USB Controller
  attachUsbController: (vmId: string, model: string) =>
    invoke<void>('attach_usb_controller', { vmId, model }),

  // SCSI Controller
  attachScsiController: (vmId: string, model: string) =>
    invoke<void>('attach_scsi_controller', { vmId, model }),

  // Panic Notifier
  attachPanicNotifier: (vmId: string, model: string) =>
    invoke<void>('attach_panic_notifier', { vmId, model }),

  // VirtIO VSOCK
  attachVsock: (vmId: string, cid: number) =>
    invoke<void>('attach_vsock', { vmId, cid }),

  // Parallel Port
  attachParallel: (vmId: string, targetPort: number) =>
    invoke<void>('attach_parallel', { vmId, targetPort }),

  // Smartcard Reader
  attachSmartcard: (vmId: string, mode: string) =>
    invoke<void>('attach_smartcard', { vmId, mode }),

  // NUMA Configuration
  getHostNumaTopology: () =>
    invoke<import('./types').HostNumaNode[]>('get_host_numa_topology'),
  getVmNumaConfig: (vmId: string) =>
    invoke<import('./types').VmNumaConfig | null>('get_vm_numa_config', { vmId }),
  setVmNumaConfig: (vmId: string, config: import('./types').VmNumaConfig) =>
    invoke<void>('set_vm_numa_config', { vmId, config }),
  clearVmNumaConfig: (vmId: string) =>
    invoke<void>('clear_vm_numa_config', { vmId }),

  // Window Management
  openVmDetailsWindow: (vmId: string, vmName: string) => invoke<void>('open_vm_details_window', { vmId, vmName }),
  openConsoleWindow: (vmId: string, vmName: string) => invoke<void>('open_console_window', { vmId, vmName }),
  closeVmWindows: (vmId: string) => invoke<void>('close_vm_windows', { vmId }),
  getOpenWindows: () => invoke<string[]>('get_open_windows'),
  closeWindow: (windowLabel: string) => invoke<void>('close_window', { windowLabel }),

  // Serial Console
  getSerialConsoleInfo: (vmId: string) => invoke<import('./types').SerialConsoleInfo>('get_serial_console_info', { vmId }),
  openSerialConsole: (vmId: string) => invoke<import('./types').SerialConsoleInfo>('open_serial_console', { vmId }),
  closeSerialConsole: (vmId: string) => invoke<void>('close_serial_console', { vmId }),
  readSerialConsole: (vmId: string) => invoke<string>('read_serial_console', { vmId }),
  writeSerialConsole: (vmId: string, input: string) => invoke<void>('write_serial_console', { vmId, input }),
  isSerialConsoleConnected: (vmId: string) => invoke<boolean>('is_serial_console_connected', { vmId }),
}
