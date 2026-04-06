import { useState, useEffect } from 'react';
import { dnsApi, type DnsServer, type AppRule } from '../lib/tauri';
import AppSelector from './AppSelector';
import { ToggleSwitch } from './ui/ToggleSwitch';
import { inputClasses } from '../lib/constants';
import { getErrorMessage } from '../lib/utils';

interface RuleFormProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (appName: string, appPath: string | undefined, dnsServerId: number, useLdPreload: boolean) => void;
  loading: boolean;
  initialData?: AppRule | null;
}

function RuleForm({ isOpen, onClose, onSubmit, loading, initialData }: RuleFormProps) {
  const [selectedApp, setSelectedApp] = useState<{ name: string; path?: string } | undefined>();
  const [dnsServers, setDnsServers] = useState<DnsServer[]>([]);
  const [selectedServerId, setSelectedServerId] = useState<number | null>(null);
  const [useLdPreload, setUseLdPreload] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadingServers, setLoadingServers] = useState(false);

  useEffect(() => {
    if (isOpen) {
      if (initialData) {
        setSelectedApp({ name: initialData.app_name, path: initialData.app_path });
        setSelectedServerId(initialData.dns_server_id);
        setUseLdPreload(initialData.use_ld_preload);
      } else {
        setSelectedApp(undefined);
        setSelectedServerId(null);
        setUseLdPreload(false);
      }
      setError(null);
      fetchDnsServers();
    }
  }, [isOpen, initialData]);

  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  const fetchDnsServers = async () => {
    try {
      setLoadingServers(true);
      const servers = await dnsApi.getDnsServers();
      setDnsServers(servers);
      const defaultServer = servers.find((s) => s.is_default);
      if (!initialData) {
        if (defaultServer && defaultServer.id !== undefined) {
          setSelectedServerId(defaultServer.id);
        } else if (servers.length > 0 && servers[0].id !== undefined) {
          setSelectedServerId(servers[0].id);
        }
      }
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to load DNS servers'));
    } finally {
      setLoadingServers(false);
    }
  };

  const handleAppSelect = (appName: string, appPath: string | undefined) => {
    setSelectedApp({ name: appName, path: appPath });
    setError(null);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!selectedApp) {
      setError('Please select an application');
      return;
    }

    if (selectedServerId === null) {
      setError('Please select a DNS server');
      return;
    }

    onSubmit(selectedApp.name, selectedApp.path, selectedServerId, useLdPreload);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in sm:p-6 md:p-8">
      <div
        className="absolute inset-0 modal-backdrop"
        onClick={onClose}
      />

      <div role="dialog" aria-modal="true" aria-label="Add DNS Rule" className="relative w-full max-w-lg rounded-xl bg-surface-400 border border-surface-100/30 p-6 shadow-2xl max-h-[90vh] overflow-y-auto animate-slide-in-up mx-auto my-auto">
        <div className="mb-6 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent-500/15">
              <svg className="h-4 w-4 text-accent-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
              </svg>
            </div>
            <h2 className="text-base font-semibold text-surface-50">{initialData ? 'Edit DNS Rule' : 'Add DNS Rule'}</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="rounded-lg p-1.5 text-slate-500 transition-colors hover:bg-surface-100/30 hover:text-slate-300"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-5">
          {error && (
            <div className="rounded-lg border border-danger-500/30 bg-danger-500/10 px-4 py-3 text-sm text-danger-400">
              {error}
            </div>
          )}

          <div>
            <label className="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-500">
              Select Application
            </label>
            <AppSelector onSelect={handleAppSelect} selectedApp={selectedApp} />
          </div>

          <div>
            <label htmlFor="dns-server" className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">
              DNS Server
            </label>
            {loadingServers ? (
              <div className="flex items-center gap-2 rounded-lg border border-surface-100/30 bg-surface-100/10 px-3.5 py-2.5">
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-surface-100/50 border-t-primary-500" />
                <span className="text-sm text-slate-500">Loading servers...</span>
              </div>
            ) : (
              <select
                id="dns-server"
                value={selectedServerId ?? ''}
                onChange={(e) => setSelectedServerId(Number(e.target.value) || null)}
                className={inputClasses}
              >
                <option value="">Select a DNS server</option>
                {dnsServers.map((server) => (
                  <option key={server.id} value={server.id}>
                    {server.name} ({server.address}){server.is_default ? ' - Default' : ''}
                  </option>
                ))}
              </select>
            )}
          </div>

          <div className="flex items-center justify-between rounded-lg border border-surface-100/30 bg-surface-100/10 px-4 py-3">
            <div>
              <p className="text-sm font-medium text-surface-50">Enable LD_PRELOAD</p>
              <p className="mt-0.5 text-xs text-slate-500">
                Inject DNS resolver into application process
              </p>
              <p className="mt-1 text-xs text-warning-400 font-medium flex items-center gap-1">
                <svg className="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                </svg>
                Requires launching the app through DNSFlow
              </p>
            </div>
            <ToggleSwitch
              enabled={useLdPreload}
              onChange={() => setUseLdPreload(!useLdPreload)}
              label="Enable LD_PRELOAD"
            />
          </div>

          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 rounded-lg border border-surface-100/50 bg-surface-100/20 px-4 py-2.5 text-sm font-medium text-slate-300 transition-colors hover:bg-surface-100/40 hover:text-surface-50"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="flex-1 rounded-lg bg-gradient-to-r from-primary-500 to-accent-500 px-4 py-2.5 text-sm font-medium text-white transition-all hover:from-primary-400 hover:to-accent-400 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'Saving...' : initialData ? 'Update Rule' : 'Save Rule'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default RuleForm;
