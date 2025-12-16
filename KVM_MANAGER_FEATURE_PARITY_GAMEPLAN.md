# KVM Manager Feature Parity Game Plan
**Date**: 2025-12-12
**Goal**: Achieve 1:1 feature parity with virt-manager while maintaining modern UX enhancements

---

## Executive Summary

Based on comprehensive analysis of virt-manager screenshots, codebase, and the current KVM Manager implementation:

**Current Status**:
- ✅ **Core VM Operations**: 95% complete
- ⚠️ **Hardware Configuration**: 60% complete (UI ready, backend needs work)
- ⚠️ **Advanced Features**: 40% complete
- ❌ **Critical Missing**: GPU/PCI passthrough UI, SPICE console, Remote connections

**Overall Feature Parity**: ~65% → **Target**: 95%+

---

## Analysis Summary

### What Virt-Manager Can Do (From Screenshots)

#### 1. **VM Creation Wizard** (onboard-*.png)
- ✅ Local install media (ISO/CDROM) - **WE HAVE**
- ❌ Network install (HTTP, HTTPS, FTP) - **MISSING**
- ❌ Import existing disk image - **MISSING**
- ❌ Manual install - **MISSING**
- ✅ OS detection from media - **WE HAVE** (partially)
- ✅ Memory and CPU configuration - **WE HAVE**
- ✅ Storage configuration (create/select) - **WE HAVE**
- ✅ Network selection - **WE HAVE**
- ❌ "Customize configuration before install" option - **MISSING**

#### 2. **Hardware Configuration** (config-*.png, hardware-*.png)
The left sidebar shows ALL hardware devices that can be configured:

**✅ Currently Configurable (UI exists)**:
- Overview (basic details, firmware, chipset)
- OS information
- CPUs (vCPU allocation, topology)
- Memory
- Boot Options (autostart, boot device order, direct kernel boot)
- VirtIO Disk
- SATA CDROM
- NIC (network interface)

**❌ Missing Hardware Configuration**:
- Tablet (input device)
- Display (Spice/VNC configuration)
- Sound (ich9, AC97, etc.)
- Console (serial, parallel)
- Channel devices (qemu-ga, spice)
- Video (Virtio, QXL, VGA models)
- Controllers (USB, PCIe, SCSI, SATA)
- USB Host Device passthrough
- USB Redirector
- PCI Host Device passthrough
- MDEV Host Device
- Watchdog
- Filesystem (virtio-9p)
- Smartcard
- TPM
- RNG (random number generator)
- Panic Notifier
- VirtIO VSOCK

**"Add Hardware" Dialog** (hardware-1.png) shows 20+ device types:
- Storage, Controller, Network, Input, Graphics, Sound
- Serial, Parallel, Console, Channel
- USB Host Device, PCI Host Device, MDEV Host Device
- Video, Watchdog, Filesystem, Smartcard
- USB Redirection, TPM, RNG, Panic Notifier, VirtIO VSOCK

#### 3. **Per-Device Configuration Screens**

**Boot Options** (config-5.png):
- ✅ Autostart on host boot - **PARTIALLY** (field exists)
- ✅ Enable boot menu - **WE HAVE**
- ✅ Boot device order (drag & drop) - **WE HAVE** (BootOrderEditor.tsx)
- ❌ Direct kernel boot - **MISSING**

**CPUs** (config-3.png):
- ✅ vCPU allocation - **WE HAVE**
- ✅ Copy host CPU configuration - **UI exists**
- ❌ Topology (sockets/cores/threads) expandable section - **UI EXISTS** (collapsible)
- ❌ CPU model selection - **MISSING**
- ❌ CPU features - **MISSING**
- ❌ CPU pinning - **MISSING**

**Network Interface** (config-8.png):
- ✅ Network source selection - **WE HAVE**
- ✅ Device model (virtio, e1000, etc.) - **PARTIALLY**
- ✅ MAC address display - **WE HAVE**
- ❌ MAC address editing - **MISSING**
- ❌ IP address detection - **MISSING** (needs guest agent)
- ❌ Link state control - **MISSING**

