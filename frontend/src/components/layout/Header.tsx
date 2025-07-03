import { LogOut, Settings, User } from 'lucide-react';
import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../../store/auth';
import Button from '../ui/Button';

interface HeaderProps {
  variant?: 'default' | 'landing';
  className?: string;
}

const Header: React.FC<HeaderProps> = ({ variant = 'default', className = '' }) => {
  const { user, isAuthenticated, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  // Different styles for different variants
  const headerClasses = variant === 'landing' 
    ? "sticky top-0 bg-secondary-950/90 backdrop-blur-md z-50 border-b border-secondary-800"
    : "sticky top-0 z-50 bg-white dark:bg-secondary-900 border-b border-gray-200 dark:border-secondary-800";
    
  const containerClasses = variant === 'landing'
    ? "container mx-auto px-6 py-4"
    : "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8";
    
  const itemsClasses = variant === 'landing'
    ? "flex justify-between items-center"
    : "flex justify-between items-center h-16";

  return (
    <header className={`${headerClasses} ${className}`}>
      <div className={containerClasses}>
        <div className={itemsClasses}>
          {/* Logo */}
          {variant === 'landing' ? (
            <a href="#top" className="flex items-center">
              <img src="/logo.png" alt="Uveddi Logo" className="w-8 h-8 rounded-lg mr-2" />
              <h1 className="font-display text-headline-md font-headline text-white">uveddi</h1>
            </a>
          ) : (
            <Link to="/" className="flex items-center">
              <img src="/logo.png" alt="Uveddi Logo" className="h-10 w-auto mr-2" />
              <h1 className="font-display text-headline-md font-headline text-white">uveddi</h1>
            </Link>
          )}

          {/* Navigation */}
          <nav className="hidden md:flex items-center space-x-8">
            {variant === 'landing' ? (
              // Landing page navigation
              <>
                <a href="#features" className="text-secondary-300 hover:text-primary-400 transition-colors">Features</a>
                <a href="#use-cases" className="text-secondary-300 hover:text-primary-400 transition-colors">Use Cases</a>
                <a href="#integrations" className="text-secondary-300 hover:text-primary-400 transition-colors">Integrations</a>
                <a href="#pricing" className="text-secondary-300 hover:text-primary-400 transition-colors">Pricing</a>
                <a href="#docs" className="text-secondary-300 hover:text-primary-400 transition-colors">Docs</a>
                <a href="#faq" className="text-secondary-300 hover:text-primary-400 transition-colors">FAQ</a>
              </>
            ) : isAuthenticated ? (
              // Authenticated app navigation
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
              // Public app navigation
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

          {/* Right side actions */}
          <div className="flex items-center space-x-4">
            {variant === 'landing' ? (
              // Landing page actions
              <>
                <Link to="/login" className="text-secondary-300 hover:text-primary-400 transition-colors font-medium">
                  Sign In
                </Link>
                <button className="px-4 py-2 bg-success-500 hover:bg-success-600 text-white rounded-lg font-medium transition-all duration-300 hover:shadow-glow-success">
                  Get Started Free
                </button>
              </>
            ) : isAuthenticated && user ? (
              // Authenticated user menu
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
              // Public app actions
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