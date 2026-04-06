import { useState, useEffect, useCallback } from 'react';
import { ruleApi, dnsApi, type AppRule, type DnsServer } from '../lib/tauri';
import RulesList from '../components/RulesList';
import RuleForm from '../components/RuleForm';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { getErrorMessage } from '../lib/utils';

function Rules() {
  const [rules, setRules] = useState<AppRule[]>([]);
  const [dnsServers, setDnsServers] = useState<DnsServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [formOpen, setFormOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingRule, setEditingRule] = useState<AppRule | null>(null);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      const [rulesResult, serversResult] = await Promise.all([
        ruleApi.getRules(),
        dnsApi.getDnsServers(),
      ]);
      setRules(rulesResult);
      setDnsServers(serversResult);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to load rules'));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleAddOrUpdateRule = async (
    appName: string,
    appPath: string | undefined,
    dnsServerId: number,
    useLdPreload: boolean
  ) => {
    try {
      setSubmitting(true);
      if (editingRule?.id) {
        await ruleApi.updateRule(editingRule.id, appName, appPath, dnsServerId, useLdPreload);
      } else {
        await ruleApi.addRule(appName, appPath, dnsServerId, useLdPreload);
      }
      setFormOpen(false);
      setEditingRule(null);
      await fetchData();
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to save rule'));
    } finally {
      setSubmitting(false);
    }
  };

  const handleEditRule = (rule: AppRule) => {
    setEditingRule(rule);
    setFormOpen(true);
  };

  const handleToggleRule = async (id: number, enabled: boolean) => {
    try {
      await ruleApi.toggleRule(id, enabled);
      await fetchData();
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to toggle rule'));
    }
  };

  const handleDeleteRule = async (id: number) => {
    try {
      await ruleApi.deleteRule(id);
      await fetchData();
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to delete rule'));
    }
  };

  return (
    <div className="animate-fade-in relative z-0">
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-50">DNS Rules</h1>
          <p className="mt-1 text-sm text-slate-400">
            Configure per-application DNS routing and filtering rules.
          </p>
        </div>
        <button
          type="button"
          onClick={() => {
            setEditingRule(null);
            setFormOpen(true);
          }}
          className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-500 to-accent-500 px-4 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:from-primary-400 hover:to-accent-400 shadow-lg shadow-primary-500/20"
        >
          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          Add Rule
        </button>
      </div>

      {error && <div className="mb-6"><ErrorBanner message={error} onDismiss={() => setError(null)} /></div>}

      <RulesList
        rules={rules}
        dnsServers={dnsServers}
        loading={loading}
        onToggle={handleToggleRule}
        onEdit={handleEditRule}
        onDelete={handleDeleteRule}
      />

      <RuleForm
        isOpen={formOpen}
        onClose={() => {
          setFormOpen(false);
          setEditingRule(null);
        }}
        onSubmit={handleAddOrUpdateRule}
        loading={submitting}
        initialData={editingRule}
      />
    </div>
  );
}

export default Rules;
