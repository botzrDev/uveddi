use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: Option<String>,
    pub source: DependencySource,
    pub scope: DependencyScope,
    pub resolved_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencySource {
    Registry(String),
    Git { url: String, branch: Option<String> },
    Local(PathBuf),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyScope {
    Production,
    Development,
    Build,
    Test,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub roots: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub info: DependencyInfo,
    pub depth: usize,
    pub is_direct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Direct,
    Transitive,
    Optional,
    PeerDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: VulnerabilitySeverity,
    pub cve_id: Option<String>,
    pub description: String,
    pub affected_versions: Vec<String>,
    pub fixed_versions: Vec<String>,
    pub published_date: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: VulnerabilitySeverity,
    pub affected_packages: Vec<String>,
    pub patched_versions: Vec<String>,
    pub cwe_ids: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub spdx_id: Option<String>,
    pub name: String,
    pub url: Option<String>,
    pub is_osi_approved: bool,
    pub is_fsf_approved: bool,
    pub category: LicenseCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseCategory {
    Permissive,
    Copyleft,
    WeakCopyleft,
    Proprietary,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConflict {
    pub package1: String,
    pub license1: LicenseInfo,
    pub package2: String,
    pub license2: LicenseInfo,
    pub conflict_type: ConflictType,
    pub resolution_suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    Incompatible,
    RequiresAttribution,
    RequiresDisclosure,
    RestrictsCommercialUse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIntegrity {
    pub package_name: String,
    pub version: String,
    pub checksum: Option<String>,
    pub signature: Option<String>,
    pub is_verified: bool,
    pub verification_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetrics {
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub transitive_dependencies: usize,
    pub outdated_count: usize,
    pub vulnerable_count: usize,
    pub max_depth: usize,
    pub avg_depth: f64,
    pub license_types: HashMap<LicenseCategory, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub cycle: Vec<String>,
    pub entry_point: String,
    pub severity: CircularDependencySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CircularDependencySeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedDependency {
    pub package_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub version_behind: VersionDistance,
    pub update_urgency: UpdateUrgency,
    pub breaking_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionDistance {
    Patch(u32),
    Minor(u32),
    Major(u32),
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateUrgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainRisk {
    pub package_name: String,
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactor {
    UnmaintainedPackage { last_update_days: u32 },
    FewContributors { count: usize },
    NoSecurityPolicy,
    SuspiciousPatterns { patterns: Vec<String> },
    UnverifiedPublisher,
    RecentOwnershipChange,
    Typosquatting { similar_to: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysisResult {
    pub graph: DependencyGraph,
    pub vulnerabilities: Vec<(String, Vec<Vulnerability>)>,
    pub license_conflicts: Vec<LicenseConflict>,
    pub circular_dependencies: Vec<CircularDependency>,
    pub outdated_dependencies: Vec<OutdatedDependency>,
    pub supply_chain_risks: Vec<SupplyChainRisk>,
    pub metrics: DependencyMetrics,
}

#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    #[error("Failed to parse dependency file: {0}")]
    ParseError(String),

    #[error("Dependency not found: {0}")]
    NotFound(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("License conflict: {0}")]
    LicenseConflict(String),

    #[error("Security vulnerability: {0}")]
    SecurityVulnerability(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    NetworkError(String),
}
