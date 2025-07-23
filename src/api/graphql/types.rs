//! GraphQL type definitions for Uveddi analysis results

use async_graphql::{SimpleObject, Object, Enum, InputObject, Union, ID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::database::models::{
    ArchitecturalIssue as DbArchitecturalIssue,
    AnalysisRun as DbAnalysisRun,
    AntiPatternType as DbAntiPatternType,
    ComponentPerformanceMetrics as DbComponentPerformanceMetrics,
    Dependency as DbDependency,
};

// Enums

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Running,
    Completed,
    Failed,
    Pending,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DependencyType {
    Use,
    Mod,
    External,
    Import,
    DataFlow,
    ControlFlow,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum AntiPatternCategory {
    Structural,
    Behavioral,
    Creational,
    AbstractionBased,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum AnalysisRunOrderField {
    StartTime,
    EndTime,
    TotalIssues,
    TotalFiles,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum SearchType {
    Issues,
    Files,
    AntiPatterns,
    All,
}

// Core Types

#[derive(SimpleObject, Debug, Clone)]
pub struct Project {
    pub id: ID,
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AnalysisRun {
    pub id: ID,
    pub project_id: ID,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: AnalysisStatus,
    pub total_files_analyzed: Option<i32>,
    pub total_issues_found: Option<i32>,
    pub analysis_config: serde_json::Value,
}

#[Object]
impl AnalysisRun {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn project_id(&self) -> &ID {
        &self.project_id
    }

    async fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    async fn end_time(&self) -> Option<DateTime<Utc>> {
        self.end_time
    }

    async fn status(&self) -> AnalysisStatus {
        self.status
    }

    async fn total_files_analyzed(&self) -> Option<i32> {
        self.total_files_analyzed
    }

    async fn total_issues_found(&self) -> Option<i32> {
        self.total_issues_found
    }

    async fn analysis_config(&self) -> &serde_json::Value {
        &self.analysis_config
    }
}

#[derive(Debug, Clone)]
pub struct ArchitecturalIssue {
    pub id: ID,
    pub analysis_run_id: ID,
    pub anti_pattern_type_id: ID,
    pub file_path: String,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub severity: SeverityLevel,
    pub description: String,
    pub code_snippet: Option<String>,
    pub ai_explanation: Option<String>,
}

#[Object]
impl ArchitecturalIssue {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn analysis_run_id(&self) -> &ID {
        &self.analysis_run_id
    }

    async fn file_path(&self) -> &String {
        &self.file_path
    }

    async fn start_line(&self) -> Option<i32> {
        self.start_line
    }

    async fn end_line(&self) -> Option<i32> {
        self.end_line
    }

    async fn severity(&self) -> SeverityLevel {
        self.severity
    }

    async fn description(&self) -> &String {
        &self.description
    }

    async fn code_snippet(&self) -> Option<&String> {
        self.code_snippet.as_ref()
    }

    async fn ai_explanation(&self) -> Option<&String> {
        self.ai_explanation.as_ref()
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AntiPatternType {
    pub id: ID,
    pub name: String,
    pub description: String,
    pub category: AntiPatternCategory,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Dependency {
    pub from_file: String,
    pub to_module: String,
    pub dependency_type: DependencyType,
    pub line_number: Option<i32>,
    pub is_cyclic: bool,
    pub cyclic_path: Vec<String>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ParsedFile {
    pub file_path: String,
    pub language: SourceLanguage,
    pub has_ast: bool,
    pub source: Option<String>,
    pub modified_at: DateTime<Utc>,
    pub lines_of_code: i32,
    pub complexity: Option<f64>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ASTNode {
    pub node_type: String,
    pub start_line: i32,
    pub end_line: i32,
    pub start_column: i32,
    pub end_column: i32,
    pub text: Option<String>,
    pub children: Vec<ASTNode>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub cycles: Vec<DependencyCycle>,
    pub metrics: DependencyGraphMetrics,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DependencyNode {
    pub id: String,
    pub file_path: String,
    pub component_type: Option<String>,
    pub in_degree: i32,
    pub out_degree: i32,
    pub centrality: f64,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
    pub weight: i32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DependencyCycle {
    pub nodes: Vec<String>,
    pub severity: SeverityLevel,
    pub description: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DependencyGraphMetrics {
    pub total_nodes: i32,
    pub total_edges: i32,
    pub cycle_count: i32,
    pub max_depth: i32,
    pub modularity: f64,
    pub coupling: f64,
    pub cohesion: f64,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ComponentPerformanceMetrics {
    pub id: ID,
    pub component_id: String,
    pub analysis_run_id: ID,
    pub execution_time_ms: i64,
    pub memory_usage_bytes: i64,
    pub ast_parse_time_ms: Option<i64>,
    pub symbol_resolution_time_ms: Option<i64>,
    pub dependency_extraction_time_ms: Option<i64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct IncrementalAnalysisInfo {
    pub was_incremental: bool,
    pub files_reanalyzed: i32,
    pub total_files: i32,
    pub time_saved_ms: i64,
    pub analysis_time: DateTime<Utc>,
    pub change_summary: ChangeSummary,
    pub performance_metrics: IncrementalPerformanceMetrics,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ChangeSummary {
    pub changed_files: Vec<String>,
    pub affected_files: Vec<String>,
    pub new_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub change_percentage: f64,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct IncrementalPerformanceMetrics {
    pub cache_hit_rate: f64,
    pub memory_reduction: f64,
    pub time_reduction: f64,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct SeverityCount {
    pub severity: SeverityLevel,
    pub count: i32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AnalyticsSummary {
    pub total_projects: i32,
    pub total_analysis_runs: i32,
    pub total_issues_found: i32,
    pub most_common_anti_patterns: Vec<AntiPatternTypeCount>,
    pub language_distribution: Vec<LanguageCount>,
    pub trend_data: TrendData,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AntiPatternTypeCount {
    pub anti_pattern_type: AntiPatternType,
    pub count: i32,
    pub percentage: f64,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct LanguageCount {
    pub language: SourceLanguage,
    pub count: i32,
    pub percentage: f64,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct TrendData {
    pub issues_trend: Vec<TrendPoint>,
    pub performance_trend: Vec<TrendPoint>,
    pub quality_score_trend: Vec<TrendPoint>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct TrendPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

// Connection Types for Pagination

#[derive(SimpleObject, Debug, Clone)]
pub struct AnalysisRunConnection {
    pub edges: Vec<AnalysisRunEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AnalysisRunEdge {
    pub node: AnalysisRun,
    pub cursor: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ArchitecturalIssueConnection {
    pub edges: Vec<ArchitecturalIssueEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
    pub severity_breakdown: Vec<SeverityCount>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ArchitecturalIssueEdge {
    pub node: ArchitecturalIssue,
    pub cursor: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ProjectConnection {
    pub edges: Vec<ProjectEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ProjectEdge {
    pub node: Project,
    pub cursor: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct SearchResultConnection {
    pub edges: Vec<SearchResultEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct SearchResultEdge {
    pub node: SearchResult,
    pub cursor: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

// Union Types

#[derive(Union)]
pub enum SearchResult {
    Issue(ArchitecturalIssue),
    File(ParsedFile),
    AntiPattern(AntiPatternType),
}

// Input Types

#[derive(InputObject, Debug, Clone)]
pub struct AnalysisRunOrderBy {
    pub field: AnalysisRunOrderField,
    pub direction: OrderDirection,
}

#[derive(InputObject, Debug, Clone)]
pub struct AnalysisConfigInput {
    pub enabled_detectors: Option<Vec<String>>,
    pub enable_ai: Option<bool>,
    pub ai_provider: Option<String>,
    pub max_files_per_batch: Option<i32>,
    pub enable_incremental: Option<bool>,
    pub output_format: Option<String>,
    pub custom_rules: Option<serde_json::Value>,
}

#[derive(InputObject, Debug, Clone)]
pub struct FileFilter {
    pub paths: Option<Vec<String>>,
    pub languages: Option<Vec<SourceLanguage>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub include_patterns: Option<Vec<String>>,
}

#[derive(InputObject, Debug, Clone)]
pub struct CreateProjectInput {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub analysis_config: Option<AnalysisConfigInput>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
    pub analysis_config: Option<AnalysisConfigInput>,
}

#[derive(InputObject, Debug, Clone)]
pub struct StartAnalysisInput {
    pub project_id: ID,
    pub config: Option<AnalysisConfigInput>,
    pub file_filter: Option<FileFilter>,
    pub enable_incremental: Option<bool>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateIssueInput {
    pub description: Option<String>,
    pub severity: Option<SeverityLevel>,
    pub dismissed: Option<bool>,
    pub notes: Option<String>,
}

#[derive(InputObject, Debug, Clone)]
pub struct TimeRangeInput {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

// Payload Types

#[derive(SimpleObject, Debug)]
pub struct CreateProjectPayload {
    pub project: Option<Project>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct UpdateProjectPayload {
    pub project: Option<Project>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct DeleteProjectPayload {
    pub deleted_project_id: Option<ID>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct StartAnalysisPayload {
    pub analysis_run: Option<AnalysisRun>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct CancelAnalysisPayload {
    pub analysis_run: Option<AnalysisRun>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct UpdateIssuePayload {
    pub issue: Option<ArchitecturalIssue>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct DismissIssuePayload {
    pub issue: Option<ArchitecturalIssue>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct UpdateAnalysisConfigPayload {
    pub project: Option<Project>,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct ClearCachePayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug)]
pub struct WarmupCachePayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserError {
    pub field: Option<String>,
    pub message: String,
    pub code: Option<String>,
}

// Subscription Types

#[derive(SimpleObject, Debug, Clone)]
pub struct AnalysisProgressUpdate {
    pub analysis_run_id: ID,
    pub files_processed: i32,
    pub total_files: i32,
    pub current_file: Option<String>,
    pub issues_found: i32,
    pub estimated_time_remaining: Option<i32>,
    pub status: AnalysisStatus,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct CacheStats {
    pub current_size: i32,
    pub max_size: i32,
    pub hit_rate: f64,
    pub total_hits: i64,
    pub total_misses: i64,
}

// Conversion implementations

impl From<DbArchitecturalIssue> for ArchitecturalIssue {
    fn from(db_issue: DbArchitecturalIssue) -> Self {
        Self {
            id: ID::from(db_issue.issue_id.unwrap_or(0).to_string()),
            analysis_run_id: ID::from(db_issue.analysis_run_id.to_string()),
            anti_pattern_type_id: ID::from(db_issue.anti_pattern_type_id.to_string()),
            file_path: db_issue.file_path,
            start_line: db_issue.start_line,
            end_line: db_issue.end_line,
            severity: match db_issue.severity.as_str() {
                "low" => SeverityLevel::Low,
                "medium" => SeverityLevel::Medium,
                "high" => SeverityLevel::High,
                "critical" => SeverityLevel::Critical,
                _ => SeverityLevel::Medium,
            },
            description: db_issue.description,
            code_snippet: db_issue.code_snippet,
            ai_explanation: db_issue.ai_explanation,
        }
    }
}

impl From<DbAnalysisRun> for AnalysisRun {
    fn from(db_run: DbAnalysisRun) -> Self {
        Self {
            id: ID::from(db_run.run_id.unwrap_or(0).to_string()),
            project_id: ID::from(db_run.project_id.to_string()),
            start_time: db_run.start_time,
            end_time: db_run.end_time,
            status: match db_run.status.as_str() {
                "running" => AnalysisStatus::Running,
                "completed" => AnalysisStatus::Completed,
                "failed" => AnalysisStatus::Failed,
                _ => AnalysisStatus::Pending,
            },
            total_files_analyzed: db_run.total_files_analyzed,
            total_issues_found: db_run.total_issues_found,
            analysis_config: serde_json::from_str(&db_run.analysis_config)
                .unwrap_or_default(),
        }
    }
}

impl From<DbAntiPatternType> for AntiPatternType {
    fn from(db_type: DbAntiPatternType) -> Self {
        Self {
            id: ID::from(db_type.anti_pattern_type_id.unwrap_or(0).to_string()),
            name: db_type.name,
            description: db_type.description,
            category: match db_type.category.as_str() {
                "structural" => AntiPatternCategory::Structural,
                "behavioral" => AntiPatternCategory::Behavioral,
                "creational" => AntiPatternCategory::Creational,
                "Abstraction-Based" => AntiPatternCategory::AbstractionBased,
                _ => AntiPatternCategory::Structural,
            },
        }
    }
}