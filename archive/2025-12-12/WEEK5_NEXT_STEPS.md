# Week 5 - Next Steps & Priorities

**Current Status**: Day 1 Complete - 75% of Week 5 goals achieved
**Last Updated**: December 12, 2025

---

## 🎯 Immediate Next Steps (Day 2)

### High Priority - Console Reconnection Logic
**Why**: Improve reliability when VNC connection drops

**Tasks**:
1. Add reconnection state to VncViewer component
2. Implement exponential backoff algorithm
3. Add "Reconnect" button in disconnected state
4. Test connection drops and recovery
5. Add reconnection attempt counter

**Expected Time**: 2-3 hours

**Files to Modify**:
- `src/components/console/VncViewer.tsx`
- `src/pages/ConsoleWindow.tsx`

---

### Medium Priority - Scale/Fit Options
**Why**: Better viewing experience for different VM resolutions

**Tasks**:
1. Add scale mode state to ConsoleWindow
2. Create scale mode dropdown in ConsoleToolbar
3. Implement three modes:
   - **Scale to Window** (default): RFB.scaleViewport = true
   - **1:1 Pixel Mapping**: RFB.scaleViewport = false
   - **Stretch to Fill**: Custom CSS transform
4. Persist scale preference per VM

**Expected Time**: 2-3 hours

**Files to Modify**:
- `src/components/console/ConsoleToolbar.tsx`
- `src/components/console/VncViewer.tsx`
- `src/pages/ConsoleWindow.tsx`

**Implementation Notes**:
```typescript
// In VncViewer
const setScaleMode = (mode: 'scale' | 'fit' | 'stretch') => {
  if (rfbRef.current) {
    switch (mode) {
      case 'scale':
        rfbRef.current.scaleViewport = true
        break
      case 'fit':
        rfbRef.current.scaleViewport = false
        break
      case 'stretch':
        // Use CSS transform: scale()
        break
    }
  }
}
```

---

### Medium Priority - Complete Send Keys Menu
**Why**: Users need to send special key combinations to VMs

**Tasks**:
1. Implement Ctrl+Alt+Backspace handler
2. Implement Ctrl+Alt+F1 through F12 handlers
3. Add keysym constants for all function keys
4. Test with Linux VMs (TTY switching)
5. Add tooltips explaining what each key combo does

**Expected Time**: 1-2 hours

**Files to Modify**:
- `src/components/console/VncViewer.tsx` (add sendKeys helper)
- `src/components/console/ConsoleToolbar.tsx` (complete menu)

**Keysym Reference**:
```typescript
// noVNC keysym values
const KEYSYMS = {
  XK_F1: 0xffbe,
  XK_F2: 0xffbf,
  XK_F3: 0xffc0,
  // ... F4-F12
  XK_BackSpace: 0xff08,
}

function sendCtrlAltFn(fnNum: number) {
  // Send Ctrl+Alt+Fn key combination
  rfb.sendKey(0xffe3, 'ControlLeft', true)   // Ctrl down
  rfb.sendKey(0xffe9, 'AltLeft', true)       // Alt down
  rfb.sendKey(KEYSYMS[`XK_F${fnNum}`], `F${fnNum}`, true)  // Fn down
  // Release in reverse order
  rfb.sendKey(KEYSYMS[`XK_F${fnNum}`], `F${fnNum}`, false)
  rfb.sendKey(0xffe9, 'AltLeft', false)
  rfb.sendKey(0xffe3, 'ControlLeft', false)
}
```

---

### Low Priority - Toolbar Auto-Hide
**Why**: Cleaner fullscreen experience

**Tasks**:
1. Add toolbar visibility state
2. Implement mouse move detector in fullscreen
3. Show toolbar when mouse moves to top
4. Hide toolbar after 3 seconds of inactivity
5. Always show toolbar in windowed mode

**Expected Time**: 2 hours

**Files to Modify**:
- `src/components/console/ConsoleToolbar.tsx`
- `src/pages/ConsoleWindow.tsx`

---

## 🧪 Testing Priorities (Day 3)

### Critical - Manual Testing with Real VMs
**Why**: Verify everything works in real-world scenarios

**Test Scenarios**:

#### Snapshot Testing
1. **Create disk snapshot** (VM stopped)
   - Stop a VM
   - Create snapshot without memory
   - Verify snapshot appears in list

2. **Create memory snapshot** (VM running)
   - Start a VM
   - Make changes (create a file, open apps)
   - Create snapshot with memory
   - Verify snapshot appears with "running" state

3. **Revert snapshot**
   - Make more changes to VM
   - Revert to previous snapshot
   - Verify changes are gone
   - Verify VM state matches snapshot

4. **Delete snapshot**
   - Delete oldest snapshot
   - Verify snapshot removed from list
   - Verify libvirt shows snapshot deleted

#### Console Testing
1. **VNC Connection**
   - Start VM with VNC graphics
   - Open console window
   - Verify connection establishes
   - Verify mouse and keyboard input works

2. **Fullscreen**
   - Press F11
   - Verify fullscreen works
   - Verify toolbar visible
   - Press Escape
   - Verify exits fullscreen

