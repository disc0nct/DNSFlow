import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Settings from '../Settings';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('Settings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case 'get_config':
          return {
            proxy_port: 5353,
            log_enabled: true,
            auto_start: false,
            debug: false,
          };
        case 'get_dns_servers':
          return [
            { id: 1, name: 'Cloudflare', address: '1.1.1.1', secondary_address: undefined, protocol: 'udp', is_default: true },
          ];
        default:
          return null;
      }
    });
  });

  const renderSettings = () => {
    return render(<Settings />);
  };

  it('renders loading shimmer initially', () => {
    const { container } = renderSettings();

    const shimmerElements = container.querySelectorAll('.animate-shimmer');
    expect(shimmerElements.length).toBeGreaterThan(0);
  });

  it('renders settings form after loading', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Settings')).toBeInTheDocument();
    });

    expect(screen.getByText('General Settings')).toBeInTheDocument();
  });

  it('displays auto-start toggle', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Auto-start on boot')).toBeInTheDocument();
    });

    expect(screen.getByText('Launch DNSFlow automatically when system starts')).toBeInTheDocument();
  });

  it('displays log DNS queries toggle', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Log DNS queries')).toBeInTheDocument();
    });

    expect(screen.getByText('Record DNS query history for analysis')).toBeInTheDocument();
  });

  it('displays proxy port input', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Proxy port')).toBeInTheDocument();
    });

    const portInput = screen.getByDisplayValue('5353');
    expect(portInput).toBeInTheDocument();
  });

  it('renders export config button', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Export Config')).toBeInTheDocument();
    });
  });

  it('renders import config button', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Import Config')).toBeInTheDocument();
    });
  });

  it('renders reset to defaults button', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Reset to Defaults')).toBeInTheDocument();
    });
  });

  it('displays system DNS servers', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('System DNS Servers')).toBeInTheDocument();
    });

    expect(screen.getByText('Cloudflare')).toBeInTheDocument();
    expect(screen.getByText('1.1.1.1')).toBeInTheDocument();
  });

  it('displays app version', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('App Version')).toBeInTheDocument();
    });

    expect(screen.getByText('v1.0.0')).toBeInTheDocument();
  });

  it('toggles auto-start setting', async () => {
    const user = userEvent.setup();

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'update_config') return true;
      if (cmd === 'get_config') return { proxy_port: 5353, log_enabled: true, auto_start: false, debug: false };
      if (cmd === 'get_dns_servers') return [];
      return null;
    });

    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Auto-start on boot')).toBeInTheDocument();
    });

    const switches = screen.getAllByRole('switch');
    const autoStartToggle = switches[0];
    await user.click(autoStartToggle);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('update_config', expect.objectContaining({
        config: expect.objectContaining({ auto_start: true }),
      }));
    });
  });

  it('shows success message after saving', async () => {
    const user = userEvent.setup();

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'update_config') return true;
      if (cmd === 'get_config') return { proxy_port: 5353, log_enabled: true, auto_start: false, debug: false };
      if (cmd === 'get_dns_servers') return [];
      return null;
    });

    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Log DNS queries')).toBeInTheDocument();
    });

    const switches = screen.getAllByRole('switch');
    const logToggle = switches[1];
    await user.click(logToggle);

    await waitFor(() => {
      expect(screen.getByText('Settings saved successfully')).toBeInTheDocument();
    });
  });

  it('displays error message on save failure', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') throw new Error('Failed to load settings');
      if (cmd === 'get_dns_servers') return [];
      return null;
    });

    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Failed to load settings')).toBeInTheDocument();
    });
  });

  it('displays data management section', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('Data Management')).toBeInTheDocument();
    });

    expect(screen.getByText('Export, import, or reset your configuration')).toBeInTheDocument();
  });

  it('displays system info section', async () => {
    renderSettings();

    await waitFor(() => {
      expect(screen.getByText('System Info')).toBeInTheDocument();
    });

    expect(screen.getByText('Current system configuration details')).toBeInTheDocument();
  });
});
