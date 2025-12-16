# Week 5: Console Integration & Snapshot Management

**Date Started**: December 12, 2025
**Status**: 🚀 **READY TO BEGIN**
**Phase**: Core Feature Completion
**Duration**: 5-7 days

---

## Executive Summary

Week 5 focuses on completing two critical MVP features that will bring KVM Manager to full functional parity with essential virtualization tools:

1. **Console Integration** (Currently 70% → Target: 100%)
   - Integrate noVNC library for real VNC viewing
   - Add console toolbar features (fullscreen, send keys, screenshots)
   - Implement SPICE console support (stretch goal)

2. **Snapshot Management** (Currently 60% → Target: 100%)
   - Implement backend snapshot commands (create, delete, revert, list)
   - Complete snapshot UI with tree visualization
   - Add snapshot metadata (name, description, timestamp)

### Why These Features?

**Console**: Essential for VM management - users need to interact with VMs through the graphical console. Window structure is ready, just needs viewer integration.

**Snapshots**: Critical for safe VM management - take snapshots before updates, quickly rollback if something breaks. UI framework exists, needs backend implementation.

---

## 📋 Week 5 Implementation Plan

### Day 1-3: Console Integration (noVNC)

#### Day 1: noVNC Library Integration
**Goal**: Get basic VNC viewing working in the console window

**Tasks**:
- [ ] Install noVNC npm package (`npm install @novnc/novnc`)
- [ ] Create VncViewer React component
- [ ] Set up VNC connection to libvirt's VNC socket
- [ ] Handle VNC authentication (if required)
- [ ] Display VNC stream in ConsoleWindow
- [ ] Test with a running VM

**Backend Work**:
- [ ] Implement `get_vnc_connection_info` command
  - Return VNC host, port, password
  - Handle libvirt VNC socket configuration
- [ ] Implement VNC proxy if direct connection not possible
  - WebSocket proxy for VNC over TCP
  - Security considerations

**Success Criteria**:
- Can see VM screen in console window
- Mouse and keyboard input works
- Connection status shows clearly

---

#### Day 2: Console Toolbar Features
**Goal**: Add professional console controls

**Features to Implement**:
- [ ] **Fullscreen Toggle**
  - Enter/exit fullscreen mode (F11 or button)
  - Escape key to exit fullscreen
- [ ] **Screenshot**
  - Capture current console view
  - Save as PNG to user's Pictures folder
  - Show save location in toast
- [ ] **Send Special Keys**
  - Ctrl+Alt+Del (for Windows VMs)
  - Ctrl+Alt+Backspace
  - Ctrl+Alt+F1-F12 (switch TTY on Linux)
  - Menu with common key combinations
- [ ] **Connection Status**
  - Show "Connected", "Connecting...", "Disconnected"
  - Reconnect button if connection lost
  - Connection quality indicator
- [ ] **Scale/Fit Options**
  - Scale to window (default)
  - 1:1 pixel mapping
  - Stretch to fill
  - Auto-resize guest display (requires guest agent)

**UI Enhancements**:
- [ ] Minimal toolbar (semi-transparent, auto-hide in fullscreen)
- [ ] Keyboard shortcut overlay (F1 for help)
- [ ] Console performance stats (optional: FPS, latency)

**Success Criteria**:
- Fullscreen mode works smoothly
- Can send Ctrl+Alt+Del to Windows VMs
- Screenshot saves successfully
- Toolbar auto-hides in fullscreen

---

#### Day 3: Console Polish & Error Handling
**Goal**: Make console reliable and user-friendly

**Tasks**:
- [ ] **Connection Error Handling**
  - Handle "VM not running" (show helpful message)
  - Handle "VNC not enabled" (suggest enabling graphics)
  - Handle network/socket errors
  - Retry logic with exponential backoff
- [ ] **Loading States**
  - Show "Connecting to VM..." spinner
  - Skeleton loader before connection
  - Smooth transition when connected
- [ ] **Clipboard Integration** (if possible with noVNC)
  - Bidirectional clipboard (host ↔ guest)
  - Paste text from host to guest
  - Copy text from guest to host
