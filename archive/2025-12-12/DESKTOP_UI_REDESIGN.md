# KVM Manager Desktop UI/UX Redesign
**Date**: 2025-12-12
**Goal**: Transform KVM Manager from web-app style to native desktop application feel

---

## Executive Summary

**Current State**: KVM Manager has a modern, feature-rich UI but feels like a web application with persistent sidebar navigation and single-page routing.

**Target State**: Desktop-native application that flows like virt-manager - focused, clean, uses multiple windows where appropriate, with desktop UI patterns.

**Key Changes**:
1. **Simplified main window** - Focus on VM management
2. **Context-based navigation** - Replace persistent sidebar with contextual toolbars
3. **Window management** - Use Tauri multi-window for VM details, console
4. **Desktop patterns** - Menu bars, toolbars, status bars, keyboard shortcuts
5. **Native feel** - Platform-appropriate UI elements and behaviors

---

## Analysis: Virt-Manager vs Current KVM Manager

### Virt-Manager UI Pattern (Desktop-Native)

#### **Main Window** (homepage.png)
```
┌─────────────────────────────────────────────────┐
│ File  Edit  View  Help                      ─ □ × │ ← Menu bar
├─────────────────────────────────────────────────┤
│ [📁] [💻] Open  [▶] [⏸] [⏻] ▼                  │ ← Toolbar (icon buttons)
├──────────────┬──────────────────────────────────┤
│ Name         │                      CPU usage   │ ← Table header
├──────────────┴──────────────────────────────────┤
│ QEMU/KVM (connection)                           │
│                                                  │
│                                                  │
│ (VM list would appear here)                     │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Key Features**:
- ✅ Classic menu bar (File, Edit, View, Help)
- ✅ Icon toolbar for common actions
- ✅ Simple, clean table/list of VMs
- ✅ No persistent sidebar - very focused
- ✅ Connection shown as tree node (QEMU/KVM)

#### **VM Details Window** (onboard-7.png) - SEPARATE WINDOW
```
┌──────────────────── ubuntu24.04 on QEMU/KVM ─────────────────┐
│ Begin Installation    Cancel Installation                 ─ □ × │
├──────────────┬────────────────────────────────────────────────┤
│              │ Details    XML                                 │ ← Tabs
│ Overview     │ ┌──────────────────────────────────────────┐  │
│ OS info      │ │ Basic Details                            │  │
│ CPUs         │ │ Name: ubuntu24.04                        │  │
│ Memory       │ │ UUID: 3ef9b3b5-...                       │  │
│ Boot Options │ │ Status: Shutoff                          │  │
│ VirtIO Disk  │ │ Title: [              ]                  │  │
│ SATA CDROM   │ │ Description: [                          ]│  │
│ NIC          │ │                                           │  │
│ Tablet       │ │ Hypervisor Details                       │  │
│ Display      │ │ Hypervisor: KVM                          │  │
│ Sound        │ │ Architecture: x86_64                     │  │
│ Console      │ │ Emulator: /usr/bin/qemu-system-x86_64   │  │
│ Channel (ga) │ │ Chipset: Q35 ▼                           │  │
│ Channel (sp) │ │ Firmware: BIOS ▼                         │  │
│ Video        │ │                                           │  │
│ Controllers  │ └──────────────────────────────────────────┘  │
│ USB Redirect │                                                │
│ RNG          │                                                │
│              │                                                │
│ Add Hardware │                                                │
└──────────────┴────────────────────────────────────────────────┘
```

**Key Features**:
- ✅ **Separate window** (not a page in the same window)
- ✅ Window title shows VM name and connection
- ✅ Action buttons at top (context-specific)
- ✅ **Left sidebar**: Hardware device tree
- ✅ **Right panel**: Configuration details for selected device
- ✅ Tabs for Details/XML view
- ✅ "Add Hardware" button at bottom of device list

---

### Current KVM Manager Pattern (Web-Style)

#### **Main Window**
```
┌─────────────────────────────────────────────────────────────┐
│ ┌─────────┐                                              ─ □ × │
│ │ [Server]│ KVM Manager                                     │
│ │         │                                                  │
│ │ [≡] Dashboard    ← Always visible navigation              │
│ │ [☐] VMs                                                    │
│ │ [▭] Storage                                                │
│ │ [⚡] Networks        ┌────────────────────────────┐        │
│ │ [💡] Insights       │                            │        │
│ │ [📄] Templates      │  VM List Page              │        │
│ │ [🕐] Schedules      │  (or Storage, Networks,    │        │
│ │ [🔔] Alerts         │   Templates, etc.)         │        │
│ │ [💾] Backups        │                            │        │
│ │ [⚙] Settings       │                            │        │
│ │                     │                            │        │
│ │ [<] Collapse        │                            │        │
│ └─────────┘           └────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

