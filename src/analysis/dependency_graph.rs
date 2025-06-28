use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::analysis::dependency_extractor::Dependency;

/// In-memory dependency graph for cycle detection
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Module name -> set of dependencies
    adjacency_list: HashMap<String, HashSet<String>>,
    /// Module name -> file path mapping
    module_files: HashMap<String, PathBuf>,
    /// All dependencies with metadata
    dependencies: Vec<Dependency>,
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
            let from_module = self.extract_module_name(&dep.from_file);
            self.adjacency_list
                .entry(from_module.clone())
                .or_insert_with(HashSet::new)
                .insert(dep.to_module.clone());
            self.module_files.insert(from_module, dep.from_file);
        }
    }

    /// Extract module name from file path
    fn extract_module_name(&self, file_path: &PathBuf) -> String {
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

/// Analysis results container
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    pub cycles: Vec<Cycle>,
    pub total_modules: usize,
    pub total_dependencies: usize,
    pub analysis_duration: std::time::Duration,
}

/// Represents a cyclic dependency
#[derive(Debug, Clone)]
pub struct Cycle {
    pub modules: Vec<String>,
    pub file_paths: Vec<PathBuf>,
    pub severity: CycleSeverity,
}

#[derive(Debug, Clone, PartialEq)]
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
