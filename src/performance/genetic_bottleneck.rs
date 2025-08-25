//! Genetic Algorithm Bottleneck Detection for UV-249 Phase 3
//!
//! Implements multi-objective genetic algorithm optimization for automated
//! bottleneck identification and performance optimization recommendations.

use anyhow::{anyhow, Result};
use rand::prelude::*;
use rand::{SeedableRng, rngs::StdRng, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::monitoring::PerformanceMetricsCollector;
use crate::performance::statistical_analysis::StatisticalAnalyzer;

/// Helper struct for resource statistics
#[derive(Debug, Clone)]
struct ResourceStats {
    mean: f64,
    peak: f64,
    variance: f64,
}

/// Genetic algorithm bottleneck detector
#[derive(Debug)]
pub struct GeneticBottleneckDetector {
    population_size: usize,
    generations: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    elitism_rate: f64,
    fitness_evaluator: FitnessEvaluator,
    rng: StdRng,
}

/// Chromosome representing bottleneck detection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckChromosome {
    /// Resource weights for CPU, Memory, I/O, Network
    pub resource_weights: Vec<f64>,
    /// Detection thresholds for each resource type
    pub threshold_values: Vec<f64>,
    /// Optimization targets and their priorities
    pub optimization_targets: Vec<OptimizationTarget>,
    /// Fitness score (calculated during evaluation)
    pub fitness: f64,
}

/// Optimization target with weight and direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTarget {
    pub objective: OptimizationObjective,
    pub weight: f64,
    pub target_direction: TargetDirection,
}

/// Multi-objective optimization objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationObjective {
    ExecutionTime,
    MemoryUsage,
    IOWait,
    CPUUtilization,
    NetworkLatency,
    Throughput,
    ResourceContention,
}

/// Target optimization direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetDirection {
    Minimize,
    Maximize,
}

/// Identified bottleneck with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub resource_type: ResourceType,
    pub severity: BottleneckSeverity,
    pub confidence: f64,
    pub impact_score: f64,
    pub location: BottleneckLocation,
    pub metrics: BottleneckMetrics,
}

/// Resource type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    CPU,
    Memory,
    IO,
    Network,
    Database,
    Cache,
}

/// Bottleneck severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Critical, // >80% impact
    High,     // 60-80% impact
    Medium,   // 40-60% impact
    Low,      // 20-40% impact
    Minimal,  // <20% impact
}

/// Location information for bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckLocation {
    pub component: String,
    pub function: Option<String>,
    pub line_number: Option<usize>,
    pub file_path: Option<String>,
}

/// Detailed metrics for bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckMetrics {
    pub current_utilization: f64,
    pub baseline_utilization: f64,
    pub peak_utilization: f64,
    pub average_utilization: f64,
    pub variance: f64,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_impact: f64,
    pub implementation_effort: ImplementationEffort,
    pub category: RecommendationCategory,
    pub specific_actions: Vec<String>,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Immediate, // Critical bottlenecks
    High,      // High impact, low effort
    Medium,    // Moderate impact/effort
    Low,       // Low impact or high effort
}

/// Implementation effort estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,   // <1 day
    Low,       // 1-3 days
    Medium,    // 1-2 weeks
    High,      // 2-4 weeks
    Extensive, // >1 month
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    CodeOptimization,
    ResourceScaling,
    ArchitecturalChange,
    ConfigurationTuning,
    CachingStrategy,
    DatabaseOptimization,
}

/// Complete bottleneck analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    pub identified_bottlenecks: Vec<Bottleneck>,
    pub optimization_recommendations: Vec<Recommendation>,
    pub overall_confidence: f64,
    pub performance_score: f64,
    pub genetic_generations: usize,
    pub convergence_achieved: bool,
    pub analysis_metadata: AnalysisMetadata,
}

/// Analysis metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analysis_duration_ms: u64,
    pub population_size: usize,
    pub final_generation: usize,
    pub best_fitness: f64,
    pub convergence_generation: Option<usize>,
    pub metrics_analyzed: usize,
}

/// Fitness evaluator for multi-objective optimization
#[derive(Debug)]
pub struct FitnessEvaluator {
    objectives: Vec<OptimizationObjective>,
    weights: Vec<f64>,
    baseline_metrics: HashMap<String, f64>,
}

impl GeneticBottleneckDetector {
    /// Create new genetic bottleneck detector with default parameters
    pub fn new() -> Self {
        Self {
            population_size: 50,
            generations: 100,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            elitism_rate: 0.1,
            fitness_evaluator: FitnessEvaluator::new(),
            rng: StdRng::from_rng(&mut thread_rng()),
        }
    }

    /// Configure population size
    pub fn with_population_size(mut self, size: usize) -> Self {
        self.population_size = size;
        self
    }

    /// Configure number of generations
    pub fn with_generations(mut self, generations: usize) -> Self {
        self.generations = generations;
        self
    }

