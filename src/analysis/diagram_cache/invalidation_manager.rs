//! Cache Invalidation Management System
//!
//! Manages intelligent cache invalidation strategies based on change impact analysis
//! and dependency relationships to maintain cache coherency while maximizing hit rates.

use super::{DiagramCacheError, Result};
use crate::analysis::incremental::ChangeSet;
use crate::core::logging::{debug, info, warn};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Cache invalidation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    /// Invalidate based on direct dependencies only
    DirectDependency,

    /// Invalidate based on dependency graph propagation
    DependencyBased,

    /// Time-based invalidation with dependency awareness
    TimeBased { ttl_hours: u32 },

    /// Adaptive invalidation based on change patterns
    Adaptive,

    /// Conservative invalidation (invalidate more to ensure correctness)
    Conservative,

    /// Aggressive caching (invalidate less for better performance)
    Aggressive,
}

/// Invalidation decision with reasoning
#[derive(Debug, Clone)]
pub struct InvalidationDecision {
    /// Whether to invalidate the diagram
    pub should_invalidate: bool,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,

    /// Reasoning for the decision
    pub reasoning: String,

    /// Priority level (higher = more urgent)
    pub priority: u8,
}

/// Tracks invalidation patterns for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationPattern {
    /// File path that triggered invalidation
    pub trigger_file: PathBuf,

    /// Diagrams that were invalidated
    pub invalidated_diagrams: Vec<String>,

    /// Time of invalidation
    pub timestamp: DateTime<Utc>,

    /// Whether the invalidation was correct (determined later)
    pub was_correct: Option<bool>,

    /// Time taken to regenerate affected diagrams
    pub regeneration_time_ms: Option<u64>,
}

/// Manages cache invalidation decisions and strategies
pub struct InvalidationManager {
    /// Current invalidation strategy
    strategy: InvalidationStrategy,

    /// History of invalidation patterns for learning
    invalidation_history: Vec<InvalidationPattern>,

    /// File change frequency tracking
    file_change_frequency: HashMap<PathBuf, u32>,

    /// Diagram regeneration costs (in milliseconds)
    regeneration_costs: HashMap<String, u64>,

    /// Last invalidation time per diagram
    last_invalidation: HashMap<String, DateTime<Utc>>,
}

impl InvalidationManager {
    /// Creates a new invalidation manager
    pub fn new(strategy: InvalidationStrategy) -> Self {
        Self {
            strategy,
            invalidation_history: Vec::new(),
            file_change_frequency: HashMap::new(),
            regeneration_costs: HashMap::new(),
            last_invalidation: HashMap::new(),
        }
    }

    /// Decides whether to invalidate a diagram based on changes
    pub async fn should_invalidate_diagram(
        &self,
        diagram_id: &str,
        changeset: &ChangeSet,
        diagram_dependencies: &HashSet<PathBuf>,
    ) -> Result<InvalidationDecision> {
        match &self.strategy {
            InvalidationStrategy::DirectDependency => {
                self.direct_dependency_decision(diagram_id, changeset, diagram_dependencies)
                    .await
            }
            InvalidationStrategy::DependencyBased => {
                self.dependency_based_decision(diagram_id, changeset, diagram_dependencies)
                    .await
            }
            InvalidationStrategy::TimeBased { ttl_hours } => {
                self.time_based_decision(diagram_id, *ttl_hours).await
            }
            InvalidationStrategy::Adaptive => {
                self.adaptive_decision(diagram_id, changeset, diagram_dependencies)
                    .await
            }
            InvalidationStrategy::Conservative => {
                self.conservative_decision(diagram_id, changeset, diagram_dependencies)
                    .await
            }
            InvalidationStrategy::Aggressive => {
                self.aggressive_decision(diagram_id, changeset, diagram_dependencies)
                    .await
            }
        }
    }

    /// Records an invalidation pattern for learning
    pub fn record_invalidation_pattern(&mut self, pattern: InvalidationPattern) {
        self.invalidation_history.push(pattern);

        // Keep only recent history (last 1000 patterns)
        if self.invalidation_history.len() > 1000 {
            self.invalidation_history.remove(0);
        }
    }

    /// Updates file change frequency tracking
    pub fn update_file_change_frequency(&mut self, files: &HashSet<PathBuf>) {
        for file in files {
            *self.file_change_frequency.entry(file.clone()).or_insert(0) += 1;
        }
    }

