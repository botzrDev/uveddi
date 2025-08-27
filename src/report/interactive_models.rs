//! Interactive Report Models v1
//!
//! This module defines the standardized data structures for the interactive reporting system.
//! These models provide a versioned JSON API contract for serving analysis results to the
//! React SPA frontend while maintaining backward compatibility with existing report formats.
//!
//! # Schema Version
//!
//! This implements Report v1 schema as specified in the interactive reporting blueprint:
//! - Stable, versioned JSON contracts
//! - Consistent naming and structure
//! - Support for real-time updates via WebSocket
//! - Progressive enhancement with AI insights
//!
//! # Integration
//!
//! These models are designed to:
//! - Bridge existing database models with frontend requirements
//! - Support multiple output formats (HTML, JSON, portable bundles)
//! - Enable rich visualizations (Mermaid, Cytoscape, Chart.js, D3)
//! - Maintain offline-first capabilities

use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue, Dependency};
use crate::models::visualization::{ArchitecturalComponent, DiagramMetadata, DiagramType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Schema version for the interactive report format
pub const REPORT_SCHEMA_VERSION: &str = "1.0";

/// Complete interactive report data structure
///
/// This is the root data structure served by the API endpoints for interactive reports.
/// It aggregates all analysis results, metadata, and visualization data needed for
/// the React SPA to render comprehensive, interactive views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveReport {
    /// Schema version for API contract stability
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    /// Project metadata and context
    pub project: ProjectMetadata,
    /// High-level analysis summary with KPIs
    pub summary: AnalysisSummary,
    /// Detailed findings from the analysis
    pub findings: Vec<Finding>,
    /// Dependency graph data for Cytoscape visualization
    #[serde(rename = "dependencyGraph")]
    pub dependency_graph: DependencyGraph,
    /// Diagram definitions for Mermaid and custom visualizations
    pub diagrams: Vec<DiagramDefinition>,
    /// Chart.js compatible metrics data for dashboards
    #[serde(rename = "chartData")]
    pub chart_data: Option<ChartDatasets>,
    /// Enhanced performance metrics for React dashboard
    #[serde(rename = "performanceMetrics")]
    pub performance_metrics: Option<PerformanceMetrics>,
    /// Optional AI-generated insights and recommendations
    #[serde(rename = "aiInsights")]
    pub ai_insights: Option<AiInsights>,
    /// Security analysis results
    #[cfg(feature = "security")]
    #[serde(rename = "securityAnalysis")]
    pub security_analysis: Option<SecurityAnalysis>,
    /// Generation metadata
    pub metadata: ReportMetadata,
}

/// Project context and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Unique project identifier
    pub id: String,
    /// Project name (typically repository name)
    pub name: String,
    /// Git commit SHA if available
    pub commit: Option<String>,
    /// Git branch name if available
    pub branch: Option<String>,
    /// Repository URL if available
    #[serde(rename = "repoUrl")]
    pub repo_url: Option<String>,
    /// Root path analyzed
    pub path: String,
    /// Programming languages detected
    pub languages: Vec<String>,
}

/// High-level analysis summary with key performance indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Overall code quality score (0-100)
    pub coverage: f64,
    /// Total number of issues found
    #[serde(rename = "issuesTotal")]
    pub issues_total: u32,
    /// Issues grouped by severity level
    #[serde(rename = "issuesBySeverity")]
    pub issues_by_severity: HashMap<String, u32>,
    /// Issues grouped by category/type
    #[serde(rename = "issuesByCategory")]
    pub issues_by_category: HashMap<String, u32>,
    /// Total files analyzed
    #[serde(rename = "filesAnalyzed")]
    pub files_analyzed: u32,
    /// Total components identified
    #[serde(rename = "componentsAnalyzed")]
    pub components_analyzed: u32,
    /// Analysis duration in milliseconds
    #[serde(rename = "analysisDurationMs")]
    pub analysis_duration_ms: u64,
    /// Timestamp when analysis was completed
    #[serde(rename = "timeGenerated")]
    pub time_generated: DateTime<Utc>,
}

/// Individual finding/issue from the analysis
///
/// Represents a specific code quality issue, anti-pattern, or architectural problem
/// with all necessary metadata for display and navigation in the SPA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier for this finding
    pub id: String,
    /// Type of issue (e.g., "LongMethod", "DeadCode", "CyclicDependency")
    #[serde(rename = "type")]
    pub finding_type: String,
    /// Severity level (critical, high, medium, low)
    pub severity: String,
    /// Short, descriptive title
    pub title: String,
    /// Detailed description of the issue
    pub message: String,
    /// File path where the issue was found
    pub file: String,
    /// Starting line number
    #[serde(rename = "startLine")]
    pub start_line: Option<u32>,
    /// Ending line number
    #[serde(rename = "endLine")]
    pub end_line: Option<u32>,
    /// Column information if available
    pub column: Option<u32>,
    /// Code snippet showing the problematic code
    #[serde(rename = "codeSnippet")]
    pub code_snippet: Option<String>,
    /// Categorization tags for filtering
    pub tags: Vec<String>,
    /// Detector that found this issue
    pub detector: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Optional AI-generated explanation
    #[serde(rename = "aiExplanation")]
    pub ai_explanation: Option<String>,
    /// Recommended fix or remediation steps
    pub recommendation: Option<String>,
    /// Related findings (by ID)
    #[serde(rename = "relatedFindings")]
    pub related_findings: Vec<String>,
    /// Security-specific metadata
    #[cfg(feature = "security")]
    #[serde(rename = "securityMetadata")]
    pub security_metadata: Option<SecurityFindingMetadata>,
}

