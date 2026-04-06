import { useState, useEffect } from 'react';
import { inputClasses } from '../lib/constants';
import { type DnsServer } from '../lib/tauri';

interface DnsServerFormProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (name: string, address: string, secondary_address: string | undefined, protocol: string) => void;
  loading: boolean;
  initialData?: DnsServer | null;
}

interface Preset {
  name: string;
  address: string;
  secondaryAddress: string;
  protocol: string;
}

const PRESETS: Preset[] = [
  { name: 'Google', address: '8.8.8.8', secondaryAddress: '8.8.4.4', protocol: 'UDP' },
  { name: 'Cloudflare', address: '1.1.1.1', secondaryAddress: '1.0.0.1', protocol: 'UDP' },
  { name: 'Quad9', address: '9.9.9.9', secondaryAddress: '149.112.112.112', protocol: 'UDP' },
  { name: 'OpenDNS', address: '208.67.222.222', secondaryAddress: '208.67.220.220', protocol: 'UDP' },
  { name: 'AdGuard', address: '94.140.14.14', secondaryAddress: '94.140.15.15', protocol: 'UDP' },
];

const PROTOCOLS = ['UDP', 'TCP', 'DoH', 'DoT'];

function DnsServerForm({ isOpen, onClose, onSubmit, loading, initialData }: DnsServerFormProps) {
  const [name, setName] = useState('');
  const [address, setAddress] = useState('');
  const [secondary_address, setSecondaryAddress] = useState('');
  const [protocol, setProtocol] = useState('UDP');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      if (initialData) {
        setName(initialData.name);
        setAddress(initialData.address);
        setSecondaryAddress(initialData.secondary_address || '');
        setProtocol(initialData.protocol);
      } else {
        setName('');
        setAddress('');
        setSecondaryAddress('');
        setProtocol('UDP');
      }
      setError(null);
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

  const validateAddress = (addr: string): boolean => {
    const ipv4Pattern = /^(\d{1,3}\.){3}\d{1,3}$/;
    const ipv6Pattern = /^([0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}$/;
    const domainPattern = /^(https?:\/\/)?[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)*(\.[a-zA-Z]{2,})?(\/.*)?$/;

    if (ipv4Pattern.test(addr)) {
      const octets = addr.split('.').map(Number);
      return octets.every((octet) => octet >= 0 && octet <= 255);
    }

    if (protocol === 'DoH' || protocol === 'DoT') {
      return domainPattern.test(addr) || ipv4Pattern.test(addr) || ipv6Pattern.test(addr);
    }

    return ipv4Pattern.test(addr) || ipv6Pattern.test(addr);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const trimmedName = name.trim();
    const trimmedAddress = address.trim();
    const trimmedSecondary = secondary_address.trim();

    if (!trimmedName) {
      setError('Name is required');
      return;
    }

    if (!trimmedAddress) {
      setError('Primary address is required');
      return;
    }

    if (!validateAddress(trimmedAddress)) {
      setError('Invalid primary address format');
      return;
    }

    if (trimmedSecondary && !validateAddress(trimmedSecondary)) {
      setError('Invalid secondary address format');
      return;
    }

    console.log("Form submitting secondary:", trimmedSecondary);
    onSubmit(trimmedName, trimmedAddress, trimmedSecondary || undefined, protocol);
  };

  const applyPreset = (preset: Preset) => {
    setName(preset.name);
    setAddress(preset.address);
    setSecondaryAddress(preset.secondaryAddress);
    setProtocol(preset.protocol);
    setError(null);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in sm:p-6 md:p-8">
      <div
        className="absolute inset-0 modal-backdrop"
        onClick={onClose}
      />

      <div role="dialog" aria-modal="true" aria-label="Add DNS Server" className="relative w-full max-w-md rounded-xl bg-surface-400 border border-surface-100/30 p-6 shadow-2xl animate-slide-in-up max-h-[90vh] overflow-y-auto mx-auto my-auto">
        <div className="mb-6 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary-500/15">
              <svg className="h-4 w-4 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418" />
              </svg>
            </div>
            <h2 className="text-base font-semibold text-surface-50">{initialData ? 'Edit DNS Server' : 'Add DNS Server'}</h2>
          </div>
          <button
            onClick={onClose}
            aria-label="Close"
            className="rounded-lg p-1.5 text-slate-500 transition-colors hover:bg-surface-100/30 hover:text-slate-300"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="mb-6">
          <label className="mb-2 block text-[10px] font-semibold uppercase tracking-widest text-slate-500">
            Quick Setup
          </label>
          <div className="flex flex-wrap gap-2">
            {PRESETS.map((preset) => (
              <button
                key={preset.name}
                type="button"
                onClick={() => applyPreset(preset)}
                className="flex-1 min-w-[80px] rounded-lg border border-surface-100/30 bg-surface-100/10 px-3 py-2 text-sm font-medium text-slate-300 transition-all hover:border-primary-500/30 hover:bg-primary-500/10 hover:text-primary-400"
              >
                {preset.name}
              </button>
            ))}
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="rounded-lg border border-danger-500/30 bg-danger-500/10 px-4 py-3 text-sm text-danger-400">
              {error}
            </div>
          )}

          <div>
            <label htmlFor="name" className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">
              Name
            </label>
            <input
              type="text"
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Google DNS"
              className={inputClasses}
            />
          </div>

          <div>
            <label htmlFor="address" className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">
              Primary Address
            </label>
            <input
              type="text"
              id="address"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="e.g., 8.8.8.8 or dns.google"
              className={`font-mono ${inputClasses}`}
            />
          </div>

          <div>
            <label htmlFor="secondary-address" className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">
              Secondary Address (Optional)
            </label>
            <input
              type="text"
              id="secondary-address"
              value={secondary_address}
              onChange={(e) => setSecondaryAddress(e.target.value)}
              placeholder="e.g., 8.8.4.4"
              className={`font-mono ${inputClasses}`}
            />
          </div>

          <div>
            <label htmlFor="protocol" className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">
              Protocol
            </label>
            <select
              id="protocol"
              value={protocol}
              onChange={(e) => setProtocol(e.target.value)}
              className={inputClasses}
            >
              {PROTOCOLS.map((p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))}
            </select>
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
              {loading ? 'Saving...' : initialData ? 'Update' : 'Save'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default DnsServerForm;
