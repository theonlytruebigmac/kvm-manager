# KVM Manager

A modern, user-friendly desktop application for managing KVM/QEMU virtual machines through libvirt.

**Status**: 🚀 Active Development (v0.2.0)
**Platform**: Linux (libvirt + KVM)
**Tech Stack**: Tauri 2.x + React 19 + TypeScript + Rust

---

## Features

### ✅ Implemented (v0.2.0)

#### VM Management
- 📋 **VM List View**: See all VMs with status, CPU, memory, disk usage
- ▶️ **VM Controls**: Start, pause, resume, shutdown, force stop, reboot
- 🔄 **Real-time Status**: Live VM state updates
- 💾 **VM Creation**: Multi-step wizard for new VMs
- 🗑️ **VM Deletion**: Remove VMs and associated storage

#### Console Integration
- 🖥️ **VNC Console**: Real-time VM display with full input control
- 🔄 **Auto-reconnection**: Exponential backoff for reliable connections
- 📐 **Display Modes**: Scale to Window, 1:1 Pixels, Stretch to Fill
- ⌨️ **Special Keys**: Send Ctrl+Alt+Del, Ctrl+Alt+F1-F12, and more
- 📸 **Screenshots**: Capture VM display as PNG
- 🎯 **Fullscreen**: Immersive full-screen viewing

#### Snapshot Management
- 📷 **Create Snapshots**: Save VM state with name and description
- 🌳 **Tree View**: Visualize snapshot hierarchy
- ⏮️ **Revert**: Restore VM to previous snapshot
- 🗑️ **Delete**: Remove unwanted snapshots
- 📝 **Metadata**: Timestamp, size, and description for each snapshot

#### Network Management
- 🌐 **Network List**: View all virtual networks
- 🟢 **Network Control**: Start, stop, autostart networks
- 📊 **Network Details**: DHCP ranges, connected VMs, bridge info
- ➕ **Create Networks**: Wizard for NAT and isolated networks

#### Storage Management
- 💿 **Storage Pools**: Manage storage backends (dir, LVM, NFS, etc.)
- 📊 **Pool Details**: Capacity, allocation, available space
- 📁 **Volume Browser**: View and manage disk images
- ➕ **Create Volumes**: New disk images for VMs
- 🔄 **Refresh**: Update pool information

#### Hardware Management
- 🔌 **Device Tree**: Visual hardware hierarchy
- ⚙️ **Hot-plug**: Add/remove devices while VM runs
- 💾 **Storage Devices**: Manage disks, CDROMs, USBs
- 🌐 **Network Interfaces**: Add/remove NICs
- 🖥️ **Display/Graphics**: Configure video adapters

#### Desktop Integration
- 🎨 **Modern UI**: Clean, responsive interface with Tailwind CSS
- 📱 **Multi-window**: Separate windows for console, details, etc.
- ⚡ **Performance**: Fast Rust backend, efficient React frontend
- 🎯 **Keyboard Shortcuts**: Power user productivity
- 🌙 **Dark Mode Ready**: Prepared for theme support

---

## Documentation

- 📖 **[Console User Guide](docs/CONSOLE_USER_GUIDE.md)**: Complete guide to VM console features
- 📋 **[Project Plan](PROJECT_PLAN.md)**: Full project roadmap and architecture
- 🧪 **[Testing Guide](WEEK5_DAY3_TESTING_GUIDE.md)**: Comprehensive testing procedures
- 👥 **[Agent System](AGENTS.md)**: Multi-agent development approach
- 📊 **[Current Status](CURRENT_STATUS.md)**: Latest development status

---

## Quick Start

### Prerequisites

- **Linux OS** (tested on Ubuntu 22.04+, Fedora 38+)
- **KVM/QEMU** installed and configured
- **libvirt** daemon running
- **Rust** 1.75+ (for building from source)
- **Node.js** 18+ (for building from source)

### Installation

#### From Binary (Recommended - Coming Soon)
```bash
# Download latest release
wget https://github.com/yourusername/kvm-manager/releases/latest/kvm-manager.AppImage

# Make executable
chmod +x kvm-manager.AppImage

# Run
./kvm-manager.AppImage
```

