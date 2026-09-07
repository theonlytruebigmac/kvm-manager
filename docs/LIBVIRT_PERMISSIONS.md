# Libvirt Permissions Troubleshooting

## Privileged readiness actions

KVM Manager runs as the desktop user. Supported readiness repairs launch only predefined package
operations through `/usr/bin/pkexec`; the UI cannot supply a program, argument, package, service,
environment value, or path. The desktop policy agent owns authentication, so passwords do not
cross the application boundary or appear in logs.

If no policy agent is running, authorization is cancelled, or the distribution is not verified,
use the displayed manual guidance. Do not launch KVM Manager as root. Group changes require a new
login session and are intentionally not automated.

Libvirt intentionally runs guests under a distribution-defined service account and often an
SELinux or AppArmor policy. A file can be world-readable and still be inaccessible because a
private parent directory cannot be traversed or mandatory access control rejects it.

## ISO in Downloads reports “Permission denied”

Use **Import into selected storage pool** in the VM wizard. KVM Manager opens the downloaded ISO
read-only and transfers it through a libvirt stream. Libvirt creates the destination volume with
the selected pool's ownership, labeling, and access policy; the downloaded source remains intact.
An existing volume is never overwritten implicitly.

Do not solve this by changing ownership to a guessed account, recursively changing a home
directory, making `$HOME` traversable, disabling SELinux/AppArmor, or running KVM Manager as root.
Those changes expose unrelated user data and are not portable between distributions.

## Connection and pool checks

- Confirm the intended system, session, test, or remote connection is selected.
- Review Host Readiness for that connection.
- Select an active pool with enough available capacity. No `default` pool is required.
- If storage setup is needed, review the connection, target, autostart setting, and activation
  effect in the Storage wizard before confirming.
- For a remote connection, the destination pool and QEMU policy live remotely, while the import
  source is read by the desktop application and streamed to that host.

## Administrator diagnostics

If a libvirt-managed volume is still rejected, inspect service logs and mandatory-access-control
audit records using commands documented by the host distribution. Service unit names, QEMU
users/groups, log locations, and repair commands vary. See [Linux Support](LINUX_SUPPORT.md) for
the verified family matrix and do not copy another distribution's ownership commands.