/// Dependency graph structure for Cytoscape.js visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Nodes in the dependency graph
    pub nodes: Vec<GraphNode>,
    /// Edges connecting the nodes
    pub edges: Vec<GraphEdge>,
    /// Graph-level metadata
    pub metadata: GraphMetadata,
}

/// Node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: String,
    /// Display label for the node
    pub label: String,
    /// File path this node represents
    pub path: String,
    /// Type of node (module, class, function, etc.)
    #[serde(rename = "type")]
    pub node_type: String,
    /// Component metrics for sizing/coloring
    pub metrics: Option<NodeMetrics>,
    /// Hierarchical group (for clustering)
    pub group: Option<String>,
    /// Additional properties for visualization
    pub properties: HashMap<String, String>,
}

/// Edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of relationship
    #[serde(rename = "type")]
    pub edge_type: String,
    /// Weight/strength of the relationship
    pub weight: Option<f64>,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Metrics for graph nodes (used for visual styling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Lines of code
    pub loc: Option<u32>,
    /// Complexity score
    pub complexity: Option<f64>,
    /// Number of dependencies
    pub dependencies: u32,
    /// Number of dependents
    pub dependents: u32,
}

/// Graph-level metadata with performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Total number of nodes
    #[serde(rename = "nodeCount")]
    pub node_count: u32,
    /// Total number of edges
    #[serde(rename = "edgeCount")]
    pub edge_count: u32,
    /// Whether the graph has cycles
    #[serde(rename = "hasCycles")]
    pub has_cycles: bool,
    /// Maximum depth in the dependency hierarchy
    #[serde(rename = "maxDepth")]
    pub max_depth: u32,
    /// Recommended Cytoscape layout based on graph characteristics
    #[serde(rename = "suggestedLayout")]
    pub suggested_layout: CytoscapeLayout,
    /// Layout configuration optimized for this graph
    #[serde(rename = "layoutConfig")]
    pub layout_config: HashMap<String, serde_json::Value>,
    /// Performance optimization settings
    #[serde(rename = "performanceConfig")]
    pub performance_config: GraphPerformanceConfig,
    /// Clustering/grouping suggestions for large graphs
    #[serde(rename = "clusteringHints")]
    pub clustering_hints: Vec<NodeCluster>,
    /// Detected cycles in the graph for visualization
    pub cycles: Vec<Vec<String>>,
}

/// Recommended Cytoscape layout based on graph characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CytoscapeLayout {
    /// Best for small graphs (< 100 nodes)
    #[serde(rename = "cose")]
    Cose,
    /// Best for large graphs (> 1000 nodes) - multi-threaded
    #[serde(rename = "cose-bilkent")]
    CoseBilkent,
    /// Best for hierarchical structures
    #[serde(rename = "dagre")]
    Dagre,
    /// Best for tree-like structures
    #[serde(rename = "breadthfirst")]
    BreadthFirst,
    /// Best for performance with many disconnected components
    #[serde(rename = "grid")]
    Grid,
    /// Best for highlighting cyclical relationships
    #[serde(rename = "circle")]
    Circle,
}

/// Performance configuration for large graph rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPerformanceConfig {
    /// Enable Level of Detail (LOD) rendering
    #[serde(rename = "enableLod")]
    pub enable_lod: bool,
    /// Batch size for rendering updates
    #[serde(rename = "batchSize")]
    pub batch_size: usize,
    /// Use texture during interactions for better performance
    #[serde(rename = "textureOnViewport")]
    pub texture_on_viewport: bool,
    /// Hide labels when zoomed out
    #[serde(rename = "hideLabelsOnViewport")]
    pub hide_labels_on_viewport: bool,
    /// Suggested viewport for initial load
    #[serde(rename = "initialViewport")]
    pub initial_viewport: Option<ViewportConfig>,
    /// Enable web worker for layout calculations
    #[serde(rename = "useWebWorker")]
    pub use_web_worker: bool,
}

/// Viewport configuration for optimal initial view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Initial zoom level (1.0 = 100%)
    pub zoom: f64,
    /// Center point x coordinate
    #[serde(rename = "centerX")]
    pub center_x: f64,
    /// Center point y coordinate
    #[serde(rename = "centerY")]
    pub center_y: f64,
}

