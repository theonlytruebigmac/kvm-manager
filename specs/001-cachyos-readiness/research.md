# Research: Modern Linux Readiness

## Decision: Maintain an explicit, tested distribution support matrix

**Rationale**: “Most modern Linux distributions” must translate to a reviewable contract. The
initial verified matrix is Arch/CachyOS, Debian/Ubuntu LTS, Fedora/RHEL-compatible, and openSUSE.
These families cover the package and service differences relevant to Tauri, libvirt, QEMU, UEFI,
and firewall integration. Other distributions receive best-effort diagnostics, not unverified
installation instructions.

**Alternatives considered**:

- Claim all Linux distributions: rejected because package names, service layouts, firmware paths,
  security policies, and graphics dependencies differ and cannot be verified generically.
- Support only CachyOS: rejected because the app's desktop and libvirt stack are portable across
  major Linux families.

**Sources**: [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/),
[libvirt supported host platforms](https://www.libvirt.org/platforms.html),
[libvirt distribution-package guidance](https://www.libvirt.org/compiling.html).

## Decision: Use libvirt firmware auto-selection before path fallbacks

**Rationale**: Current Arch `edk2-ovmf` ships QEMU firmware descriptors and uses
`/usr/share/edk2/x64/OVMF_CODE.4m.fd` naming. Libvirt supports UEFI firmware auto-selection and
copies the selected NVRAM master store at guest startup. Querying libvirt domain capabilities and
using its selected firmware avoids stale hard-coded paths.

**Alternatives considered**:

- Maintain a longer list of CachyOS paths: rejected as package layouts can change again.
- Keep the current `_4M`/legacy paths: rejected because they do not match current Arch packaging.

**Sources**: [Libvirt domain XML firmware documentation](https://libvirt.org/formatdomain.html),
[libvirt domain capabilities API](https://libvirt.org/html/libvirt-libvirt-domain.html),
[current Arch edk2-ovmf file list](https://archlinux.org/packages/extra/any/edk2-ovmf/files/).

## Decision: Represent host readiness as independent capability results

**Rationale**: A libvirt daemon can be reachable without a usable QEMU emulator, firmware, or
permission to perform all requested operations. Independent results make remediation accurate and
allow the desktop application to start in a degraded state.

**Alternatives considered**:

- Fail during application-state initialization: rejected because it prevents diagnostics.
- Use a single ready/not-ready boolean: rejected because it obscures recovery actions.

## Decision: Use an authorized, network-scoped native forwarding helper

**Rationale**: Libvirt network configuration does not model general inbound host-to-guest DNAT
rules. A normal desktop user cannot safely alter host firewall state, while elevating the entire
Tauri application would violate least privilege. A root-owned helper at a fixed installed path,
selected by one operating-system policy action, accepts only one validated network-scoped request,
checks that the guest address belongs to the selected libvirt network, and manages only
application-owned nftables tables. The helper must not make authorization decisions from Polkit's
`command_line` variable; it repeats validation after authorization. The desktop app remains
unprivileged. The Arch/CachyOS, Debian, and RPM native packages install this component; the
portable AppImage without the helper reports the feature as unavailable.

**Alternatives considered**:

- `sh -c` plus iptables strings: rejected because any unvalidated component is command injection.
- Disable forwarding entirely: rejected because it removes an advertised management capability.
- Run the desktop application as root: rejected because it elevates unrelated VM and host commands.
- Build a general root daemon: rejected; the helper exposes only add, remove, and inspect for an
  application-owned table, not a general administrative API.

**Sources**: [Polkit system architecture](https://polkit.pages.freedesktop.org/polkit/polkit.8.html),
[pkexec action-path and command-line guidance](https://polkit.pages.freedesktop.org/polkit/pkexec.1.html),
[nftables rule handles](https://wiki.nftables.org/wiki-nftables/index.php/Simple_rule_management),
[nftables JSON ruleset output](https://wiki.nftables.org/wiki-nftables/index.php/Operations_at_ruleset_level),
and [CachyOS QEMU and VMM setup](https://wiki.cachyos.org/virtualization/qemu_and_vmm_setup/).

## Decision: Gate publication on artifact-specific quality and smoke tests

**Rationale**: Standard build and static checks can run in a reproducible Linux runner. Native
packages and AppImages have different installation and privilege capabilities, and the
virtualization stack, package layout, firmware descriptors, socket activation, and firewall behavior
must be validated on designated support-matrix runners. The existing workflow combines a
self-hosted label with Debian-specific installation commands, making neither guarantee reliable.
GitHub Actions supports a reusable workflow called directly by CI and release jobs, and `needs`
causes a publication job to be skipped when a required predecessor fails or is skipped.

**Alternatives considered**:

- Run only build checks: rejected because no host-management behavior is exercised.
- Require every contributor to run a physical-host test: rejected because it is not reproducible or
safe; use a designated isolated CachyOS runner instead.
- Publish as soon as the bundle command succeeds: rejected because a produced artifact can still
  fail to install, start, or expose the intended package-specific capabilities.

**Sources**: [GitHub Actions reusable workflows](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows),
[GitHub Actions job dependencies](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax).
