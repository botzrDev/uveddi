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
                {/* Default route - redirect to demo dashboard */}
                <Route path="/" element={<Navigate to="/dashboard/demo" replace />} />
                
                {/* Dashboard routes */}
                <Route path="/dashboard/:reportId" element={<DashboardPage />} />
                
                {/* Report detail routes */}
                <Route path="/reports" element={<ReportsListPage />} />
                <Route path="/reports/:reportId" element={<ReportPage />} />
                
                {/* Catch-all route */}
                <Route path="*" element={<Navigate to="/dashboard/demo" replace />} />
              </Routes>
            </Layout>
          </Router>
        </ThemeProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;