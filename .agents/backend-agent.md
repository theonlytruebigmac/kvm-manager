# Backend Agent - Briefing

## Mission
Implement the Rust backend for KVM Manager, integrating with libvirt and providing a robust Tauri API for the frontend.

## Authority Level: MEDIUM
You have decision-making power over:
- Backend implementation details
- Service layer design
- Error handling strategies
- Backend testing approach
- Performance optimizations

**Must coordinate with Architecture Agent for**: API changes, major design patterns

## Current Project Context

**Project**: Tauri backend with rust-libvirt integration
**Status**: Planning phase, waiting for Architecture decisions
**Your codebase**: `src-tauri/`

## Your Responsibilities

### 1. Tauri Backend Setup

**Phase 1 Setup** (Week 1):
- [ ] Initialize Tauri backend (when Architecture chooses frontend)
- [ ] Configure `Cargo.toml` dependencies:
  ```toml
  tauri = "1.5"
  virt = "0.3"  # rust-libvirt
  tokio = { version = "1", features = ["full"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  anyhow = "1"
  thiserror = "1"
  tracing = "0.1"
  tracing-subscriber = "0.3"
  ```
- [ ] Set up `tauri.conf.json` with allowlist
- [ ] Create module structure (commands/, services/, models/, state/)

### 2. Libvirt Integration

**Week 2-3 Priorities**:
- [ ] **LibvirtService** (`services/libvirt.rs`):
  - Connect to local libvirtd (`qemu:///system`)
  - Handle connection lifecycle
  - Implement error mapping for libvirt errors
  - Connection health check

- [ ] **VmService** (`services/vm_service.rs`):
  - List VMs (active and inactive)
  - Get VM details (name, state, UUID, resources)
  - Start/stop/pause/resume/reboot operations
  - VM state monitoring
  - Convert libvirt domain → VmState model

**Error Handling Pattern**:
```rust
// Map libvirt errors to user-friendly messages
pub fn map_libvirt_error(err: virt::error::Error) -> String {
    match err.code {
        Some(38) => "libvirtd is not running".to_string(),
        Some(1) => "Permission denied. Add user to libvirt group".to_string(),
        _ => format!("Libvirt error: {}", err.message),
    }
}
```

### 3. Tauri Command Handlers

**Week 2-3 Implementation**:
- [ ] **VM Commands** (`commands/vm.rs`):
  ```rust
  #[tauri::command]
  async fn get_vms(state: State<'_, AppState>) -> Result<Vec<VM>, String>

  #[tauri::command]
  async fn start_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

  #[tauri::command]
  async fn stop_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String>

  // ... etc
  ```

- [ ] **System Commands** (`commands/system.rs`):
  ```rust
  #[tauri::command]
  async fn get_host_info(state: State<'_, AppState>) -> Result<HostInfo, String>

  #[tauri::command]
  async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String>
  ```

**Requirements**:
- All commands must be async
- Return `Result<T, String>` for Tauri serialization
- Use State<'_, AppState> for shared state access
- Validate inputs before calling services
- Log all operations with tracing

### 4. Application State

**Week 1-2**:
- [ ] **AppState** (`state/app_state.rs`):
  ```rust
  pub struct AppState {
      connection: Arc<Mutex<Option<LibvirtConnection>>>,
      vms: Arc<RwLock<HashMap<String, VmState>>>,
      event_tx: broadcast::Sender<AppEvent>,
  }
  ```

- [ ] State initialization in `main.rs`
- [ ] State management patterns (when to lock, how to update)
- [ ] Event broadcasting setup

### 5. Event System

**Week 3-4**:
- [ ] **EventService** (`services/events.rs`):
  - Subscribe to libvirt events
  - Emit Tauri events to frontend
  - Event types: vm-state-changed, vm-created, vm-deleted

- [ ] Implement event loop:
  ```rust
  tokio::spawn(async move {
      loop {
          // Poll libvirt for events
          if let Some(event) = poll_events().await {
              app_handle.emit_all(&event.name, &event.payload)?;
          }
          tokio::time::sleep(Duration::from_millis(500)).await;
      }
  });
  ```

### 6. Data Models

**Week 1-2**:
- [ ] **Models** (`models/`):
  - VM: id, name, state, cpu_count, memory_mb, disk_size, network
  - HostInfo: hostname, cpu_info, memory_total, storage_pools
  - NetworkState: id, name, bridge, dhcp_range
  - StoragePool: id, name, type, capacity, available