/// Node clustering suggestion for large graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCluster {
    /// Cluster identifier
    pub id: String,
    /// Display name for the cluster
    pub name: String,
    /// Nodes belonging to this cluster
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<String>,
    /// Suggested color for cluster visualization
    pub color: String,
    /// Whether this cluster should be collapsed by default
    #[serde(rename = "defaultCollapsed")]
    pub default_collapsed: bool,
}

/// Diagram definition for Mermaid and custom visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramDefinition {
    /// Unique diagram identifier
    pub id: String,
    /// Type of diagram (mermaid, cytoscape, chart, d3)
    pub kind: String,
    /// Human-readable title
    pub title: String,
    /// Source code for the diagram (Mermaid syntax, JSON config, etc.)
    pub source: String,
    /// Optional description
    pub description: Option<String>,
    /// Related components (for context)
    pub components: Vec<String>,
    /// Metadata for rendering
    pub metadata: DiagramRenderMetadata,
}

/// Metadata for diagram rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramRenderMetadata {
    /// Preferred width for rendering
    pub width: Option<u32>,
    /// Preferred height for rendering
    pub height: Option<u32>,
    /// Theme preference (light, dark, auto)
    pub theme: Option<String>,
    /// Layout direction (TD, LR, etc.)
    pub direction: Option<String>,
    /// Additional rendering options
    pub options: HashMap<String, String>,
}

/// AI-generated insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsights {
    /// Overall codebase assessment
    #[serde(rename = "overallAssessment")]
    pub overall_assessment: Option<String>,
    /// Top priority recommendations
    #[serde(rename = "topRecommendations")]
    pub top_recommendations: Vec<String>,
    /// Identified patterns and trends
    pub patterns: Vec<IdentifiedPattern>,
    /// Risk assessment
    #[serde(rename = "riskAssessment")]
    pub risk_assessment: Option<RiskAssessment>,
    /// Refactoring opportunities
    #[serde(rename = "refactoringOpportunities")]
    pub refactoring_opportunities: Vec<RefactoringOpportunity>,
}

/// AI-identified pattern or trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedPattern {
    /// Pattern name or type
    pub name: String,
    /// Description of the pattern
    pub description: String,
    /// Confidence in the pattern detection
    pub confidence: f64,
    /// Files or components where pattern is observed
    pub locations: Vec<String>,
    /// Impact assessment
    pub impact: String,
}

/// Risk assessment from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level (low, medium, high, critical)
    #[serde(rename = "overallRisk")]
    pub overall_risk: String,
    /// Risk factors identified
    #[serde(rename = "riskFactors")]
    pub risk_factors: Vec<RiskFactor>,
    /// Mitigation strategies
    #[serde(rename = "mitigationStrategies")]
    pub mitigation_strategies: Vec<String>,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Risk factor name
    pub name: String,
    /// Description of the risk
    pub description: String,
    /// Risk level (low, medium, high, critical)
    pub level: String,
    /// Likelihood of the risk materializing
    pub likelihood: String,
    /// Potential impact if risk materializes
    pub impact: String,
    /// Affected components or areas
    #[serde(rename = "affectedAreas")]
    pub affected_areas: Vec<String>,
}

/// Refactoring opportunity identified by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOpportunity {
    /// Type of refactoring (extract_method, extract_class, etc.)
    #[serde(rename = "type")]
    pub refactoring_type: String,
    /// Description of the opportunity
    pub description: String,
    /// Estimated effort (low, medium, high)
    pub effort: String,
    /// Expected benefits
    pub benefits: Vec<String>,
    /// Files or components involved
    pub scope: Vec<String>,
    /// Priority level
    pub priority: String,
}

/// Chart.js compatible metrics data for visualization dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDatasets {
    /// Code quality metrics over time
    #[serde(rename = "qualityTrends")]
    pub quality_trends: TimeSeriesChart,
    /// Issue distribution by severity
    #[serde(rename = "severityDistribution")]
    pub severity_distribution: PieChart,
    /// Issues by category/type
    #[serde(rename = "categoryBreakdown")]
    pub category_breakdown: BarChart,
    /// Component complexity metrics
    #[serde(rename = "complexityMetrics")]
    pub complexity_metrics: ScatterChart,
    /// Performance and technical debt evolution
    #[serde(rename = "technicalDebtTrends")]
    pub technical_debt_trends: TimeSeriesChart,
}

/// Time series chart data for Chart.js line charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesChart {
    pub labels: Vec<String>,
    pub datasets: Vec<TimeSeriesDataset>,
}

/// Dataset for time series charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDataset {
    pub label: String,
    pub data: Vec<f64>,
    #[serde(rename = "borderColor")]
    pub border_color: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    pub tension: f64,
}

