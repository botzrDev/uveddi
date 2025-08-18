// Dashboard component exports
// This file provides a centralized export for all advanced dashboard components

// Phase 1: Foundation Components
export { default as PriorityMatrix } from './PriorityMatrix';
export { default as QualityScoreCard } from './QualityScoreCard';

// Phase 2: Data Visualization Components
export { default as InteractiveDependencyGraph } from './InteractiveDependencyGraph';
export { default as TechnicalDebtTracker } from './TechnicalDebtTracker';

// Phase 3: Interactive Features
export { default as SmartSearchPanel } from './SmartSearchPanel';
export { default as FixSuggestionPanel } from './FixSuggestionPanel';

// Phase 4: Advanced Features
export { default as HistoricalComparison } from './HistoricalComparison';
export { default as CollaborationPanel } from './CollaborationPanel';

// Phase 5: Customization Components
export { default as DashboardBuilder } from './DashboardBuilder';
export { default as ExportHub } from './ExportHub';

// Type exports for component props
export type {
  PriorityMatrixItem,
  QuadrantData,
  TechnicalDebtSummary,
  TechnicalDebtMetric,
  QualityScoreBreakdown,
  QualityMetric,
  EnhancedGraphNode,
  EnhancedGraphEdge,
  GraphAnalytics,
  FixSuggestion,
  FixStep,
  SearchQuery,
  SearchResult,
  HistoricalSnapshot,
  ComparisonResult,
  Annotation,
  Review,
  DashboardLayout,
  DashboardWidget,
  ExportConfiguration,
  // Base types
  ResponsiveComponentProps,
  ExportableComponentProps,
  FilterableComponentProps,
  BaseComponentProps,
  ComponentEvent,
  SelectionEvent,
  FilterEvent,
  DrillDownEvent,
} from '../../types/dashboard';

// Utility exports
export { dataTransformers } from '../../utils/dataTransformers';
export { dashboardTheme } from '../../utils/dashboardTheme';

// Component configuration constants
export const DASHBOARD_COMPONENTS = {
  PRIORITY_MATRIX: 'priority_matrix',
  QUALITY_SCORECARD: 'quality_scorecard',
  DEPENDENCY_GRAPH: 'dependency_graph',
  DEBT_TRACKER: 'debt_tracker',
  SEARCH_PANEL: 'search_panel',
  FIX_SUGGESTIONS: 'fix_suggestions',
  HISTORICAL_COMPARISON: 'historical_comparison',
  COLLABORATION: 'collaboration',
  DASHBOARD_BUILDER: 'dashboard_builder',
  EXPORT_HUB: 'export_hub',
} as const;

export type DashboardComponentType = typeof DASHBOARD_COMPONENTS[keyof typeof DASHBOARD_COMPONENTS];

// Default configurations for components
export const DEFAULT_COMPONENT_CONFIGS = {
  [DASHBOARD_COMPONENTS.PRIORITY_MATRIX]: {
    interactive: true,
    showQuadrantLabels: true,
    showLegend: true,
    height: 400,
    exportable: true,
  },
  [DASHBOARD_COMPONENTS.QUALITY_SCORECARD]: {
    showTrends: true,
    showBenchmarks: true,
    showBreakdown: false,
    detailed: false,
    exportable: true,
  },
  [DASHBOARD_COMPONENTS.DEPENDENCY_GRAPH]: {
    interactive: true,
    showControls: true,
    showMetrics: true,
    height: 600,
    exportable: true,
  },
  [DASHBOARD_COMPONENTS.DEBT_TRACKER]: {
    showTrends: true,
    showMetrics: true,
    showCategories: true,
    timeRange: '30d',
    exportable: true,
  },
  [DASHBOARD_COMPONENTS.SEARCH_PANEL]: {
    enableSemanticSearch: true,
    enableSavedSearches: true,
    maxResults: 100,
    height: 400,
  },
  [DASHBOARD_COMPONENTS.FIX_SUGGESTIONS]: {
    showAlternatives: true,
    showCodeChanges: true,
    interactive: true,
  },
  [DASHBOARD_COMPONENTS.HISTORICAL_COMPARISON]: {
    timeRange: '30d',
    comparisonMode: 'timeline',
    showPredictions: true,
    height: 400,
    exportable: true,
  },
  [DASHBOARD_COMPONENTS.COLLABORATION]: {
    showControls: true,
    height: 600,
  },
  [DASHBOARD_COMPONENTS.DASHBOARD_BUILDER]: {
    readOnly: false,
    height: 800,
    exportable: true,
  },
  [DASHBOARD_COMPONENTS.EXPORT_HUB]: {
    height: 600,
  },
};