**Display/Graphics** (config-10.png):
- ❌ Spice server configuration - **MISSING**
- ✅ VNC server - **WE HAVE**
- ❌ Listen type (address/socket/none) - **MISSING**
- ❌ Password protection - **MISSING**
- ❌ OpenGL acceleration - **MISSING**

**Video** (config-15.png):
- ❌ Video model (Virtio, QXL, VGA, etc.) - **MISSING**
- ❌ 3D acceleration toggle - **MISSING**

**Storage** (hardware-1.png):
- ✅ Create disk image - **WE HAVE**
- ✅ Select existing storage - **WE HAVE**
- ✅ Device type (Disk/CDROM/Floppy) - **PARTIALLY**
- ✅ Bus type (VirtIO, SATA, SCSI, IDE, USB) - **PARTIALLY**
- ❌ Advanced options (cache, I/O mode, discard) - **MISSING**

**Graphics/Display** (hardware-5.png):
- ❌ Type selection (Spice/VNC) - **MISSING**
- ❌ Listen type configuration - **MISSING**
- ❌ Address binding - **MISSING**
- ❌ Port configuration - **PARTIALLY** (VNC only)
- ❌ Password - **MISSING**
- ❌ OpenGL - **MISSING**

**Channel Device** (hardware-10.png):
- ❌ Channel name (com.redhat.spice.0, etc.) - **MISSING**
- ❌ Device type (Spice agent, QEMU GA) - **MISSING**

**Watchdog** (hardware-15.png):
- ❌ Model (i6300esb, etc.) - **MISSING**
- ❌ Action (Forcefully reset, Shutdown, Poweroff, Pause, None) - **MISSING**

**RNG Device** (config-20.png):
- ❌ Type (Random) - **MISSING**
- ❌ Host device (/dev/urandom, /dev/random) - **MISSING**

---

## Game Plan: 5-Phase Implementation Roadmap

---

## PHASE 1: Complete Existing Features (Weeks 1-2)
**Goal**: Wire up all UI fields that already exist but aren't connected to backend

### 1.1 Backend Implementation for Existing UI Fields

#### **UEFI + TPM Support** 🔴 CRITICAL
**Status**: UI fields exist (firmware, tpmEnabled), backend needs implementation

**Tasks**:
- [ ] Update `vm_service.rs::create_vm()` to generate UEFI firmware XML
  ```xml
  <os firmware='efi'>
    <loader readonly='yes' type='pflash'>/usr/share/OVMF/OVMF_CODE.fd</loader>
    <nvram>/var/lib/libvirt/qemu/nvram/{vm_name}_VARS.fd</nvram>
  </os>
  ```
- [ ] Add TPM device XML generation when `tpm_enabled = true`
  ```xml
  <tpm model='tpm-crb'>
    <backend type='emulator' version='2.0'/>
  </tpm>
  ```
- [ ] Detect OVMF firmware paths on host system
- [ ] Handle UEFI Secure Boot (firmware='uefi-secure')
- [ ] Add validation: TPM requires UEFI (not BIOS)
- [ ] Update `get_vm()` to read firmware type from existing VMs

**Files to modify**:
- `src-tauri/src/services/vm_service.rs`
- `src-tauri/src/services/libvirt.rs` (XML generation)

**Testing**:
- Create Windows 11 VM (requires UEFI + TPM)
- Verify UEFI boot menu shows
- Verify TPM device in guest OS

---

#### **CPU Topology** 🔴 CRITICAL
**Status**: UI fully implemented (sockets/cores/threads in CreateVmWizard), backend needs XML

**Tasks**:
- [ ] Generate CPU topology XML in `create_vm()`
  ```xml
  <vcpu placement='static'>4</vcpu>
  <cpu mode='host-passthrough'>
    <topology sockets='1' cores='2' threads='2'/>
  </cpu>
  ```
