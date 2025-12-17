# KVM Manager - Current Status & Roadmap
**Last Updated**: December 17, 2025 (Phase 4 Backend Integration Progress - Session 14)
**Version**: 0.2.18
**Status**: Phase 4 Complete ✅ | **Desktop UI Redesign: Phase 4 Major Progress**

---

## 🎯 Project Overview

**KVM Manager** is a modern desktop application for managing KVM/QEMU virtual machines on Linux, built with Tauri + React + Rust. The goal is to provide a native desktop experience with feature parity to virt-manager while offering a modern, intuitive UI.

**Key Technologies**:
- **Frontend**: React 19, TypeScript, Vite, TanStack Query, Tailwind CSS, shadcn/ui
- **Backend**: Rust, Tauri 2.x, rust-libvirt
- **Console**: noVNC 1.6.0, SPICE (spice-html5)
- **Platform**: Linux (Ubuntu/Debian primary target)

---

## 🔄 Recent Session Work

**Session 14 Continued (Dec 17) - Performance Monitor & VFIO:**
- ✅ **Performance Monitor Dashboard** (NEW PAGE)
  - Created comprehensive /performance page with real-time monitoring
  - System-wide aggregated stats for all running VMs (CPU, Memory, Network, Disk)
  - Per-VM performance cards with CPU/Memory progress bars
  - Alert thresholds: configurable warning (70%) and critical (90%) levels
  - Visual alert system with badges for VMs exceeding thresholds
  - Alert summary banners: green (all normal), yellow (warnings), red (critical)
  - Real-time aggregated graphs: CPU, Memory, Network I/O, Disk I/O
  - Auto-refresh toggle for live monitoring (1 second intervals)
  - Threshold settings panel with collapsible configuration
  - Navigation: Command palette, Tools menu, /performance route
  - Uses AreaChart for CPU/Memory, LineChart for I/O metrics
- ✅ **VFIO Driver Management** (Complete)
  - Backend: bind_vfio_driver, unbind_vfio_driver, is_device_bound_to_vfio
  - Tauri commands: bind_pci_to_vfio, unbind_pci_from_vfio
  - Frontend: VFIO binding controls in AddHardwareDialog PCI section
  - Driver status badges (green=vfio-pci, yellow=other driver)
  - Bind/Unbind buttons per PCI device with loading state

**Session 14 Continued (Dec 17) - Guest Agent VM List Integration:**
- ✅ **Guest Agent Status in VM Table**
  - Added Bot/BotOff icons to show agent status in Network column
  - Agent status query checks every 10 seconds for running VMs
  - Guest network info fetched from agent for accurate IP addresses
  - IP address priority: Guest agent data → libvirt DHCP leases
  - Tooltip shows guest OS name, version, hostname when agent available
  - IPv4 addresses preferred over IPv6 (excluding link-local)
  - Skips loopback interfaces when looking for IP
- ✅ **Feature Parity Audit**
  - Verified SPICE Console fully integrated (SpiceViewer.tsx in ConsoleWindow)
  - Verified Serial Console fully integrated (SerialConsole.tsx with tabs)
  - Verified Guest Agent UI fully integrated (GuestInfo.tsx in VmDetails)
  - Updated status documentation to reflect accurate feature state
  - VM Console now at 100% complete (VNC + SPICE + Serial)

**Session 14 (Dec 17) - Live Migration Enhancement:**
- ✅ **Enhanced Live Migration Support**
  - Backend: check_migration_compatibility - checks for PCI/MDEV passthrough, local disks, TPM, USB devices
  - Backend: get_migration_targets - lists remote connections from ConnectionService
  - Added MigrationTarget and MigrationCompatibility types to types.ts
  - Added checkMigrationCompatibility and getMigrationTargets to API
  - MigrationDialog already fully implemented with:
    - Saved connection selection or custom URI
    - Live migration toggle (for running VMs)
    - Unsafe migration option (skip storage checks)
    - Migration info display (memory, disk size, estimated downtime)
    - Full integration with VM context menu
  - Registered new commands: check_migration_compatibility, get_migration_targets

**Session 14 (Dec 17) - Clone with Snapshots, IPv6 Networks, USB Hot-plug, SR-IOV & Advanced Networking:**
- ✅ **Enhanced VM Clone with Disk & Snapshot Cloning**
  - Added CloneConfig model with cloneDisks, cloneSnapshots, targetPool, description options
  - Backend: clone_vm_with_options - full-featured clone with disk copying
  - Disk cloning uses qemu-img (tries backing file first, then full copy)
  - Validates source VM is not running before cloning
  - Updates disk paths in cloned VM XML to point to new files
  - Snapshot cloning copies snapshot metadata to cloned VM (disk-only snapshots)
  - Target storage pool selection for cloned disk location
  - Keeps legacy clone_vm for backward compatibility
  - Frontend: CloneVmDialog completely redesigned
  - Clone disks checkbox (creates independent copies)
  - Clone snapshots checkbox (copies snapshot metadata)
  - Advanced options collapsible: target pool selector, description field
  - Warning message when not cloning disks (linked clone warning)
  - Storage pool query for target selection
