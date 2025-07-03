use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Core analysis run tracking - aligns with ERD AnalysisRun entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub run_id: Option<i64>,
    pub project_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String, // "running", "completed", "failed"
    pub total_files_analyzed: Option<i32>,
    pub total_issues_found: Option<i32>,
    pub analysis_config: String, // JSON serialized config
}

/// Architectural issues with severity and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalIssue {
    pub issue_id: Option<i64>,
    pub analysis_run_id: i64,
    pub anti_pattern_type_id: i64,
    pub file_path: String,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub severity: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub code_snippet: Option<String>,
    pub ai_explanation: Option<String>,
}

/// Represents a dependency between two modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    pub from_file: PathBuf,
    pub to_module: String,
    pub dependency_type: DependencyType,
    pub line_number: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum DependencyType {
    Use,
    Mod,
    External,
    Import, // Generic for Python/JS
}

/// In-memory dependency graph for cycle detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Module name -> set of dependencies
    pub adjacency_list: HashMap<String, HashSet<String>>,
    /// Module name -> file path mapping
    pub module_files: HashMap<String, PathBuf>,
    /// All dependencies with metadata
    pub dependencies: Vec<Dependency>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            module_files: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Build graph from extracted dependencies
    pub fn build_from_dependencies(&mut self, dependencies: Vec<Dependency>) {
        self.dependencies = dependencies.clone();
        for dep in dependencies {
            let from_module = self.extract_module_name(dep.from_file.as_path());
            self.adjacency_list
                .entry(from_module.clone())
                .or_insert_with(HashSet::new)
                .insert(dep.to_module.clone());
            self.module_files.insert(from_module, dep.from_file);
        }
    }

    /// Extract module name from file path
    fn extract_module_name(&self, file_path: &std::path::Path) -> String {
        if let Some(file_stem) = file_path.file_stem().and_then(|s| s.to_str()) {
            if file_stem == "mod" {
                if let Some(parent) = file_path.parent() {
                    if let Some(parent_name) = parent.file_name().and_then(|s| s.to_str()) {
                        return parent_name.to_string();
                    }
                }
            }
            file_stem.to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn get_modules(&self) -> Vec<&String> {
        self.adjacency_list.keys().collect()
    }

    pub fn get_dependencies(&self, module: &str) -> Option<&HashSet<String>> {
        self.adjacency_list.get(module)
    }

    pub fn get_file_path(&self, module: &str) -> Option<&PathBuf> {
        self.module_files.get(module)
    }

    pub fn get_all_dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternType {
    pub anti_pattern_type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}
