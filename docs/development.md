# Development Guide

## Prerequisites

See [Installation Guide](installation.md) for system setup.

## Project Structure

```
dnsflow/
├── src/                    # React frontend
├── src-tauri/              # Rust backend
├── dnsflow-common/         # Shared types
├── dnsflow-ebpf/           # eBPF programs
├── dnsflow-shim/           # LD_PRELOAD shim
└── docs/                   # Documentation
```

## Development Workflow

### Start Development Server

```bash
npm run tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420`
- Rust backend compilation with hot-reload
- Tauri window

### Frontend Development

```bash
# Frontend only (no Rust backend)
npm run dev
```

Changes to `src/` hot-reload automatically.

### Backend Development

Changes to `src-tauri/src/` trigger Rust recompilation.

```bash
# Check Rust code without running
cd src-tauri && cargo check

# Run tests
cargo test
```

## Adding Features

### 1. Add a Tauri Command

Create in `src-tauri/src/commands/<module>.rs`:

```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Hello {}", param))
}
```

Register in `src-tauri/src/commands/mod.rs`:

```rust
pub mod my_module;
pub use my_module::*;
```

Register in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    my_command,
])
```

### 2. Add Frontend API Wrapper

In `src/lib/tauri.ts`:

```typescript
export const myApi = {
  myCommand: (param: string) => invoke<string>('my_command', { param }),
};
```

### 3. Add UI Component

Create in `src/components/`:

```tsx
export function MyComponent() {
  const [data, setData] = useState<string>('');
  
  useEffect(() => {
    myApi.myCommand('world').then(setData);
  }, []);
  
  return <div>{data}</div>;
}
```

### 4. Add Page Route

In `src/App.tsx`:

```tsx
<Route path="my-page" element={<MyPage />} />
```

Add sidebar link in `src/components/Layout.tsx`.

## Testing

### Rust Tests

```bash
# All tests
cargo test

# Specific module
cargo test --package dnsflow --lib dns::tests

# Specific test
cargo test test_rules_engine_lookup

# With output
cargo test -- --nocapture
```

### Test Structure

Tests live in `tests/` subdirectories alongside source:

```
src-tauri/src/
├── dns/
│   ├── mod.rs
│   ├── proxy.rs
│   ├── rules.rs
│   └── tests/
│       ├── mod.rs
│       └── rules_test.rs
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synchronous() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_asynchronous() {
        let result = some_async_function().await;
        assert!(result.is_ok());
    }
}
```

## Building

### Development Build

```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Output:
- Binary: `src-tauri/target/release/dnsflow`
- Bundle: `src-tauri/target/release/bundle/`

### Build Specific Crates

```bash
# LD_PRELOAD shim
cargo build -p dnsflow-shim --release

# Common types
cargo build -p dnsflow-common

# eBPF (requires nightly)
cargo +nightly build --target bpfel-unknown-none -p dnsflow-ebpf
```

## Code Style

### Rust

- Edition 2021
- `#[derive(Debug, Clone, Serialize, Deserialize, Default)]` on all state structs
- `async fn` for all Tauri commands
- `Arc<RwLock<T>>` for shared state
- `.map_err(|e| e.to_string())` for error conversion

### TypeScript

- React 19 with `react-jsx` transform
- Tailwind CSS v4 for styling
- Typed interfaces matching Rust structs
- Functional components with hooks

## Windows Development

### Prerequisites

1. MSVC Build Tools (see installation.md)
2. Windows 10+ SDK
3. Rust stable with `x86_64-pc-windows-msvc` target

### Build Commands

```powershell
# Frontend only
npm run dev

# Full Tauri app
npm run tauri dev

# Build specific Windows crate
cargo build -p dnsflow-hook --target x86_64-pc-windows-msvc

# Build all Windows-compatible crates
cargo build -p dnsflow -p dnsflow-common -p dnsflow-hook --target x86_64-pc-windows-msvc
```

### Testing on Windows

```powershell
# Run all tests
cargo test --target x86_64-pc-windows-msvc

# Run Windows-specific tests
cargo test --target x86_64-pc-windows-msvc -- platform

# Test DLL injection
cargo test --target x86_64-pc-windows-msvc -- injection
```

### Debugging

#### WinDivert

```powershell
# Enable WinDivert debug logging
$env:RUST_LOG="windivert=debug"
npm run tauri dev
```

#### DNSAPI Hook

```powershell
# Hook DLL outputs to debugger (use DebugView)
# Download: https://learn.microsoft.com/en-us/sysinternals/downloads/debugview
```

#### Process Monitor

```powershell
# Use Process Monitor to watch DNS queries
# Download: https://learn.microsoft.com/en-us/sysinternals/downloads/procmon
# Filter: Operation == "UDP Send" && Path ends with ":53"
```

### Adding Windows-Specific Features

1. Add trait method in `src-tauri/src/platform/mod.rs`
2. Implement in `src-tauri/src/platform/windows.rs`
3. Update `src-tauri/src/platform/linux.rs` (stub or implement)
4. Update Tauri command to use trait

## Cross-Platform Testing

### Matrix Build

GitHub Actions runs on both platforms:

```yaml
jobs:
  build-linux:
    runs-on: ubuntu-latest
  build-windows:
    runs-on: windows-latest
```

### Platform-Specific Tests

```rust
#[cfg(test)]
#[cfg(target_os = "linux")]
mod linux_tests {
    // Linux-only tests
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod windows_tests {
    // Windows-only tests
}
```

### CI Artifacts

Both platforms produce artifacts:
- Linux: AppImage, .deb
- Windows: .exe (NSIS installer)

## Debugging

### Rust Backend

```bash
# Enable debug logging
RUST_LOG=debug npm run tauri dev

# Specific module
RUST_LOG=dnsflow_lib::dns=debug npm run tauri dev
```

### Frontend

Open DevTools in Tauri window:
- Right-click → Inspect
- Or press `Ctrl+Shift+I`

### Tauri IPC

Log all IPC calls:

```typescript
// In src/lib/tauri.ts
const originalInvoke = invoke;
window.invoke = async (...args) => {
  console.log('[IPC]', ...args);
  return originalInvoke(...args);
};
```

## Common Tasks

### Add a New DNS Provider Preset

1. Add to `src-tauri/migrations/002_seed.sql`
2. Add preset button in `src/components/DnsServerForm.tsx`

### Add a New Settings Option

1. Add field to `AppConfig` in `src-tauri/src/state.rs`
2. Add to `config` table schema
3. Add toggle in `src/pages/Settings.tsx`
4. Wire to `configApi.updateConfig()`

### Add a New Page

1. Create `src/pages/MyPage.tsx`
2. Add route in `src/App.tsx`
3. Add sidebar link in `src/components/Layout.tsx`

### Modify Database Schema

1. Create new migration: `src-tauri/migrations/NNN_description.sql`
2. Update models in `src-tauri/src/state.rs`
3. Update commands to use new schema

## Troubleshooting

### Build fails with "pkg-config not found"

```bash
sudo apt install pkg-config libwebkit2gtk-4.1-dev
```

### Rust compilation slow

```bash
# Use sccache for faster rebuilds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Frontend not updating

```bash
# Clear Vite cache
rm -rf node_modules/.vite
npm run dev
```

### Tauri window blank

```bash
# Check frontend build
npm run build
ls dist/
```