- ✅ **IPv6 Network Support** - Dual-stack virtual networks
  - Extended NetworkConfig model with IPv6 fields (ipv6Enabled, ipv6Address, ipv6Prefix, ipv6DhcpStart, ipv6DhcpEnd)
  - Backend: Updated create_network to generate IPv6 XML with DHCPv6 range
  - IPv6 section in network XML with family='ipv6' attribute
  - Frontend: IPv6 configuration in Create Network dialog
  - Enable IPv6 switch with collapsible configuration section
  - IPv6 address and prefix length inputs
  - DHCPv6 range configuration (start/end addresses)
  - ULA (fd00::/8) address recommendations for private networks
  - Hidden in bridge mode (no IP config for bridge networks)
- ✅ **USB Hot-plug Support** - Attach/detach USB devices on running VMs
  - Backend: get_vm_usb_devices function in usb_service.rs
  - Parses VM XML to find attached USB hostdev devices
  - Uses lsusb to lookup device vendor/product names
  - Frontend: UsbDeviceManager component with tabs for attached/available devices
  - Attach/detach USB devices with immediate effect on running VMs
  - Auto-refresh every 5 seconds to track device state
  - Shows device details (name, vendor ID, product ID, speed)
  - Integrated into VM context menu as "USB Devices" option
- ✅ **SR-IOV Network Support** - High-performance VF passthrough
  - Backend: sriov_service.rs with full SR-IOV VF management
  - list_sriov_pfs: Scan /sys/bus/pci/devices for SR-IOV capable NICs
  - list_vfs: List Virtual Functions for a Physical Function
  - enable_vfs: Enable/disable VFs on a PF (writes sriov_numvfs)
  - configure_vf: Set MAC, VLAN, spoof check, trust via ip link
  - attach_vf_to_vm: Attach VF to running VM as hostdev
  - detach_vf_from_vm: Detach VF from VM
  - Commands: list_sriov_pfs, list_sriov_vfs, enable_sriov_vfs, configure_sriov_vf, attach_sriov_vf, detach_sriov_vf
  - Frontend: SriovManager component with collapsible PF tree
  - Tabs: Physical Functions view / Attached to VM view
  - +/- buttons to enable/disable VFs on each PF
  - Configure VF dialog: MAC address, VLAN ID, spoof check, trust mode
  - Attach/detach VFs to running VM
  - Shows VF status (available, in use, attached to VM)
  - Integrated into VM context menu as "SR-IOV Network" option
- ✅ **Advanced Network Interface Types** - Macvtap, Bridge, OVS support
  - Backend: attach_interface_advanced with support for multiple interface types
  - Interface types: network (NAT/isolated), bridge, direct (macvtap), ovs (Open vSwitch)
  - Macvtap modes: bridge, vepa, private, passthrough
  - Open vSwitch VLAN tagging support
  - Custom MTU configuration per interface
  - list_host_interfaces: Discover host NICs for macvtap/direct attachment
  - Shows physical interfaces with speed, state, driver info
  - Frontend: AddNetworkInterfaceDialog component
  - Interface type selector with descriptions
  - Dynamic source selection based on type (network list, host interfaces, bridge name)
  - Macvtap mode selector with mode descriptions
  - NIC model selector (virtio recommended, e1000e, rtl8139)
  - Advanced options: MAC address, VLAN ID (OVS), MTU
  - Passthrough mode warning about exclusive NIC access
  - HostNetworkInterface type for host NIC discovery

**Session 13 (Dec 16) - Network Storage & OVA Import:**
- ✅ **Network Storage Pools - GlusterFS & Ceph RBD** - Enterprise storage support
  - Extended StoragePoolConfig with Gluster and RBD specific fields
  - Backend: build_gluster_pool_xml, build_rbd_pool_xml functions
  - GlusterFS: host, volume name, optional subdirectory path
  - Ceph RBD: pool name, multiple monitors, optional auth (user + secret UUID)
  - Frontend: CreateStoragePoolWizard extended with Gluster and RBD options
  - Pool type selector now includes GlusterFS and Ceph RBD
  - Configuration forms for Gluster server/volume and Ceph monitors/auth
  - Review step shows pool-specific configuration summary
  - Contextual warnings for each storage type
  - (NFS and iSCSI were already supported)
- ✅ **OVA/OVF Import** - Import VMs from VMware/VirtualBox
  - New ova_service.rs with full OVF parsing and disk extraction
  - get_ova_metadata: Parse OVF XML to extract VM config (name, CPU, memory, disks, networks)
  - import_ova: Extract OVA tarball, locate disk files, convert to qcow2
  - Supports OVA (single tarball) and OVF (XML + disk files) formats
  - Automatic disk format detection and conversion using qemu-img
  - Backend commands: get_ova_metadata, import_ova
  - Frontend: ImportOvaDialog component in Storage Manager
  - Step-by-step wizard: select file → preview metadata → configure → import
  - Shows detected configuration: CPUs, memory, OS type, disks, networks
  - Target pool selection for disk extraction
  - Optional: Convert to qcow2 (recommended) toggle
  - Progress indicator during import
  - Added "tar" crate dependency for OVA extraction