3. **Screenshot**
   - Press F10 in console
   - Choose save location
   - Verify PNG file saved
   - Verify image shows current VM screen

4. **Send Keys**
   - Click "Send Ctrl+Alt+Delete"
   - On Windows VM: verify shows security screen
   - On Linux VM: verify triggers reboot prompt

5. **Connection Loss Recovery**
   - Stop VM while console open
   - Verify shows "disconnected" state
   - Start VM again
   - Test reconnection (manual for now)

---

## 📚 Documentation Tasks (Day 3-4)

### User Documentation
**File**: `docs/USER_GUIDE.md` (to be created)

**Sections to Write**:
1. **Snapshot Management**
   - How to create snapshots
   - Disk vs. memory snapshots
   - When to use snapshots
   - How to revert
   - Best practices

2. **Console Usage**
   - Opening VM console
   - Fullscreen mode
   - Taking screenshots
   - Sending special keys
   - Troubleshooting connection issues

### Developer Documentation
**File**: Update `PROJECT_PLAN.md`

**Updates Needed**:
- Mark Week 5 progress
- Update milestone status
- Document noVNC CDN decision
- Update architecture diagrams

---

## 🚀 Optional Enhancements (If Time Permits)

### Console Quality Settings
**Why**: Users may want to adjust quality vs. performance

**Implementation**:
1. Add settings dropdown in toolbar
2. Quality slider (0-9)
3. Compression slider (0-9)
4. Apply settings to RFB instance
5. Persist settings per user preference

**Expected Time**: 2-3 hours

### Console Performance Monitoring
**Why**: Debugging connection issues

**Implementation**:
1. Track FPS (frames per second)
2. Track latency (round-trip time)
3. Display in status bar (optional)
4. Log to console for debugging

**Expected Time**: 2 hours

### Snapshot Tree Visualization
**Why**: Better understanding of snapshot relationships

**Implementation**:
1. Install tree visualization library (e.g., react-tree-graph)
2. Parse snapshot parent relationships
3. Render tree view
4. Add toggle between list and tree view

**Expected Time**: 4-5 hours
**Note**: This is a nice-to-have, not critical for MVP

---

## 🎯 Success Criteria for Week 5

### Must Have (Critical)
- [x] Snapshot CRUD operations working ✅
- [x] VNC viewer functioning ✅
- [x] Build passing ✅
- [ ] Reconnection logic working
- [ ] Manual testing complete
- [ ] Basic documentation

### Should Have (Important)
- [x] Screenshot functionality ✅
- [ ] Scale/fit options
- [ ] Complete send keys menu
- [ ] User guide for snapshots and console

### Nice to Have (Optional)
- [ ] Toolbar auto-hide
- [ ] Quality settings UI
- [ ] Performance monitoring
- [ ] Snapshot tree visualization

---

## 📊 Time Estimates

| Task | Priority | Time | Status |
|------|----------|------|--------|
| Reconnection Logic | High | 2-3h | Not started |
| Scale/Fit Options | Medium | 2-3h | Not started |
| Complete Send Keys | Medium | 1-2h | Not started |
| Manual Testing | High | 2-3h | Not started |
| User Documentation | Medium | 2-3h | Not started |
| Toolbar Auto-Hide | Low | 2h | Not started |
| Quality Settings | Optional | 2-3h | Not started |

**Total Estimated Time Remaining**: 11-16 hours
**Days Remaining in Week 5**: 4 days (Day 2-5)
**Comfortable Pace**: 3-4 hours per day

---

## 🔄 Daily Workflow Recommendation

### Day 2 (Friday)
- Morning: Implement reconnection logic
- Afternoon: Implement scale/fit options
- End of day: Test with real VMs

### Day 3 (Monday)
- Morning: Complete send keys menu
- Afternoon: Manual testing session
- End of day: Document bugs/issues found

### Day 4 (Tuesday)
- Morning: Fix bugs from testing
- Afternoon: Write user documentation
- End of day: Review and polish

### Day 5 (Wednesday)
- Morning: Optional enhancements
- Afternoon: Final testing
- End of day: Week 5 completion report

---

## 🎉 Celebration Milestones

- ✅ **Day 1**: Completed 75% of Week 5 goals! 🎊
- [ ] **Day 2**: Console polish complete
- [ ] **Day 3**: All testing passed
- [ ] **Day 4**: Documentation complete
- [ ] **Day 5**: Week 5 fully complete, ahead of schedule!

---

## 📝 Notes

### Build System
- noVNC is loaded from CDN (jsDelivr)
- If offline support needed, download and bundle locally
- Current solution is stable and maintainable

### Testing Strategy
- Manual testing is critical for console features
- Automated testing can be added in Week 6
- Focus on real-world usage scenarios

### Code Quality
- Maintain TypeScript strict mode
- Keep components small and focused
- Document complex logic
- Handle all error cases

---

**Remember**: Week 5 is already 75% complete after just Day 1. The remaining work is polish, testing, and documentation. We're in a great position! 🚀