    /// Configure mutation rate (0.0-1.0)
    pub fn with_mutation_rate(mut self, rate: f64) -> Self {
        self.mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Configure crossover rate (0.0-1.0)
    pub fn with_crossover_rate(mut self, rate: f64) -> Self {
        self.crossover_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Configure elitism rate (0.0-1.0)
    pub fn with_elitism_rate(mut self, rate: f64) -> Self {
        self.elitism_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Main genetic algorithm evolution for bottleneck detection
    pub async fn evolve_bottleneck_detection(
        &mut self,
        performance_data: &[PerformanceDataPoint],
    ) -> Result<BottleneckAnalysis> {
        let start_time = std::time::Instant::now();

        // Initialize population
        let mut population = self.initialize_population()?;

        // Evaluate initial fitness
        self.evaluate_population(&mut population, performance_data)?;

        let mut best_fitness = 0.0;
        let mut convergence_generation = None;
        let mut generations_without_improvement = 0;

        // Evolution loop
        for generation in 0..self.generations {
            // Selection
            let parents = self.selection(&population)?;

            // Crossover and mutation
            let mut offspring = self.crossover_and_mutation(&parents)?;

            // Evaluate offspring
            self.evaluate_population(&mut offspring, performance_data)?;

            // Combine and select next generation
            population = self.survivor_selection(&population, &offspring)?;

            // Check for convergence
            let current_best = population.iter().map(|c| c.fitness).fold(0.0, f64::max);

            if current_best > best_fitness {
                best_fitness = current_best;
                generations_without_improvement = 0;
                if convergence_generation.is_none() && current_best > 0.95 {
                    convergence_generation = Some(generation);
                }
            } else {
                generations_without_improvement += 1;
            }

            // Early termination if converged
            if generations_without_improvement > 20 && current_best > 0.9 {
                break;
            }
        }

        // Extract best solution and generate analysis
        let best_chromosome = population
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .ok_or_else(|| anyhow!("No valid chromosome found"))?;

        let analysis = self
            .generate_bottleneck_analysis(
                best_chromosome,
                performance_data,
                AnalysisMetadata {
                    analysis_duration_ms: start_time.elapsed().as_millis() as u64,
                    population_size: self.population_size,
                    final_generation: self.generations,
                    best_fitness,
                    convergence_generation,
                    metrics_analyzed: performance_data.len(),
                },
            )
            .await?;

        Ok(analysis)
    }

    /// Initialize random population of chromosomes
    fn initialize_population(&mut self) -> Result<Vec<BottleneckChromosome>> {
        let mut population = Vec::with_capacity(self.population_size);

        for _ in 0..self.population_size {
            let chromosome = self.create_random_chromosome()?;
            population.push(chromosome);
        }

        Ok(population)
    }

    /// Create a random chromosome with valid parameters
    fn create_random_chromosome(&mut self) -> Result<BottleneckChromosome> {
        // Resource weights for CPU, Memory, I/O, Network (sum to 1.0)
        let mut resource_weights = vec![
            self.rng.gen::<f64>(),
            self.rng.gen::<f64>(),
            self.rng.gen::<f64>(),
            self.rng.gen::<f64>(),
        ];
        let sum: f64 = resource_weights.iter().sum();
        resource_weights.iter_mut().for_each(|w| *w /= sum);

        // Threshold values (0.1 to 0.9 for each resource)
        let threshold_values = (0..4).map(|_| 0.1 + self.rng.gen::<f64>() * 0.8).collect();

        // Optimization targets
        let optimization_targets = vec![
            OptimizationTarget {
                objective: OptimizationObjective::ExecutionTime,
                weight: self.rng.gen::<f64>(),
                target_direction: TargetDirection::Minimize,
            },
            OptimizationTarget {
                objective: OptimizationObjective::MemoryUsage,
                weight: self.rng.gen::<f64>(),
                target_direction: TargetDirection::Minimize,
            },
            OptimizationTarget {
                objective: OptimizationObjective::Throughput,
                weight: self.rng.gen::<f64>(),
                target_direction: TargetDirection::Maximize,
            },
        ];

        Ok(BottleneckChromosome {
            resource_weights,
            threshold_values,
            optimization_targets,
            fitness: 0.0,
        })
    }

    /// Evaluate fitness for entire population
    fn evaluate_population(
        &self,
        population: &mut [BottleneckChromosome],
        performance_data: &[PerformanceDataPoint],
    ) -> Result<()> {
        for chromosome in population.iter_mut() {
            chromosome.fitness = self
                .fitness_evaluator
                .evaluate_fitness(chromosome, performance_data)?;
        }
        Ok(())
    }

    /// Tournament selection for parent selection
    fn selection(
        &mut self,
        population: &[BottleneckChromosome],
    ) -> Result<Vec<BottleneckChromosome>> {
        let tournament_size = 3;
        let mut parents = Vec::new();

        for _ in 0..population.len() {
            let mut tournament = Vec::new();
            for _ in 0..tournament_size {
                let idx = self.rng.gen_range(0..population.len());
                tournament.push(&population[idx]);
            }

            let winner = tournament
                .iter()
                .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
                .ok_or_else(|| anyhow!("Tournament selection failed"))?;

            parents.push((*winner).clone());
        }

        Ok(parents)
    }

    /// Crossover and mutation operations
    fn crossover_and_mutation(
        &mut self,
        parents: &[BottleneckChromosome],
    ) -> Result<Vec<BottleneckChromosome>> {
        let mut offspring = Vec::new();

        for i in (0..parents.len()).step_by(2) {
            let parent1 = &parents[i];
            let parent2 = &parents[(i + 1) % parents.len()];

            let (mut child1, mut child2) = if self.rng.gen::<f64>() < self.crossover_rate {
                self.crossover(parent1, parent2)?
            } else {
                (parent1.clone(), parent2.clone())
            };

            // Apply mutation
            if self.rng.gen::<f64>() < self.mutation_rate {
                self.mutate(&mut child1)?;
            }
            if self.rng.gen::<f64>() < self.mutation_rate {
                self.mutate(&mut child2)?;
            }

            offspring.push(child1);
            offspring.push(child2);
        }

        Ok(offspring)
    }

    /// Single-point crossover for chromosomes
    fn crossover(
        &mut self,
        parent1: &BottleneckChromosome,
        parent2: &BottleneckChromosome,
    ) -> Result<(BottleneckChromosome, BottleneckChromosome)> {
        let crossover_point = self.rng.gen_range(0..parent1.resource_weights.len());

        let mut child1 = parent1.clone();
        let mut child2 = parent2.clone();

        // Crossover resource weights
        for i in crossover_point..parent1.resource_weights.len() {
            child1.resource_weights[i] = parent2.resource_weights[i];
            child2.resource_weights[i] = parent1.resource_weights[i];
        }

        // Normalize weights
        let sum1: f64 = child1.resource_weights.iter().sum();
        let sum2: f64 = child2.resource_weights.iter().sum();
        child1.resource_weights.iter_mut().for_each(|w| *w /= sum1);
        child2.resource_weights.iter_mut().for_each(|w| *w /= sum2);

        // Reset fitness
        child1.fitness = 0.0;
        child2.fitness = 0.0;

        Ok((child1, child2))
    }

    /// Mutation operation for chromosome
    fn mutate(&mut self, chromosome: &mut BottleneckChromosome) -> Result<()> {
        // Mutate resource weights
        let idx = self.rng.gen_range(0..chromosome.resource_weights.len());
        chromosome.resource_weights[idx] += self.rng.gen_range(-0.1..0.1);
        chromosome.resource_weights[idx] = chromosome.resource_weights[idx].clamp(0.01, 1.0);

        // Normalize weights
        let sum: f64 = chromosome.resource_weights.iter().sum();
        chromosome
            .resource_weights
            .iter_mut()
            .for_each(|w| *w /= sum);

        // Mutate threshold values
        let idx = self.rng.gen_range(0..chromosome.threshold_values.len());
        chromosome.threshold_values[idx] += self.rng.gen_range(-0.05..0.05);
        chromosome.threshold_values[idx] = chromosome.threshold_values[idx].clamp(0.1, 0.9);

        // Reset fitness
        chromosome.fitness = 0.0;

        Ok(())
    }

    /// Survivor selection combining parents and offspring
    fn survivor_selection(
        &self,
        parents: &[BottleneckChromosome],
        offspring: &[BottleneckChromosome],
    ) -> Result<Vec<BottleneckChromosome>> {
        let mut combined = Vec::new();
        combined.extend_from_slice(parents);
        combined.extend_from_slice(offspring);

        // Sort by fitness (descending)
        combined.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        // Select top individuals
        combined.truncate(self.population_size);

        Ok(combined)
    }

    /// Generate comprehensive bottleneck analysis from best chromosome
    async fn generate_bottleneck_analysis(
        &self,
        best_chromosome: &BottleneckChromosome,
        performance_data: &[PerformanceDataPoint],
        metadata: AnalysisMetadata,
    ) -> Result<BottleneckAnalysis> {
        // Identify bottlenecks using the evolved parameters
        let bottlenecks = self.identify_bottlenecks(best_chromosome, performance_data)?;

        // Generate optimization recommendations
        let recommendations = self.generate_recommendations(&bottlenecks)?;

        // Calculate overall metrics
        let overall_confidence = if bottlenecks.is_empty() {
            0.8 // High confidence when no bottlenecks are detected (good performance)
        } else {
            bottlenecks.iter().map(|b| b.confidence).sum::<f64>() / bottlenecks.len() as f64
        };

        let performance_score = self.calculate_performance_score(&bottlenecks);

        Ok(BottleneckAnalysis {
            identified_bottlenecks: bottlenecks,
            optimization_recommendations: recommendations,
            overall_confidence,
            performance_score,
            genetic_generations: metadata.final_generation,
            convergence_achieved: metadata.convergence_generation.is_some(),
            analysis_metadata: metadata,
        })
    }

    /// Identify bottlenecks using evolved chromosome parameters
    fn identify_bottlenecks(
        &self,
        chromosome: &BottleneckChromosome,
        performance_data: &[PerformanceDataPoint],
    ) -> Result<Vec<Bottleneck>> {
        let mut bottlenecks = Vec::new();

        if performance_data.is_empty() {
            return Ok(bottlenecks);
        }

        // Get evolved thresholds
        let cpu_threshold = chromosome.threshold_values.get(0).unwrap_or(&0.7);
        let mem_threshold = chromosome.threshold_values.get(1).unwrap_or(&0.7);
        let io_threshold = chromosome.threshold_values.get(2).unwrap_or(&0.5);
        let net_threshold = chromosome.threshold_values.get(3).unwrap_or(&0.5);

        // Calculate resource utilization statistics
        let cpu_stats = self.calculate_resource_stats(
            &performance_data
                .iter()
                .map(|p| p.cpu_usage)
                .collect::<Vec<_>>(),
        );
        let mem_stats = self.calculate_resource_stats(
            &performance_data
                .iter()
                .map(|p| p.memory_usage)
                .collect::<Vec<_>>(),
        );
        let io_stats = self.calculate_resource_stats(
            &performance_data
                .iter()
                .map(|p| p.io_wait)
                .collect::<Vec<_>>(),
        );
        let net_stats = self.calculate_resource_stats(
            &performance_data
                .iter()
                .map(|p| p.network_latency)
                .collect::<Vec<_>>(),
        );

        // Check for CPU bottlenecks
        if cpu_stats.mean > *cpu_threshold || cpu_stats.peak > 0.9 {
            let severity = self.determine_severity(cpu_stats.mean, *cpu_threshold, cpu_stats.peak);
            let confidence = self.calculate_confidence(&cpu_stats, *cpu_threshold);

            bottlenecks.push(Bottleneck {
                resource_type: ResourceType::CPU,
                severity,
                confidence,
                impact_score: self.calculate_impact_score(
                    cpu_stats.mean,
                    *cpu_threshold,
                    &cpu_stats,
                ),
                location: self.identify_bottleneck_location(ResourceType::CPU, performance_data),
                metrics: BottleneckMetrics {
                    current_utilization: cpu_stats.mean,
                    baseline_utilization: *cpu_threshold,
                    peak_utilization: cpu_stats.peak,
                    average_utilization: cpu_stats.mean,
                    variance: cpu_stats.variance,
                },
            });
        }

        // Check for Memory bottlenecks
        if mem_stats.mean > *mem_threshold || mem_stats.peak > 0.9 {
            let severity = self.determine_severity(mem_stats.mean, *mem_threshold, mem_stats.peak);
            let confidence = self.calculate_confidence(&mem_stats, *mem_threshold);

            bottlenecks.push(Bottleneck {
                resource_type: ResourceType::Memory,
                severity,
                confidence,
                impact_score: self.calculate_impact_score(
                    mem_stats.mean,
                    *mem_threshold,
                    &mem_stats,
                ),
                location: self.identify_bottleneck_location(ResourceType::Memory, performance_data),
                metrics: BottleneckMetrics {
                    current_utilization: mem_stats.mean,
                    baseline_utilization: *mem_threshold,
                    peak_utilization: mem_stats.peak,
                    average_utilization: mem_stats.mean,
                    variance: mem_stats.variance,
                },
            });
        }

        // Check for I/O bottlenecks
        if io_stats.mean > *io_threshold || io_stats.peak > 0.8 {
            let severity = self.determine_severity(io_stats.mean, *io_threshold, io_stats.peak);
            let confidence = self.calculate_confidence(&io_stats, *io_threshold);

            bottlenecks.push(Bottleneck {
                resource_type: ResourceType::IO,
                severity,
                confidence,
                impact_score: self.calculate_impact_score(io_stats.mean, *io_threshold, &io_stats),
                location: self.identify_bottleneck_location(ResourceType::IO, performance_data),
                metrics: BottleneckMetrics {
                    current_utilization: io_stats.mean,
                    baseline_utilization: *io_threshold,
                    peak_utilization: io_stats.peak,
                    average_utilization: io_stats.mean,
                    variance: io_stats.variance,
                },
            });
        }

        // Check for Network bottlenecks (convert latency to normalized scale)
        let normalized_net_mean = net_stats.mean / 100.0;
        let normalized_net_peak = net_stats.peak / 100.0;

        if normalized_net_mean > *net_threshold || normalized_net_peak > 1.0 {
            let severity =
                self.determine_severity(normalized_net_mean, *net_threshold, normalized_net_peak);
            let confidence = self.calculate_confidence(
                &ResourceStats {
                    mean: normalized_net_mean,
                    peak: normalized_net_peak,
                    variance: net_stats.variance / 10000.0, // Normalize variance too
                },
                *net_threshold,
            );

            bottlenecks.push(Bottleneck {
                resource_type: ResourceType::Network,
                severity,
                confidence,
                impact_score: self.calculate_impact_score(
                    normalized_net_mean,
                    *net_threshold,
                    &ResourceStats {
                        mean: normalized_net_mean,
                        peak: normalized_net_peak,
                        variance: net_stats.variance / 10000.0,
                    },
                ),
                location: self
                    .identify_bottleneck_location(ResourceType::Network, performance_data),
                metrics: BottleneckMetrics {
                    current_utilization: net_stats.mean,
                    baseline_utilization: *net_threshold * 100.0,
                    peak_utilization: net_stats.peak,
                    average_utilization: net_stats.mean,
                    variance: net_stats.variance,
                },
            });
        }

        // Check for compound bottlenecks (multiple resources stressed simultaneously)
        let compound_bottleneck = self.detect_compound_bottleneck(
            chromosome,
            performance_data,
            &[
                (ResourceType::CPU, cpu_stats.mean, *cpu_threshold),
                (ResourceType::Memory, mem_stats.mean, *mem_threshold),
                (ResourceType::IO, io_stats.mean, *io_threshold),
                (ResourceType::Network, normalized_net_mean, *net_threshold),
            ],
        );

        if let Some(compound) = compound_bottleneck {
            bottlenecks.push(compound);
        }

        // Sort bottlenecks by impact score (descending)
        bottlenecks.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());

        Ok(bottlenecks)
    }

    /// Calculate resource utilization statistics
    fn calculate_resource_stats(&self, values: &[f64]) -> ResourceStats {
        if values.is_empty() {
            return ResourceStats {
                mean: 0.0,
                peak: 0.0,
                variance: 0.0,
            };
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let peak = values.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        ResourceStats {
            mean,
            peak,
            variance,
        }
    }

    /// Determine bottleneck severity based on utilization levels
    fn determine_severity(
        &self,
        mean_utilization: f64,
        threshold: f64,
        peak_utilization: f64,
    ) -> BottleneckSeverity {
        let excess_ratio = (mean_utilization - threshold) / threshold;
        let peak_ratio = peak_utilization / threshold;

        if peak_utilization > 0.95 || excess_ratio > 0.6 {
            BottleneckSeverity::Critical
        } else if peak_utilization > 0.85 || excess_ratio > 0.4 {
            BottleneckSeverity::High
        } else if peak_utilization > 0.75 || excess_ratio > 0.2 {
            BottleneckSeverity::Medium
        } else if peak_utilization > 0.65 || excess_ratio > 0.1 {
            BottleneckSeverity::Low
        } else {
            BottleneckSeverity::Minimal
        }
    }

    /// Calculate confidence in bottleneck identification
    fn calculate_confidence(&self, stats: &ResourceStats, threshold: f64) -> f64 {
        let consistency = if stats.variance > 0.0 {
            1.0 / (1.0 + stats.variance.sqrt())
        } else {
            1.0
        };

        let threshold_distance = if threshold > 0.0 {
            ((stats.mean - threshold) / threshold).max(0.0)
        } else {
            0.0
        };

        let peak_consistency = if stats.peak > 0.0 {
            stats.mean / stats.peak
        } else {
            1.0
        };

        // Combine factors for overall confidence
        let confidence: f64 =
            (consistency * 0.4) + (threshold_distance.min(1.0) * 0.4) + (peak_consistency * 0.2);
        confidence.clamp(0.0, 1.0)
    }

    /// Calculate impact score for bottleneck
    fn calculate_impact_score(
        &self,
        utilization: f64,
        threshold: f64,
        stats: &ResourceStats,
    ) -> f64 {
        let severity_impact = ((utilization - threshold) / threshold).max(0.0);
        let peak_impact = (stats.peak - threshold).max(0.0) / threshold;
        let variability_penalty = stats.variance.sqrt() * 0.1;

        let total_impact = (severity_impact * 0.5) + (peak_impact * 0.4) - variability_penalty;
        total_impact.clamp(0.0, 1.0)
    }

    /// Identify location of bottleneck
    fn identify_bottleneck_location(
        &self,
        resource_type: ResourceType,
        performance_data: &[PerformanceDataPoint],
    ) -> BottleneckLocation {
        // Find the data point with highest utilization for this resource type
        let worst_point = performance_data.iter().max_by(|a, b| {
            let a_val = self.get_resource_value(a, &resource_type);
            let b_val = self.get_resource_value(b, &resource_type);
            a_val.partial_cmp(&b_val).unwrap()
        });

        if let Some(point) = worst_point {
            BottleneckLocation {
                component: point.component.clone(),
                function: None, // Could be enhanced with more detailed profiling
                line_number: None,
                file_path: None,
            }
        } else {
            BottleneckLocation {
                component: "Unknown".to_string(),
                function: None,
                line_number: None,
                file_path: None,
            }
        }
    }

    /// Get resource value for specific resource type
    fn get_resource_value(
        &self,
        data_point: &PerformanceDataPoint,
        resource_type: &ResourceType,
    ) -> f64 {
        match resource_type {
            ResourceType::CPU => data_point.cpu_usage,
            ResourceType::Memory => data_point.memory_usage,
            ResourceType::IO => data_point.io_wait,
            ResourceType::Network => data_point.network_latency,
            ResourceType::Database => data_point.execution_time, // Approximate with execution time
            ResourceType::Cache => 1.0 - data_point.throughput / 1000.0, // Inverse throughput as cache miss rate
        }
    }

    /// Detect compound bottlenecks where multiple resources are stressed
    fn detect_compound_bottleneck(
        &self,
        chromosome: &BottleneckChromosome,
        performance_data: &[PerformanceDataPoint],
        resource_states: &[(ResourceType, f64, f64)], // (type, utilization, threshold)
    ) -> Option<Bottleneck> {
        let stressed_resources: Vec<_> = resource_states
            .iter()
            .filter(|(_, utilization, threshold)| *utilization > *threshold * 0.8)
            .collect();

        if stressed_resources.len() >= 2 {
            // Calculate weighted impact across resources
            let total_weight: f64 = chromosome.resource_weights.iter().sum();
            let weighted_impact: f64 = stressed_resources
                .iter()
                .enumerate()
                .map(|(i, (_, utilization, threshold))| {
                    let weight = chromosome.resource_weights.get(i).unwrap_or(&0.25) / total_weight;
                    let excess = (utilization - threshold) / threshold;
                    weight * excess
                })
                .sum();

            if weighted_impact > 0.3 {
                return Some(Bottleneck {
                    resource_type: ResourceType::CPU, // Primary resource type
                    severity: BottleneckSeverity::High,
                    confidence: 0.8,
                    impact_score: weighted_impact.min(1.0),
                    location: BottleneckLocation {
                        component: "System-wide".to_string(),
                        function: Some("compound_resource_contention".to_string()),
                        line_number: None,
                        file_path: None,
                    },
                    metrics: BottleneckMetrics {
                        current_utilization: weighted_impact,
                        baseline_utilization: 0.3,
                        peak_utilization: weighted_impact * 1.2,
                        average_utilization: weighted_impact,
                        variance: 0.05,
                    },
                });
            }
        }

        None
    }

    /// Generate optimization recommendations based on identified bottlenecks
    fn generate_recommendations(&self, bottlenecks: &[Bottleneck]) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        for bottleneck in bottlenecks {
            match bottleneck.resource_type {
                ResourceType::CPU => {
                    recommendations.extend(self.generate_cpu_recommendations(bottleneck));
                }
                ResourceType::Memory => {
                    recommendations.extend(self.generate_memory_recommendations(bottleneck));
                }
                ResourceType::IO => {
                    recommendations.extend(self.generate_io_recommendations(bottleneck));
                }
                ResourceType::Network => {
                    recommendations.extend(self.generate_network_recommendations(bottleneck));
                }
                ResourceType::Database => {
                    recommendations.extend(self.generate_database_recommendations(bottleneck));
                }
                ResourceType::Cache => {
                    recommendations.extend(self.generate_cache_recommendations(bottleneck));
                }
            }
        }

        // Sort recommendations by priority and estimated impact
        recommendations.sort_by(|a, b| {
            let a_priority_score = self.get_priority_score(&a.priority);
            let b_priority_score = self.get_priority_score(&b.priority);

            // First sort by priority, then by impact
            b_priority_score
                .cmp(&a_priority_score)
                .then_with(|| b.estimated_impact.partial_cmp(&a.estimated_impact).unwrap())
        });

        // Remove duplicate recommendations
        recommendations.dedup_by(|a, b| a.title == b.title);

        Ok(recommendations)
    }

    /// Generate CPU-specific optimization recommendations
    fn generate_cpu_recommendations(&self, bottleneck: &Bottleneck) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        match bottleneck.severity {
            BottleneckSeverity::Critical | BottleneckSeverity::High => {
                recommendations.push(Recommendation {
                    title: "Implement CPU-intensive task optimization".to_string(),
                    description: format!(
                        "CPU utilization is at {:.1}% (baseline: {:.1}%). Consider implementing parallel processing, \
                        algorithm optimization, or moving CPU-intensive operations to background threads.",
                        bottleneck.metrics.current_utilization * 100.0,
                        bottleneck.metrics.baseline_utilization * 100.0
                    ),
                    priority: RecommendationPriority::Immediate,
                    estimated_impact: bottleneck.impact_score * 0.8,
                    implementation_effort: ImplementationEffort::Medium,
                    category: RecommendationCategory::CodeOptimization,
                    specific_actions: vec![
                        "Profile CPU-intensive functions using perf or similar tools".to_string(),
                        "Implement parallel processing for independent operations".to_string(),
                        "Consider algorithmic optimizations (O(n²) → O(n log n))".to_string(),
                        "Move blocking operations to async/background processing".to_string(),
                        "Enable compiler optimizations (-O3, LTO)".to_string(),
                    ],
                });

                if bottleneck.metrics.peak_utilization > 0.95 {
                    recommendations.push(Recommendation {
                        title: "Scale CPU resources".to_string(),
                        description: "Peak CPU utilization exceeds 95%. Consider vertical scaling or distributing load.".to_string(),
                        priority: RecommendationPriority::High,
                        estimated_impact: 0.7,
                        implementation_effort: ImplementationEffort::Low,
                        category: RecommendationCategory::ResourceScaling,
                        specific_actions: vec![
                            "Increase CPU cores/threads in deployment configuration".to_string(),
                            "Implement horizontal scaling with load balancers".to_string(),
                            "Consider auto-scaling policies based on CPU metrics".to_string(),
                        ],
                    });
                }
            }
            BottleneckSeverity::Medium => {
                recommendations.push(Recommendation {
                    title: "Optimize CPU usage patterns".to_string(),
                    description: "CPU usage shows room for improvement. Focus on identifying hot paths and optimizing them.".to_string(),
                    priority: RecommendationPriority::Medium,
                    estimated_impact: bottleneck.impact_score * 0.6,
                    implementation_effort: ImplementationEffort::Low,
                    category: RecommendationCategory::CodeOptimization,
                    specific_actions: vec![
                        "Profile application to identify CPU hot spots".to_string(),
                        "Implement caching for expensive computations".to_string(),
                        "Review and optimize loops and recursive algorithms".to_string(),
                    ],
                });
            }
            _ => {} // Low and Minimal don't need immediate action
        }

        recommendations
    }

    /// Generate Memory-specific optimization recommendations
    fn generate_memory_recommendations(&self, bottleneck: &Bottleneck) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        match bottleneck.severity {
            BottleneckSeverity::Critical | BottleneckSeverity::High => {
                recommendations.push(Recommendation {
                    title: "Address memory consumption issues".to_string(),
                    description: format!(
                        "Memory usage is at {:.1}% (baseline: {:.1}%). High memory usage can lead to GC pressure \
                        and potential OOM conditions.",
                        bottleneck.metrics.current_utilization * 100.0,
                        bottleneck.metrics.baseline_utilization * 100.0
                    ),
                    priority: RecommendationPriority::Immediate,
                    estimated_impact: bottleneck.impact_score * 0.9,
                    implementation_effort: ImplementationEffort::Medium,
                    category: RecommendationCategory::CodeOptimization,
                    specific_actions: vec![
                        "Conduct memory profiling to identify leaks and large allocations".to_string(),
                        "Implement object pooling for frequently allocated objects".to_string(),
                        "Review data structures for memory efficiency".to_string(),
                        "Implement lazy loading and data pagination".to_string(),
                        "Consider streaming processing for large datasets".to_string(),
                    ],
                });

                if bottleneck.metrics.variance > 0.1 {
                    recommendations.push(Recommendation {
                        title: "Stabilize memory usage patterns".to_string(),
                        description: "Memory usage shows high variability, indicating potential memory leaks or inefficient allocation patterns.".to_string(),
                        priority: RecommendationPriority::High,
                        estimated_impact: 0.6,
                        implementation_effort: ImplementationEffort::High,
                        category: RecommendationCategory::CodeOptimization,
                        specific_actions: vec![
                            "Implement comprehensive memory leak detection".to_string(),
                            "Review garbage collection settings and strategies".to_string(),
                            "Add memory usage monitoring and alerting".to_string(),
                        ],
                    });
                }
            }
            BottleneckSeverity::Medium => {
                recommendations.push(Recommendation {
                    title: "Optimize memory allocation patterns".to_string(),
                    description: "Memory usage could be optimized to prevent future bottlenecks."
                        .to_string(),
                    priority: RecommendationPriority::Medium,
                    estimated_impact: bottleneck.impact_score * 0.5,
                    implementation_effort: ImplementationEffort::Low,
                    category: RecommendationCategory::CodeOptimization,
                    specific_actions: vec![
                        "Review large object allocations".to_string(),
                        "Implement memory-efficient data structures".to_string(),
                        "Consider memory mapping for large files".to_string(),
                    ],
                });
            }
            _ => {}
        }

        recommendations
    }

