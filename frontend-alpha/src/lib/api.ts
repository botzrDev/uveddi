import axios, { AxiosError, type AxiosInstance } from 'axios';
import type { 
  User, 
  Organization, 
  OrganizationCreate,
  Project, 
  ProjectCreate,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  APIError
} from '../types/api';

// API Configuration
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000';

// Create axios instance
const api: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Token management
const TOKEN_KEY = 'uveddi_token';

export const tokenManager = {
  getToken: (): string | null => {
    return localStorage.getItem(TOKEN_KEY);
  },
  
  setToken: (token: string): void => {
    localStorage.setItem(TOKEN_KEY, token);
  },
  
  removeToken: (): void => {
    localStorage.removeItem(TOKEN_KEY);
  },
  
  isAuthenticated: (): boolean => {
    return !!localStorage.getItem(TOKEN_KEY);
  }
};

// Request interceptor to add auth token
api.interceptors.request.use(
  (config) => {
    const token = tokenManager.getToken();
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor to handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    if (error.response?.status === 401) {
      tokenManager.removeToken();
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// API Error handler
export const handleAPIError = (error: unknown): APIError => {
  if (axios.isAxiosError(error)) {
    if (error.response?.data?.detail) {
      return {
        detail: error.response.data.detail,
        status_code: error.response.status || 500
      };
    }
    return {
      detail: error.message || 'Network error occurred',
      status_code: error.response?.status || 500
    };
  }
  
  return {
    detail: 'An unexpected error occurred',
    status_code: 500
  };
};

// API Methods
export const authAPI = {
  // Note: These endpoints don't exist yet in the backend, but are planned
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    try {
      const response = await api.post('/auth/login', credentials);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  },

  register: async (userData: RegisterRequest): Promise<LoginResponse> => {
    try {
      const response = await api.post('/auth/register', userData);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  },

  logout: async (): Promise<void> => {
    try {
      await api.post('/auth/logout');
    } catch (error) {
      // Continue with logout even if API call fails
      // Logout API call failed, but we'll logout locally anyway
    } finally {
      tokenManager.removeToken();
    }
  }
};

export const userAPI = {
  getCurrentUser: async (): Promise<User> => {
    try {
      const response = await api.get('/users/me');
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  }
};

export const organizationAPI = {
  createOrganization: async (orgData: OrganizationCreate): Promise<Organization> => {
    try {
      const response = await api.post('/organizations', orgData);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  },

  getOrganization: async (organizationId: number): Promise<Organization> => {
    try {
      const response = await api.get(`/organizations/${organizationId}`);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  },

  getOrganizationUsers: async (organizationId: number): Promise<User[]> => {
    try {
      const response = await api.get(`/organizations/${organizationId}/users`);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  }
};

export const projectAPI = {
  createProject: async (projectData: ProjectCreate): Promise<Project> => {
    try {
      const response = await api.post('/projects', projectData);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  },

  getProject: async (projectId: number): Promise<Project> => {
    try {
      const response = await api.get(`/projects/${projectId}`);
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  },

  getOrganizationProjects: async (
    organizationId: number, 
    limit: number = 100, 
    offset: number = 0
  ): Promise<Project[]> => {
    try {
      const response = await api.get(`/organizations/${organizationId}/projects`, {
        params: { limit, offset }
      });
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  }
};

// Health check
export const healthAPI = {
  check: async (): Promise<{ status: string; timestamp: string }> => {
    try {
      const response = await api.get('/health');
      return response.data;
    } catch (error) {
      throw handleAPIError(error);
    }
  }
};

export default api;