**Session 12 (Dec 16) - Storage Encryption & Power Management:**
- ✅ **LUKS Disk Encryption** - Create encrypted storage volumes
  - Extended VolumeConfig with encrypted, passphrase fields
  - Libvirt secret management for LUKS passphrases
  - Volume XML with `<encryption format='luks'><secret uuid='...'/></encryption>`
  - Frontend: Encryption toggle in Create Volume dialog
  - Passphrase input with confirmation and visibility toggle
  - Minimum 8 character requirement with validation
  - Warning about passphrase recovery
  - Automatic secret cleanup on volume creation failure
  - API: getVolumeEncryptionInfo for checking volume encryption status
- ✅ **VM Hibernate (Managed Save)** - Suspend VM to disk
  - Backend: hibernate_vm (managed_save), has_managed_save, remove_managed_save
  - Saves VM memory state to disk and stops the VM
  - VM resumes from saved state on next start
  - Frontend: Hibernate action in VM context menus (VmTable, VmContextMenu)
  - Moon icon for hibernate action
  - Toast notifications for hibernate success
  - Useful for: power management, long-running VMs, laptop suspend

**Session 11 (Dec 15) - Network, CPU & Serial Console Improvements:**
- ✅ **NIC Link State Control** - Virtual network cable connect/disconnect
  - Backend: set_interface_link_state, get_interface_link_state
  - virsh domif-setlink for live VMs, XML <link state='up/down'/> for persistent
  - Frontend: Link State toggle in NetworkEditor Advanced tab
  - Visual cable status with Plug/Unplug icons
  - Useful for: network failover testing, DHCP renewal, diagnosing connectivity
- ✅ **CPU Model Selection** - Configurable CPU mode and model
  - Backend: get_cpu_model, set_cpu_model, get_available_cpu_models
  - Modes: host-passthrough (best performance), host-model (migratable), custom
  - Custom mode allows selecting specific CPU models (Intel/AMD architectures)
  - Available models fetched via virsh cpu-models x86_64
  - Frontend: CPU Model section in CpuEditor with mode/model dropdowns
  - Persists topology when changing CPU mode
  - Critical for: live migration compatibility, nested virtualization
- ✅ **Serial Console Configuration** - Proper default serial/console setup
  - VM creation now includes both `<serial>` and `<console>` devices
  - Added virtio-serial controller for guest agent channel support
  - Serial port 0 (COM1/ttyS0) linked to console for proper access
  - Enables: virsh console, guest agent communication, text-mode access

**Session 10 (Dec 15) - Advanced Performance Features:**
- ✅ **Hugepages Memory Support** - Large pages for reduced TLB misses
  - Backend: get_hugepages_settings, set_hugepages, get_host_hugepage_info
  - XML generation for <memoryBacking><hugepages><page size='X'/></hugepages></memoryBacking>
  - Host hugepage info from /sys/kernel/mm/hugepages
  - Frontend: Hugepages card in MemoryEditor with host page sizes
  - Enable/disable switch with page size selector (2MB, 1GB)
  - Critical for: GPU passthrough, gaming VMs, memory-intensive workloads
- ✅ **USB Redirection (SPICE)** - USB device sharing over SPICE
  - Backend: get_usb_redirection, attach_usb_redirection, remove_usb_redirection
  - XML generation for <redirdev bus='usb' type='spicevmc'> channels
  - Configurable channel count (1-4)
  - Frontend: USB Redirection card in GraphicsEditor (SPICE only)
  - Enables: USB devices forwarded through SPICE client
- ✅ **Multi-Monitor Support** - Multiple virtual displays
  - VideoEditor enhanced with editable heads (1-4 monitors)
  - Editable VRAM (16MB-256MB) for multi-head support
  - Already existing backend, now fully exposed in UI
  - Requires: SPICE or VNC graphics, compatible video card
- ✅ **Evdev Input Passthrough** - Direct host input device forwarding
  - Backend: list_evdev_devices, attach_evdev, get_vm_evdev_devices, detach_evdev
  - Scans /dev/input/by-id/ for available keyboards, mice, joysticks
  - XML generation for <input type='evdev'><source dev='...' grab='all'/></input>
  - Frontend: Evdev Passthrough section in InputEditor
  - Available/attached device lists with attach/detach buttons
  - Exclusive grab toggle for gaming use cases
  - Critical for: GPU passthrough, gaming, low-latency input

**Session 9 (Dec 15) - Direct Kernel Boot & Network QoS:**
- ✅ **Direct Kernel Boot** - Boot VMs from host kernel/initrd
  - Extended VmConfig with kernel_path, initrd_path, kernel_args, dtb_path fields
  - Backend functions: get_kernel_boot_settings, set_kernel_boot_settings
  - XML generation for <kernel>, <initrd>, <cmdline>, <dtb> elements
  - Path validation ensures kernel/initrd files exist
  - Frontend: New "Direct Kernel Boot" tab in BootEditor
  - File pickers for kernel, initrd, DTB selection
  - Textarea for kernel command line arguments
  - Useful for: kernel development, debugging, custom boot scenarios
- ✅ **Network QoS / Bandwidth Limits** - Traffic shaping for NICs
  - Extended NetworkInterface model with inbound/outbound bandwidth fields
  - Backend: update_interface_bandwidth with virsh domiftune for live VMs
  - XML parsing extracts existing <bandwidth><inbound/><outbound/> settings
  - Frontend: Functional bandwidth controls in NetworkEditor Advanced tab
  - Settings: average (KB/s), peak (KB/s), burst (KB) for inbound/outbound
  - Live application via virsh domiftune for running VMs
  - Persistent configuration update for stopped VMs