    /// Generate I/O-specific optimization recommendations
    fn generate_io_recommendations(&self, bottleneck: &Bottleneck) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        match bottleneck.severity {
            BottleneckSeverity::Critical | BottleneckSeverity::High => {
                recommendations.push(Recommendation {
                    title: "Optimize I/O operations".to_string(),
                    description: "High I/O wait times are impacting performance. Focus on reducing blocking I/O operations.".to_string(),
                    priority: RecommendationPriority::High,
                    estimated_impact: bottleneck.impact_score * 0.8,
                    implementation_effort: ImplementationEffort::Medium,
                    category: RecommendationCategory::CodeOptimization,
                    specific_actions: vec![
                        "Implement asynchronous I/O operations".to_string(),
                        "Add read/write caching layers".to_string(),
                        "Batch small I/O operations".to_string(),
                        "Use memory-mapped files for large datasets".to_string(),
                        "Consider SSD storage for better I/O performance".to_string(),
                    ],
                });
            }
            BottleneckSeverity::Medium => {
                recommendations.push(Recommendation {
                    title: "Improve I/O efficiency".to_string(),
                    description: "I/O operations show room for optimization.".to_string(),
                    priority: RecommendationPriority::Medium,
                    estimated_impact: bottleneck.impact_score * 0.6,
                    implementation_effort: ImplementationEffort::Low,
                    category: RecommendationCategory::CodeOptimization,
                    specific_actions: vec![
                        "Review file access patterns".to_string(),
                        "Implement buffered I/O where appropriate".to_string(),
                        "Consider I/O operation batching".to_string(),
                    ],
                });
            }
            _ => {}
        }

