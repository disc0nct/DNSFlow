import { useState, useEffect, useCallback } from 'react';
import { dnsApi, type DnsServer } from '../lib/tauri';
import DnsServerList from '../components/DnsServerList';
import DnsServerForm from '../components/DnsServerForm';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { getErrorMessage } from '../lib/utils';

function DnsServers() {
  const [servers, setServers] = useState<DnsServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [formOpen, setFormOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingServer, setEditingServer] = useState<DnsServer | null>(null);

  const fetchServers = useCallback(async () => {
    try {
      setLoading(true);
      const result = await dnsApi.getDnsServers();
      setServers(result);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to load DNS servers'));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchServers();
  }, [fetchServers]);

  const handleAddOrUpdate = async (name: string, address: string, secondary_address: string | undefined, protocol: string) => {
    setSubmitting(true);
    try {
      if (editingServer?.id) {
        await dnsApi.updateDnsServer(editingServer.id, name, address, secondary_address, protocol);
      } else {
        await dnsApi.addDnsServer(name, address, secondary_address, protocol);
      }
      setFormOpen(false);
      setEditingServer(null);
      await fetchServers();
    } catch (err) {
      console.error("API error:", err);
      setError(getErrorMessage(err, String(err)));
    } finally {
      setSubmitting(false);
    }
  };

  const handleEdit = (server: DnsServer) => {
    setEditingServer(server);
    setFormOpen(true);
  };

  const handleDelete = async (id: number) => {
    try {
      await dnsApi.deleteDnsServer(id);
      await fetchServers();
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to delete DNS server'));
    }
  };

  const handleSetDefault = async (id: number) => {
    try {
      await dnsApi.setDefaultDnsServer(id);
      await fetchServers();
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to set default DNS server'));
    }
  };

  return (
    <div className="animate-fade-in relative z-0">
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-50">DNS Servers</h1>
          <p className="mt-1 text-sm text-slate-400">
            Configure upstream DNS servers and resolver settings.
          </p>
        </div>
        <button
          onClick={() => {
            setEditingServer(null);
            setFormOpen(true);
          }}
          className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-500 to-accent-500 px-4 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:from-primary-400 hover:to-accent-400 shadow-lg shadow-primary-500/20"
        >
          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          Add DNS Server
        </button>
      </div>

      {error && <div className="mb-6"><ErrorBanner message={error} onDismiss={() => setError(null)} /></div>}

      <DnsServerList
        servers={servers}
        loading={loading}
        onEdit={handleEdit}
        onDelete={handleDelete}
        onSetDefault={handleSetDefault}
      />

      <DnsServerForm
        isOpen={formOpen}
        onClose={() => {
          setFormOpen(false);
          setEditingServer(null);
        }}
        onSubmit={handleAddOrUpdate}
        loading={submitting}
        initialData={editingServer}
      />
    </div>
  );
}

export default DnsServers;
