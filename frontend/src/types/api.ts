/**
 * Core API interfaces for Uveddi analysis results and interactive reports
 * 
 * This module defines the complete contract between the Rust analysis engine
 * and the TypeScript frontend for data exchange. These type definitions mirror
 * the Rust Report v1 data structures to ensure type safety and consistency
 * across the application boundary.
 * 
 * The interfaces support both synchronous analysis results and real-time
 * interactive report updates through WebSocket connections.
 * 
 * @example Basic Usage
 * ```typescript
 * // Fetch analysis summary
 * const report: InteractiveReport = await fetchAnalysisReport();
 * console.log(`Found ${report.summary.issuesTotal} issues`);
 * 
 * // Access findings by severity
 * const criticalIssues = report.findings.filter(f => f.severity === 'Critical');
 * ```
 * 
 * @example Real-time Updates
 * ```typescript
 * // Subscribe to analysis progress
 * websocket.onmessage = (event) => {
 *   const update: AnalysisProgress = JSON.parse(event.data);
 *   updateProgressBar(update.completionPercent);
 * };
 * ```
 */

/**
 * Main interactive report structure containing all analysis results
 * 
 * This is the root interface for all analysis data returned by the Uveddi engine.
 * It includes project metadata, analysis summary, detailed findings, dependency graphs,
 * and optional AI insights and security analysis.
 * 
 * @interface InteractiveReport
 * @property {string} schemaVersion - Version of the report schema for compatibility
 * @property {ProjectMetadata} project - Project identification and metadata
 * @property {AnalysisSummary} summary - High-level analysis statistics
 * @property {Finding[]} findings - Detailed list of all detected issues
 * @property {DependencyGraph} dependencyGraph - Code dependency relationships
 * @property {DiagramDefinition[]} diagrams - Visual diagrams for the report
 * @property {AiInsights} aiInsights - AI-powered insights and recommendations (optional)
 * @property {SecurityAnalysis} securityAnalysis - Security-specific analysis results (optional)
 * @property {ReportMetadata} metadata - Report generation metadata
 */
export interface InteractiveReport {
  schemaVersion: string;
  project: ProjectMetadata;
  summary: AnalysisSummary;
  findings: Finding[];
  dependencyGraph: DependencyGraph;
  diagrams: DiagramDefinition[];
  aiInsights?: AiInsights;
  securityAnalysis?: SecurityAnalysis;
  metadata: ReportMetadata;
}

/**
 * Project identification and source control metadata
 * 
 * Contains information about the analyzed project including source control details
 * and programming languages detected.
 * 
 * @interface ProjectMetadata
 * @property {string} id - Unique project identifier
 * @property {string} name - Project name from package.json/Cargo.toml
 * @property {string} commit - Git commit hash (optional)
 * @property {string} branch - Git branch name (optional)
 * @property {string} repoUrl - Repository URL (optional)
 * @property {string} path - Local filesystem path to project root
 * @property {string[]} languages - Programming languages detected in project
 */
export interface ProjectMetadata {
  id: string;
  name: string;
  commit?: string;
  branch?: string;
  repoUrl?: string;
  path: string;
  languages: string[];
}

/**
 * High-level analysis statistics and performance metrics
 * 
 * Provides aggregate information about the analysis results including
 * issue counts, performance metrics, and coverage statistics.
 * 
 * @interface AnalysisSummary
 * @property {number} coverage - Analysis coverage percentage (0-100)
 * @property {number} issuesTotal - Total number of issues found
 * @property {Record<string, number>} issuesBySeverity - Issue counts grouped by severity level
 * @property {Record<string, number>} issuesByCategory - Issue counts grouped by category
 * @property {number} filesAnalyzed - Number of files processed
 * @property {number} componentsAnalyzed - Number of components/modules analyzed
 * @property {number} analysisDurationMs - Time taken for analysis in milliseconds
 * @property {string} timeGenerated - ISO timestamp when analysis was completed
 */
export interface AnalysisSummary {
  coverage: number;
  issuesTotal: number;
  issuesBySeverity: Record<string, number>;
  issuesByCategory: Record<string, number>;
  filesAnalyzed: number;
  componentsAnalyzed: number;
  analysisDurationMs: number;
  timeGenerated: string;
}

/**
 * Individual code issue or anti-pattern detected during analysis
 * 
 * Represents a specific problem found in the codebase with location information,
 * severity classification, and optional AI-generated explanations.
 * 
 * @interface Finding
 * @property {string} id - Unique finding identifier
 * @property {string} type - Type of issue (e.g., 'GodObject', 'CircularDependency')
 * @property {'critical' | 'high' | 'medium' | 'low'} severity - Issue severity level
 * @property {string} title - Human-readable issue title
 * @property {string} message - Detailed issue description
 * @property {string} file - File path where issue was found
 * @property {number} startLine - Starting line number (optional)
 * @property {number} endLine - Ending line number (optional)
 * @property {number} column - Column position (optional)
 * @property {string} codeSnippet - Relevant code excerpt (optional)
 * @property {string[]} tags - Classification tags
 * @property {string} detector - Name of detector that found this issue
 * @property {number} confidence - Confidence score (0-100)
 * @property {string} aiExplanation - AI-generated explanation (optional)
 * @property {string} recommendation - Suggested fix (optional)
 * @property {string[]} relatedFindings - IDs of related findings
 */
