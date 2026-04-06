import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { dnsApi, ruleApi, queryLogApi, type DnsServer, type AppRule, type DnsQueryLog } from '../lib/tauri';
import { SkeletonBlock } from '../components/ui/SkeletonBlock';
import { ToggleSwitch } from '../components/ui/ToggleSwitch';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { EmptyState } from '../components/ui/EmptyState';
import { getServerName, getErrorMessage } from '../lib/utils';
import { LATENCY_GOOD, LATENCY_OK } from '../lib/constants';
import { useTheme } from '../lib/ThemeContext';

function DashboardSkeleton() {
  return (
    <div className="space-y-6">
      <div className="mb-8">
        <SkeletonBlock className="h-7 w-40 mb-2" />
        <SkeletonBlock className="h-4 w-72" />
      </div>
      <div className="glass-card rounded-xl p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <SkeletonBlock className="h-12 w-12 rounded-full" />
            <div>
              <SkeletonBlock className="h-5 w-28 mb-2" />
              <SkeletonBlock className="h-3 w-44" />
            </div>
          </div>
          <SkeletonBlock className="h-10 w-28 rounded-lg" />
        </div>
      </div>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        {[1, 2, 3].map((i) => (
          <div key={i} className="glass-card rounded-xl p-6">
            <div className="flex items-center gap-3">
              <SkeletonBlock className="h-10 w-10 rounded-lg" />
              <div>
                <SkeletonBlock className="h-3 w-24 mb-2" />
                <SkeletonBlock className="h-6 w-12" />
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function Dashboard() {
  const { theme, toggleTheme } = useTheme();
  const [proxyStatus, setProxyStatus] = useState<string>('stopped');
  const [dnsServers, setDnsServers] = useState<DnsServer[]>([]);
  const [rules, setRules] = useState<AppRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [proxyLoading, setProxyLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [recentQueries, setRecentQueries] = useState<DnsQueryLog[]>([]);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      const [statusResult, serversResult, rulesResult, queryLogsResult] = await Promise.all([
        dnsApi.getDnsStatus(),
        dnsApi.getDnsServers(),
        ruleApi.getRules(),
        queryLogApi.getQueryLogs(10),
      ]);
      setProxyStatus(statusResult);
      setDnsServers(serversResult);
      setRules(rulesResult);
      setRecentQueries(queryLogsResult);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to load dashboard data'));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleProxyToggle = async () => {
    if (proxyStatus === 'running' && !window.confirm('Stop DNS proxy? This will disrupt DNS for all routed applications.')) {
      return;
    }
    try {
      setProxyLoading(true);
      if (proxyStatus === 'running') {
        await dnsApi.stopDnsProxy();
        setProxyStatus('stopped');
      } else {
        await dnsApi.startDnsProxy();
        setProxyStatus('running');
      }
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to toggle proxy'));
    } finally {
      setProxyLoading(false);
    }
  };

  const handleToggleRule = async (id: number, enabled: boolean) => {
    try {
      await ruleApi.toggleRule(id, enabled);
      await fetchData();
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to toggle rule'));
    }
  };

  const enabledRules = rules.filter((rule) => rule.enabled);
  const protectedAppsCount = enabledRules.length;

  const totalQueriesToday = recentQueries.length;
  const activeRulesCount = enabledRules.length;
  const averageLatency = recentQueries.length > 0
    ? Math.round(recentQueries.reduce((sum, q) => sum + (q.latency_ms ?? 0), 0) / recentQueries.length)
    : 0;

  if (loading) {
    return <DashboardSkeleton />;
  }

  return (
    <div className="space-y-6 stagger-children">
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-50">Dashboard</h1>
          <p className="mt-1 text-sm text-slate-400">
            Overview of DNS activity, query statistics, and system health.
          </p>
        </div>
        <button
          onClick={toggleTheme}
          className="group flex items-center gap-3 rounded-xl border border-surface-100/50 bg-surface-100/20 p-1.5 pr-4 transition-all hover:border-primary-500/50 hover:bg-surface-100/30"
          aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`}
        >
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-surface-100/50 text-primary-400 shadow-sm transition-colors group-hover:bg-primary-500/10 group-hover:text-primary-300">
            {theme === 'dark' ? (
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h1M3 12h1m15.364-6.364l-.707.707M6.343 17.657l-.707.707m12.728 0l-.707-.707M6.343 6.343l-.707-.707M12 7a5 5 0 100 10 5 5 0 000-10z" />
              </svg>
            ) : (
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
              </svg>
            )}
          </div>
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
            {theme === 'dark' ? 'Light Mode' : 'Dark Mode'}
          </span>
        </button>
      </div>

      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <div className="glass-card rounded-xl p-6 glow-border">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <div className={`relative flex h-12 w-12 items-center justify-center rounded-full ${
              proxyStatus === 'running'
                ? 'bg-success-500/15'
                : 'bg-surface-100/50'
            }`}>
              {proxyStatus === 'running' && (
                <span className="absolute inset-0 rounded-full animate-pulse-glow" />
              )}
              <svg className={`h-6 w-6 ${proxyStatus === 'running' ? 'text-success-400' : 'text-slate-500'}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                {proxyStatus === 'running' ? (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                ) : (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                )}
              </svg>
            </div>
            <div>
              <h2 className="text-lg font-semibold text-surface-50">Proxy Status</h2>
              <p className="text-sm text-slate-400">
                {proxyStatus === 'running' ? 'DNS proxy is active and intercepting queries' : 'DNS proxy is not running'}
              </p>
            </div>
          </div>
          <button
            onClick={handleProxyToggle}
            disabled={proxyLoading}
            className={`inline-flex items-center gap-2 rounded-lg px-5 py-2.5 text-sm font-medium transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed ${
              proxyStatus === 'running'
                ? 'bg-danger-500/15 text-danger-400 border border-danger-500/30 hover:bg-danger-500/25 hover:border-danger-500/50'
                : 'bg-gradient-to-r from-primary-500 to-accent-500 text-white hover:from-primary-400 hover:to-accent-400 shadow-lg shadow-primary-500/20'
            }`}
          >
            {proxyLoading ? (
              <>
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                {proxyStatus === 'running' ? 'Stopping...' : 'Starting...'}
              </>
            ) : (
              <>
                {proxyStatus === 'running' ? (
                  <>
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
                    </svg>
                    Stop Proxy
                  </>
                ) : (
                  <>
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    Start Proxy
                  </>
                )}
              </>
            )}
          </button>
        </div>

        <div className="mt-6 pt-6 border-t border-surface-100/30">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-500 mb-3">System DNS Servers</h3>
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
              <span className="text-sm text-slate-500">No DNS servers configured</span>
            )}
          </div>
        </div>

        <div className="mt-4 pt-4 border-t border-surface-100/30">
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-400">Protected Applications</span>
            <span className="text-lg font-semibold text-primary-400">{protectedAppsCount}</span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3 stagger-children">
        <div className="glass-card rounded-xl p-6 glow-border group">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary-500/15 group-hover:bg-primary-500/25 transition-colors">
              <svg className="h-5 w-5 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
              </svg>
            </div>
            <div>
              <p className="text-xs text-slate-500 uppercase tracking-wider">Total Queries</p>
              <p className="text-2xl font-bold text-surface-50 font-mono">{totalQueriesToday}</p>
            </div>
          </div>
        </div>

        <div className="glass-card rounded-xl p-6 glow-border group">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-accent-500/15 group-hover:bg-accent-500/25 transition-colors">
              <svg className="h-5 w-5 text-accent-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
              </svg>
            </div>
            <div>
              <p className="text-xs text-slate-500 uppercase tracking-wider">Active Rules</p>
              <p className="text-2xl font-bold text-surface-50 font-mono">{activeRulesCount}</p>
            </div>
          </div>
        </div>

        <div className="glass-card rounded-xl p-6 glow-border group">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-warning-500/15 group-hover:bg-warning-500/25 transition-colors">
              <svg className="h-5 w-5 text-warning-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
              </svg>
            </div>
            <div>
              <p className="text-xs text-slate-500 uppercase tracking-wider">Avg Latency</p>
              <p className="text-2xl font-bold text-surface-50 font-mono">{averageLatency}<span className="text-sm text-slate-500">ms</span></p>
            </div>
          </div>
        </div>
      </div>

      <div className="glass-card rounded-xl overflow-hidden">
        <div className="flex items-center justify-between border-b border-surface-100/30 px-6 py-4">
          <h2 className="text-base font-semibold text-surface-50">Recent Queries</h2>
          <Link
            to="/query-log"
            className="text-sm font-medium text-primary-400 hover:text-primary-300 transition-colors flex items-center gap-1"
          >
            View All
            <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
            </svg>
          </Link>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full dark-table">
            <thead>
              <tr className="border-b border-surface-100/20">
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Domain</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">App</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">DNS Server</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Latency</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Time</th>
              </tr>
            </thead>
            <tbody>
              {recentQueries.length > 0 ? (
                recentQueries.map((query, index) => (
                  <tr key={query.id ?? index} className="border-b border-surface-100/10 border-l-2 border-l-transparent">
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-sm font-medium text-surface-50 font-mono">{query.domain}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-sm text-slate-400">{query.app_name || 'Unknown'}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-sm text-slate-400">{getServerName(query.dns_server_id, dnsServers)}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className={`text-sm font-medium font-mono ${
                        (query.latency_ms ?? 0) < LATENCY_GOOD ? 'text-success-400' :
                        (query.latency_ms ?? 0) < LATENCY_OK ? 'text-warning-400' : 'text-danger-400'
                      }`}>
                        {query.latency_ms ?? 0}ms
                      </span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-xs text-slate-500 font-mono">{query.timestamp}</span>
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan={5} className="px-6 py-12 text-center text-sm text-slate-500">
                    No recent queries
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      <div className="glass-card rounded-xl overflow-hidden">
        <div className="flex items-center justify-between border-b border-surface-100/30 px-6 py-4">
          <h2 className="text-base font-semibold text-surface-50">Active Rules</h2>
          <Link
            to="/rules"
            className="text-sm font-medium text-primary-400 hover:text-primary-300 transition-colors flex items-center gap-1"
          >
            Manage Rules
            <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
            </svg>
          </Link>
        </div>
        <div>
          {enabledRules.length > 0 ? (
            enabledRules.map((rule) => (
              <div key={rule.id} className="flex items-center justify-between px-6 py-4 border-b border-surface-100/10 last:border-b-0 hover:bg-primary-500/[0.03] transition-colors">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-surface-100/40">
                    <svg className="h-5 w-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M17.982 18.725A7.488 7.488 0 0012 15.75a7.488 7.488 0 00-5.982 2.975m11.963 0a9 9 0 10-11.963 0m11.963 0A8.966 8.966 0 0112 21a8.966 8.966 0 01-5.982-2.275M15 9.75a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                  </div>
                  <div>
                    <p className="text-sm font-medium text-surface-50">{rule.app_name}</p>
                    <p className="text-xs text-slate-500 font-mono">
                      {getServerName(rule.dns_server_id, dnsServers)}
                    </p>
                  </div>
                </div>
                <ToggleSwitch
                  enabled={rule.enabled}
                  onChange={() => rule.id !== undefined && handleToggleRule(rule.id, !rule.enabled)}
                  label={`Toggle ${rule.app_name}`}
                  disabled={rule.id === undefined}
                  variant="success"
                />
              </div>
            ))
          ) : (
            <EmptyState
              icon={
                <svg className="h-8 w-8 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
                </svg>
              }
              title="No active rules"
              description="Add rules to configure per-app DNS routing."
            />
          )}
        </div>
      </div>
    </div>
  );
}

export default Dashboard;