        recommendations
    }

    /// Generate Network-specific optimization recommendations
    fn generate_network_recommendations(&self, bottleneck: &Bottleneck) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        match bottleneck.severity {
            BottleneckSeverity::Critical | BottleneckSeverity::High => {
                recommendations.push(Recommendation {
                    title: "Reduce network latency".to_string(),
                    description: format!(
                        "Network latency is at {:.1}ms (baseline: {:.1}ms). High latency impacts user experience and system performance.",
                        bottleneck.metrics.current_utilization,
                        bottleneck.metrics.baseline_utilization
                    ),
                    priority: RecommendationPriority::High,
                    estimated_impact: bottleneck.impact_score * 0.7,
                    implementation_effort: ImplementationEffort::Medium,
                    category: RecommendationCategory::ArchitecturalChange,
                    specific_actions: vec![
                        "Implement connection pooling and reuse".to_string(),
                        "Add CDN for static content delivery".to_string(),
                        "Implement request/response compression".to_string(),
                        "Consider moving services closer to users (edge deployment)".to_string(),
                        "Optimize API payload sizes".to_string(),
                    ],
                });
            }
            BottleneckSeverity::Medium => {
                recommendations.push(Recommendation {
                    title: "Optimize network usage".to_string(),
                    description: "Network performance can be improved through optimization."
                        .to_string(),
                    priority: RecommendationPriority::Medium,
                    estimated_impact: bottleneck.impact_score * 0.5,
                    implementation_effort: ImplementationEffort::Low,
                    category: RecommendationCategory::ConfigurationTuning,
                    specific_actions: vec![
                        "Review network timeout configurations".to_string(),
                        "Implement request batching where possible".to_string(),
                        "Consider local caching of remote data".to_string(),
                    ],
                });
            }
            _ => {}
        }

        recommendations
    }

    /// Generate Database-specific optimization recommendations
    fn generate_database_recommendations(&self, bottleneck: &Bottleneck) -> Vec<Recommendation> {
        vec![Recommendation {
            title: "Optimize database queries".to_string(),
            description:
                "Database performance issues detected. Focus on query optimization and indexing."
                    .to_string(),
            priority: RecommendationPriority::High,
            estimated_impact: bottleneck.impact_score * 0.8,
            implementation_effort: ImplementationEffort::Medium,
            category: RecommendationCategory::DatabaseOptimization,
            specific_actions: vec![
                "Profile slow queries and add appropriate indexes".to_string(),
                "Implement query result caching".to_string(),
                "Consider database connection pooling".to_string(),
                "Review database schema for normalization issues".to_string(),
            ],
        }]
    }

    /// Generate Cache-specific optimization recommendations
    fn generate_cache_recommendations(&self, bottleneck: &Bottleneck) -> Vec<Recommendation> {
        vec![
            Recommendation {
                title: "Improve caching strategy".to_string(),
                description: "Cache performance issues detected. Consider cache sizing, TTL, and eviction policies.".to_string(),
                priority: RecommendationPriority::Medium,
                estimated_impact: bottleneck.impact_score * 0.6,
                implementation_effort: ImplementationEffort::Low,
                category: RecommendationCategory::CachingStrategy,
                specific_actions: vec![
                    "Review cache hit rates and adjust cache sizes".to_string(),
                    "Optimize cache key patterns and TTL values".to_string(),
                    "Consider implementing multi-level caching".to_string(),
                    "Add cache warming strategies for critical data".to_string(),
                ],
            }
        ]
    }

    /// Get numeric priority score for sorting
    fn get_priority_score(&self, priority: &RecommendationPriority) -> u8 {
        match priority {
            RecommendationPriority::Immediate => 4,
            RecommendationPriority::High => 3,
            RecommendationPriority::Medium => 2,
            RecommendationPriority::Low => 1,
        }
    }

    /// Calculate overall performance score
    fn calculate_performance_score(&self, bottlenecks: &[Bottleneck]) -> f64 {
        if bottlenecks.is_empty() {
            return 100.0;
        }

        let total_impact: f64 = bottlenecks.iter().map(|b| b.impact_score).sum();

        (100.0 - (total_impact / bottlenecks.len() as f64 * 100.0)).max(0.0)
    }
}