- [ ] **Console Settings** (in Settings page)
  - VNC quality/compression settings
  - Keyboard layout selection
  - Mouse capture mode
- [ ] **Multi-monitor Support** (stretch)
  - If VM has multiple displays, show selector

**Testing**:
- [ ] Test with Linux VMs (Ubuntu, Fedora)
- [ ] Test with Windows VMs (if available)
- [ ] Test connection failures and recovery
- [ ] Test keyboard layouts (US, UK, etc.)

**Success Criteria**:
- Graceful error messages with actionable suggestions
- Reconnects automatically if connection drops
- Fast loading with clear status indicators
- Works reliably with different VM types

---

### Day 4-5: Snapshot Management Backend

#### Day 4: Snapshot Backend Commands
**Goal**: Implement all snapshot operations in Rust

**Backend Tasks** (src-tauri/src/commands/snapshots.rs):

1. [ ] **list_snapshots** command
   ```rust
   #[tauri::command]
   async fn list_snapshots(vm_id: String) -> Result<Vec<Snapshot>, String>
   ```
   - Get all snapshots for a VM
   - Return snapshot metadata (name, description, timestamp, parent)
   - Build snapshot tree structure

2. [ ] **create_snapshot** command
   ```rust
   #[tauri::command]
   async fn create_snapshot(
       vm_id: String,
       name: String,
       description: Option<String>
   ) -> Result<Snapshot, String>
   ```
   - Create new snapshot
   - Validate snapshot name (unique, no special chars)
   - Handle snapshot of running vs stopped VMs
   - Optional: Quiesce filesystem (if guest agent available)

3. [ ] **delete_snapshot** command
   ```rust
   #[tauri::command]
   async fn delete_snapshot(
       vm_id: String,
       snapshot_name: String
   ) -> Result<(), String>
   ```
   - Delete snapshot by name
   - Handle snapshot with children (delete children or fail)
   - Update UI after deletion

4. [ ] **revert_snapshot** command
   ```rust
   #[tauri::command]
   async fn revert_snapshot(
       vm_id: String,
       snapshot_name: String
   ) -> Result<(), String>
   ```
   - Revert VM to snapshot
   - Stop VM if running (with confirmation)
   - Restore VM state and disk
   - Restart VM if it was running

5. [ ] **get_snapshot_info** command
   ```rust
   #[tauri::command]
   async fn get_snapshot_info(
       vm_id: String,
       snapshot_name: String
   ) -> Result<SnapshotDetails, String>
   ```
   - Get detailed info about a snapshot
   - Disk usage, memory state, timestamp
   - Parent/children relationships

