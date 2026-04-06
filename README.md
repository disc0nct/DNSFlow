<div align="center">
  <img src="/public/DNSFlow-logo.png" alt="DNSFlow Logo" width="200">
  <h1>DNSFlow</h1>
  <p><strong>Professional Per-Application DNS Routing & Monitoring</strong></p>

  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Tauri v2](https://img.shields.io/badge/Tauri-v2-3977ad?logo=tauri)](https://tauri.app)
  [![Rust](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust)](https://www.rust-lang.org)
  [![React 19](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)
  [![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript)](https://www.typescriptlang.org)
  [![Tailwind CSS v4](https://img.shields.io/badge/Tailwind_CSS-v4-06B6D4?logo=tailwind-css)](https://tailwindcss.com)
</div>

---

**DNSFlow** is a powerful, cross-platform (Linux/Windows) per-application DNS routing tool built with Tauri v2, Rust, and React 19. It allows you to route DNS queries for specific applications through dedicated resolvers without affecting the rest of your system's network configuration.

Whether you're testing regional availability, enforcing privacy for specific browsers, or debugging local services, DNSFlow provides the kernel-level precision you need.

## ✨ Key Features

- 🎯 **Per-App Routing**: Assign different DNS servers (Google, Cloudflare, Quad9, or custom DoH/DoT) to specific applications.
- 🐧 **Linux-Native Interception**:
  - **Mount Namespaces**: Transparently bind-mount custom `/etc/resolv.conf` per-application for zero-config isolation.
  - **eBPF Attribution**: High-fidelity process attribution using kernel kprobes and `aya-rs`.
- 🪟 **Windows-Native Interception**:
  - **WinDivert Interception**: Kernel-level packet capture and re-injection for system-wide accuracy.
- 🌳 **Ancestry Tracking**: Reliable matching via process tree resolution, automatically applying rules to child processes (e.g., shell → browser).
- 🔒 **Encrypted DNS**: Full support for DNS-over-HTTPS (DoH) and DNS-over-TLS (DoT) via `hickory-dns`.
- 🎨 **Modern UI**: Clean React 19 interface with Light/Dark theme support (React 19 theme system) and persistent state.
- 🌐 **Browser Optimization**: Automatically detects and disables internal DoH/TRR in major browsers (Firefox, Chrome, Edge, Brave) to ensure rules are respected.

---

## 🏗️ Architecture

DNSFlow uses a platform-agnostic trait layer that abstracts away the complexities of Linux and Windows networking.

```text
┌───────────────────────────────────────────────────────────────┐
│                    REACT 19 FRONTEND                          │
│  Dashboard | Rules | DNS Servers | Query Log | Settings       │
├───────────────────────────────────────────────────────────────┤
│                 TAURI v2 IPC COMMANDS                         │
├───────────────────────────────────────────────────────────────┤
│              PLATFORM ABSTRACTION LAYER                       │
│  ProcessEnumerator | SocketLookup | DnsDetector              │
│  DnsInterceptor | AppLauncher | AncestryTracker               │
├────────────────────────┬──────────────────────────────────────┤
│     LINUX KERNEL       │         WINDOWS KERNEL               │
│  Mount Namespaces      │  WinDivert (Packet Capture)          │
│  eBPF (aya-rs)         │  DNSAPI.dll Hooking (Injection)      │
│  LD_PRELOAD Shim       │  CreateToolhelp32Snapshot            │
│  /proc/net/udp         │  GetExtendedUdpTable                 │
├────────────────────────┴──────────────────────────────────────┤
│                     NETWORK STACK                             │
│       hickory-dns (Local Proxy on localhost:5353)             │
└───────────────────────────────────────────────────────────────┘
```

---

## 🚀 Getting Started

### Prerequisites

#### Linux
- **OS**: Modern Linux (Kernel 5.10+ recommended for eBPF)
- **Dependencies**: `sudo apt install pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev`
- **Permissions**: Root privileges or `CAP_BPF`/`CAP_NET_ADMIN` are required for eBPF and Namespacing.

#### Windows
- **OS**: Windows 10/11
- **Tools**: [MSVC Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), Rust stable (MSVC target), Node.js 20+
- **WebView2**: [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- **Driver**: WinDivert driver (automatically managed by DNSFlow).
- **Permissions**: Administrator privileges for packet interception.

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/dnsflow/dnsflow.git
   cd dnsflow
   ```

2. **Install Frontend Dependencies**
   ```bash
   npm install
   ```

3. **Build the Interceptors**
   ```bash
   # Build Linux Shim (LD_PRELOAD)
   cargo build -p dnsflow-shim
   
   # Build Windows Hook (DLL)
   cargo build -p dnsflow-hook --target x86_64-pc-windows-msvc
   ```

4. **Run in Development Mode**
   ```bash
   npm run tauri dev
   ```

---

## 📖 Usage Guide

1. **Add DNS Servers**: Go to the **DNS Servers** tab and add your preferred upstream providers (e.g., `1.1.1.1` for Cloudflare, or a custom DoH/DoT endpoint).
2. **Define Rules**: In the **Rules** tab, create a new rule. You can select a currently running process or browse for an executable.
3. **Start the Proxy**: Hit **Start Proxy** on the Dashboard. DNSFlow will start listening for intercepted queries and routing them based on your rules.

---

## 🛠️ Tech Stack

- **Frontend**: **React 19**, **TypeScript**, **Tailwind CSS v4**, Lucide Icons.
- **Backend**: **Rust**, **Tauri v2**, SQLite (via `sqlx`).
- **Networking**: **hickory-dns** (DoH/DoT/UDP), **aya-rs** (for eBPF), **WinDivert**.
- **State Management**: React Context with persistent local storage.

---

## 📂 Project Structure

- `src/`: React frontend (Pages, Components, Hooks).
- `src-tauri/`: Main Tauri application and platform-specific Rust logic.
- `dnsflow-ebpf/`: eBPF kernel-space programs for Linux.
- `dnsflow-shim/`: Linux `LD_PRELOAD` shared library.
- `dnsflow-hook/`: Windows DNSAPI injection DLL.
- `dnsflow-common/`: Shared types between kernel and userspace.

---

## 🛡️ License

Distributed under the MIT License. See `LICENSE` for more information.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app) - For the amazing cross-platform framework.
- [hickory-dns](https://github.com/hickory-dns/hickory-dns) - For the robust DNS implementation.
- [Aya](https://aya-rs.dev/) - For making eBPF in Rust a reality.
- [WinDivert](https://github.com/basil00/Divert) - For Windows packet interception.
