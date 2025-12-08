# ADR-001: Frontend Technology Stack

Date: 2025-12-07
Status: Accepted
Decision Makers: Architecture Agent, Project Owner

## Context

KVM Manager requires a modern, performant frontend framework for building the GUI. The application needs to provide a VMware Workstation/Hyper-V Manager quality user experience while integrating cleanly with Tauri's Rust backend.

### Requirements

1. **Modern UI**: Clean, polished interface with rich interactivity
2. **Rich Ecosystem**: Access to high-quality component libraries, charting tools, and state management solutions
3. **TypeScript Support**: Strong typing for maintainability and developer experience
4. **Tauri Integration**: Excellent compatibility with Tauri's IPC system
5. **Accessibility**: Built-in or easy-to-add accessibility features
6. **Performance**: Fast rendering, especially with many VMs (100+)
7. **Developer Experience**: Good tooling, documentation, and community support

### Constraints

- Must work well with Tauri (no framework-specific limitations)
- Bundle size is less critical (Tauri doesn't bundle Chromium)
- Team familiarity may vary (opensource project)
- Need for complex state management (server state + local UI state)

## Decision

**Selected Stack:**
- **Framework**: React 18+ with TypeScript
- **UI Library**: shadcn/ui (built on Radix UI)
- **Styling**: Tailwind CSS
- **Icons**: Lucide React

## Rationale

### Why React + TypeScript?

1. **Ecosystem Maturity**
   - Largest ecosystem of any frontend framework
   - Best-in-class libraries for our needs:
     - TanStack Query (React Query) for server state
     - Recharts/Victory for performance graphs
     - shadcn/ui for modern, accessible components
   - Extensive Tauri examples and community support

2. **State Management Options**
   - TanStack Query: Industry-standard for server state synchronization
   - Zustand/Jotai: Lightweight options for local UI state
   - Proven patterns for complex applications

3. **Developer Experience**
   - Massive community and resources
   - Excellent TypeScript support
   - Great debugging tools (React DevTools)
   - Most developers have React experience

4. **Component Libraries**
   - shadcn/ui provides exactly what we need:
     - Copy-paste components (full control)
     - Built on Radix UI (accessibility-first)
     - Tailwind CSS integration
     - Modern, clean aesthetic
   - Alternative: Mantine, Chakra UI (also excellent)

5. **Performance**
   - React 18's concurrent features
   - Virtual scrolling libraries (@tanstack/react-virtual)
   - Optimization patterns are well-documented
   - Bundle size less important with Tauri

### Why shadcn/ui?

1. **Copy-Paste Model**: Components live in our codebase (full customization)
2. **Accessibility**: Built on Radix UI (ARIA-compliant)
3. **Modern Design**: Clean, professional look out-of-box
4. **Tailwind Integration**: Consistent with our styling approach
5. **TypeScript-First**: Excellent type safety
6. **Active Development**: Well-maintained, growing ecosystem

### Why Tailwind CSS?

1. **Utility-First**: Rapid UI development
2. **Consistency**: Design system tokens built-in
3. **Performance**: Purges unused styles
4. **Customization**: Easy theming and brand colors
5. **Industry Standard**: Most modern UIs use Tailwind

## Consequences

### Positive

1. **Rich Component Ecosystem**: Access to high-quality, accessible components
2. **Proven Patterns**: Well-established patterns for complex state management
3. **Easy Recruitment**: Most frontend developers know React
4. **Great Documentation**: Extensive resources for React, shadcn/ui, and Tailwind
5. **Modern Aesthetics**: shadcn/ui provides professional, clean UI out-of-box
6. **Accessibility**: Radix UI ensures WCAG compliance
7. **TypeScript Support**: Excellent type safety across the stack
8. **Charting Libraries**: Best options (Recharts, Victory) available
9. **Tauri Integration**: Well-tested and documented

### Negative

1. **Bundle Size**: Larger than Svelte (mitigated by Tauri's architecture)
2. **Boilerplate**: More verbose than Svelte (trade-off for explicitness)
3. **Learning Curve**: Steeper for developers unfamiliar with React hooks
4. **Re-render Concerns**: Need to be mindful of optimization (useMemo, useCallback)

### Trade-offs Accepted

- **Bundle Size vs Ecosystem**: We chose ecosystem richness over minimal bundle size
  - Justification: Tauri doesn't bundle Chromium, so bundle size impact is minimal
  - React's ~150KB gzipped is acceptable for the value gained

- **Simplicity vs Power**: React's complexity buys us powerful patterns
  - TanStack Query handles complex server state elegantly
  - Proven patterns for real-time updates and caching

- **Framework Lock-in vs Component Quality**: shadcn/ui is React-specific
  - Benefit: Best-in-class components with accessibility
  - Mitigation: Components live in our repo (can customize heavily)

## Alternatives Considered

### Svelte + TypeScript
**Pros:**
- Smaller bundle size (~20-30KB)
- Less boilerplate, more intuitive
- Great performance (compiles to vanilla JS)
- Growing ecosystem

**Cons:**
- Smaller component library ecosystem
- Fewer charting library options (Chart.js, but not Recharts)
- Less community support for Tauri integration
- Fewer developers familiar with Svelte
- State management less mature than React Query

**Why Rejected:** Ecosystem trade-off not worth the bundle size savings given Tauri's architecture.

### Vue 3 + TypeScript
**Pros:**
- Good balance of simplicity and features
- Composition API is clean
- Decent component libraries (PrimeVue, Vuetify)
- Good TypeScript support

**Cons:**
- Smaller ecosystem than React
- Fewer developers familiar with Vue 3 Composition API
- Component library quality slightly behind React
- Less Tauri community examples

**Why Rejected:** React's ecosystem and community support provided more value.

### Solid + TypeScript
**Pros:**
- Excellent performance (fine-grained reactivity)
- React-like API (easier migration path)
- Small bundle size
- Growing momentum

**Cons:**
- Immature ecosystem (fewer libraries)
- Very limited component libraries
- Smaller community
- Uncertain long-term viability

**Why Rejected:** Too risky for a project requiring mature libraries and long-term maintenance.

## Implementation Notes

### Package Structure
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tanstack/react-query": "^5.x",
    "zustand": "^4.x",
    "lucide-react": "^0.x",
    "@tauri-apps/api": "^1.x"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "typescript": "^5.x",
    "tailwindcss": "^3.x",
    "vite": "^5.x"
  }
}
```

### Component Organization
```
src/
├── components/
│   ├── ui/           # shadcn/ui components
│   ├── VmCard.tsx    # Domain components
│   └── ...
├── lib/
│   ├── tauri.ts      # Tauri invoke wrappers
│   └── utils.ts
├── hooks/
│   ├── useVms.ts     # TanStack Query hooks
│   └── ...
└── stores/
    └── ui-store.ts   # Zustand store for UI state
```

### Key Libraries to Add

**Phase 1 (MVP):**
- shadcn/ui: Button, Card, Dialog, Table, Badge, Dropdown
- TanStack Query: Server state management
- Zustand: Local UI state
- Lucide React: Icons
- Sonner: Toast notifications

**Phase 2:**
- Recharts: Performance graphs
- @tanstack/react-virtual: Virtual scrolling for VM lists
- react-hook-form: Form validation in wizards

## Review and Approval

- **Architecture Agent**: Approved (2025-12-07)
- **Project Owner**: Approved (2025-12-07 - decision made)
- **Frontend Agent**: To implement
- **Backend Agent**: Acknowledged (affects type generation)

## Related Decisions

- ADR-002: State Management Patterns (details TanStack Query + Zustand approach)

## References

- [React Documentation](https://react.dev)
- [shadcn/ui](https://ui.shadcn.com)
- [Tailwind CSS](https://tailwindcss.com)
- [TanStack Query](https://tanstack.com/query)
- [Tauri + React Guide](https://tauri.app/v1/guides/getting-started/setup/react)