**Data Models** (src-tauri/src/models/snapshot.rs):
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub name: String,
    pub description: Option<String>,
    pub created_at: String, // ISO 8601 timestamp
    pub state: SnapshotState, // Running, ShutOff, etc.
    pub parent: Option<String>,
    pub children: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SnapshotDetails {
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub state: SnapshotState,
    pub memory_size: Option<u64>, // bytes
    pub disk_size: u64, // bytes
    pub parent: Option<String>,
    pub children: Vec<String>,
}
```

**Libvirt Integration**:
- Use `virt::domain::Domain::list_all_snapshots()`
- Use `virt::domain::Domain::snapshot_create_xml()`
- Use `virt::snapshot::DomainSnapshot::delete()`
- Use `virt::domain::Domain::revert_to_snapshot()`

**Success Criteria**:
- All snapshot commands work with libvirt
- Error handling for common failures (name conflicts, missing snapshots)
- Snapshots persist across VM restarts
- Can create/delete/revert snapshots successfully

---

#### Day 5: Snapshot UI Enhancement
**Goal**: Complete the snapshot manager UI

**Frontend Tasks** (src/components/vm/SnapshotManager.tsx):

1. [ ] **Snapshot List with Tree View**
   - Display snapshots in tree structure (parent-child)
   - Show current snapshot (highlighted)
   - Show snapshot metadata (name, date, state)
   - Expand/collapse tree branches

2. [ ] **Create Snapshot Dialog**
   - Text input for snapshot name
   - Textarea for description (optional)
   - Checkbox: "Quiesce filesystem" (if guest agent available)
   - Preview: "Snapshot will be created as child of <current>"
   - Create button with loading state

3. [ ] **Snapshot Actions**
   - Context menu on each snapshot:
     - Revert to this snapshot
     - Delete snapshot
     - Delete snapshot and children
     - View details
   - Toolbar buttons:
     - Create Snapshot (Plus icon)
     - Delete Selected
     - Revert to Selected
   - Keyboard shortcuts:
     - Ctrl+T → Create snapshot
     - Delete → Delete selected snapshot
     - Ctrl+R → Revert to selected

4. [ ] **Snapshot Details Panel**
   - Show when snapshot selected
   - Display all metadata
   - Show disk usage
   - Show memory size (if snapshot has memory state)
   - "Revert" and "Delete" buttons

5. [ ] **Confirmation Dialogs**
   - Confirm before reverting (warns about losing current state)
   - Confirm before deleting (especially if has children)
   - Show impact: "This will stop the VM and restore state from <date>"

6. [ ] **Error Handling**
   - Show error if snapshot creation fails
   - Show error if revert fails
   - Suggest actions (e.g., "Stop VM first to revert")

7. [ ] **Loading States**
   - Show spinner while loading snapshots
   - Skeleton loader for snapshot tree
   - Disable buttons during operations
   - Show progress for long operations

**Integration**:
- [ ] Add snapshot indicator to VM cards
  - Show snapshot count badge (e.g., "3 snapshots")
  - Quick action: "Create Snapshot"
- [ ] Add snapshot tab to VM details window (already exists)
- [ ] Update toolbar to show "Take Snapshot" button

**Success Criteria**:
- Can create snapshots with names and descriptions
- Can view snapshot tree clearly
- Can revert to any snapshot
- Can delete snapshots (with confirmation)
- All operations show clear feedback (success/error)

---

### Day 6-7: Testing, Polish & Documentation

#### Day 6: Integration Testing
**Goal**: Ensure console and snapshots work reliably

**Console Testing**:
- [ ] Test VNC connection with multiple VMs simultaneously
- [ ] Test console performance (low latency, smooth)
- [ ] Test fullscreen mode across different window sizes
- [ ] Test special key sending (Ctrl+Alt+Del)
- [ ] Test connection recovery (stop/start VM while console open)
- [ ] Test screenshot feature

**Snapshot Testing**:
- [ ] Create 10+ snapshots in tree structure
- [ ] Revert to different points in tree
- [ ] Delete snapshots (leaf nodes and with children)
- [ ] Test snapshot creation on running VMs
- [ ] Test snapshot creation on stopped VMs
- [ ] Test snapshot with large VMs (multi-GB disks)
- [ ] Verify snapshot metadata accuracy

**Cross-Feature Testing**:
- [ ] Take snapshot, revert, then open console
- [ ] Open console, take snapshot while viewing
- [ ] Delete VM with snapshots (should prevent or warn)

**Performance Testing**:
- [ ] Console responsiveness (measure latency)
- [ ] Snapshot creation time (for various VM sizes)
- [ ] Memory usage during console viewing
- [ ] Multiple consoles open simultaneously

---

#### Day 7: Polish, Documentation & Week Review

**Polish Tasks**:
- [ ] Fix any bugs found during testing
- [ ] Improve console toolbar auto-hide behavior
- [ ] Add tooltips to all snapshot actions
- [ ] Ensure keyboard shortcuts work everywhere
- [ ] Update StatusBar to show console/snapshot activity

**Documentation**:
- [ ] Update CURRENT_STATUS.md with completed features
- [ ] Create WEEK5_COMPLETE.md with summary
- [ ] Update README.md with console and snapshot features
- [ ] Add inline code comments for complex logic
- [ ] Create user guide section for snapshots

**User Experience**:
- [ ] Ensure error messages are helpful
- [ ] Add "First time?" hints for new users
- [ ] Verify all buttons have proper hover states
- [ ] Check accessibility (keyboard navigation, ARIA labels)

**Build Verification**:
- [ ] Run full TypeScript build (npm run build)
- [ ] Run Rust build (cargo build --release)
- [ ] Test in production mode (npm run tauri build)
- [ ] Verify no console errors or warnings

---

## 🎯 Success Criteria (Week 5)

### Console Integration ✅
- [x] Can view VM console through noVNC
- [x] Fullscreen mode works
- [x] Can send Ctrl+Alt+Del and special keys
- [x] Screenshot feature works
- [x] Connection errors handled gracefully
- [x] Console toolbar is intuitive and minimal

### Snapshot Management ✅
- [x] Can create snapshots with name/description
- [x] Can list all snapshots in tree view
- [x] Can revert to any snapshot
- [x] Can delete snapshots
- [x] Snapshot metadata is accurate
- [x] UI shows clear feedback for all operations

### Overall Quality ✅
- [x] Zero compilation errors
- [x] All features tested and working
- [x] Documentation updated
- [x] User experience is smooth and intuitive

---

## 🚀 Stretch Goals (If Time Permits)

### Advanced Console Features
- [ ] SPICE console support (better performance, USB redirection)
- [ ] Serial console viewer (text-based)
- [ ] Console recording (video capture)
- [ ] Console thumbnail in VM card

### Advanced Snapshot Features
- [ ] Snapshot scheduling (auto-snapshot on schedule)
- [ ] Snapshot disk usage optimization (merge snapshots)
- [ ] Export/import snapshots
- [ ] Snapshot comparison (show what changed)

### Performance
- [ ] VNC connection caching
- [ ] Snapshot lazy loading (only load tree on demand)
- [ ] Console buffer optimization

---

## 📦 Dependencies & Setup

### NPM Packages to Install
```bash
npm install @novnc/novnc
npm install --save-dev @types/novnc  # if available
```

### Rust Crates (already in Cargo.toml)
- `virt` - For libvirt snapshot operations
- `serde_json` - For snapshot metadata
- `chrono` - For timestamps

### Libvirt Requirements
- VMs must have graphics device configured (VNC or SPICE)
- Snapshot support requires qcow2 disk format (not raw)
- External snapshots may require additional configuration

---

## 📊 Technical Architecture

### Console Architecture
```
┌─────────────────────────────────────────┐
│   ConsoleWindow (React Component)      │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │   Console Toolbar                 │  │
│  │   [Fullscreen] [Keys] [Screenshot]│  │
│  └───────────────────────────────────┘  │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │   VncViewer Component (noVNC)     │  │
│  │                                   │  │
│  │   [VM Screen Display]             │  │
│  │                                   │  │
│  └───────────────────────────────────┘  │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │   Status Bar                      │  │
│  │   Connected | 60 FPS | 1920x1080 │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
           │
           │ Tauri IPC
           ▼
