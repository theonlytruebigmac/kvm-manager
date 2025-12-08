# KVM Manager - Agent System

## Overview

This project uses a specialized agent system to manage different aspects of development. Each agent has specific responsibilities, decision-making authority, and coordinates with other agents to maintain project coherence.

## Agent Architecture

```
                    ┌─────────────────────────┐
                    │   Project Coordinator   │
                    │  (Human + Lead Agent)   │
                    └───────────┬─────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
        ┌───────▼──────┐ ┌─────▼──────┐ ┌─────▼──────┐
        │  Architecture │ │  Backend   │ │  Frontend  │
        │     Agent     │ │   Agent    │ │   Agent    │
        └───────┬───────┘ └─────┬──────┘ └─────┬──────┘
                │               │               │
        ┌───────▼──────┐ ┌─────▼──────┐ ┌─────▼──────┐
        │ Guest Agent  │ │   DevOps   │ │    Docs    │
        │  Specialist  │ │   Agent    │ │   Agent    │
        └──────────────┘ └────────────┘ └────────────┘
```

## Agent Roster

### 0. Project Coordinator Agent ⭐
**Role**: Technical Project Manager & Team Coordinator
**Authority**: High-level coordination, planning, blocker resolution
**Responsibilities**:
- Coordinate all specialized agents
- Track milestones and progress
- Resolve cross-agent blockers
- Provide weekly status to project owner
- Facilitate decisions and integration
- Manage project timeline and scope

**Key Deliverables**:
- Weekly status reports to project owner
- Blocker tracking and resolution
- Milestone progress updates
- Agent coordination and facilitation

**Briefing File**: `.agents/project-coordinator.md`

**Special Role**: This agent works directly WITH the human project owner as a partner in managing the team of specialized agents.

---

### 1. Architecture Agent
**Role**: System Design & Technical Leadership
**Authority**: High-level technical decisions, architectural patterns
**Responsibilities**:
- Make technology stack decisions (frontend framework, libraries)
- Design system architecture and communication patterns
- Resolve technical conflicts between agents
- Maintain architectural coherence across components
- Review and approve major design changes

**Key Deliverables**:
- Architecture Decision Records (ADRs)
- System design diagrams
- Technology evaluation reports
- Integration patterns and guidelines

**Briefing File**: `.agents/architecture-agent.md`

---

### 2. Backend Agent
**Role**: Rust Backend & Libvirt Integration
**Authority**: Backend implementation, libvirt integration, Tauri commands
**Responsibilities**:
- Implement Tauri backend (src-tauri/)
- Integrate rust-libvirt for VM management
- Design and implement services (LibvirtService, StorageService, etc.)
- Create Tauri command handlers
- Implement state management in Rust
- Handle backend error management
- Write backend unit/integration tests

**Key Deliverables**:
- Rust backend codebase
- Tauri command implementations
- Service layer code
- Backend tests
- API documentation (rustdoc)

**Briefing File**: `.agents/backend-agent.md`

---

### 3. Frontend Agent
**Role**: Web UI & User Experience
**Authority**: Frontend implementation, UI/UX decisions, component design
**Responsibilities**:
- Implement frontend in chosen framework (React/Svelte)
- Design and build UI components
- Implement state management (TanStack Query, Zustand)
- Create responsive, accessible interfaces
- Integrate with Tauri IPC
- Handle frontend error states and loading
- Write frontend tests (component, E2E)

**Key Deliverables**:
- Frontend codebase (src/)
- UI component library
- Page implementations
- Frontend tests
- Style system (Tailwind config, themes)

**Briefing File**: `.agents/frontend-agent.md`

---

### 4. Guest Agent Specialist
**Role**: Guest Agent Development (Linux & Windows)
**Authority**: Guest agent architecture, protocol design, platform-specific implementations
**Responsibilities**:
- Design JSON-RPC protocol (agent-common/)
- Implement Linux guest agent (agent-linux/)
- Implement Windows guest agent (agent-windows/)
- Handle virtio-serial/VSOCK communication
- Create installer packages (.deb, .rpm, MSI)
- Write platform-specific handlers
- Test on multiple OS versions

**Key Deliverables**:
- Guest agent codebase (guest-agent/)
- Protocol specification
- Linux and Windows binaries
- Installation packages
- Agent documentation

**Briefing File**: `.agents/guest-agent.md`

---

### 5. DevOps Agent
**Role**: CI/CD, Build, Package, Deploy
**Authority**: Build systems, CI/CD pipelines, release process
**Responsibilities**:
- Set up GitHub Actions workflows
- Configure Tauri build pipeline
- Create packaging for multiple formats (AppImage, .deb, .rpm)
- Implement release automation
- Set up testing infrastructure
- Monitor build health
- Handle dependency updates

**Key Deliverables**:
- CI/CD pipelines (.github/workflows/)
- Build scripts
- Package configurations
- Release automation
- Deployment documentation

**Briefing File**: `.agents/devops-agent.md`

---

### 6. Documentation Agent
**Role**: Documentation & Knowledge Management
**Authority**: All documentation, guides, API docs
**Responsibilities**:
- Maintain PROJECT_PLAN.md
- Write user documentation
- Create developer guides
- Generate API documentation
- Keep README up to date
- Document architectural decisions
- Create troubleshooting guides

**Key Deliverables**:
- User documentation
- Developer guides
- API documentation
- Architecture Decision Records
- Contributing guidelines