impl FitnessEvaluator {
    /// Create new fitness evaluator with default objectives
    pub fn new() -> Self {
        Self {
            objectives: vec![
                OptimizationObjective::ExecutionTime,
                OptimizationObjective::MemoryUsage,
                OptimizationObjective::Throughput,
            ],
            weights: vec![0.4, 0.3, 0.3],
            baseline_metrics: HashMap::new(),
        }
    }

    /// Evaluate fitness of a chromosome against performance data
    pub fn evaluate_fitness(
        &self,
        chromosome: &BottleneckChromosome,
        performance_data: &[PerformanceDataPoint],
    ) -> Result<f64> {
        if performance_data.is_empty() {
            return Ok(0.0);
        }

        let mut total_fitness = 0.0;
        let mut objective_scores = Vec::new();

        // Calculate fitness for each optimization objective
        for (i, objective) in self.objectives.iter().enumerate() {
            let weight = self.weights.get(i).unwrap_or(&1.0);
            let score = self.calculate_objective_score(objective, chromosome, performance_data)?;
            objective_scores.push(score);
            total_fitness += score * weight;
        }

        // Apply multi-objective optimization with Pareto dominance consideration
        let pareto_fitness = self.calculate_pareto_fitness(&objective_scores);

        // Consider bottleneck detection accuracy
        let detection_accuracy = self.evaluate_detection_accuracy(chromosome, performance_data)?;

        // Combine scores with weights
        let combined_fitness =
            (total_fitness * 0.6) + (pareto_fitness * 0.2) + (detection_accuracy * 0.2);

        Ok(combined_fitness.clamp(0.0, 1.0))
    }