**Characteristics**:
- ❌ Persistent left sidebar (like a web app)
- ❌ Many top-level navigation items (10+)
- ❌ Single-page application feel
- ❌ No menu bar
- ❌ No traditional desktop patterns

---

## Desktop UI Best Practices (Tauri Apps)

### 1. **Window Management**
- **Main window**: Primary task/content (VM list)
- **Secondary windows**: Detail views, editors, consoles
- **Dialogs**: Temporary, focused tasks (wizards, confirmations)
- **Use Tauri's multi-window API**: `WebviewWindow::new()`

### 2. **Navigation Patterns**
- **Menu bar**: File operations, app commands, help
- **Toolbar**: Frequently used actions (icon buttons)
- **Context menus**: Right-click actions
- **Keyboard shortcuts**: Power user efficiency

### 3. **Layout Patterns**
- **Master-detail**: List on left, details on right (or separate window)
- **Toolbars**: Horizontal action bars
- **Status bars**: Bottom of window for status/info
- **Sidebars**: Contextual, collapsible (not navigation)

### 4. **Visual Design**
- **Native look**: Follow platform conventions (titlebar, window controls)
- **Subtle chrome**: Less is more - focus on content
- **Consistent spacing**: Desktop apps have denser layouts than web
- **Desktop colors**: More muted, professional palette

---

## Proposed Redesign: KVM Manager Desktop UI

### Architecture: 3-Window System

#### **Window 1: Main Window (VM Manager)**
Primary window - always open, focus on VM management

```
┌──────────────────────── KVM Manager ──────────────────────────┐
│ File  Edit  View  Actions  Tools  Help                    ─ □ × │ ← Menu Bar
├───────────────────────────────────────────────────────────────┤
│ [➕ New] [▶ Start] [⏹ Stop] [⏸ Pause] │ [🔍 Search...] [⚙]   │ ← Toolbar
├───────────────────────────────────────────────────────────────┤
│ Connection: [QEMU/KVM (localhost) ▼] [Connect]  [+]          │ ← Connection bar
├───────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ ☑ Name          State    CPU   Memory   Disk   Network  │ │ ← VM Table
│  ├──────────────────────────────────────────────────────────┤ │
│  │ ☐ ubuntu-server Running  25%    4.0GB   20GB  192.168.1 │ │
│  │ ☐ windows11     Stopped   -     8.0GB   50GB     -      │ │
│  │ ☐ fedora-dev    Running  60%    2.0GB   30GB  10.0.0.5  │ │
│  │ ☐ test-vm       Paused   10%    1.0GB   10GB  192.168.1 │ │
│  │                                                           │ │
│  │ Right-click for menu: Open, Console, Edit, Clone, Delete │ │
│  │ Double-click to open VM details                          │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
├───────────────────────────────────────────────────────────────┤
│ 4 VMs │ 2 Running, 1 Stopped, 1 Paused │ CPU: 35% │ Mem: 7/16GB│ ← Status Bar
└───────────────────────────────────────────────────────────────┘
```

**Features**:
- ✅ Traditional menu bar
- ✅ Icon toolbar for common actions
- ✅ Connection selector (supports remote connections)
- ✅ Clean table view (similar to virt-manager)
- ✅ Status bar with aggregate stats
- ✅ No persistent sidebar - focus on VMs
- ✅ Right-click context menu
- ✅ Double-click to open VM details in new window

---

#### **Window 2: VM Details Window (Per VM)**
Opens when double-clicking a VM or selecting "Edit" - separate window

