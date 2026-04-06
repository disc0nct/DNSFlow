# Installation Guide

## System Requirements

| Platform | Minimum Version | Notes |
|----------|-----------------|-------|
| Windows | 10 (Build 1903)+ | MSVC Build Tools required |
| Ubuntu | 22.04+ | LTS recommended |
| Fedora | 38+ | |
| Debian | 12+ | Bookworm |
| Arch | Rolling | |
| Linux Kernel | 5.15+ | For eBPF features |

## Prerequisites

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y \
    pkg-config \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    librsvg2-dev \
    build-essential \
    curl
```

### Fedora

```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    gtk3-devel \
    librsvg2-devel \
    gcc \
    curl
```

### Arch Linux

```bash
sudo pacman -S --needed \
    webkit2gtk-4.1 \
    gtk3 \
    librsvg \
    base-devel \
    curl
```

### Windows

1. **Microsoft C++ Build Tools**
   - Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Select "Desktop development with C++"
   - Install (requires ~6GB disk space)

2. **Node.js 20+**
   - Download: https://nodejs.org/
   - Or use nvm-windows: https://github.com/coreybutler/nvm-windows

3. **Rust (stable)**
   ```powershell
   # Download and run rustup-init.exe from https://rustup.rs
   # Or via PowerShell:
   Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
   .\rustup-init.exe
   ```

4. **WebView2** (usually pre-installed on Windows 10+)
   - Download: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

## Rust Installation

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:
```bash
rustc --version
cargo --version
```

## Node.js Installation

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

Verify:
```bash
node --version
npm --version
```

## Project Setup

```bash
git clone https://github.com/yourname/dnsflow.git
cd dnsflow
npm install
```

## Build

### Development Mode

```bash
# Linux
npm run tauri dev

# Windows (auto-requests UAC elevation)
npm run tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420`
- Rust backend compilation
- Tauri window with hot-reload

### Production Build

```bash
# Linux
npm run tauri build

# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc
```

Output:
- Linux: `src-tauri/target/release/bundle/appimage/` or `deb/`
- Windows: `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*.exe`

## Optional: eBPF Support

eBPF programs require nightly Rust and kernel 5.15+.

```bash
# Install nightly toolchain
rustup toolchain install nightly
rustup target add bpfel-unknown-none --toolchain nightly

# Build eBPF programs
cargo +nightly build --target bpfel-unknown-none -p dnsflow-ebpf --release
```

## Optional: LD_PRELOAD Shim

```bash
cargo build -p dnsflow-shim --release
# Output: target/release/libdnsflow_shim.so
```

## Optional: DNSAPI Hook DLL (Windows)

```powershell
cargo build -p dnsflow-hook --release --target x86_64-pc-windows-msvc
# Output: target/x86_64-pc-windows-msvc/release/dnsflow_hook.dll
```

## Optional: WinDivert (Windows)

WinDivert is bundled automatically via the `windivert-sys` crate's `static` feature.
No manual installation required.

## Verification

```bash
# Check all components
npm run tauri dev

# In the app:
# 1. Dashboard should show "Proxy: Stopped"
# 2. DNS Servers page should show 6 preset providers
# 3. Rules page should be empty (ready for rules)
```

## Troubleshooting

### `pkg-config` not found

```bash
sudo apt install pkg-config  # Ubuntu/Debian
sudo dnf install pkgconfig   # Fedora
```

### `webkit2gtk-4.1` not found

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev

# If version mismatch, check available:
apt-cache search webkit2gtk
```

### Cargo build fails with permission errors

```bash
# Fix cargo cache permissions
sudo chown -R $USER:$USER ~/.cargo
```

### npm install fails

```bash
# Clear cache and retry
rm -rf node_modules package-lock.json
npm cache clean --force
npm install
```

### Tauri window doesn't appear

```bash
# Check if running in headless environment
# Tauri requires a display server (X11 or Wayland)
export DISPLAY=:0  # or :1
```

### eBPF compilation fails

```bash
# Ensure nightly is default for this build
rustup override set nightly
cargo build -p dnsflow-ebpf
rustup override set stable
```

### Windows: MSVC Build Tools not found

Install Visual Studio Build Tools:
```powershell
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Or use winget:
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools"
```

### Windows: WebView2 not found

Download and install WebView2 Runtime:
```powershell
# Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
# Or use winget:
winget install Microsoft.EdgeWebView2Runtime
```

### Windows: UAC elevation fails

Ensure you're running as a user with admin privileges. The app will:
1. Check if elevated
2. Show message if not elevated
3. Attempt to relaunch via `ShellExecuteExW` with `runas`

If elevation fails, try running PowerShell as Administrator first.

### Windows: WinDivert driver blocked

Some antivirus software blocks WinDivert's kernel driver. Solutions:
1. Add DNSFlow to antivirus exclusions
2. Temporarily disable real-time protection during installation
3. Use DNSAPI hook mode instead (per-process interception)
