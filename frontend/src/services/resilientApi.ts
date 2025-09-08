// API Error Handling and Recovery Service
// Provides resilient API communication with automatic retry, circuit breaker, and graceful degradation

interface RetryConfig {
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
  backoffFactor: number;
}

interface CircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeout: number;
  monitoringWindow: number;
}

enum CircuitState {
  CLOSED = 'CLOSED',     // Normal operation
  OPEN = 'OPEN',         // Circuit breaker active
  HALF_OPEN = 'HALF_OPEN' // Testing recovery
}

class APIError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
    public retryable: boolean = false
  ) {
    super(message);
    this.name = 'APIError';
  }
}

class ResilientAPIService {
  private baseURL: string;
  private retryConfig: RetryConfig;
  private circuitBreakerConfig: CircuitBreakerConfig;
  private circuitState: CircuitState = CircuitState.CLOSED;
  private failureCount: number = 0;
  private lastFailureTime: number = 0;
  private successCount: number = 0;

  constructor(baseURL: string = 'http://localhost:8000') {
    this.baseURL = baseURL;
    this.retryConfig = {
      maxRetries: 3,
      baseDelay: 1000,
      maxDelay: 10000,
      backoffFactor: 2
    };
    this.circuitBreakerConfig = {
      failureThreshold: 5,
      recoveryTimeout: 30000, // 30 seconds
      monitoringWindow: 60000  // 1 minute
    };
  }

  private async delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private calculateRetryDelay(attempt: number): number {
    const delay = this.retryConfig.baseDelay * Math.pow(this.retryConfig.backoffFactor, attempt);
    return Math.min(delay, this.retryConfig.maxDelay);
  }

  private isRetryableError(error: any): boolean {
    // Network errors
    if (!error.status) return true;
    
    // Server errors that might be temporary
    if (error.status >= 500) return true;
    
    // Rate limiting
    if (error.status === 429) return true;
    
    // Timeout errors
    if (error.status === 408) return true;
    
    return false;
  }

  private updateCircuitBreaker(success: boolean): void {
    const now = Date.now();
    
    if (success) {
      this.successCount++;
      
      if (this.circuitState === CircuitState.HALF_OPEN && this.successCount >= 2) {
        console.log('Circuit breaker: Recovered, closing circuit');
        this.circuitState = CircuitState.CLOSED;
        this.failureCount = 0;
        this.successCount = 0;
      }
    } else {
      this.failureCount++;
      this.lastFailureTime = now;
      
      if (this.circuitState === CircuitState.CLOSED && 
          this.failureCount >= this.circuitBreakerConfig.failureThreshold) {
        console.log('Circuit breaker: Opening circuit due to failures');
        this.circuitState = CircuitState.OPEN;
      } else if (this.circuitState === CircuitState.HALF_OPEN) {
        console.log('Circuit breaker: Half-open test failed, reopening circuit');
        this.circuitState = CircuitState.OPEN;
        this.successCount = 0;
      }
    }
  }

  private shouldAllowRequest(): boolean {
    const now = Date.now();
    
    switch (this.circuitState) {
      case CircuitState.CLOSED:
        return true;
        
      case CircuitState.OPEN:
        if (now - this.lastFailureTime >= this.circuitBreakerConfig.recoveryTimeout) {
          console.log('Circuit breaker: Attempting recovery, half-opening circuit');
          this.circuitState = CircuitState.HALF_OPEN;
          this.successCount = 0;
          return true;
        }
        return false;
        
      case CircuitState.HALF_OPEN:
        return true;
        
      default:
        return true;
    }
  }

