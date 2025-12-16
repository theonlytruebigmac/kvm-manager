# Desktop UI Architecture - Fixed Viewport Pattern

**Date**: December 8, 2025
**Status**: Implemented

---

## Overview

The KVM Manager desktop application now uses a **fixed-viewport layout pattern** where the application never requires scrolling at the window level. Instead, scrolling is contained within specific content areas, creating a true desktop application feel.

### Key Principle
**Everything stays within the window bounds** - the UI dynamically resizes when the window resizes, but no content ever extends beyond the viewport requiring page-level scrolling.

---

## Architecture Components

### 1. Root Layout (`src/components/layout/Layout.tsx`)

The root layout establishes the fixed-viewport structure:

```tsx
<div className="h-screen w-screen flex flex-col bg-background overflow-hidden">
  <header className="flex-shrink-0 border-b bg-card shadow-sm">
    {/* Fixed navigation header - never scrolls */}
  </header>
  <main className="flex-1 min-h-0 overflow-hidden">
    {/* Page content goes here - each page manages its own scroll */}
    {children}
  </main>
</div>
```

**Key CSS classes:**
- `h-screen w-screen` - Take full viewport dimensions
- `flex flex-col` - Vertical flexbox layout
- `overflow-hidden` - Prevent root-level scrolling
- `flex-shrink-0` - Header never shrinks
- `flex-1 min-h-0` - Main takes remaining space, allows child overflow

### 2. Page Container Components (`src/components/layout/PageContainer.tsx`)

Reusable components for consistent page structure:

#### `<PageContainer>`
Main wrapper for all page content. Provides the flex column structure.

```tsx
<PageContainer>
  {children}
</PageContainer>
```

#### `<PageHeader>`
Fixed header section that never scrolls. Contains title, description, stats, and actions.

```tsx
<PageHeader
  title="Page Title"
  description="Optional description"
  stats={<div>Stats JSX</div>}
  actions={<>Action buttons</>}
/>
```

#### `<PageToolbar>`
Optional fixed toolbar below header. For filters, search bars, and controls.

```tsx
<PageToolbar>
  <div className="space-y-3">
    {/* Search, filters, batch operations */}
  </div>
</PageToolbar>
```

#### `<PageContent>`
Scrollable content area. Takes all remaining vertical space.

```tsx
<PageContent>
  {/* Your scrollable content */}
</PageContent>
```

Or without padding:

```tsx
<PageContent noPadding>
  {/* Full-width content */}
</PageContent>
```

---

## Usage Patterns

### Simple Page (e.g., Dashboard)

```tsx
import { PageContainer, PageHeader, PageContent } from '@/components/layout/PageContainer'

export function Dashboard() {
  return (
    <PageContainer>
      <PageHeader
        title="Dashboard"
        description="System overview"
      />
      <PageContent>
        <div className="space-y-6">
          {/* Cards, charts, etc. */}
        </div>
      </PageContent>
    </PageContainer>
  )
}
```

### Complex Page with Toolbar (e.g., VmList)

```tsx
import { PageContainer, PageHeader, PageToolbar, PageContent } from '@/components/layout/PageContainer'

export function VmList() {
  return (
    <PageContainer>
      <PageHeader
        title="Virtual Machines"
        description="Manage and monitor your VMs"
        stats={<div>20 VMs</div>}
        actions={
          <>
            <Button>Create VM</Button>
          </>
        }
      />

      <PageToolbar>
        <div className="space-y-3">
          {/* Search input */}
          {/* Filter badges */}
          {/* Batch operations */}
        </div>
      </PageToolbar>

      <PageContent>
        {/* Scrollable VM list */}
      </PageContent>
    </PageContainer>
  )
}
```

### Loading/Error States

Always use `h-full` for centered loading/error states:

```tsx
if (isLoading) {
  return (
    <div className="h-full flex items-center justify-center">
      <Loader2 className="w-8 h-8 animate-spin" />
    </div>
  )
}

if (error) {
  return (
    <div className="h-full flex items-center justify-center">
      <AlertCircle className="w-8 h-8 text-destructive" />
      <p>Error message</p>
    </div>
  )
}
```

---

## CSS Foundation

### Global Styles (`src/styles/globals.css`)

```css
html, body, #root {
  @apply h-full overflow-hidden;
}
```

This ensures:
- Full viewport height at all levels
- No scrolling at document level
- React root fills the viewport

### Tailwind Classes Reference

#### Flexbox Layout
- `flex` - Display as flexbox
- `flex-col` - Vertical direction
- `flex-1` - Take remaining space
- `flex-shrink-0` - Never shrink

#### Height Management
- `h-full` - 100% of parent height
- `h-screen` - 100vh (viewport height)
- `min-h-0` - Allow flex children to overflow

