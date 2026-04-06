import { useState, useEffect, useCallback } from 'react';
import { type AppRule, type DnsServer, launcherApi, processApi, ruleApi } from '../lib/tauri';
import { ToggleSwitch } from './ui/ToggleSwitch';
import { EmptyState } from './ui/EmptyState';
import { getServerName, getErrorMessage } from '../lib/utils';

interface RulesListProps {
  rules: AppRule[];
  dnsServers: DnsServer[];
  loading: boolean;
  onToggle: (id: number, enabled: boolean) => void;
  onEdit: (rule: AppRule) => void;
  onDelete: (id: number) => void;
}

function RulesList({ rules, dnsServers, loading, onToggle, onEdit, onDelete }: RulesListProps) {
  const [activePids, setActivePids] = useState<Record<number, number>>({});
  const [launchingId, setLaunchingId] = useState<number | null>(null);
  const [feedbackMessage, setFeedbackMessage] = useState<{id: number, type: 'success' | 'error', text: string} | null>(null);

  const fetchActiveSessions = useCallback(async () => {
    try {
      const sessions = await ruleApi.getActiveRuleSessions();
      setActivePids(sessions);
    } catch (err) {
      console.error("Failed to fetch active sessions:", err);
    }
  }, []);

  useEffect(() => {
    fetchActiveSessions();
    const interval = setInterval(fetchActiveSessions, 5000);
    return () => clearInterval(interval);
  }, [fetchActiveSessions]);

  const handleLaunch = async (rule: AppRule) => {
    if (rule.id === undefined || !rule.app_path) return;

    setLaunchingId(rule.id);
    setFeedbackMessage(null);

    try {
      const pid = await launcherApi.launchWithShim(rule.app_path, rule.id);
      if (pid > 0) {
        setActivePids(prev => ({ ...prev, [rule.id!]: pid }));
        setFeedbackMessage({ id: rule.id, type: 'success', text: 'App launched successfully' });
      } else {
        setFeedbackMessage({ id: rule.id, type: 'error', text: 'Failed to launch application' });
      }
    } catch (err) {
      setFeedbackMessage({
        id: rule.id,
        type: 'error',
        text: getErrorMessage(err, 'Failed to launch application')
      });
    } finally {
      setLaunchingId(null);

      setTimeout(() => {
        setFeedbackMessage(prev => prev?.type === 'success' && prev.id === rule.id ? null : prev);
      }, 3000);
    }
  };

  const handleStop = async (rule: AppRule) => {
    if (rule.id === undefined) return;
    const pid = activePids[rule.id];
    if (!pid) return;

    try {
      const success = await processApi.killProcess(pid);
      if (success) {
        setActivePids(prev => {
          const newState = { ...prev };
          delete newState[rule.id!];
          return newState;
        });
        setFeedbackMessage({ id: rule.id, type: 'success', text: 'App stopped' });
      } else {
        setFeedbackMessage({ id: rule.id, type: 'error', text: 'Failed to stop app' });
      }
    } catch (err) {
      setFeedbackMessage({
        id: rule.id,
        type: 'error',
        text: getErrorMessage(err, 'Failed to stop app')
      });
    }
  };

  if (loading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <div key={i} className="glass-card rounded-xl p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="animate-shimmer h-5 w-28 rounded" />
                <div className="animate-shimmer h-4 w-24 rounded" />
              </div>
              <div className="flex gap-2">
                <div className="animate-shimmer h-8 w-16 rounded" />
                <div className="animate-shimmer h-8 w-16 rounded" />
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (rules.length === 0) {
    return (
      <div className="glass-card rounded-xl">
        <EmptyState
          icon={
            <svg className="h-8 w-8 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
            </svg>
          }
          title="No rules configured"
          description="Add a rule to configure per-app DNS routing."
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
            <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">App Name</th>
            <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">DNS Server</th>
            <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Status</th>
            <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">LD_PRELOAD</th>
            <th className="px-6 py-3 text-right text-[10px] font-semibold uppercase tracking-widest text-slate-500">Actions</th>
          </tr>
        </thead>
        <tbody>
          {rules.map((rule) => (
            <tr key={rule.id} className="border-b border-surface-100/10 border-l-2 border-l-transparent">
              <td className="whitespace-nowrap px-6 py-4">
                <div>
                  <p className="text-sm font-medium text-surface-50">{rule.app_name}</p>
                  {rule.app_path && (
                    <p className="mt-0.5 text-xs text-slate-500 font-mono truncate max-w-xs">
                      {rule.app_path}
                    </p>
                  )}
                </div>
              </td>
              <td className="whitespace-nowrap px-6 py-4">
                <span className="text-sm text-slate-400">
                  {getServerName(rule.dns_server_id, dnsServers)}
                </span>
              </td>
              <td className="whitespace-nowrap px-6 py-4">
                <ToggleSwitch
                  enabled={rule.enabled}
                  onChange={() => rule.id !== undefined && onToggle(rule.id, !rule.enabled)}
                  label={`Toggle rule for ${rule.app_name}`}
                  disabled={rule.id === undefined}
                  variant="success"
                />
              </td>
              <td className="whitespace-nowrap px-6 py-4">
                {rule.use_ld_preload ? (
                  <span className="inline-flex items-center rounded-full bg-purple-500/15 px-2.5 py-0.5 text-xs font-medium text-purple-400 border border-purple-500/20">
                    Enabled
                  </span>
                ) : (
                  <span className="text-sm text-slate-600">Disabled</span>
                )}
              </td>
              <td className="whitespace-nowrap px-6 py-4 text-right">
                <div className="flex items-center justify-end gap-2">
                  {rule.use_ld_preload && rule.app_path && (
                    <>
                      {activePids[rule.id!] ? (
                        <button
                          type="button"
                          onClick={() => handleStop(rule)}
                          className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-danger-400 bg-danger-500/10 transition-colors hover:bg-danger-500/20"
                        >
                          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                          </svg>
                          Stop
                        </button>
                      ) : (
                        <button
                          type="button"
                          onClick={() => handleLaunch(rule)}
                          disabled={launchingId === rule.id}
                          className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-primary-400 bg-primary-500/10 transition-colors hover:bg-primary-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                          {launchingId === rule.id ? (
                            <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary-400/30 border-t-primary-400" />
                          ) : (
                            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
                            </svg>
                          )}
                          Launch
                        </button>
                      )}
                    </>
                  )}
                  <button
                    type="button"
                    onClick={() => onEdit(rule)}
                    className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-primary-400 bg-transparent transition-colors hover:bg-primary-500/10"
                  >
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Edit
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      if (rule.id !== undefined && window.confirm(`Delete rule for "${rule.app_name}"?`)) {
                        onDelete(rule.id);
                      }
                    }}
                    disabled={rule.id === undefined}
                    className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-danger-400 transition-colors hover:bg-danger-500/10 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    Delete
                  </button>
                </div>
                {feedbackMessage?.id === rule.id && (
                  <div className={`mt-2 text-xs font-medium text-right ${feedbackMessage!.type === 'success' ? 'text-success-400' : 'text-danger-400'}`}>
                    {feedbackMessage!.text}
                  </div>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      </div>
    </div>
  );
}

export default RulesList;
