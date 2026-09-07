# Tauri Contract: Readiness and Forwarding

## `get_host_readiness`

**Purpose**: Return a non-destructive readiness assessment and distribution profile for the selected
local connection.

**Input**: Optional connection identifier; defaults to the active local connection.

**Output**: A Host Readiness Report containing the detected support-matrix profile, overall state,
and independent capability results.

**Errors**: The command itself returns a report for expected capability failures. It only returns an
error for an internal failure that prevents producing any report.

**Distribution behavior**: Supported profiles are Arch/CachyOS, Debian/Ubuntu,
Fedora/RHEL-compatible, and openSUSE. Other distributions return a best-effort profile with safe
generic diagnostics.

## `get_firmware_candidates`

**Purpose**: Return verified UEFI and Secure Boot choices appropriate to the selected connection.

**Input**: Requested architecture and machine family.

**Output**: Zero or more Firmware Candidates. An empty result is valid and is accompanied by a
readiness remediation message.

## `add_port_forward`

**Purpose**: Add one validated host-to-guest forwarding rule.

**Input**: Selected local virtual network, protocol, host port, parsed guest address, and guest
port.

**Output**: The requested Port-Forward Rule with `present`, `failed`, `insufficient privilege`, or
`unavailable` state.

**Guarantees**: Invalid input is rejected before execution; no user-controlled input is interpreted
as shell syntax; destinations not associated with the selected network are rejected; a failed
request does not remove or alter an unrelated rule.

**Authorization**: On supported native packages, the request is sent to a root-owned, narrowly
scoped helper through its operating-system policy action. The helper repeats canonical input and
network-membership validation and may add, remove, or inspect only rules it owns. On portable builds
without that component, the command returns `unavailable` and describes the native-package
requirement; it does not request the desktop application to run with broad administrator privileges.

## `remove_port_forward`

**Purpose**: Remove the exact matching forwarding rule.

**Input**: The same canonical network-scoped fields used to add the rule.

**Output**: The requested Port-Forward Rule with `absent`, `failed`, `insufficient privilege`, or
`unavailable` state.

**Guarantees**: It removes only the exact matching rule and reports when no matching rule exists.
The helper never flushes, edits, or inspects rules outside its application-owned scope.