┌─────────────────────────────────────────┐
│   Backend (Rust)                        │
│                                         │
│   get_vnc_connection_info()             │
│   ├─ Return VNC host:port               │
│   └─ Return VNC password (if set)       │
│                                         │
│   Optional: VNC WebSocket Proxy         │
│   ├─ Bridge VNC TCP ↔ WebSocket        │
│   └─ Handle authentication              │
└─────────────────────────────────────────┘
```

### Snapshot Architecture
```
┌─────────────────────────────────────────┐
│   SnapshotManager (React Component)    │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │   Snapshot Tree View              │  │
│  │   ├─ Snapshot1 (root)             │  │
│  │   │  ├─ Snapshot2                 │  │
│  │   │  └─ Snapshot3                 │  │
│  │   └─ Snapshot4 (root)             │  │
│  └───────────────────────────────────┘  │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │   Snapshot Details Panel          │  │
│  │   Name: Snapshot2                 │  │
│  │   Created: 2025-12-10 14:30       │  │
│  │   Size: 2.5 GB                    │  │
│  │   [Revert] [Delete]               │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
           │
           │ Tauri IPC
           ▼
┌─────────────────────────────────────────┐
│   Backend (Rust)                        │
│                                         │
│   list_snapshots(vm_id)                 │
│   ├─ Query libvirt snapshots            │
│   └─ Build tree structure               │
│                                         │
│   create_snapshot(vm_id, name, desc)    │
│   ├─ Generate snapshot XML              │
│   ├─ Call libvirt snapshot_create_xml   │
│   └─ Return snapshot metadata           │
│                                         │
│   revert_snapshot(vm_id, snapshot_name) │
│   ├─ Stop VM if running                 │
│   ├─ Call libvirt revert_to_snapshot    │
│   └─ Restart VM if was running          │
│                                         │
│   delete_snapshot(vm_id, snapshot_name) │
│   ├─ Find snapshot by name              │
│   ├─ Call snapshot.delete()             │
│   └─ Update cached snapshot list        │
└─────────────────────────────────────────┘
```

---

## 🗂️ File Structure

### New Files to Create
```
src/
├── components/
│   └── console/
│       ├── VncViewer.tsx              # noVNC integration component
│       ├── ConsoleToolbar.tsx         # Toolbar with fullscreen, keys, etc.
│       └── SendKeysMenu.tsx           # Menu for special key combinations
│
src-tauri/src/
├── commands/
│   ├── console.rs                     # Console-related commands
│   │   ├── get_vnc_connection_info
│   │   ├── vnc_proxy (optional)
│   │   └── get_graphics_config
│   └── snapshots.rs                   # Snapshot commands (NEW)
│       ├── list_snapshots
│       ├── create_snapshot
│       ├── delete_snapshot
│       ├── revert_snapshot
│       └── get_snapshot_info
├── models/
│   └── snapshot.rs                    # Snapshot data models (NEW)
│       ├── Snapshot struct
│       ├── SnapshotDetails struct
│       └── SnapshotState enum
└── services/
    └── snapshot_service.rs            # Snapshot business logic (NEW)