**Briefing File**: `.agents/docs-agent.md`

---

## Agent Coordination Protocols

### Communication Channels

1. **Agent Status Reports**: `.agents/status/`
   - Each agent maintains a status file
   - Updated weekly or after major milestones
   - Format: Markdown with sections (Completed, In Progress, Blocked, Next)

2. **Decision Logs**: `.agents/decisions/`
   - Major decisions documented here
   - Format: ADR (Architecture Decision Record)
   - Numbered sequentially (001-decision-name.md)

3. **Integration Points**: `.agents/integration/`
   - Documents how agents' work integrates
   - API contracts, interfaces, protocols
   - Updated when interfaces change

### Dependency Management

**Frontend ← Backend**:
- Frontend depends on Tauri command signatures
- Backend Agent must notify Frontend Agent of API changes
- Changes documented in `.agents/integration/tauri-commands.md`

**Backend ← Guest Agent**:
- Backend depends on guest agent protocol
- Guest Agent Specialist must maintain protocol spec
- Protocol defined in `.agents/integration/guest-protocol.md`

**All ← Architecture**:
- All agents follow architecture decisions
- Architecture Agent documents decisions in ADRs
- Agents can propose architecture changes via issues

**All ← DevOps**:
- All agents must ensure code passes CI
- DevOps Agent provides build feedback
- Agents fix issues flagged by CI

### Conflict Resolution

1. **Technical Conflicts**: Architecture Agent has final say
2. **Timeline Conflicts**: Project Coordinator (human) decides
3. **Cross-agent Blockers**: Document in status, escalate to coordinator
4. **API Disagreements**: Architecture Agent mediates, Backend/Frontend negotiate

### Work Cadence

**Weekly Cycle**:
- **Monday**: Agents review PROJECT_PLAN.md, update status
- **Mid-week**: Implementation work, coordination as needed
- **Friday**: Status update, document blockers, plan next week

**Milestone-based**:
- Agents align work to roadmap milestones (see PROJECT_PLAN.md Section 4)
- Integration testing at milestone completion
- Demo/review with coordinator

### Quality Standards

All agents must ensure:
- ✅ Code passes CI (lint, test, build)
- ✅ Documentation updated with code changes
- ✅ Tests written for new features
- ✅ Security considerations addressed
- ✅ Performance impact evaluated
- ✅ Accessibility maintained (Frontend)

---

## Getting Started

### For Agents

1. **Read your briefing**: `.agents/[your-role]-agent.md`
2. **Review PROJECT_PLAN.md**: Understand overall goals
3. **Check current phase**: See roadmap (PROJECT_PLAN.md Section 4)
4. **Update status**: Create `.agents/status/[your-role]-status.md`
5. **Start work**: Begin with highest priority items
6. **Coordinate**: Check integration points, notify dependent agents

### For Coordinator

1. **Assign work**: Map roadmap phases to agents
2. **Monitor status**: Check `.agents/status/` weekly
3. **Resolve blockers**: Help agents overcome obstacles
4. **Review milestones**: Validate milestone completion
5. **Adjust plan**: Update PROJECT_PLAN.md as needed

---

## Agent Spawning

To activate an agent, use:

```bash
# Example with Claude Code Task tool
task: "Activate [Agent Name] to work on [specific task]"
subagent_type: "general-purpose"  # or specialized type
```

Provide the agent with:
- Link to their briefing file
- Current project context
- Specific task or goal
- Dependencies and blockers

---

## Directory Structure

```
kvm-manager/
├── .agents/                       # Agent system files
│   ├── architecture-agent.md      # Architecture Agent briefing
│   ├── backend-agent.md           # Backend Agent briefing
│   ├── frontend-agent.md          # Frontend Agent briefing
│   ├── guest-agent.md             # Guest Agent Specialist briefing
│   ├── devops-agent.md            # DevOps Agent briefing
│   ├── docs-agent.md              # Documentation Agent briefing
│   ├── status/                    # Agent status reports
│   │   ├── architecture-status.md
│   │   ├── backend-status.md
│   │   ├── frontend-status.md
│   │   ├── guest-agent-status.md
│   │   ├── devops-status.md
│   │   └── docs-status.md
│   ├── decisions/                 # Architecture Decision Records
│   │   ├── 001-frontend-framework.md
│   │   ├── 002-state-management.md
│   │   └── ...
│   └── integration/               # Integration point docs
│       ├── tauri-commands.md      # Backend-Frontend contract
│       ├── guest-protocol.md      # Guest agent protocol spec
│       └── event-schema.md        # Event definitions
├── PROJECT_PLAN.md                # Master project plan
├── AGENTS.md                      # This file
└── ... (rest of project)
```

---

## Success Metrics

Agents are effective when:
- ✅ Milestones are hit on schedule
- ✅ Code quality remains high (CI green, tests passing)
- ✅ Integration between components is smooth
- ✅ Blockers are identified and resolved quickly
- ✅ Documentation stays current
- ✅ Architectural coherence is maintained

---

## Next Steps

1. **Create agent briefing files** in `.agents/`
2. **Initialize status tracking** in `.agents/status/`
3. **Set up decision log** in `.agents/decisions/`
4. **Define first integration points** in `.agents/integration/`
5. **Spawn first agents** to begin Phase 1 work

---

*This agent system is designed to scale with project complexity while maintaining clear ownership and accountability.*