- [ ] Read topology from existing VMs in `get_vm()`
- [ ] Validate: `sockets * cores * threads = cpuCount`
- [ ] Add CPU model selection dropdown (use libvirt capabilities)
- [ ] Implement "Copy host CPU configuration" checkbox

**Files to modify**:
- `src-tauri/src/services/vm_service.rs`
- `src/components/vm/CreateVmWizard.tsx` (add CPU model dropdown)

---

#### **Chipset (Q35 vs PC)** 🟠 HIGH
**Status**: UI field exists, backend needs implementation

**Tasks**:
- [ ] Generate machine type in VM XML: `<os><type arch='x86_64' machine='pc-q35-8.1'>hvm</type></os>`
- [ ] Map `chipset='q35'` → `machine='pc-q35-*'`
- [ ] Map `chipset='pc'` → `machine='pc-i440fx-*'`
- [ ] Auto-select Q35 for UEFI VMs (recommendation)

---

#### **Boot Order** ✅ READY
**Status**: UI fully implemented (BootOrderEditor.tsx), backend needs XML

**Tasks**:
- [ ] Generate boot order XML from `boot_order` array
  ```xml
  <os>
    <boot dev='cdrom'/>
    <boot dev='hd'/>
    <boot dev='network'/>
    <bootmenu enable='yes'/>
  </os>
  ```
- [ ] Read boot order from existing VMs

---

#### **Cloud-Init Integration** 🔴 CRITICAL
**Status**: UI missing, backend service exists (`cloud_init_service.rs`)

**Tasks**:
- [ ] Add Cloud-Init tab to CreateVmWizard (Step 3.5)
  - Hostname
  - Username/password
  - SSH public key
  - Network config (static IP/DHCP)
  - Package installation
  - Run commands
- [ ] Generate cloud-init ISO using `cloud_init_service.rs`
- [ ] Attach cloud-init ISO as secondary CDROM
- [ ] Add cloud-init templates for common distros

**UI Components to create**:
- `src/components/vm/CloudInitConfig.tsx`

---

### 1.2 Quick Wins (Low effort, high impact)

#### **VM Rename** ✅ EASY WIN
- [ ] Add rename action to VM dropdown menu
- [ ] Backend: `virDomainRename()` API call
- [ ] Update XML domain name

#### **Autostart on Boot** ✅ EASY WIN
**Status**: Field exists in Boot Options, needs backend

- [ ] Checkbox in Boot Options tab
- [ ] Backend: `virDomainSetAutostart()` API
- [ ] Read autostart status in `get_vm()`

#### **MAC Address Configuration** ✅ EASY WIN
- [ ] Add MAC address input field in network config
- [ ] Auto-generate MAC if empty
- [ ] Validate MAC address format

---

## PHASE 2: Hardware Device Management (Weeks 3-5)
**Goal**: Implement "Add Hardware" dialog with all device types from virt-manager

### 2.1 Hardware Management Infrastructure

#### **VM Details Page Redesign** 🔴 CRITICAL
**Current**: Basic details page
**Target**: Virt-manager-style hardware list (onboard-7.png)

**Tasks**:
- [ ] Create left sidebar with hardware device tree
  - Overview
  - CPUs
  - Memory
  - Boot Options
  - Disks (expandable list)
  - Network Interfaces (expandable list)
  - Graphics
  - Display
  - Sound
  - Input Devices
  - Serial/Parallel
  - Channels
  - USB Controllers
  - Other Controllers
  - PCI Devices
  - TPM
  - RNG
  - Watchdog
- [ ] Right panel shows details for selected device
- [ ] "Add Hardware" button at bottom
- [ ] "Remove" button for removable devices

**UI Components to create**:
- `src/components/vm/HardwareList.tsx`
- `src/components/vm/HardwareDetails.tsx`
- `src/components/vm/AddHardwareDialog.tsx`

---