// Component metadata for dashboard builder
export const COMPONENT_METADATA = {
  [DASHBOARD_COMPONENTS.PRIORITY_MATRIX]: {
    name: 'Priority Matrix',
    description: 'Impact vs Effort analysis for prioritizing fixes',
    category: 'Analysis',
    icon: 'grid_view',
    minSize: { width: 6, height: 4 },
    defaultSize: { width: 8, height: 6 },
    maxSize: { width: 12, height: 8 },
  },
  [DASHBOARD_COMPONENTS.QUALITY_SCORECARD]: {
    name: 'Quality Score Card',
    description: 'Comprehensive quality scoring with category breakdowns',
    category: 'Metrics',
    icon: 'assessment',
    minSize: { width: 4, height: 3 },
    defaultSize: { width: 6, height: 4 },
    maxSize: { width: 8, height: 6 },
  },
  [DASHBOARD_COMPONENTS.DEPENDENCY_GRAPH]: {
    name: 'Dependency Graph',
    description: 'Interactive visualization of code dependencies',
    category: 'Visualization',
    icon: 'account_tree',
    minSize: { width: 8, height: 6 },
    defaultSize: { width: 12, height: 8 },
    maxSize: { width: 12, height: 12 },
  },
  [DASHBOARD_COMPONENTS.DEBT_TRACKER]: {
    name: 'Technical Debt Tracker',
    description: 'Track technical debt accumulation and resolution',
    category: 'Metrics',
    icon: 'trending_up',
    minSize: { width: 6, height: 4 },
    defaultSize: { width: 8, height: 6 },
    maxSize: { width: 12, height: 8 },
  },
  [DASHBOARD_COMPONENTS.SEARCH_PANEL]: {
    name: 'Smart Search',
    description: 'Advanced search with AI-powered filtering',
    category: 'Tools',
    icon: 'search',
    minSize: { width: 6, height: 4 },
    defaultSize: { width: 8, height: 6 },
    maxSize: { width: 12, height: 8 },
  },
  [DASHBOARD_COMPONENTS.FIX_SUGGESTIONS]: {
    name: 'Fix Suggestions',
    description: 'Step-by-step remediation guides',
    category: 'Tools',
    icon: 'lightbulb',
    minSize: { width: 6, height: 6 },
    defaultSize: { width: 10, height: 8 },
    maxSize: { width: 12, height: 10 },
  },
  [DASHBOARD_COMPONENTS.HISTORICAL_COMPARISON]: {
    name: 'Historical Analysis',
    description: 'Trend analysis and historical comparison',
    category: 'Analysis',
    icon: 'timeline',
    minSize: { width: 8, height: 6 },
    defaultSize: { width: 12, height: 8 },
    maxSize: { width: 12, height: 10 },
  },
  [DASHBOARD_COMPONENTS.COLLABORATION]: {
    name: 'Team Collaboration',
    description: 'Team discussions and code reviews',
    category: 'Collaboration',
    icon: 'group',
    minSize: { width: 6, height: 6 },
    defaultSize: { width: 8, height: 8 },
    maxSize: { width: 12, height: 10 },
  },
  [DASHBOARD_COMPONENTS.DASHBOARD_BUILDER]: {
    name: 'Dashboard Builder',
    description: 'Drag-and-drop dashboard customization',
    category: 'Tools',
    icon: 'dashboard',
    minSize: { width: 12, height: 10 },
    defaultSize: { width: 12, height: 12 },
    maxSize: { width: 12, height: 16 },
  },
  [DASHBOARD_COMPONENTS.EXPORT_HUB]: {
    name: 'Export Hub',
    description: 'Comprehensive export and integration options',
    category: 'Tools',
    icon: 'get_app',
    minSize: { width: 8, height: 6 },
    defaultSize: { width: 10, height: 8 },
    maxSize: { width: 12, height: 10 },
  },
};

// Event types for component communication
export const DASHBOARD_EVENTS = {
  SELECTION_CHANGED: 'selection_changed',
  FILTERS_CHANGED: 'filters_changed',
  DRILL_DOWN: 'drill_down',
  EXPORT_REQUESTED: 'export_requested',
  LAYOUT_CHANGED: 'layout_changed',
  WIDGET_ADDED: 'widget_added',
  WIDGET_REMOVED: 'widget_removed',
  WIDGET_CONFIGURED: 'widget_configured',
} as const;

// Helper functions for component integration
export const createComponentId = (type: DashboardComponentType, suffix?: string): string => {
  const id = `${type}_${Date.now()}`;
  return suffix ? `${id}_${suffix}` : id;
};

export const getComponentConfig = (type: DashboardComponentType) => {
  return DEFAULT_COMPONENT_CONFIGS[type] || {};
};

