// Type definitions for the Uveddi Interactive Reports API
// These types mirror the Rust Report v1 data structures

export interface InteractiveReport {
  schemaVersion: string;
  project: ProjectMetadata;
  summary: AnalysisSummary;
  findings: Finding[];
  dependencyGraph: DependencyGraph;
  diagrams: DiagramDefinition[];
  aiInsights?: AiInsights;
  metadata: ReportMetadata;
}

export interface ProjectMetadata {
  id: string;
  name: string;
  commit?: string;
  branch?: string;
  repoUrl?: string;
  path: string;
  languages: string[];
}

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