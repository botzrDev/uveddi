// Data transformation utilities for converting InteractiveReport data
// to component-specific formats for advanced dashboard components

import type {
  InteractiveReport,
  Finding,
  AnalysisSummary,
  DependencyGraph,
  GraphNode,
  GraphEdge,
} from '../types/api';

import type {
  PriorityMatrixItem,
  QuadrantData,
  TechnicalDebtSummary,
  TechnicalDebtMetric,
  DebtEvolutionPoint,
  QualityScoreBreakdown,
  QualityMetric,
  QualitySubMetric,
  EnhancedGraphNode,
  EnhancedGraphEdge,
  GraphAnalytics,
  GraphCluster,
  HistoricalSnapshot,
  SearchResult,
} from '../types/dashboard';

/**
 * PRIORITY MATRIX TRANSFORMATIONS
 */

// Impact scoring based on finding characteristics
const calculateImpact = (finding: Finding): number => {
  let impact = 0;
  
  // Base impact from severity
  switch (finding.severity) {
    case 'critical': impact += 8; break;
    case 'high': impact += 6; break;
    case 'medium': impact += 4; break;
    case 'low': impact += 2; break;
  }
  
  // Adjust for confidence
  impact *= (finding.confidence / 100);
  
  // Adjust for code location (assuming more central files have higher impact)
  if (finding.file.includes('index') || finding.file.includes('main')) impact += 1;
  if (finding.file.includes('component') || finding.file.includes('service')) impact += 0.5;
  
  return Math.min(Math.max(Math.round(impact), 1), 10);
};

// Effort estimation based on finding type and complexity
const calculateEffort = (finding: Finding): number => {
  let effort = 5; // Default medium effort
  
  // Adjust by detector type
  const detectorEffortMap: Record<string, number> = {
    'god_object': 8,
    'circular_dependency': 7,
    'tight_coupling': 6,
    'magic_values': 3,
    'dead_code': 2,
    'unused_variable': 1,
    'missing_documentation': 2,
    'code_duplication': 4,
  };
  
  effort = detectorEffortMap[finding.detector] || effort;
  
  // Adjust by file size (assuming larger files are harder to fix)
  const codeLines = finding.codeSnippet?.split('\n').length || 5;
  if (codeLines > 50) effort += 2;
  else if (codeLines > 20) effort += 1;
  
  return Math.min(Math.max(Math.round(effort), 1), 10);
};

export const transformFindingsToPriorityMatrix = (findings: Finding[]): PriorityMatrixItem[] => {
  const matrixItems: PriorityMatrixItem[] = findings.map(finding => ({
    id: finding.id,
    title: finding.title,
    impact: calculateImpact(finding),
    effort: calculateEffort(finding),
    severity: finding.severity,
    category: finding.detector,
    findingIds: [finding.id],
    aiRecommendation: finding.recommendation,
  }));
  
  return matrixItems;
};

export const groupMatrixItemsByQuadrant = (items: PriorityMatrixItem[]): QuadrantData => {
  const quadrants: QuadrantData = {
    quickWins: [],      // High Impact, Low Effort (top-left)
    majorProjects: [],  // High Impact, High Effort (top-right)
    fillIns: [],        // Low Impact, Low Effort (bottom-left)
    thanklessWork: [],  // Low Impact, High Effort (bottom-right)
  };
  
  items.forEach(item => {
    const highImpact = item.impact >= 6;
    const highEffort = item.effort >= 6;
    
    if (highImpact && !highEffort) {
      quadrants.quickWins.push(item);
    } else if (highImpact && highEffort) {
      quadrants.majorProjects.push(item);
    } else if (!highImpact && !highEffort) {
      quadrants.fillIns.push(item);
    } else {
      quadrants.thanklessWork.push(item);
    }
  });
  
  return quadrants;
};

/**
 * TECHNICAL DEBT TRANSFORMATIONS
 */

export const calculateTechnicalDebtScore = (findings: Finding[]): number => {
  let totalDebt = 0;
  
  findings.forEach(finding => {
    const severityWeight = {
      critical: 10,
      high: 7,
      medium: 4,
      low: 1,
    };
    
    const weight = severityWeight[finding.severity];
    const confidence = finding.confidence / 100;
    
    totalDebt += weight * confidence;
  });
  
  // Normalize to 0-100 scale
  return Math.min(Math.round((totalDebt / findings.length) * 10), 100);
};

