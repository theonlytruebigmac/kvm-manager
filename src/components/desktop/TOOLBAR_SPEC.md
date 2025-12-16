# Toolbar Component Specification

## Visual Design

```
┌────────────────────────────────────────────────────────────────────────────┐
│  [+ New VM]  │  [▶ Start]  [⏸ Pause]  [⏹ Stop]  │  [🖥 Console]  ⎵  [⚙]  │
└────────────────────────────────────────────────────────────────────────────┘
```

## Measurements

### Toolbar Container
- **Height**: Auto (based on content, ~40px typical)
- **Padding**:
  - Horizontal: 8px (`px-2`)
  - Vertical: 6px (`py-1.5`)
- **Gap**: 4px between items (`gap-1`)
- **Background**: `bg-muted/30` (subtle gray)
- **Border**: Bottom only (`border-b`)

### ToolbarButton
- **Icon Size**: 16x16px (`w-4 h-4`)
- **Padding**:
  - Horizontal: 12px (`px-3`)
  - Vertical: 6px (`py-1.5`)
- **Gap**: 6px between icon and label (`gap-1.5`)
- **Font**: `text-sm font-medium`
- **Border Radius**: `rounded-md` (6px)
- **States**:
  - Default: Transparent background
  - Hover: `bg-accent` with `text-accent-foreground`
  - Focus: 2px ring with 1px offset
  - Disabled: 50% opacity, no pointer events

### ToolbarSeparator
- **Width**: 1px (`w-px`)
- **Height**: 24px (`h-6`)
- **Color**: `bg-border`
- **Margin**: 4px horizontal (`mx-1`)

### ToolbarSpacer
- **Width**: Flexible (`flex-1`)
- **Height**: Minimal
- **Purpose**: Pushes items to the right

## Color Tokens

Uses theme colors:
- `muted`: Subtle background for toolbar
- `accent`: Hover state background
- `accent-foreground`: Hover state text
- `border`: Separator color
- `ring`: Focus ring color

## Typography

- **Size**: `text-sm` (14px)
- **Weight**: `font-medium` (500)
- **Line Height**: Default for text-sm

## Spacing System

Following Tailwind's spacing scale:
- `gap-1` = 4px (tight spacing between items)
- `gap-1.5` = 6px (icon-to-label spacing)
- `px-2` = 8px (toolbar horizontal padding)
- `px-3` = 12px (button horizontal padding)
- `py-1.5` = 6px (vertical padding)
- `mx-1` = 4px (separator margins)

## Interaction States

### Button States
1. **Default**:
   - Transparent background
   - Default text color

2. **Hover**:
   - `bg-accent` background
   - `text-accent-foreground` text color
   - Smooth transition (transition-colors)

3. **Focus**:
   - Visible focus ring (2px, ring color)
   - 1px offset from button
   - Only visible on keyboard focus (focus-visible)

4. **Active/Pressed**:
   - Same as hover (browser default)

5. **Disabled**:
   - 50% opacity
   - No pointer events
   - Cursor not-allowed (browser default)
   - No tooltip

## Layout Patterns

### Pattern 1: All Labeled Buttons
```tsx
<Toolbar>
  <ToolbarButton icon={Plus}>New VM</ToolbarButton>
  <ToolbarButton icon={Play}>Start</ToolbarButton>
  <ToolbarButton icon={Square}>Stop</ToolbarButton>
</Toolbar>
```

### Pattern 2: Icon-Only with Tooltips
```tsx
<Toolbar>
  <ToolbarButton icon={Plus} tooltip="New VM" />
  <ToolbarButton icon={Play} tooltip="Start" />
  <ToolbarButton icon={Square} tooltip="Stop" />
</Toolbar>
```

### Pattern 3: Mixed with Separators
```tsx
<Toolbar>
  <ToolbarButton icon={Plus}>New VM</ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton icon={Play}>Start</ToolbarButton>
  <ToolbarButton icon={Square}>Stop</ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton icon={Monitor}>Console</ToolbarButton>
</Toolbar>
```

### Pattern 4: Left and Right Groups
```tsx
<Toolbar>
  <ToolbarButton icon={Plus}>New VM</ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton icon={Play}>Start</ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton icon={Settings} tooltip="Settings" />
</Toolbar>
```

## Responsive Behavior

The toolbar is designed for desktop use and does not currently have responsive breakpoints. On smaller screens, consider:

1. **Hiding labels**: Convert to icon-only mode
2. **Overflow menu**: Move less important items to a dropdown
3. **Stacking**: Not recommended for toolbars

## Desktop App Reference

This component is designed to match the look and feel of toolbars in native desktop applications like:
- Visual Studio Code
- Slack Desktop
- Discord Desktop
- macOS/Windows native apps

Key characteristics:
- Subtle, unobtrusive background
- Tight spacing for efficiency
- Small, consistent icons
- Clear visual grouping with separators
- Left-to-right flow with right-aligned utilities
