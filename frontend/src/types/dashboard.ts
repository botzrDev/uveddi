// Advanced dashboard component types for Uveddi Interactive Reports
// These extend the base InteractiveReport interfaces with specialized dashboard features

import type { Finding, GraphNode, GraphEdge } from './api';

// Declare InteractiveReport locally for dashboard usage
export interface InteractiveReport {
  id: string;
  title: string;
  summary: {
    qualityScore: number;
    totalFindings: number;
    severityBreakdown: Record<string, number>;
  };
  findings: Finding[];
  metadata: {
    timestamp: string;
    analysisTimeMs: number;
    version: string;
  };
}

// ===== PRIORITY MATRIX TYPES =====
export interface PriorityMatrixItem {
  id: string;
  title: string;
  impact: number; // 1-10 scale
  effort: number; // 1-10 scale
  severity: Finding['severity'];
  category: string;
  findingIds: string[];
  aiRecommendation?: string;
}

export interface QuadrantData {
  quickWins: PriorityMatrixItem[];
  majorProjects: PriorityMatrixItem[];
  fillIns: PriorityMatrixItem[];
  thanklessWork: PriorityMatrixItem[];
}

// ===== TECHNICAL DEBT TRACKER TYPES =====
export interface TechnicalDebtMetric {
  id: string;
  name: string;
  currentValue: number;
  targetValue: number;
  unit: string;
  trend: 'improving' | 'stable' | 'worsening';
  priority: 'high' | 'medium' | 'low';
  category: 'code_quality' | 'architecture' | 'performance' | 'security' | 'maintainability';
}

export interface DebtEvolutionPoint {
  timestamp: string;
  totalDebt: number;
  newDebt: number;
  resolvedDebt: number;
  categories: Record<string, number>;
}

export interface TechnicalDebtSummary {
  totalDebtScore: number;
  debtTrend: 'improving' | 'stable' | 'worsening';
  criticalIssues: number;
  estimatedResolutionHours: number;
  topCategories: { name: string; value: number; percentage: number }[];
  evolutionData: DebtEvolutionPoint[];
  metrics: TechnicalDebtMetric[];
}

// ===== INTERACTIVE DEPENDENCY GRAPH TYPES =====
export interface EnhancedGraphNode extends GraphNode {
  centralityScore?: number;
  clusterGroup?: string;
  riskScore?: number;
  changeFrequency?: number;
  impactRadius?: number;
  technicalDebt?: number;
  position?: { x: number; y: number };
}

export interface EnhancedGraphEdge extends GraphEdge {
  strength?: number;
  changeCorrelation?: number;
  riskLevel?: 'low' | 'medium' | 'high';
}

export interface GraphAnalytics {
  clusters: GraphCluster[];
  hotspots: GraphNode[];
  criticalPaths: GraphPath[];
  isolatedNodes: GraphNode[];
  metrics: GraphMetrics;
}

export interface GraphCluster {
  id: string;
  nodes: string[];
  cohesion: number;
  coupling: number;
  purpose?: string;
}

export interface GraphPath {
  id: string;
  nodes: string[];
  weight: number;
  description: string;
}

export interface GraphMetrics {
  modularity: number;
  avgPathLength: number;
  clusteringCoefficient: number;
  centralityDistribution: Record<string, number>;
}

// ===== QUALITY SCORECARD TYPES =====
export interface QualityMetric {
  id: string;
  name: string;
  description: string;
  value: number;
  maxValue: number;
  weight: number;
  category: 'maintainability' | 'reliability' | 'security' | 'performance' | 'testability';
  trend?: 'up' | 'down' | 'stable';
  benchmark?: number;
  subMetrics?: QualitySubMetric[];
}

export interface QualitySubMetric {
  name: string;
  value: number;
  unit: string;
  threshold?: { warning: number; critical: number };
}

export interface QualityScoreBreakdown {
  overall: number;
  categories: Record<string, number>;
  metrics: QualityMetric[];
  trends: QualityTrend[];
  recommendations: QualityRecommendation[];
}

export interface QualityTrend {
  metric: string;
  direction: 'improving' | 'degrading' | 'stable';
  change: number;
  period: string;
}

export interface QualityRecommendation {
  id: string;
  priority: 'high' | 'medium' | 'low';
  title: string;
  description: string;
  impact: number;
  effort: number;
  category: string;
  relatedMetrics: string[];
}

