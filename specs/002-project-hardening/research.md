# Research: Trustworthy Core Operations

## Decision 1: Capture one immutable connection context per operation

**Decision**: Resolve the selected connection once at Tauri command entry and pass an operation
context containing the connection ID, safe display name, URI classification, capabilities, and
connection handle through the service call and refresh path.

**Rationale**: Libvirt supports local, session, test, and multiple remote URI forms. A single captured
context prevents selection races and makes it impossible for an operation to silently use the fixed
local connection after the UI selected another host.

**Alternatives considered**:

- Keep the fixed `qemu:///system` service and use saved connections only for migration: rejected
  because the UI exposes connection management and active status, creating a false product claim.
- Read the active connection separately in every service method: rejected because selection may
  change between a mutation and its refresh or confirmation.
- Remove remote connections entirely: rejected because tested remote management is valuable; each
  unsupported feature can instead advertise an unavailable capability.

## Decision 2: Migrate XML boundaries incrementally using structural events

**Decision**: Introduce an event-based XML reader/writer for escaping text and attributes, validating
document roots, and transforming specific owned subtrees. Preserve unknown elements and namespaces
when modifying an existing libvirt definition. Start with network, storage, filter, and VM creation
generators, then migrate VM edit paths in bounded groups.

**Rationale**: Current string interpolation allows field values to alter document structure, while
substring parsing depends on quote style, order, and formatting. Libvirt definitions are extensive
and evolve over time, so a complete hand-maintained domain schema would discard valid data the app
does not own. An event stream safely escapes emitted data and can preserve untouched events.

**Alternatives considered**:

- Continue interpolation with a shared escape function: rejected because it does not fix structural
  parsing or fragile substring edits.
- Deserialize every libvirt document into a complete application-owned object model: rejected due to
  schema breadth, extension namespaces, and the risk of dropping unknown valid content.
- Use regular expressions for targeted replacement: rejected because XML nesting and namespaces are
  not regular-string contracts.

## Decision 3: Use stable safe error envelopes across IPC

**Decision**: Public commands return a serializable failure envelope with a stable category, safe
summary, operation context, mutation outcome, retryability, and optional recovery action. Detailed
source errors remain internal and are logged only after protected-field filtering.

**Rationale**: Returning arbitrary `String` values couples UI behavior to wording and may expose
libvirt responses or paths. Stable categories let the UI render correct recovery for unavailable,
invalid, conflict, authorization, integration, and partial-application states.

**Alternatives considered**:

- Keep display strings and add frontend pattern matching: rejected as brittle and unable to prove
  sensitive values are absent.
- Return complete error chains to the UI: rejected because source errors may contain host paths,
  definitions, credentials, or transport details.
- Collapse all failures into one generic message: rejected because it removes actionable context and
  violates observable-failure requirements.

## Decision 4: Make diagnostic fields allowlisted rather than redact-after-formatting

**Decision**: Diagnostic APIs accept only operation category, safe target identity, connection ID,
outcome, correlation ID, and enumerated reason codes. Protected values never enter formatted log
events. Captured-output tests additionally scan for sentinel secrets, guest content, and paths.

**Rationale**: Post-hoc text replacement cannot reliably find nested, encoded, or partially printed
secrets. Allowlisting provides a reviewable boundary and makes regressions testable.

**Alternatives considered**:

- Lower sensitive messages from info to debug: rejected because debug logs are commonly collected
  during troubleshooting.
- Redact known field names in final strings: rejected because arbitrary error chains and raw XML do
  not preserve field names.
- Disable diagnostics around privileged work: rejected because safe operation identity and outcomes
  are needed to troubleshoot partial changes.

## Decision 5: Derive desktop content policy from enabled feature traffic

**Decision**: Inventory production asset, IPC, VNC, and SPICE traffic; define the narrowest explicit
content policy that permits those sources; and add a production check that fails on an unrestricted
policy or an unjustified source.

**Rationale**: Tauri's WebView/IPC boundary is capability-controlled, but an unrestricted content
policy expands the impact of injected frontend content. Console WebSocket requirements must be
represented deliberately rather than disabling the policy globally.

**Alternatives considered**:

- Leave the policy unrestricted because assets are local: rejected because local WebView content is
  still part of the privileged IPC trust boundary.
- Add a broad wildcard policy: rejected because it does not materially constrain connectable or
  executable sources.
- Disable embedded consoles: rejected because the current product depends on them and narrow local
  connection rules can be tested.

## Decision 6: Fail quality checks closed and lock application dependencies

**Decision**: Commit JavaScript and Rust application lock files, use locked installs in clean-checkout
validation, add an explicit frontend lint command, reject unexpectedly empty test suites, and retain
strict Rust formatting and static analysis.

**Rationale**: The repository currently ignores the Rust lock file even though it is an application,
and the quality workflow has no frontend lint step. A passing optional or empty check is not evidence
that the intended check ran.

**Alternatives considered**:

- Rely only on compatible version ranges: rejected because clean builds may resolve different
  dependency graphs.
- Keep lint optional until configuration exists: rejected because optional execution silently hides
  the missing gate.
- Require coverage percentages immediately: deferred because meaningful privileged-boundary coverage
  is more valuable than an arbitrary global percentage during initial characterization.

## Decision 7: Validate routing with isolated libvirt test connections

**Decision**: Use distinct libvirt test-driver fixtures for automated routing semantics, plus
documented isolated QEMU/libvirt hosts for console and integration behavior the test driver cannot
exercise.

**Rationale**: Two fixtures with same-named resources can prove the operation context selects the
intended connection without touching production VMs. Real-host validation remains necessary for
console transports, device operations, and disconnection behavior.

**Alternatives considered**:

- Mock every libvirt call: rejected as the only method because mocks cannot prove URI routing.
- Test only against the developer's system daemon: rejected because resource mutations are unsafe and
  nondeterministic.
- Require two physical hosts for every unit test: rejected because it would make routine regression
  checks inaccessible.

## Sources

- Tauri security model and WebView/IPC trust boundary: https://v2.tauri.app/security/
- Libvirt connection URI forms and test driver: https://libvirt.org/uri.html
- Libvirt XML schema/documentation index: https://libvirt.org/docs.html
- Libvirt domain XML extensibility: https://www.libvirt.org/formatdomain
- Event-based XML reader/writer candidate documentation: https://docs.rs/quick-xml/latest/quick_xml/
- Cargo application manifest and lock-file guidance: https://doc.rust-lang.org/cargo/guide/
