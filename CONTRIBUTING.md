# Contributing to KVM Manager

Thank you for improving KVM Manager. Keep changes focused, include tests for behavior changes, and
avoid committing host paths, credentials, guest content, or generated local state.

## Local quality checks

Use a clean dependency install before submitting a change:

```bash
npm ci
npm run test:suites
npm run lint
npm run test:truthfulness
npm audit --audit-level=high
npm test
npm run build
```

Run the Rust checks from the repository root:

```bash
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo install cargo-audit --version 0.22.0 --locked
cargo audit --manifest-path src-tauri/Cargo.toml
```

`npm audit` fails on high- or critical-severity JavaScript findings. `cargo-audit` checks the locked
Rust dependency graph against the RustSec advisory database. Address findings before opening a pull
request, or document a narrowly scoped, time-bounded exception in the pull request.

Before opening a pull request, also run the applicable feature quickstart under `specs/` from a
clean worktree. For Spec 002, record only command outcomes, safe operation IDs, fixture connection
IDs, and capability classifications. Never commit credentials, guest content, raw definitions, or
host paths. Changes to distro support, packaging, firmware, privileged forwarding, or release
smoke tests must continue to follow the authoritative Feature 001 guidance.

## Linux testing

Test the supported distro and packaging guidance in the README and feature specifications. For
desktop development under affected Wayland/WebKit combinations, use the documented renderer
workaround only for development; do not weaken the production content policy to work around it.
