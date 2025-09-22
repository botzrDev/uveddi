use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::config::*;
use super::{DependencyAnalyzer, AnalysisOutput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveAnalysisResult {
    pub transitive_dependencies: HashMap<String, Vec<TransitiveDependency>>,
    pub depth_analysis: DepthAnalysis,
    pub hidden_dependencies: Vec<HiddenDependency>,
    pub transitive_vulnerabilities: Vec<TransitiveVulnerability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveDependency {
    pub name: String,
    pub version: String,
    pub depth: usize,
    pub path: Vec<String>,
    pub is_dev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthAnalysis {
    pub max_depth: usize,
    pub avg_depth: f64,
    pub depth_distribution: HashMap<usize, usize>,
    pub deep_chains: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenDependency {
    pub name: String,
    pub introduced_by: Vec<String>,
    pub risk_level: RiskLevel,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveVulnerability {
    pub package: String,
    pub vulnerability: Vulnerability,
    pub exposure_path: Vec<String>,
    pub mitigation: String,
}

pub struct TransitiveAnalyzer {
    dependency_tree: HashMap<String, HashSet<String>>,
    visited: HashSet<String>,
}

impl TransitiveAnalyzer {
    pub fn new() -> Self {
        Self {
            dependency_tree: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    fn analyze_transitive(&mut self, dependencies: &[DependencyInfo]) -> TransitiveAnalysisResult {
        self.build_dependency_tree(dependencies);
        
        let transitive_deps = self.find_transitive_dependencies(dependencies);
        let depth_analysis = self.analyze_depth(&transitive_deps);
        let hidden = self.find_hidden_dependencies(&transitive_deps);
        let vulnerabilities = self.check_transitive_vulnerabilities(&transitive_deps);

        TransitiveAnalysisResult {
            transitive_dependencies: transitive_deps,
            depth_analysis,
            hidden_dependencies: hidden,
            transitive_vulnerabilities: vulnerabilities,
        }
    }

    fn build_dependency_tree(&mut self, dependencies: &[DependencyInfo]) {
        for dep in dependencies {
            self.dependency_tree.entry(dep.name.clone())
                .or_insert_with(HashSet::new);
            
            // Simulate transitive dependencies
            for other in dependencies {
                if dep.name != other.name && self.is_likely_dependency(&dep.name, &other.name) {
                    self.dependency_tree.get_mut(&dep.name)
                        .unwrap()
                        .insert(other.name.clone());
                }
            }
        }
    }

    fn is_likely_dependency(&self, parent: &str, child: &str) -> bool {
        // Simple heuristic based on naming patterns
        parent.len() < child.len() && 
        (child.starts_with(&parent[..parent.len().min(3)]) ||
         parent.contains("core") && !child.contains("test"))
    }

    fn find_transitive_dependencies(
        &mut self,
        dependencies: &[DependencyInfo]
    ) -> HashMap<String, Vec<TransitiveDependency>> {
        let mut result = HashMap::new();
        
        for dep in dependencies {
            let mut path = Vec::new();
            let mut transitive = Vec::new();
            self.visited.clear();
            
            self.dfs_transitive(
                &dep.name,
                0,
                &mut path,
                &mut transitive,
                dependencies
            );
            
            if !transitive.is_empty() {
                result.insert(dep.name.clone(), transitive);
            }
        }
        
        result
    }

    fn dfs_transitive(
        &mut self,
        node: &str,
        depth: usize,
        path: &mut Vec<String>,
        transitive: &mut Vec<TransitiveDependency>,
        all_deps: &[DependencyInfo]
    ) {
        if self.visited.contains(node) || depth > 10 {
            return;
        }
        
        self.visited.insert(node.to_string());
        path.push(node.to_string());
        
        if depth > 0 {
            let version = all_deps.iter()
                .find(|d| d.name == node)
                .and_then(|d| d.version.clone())
                .unwrap_or_else(|| "unknown".to_string());
            
            transitive.push(TransitiveDependency {
                name: node.to_string(),
                version,
                depth,
                path: path.clone(),
                is_dev: node.contains("test") || node.contains("dev"),
            });
        }
        
        if let Some(children) = self.dependency_tree.get(node) {
            for child in children {
                self.dfs_transitive(child, depth + 1, path, transitive, all_deps);
            }
        }
        
        path.pop();
    }

    fn analyze_depth(
        &self,
        transitive_deps: &HashMap<String, Vec<TransitiveDependency>>
    ) -> DepthAnalysis {
        let mut max_depth = 0;
        let mut total_depth = 0;
        let mut count = 0;
        let mut depth_distribution = HashMap::new();
        let mut deep_chains = Vec::new();
        
        for deps in transitive_deps.values() {
            for dep in deps {
                max_depth = max_depth.max(dep.depth);
                total_depth += dep.depth;
                count += 1;
                *depth_distribution.entry(dep.depth).or_insert(0) += 1;
                
                if dep.depth > 5 {
                    deep_chains.push(dep.path.clone());
                }
            }
        }
        
        let avg_depth = if count > 0 {
            total_depth as f64 / count as f64
        } else {
            0.0
        };
        
        DepthAnalysis {
            max_depth,
            avg_depth,
            depth_distribution,
            deep_chains,
        }
    }

    fn find_hidden_dependencies(
        &self,
        transitive_deps: &HashMap<String, Vec<TransitiveDependency>>
    ) -> Vec<HiddenDependency> {
        let mut hidden = Vec::new();
        let mut seen_at_depth = HashMap::new();
        
        for (parent, deps) in transitive_deps {
            for dep in deps {
                if dep.depth > 2 {
                    seen_at_depth.entry(dep.name.clone())
                        .or_insert_with(Vec::new)
                        .push((parent.clone(), dep.depth));
                }
            }
        }
        
        for (name, occurrences) in seen_at_depth {
            if occurrences.len() > 1 {
                let introduced_by: Vec<String> = occurrences.iter()
                    .map(|(p, _)| p.clone())
                    .collect();
                
                let max_depth = occurrences.iter().map(|(_, d)| *d).max().unwrap_or(0);
                let risk_level = if max_depth > 5 {
                    RiskLevel::High
                } else if max_depth > 3 {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                };
                
                hidden.push(HiddenDependency {
                    name,
                    introduced_by,
                    risk_level,
                    reason: format!("Deeply nested at depth {}", max_depth),
                });
            }
        }
        
        hidden
    }

    fn check_transitive_vulnerabilities(
        &self,
        transitive_deps: &HashMap<String, Vec<TransitiveDependency>>
    ) -> Vec<TransitiveVulnerability> {
        let mut vulnerabilities = Vec::new();
        
        for (_, deps) in transitive_deps {
            for dep in deps {
                // Simulate vulnerability detection
                if dep.name.contains("log4") || dep.name.contains("jackson") {
                    vulnerabilities.push(TransitiveVulnerability {
                        package: dep.name.clone(),
                        vulnerability: Vulnerability {
                            id: "CVE-2021-44228".to_string(),
                            severity: VulnerabilitySeverity::Critical,
                            cve_id: Some("CVE-2021-44228".to_string()),
                            description: "Remote code execution vulnerability".to_string(),
                            affected_versions: vec![dep.version.clone()],
                            fixed_versions: vec!["2.17.0".to_string()],
                            published_date: Some("2021-12-10".to_string()),
                            references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2021-44228".to_string()],
                        },
                        exposure_path: dep.path.clone(),
                        mitigation: "Upgrade to version 2.17.0 or later".to_string(),
                    });
                }
            }
        }
        
        vulnerabilities
    }
}

impl DependencyAnalyzer for TransitiveAnalyzer {
    fn analyze(
        &self,
        dependencies: &[DependencyInfo],
        _config: &DependencyDetectorConfig,
    ) -> Result<AnalysisOutput, DependencyError> {
        let mut analyzer = Self::new();
        let analysis = analyzer.analyze_transitive(dependencies);
        Ok(AnalysisOutput::Transitive(analysis))
    }

    fn name(&self) -> &str {
        "TransitiveAnalyzer"
    }

    fn description(&self) -> &str {
        "Analyzes transitive dependencies and their impacts"
    }
}