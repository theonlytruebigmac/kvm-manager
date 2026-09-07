# Quickstart: Validate First-Run Onboarding

Use an isolated host or disposable VM. Do not validate storage creation against a production host.
Follow the matching setup guidance in [Linux Support](../../docs/LINUX_SUPPORT.md), not a command
copied from another distribution family.

## Automated checks

From the repository root:

```bash
cd src-tauri && cargo test --test host_readiness --test ipc_contracts --test mutation_outcomes
cargo test --test readiness_repair --test destructive_confirmation
cd .. && npm test
npm run lint
npm run build
```

Run the focused feature tests added by implementation as well: distribution fixture coverage,
storage readiness/preflight fixture coverage, VM-creation no-mutation rejection tests, and
onboarding/wizard component tests.

## Manual scenarios

Interactive repair extension:

- On an isolated supported local-system host, open failed rows and verify previews name the
  connection, distribution, privilege, and effects. Cancel once and confirm no process runs; then
  authorize and verify readiness refreshes.
- Repeat on remote, local-session, test, and best-effort profiles. Only manual/navigation guidance
  may appear and direct execution through IPC must be rejected.
- Verify unavailable/cancelled authorization and package-manager failure return safe outcomes with
  no stdout, stderr, commands, paths, or credentials in UI or logs.

1. Connect to a disposable local-system libvirt connection with no active storage pool. On app
   start or before opening the VM wizard, verify onboarding identifies the selected connection,
   says storage is unavailable, and offers inspect/setup without mutating the host.
2. Cancel storage setup. Confirm no pool, directory, volume, or VM is created.
3. Use the reviewed storage setup flow to create or activate one disposable pool. Confirm the
   connection and target are visible before the final action, then verify readiness refreshes.
4. Create at least two active disposable pools with different free capacity. Start ISO, network,
   and manual VM flows; verify only actual connection-owned pool choices appear, the chosen UUID
   is shown on review, and no choice is inferred from its name.
5. Select a disk larger than a pool's available capacity. Confirm the wizard blocks review/submit
   and backend creation leaves no new volume or domain.
6. Select UEFI Secure Boot and TPM for a Windows 11 profile. On a capable host, verify review
   passes. Repeat with secure firmware or TPM unavailable; verify the specific failure appears
   before any disk or VM exists.
7. Begin a review, switch to a distinct system/session/test/remote connection, then continue.
   Verify stale pool/readiness results are discarded and the prior connection cannot receive a
   volume.
8. Repeat the relevant setup and Windows profile checks on every supported Linux family affected
   by the release. For remote/test/session connections, verify the UI does not offer untrue local
   package, service, permission, or path instructions.

Record distribution, connection scope, libvirt URI, selected pool state, firmware/TPM availability,
and outcomes in the release evidence. Do not record paths, credentials, guest contents, or raw
libvirt output.

## Release evidence

Automated validation on 2026-09-06:

- Rust formatting and the full Rust test suite: passed (80 tests across unit and integration
  targets).
- Frontend component tests (20 tests), lint, type-check, and production build: passed.
- Interactive repair allowlist, scope gating, fixed process inputs, manual cancellation, privilege
  preview, and safe fallback coverage: passed.
- Libvirt test-driver no-mutation rejection: covered by `mutation_outcomes`.
- Pool-state, distribution-scope, firmware, Secure Boot, TPM, and safe-output fixtures: covered by
  `host_readiness` and `ipc_contracts`.

The physical multi-distribution/remote-host matrix remains a release-environment check. It cannot
be truthfully recorded from a single development host; complete the eight manual scenarios above
on disposable hosts before release.

Read-only local-system observation recorded on 2026-09-06: one active pool and one active virtual
network were reported, and Q35 domain capabilities exposed UEFI and TPM. Secure firmware with
enrolled keys was not advertised, so the Windows 11 preflight is expected to block that profile
until the host firmware configuration is corrected. No host mutation was performed.

The local-session connection was also inspected read-only and reported zero pools and zero
networks. Scope fixtures verify that this result receives no local-system package or permission
guidance. Remote and the four physical distribution-family runs still require disposable release
hosts.