// ===== FIX SUGGESTION PANEL TYPES =====
export interface FixSuggestion {
  id: string;
  findingId: string;
  title: string;
  description: string;
  confidence: number;
  complexity: 'simple' | 'moderate' | 'complex';
  estimatedTime: string;
  steps: FixStep[];
  codeChanges?: CodeChange[];
  prerequisites?: string[];
  risks?: string[];
  alternatives?: FixAlternative[];
  references?: string[];
}

export interface FixStep {
  order: number;
  title: string;
  description: string;
  type: 'code_change' | 'configuration' | 'refactoring' | 'testing';
  codeSnippet?: string;
  filePath?: string;
  validation?: string;
}

export interface CodeChange {
  filePath: string;
  startLine: number;
  endLine: number;
  oldCode: string;
  newCode: string;
  explanation: string;
}

export interface FixAlternative {
  title: string;
  description: string;
  pros: string[];
  cons: string[];
  effort: number;
}

// ===== SEARCH & FILTER TYPES =====
export interface SearchQuery {
  text: string;
  filters: SearchFilters;
  sort: SearchSort;
  semantic?: boolean;
}

export interface SearchFilters {
  severity?: Finding['severity'][];
  categories?: string[];
  files?: string[];
  dateRange?: { start: string; end: string };
  confidence?: { min: number; max: number };
  hasAiInsights?: boolean;
  hasFixSuggestions?: boolean;
  tags?: string[];
}

export interface SearchSort {
  field: keyof Finding | 'relevance';
  direction: 'asc' | 'desc';
}

export interface SearchResult {
  finding: Finding;
  relevanceScore: number;
  matchedFields: string[];
  highlights: Record<string, string>;
}

// ===== HISTORICAL COMPARISON TYPES =====
export interface HistoricalSnapshot {
  id: string;
  timestamp: string;
  projectVersion?: string;
  commit?: string;
  summary: {
    totalFindings: number;
    qualityScore: number;
    debtScore: number;
    coverage: number;
  };
  categoryBreakdown: Record<string, number>;
  severityBreakdown: Record<string, number>;
}

export interface ComparisonResult {
  current: HistoricalSnapshot;
  previous: HistoricalSnapshot;
  changes: {
    totalFindings: ChangeMetric;
    qualityScore: ChangeMetric;
    debtScore: ChangeMetric;
    coverage: ChangeMetric;
    newIssues: Finding[];
    resolvedIssues: Finding[];
    categoryChanges: Record<string, ChangeMetric>;
  };
  trends: TrendAnalysis[];
}

export interface ChangeMetric {
  absolute: number;
  relative: number;
  direction: 'improved' | 'degraded' | 'stable';
}

export interface TrendAnalysis {
  metric: string;
  trend: 'improving' | 'stable' | 'worsening';
  confidence: number;
  prediction?: number;
  timeframe: string;
}

// ===== COLLABORATION TYPES =====
export interface Annotation {
  id: string;
  findingId: string;
  author: User;
  content: string;
  type: 'comment' | 'question' | 'suggestion' | 'approval';
  timestamp: string;
  replies?: AnnotationReply[];
  mentions?: User[];
  tags?: string[];
}

export interface AnnotationReply {
  id: string;
  author: User;
  content: string;
  timestamp: string;
}

export interface User {
  id: string;
  name: string;
  email: string;
  avatar?: string;
  role: 'developer' | 'lead' | 'architect' | 'manager';
}

export interface Review {
  id: string;
  title: string;
  description: string;
  assignee: User;
  reviewer: User;
  status: 'pending' | 'in_progress' | 'completed' | 'dismissed';
  priority: 'low' | 'medium' | 'high' | 'critical';
  dueDate?: string;
  findings: string[];
  annotations: string[];
  resolution?: ResolutionInfo;
}

export interface ResolutionInfo {
  timestamp: string;
  action: 'fixed' | 'dismissed' | 'deferred';
  comment: string;
  verifiedBy?: User;
}

// ===== DASHBOARD BUILDER TYPES =====
export interface DashboardLayout {
  id: string;
  name: string;
  description?: string;
  isDefault?: boolean;
  widgets: DashboardWidget[];
  layout: LayoutConfiguration;
  filters?: DashboardFilters;
  createdBy?: User;
  createdAt: string;
  lastModified: string;
}