#### Overflow
- `overflow-hidden` - No scrollbars
- `overflow-y-auto` - Vertical scroll only when needed
- `overflow-x-hidden` - No horizontal scroll

---

## Migration Guide

### Converting Existing Pages

**Old Pattern** (with page-level scrolling):
```tsx
export function MyPage() {
  return (
    <div className="space-y-6">
      <h1>Title</h1>
      {/* Content that scrolls the whole page */}
    </div>
  )
}
```

**New Pattern** (with contained scrolling):
```tsx
export function MyPage() {
  return (
    <PageContainer>
      <PageHeader title="Title" />
      <PageContent>
        <div className="space-y-6">
          {/* Only this content scrolls */}
        </div>
      </PageContent>
    </PageContainer>
  )
}
```

### Checklist for Page Updates

- [ ] Import `PageContainer` components
- [ ] Wrap page in `<PageContainer>`
- [ ] Extract title/actions into `<PageHeader>`
- [ ] Extract filters/search into `<PageToolbar>` (if needed)
- [ ] Wrap main content in `<PageContent>`
- [ ] Update loading/error states to use `h-full`
- [ ] Test at different window sizes
- [ ] Verify no double scrollbars appear

---

## Pages Updated

### Completed ✅
- `Layout.tsx` - Root layout structure
- `VmList.tsx` - Full page with header, toolbar, and scrollable content
- `Dashboard.tsx` - Simple page with header and scrollable content

### Pending 🔄
- `VmDetails.tsx` - Complex detail page with tabs
- `StorageManager.tsx` - Storage pools and volumes
- `NetworkManager.tsx` - Network configuration
- `Insights.tsx` - System insights dashboard
- `Templates.tsx` - VM templates
- `Schedules.tsx` - Scheduled operations
- `Alerts.tsx` - Alert configuration
- `Backups.tsx` - Backup management
- `Settings.tsx` - Application settings

---

## Benefits

### User Experience
- ✅ **No confusing scrollbars** - Clear visual hierarchy
- ✅ **Responsive design** - Works at any window size
- ✅ **Native feel** - Behaves like desktop applications
- ✅ **Predictable layout** - Content always in expected locations

### Developer Experience
- ✅ **Consistent patterns** - Reusable components
- ✅ **Clear structure** - Easy to understand layout hierarchy
- ✅ **Maintainable** - Centralized layout logic
- ✅ **Flexible** - Components adapt to different use cases

### Performance
- ✅ **Efficient rendering** - Only scroll areas re-render on scroll
- ✅ **GPU acceleration** - Fixed elements don't repaint
- ✅ **Smooth animations** - No layout thrashing

---

## Best Practices

### Do's ✅
- Use `PageContainer` components for all pages
- Keep headers and toolbars fixed (no scrolling)
- Use `overflow-y-auto` only in `PageContent`
- Test at multiple window sizes (min: 1024x768)
- Use `h-full` for centered loading/error states

### Don'ts ❌
- Don't use `min-h-[Xpx]` on page-level elements
- Don't add `overflow-y-auto` to multiple levels
- Don't use fixed pixel heights on containers
- Don't mix old and new patterns in same page
- Don't forget to test window resizing

---

## Troubleshooting

### Problem: Content gets cut off

**Cause**: Missing `min-h-0` on flex child
**Solution**: Add `min-h-0` to the `<main>` or flex container

### Problem: Double scrollbars

**Cause**: Multiple `overflow-y-auto` in hierarchy
**Solution**: Only `PageContent` should have `overflow-y-auto`

### Problem: Content doesn't fill height

**Cause**: Missing `h-full` on container
**Solution**: Ensure full chain has `h-full`: `html` → `body` → `#root` → `Layout` → `Page`

### Problem: Window resize breaks layout

**Cause**: Fixed pixel heights somewhere
**Solution**: Use flex-based heights (`flex-1`, `h-full`) instead of `h-[Xpx]`

---

## Future Enhancements

Potential improvements to consider:

1. **Collapsible Header** - Hide header on scroll for more vertical space
2. **Split Panes** - Resizable panes for detail views
3. **Virtual Scrolling** - For very long lists (1000+ items)
4. **Sticky Toolbars** - Context-aware toolbars that stick to content
5. **Custom Scrollbars** - Styled scrollbars for better aesthetics

---

## References

- [Tailwind Flexbox Documentation](https://tailwindcss.com/docs/flex)
- [CSS Tricks: Flexbox Guide](https://css-tricks.com/snippets/css/a-guide-to-flexbox/)
- [MDN: Flexbox](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Flexible_Box_Layout)

---

*This architecture provides a solid foundation for a polished desktop application experience.*
