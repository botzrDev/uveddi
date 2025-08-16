// API service for communicating with the Uveddi backend
import type { 
  InteractiveReport, 
  DependencyGraph,
  ApiResponse,
  ApiError
} from '@/types/api';

class ApiService {
  private baseUrl: string;

  constructor(baseUrl = '/api/v1') {
    this.baseUrl = baseUrl;
  }

  private async fetchWithErrorHandling<T>(url: string, options?: RequestInit): Promise<T> {
    try {
      const response = await fetch(url, {
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
        ...options,
      });

      if (!response.ok) {
        const errorData: ApiError = await response.json().catch(() => ({
          error: 'HTTP Error',
          message: `Request failed with status ${response.status}`,
          timestamp: new Date().toISOString(),
        }));
        throw new Error(`${errorData.error}: ${errorData.message}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      if (error instanceof Error) {
        throw error;
      }
      throw new Error('Network error occurred');
    }
  }

  /**
   * Get a specific report by ID
   */
  async getReport(id: string): Promise<InteractiveReport> {
    const response = await this.fetchWithErrorHandling<ApiResponse<InteractiveReport>>(
      `${this.baseUrl}/reports/${id}`
    );
    return response.data;
  }

  /**
   * Get dependency graph for a specific report
   */
  async getDependencyGraph(reportId: string): Promise<DependencyGraph> {
    const response = await this.fetchWithErrorHandling<ApiResponse<DependencyGraph>>(
      `${this.baseUrl}/reports/${reportId}/graphs/dependency`
    );
    return response.data;
  }

  /**
   * List available reports with pagination
   */
  async listReports(params: {
    page?: number;
    limit?: number;
    sort?: string;
    order?: 'asc' | 'desc';
  } = {}): Promise<{
    reports: Array<{
      id: string;
      name: string;
      path: string;
      generatedAt: string;
      issuesTotal: number;
      status: string;
      filesAnalyzed: number;
    }>;
    pagination: {
      page: number;
      limit: number;
      total: number;
      pages: number;
    };
  }> {
    const searchParams = new URLSearchParams();
    if (params.page) searchParams.set('page', params.page.toString());
    if (params.limit) searchParams.set('limit', params.limit.toString());
    if (params.sort) searchParams.set('sort', params.sort);
    if (params.order) searchParams.set('order', params.order);

    const response = await this.fetchWithErrorHandling<ApiResponse<any>>(
      `${this.baseUrl}/reports?${searchParams.toString()}`
    );
    return response.data;
  }

  /**
   * Get the demo report for development and testing
   */
  async getDemoReport(): Promise<InteractiveReport> {
    const response = await this.fetchWithErrorHandling<ApiResponse<InteractiveReport>>(
      `${this.baseUrl}/reports/demo`
    );
    return response.data;
  }

  /**
   * Check API health status
   */
  async getHealthStatus(): Promise<{
    status: string;
    service: string;
    timestamp: string;
    version: string;
    apiVersion: string;
  }> {
    return this.fetchWithErrorHandling('/health');
  }

  /**
   * Get API metrics
   */
  async getMetrics(): Promise<{
    reportsServed: number;
    cacheHitRate: number;
    averageResponseTimeMs: number;
    activeConnections: number;
  }> {
    return this.fetchWithErrorHandling('/metrics');
  }
}

// Create a singleton instance
export const apiService = new ApiService();

// Export the class for testing
export { ApiService };