export const transformToTechnicalDebtSummary = (
  report: InteractiveReport,
  historicalData?: DebtEvolutionPoint[]
): TechnicalDebtSummary => {
  const findings = report.findings || [];
  const totalDebtScore = calculateTechnicalDebtScore(findings);
  
  // Calculate category breakdown
  const categoryBreakdown: Record<string, number> = {};
  findings.forEach(finding => {
    const category = finding.detector;
    categoryBreakdown[category] = (categoryBreakdown[category] || 0) + 1;
  });
  
  const topCategories = Object.entries(categoryBreakdown)
    .sort(([,a], [,b]) => b - a)
    .slice(0, 5)
    .map(([name, value]) => ({
      name,
      value,
      percentage: Math.round((value / findings.length) * 100),
    }));
  
  // Create metrics
  const metrics: TechnicalDebtMetric[] = [
    {
      id: 'code_quality',
      name: 'Code Quality Score',
      currentValue: Math.round(report.summary?.coverage || 0),
      targetValue: 90,
      unit: '%',
      trend: 'stable',
      priority: 'high',
      category: 'code_quality',
    },
    {
      id: 'critical_issues',
      name: 'Critical Issues',
      currentValue: report.summary?.issuesBySeverity?.critical || 0,
      targetValue: 0,
      unit: 'issues',
      trend: 'worsening',
      priority: 'high',
      category: 'quality',
    },
  ];
  
  // Estimate resolution time (rough heuristic)
  const estimatedHours = findings.reduce((total, finding) => {
    const hoursByType: Record<string, number> = {
      god_object: 16,
      circular_dependency: 12,
      tight_coupling: 8,
      magic_values: 2,
      dead_code: 1,
      unused_variable: 0.5,
    };
    return total + (hoursByType[finding.detector] || 4);
  }, 0);
  
  return {
    totalDebtScore,
    debtTrend: 'stable',
    criticalIssues: report.summary?.issuesBySeverity?.critical || 0,
    estimatedResolutionHours: Math.round(estimatedHours),
    topCategories,
    evolutionData: historicalData || [],
    metrics,
  };
};

/**
 * QUALITY SCORECARD TRANSFORMATIONS
 */

const calculateMetricScore = (findings: Finding[], category: string): number => {
  const categoryFindings = findings.filter(f => 
    f.tags.includes(category) || f.detector.includes(category)
  );
  
  if (categoryFindings.length === 0) return 85; // Default good score
  
  const severityPenalty = {
    critical: 20,
    high: 10,
    medium: 5,
    low: 1,
  };
  
  let penalty = 0;
  categoryFindings.forEach(finding => {
    penalty += severityPenalty[finding.severity] * (finding.confidence / 100);
  });
  
  // Start with 100 and subtract penalties, minimum 0
  return Math.max(0, Math.min(100, 100 - Math.round(penalty)));
};

export const transformToQualityScoreBreakdown = (report: InteractiveReport): QualityScoreBreakdown => {
  const findings = report.findings || [];
  
  const categories = [
    'maintainability',
    'reliability', 
    'security',
    'performance',
    'testability'
  ];
  
  const metrics: QualityMetric[] = categories.map(category => {
    const score = calculateMetricScore(findings, category);
    
    return {
      id: category,
      name: category.charAt(0).toUpperCase() + category.slice(1),
      description: `${category} quality metrics`,
      value: score,
      maxValue: 100,
      weight: 1.0,
      category: category as any,
      trend: 'stable',
      benchmark: 80,
      subMetrics: [
        {
          name: `${category} Issues`,
          value: findings.filter(f => f.detector.includes(category)).length,
          unit: 'count',
          threshold: { warning: 5, critical: 10 },
        },
      ],
    };
  });
  
  const categoryScores = categories.reduce((acc, cat) => {
    acc[cat] = calculateMetricScore(findings, cat);
    return acc;
  }, {} as Record<string, number>);
  
  const overall = Math.round(
    Object.values(categoryScores).reduce((sum, score) => sum + score, 0) / categories.length
  );
  
  return {
    overall,
    categories: categoryScores,
    metrics,
    trends: [],
    recommendations: [],
  };
};

/**
 * DEPENDENCY GRAPH TRANSFORMATIONS
 */

export const enhanceGraphNodes = (nodes: GraphNode[]): EnhancedGraphNode[] => {
  return nodes.map(node => ({
    ...node,
    centralityScore: Math.random() * 100, // TODO: Implement actual centrality calculation
    clusterGroup: `cluster_${Math.floor(Math.random() * 5)}`,
    riskScore: Math.random() * 10,
    changeFrequency: Math.random() * 100,
    impactRadius: Math.random() * 20,
    technicalDebt: Math.random() * 50,
  }));
};

export const enhanceGraphEdges = (edges: GraphEdge[]): EnhancedGraphEdge[] => {
  return edges.map(edge => ({
    ...edge,
    strength: Math.random() * 1.0,
    changeCorrelation: Math.random() * 1.0,
    riskLevel: Math.random() > 0.7 ? 'high' : Math.random() > 0.4 ? 'medium' : 'low',
  }));
};

export const analyzeGraph = (
  nodes: EnhancedGraphNode[], 
  edges: EnhancedGraphEdge[]
): GraphAnalytics => {
  // Simple clustering based on connections (placeholder implementation)
  const clusters: GraphCluster[] = [
    {
      id: 'cluster_0',
      nodes: nodes.filter(n => n.clusterGroup === 'cluster_0').map(n => n.id),
      cohesion: Math.random(),
      coupling: Math.random(),
      purpose: 'Core functionality',
    },
  ];
  
  // Find nodes with highest centrality as hotspots
  const hotspots = nodes
    .sort((a, b) => (b.centralityScore || 0) - (a.centralityScore || 0))
    .slice(0, 5);
  
  return {
    clusters,
    hotspots,
    criticalPaths: [],
    isolatedNodes: nodes.filter(n => !edges.some(e => e.source === n.id || e.target === n.id)),
    metrics: {
      modularity: Math.random(),
      avgPathLength: Math.random() * 10,
      clusteringCoefficient: Math.random(),
      centralityDistribution: {},
    },
  };
};

