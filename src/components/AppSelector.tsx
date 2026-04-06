import { useState, useEffect, useCallback } from 'react';
import { processApi, filePicker, type ProcessInfo } from '../lib/tauri';
import { inputClasses } from '../lib/constants';
import { getErrorMessage } from '../lib/utils';

interface AppSelectorProps {
  onSelect: (appName: string, appPath: string | undefined) => void;
  selectedApp?: { name: string; path?: string };
}

type TabType = 'processes' | 'browse';

function AppSelector({ onSelect, selectedApp }: AppSelectorProps) {
  const [activeTab, setActiveTab] = useState<TabType>('processes');
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [manualPath, setManualPath] = useState('');

  const fetchProcesses = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await processApi.getRunningProcesses();
      setProcesses(result);
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to load processes'));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (activeTab === 'processes') {
      fetchProcesses();
    }
  }, [activeTab, fetchProcesses]);

  const filteredProcesses = processes.filter(
    (p) =>
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (p.exe_path && p.exe_path.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  const handleProcessSelect = (process: ProcessInfo) => {
    onSelect(process.name, process.exe_path);
  };

  const handleBrowseFile = async () => {
    try {
      const path = await filePicker.pickExecutable();
      if (path) {
        setManualPath(path);
        const name = path.split('/').pop() || path.split('\\').pop() || path;
        onSelect(name, path);
      }
    } catch (err) {
      setError(getErrorMessage(err, 'Failed to open file picker'));
    }
  };

  const handleManualPathSubmit = () => {
    if (!manualPath.trim()) {
      setError('Please enter a valid path');
      return;
    }
    const name = manualPath.split('/').pop() || manualPath.split('\\').pop() || manualPath;
    onSelect(name, manualPath.trim());
    setError(null);
  };

  return (
    <div className="rounded-lg border border-surface-100/30 bg-surface-100/10 overflow-hidden">
      <div className="border-b border-surface-100/20">
        <div className="flex">
          <button
            type="button"
            onClick={() => setActiveTab('processes')}
            className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === 'processes'
                ? 'border-b-2 border-primary-500 text-primary-400'
                : 'text-slate-500 hover:text-slate-300'
            }`}
          >
            Running Processes
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('browse')}
            className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === 'browse'
                ? 'border-b-2 border-primary-500 text-primary-400'
                : 'text-slate-500 hover:text-slate-300'
            }`}
          >
            Browse Files
          </button>
        </div>
      </div>

      <div className="p-4">
        {activeTab === 'processes' ? (
          <div className="space-y-3">
            <div className="relative">
              <svg className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
              </svg>
              <input
                type="text"
                placeholder="Search processes..."
                aria-label="Search running processes"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className={`${inputClasses} pl-10`}
              />
            </div>

            {loading ? (
              <div className="flex items-center justify-center py-8">
                <div className="flex flex-col items-center gap-3">
                  <div className="h-6 w-6 animate-spin rounded-full border-2 border-surface-100/50 border-t-primary-500" />
                  <p className="text-sm text-slate-500">Loading processes...</p>
                </div>
              </div>
            ) : error ? (
              <div className="rounded-lg border border-danger-500/30 bg-danger-500/10 px-4 py-3 text-sm text-danger-400">
                {error}
              </div>
            ) : (
              <div className="max-h-48 overflow-y-auto rounded-lg border border-surface-100/20">
                {filteredProcesses.length === 0 ? (
                  <div className="px-4 py-6 text-center text-sm text-slate-500">
                    No processes found
                  </div>
                ) : (
                  <ul>
                    {filteredProcesses.map((process) => (
                      <li
                        key={process.pid}
                        tabIndex={0}
                        role="button"
                        aria-label={`Select ${process.name}`}
                        onClick={() => handleProcessSelect(process)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            handleProcessSelect(process);
                          }
                        }}
                        className={`cursor-pointer px-4 py-3 border-b border-surface-100/10 last:border-b-0 transition-colors hover:bg-primary-500/[0.05] focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:ring-inset ${
                          selectedApp?.name === process.name
                            ? 'bg-primary-500/10 border-l-2 border-l-primary-500'
                            : 'border-l-2 border-l-transparent'
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <div>
                            <p className="text-sm font-medium text-surface-50">
                              {process.name}
                            </p>
                            {process.exe_path && (
                              <p className="mt-0.5 text-xs text-slate-500 truncate max-w-xs font-mono">
                                {process.exe_path}
                              </p>
                            )}
                          </div>
                          <span className="text-xs text-slate-600 font-mono">
                            PID:{process.pid}
                          </span>
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="/path/to/executable"
                aria-label="Enter executable path manually"
                value={manualPath}
                onChange={(e) => setManualPath(e.target.value)}
                className={`flex-1 font-mono ${inputClasses}`}
              />
              <button
                type="button"
                onClick={handleBrowseFile}
                className="rounded-lg border border-surface-100/50 bg-surface-100/20 px-4 py-2.5 text-sm font-medium text-slate-300 transition-colors hover:bg-surface-100/40 hover:text-surface-50"
              >
                Browse
              </button>
            </div>
            <button
              type="button"
              onClick={handleManualPathSubmit}
              disabled={!manualPath.trim()}
              className="w-full rounded-lg bg-gradient-to-r from-primary-500 to-accent-500 px-4 py-2.5 text-sm font-medium text-white transition-all hover:from-primary-400 hover:to-accent-400 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Select Path
            </button>
            {error && (
              <div className="rounded-lg border border-danger-500/30 bg-danger-500/10 px-4 py-3 text-sm text-danger-400">
                {error}
              </div>
            )}
          </div>
        )}

        {selectedApp && (
          <div className="mt-4 rounded-lg bg-primary-500/10 border border-primary-500/20 px-4 py-3">
            <p className="text-[10px] font-semibold uppercase tracking-widest text-primary-400">
              Selected Application
            </p>
            <p className="mt-1 text-sm font-medium text-surface-50">{selectedApp.name}</p>
            {selectedApp.path && (
              <p className="mt-0.5 text-xs text-slate-500 font-mono truncate">
                {selectedApp.path}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default AppSelector;
