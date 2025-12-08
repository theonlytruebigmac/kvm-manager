# Backend Setup Guide

This guide covers setting up the KVM Manager Rust backend for development.

## Prerequisites

### 1. Install Tauri System Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### Fedora/RHEL
```bash
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

#### Arch Linux
```bash
sudo pacman -S webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  libappindicator-gtk3 \
  librsvg
```

Full prerequisites: https://tauri.app/start/prerequisites/

### 2. Install Libvirt

#### Ubuntu/Debian
```bash
sudo apt install libvirt-daemon-system libvirt-clients libvirt-dev
```

#### Fedora/RHEL
```bash
sudo dnf install libvirt libvirt-devel
```

#### Arch Linux
```bash
sudo pacman -S libvirt
```

### 3. Configure Libvirt

```bash
# Start libvirtd service
sudo systemctl start libvirtd
sudo systemctl enable libvirtd

# Add your user to libvirt group
sudo usermod -aG libvirt $USER

# Re-login or use:
newgrp libvirt

# Verify connection
virsh -c qemu:///system list --all
```

### 4. Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 5. Install Node.js (if not already installed)

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install --lts

# Or via package manager
sudo apt install nodejs npm  # Ubuntu/Debian
```

## Building the Backend

### 1. Install NPM Dependencies

```bash
npm install
```

### 2. Build Rust Backend

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

### 3. Run Development Server

```bash
npm run tauri dev
```

This will:
- Build the Rust backend
- Start the Vite dev server for frontend
- Open the app window
- Enable hot reload for both frontend and backend

## Project Structure

```
src-tauri/
├── Cargo.toml          # Rust dependencies
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # App initialization & command registration
│   ├── commands/       # Tauri command handlers
│   │   ├── vm.rs       # VM lifecycle commands
│   │   └── system.rs   # Host info commands
│   ├── services/       # Business logic
│   │   ├── libvirt.rs  # Libvirt connection management
│   │   └── vm_service.rs # VM operations
│   ├── models/         # Data structures
│   │   ├── vm.rs       # VM models
│   │   └── host.rs     # Host info models
│   ├── state/          # Application state
│   │   └── app_state.rs
│   └── utils/          # Utilities
│       └── error.rs    # Error handling
└── target/             # Build output (gitignored)
```

## Available Commands

Once running, the frontend can invoke these Rust commands:

### VM Commands
- `get_vms()` - List all VMs
- `get_vm(vm_id)` - Get VM details
- `start_vm(vm_id)` - Start a VM
- `stop_vm(vm_id)` - Stop a VM
- `pause_vm(vm_id)` - Pause a VM
- `resume_vm(vm_id)` - Resume a VM
- `reboot_vm(vm_id)` - Reboot a VM

### System Commands
- `get_host_info()` - Get host information
- `get_connection_status()` - Check libvirt connection

## Testing

### Manual Testing

```bash
# In the Tauri app console (F12), try:
await window.__TAURI__.core.invoke('getVms')
await window.__TAURI__.core.invoke('getHostInfo')
```

### Unit Tests (Coming in Week 2)

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Troubleshooting

### "libvirtd is not running"
```bash
sudo systemctl start libvirtd
sudo systemctl status libvirtd
```

### "Permission denied"
```bash
# Ensure user is in libvirt group
groups | grep libvirt

# If not, add user
sudo usermod -aG libvirt $USER
# Then logout and login again
```

### Webkit/GTK Errors
```bash
# Make sure all system dependencies are installed
# See Prerequisites section above
```

### Cargo Build Errors
```bash
# Clean and rebuild
cargo clean --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

## Development Tips

1. **Hot Reload**: Changes to Rust code trigger automatic rebuild
2. **Logging**: Use `RUST_LOG=debug npm run tauri dev` for verbose logs
3. **DevTools**: Press F12 in the app for browser devtools
4. **Backend Logs**: Check terminal output for Rust tracing logs

## Next Steps

- See `.agents/status/backend-status.md` for current status
- See `.agents/integration/tauri-commands.md` for full API contract
- Coordinate with Frontend Agent for integration testing

## Resources

- Tauri Docs: https://tauri.app/
- rust-libvirt: https://docs.rs/virt/latest/virt/
- Libvirt API: https://libvirt.org/html/index.html
