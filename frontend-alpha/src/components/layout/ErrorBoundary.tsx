import React from 'react';
import { errorMonitoring } from '../../utils/errorMonitoring';

interface ErrorBoundaryProps {
  children: React.ReactNode;
  fallback?: React.ComponentType<{ error: Error; resetError: () => void }>;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

const DefaultErrorFallback: React.FC<{ error: Error; resetError: () => void }> = ({ 
  error, 
  resetError 
}) => (
  <div className="min-h-screen bg-secondary-950 flex items-center justify-center">
    <div className="text-center text-white p-8 max-w-md">
      <div className="mb-6">
        <div className="w-16 h-16 bg-red-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg className="w-8 h-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <h1 className="text-2xl font-bold mb-2">Something went wrong</h1>
        <p className="text-secondary-300 mb-4">
          We're sorry, but something unexpected happened. Please try again.
        </p>
        {process.env.NODE_ENV === 'development' && (
          <details className="text-left bg-secondary-800 p-4 rounded-lg mb-4">
            <summary className="cursor-pointer text-red-400 mb-2">Error Details</summary>
            <pre className="text-xs text-secondary-300 overflow-auto">
              {error.message}
              {error.stack}
            </pre>
          </details>
        )}
      </div>
      <div className="space-y-3">
        <button
          onClick={resetError}
          className="w-full px-4 py-2 bg-primary-500 hover:bg-primary-600 rounded-lg transition-colors"
        >
          Try Again
        </button>
        <button
          onClick={() => window.location.href = '/'}
          className="w-full px-4 py-2 bg-secondary-700 hover:bg-secondary-600 rounded-lg transition-colors"
        >
          Go Home
        </button>
      </div>
    </div>
  </div>
);

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // Report error to monitoring service
    errorMonitoring.reportError(error, errorInfo, 'high').catch(reportError => {
      console.error('Failed to report error to monitoring service:', reportError);
    });
  }

  resetError = () => {
    this.setState({ hasError: false, error: undefined });
  };

  render() {
    if (this.state.hasError && this.state.error) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback;
      return <FallbackComponent error={this.state.error} resetError={this.resetError} />;
    }

    return this.props.children;
  }
}

export default ErrorBoundary;