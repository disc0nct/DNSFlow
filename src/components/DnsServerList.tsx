import type { DnsServer } from '../lib/tauri';
import { EmptyState } from './ui/EmptyState';

interface DnsServerListProps {
  servers: DnsServer[];
  loading: boolean;
  onEdit: (server: DnsServer) => void;
  onDelete: (id: number) => void;
  onSetDefault: (id: number) => void;
}

function formatAddress(server: DnsServer): string {
  if (server.secondary_address) {
    return `${server.address} / ${server.secondary_address}`;
  }
  return server.address;
}

function DnsServerList({ servers, loading, onEdit, onDelete, onSetDefault }: DnsServerListProps) {
  if (loading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <div key={i} className="glass-card rounded-xl p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="animate-shimmer h-5 w-24 rounded" />
                <div className="animate-shimmer h-4 w-32 rounded" />
              </div>
              <div className="animate-shimmer h-8 w-16 rounded" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (servers.length === 0) {
    return (
      <div className="glass-card rounded-xl">
        <EmptyState
          icon={
            <svg className="h-8 w-8 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418" />
            </svg>
          }
          title="No DNS servers configured"
          description="Add a DNS server to get started with DNS routing."
        />
      </div>
    );
  }

  return (
    <div className="glass-card rounded-xl overflow-hidden">
      <div className="overflow-x-auto">
        <table className="min-w-full dark-table">
          <thead>
            <tr className="border-b border-surface-100/20">
              <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Name</th>
              <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Address</th>
              <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Protocol</th>
              <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Default</th>
              <th className="px-6 py-3 text-right text-[10px] font-semibold uppercase tracking-widest text-slate-500">Actions</th>
            </tr>
          </thead>
          <tbody>
            {servers.map((server) => (
              <tr key={server.id} className="border-b border-surface-100/10 border-l-2 border-l-transparent">
                <td className="whitespace-nowrap px-6 py-4">
                  <span className="text-sm font-medium text-surface-50">{server.name}</span>
                </td>
                <td className="whitespace-nowrap px-6 py-4">
                  <code className="rounded bg-surface-100/30 px-2 py-1 text-xs font-mono text-primary-400">
                    {formatAddress(server)}
                  </code>
                </td>
                <td className="whitespace-nowrap px-6 py-4">
                  <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${getProtocolBadgeStyle(server.protocol)}`}>
                    {server.protocol}
                  </span>
                </td>
                <td className="whitespace-nowrap px-6 py-4">
                  {server.is_default ? (
                    <span className="inline-flex items-center gap-1 text-sm text-success-400">
                      <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                      </svg>
                      Yes
                    </span>
                  ) : (
                    <button
                      onClick={() => server.id !== undefined && onSetDefault(server.id)}
                      className="text-sm text-slate-500 hover:text-primary-400 transition-colors"
                    >
                      Make Default
                    </button>
                  )}
                </td>
                <td className="whitespace-nowrap px-6 py-4 text-right">
                  <div className="flex items-center justify-end gap-2">
                    <button
                      onClick={() => onEdit(server)}
                      className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-primary-400 transition-colors hover:bg-primary-500/10"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                      Edit
                    </button>
                    <button
                      onClick={() => {
                        if (server.id !== undefined && window.confirm(`Delete DNS server "${server.name}"? This may affect rules using it.`)) {
                          onDelete(server.id);
                        }
                      }}
                      disabled={server.id === undefined}
                      className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-danger-400 transition-colors hover:bg-danger-500/10 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function getProtocolBadgeStyle(protocol: string): string {
  switch (protocol.toUpperCase()) {
    case 'DOH':
      return 'bg-success-500/15 text-success-400 border border-success-500/20';
    case 'DOT':
      return 'bg-purple-500/15 text-purple-400 border border-purple-500/20';
    case 'TCP':
      return 'bg-primary-500/15 text-primary-400 border border-primary-500/20';
    case 'UDP':
    default:
      return 'bg-surface-100/30 text-slate-400 border border-surface-100/20';
  }
}

export default DnsServerList;