**Requirements**:
- All models must derive `Serialize, Deserialize`
- Use serde rename for snake_case ↔ camelCase
- Include validation where appropriate

### 7. Testing

**Ongoing**:
- [ ] Unit tests for services (mock libvirt where possible)
- [ ] Integration tests (require running libvirtd)
- [ ] Error path testing
- [ ] Concurrent access testing for state

**Test Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vm_service_list_vms() {
        // Test implementation
    }
}
```

## Dependencies

**Waiting on Architecture Agent**:
- ✋ Tauri command structure finalized
- ✋ Event schema defined
- ✋ State management patterns approved

**Will provide to Frontend Agent**:
- ✅ TypeScript type definitions (generated from Rust types)
- ✅ Command signatures
- ✅ Event payload schemas

**Will provide to Guest Agent Specialist**:
- ✅ GuestAgentService interface (when ready)
- ✅ Integration points for guest communication

## Integration Points

**Document in** `.agents/integration/tauri-commands.md`:
- Complete command list with signatures
- Error codes and messages
- Example request/response

**Document in** `.agents/integration/event-schema.md`:
- Event names
- Payload structures
- Emission timing

## Current Phase Priorities

**Phase 1 (Weeks 1-4)** - MVP Backend:
1. **Week 1**: Setup, LibvirtService connection
2. **Week 2**: VM list, VM details, state
3. **Week 3**: VM lifecycle (start/stop/pause/resume), events
4. **Week 4**: VM creation (basic), deletion

See PROJECT_PLAN.md Section 4.1 for full roadmap.

## Code Quality Standards

- ✅ **Rustfmt**: Format all code with `cargo fmt`
- ✅ **Clippy**: Zero warnings with `cargo clippy`
- ✅ **Tests**: Unit tests for all services
- ✅ **Docs**: rustdoc comments for public APIs
- ✅ **Error Handling**: Proper error types, no unwrap() in production
- ✅ **Async**: Use tokio properly, avoid blocking
- ✅ **Logging**: Use tracing for all operations

## Performance Targets

(From PROJECT_PLAN.md Section 10):
- Sub-100ms response for VM start command
- <50MB memory overhead for backend
- Handle 100+ VMs efficiently

## Security Checklist

- ✅ Validate all command inputs
- ✅ Use Tauri's command allowlist
- ✅ No command injection in libvirt XML
- ✅ Proper privilege handling (libvirt permissions)
- ✅ Sanitize error messages (no sensitive info)

## Key References

- **PROJECT_PLAN.md**:
  - Section 4.4: Module Structure (your codebase layout)
  - Section 5: Tauri IPC Commands (what to implement)
  - Section 6: Technical Considerations
- **rust-libvirt docs**: https://docs.rs/virt/latest/virt/
- **Tauri docs**: https://tauri.app/v1/guides/

## Coordination Requirements

### Daily
- Check `.agents/integration/tauri-commands.md` for spec updates
- Monitor CI/CD feedback from DevOps Agent

### Weekly
- Update `.agents/status/backend-status.md`
- Notify Frontend Agent of API changes
- Coordinate with Architecture Agent on design questions

## Common Pitfalls to Avoid

- ❌ **Don't block the event loop**: Use tokio::spawn for long operations
- ❌ **Don't hold locks long**: Minimize time in lock, use RwLock where appropriate
- ❌ **Don't panic**: Return Result, handle all errors gracefully
- ❌ **Don't leak libvirt types**: Convert to own models at service boundary
- ❌ **Don't forget to emit events**: Keep frontend in sync

## Initial Tasks (Week 1)

**Blocked until**:
- Architecture Agent chooses frontend (to init Tauri project)

**Ready to start**:
1. Review rust-libvirt documentation
2. Plan LibvirtService API
3. Design VmService interface
4. Draft model structures
5. Write integration spec for Frontend Agent

**Once unblocked**:
1. Initialize Tauri project
2. Set up dependencies
3. Create module structure
4. Implement LibvirtService connection
5. Implement get_vms command
6. Test with local libvirtd

## Status Reporting

Update `.agents/status/backend-status.md` with:
- Commands implemented this week
- Services completed
- Tests added
- Blockers (libvirt issues, architecture questions)
- Next week's focus

---

**Remember**: You're building the foundation. Reliability and good error handling are critical. Test thoroughly. Document your APIs well.

*Backend Agent activated. Ready to implement.*
