# Data Model: Modern Linux Readiness

## Host Readiness Report

| Field | Meaning | Validation |
|-------|---------|------------|
| `checked_at` | Time of the capability probe | Always present |
| `connection_uri` | Evaluated libvirt connection | Defaults to local system connection |
| `distribution` | Detected Distribution Profile | Always present, including best-effort profiles |
| `overall_state` | Ready, degraded, or unavailable | Derived from required capabilities |
| `capabilities` | Individual capability results | One result per required capability |

## Capability Result

| Field | Meaning | Validation |
|-------|---------|------------|
| `kind` | Distribution support, libvirt access, QEMU emulator, KVM, UEFI, Secure Boot, or forwarding privilege | Closed enum |
| `state` | Available, unavailable, warning, or unknown | Closed enum |
| `summary` | Short user-facing result | No secrets or raw credentials |
| `remediation` | Current distribution-specific recovery guidance | Optional; present when not available |
| `details` | Safe diagnostic context | Must omit secrets and guest contents |

## Firmware Candidate

| Field | Meaning | Validation |
|-------|---------|------------|
| `boot_mode` | UEFI or Secure Boot | Closed enum |
| `code_path` | Firmware code image selected by libvirt | Existing, readable regular file |
| `vars_template_path` | NVRAM variable-store template | Existing, readable regular file when required |
| `source` | Libvirt auto-selection or verified fallback | Closed enum |

## Port-Forward Rule

| Field | Meaning | Validation |
|-------|---------|------------|
| `protocol` | Transport protocol | TCP or UDP only |
| `host_port` | Exposed host port | Integer 1 through 65535 |
| `guest_address` | IPv4 or IPv6 destination | Parsed IP address; never a raw shell fragment |
| `guest_port` | Guest destination port | Integer 1 through 65535 |
| `network_id` | Selected local virtual network | Must resolve through libvirt and contain the destination |
| `state` | Requested, present, absent, failed, insufficient privilege, or unavailable | Derived from executor result and installed artifact |

## Forwarding Authorization Request

| Field | Meaning | Validation |
|-------|---------|------------|
| `operation` | Add or remove one Port-Forward Rule | Closed enum |
| `rule` | Requested network-scoped forwarding rule | Fully validated before authorization |
| `authorization_state` | Not required, pending, granted, denied, or unavailable | Derived from the operating system |
| `package_support` | Native authorized helper or unavailable portable build | Derived from the installed artifact |

## Distribution Profile

| Field | Meaning | Validation |
|-------|---------|------------|
| `family` | Arch/CachyOS, Debian/Ubuntu, Fedora/RHEL-compatible, openSUSE, or best-effort | Closed enum |
| `package_manager` | Command family used in guidance | Derived from OS metadata |
| `guidance` | Setup, permission, service, and firmware instructions | Versioned application content |

## Relationships and State Transitions

- A Host Readiness Report contains many Capability Results.
- UEFI creation selects one compatible Firmware Candidate after the relevant readiness result is
  available.
- A Port-Forward Rule transitions from requested to present only after exact application is
  confirmed; it transitions to failed or insufficient privilege without changing unrelated rules.
- Each Forwarding Authorization Request covers exactly one operation on one Port-Forward Rule. A
  portable artifact transitions it directly to unavailable without requesting authorization.
