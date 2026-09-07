# Diagnostic Contract: Protected Operations

## Allowed event fields

A privileged-operation diagnostic event may contain only:

- event name from a reviewed enum;
- operation correlation ID;
- connection ID and non-sensitive display label;
- resource kind and safe stable identifier;
- mutation outcome;
- stable failure or capability reason code;
- elapsed duration and retry count;
- application version.

## Prohibited event fields

Events, spans, error payloads, and production recovery views must not contain:

- passwords, passphrases, tokens, private keys, or SSH public-key content;
- cloud-init/user-data, guest commands, guest file content, or raw guest-agent responses;
- VNC/SPICE credentials or credential-bearing URLs;
- raw VM, network, filter, storage, secret, or capability XML;
- local source, destination, disk, ISO, firmware, device, archive, database, or temporary paths;
- unfiltered external-command stderr or nested error chains that may include the above;
- frontend or Rust stack traces in user-visible production output.

## Event outcomes

Each mutation emits at most one terminal diagnostic outcome: `applied`, `rejected`, `rolled_back`,
`partial`, or `unknown`. A terminal `partial` or `unknown` event includes a stable reconciliation
reason code, never raw residual data.

## Verification contract

Captured-output tests seed a unique sentinel for every protected category, exercise both success and
failure paths, and assert that no sentinel or encoded equivalent appears. Tests also assert that the
operation ID, safe target identity, connection ID, and outcome remain available.
