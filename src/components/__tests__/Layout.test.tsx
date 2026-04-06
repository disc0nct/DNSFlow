import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import Layout from '../Layout';

describe('Layout', () => {
  const renderLayout = (initialRoute = '/') => {
    return render(
      <MemoryRouter initialEntries={[initialRoute]}>
        <Layout />
      </MemoryRouter>
    );
  };

  it('renders sidebar with all 5 navigation items', () => {
    renderLayout();

    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Rules')).toBeInTheDocument();
    expect(screen.getByText('DNS Servers')).toBeInTheDocument();
    expect(screen.getByText('Query Log')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('renders DNSFlow branding', () => {
    renderLayout();

    expect(screen.getByText('DNSFlow')).toBeInTheDocument();
  });

  it('highlights active NavLink for dashboard', () => {
    renderLayout('/');

    const dashboardLink = screen.getByText('Dashboard').closest('a');
    expect(dashboardLink).toHaveClass('bg-primary-500/10', 'text-primary-400');
  });

  it('highlights active NavLink for rules', () => {
    renderLayout('/rules');

    const rulesLink = screen.getByText('Rules').closest('a');
    expect(rulesLink).toHaveClass('bg-primary-500/10', 'text-primary-400');

    const dashboardLink = screen.getByText('Dashboard').closest('a');
    expect(dashboardLink).toHaveClass('text-slate-400');
  });

  it('highlights active NavLink for settings', () => {
    renderLayout('/settings');

    const settingsLink = screen.getByText('Settings').closest('a');
    expect(settingsLink).toHaveClass('bg-primary-500/10', 'text-primary-400');
  });

  it('renders Outlet content area', () => {
    const { container } = renderLayout();

    const main = container.querySelector('main');
    expect(main).toBeInTheDocument();
    expect(main).toHaveClass('flex-1');
  });

  it('renders navigation with SVG icons', () => {
    const { container } = renderLayout();

    const svgIcons = container.querySelectorAll('nav svg');
    expect(svgIcons.length).toBeGreaterThanOrEqual(5);
  });

  it('renders collapse sidebar button', () => {
    renderLayout();

    const collapseButton = screen.getByTitle('Collapse sidebar');
    expect(collapseButton).toBeInTheDocument();
  });
});