/// Pie chart data for Chart.js pie charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieChart {
    pub labels: Vec<String>,
    pub datasets: Vec<PieDataset>,
}

/// Dataset for pie charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieDataset {
    pub label: String,
    pub data: Vec<u32>,
    #[serde(rename = "backgroundColor")]
    pub background_color: Vec<String>,
    #[serde(rename = "borderColor")]
    pub border_color: Vec<String>,
    #[serde(rename = "borderWidth")]
    pub border_width: u32,
}

/// Bar chart data for Chart.js bar charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarChart {
    pub labels: Vec<String>,
    pub datasets: Vec<BarDataset>,
}

/// Dataset for bar charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarDataset {
    pub label: String,
    pub data: Vec<u32>,
    #[serde(rename = "backgroundColor")]
    pub background_color: Vec<String>,
    #[serde(rename = "borderColor")]
    pub border_color: Vec<String>,
    #[serde(rename = "borderWidth")]
    pub border_width: u32,
}

/// Scatter chart data for Chart.js scatter plots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterChart {
    pub datasets: Vec<ScatterDataset>,
}

/// Dataset for scatter charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterDataset {
    pub label: String,
    pub data: Vec<ScatterPoint>,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    #[serde(rename = "borderColor")]
    pub border_color: String,
    #[serde(rename = "pointRadius")]
    pub point_radius: u32,
}

/// Point in a scatter chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    /// Optional metadata for tooltips
    pub metadata: Option<HashMap<String, String>>,
}

/// Enhanced performance metrics for React dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Memory usage throughout analysis
    #[serde(rename = "memoryUsage")]
    pub memory_usage: Vec<MemoryDataPoint>,
    /// CPU utilization over time
    #[serde(rename = "cpuUtilization")]
    pub cpu_utilization: Vec<CpuDataPoint>,
    /// File processing rates
    #[serde(rename = "processingRates")]
    pub processing_rates: ProcessingRates,
    /// Analysis bottlenecks identified
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Memory usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDataPoint {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "usedMb")]
    pub used_mb: f64,
    #[serde(rename = "availableMb")]
    pub available_mb: f64,
}

/// CPU utilization data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuDataPoint {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "utilizationPercent")]
    pub utilization_percent: f64,
}

/// File processing rate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRates {
    #[serde(rename = "filesPerSecond")]
    pub files_per_second: f64,
    #[serde(rename = "linesPerSecond")]
    pub lines_per_second: f64,
    #[serde(rename = "bytesPerSecond")]
    pub bytes_per_second: f64,
}

/// Performance bottleneck identified during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub phase: String,
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,
    #[serde(rename = "percentOfTotal")]
    pub percent_of_total: f64,
    pub description: String,
}

/// Report generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generation timestamp
    #[serde(rename = "generatedAt")]
    pub generated_at: DateTime<Utc>,
    /// Version of Uveddi that generated the report
    #[serde(rename = "uveddiVersion")]
    pub uveddi_version: String,
    /// Configuration used for analysis
    pub configuration: HashMap<String, String>,
    /// Performance metrics for report generation
    pub performance: Option<GenerationPerformance>,
}

/// Performance metrics for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPerformance {
    /// Analysis duration in milliseconds
    #[serde(rename = "analysisDurationMs")]
    pub analysis_duration_ms: u64,
    /// Report generation duration in milliseconds
    #[serde(rename = "generationDurationMs")]
    pub generation_duration_ms: u64,
    /// Peak memory usage in bytes
    #[serde(rename = "peakMemoryBytes")]
    pub peak_memory_bytes: Option<u64>,
    /// Files processed per second
    #[serde(rename = "filesPerSecond")]
    pub files_per_second: Option<f64>,
}