**Session 8 (Dec 15) - Disk I/O Tuning & NVMe Support:**
- ✅ **Disk I/O Tuning** - Full performance options for disk devices
  - Extended DiskDevice model with cache, io, discard, detect_zeroes fields
  - Added I/O throttling support (IOPS and bandwidth limits)
  - Backend: update_disk_settings command with virsh blkdeviotune for live VMs
  - Frontend: DiskEditor with Performance and I/O Throttling tabs
  - Cache modes: none, writeback, writethrough, directsync, unsafe
  - I/O modes: native, threads, io_uring
  - Discard modes: unmap, ignore
  - Live I/O throttling adjustable on running VMs
- ✅ **NVMe Bus Type Support** - High-performance disk bus option
  - Added NVMe to valid bus types in attach_disk
  - Updated AddHardwareDialog with NVMe option
  - Updated DiskManager with NVMe option
  - Shows NVMe in DiskEditor bus configuration

**Session 7 (Dec 15) - Network Install & NUMA Configuration:**
- ✅ **Network Installation Support** - Create VMs with network boot
  - Added `network_install_url` field to VmConfig model (Rust)
  - Added `networkInstallUrl` field to frontend types
  - Updated CreateVmWizard with 4 installation types: ISO, Network, Import, Manual
  - Network interface gets `<boot order='1'/>` for PXE/network boot
  - URL validation for http://, https://, ftp:// protocols
  - Enables installing from Debian/Ubuntu netboot, CentOS/Fedora mirrors, PXE servers
- ✅ **NUMA Configuration** - Full multi-socket NUMA support
  - Backend: get_host_numa_topology, get_vm_numa_config, set_vm_numa_config, clear_vm_numa_config
  - Host NUMA topology detection from /sys/devices/system/node/
  - VM numatune XML generation (mode: strict/preferred/interleave)
  - Frontend: NumaEditor component in Additional Hardware tab
  - Visual node selector with memory info per node
- ✅ **"Customize before install" Workflow** - virt-manager parity feature
  - Added checkbox in CreateVmWizard review step
  - When enabled, VM creation opens VM Details window for hardware customization
  - Matches virt-manager's workflow for pre-install configuration

---

## 🚀 Desktop UI Redesign - IN PROGRESS

**Goal**: Transform from web-app style to native desktop application feel (like virt-manager)

### ✅ Phase 1: Main Window Redesign - COMPLETE
- ✅ Removed persistent sidebar navigation
- ✅ Added desktop-style Toolbar with New VM, Start, Stop, Pause, Console buttons
- ✅ Added ConnectionBar for connection selection
- ✅ Added StatusBar with VM counts and connection info
- ✅ Table-based VM list (VmTable component)

### ✅ Phase 2: Multi-Window Support - COMPLETE
- ✅ Separate VmDetailsWindow for VM configuration
- ✅ Separate ConsoleWindow for VNC console
- ✅ Window state persistence
- ✅ Independent windows per VM

### ✅ Phase 3: Hardware Device Editors - COMPLETE
**Completed (Dec 13-14):**
- ✅ DiskEditor - Full disk configuration UI with detach support
- ✅ CdromEditor - CD-ROM drive configuration
- ✅ NetworkEditor - Network interface configuration with remove support
- ✅ GraphicsEditor - VNC/SPICE graphics configuration
- ✅ VideoEditor - Video device configuration
- ✅ SoundEditor - Sound card configuration
- ✅ InputEditor - Keyboard/Mouse/Tablet configuration
- ✅ TpmEditor - TPM device configuration
- ✅ AddHardwareDialog - Full hardware catalog with working disk/network addition
- ✅ HardwareTree integration with all editors

**Previously Completed:**
- ✅ CpuEditor - vCPU and topology configuration
- ✅ MemoryEditor - Memory allocation
- ✅ BootEditor - Boot order and options with autostart toggle
- ✅ OverviewPanel - VM summary view

### 🔄 Phase 4: Backend Integration - MAJOR PROGRESS
**Completed Today (Dec 15 - Session 7):**
- ✅ **Smartcard Device** - Full implementation
  - Backend: attach_smartcard command (passthrough and emulated modes)
  - Frontend: Smartcard UI in Add Hardware dialog
  - Completes Phase 2 Hardware Device Management (21/21 device types!)
- ✅ **CPU Pinning** - Full implementation
  - Backend: get_cpu_pinning, set_cpu_pin, clear_cpu_pin commands
  - Parses and generates cputune XML with vcpupin elements
  - Frontend: CPU Pinning section in CpuEditor
  - Visual pin editor with host CPU selection grid
- ✅ **Memory Ballooning Support**
  - New VMs created with maxMemory (2x current, capped at 128GB)
  - memballoon virtio device added to new VMs
  - Frontend shows max memory and ballooning status
  - Enables dynamic memory adjustment for running VMs

**Completed Today (Dec 15 - Session 6):**
- ✅ **FIXED: Live Performance Stats** - CPU, Disk I/O, and Network stats now accurate
  - get_vm_stats now uses guest agent for CPU usage when available
  - Falls back to libvirt CPU time delta calculation
  - Disk I/O stats from virsh domblkstat (read/write bytes)
  - Network I/O stats from virsh domifstat (rx/tx bytes)
  - Performance section now matches Guest Agent CPU display