#### **Add Hardware Dialog** 🔴 CRITICAL
**Reference**: hardware-1.png, hardware-5.png, hardware-10.png, hardware-15.png

**Tasks**:
- [ ] Create modal dialog with left category list
- [ ] Implement device type panels:
  - [x] Storage (already have DiskManager)
  - [ ] Controller
  - [x] Network (already have network config)
  - [ ] Input (mouse, keyboard, tablet)
  - [ ] Graphics (Spice, VNC)
  - [ ] Sound (ich9, AC97, SB16, USB audio)
  - [ ] Serial
  - [ ] Parallel
  - [ ] Console
  - [ ] Channel (qemu-ga, spice)
  - [ ] USB Host Device
  - [ ] PCI Host Device ← **CRITICAL**
  - [ ] MDEV Host Device
  - [ ] Video (Virtio, QXL, VGA, Bochs, Ramfb)
  - [ ] Watchdog (i6300esb, ib700, diag288)
  - [ ] Filesystem (virtio-9p, virtiofs)
  - [ ] Smartcard
  - [ ] USB Redirection
  - [ ] TPM (already in wizard)
  - [ ] RNG
  - [ ] Panic Notifier
  - [ ] VirtIO VSOCK

**UI Components to create**:
- `src/components/vm/devices/StorageDevice.tsx` ✅ (exists)
- `src/components/vm/devices/NetworkDevice.tsx`
- `src/components/vm/devices/GraphicsDevice.tsx`
- `src/components/vm/devices/SoundDevice.tsx`
- `src/components/vm/devices/InputDevice.tsx`
- `src/components/vm/devices/PciDevice.tsx` ← **CRITICAL**
- `src/components/vm/devices/VideoDevice.tsx`
- `src/components/vm/devices/WatchdogDevice.tsx`
- `src/components/vm/devices/RngDevice.tsx`
- ... (15+ more)

---

### 2.2 GPU/PCI Passthrough UI 🔴 CRITICAL
**Priority**: Highest - Gaming, ML workloads depend on this

**Tasks**:

#### **Backend: PCI Device Discovery**
- [ ] Scan host for PCI devices (`lspci`)
- [ ] Parse `/sys/bus/pci/devices/*` for details
- [ ] Detect IOMMU groups
- [ ] Filter GPU devices
- [ ] Check if device bound to vfio-pci driver

**New files**:
- `src-tauri/src/services/pci_service.rs` ✅ (already exists!)
- `src-tauri/src/models/pci.rs` ✅ (already exists!)
- `src-tauri/src/commands/pci.rs` ✅ (already exists!)

**Review existing implementation and enhance**:
- [ ] Verify IOMMU detection works
- [ ] Add GPU-specific detection (NVIDIA, AMD, Intel)
- [ ] Add warnings for devices in same IOMMU group

#### **Frontend: PCI Passthrough UI**
- [ ] PCI device selector in "Add Hardware" dialog
  - Device list with vendor, model, ID, IOMMU group
  - GPU filtering option
  - IOMMU group warnings
  - Driver binding status
- [ ] XML generation for hostdev passthrough
  ```xml
  <hostdev mode='subsystem' type='pci' managed='yes'>
    <source>
      <address domain='0x0000' bus='0x01' slot='0x00' function='0x0'/>
    </source>
  </hostdev>
  ```
- [ ] Validation:
  - IOMMU enabled on host
  - Device not in use
  - vfio-pci driver bound (or auto-bind)

**UI Components**:
- `src/components/vm/PciPassthroughManager.tsx`
- `src/components/vm/devices/PciDevice.tsx` ✅ (may exist)

---

### 2.3 Essential Device Types

#### **Graphics/Display Configuration** 🟠 HIGH
- [ ] Backend: Spice server configuration
  - Port allocation
  - TLS support
  - Password authentication
  - Listen address
- [ ] Backend: VNC server configuration (enhance existing)
  - Port, password, listen address
