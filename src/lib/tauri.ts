import { invoke } from '@tauri-apps/api/core';

export interface DnsServer {
  id?: number;
  name: string;
  address: string;
  secondary_address?: string;
  protocol: string;
  is_default: boolean;
}

export interface AppRule {
  id?: number;
  app_name: string;
  app_path?: string;
  dns_server_id: number;
  enabled: boolean;
  use_ld_preload: boolean;
}

export interface DnsQueryLog {
  id?: number;
  domain: string;
  pid?: number;
  app_name?: string;
  dns_server_id?: number;
  resolved_ip?: string;
  latency_ms?: number;
  timestamp: string;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  exe_path?: string;
  cmdline?: string;
  rule_id?: number;
}

export interface AppConfig {
  proxy_port: number;
  log_enabled: boolean;
  auto_start: boolean;
  debug: boolean;
}

export const dnsApi = {
  getDnsServers: () => {
    console.log("[tauri] getDnsServers called");
    return invoke<DnsServer[]>('get_dns_servers');
  },
  addDnsServer: (name: string, address: string, secondary_address: string | undefined, protocol: string) => {
    const payload = { name, address, secondaryAddress: secondary_address, protocol };
    console.log("[tauri] addDnsServer payload:", payload);
    return invoke<DnsServer>('add_dns_server', payload);
  },
  updateDnsServer: (id: number, name: string, address: string, secondary_address: string | undefined, protocol: string) => {
    const payload = { id, name, address, secondaryAddress: secondary_address, protocol };
    return invoke<DnsServer>('update_dns_server', payload);
  },
  deleteDnsServer: (id: number) => invoke<boolean>('delete_dns_server', { id }),
  setDefaultDnsServer: (id: number) => invoke<boolean>('set_default_dns_server', { id }),
  startDnsProxy: () => invoke<boolean>('start_dns_proxy'),
  stopDnsProxy: () => invoke<boolean>('stop_dns_proxy'),
  getDnsStatus: () => invoke<string>('get_dns_status'),
};

export const processApi = {
  getRunningProcesses: () => invoke<ProcessInfo[]>('get_running_processes'),
  getProcessInfo: (pid: number) => invoke<ProcessInfo | null>('get_process_info', { pid }),
  killProcess: (pid: number) => invoke<boolean>('kill_process', { pid }),
};

export const configApi = {
  getConfig: () => invoke<AppConfig>('get_config'),
  updateConfig: (config: AppConfig) => invoke<boolean>('update_config', { config }),
  exportConfig: () => invoke<string>('export_config'),
  importConfig: (json: string) => invoke<boolean>('import_config', { json }),
  resetConfig: () => invoke<boolean>('reset_config'),
  saveConfigFile: (json: string) => invoke<string>('save_config_file', { json }),
};

export const ruleApi = {
  getRules: () => invoke<AppRule[]>('get_rules'),
  getActiveRuleSessions: () => invoke<Record<number, number>>('get_active_rule_sessions'),
  addRule: (appName: string, appPath: string | undefined, dnsServerId: number, useLdPreload: boolean) =>
    invoke<AppRule>('add_rule', { appName, appPath, dnsServerId, useLdPreload }),
  updateRule: (id: number, appName: string, appPath: string | undefined, dnsServerId: number, useLdPreload: boolean) =>
    invoke<AppRule>('update_rule', { id, appName, appPath, dnsServerId, useLdPreload }),
  toggleRule: (id: number, enabled: boolean) => invoke<boolean>('toggle_rule', { id, enabled }),
  deleteRule: (id: number) => invoke<boolean>('delete_rule', { id }),
};

export const filePicker = {
  pickExecutable: async (): Promise<string | null> => {
    return window.prompt("Enter executable path:");
  },
};

export const launcherApi = {
  launchWithShim: (appPath: string, ruleId?: number) => invoke<number>('launch_with_shim', { appPath, ruleId }),
};

export const queryLogApi = {
  getQueryLogs: (limit?: number) => invoke<DnsQueryLog[]>('get_query_logs', { limit: limit ?? 50 }),
  clearQueryLogs: () => invoke<boolean>('clear_query_logs'),
};
