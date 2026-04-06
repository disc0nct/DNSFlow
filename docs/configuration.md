# Configuration Reference

## Application Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DNSFLOW_PROXY_PORT` | `5353` | DNS proxy listen port |
| `DNSFLOW_LOG_LEVEL` | `info` | Log verbosity |
| `DNSFLOW_RULES` | `/tmp/dnsflow_rules.json` | LD_PRELOAD rules file |
| `RUST_LOG` | — | Rust logging filter |
| `SUDO_UID` | — | Linux: privilege dropping |
| `SUDO_GID` | — | Linux: privilege dropping |
| `SUDO_USER` | — | Linux: privilege dropping |
| `DISPLAY` | — | Linux: GUI session forwarding |
| `WAYLAND_DISPLAY` | — | Linux: GUI session forwarding |
| `XAUTHORITY` | — | Linux: GUI session forwarding |
| `WEBKIT_DISABLE_COMPOSITING_MODE` | — | Linux: GPU fallback |
| `LIBGL_ALWAYS_SOFTWARE` | — | Linux: GPU fallback |
| `XDG_RUNTIME_DIR` | — | Linux: fallback path for rules JSON |

### Windows Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DNSFLOW_PROXY_PORT` | `5353` | DNS proxy listen port |
| `DNSFLOW_LOG_LEVEL` | `info` | Log verbosity |
| `DNSFLOW_RULES` | `%TEMP%\dnsflow_rules.json` | Hook DLL rules file |
| `DNSFLOW_HOOK_DLL_PATH` | App directory | Path to dnsflow_hook.dll |

### Database Location

SQLite database stored at:
- Linux: `~/.local/share/com.kali.dnsflow/dnsflow.db`
- Windows: `%APPDATA%\com.kali.dnsflow\dnsflow.db`
- Configurable via Tauri app data dir

---

## Tauri Configuration

File: `src-tauri/tauri.conf.json`

```json
{
  "productName": "dnsflow",
  "version": "0.1.0",
  "identifier": "com.kali.dnsflow",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "dnsflow",
        "width": 800,
        "height": 600
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all"
  }
}
```

---

## DNS Server Configuration

### Preset Providers

| Name | Primary | Secondary | Protocol |
|------|---------|-----------|----------|
| Google DNS | 8.8.8.8 | 8.8.4.4 | UDP |
| Cloudflare | 1.1.1.1 | 1.0.0.1 | UDP |
| Quad9 | 9.9.9.9 | 149.112.112.112 | UDP |

### Custom Servers

Supported formats:
- IPv4: `8.8.8.8`
- IPv6: `2001:4860:4860::8888`
- DoH URL: `https://dns.google/dns-query`
- DoT hostname: `dns.google:853`

### Protocol Types

| Protocol | Description | Port |
|----------|-------------|------|
| `udp` | Standard DNS | 53 |
| `tcp` | DNS over TCP | 53 |
| `doh` | DNS-over-HTTPS | 443 |
| `dot` | DNS-over-TLS | 853 |

---

## Rule Configuration

### Rule Structure

```typescript
interface AppRule {
  id?: number;
  app_name: string;           // Display name
  app_path?: string;          // Full executable path
  dns_server_id: number;      // Target DNS server ID
  enabled: boolean;           // Active/inactive
  use_ld_preload: boolean;    // Use LD_PRELOAD injection
}
```

### AppConfig Structure

```typescript
interface AppConfig {
  proxy_port: number;         // DNS proxy listen port
  log_enabled: boolean;       // Enable/disable logging
  auto_start: boolean;        // Start on system boot
  debug: boolean;             // Enable detailed debug logging
}
```

### Rule Matching

Rules match by:
1. **App name** — Process name from `/proc/<pid>/stat`
2. **App path** — Executable path from `/proc/<pid>/exe`

Matching is case-sensitive substring match.

### LD_PRELOAD Rules File

When `use_ld_preload` is enabled, rules are written to:

```
/tmp/dnsflow_rules.json
```

Format:
```json
{
  "intercept": true,
  "proxy_port": 5353
}
```

---

## Database Schema

### dns_servers

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | TEXT | Display name |
| `address` | TEXT | IP address or URL |
| `protocol` | TEXT | udp/tcp/doh/dot |
| `is_default` | INTEGER | Default server flag |
| `created_at` | TEXT | Creation timestamp |