    /// Calculate score for specific optimization objective
    fn calculate_objective_score(
        &self,
        objective: &OptimizationObjective,
        chromosome: &BottleneckChromosome,
        performance_data: &[PerformanceDataPoint],
    ) -> Result<f64> {
        let values: Vec<f64> = match objective {
            OptimizationObjective::ExecutionTime => {
                performance_data.iter().map(|p| p.execution_time).collect()
            }
            OptimizationObjective::MemoryUsage => {
                performance_data.iter().map(|p| p.memory_usage).collect()
            }
            OptimizationObjective::IOWait => performance_data.iter().map(|p| p.io_wait).collect(),
            OptimizationObjective::CPUUtilization => {
                performance_data.iter().map(|p| p.cpu_usage).collect()
            }
            OptimizationObjective::NetworkLatency => {
                performance_data.iter().map(|p| p.network_latency).collect()
            }
            OptimizationObjective::Throughput => {
                performance_data.iter().map(|p| p.throughput).collect()
            }
            OptimizationObjective::ResourceContention => {
                // Calculate resource contention as combination of resource utilizations
                performance_data
                    .iter()
                    .map(|p| {
                        let cpu_weight = chromosome.resource_weights.get(0).unwrap_or(&0.25);
                        let mem_weight = chromosome.resource_weights.get(1).unwrap_or(&0.25);
                        let io_weight = chromosome.resource_weights.get(2).unwrap_or(&0.25);
                        let net_weight = chromosome.resource_weights.get(3).unwrap_or(&0.25);

                        p.cpu_usage * cpu_weight
                            + p.memory_usage * mem_weight
                            + p.io_wait * io_weight
                            + (p.network_latency / 100.0) * net_weight
                    })
                    .collect()
            }
        };

        if values.is_empty() {
            return Ok(0.0);
        }

        // Calculate statistical measures
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Score based on objective direction and statistical properties
        let baseline = self
            .baseline_metrics
            .get(&format!("{:?}", objective))
            .unwrap_or(&mean);

        let improvement_ratio = match objective {
            OptimizationObjective::Throughput => {
                // Higher is better
                if *baseline > 0.0 {
                    mean / baseline
                } else {
                    1.0
                }
            }
            _ => {
                // Lower is better for most objectives
                if mean > 0.0 {
                    baseline / mean
                } else {
                    1.0
                }
            }
        };

        // Consider consistency (lower variance is better)
        let consistency_score = if std_dev > 0.0 {
            1.0 / (1.0 + std_dev / mean)
        } else {
            1.0
        };

        // Combine improvement and consistency
        let score = (improvement_ratio * 0.7) + (consistency_score * 0.3);
        Ok(score.clamp(0.0, 2.0)) // Allow scores up to 2.0 for exceptional performance
    }

