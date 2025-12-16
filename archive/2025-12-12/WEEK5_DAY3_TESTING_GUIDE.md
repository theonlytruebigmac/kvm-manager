# Week 5 Day 3: Console Testing Guide

**Date**: December 12, 2025 (Evening)
**Status**: Testing Phase
**Purpose**: Comprehensive manual testing checklist for console features

---

## Overview

This guide provides step-by-step testing procedures for all console features implemented in Week 5 Days 1-2.

**Features to Test**:
1. ✅ Basic VNC Connection
2. ✅ Reconnection Logic with Exponential Backoff
3. ✅ Display Scale Modes (3 modes)
4. ✅ Send Keys Menu (13 key combinations)
5. ✅ Screenshot Functionality
6. ✅ Status Indicators
7. ✅ Keyboard Shortcuts

---

## Pre-Testing Setup

### Environment Requirements
- [ ] KVM Manager built and running (`npm run tauri dev`)
- [ ] At least 2 test VMs available:
  - [ ] 1 Linux VM (Ubuntu/Fedora recommended)
  - [ ] 1 Windows VM (if available)
- [ ] VMs configured with VNC graphics (not SPICE)
- [ ] VMs in various states (running, paused, stopped)

### How to Verify VNC is Enabled
```bash
# Check VM XML for graphics configuration
virsh dumpxml <vm-name> | grep -A 5 "<graphics"

# Should see something like:
# <graphics type='vnc' port='5900' autoport='yes' listen='127.0.0.1'>
```

### Test Data Preparation
- [ ] Note down VM names and IDs for testing
- [ ] Have a text file ready for clipboard testing (future)
- [ ] Create a screenshots folder: `~/Pictures/kvm-manager-test/`

---

## Test Suite 1: Basic Connection

### Test 1.1: Open Console Window
**Steps**:
1. Launch KVM Manager
2. Navigate to VM List page
3. Find a **running** VM with VNC graphics
4. Click "Console" button in VM actions

**Expected Result**:
- ✅ New console window opens
- ✅ Window title shows VM name
- ✅ Status bar shows "Connecting..."
- ✅ Connection establishes within 2-3 seconds
- ✅ VM screen visible and responsive

**Failure Scenarios**:
- ❌ Window doesn't open → Check Tauri window permissions
- ❌ "Connecting..." hangs → Check VM is running, VNC enabled
- ❌ Black screen → Check VNC port/socket configuration

---

### Test 1.2: Mouse and Keyboard Input
**Steps**:
1. With console connected, move mouse
2. Click on VM screen
3. Type text into VM (e.g., in terminal or notepad)
4. Try keyboard shortcuts (Ctrl+C, Ctrl+V)

**Expected Result**:
- ✅ Mouse cursor moves smoothly
- ✅ Clicks register correctly
- ✅ Typing appears in VM
- ✅ Basic shortcuts work

**Known Issues**:
- Mouse may need to be clicked once to "capture"
- Some keyboard layouts may differ between host/guest

---

### Test 1.3: Connection Status Indicators
**Steps**:
1. Observe status bar during connection
2. Note connection state changes

**Expected Result**:
- ✅ Shows "Connecting..." initially
- ✅ Changes to "Connected" when established
- ✅ Shows connection time (e.g., "Connected • 00:45")
- ✅ Status dot is green when connected

---

## Test Suite 2: Reconnection Logic

### Test 2.1: Automatic Reconnection (VM Paused)
**Steps**:
1. Connect to a running VM
2. Pause the VM from VM List (Tauri window) or virsh:
   ```bash
   virsh suspend <vm-name>
   ```
3. Observe console window behavior
4. Resume the VM:
   ```bash
   virsh resume <vm-name>
   ```

**Expected Result**:
- ✅ Status changes to "Disconnected"
- ✅ "Attempting to reconnect..." message appears
- ✅ Reconnection attempts visible (Attempt 1/5, 2/5, etc.)
- ✅ After VM resumes, reconnects automatically
- ✅ Exponential backoff delays: 1s, 2s, 4s, 8s, 16s

