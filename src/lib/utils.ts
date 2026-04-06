import { DnsServer } from './tauri';

export function getServerName(serverId: number | undefined, servers: DnsServer[]): string {
  if (!serverId) return 'Unknown';
  const server = servers.find((s) => s.id === serverId);
  return server ? server.name : `Server #${serverId}`;
}

export function getErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;
  return fallback;
}
