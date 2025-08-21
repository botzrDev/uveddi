import React, { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ThemeProvider } from '@mui/material/styles';
import { CssBaseline } from '@mui/material';
import { vi } from 'vitest';
import dashboardTheme from '../utils/dashboardTheme';
import { DashboardEventProvider } from './mocks/DashboardEventProvider';

// Create a test query client
const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        staleTime: Infinity,
      },
      mutations: {
        retry: false,
      },
    },
  });

interface AllTheProvidersProps {
  children: React.ReactNode;
}

const AllTheProviders: React.FC<AllTheProvidersProps> = ({ children }) => {
  const queryClient = createTestQueryClient();

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider theme={dashboardTheme.createTheme('light')}>
        <CssBaseline />
        <DashboardEventProvider>
          {children}
        </DashboardEventProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
};

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) => render(ui, { wrapper: AllTheProviders, ...options });

// Re-export everything
export * from '@testing-library/react';
import userEvent from '@testing-library/user-event';
export { userEvent };

// Override render method
export { customRender as render };

// Helper functions for common test scenarios
export const waitForChartRender = () => new Promise(resolve => setTimeout(resolve, 100));

export const mockIntersectionObserver = () => {
  const mockIntersectionObserver = vi.fn(() => ({
    observe: () => null,
    unobserve: () => null,
    disconnect: () => null,
    takeRecords: () => [],
    root: null,
    rootMargin: '0px',
    thresholds: [0]
  }));
  window.IntersectionObserver = mockIntersectionObserver;
};

export const mockResizeObserver = () => {
  const mockResizeObserver = vi.fn(() => ({
    observe: () => null,
    unobserve: () => null,
    disconnect: () => null
  }));
  window.ResizeObserver = mockResizeObserver;
};

export const createMockFile = (name: string, size: number, type: string): File => {
  const file = new File([''], name, { type });
  Object.defineProperty(file, 'size', { value: size });
  return file;
};