**Completed (Dec 15 - Session 5):**
- ✅ Import Existing Disk Image - Full backend support in create_vm
  - Added installation_type field to VmConfig (iso/import/manual)
  - Added existing_disk_path field for imported disks
  - Automatic disk format detection from file extension
  - Validation of existing disk path
  - Skip volume creation when importing
- ✅ Manual Installation - VM creation without ISO (boot from disk/network)
- ✅ CreateVmWizard UI updates for installation type selection
- ✅ Panic Notifier device (attach_panic_notifier command)
  - Notifies host when guest kernel panics
  - Models: isa, hyperv, pseries
- ✅ VirtIO VSOCK device (attach_vsock command)
  - Fast guest-host communication without network
  - CID validation (must be >= 3)
- ✅ Parallel Port device (attach_parallel command)
  - Legacy device support
  - LPT1/LPT2/LPT3 port selection

**Completed (Dec 15 - Session 4):**
- ✅ Serial Port device (attach_serial command with pty/tcp/unix types)
- ✅ Serial Port UI in Add Hardware dialog (port type, target port)
- ✅ Console device (attach_console command with virtio/serial types)
- ✅ Console device UI in Add Hardware dialog
- ✅ TPM hotplug (attach_tpm command for existing VMs)
- ✅ TPM UI in Add Hardware dialog (model: tpm-crb/tpm-tis, version: 1.2/2.0)
- ✅ USB Controller (attach_usb_controller command)
- ✅ USB Controller UI in Add Hardware dialog (USB 1.1/2.0/3.0)
- ✅ SCSI Controller (attach_scsi_controller command)
- ✅ SCSI Controller UI in Add Hardware dialog (virtio-scsi, lsilogic, etc.)

**Completed (Dec 15 - Session 3):**
- ✅ RNG Device backend (attach_rng command, VmConfig fields)
- ✅ RNG Device UI in Add Hardware dialog (backend selection)
- ✅ Watchdog Device backend (attach_watchdog command)
- ✅ Watchdog Device UI in Add Hardware dialog (model/action selection)
- ✅ Configurable Graphics type (VNC/Spice) in VmConfig
- ✅ Configurable Video model (qxl/virtio/vga) in VmConfig
- ✅ USB Host Device passthrough (full stack: Rust service, commands, frontend)
- ✅ Channel Device (QEMU Guest Agent, Spice Agent) - attach_channel command
- ✅ Filesystem Sharing (virtio-9p, virtiofs) - attach_filesystem command

**Completed (Dec 14 - Session 2):**
- ✅ Cloud-Init UI Integration (CloudInitConfig.tsx component)
- ✅ ISO Mount backend (mount_iso command)
- ✅ CD/DVD attachment in Add Hardware dialog (functional)
- ✅ Sound device attach/detach backend (attach_sound, detach_sound)
- ✅ Sound device in Add Hardware dialog (functional)
- ✅ Input device attach backend (attach_input)
- ✅ Input device in Add Hardware dialog (functional)
- ✅ PCI Passthrough UI in Add Hardware dialog
  - Lists available PCI devices from host
  - Shows IOMMU status and warnings
  - Filters devices safe for passthrough
  - Full attach_pci_device backend integration
- ✅ Frontend API wrappers for all PCI functions

**Completed Earlier (Dec 14 - Session 1):**
- ✅ VM Autostart backend (get_vm_autostart, set_vm_autostart)
- ✅ VM Autostart frontend toggle in Boot Options
- ✅ VM Rename backend (already existed, exposed to frontend API)
- ✅ VM Rename dialog in EnhancedVmRow dropdown menu
- ✅ Network Interface attachment (attach_interface, detach_interface)
- ✅ Add Hardware Dialog - Network Interface fully functional
- ✅ Add Hardware Dialog - Disk attachment fully functional
- ✅ Network Editor - Remove interface button functional
- ✅ Disk Editor - Detach disk button functional

**Add Hardware Dialog Now Supports (20 device types):**
- ✅ Storage (Disk) - Attach existing disk images
- ✅ CD/DVD - Mount ISO files
- ✅ Network Interface - Add NICs to networks
- ✅ Sound - Add sound cards (ich9, ac97, etc.)
- ✅ Input - Add tablet/mouse/keyboard devices
- ✅ PCI Host Device - GPU/NIC passthrough with IOMMU
- ✅ RNG - Random Number Generator (/dev/urandom, /dev/random)
- ✅ Watchdog - System watchdog (i6300esb, ib700)
- ✅ USB Host Device - USB passthrough by vendor:product ID
- ✅ Channel - QEMU Guest Agent and Spice Agent
- ✅ Shared Folder - Filesystem sharing (virtio-9p, virtiofs)
- ✅ Graphics - VNC and SPICE display servers
- ✅ Video - Virtual video cards (virtio, QXL, VGA, bochs, cirrus, ramfb)
- ✅ MDEV Host Device - Mediated devices (vGPU: Intel GVT-g, NVIDIA vGPU, AMD SR-IOV)
- ✅ Serial Port - Virtual serial ports (pty, tcp, unix socket)
- ✅ Console - VirtIO and serial console devices
- ✅ TPM - Trusted Platform Module (tpm-crb, tpm-tis, versions 1.2/2.0)
- ✅ USB Controller - USB 1.1, 2.0, 3.0 controllers
- ✅ SCSI Controller - VirtIO SCSI, LSI, MegaSAS controllers
- ✅ Smartcard - Smartcard reader (passthrough and emulated modes)