```
┌────────────────── ubuntu-server (QEMU/KVM) ──────────────────┐
│ File  Edit  View  Help                                   ─ □ × │
├───────────────────────────────────────────────────────────────┤
│ [▶ Start] [⏹ Stop] [⏸ Pause] [↻ Reboot] │ [💻 Console] [❌]  │ ← Actions
├──────────────┬────────────────────────────────────────────────┤
│              │ ⚙ Overview    📊 Performance    📸 Snapshots   │ ← Tabs
│ Hardware     │ ┌──────────────────────────────────────────┐  │
│              │ │ Virtual Machine Details                  │  │
│ 🔹 Overview  │ │                                           │  │
│ 🧠 CPUs      │ │ Name: ubuntu-server                      │  │
│ 💾 Memory    │ │ State: ● Running                         │  │
│ ⚙ Boot       │ │ Uptime: 2d 4h 23m                        │  │
│              │ │                                           │  │
│ Storage      │ │ Resources                                │  │
│ 💿 Disk 1    │ │ • CPUs: 2 cores (1 socket × 2 cores)     │  │
│ 💿 Disk 2    │ │ • Memory: 4096 MB                        │  │
│ 📀 CDROM     │ │ • Disk: 20 GB (qcow2)                    │  │
│              │ │                                           │  │
│ Network      │ │ Guest OS Information                     │  │
│ 🌐 NIC 1     │ │ • OS: Ubuntu 24.04 LTS                   │  │
│              │ │ • Kernel: 6.8.0-48-generic               │  │
│ Display      │ │ • IP Address: 192.168.122.45            │  │
│ 🖥 Graphics   │ │ • Hostname: ubuntu-server                │  │
│ 🎬 Video     │ │                                           │  │
│              │ │ Firmware & Hardware                      │  │
│ Other        │ │ • Firmware: BIOS                         │  │
│ 🎵 Sound     │ │ • Chipset: Q35                           │  │
│ ⌨ Input      │ │ • Architecture: x86_64                   │  │
│ 🎲 RNG       │ │                                           │  │
│ 🔒 TPM       │ └──────────────────────────────────────────┘  │
│ 📡 Channels  │                                                │
│ 🎛 Control   │  [Edit Hardware] [Advanced Settings]          │
│              │                                                │
│ [➕ Add]     │                                                │
└──────────────┴────────────────────────────────────────────────┘
```

**Features**:
- ✅ **Separate window** for each VM (can have multiple open)
- ✅ Window title: VM name + connection
- ✅ Action toolbar at top
- ✅ **Left sidebar**: Hardware tree (like virt-manager)
  - Grouped by category (Hardware, Storage, Network, Display, Other)
  - Icons for each device type
  - Expandable/collapsible groups
- ✅ **Right panel**: Tabbed interface
  - Overview tab: Summary + quick actions
  - Performance tab: Live graphs
  - Snapshots tab: Snapshot management
- ✅ Device-specific configuration when clicking device in tree
- ✅ "Add Hardware" button in left sidebar

**Tabs in Details Panel**:
1. **Overview**: General info, quick stats, guest agent data
2. **Performance**: ResourceGraphs component (already exists)
3. **Snapshots**: Snapshot tree and management

**When Clicking a Device** (e.g., "CPUs"):
```
Right panel switches to:
┌──────────────────────────────────────────┐
│ 🧠 CPU Configuration                     │
│ ┌──────┬──────────────────────────────┐ │
│ │ Details │ XML                       │ │
│ └──────┴──────────────────────────────┘ │
│                                          │
│ vCPUs: [2] ▲▼                            │
│                                          │
│ ⚙ CPU Topology                           │
│ ├─ Sockets: [1] ▲▼                       │
│ ├─ Cores per socket: [2] ▲▼              │
│ └─ Threads per core: [1] ▲▼              │
│                                          │
│ CPU Model: [host-passthrough ▼]          │
│                                          │
│ ☑ Copy host CPU configuration            │
│                                          │
│           [Apply] [Revert]               │
└──────────────────────────────────────────┘
```

---

#### **Window 3: Console Window (Per VM)**
Opens when clicking "Console" - separate window for each VM

```
┌──────────── ubuntu-server Console ─────────────┐
│ View  Send  Help                            ─ □ × │
├────────────────────────────────────────────────┤
│ [⛶ Fullscreen] [📷 Screenshot] [⌨ Send Keys ▼]│
├────────────────────────────────────────────────┤
│                                                 │
│  ┌─────────────────────────────────────────┐  │
│  │                                          │  │
│  │                                          │  │
│  │     VNC/SPICE Console Display            │  │
│  │                                          │  │
│  │     (VM screen output)                   │  │
│  │                                          │  │
│  │                                          │  │
│  │                                          │  │
│  │                                          │  │
│  └─────────────────────────────────────────┘  │
│                                                 │
├────────────────────────────────────────────────┤
│ Connected via VNC │ Resolution: 1920×1080     │
└────────────────────────────────────────────────┘
```

