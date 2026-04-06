import { useState, useEffect, useCallback } from 'react';
import { configApi, dnsApi, type AppConfig, type DnsServer } from '../lib/tauri';
import { SkeletonBlock } from '../components/ui/SkeletonBlock';
import { ToggleSwitch } from '../components/ui/ToggleSwitch';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { SUCCESS_MESSAGE_DURATION } from '../lib/constants';
import { getErrorMessage } from '../lib/utils';

function SettingsSkeleton() {
  return (
    <div className="animate-fade-in">
      <div className="mb-8">
        <SkeletonBlock className="h-7 w-32 mb-2" />
        <SkeletonBlock className="h-4 w-64" />
      </div>
      <div className="space-y-6">
        {[1, 2, 3].map((i) => (
          <div key={i} className="glass-card rounded-xl p-6">
            <SkeletonBlock className="h-5 w-40 mb-2" />
            <SkeletonBlock className="h-3 w-56 mb-4" />
            <SkeletonBlock className="h-12 w-full" />
          </div>
        ))}
      </div>
    </div>
  );
}

function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [dnsServers, setDnsServers] = useState<DnsServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [importedFileName, setImportedFileName] = useState<string | null>(null);
  const [portValue, setPortValue] = useState<string>('');

  useEffect(() => {
    if (config) {
      setPortValue(String(config.proxy_port ?? 5353));
    }
  }, [config]);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      const [configResult, serversResult] = await Promise.all([
        configApi.getConfig(),
        dnsApi.getDnsServers(),
      ]);
      setConfig(configResult);
      setDnsServers(serversResult);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to load settings'));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleConfigUpdate = async (updates: Partial<AppConfig>) => {
    if (!config) return;

    const newConfig = { ...config, ...updates };
    try {
      setSaving(true);
      await configApi.updateConfig(newConfig);
      setConfig(newConfig);
      setSuccess('Settings saved successfully');
      setTimeout(() => setSuccess(null), SUCCESS_MESSAGE_DURATION);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to save settings'));
    } finally {
      setSaving(false);
    }
  };

  const handleExportConfig = async () => {
    try {
      setExporting(true);
      const json = await configApi.exportConfig();
      const filePath = await configApi.saveConfigFile(json);
      setSuccess(`Configuration exported to ${filePath}`);
      setTimeout(() => setSuccess(null), 6000);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to export configuration'));
    } finally {
      setExporting(false);
    }
  };

  const handleImportConfig = async () => {
    if (!window.confirm('Import configuration? This will overwrite all your current DNS servers and rules.')) {
      return;
    }

    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) {
        document.body.removeChild(input);
        return;
      }

      try {
        setImporting(true);
        const json = await file.text();
        await configApi.importConfig(json);
        await fetchData();
        setImportedFileName(file.name);
        setSuccess('Configuration imported successfully');
        setTimeout(() => setSuccess(null), SUCCESS_MESSAGE_DURATION);
      } catch (err) {
        setError(getErrorMessage(err, 'Failed to import configuration'));
      } finally {
        setImporting(false);
        if (document.body.contains(input)) {
          document.body.removeChild(input);
        }
      }
    };
    document.body.appendChild(input);
    input.click();
  };

  const handleResetDefaults = async () => {
    if (!window.confirm('Reset all configuration to defaults? This will delete all your custom DNS servers and rules.')) {
      return;
    }

    try {
      setSaving(true);
      await configApi.resetConfig();
      await fetchData();
      setSuccess('Settings reset to defaults');
      setTimeout(() => setSuccess(null), SUCCESS_MESSAGE_DURATION);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to reset settings'));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <SettingsSkeleton />;
  }

  return (
    <div className="animate-fade-in">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-surface-50">Settings</h1>
        <p className="mt-1 text-sm text-slate-400">
          Application preferences, logging options, and general configuration.
        </p>
      </div>

      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      {success && (
        <div className="mb-6 rounded-lg border border-success-500/30 bg-success-500/10 px-4 py-3 text-sm text-success-400">
          <div className="flex items-center gap-2">
            <svg className="h-4 w-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {success}
          </div>
        </div>
      )}

      <div className="space-y-6 stagger-children">
        <div className="glass-card rounded-xl overflow-hidden">
          <div className="border-b border-surface-100/30 px-6 py-4">
            <div className="flex items-center gap-2">
              <svg className="h-5 w-5 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              <h2 className="text-base font-semibold text-surface-50">General Settings</h2>
            </div>
            <p className="mt-1 text-sm text-slate-500">Configure basic application behavior</p>
          </div>
          <div className="divide-y divide-surface-100/20">
            <div className="flex flex-col gap-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-medium text-surface-50">Auto-start on boot</p>
                <p className="text-xs text-slate-500">Launch DNSFlow automatically when system starts</p>
              </div>
              <ToggleSwitch
                enabled={config?.auto_start ?? false}
                onChange={() => handleConfigUpdate({ auto_start: !config?.auto_start })}
                label="Auto-start on boot"
                disabled={saving}
              />
            </div>

            <div className="flex flex-col gap-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-medium text-surface-50">Log DNS queries</p>
                <p className="text-xs text-slate-500">Record DNS query history for analysis</p>
              </div>
              <ToggleSwitch
                enabled={config?.log_enabled ?? false}
                onChange={() => handleConfigUpdate({ log_enabled: !config?.log_enabled })}
                label="Log DNS queries"
                disabled={saving}
              />
            </div>

            <div className="flex flex-col gap-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-medium text-surface-50">Debug mode</p>
                <p className="text-xs text-slate-500">Enable detailed logging for troubleshooting</p>
              </div>
              <ToggleSwitch
                enabled={config?.debug ?? false}
                onChange={() => handleConfigUpdate({ debug: !config?.debug })}
                label="Debug mode"
                disabled={saving}
              />
            </div>

            <div className="flex flex-col gap-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-medium text-surface-50">Proxy port</p>
                <p className="text-xs text-slate-500">Local port for DNS proxy listener</p>
              </div>
              <input
                type="number"
                value={portValue}
                onChange={(e) => setPortValue(e.target.value)}
                onBlur={() => {
                  const port = parseInt(portValue, 10);
                  if (!isNaN(port) && port > 0 && port <= 65535 && port !== config?.proxy_port) {
                    handleConfigUpdate({ proxy_port: port });
                  } else {
                    setPortValue(String(config?.proxy_port ?? 5353));
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    (e.target as HTMLInputElement).blur();
                  }
                }}
                disabled={saving}
                min={1}
                max={65535}
                aria-label="Proxy port"
                className="w-24 rounded-lg border border-surface-100/50 bg-surface-100/20 px-3 py-1.5 text-sm text-surface-50 text-center font-mono focus:border-primary-500/50 focus:outline-none focus:ring-1 focus:ring-primary-500/20 disabled:opacity-50"
              />
            </div>
          </div>
        </div>

        <div className="glass-card rounded-xl overflow-hidden">
          <div className="border-b border-surface-100/30 px-6 py-4">
            <div className="flex items-center gap-2">
              <svg className="h-5 w-5 text-accent-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M11.42 15.17L17.25 21A2.652 2.652 0 0021 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 11-3.586-3.586l6.837-5.63m5.108-.233c.55-.164 1.163-.188 1.743-.14a4.5 4.5 0 004.486-6.336l-3.276 3.277a3.004 3.004 0 01-2.25-2.25l3.276-3.276a4.5 4.5 0 00-6.336 4.486c.091 1.076-.071 2.264-.904 2.95l-.102.085m-1.745 1.437L5.909 7.5H4.5L2.25 3.75l1.5-1.5L7.5 4.5v1.409l4.26 4.26m-1.745 1.437l1.745-1.437m6.615 8.206L15.75 15.75M4.867 19.125h.008v.008h-.008v-.008z" />
              </svg>
              <h2 className="text-base font-semibold text-surface-50">System Info</h2>
            </div>
            <p className="mt-1 text-sm text-slate-500">Current system configuration details</p>
          </div>
          <div className="divide-y divide-surface-100/20">
            <div className="px-6 py-4">
              <p className="text-xs font-semibold uppercase tracking-wider text-slate-500 mb-2">System DNS Servers</p>
              <div className="flex flex-wrap gap-2">
                {dnsServers.length > 0 ? (
                  dnsServers.map((server) => (
                    <span
                      key={server.id}
                      className="inline-flex items-center gap-1.5 rounded-full bg-surface-100/40 border border-surface-100/30 px-3 py-1.5 text-xs font-medium text-slate-300"
                    >
                      <span className={`h-2 w-2 rounded-full ${
                        server.is_default ? 'bg-success-400 animate-pulse' : 'bg-slate-600'
                      }`} />
                      {server.name}
                      <span className="text-slate-500 font-mono">{server.address}</span>
                    </span>
                  ))
                ) : (
                  <span className="text-sm text-slate-500">No DNS servers detected</span>
                )}
              </div>
            </div>

            <div className="flex flex-col gap-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-medium text-surface-50">App Version</p>
                <p className="text-xs text-slate-500">Current DNSFlow version</p>
              </div>
              <span className="inline-flex items-center rounded-full bg-primary-500/15 border border-primary-500/20 px-3 py-1 text-xs font-medium text-primary-400 font-mono">
                v1.0.0
              </span>
            </div>
          </div>
        </div>

        <div className="glass-card rounded-xl overflow-hidden">
          <div className="border-b border-surface-100/30 px-6 py-4">
            <div className="flex items-center gap-2">
              <svg className="h-5 w-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
              </svg>
              <h2 className="text-base font-semibold text-surface-50">Data Management</h2>
            </div>
            <p className="mt-1 text-sm text-slate-500">Export, import, or reset your configuration</p>
          </div>
          <div className="p-6">
            <div className="flex flex-col gap-4">
              <div className="flex flex-wrap gap-3">
                <button
                  onClick={handleExportConfig}
                  disabled={exporting || importing || saving}
                  className="inline-flex items-center gap-2 rounded-lg border border-surface-100/50 bg-surface-100/20 px-4 py-2.5 text-sm font-medium text-slate-300 transition-all hover:bg-surface-100/40 hover:text-surface-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {exporting ? (
                    <>
                      <div className="h-4 w-4 animate-spin rounded-full border-2 border-slate-400 border-t-transparent" />
                      Exporting...
                    </>
                  ) : (
                    <>
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                      </svg>
                      Export Config
                    </>
                  )}
                </button>

                <button
                  onClick={handleImportConfig}
                  disabled={exporting || importing || saving}
                  className="inline-flex items-center gap-2 rounded-lg border border-surface-100/50 bg-surface-100/20 px-4 py-2.5 text-sm font-medium text-slate-300 transition-all hover:bg-surface-100/40 hover:text-surface-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {importing ? (
                    <>
                      <div className="h-4 w-4 animate-spin rounded-full border-2 border-slate-400 border-t-transparent" />
                      Importing...
                    </>
                  ) : (
                    <>
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
                      </svg>
                      Import Config
                    </>
                  )}
                </button>

                <button
                  onClick={handleResetDefaults}
                  disabled={exporting || importing || saving}
                  className="inline-flex items-center gap-2 rounded-lg border border-danger-500/30 bg-danger-500/10 px-4 py-2.5 text-sm font-medium text-danger-400 transition-all hover:bg-danger-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182" />
                  </svg>
                  Reset to Defaults
                </button>
              </div>
              {importedFileName && (
                <p className="text-sm text-slate-500">
                  Last imported: <span className="font-medium text-surface-50 font-mono">{importedFileName}</span>
                </p>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Settings;
