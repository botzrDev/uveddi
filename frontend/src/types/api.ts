// API types matching the FastAPI backend models

export interface User {
  user_id: number;
  email: string;
  username: string;
  role: 'individual' | 'org_admin' | 'org_member';
  organization_id?: number;
  created_at: string;
  last_login?: string;
  is_verified: boolean; // Indicates if the user's email/account is verified
}

export interface Organization {
  organization_id: number;
  name: string;
  subscription_tier: 'free' | 'paid_api' | 'enterprise';
  created_at: string;
  api_key_usage_limit?: number;
}

export interface OrganizationCreate {
  name: string;
  subscription_tier?: 'free' | 'paid_api' | 'enterprise';
  api_key_usage_limit?: number;
}

export interface Project {
  project_id: number;
  organization_id: number;
  name: string;
  repository_url: string;
  last_analyzed_commit?: string;
  config_data?: Record<string, any>;
  created_at: string;
}

export interface ProjectCreate {
  name: string;
  repository_url: string;
  last_analyzed_commit?: string;
  config_data?: Record<string, any>;
}

export interface AnalysisRun {
  run_id: number;
  project_id: number;
  user_id?: number;
  start_time: string;
  end_time?: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  ai_model_used?: string;
  total_files_scanned?: number;
  total_issues_found?: number;
  exit_code?: number;
  raw_analysis_output?: Record<string, any>;
}

export interface ArchitecturalIssue {
  issue_id: number;
  run_id: number;
  anti_pattern_type_id: number;
  file_path: string;
  line_start?: number;
  line_end?: number;
  severity: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  description: string;
  ai_refactoring_suggestion?: string;
  is_ignored: boolean;
  ignored_by_user_id?: number;
  ignored_at?: string;
}

export interface AntiPatternType {
  anti_pattern_type_id: number;
  name: string;
  description?: string;
  category?: string;
  detection_heuristic_notes?: string;
}

export interface Report {
  report_id: number;
  run_id: number;
  file_path?: string;
  content?: string;
  generated_at: string;
}

// API Error types
export interface APIError {
  detail: string;
  status_code: number;
}

// Authentication types
export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  token_type: string;
  user: User;
}

export interface RegisterRequest {
  email: string;
  username: string;
  password: string;
}

// API Response wrapper
export interface APIResponse<T> {
  data: T;
  message?: string;
}

// Pagination
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  size: number;
  pages: number;
}