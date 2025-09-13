import React, { Component, ErrorInfo, ReactNode } from 'react';
import {
  Alert,
  AlertTitle,
  Box,
  Button,
  Container,
  Paper,
  Typography,
  useTheme
} from '@mui/material';
import {
  ErrorOutline,
  RefreshOutlined,
  HomeOutlined,
  BugReportOutlined
} from '@mui/icons-material';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorId: string | null;
}

class ApiErrorBoundary extends Component<Props, State> {
  private retryCount = 0;
  private maxRetries = 3;

  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
    errorId: null,
  };

  public static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
      errorId: `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('API Error Boundary caught error:', error, errorInfo);

    this.setState({
      error,
      errorInfo,
    });

    // Call optional error callback
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    // Report to monitoring system if available
    if (window.analytics) {
      window.analytics.track('Dashboard Error', {
        error: error.message,
        stack: error.stack,
        componentStack: errorInfo.componentStack,
        errorId: this.state.errorId,
      });
    }
  }

  private handleRetry = () => {
    if (this.retryCount < this.maxRetries) {
      this.retryCount++;
      this.setState({
        hasError: false,
        error: null,
        errorInfo: null,
        errorId: null,
      });
    } else {
      // Max retries reached, show different message
      this.setState(prevState => ({
        ...prevState,
        error: new Error('Maximum retry attempts exceeded. Please reload the page or contact support.'),
      }));
    }
  };

  private handleReload = () => {
    window.location.reload();
  };

  private handleGoHome = () => {
    window.location.href = '/';
  };

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <Container maxWidth="md" sx={{ py: 4 }}>
          <Paper
            elevation={3}
            sx={{
              p: 4,
              textAlign: 'center',
              background: (theme) =>
                theme.palette.mode === 'dark'
                  ? 'linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%)'
                  : 'linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%)',
            }}
          >
            <Box sx={{ mb: 3 }}>
              <ErrorOutline
                sx={{
                  fontSize: 64,
                  color: 'error.main',
                  mb: 2,
                }}
              />
            </Box>

            <Typography
              variant="h4"
              component="h1"
              gutterBottom
              color="error"
              sx={{ mb: 3 }}
            >
              Dashboard Error
            </Typography>

            <Typography
              variant="body1"
              color="text.secondary"
              sx={{ mb: 4, maxWidth: '600px', mx: 'auto' }}
            >
              We're sorry, but there was an unexpected error in the dashboard.
              This might be due to a network issue, server problem, or temporary data inconsistency.
            </Typography>

            {this.state.error && (
              <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>
                <AlertTitle>Error Details</AlertTitle>
                <Typography variant="body2" component="div">
                  <strong>Error:</strong> {this.state.error.message}
                </Typography>
                {this.state.errorId && (
                  <Typography variant="body2" component="div" sx={{ mt: 1 }}>
                    <strong>Error ID:</strong> {this.state.errorId}
                  </Typography>
                )}
              </Alert>
            )}

            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
              {this.retryCount < this.maxRetries && (
                <Button
                  variant="contained"
                  color="primary"
                  startIcon={<RefreshOutlined />}
                  onClick={this.handleRetry}
                  size="large"
                >
                  Try Again ({this.maxRetries - this.retryCount} attempts left)
                </Button>
              )}

              <Button
                variant="outlined"
                color="primary"
                startIcon={<RefreshOutlined />}
                onClick={this.handleReload}
                size="large"
              >
                Reload Page
              </Button>

              <Button
                variant="outlined"
                color="secondary"
                startIcon={<HomeOutlined />}
                onClick={this.handleGoHome}
                size="large"
              >
                Go Home
              </Button>
            </Box>

            {process.env.NODE_ENV === 'development' && this.state.errorInfo && (
              <Box sx={{ mt: 4, p: 3, bgcolor: 'grey.100', borderRadius: 1 }}>
                <Typography variant="h6" gutterBottom>
                  Error Details (Development Only)
                </Typography>
                <Typography
                  variant="body2"
                  component="pre"
                  sx={{
                    overflow: 'auto',
                    textAlign: 'left',
                    fontFamily: 'monospace',
                    fontSize: '0.75rem',
                    lineHeight: 1.4,
                  }}
                >
                  {this.state.error?.stack}
                  {this.state.errorInfo?.componentStack}
                </Typography>
              </Box>
            )}

            <Typography
              variant="body2"
              color="text.secondary"
              sx={{ mt: 4 }}
            >
              If this problem persists, please contact support with Error ID: {this.state.errorId}
            </Typography>
          </Paper>
        </Container>
      );
    }

    return this.props.children;
  }
}

export default ApiErrorBoundary;