    /// Records diagram regeneration cost
    pub fn record_regeneration_cost(&mut self, diagram_id: &str, cost_ms: u64) {
        self.regeneration_costs
            .insert(diagram_id.to_string(), cost_ms);
    }

    /// Gets invalidation statistics
    pub fn get_invalidation_stats(&self) -> InvalidationStats {
        let total_invalidations = self.invalidation_history.len();
        let correct_invalidations = self
            .invalidation_history
            .iter()
            .filter(|p| p.was_correct == Some(true))
            .count();

        let accuracy = if total_invalidations > 0 {
            correct_invalidations as f64 / total_invalidations as f64
        } else {
            0.0
        };

        let avg_regeneration_time = if !self.regeneration_costs.is_empty() {
            self.regeneration_costs.values().sum::<u64>() as f64
                / self.regeneration_costs.len() as f64
        } else {
            0.0
        };

        let most_changed_files = {
            let mut files: Vec<_> = self.file_change_frequency.iter().collect();
            files.sort_by(|a, b| b.1.cmp(a.1));
            files
                .into_iter()
                .take(10)
                .map(|(f, c)| (f.clone(), *c))
                .collect()
        };

        InvalidationStats {
            total_invalidations,
            correct_invalidations,
            accuracy,
            average_regeneration_time_ms: avg_regeneration_time,
            most_changed_files,
        }
    }

    /// Optimizes invalidation strategy based on historical data
    pub async fn optimize_strategy(&mut self) -> Result<()> {
        let stats = self.get_invalidation_stats();

        // If accuracy is low, switch to more conservative strategy
        if stats.accuracy < 0.7 {
            warn!(
                "Low invalidation accuracy ({:.1}%), switching to conservative strategy",
                stats.accuracy * 100.0
            );
            self.strategy = InvalidationStrategy::Conservative;
        }
        // If accuracy is very high, we might be too conservative
        else if stats.accuracy > 0.95 && stats.average_regeneration_time_ms < 100.0 {
            info!(
                "High invalidation accuracy ({:.1}%), switching to aggressive strategy",
                stats.accuracy * 100.0
            );
            self.strategy = InvalidationStrategy::Aggressive;
        }

        Ok(())
    }

    /// Direct dependency invalidation decision
    async fn direct_dependency_decision(
        &self,
        _diagram_id: &str,
        changeset: &ChangeSet,
        diagram_dependencies: &HashSet<PathBuf>,
    ) -> Result<InvalidationDecision> {
        let has_direct_change = diagram_dependencies.iter().any(|dep| {
            changeset.modified.contains(dep)
                || changeset.added.contains(dep)
                || changeset.deleted.contains(dep)
        });

        Ok(InvalidationDecision {
            should_invalidate: has_direct_change,
            confidence: if has_direct_change { 1.0 } else { 0.0 },
            reasoning: if has_direct_change {
                "Direct dependency changed".to_string()
            } else {
                "No direct dependencies changed".to_string()
            },
            priority: if has_direct_change { 10 } else { 0 },
        })
    }

    /// Dependency-based invalidation decision
    async fn dependency_based_decision(
        &self,
        diagram_id: &str,
        changeset: &ChangeSet,
        diagram_dependencies: &HashSet<PathBuf>,
    ) -> Result<InvalidationDecision> {
        // Check direct dependencies
        let direct_change = self
            .direct_dependency_decision(diagram_id, changeset, diagram_dependencies)
            .await?;

        if direct_change.should_invalidate {
            return Ok(direct_change);
        }

        // Check indirect dependencies through affected_by_dependencies
        let has_indirect_change = diagram_dependencies
            .iter()
            .any(|dep| changeset.affected_by_dependencies.contains(dep));

        Ok(InvalidationDecision {
            should_invalidate: has_indirect_change,
            confidence: if has_indirect_change { 0.8 } else { 0.0 },
            reasoning: if has_indirect_change {
                "Indirect dependency changed".to_string()
            } else {
                "No dependencies changed".to_string()
            },
            priority: if has_indirect_change { 7 } else { 0 },
        })
    }