**Features**:
- ✅ Dedicated window for console (can have multiple VMs open)
- ✅ Minimal UI - focus on VM screen
- ✅ Console-specific toolbar (fullscreen, screenshot, send keys)
- ✅ Status bar showing connection type and resolution
- ✅ Can be maximized/fullscreened independently

---

### Menu Bar Structure

#### **File Menu**
```
File
├─ New VM...                    Ctrl+N
├─ Import VM...                 Ctrl+I
├─ ──────────────
├─ New Connection...
├─ Close Connection
├─ ──────────────
├─ Preferences...               Ctrl+,
├─ ──────────────
├─ Quit                         Ctrl+Q
```

#### **Edit Menu** (VM-specific, context-aware)
```
Edit
├─ VM Details...                Ctrl+D
├─ Clone VM...
├─ Rename...
├─ Delete...                    Delete
├─ ──────────────
├─ Take Snapshot...             Ctrl+S
├─ Manage Snapshots...
```

#### **View Menu**
```
View
├─ Refresh                      F5
├─ ──────────────
├─ ☑ Toolbar
├─ ☑ Status Bar
├─ ──────────────
├─ Filter by Tag >
│  ├─ All VMs
│  ├─ Production
│  ├─ Development
│  └─ Testing
```

#### **Actions Menu** (VM operations)
```
Actions
├─ Start                        Ctrl+↑
├─ Stop                         Ctrl+↓
├─ Force Stop                   Ctrl+Shift+↓
├─ Pause                        Ctrl+P
├─ Resume
├─ Reboot                       Ctrl+R
├─ ──────────────
├─ Open Console                 Ctrl+O
├─ ──────────────
├─ Batch Operations >
│  ├─ Start Selected
│  ├─ Stop Selected
│  └─ Delete Selected
```

#### **Tools Menu**
```
Tools
├─ Storage Manager...
├─ Network Manager...
├─ ──────────────
├─ Templates...
├─ Schedules...
├─ Alerts...
├─ Backups...
├─ ──────────────
├─ Performance Monitor
├─ Optimization Suggestions
```

#### **Help Menu**
```
Help
├─ Documentation
├─ Keyboard Shortcuts           Ctrl+?
├─ ──────────────
├─ Check for Updates...
├─ Report Issue...
├─ ──────────────
├─ About KVM Manager
```

---

### Where Do Current Features Live?

#### **Current Pages → New Locations**

| Current Page | New Location | Access Method |
|-------------|--------------|---------------|
| Dashboard | **Remove** | Stats shown in main window status bar |
| VMs | **Main Window** | Default view |
| Storage | **Tools → Storage Manager** | Opens dialog/window |
| Networks | **Tools → Network Manager** | Opens dialog/window |
| Insights | **Tools → Performance Monitor** | Opens window |
| Templates | **Tools → Templates** | Opens dialog |
| Schedules | **Tools → Schedules** | Opens dialog |
| Alerts | **Tools → Alerts** | Opens dialog |
| Backups | **Tools → Backups** | Opens dialog |
| Settings | **File → Preferences** | Opens dialog |

**Rationale**: Desktop apps focus on the primary task (VM management) in the main window. Secondary features become tools accessed via menu/toolbar.

---

## Implementation Plan

### Phase 1: Restructure Main Window (Week 1)

#### **Tasks**:
- [ ] Remove persistent left sidebar
- [ ] Add menu bar using Tauri's menu API
  ```rust
  use tauri::menu::{Menu, MenuItem, Submenu};
  ```
- [ ] Add toolbar component
  ```tsx
  // src/components/layout/Toolbar.tsx
  <Toolbar>
    <ToolbarButton icon={Plus} onClick={createVm}>New VM</ToolbarButton>
    <ToolbarButton icon={Play} onClick={startVm}>Start</ToolbarButton>
    <ToolbarButton icon={Square} onClick={stopVm}>Stop</ToolbarButton>
    <ToolbarSeparator />
    <ToolbarButton icon={Monitor} onClick={openConsole}>Console</ToolbarButton>
  </Toolbar>
  ```