/**
 * SEARCH TRANSFORMATIONS
 */

export const searchFindings = (
  findings: Finding[],
  query: string,
  filters?: any
): SearchResult[] => {
  const lowerQuery = query.toLowerCase();
  
  const results: SearchResult[] = findings
    .map(finding => {
      let relevanceScore = 0;
      const matchedFields: string[] = [];
      const highlights: Record<string, string> = {};
      
      // Title matching (highest weight)
      if (finding.title.toLowerCase().includes(lowerQuery)) {
        relevanceScore += 10;
        matchedFields.push('title');
        highlights.title = finding.title;
      }
      
      // Message matching
      if (finding.message.toLowerCase().includes(lowerQuery)) {
        relevanceScore += 5;
        matchedFields.push('message');
        highlights.message = finding.message;
      }
      
      // File path matching
      if (finding.file.toLowerCase().includes(lowerQuery)) {
        relevanceScore += 3;
        matchedFields.push('file');
        highlights.file = finding.file;
      }
      
      // Tags matching
      const matchingTags = finding.tags.filter(tag => 
        tag.toLowerCase().includes(lowerQuery)
      );
      if (matchingTags.length > 0) {
        relevanceScore += 2 * matchingTags.length;
        matchedFields.push('tags');
        highlights.tags = matchingTags.join(', ');
      }
      
      return {
        finding,
        relevanceScore,
        matchedFields,
        highlights,
      };
    })
    .filter(result => result.relevanceScore > 0)
    .sort((a, b) => b.relevanceScore - a.relevanceScore);
  
  return results;
};

/**
 * HISTORICAL COMPARISON TRANSFORMATIONS
 */

export const createHistoricalSnapshot = (report: InteractiveReport): HistoricalSnapshot => {
  const summary = report.summary;
  
  return {
    id: `snapshot_${Date.now()}`,
    timestamp: new Date().toISOString(),
    projectVersion: report.project?.commit || '1.0.0',
    commit: report.project?.commit,
    summary: {
      totalFindings: summary?.issuesTotal || 0,
      qualityScore: Math.round(summary?.coverage || 0),
      debtScore: calculateTechnicalDebtScore(report.findings || []),
      coverage: Math.round(summary?.coverage || 0),
    },
    categoryBreakdown: summary?.issuesByCategory || {},
    severityBreakdown: summary?.issuesBySeverity || {},
  };
};

/**
 * UTILITY FUNCTIONS
 */

// Format numbers for display
export const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  } else if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}K`;
  }
  return num.toString();
};

// Format percentages
export const formatPercentage = (num: number, decimals = 1): string => {
  return `${num.toFixed(decimals)}%`;
};

// Format duration
export const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.round(ms / 60000)}m`;
};

// Get severity order for sorting
export const getSeverityOrder = (severity: Finding['severity']): number => {
  const order = { critical: 0, high: 1, medium: 2, low: 3 };
  return order[severity];
};

// Group findings by a specific field
export const groupFindingsBy = <K extends keyof Finding>(
  findings: Finding[],
  field: K
): Record<string, Finding[]> => {
  return findings.reduce((groups, finding) => {
    const key = String(finding[field]);
    if (!groups[key]) groups[key] = [];
    groups[key].push(finding);
    return groups;
  }, {} as Record<string, Finding[]>);
};

// Calculate trend direction
export const calculateTrend = (current: number, previous: number): 'improving' | 'stable' | 'worsening' => {
  const change = ((current - previous) / previous) * 100;
  if (Math.abs(change) < 5) return 'stable';
  return change > 0 ? 'worsening' : 'improving'; // Note: Higher numbers typically mean worse for issues
};

// Export all transformations
export const dataTransformers = {
  priorityMatrix: {
    transformFindings: transformFindingsToPriorityMatrix,
    groupByQuadrant: groupMatrixItemsByQuadrant,
  },
  technicalDebt: {
    calculateScore: calculateTechnicalDebtScore,
    transformSummary: transformToTechnicalDebtSummary,
  },
  qualityScore: {
    transformBreakdown: transformToQualityScoreBreakdown,
  },
  dependencyGraph: {
    enhanceNodes: enhanceGraphNodes,
    enhanceEdges: enhanceGraphEdges,
    analyze: analyzeGraph,
  },
  search: {
    searchFindings,
  },
  historical: {
    createSnapshot: createHistoricalSnapshot,
  },
  utils: {
    formatNumber,
    formatPercentage,
    formatDuration,
    getSeverityOrder,
    groupFindingsBy,
    calculateTrend,
  },
};