//! Diagram Dependency Tracking System
//!
//! Tracks relationships between code files and diagrams to enable intelligent
//! cache invalidation and selective regeneration based on change impact analysis.

use super::{DiagramCacheError, DiagramType, Result};
use crate::analysis::incremental::ChangeSet;
use crate::core::logging::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Represents a dependency relationship between a diagram and source files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramDependency {
    /// Diagram identifier
    pub diagram_id: String,

    /// Type of diagram
    pub diagram_type: DiagramType,

    /// Source files this diagram depends on
    pub source_files: HashSet<PathBuf>,

    /// Other diagrams this diagram depends on
    pub diagram_dependencies: HashSet<String>,

    /// Dependency strength (0.0 - 1.0, higher = stronger dependency)
    pub dependency_strength: f64,

    /// Last time dependencies were updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Tracks diagram dependencies for intelligent cache management
pub struct DiagramDependencyTracker {
    /// Map from diagram ID to its dependencies
    diagram_dependencies: RwLock<HashMap<String, DiagramDependency>>,

    /// Reverse mapping: source file to diagrams that depend on it
    file_to_diagrams: RwLock<HashMap<PathBuf, HashSet<String>>>,

    /// Diagram-to-diagram dependency graph
    diagram_graph: RwLock<HashMap<String, HashSet<String>>>,
}

impl DiagramDependencyTracker {
    /// Creates a new diagram dependency tracker
    pub fn new() -> Self {
        Self {
            diagram_dependencies: RwLock::new(HashMap::new()),
            file_to_diagrams: RwLock::new(HashMap::new()),
            diagram_graph: RwLock::new(HashMap::new()),
        }
    }

    /// Adds dependencies for a diagram
    pub async fn add_diagram_dependencies(
        &self,
        diagram_id: &str,
        diagram_type: &DiagramType,
        source_files: &[PathBuf],
    ) -> Result<()> {
        let dependency = DiagramDependency {
            diagram_id: diagram_id.to_string(),
            diagram_type: diagram_type.clone(),
            source_files: source_files.iter().cloned().collect(),
            diagram_dependencies: HashSet::new(),
            dependency_strength: 1.0, // Default full dependency
            last_updated: chrono::Utc::now(),
        };

        // Update diagram dependencies
        {
            let mut deps = self.diagram_dependencies.write().await;
            deps.insert(diagram_id.to_string(), dependency);
        }

        // Update reverse mapping
        {
            let mut file_map = self.file_to_diagrams.write().await;
            for file in source_files {
                file_map
                    .entry(file.clone())
                    .or_insert_with(HashSet::new)
                    .insert(diagram_id.to_string());
            }
        }

        debug!(
            "Added dependencies for diagram: {} (files: {})",
            diagram_id,
            source_files.len()
        );
        Ok(())
    }

    /// Gets diagrams affected by a changeset
    pub async fn get_affected_diagrams(&self, changeset: &ChangeSet) -> Result<HashSet<String>> {
        let mut affected_diagrams = HashSet::new();
        let file_map = self.file_to_diagrams.read().await;

        // Check modified files
        for modified_file in &changeset.modified {
            if let Some(diagrams) = file_map.get(modified_file) {
                affected_diagrams.extend(diagrams.clone());
            }
        }

        // Check added files (may affect existing diagrams)
        for added_file in &changeset.added {
            if let Some(diagrams) = file_map.get(added_file) {
                affected_diagrams.extend(diagrams.clone());
            }
        }

        // Check deleted files
        for deleted_file in &changeset.deleted {
            if let Some(diagrams) = file_map.get(deleted_file) {
                affected_diagrams.extend(diagrams.clone());
            }
        }

        // Check dependency-affected files
        for dep_affected_file in &changeset.affected_by_dependencies {
            if let Some(diagrams) = file_map.get(dep_affected_file) {
                affected_diagrams.extend(diagrams.clone());
            }
        }

        // Propagate through diagram dependencies
        let propagated_diagrams = self
            .propagate_diagram_dependencies(&affected_diagrams)
            .await?;
        affected_diagrams.extend(propagated_diagrams);

        info!(
            "Found {} diagrams affected by changeset",
            affected_diagrams.len()
        );
        Ok(affected_diagrams)
    }

    /// Gets diagrams that need regeneration based on changes
    pub async fn get_diagrams_for_regeneration(
        &self,
        changeset: &ChangeSet,
    ) -> Result<Vec<String>> {
        let affected_diagrams = self.get_affected_diagrams(changeset).await?;

        // Filter diagrams based on dependency strength and change impact
        let mut diagrams_to_regenerate = Vec::new();
        let deps = self.diagram_dependencies.read().await;

        for diagram_id in affected_diagrams {
            if let Some(dependency) = deps.get(&diagram_id) {
                // Check if any of the changed files have strong dependencies
                let has_strong_dependency = dependency.source_files.iter().any(|file| {
                    changeset.modified.contains(file)
                        || changeset.added.contains(file)
                        || changeset.deleted.contains(file)
                });

                if has_strong_dependency || dependency.dependency_strength > 0.7 {
                    diagrams_to_regenerate.push(diagram_id);
                }
            }
        }

        // Sort by dependency strength (strongest first)
        diagrams_to_regenerate.sort_by(|a, b| {
            let strength_a = deps.get(a).map(|d| d.dependency_strength).unwrap_or(0.0);
            let strength_b = deps.get(b).map(|d| d.dependency_strength).unwrap_or(0.0);
            strength_b
                .partial_cmp(&strength_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        info!(
            "Identified {} diagrams for regeneration",
            diagrams_to_regenerate.len()
        );
        Ok(diagrams_to_regenerate)
    }

    /// Removes dependencies for deleted diagrams
    pub async fn remove_affected_dependencies(&self, diagram_ids: &HashSet<String>) -> Result<()> {
        // Remove from main dependency map
        {
            let mut deps = self.diagram_dependencies.write().await;
            for diagram_id in diagram_ids {
                deps.remove(diagram_id);
            }
        }

        // Clean up reverse mapping
        {
            let mut file_map = self.file_to_diagrams.write().await;
            for diagrams in file_map.values_mut() {
                for diagram_id in diagram_ids {
                    diagrams.remove(diagram_id);
                }
            }

            // Remove empty entries
            file_map.retain(|_, diagrams| !diagrams.is_empty());
        }

        // Clean up diagram graph
        {
            let mut graph = self.diagram_graph.write().await;
            for diagram_id in diagram_ids {
                graph.remove(diagram_id);
            }

            // Remove references to deleted diagrams
            for deps in graph.values_mut() {
                for diagram_id in diagram_ids {
                    deps.remove(diagram_id);
                }
            }
        }

        debug!("Removed dependencies for {} diagrams", diagram_ids.len());
        Ok(())
    }

    /// Adds diagram-to-diagram dependency
    pub async fn add_diagram_dependency(&self, from_diagram: &str, to_diagram: &str) -> Result<()> {
        let mut graph = self.diagram_graph.write().await;
        graph
            .entry(from_diagram.to_string())
            .or_insert_with(HashSet::new)
            .insert(to_diagram.to_string());

        // Update the dependency object
        let mut deps = self.diagram_dependencies.write().await;
        if let Some(dependency) = deps.get_mut(from_diagram) {
            dependency
                .diagram_dependencies
                .insert(to_diagram.to_string());
            dependency.last_updated = chrono::Utc::now();
        }

        debug!(
            "Added diagram dependency: {} -> {}",
            from_diagram, to_diagram
        );
        Ok(())
    }

    /// Gets all dependencies for a diagram
    pub async fn get_diagram_dependencies(
        &self,
        diagram_id: &str,
    ) -> Result<Option<DiagramDependency>> {
        let deps = self.diagram_dependencies.read().await;
        Ok(deps.get(diagram_id).cloned())
    }

    /// Updates dependency strength based on usage patterns
    pub async fn update_dependency_strength(
        &self,
        diagram_id: &str,
        new_strength: f64,
    ) -> Result<()> {
        let mut deps = self.diagram_dependencies.write().await;
        if let Some(dependency) = deps.get_mut(diagram_id) {
            dependency.dependency_strength = new_strength.clamp(0.0, 1.0);
            dependency.last_updated = chrono::Utc::now();
            debug!(
                "Updated dependency strength for {}: {:.2}",
                diagram_id, new_strength
            );
        }
        Ok(())
    }

    /// Analyzes dependency patterns for optimization
    pub async fn analyze_dependency_patterns(&self) -> Result<DependencyAnalysis> {
        let deps = self.diagram_dependencies.read().await;
        let file_map = self.file_to_diagrams.read().await;

        let total_diagrams = deps.len();
        let total_files = file_map.len();

        // Calculate average dependencies per diagram
        let total_deps: usize = deps.values().map(|d| d.source_files.len()).sum();
        let avg_deps_per_diagram = if total_diagrams > 0 {
            total_deps as f64 / total_diagrams as f64
        } else {
            0.0
        };

        // Find hot files (files that many diagrams depend on)
        let mut hot_files = Vec::new();
        for (file, diagrams) in file_map.iter() {
            if diagrams.len() > 5 {
                // Threshold for "hot" files
                hot_files.push((file.clone(), diagrams.len()));
            }
        }
        hot_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Calculate dependency strength distribution
        let strengths: Vec<f64> = deps.values().map(|d| d.dependency_strength).collect();
        let avg_strength = if !strengths.is_empty() {
            strengths.iter().sum::<f64>() / strengths.len() as f64
        } else {
            0.0
        };

        Ok(DependencyAnalysis {
            total_diagrams,
            total_files,
            avg_dependencies_per_diagram: avg_deps_per_diagram,
            hot_files: hot_files.into_iter().take(10).collect(), // Top 10 hot files
            average_dependency_strength: avg_strength,
        })
    }

    /// Clears all dependency tracking data
    pub async fn clear(&self) -> Result<()> {
        {
            let mut deps = self.diagram_dependencies.write().await;
            deps.clear();
        }

        {
            let mut file_map = self.file_to_diagrams.write().await;
            file_map.clear();
        }

        {
            let mut graph = self.diagram_graph.write().await;
            graph.clear();
        }

        info!("Cleared all diagram dependency tracking data");
        Ok(())
    }

    /// Propagates dependencies through the diagram graph
    async fn propagate_diagram_dependencies(
        &self,
        initial_diagrams: &HashSet<String>,
    ) -> Result<HashSet<String>> {
        let mut propagated = HashSet::new();
        let mut to_process: Vec<String> = initial_diagrams.iter().cloned().collect();
        let graph = self.diagram_graph.read().await;

        while let Some(diagram_id) = to_process.pop() {
            if propagated.contains(&diagram_id) {
                continue;
            }

            propagated.insert(diagram_id.clone());

            // Add dependent diagrams to processing queue
            if let Some(dependents) = graph.get(&diagram_id) {
                for dependent in dependents {
                    if !propagated.contains(dependent) {
                        to_process.push(dependent.clone());
                    }
                }
            }
        }

        // Remove initial diagrams from result (we only want propagated ones)
        for initial in initial_diagrams {
            propagated.remove(initial);
        }

        Ok(propagated)
    }
}

impl Default for DiagramDependencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis results for dependency patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// Total number of tracked diagrams
    pub total_diagrams: usize,

    /// Total number of tracked files
    pub total_files: usize,

    /// Average number of dependencies per diagram
    pub avg_dependencies_per_diagram: f64,

    /// Files that many diagrams depend on (file, diagram_count)
    pub hot_files: Vec<(PathBuf, usize)>,

    /// Average dependency strength
    pub average_dependency_strength: f64,
}