impl InteractiveReport {
    /// Create a new InteractiveReport from existing analysis data
    pub fn from_analysis_data(
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &[AntiPatternType],
        components: Option<&[ArchitecturalComponent]>,
        dependencies: &[Dependency],
        diagrams: &[DiagramMetadata],
        project_name: String,
        project_path: String,
    ) -> Self {
        let project_id = analysis_run
            .run_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Build anti-pattern lookup map
        let anti_pattern_map: HashMap<i64, &AntiPatternType> = anti_pattern_types
            .iter()
            .filter_map(|apt| apt.anti_pattern_type_id.map(|id| (id, apt)))
            .collect::<HashMap<i64, &AntiPatternType>>(); // Explicitly collect into lookup map

        // Calculate summary statistics
        let mut issues_by_severity = HashMap::new();
        let mut issues_by_category = HashMap::new();

        for issue in issues {
            *issues_by_severity
                .entry(issue.severity.clone())
                .or_insert(0) += 1;

            if let Some(anti_pattern) = anti_pattern_map.get(&issue.anti_pattern_type_id) {
                *issues_by_category
                    .entry(anti_pattern.category.clone())
                    .or_insert(0) += 1;
            }
        }

        let summary = AnalysisSummary {
            coverage: Self::calculate_coverage_score(issues, components),
            issues_total: issues.len() as u32,
            issues_by_severity,
            issues_by_category,
            files_analyzed: analysis_run.total_files_analyzed.unwrap_or(0) as u32,
            components_analyzed: components.map(|c| c.len()).unwrap_or(0) as u32,
            analysis_duration_ms: Self::calculate_analysis_duration(analysis_run),
            time_generated: analysis_run.end_time.unwrap_or_else(Utc::now),
        };

        let findings = Self::convert_issues_to_findings(issues, &anti_pattern_map);
        let dependency_graph = Self::build_dependency_graph(components, dependencies);
        let diagram_definitions = Self::convert_diagrams(diagrams);

        Self {
            schema_version: REPORT_SCHEMA_VERSION.to_string(),
            project: ProjectMetadata {
                id: project_id,
                name: project_name,
                commit: None,   // TODO: Extract from git if available
                branch: None,   // TODO: Extract from git if available
                repo_url: None, // TODO: Extract from git if available
                path: project_path,
                languages: Self::detect_languages(components),
            },
            summary,
            findings,
            dependency_graph,
            diagrams: diagram_definitions,
            chart_data: None,          // TODO: Implement chart data generation
            performance_metrics: None, // TODO: Implement performance metrics
            ai_insights: None,         // TODO: Implement AI insights integration
            metadata: ReportMetadata {
                generated_at: Utc::now(),
                uveddi_version: env!("CARGO_PKG_VERSION").to_string(),
                configuration: HashMap::new(), // TODO: Include analysis configuration
                performance: None,             // TODO: Include performance metrics
            },
            #[cfg(feature = "security")]
            security_analysis: None, // TODO: Implement security analysis integration
        }
    }

    fn calculate_coverage_score(
        issues: &[ArchitecturalIssue],
        components: Option<&[ArchitecturalComponent]>,
    ) -> f64 {
        let total_components = components.map(|c| c.len()).unwrap_or(1) as f64;
        let total_issues = issues.len() as f64;

        // Simple scoring: fewer issues per component = higher score
        let issues_per_component = total_issues / total_components;
        (100.0 - (issues_per_component * 10.0)).max(0.0).min(100.0)
    }

    fn calculate_analysis_duration(analysis_run: &AnalysisRun) -> u64 {
        if let Some(end_time) = analysis_run.end_time {
            (end_time - analysis_run.start_time)
                .num_milliseconds()
                .max(0) as u64
        } else {
            0
        }
    }

