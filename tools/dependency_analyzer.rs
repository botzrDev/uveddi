use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Circular dependency representation
#[derive(Debug, Clone)]
pub struct CircularDependency {
    pub modules: Vec<String>,
    pub severity: CycleSeverity,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CycleSeverity {
    Low,      // 2-3 modules, simple cycles
    Medium,   // 4-6 modules, moderate complexity
    High,     // 7+ modules, complex cycles
    Critical, // Core infrastructure cycles
}

/// Tool for detecting and analyzing circular dependencies
pub struct DependencyAnalyzer {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
    module_files: HashMap<String, Vec<PathBuf>>,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            module_files: HashMap::new(),
        }
    }

    /// Analyze project and return circular dependencies
    pub fn analyze_project(
        &mut self,
        project_path: &Path,
    ) -> Result<Vec<CircularDependency>, Box<dyn std::error::Error>> {
        println!("🔍 Analyzing dependencies in {:?}", project_path);

        self.build_dependency_graph(project_path)?;
        let cycles = self.detect_cycles();

        println!("📊 Analysis complete:");
        println!("  - Total modules: {}", self.node_map.len());
        println!("  - Total edges: {}", self.graph.edge_count());
        println!("  - Circular dependencies: {}", cycles.len());

        Ok(cycles)
    }

    /// Build dependency graph from Rust source files
    fn build_dependency_graph(
        &mut self,
        project_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Err("src directory not found".into());
        }

        // First pass: collect all modules
        self.collect_modules(&src_path)?;

        // Second pass: analyze dependencies
        self.analyze_dependencies(&src_path)?;

        Ok(())
    }

    /// Collect all module names from the codebase
    fn collect_modules(&mut self, src_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for entry in walkdir::WalkDir::new(src_path) {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "rs") {
                let module_name = self.path_to_module_name(src_path, path);

                // Add to graph if not exists
                if !self.node_map.contains_key(&module_name) {
                    let node_idx = self.graph.add_node(module_name.clone());
                    self.node_map.insert(module_name.clone(), node_idx);
                }

                // Track files for each module
                self.module_files
                    .entry(module_name)
                    .or_insert_with(Vec::new)
                    .push(path.to_path_buf());
            }
        }

        Ok(())
    }

    /// Analyze dependencies between modules
    fn analyze_dependencies(&mut self, src_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let use_regex = Regex::new(r"use\s+crate::([^;:\s]+)")?;

        for entry in walkdir::WalkDir::new(src_path) {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(path)?;
                let from_module = self.path_to_module_name(src_path, path);

                // Find all crate:: dependencies
                for captures in use_regex.captures_iter(&content) {
                    if let Some(to_module_match) = captures.get(1) {
                        let to_module = to_module_match.as_str().to_string();

                        // Add edge if both modules exist
                        if let (Some(&from_idx), Some(&to_idx)) = (
                            self.node_map.get(&from_module),
                            self.node_map.get(&to_module),
                        ) {
                            self.graph.add_edge(from_idx, to_idx, ());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert file path to module name
    fn path_to_module_name(&self, src_path: &Path, file_path: &Path) -> String {
        let relative = file_path.strip_prefix(src_path).unwrap();
        let mut components: Vec<&str> = relative
            .components()
            .map(|c| c.as_os_str().to_str().unwrap())
            .collect();

        // Remove .rs extension and handle mod.rs
        if let Some(last) = components.last_mut() {
            if last.ends_with(".rs") {
                *last = &last[..last.len() - 3];
            }
            if *last == "mod" {
                components.pop();
            }
        }

        components.join("::")
    }

    /// Detect circular dependencies using strongly connected components
    fn detect_cycles(&self) -> Vec<CircularDependency> {
        let sccs = kosaraju_scc(&self.graph);

        sccs.into_iter()
            .filter(|scc| scc.len() > 1) // Only actual cycles
            .map(|scc| {
                let modules: Vec<String> = scc
                    .into_iter()
                    .map(|node_idx| self.graph[node_idx].clone())
                    .collect();

                CircularDependency {
                    severity: self.calculate_severity(&modules),
                    suggested_fix: self.suggest_fix(&modules),
                    modules,
                }
            })
            .collect()
    }

    /// Calculate severity based on cycle characteristics
    fn calculate_severity(&self, modules: &[String]) -> CycleSeverity {
        // Check for critical infrastructure modules
        let critical_modules = ["analysis", "database", "ast", "security", "monitoring"];
        let has_critical = modules
            .iter()
            .any(|m| critical_modules.iter().any(|&crit| m.starts_with(crit)));

        if has_critical && modules.len() >= 2 {
            return CycleSeverity::Critical;
        }

        match modules.len() {
            2..=3 => CycleSeverity::Low,
            4..=6 => CycleSeverity::Medium,
            _ => CycleSeverity::High,
        }
    }

    /// Suggest fix based on cycle pattern
    fn suggest_fix(&self, modules: &[String]) -> String {
        if modules.iter().any(|m| m.contains("analysis"))
            && modules.iter().any(|m| m.contains("database"))
        {
            "Create persistence interface to break Analysis ↔ Database dependency".to_string()
        } else if modules.iter().any(|m| m.contains("ast"))
            && modules.iter().any(|m| m.contains("analysis"))
        {
            "Implement event-driven communication between AST and Analysis".to_string()
        } else if modules.iter().any(|m| m.contains("security")) {
            "Extract security audit interface".to_string()
        } else if modules.iter().any(|m| m.contains("monitoring")) {
            "Create metrics collection interface".to_string()
        } else {
            "Consider extracting shared interfaces or using dependency injection".to_string()
        }
    }

    /// Generate detailed report
    pub fn generate_report(&self, cycles: &[CircularDependency]) -> String {
        let mut report = String::new();

        report.push_str("# Circular Dependency Analysis Report\n\n");
        report.push_str(&format!("**Total Cycles Found**: {}\n\n", cycles.len()));

        // Group by severity
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for cycle in cycles {
            match cycle.severity {
                CycleSeverity::Critical => critical += 1,
                CycleSeverity::High => high += 1,
                CycleSeverity::Medium => medium += 1,
                CycleSeverity::Low => low += 1,
            }
        }

        report.push_str("## Severity Breakdown\n");
        report.push_str(&format!("- 🚨 Critical: {}\n", critical));
        report.push_str(&format!("- 🔴 High: {}\n", high));
        report.push_str(&format!("- 🟡 Medium: {}\n", medium));
        report.push_str(&format!("- 🟢 Low: {}\n\n", low));

        // Detailed cycle information
        report.push_str("## Detailed Analysis\n\n");
        for (i, cycle) in cycles.iter().enumerate() {
            let severity_icon = match cycle.severity {
                CycleSeverity::Critical => "🚨",
                CycleSeverity::High => "🔴",
                CycleSeverity::Medium => "🟡",
                CycleSeverity::Low => "🟢",
            };

            report.push_str(&format!("### {} Cycle #{}\n", severity_icon, i + 1));
            report.push_str(&format!("**Modules**: {}\n", cycle.modules.join(" → ")));
            report.push_str(&format!("**Severity**: {:?}\n", cycle.severity));
            report.push_str(&format!("**Suggested Fix**: {}\n\n", cycle.suggested_fix));
        }

        report
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = DependencyAnalyzer::new();
    let cycles = analyzer.analyze_project(Path::new("."))?;

    // Generate and save report
    let report = analyzer.generate_report(&cycles);
    fs::write("circular_dependency_report.md", &report)?;

    println!("📝 Report saved to circular_dependency_report.md");
    println!("\n{}", report);

    Ok(())
}
