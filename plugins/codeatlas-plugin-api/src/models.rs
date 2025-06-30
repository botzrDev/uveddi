use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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

/// Represents a dependency between two modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from_file: PathBuf,
    pub to_module: String,
    pub dependency_type: DependencyType,
    pub line_number: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    Use,
    Mod,
    External,
    Import, // Generic for Python/JS
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

impl ArchitecturalIssue {
    pub fn from_cycle(cycle: Cycle, graph: &DependencyGraph) -> Self {
        let severity_str = match cycle.severity {
            CycleSeverity::Low => "low".to_string(),
            CycleSeverity::Medium => "medium".to_string(),
            CycleSeverity::High => "high".to_string(),
        };

        let description = format!(
            "Cyclic dependency detected involving modules: {}. Files: {}.",
            cycle.modules.join(", "),
            cycle.file_paths.iter().map(|p| p.display().to_string()).collect::<Vec<String>>().join(", ")
        );

        // For simplicity, we'll use the first file in the cycle as the primary file_path
        // and set start/end lines to None as it's a project-wide issue.
        let file_path = cycle.file_paths.first().map_or("unknown".to_string(), |p| p.display().to_string());

        // Attempt to extract a code snippet from the first file (if available)
        let code_snippet = cycle.file_paths.first().and_then(|path| {
            std::fs::read_to_string(path).ok().map(|content| {
                let lines: Vec<&str> = content.lines().take(10).collect();
                lines.join("\n")
            })
        });

        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 0, // This will be set when saving to DB
            anti_pattern_type_id: 0, // This will be set when saving to DB
            file_path,
            start_line: None,
            end_line: None,
            severity: severity_str,
            description,
            code_snippet,
            ai_explanation: None,
        }
    }
}

/// Represents a cyclic dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub modules: Vec<String>,
    pub file_paths: Vec<PathBuf>,
    pub severity: CycleSeverity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CycleSeverity {
    Low,    // 2-3 modules
    Medium, // 4-6 modules  
    High,   // 7+ modules
}

impl Cycle {
    pub fn new(modules: Vec<String>, graph: &DependencyGraph) -> Self {
        let file_paths = modules.iter()
            .filter_map(|module| graph.get_file_path(module).cloned())
            .collect();
        let severity = match modules.len() {
            2..=3 => CycleSeverity::Low,
            4..=6 => CycleSeverity::Medium,
            _ => CycleSeverity::High,
        };
        Self {
            modules,
            file_paths,
            severity,
        }
    }
}