    fn convert_issues_to_findings(
        issues: &[ArchitecturalIssue],
        anti_pattern_map: &HashMap<i64, &AntiPatternType>,
    ) -> Vec<Finding> {
        issues
            .iter()
            .enumerate()
            .map(|(index, issue)| {
                let anti_pattern = anti_pattern_map.get(&issue.anti_pattern_type_id);
                let finding_type = anti_pattern
                    .map(|ap| ap.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let tags = vec![
                    issue.severity.clone(),
                    anti_pattern
                        .map(|ap| ap.category.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                ];

                Finding {
                    id: issue
                        .issue_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| format!("finding-{}", index)),
                    finding_type,
                    severity: issue.severity.clone(),
                    title: Self::generate_finding_title(issue, anti_pattern.copied()),
                    message: issue.description.clone(),
                    file: issue.file_path.clone(),
                    start_line: issue.start_line.map(|l| l as u32),
                    end_line: issue.end_line.map(|l| l as u32),
                    column: issue.column_number.map(|c| c as u32),
                    code_snippet: issue.code_snippet.clone(),
                    tags,
                    detector: issue.detector_name.clone(),
                    confidence: 1.0, // TODO: Extract from metadata if available
                    ai_explanation: issue.ai_explanation.clone(),
                    recommendation: None,     // TODO: Generate recommendations
                    related_findings: vec![], // TODO: Implement finding correlation
                    #[cfg(feature = "security")]
                    security_metadata: Self::extract_security_metadata(&issue.metadata),
                }
            })
            .collect()
    }

    fn generate_finding_title(
        issue: &ArchitecturalIssue,
        anti_pattern: Option<&AntiPatternType>,
    ) -> String {
        let default_name = "Issue".to_string();
        let pattern_name = anti_pattern.map(|ap| &ap.name).unwrap_or(&default_name);
        let file_name = std::path::Path::new(&issue.file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        format!("{} in {}", pattern_name, file_name)
    }

    fn build_dependency_graph(
        components: Option<&[ArchitecturalComponent]>,
        dependencies: &[Dependency],
    ) -> DependencyGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_ids = std::collections::HashSet::new();

        // Add component nodes
        if let Some(comps) = components {
            for component in comps {
                let node_id = component.component_id.to_string();
                if node_ids.insert(node_id.clone()) {
                    nodes.push(GraphNode {
                        id: node_id,
                        label: component.name.clone(),
                        path: component.file_path.to_string_lossy().to_string(),
                        node_type: component.component_type.as_str().to_string(),
                        metrics: Some(NodeMetrics {
                            loc: component.metrics.lines_of_code,
                            complexity: component.metrics.complexity,
                            dependencies: component.metrics.efferent_coupling,
                            dependents: component.metrics.afferent_coupling,
                        }),
                        group: component.group.clone(),
                        properties: HashMap::new(),
                    });
                }
            }
        }

        // Add dependency edges
        for (_index, dep) in dependencies.iter().enumerate() {
            let source = dep.from_file.to_string_lossy().to_string();
            let target = dep.to_module.clone();

            // Ensure source and target nodes exist
            if node_ids.insert(source.clone()) {
                nodes.push(GraphNode {
                    id: source.clone(),
                    label: std::path::Path::new(&source)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&source)
                        .to_string(),
                    path: source.clone(),
                    node_type: "module".to_string(),
                    metrics: None,
                    group: None,
                    properties: HashMap::new(),
                });
            }

            if node_ids.insert(target.clone()) {
                nodes.push(GraphNode {
                    id: target.clone(),
                    label: target.clone(),
                    path: target.clone(),
                    node_type: "external".to_string(),
                    metrics: None,
                    group: None,
                    properties: HashMap::new(),
                });
            }

            edges.push(GraphEdge {
                source,
                target,
                edge_type: format!("{:?}", dep.dependency_type),
                weight: Some(1.0),
                properties: HashMap::new(),
            });
        }

        DependencyGraph {
            metadata: GraphMetadata {
                node_count: nodes.len() as u32,
                edge_count: edges.len() as u32,
                has_cycles: false, // TODO: Implement cycle detection
                max_depth: 0,      // TODO: Calculate graph depth
                suggested_layout: CytoscapeLayout::Cose, // Default Cytoscape layout
                layout_config: HashMap::new(),
                performance_config: GraphPerformanceConfig {
                    enable_lod: true,
                    batch_size: 100,
                    texture_on_viewport: true,
                    hide_labels_on_viewport: true,
                    initial_viewport: None,
                    use_web_worker: false,
                },
                clustering_hints: vec![],
                cycles: vec![],
            },
            nodes,
            edges,
        }
    }

    fn convert_diagrams(diagrams: &[DiagramMetadata]) -> Vec<DiagramDefinition> {
        diagrams
            .iter()
            .enumerate()
            .map(|(index, diagram)| DiagramDefinition {
                id: format!("diagram-{}", index),
                kind: match diagram.diagram_type {
                    DiagramType::Dependency => "mermaid".to_string(),
                    _ => "mermaid".to_string(),
                },
                title: format!("{:?} Diagram", diagram.diagram_type),
                source: diagram.mermaid_src.clone(),
                description: None,
                components: diagram.components.iter().map(|c| c.to_string()).collect::<Vec<String>>(), // Explicitly collect component names
                metadata: DiagramRenderMetadata {
                    width: None,
                    height: None,
                    theme: Some("default".to_string()),
                    direction: Some("TD".to_string()),
                    options: HashMap::new(),
                },
            })
            .collect()
    }

    fn detect_languages(components: Option<&[ArchitecturalComponent]>) -> Vec<String> {
        let mut languages = std::collections::HashSet::new();

        if let Some(comps) = components {
            for component in comps {
                if let Some(lang) = component.component_type.language() {
                    languages.insert(lang);
                }
            }
        }

        // Add default based on file extensions if no language-specific components found
        if languages.is_empty() {
            languages.insert("unknown".to_string());
        }

        languages.into_iter().collect::<Vec<String>>() // Explicitly collect language list
    }