- [ ] Frontend: Graphics device editor
  - Type: Spice / VNC / None
  - Listen type: Address / Socket / None
  - Port (auto or manual)
  - Password toggle
  - OpenGL checkbox (Spice)

**XML Examples**:
```xml
<!-- Spice -->
<graphics type='spice' autoport='yes'>
  <listen type='address'/>
  <gl enable='yes'/>
</graphics>

<!-- VNC -->
<graphics type='vnc' port='5900' autoport='yes' listen='0.0.0.0'>
  <listen type='address' address='0.0.0.0'/>
</graphics>
```

#### **Video Device Configuration** 🟠 HIGH
- [ ] Video model selection:
  - Virtio (best performance)
  - QXL (Spice optimized)
  - VGA (compatibility)
  - Bochs (simple)
  - Ramfb (UEFI only)
- [ ] 3D acceleration toggle
- [ ] VRAM size
- [ ] Multi-head display count

**XML**:
```xml
<video>
  <model type='virtio' heads='1' primary='yes'>
    <acceleration accel3d='yes'/>
  </model>
</video>
```

#### **Sound Device** 🟡 MEDIUM
- [ ] Sound card models:
  - ich9 (modern, recommended)
  - AC97 (older)
  - SB16 (legacy)
  - USB audio
- [ ] Audio backend selection

**XML**:
```xml
<sound model='ich9'>
  <audio id='1'/>
</sound>
<audio id='1' type='spice'/>
```

#### **Input Devices** 🟡 MEDIUM
- [ ] Tablet (absolute pointer - default for Spice)
- [ ] Mouse (relative pointer)
- [ ] Keyboard
- [ ] Evdev (passthrough host input)

**XML**:
```xml
<input type='tablet' bus='usb'/>
<input type='keyboard' bus='usb'/>
```

#### **Channel Devices** 🟡 MEDIUM
- [ ] QEMU Guest Agent channel (`org.qemu.guest_agent.0`)
- [ ] Spice agent channel (`com.redhat.spice.0`)
- [ ] Custom virtio serial ports

**XML**:
```xml
<channel type='unix'>
  <target type='virtio' name='org.qemu.guest_agent.0'/>
</channel>
```

#### **Watchdog Device** 🟡 MEDIUM
- [ ] Model selection (i6300esb, ib700, diag288)
- [ ] Action on timeout:
  - Forcefully reset
  - Graceful shutdown
  - Poweroff
  - Pause
  - None

**XML**:
```xml
<watchdog model='i6300esb' action='reset'/>
```

#### **RNG (Random Number Generator)** 🟡 MEDIUM
- [ ] Backend: /dev/urandom, /dev/random
- [ ] Rate limiting

**XML**:
```xml
<rng model='virtio'>
  <backend model='random'>/dev/urandom</backend>
</rng>
```

#### **USB Controller & Devices** 🟠 HIGH
- [ ] USB controller types (USB 1.1, 2.0, 3.0)
- [ ] USB host device passthrough (list host USB devices)
- [ ] USB redirection (for remote Spice)

---

## PHASE 3: Advanced Networking & Storage (Weeks 6-7)

### 3.1 Advanced Network Features

#### **IPv6 Support** 🟠 HIGH
- [ ] IPv6 address configuration in virtual networks
- [ ] DHCPv6 support
- [ ] IPv6 NAT
- [ ] Display IPv6 addresses in VM list

#### **SR-IOV Virtual Functions** 🟡 MEDIUM
- [ ] Detect SR-IOV capable NICs
- [ ] Create VF pools
- [ ] Assign VFs to VMs
- [ ] VLAN tagging

#### **Network QoS** 🟡 MEDIUM
- [ ] Bandwidth limits (inbound/outbound)
- [ ] Burst limits
- [ ] Network priority

**XML**:
```xml
<interface type='network'>
  <bandwidth>
    <inbound average='1000' peak='5000' burst='5120'/>
    <outbound average='1000'/>
  </bandwidth>
</interface>
```