export interface Finding {
  id: string;
  type: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  message: string;
  file: string;
  startLine?: number;
  endLine?: number;
  column?: number;
  codeSnippet?: string;
  tags: string[];
  detector: string;
  confidence: number;
  aiExplanation?: string;
  recommendation?: string;
  relatedFindings: string[];
}

/**
 * Project dependency graph representation
 * 
 * Contains the complete dependency structure of the analyzed project
 * including nodes (files/modules), edges (dependencies), and graph metadata.
 * 
 * @interface DependencyGraph
 * @property {GraphNode[]} nodes - All nodes in the dependency graph
 * @property {GraphEdge[]} edges - All dependency relationships
 * @property {GraphMetadata} metadata - Graph statistics and properties
 */
export interface DependencyGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metadata: GraphMetadata;
}

export interface GraphNode {
  id: string;
  label: string;
  path: string;
  type: string;
  metrics?: NodeMetrics;
  group?: string;
  properties: Record<string, string>;
}

export interface GraphEdge {
  source: string;
  target: string;
  type: string;
  weight?: number;
  properties: Record<string, string>;
}

export interface NodeMetrics {
  loc?: number;
  complexity?: number;
  dependencies: number;
  dependents: number;
}

export interface GraphMetadata {
  nodeCount: number;
  edgeCount: number;
  hasCycles: boolean;
  maxDepth: number;
  layout?: string;
}

export interface DiagramDefinition {
  id: string;
  kind: string;
  title: string;
  source: string;
  description?: string;
  components: string[];
  metadata: DiagramRenderMetadata;
}

export interface DiagramRenderMetadata {
  width?: number;
  height?: number;
  theme?: string;
  direction?: string;
  options: Record<string, string>;
}

export interface AiInsights {
  overallAssessment?: string;
  topRecommendations: string[];
  patterns: IdentifiedPattern[];
  riskAssessment?: RiskAssessment;
  refactoringOpportunities: RefactoringOpportunity[];
}

export interface IdentifiedPattern {
  name: string;
  description: string;
  confidence: number;
  locations: string[];
  impact: string;
}

export interface RiskAssessment {
  overallRisk: string;
  riskFactors: RiskFactor[];
  mitigationStrategies: string[];
}

export interface RiskFactor {
  name: string;
  description: string;
  level: string;
  likelihood: string;
  impact: string;
  affectedAreas: string[];
}

export interface RefactoringOpportunity {
  type: string;
  description: string;
  effort: string;
  benefits: string[];
  scope: string[];
  priority: string;
}

export interface ReportMetadata {
  generatedAt: string;
  uveddiVersion: string;
  configuration: Record<string, string>;
  performance?: GenerationPerformance;
}

export interface GenerationPerformance {
  analysisDurationMs: number;
  generationDurationMs: number;
  peakMemoryBytes?: number;
  filesPerSecond?: number;
}

// API Response wrapper types
export interface ApiResponse<T> {
  data: T;
  timestamp: string;
  schemaVersion: string;
}

export interface ApiError {
  error: string;
  message: string;
  timestamp: string;
  details?: any;
}

// Additional UI-specific types
export type Severity = Finding['severity'];

export interface FilterState {
  severity: Severity[];
  categories: string[];
  search: string;
  files: string[];
}

export interface SortState {
  field: keyof Finding;
  direction: 'asc' | 'desc';
}

export interface ViewState {
  activeTab: 'dashboard' | 'findings' | 'dependencies' | 'diagrams';
  findingsView: 'list' | 'grid';
  showCodeSnippets: boolean;
  showAiInsights: boolean;
}

// Security Analysis Types
/**
 * Comprehensive security analysis results
 * 
 * Contains security-specific findings including OWASP coverage,
 * taint flow analysis, and compliance status.
 * 
 * @interface SecurityAnalysis
 * @property {SecuritySummary} summary - High-level security statistics
 * @property {Record<string, OwaspCategoryStats>} owaspCoverage - OWASP Top 10 coverage
 * @property {SecurityIssue[]} issues - Detailed security issues
 * @property {TaintFlow[]} taintFlows - Data flow vulnerabilities
 * @property {SecurityCorrelation[]} correlations - Related architectural/security issues
 * @property {ComplianceStatus} compliance - Compliance framework status (optional)
 */
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