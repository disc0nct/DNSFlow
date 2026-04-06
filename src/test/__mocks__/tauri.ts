import { vi } from 'vitest';

// Mock Tauri invoke function
export const invoke = vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
  switch (cmd) {
    case 'get_dns_status':
      return 'stopped';
    case 'get_dns_servers':
      return [
        { id: 1, name: 'Cloudflare', address: '1.1.1.1', protocol: 'udp', is_default: true },
        { id: 2, name: 'Google', address: '8.8.8.8', protocol: 'udp', is_default: false },
      ];
    case 'get_rules':
      return [
        {
          id: 1,
          app_name: 'firefox',
          app_path: '/usr/bin/firefox',
          dns_server_id: 1,
          enabled: true,
          use_ld_preload: false,
        },
      ];
    case 'start_dns_proxy':
      return true;
    case 'stop_dns_proxy':
      return true;
    case 'get_config':
      return {
        proxy_port: 5353,
        log_enabled: true,
        auto_start: false,
      };
    case 'update_config':
      return true;
    case 'export_config':
      return JSON.stringify({ proxy_port: 5353, log_enabled: true, auto_start: false });
    case 'import_config':
      return true;
    case 'toggle_rule':
      return true;
    case 'add_rule':
      return { id: 2, ...args };
    case 'delete_rule':
      return true;
    case 'add_dns_server':
      return { id: 3, ...args };
    case 'delete_dns_server':
      return true;
    case 'get_running_processes':
      return [
        { pid: 1, name: 'systemd', exe_path: '/usr/lib/systemd/systemd', cmdline: '/sbin/init' },
        { pid: 100, name: 'firefox', exe_path: '/usr/bin/firefox', cmdline: 'firefox' },
      ];
    case 'get_process_info':
      return { pid: args?.pid, name: 'test', exe_path: '/usr/bin/test', cmdline: 'test' };
    case 'launch_with_shim':
      return true;
    default:
      console.warn(`Unhandled Tauri command: ${cmd}`);
      return null;
  }
});

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke,
}));

// Mock @tauri-apps/plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(async () => '/mock/path/to/file'),
}));

// Mock @tauri-apps/plugin-opener
vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: vi.fn(),
}));

// Mock @tauri-apps/plugin-sql
vi.mock('@tauri-apps/plugin-sql', () => ({
  Database: {
    load: vi.fn(async () => ({
      select: vi.fn(async () => []),
      execute: vi.fn(async () => ({ rowsAffected: 0 })),
    })),
  },
}));