export interface DashboardWidget {
  id: string;
  type: WidgetType;
  title: string;
  configuration: WidgetConfiguration;
  position: WidgetPosition;
  size: WidgetSize;
  dataSource: string;
  refreshInterval?: number;
}

export type WidgetType = 
  | 'priority_matrix' 
  | 'quality_scorecard'
  | 'debt_tracker'
  | 'dependency_graph'
  | 'findings_list'
  | 'trends_chart'
  | 'metrics_summary'
  | 'collaboration_feed';

export interface WidgetConfiguration {
  [key: string]: any;
  chartType?: string;
  timeRange?: string;
  groupBy?: string;
  showLegend?: boolean;
  maxItems?: number;
}

export interface WidgetPosition {
  x: number;
  y: number;
}

export interface WidgetSize {
  width: number;
  height: number;
  minWidth?: number;
  minHeight?: number;
  maxWidth?: number;
  maxHeight?: number;
}

export interface LayoutConfiguration {
  columns: number;
  rowHeight: number;
  margin: [number, number];
  compactType?: 'vertical' | 'horizontal' | null;
  preventCollision?: boolean;
}

export interface DashboardFilters {
  dateRange?: { start: string; end: string };
  projects?: string[];
  severity?: Finding['severity'][];
  categories?: string[];
}

// ===== EXPORT TYPES =====
export interface ExportConfiguration {
  format: ExportFormat;
  template?: string;
  includeCharts: boolean;
  includeDiagrams: boolean;
  includeRawData: boolean;
  customSections?: ExportSection[];
  branding?: ExportBranding;
  scheduling?: ExportSchedule;
}

export type ExportFormat = 
  | 'pdf' 
  | 'html' 
  | 'json' 
  | 'csv' 
  | 'xlsx' 
  | 'markdown' 
  | 'docx';

export interface ExportSection {
  id: string;
  title: string;
  type: 'summary' | 'findings' | 'charts' | 'custom';
  configuration: Record<string, any>;
  order: number;
}

export interface ExportBranding {
  logo?: string;
  companyName?: string;
  colors?: {
    primary: string;
    secondary: string;
    accent: string;
  };
  customCSS?: string;
}

export interface ExportSchedule {
  enabled: boolean;
  frequency: 'daily' | 'weekly' | 'monthly';
  recipients: string[];
  subject: string;
  body?: string;
}

// ===== SHARED COMPONENT PROPS INTERFACES =====
export interface BaseComponentProps {
  loading?: boolean;
  error?: string;
  className?: string;
  testId?: string;
}

export interface ResponsiveComponentProps extends BaseComponentProps {
  responsive?: boolean;
  minHeight?: number;
  maxHeight?: number;
}

export interface FilterableComponentProps extends BaseComponentProps {
  filters?: Record<string, any>;
  onFiltersChange?: (filters: Record<string, any>) => void;
}

export interface ExportableComponentProps extends BaseComponentProps {
  exportable?: boolean;
  onExport?: (format: ExportFormat) => void;
}

// Missing types referenced by tests and components
export interface Issue extends Finding {
  // Issue is an alias for Finding
}

export interface QualityMetrics {
  // Legacy name - use QualityMetric instead
  overall: number;
  categories: Record<string, number>;
}

export interface DependencyNode extends GraphNode {
  // DependencyNode is an alias for GraphNode
}

export interface TechnicalDebt {
  id: string;
  name: string;
  score: number;
  category: string;
  impact: number;
  effort: number;
}

export interface DashboardEvent<T = any> {
  type: string;
  payload: T;
  timestamp: string;
  source: string;
}

// ===== EVENT TYPES =====
export interface ComponentEvent<T = any> {
  type: string;
  payload: T;
  timestamp: string;
  source: string;
}

export interface SelectionEvent extends ComponentEvent<string[]> {
  type: 'selection_changed';
  payload: string[]; // Selected item IDs
}

export interface FilterEvent extends ComponentEvent<Record<string, any>> {
  type: 'filters_changed';
  payload: Record<string, any>;
}

export interface DrillDownEvent extends ComponentEvent<{ itemId: string; context: any }> {
  type: 'drill_down';
  payload: { itemId: string; context: any };
}