- [ ] Add connection selector bar
  ```tsx
  <ConnectionBar>
    <Select value="qemu:///system">
      <SelectItem>QEMU/KVM (localhost)</SelectItem>
      <SelectItem>Add connection...</SelectItem>
    </Select>
  </ConnectionBar>
  ```
- [ ] Add status bar
  ```tsx
  <StatusBar>
    <StatusItem>{vmCount} VMs</StatusItem>
    <StatusItem>{runningCount} Running</StatusItem>
    <StatusSpacer />
    <StatusItem>CPU: {cpuUsage}%</StatusItem>
    <StatusItem>Memory: {memUsed}/{memTotal}GB</StatusItem>
  </StatusBar>
  ```
- [ ] Simplify VM list to table view (remove cards)
  - Use data table component
  - Columns: Checkbox, Name, State, CPU, Memory, Disk, Network
  - Row actions: Right-click context menu
  - Double-click to open details

#### **Files to Create**:
- `src/components/layout/MenuBar.tsx`
- `src/components/layout/Toolbar.tsx`
- `src/components/layout/ConnectionBar.tsx`
- `src/components/layout/StatusBar.tsx`
- `src/components/vm/VmTable.tsx` (replace card-based list)

#### **Files to Modify**:
- `src/components/layout/Layout.tsx` - Complete redesign
- `src/pages/VmList.tsx` - Simplify to just table
- `src-tauri/src/lib.rs` - Add menu configuration

---

### Phase 2: Multi-Window Support (Week 2)

#### **Tasks**:
- [ ] Configure Tauri for multi-window
  ```json
  // src-tauri/tauri.conf.json
  "windows": [
    {
      "label": "main",
      "title": "KVM Manager",
      "width": 1200,
      "height": 800
    }
  ]
  ```
- [ ] Create VM Details window
  ```rust
  // src-tauri/src/commands/window.rs
  #[tauri::command]
  pub fn open_vm_details(app: AppHandle, vm_id: String) -> Result<(), String> {
      let window = tauri::WebviewWindow::builder(
          &app,
          format!("vm-details-{}", vm_id),
          tauri::WebviewUrl::App(format!("vm/{}", vm_id).into())
      )
      .title(format!("VM Details: {}", vm_id))
      .build()
      .map_err(|e| e.to_string())?;
      Ok(())
  }
  ```
- [ ] Create Console window
  ```rust
  #[tauri::command]
  pub fn open_console_window(app: AppHandle, vm_id: String) -> Result<(), String> {
      let window = tauri::WebviewWindow::builder(
          &app,
          format!("console-{}", vm_id),
          tauri::WebviewUrl::App(format!("console/{}", vm_id).into())
      )
      .title(format!("Console: {}", vm_id))
      .build()
      .map_err(|e| e.to_string())?;
      Ok(())
  }
  ```
- [ ] Update routing to support windowed navigation
  ```tsx
  // src/App.tsx
  <Routes>
    <Route path="/" element={<MainWindow />} />
    <Route path="/vm/:vmId" element={<VmDetailsWindow />} />
    <Route path="/console/:vmId" element={<ConsoleWindow />} />
  </Routes>
  ```

#### **Window Management**:
- Track open windows to prevent duplicates
- Auto-close detail window when VM is deleted
- Window state persistence (size, position)

---

### Phase 3: VM Details Window Redesign (Week 3)

#### **Tasks**:
- [ ] Create hardware device tree sidebar
  ```tsx
  // src/components/vm/HardwareTree.tsx
  <TreeView>
    <TreeSection title="Hardware">
      <TreeItem icon={Monitor} label="Overview" selected />
      <TreeItem icon={Cpu} label="CPUs" />
      <TreeItem icon={MemoryStick} label="Memory" />
      <TreeItem icon={Settings} label="Boot Options" />
    </TreeSection>
    <TreeSection title="Storage">
      <TreeItem icon={HardDrive} label="VirtIO Disk 1" />
      <TreeItem icon={Disc} label="SATA CDROM 1" />
    </TreeSection>
    <TreeSection title="Network">
      <TreeItem icon={Network} label="NIC 1" />
    </TreeSection>
    <TreeSection title="Display">
      <TreeItem icon={Monitor} label="Graphics (VNC)" />
      <TreeItem icon={Video} label="Video (Virtio)" />
    </TreeSection>
    <TreeSection title="Other">
      <TreeItem icon={Volume2} label="Sound" />
      <TreeItem icon={Keyboard} label="Input Devices" />
      <TreeItem icon={ShieldCheck} label="TPM" />
    </TreeSection>
    <TreeAction>
      <Button icon={Plus}>Add Hardware</Button>
    </TreeAction>
  </TreeView>
  ```