    /// Calculate Pareto fitness considering multiple objectives
    fn calculate_pareto_fitness(&self, objective_scores: &[f64]) -> f64 {
        if objective_scores.is_empty() {
            return 0.0;
        }

        // Calculate geometric mean for balanced multi-objective performance
        let product: f64 = objective_scores.iter().product();
        let geometric_mean = product.powf(1.0 / objective_scores.len() as f64);

        // Consider trade-offs between objectives
        let variance = objective_scores
            .iter()
            .map(|&score| {
                let mean = objective_scores.iter().sum::<f64>() / objective_scores.len() as f64;
                (score - mean).powi(2)
            })
            .sum::<f64>()
            / objective_scores.len() as f64;

        // Balance between performance and trade-off management
        geometric_mean - (variance.sqrt() * 0.1)
    }

    /// Evaluate how accurately the chromosome detects bottlenecks
    fn evaluate_detection_accuracy(
        &self,
        chromosome: &BottleneckChromosome,
        performance_data: &[PerformanceDataPoint],
    ) -> Result<f64> {
        let mut detection_score = 0.0;
        let mut total_evaluations = 0;
        let mut bottleneck_count = 0;

        for data_point in performance_data {
            // Check if chromosome correctly identifies bottlenecks
            let cpu_threshold = chromosome.threshold_values.get(0).unwrap_or(&0.7);
            let mem_threshold = chromosome.threshold_values.get(1).unwrap_or(&0.7);
            let io_threshold = chromosome.threshold_values.get(2).unwrap_or(&0.5);
            let net_threshold = chromosome.threshold_values.get(3).unwrap_or(&0.5);

            // Evaluate detection accuracy for each resource type
            if data_point.cpu_usage > *cpu_threshold {
                let cpu_weight = chromosome.resource_weights.get(0).unwrap_or(&0.25);
                detection_score += cpu_weight
                    * self.calculate_detection_precision(data_point.cpu_usage, *cpu_threshold);
                bottleneck_count += 1;
            }

            if data_point.memory_usage > *mem_threshold {
                let mem_weight = chromosome.resource_weights.get(1).unwrap_or(&0.25);
                detection_score += mem_weight
                    * self.calculate_detection_precision(data_point.memory_usage, *mem_threshold);
                bottleneck_count += 1;
            }

            if data_point.io_wait > *io_threshold {
                let io_weight = chromosome.resource_weights.get(2).unwrap_or(&0.25);
                detection_score += io_weight
                    * self.calculate_detection_precision(data_point.io_wait, *io_threshold);
                bottleneck_count += 1;
            }

            if data_point.network_latency > (*net_threshold * 100.0) {
                let net_weight = chromosome.resource_weights.get(3).unwrap_or(&0.25);
                detection_score += net_weight
                    * self.calculate_detection_precision(
                        data_point.network_latency / 100.0,
                        *net_threshold,
                    );
                bottleneck_count += 1;
            }

            total_evaluations += 1;
        }

        if bottleneck_count > 0 {
            Ok(detection_score / bottleneck_count as f64)
        } else {
            // No bottlenecks detected - this could be good (healthy system) or bad (poor detection)
            // Check if the system actually has good performance characteristics
            let avg_cpu = performance_data.iter().map(|p| p.cpu_usage).sum::<f64>()
                / performance_data.len() as f64;
            let avg_mem = performance_data.iter().map(|p| p.memory_usage).sum::<f64>()
                / performance_data.len() as f64;
            let avg_io = performance_data.iter().map(|p| p.io_wait).sum::<f64>()
                / performance_data.len() as f64;

            if avg_cpu < 0.6 && avg_mem < 0.6 && avg_io < 0.3 {
                Ok(0.8) // High score for correctly identifying healthy system
            } else {
                Ok(0.3) // Low score for missing bottlenecks in a stressed system
            }
        }
    }

