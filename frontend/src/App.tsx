/**
 * Main Uveddi Dashboard Application
 * 
 * This is the root React component for the Uveddi Interactive Reports dashboard.
 * It sets up the application's theme system, routing, state management, and global
 * error handling. The app provides a comprehensive interface for viewing and
 * interacting with code analysis results.
 * 
 * Key features:
 * - Material-UI theming with dark/light/system modes
 * - React Query for data fetching and caching
 * - Real-time updates via WebSocket connections
 * - Multi-page routing for different analysis views
 * - Global error boundary for robust error handling
 * 
 * @component App
 * @returns {JSX.Element} The main application component
 * 
 * @example
 * ```typescript
 * // App is typically rendered at the root of the React tree
 * import { createRoot } from 'react-dom/client';
 * import App from './App';
 * 
 * const root = createRoot(document.getElementById('root')!);
 * root.render(<App />);
 * ```
 */

import { getTheme, ThemeMode } from '@/utils/theme';
import { CssBaseline, ThemeProvider } from '@mui/material';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';
import { Navigate, Route, BrowserRouter as Router, Routes } from 'react-router-dom';

// Components
import ErrorBoundary from '@/components/ErrorBoundary';
import Layout from '@/components/Layout';
import RealtimeUpdates from '@/components/RealtimeUpdates';
import DashboardPage from '@/pages/DashboardPage';
import ReportPage from '@/pages/ReportPage';
import ReportsListPage from '@/pages/ReportsListPage';

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes
    },
  },
});

function App() {
  const [themeMode, setThemeMode] = useState<ThemeMode>(() => {
    // Check system preference and localStorage
    const savedTheme = localStorage.getItem('uveddi-theme') as ThemeMode;
    if (savedTheme) return savedTheme;
    
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });

  const theme = useMemo(() => getTheme(themeMode), [themeMode]);

  // Set data-theme attribute on html element for CSS variables
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', themeMode);
  }, [themeMode]);

  const toggleTheme = () => {
    const newMode = themeMode === 'light' ? 'dark' : 'light';
    setThemeMode(newMode);
    localStorage.setItem('uveddi-theme', newMode);
  };

  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <ThemeProvider theme={theme}>
          <CssBaseline />
          <RealtimeUpdates 
            onNewAnalysis={() => {
              // Refresh data by invalidating queries
              queryClient.invalidateQueries({ queryKey: ['reports'] });
              queryClient.invalidateQueries({ queryKey: ['report'] });
            }} 
          />
          <Router
            future={{
              v7_startTransition: true,
              v7_relativeSplatPath: true,
            }}
          >
            <Layout themeMode={themeMode} onToggleTheme={toggleTheme}>
              <Routes>
                {/* Default route - redirect to latest real report */}
                <Route path="/" element={<Navigate to="/dashboard/5" replace />} />
                
                {/* Dashboard routes */}
                <Route path="/dashboard/:reportId" element={<DashboardPage />} />
                
                {/* Report detail routes */}
                <Route path="/reports" element={<ReportsListPage />} />
                <Route path="/reports/:reportId" element={<ReportPage />} />
                
                {/* Catch-all route */}
                <Route path="*" element={<Navigate to="/dashboard/5" replace />} />
              </Routes>
            </Layout>
          </Router>
        </ThemeProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;