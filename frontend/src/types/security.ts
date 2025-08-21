// Security-specific types for the frontend dashboard
// These correspond to the security analysis types in the backend

export interface SecurityAnalysis {
  summary: SecuritySummary;
  owaspCoverage: Record<string, OwaspCategoryStats>;
  issues: SecurityIssue[];
  taintFlows: TaintFlow[];
  correlations: SecurityCorrelation[];
  compliance?: ComplianceStatus;
}

export interface SecuritySummary {
  totalIssues: number;
  criticalCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  confidenceDistribution: Record<string, number>;
  mostCommonIssues: IssueTypeStats[];
  securityScore: number;
}

export interface OwaspCategoryStats {
  issuesFound: number;
  coveragePercentage: number;
  avgConfidence: number;
  severityDistribution: Record<string, number>;
}

export interface IssueTypeStats {
  issueType: string;
  count: number;
  avgSeverity: string;
  avgConfidence: number;
}

export interface SecurityIssue {
  id: string;
  issueType: string;
  severity: string;
  confidenceScore: number;
  location: SecurityLocation;
  description: string;
  remediation: string;
  owaspCategory?: string;
  cweId?: string;
  cvssScore?: number;
  references: string[];
  relatedTaintFlows: string[];
  attackVector?: string;
}

export interface SecurityLocation {
  file: string;
  startLine: number;
  endLine: number;
  startColumn?: number;
  endColumn?: number;
  codeSnippet?: string;
}

export interface TaintFlow {
  id: string;
  source: FlowNode;
  sink: FlowNode;
  confidence: number;
  sanitizers: FlowNode[];
  path: FlowNode[];
  vulnerabilityType: string;
}

export interface FlowNode {
  name: string;
  location: string;
  nodeType: string;
  lineNumber: number;
  properties: Record<string, string>;
}

export interface SecurityCorrelation {
  securityIssueId: string;
  architecturalIssueId: string;
  correlationStrength: number;
  correlationType: string;
  explanation: string;
}

export interface ComplianceStatus {
  owaspScore: number;
  cweScore: number;
  standards: Record<string, StandardCompliance>;
}

export interface StandardCompliance {
  name: string;
  score: number;
  requiredControls: number;
  passedControls: number;
  failedControls: number;
}

export interface SecurityFindingMetadata {
  owaspCategory?: string;
  cweId?: string;
  cvssScore?: number;
  attackComplexity?: string;
  attackVector?: string;
  privilegesRequired?: string;
  userInteraction?: string;
  scope?: string;
  availabilityImpact?: string;
  confidentialityImpact?: string;
  integrityImpact?: string;
}

// API response types for security endpoints
export interface SecurityIssuesResponse {
  issues: SecurityIssue[];
  total: number;
  filters?: {
    severity?: string;
    category?: string;
    confidence?: number;
  };
}

export interface SecuritySummaryResponse {
  summary: SecuritySummary;
  timestamp: string;
  analysisId: string;
}

export interface OwaspCoverageResponse {
  coverage: Record<string, OwaspCategoryStats>;
  timestamp: string;
  analysisId: string;
}

export interface TaintFlowsResponse {
  flows: TaintFlow[];
  total: number;
  timestamp: string;
  analysisId: string;
}

export interface SarifExportResponse {
  sarif: string; // SARIF JSON as string
  filename: string;
  timestamp: string;
}

// Filter and search types
export interface SecurityFilters {
  severity?: string[];
  category?: string[];
  confidence?: number;
  search?: string;
  owaspCategory?: string[];
  cweId?: string[];
}

export interface SecuritySearchQuery {
  query: string;
  filters?: SecurityFilters;
  sortBy?: 'severity' | 'confidence' | 'file' | 'type';
  sortOrder?: 'asc' | 'desc';
  limit?: number;
  offset?: number;
}

export interface SecuritySearchResult {
  issues: SecurityIssue[];
  total: number;
  facets: {
    severities: Record<string, number>;
    categories: Record<string, number>;
    fileTypes: Record<string, number>;
  };
}

// Dashboard configuration types
export interface SecurityDashboardConfig {
  showOwaspCoverage: boolean;
  showTaintFlows: boolean;
  maxIssuesDisplay: number;
  autoRefresh: boolean;
  refreshInterval: number;
  defaultFilters: SecurityFilters;
}

export interface SecurityWidgetProps {
  securityAnalysis: SecurityAnalysis;
  config?: Partial<SecurityDashboardConfig>;
  onIssueSelect?: (issueId: string) => void;
  onFilterChange?: (filters: SecurityFilters) => void;
  onExportSarif?: () => Promise<void>;
  className?: string;
}

// Security metrics calculation helpers
export type SecuritySeverity = 'Critical' | 'High' | 'Medium' | 'Low';
export type ConfidenceLevel = 'High' | 'Medium' | 'Low';

export interface SecurityMetrics {
  totalVulnerabilities: number;
  severityDistribution: Record<SecuritySeverity, number>;
  confidenceDistribution: Record<ConfidenceLevel, number>;
  owaspCoverageScore: number;
  riskScore: number;
  remediationEffort: number;
}

// Event types for security dashboard interactions
export type SecurityEventType = 
  | 'issue_selected'
  | 'filter_changed'
  | 'export_requested'
  | 'sarif_downloaded'
  | 'remediation_viewed'
  | 'flow_analyzed';

export interface SecurityEvent {
  type: SecurityEventType;
  payload: any;
  timestamp: Date;
  userId?: string;
}

// Export configuration for security reports
export interface SecurityExportConfig {
  format: 'sarif' | 'pdf' | 'csv' | 'json';
  includeRemediation: boolean;
  includeTaintFlows: boolean;
  filterBySeverity?: SecuritySeverity[];
  customTemplate?: string;
}

export default SecurityAnalysis;