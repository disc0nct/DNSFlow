# Contributing to DNSFlow

First off, thank you for considering contributing to DNSFlow! It's people like you that make DNSFlow a great tool. 

DNSFlow is a Tauri v2 cross-platform (Linux/Windows) desktop app for per-application DNS routing. It uses a React 19 + TypeScript frontend, a Rust backend, eBPF (Linux), WinDivert/DNSAPI (Windows), and SQLite.

## Getting Started

To build and run DNSFlow locally, you will need the following prerequisites:

- **Node.js**: v20 or newer
- **Rust**:
  - Stable toolchain (default)
  - Nightly toolchain (required for eBPF compilation on Linux: `rustup toolchain install nightly`, `rustup component add rust-src --toolchain nightly`)
- **Windows Build Tools**: MSVC (if developing on Windows)
- **Linux Dependencies**: `pkg-config`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev` (for Tauri)

### Local Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/dnsflow.git
   cd dnsflow
   ```
2. Install frontend dependencies:
   ```bash
   npm install
   ```
3. Run the development server (starts both Vite and the Tauri Rust backend):
   ```bash
   npm run tauri dev
   ```

## Project Structure

DNSFlow is organized as a Cargo workspace with multiple crates, alongside a Vite frontend:

- `src/` - React frontend (pages, components, lib).
- `src-tauri/` - The main Tauri app and Rust backend. Contains IPC commands, DNS proxy, SQLite database logic, and platform-specific implementations.
- `dnsflow-common/` - Shared `no_std` crate containing core types (like `DnsEvent`) used across kernel and userspace.
- `dnsflow-ebpf/` - eBPF kernel programs for Linux (kprobe, cgroup/connect4 hooks).
- `dnsflow-shim/` - LD_PRELOAD shared library (`cdylib`) for Linux application hooking.
- `dnsflow-hook/` - Windows DNSAPI.dll hook (per-process DLL injection).

## Pull Request Workflow

We actively welcome your pull requests. Please follow this standard GitHub workflow:

1. **Fork** the repository on GitHub.
2. **Branch** out from the `main` branch (`git checkout -b feature/my-awesome-feature`).
3. **Commit** your changes with clear, descriptive commit messages.
4. **Push** your branch to your fork.
5. **Submit a Pull Request** against the upstream `main` branch.

## Code Style Guidelines

- **Rust**: We use `cargo fmt` with default settings (no `rustfmt.toml` is currently used). Please ensure you run `cargo fmt` before submitting your PR.
- **Frontend (React/TypeScript)**: Styling is handled via Tailwind CSS v4 utilizing CSS variables. Follow the existing patterns in the `src/components` and `src/pages` directories.

## Testing and Building

Before submitting a pull request, verify that all tests pass and the project builds successfully:

- Run all Rust tests across the workspace:
  ```bash
  cargo test
  ```
- Verify the frontend builds successfully:
  ```bash
  npm run build
  ```
- To test a full Tauri release build:
  ```bash
  npm run tauri build
  ```

## Mini-Guide: Adding a Tauri Command

If you need to add new backend functionality and expose it to the frontend, follow these steps:

1. **Create the Rust Command**:
   Create a new file or add to an existing one in `src-tauri/src/commands/`.
   ```rust
   #[tauri::command]
   pub async fn my_new_command(arg: String) -> Result<String, String> {
       Ok(format!("Received: {}", arg))
   }
   ```
2. **Register the Module**:
   If you created a new file, expose it in `src-tauri/src/commands/mod.rs`.
3. **Register the Command with Tauri**:
   In `src-tauri/src/lib.rs`, add your command to the `invoke_handler`:
   ```rust
   tauri::Builder::default()
       .invoke_handler(tauri::generate_handler![
           // ... existing commands
           commands::my_module::my_new_command
       ])
   ```
4. **Add Frontend API Wrapper**:
   Expose the command to the React frontend by adding a wrapper in `src/lib/tauri.ts`:
   ```typescript
   import { invoke } from '@tauri-apps/api/core';

   export const myNewCommand = async (arg: string): Promise<string> => {
     return await invoke('my_new_command', { arg });
   };
   ```

## AI-Assisted Development Context

You will notice `AGENTS.md` files in the repository. These files serve as a hierarchical knowledge base specifically designed for AI agents and coding assistants. They contain project context, architectural decisions, and code maps. If you make significant structural changes, please consider updating the relevant sections in `AGENTS.md` to keep the AI context accurate.