#### **Advanced Network Types** 🟡 MEDIUM
- [ ] Open vSwitch integration
- [ ] VLAN configuration
- [ ] Macvtap (direct host NIC)
- [ ] passt backend (rootless networking)

---

### 3.2 Advanced Storage Features

#### **Disk I/O Tuning** 🟡 MEDIUM
- [ ] Cache mode (none, writethrough, writeback, directsync)
- [ ] I/O mode (native, threads, io_uring)
- [ ] Discard mode (unmap, ignore)
- [ ] Detect discard (on/off)
- [ ] I/O throttling (IOPS, bandwidth limits)

**XML**:
```xml
<disk type='file' device='disk'>
  <driver name='qemu' type='qcow2' cache='none' io='native' discard='unmap'/>
  <iotune>
    <read_iops_sec>1000</read_iops_sec>
    <write_iops_sec>1000</write_iops_sec>
  </iotune>
</disk>
```

#### **Disk Encryption (LUKS)** 🟠 HIGH
- [ ] Create encrypted qcow2 volumes
- [ ] Manage encryption secrets in libvirt
- [ ] Password prompt on VM start

**XML**:
```xml
<disk type='file'>
  <source file='/var/lib/libvirt/images/encrypted.qcow2'/>
  <encryption format='luks'>
    <secret type='passphrase' uuid='...'/>
  </encryption>
</disk>
```

#### **NVMe Disks** 🟠 HIGH
- [ ] NVMe disk device type
- [ ] NVMe namespace configuration

**XML**:
```xml
<disk type='file' device='disk'>
  <driver name='qemu' type='qcow2'/>
  <source file='/path/to/disk.qcow2'/>
  <target dev='vda' bus='nvme'/>
</disk>
```

#### **Network Storage** 🟠 HIGH
- [ ] iSCSI storage pools
- [ ] NFS storage pools
- [ ] Ceph/RBD storage pools
- [ ] Mount/unmount remote storage

---

## PHASE 4: Console & Display Enhancements (Week 8)

### 4.1 SPICE Protocol Support 🔴 CRITICAL

**Tasks**:
- [ ] Install/bundle Spice client library (spice-html5 or native)
- [ ] Spice console component (replace/augment VNC)
- [ ] Spice features:
  - Clipboard sharing (bidirectional)
  - USB redirection over network
  - Audio redirection
  - Multi-monitor support
  - Better performance than VNC
  - OpenGL/3D acceleration (SpiceGL)

**Frontend**:
- [ ] `src/components/vm/SpiceConsole.tsx`
- [ ] Detect if VM has Spice graphics device
- [ ] Switch between VNC/Spice consoles

**Backend**:
- [ ] Spice connection info API
- [ ] Spice TLS certificate handling

---

### 4.2 Console Features

#### **Multi-Monitor Support** 🟡 MEDIUM
- [ ] Configure number of heads/monitors in video device
- [ ] Multi-window console view

#### **Keyboard Shortcuts** ✅ EASY WIN
- [ ] Ctrl+Alt+F (fullscreen toggle)
- [ ] Ctrl+Alt+G (release grab)
- [ ] Send Ctrl+Alt+Del
- [ ] Send Ctrl+Alt+Backspace

#### **Console Scaling & Quality** 🟢 LOW
- [ ] Scaling modes (auto-fit, 1:1, custom)
- [ ] Quality/compression settings (Spice)

---

## PHASE 5: Remote Management & Migration (Weeks 9-10)

### 5.1 Remote Libvirt Connections 🔴 CRITICAL

**Tasks**:
- [ ] Connection manager UI
  - Add connection dialog (SSH, TLS, local)
  - Connection list
  - Switch active connection
  - Save credentials securely
- [ ] Backend: SSH tunneling
  - `qemu+ssh://user@host/system`
  - SSH key authentication
  - Password authentication
- [ ] Backend: TLS connections
  - `qemu+tls://host/system`
  - Certificate management