    /// Extract security metadata from issue metadata JSON
    #[cfg(feature = "security")]
    fn extract_security_metadata(metadata_json: &str) -> Option<SecurityFindingMetadata> {
        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(metadata_json) {
            Some(SecurityFindingMetadata {
                owasp_category: metadata
                    .get("owasp_category")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                cwe_id: metadata
                    .get("cwe_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                cvss_score: metadata.get("cvss_score").and_then(|v| v.as_f64()),
                attack_complexity: metadata
                    .get("attack_complexity")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                attack_vector: metadata
                    .get("attack_vector")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                privileges_required: metadata
                    .get("privileges_required")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                user_interaction: metadata
                    .get("user_interaction")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                scope: metadata
                    .get("scope")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                availability_impact: metadata
                    .get("availability_impact")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                confidentiality_impact: metadata
                    .get("confidentiality_impact")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                integrity_impact: metadata
                    .get("integrity_impact")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        } else {
            None
        }
    }
}

/// Security analysis results for interactive reports
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Overall security summary with counts
    pub summary: SecuritySummary,
    /// OWASP Top 10 coverage analysis
    #[serde(rename = "owaspCoverage")]
    pub owasp_coverage: HashMap<String, OwaspCategoryStats>,
    /// Security issues found during analysis
    pub issues: Vec<SecurityIssue>,
    /// Data flow analysis results
    #[serde(rename = "taintFlows")]
    pub taint_flows: Vec<TaintFlow>,
    /// Security correlations with architectural issues
    pub correlations: Vec<SecurityCorrelation>,
    /// Compliance status with security standards
    pub compliance: Option<ComplianceStatus>,
}

/// Security summary statistics
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    /// Total number of security issues
    #[serde(rename = "totalIssues")]
    pub total_issues: u32,
    /// Critical security issues
    #[serde(rename = "criticalCount")]
    pub critical_count: u32,
    /// High severity security issues
    #[serde(rename = "highCount")]
    pub high_count: u32,
    /// Medium severity security issues
    #[serde(rename = "mediumCount")]
    pub medium_count: u32,
    /// Low severity security issues
    #[serde(rename = "lowCount")]
    pub low_count: u32,
    /// Confidence distribution
    #[serde(rename = "confidenceDistribution")]
    pub confidence_distribution: HashMap<String, u32>,
    /// Most common issue types
    #[serde(rename = "mostCommonIssues")]
    pub most_common_issues: Vec<IssueTypeStats>,
    /// Overall security score (0-100)
    #[serde(rename = "securityScore")]
    pub security_score: f64,
}

/// OWASP category statistics
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwaspCategoryStats {
    /// Number of issues found in this category
    #[serde(rename = "issuesFound")]
    pub issues_found: u32,
    /// Coverage percentage for this category
    #[serde(rename = "coveragePercentage")]
    pub coverage_percentage: f64,
    /// Average confidence of detections
    #[serde(rename = "avgConfidence")]
    pub avg_confidence: f64,
    /// Severity distribution within category
    #[serde(rename = "severityDistribution")]
    pub severity_distribution: HashMap<String, u32>,
}

/// Issue type statistics
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeStats {
    /// Type of security issue
    #[serde(rename = "issueType")]
    pub issue_type: String,
    /// Number of occurrences
    pub count: u32,
    /// Average severity level
    #[serde(rename = "avgSeverity")]
    pub avg_severity: String,
    /// Average confidence score
    #[serde(rename = "avgConfidence")]
    pub avg_confidence: f64,
}

/// Security issue details
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Unique identifier
    pub id: String,
    /// Type of security issue
    #[serde(rename = "issueType")]
    pub issue_type: String,
    /// Severity level
    pub severity: String,
    /// Confidence score (0.0-1.0)
    #[serde(rename = "confidenceScore")]
    pub confidence_score: f64,
    /// Location information
    pub location: SecurityLocation,
    /// Issue description
    pub description: String,
    /// Remediation advice
    pub remediation: String,
    /// OWASP category
    #[serde(rename = "owaspCategory")]
    pub owasp_category: Option<String>,
    /// CWE identifier
    #[serde(rename = "cweId")]
    pub cwe_id: Option<String>,
    /// CVSS score
    #[serde(rename = "cvssScore")]
    pub cvss_score: Option<f64>,
    /// External references
    pub references: Vec<String>,
    /// Related taint flows
    #[serde(rename = "relatedTaintFlows")]
    pub related_taint_flows: Vec<String>,
    /// Attack vector information
    #[serde(rename = "attackVector")]
    pub attack_vector: Option<String>,
}

/// Security issue location
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLocation {
    /// File path
    pub file: String,
    /// Starting line number
    #[serde(rename = "startLine")]
    pub start_line: u32,
    /// Ending line number
    #[serde(rename = "endLine")]
    pub end_line: u32,
    /// Starting column
    #[serde(rename = "startColumn")]
    pub start_column: Option<u32>,
    /// Ending column
    #[serde(rename = "endColumn")]
    pub end_column: Option<u32>,
    /// Code snippet
    #[serde(rename = "codeSnippet")]
    pub code_snippet: Option<String>,
}

/// Data flow analysis result
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintFlow {
    /// Unique identifier
    pub id: String,
    /// Source of the taint
    pub source: FlowNode,
    /// Sink where taint reaches
    pub sink: FlowNode,
    /// Confidence in this flow
    pub confidence: f64,
    /// Sanitizers in the flow path
    pub sanitizers: Vec<FlowNode>,
    /// Flow path (intermediate nodes)
    pub path: Vec<FlowNode>,
    /// Vulnerability type this flow can lead to
    #[serde(rename = "vulnerabilityType")]
    pub vulnerability_type: String,
}