export const getComponentMetadata = (type: DashboardComponentType) => {
  return COMPONENT_METADATA[type];
};

// Validation helpers
export const validateComponentProps = (type: DashboardComponentType, props: any): boolean => {
  // Add validation logic for different component types
  switch (type) {
    case DASHBOARD_COMPONENTS.PRIORITY_MATRIX:
      return Array.isArray(props.findings);
    case DASHBOARD_COMPONENTS.QUALITY_SCORECARD:
      return props.report && props.report.summary;
    case DASHBOARD_COMPONENTS.DEPENDENCY_GRAPH:
      return props.dependencyGraph && Array.isArray(props.dependencyGraph.nodes);
    case DASHBOARD_COMPONENTS.DEBT_TRACKER:
      return props.report && Array.isArray(props.report.findings);
    case DASHBOARD_COMPONENTS.SEARCH_PANEL:
      return Array.isArray(props.findings);
    case DASHBOARD_COMPONENTS.FIX_SUGGESTIONS:
      return Array.isArray(props.findings);
    case DASHBOARD_COMPONENTS.HISTORICAL_COMPARISON:
      return props.currentReport;
    case DASHBOARD_COMPONENTS.COLLABORATION:
      return Array.isArray(props.findings);
    case DASHBOARD_COMPONENTS.DASHBOARD_BUILDER:
      return props.report;
    case DASHBOARD_COMPONENTS.EXPORT_HUB:
      return props.report;
    default:
      return true;
  }
};

// Theme integration helpers
export const getComponentTheme = (type: DashboardComponentType, mode: 'light' | 'dark' = 'light') => {
  return dashboardTheme.createTheme(mode);
};

// Performance optimization helpers
export const shouldComponentUpdate = (
  type: DashboardComponentType,
  prevProps: any,
  nextProps: any
): boolean => {
  // Implement shallow comparison logic for different components
  switch (type) {
    case DASHBOARD_COMPONENTS.PRIORITY_MATRIX:
      return (
        prevProps.findings !== nextProps.findings ||
        prevProps.interactive !== nextProps.interactive
      );
    case DASHBOARD_COMPONENTS.QUALITY_SCORECARD:
      return (
        prevProps.report !== nextProps.report ||
        prevProps.showTrends !== nextProps.showTrends
      );
    case DASHBOARD_COMPONENTS.DEPENDENCY_GRAPH:
      return (
        prevProps.dependencyGraph !== nextProps.dependencyGraph ||
        prevProps.showControls !== nextProps.showControls
      );
    case DASHBOARD_COMPONENTS.DEBT_TRACKER:
      return (
        prevProps.report !== nextProps.report ||
        prevProps.timeRange !== nextProps.timeRange
      );
    case DASHBOARD_COMPONENTS.SEARCH_PANEL:
      return (
        prevProps.findings !== nextProps.findings ||
        prevProps.enableSemanticSearch !== nextProps.enableSemanticSearch
      );
    case DASHBOARD_COMPONENTS.FIX_SUGGESTIONS:
      return (
        prevProps.findings !== nextProps.findings ||
        prevProps.selectedFindingId !== nextProps.selectedFindingId
      );
    case DASHBOARD_COMPONENTS.HISTORICAL_COMPARISON:
      return (
        prevProps.currentReport !== nextProps.currentReport ||
        prevProps.timeRange !== nextProps.timeRange
      );
    case DASHBOARD_COMPONENTS.COLLABORATION:
      return (
        prevProps.findings !== nextProps.findings ||
        prevProps.currentUser !== nextProps.currentUser
      );
    case DASHBOARD_COMPONENTS.DASHBOARD_BUILDER:
      return (
        prevProps.report !== nextProps.report ||
        prevProps.readOnly !== nextProps.readOnly
      );
    case DASHBOARD_COMPONENTS.EXPORT_HUB:
      return prevProps.report !== nextProps.report;
    default:
      return true;
  }
};

export default {
  // Components
  PriorityMatrix,
  QualityScoreCard,
  InteractiveDependencyGraph,
  TechnicalDebtTracker,
  SmartSearchPanel,
  FixSuggestionPanel,
  HistoricalComparison,
  CollaborationPanel,
  DashboardBuilder,
  ExportHub,
  
  // Configuration
  DASHBOARD_COMPONENTS,
  DEFAULT_COMPONENT_CONFIGS,
  COMPONENT_METADATA,
  DASHBOARD_EVENTS,
  
  // Utilities
  dataTransformers,
  dashboardTheme,
  createComponentId,
  getComponentConfig,
  getComponentMetadata,
  validateComponentProps,
  getComponentTheme,
  shouldComponentUpdate,
};