**Phase 2 Hardware Device Management: 100% COMPLETE** (All 21 device types implemented!)

---

## 🎉 Week 5 Complete - MAJOR MILESTONE ACHIEVED!

**Status**: ✅ **100% COMPLETE** (All Week 5 Goals Achieved in 3 Days!)

### ✅ Snapshot Management - 100% COMPLETE
- Full CRUD operations (Create, Read, Update, Delete)
- Backend: Rust services with libvirt integration
- Frontend: Complete UI with tree visualization
- Features:
  - Create snapshots with name and description
  - List all snapshots with metadata
  - Delete snapshots with confirmation
  - Revert VM to any snapshot
  - Tree visualization showing hierarchy
  - Real-time updates with TanStack Query
  - Parent-child relationship tracking

### ✅ Console Integration - 100% COMPLETE
- Full VNC viewer implementation (noVNC 1.6.0)
- **Automatic Reconnection**: Exponential backoff (5 attempts: 1s, 2s, 4s, 8s, 16s)
- **Display Modes**: Scale to Window, 1:1 Pixels, Stretch to Fill
- **Send Keys Menu**: Ctrl+Alt+Delete, Ctrl+Alt+Backspace, Ctrl+Alt+F1-F12
- **Screenshot Capture**: Save VM display as PNG
- **Fullscreen Support**: F11 toggle, Escape to exit
- **Status Indicators**: Connection state, duration, scale mode
- **Error Handling**: Graceful failures with helpful messages
- **Multi-Window**: Independent console per VM

### ✅ Documentation - COMPLETE
- [Console User Guide](docs/CONSOLE_USER_GUIDE.md) (1,000+ lines)
- [Testing Guide](WEEK5_DAY3_TESTING_GUIDE.md) (1,200+ lines, 38 test scenarios)
- [README.md](README.md) (Complete rewrite with project info)
- [Week 5 Final Report](WEEK5_FINAL_COMPLETION_REPORT.md) (2,000+ lines)

**Build Status**: ✅ Passing (0 TypeScript errors, 0 Rust warnings)
**Performance**: ✅ Excellent (2.94s build, 300KB gzipped, no memory leaks)

---

## 📊 Overall Project Status

**MVP Completion**: **75%** (4 out of 5 phases complete)

### Phase Breakdown

#### VM Management
- ✅ List all VMs with real-time status
- ✅ Create VM wizard (4-step process)
  - Basic info (name, CPU, memory, disk)
  - Configuration (OS, network, disk format)
  - Advanced (firmware, chipset, CPU topology, TPM)
  - Review & confirm
- ✅ Start/Stop/Pause/Resume/Force Stop VMs
- ✅ Delete VMs (with option to preserve disks)
- ✅ Clone VMs
- ✅ Import VMs from XML
- ✅ VM state monitoring (running, stopped, paused, suspended)
- ✅ Enhanced VM cards with quick actions
- ✅ Keyboard shortcuts for VM operations
- ✅ **Snapshot Management** (NEW!)
  - Create snapshots (disk/memory)
  - List snapshots with metadata
  - Delete snapshots
  - Revert to snapshots

#### Hardware Configuration
- ✅ CPU configuration (count, topology: sockets/cores/threads)
- ✅ Memory allocation
- ✅ Disk management (VirtIO, multiple formats: qcow2, raw)
- ✅ Network interfaces (bridge, NAT)
- ✅ Firmware selection (BIOS, UEFI, UEFI + Secure Boot)
- ✅ Chipset selection (PC i440fx, Q35 PCIe)
- ✅ Boot order configuration
- ✅ Boot menu toggle
- ✅ TPM 2.0 support

#### Storage
- ✅ Storage pool management
- ✅ Volume creation and management
- ✅ Disk format support (qcow2, raw)
- ✅ Automatic cleanup on VM creation failure

#### Network
- ✅ Network interface management
- ✅ Virtual network support
- ✅ Bridge network support

---

## 🆕 Recent Additions (December 12, 2025)

### Multi-Window Support ✅
- Implemented Tauri multi-window architecture
- VM details open in separate windows
- Console windows for each VM
- Window deduplication (prevents duplicate windows)
- Automatic cleanup when VMs are deleted
- Commands: `open_vm_details_window`, `open_console_window`, `close_vm_windows`

### Bug Fixes ✅
- Fixed Q35 chipset + UEFI compatibility (IDE → SATA for CDROM)
- Fixed CPU topology validation and synchronization
- Fixed storage volume cleanup on VM creation failure
- Fixed ISO file picker to open in Downloads folder by default
- Removed confusing CPU topology formula from UI
- Fixed build configuration issues (TypeScript errors, missing dependencies)

### Build System ✅
- Successfully built release packages:
  - Debian package (.deb)
  - RPM package (.rpm)
  - AppImage (portable)
