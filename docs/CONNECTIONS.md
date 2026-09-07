# Connections and Capability Limits

Every query and mutation captures the selected connection at command entry. Resource names are
therefore scoped by connection ID: the same VM name on two hosts is not the same target.

## Supported scopes

- Local system and local session connections support resource management according to libvirt
  permissions and host configuration.
- Remote and test connections support only features whose selected connection can perform them.
- Graphical/serial console, host-device passthrough, SR-IOV, PCI, USB, mediated devices, and
  similar host-local operations are disabled when the connection capability says they are not
  available. The UI shows the stable reason instead of falling back to the local host.

Connection switching invalidates pending destructive confirmations. Refreshes and mutation follow-up
queries remain on the connection captured for the operation.

## Troubleshooting

Reconnect the selected connection and retry only when the displayed recovery action says it is
safe. For permission or capability failures, use the Linux-specific guidance in
[LINUX_SUPPORT.md](LINUX_SUPPORT.md) and [LIBVIRT_PERMISSIONS.md](LIBVIRT_PERMISSIONS.md). For
console validation, use isolated non-production hosts and confirm that the selected connection ID
matches the console outcome.