    /// Calculate precision of bottleneck detection
    fn calculate_detection_precision(&self, actual_value: f64, threshold: f64) -> f64 {
        if threshold <= 0.0 {
            return 0.0;
        }

        // Calculate how far above threshold the value is (normalized)
        let excess_ratio = (actual_value - threshold) / threshold;

        // Score based on appropriate threshold setting
        // Higher excess with lower threshold = better detection
        // But penalize thresholds that are too low (false positives)
        let precision = if excess_ratio > 0.5 {
            1.0 // Clear bottleneck, good detection
        } else if excess_ratio > 0.1 {
            0.8 // Moderate bottleneck
        } else {
            0.4 // Marginal bottleneck, might be false positive
        };

        precision
    }
}

/// Performance data point for genetic algorithm analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub io_wait: f64,
    pub network_latency: f64,
    pub execution_time: f64,
    pub throughput: f64,
    pub component: String,
}

impl Default for GeneticBottleneckDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BottleneckSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BottleneckSeverity::Critical => write!(f, "Critical"),
            BottleneckSeverity::High => write!(f, "High"),
            BottleneckSeverity::Medium => write!(f, "Medium"),
            BottleneckSeverity::Low => write!(f, "Low"),
            BottleneckSeverity::Minimal => write!(f, "Minimal"),
        }
    }
}

impl fmt::Display for RecommendationPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecommendationPriority::Immediate => write!(f, "Immediate"),
            RecommendationPriority::High => write!(f, "High"),
            RecommendationPriority::Medium => write!(f, "Medium"),
            RecommendationPriority::Low => write!(f, "Low"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genetic_detector_creation() {
        let detector = GeneticBottleneckDetector::new();
        assert_eq!(detector.population_size, 50);
        assert_eq!(detector.generations, 100);
        assert_eq!(detector.mutation_rate, 0.1);
        assert_eq!(detector.crossover_rate, 0.8);
    }

    #[test]
    fn test_detector_configuration() {
        let detector = GeneticBottleneckDetector::new()
            .with_population_size(100)
            .with_generations(200)
            .with_mutation_rate(0.15)
            .with_crossover_rate(0.9);

        assert_eq!(detector.population_size, 100);
        assert_eq!(detector.generations, 200);
        assert_eq!(detector.mutation_rate, 0.15);
        assert_eq!(detector.crossover_rate, 0.9);
    }

    #[test]
    fn test_chromosome_creation() {
        let mut detector = GeneticBottleneckDetector::new();
        let chromosome = detector.create_random_chromosome().unwrap();

        // Check resource weights sum to approximately 1.0
        let sum: f64 = chromosome.resource_weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);

        // Check threshold values are in valid range
        for &threshold in &chromosome.threshold_values {
            assert!(threshold >= 0.1 && threshold <= 0.9);
        }

        // Check optimization targets exist
        assert!(!chromosome.optimization_targets.is_empty());
    }

    #[test]
    fn test_fitness_evaluator() {
        let evaluator = FitnessEvaluator::new();
        let mut detector = GeneticBottleneckDetector::new();
        let chromosome = detector.create_random_chromosome().unwrap();

        let performance_data = vec![PerformanceDataPoint {
            timestamp: 1000,
            cpu_usage: 0.5,
            memory_usage: 0.6,
            io_wait: 0.1,
            network_latency: 10.0,
            execution_time: 100.0,
            throughput: 1000.0,
            component: "test".to_string(),
        }];

        let fitness = evaluator
            .evaluate_fitness(&chromosome, &performance_data)
            .unwrap();
        assert!(fitness >= 0.0 && fitness <= 1.0);
    }

    #[test]
    fn test_population_initialization() {
        let mut detector = GeneticBottleneckDetector::new().with_population_size(10);
        let population = detector.initialize_population().unwrap();

        assert_eq!(population.len(), 10);

        // Check all chromosomes have valid resource weights
        for chromosome in &population {
            let sum: f64 = chromosome.resource_weights.iter().sum();
            assert!((sum - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_crossover_operation() {
        let mut detector = GeneticBottleneckDetector::new();
        let parent1 = detector.create_random_chromosome().unwrap();
        let parent2 = detector.create_random_chromosome().unwrap();

        let (child1, child2) = detector.crossover(&parent1, &parent2).unwrap();

        // Check children have valid resource weights
        let sum1: f64 = child1.resource_weights.iter().sum();
        let sum2: f64 = child2.resource_weights.iter().sum();
        assert!((sum1 - 1.0).abs() < 1e-10);
        assert!((sum2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_mutation_operation() {
        let mut detector = GeneticBottleneckDetector::new();
        let mut chromosome = detector.create_random_chromosome().unwrap();
        let original_weights = chromosome.resource_weights.clone();

        detector.mutate(&mut chromosome).unwrap();

        // Check weights are still normalized
        let sum: f64 = chromosome.resource_weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);

        // Check that mutation occurred (weights should be different)
        let weights_changed = chromosome.resource_weights != original_weights;
        assert!(weights_changed);
    }
}