- Fixed all compilation errors and warnings
- Clean dev and production builds

---

## ⚠️ In Progress / Partial Implementation

### VM Console (100% COMPLETE) ✅
- ✅ VNC console via noVNC 1.6.0
- ✅ SPICE console via spice-html5
- ✅ Serial console (text-mode for headless VMs)
- ✅ Console window management
- ✅ Graphical/Serial tab switching
- ✅ Auto graphics type detection
- ⚠️ In-app audio forwarding (Spice audio works via native clients)

### Guest Agent (80%)
- ✅ Agent communication protocol (JSON-RPC over virtio-serial)
- ✅ Linux guest agent implementation
- ✅ ISO packaging for easy deployment
- ✅ 10 agent methods implemented:
  - ping, get_os_info, get_hostname, get_uptime
  - get_ip_addresses, get_processes, exec_command
  - shutdown, reboot, get_disk_usage
- ⚠️ Windows guest agent (started, not complete)
- ❌ Automatic agent detection
- ❌ Agent status indicator in UI

### PCI Passthrough (90%)
- ✅ Backend service structure
- ✅ List PCI devices
- ✅ IOMMU group detection
- ✅ Device attach/detach (full implementation)
- ✅ PCI Passthrough UI in Add Hardware dialog
- ✅ VFIO driver management (automatic unbind/rebind) - DONE Session 14

---

## ❌ Missing Features (Target: Feature Parity)

### Critical Missing Features

#### 1. **VM Creation Options**
- ✅ Network install (HTTP, HTTPS, FTP) - DONE
- ✅ Import existing disk image - DONE
- ✅ Manual install (no media) - DONE
- ✅ "Customize before install" workflow - DONE

#### 2. **Hardware Configuration UI**
✅ **ALL COMPLETE** - 21 device types in Add Hardware dialog!

#### 3. **Advanced Features**
- ✅ Snapshots management - DONE
- ✅ Remote connections (SSH tunnels, TCP) - DONE (ConnectionManager)
- ✅ Migration (live/offline) - DONE (MigrationDialog)
- ✅ VM cloning - DONE
- ✅ Performance tuning options - DONE (CPU pinning, ballooning)
- ✅ Memory ballooning - DONE
- ✅ CPU pinning - DONE
- ✅ NUMA configuration - DONE

#### 4. **Storage Features**
- ✅ Storage pool types (NFS, iSCSI, LVM) - DONE (CreateStoragePoolWizard)
- ✅ Volume upload/download - DONE (StorageManager)
- ✅ Volume resize - DONE (StorageManager page)
- ✅ Snapshot management - DONE (SnapshotManager)

#### 5. **Network Features**
- ✅ Virtual network creation/editing - DONE (NetworkManager page)
- ✅ NAT/routing configuration - DONE (Forward mode in create dialog)
- ✅ Port forwarding setup - DONE (PortForwardingManager component)
- ✅ Network filtering - DONE (NwFilter backend/API)

---

## 🎨 UI/UX Transformation Plan

### Phase 1: Desktop-Native Layout ⚠️ IN PROGRESS
Transform from web-app to desktop application:

**Changes Needed**:
1. **Remove persistent sidebar** → Context-based navigation
2. **Add menu bar** (File, Edit, View, VM, Help)
3. **Add toolbar** with icon buttons for common actions
4. **Simplify main window** → Focus on VM list only
5. **Use separate windows** for VM details, console, etc.
6. **Add status bar** for connection status, notifications

**Reference**: See `DESKTOP_UI_REDESIGN.md` for detailed mockups

### Phase 2: Component Consolidation
- Move storage manager to menu → Tools → Storage
- Move network manager to menu → Tools → Networks
- Move templates to menu → File → Templates
- Remove dashboard (or make it optional view)
- Keep alerts, backups, schedules as optional views

---

## 🏗️ Architecture

### Frontend Structure
```
src/
├── components/
│   ├── layout/         # PageContainer, headers
│   ├── network/        # Network manager
│   ├── storage/        # Storage manager
│   ├── ui/            # Reusable UI components (shadcn)
│   └── vm/            # VM-related components
│       ├── CreateVmWizard.tsx
│       ├── EnhancedVmRow.tsx
│       ├── GuestInfo.tsx
│       ├── TemplateManager.tsx
│       └── VmCard.tsx
├── hooks/
│   ├── useKeyboardShortcuts.ts
│   └── useVmEvents.ts
├── lib/
│   ├── tauri.ts       # Tauri command wrappers
│   ├── types.ts       # TypeScript types
│   └── utils.ts       # Utilities
├── pages/
│   ├── Dashboard.tsx
│   ├── VmList.tsx
│   ├── VmDetails.tsx
│   └── ... (other pages)
└── styles/
    └── globals.css
```

### Backend Structure
```
src-tauri/src/
├── commands/          # Tauri command handlers
│   ├── guest_agent.rs
│   ├── network.rs
│   ├── pci.rs
│   ├── storage.rs
│   ├── vm.rs
│   └── window.rs      # NEW: Multi-window commands
├── models/            # Data structures
│   ├── cloud_init.rs
│   ├── network.rs
│   ├── pci.rs
│   ├── storage.rs
│   └── vm.rs
├── services/          # Business logic
│   ├── guest_agent_service.rs
│   ├── libvirt_service.rs
│   ├── network_service.rs
│   ├── pci_service.rs
│   ├── storage_service.rs
│   └── vm_service.rs
├── state/
│   └── app_state.rs   # Global application state
└── utils/
    └── error.rs       # Error handling
```