- [ ] Multi-host view (aggregate multiple connections)

**UI Components**:
- `src/components/connections/ConnectionManager.tsx`
- `src/components/connections/AddConnection.tsx`
- Update `Layout.tsx` to show current connection

**Backend**:
- `src-tauri/src/services/connection_service.rs`
- Store connections in app config

---

### 5.2 Live Migration 🔴 CRITICAL

**Tasks**:
- [ ] Detect migration capabilities
- [ ] Migration wizard:
  - Select destination host
  - Shared storage check
  - Migration options (live, persistent, tunneled)
  - Bandwidth limit
- [ ] Progress monitoring
- [ ] Error handling

**Backend**:
- [ ] `virDomainMigrate3()` API wrapper
- [ ] Migration progress events

**UI**:
- `src/components/vm/MigrateVmDialog.tsx`

---

## PHASE 6: Polish & Production Readiness (Weeks 11-12)

### 6.1 UI/UX Polish

#### **Dark Mode** 🟠 HIGH (Already planned?)
- [ ] Verify dark mode CSS
- [ ] Test all components in dark mode
- [ ] Add theme toggle in settings

#### **Keyboard Shortcuts** ✅ EASY WIN
- [ ] Global shortcuts (already has KeyboardShortcutsDialog.tsx)
- [ ] VM list shortcuts (start, stop, delete)
- [ ] Console shortcuts

#### **Accessibility** 🟡 MEDIUM
- [ ] ARIA labels
- [ ] Keyboard navigation
- [ ] Screen reader support
- [ ] High contrast mode

---

### 6.2 Documentation & Help

- [ ] In-app help system
- [ ] Tooltips for all fields
- [ ] Getting started guide
- [ ] Hardware passthrough guide
- [ ] Migration guide
- [ ] Troubleshooting guide

---

### 6.3 Performance & Reliability

#### **Optimization Suggestions** ✅ ALREADY HAVE
- Enhance existing OptimizationSuggestions.tsx
- [ ] Suggest Q35 for modern VMs
- [ ] Suggest VirtIO for all devices
- [ ] Suggest hugepages for large VMs
- [ ] Suggest CPU pinning for performance VMs

#### **Error Handling**
- [ ] Better error messages
- [ ] Suggest fixes for common errors
- [ ] Libvirt error translation

---

## Implementation Priority Matrix

### Must Have (Phase 1-2) - Weeks 1-5
| Feature | Impact | Effort | Priority Score |
|---------|--------|--------|----------------|
| UEFI + TPM Support | 🔴 Critical | 🟢 Low | **10** |
| CPU Topology | 🔴 Critical | 🟢 Low | **10** |
| Cloud-Init | 🔴 Critical | 🟡 Medium | **9** |
| GPU/PCI Passthrough | 🔴 Critical | 🟡 Medium | **9** |
| Hardware Device List UI | 🔴 Critical | 🟠 High | **8** |
| Spice Console | 🔴 Critical | 🟠 High | **8** |
| Boot Order | 🟢 Low | 🟢 Low | **7** |
| VM Rename | 🟢 Low | 🟢 Low | **6** |

### Should Have (Phase 3-4) - Weeks 6-8
| Feature | Impact | Effort | Priority Score |
|---------|--------|--------|----------------|
| Video Device Config | 🟠 High | 🟢 Low | **8** |
| Sound Devices | 🟡 Medium | 🟢 Low | **7** |
| IPv6 Networking | 🟠 High | 🟡 Medium | **7** |
| Disk Encryption | 🟠 High | 🟡 Medium | **7** |
| NVMe Disks | 🟠 High | 🟢 Low | **7** |
| Network Storage | 🟠 High | 🟡 Medium | **6** |
| USB Passthrough | 🟠 High | 🟡 Medium | **6** |

