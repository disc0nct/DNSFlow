# API Reference

All frontend-backend communication uses Tauri v2 `invoke()` commands.

## Types

### DnsServer

```typescript
interface DnsServer {
  id?: number;
  name: string;
  address: string;
  secondary_address?: string;
  protocol: string;
  is_default: boolean;
}
```

### AppRule

```typescript
interface AppRule {
  id?: number;
  app_name: string;
  app_path?: string;
  dns_server_id: number;
  enabled: boolean;
  use_ld_preload: boolean;
}
```

### DnsQueryLog

```typescript
interface DnsQueryLog {
  id?: number;
  domain: string;
  pid?: number;
  app_name?: string;
  dns_server_id?: number;
  resolved_ip?: string;
  latency_ms?: number;
  timestamp: string;
}
```

### ProcessInfo

```typescript
interface ProcessInfo {
  pid: number;
  ppid?: number;
  name: string;
  exe_path?: string;
  cmdline?: string;
  rule_id?: number;
}
```

### AppConfig

```typescript
interface AppConfig {
  proxy_port: number;
  log_enabled: boolean;
  auto_start: boolean;
  debug: boolean;
}
```

---

## DNS Commands

### get_dns_servers

Get all configured DNS servers.

```typescript
dnsApi.getDnsServers(): Promise<DnsServer[]>
```

**Returns:** Array of DNS server configurations.

### add_dns_server

Add a new DNS server.

```typescript
dnsApi.addDnsServer(
  name: string,
  address: string,
  secondary_address: string | undefined,
  protocol: string
): Promise<DnsServer>
```

**Parameters:**
- `name` — Display name (e.g., "Google DNS")
- `address` — IP address (e.g., "8.8.8.8")
- `secondary_address` — Optional secondary IP
- `protocol` — Protocol type ("udp", "tcp", "doh", "dot")

**Returns:** Created DNS server with assigned ID.

### update_dns_server

Update an existing DNS server.

```typescript
dnsApi.updateDnsServer(
  id: number,
  name: string,
  address: string,
  secondary_address: string | undefined,
  protocol: string
): Promise<DnsServer>
```

**Parameters:**
- `id` — DNS server ID
- `name` — Display name
- `address` — IP address
- `secondary_address` — Optional secondary IP
- `protocol` — Protocol type

**Returns:** Updated DNS server.

### set_default_dns_server

Set a DNS server as the system-wide default for non-routed traffic.

```typescript
dnsApi.setDefaultDnsServer(id: number): Promise<boolean>
```

**Parameters:**
- `id` — DNS server ID

**Returns:** `true` on success.

### delete_dns_server

Delete a DNS server.

```typescript
dnsApi.deleteDnsServer(id: number): Promise<boolean>
```

**Parameters:**
- `id` — DNS server ID

**Returns:** `true` on success.

### start_dns_proxy

Start the DNS proxy server on 127.0.0.53:53 (standard DNS).

```typescript
dnsApi.startDnsProxy(): Promise<boolean>
```

**Returns:** `true` on success.

**Behavior:**
- Binds to `127.0.0.53:53` (standard loopback).
- Fallback: `127.0.0.54:53` then `127.0.0.1:5353` if port 53 is occupied.
- Automatically handles socket cleanup on failure.

**Error:** Throws if proxy already running or all ports occupied.

### stop_dns_proxy

Stop the DNS proxy server.

```typescript
dnsApi.stopDnsProxy(): Promise<boolean>
```

**Returns:** `true` on success.

### get_dns_status

Get current proxy status.

```typescript
dnsApi.getDnsStatus(): Promise<string>
```

**Returns:** `"running"` or `"stopped"`.

### get_query_logs

Get DNS query history.

```typescript
dnsApi.getQueryLogs(limit?: number): Promise<DnsQueryLog[]>
```

**Parameters:**
- `limit` — Maximum number of entries (default: 50)

**Returns:** Array of recent DNS query logs.

---

## Rule Commands

### get_rules

Get all DNS routing rules.

```typescript
ruleApi.getRules(): Promise<AppRule[]>
```

### get_active_rule_sessions

Get currently active process sessions tracked by rules.

```typescript
ruleApi.getActiveRuleSessions(): Promise<Record<number, number>>
```

**Returns:** A mapping from `rule_id` to `pid` for all currently intercepted processes.

### add_rule

Add a new routing rule.

