/**
 * Comprehensive accessibility tests for Dashboard Builder component
 */

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, expectNoAccessibilityViolations } from '../../test-utils/accessibility';
import DashboardBuilder from '../../components/dashboard/DashboardBuilder';
import { ThemeProvider, createTheme } from '@mui/material/styles';

// Mock data
const mockReport = {
  id: 'test-report',
  projectName: 'Test Project',
  timestamp: new Date().toISOString(),
  summary: {
    totalFiles: 10,
    totalLines: 1000,
    findings: []
  },
  findings: [],
  metrics: {
    codeQuality: 85,
    maintainabilityIndex: 70,
    technicalDebt: 30
  },
  dependencyGraph: {
    nodes: [],
    edges: []
  }
};

const mockUser = {
  id: 'user-1',
  name: 'Test User',
  email: 'test@example.com'
};

const theme = createTheme();

const DashboardBuilderWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <ThemeProvider theme={theme}>
    {children}
  </ThemeProvider>
);

describe('Dashboard Builder Accessibility Tests', () => {
  test('Dashboard Builder should be accessible', async () => {
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    // Wait for component to render
    await screen.findByText('Dashboard Builder');

    // Check for accessibility violations
    await expectNoAccessibilityViolations(container);
  });

  test('Dashboard controls should be properly labeled', async () => {
    render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    // Check for properly labeled controls
    const editModeSwitch = screen.getByRole('checkbox', { 
      name: /toggle dashboard edit mode/i 
    });
    expect(editModeSwitch).toBeInTheDocument();

    const previewSwitch = screen.getByRole('checkbox', { 
      name: /toggle preview mode/i 
    });
    expect(previewSwitch).toBeInTheDocument();

    const addWidgetButton = screen.getByRole('button', { 
      name: /add new widget to dashboard/i 
    });
    expect(addWidgetButton).toBeInTheDocument();
  });

  test('Empty dashboard should be accessible', async () => {
    const emptyReport = { ...mockReport, findings: [] };
    
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={emptyReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    await expectNoAccessibilityViolations(container);
  });

  test('Widget panel should be accessible when opened', async () => {
    const user = userEvent.setup();
    
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    // Open widget panel
    const addWidgetButton = screen.getByRole('button', { 
      name: /add new widget to dashboard/i 
    });
    await user.click(addWidgetButton);

    // Wait for drawer to open (it might be a presentation role)
    await screen.findByText('Add Widget');

    // Check accessibility
    await expectNoAccessibilityViolations(container);
  });

  test('Widget tabs should be keyboard navigable', async () => {
    const user = userEvent.setup();
    
    render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    // Open widget panel
    const addWidgetButton = screen.getByRole('button', { 
      name: /add new widget to dashboard/i 
    });
    await user.click(addWidgetButton);

    // Find tabs
    const tabs = screen.getAllByRole('tab');
    expect(tabs).toHaveLength(3);

    // Check tab accessibility attributes
    tabs.forEach((tab, index) => {
      expect(tab).toHaveAttribute('id');
      expect(tab).toHaveAttribute('aria-controls');
    });
  });

  test('Loading state should be accessible', async () => {
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          loading={true}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    await expectNoAccessibilityViolations(container);
  });

  test('Error state should be accessible', async () => {
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          error="Test error message"
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    await expectNoAccessibilityViolations(container);
  });

  test('Read-only mode should be accessible', async () => {
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          readOnly={true}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    await expectNoAccessibilityViolations(container);
  });

  test('All interactive elements should have proper ARIA attributes', async () => {
    render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    // Check main heading
    const mainHeading = screen.getByRole('heading', { 
      name: /dashboard builder/i 
    });
    expect(mainHeading).toBeInTheDocument();
    expect(mainHeading).toHaveAttribute('id', 'dashboard-title');

    // Check switches have proper labels
    const switches = screen.getAllByRole('checkbox');
    switches.forEach(switchElement => {
      expect(switchElement).toHaveAttribute('aria-label');
    });
  });

  test('Color contrast should meet WCAG standards', async () => {
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    const results = await axe(container, {
      rules: {
        'color-contrast': { enabled: true }
      }
    });

    expect(results.violations.filter(v => v.id === 'color-contrast')).toHaveLength(0);
  });

  test('Should have no duplicate IDs', async () => {
    const { container } = render(
      <DashboardBuilderWrapper>
        <DashboardBuilder 
          report={mockReport} 
          currentUser={mockUser}
          testId="dashboard-builder"
        />
      </DashboardBuilderWrapper>
    );

    const results = await axe(container, {
      rules: {
        'duplicate-id': { enabled: true }
      }
    });

    expect(results.violations.filter(v => v.id === 'duplicate-id')).toHaveLength(0);
  });
});