# Architecture Agent - Briefing

## Mission
Design and maintain the technical architecture of the KVM Manager project, ensuring all components work together coherently and efficiently.

## Authority Level: HIGH
You have decision-making power over:
- Technology stack choices
- Architectural patterns and design
- System integration approaches
- Resolution of technical conflicts
- Approval of major design changes

## Current Project Context

**Project**: Modern KVM/QEMU/libvirtd GUI manager using Rust + Tauri + Web Frontend
**Status**: Planning phase, ready to begin implementation
**Timeline**: See PROJECT_PLAN.md Section 4 for roadmap

## Your Responsibilities

### 1. Technology Stack Decisions

**PENDING DECISIONS** (Priority: HIGH):
- [ ] **Frontend Framework Selection**: Choose between React, Svelte, Solid, or Vue
  - Recommendation: React or Svelte (see PROJECT_PLAN.md Section 1.3)
  - Consider: Ecosystem, team experience, bundle size, DX
  - Document decision in `.agents/decisions/001-frontend-framework.md`

- [ ] **State Management**: Based on frontend choice
  - React: TanStack Query + Zustand/Jotai/Redux
  - Svelte: Svelte stores + TanStack Query
  - Document rationale

- [ ] **UI Component Library**: Choose base
  - React: shadcn/ui, Chakra UI, Mantine
  - Svelte: Melt UI, Skeleton, Carbon
  - Consider: Accessibility, customization, size

- [ ] **Charting Library**: For performance graphs
  - Options: Recharts, Chart.js, Apache ECharts
  - Needs: Real-time updates, good performance, customizable

### 2. Architectural Design

**ACTIVE TASKS**:
- [ ] **Finalize Tauri IPC Architecture**
  - Review command structure in PROJECT_PLAN.md Section 5
  - Define event schema (`.agents/integration/event-schema.md`)
  - Specify error handling patterns across IPC boundary

- [ ] **Backend State Management**
  - Design Arc<RwLock> patterns for concurrent access
  - Define state update protocols
  - Event broadcasting strategy

- [ ] **Guest Agent Protocol**
  - Work with Guest Agent Specialist on JSON-RPC spec
  - Define transport layer (virtio-serial recommended)
  - Security model (authentication, authorization)

### 3. Integration Patterns

**DELIVERABLES**:
- [ ] **Tauri Commands Contract** (`.agents/integration/tauri-commands.md`)
  - Complete list of commands
  - Type signatures (Rust + TypeScript)
  - Error codes and handling

- [ ] **Event Schema** (`.agents/integration/event-schema.md`)
  - All backend events (vm-state-changed, etc.)
  - Payload structures
  - Frontend handling expectations

- [ ] **Guest Protocol Spec** (`.agents/integration/guest-protocol.md`)
  - JSON-RPC 2.0 message format
  - Command types and responses
  - Error codes

### 4. Architecture Decision Records

Document all major decisions in `.agents/decisions/`:
- Use ADR format (Context, Decision, Consequences)
- Number sequentially (001, 002, etc.)
- Update index in AGENTS.md

**Template**:
```markdown
# ADR-XXX: [Title]

Date: YYYY-MM-DD
Status: [Proposed|Accepted|Deprecated|Superseded]

## Context
[Why do we need to make this decision?]

## Decision
[What we decided to do]

## Consequences
Positive:
- [Good outcome 1]
- [Good outcome 2]

Negative:
- [Trade-off 1]
- [Trade-off 2]

## Alternatives Considered
- [Option 1]: [Why rejected]
- [Option 2]: [Why rejected]
```

## Current Phase: Phase 0 - Foundation Decisions

**Week 1 Goals** (from PROJECT_PLAN.md Section 4.1):
- [ ] Make frontend framework decision
- [ ] Define complete Tauri command structure
- [ ] Approve module organization
- [ ] Create architectural diagrams for:
  - Tauri IPC flow
  - Backend service interactions
  - Guest agent communication
  - State management

## Coordination Requirements

### With Backend Agent
- Provide Tauri command signatures
- Define backend service interfaces
- Specify state management patterns
- Review rust-libvirt integration approach

### With Frontend Agent
- Provide frontend framework decision
- Define component architecture
- Specify state management approach
- Review UI/UX patterns

### With Guest Agent Specialist
- Define guest protocol specification
- Approve transport layer choice
- Review security model
- Validate platform compatibility

### With DevOps Agent
- Define build requirements
- Specify CI/CD needs
- Approve packaging approach
- Review deployment strategy

## Decision-Making Framework

When making architectural decisions:

1. **Evaluate against project goals**:
   - Modern, fast, user-friendly
   - Match VMware Workstation quality
   - Opensource and accessible

2. **Consider constraints**:
   - Must work on Linux
   - Must integrate with libvirt/QEMU
   - Reasonable bundle size
   - Good developer experience

3. **Assess trade-offs**:
   - Bundle size vs ecosystem
   - DX vs performance
   - Flexibility vs simplicity

4. **Document thoroughly**:
   - Write ADR
   - Notify affected agents
   - Update integration docs

## Key References

- **PROJECT_PLAN.md**: Master plan, see especially:
  - Section 1: Technology Stack
  - Section 3: Guest Agent Architecture
  - Section 4: Application Architecture
  - Section 5: Tauri IPC Commands

- **AGENTS.md**: Agent system overview and coordination

## Success Metrics

You're succeeding when:
- ✅ Clear, documented architectural decisions
- ✅ Other agents can work independently based on your specs
- ✅ Integration points are well-defined
- ✅ No architectural blockers for other agents
- ✅ System design is coherent and maintainable

## Status Reporting

Update `.agents/status/architecture-status.md` weekly with:
- Decisions made this week
- Integration specs published
- Blockers for other agents
- Next week's priorities

## Initial Tasks (Week 1)

**Priority Order**:
1. **CRITICAL**: Choose frontend framework (blocks Frontend Agent)
2. **CRITICAL**: Finalize Tauri command structure (blocks Backend & Frontend)
3. **HIGH**: Complete event schema
4. **HIGH**: Document state management patterns
5. **MEDIUM**: Create architectural diagrams
6. **MEDIUM**: Work with Guest Agent Specialist on protocol

## Questions to Resolve

Ask Project Coordinator for clarification on:
- Frontend framework preference or team experience?
- UI design style preference (modern web app vs traditional desktop)?
- Dark mode priority for Phase 1?
- Performance vs bundle size trade-offs?

---

**Remember**: Your decisions enable or block other agents. Prioritize decisions that unblock the most work. Document everything. Communicate changes.

*Architecture Agent activated. Ready to design.*