- [ ] Create tabbed details panel (Overview, Performance, Snapshots)
- [ ] Device-specific configuration panels
  - When clicking device in tree, show its config in right panel
  - Each device type has its own editor component
- [ ] Implement "Add Hardware" dialog (launched from tree)

#### **Files to Create**:
- `src/components/vm/HardwareTree.tsx`
- `src/components/vm/VmDetailsWindow.tsx`
- `src/components/vm/DeviceEditor.tsx` (generic)
- `src/components/vm/devices/CpuEditor.tsx`
- `src/components/vm/devices/MemoryEditor.tsx`
- `src/components/vm/devices/DiskEditor.tsx`
- ... (one for each device type)

---

### Phase 4: Desktop Polish (Week 4)

#### **Visual Design Updates**:
- [ ] **Reduce spacing** - Desktop apps are denser than web
  - Tighter padding in tables
  - Smaller gaps between elements
  - More compact toolbar buttons
- [ ] **Desktop color palette**
  - More muted colors
  - Less saturated accent colors
  - Professional grays
- [ ] **Typography adjustments**
  - Slightly smaller base font size (14px → 13px)
  - Less bold text
  - More consistent sizing
- [ ] **Icon updates**
  - Consistent icon set (lucide-react)
  - Monochrome icons in toolbars
  - Smaller icon sizes (20px → 16px in toolbars)

#### **Interactions**:
- [ ] **Right-click context menus**
  ```tsx
  <ContextMenu>
    <ContextMenuItem icon={Play}>Start</ContextMenuItem>
    <ContextMenuItem icon={Square}>Stop</ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem icon={Monitor}>Open Console</ContextMenuItem>
    <ContextMenuItem icon={Edit}>Edit Details</ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem icon={Copy}>Clone...</ContextMenuItem>
    <ContextMenuItem icon={Trash}>Delete...</ContextMenuItem>
  </ContextMenu>
  ```
- [ ] **Keyboard shortcuts** (already have some)
  - Enhance existing keyboard shortcuts
  - Add shortcuts for menu items
  - Show shortcuts in tooltips
- [ ] **Double-click behaviors**
  - Double-click VM → Open details window
  - Double-click device in tree → Edit inline
- [ ] **Drag & drop** (future)
  - Drag ISO onto VM to attach
  - Reorder boot devices

#### **Desktop Conventions**:
- [ ] **Window management**
  - Remember window positions
  - Restore open windows on app launch
  - Proper window close behavior (minimize to tray?)
- [ ] **Native title bar** (optional)
  - Use system title bar instead of custom
  - Platform-appropriate window controls
- [ ] **System tray integration** (optional)
  - Minimize to tray
  - Quick actions from tray menu
  - VM state notifications

---

## Visual Design System

### Color Palette (Desktop-Native)

#### **Light Theme** (Default)
```css
:root {
  /* Backgrounds */
  --window-bg: #f5f5f5;           /* Window background */
  --panel-bg: #ffffff;            /* Panels, cards */
  --sidebar-bg: #f0f0f0;          /* Sidebar background */
  --toolbar-bg: #fafafa;          /* Toolbar background */

  /* Borders */
  --border: #d0d0d0;              /* Standard borders */
  --border-subtle: #e0e0e0;       /* Subtle dividers */

  /* Text */
  --text-primary: #2c2c2c;        /* Main text */
  --text-secondary: #666666;      /* Secondary text */
  --text-disabled: #999999;       /* Disabled text */

  /* Accents */
  --accent-blue: #0066cc;         /* Primary actions */
  --accent-green: #00aa00;        /* Success, running state */
  --accent-orange: #cc6600;       /* Warnings, paused state */
  --accent-red: #cc0000;          /* Errors, stopped state */

  /* Selection */
  --selected-bg: #e3f2fd;         /* Selected items */
  --hover-bg: #f5f5f5;            /* Hover state */
}
```

