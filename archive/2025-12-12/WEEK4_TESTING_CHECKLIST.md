# Week 4 Testing Checklist

## 🔍 Manual Testing Guide

Use this checklist to verify all Week 4 features are working correctly.

---

## 1. Context Menus

### VM Card Context Menu
- [ ] Right-click on a stopped VM card
  - [ ] "Start VM" option appears
  - [ ] Click "Start VM" - VM starts successfully
- [ ] Right-click on a running VM card
  - [ ] "Stop VM" option appears
  - [ ] "Pause VM" option appears
  - [ ] "Open Console" option appears
  - [ ] Click "Stop VM" - VM stops successfully
- [ ] Right-click on any VM card
  - [ ] "Open Details" option (with Enter shortcut shown)
  - [ ] "Clone VM" option
  - [ ] "Delete VM" option (with Del shortcut shown)
  - [ ] Click "Open Details" - Details window opens

### Hardware Tree Context Menu
- [ ] Open VM Details window
- [ ] Right-click on a hardware item (e.g., CPU)
  - [ ] "Edit Device" option appears
  - [ ] "Remove Device" option appears
  - [ ] "Add Hardware" option appears
  - [ ] Click "Edit Device" - Device editor appears

---

## 2. Double-Click Behaviors

### VM Card Double-Click
- [ ] Double-click on a VM card
  - [ ] VM Details window opens
  - [ ] Window shows correct VM information

### Hardware Tree Double-Click
- [ ] Open VM Details window
- [ ] Double-click on a hardware item
  - [ ] Device editor appears for that item
  - [ ] Correct device is selected

---

## 3. Window State Persistence

### Basic Save/Restore
- [ ] Open VM Details window for any VM
- [ ] Move the window to a specific position
- [ ] Resize the window (make it wider/taller)
- [ ] Close the window
- [ ] Open VM Details again for the same VM
  - [ ] Window opens at the same position
  - [ ] Window has the same size

### Multiple Windows
- [ ] Open VM Details for VM #1, position it on left side
- [ ] Open VM Details for VM #2, position it on right side
- [ ] Close both windows
- [ ] Reopen both windows
  - [ ] Each window restores to its own position

### Console Window
- [ ] Open Console window for a running VM
- [ ] Move and resize it
- [ ] Close it
- [ ] Open Console again
  - [ ] Position and size are restored

### Reset Functionality
- [ ] Open Settings page (Ctrl+,)
- [ ] Scroll to "Window Management" section
- [ ] Click "Reset Windows" button
  - [ ] Success toast appears
- [ ] Open VM Details window
  - [ ] Window opens at default position (not saved position)

---

## 4. Keyboard Shortcuts

### Help Dialog
- [ ] Press `Ctrl+?` (or Ctrl+Shift+/)
  - [ ] Keyboard Shortcuts dialog opens
  - [ ] Dialog shows 20+ shortcuts
  - [ ] Shortcuts are organized by category
- [ ] Click Help button (? icon) in toolbar
  - [ ] Same dialog opens
- [ ] Press Escape or click outside
  - [ ] Dialog closes

### Toolbar Shortcuts
- [ ] Hover over "New VM" button
  - [ ] Tooltip shows "Create New Virtual Machine (Ctrl+N)"
- [ ] Hover over "Start" button
  - [ ] Tooltip shows "Start Virtual Machine (Ctrl+P)"
- [ ] Hover over "Stop" button
  - [ ] Tooltip shows "Stop Virtual Machine (Ctrl+S)"
- [ ] Hover over "Console" button
  - [ ] Tooltip shows "Open Console (Ctrl+O)"
- [ ] Hover over "Settings" button
  - [ ] Tooltip shows "Settings (Ctrl+,)"
- [ ] Hover over Help button
  - [ ] Tooltip shows "Keyboard Shortcuts (Ctrl+?)"

### Shortcut Execution
- [ ] Click on a VM to focus it
- [ ] Press `Ctrl+P`
  - [ ] VM starts (if stopped)
- [ ] Press `Ctrl+S`
  - [ ] VM stops (if running)
- [ ] Press `Ctrl+,`
  - [ ] Settings page opens
- [ ] Press `Ctrl+D`
  - [ ] VM Details window opens for focused VM
