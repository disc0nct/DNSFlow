import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import Dashboard from '../Dashboard';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case 'get_dns_status':
          return 'stopped';
        case 'get_dns_servers':
          return [
            { id: 1, name: 'Cloudflare', address: '1.1.1.1', secondary_address: undefined, protocol: 'udp', is_default: true },
          ];
        case 'get_rules':
          return [
            {
              id: 1,
              app_name: 'firefox',
              app_path: '/usr/bin/firefox',
              dns_server_id: 1,
              enabled: true,
              use_ld_preload: false,
            },
          ];
        case 'get_query_logs':
          return [
            { id: 1, domain: 'google.com', app_name: 'firefox', dns_server_id: 1, latency_ms: 23, timestamp: '2024-01-15 10:30:45' },
            { id: 2, domain: 'github.com', app_name: 'chrome', dns_server_id: 1, latency_ms: 18, timestamp: '2024-01-15 10:30:42' },
          ];
        default:
          return [];
      }
    });
  });

  const renderDashboard = () => {
    return render(
      <MemoryRouter>
        <Dashboard />
      </MemoryRouter>
    );
  };

  it('renders loading shimmer initially', () => {
    const { container } = renderDashboard();

    const shimmerElements = container.querySelectorAll('.animate-shimmer');
    expect(shimmerElements.length).toBeGreaterThan(0);
  });

  it('shows proxy status card after data loads', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Proxy Status')).toBeInTheDocument();
    });

    expect(screen.getByText('DNS proxy is not running')).toBeInTheDocument();
  });

  it('displays stats cards with queries, rules, and latency', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Total Queries')).toBeInTheDocument();
    });

    expect(screen.getAllByText('Active Rules').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Avg Latency')).toBeInTheDocument();
  });

  it('renders Start Proxy button when stopped', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Start Proxy')).toBeInTheDocument();
    });
  });

  it('toggles proxy status on button click', async () => {
    const user = userEvent.setup();

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'start_dns_proxy') return true;
      if (cmd === 'get_dns_status') return 'stopped';
      if (cmd === 'get_dns_servers') return [];
      if (cmd === 'get_rules') return [];
      if (cmd === 'get_query_logs') return [];
      return [];
    });

    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Start Proxy')).toBeInTheDocument();
    });

    const startButton = screen.getByText('Start Proxy').closest('button');
    await user.click(startButton!);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('start_dns_proxy');
    });
  });

  it('renders recent queries table', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Recent Queries')).toBeInTheDocument();
    });

    expect(screen.getByText('Domain')).toBeInTheDocument();
    expect(screen.getByText('App')).toBeInTheDocument();
    expect(screen.getByText('DNS Server')).toBeInTheDocument();
    expect(screen.getByText('Latency')).toBeInTheDocument();
    expect(screen.getByText('Time')).toBeInTheDocument();
  });

  it('displays query entries in table', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('google.com')).toBeInTheDocument();
    });

    expect(screen.getByText('github.com')).toBeInTheDocument();
    // 'firefox' appears in both queries table and active rules section
    expect(screen.getAllByText('firefox').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('chrome').length).toBeGreaterThanOrEqual(1);
  });

  it('shows View All link to query log', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('View All')).toBeInTheDocument();
    });
  });

  it('displays system DNS servers', async () => {
    renderDashboard();

    await waitFor(() => {
      // 'Cloudflare' appears in both DNS servers section and active rules
      expect(screen.getAllByText('Cloudflare').length).toBeGreaterThanOrEqual(1);
    });

    expect(screen.getByText('1.1.1.1')).toBeInTheDocument();
  });

  it('shows protected applications count', async () => {
    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Protected Applications')).toBeInTheDocument();
    });
  });

  it('renders active rules section', async () => {
    renderDashboard();

    await waitFor(() => {
      // "Active Rules" appears in both stats card and section header
      expect(screen.getAllByText('Active Rules').length).toBeGreaterThanOrEqual(1);
    });

    expect(screen.getByText('Manage Rules')).toBeInTheDocument();
  });

  it('displays error message on fetch failure', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Network error')).toBeInTheDocument();
    });
  });
});
