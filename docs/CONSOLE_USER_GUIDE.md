# KVM Manager Console User Guide

**Version**: 0.2.0
**Last Updated**: December 12, 2025
**Feature Status**: ✅ Fully Implemented (Week 5)

---

## Table of Contents

1. [Overview](#overview)
2. [Opening the Console](#opening-the-console)
3. [Console Window Layout](#console-window-layout)
4. [Basic Operations](#basic-operations)
5. [Display Modes](#display-modes)
6. [Send Special Keys](#send-special-keys)
7. [Screenshot Capture](#screenshot-capture)
8. [Connection Management](#connection-management)
9. [Keyboard Shortcuts](#keyboard-shortcuts)
10. [Troubleshooting](#troubleshooting)
11. [Tips & Best Practices](#tips--best-practices)

---

## Overview

The KVM Manager console provides a graphical interface to interact with your virtual machines, similar to connecting a monitor and keyboard directly to a physical computer. The console uses VNC (Virtual Network Computing) technology to display the VM's screen in real-time.

### Key Features
- 🖥️ **Real-time VM Display**: See exactly what's on the VM's screen
- 🖱️ **Full Input Control**: Mouse and keyboard input sent directly to VM
- 🔄 **Automatic Reconnection**: Stays connected even if VM restarts
- 📐 **Flexible Display Modes**: Scale, fit, or stretch to your window size
- ⌨️ **Special Key Combinations**: Send Ctrl+Alt+Del and function keys
- 📸 **Screenshot Capture**: Save snapshots of your VM's display
- 🎯 **Fullscreen Support**: Immersive full-screen viewing

---

## Opening the Console

### Method 1: From VM List
1. Launch KVM Manager
2. Navigate to **Virtual Machines** page (sidebar icon)
3. Find your VM in the list
4. Click the **"Console"** button in the VM's action menu

### Method 2: Quick Access (Future)
- Right-click on VM → "Open Console"
- Keyboard shortcut: Select VM, press `Ctrl+L`

### Requirements
✅ VM must be in **running** state
✅ VM must have **VNC graphics** configured (not SPICE)
✅ Libvirt connection must be active

---

## Console Window Layout

```
┌─────────────────────────────────────────────────────────────┐
│  [VM Name - Console]                                    [_][□][×] │
├─────────────────────────────────────────────────────────────┤
│  Toolbar                                                     │
│  [↻] [📷] [⌨️] [👁️] [🖥️]                                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│                                                              │
│                    VM Display Area                           │
│                   (VNC Viewer)                               │
│                                                              │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│  Status Bar                                                  │
│  ● Connected • 00:05:32 • Scale to Window • F11: Fullscreen│
└─────────────────────────────────────────────────────────────┘
```

### Toolbar Buttons (Left to Right)
1. **Reconnect** (↻): Manually reconnect to VM
2. **Screenshot** (📷): Capture current display
3. **Send Keys** (⌨️): Send special key combinations
4. **View** (👁️): Display scale mode selector
5. **Fullscreen** (🖥️): Toggle fullscreen mode

### Status Bar Information
- **Connection Status**: Green dot = connected, Red dot = disconnected
- **Connection Time**: How long you've been connected
- **Display Mode**: Current scale mode
- **Shortcuts**: Quick reference for keyboard shortcuts

---

## Basic Operations

### Mouse Control
- **Move Mouse**: Your mouse cursor controls the VM's mouse
- **Click**: Click anywhere in the display area to "capture" mouse
- **Right-Click**: Works as expected in the VM
- **Scroll**: Mouse wheel scrolls in the VM (if supported)

### Keyboard Input
- **Normal Typing**: All regular keys sent to VM
- **Modifier Keys**: Ctrl, Alt, Shift work normally
- **Special Keys**: Use "Send Keys" menu for Ctrl+Alt combinations

⚠️ **Note**: Some host shortcuts (like Alt+Tab) may be captured by your operating system instead of the VM. Use the "Send Keys" menu for these combinations.

---

## Display Modes

The console offers three display modes to fit different needs. Access via the **View** dropdown in the toolbar.

### 1. Scale to Window (Default) 📐
```
Best for: General use, varying window sizes
```
- VM display scales proportionally to fit window
- Maintains original aspect ratio
- No scrollbars or distortion
- Letterboxing (black bars) if aspect ratios differ

**When to Use**:
- Daily VM management
- Resizing window frequently
- Viewing VMs on different displays

---

### 2. 1:1 Pixel Mapping 🔍
```
Best for: Precise work, detailed viewing
```
- VM display shown at actual pixel resolution
- No scaling or blurriness
- Scrollbars appear if VM resolution > window size
- Perfect for seeing exact details

**When to Use**:
- Screenshot verification
- Precise graphical work
- Reading small text
- Comparing exact pixels

**Example**:
- VM resolution: 1920x1080
- Window size: 1280x720
- Result: Scrollbars let you pan around full 1920x1080

---

### 3. Stretch to Fill 📏
```
Best for: Maximizing screen space
```
- VM display stretches to fill entire window
- Does NOT maintain aspect ratio
- No scrollbars or letterboxing
- May appear distorted

**When to Use**:
- Ultra-wide monitors
- Maximizing visible area
- Don't care about aspect ratio distortion

**Warning**: This mode will stretch the image, which may make circles appear as ovals, text may look odd.

---

### Changing Display Modes

1. Click **"View"** dropdown in toolbar
2. Select desired mode (radio button shows current)
3. Display updates immediately
4. Toast notification confirms change

**Current Mode Indicator**: Check status bar bottom-left for current mode.

---

## Send Special Keys

Some key combinations (like Ctrl+Alt+Delete) can't be sent normally because your operating system intercepts them. Use the **"Send Keys"** menu to send these to your VM.

### Available Key Combinations

#### For Windows VMs
| Key Combination | Purpose | Common Use |
|----------------|---------|------------|
| **Ctrl+Alt+Delete** | Windows Security Screen | Task Manager, Lock, Sign Out, Change Password |

#### For Linux VMs
| Key Combination | Purpose | Common Use |
|----------------|---------|------------|
| **Ctrl+Alt+Backspace** | Restart X Server | Force logout from graphical session (older systems) |
| **Ctrl+Alt+F1** | Switch to TTY1 | Text console 1 |
| **Ctrl+Alt+F2** | Switch to TTY2 | Text console 2 |
| **Ctrl+Alt+F3** | Switch to TTY3 | Text console 3 |
| **Ctrl+Alt+F4** | Switch to TTY4 | Text console 4 |
| **Ctrl+Alt+F5** | Switch to TTY5 | Text console 5 |
| **Ctrl+Alt+F6** | Switch to TTY6 | Text console 6 |
| **Ctrl+Alt+F7** | Switch to TTY7 | Graphical session (common) |
| **Ctrl+Alt+F8-F12** | Switch to TTY8-12 | Additional consoles |

### How to Send Keys

1. Click **"Send Keys" (⌨️)** in toolbar
2. Browse the menu
3. Click desired key combination
4. Toast notification confirms key sent
5. VM receives the keys immediately

### Practical Examples

**Windows: Open Task Manager**
1. Send Keys → Ctrl+Alt+Delete
2. In Windows, click "Task Manager"

**Linux: Switch to Text Console**
1. Send Keys → Ctrl+Alt+F2
2. See text-mode login prompt
3. Login and use terminal
4. Send Keys → Ctrl+Alt+F7 to return to GUI

**Linux: Force X Server Restart** (Older systems)
1. Send Keys → Ctrl+Alt+Backspace
2. X server restarts (logs you out)
3. Return to login screen

---

## Screenshot Capture

Save a PNG image of your VM's current display.

### How to Take Screenshot

1. **Click Screenshot Button** (📷) in toolbar
2. **File dialog appears**
   - Default name: `kvm-manager-screenshot-<timestamp>.png`
   - Default location: Your Pictures folder
3. **Choose location** and filename
4. **Click "Save"**
5. **Toast notification** confirms save location

### Screenshot Details
- **Format**: PNG (lossless, high quality)
- **Resolution**: Captures at VM's native resolution (not scaled)
- **What's Captured**: Only VM display (not toolbar/status bar)
- **File Size**: Typically 100KB - 2MB depending on content

### Example Filename
```
kvm-manager-screenshot-2025-12-12-143052.png
               │         │        │
               │         │        └─ Time: 14:30:52
               │         └────────── Date: Dec 12, 2025
               └──────────────────── Prefix
```

### Use Cases
- 📚 Documentation
- 🐛 Bug reporting
- 🎓 Tutorials
- 💾 Record VM state
- ✅ Verification

---

## Connection Management

The console automatically manages your connection to the VM, including handling disconnections gracefully.

### Connection States

#### 🟢 Connected
```
Status: ● Connected • 00:05:32
```
- Active VNC connection
- VM display updating in real-time
- Mouse and keyboard input working
- All toolbar features available

#### 🟡 Connecting...
```
Status: ⏳ Connecting...
```
- Initial connection in progress
- Typically lasts 1-3 seconds
- Display area shows loading indicator

#### 🔴 Disconnected
```
Status: ● Disconnected
```
- VNC connection lost
- VM may be paused, stopped, or network issue
- Display frozen on last frame
- Automatic reconnection in progress (if enabled)

#### 🔄 Reconnecting...
```
Status: ⏳ Attempting to reconnect... (Attempt 2/5)
```
- Automatic reconnection in progress
- Shows attempt number and remaining tries
- Exponential backoff delays between attempts

---

### Automatic Reconnection

If the connection drops (VM paused, network hiccup, VNC restart), the console **automatically tries to reconnect**.

**Reconnection Strategy**: Exponential Backoff
- **Attempt 1**: Wait 1 second
- **Attempt 2**: Wait 2 seconds
- **Attempt 3**: Wait 4 seconds
- **Attempt 4**: Wait 8 seconds
- **Attempt 5**: Wait 16 seconds

**Total Duration**: ~31 seconds for all 5 attempts

**Why Exponential Backoff?**
- Gives VM time to fully restart
- Doesn't overwhelm system with rapid retries
- Standard practice for network reconnections

### Manual Reconnection

If automatic reconnection fails or you want to reconnect immediately:

1. Click **"Reconnect" (↻)** button in toolbar
2. Connection attempt starts immediately
3. Resets retry counter to 1/5
4. Toast notification on success

**When to Use Manual Reconnect**:
- Automatic reconnection gave up after 5 tries
- You just resumed a paused VM
- You know VM is ready but not connected

---

### Common Connection Scenarios

#### Scenario 1: VM Paused
```
You pause the VM → Connection drops → Status: "Disconnected"
Auto-reconnect attempts every 1s, 2s, 4s, 8s, 16s
You resume VM → Connection restores → Status: "Connected"
```

#### Scenario 2: VM Stopped
```
VM stops/crashes → Connection drops → Status: "Disconnected"
Auto-reconnect attempts 5 times → All fail → Status: "Failed to reconnect"
Manual action: Start VM, then click "Reconnect" button
```

#### Scenario 3: Network Hiccup
```
Brief network interruption → Connection drops
Auto-reconnect on Attempt 1 or 2 → Status: "Connected"
```

---

## Keyboard Shortcuts

### Global Shortcuts (Console Window Focused)

| Shortcut | Action | Notes |
|----------|--------|-------|
| **F11** | Toggle Fullscreen | Also in toolbar |
| **Escape** | Exit Fullscreen | Only when in fullscreen |
| **Ctrl+S** | Screenshot (Future) | Currently use toolbar button |
| **Ctrl+R** | Reconnect (Future) | Currently use toolbar button |

### VM-Specific Shortcuts

These keys are sent directly to the VM:
- **Ctrl+C**: Copy (in VM)
- **Ctrl+V**: Paste (in VM)
- **Ctrl+Alt**: Used for Linux actions (but capture by OS, use Send Keys menu)
- **Alt+Tab**: Switch windows (in VM - may be captured by host OS)

---

## Troubleshooting

### Problem: Console window opens but shows "Connecting..." forever

**Possible Causes**:
1. VM is not running
2. VNC graphics not enabled on VM
3. Firewall blocking VNC port
4. Libvirt VNC socket not accessible

**Solutions**:
```bash
# Check if VM is running
virsh list --all

# Check VNC configuration
virsh dumpxml <vm-name> | grep -A 5 "<graphics"

# Should see:
# <graphics type='vnc' port='5900' autoport='yes' listen='127.0.0.1'>

# If not present, edit VM to add VNC graphics:
virsh edit <vm-name>
# Add:
# <graphics type='vnc' autoport='yes' listen='127.0.0.1'/>
```

---

### Problem: Keyboard input not working

**Possible Causes**:
1. Mouse not "captured" by console
2. VM has different keyboard layout
3. Focus not on console window

**Solutions**:
1. Click once in the display area to capture input
2. Check VM's keyboard layout settings
3. Ensure console window has focus (click title bar)

---

### Problem: Mouse not aligned / offset from cursor

**Possible Causes**:
1. VM using tablet input device (should be automatic)
2. Scale mode causing visual offset
3. VM mouse driver issue

**Solutions**:
1. Try switching to "1:1 Pixel Mapping" mode
2. Check VM XML for `<input type='tablet'/>` (recommended)
3. Restart VM
4. Install VirtIO drivers in guest OS

---

### Problem: "Failed to reconnect after 5 attempts"

**Possible Causes**:
1. VM stopped and not restarted
2. VNC service crashed in VM
3. Network/socket issue

**Solutions**:
1. Check if VM is running: `virsh list`
2. If stopped, start VM: `virsh start <vm-name>`
3. Click "Reconnect" button manually
4. Check libvirt logs: `journalctl -u libvirtd`

---

### Problem: Screenshot button disabled / not working

**Possible Causes**:
1. Not connected to VM
2. VNC connection establishing

**Solutions**:
1. Wait for "Connected" status
2. Reconnect if disconnected
3. Screenshot only works when connected

---

### Problem: Display looks blurry or pixelated

**Possible Causes**:
1. Scaling artifacts in "Scale to Window" mode
2. Low VM resolution
3. Window size mismatch

**Solutions**:
1. Try "1:1 Pixel Mapping" mode for sharp display
2. Increase VM display resolution in guest OS
3. Match window size to VM resolution
4. Consider using higher quality VNC settings (future feature)

---

### Problem: Fullscreen mode shows black bars

**Possible Causes**:
1. VM aspect ratio ≠ monitor aspect ratio
2. Using "Scale to Window" mode (maintains aspect)

**Solutions**:
1. Switch to "Stretch to Fill" mode (may distort)
2. Adjust VM resolution to match monitor
3. Black bars are normal with "Scale to Window" to prevent distortion

---

## Tips & Best Practices

### General Usage

✅ **Use "Scale to Window" for daily use**
Most versatile mode, adapts to window size.

✅ **Click display to capture input**
First click "captures" mouse/keyboard for VM.

✅ **Use Send Keys menu for special keys**
Don't try to press Ctrl+Alt+Del directly.

✅ **Let auto-reconnect do its job**
Wait 30 seconds before manual reconnect.

✅ **Take screenshots for documentation**
Easier than alt-tabbing and using external tools.

---

### For Linux VMs

🐧 **Know your TTYs**:
- **F1-F6**: Text consoles (login prompts)
- **F7 or F1**: Graphical session (depends on distro)
- To return to GUI: Try F7 first, then F1

🐧 **Ctrl+Alt+Backspace may not work on Wayland**
This is an X11-specific feature, disabled on newer systems.

🐧 **Test in Ubuntu/Fedora first**
Mainstream distros have best VNC support.

---

### For Windows VMs

🪟 **Ctrl+Alt+Delete is essential**
Access Task Manager, lock screen, sign out.

🪟 **Install VirtIO drivers**
Improves mouse tracking and graphics performance.

🪟 **Use full-screen for immersive experience**
Makes it feel like a local Windows machine.

---

### Performance Optimization

⚡ **Use lower VM resolution for remote connections**
Less bandwidth needed (e.g., 1280x720 instead of 1920x1080).

⚡ **Close console window when not in use**
Frees up resources.

⚡ **Disable desktop effects in guest OS**
Improves VNC responsiveness (e.g., turn off transparency).

⚡ **Consider SPICE for better performance** (Future)
Currently VNC-only, SPICE support planned.

---

### Security

🔒 **Libvirt VNC binds to localhost by default**
VNC traffic stays on local machine, not exposed to network.

🔒 **For remote access, use SSH tunnel**
Don't expose VNC port directly to internet:
```bash
ssh -L 5900:localhost:5900 remote-host
```

🔒 **Set VNC password in production** (Future feature)
Adds authentication layer.

---

## Advanced Features (Coming Soon)

### Planned Enhancements

🔮 **Clipboard Sharing**
Copy/paste between host and guest.

🔮 **SPICE Console Support**
Better performance and more features than VNC.

🔮 **Auto-resize Guest Display**
Guest resolution matches window size automatically (requires guest agent).

🔮 **Multi-monitor Support**
View and switch between multiple VM displays.

🔮 **Quality/Compression Settings**
Adjust VNC quality for performance vs clarity.

🔮 **Toolbar Auto-hide in Fullscreen**
Toolbar hides when not moving mouse.

🔮 **USB Device Passthrough UI**
Attach USB devices to VM from console.

---

## Frequently Asked Questions (FAQ)

### Q: Why does my console show a black screen?

**A**: Most likely:
1. VM is not running (start it first)
2. VM hasn't booted to graphical display yet (wait ~30 seconds)
3. VNC not enabled on VM (check graphics settings)

---

### Q: Can I use the console with multiple VMs at once?

**A**: Yes! Each console opens in a separate window. Open as many as you need.

---

### Q: Does the console work with VMs on remote libvirt hosts?

**A**: Yes, as long as you're connected to that libvirt host in KVM Manager. The VNC connection tunnels through libvirt.

---

### Q: Why is there lag in the console?

**A**: VNC can have latency, especially:
- Remote libvirt hosts
- High resolution VMs
- Heavy graphics in guest OS
- Network congestion

**Solutions**: Lower VM resolution, disable desktop effects, use wired connection.

---

### Q: Can I record video of my VM console?

**A**: Not built-in currently. Workaround: Use external screen recording tools to record the console window.

---

### Q: Does KVM Manager support SPICE consoles?

**A**: Not yet. VNC is supported in v0.2.0. SPICE support is planned for future releases.

---

### Q: What happens if I close the console while a VM is running?

**A**: Closing the console window does NOT affect the VM. The VM continues running. You can reopen the console anytime.

---

### Q: Can I open console for a stopped VM?

**A**: You can open the window, but it will show "Disconnected" status. Start the VM, then it will connect automatically.

---

## Version History

### v0.2.0 (December 2025) - Current
- ✅ Full VNC console implementation
- ✅ Automatic reconnection with exponential backoff
- ✅ Three display scale modes
- ✅ Complete send keys menu (Ctrl+Alt+Del, F1-F12)
- ✅ Screenshot capture
- ✅ Fullscreen support
- ✅ Connection status indicators

### v0.1.0 (November 2025)
- Basic console window structure
- Placeholder console view

---

## Getting Help

### Support Resources

📖 **Documentation**: [PROJECT_PLAN.md](../PROJECT_PLAN.md)
🐛 **Report Bugs**: GitHub Issues
💬 **Community**: Discussions tab
📧 **Contact**: See README.md

### Before Reporting Issues

Please include:
1. **KVM Manager version**: Check About dialog
2. **Guest OS**: Linux (distro/version) or Windows (version)
3. **Reproduction steps**: What you did before the issue
4. **Expected behavior**: What should happen
5. **Actual behavior**: What actually happened
6. **Screenshots**: If visual issue
7. **Logs**: Check developer console (F12)

---

*This guide covers the console features implemented in Week 5 (December 2025). For general KVM Manager usage, see the main [README.md](../README.md).*

**Last Updated**: December 12, 2025
**Document Version**: 1.0
**Applies to**: KVM Manager v0.2.0+