### Nice to Have (Phase 5-6) - Weeks 9-12
| Feature | Impact | Effort | Priority Score |
|---------|--------|--------|----------------|
| Remote Connections | 🔴 Critical | 🔴 Very High | **7** |
| Live Migration | 🔴 Critical | 🔴 Very High | **6** |
| SR-IOV Networking | 🟡 Medium | 🟠 High | **5** |
| Disk I/O Tuning | 🟡 Medium | 🟡 Medium | **5** |
| Watchdog | 🟡 Medium | 🟢 Low | **5** |
| RNG Device | 🟡 Medium | 🟢 Low | **5** |
| Multi-Monitor | 🟡 Medium | 🟡 Medium | **4** |

---

## Success Metrics

### Functional Parity
- ✅ **95%+ feature coverage** of virt-manager core features
- ✅ **Can create any VM type** (Windows 11, Ubuntu, RHEL, etc.)
- ✅ **Can configure any hardware** (GPU, sound, USB, serial, etc.)
- ✅ **Can manage remote hosts** (SSH/TLS connections)

### User Experience
- ✅ **Faster VM creation** than virt-manager (wizard < 60 seconds)
- ✅ **Better performance monitoring** (already have this)
- ✅ **Modern UI** (already have this)
- ✅ **Batch operations** (already have this)

### Technical Excellence
- ✅ **No crashes** during normal operations
- ✅ **Fast response times** (< 100ms for UI interactions)
- ✅ **Comprehensive error handling**
- ✅ **Full test coverage** for critical paths

---

## Risk Assessment

### High Risk Items
1. **SPICE Console Integration** - Complex library, may need native components
2. **Live Migration** - Shared storage requirements, network complexity
3. **Remote Connections** - Security, SSH key management, multi-host state
4. **GPU Passthrough** - IOMMU complexity, driver binding, hardware conflicts

### Mitigation Strategies
1. **SPICE**: Start with spice-html5, fall back to VNC if issues
2. **Migration**: Start with local migration, add remote later
3. **Remote**: Use libvirt's built-in SSH, don't reinvent authentication
4. **GPU**: Provide clear error messages, detect common issues automatically

---

## Next Steps (Immediate Action Plan)

### Week 1: Foundation
- [ ] **Day 1-2**: Implement UEFI + TPM backend (vm_service.rs)
- [ ] **Day 3-4**: Implement CPU topology XML generation
- [ ] **Day 5**: Test Windows 11 VM creation (UEFI+TPM requirement)

### Week 2: Hardware Framework
- [ ] **Day 1-3**: Build HardwareList.tsx component (device tree sidebar)
- [ ] **Day 4-5**: Build AddHardwareDialog.tsx framework

### Week 3-4: GPU Passthrough
- [ ] **Week 3**: Review existing PCI service, enhance detection
- [ ] **Week 4**: Build PCI passthrough UI, test with real GPU

### Week 5: Testing & Stabilization
- [ ] Create test VMs with all configurations
- [ ] Document any missing features
- [ ] Fix bugs from Weeks 1-4

---

## Conclusion

**Current State**: KVM Manager has excellent UX and modern features (templates, schedules, alerts, batch operations) that virt-manager lacks.

**Gap**: Missing critical hardware configuration UIs and advanced device management that power users need.

**Path to Parity**: The UI framework is already modern and well-structured. Most work is:
1. **Backend**: XML generation for devices (straightforward libvirt work)
2. **Frontend**: Device configuration forms (repetitive but simple React components)
3. **Integration**: Wire up existing UI fields to backend

**Timeline**: 12 weeks to 95%+ feature parity while maintaining UX advantages.

**Competitive Advantage After Completion**:
- ✅ All virt-manager features
- ✅ Better UX (modern React UI)
- ✅ Better monitoring (performance graphs, alerts)
- ✅ Better automation (templates, schedules, batch ops)
- ✅ Better organization (tags, groups)
- ✅ Custom guest agent (already implemented)

**Outcome**: Best-in-class KVM management tool for Linux desktops. 🚀
