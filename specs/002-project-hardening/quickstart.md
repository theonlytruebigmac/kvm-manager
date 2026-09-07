# Quickstart Validation: Trustworthy Core Operations

## Prerequisites

- A clean checkout with the committed JavaScript and Rust dependency locks.
- Node.js 20 or newer and the repository's supported stable Rust toolchain.
- Tauri/Linux development dependencies from `docs/LINUX_SUPPORT.md`.
- Libvirt's test driver plus two disposable fixture definitions containing distinct same-named VMs,
  networks, and storage resources.
- For console checks, two isolated non-production QEMU/libvirt hosts or VMs.

Never run these scenarios against production guests or host firewall state.

## 1. Clean quality baseline

Run:

```bash
npm ci
npx tsc --noEmit
npm run lint
npm test
npm run build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

Expected: every command executes a configured check and passes. Removing a lint configuration,
making a test suite discover zero expected boundary tests, or changing a locked dependency causes a
clear failure.

## 2. Configuration safety corpus

Run the configuration safety suite against the two test-driver fixtures.

Expected:

- at least 100 malformed or format-sensitive values are exercised;
- accepted values appear only as data in the intended field;
- rejected values cause no resource mutation;
- unrelated and unknown XML elements survive targeted edits;
- wrong-root raw documents are rejected before definition.

## 3. Protected diagnostic capture

Run the diagnostic suite with unique sentinel values for credentials, keys, guest content, console
secrets, raw definitions, and host paths.

Expected: no sentinel appears in captured Rust logs, IPC error envelopes, browser logs, or production
recovery output. Safe operation ID, connection ID, target identity, and outcome remain present.

## 4. Two-connection routing

Open fixture A and fixture B, which intentionally contain same-named resources. Select each in turn
and run list, inspect, create, edit, start/stop, snapshot, storage, and network operations that the
test driver supports.

Expected:

- every query, mutation, and refresh affects only the selected fixture;
- changing selection during confirmation invalidates the pending destructive action;
- no operation falls back to `qemu:///system`;
- local-only capabilities are unavailable on remote/test connections with a reason.

Repeat graphical and serial console routing on isolated real hosts. Confirm the console opens only
for the selected connection or is disabled before invocation.

## 5. Failure and partial-operation recovery

Inject connection loss, malformed host XML, missing commands, synchronization failure, failed window
creation, and a failure after the first step of a multi-step mutation.

Expected:

- no command handler or application process panics;
- each failure has a stable category and safe recovery action;
- every mutation reports applied, rejected, rolled back, partial, or unknown;
- unaffected views and saved connections remain usable;
- production recovery UI shows no stack trace or raw host detail.

## 6. Desktop boundary and product truthfulness

Build and launch the production bundle, inspect its effective content policy, then exercise local
assets and enabled console connections. Inspect connection status, host versions, unfinished
features, About/project metadata, and support links.

Expected:

- the policy is explicit and contains only justified sources;
- local assets and enabled consoles still function;
- displayed runtime facts match queried state;
- incomplete operations cannot be invoked;
- no template project/owner placeholders remain.

## Evidence

Record the commit, clean-worktree status, fixture IDs, isolated host family/version, each command's
result, and any expected unavailable capability. Do not record protected values or host paths.

### Local boundary evidence (2026-09-06)

- Rust unit and integration suites: **PASS** (`cargo test --locked --manifest-path src-tauri/Cargo.toml`).
- Strict Rust static checks: **PASS** (`cargo fmt --check` and locked Clippy with warnings denied).
- Configuration corpus, diagnostic-redaction, XML-import, and destructive-confirmation suites: **PASS**.
- Frontend required-suite guard, TypeScript production build, and project checks: **PASS**.
- Production Vite build: **PASS**; the existing bundle-size advisory remains reported and is not treated as a failure.
- No fixture credentials, protected sentinels, or host paths were recorded in this evidence.

### Connection-routing evidence (2026-09-06)

- Isolated test-driver routing suite: **PASS** (`cargo test --locked --manifest-path src-tauri/Cargo.toml connection_routing`).
- Fixture connection IDs: `fixture-a` and `fixture-b`; same-named VM identities remained distinct and each captured operation retained its selected fixture after a selection change.
- The suite verified query, mutation, refresh, disconnect, duplicate-name, and no-local-fallback behavior.
- Local-only console and host-device capabilities are reported unavailable for non-local scopes before their adapters are invoked.
- The real-host graphical and serial-console scenario remains an operator-run validation because this workspace has no isolated console host attached. Record only the selected connection ID, capability result, and outcome when it is run.

### Recovery and product-boundary evidence (2026-09-06)

- Safe-failure, ErrorBoundary, fault-recovery, mutation-outcome, and isolated connection-routing suites: **PASS**. Rejected IPC payloads are normalized at the frontend boundary and production recovery screens expose only a classified summary and recovery action.
- Required frontend-suite guard, project-truthfulness checks, full frontend test suite, TypeScript check, and production web build: **PASS**.
- A locked frontend dependency install (`npm ci`) and frontend lint with **zero warnings**: **PASS**.
- Production content-policy and local console regression coverage: **PASS** through the frontend suite. The policy, metadata, and incomplete-control assertions passed without recording protected values.
- Locked Rust format, Clippy with warnings denied, and full Rust suite: **PASS**. JavaScript audit found no high- or critical-severity vulnerabilities.
- Production Tauri Debian bundle: **PASS** (`npm run tauri build -- --bundles deb`); the package
  artifact was produced. AppImage and RPM packaging are verified by their Feature 001
  platform-specific release scenarios.
- The Vite production build emits the existing large-chunk advisory for the main bundle; it is a size optimization advisory, not a failed or skipped check.
- The real two-host graphical and serial-console scenario remains pending because no isolated console host is attached to this workspace. The repository worktree also contained pre-existing changes before validation, so clean-worktree evidence cannot be claimed in this run.

### Clean snapshot evidence (2026-09-06)

- A disposable local snapshot of the current source was committed only inside its temporary
  validation directory, then verified clean before and after the locked install, frontend quality
  suite, strict Rust checks, full Rust suite, and production Debian bundle build.
- The snapshot passed `npm ci`, required-suite and truthfulness checks, TypeScript, zero-warning
  lint, frontend tests/build, locked Rust format/Clippy/tests, and the Feature 001 reproducible
  build-and-regression commands.
- The snapshot identified generated noVNC and SPICE bundle files as the only post-build worktree
  residue. They are now ignored as generated artifacts; the rebuilt snapshot worktree was clean.
- The Vite main-bundle size advisory remains documented as an expected, non-failing optimization
  notice. Real two-host console validation remains separately pending.
