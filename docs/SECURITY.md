# Security and Recovery

KVM Manager controls host virtualization resources. Use it only with connections and guests you
are authorized to manage. Never run routine desktop sessions as root.

## Safe diagnostics

The application records only a correlation ID, selected connection ID, safe resource identity,
operation outcome, and stable reason code. It does not include credentials, keys, guest content,
raw libvirt XML, external-command output, console secrets, or local paths in IPC failures,
diagnostics, or production recovery screens.

If an operation fails, retain the displayed operation ID and failure category for support. Do not
paste browser console output, raw XML, or host logs containing protected values into an issue.

## Destructive operations

Deletion, force-stop, device changes, host-device changes, and similar high-impact actions show
the selected connection, exact target, and material effect before execution. The confirmation token
is single-use and expires when the selected connection or target changes. Re-open the confirmation
dialog rather than retrying a stale request.

## Recovery behavior

Failures are classified as unavailable, invalid input, conflict, unauthorized, integration,
unsupported, partial, or internal. The interface provides only safe recovery guidance. For a
`partial` or `unknown` outcome, inspect the resource on the same selected connection before trying
another mutation; never assume the previous action completed or rolled back.

See [CONNECTIONS.md](CONNECTIONS.md) for capability and connection-scope limits.