---

## 🚀 Build & Run

### Development
```bash
# Install dependencies
npm install

# Run in dev mode (with libvirt permissions)
sg libvirt -c "npm run tauri dev"
```

### Production Build
```bash
# Build release packages
npm run tauri build

# Outputs:
# - src-tauri/target/release/bundle/deb/KVM Manager_0.1.0_amd64.deb
# - src-tauri/target/release/bundle/rpm/KVM Manager-0.1.0-1.x86_64.rpm
# - src-tauri/target/release/bundle/appimage/KVM Manager_0.1.0_amd64.AppImage
```

### Prerequisites
- Node.js 18+
- Rust 1.70+
- libvirt-dev
- Tauri dependencies (webkit2gtk, etc.)
- User in `libvirt` and `kvm` groups

---

## 📋 Next Steps (Priority Order)

### High Priority
1. **Desktop UI Transformation** (see DESKTOP_UI_REDESIGN.md)
   - Remove sidebar, add menu bar
   - Simplify main window to VM list
   - Add toolbar with icon buttons
   - Status bar for connection info

2. **Hardware Configuration UI**
   - Add missing device panels (graphics, sound, USB, etc.)
   - Implement device add/remove functionality
   - GPU/PCI passthrough UI

3. **Console Improvements**
   - SPICE console support
   - Console settings (resolution, acceleration)
   - Copy/paste support

### Medium Priority
4. **Snapshot Management**
   - Create/restore/delete snapshots
   - Snapshot browser UI
   - Snapshot reverting

5. **Storage Enhancements**
   - Additional pool types (NFS, iSCSI)
   - Volume upload/download
   - Volume resize

6. **Network Management**
   - Virtual network creation UI
   - Port forwarding setup
   - Network isolation

---

## 📋 Next Steps (Priority Order)

### High Priority (Feature Completion)
1. ~~**VFIO Driver Management**~~ ✅ COMPLETE (Session 14)
   - ✅ Automatic unbind from host drivers
   - ✅ Rebind to vfio-pci for GPU passthrough
   - ✅ Driver state tracking in UI

2. ~~**Performance Monitoring Dashboard**~~ ✅ COMPLETE (Session 14)
   - ✅ Real-time CPU/memory graphs
   - ✅ Historical performance data (aggregated)
   - ✅ Alert thresholds (configurable warning/critical)

3. **Windows Guest Agent**
   - Complete Windows implementation
   - MSI installer package
   - Service installation and auto-start

### Medium Priority (Polish)
3. **In-App Audio Forwarding**
   - SPICE audio integration
   - PulseAudio/PipeWire backend

4. **Performance Monitoring Dashboard**
   - Real-time CPU/memory graphs
   - Historical performance data
   - Alert thresholds

### Low Priority (Nice-to-Have)
5. **UI Polish**
   - Desktop menu bar implementation
   - Keyboard shortcut improvements
   - Status bar for connection info

6. **Additional Storage Types**
   - ZFS pool support
   - LVM thin provisioning
   - Multipath storage

---

## 📚 Documentation

### Main Documents
- `PROJECT_PLAN.md` - Original project plan and roadmap
- `AGENTS.md` - Agent system architecture (if using AI agents)
- `README.md` - Getting started guide

### Recent Planning Documents
- `DESKTOP_UI_REDESIGN.md` - Desktop UI transformation plan
- `KVM_MANAGER_FEATURE_PARITY_GAMEPLAN.md` - Feature parity analysis
- `WEEK2_MULTI_WINDOW_SUMMARY.md` - Multi-window implementation

### Technical Documents
- `BACKEND_SETUP.md` - Rust backend setup
- `LIBVIRT_PERMISSIONS.md` - Permission configuration
- `UEFI_SETUP.md` - UEFI/Secure Boot setup
- `UI_ARCHITECTURE.md` - Frontend architecture

### Status Reports (Older)
- `DEVOPS_STATUS.md` - Build system status
- `DEPLOYMENT_REPORT.md` - Deployment configuration
- `PHASE4_STATUS_UPDATE.md` - Phase 4 progress

---

## 🐛 Known Issues

1. **Guest Agent**: Windows agent incomplete, no auto-detection in VM list
2. **PCI Passthrough**: No automatic VFIO driver rebind
3. **Audio**: SPICE audio requires native client (no in-app forwarding)

---

## 🤝 Contributing

This is a private project but follows standard practices:
- TypeScript for frontend
- Rust for backend
- Test before committing
- Follow existing code style
- Update documentation

---

## 📝 Notes

- Project uses Tauri 2.x (latest stable)
- Targets modern Linux distributions
- Requires libvirt 6.0+
- Designed for local VM management (remote planned for future)
- Focus on user experience over feature count

---

**For detailed implementation plans, see**:
- `DESKTOP_UI_REDESIGN.md` - UI transformation roadmap
- `KVM_MANAGER_FEATURE_PARITY_GAMEPLAN.md` - Feature checklist