  private async makeRequest<T>(
    url: string, 
    options: RequestInit = {}
  ): Promise<T> {
    if (!this.shouldAllowRequest()) {
      throw new APIError(
        'Service temporarily unavailable (circuit breaker open)',
        503,
        'CIRCUIT_BREAKER_OPEN',
        false
      );
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000); // 30s timeout

    try {
      const response = await fetch(`${this.baseURL}${url}`, {
        ...options,
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorText = await response.text().catch(() => 'Unknown error');
        throw new APIError(
          `API Error: ${response.status} ${response.statusText} - ${errorText}`,
          response.status,
          'HTTP_ERROR',
          this.isRetryableError({ status: response.status })
        );
      }

      const data = await response.json();
      this.updateCircuitBreaker(true);
      return data;

    } catch (error: any) {
      clearTimeout(timeoutId);
      
      if (error.name === 'AbortError') {
        const timeoutError = new APIError(
          'Request timeout',
          408,
          'TIMEOUT',
          true
        );
        this.updateCircuitBreaker(false);
        throw timeoutError;
      }

      if (error instanceof APIError) {
        this.updateCircuitBreaker(false);
        throw error;
      }

      // Network or other errors
      const networkError = new APIError(
        `Network error: ${error.message}`,
        undefined,
        'NETWORK_ERROR',
        true
      );
      this.updateCircuitBreaker(false);
      throw networkError;
    }
  }

  private async retryOperation<T>(
    operation: () => Promise<T>
  ): Promise<T> {
    let lastError: APIError;

    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error: any) {
        lastError = error instanceof APIError ? error : new APIError(error.message);
        
        if (attempt === this.retryConfig.maxRetries || !lastError.retryable) {
          throw lastError;
        }

        const delayMs = this.calculateRetryDelay(attempt);
        console.log(`API request failed (attempt ${attempt + 1}), retrying in ${delayMs}ms:`, lastError.message);
        await this.delay(delayMs);
      }
    }

    throw lastError!;
  }

  // Public API methods with resilience
  async getLatestReport(): Promise<any> {
    return this.retryOperation(() => 
      this.makeRequest('/api/v1/reports/latest')
    );
  }

  async getReports(): Promise<any[]> {
    return this.retryOperation(() => 
      this.makeRequest('/api/v1/reports')
    );
  }

  async exportLatestReport(): Promise<Blob> {
    return this.retryOperation(async () => {
      const response = await fetch(`${this.baseURL}/api/v1/reports/latest/export`, {
        headers: {
          'Accept': 'application/json'
        }
      });

      if (!response.ok) {
        throw new APIError(
          `Export failed: ${response.status} ${response.statusText}`,
          response.status,
          'EXPORT_ERROR',
          this.isRetryableError({ status: response.status })
        );
      }

      return response.blob();
    });
  }

  async getSystemHealth(): Promise<any> {
    return this.retryOperation(() => 
      this.makeRequest('/health')
    );
  }

  // Get circuit breaker status for monitoring
  getCircuitBreakerStatus() {
    return {
      state: this.circuitState,
      failureCount: this.failureCount,
      successCount: this.successCount,
      lastFailureTime: this.lastFailureTime
    };
  }

  // Reset circuit breaker manually
  resetCircuitBreaker() {
    this.circuitState = CircuitState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
    this.lastFailureTime = 0;
    console.log('Circuit breaker manually reset');
  }
}

// Singleton instance
export const apiService = new ResilientAPIService();

// React hook for API service with error handling
import { useState, useEffect, useCallback } from 'react';

interface UseAPIState<T> {
  data: T | null;
  loading: boolean;
  error: APIError | null;
  retry: () => void;
  circuitBreakerStatus: any;
}

export function useResilientAPI<T>(
  apiCall: () => Promise<T>,
  dependencies: any[] = []
): UseAPIState<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<APIError | null>(null);

  const executeRequest = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await apiCall();
      setData(result);
    } catch (err) {
      const apiError = err instanceof APIError ? err : new APIError(String(err));
      setError(apiError);
      console.error('API request failed:', apiError);
    } finally {
      setLoading(false);
    }
  }, dependencies);

  const retry = useCallback(() => {
    executeRequest();
  }, [executeRequest]);

  useEffect(() => {
    executeRequest();
  }, dependencies);

  return {
    data,
    loading,
    error,
    retry,
    circuitBreakerStatus: apiService.getCircuitBreakerStatus()
  };
}

export { APIError, ResilientAPIService };
