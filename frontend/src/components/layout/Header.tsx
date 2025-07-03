import { LogOut, Settings, User } from 'lucide-react';
import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../../store/auth';
import Button from '../ui/Button';

const Header: React.FC = () => {
  const { user, isAuthenticated, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <header className="sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          {/* Logo */}
          <Link to="/" className="flex items-center">
            <img src="/logo.png" alt="Uveddi Logo" className="h-10 w-auto mr-2" />
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">uveddi</h1>
          </Link>

          {/* Navigation */}
          <nav className="hidden md:flex items-center space-x-8">
            {isAuthenticated ? (
              <>
                <Link 
                  to="/dashboard" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Dashboard
                </Link>
                <Link 
                  to="/projects" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Projects
                </Link>
                <Link 
                  to="/organization" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Organization
                </Link>
              </>
            ) : (
              <>
                <Link 
                  to="/features" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Features
                </Link>
                <Link 
                  to="/pricing" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Pricing
                </Link>
                <Link 
                  to="/docs" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Docs
                </Link>
              </>
            )}
          </nav>

          {/* User menu */}
          <div className="flex items-center space-x-4">
            {isAuthenticated && user ? (
              <div className="flex items-center space-x-4">
                <div className="flex items-center space-x-2">
                  <User className="w-5 h-5 text-gray-500" />
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    {user.username}
                  </span>
                </div>
                <Link to="/settings">
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                </Link>
                <Button 
                  variant="ghost" 
                  size="sm"
                  onClick={handleLogout}
                >
                  <LogOut className="w-4 h-4" />
                </Button>
              </div>
            ) : (
              <div className="flex items-center space-x-4">
                <Link to="/login">
                  <Button variant="ghost">
                    Sign In
                  </Button>
                </Link>
                <Link to="/register">
                  <Button variant="primary">
                    Get Started
                  </Button>
                </Link>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
};

export default Header;