### app_rules

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `app_name` | TEXT | Application name |
| `app_path` | TEXT | Executable path (nullable) |
| `dns_server_id` | INTEGER | Foreign key to dns_servers |
| `enabled` | INTEGER | Active flag |
| `use_ld_preload` | INTEGER | LD_PRELOAD injection flag |
| `created_at` | TEXT | Creation timestamp |

### dns_query_log

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `domain` | TEXT | Queried domain |
| `pid` | INTEGER | Process ID (nullable) |
| `app_name` | TEXT | Application name (nullable) |
| `dns_server_id` | INTEGER | Used DNS server (nullable) |
| `resolved_ip` | TEXT | Resolved IP (nullable) |
| `latency_ms` | INTEGER | Query latency (nullable) |
| `timestamp` | TEXT | Query timestamp |

### config

| Column | Type | Description |
|--------|------|-------------|
| `key` | TEXT | Primary key |
| `value` | TEXT | Configuration value |
| `updated_at` | TEXT | Last update timestamp |

## Proxy Configuration

### Listen Address

Default: `127.0.0.53:53`

Redirection fallback sequence:
1. `127.0.0.53:53`
2. `127.0.0.54:53`
3. `127.0.0.1:5353`

Configurable via:
- Environment variable: `DNSFLOW_PROXY_PORT`
- UI: Settings → Proxy Port

### Upstream Resolution

Uses hickory-dns resolver with system DNS as fallback.

### DoH/DoT Configuration

When using DoH or DoT servers:
- hickory-dns handles TLS/HTTPS negotiation
- Certificate validation enabled by default
- Connection pooling for performance

---

## eBPF Configuration

### Kernel Requirements

- Linux kernel 5.15+
- `bpf` syscall support
- `/sys/kernel/debug/tracing` mounted

### Permissions

eBPF programs require:
- `CAP_BPF` capability, OR
- Root privileges

### BPF Object Location

Default: `target/bpfel-unknown-none/release/dnsflow-ebpf`

Configurable in `src-tauri/src/ebpf/loader.rs`.

---

## LD_PRELOAD Configuration

### Shim Library Location

Default search paths:
1. `target/debug/libdnsflow_shim.so`
2. `target/release/libdnsflow_shim.so`
3. `../target/debug/libdnsflow_shim.so`

### Rules File

Default: `/tmp/dnsflow_rules.json`

Written atomically:
1. Write to `/tmp/dnsflow_rules.json.tmp`
2. `fsync()`
3. Rename to `/tmp/dnsflow_rules.json`

---

## Windows Configuration

### UAC Elevation

The app automatically checks for admin privileges on startup:

```json
// Not configurable - always checks elevation
// If not elevated: auto-relaunches via ShellExecuteExW
```

### WinDivert Filter

Default filter: `"outbound and udp.DstPort == 53"`

Captures all outbound DNS queries. Cannot be modified without recompiling.

### DNSAPI Hook Rules

Rules file: `%TEMP%\dnsflow_rules.json`

```json
{
  "intercept": true,
  "proxy_port": 5353
}
```

Written atomically when rules change. The hook DLL reads this file on each DNS query.

### DLL Injection

Target processes are launched with:
1. `CreateProcessW` (CREATE_SUSPENDED)
2. `VirtualAllocEx` + `WriteProcessMemory` (DLL path)
3. `CreateRemoteThread` (LoadLibraryW)
4. `ResumeThread`

---

## Logging Configuration

### Log Levels

| Level | Description |
|-------|-------------|
| `error` | Errors only |
| `warn` | Warnings + errors |
| `info` | General information |
| `debug` | Detailed debugging |
| `trace` | Very verbose |

### Enable Logging

```bash
RUST_LOG=debug npm run tauri dev
```

### Module-Specific Logging

```bash
RUST_LOG=dnsflow_lib::dns=debug,dnsflow_lib::ebpf=trace npm run tauri dev
```

---

## Config Export/Import

### Export Format

```json
{
  "version": "1.0",
  "app_config": {
    "proxy_port": 5353,
    "log_enabled": true,
    "auto_start": false,
    "debug": false
  },
  "dns_servers": [
    {
      "id": 1,
      "name": "Google DNS",
      "address": "8.8.8.8",
      "protocol": "udp",
      "is_default": false
    }
  ],
  "app_rules": [
    {
      "id": 1,
      "app_name": "Firefox",
      "app_path": "/usr/bin/firefox",
      "dns_server_id": 1,
      "enabled": true,
      "use_ld_preload": false
    }
  ]
}
```

### Export

Settings → Export Config → Downloads JSON file

### Import

Settings → Import Config → Select JSON file → Validates → Replaces current config