#### **Dark Theme**
```css
:root[data-theme="dark"] {
  /* Backgrounds */
  --window-bg: #1e1e1e;
  --panel-bg: #252525;
  --sidebar-bg: #2a2a2a;
  --toolbar-bg: #2d2d2d;

  /* Borders */
  --border: #3a3a3a;
  --border-subtle: #303030;

  /* Text */
  --text-primary: #e0e0e0;
  --text-secondary: #a0a0a0;
  --text-disabled: #606060;

  /* Accents */
  --accent-blue: #4da3ff;
  --accent-green: #4dff4d;
  --accent-orange: #ffaa4d;
  --accent-red: #ff4d4d;

  /* Selection */
  --selected-bg: #1a4d7a;
  --hover-bg: #2a2a2a;
}
```

### Typography

```css
/* Desktop-appropriate sizing */
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Ubuntu', sans-serif;
  font-size: 13px;        /* Base size - slightly smaller than web */
  line-height: 1.4;
}

.menu-bar {
  font-size: 13px;
}

.toolbar-button {
  font-size: 12px;
}

.table-header {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.table-cell {
  font-size: 13px;
}

.status-bar {
  font-size: 11px;
}

h1 { font-size: 24px; font-weight: 600; }
h2 { font-size: 18px; font-weight: 600; }
h3 { font-size: 14px; font-weight: 600; }
h4 { font-size: 13px; font-weight: 600; }
```

### Spacing & Density

```css
/* Desktop apps are denser than web */

/* Panel padding */
.panel {
  padding: 12px;          /* vs 16-24px on web */
}

/* Toolbar */
.toolbar {
  padding: 6px 8px;
  gap: 4px;               /* Tight spacing between buttons */
}

.toolbar-button {
  padding: 6px 12px;
  gap: 6px;
}

/* Table */
.table-row {
  height: 32px;           /* Compact rows */
  padding: 0 12px;
}

/* Sidebar tree */
.tree-item {
  height: 28px;
  padding: 4px 8px;
}

/* Form fields */
.form-field {
  margin-bottom: 12px;
}

.label {
  margin-bottom: 4px;
}
```

---

## Migration Strategy

### Approach: Gradual Transition

**Phase 1**: Main window only (keep existing pages as fallback)
- Implement new Layout.tsx with menu bar, toolbar
- Keep sidebar navigation temporarily (marked as "legacy")
- Add toggle: "Use desktop UI" in settings

**Phase 2**: Add multi-window support
- VM details can open in new window OR in-app (user choice)
- Console always opens in new window

**Phase 3**: Full transition
- Remove sidebar navigation
- Convert all "pages" to dialogs/windows
- Make desktop UI the default

### User Communication
- [ ] Add "What's New" dialog on first launch after update
- [ ] Show benefits of desktop UI (multiple windows, cleaner interface)
- [ ] Provide setting to revert temporarily (during transition period)
- [ ] Create video/guide showing new workflow

---

## Success Metrics

### User Experience
- ✅ Users describe app as "desktop-native" not "web-like"
- ✅ Faster VM operations (fewer clicks)
- ✅ Reduced visual clutter
- ✅ Keyboard shortcuts feel natural

### Technical
- ✅ Main window loads < 500ms
- ✅ Can open 5+ VM detail windows simultaneously
- ✅ Window state persists across sessions
- ✅ Memory usage < 200MB with 3 windows open

### Parity
- ✅ All virt-manager workflows supported
- ✅ Feature parity maintained (no features lost)
- ✅ Better multi-tasking (multiple VMs/consoles open)

---

## Example Workflows

### Workflow 1: Create and Start a VM
**Virt-Manager**:
1. Click "New VM" button in toolbar
2. Wizard opens (same window)
3. Complete 5 steps
4. Click "Begin Installation"
5. New window opens with VM console

**KVM Manager (New)**:
1. Click "New VM" button in toolbar (or Ctrl+N, or File → New VM)
2. Wizard dialog opens
3. Complete 4 steps
4. Click "Create"
5. VM appears in main window list
6. Double-click to open details window
7. Click "Start" in details window
8. Click "Console" to open console window

### Workflow 2: Configure VM Hardware
**Virt-Manager**:
1. Double-click VM in main window
2. VM details window opens
3. Click device in left sidebar (e.g., "CPUs")
4. Right panel shows CPU configuration
5. Make changes
6. Click "Apply"

**KVM Manager (New)** - IDENTICAL:
1. Double-click VM in main window
2. VM details window opens
3. Click device in left sidebar (e.g., "CPUs")
4. Right panel shows CPU configuration
5. Make changes
6. Click "Apply"

