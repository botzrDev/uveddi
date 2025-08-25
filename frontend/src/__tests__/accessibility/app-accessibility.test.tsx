/**
 * Comprehensive accessibility tests for the main App component
 */

import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { axe, expectNoAccessibilityViolations } from '../../test-utils/accessibility';
import App from '../../App';

// Mock the RealtimeUpdates component to avoid WebSocket issues in tests
jest.mock('../../components/RealtimeUpdates', () => {
  return function MockRealtimeUpdates() {
    return null;
  };
});

// Mock the pages to avoid complex component tree
jest.mock('../../pages/DashboardPage', () => {
  return function MockDashboardPage() {
    return <div data-testid="dashboard-page">Dashboard Content</div>;
  };
});

jest.mock('../../pages/ReportPage', () => {
  return function MockReportPage() {
    return <div data-testid="report-page">Report Content</div>;
  };
});

jest.mock('../../pages/ReportsListPage', () => {
  return function MockReportsListPage() {
    return <div data-testid="reports-list-page">Reports List Content</div>;
  };
});

describe('App Accessibility Tests', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
    
    // Mock matchMedia for theme detection
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: jest.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: jest.fn(),
        removeListener: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn(),
      })),
    });
  });

  test('App component should be accessible in light mode', async () => {
    const { container } = render(<App />);
    
    // Wait for the app to render
    await screen.findByText('Uveddi');
    
    // Check for accessibility violations
    await expectNoAccessibilityViolations(container);
  });

  test('App component should be accessible in dark mode', async () => {
    // Set dark theme preference
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: jest.fn().mockImplementation(query => ({
        matches: query === '(prefers-color-scheme: dark)',
        media: query,
        onchange: null,
        addListener: jest.fn(),
        removeListener: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn(),
      })),
    });

    const { container } = render(<App />);
    
    // Wait for the app to render
    await screen.findByText('Uveddi');
    
    // Check for accessibility violations
    await expectNoAccessibilityViolations(container);
  });

  test('Skip link should be present and functional', async () => {
    render(<App />);
    
    // Check for skip link
    const skipLink = screen.getByText('Skip to main content');
    expect(skipLink).toBeInTheDocument();
    expect(skipLink).toHaveAttribute('href', '#main-content');
  });

  test('Main content should have proper landmark', async () => {
    render(<App />);
    
    // Check for main landmark
    const mainContent = document.getElementById('main-content');
    expect(mainContent).toBeInTheDocument();
  });

  test('App should have proper heading hierarchy', async () => {
    render(<App />);
    
    // Check for main heading
    const mainHeading = screen.getByRole('heading', { level: 1 });
    expect(mainHeading).toBeInTheDocument();
    expect(mainHeading).toHaveTextContent('Uveddi');
  });

  test('Theme toggle should be accessible', async () => {
    render(<App />);
    
    // Check for theme toggle button
    const themeToggle = screen.getByRole('button', { 
      name: /switch to (dark|light) theme mode/i 
    });
    expect(themeToggle).toBeInTheDocument();
    expect(themeToggle).toHaveAttribute('aria-label');
  });

  test('Navigation should be keyboard accessible', async () => {
    render(<App />);
    
    // Check that all interactive elements are focusable
    const logo = screen.getByRole('button', { name: /Uveddi/i });
    const themeToggle = screen.getByRole('button', { 
      name: /switch to (dark|light) theme mode/i 
    });
    
    expect(logo).toHaveAttribute('tabIndex', '0');
    expect(themeToggle).not.toHaveAttribute('tabIndex', '-1');
  });

  test('App should follow color contrast guidelines', async () => {
    const { container } = render(<App />);
    
    // Run specific color contrast checks
    const results = await axe(container, {
      rules: {
        'color-contrast': { enabled: true }
      }
    });
    
    expect(results.violations.filter(v => v.id === 'color-contrast')).toHaveLength(0);
  });

  test('App should have no duplicate IDs', async () => {
    const { container } = render(<App />);
    
    const results = await axe(container, {
      rules: {
        'duplicate-id': { enabled: true }
      }
    });
    
    expect(results.violations.filter(v => v.id === 'duplicate-id')).toHaveLength(0);
  });

  test('All images should have alt text', async () => {
    const { container } = render(<App />);
    
    const results = await axe(container, {
      rules: {
        'image-alt': { enabled: true }
      }
    });
    
    expect(results.violations.filter(v => v.id === 'image-alt')).toHaveLength(0);
  });

  test('Form elements should be properly labeled', async () => {
    const { container } = render(<App />);
    
    const results = await axe(container, {
      rules: {
        'label': { enabled: true }
      }
    });
    
    expect(results.violations.filter(v => v.id === 'label')).toHaveLength(0);
  });
});