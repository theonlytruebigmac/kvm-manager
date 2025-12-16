# Desktop Components

Desktop-style UI components designed to make KVM Manager look and feel like a native desktop application.

## Toolbar

A horizontal toolbar component with tight spacing and subtle background, designed to replicate the look of native desktop application toolbars.

### Components

#### `Toolbar`

Main container for toolbar items.

**Styling:**
- Horizontal flex layout with 4px gap (`gap-1`)
- Subtle background: `bg-muted/30`
- Border on bottom: `border-b`
- Padding: 8px horizontal (`px-2`), 6px vertical (`py-1.5`)

**Usage:**
```tsx
import { Toolbar } from "@/components/desktop"

<Toolbar>
  {/* Toolbar items */}
</Toolbar>
```

---

#### `ToolbarButton`

Icon button with optional text label.

**Props:**
- `icon` (required): Lucide icon component
- `children` (optional): Text label to display next to icon
- `tooltip` (optional): Tooltip text (uses native HTML title attribute)
- `disabled` (optional): Disable the button
- All standard button HTML attributes

**Styling:**
- Icons: 16px (`w-4 h-4`)
- Padding: 12px horizontal (`px-3`), 6px vertical (`py-1.5`)
- Gap between icon and text: 6px (`gap-1.5`)
- Rounded corners: `rounded-md`
- Hover effect: `hover:bg-accent hover:text-accent-foreground`
- Focus ring: `focus-visible:ring-2`
- Disabled state: 50% opacity, no pointer events

**Usage:**
```tsx
import { Play, Monitor } from "lucide-react"
import { ToolbarButton } from "@/components/desktop"

// With label
<ToolbarButton icon={Play} onClick={handleStart}>
  Start
</ToolbarButton>

// Icon only with tooltip
<ToolbarButton icon={Monitor} onClick={handleConsole} tooltip="Open console" />

// Disabled
<ToolbarButton icon={Play} onClick={handleStart} disabled>
  Start
</ToolbarButton>
```

---

#### `ToolbarSeparator`

Vertical divider line to visually separate groups of toolbar buttons.

**Styling:**
- Width: 1px (`w-px`)
- Height: 24px (`h-6`)
- Color: `bg-border`
- Margins: 4px horizontal (`mx-1`)

**Usage:**
```tsx
import { ToolbarSeparator } from "@/components/desktop"

<Toolbar>
  <ToolbarButton icon={Plus} onClick={handleNew}>New</ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton icon={Play} onClick={handleStart}>Start</ToolbarButton>
</Toolbar>
```

---

#### `ToolbarSpacer`

Flex spacer that pushes subsequent items to the right side of the toolbar.

**Styling:**
- `flex-1` to fill available space

**Usage:**
```tsx
import { ToolbarSpacer } from "@/components/desktop"

<Toolbar>
  <ToolbarButton icon={Plus} onClick={handleNew}>New</ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton icon={Settings} onClick={handleSettings} />
</Toolbar>
```

---

### Complete Example

```tsx
import { Plus, Play, Square, Pause, Monitor, Trash2, Settings, RefreshCw } from "lucide-react"
import { Toolbar, ToolbarButton, ToolbarSeparator, ToolbarSpacer } from "@/components/desktop"

function VmToolbar() {
  return (
    <Toolbar>
      {/* Left-aligned group: VM actions */}
      <ToolbarButton icon={Plus} onClick={handleNew}>
        New VM
      </ToolbarButton>
      <ToolbarSeparator />

      {/* VM control buttons */}
      <ToolbarButton icon={Play} onClick={handleStart}>Start</ToolbarButton>
      <ToolbarButton icon={Pause} onClick={handlePause}>Pause</ToolbarButton>
      <ToolbarButton icon={Square} onClick={handleStop}>Stop</ToolbarButton>

      <ToolbarSeparator />

      {/* VM management */}
      <ToolbarButton icon={Monitor} onClick={handleConsole}>Console</ToolbarButton>
      <ToolbarButton icon={Trash2} onClick={handleDelete}>Delete</ToolbarButton>

      {/* Push remaining items to the right */}
      <ToolbarSpacer />

      {/* Right-aligned group: Utilities */}
      <ToolbarButton icon={RefreshCw} onClick={handleRefresh} tooltip="Refresh list" />
      <ToolbarButton icon={Settings} onClick={handleSettings} tooltip="Settings" />
    </Toolbar>
  )
}
```

### Design Patterns

#### Icon-only Toolbar (Compact)
Use tooltips to provide context for icon-only buttons:

```tsx
<Toolbar>
  <ToolbarButton icon={Plus} onClick={handleNew} tooltip="Create new VM" />
  <ToolbarButton icon={Play} onClick={handleStart} tooltip="Start VM" />
  <ToolbarButton icon={Square} onClick={handleStop} tooltip="Stop VM" />
</Toolbar>
```

#### Mixed Icon + Label Toolbar
Combine labeled and icon-only buttons:

```tsx
<Toolbar>
  <ToolbarButton icon={Plus} onClick={handleNew}>New VM</ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton icon={Play} onClick={handleStart}>Start</ToolbarButton>
  <ToolbarButton icon={Square} onClick={handleStop}>Stop</ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton icon={Settings} onClick={handleSettings} tooltip="Settings" />
</Toolbar>
```

#### Conditional Toolbar Items
Show/hide buttons based on state:

```tsx
<Toolbar>
  <ToolbarButton icon={Plus} onClick={handleNew}>New VM</ToolbarButton>
  <ToolbarSeparator />

  {vmState === "running" ? (
    <>
      <ToolbarButton icon={Pause} onClick={handlePause}>Pause</ToolbarButton>
      <ToolbarButton icon={Square} onClick={handleStop}>Stop</ToolbarButton>
    </>
  ) : (
    <ToolbarButton icon={Play} onClick={handleStart}>Start</ToolbarButton>
  )}
</Toolbar>
```

#### Disabled State
Disable buttons when actions aren't available:

```tsx
<Toolbar>
  <ToolbarButton icon={Plus} onClick={handleNew}>New VM</ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton
    icon={Play}
    onClick={handleStart}
    disabled={!selectedVm || vmState === "running"}
  >
    Start
  </ToolbarButton>
</Toolbar>
```

---

### Integration with Existing Components

The Toolbar component uses the same design tokens as other UI components:

- **Colors**: Uses `muted`, `accent`, `border` from your theme
- **Spacing**: Consistent with button component (`px-3`, `py-1.5`)
- **Typography**: `text-sm font-medium` matches button text
- **Focus rings**: Same focus-visible styles as Button component
- **Utilities**: Uses the `cn()` helper for className merging

---

### Future Enhancements

For advanced features, consider:

1. **Advanced Tooltips**: Replace native `title` attribute with a proper Tooltip component (e.g., Radix UI Tooltip)
2. **Dropdown Toolbars**: Add ToolbarDropdown for overflow items
3. **Button Groups**: Add ToolbarButtonGroup for radio-style button groups
4. **Search**: Add ToolbarSearch component for inline search
5. **Overflow Menu**: Automatically collapse items into a "More" menu on small screens

---

### Accessibility

- All buttons have `type="button"` to prevent form submission
- Disabled buttons have `disabled:pointer-events-none`
- Focus rings are visible with `focus-visible:ring-2`
- Tooltips provide context for icon-only buttons
- Semantic HTML with proper button elements

---

### Browser Support

Works in all modern browsers that support:
- CSS Flexbox
- CSS custom properties (for theming)
- Modern JavaScript (ES6+)