```typescript
ruleApi.addRule(
  appName: string,
  appPath: string | undefined,
  dnsServerId: number,
  useLdPreload: boolean
): Promise<AppRule>
```

### update_rule

Update an existing routing rule.

```typescript
ruleApi.updateRule(
  id: number,
  appName: string,
  appPath: string | undefined,
  dnsServerId: number,
  useLdPreload: boolean
): Promise<AppRule>
```

**Parameters:**
- `id` — Rule ID
- `appName` — Application name (for display)
- `appPath` — Full path to binary (optional)
- `dnsServerId` — ID of DNS server to use
- `useLdPreload` — Enable LD_PRELOAD/shim injection

**Returns:** Updated rule object.

### toggle_rule

Enable or disable a rule.

```typescript
ruleApi.toggleRule(id: number, enabled: boolean): Promise<boolean>
```

### delete_rule

Delete a rule.

```typescript
ruleApi.deleteRule(id: number): Promise<boolean>
```

---

## Process Commands

### get_running_processes

Get all running processes.

```typescript
processApi.getRunningProcesses(): Promise<ProcessInfo[]>
```

### get_process_info

Get information about a specific process.

```typescript
processApi.getProcessInfo(pid: number): Promise<ProcessInfo | null>
```

### kill_process

Kill a process and its entire session group.

```typescript
processApi.killProcess(pid: number): Promise<boolean>
```

**Parameters:**
- `pid` — Process ID to kill

**Behavior:**
- Untracks process from rules engine
- Finds the session ID (`sid`) for the process
- Kills entire session group (`kill -9 -<sid>`)
- Kills specific PID as fallback

**Returns:** `true` on success.

### get_system_dns

Get system DNS servers from /etc/resolv.conf.

```typescript
// Not exposed in frontend API yet
invoke<string[]>('get_system_dns')
```

---

## Config Commands

### get_config

Get application configuration.

```typescript
configApi.getConfig(): Promise<AppConfig>
```

### update_config

Update application configuration.

```typescript
configApi.updateConfig(config: AppConfig): Promise<boolean>
```

### export_config

Export all configuration as JSON string.

```typescript
configApi.exportConfig(): Promise<string>
```

**Returns:** JSON string containing servers, rules, and app config.

### import_config

Import configuration from JSON string.

```typescript
configApi.importConfig(json: string): Promise<boolean>
```

**Parameters:**
- `json` — JSON string from `export_config`

### save_config_file

Save configuration JSON to a file in the user's Downloads directory.

```typescript
configApi.saveConfigFile(json: string): Promise<string>
```

**Parameters:**
- `json` — Configuration JSON string

**Behavior:**
- Automatically handles `SUDO_USER` to save to their home/Downloads directory
- Sets correct file permissions (0644) and ownership

**Returns:** Full file path of the saved configuration.

### reset_config

Reset all configuration and rules to defaults.

```typescript
configApi.resetConfig(): Promise<boolean>
```

**Behavior:**
- Clears all rules, DNS servers, and application configuration
- Reseeds from initial migration data

**Returns:** `true` on success.

---

## Launcher Commands

### launch_with_shim

Launch an application with LD_PRELOAD DNS injection.

```typescript
launcherApi.launchWithShim(appPath: string, ruleId?: number): Promise<number>
```

**Parameters:**
- `appPath` — Full path to executable
- `ruleId` — Optional ID of routing rule to apply

**Returns:** PID of the launched process.

**Behavior:**
- Automatically starts DNS proxy if not already running
- Loads active rules into rules engine
- Tracks PID in rules engine if `ruleId` is provided
- Injects `libdnsflow_shim.so` (Linux) or `dnsflow_hook.dll` (Windows)

**Requirements:**
- DNS proxy must be running (handled automatically)
- Shim library must be compiled and available

---

## File Picker

### pickExecutable

Open file picker dialog for selecting executables.

```typescript
filePicker.pickExecutable(): Promise<string | null>
```

**Returns:** Selected file path or `null` if cancelled.

---

## Error Handling

All commands return `Result<T, String>`. Errors are thrown as JavaScript exceptions:

```typescript
try {
  await dnsApi.startDnsProxy();
} catch (error) {
  console.error('Failed to start proxy:', error);
}
```

Common errors:
- `"Proxy already running"` — `start_dns_proxy` called twice
- `"DNS Proxy must be running to use LD_PRELOAD"` — `launch_with_shim` before starting proxy
- `"Could not find libdnsflow_shim.so"` — Shim not compiled
