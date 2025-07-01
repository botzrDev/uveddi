import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from './lib/queryClient';
import { useIsAuthenticated } from './store/auth';

// Pages
import LoginPage from './pages/LoginPage';
import DashboardPage from './pages/DashboardPage';

// Landing page component (simplified version of the original)
const LandingPage: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="sticky top-0 bg-gray-900 bg-opacity-90 backdrop-blur-md z-50">
        <nav className="container mx-auto px-6 py-4 flex justify-between items-center">
          <div className="text-2xl font-bold">uveddi</div>
          <div className="flex items-center space-x-4">
            <a href="/login" className="text-sm hover:text-green-400">Sign In</a>
            <a href="/login" className="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded">
              Get Started Free
            </a>
          </div>
        </nav>
      </header>

      <main>
        <section className="hero text-center py-20 px-6">
          <h1 className="text-5xl font-bold mb-4">
            uveddi: High-Level Code Analysis, Instantly.
          </h1>
          <p className="text-xl text-gray-400 mb-8">
            Transform complex codebases into actionable intelligence, securing and optimizing your projects from the command line.
          </p>
          <div className="flex justify-center space-x-4">
            <a href="/login" className="bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded">
              Get Started
            </a>
            <a href="#demo" className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-6 rounded">
              Request a Demo
            </a>
          </div>
        </section>
      </main>
    </div>
  );
};

// Protected Route component
interface ProtectedRouteProps {
  children: React.ReactNode;
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children }) => {
  const isAuthenticated = useIsAuthenticated();
  
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  
  return <>{children}</>;
};

// Public Route component (redirects to dashboard if authenticated)
interface PublicRouteProps {
  children: React.ReactNode;
}

const PublicRoute: React.FC<PublicRouteProps> = ({ children }) => {
  const isAuthenticated = useIsAuthenticated();
  
  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />;
  }
  
  return <>{children}</>;
};

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Router>
        <div className="App">
          <Routes>
            {/* Public routes */}
            <Route 
              path="/" 
              element={
                <PublicRoute>
                  <LandingPage />
                </PublicRoute>
              } 
            />
            <Route 
              path="/login" 
              element={
                <PublicRoute>
                  <LoginPage />
                </PublicRoute>
              } 
            />
            
            {/* Protected routes */}
            <Route
              path="/dashboard"
              element={
                <ProtectedRoute>
                  <DashboardPage />
                </ProtectedRoute>
              }
            />
            
            {/* Redirect unknown routes */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </div>
      </Router>
    </QueryClientProvider>
  );
}

export default App;