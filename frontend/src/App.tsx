import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate, Link } from 'react-router-dom';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from './lib/queryClient';
import { useIsAuthenticated } from './store/auth';

// Pages
import LoginPage from './pages/LoginPage';
import RegisterPage from './pages/RegisterPage';
import DashboardPage from './pages/DashboardPage';
import TestPage from './pages/TestPage';
import SimpleTest from './SimpleTest';

// Landing page component (simplified version of the original)
const LandingPage: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="sticky top-0 bg-gray-900 bg-opacity-90 backdrop-blur-md z-50">
        <nav className="container mx-auto px-6 py-4 flex justify-between items-center">
          <Link to="/" className="text-2xl font-bold">uveddi</Link>
          <div className="flex items-center space-x-4">
            <Link to="/login" className="text-sm hover:text-green-400 transition-colors">
              Sign In
            </Link>
            <Link to="/register" className="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded transition-colors">
              Get Started Free
            </Link>
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
            <Link to="/register" className="bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded transition-colors">
              Get Started
            </Link>
            <Link to="/test" className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-6 rounded transition-colors">
              View Demo
            </Link>
          </div>
          
          {/* Quick test links */}
          <div className="mt-8 text-sm text-gray-400">
            <p>Quick navigation:</p>
            <div className="flex justify-center space-x-4 mt-2">
              <Link to="/simple" className="text-yellow-400 hover:text-yellow-300 underline">
                Simple Test
              </Link>
              <Link to="/test" className="text-blue-400 hover:text-blue-300 underline">
                CSS Test Page
              </Link>
              <Link to="/login" className="text-green-400 hover:text-green-300 underline">
                Login Page
              </Link>
              <Link to="/register" className="text-purple-400 hover:text-purple-300 underline">
                Register Page
              </Link>
            </div>
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
            <Route 
              path="/register" 
              element={
                <PublicRoute>
                  <RegisterPage />
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
            
            {/* Test routes */}
            <Route path="/test" element={<TestPage />} />
            <Route path="/simple" element={<SimpleTest />} />
            
            {/* Redirect unknown routes */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </div>
      </Router>
    </QueryClientProvider>
  );
}

export default App;