### Workflow 3: Monitor Multiple VMs
**Virt-Manager**: Not great - can only view one VM's details at a time

**KVM Manager (New)**: Better - multiple windows
1. Double-click VM 1 → Opens details window 1
2. Double-click VM 2 → Opens details window 2
3. Click "Performance" tab in each window
4. Arrange windows side-by-side
5. Monitor both simultaneously

---

## Component Library Updates

### New Desktop Components

#### **Toolbar**
```tsx
// src/components/desktop/Toolbar.tsx
interface ToolbarProps {
  children: React.ReactNode
}

export function Toolbar({ children }: ToolbarProps) {
  return (
    <div className="toolbar">
      {children}
    </div>
  )
}

export function ToolbarButton({ icon: Icon, children, onClick, disabled }) {
  return (
    <button className="toolbar-button" onClick={onClick} disabled={disabled}>
      <Icon className="w-4 h-4" />
      {children && <span>{children}</span>}
    </button>
  )
}

export function ToolbarSeparator() {
  return <div className="toolbar-separator" />
}
```

#### **StatusBar**
```tsx
// src/components/desktop/StatusBar.tsx
export function StatusBar({ children }: { children: React.ReactNode }) {
  return <div className="status-bar">{children}</div>
}

export function StatusItem({ children }) {
  return <div className="status-item">{children}</div>
}

export function StatusSpacer() {
  return <div className="status-spacer" />
}
```

#### **TreeView**
```tsx
// src/components/desktop/TreeView.tsx
export function TreeView({ children }) {
  return <div className="tree-view">{children}</div>
}

export function TreeSection({ title, children, defaultOpen = true }) {
  const [open, setOpen] = useState(defaultOpen)
  return (
    <div className="tree-section">
      <div className="tree-section-header" onClick={() => setOpen(!open)}>
        {open ? <ChevronDown /> : <ChevronRight />}
        <span>{title}</span>
      </div>
      {open && <div className="tree-section-content">{children}</div>}
    </div>
  )
}

export function TreeItem({ icon: Icon, label, selected, onClick }) {
  return (
    <div
      className={cn("tree-item", selected && "selected")}
      onClick={onClick}
    >
      <Icon className="w-4 h-4" />
      <span>{label}</span>
    </div>
  )
}
```

---

## Next Steps (Immediate Actions)

### Week 1: Main Window Redesign
- [ ] **Day 1**: Design and implement Toolbar component
- [ ] **Day 2**: Design and implement StatusBar component
- [ ] **Day 3**: Redesign Layout.tsx with menu bar
- [ ] **Day 4**: Convert VM list from cards to table (VmTable.tsx)
- [ ] **Day 5**: Add connection selector bar

### Week 2: Multi-Window Foundation
- [ ] **Day 1-2**: Configure Tauri for multi-window support
- [ ] **Day 3**: Implement window management commands
- [ ] **Day 4**: Create basic VM details window shell
- [ ] **Day 5**: Test window creation/closing/management

### Week 3: Hardware Tree & Details
- [ ] **Day 1-2**: Build TreeView and HardwareTree components
- [ ] **Day 3**: Integrate hardware tree with VM details window
- [ ] **Day 4-5**: Create device editor components (CPU, Memory, Disk)

### Week 4: Polish & Refinement
- [ ] **Day 1**: Visual design updates (colors, spacing, typography)
- [ ] **Day 2**: Right-click context menus
- [ ] **Day 3**: Enhanced keyboard shortcuts
- [ ] **Day 4**: Window state persistence
- [ ] **Day 5**: Testing and bug fixes

---

## Conclusion

**Goal**: Transform KVM Manager from a feature-rich web app into a **best-in-class desktop application** that:
- ✅ Feels native to the desktop environment
- ✅ Flows like virt-manager (familiar to users)
- ✅ Leverages modern UI patterns (clean, focused)
- ✅ Supports power-user workflows (multiple windows, keyboard shortcuts)
- ✅ Maintains feature advantages (templates, schedules, better monitoring)

**Result**: A virtualization manager that combines:
- Virt-manager's **desktop-native UX**
- KVM Manager's **modern feature set**
- Best-in-class **visual design**

**Timeline**: 4 weeks to complete desktop UI transformation 🚀