#### From Source
```bash
# Clone repository
git clone https://github.com/yourusername/kvm-manager.git
cd kvm-manager

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### First Launch

1. **Grant Permissions**: Add your user to libvirt group
   ```bash
   sudo usermod -aG libvirt $USER
   newgrp libvirt
   ```

2. **Launch Application**: Run KVM Manager

3. **Connect to libvirt**: Default connection to `qemu:///system`

4. **Start Managing VMs**: Click "Virtual Machines" in sidebar

---

## Usage

### Opening VM Console

1. Go to **Virtual Machines** page
2. Find a running VM
3. Click **"Console"** button
4. Interact with VM display

See [Console User Guide](docs/CONSOLE_USER_GUIDE.md) for detailed console features.

### Creating a Snapshot

1. Select a VM
2. Go to **Snapshots** tab
3. Click **"Create Snapshot"**
4. Enter name and description
5. Click **"Create"**

### Managing Networks

1. Go to **Networks** page (sidebar)
2. View all virtual networks
3. Start/stop networks as needed
4. Create new networks with wizard

---

## Development

### Tech Stack

**Frontend**:
- React 19 (UI framework)
- TypeScript (type safety)
- Vite (build tool)
- Tailwind CSS (styling)
- TanStack Query (state management)
- shadcn/ui (component library)

**Backend**:
- Rust (systems programming)
- Tauri 2.x (desktop framework)
- virt (libvirt bindings)

**Console**:
- noVNC 1.6.0 (VNC viewer)

### Project Structure

```
kvm-manager/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Application pages
│   ├── hooks/              # Custom React hooks
│   └── lib/                # Utilities
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri commands
│       └── services/       # Business logic
├── guest-agent/            # VM guest agent (future)
├── docs/                   # Documentation
└── archive/                # Historical records
```

### Development Workflow

```bash
# Start development server
npm run tauri dev

# Run frontend only (fast refresh)
npm run dev

# Build for production
npm run tauri build

# Run tests
npm test
cargo test

# Lint
npm run lint
cargo clippy
```

### Contributing

See [AGENTS.md](AGENTS.md) for our multi-agent development approach.

---

## Roadmap

See [PROJECT_PLAN.md](PROJECT_PLAN.md) for detailed roadmap.

### Upcoming Features (Week 6+)

- 🎨 **Theme System**: Dark/light mode with customization
- 📊 **Performance Monitoring**: Real-time CPU/memory graphs
- 🔐 **Remote Connections**: Manage VMs on remote libvirt hosts
- 📦 **VM Cloning**: Duplicate VMs easily
- 🔧 **Advanced Settings**: Detailed VM configuration
- 🎭 **SPICE Console**: Alternative to VNC with better performance
- 🤖 **Guest Agent**: Enhanced guest OS integration
- 📱 **System Tray**: Background monitoring
- 🚀 **Quick Actions**: Global keyboard shortcuts

---

## Known Issues

- Console reconnection tested in development, needs real VM testing
- Snapshot tree visualization works, needs UX polish
- Network wizard covers NAT/isolated, more types planned
- Guest agent protocol defined, implementations in progress

---

## Requirements

### System Requirements

**Minimum**:
- Linux kernel 4.5+
- 2 GB RAM
- 100 MB disk space

**Recommended**:
- Linux kernel 5.0+
- 4 GB RAM
- 500 MB disk space
- Hardware virtualization (Intel VT-x / AMD-V)

### Software Dependencies

- **libvirt** 6.0+ (VM management)
- **qemu** 4.0+ (hypervisor)
- **KVM** (kernel module)
- Optional: **virt-manager** (for comparison/troubleshooting)

---

## License

[MIT License](LICENSE) - See LICENSE file for details

---

## Acknowledgments

- **libvirt**: Virtualization API
- **Tauri**: Desktop application framework
- **noVNC**: JavaScript VNC client
- **shadcn/ui**: Component library
- **virt-manager**: Inspiration and reference

---

## Support & Community

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/yourusername/kvm-manager/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/yourusername/kvm-manager/discussions)
- 📧 **Email**: your-email@example.com

---

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [VS Code](https://code.visualstudio.com/) + [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) + [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)

---

**Built with ❤️ for the Linux virtualization community**