```

### Files to Modify
```
src/
├── pages/
│   └── ConsoleWindow.tsx              # Update with VncViewer component
├── components/vm/
│   └── SnapshotManager.tsx            # Enhance with full CRUD UI
└── lib/
    └── tauri.ts                       # Add snapshot and console APIs

src-tauri/src/
├── lib.rs                             # Register new commands
└── commands/mod.rs                    # Add snapshots module
```

---

## 📚 Resources & References

### noVNC Documentation
- [noVNC GitHub](https://github.com/novnc/noVNC)
- [noVNC API Docs](https://novnc.com/info.html)
- [noVNC Integration Examples](https://github.com/novnc/noVNC/tree/master/docs)

### Libvirt Snapshot Documentation
- [Libvirt Snapshot XML Format](https://libvirt.org/formatsnapshot.html)
- [rust-libvirt Snapshot API](https://docs.rs/virt/latest/virt/domain/struct.Domain.html#snapshots)
- [Snapshot Best Practices](https://libvirt.org/kbase/snapshots.html)

### Related Issues
- Console integration: Essential for VM interaction
- Snapshot management: Critical for safe VM testing/development

---

## 🎯 Definition of Done (Week 5)

Week 5 is complete when:

1. ✅ **Console Viewing**
   - Can view VM console in real-time
   - Keyboard and mouse input works correctly
   - Connection status is clear

2. ✅ **Console Features**
   - Fullscreen mode works
   - Can send special keys (Ctrl+Alt+Del)
   - Screenshot feature functional

3. ✅ **Snapshot CRUD**
   - Can create snapshots with metadata
   - Can list all snapshots
   - Can revert to any snapshot
   - Can delete snapshots

4. ✅ **User Experience**
   - All operations have clear feedback
   - Error messages are helpful
   - Loading states are smooth
   - Documentation is updated

5. ✅ **Quality**
   - No compilation errors
   - All features tested
   - Performance is acceptable
   - Ready for user testing

---

**Let's make Week 5 great! 🚀**

*Start Date: December 12, 2025*
*Target Completion: December 18-19, 2025*
