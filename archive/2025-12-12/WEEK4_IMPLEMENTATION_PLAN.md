# Week 4: Desktop Polish & Refinement - Implementation Plan

**Date Started**: December 12, 2025
**Status**: 🚀 **IN PROGRESS**
**Phase**: Desktop UI Transformation - Final Polish

---

## Executive Summary

Week 4 focuses on transforming KVM Manager into a truly **desktop-native application** by implementing polish, refinements, and desktop conventions that users expect from professional desktop software. This builds on the solid foundation from Weeks 1-3.

### Previous Weeks Recap:
- ✅ **Week 1**: Menu bar + toolbar desktop-style layout
- ✅ **Week 2**: Multi-window support (VM details, console windows)
- ✅ **Week 3**: Hardware device tree sidebar and tabbed details panels

### Week 4 Goals:
Transform the UI from "functional" to "polished and professional" through:
1. **Visual Design** - Desktop-appropriate colors, spacing, typography
2. **Interactions** - Context menus, keyboard shortcuts, double-click behaviors
3. **Desktop Conventions** - Window state persistence, native behaviors
4. **Quality Assurance** - Comprehensive testing and bug fixes

---

## 📋 Implementation Checklist

### Day 1-2: Visual Design Updates

#### Color Palette Update
- [ ] Create new design token system in Tailwind config
- [ ] Implement desktop-native light theme colors
  - Muted backgrounds (#f5f5f5 window, #ffffff panels)
  - Professional borders (#d0d0d0 vs bright blues)
  - Subtle text hierarchy
- [ ] Implement desktop-native dark theme
  - Dark backgrounds (#1e1e1e, #252525)
  - Appropriate contrast for dark mode
- [ ] Update all components to use new color tokens
- [ ] Test light/dark theme switching

#### Spacing & Density
- [ ] Reduce padding in tables (py-3 → py-2)
- [ ] Tighten gaps between elements (gap-4 → gap-2)
- [ ] Make toolbar buttons more compact
- [ ] Reduce card padding for denser layout
- [ ] Update VM list card spacing
- [ ] Update hardware tree spacing
- [ ] Update details panel spacing

#### Typography
- [ ] Adjust base font size (14px → 13px)
- [ ] Reduce excessive bold text
- [ ] Standardize heading sizes
- [ ] Update toolbar button text size
- [ ] Update table header styling (uppercase, smaller)
- [ ] Ensure consistent line-height

#### Icons
- [ ] Audit all icons for consistency (use Lucide only)
- [ ] Reduce toolbar icon sizes (20px → 16px)
- [ ] Use monochrome icons in toolbars
- [ ] Keep colorful status icons in VM list
- [ ] Ensure icon alignment in all components

---

### Day 3: Right-Click Context Menus

#### Core Context Menu System
- [ ] Install/configure Radix UI Context Menu
- [ ] Create `ContextMenu` component wrapper
- [ ] Create `ContextMenuItem` with icon support
- [ ] Add keyboard shortcut display in menu items

#### VM List Context Menu
**Location**: VM cards in Dashboard
**Menu Items**:
- [ ] Start (Play icon)
- [ ] Stop (Square icon)
- [ ] Force Stop (XCircle icon)
- [ ] Pause (Pause icon)
- [ ] Resume (Play icon)
- [ ] --- Separator ---
- [ ] Open Console (Monitor icon) - Ctrl+Shift+C
- [ ] Open Details (Edit icon) - Enter
- [ ] --- Separator ---
- [ ] Clone (Copy icon)
- [ ] Delete (Trash icon) - Delete key
- [ ] --- Separator ---
- [ ] Properties (Info icon)

#### Hardware Tree Context Menu
**Location**: Hardware items in tree sidebar
**Menu Items** (per device type):
- [ ] Edit Device (Edit icon)
- [ ] Remove Device (Trash icon)
- [ ] --- Separator ---
- [ ] Add Hardware (Plus icon)

#### Toolbar Context Menu
**Location**: Main window toolbar
- [ ] Right-click on toolbar → Customize Toolbar (future)

---

### Day 4: Enhanced Keyboard Shortcuts

#### Existing Shortcuts (Verify & Document)
Current shortcuts in `useKeyboardShortcuts.ts`:
- [ ] Verify Ctrl+N → New VM
- [ ] Verify Delete → Delete VM
- [ ] Verify F5 → Refresh
- [ ] Verify Ctrl+S → Start VM
- [ ] Verify Ctrl+Q → Stop VM
- [ ] Verify Ctrl+P → Pause/Resume VM

#### New Shortcuts to Add
- [ ] **Ctrl+Shift+C** → Open Console
- [ ] **Enter** → Open VM Details (when VM selected)
- [ ] **Ctrl+W** → Close current window
- [ ] **Ctrl+,** → Open Settings
- [ ] **Ctrl+Shift+N** → New Virtual Network
- [ ] **Ctrl+Shift+S** → New Storage Pool
- [ ] **Escape** → Close dialogs/modals
- [ ] **Ctrl+F** → Focus search/filter (future)

#### Keyboard Shortcut Display
- [ ] Show shortcuts in tooltips (e.g., "Start VM (Ctrl+S)")
- [ ] Show shortcuts in context menus
- [ ] Create keyboard shortcuts help dialog (Ctrl+?)
- [ ] Add "View > Keyboard Shortcuts" menu item

---

### Day 5: Double-Click Behaviors

#### VM List Double-Click
- [ ] Double-click VM card → Open VM Details window
- [ ] Ensure doesn't conflict with button clicks inside card
- [ ] Add smooth transition/animation

#### Hardware Tree Double-Click
- [ ] Double-click device → Expand/collapse (for sections)
- [ ] Double-click leaf device → Open inline editor (future)
- [ ] Ensure visual feedback on double-click

#### Table Double-Click (Storage, Network)
- [ ] Double-click storage volume → Edit/Mount
- [ ] Double-click network → Edit network
- [ ] Double-click disk in VM → Edit disk settings

---

### Day 6: Window State Persistence

#### Window Position & Size
**Backend Commands** (Rust):
- [ ] Add `save_window_state` Tauri command
  - Parameters: label, x, y, width, height
  - Save to app config file
- [ ] Add `load_window_state` Tauri command
  - Returns saved position/size for window label
- [ ] Update `open_vm_details_window` to restore position
- [ ] Update `open_console_window` to restore position

**Frontend Implementation**:
- [ ] Call `save_window_state` on window move/resize
- [ ] Debounce save calls (don't save every pixel)
- [ ] Load window state on window open
- [ ] Handle multi-monitor scenarios
- [ ] Validate window position (ensure on-screen)

#### Open Windows Restoration
- [ ] Save list of open windows on app close
- [ ] Restore open windows on app launch
- [ ] Don't restore console windows (only details)
- [ ] Add preference: "Restore windows on startup"

#### Window Close Behavior
- [ ] Closing main window → Close all child windows
- [ ] Closing details window → Just close that window
- [ ] ESC key → Close current window (if not main)
- [ ] Ctrl+W → Close current window

---

### Day 7: Additional Desktop Conventions

#### Focus Management
- [ ] Proper focus on window open (details window)
- [ ] Tab navigation through form fields
- [ ] Focus first input in dialogs
- [ ] Restore focus after dialog close

#### Loading States
- [ ] Skeleton loaders for slow operations
- [ ] Progress indicators for long-running tasks
- [ ] Disable buttons during operations
- [ ] Show loading cursor when appropriate

#### Error Handling
- [ ] Consistent error toast styling
- [ ] Desktop-style error dialogs (for critical errors)
- [ ] Non-intrusive warnings (subtle toasts)
- [ ] Auto-dismiss success messages (3s)

#### Accessibility
- [ ] Ensure keyboard navigation works everywhere
- [ ] ARIA labels on icon-only buttons
- [ ] Focus visible styles
- [ ] Screen reader friendly

---

### Day 8: Testing & Bug Fixes

#### Visual Regression Testing
- [ ] Test light theme across all views
- [ ] Test dark theme across all views
- [ ] Verify consistent spacing
- [ ] Check icon sizes and alignment
- [ ] Verify typography consistency
- [ ] Test on different screen sizes

#### Interaction Testing
- [ ] Test all context menus
- [ ] Test all keyboard shortcuts
- [ ] Test double-click behaviors
- [ ] Test drag & drop (if implemented)
- [ ] Test window management

#### Cross-Component Testing
- [ ] Create VM → verify in all views
- [ ] Delete VM → verify windows close
- [ ] Start/stop VM → verify state updates everywhere
- [ ] Multi-window scenarios
- [ ] Rapid operation testing (stress test)

#### Performance Testing
- [ ] Test with 50+ VMs
- [ ] Window open/close performance
- [ ] Memory usage over time
- [ ] Check for memory leaks (long-running)

#### Bug Fixes
- [ ] Fix any issues found during testing
- [ ] Address any visual glitches
- [ ] Fix keyboard navigation issues
- [ ] Resolve window management bugs

---

## Implementation Strategy

### Phase 1: Visual Foundation (Days 1-2)
**Goal**: Establish the desktop-native visual design system

**Approach**:
1. Update Tailwind config with design tokens
2. Create theme switching utility
3. Systematically update components, starting with:
   - Layout components (AppLayout, Toolbar, MenuBar)
   - VM cards
   - Tables (storage, network)
   - Forms and inputs
4. Test theme switching

**Success Criteria**:
- App looks "desktop-native" not "web app"
- Colors are professional and muted
- Spacing is tighter and more efficient
- Typography is consistent and readable

---

### Phase 2: Interaction Layer (Days 3-5)
**Goal**: Add desktop-style interactions users expect

**Approach**:
1. **Day 3**: Context menus
   - Build reusable context menu system
   - Add to VM cards first (most important)
   - Add to hardware tree
   - Add to tables

2. **Day 4**: Keyboard shortcuts
   - Enhance existing shortcut system
   - Add new shortcuts
   - Display shortcuts in UI
   - Create shortcuts reference

3. **Day 5**: Double-click behaviors
   - VM list double-click
   - Hardware tree double-click
   - Table double-click

**Success Criteria**:
- Right-click works everywhere expected
- Keyboard shortcuts cover common operations
- Double-click provides quick access
- Interactions feel "snappy" and responsive

---

### Phase 3: Desktop Conventions (Day 6)
**Goal**: Implement desktop app behavior patterns

**Approach**:
1. Implement window state persistence (Rust + frontend)
2. Test window restoration across restarts
3. Handle edge cases (multi-monitor, off-screen)
4. Add user preferences for behavior

**Success Criteria**:
- Windows remember position/size
- App remembers which windows were open
- Window management feels natural
- No "web app" behaviors leak through

---

### Phase 4: Quality Assurance (Days 7-8)
**Goal**: Ensure everything works perfectly

**Approach**:
1. Systematic testing of all features
2. Visual consistency audit
3. Interaction testing
4. Performance testing
5. Bug fixing

**Success Criteria**:
- Zero visual inconsistencies
- All interactions work as expected
- Performance is excellent
- Ready for user testing

---

## Technical Implementation Details

### Design Token System

**File**: `tailwind.config.js`

```js
module.exports = {
  theme: {
    extend: {
      colors: {
        // Desktop-native palette
        window: {
          bg: 'var(--window-bg)',
          border: 'var(--window-border)',
        },
        panel: {
          bg: 'var(--panel-bg)',
          border: 'var(--panel-border)',
        },
        toolbar: {
          bg: 'var(--toolbar-bg)',
          border: 'var(--toolbar-border)',
        },
        sidebar: {
          bg: 'var(--sidebar-bg)',
          border: 'var(--sidebar-border)',
        },
        // ... more tokens
      },
      spacing: {
        // Desktop-appropriate spacing
        'toolbar': '0.5rem',  // 8px
        'panel': '1rem',      // 16px
        'tight': '0.5rem',    // 8px
      },
      fontSize: {
        'desktop-xs': '11px',
        'desktop-sm': '12px',
        'desktop-base': '13px',
        'desktop-lg': '14px',
      }
    }
  }
}
```

**File**: `src/styles/desktop-theme.css`

```css
:root {
  /* Light theme tokens */
  --window-bg: #f5f5f5;
  --panel-bg: #ffffff;
  --toolbar-bg: #fafafa;
  --sidebar-bg: #f0f0f0;

  --window-border: #d0d0d0;
  --panel-border: #e0e0e0;

  --text-primary: #2c2c2c;
  --text-secondary: #666666;
  --text-disabled: #999999;

  --accent-blue: #0066cc;
  --accent-green: #00aa00;
  --accent-orange: #cc6600;
  --accent-red: #cc0000;
}

:root[data-theme="dark"] {
  /* Dark theme tokens */
  --window-bg: #1e1e1e;
  --panel-bg: #252525;
  --toolbar-bg: #2d2d2d;
  --sidebar-bg: #2a2a2a;

  --window-border: #3a3a3a;
  --panel-border: #303030;

  --text-primary: #e0e0e0;
  --text-secondary: #a0a0a0;
  --text-disabled: #606060;

  --accent-blue: #4da3ff;
  --accent-green: #4dff4d;
  --accent-orange: #ffaa4d;
  --accent-red: #ff4d4d;
}
```

---

### Context Menu Component

**File**: `src/components/ui/ContextMenu.tsx`

```tsx
import * as ContextMenuPrimitive from '@radix-ui/react-context-menu';

interface MenuItem {
  label: string;
  icon?: React.ReactNode;
  shortcut?: string;
  action: () => void;
  disabled?: boolean;
  separator?: boolean;
}

export function ContextMenu({
  children,
  items
}: {
  children: React.ReactNode;
  items: MenuItem[];
}) {
  return (
    <ContextMenuPrimitive.Root>
      <ContextMenuPrimitive.Trigger asChild>
        {children}
      </ContextMenuPrimitive.Trigger>

      <ContextMenuPrimitive.Portal>
        <ContextMenuPrimitive.Content className="desktop-context-menu">
          {items.map((item, i) => (
            item.separator ? (
              <ContextMenuPrimitive.Separator key={i} />
            ) : (
              <ContextMenuPrimitive.Item
                key={i}
                onClick={item.action}
                disabled={item.disabled}
                className="desktop-context-menu-item"
              >
                {item.icon}
                <span>{item.label}</span>
                {item.shortcut && (
                  <span className="shortcut">{item.shortcut}</span>
                )}
              </ContextMenuPrimitive.Item>
            )
          ))}
        </ContextMenuPrimitive.Content>
      </ContextMenuPrimitive.Portal>
    </ContextMenuPrimitive.Root>
  );
}
```

---

### Window State Persistence (Backend)

**File**: `src-tauri/src/window_state.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[tauri::command]
pub async fn save_window_state(
    app: tauri::AppHandle,
    label: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let state = WindowState { x, y, width, height };

    // Save to app config
    let config_path = app.path_resolver()
        .app_config_dir()
        .ok_or("Failed to get config dir")?;

    // Load existing states
    let state_file = config_path.join("window_states.json");
    let mut states: HashMap<String, WindowState> = if state_file.exists() {
        let content = std::fs::read_to_string(&state_file)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Update state
    states.insert(label, state);

    // Save back
    std::fs::create_dir_all(&config_path).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&states)
        .map_err(|e| e.to_string())?;
    std::fs::write(&state_file, content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn load_window_state(
    app: tauri::AppHandle,
    label: String,
) -> Result<Option<WindowState>, String> {
    let config_path = app.path_resolver()
        .app_config_dir()
        .ok_or("Failed to get config dir")?;

    let state_file = config_path.join("window_states.json");
    if !state_file.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&state_file)
        .map_err(|e| e.to_string())?;
    let states: HashMap<String, WindowState> = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;

    Ok(states.get(&label).cloned())
}
```

---

## Testing Plan

### Manual Testing Checklist

#### Visual Testing
- [ ] Light theme: All views look consistent
- [ ] Dark theme: All views look consistent
- [ ] Theme switching: No visual glitches
- [ ] Spacing: Everything feels "desktop tight"
- [ ] Typography: Consistent and readable
- [ ] Icons: Properly sized and aligned
- [ ] Colors: Muted and professional

#### Interaction Testing
- [ ] Context menus: All items work
- [ ] Keyboard shortcuts: All shortcuts work
- [ ] Double-click: Opens expected windows
- [ ] Tab navigation: Focus moves correctly
- [ ] ESC key: Closes modals/dialogs
- [ ] Ctrl+W: Closes windows

#### Window Management Testing
- [ ] Window position saved on move
- [ ] Window size saved on resize
- [ ] Windows restore on app restart
- [ ] Off-screen windows handled correctly
- [ ] Multi-monitor: Windows appear on correct screen
- [ ] Closing main window closes all windows

#### Cross-Feature Testing
- [ ] Create VM: Context menu updates
- [ ] Delete VM: Windows close automatically
- [ ] Start VM: State updates in all views
- [ ] Stop VM: Console window behavior
- [ ] Clone VM: New window behavior

#### Performance Testing
- [ ] Window open time: < 200ms
- [ ] Context menu open time: Instant
- [ ] Theme switch time: < 100ms
- [ ] Memory usage: Stable over 1hr
- [ ] CPU usage: Low when idle

---

## Success Metrics

### Visual Quality
- ✅ App looks indistinguishable from native desktop apps
- ✅ No "web app" visual tells
- ✅ Professional color scheme
- ✅ Consistent spacing and alignment

### User Experience
- ✅ Right-click works everywhere expected
- ✅ Keyboard shortcuts cover 90%+ of operations
- ✅ Window management feels natural
- ✅ No unexpected behaviors

### Performance
- ✅ All interactions feel instant (< 200ms)
- ✅ No memory leaks
- ✅ Smooth animations and transitions
- ✅ Handles 100+ VMs without slowdown

### Code Quality
- ✅ TypeScript compiles without errors
- ✅ No console warnings
- ✅ Clean, maintainable code
- ✅ Reusable components

---

## Deliverables

### Code Deliverables
1. **Design System**
   - Updated Tailwind config with desktop tokens
   - Theme CSS files (light/dark)
   - Theme switching utility

2. **Context Menu System**
   - Reusable ContextMenu component
   - VM list context menu
   - Hardware tree context menu
   - Table context menus

3. **Keyboard Shortcuts**
   - Enhanced keyboard shortcut system
   - Shortcut display in tooltips/menus
   - Keyboard shortcuts help dialog

4. **Window State Management**
   - Backend: Window state persistence (Rust)
   - Frontend: Window state hooks
   - Window restoration on startup

5. **Updated Components**
   - All components using new design tokens
   - Consistent spacing/typography
   - Desktop-style interactions

### Documentation Deliverables
1. **User Documentation**
   - Keyboard shortcuts reference
   - Context menu guide
   - Theme customization guide

2. **Developer Documentation**
   - Design system documentation
   - Component usage examples
   - Window management patterns

3. **Status Reports**
   - Daily progress updates
   - Week 4 completion report
   - Screenshots/demos

---

## Timeline

### Week 4 Schedule

**Monday (Day 1)**: Design System Foundation
- Morning: Update Tailwind config, create theme files
- Afternoon: Start updating core components

**Tuesday (Day 2)**: Design System Implementation
- Morning: Continue component updates
- Afternoon: Test theme switching, fix issues

**Wednesday (Day 3)**: Context Menus
- Morning: Build context menu system
- Afternoon: Implement VM list and hardware tree menus

**Thursday (Day 4)**: Keyboard Shortcuts
- Morning: Add new shortcuts
- Afternoon: Implement shortcut display

**Friday (Day 5)**: Double-Click & Additional Interactions
- Morning: Implement double-click behaviors
- Afternoon: Polish interactions, edge cases

**Weekend**: Window State Persistence
- Saturday: Backend implementation
- Sunday: Frontend integration

**Monday (Day 8)**: Testing & Bug Fixes
- All day: Comprehensive testing and fixes

---

## Risk Assessment

### High Priority Risks
1. **Context Menu Library Issues**
   - Risk: Radix UI context menu doesn't work in Tauri
   - Mitigation: Test early, fallback to custom implementation

2. **Window State Edge Cases**
   - Risk: Multi-monitor, off-screen windows
   - Mitigation: Thorough edge case testing, validation

3. **Theme System Complexity**
   - Risk: CSS variable system breaks existing styles
   - Mitigation: Incremental rollout, thorough testing

### Medium Priority Risks
1. **Performance Impact**
   - Risk: Context menus slow down app
   - Mitigation: Performance testing, optimization

2. **Keyboard Shortcut Conflicts**
   - Risk: Shortcuts conflict with system shortcuts
   - Mitigation: Test on all target platforms, make configurable

### Low Priority Risks
1. **Visual Inconsistencies**
   - Risk: Some components don't match new design
   - Mitigation: Systematic component audit

---

## Next Steps After Week 4

### Week 5+: Advanced Features
- System tray integration
- Native title bar (optional)
- Drag & drop operations
- Advanced window management
- Performance monitoring dashboard
- Guest agent UI integration

### Quality & Performance
- E2E testing with Playwright/Tauri testing
- Performance profiling and optimization
- Accessibility audit
- User testing and feedback

### Documentation & Release
- Complete user documentation
- Video tutorials/demos
- Release preparation
- Packaging improvements

---

## Conclusion

Week 4 represents the **final transformation** from web app to desktop app. After this week, KVM Manager should:

✅ **Look** like a professional desktop application
✅ **Feel** native to the desktop environment
✅ **Behave** with desktop conventions users expect
✅ **Perform** as smoothly as native apps

**End Goal**: A virtualization manager that users **prefer** over virt-manager because it combines:
- Familiar desktop UX patterns
- Modern, polished visual design
- Powerful features
- Excellent performance

🚀 **Let's make it happen!**
