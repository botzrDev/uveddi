// Error monitoring utilities for production error tracking
export interface ErrorContext {
  userId?: string;
  userAgent?: string;
  url?: string;
  timestamp: string;
  buildVersion?: string;
  environment: string;
}

export interface ErrorReport {
  error: Error;
  errorInfo?: any;
  context: ErrorContext;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

class ErrorMonitoringService {
  private isEnabled: boolean;
  private endpoint: string | null;

  constructor() {
    this.isEnabled = process.env.NODE_ENV === 'production';
    this.endpoint = process.env.VITE_ERROR_MONITORING_ENDPOINT || null;
  }

  private getErrorContext(): ErrorContext {
    return {
      userAgent: navigator?.userAgent,
      url: window?.location?.href,
      timestamp: new Date().toISOString(),
      buildVersion: process.env.VITE_BUILD_VERSION || 'unknown',
      environment: process.env.NODE_ENV || 'development',
    };
  }

  private async sendToMonitoringService(report: ErrorReport): Promise<void> {
    if (!this.isEnabled || !this.endpoint) {
      return;
    }

    try {
      await fetch(this.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message: report.error.message,
          stack: report.error.stack,
          name: report.error.name,
          context: report.context,
          severity: report.severity,
          errorInfo: report.errorInfo,
        }),
      });
    } catch (fetchError) {
      // Fallback to console if monitoring service is unreachable
      console.error('Failed to send error to monitoring service:', fetchError);
      console.error('Original error:', report.error);
    }
  }

  async reportError(
    error: Error, 
    errorInfo?: any, 
    severity: ErrorReport['severity'] = 'medium'
  ): Promise<void> {
    const report: ErrorReport = {
      error,
      errorInfo,
      context: this.getErrorContext(),
      severity,
    };

    // Always log to console in development
    if (process.env.NODE_ENV === 'development') {
      console.group('🚨 Error Report');
      console.error('Error:', error);
      console.error('Error Info:', errorInfo);
      console.log('Context:', report.context);
      console.log('Severity:', severity);
      console.groupEnd();
    }

    // Send to monitoring service in production
    await this.sendToMonitoringService(report);
  }

  async reportUnhandledError(event: ErrorEvent): Promise<void> {
    const error = new Error(event.message);
    error.stack = `at ${event.filename}:${event.lineno}:${event.colno}`;
    
    await this.reportError(error, {
      type: 'unhandled',
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
    }, 'high');
  }

  async reportUnhandledRejection(event: PromiseRejectionEvent): Promise<void> {
    const error = event.reason instanceof Error 
      ? event.reason 
      : new Error(String(event.reason));

    await this.reportError(error, {
      type: 'unhandled_rejection',
      reason: event.reason,
    }, 'high');
  }

  setupGlobalHandlers(): void {
    if (typeof window === 'undefined') return;

    // Handle unhandled JavaScript errors
    window.addEventListener('error', (event) => {
      this.reportUnhandledError(event);
    });

    // Handle unhandled promise rejections
    window.addEventListener('unhandledrejection', (event) => {
      this.reportUnhandledRejection(event);
    });
  }
}

// Create singleton instance
export const errorMonitoring = new ErrorMonitoringService();

// Initialize global error handlers
errorMonitoring.setupGlobalHandlers();