**Timing Verification**:
- Use stopwatch to verify delays increase exponentially
- Should NOT retry immediately

---

### Test 2.2: Manual Reconnection
**Steps**:
1. Connect to VM
2. Pause/stop VM to force disconnect
3. Click "Reconnect" button in toolbar (circular arrow)
4. Resume VM
5. Observe reconnection

**Expected Result**:
- ✅ Manual reconnect resets retry counter
- ✅ Reconnect button disabled during connection attempt
- ✅ Button re-enables after connection or failure
- ✅ Toast notification on successful reconnect

---

### Test 2.3: Max Reconnection Attempts
**Steps**:
1. Connect to VM
2. Stop the VM (don't resume):
   ```bash
   virsh destroy <vm-name>
   ```
3. Wait for all 5 reconnection attempts

**Expected Result**:
- ✅ Attempts 1-5 shown in status
- ✅ After 5 attempts, status shows "Disconnected"
- ✅ Error message: "Failed to reconnect after 5 attempts"
- ✅ Manual reconnect button still available
- ✅ Doesn't retry indefinitely

---

## Test Suite 3: Display Scale Modes

### Test 3.1: Scale to Window (Default)
**Steps**:
1. Connect to VM
2. In toolbar, click "View" dropdown
3. Select "Scale to Window" (should be checked by default)
4. Resize console window (drag edges)

**Expected Result**:
- ✅ VM display scales proportionally
- ✅ Aspect ratio maintained
- ✅ No scrollbars appear
- ✅ Letterboxing/pillarboxing if aspect ratios differ
- ✅ Toast notification: "Scale mode: Scale to Window"

---

### Test 3.2: 1:1 Pixel Mapping
**Steps**:
1. Change scale mode to "1:1 Pixel Mapping"
2. Resize console window to smaller than VM resolution

**Expected Result**:
- ✅ VM display shown at actual pixel size
- ✅ Scrollbars appear if VM larger than window
- ✅ Can scroll to see off-screen parts
- ✅ No scaling/blurriness
- ✅ Status bar shows "1:1 Pixels"

---

### Test 3.3: Stretch to Fill
**Steps**:
1. Change scale mode to "Stretch to Fill"
2. Resize window to different aspect ratios (wide, tall, square)

**Expected Result**:
- ✅ VM display stretches to fill entire window
- ✅ No letterboxing/pillarboxing
- ✅ Aspect ratio NOT maintained (distortion expected)
- ✅ No scrollbars
- ✅ Status bar shows "Stretch"

---

### Test 3.4: Scale Mode Persistence
**Steps**:
1. Set scale mode to "1:1 Pixel Mapping"
2. Close console window
3. Reopen console for same VM

**Expected Result**:
- ⚠️ Currently scale mode resets to "Scale to Window" (expected)
- 🔮 Future: Could persist per-VM in settings

---

## Test Suite 4: Send Keys Menu

### Test 4.1: Ctrl+Alt+Delete (Windows)
**Prerequisites**: Windows VM running, logged in or at login screen

**Steps**:
1. Connect to Windows VM
2. Click "Send Keys" in toolbar
3. Select "Ctrl+Alt+Delete"

**Expected Result**:
- ✅ Windows security screen appears (Task Manager, Lock, Sign out, etc.)
- ✅ Toast notification: "Sent Ctrl+Alt+Delete"
- ✅ Equivalent to pressing physical keys on Windows machine

---

### Test 4.2: Ctrl+Alt+Backspace (Linux)
**Prerequisites**: Linux VM (older distros with X11)

**Steps**:
1. Connect to Linux VM with X11 desktop
2. Send "Ctrl+Alt+Backspace" from menu

**Expected Result**:
- ✅ X server restarts (logs out user)
- ✅ Returns to login screen
- ⚠️ May not work on Wayland or newer systems

---

### Test 4.3: Ctrl+Alt+F1-F12 (Linux TTY Switch)
**Prerequisites**: Linux VM

**Steps**:
1. Linux VM running with GUI (usually on tty7 or tty1)
2. Send "Ctrl+Alt+F2" from menu
3. Observe switch to text-mode TTY2
4. Send "Ctrl+Alt+F7" to return to GUI

**Expected Result**:
- ✅ Switches to text console (tty2)
- ✅ Can see login prompt
- ✅ Ctrl+Alt+F7 returns to graphical session
- ✅ All F1-F12 keys available in menu

**Test Each F-Key**:
- [ ] F1 → tty1
- [ ] F2 → tty2
- [ ] F3 → tty3
- [ ] F4 → tty4
- [ ] F5 → tty5
- [ ] F6 → tty6
- [ ] F7 → tty7 (usually GUI on older systems)
- [ ] F8-F12 → tty8-12 (if configured)

---

### Test 4.4: Key Combination Rapid Fire
**Steps**:
1. Rapidly send multiple key combinations (click quickly)
2. Verify each is processed

**Expected Result**:
- ✅ All keys sent successfully
- ✅ No dropped keys
- ✅ Toast notifications for each (may stack)

---

## Test Suite 5: Screenshot Functionality

### Test 5.1: Basic Screenshot
**Steps**:
1. Connect to VM
2. Display something recognizable in VM (e.g., desktop background)
3. Click "Screenshot" button (camera icon) in toolbar
4. Note save location from toast

**Expected Result**:
- ✅ File save dialog appears
- ✅ Default filename: `kvm-manager-screenshot-<timestamp>.png`
- ✅ Can choose save location
- ✅ Toast shows: "Screenshot saved to <path>"
- ✅ Saved image matches VM screen

---

### Test 5.2: Screenshot During Different Scale Modes
**Steps**:
1. Take screenshot in "Scale to Window" mode
2. Change to "1:1 Pixel Mapping", take screenshot
3. Change to "Stretch to Fill", take screenshot
4. Compare screenshots

**Expected Result**:
- ✅ All screenshots capture at VM's native resolution
- ✅ Scale mode doesn't affect screenshot quality
- ✅ Screenshots should be identical (regardless of display mode)

---

### Test 5.3: Screenshot During Disconnection
**Steps**:
1. Disconnect VM (pause or stop)
2. Try to take screenshot

**Expected Result**:
- ✅ Screenshot button disabled when disconnected
- ⚠️ OR: Shows error toast "Cannot capture screenshot: not connected"

---

## Test Suite 6: Keyboard Shortcuts

### Test 6.1: Fullscreen (F11)
**Steps**:
1. Press F11 key
2. Press F11 again or Escape

**Expected Result**:
- ✅ F11 toggles fullscreen
- ✅ Toolbar visible in fullscreen (currently)
- ✅ ESC exits fullscreen
- ✅ No window decorations in fullscreen

**Future Enhancement**: Toolbar auto-hide in fullscreen

---

### Test 6.2: Screenshot Shortcut (Ctrl+S)
**Steps**:
1. Press Ctrl+S

**Expected Result**:
- ✅ Screenshot dialog opens
- ⚠️ OR: Not implemented yet (use toolbar button)

---

### Test 6.3: Reconnect Shortcut (Ctrl+R)
**Steps**:
1. While disconnected, press Ctrl+R

**Expected Result**:
- ✅ Triggers manual reconnection
- ⚠️ OR: Not implemented yet (use toolbar button)

---

## Test Suite 7: Error Handling

### Test 7.1: VM Not Running
**Steps**:
1. Stop a VM
2. Try to open console for stopped VM

**Expected Result**:
- ✅ Console window opens
- ✅ Shows error: "VM is not running"
- ✅ Helpful message: "Start the VM to connect"
- ❌ Should NOT show "Connecting..." indefinitely

---

### Test 7.2: VNC Not Enabled
**Steps**:
1. Configure VM without VNC graphics (use SPICE or none)
2. Try to open console

**Expected Result**:
- ✅ Error message: "VNC not enabled for this VM"
- ✅ Suggestion: "Configure graphics settings"
- ⚠️ OR: Currently may show generic connection error

---

### Test 7.3: Invalid VNC Port/Socket
**Steps**:
1. Manually edit VM XML to use invalid VNC port
2. Try to connect

**Expected Result**:
- ✅ Connection fails gracefully
- ✅ Error message visible
- ✅ Reconnect button available

---

### Test 7.4: Network Interruption
**Steps**:
1. Connect to VM on remote libvirt host (if applicable)
2. Simulate network interruption (disconnect WiFi, unplug ethernet)
3. Restore connection

**Expected Result**:
- ✅ Detects disconnection
- ✅ Attempts reconnection when network restored
- ✅ Reconnects successfully

---

## Test Suite 8: Multi-Window Behavior

### Test 8.1: Multiple Console Windows
**Steps**:
1. Open console for VM-1
2. Open console for VM-2 (separate window)
3. Interact with both

**Expected Result**:
- ✅ Both console windows work independently
- ✅ Each shows correct VM
- ✅ No interference between windows

---

### Test 8.2: Close Main Window While Console Open
**Steps**:
1. Open console window
2. Close main KVM Manager window

**Expected Result**:
- ✅ Console window remains open
- ✅ Console continues working
- ⚠️ OR: All windows close (depends on Tauri config)

---

### Test 8.3: Console Window Focus
**Steps**:
1. Open console
2. Switch to main window
3. Switch back to console

**Expected Result**:
- ✅ Keyboard input goes to correct window
- ✅ Mouse events work correctly
- ✅ No stuck keys

---

## Test Suite 9: Performance & Stability

### Test 9.1: Long-Running Connection
**Steps**:
1. Connect to VM
2. Leave console open for 10+ minutes
3. Interact periodically

**Expected Result**:
- ✅ Connection remains stable
- ✅ No memory leaks (check Task Manager)
- ✅ No performance degradation
- ✅ Responsive throughout

---

### Test 9.2: Rapid Window Resizing
**Steps**:
1. Connect to VM in "Scale to Window" mode
2. Rapidly resize window (drag edges quickly)

**Expected Result**:
- ✅ Scaling keeps up with resize
- ✅ No lag or freezing
- ✅ No visual artifacts
- ✅ Smooth scaling

---

### Test 9.3: Stress Test - Many Reconnections
**Steps**:
1. Connect to VM
2. Pause/resume VM 10 times quickly
3. Observe reconnection behavior

**Expected Result**:
- ✅ Handles rapid state changes
- ✅ No crashes
- ✅ Reconnect logic doesn't get confused
- ✅ Status accurate

---

## Test Suite 10: Different VM Types

### Test 10.1: Linux VMs
**Test with**:
- [ ] Ubuntu (latest)
- [ ] Fedora
- [ ] Debian
- [ ] Arch Linux

**Verify**:
- ✅ Console connects
- ✅ Display renders correctly
- ✅ TTY switching works (Ctrl+Alt+F-keys)
- ✅ Mouse/keyboard input accurate

---

### Test 10.2: Windows VMs
**Test with**:
- [ ] Windows 10
- [ ] Windows 11
- [ ] Windows Server 2019/2022

**Verify**:
- ✅ Console connects
- ✅ Ctrl+Alt+Delete works
- ✅ Display scaling correct
- ✅ Mouse/keyboard input accurate

---

### Test 10.3: Other OS
**Test with**:
- [ ] FreeBSD (if available)
- [ ] macOS (if available)

---

## Test Suite 11: Edge Cases

### Test 11.1: VM with No Display
**Steps**:
1. Create VM with no graphics device
2. Try to open console

**Expected Result**:
- ✅ Graceful error message
- ✅ Suggestion to add graphics device

---

### Test 11.2: VM with Multiple Displays
**Steps**:
1. Configure VM with 2+ virtual monitors (if supported)
2. Open console

**Expected Result**:
- ⚠️ Currently shows primary display only
- 🔮 Future: Display selector for multi-monitor

---

### Test 11.3: Very High Resolution VM
**Steps**:
1. Configure VM with 4K resolution (3840x2160)
2. Open console on 1080p display

**Expected Result**:
- ✅ "Scale to Window" scales down appropriately
- ✅ "1:1 Pixels" shows scrollbars
- ✅ Performance remains acceptable

---

## Test Suite 12: Accessibility

### Test 12.1: Keyboard Navigation
**Steps**:
1. Open console window
2. Navigate using Tab key
3. Activate controls with Space/Enter

**Expected Result**:
- ✅ Can focus toolbar buttons
- ✅ Can open dropdowns with keyboard
- ✅ Visible focus indicators

---

### Test 12.2: Screen Reader Compatibility
**Steps**:
1. Enable screen reader (NVDA on Windows, Orca on Linux)
2. Navigate console window

**Expected Result**:
- ✅ Button labels announced
- ✅ Status updates announced
- ⚠️ VM screen canvas not accessible (expected - it's a visual display)

---

## Results Summary Template

### Test Session Information
- **Date**: ___________
- **Tester**: ___________
- **Build Version**: ___________
- **OS**: Linux / Windows / macOS
- **Test VMs Used**:
  - VM 1: ___________
  - VM 2: ___________

### Pass/Fail Summary
| Test Suite | Total Tests | Passed | Failed | Skipped | Notes |
|------------|-------------|--------|--------|---------|-------|
| 1. Basic Connection | 3 | | | | |
| 2. Reconnection Logic | 3 | | | | |
| 3. Display Scale Modes | 4 | | | | |
| 4. Send Keys Menu | 4 | | | | |
| 5. Screenshot | 3 | | | | |
| 6. Keyboard Shortcuts | 3 | | | | |
| 7. Error Handling | 4 | | | | |
| 8. Multi-Window | 3 | | | | |
| 9. Performance | 3 | | | | |
| 10. Different VMs | 3 | | | | |
| 11. Edge Cases | 3 | | | | |
| 12. Accessibility | 2 | | | | |
| **TOTAL** | **38** | | | | |

### Critical Issues Found
1. ___________
2. ___________
3. ___________

### Minor Issues / Enhancements
1. ___________
2. ___________
3. ___________

### Recommendations
- [ ] Ready for production use
- [ ] Minor fixes needed
- [ ] Major issues require attention

---

## Automated Testing (Future)

While this guide covers manual testing, consider adding automated tests for:

1. **Unit Tests**:
   - VncViewer connection logic
   - Scale mode calculations
   - Reconnection backoff algorithm

2. **Integration Tests**:
   - Tauri command communication
   - VNC connection establishment
   - Screenshot file operations

3. **E2E Tests** (Playwright/Cypress):
   - Open console window
   - Verify toolbar interactions
   - Test scale mode switching

---

## Testing Tips

### For Linux VMs
- Use a VM with GUI desktop (GNOME, KDE) for easier testing
- TTY switching (Ctrl+Alt+F-keys) only works on real TTYs, not always in VMs
- Test both X11 and Wayland sessions

### For Windows VMs
- Ctrl+Alt+Delete is the primary test for Windows
- Test with both lock screen and desktop active
- Windows Server behavior may differ from desktop Windows

### For Reconnection Testing
```bash
# Useful virsh commands
virsh suspend <vm>      # Pause VM (triggers disconnect)
virsh resume <vm>       # Resume VM (triggers reconnect)
virsh destroy <vm>      # Force stop (hard disconnect)
virsh start <vm>        # Start stopped VM
```

### Performance Monitoring
```bash
# Check CPU/memory usage during testing
top -p $(pgrep kvm-manager)

# Monitor network traffic (if using remote libvirt)
iftop
```

---

## Sign-Off

### Tester Certification
- [ ] I have completed all applicable test suites
- [ ] All critical features work as expected
- [ ] Issues documented with reproduction steps
- [ ] Ready to proceed to production / next phase

**Signature**: ___________
**Date**: ___________

---

*This testing guide is version-controlled and should be updated as new features are added.*
