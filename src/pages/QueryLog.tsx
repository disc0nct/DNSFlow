import { useState, useEffect, useMemo, useCallback } from 'react';
import { queryLogApi, type DnsQueryLog } from '../lib/tauri';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { EmptyState } from '../components/ui/EmptyState';
import { inputClasses, LATENCY_GOOD, LATENCY_OK, SUCCESS_MESSAGE_DURATION } from '../lib/constants';
import { getErrorMessage } from '../lib/utils';

const PAGE_SIZE_OPTIONS = [10, 25, 50];

function QueryLog() {
  const [queries, setQueries] = useState<DnsQueryLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [domainFilter, setDomainFilter] = useState('');
  const [appFilter, setAppFilter] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [pageSize, setPageSize] = useState(10);
  const [currentPage, setCurrentPage] = useState(1);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [clearing, setClearing] = useState(false);

  const fetchLogs = useCallback(async (isInitial = false) => {
    try {
      if (isInitial) setLoading(true);
      const logs = await queryLogApi.getQueryLogs();
      setQueries(logs);
      setError(null);
    } catch (err) {
      console.error('Failed to fetch query logs:', err);
      setError(getErrorMessage(err, 'Failed to load query logs'));
    } finally {
      if (isInitial) setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchLogs(true);
    const interval = setInterval(() => fetchLogs(false), 3000);
    return () => clearInterval(interval);
  }, [fetchLogs]);

  const handleClearLogs = async () => {
    if (!window.confirm('Are you sure you want to clear all query logs? This action cannot be undone.')) {
      return;
    }
    
    setClearing(true);
    try {
      await queryLogApi.clearQueryLogs();
      setQueries([]);
      setSuccess('Query logs cleared successfully');
      setTimeout(() => setSuccess(null), SUCCESS_MESSAGE_DURATION);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to clear query logs'));
    } finally {
      setClearing(false);
    }
  };

  const appNames = useMemo(() => {
    const unique = new Set(queries.map((q) => q.app_name).filter(Boolean));
    return Array.from(unique).sort() as string[];
  }, [queries]);

  const filteredQueries = useMemo(() => {
    return queries.filter((query) => {
      if (domainFilter && !query.domain.toLowerCase().includes(domainFilter.toLowerCase())) {
        return false;
      }
      if (appFilter && query.app_name !== appFilter) {
        return false;
      }
      if (dateFrom && query.timestamp < dateFrom) {
        return false;
      }
      if (dateTo && query.timestamp > dateTo + ' 23:59:59') {
        return false;
      }
      return true;
    });
  }, [queries, domainFilter, appFilter, dateFrom, dateTo]);

  const totalPages = Math.ceil(filteredQueries.length / pageSize);
  const paginatedQueries = useMemo(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredQueries.slice(start, start + pageSize);
  }, [filteredQueries, currentPage, pageSize]);

  const handleFilterChange = (setter: (value: string) => void, value: string) => {
    setter(value);
    setCurrentPage(1);
  };

  const handlePageSizeChange = (size: number) => {
    setPageSize(size);
    setCurrentPage(1);
  };

  const clearFilters = () => {
    setDomainFilter('');
    setAppFilter('');
    setDateFrom('');
    setDateTo('');
    setCurrentPage(1);
  };
  const hasActiveFilters = domainFilter || appFilter || dateFrom || dateTo;

  const getLatencyColor = (latency: number | undefined) => {
    const ms = latency ?? 0;
    if (ms < LATENCY_GOOD) return 'text-success-400';
    if (ms < LATENCY_OK) return 'text-warning-400';
    return 'text-danger-400';
  };

  if (loading) {
    return (
      <div className="animate-fade-in">
        <div className="mb-8">
          <h1 className="text-2xl font-bold text-surface-50">Query Log</h1>
          <p className="mt-1 text-sm text-slate-400">
            Browse and search DNS query history with filtering options.
          </p>
        </div>
        <div className="space-y-4">
          <div className="h-32 w-full animate-pulse rounded-xl bg-surface-100/20" />
          <div className="h-64 w-full animate-pulse rounded-xl bg-surface-100/10" />
        </div>
      </div>
    );
  }

  return (
    <div className="animate-fade-in">
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-50">Query Log</h1>
          <p className="mt-1 text-sm text-slate-400">
            Browse and search DNS query history with filtering options.
          </p>
        </div>
        <button
          onClick={handleClearLogs}
          disabled={clearing || queries.length === 0}
          className="inline-flex items-center gap-1.5 rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-sm font-medium text-danger-400 transition-colors hover:bg-danger-500/20 hover:text-danger-300 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {clearing ? (
            <div className="h-4 w-4 animate-spin rounded-full border-2 border-danger-400 border-t-transparent" />
          ) : (
            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          )}
          Clear Logs
        </button>
      </div>

      {error && <div className="mb-6"><ErrorBanner message={error as string} onDismiss={() => setError(null)} /></div>}
      
      {success && (
        <div className="mb-6 rounded-lg border border-success-500/30 bg-success-500/10 px-4 py-3 text-sm text-success-400 animate-fade-in">
          <div className="flex items-center gap-2">
            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {success}
          </div>
        </div>
      )}

      <div className="mb-6 glass-card rounded-xl p-6">
        <div className="flex flex-wrap items-end gap-4">
          <div className="flex-1 min-w-[200px]">
            <label htmlFor="domain-filter" className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
              Domain
            </label>
            <input
              type="text"
              id="domain-filter"
              placeholder="Search domains..."
              value={domainFilter}
              onChange={(e) => handleFilterChange(setDomainFilter, e.target.value)}
              className={inputClasses}
            />
          </div>

          <div className="flex-1 min-w-[180px]">
            <label htmlFor="app-filter" className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
              App Name
            </label>
            <select
              id="app-filter"
              value={appFilter}
              onChange={(e) => handleFilterChange(setAppFilter, e.target.value)}
              className={inputClasses}
            >
              <option value="">All Apps</option>
              {appNames.map((name) => (
                <option key={name} value={name}>
                  {name}
                </option>
              ))}
            </select>
          </div>

          <div className="flex-1 min-w-[150px]">
            <label htmlFor="date-from" className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
              From
            </label>
            <input
              type="date"
              id="date-from"
              value={dateFrom}
              onChange={(e) => handleFilterChange(setDateFrom, e.target.value)}
              className={inputClasses}
            />
          </div>

          <div className="flex-1 min-w-[150px]">
            <label htmlFor="date-to" className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
              To
            </label>
            <input
              type="date"
              id="date-to"
              value={dateTo}
              onChange={(e) => handleFilterChange(setDateTo, e.target.value)}
              className={inputClasses}
            />
          </div>

          {hasActiveFilters && (
            <button
              onClick={clearFilters}
              className="inline-flex items-center gap-1.5 rounded-lg border border-surface-100/50 bg-surface-100/20 px-3 py-2 text-sm font-medium text-slate-300 transition-colors hover:bg-surface-100/40 hover:text-surface-50"
            >
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
              Clear Filters
            </button>
          )}
        </div>
      </div>

      <div className="mb-4 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <p className="text-sm text-slate-500">
          Showing <span className="text-slate-300 font-medium">{paginatedQueries.length}</span> of <span className="text-slate-300 font-medium">{filteredQueries.length}</span> queries
          {hasActiveFilters && <span className="text-slate-600"> (filtered from {queries.length} total)</span>}
        </p>
        <div className="flex items-center gap-2">
          <label htmlFor="page-size" className="text-sm text-slate-500">
            Per page:
          </label>
          <select
            id="page-size"
            value={pageSize}
            onChange={(e) => handlePageSizeChange(Number(e.target.value))}
            className="rounded-lg border border-surface-100/50 bg-surface-100/20 pl-2 pr-8 py-1 text-sm text-surface-50 focus:border-primary-500/50 focus:outline-none focus:ring-1 focus:ring-primary-500/20"
          >
            {PAGE_SIZE_OPTIONS.map((size) => (
              <option key={size} value={size}>
                {size}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="glass-card rounded-xl overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full dark-table">
            <thead>
              <tr className="border-b border-surface-100/20">
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Timestamp</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Domain</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">App Name</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Resolved IP</th>
                <th className="px-6 py-3 text-left text-[10px] font-semibold uppercase tracking-widest text-slate-500">Latency</th>
              </tr>
            </thead>
            <tbody>
              {paginatedQueries.length > 0 ? (
                paginatedQueries.map((query, index) => (
                  <tr key={query.id ?? index} className="border-b border-surface-100/10 border-l-2 border-l-transparent hover:bg-surface-100/5">
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-xs text-slate-500 font-mono">{query.timestamp}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-sm font-medium text-surface-50 font-mono">{query.domain}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-sm text-slate-400">{query.app_name || 'Unknown'}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className="text-sm text-slate-400 font-mono">{query.resolved_ip || '-'}</span>
                    </td>
                    <td className="whitespace-nowrap px-6 py-3">
                      <span className={`text-sm font-medium font-mono ${getLatencyColor(query.latency_ms)}`}>
                        {query.latency_ms ?? 0}ms
                      </span>
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan={5}>
                    <EmptyState
                      icon={
                        <svg className="h-8 w-8 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
                        </svg>
                      }
                      title={hasActiveFilters ? 'No queries found' : 'No query logs yet'}
                      description={hasActiveFilters
                        ? 'Try adjusting your filters to see more results.'
                        : 'DNS query history will appear here once the proxy starts resolving queries.'}
                      action={hasActiveFilters ? { label: 'Clear Filters', onClick: clearFilters } : undefined}
                    />
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {totalPages > 1 && (
          <div className="flex items-center justify-between border-t border-surface-100/20 bg-surface-100/10 px-6 py-3">
            <button
              onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
              disabled={currentPage === 1}
              className="inline-flex items-center gap-1.5 rounded-lg border border-surface-100/50 bg-surface-100/20 px-3 py-1.5 text-sm font-medium text-slate-300 transition-colors hover:bg-surface-100/40 hover:text-surface-50 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.75 19.5L8.25 12l7.5-7.5" />
              </svg>
              Previous
            </button>
            <span className="text-sm text-slate-500">
              Page <span className="text-surface-50 font-medium">{currentPage}</span> of <span className="text-surface-50 font-medium">{totalPages}</span>
            </span>
            <button
              onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages}
              className="inline-flex items-center gap-1.5 rounded-lg border border-surface-100/50 bg-surface-100/20 px-3 py-1.5 text-sm font-medium text-slate-300 transition-colors hover:bg-surface-100/40 hover:text-surface-50 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Next
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.25 4.5l7.5 7.5-7.5 7.5" />
              </svg>
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default QueryLog;