    /// Time-based invalidation decision
    async fn time_based_decision(
        &self,
        diagram_id: &str,
        ttl_hours: u32,
    ) -> Result<InvalidationDecision> {
        if let Some(last_invalidation) = self.last_invalidation.get(diagram_id) {
            let age = Utc::now() - *last_invalidation;
            let ttl = Duration::hours(ttl_hours as i64);

            let should_invalidate = age > ttl;
            let confidence = if should_invalidate { 0.9 } else { 0.1 };

            Ok(InvalidationDecision {
                should_invalidate,
                confidence,
                reasoning: format!("Diagram age: {}h, TTL: {}h", age.num_hours(), ttl_hours),
                priority: if should_invalidate { 5 } else { 1 },
            })
        } else {
            // No previous invalidation recorded, assume it's old
            Ok(InvalidationDecision {
                should_invalidate: true,
                confidence: 0.5,
                reasoning: "No previous invalidation recorded".to_string(),
                priority: 3,
            })
        }
    }

    /// Adaptive invalidation decision based on patterns
    async fn adaptive_decision(
        &self,
        diagram_id: &str,
        changeset: &ChangeSet,
        diagram_dependencies: &HashSet<PathBuf>,
    ) -> Result<InvalidationDecision> {
        // Start with dependency-based decision
        let base_decision = self
            .dependency_based_decision(diagram_id, changeset, diagram_dependencies)
            .await?;

        // Adjust based on historical patterns
        let change_frequency = diagram_dependencies
            .iter()
            .map(|dep| self.file_change_frequency.get(dep).unwrap_or(&0))
            .max()
            .unwrap_or(&0);

        let regeneration_cost = self.regeneration_costs.get(diagram_id).unwrap_or(&100);

        // High-frequency changes and low regeneration cost = more aggressive invalidation
        // Low-frequency changes and high regeneration cost = more conservative
        let frequency_factor = (*change_frequency as f64).min(10.0) / 10.0;
        let cost_factor = 1.0 - ((*regeneration_cost as f64).min(1000.0) / 1000.0);

        let adjusted_confidence =
            base_decision.confidence * (0.5 + 0.5 * frequency_factor * cost_factor);

        Ok(InvalidationDecision {
            should_invalidate: adjusted_confidence > 0.5,
            confidence: adjusted_confidence,
            reasoning: format!(
                "Adaptive: base={:.2}, freq={}, cost={}ms",
                base_decision.confidence, change_frequency, regeneration_cost
            ),
            priority: base_decision.priority,
        })
    }

    /// Conservative invalidation decision
    async fn conservative_decision(
        &self,
        diagram_id: &str,
        changeset: &ChangeSet,
        diagram_dependencies: &HashSet<PathBuf>,
    ) -> Result<InvalidationDecision> {
        let base_decision = self
            .dependency_based_decision(diagram_id, changeset, diagram_dependencies)
            .await?;

        // Be more aggressive about invalidation (lower threshold)
        let should_invalidate = base_decision.confidence > 0.3 || !changeset.modified.is_empty();

        Ok(InvalidationDecision {
            should_invalidate,
            confidence: if should_invalidate { 0.9 } else { 0.1 },
            reasoning: format!("Conservative: {}", base_decision.reasoning),
            priority: base_decision.priority + 2,
        })
    }

    /// Aggressive caching decision (invalidate less)
    async fn aggressive_decision(
        &self,
        diagram_id: &str,
        changeset: &ChangeSet,
        diagram_dependencies: &HashSet<PathBuf>,
    ) -> Result<InvalidationDecision> {
        let base_decision = self
            .dependency_based_decision(diagram_id, changeset, diagram_dependencies)
            .await?;

        // Be more conservative about invalidation (higher threshold)
        let should_invalidate = base_decision.confidence > 0.8;

        Ok(InvalidationDecision {
            should_invalidate,
            confidence: base_decision.confidence,
            reasoning: format!("Aggressive: {}", base_decision.reasoning),
            priority: if base_decision.priority > 0 {
                base_decision.priority - 1
            } else {
                0
            },
        })
    }
}

/// Statistics about invalidation performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationStats {
    /// Total number of invalidations performed
    pub total_invalidations: usize,

    /// Number of correct invalidations
    pub correct_invalidations: usize,

    /// Invalidation accuracy (0.0 - 1.0)
    pub accuracy: f64,

    /// Average time to regenerate diagrams
    pub average_regeneration_time_ms: f64,

    /// Files that change most frequently
    pub most_changed_files: Vec<(PathBuf, u32)>,
}
