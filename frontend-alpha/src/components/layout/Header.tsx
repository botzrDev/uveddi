import { LogOut, Settings, User } from 'lucide-react';
import React from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { useAuth } from '../../store/auth';
import Button from '../ui/Button';

interface HeaderProps {
  variant?: 'default' | 'landing';
  className?: string;
}

const Header: React.FC<HeaderProps> = ({ variant = 'default', className = '' }) => {
  const { user, isAuthenticated, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  // Check if we're on the landing page
  const isOnLandingPage = location.pathname === '/';

  // Custom navigation handler for landing page sections
  const handleSectionNavigation = (sectionId: string) => {
    if (isOnLandingPage) {
      // If we're already on the landing page, just scroll to the section
      const element = document.getElementById(sectionId);
      if (element) {
        element.scrollIntoView({ behavior: 'smooth' });
      }
    } else {
      // If we're on a different page, navigate to home first, then scroll
      navigate('/');
      // Use setTimeout to ensure the page has loaded before scrolling
      setTimeout(() => {
        const element = document.getElementById(sectionId);
        if (element) {
          element.scrollIntoView({ behavior: 'smooth' });
        }
      }, 100);
    }
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
          {variant === 'landing' && isOnLandingPage ? (
            <a href="#top" className="flex items-center mb-0" style={{ gap: '2px' }}>
              <img src="/logo(blue).png" alt="Uveddi Logo" className="w-10 h-10 rounded-lg mr-0" />
              <h1 className="font-mono text-2xl font-bold bg-gradient-to-r from-white to-primary-200 bg-clip-text text-transparent tracking-wider">veddi</h1>
            </a>
          ) : (
            <Link to="/" className="flex items-center mb-0" style={{ gap: '2px' }}>
              <img src={variant === 'landing' ? "/logo(dark).png" : "/logo.png"} alt="Uveddi Logo" className={variant === 'landing' ? "w-10 h-10 rounded-lg mr-0" : "h-10 w-auto mr-0"} />
              <h1 className="font-mono text-2xl font-bold bg-gradient-to-r from-white to-primary-200 bg-clip-text text-transparent tracking-wider">veddi</h1>
            </Link>
          )}

          {/* Navigation */}
          <nav className="hidden md:flex items-center space-x-8">
            {variant === 'landing' && isOnLandingPage ? (
              // Landing page navigation - use anchor links for same-page navigation
              <>
                <a href="#features" className="text-secondary-300 hover:text-primary-400 transition-colors">Features</a>
                <a href="#use-cases" className="text-secondary-300 hover:text-primary-400 transition-colors">Use Cases</a>
                <a href="#integrations" className="text-secondary-300 hover:text-primary-400 transition-colors">Integrations</a>
                <a href="#pricing" className="text-secondary-300 hover:text-primary-400 transition-colors">Pricing</a>
                <Link to="/support" className="text-secondary-300 hover:text-primary-400 transition-colors">Support</Link>
                <a href="#faq" className="text-secondary-300 hover:text-primary-400 transition-colors">FAQ</a>
              </>
            ) : variant === 'landing' ? (
              // Landing variant but not on landing page - use custom navigation
              <>
                <button 
                  onClick={() => handleSectionNavigation('features')}
                  className="text-secondary-300 hover:text-primary-400 transition-colors"
                >
                  Features
                </button>
                <button 
                  onClick={() => handleSectionNavigation('use-cases')}
                  className="text-secondary-300 hover:text-primary-400 transition-colors"
                >
                  Use Cases
                </button>
                <button 
                  onClick={() => handleSectionNavigation('integrations')}
                  className="text-secondary-300 hover:text-primary-400 transition-colors"
                >
                  Integrations
                </button>
                <button 
                  onClick={() => handleSectionNavigation('pricing')}
                  className="text-secondary-300 hover:text-primary-400 transition-colors"
                >
                  Pricing
                </button>
                <Link to="/support" className="text-secondary-300 hover:text-primary-400 transition-colors">Support</Link>
                <button 
                  onClick={() => handleSectionNavigation('faq')}
                  className="text-secondary-300 hover:text-primary-400 transition-colors"
                >
                  FAQ
                </button>
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
                  to="/support" 
                  className="text-gray-700 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400"
                >
                  Support
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
                <Link to="/register">
                  <Button variant="primary" size="sm">
                    Get Started
                  </Button>
                </Link>
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