- [ ] Press `Ctrl+O`
  - [ ] Console window opens for focused VM (if running)

---

## 5. Visual Polish

### Spacing & Density
- [ ] Open Dashboard
  - [ ] Cards feel more compact (less padding)
  - [ ] Content doesn't feel cramped
- [ ] Open VM List
  - [ ] VM cards are smaller but still readable
  - [ ] More VMs visible on screen

### Typography
- [ ] Check various pages
  - [ ] Text is smaller but still crisp and readable
  - [ ] Font size feels appropriate for desktop
  - [ ] Hierarchy is clear (headings vs body text)

### Colors
- [ ] Check Light theme
  - [ ] Window background is light gray (#f5f5f5)
  - [ ] Panels are white
  - [ ] Borders are subtle gray
  - [ ] Colors feel professional, not bright
- [ ] Check Dark theme (if implemented)
  - [ ] Dark background (#1e1e1e)
  - [ ] Appropriate contrast

### Icons
- [ ] Check toolbar icons
  - [ ] All icons are consistent size
  - [ ] Icons are crisp and clear
- [ ] Check card icons
  - [ ] CPU, Memory, Disk icons are visible
  - [ ] State indicators (running/stopped) are clear

---

## 6. Edge Cases

### Off-Screen Windows
- [ ] Save a window position
- [ ] Edit the window state file to put window off-screen:
  - Location: `~/.config/kvm-manager/window_states.json`
  - Set x: -5000, y: -5000
- [ ] Open the window
  - [ ] Window opens at default position (not off-screen)

### Invalid Window State
- [ ] Edit window state file with invalid values:
  - Set width: 10, height: 10
- [ ] Open the window
  - [ ] Window opens at default size (not tiny)

### Rapid Window Movement
- [ ] Open VM Details window
- [ ] Move it rapidly around the screen
- [ ] Close immediately after moving
- [ ] Reopen
  - [ ] Position is saved (debouncing works)

---

## 7. Integration Testing

### Multi-Window Workflow
- [ ] Open VM List (main window)
- [ ] Open VM Details for VM #1
- [ ] Open Console for VM #1
- [ ] Position all 3 windows nicely
- [ ] Close all windows
- [ ] Reopen main window
- [ ] Reopen VM Details and Console
  - [ ] All windows restore to saved positions

### Context Menu + Keyboard Shortcuts
- [ ] Right-click VM card
- [ ] Note keyboard shortcuts shown in menu
- [ ] Close menu
- [ ] Use keyboard shortcut instead
  - [ ] Same action executes

---

## Bug Watch List

Look out for these potential issues:

- [ ] Context menu appears off-screen
- [ ] Window restoration fails silently
- [ ] Keyboard shortcuts conflict with system shortcuts
- [ ] Double-click too sensitive (single click triggers)
- [ ] Tooltips overlap menu items
- [ ] Window state file grows too large
- [ ] Memory leaks from event listeners

---

## Performance Checks

- [ ] Open/close multiple windows rapidly
  - [ ] No lag or stuttering
- [ ] Right-click and open/close context menus quickly
  - [ ] Responsive, no delays
- [ ] Move windows continuously
  - [ ] Smooth movement
  - [ ] Saves don't cause lag (debouncing works)

---

## Browser Console Checks

Open browser DevTools (F12) and check:
- [ ] No error messages in console
- [ ] Window state save/load messages appear (with console.log)
- [ ] No warnings about deprecated APIs
- [ ] No failed network requests

---

## Checklist Summary

**Context Menus**: ___ / 8 tests passed
**Double-Click**: ___ / 2 tests passed
**Window State**: ___ / 5 tests passed
**Keyboard Shortcuts**: ___ / 8 tests passed
**Visual Polish**: ___ / 4 tests passed
**Edge Cases**: ___ / 3 tests passed
**Integration**: ___ / 2 tests passed
**Performance**: ___ / 3 tests passed

**TOTAL**: ___ / 35 tests passed

---

## Test Results

### Date: _______________
### Tester: _______________

### Issues Found:
1.
2.
3.

### Notes:


---

## Sign-Off

- [ ] All critical features tested and working
- [ ] No blocking bugs found
- [ ] Visual appearance is professional
- [ ] Performance is acceptable
- [ ] Ready for merge to main branch

**Approved by**: _______________ **Date**: _______________