/// Node in a data flow
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    /// Node name/identifier
    pub name: String,
    /// File location
    pub location: String,
    /// Type of node (source, sink, sanitizer, intermediate)
    #[serde(rename = "nodeType")]
    pub node_type: String,
    /// Line number
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Correlation between security and architectural issues
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCorrelation {
    /// Security issue ID
    #[serde(rename = "securityIssueId")]
    pub security_issue_id: String,
    /// Related architectural issue ID
    #[serde(rename = "architecturalIssueId")]
    pub architectural_issue_id: String,
    /// Strength of correlation (0.0-1.0)
    #[serde(rename = "correlationStrength")]
    pub correlation_strength: f64,
    /// Type of correlation
    #[serde(rename = "correlationType")]
    pub correlation_type: String,
    /// Explanation of the correlation
    pub explanation: String,
}

/// Compliance status with security standards
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// OWASP compliance score
    #[serde(rename = "owaspScore")]
    pub owasp_score: f64,
    /// CWE coverage score
    #[serde(rename = "cweScore")]
    pub cwe_score: f64,
    /// Compliance with specific standards
    pub standards: HashMap<String, StandardCompliance>,
}

/// Compliance with a specific standard
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardCompliance {
    /// Name of the standard
    pub name: String,
    /// Compliance score (0.0-1.0)
    pub score: f64,
    /// Required controls/checks
    #[serde(rename = "requiredControls")]
    pub required_controls: u32,
    /// Passed controls/checks
    #[serde(rename = "passedControls")]
    pub passed_controls: u32,
    /// Failed controls/checks
    #[serde(rename = "failedControls")]
    pub failed_controls: u32,
}

/// Security-specific metadata for findings
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFindingMetadata {
    /// OWASP Top 10 category
    #[serde(rename = "owaspCategory")]
    pub owasp_category: Option<String>,
    /// CWE identifier
    #[serde(rename = "cweId")]
    pub cwe_id: Option<String>,
    /// CVSS score
    #[serde(rename = "cvssScore")]
    pub cvss_score: Option<f64>,
    /// Attack complexity
    #[serde(rename = "attackComplexity")]
    pub attack_complexity: Option<String>,
    /// Attack vector
    #[serde(rename = "attackVector")]
    pub attack_vector: Option<String>,
    /// Required privileges
    #[serde(rename = "privilegesRequired")]
    pub privileges_required: Option<String>,
    /// User interaction required
    #[serde(rename = "userInteraction")]
    pub user_interaction: Option<String>,
    /// Scope of impact
    pub scope: Option<String>,
    /// Availability impact
    #[serde(rename = "availabilityImpact")]
    pub availability_impact: Option<String>,
    /// Confidentiality impact
    #[serde(rename = "confidentialityImpact")]
    pub confidentiality_impact: Option<String>,
    /// Integrity impact
    #[serde(rename = "integrityImpact")]
    pub integrity_impact: Option<String>,
}

impl Default for InteractiveReport {
    fn default() -> Self {
        Self {
            schema_version: REPORT_SCHEMA_VERSION.to_string(),
            project: ProjectMetadata {
                id: Uuid::new_v4().to_string(),
                name: "Demo Project".to_string(),
                commit: None,
                branch: None,
                repo_url: None,
                path: "/demo".to_string(),
                languages: vec!["rust".to_string()],
            },
            summary: AnalysisSummary {
                coverage: 85.0,
                issues_total: 12,
                issues_by_severity: [
                    ("critical".to_string(), 1),
                    ("high".to_string(), 3),
                    ("medium".to_string(), 5),
                    ("low".to_string(), 3),
                ]
                .iter()
                .cloned()
                .collect(),
                issues_by_category: [
                    ("structural".to_string(), 7),
                    ("behavioral".to_string(), 3),
                    ("performance".to_string(), 2),
                ]
                .iter()
                .cloned()
                .collect(),
                files_analyzed: 42,
                components_analyzed: 18,
                analysis_duration_ms: 1250,
                time_generated: Utc::now(),
            },
            findings: vec![],
            dependency_graph: DependencyGraph {
                nodes: vec![],
                edges: vec![],
                metadata: GraphMetadata {
                    node_count: 0,
                    edge_count: 0,
                    has_cycles: false,
                    max_depth: 0,
                    suggested_layout: CytoscapeLayout::Cose,
                    layout_config: HashMap::new(),
                    performance_config: GraphPerformanceConfig {
                        enable_lod: true,
                        batch_size: 100,
                        texture_on_viewport: true,
                        hide_labels_on_viewport: true,
                        initial_viewport: None,
                        use_web_worker: false,
                    },
                    clustering_hints: vec![],
                    cycles: vec![],
                },
            },
            diagrams: vec![],
            chart_data: None,          // TODO: Implement chart data generation
            performance_metrics: None, // TODO: Implement performance metrics
            ai_insights: None,
            #[cfg(feature = "security")]
            security_analysis: None,
            metadata: ReportMetadata {
                generated_at: Utc::now(),
                uveddi_version: env!("CARGO_PKG_VERSION").to_string(),
                configuration: HashMap::new(),
                performance: None,
            },
        }
    }
}
