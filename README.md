# KVM Manager

A modern, fast GUI for KVM/QEMU/libvirt virtualization on Linux

[![Status](https://img.shields.io/badge/status-active%20development-brightgreen)](docs/PROJECT_PLAN.md)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202.0-blue)](https://tauri.app/)
[![License](https://img.shields.io/badge/license-MIT-orange)](LICENSE)

KVM Manager is a desktop application that provides a polished, VMware Workstation / Hyper-V quality experience for Linux virtualization. Built with Rust and modern web technologies.

---

## Overview

KVM Manager aims to be the best open-source GUI for managing KVM/QEMU virtual machines. It combines the power of libvirt with a modern, intuitive interface.

### Key Features

- **Full VM Lifecycle Management** - Create, configure, start, stop, pause, and delete VMs
- **VNC/SPICE Console** - Built-in console access via noVNC with auto-reconnect
- **Snapshot Management** - Create, revert, and manage VM snapshots
- **Storage Management** - Create and manage storage pools and volumes
- **Network Management** - Configure virtual networks (NAT, isolated, bridged)
- **Hardware Editor** - Complete hardware configuration (CPU, memory, disks, NICs)
- **Guest Agent Integration** - Monitor guest OS information, execute commands
- **Multi-Window Support** - Open multiple console windows simultaneously
- **Real-time Monitoring** - Live CPU and memory usage graphs

---

## Quick Start

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt install libvirt-daemon qemu-kvm libvirt-clients

# Fedora
sudo dnf install libvirt qemu-kvm

# Arch Linux
sudo pacman -S libvirt qemu-full

# Add your user to libvirt group
sudo usermod -aG libvirt $USER
# Log out and back in for group changes to take effect
```

### Installation

Download the latest release from the [Releases](https://github.com/yourusername/kvm-manager/releases) page.

```bash
# AppImage (recommended - works on any distro)
chmod +x kvm-manager-*.AppImage
./kvm-manager-*.AppImage

# Debian/Ubuntu
sudo dpkg -i kvm-manager-*.deb

# Fedora/RHEL/openSUSE
sudo rpm -i kvm-manager-*.rpm
```

### Build from Source

```bash
git clone https://github.com/yourusername/kvm-manager.git
cd kvm-manager

npm install
npm run tauri build
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [Console Guide](docs/CONSOLE_USER_GUIDE.md) | VNC and SPICE console setup |
| [UEFI Setup](docs/UEFI_SETUP.md) | UEFI and Secure Boot configuration |
| [Libvirt Permissions](docs/LIBVIRT_PERMISSIONS.md) | Permission troubleshooting |
| [UI Architecture](docs/UI_ARCHITECTURE.md) | Technical UI documentation |
| [Project Plan](docs/PROJECT_PLAN.md) | Development roadmap |

---

## Features

### VM Management

| Feature | Description |
|---------|-------------|
| Create VMs | Full wizard with UEFI/BIOS, storage, and networking options |
| VM Control | Start, stop, pause, resume, force stop, reboot |
| Live Status | Real-time status updates with event-driven refresh |
| Grouping | Group VMs by status (Running/Paused/Stopped) |
| Multi-Select | Select and control multiple VMs at once |
| Context Menus | Right-click actions for quick access |

### Console Access

| Feature | Description |
|---------|-------------|
| VNC Console | High-quality VNC via noVNC |
| SPICE Console | SPICE protocol support for better performance |
| Keyboard Mapping | Proper keyboard handling including special keys |
| Auto-Reconnect | Automatic reconnection on connection loss |
| Multi-Window | Open multiple console windows simultaneously |

### Snapshots

| Feature | Description |
|---------|-------------|
| Create | Save VM state with descriptions |
| Revert | Restore to any previous state |
| Delete | Clean up old snapshots |
| Tree View | Visualize snapshot hierarchy |

### Storage Management

| Feature | Description |
|---------|-------------|
| Storage Pools | Create and manage storage pools (dir, LVM, etc.) |
| Volumes | Create, delete, and resize volumes |
| Format Support | qcow2, raw, vmdk, and more |
| Thin Provisioning | Efficient disk space usage |

### Network Management

| Feature | Description |
|---------|-------------|
| Virtual Networks | Create NAT, isolated, and bridged networks |
| Network Status | Monitor network activity and connected VMs |
| Interface Management | Add/remove network interfaces from VMs |

### Hardware Configuration

| Feature | Description |
|---------|-------------|
| CPU | Configure vCPUs, topology (sockets/cores/threads), CPU model |
| Memory | Set memory size with ballooning support |
| Disks | Add, remove, and configure virtual disks |
| NICs | Add and configure network interfaces |
| Graphics | Configure VNC/SPICE displays |
| Boot Order | Set boot device priority |
| USB/PCI | Passthrough USB and PCI devices |

### Guest Agent Integration

| Feature | Description |
|---------|-------------|
| System Info | OS, hostname, timezone from guest |
| Network Info | Guest IP addresses and interfaces |
| File Operations | Read/write files in guest |
| Commands | Execute commands in guest |
| User Management | List logged-in users |

---

## Development

### Tech Stack

**Backend (Rust)**
- Tauri 2.0 - Desktop application framework
- libvirt-rs - KVM/QEMU management bindings
- tokio - Async runtime
- serde - Serialization

**Frontend (TypeScript)**
- React 19 - UI framework
- TanStack Query - Server state management
- shadcn/ui - Component library
- Tailwind CSS - Styling
- noVNC - VNC console

### Project Structure

```
kvm-manager/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utilities and Tauri bindings
│   └── pages/              # Page components
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri IPC commands
│       ├── vm/             # VM management
│       ├── storage/        # Storage management
│       ├── network/        # Network management
│       └── main.rs         # Entry point
├── guest-agent/            # QEMU guest agent
│   ├── agent-linux/        # Linux agent daemon
│   └── agent-common/       # Shared protocol code
└── docs/                   # Documentation
```

### Development Commands

```bash
# Start development server (hot reload)
npm run tauri dev

# Run frontend only (faster iteration)
npm run dev

# Build for production
npm run tauri build

# Type checking
npx tsc --noEmit

# Linting
npm run lint
cargo clippy

# Testing
cargo test
```

### Contributing

Contributions are welcome. See [AGENTS.md](AGENTS.md) for the multi-agent development approach used in this project.

---

## System Requirements

### Minimum

- Linux kernel 4.5+
- 2 GB RAM
- 100 MB disk space

### Recommended

- Linux kernel 5.0+
- 4 GB RAM
- 500 MB disk space
- Hardware virtualization (Intel VT-x / AMD-V)

### Dependencies

- libvirt 6.0+ (VM management API)
- QEMU 4.0+ (hypervisor)
- KVM (kernel module)

---

## Roadmap

See [PROJECT_PLAN.md](docs/PROJECT_PLAN.md) for the detailed development roadmap.

**Planned Features**
- Theme system (dark/light mode)
- Remote libvirt connections
- VM cloning and templates
- System tray with background monitoring
- Global keyboard shortcuts

---

## License

MIT License - See [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- [libvirt](https://libvirt.org/) - Virtualization API
- [Tauri](https://tauri.app/) - Desktop application framework
- [noVNC](https://novnc.com/) - JavaScript VNC client
- [shadcn/ui](https://ui.shadcn.com/) - Component library
- [virt-manager](https://virt-manager.org/) - Inspiration and reference

---

## Support

- **Bug Reports**: [GitHub Issues](https://github.com/yourusername/kvm-manager/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/kvm-manager/discussions)

---

## IDE Setup

Recommended extensions for VS Code:
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint)
- [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)
