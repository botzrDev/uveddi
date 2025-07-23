//! Carbon Awareness and Energy Consumption Tracking for AI Workloads
//!
//! This module provides comprehensive energy consumption monitoring for Uveddi's
//! AI workloads, particularly focusing on the genetic algorithm and Ollama integration.
//! It integrates with existing observability infrastructure to enable carbon-aware
//! AI operations and sustainable computing practices.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use prometheus::{Counter, Gauge, Histogram, Registry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Carbon intensity data source for regional emission factors
#[derive(Debug, Clone)]
pub enum CarbonIntensitySource {
    /// Static regional carbon intensity (gCO2/kWh)
    Static(f64),
    /// Cloud provider API (future enhancement)
    CloudProvider(String),
    /// Real-time grid API (future enhancement)
    GridApi(String),
}

/// Configuration for carbon awareness monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonAwarenessConfig {
    /// Enable carbon tracking
    pub enabled: bool,
    /// Sampling interval for energy measurements (seconds)
    pub sampling_interval_secs: u64,
    /// Regional carbon intensity (gCO2/kWh) - default to US average
    pub carbon_intensity_g_co2_kwh: f64,
    /// CPU power estimation coefficient (watts per % utilization)
    pub cpu_power_coefficient: f64,
    /// Base system power consumption (watts)
    pub base_power_watts: f64,
    /// GPU power estimation (if applicable)
    pub gpu_power_watts: Option<f64>,
    /// Export metrics to Prometheus
    pub prometheus_enabled: bool,
}

impl Default for CarbonAwarenessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_secs: 10,
            carbon_intensity_g_co2_kwh: 429.0, // US average 2023
            cpu_power_coefficient: 1.5,
            base_power_watts: 50.0,
            gpu_power_watts: None,
            prometheus_enabled: true,
        }
    }
}

/// Energy consumption metrics for specific AI workloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConsumptionMetrics {
    /// Unique identifier for the workload
    pub workload_id: String,
    /// Workload type (genetic_algorithm, ollama_inference, etc.)
    pub workload_type: WorkloadType,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp
    pub end_time: Option<DateTime<Utc>>,
    /// Total energy consumed (watt-hours)
    pub energy_wh: f64,
    /// Peak power consumption (watts)
    pub peak_power_w: f64,
    /// Average power consumption (watts)
    pub avg_power_w: f64,
    /// Estimated CO2 emissions (grams)
    pub co2_emissions_g: f64,
    /// CPU utilization samples
    pub cpu_utilization_samples: Vec<f64>,
    /// Memory usage samples (MB)
    pub memory_usage_samples: Vec<f64>,
    /// Additional context data
    pub context: HashMap<String, String>,
}

/// Types of AI workloads tracked for carbon awareness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkloadType {
    /// Genetic algorithm bottleneck detection
    GeneticAlgorithm,
    /// Ollama local inference
    OllamaInference,
    /// General AI analysis
    AiAnalysis,
    /// Background processing
    BackgroundProcessing,
}

impl std::fmt::Display for WorkloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeneticAlgorithm => write!(f, "genetic_algorithm"),
            Self::OllamaInference => write!(f, "ollama_inference"),
            Self::AiAnalysis => write!(f, "ai_analysis"),
            Self::BackgroundProcessing => write!(f, "background_processing"),
        }
    }
}

/// Active energy tracking session
#[derive(Debug)]
pub struct EnergyTrackingSession {
    pub workload_id: String,
    pub workload_type: WorkloadType,
    pub start_time: Instant,
    pub start_time_utc: DateTime<Utc>,
    pub power_samples: Vec<f64>,
    pub cpu_samples: Vec<f64>,
    pub memory_samples: Vec<f64>,
    pub context: HashMap<String, String>,
}

/// Carbon awareness and energy tracking system
pub struct CarbonAwarenessCollector {
    config: CarbonAwarenessConfig,
    active_sessions: Arc<Mutex<HashMap<String, EnergyTrackingSession>>>,
    completed_metrics: Arc<Mutex<Vec<EnergyConsumptionMetrics>>>,
    
    // Prometheus metrics
    energy_consumption_counter: Counter,
    co2_emissions_counter: Counter,
    active_workloads_gauge: Gauge,
    power_consumption_histogram: Histogram,
    workload_duration_histogram: Histogram,
}

impl CarbonAwarenessCollector {
    /// Create a new carbon awareness collector
    pub fn new(config: CarbonAwarenessConfig, registry: Option<&Registry>) -> Result<Self> {
        let energy_consumption_counter = Counter::new(
            "uveddi_energy_consumption_wh_total",
            "Total energy consumption in watt-hours by workload type"
        )?;
        
        let co2_emissions_counter = Counter::new(
            "uveddi_co2_emissions_g_total", 
            "Total CO2 emissions in grams by workload type"
        )?;
        
        let active_workloads_gauge = Gauge::new(
            "uveddi_active_workloads",
            "Number of currently active AI workloads being tracked"
        )?;
        
        let power_consumption_histogram = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "uveddi_power_consumption_watts",
                "Power consumption distribution in watts"
            ).buckets(vec![10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0])
        )?;
        
        let workload_duration_histogram = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "uveddi_workload_duration_seconds",
                "AI workload duration distribution"
            ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0])
        )?;

        if let Some(reg) = registry {
            reg.register(Box::new(energy_consumption_counter.clone()))?;
            reg.register(Box::new(co2_emissions_counter.clone()))?;
            reg.register(Box::new(active_workloads_gauge.clone()))?;
            reg.register(Box::new(power_consumption_histogram.clone()))?;
            reg.register(Box::new(workload_duration_histogram.clone()))?;
        }

        Ok(Self {
            config,
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            completed_metrics: Arc::new(Mutex::new(Vec::new())),
            energy_consumption_counter,
            co2_emissions_counter,
            active_workloads_gauge,
            power_consumption_histogram,
            workload_duration_histogram,
        })
    }

    /// Start tracking energy consumption for a workload
    pub fn start_tracking(&self, workload_type: WorkloadType, context: Option<HashMap<String, String>>) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let workload_id = uuid::Uuid::new_v4().to_string();
        let session = EnergyTrackingSession {
            workload_id: workload_id.clone(),
            workload_type: workload_type.clone(),
            start_time: Instant::now(),
            start_time_utc: Utc::now(),
            power_samples: Vec::new(),
            cpu_samples: Vec::new(),
            memory_samples: Vec::new(),
            context: context.unwrap_or_default(),
        };

        {
            let mut sessions = self.active_sessions.lock().unwrap();
            sessions.insert(workload_id.clone(), session);
            self.active_workloads_gauge.set(sessions.len() as f64);
        }

        info!("Started energy tracking for {:?} workload: {}", workload_type, workload_id);
        workload_id
    }

    /// Stop tracking and calculate final metrics
    pub fn stop_tracking(&self, workload_id: &str) -> Result<EnergyConsumptionMetrics> {
        if !self.config.enabled || workload_id.is_empty() {
            return Err(anyhow!("Carbon tracking disabled or invalid workload ID"));
        }

        let session = {
            let mut sessions = self.active_sessions.lock().unwrap();
            let session = sessions.remove(workload_id)
                .ok_or_else(|| anyhow!("Workload session not found: {}", workload_id))?;
            self.active_workloads_gauge.set(sessions.len() as f64);
            session
        };

        let duration = session.start_time.elapsed();
        let end_time = Utc::now();

        // Calculate energy consumption
        let avg_power = if session.power_samples.is_empty() {
            self.estimate_power_consumption(50.0, 0.0) // Fallback estimation
        } else {
            session.power_samples.iter().sum::<f64>() / session.power_samples.len() as f64
        };

        let peak_power = session.power_samples.iter().cloned().fold(0.0, f64::max);
        let energy_wh = avg_power * duration.as_secs_f64() / 3600.0; // Convert to watt-hours
        let co2_emissions_g = energy_wh * self.config.carbon_intensity_g_co2_kwh / 1000.0;

        let metrics = EnergyConsumptionMetrics {
            workload_id: workload_id.to_string(),
            workload_type: session.workload_type.clone(),
            start_time: session.start_time_utc,
            end_time: Some(end_time),
            energy_wh,
            peak_power_w: peak_power,
            avg_power_w: avg_power,
            co2_emissions_g,
            cpu_utilization_samples: session.cpu_samples,
            memory_usage_samples: session.memory_samples,
            context: session.context,
        };

        // Update Prometheus metrics
        self.energy_consumption_counter.inc_by(energy_wh);
        self.co2_emissions_counter.inc_by(co2_emissions_g);
        self.power_consumption_histogram.observe(avg_power);
        self.workload_duration_histogram.observe(duration.as_secs_f64());

        {
            let mut completed = self.completed_metrics.lock().unwrap();
            completed.push(metrics.clone());
        }

        info!(
            "Completed energy tracking for {} workload: {} - {:.3} Wh, {:.3}g CO2",
            metrics.workload_type, workload_id, energy_wh, co2_emissions_g
        );

        Ok(metrics)
    }

    /// Record power sample for active workload
    pub fn record_power_sample(&self, workload_id: &str, cpu_utilization: f64, memory_mb: f64) -> Result<()> {
        if !self.config.enabled || workload_id.is_empty() {
            return Ok(());
        }

        let power_estimate = self.estimate_power_consumption(cpu_utilization, memory_mb);

        let mut sessions = self.active_sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(workload_id) {
            session.power_samples.push(power_estimate);
            session.cpu_samples.push(cpu_utilization);
            session.memory_samples.push(memory_mb);
            debug!("Recorded power sample for {}: {:.2}W (CPU: {:.1}%)", workload_id, power_estimate, cpu_utilization);
        }

        Ok(())
    }

    /// Estimate power consumption based on resource utilization
    fn estimate_power_consumption(&self, cpu_utilization: f64, _memory_mb: f64) -> f64 {
        let cpu_power = cpu_utilization * self.config.cpu_power_coefficient;
        let gpu_power = self.config.gpu_power_watts.unwrap_or(0.0);
        self.config.base_power_watts + cpu_power + gpu_power
    }

    /// Get current system resource utilization
    pub fn get_system_utilization(&self) -> Result<(f64, f64)> {
        // Read CPU utilization from /proc/stat (Linux)
        let cpu_utilization = self.read_cpu_utilization().unwrap_or(0.0);
        
        // Read memory usage from /proc/meminfo (Linux)
        let memory_mb = self.read_memory_usage_mb().unwrap_or(0.0);
        
        Ok((cpu_utilization, memory_mb))
    }

    /// Read CPU utilization from /proc/stat (Linux-specific)
    fn read_cpu_utilization(&self) -> Result<f64> {
        let stat_content = fs::read_to_string("/proc/stat")?;
        let cpu_line = stat_content.lines()
            .find(|line| line.starts_with("cpu "))
            .ok_or_else(|| anyhow!("CPU line not found in /proc/stat"))?;
        
        let fields: Vec<u64> = cpu_line.split_whitespace()
            .skip(1)
            .take(8)
            .map(|s| s.parse().unwrap_or(0))
            .collect();
        
        if fields.len() >= 4 {
            let idle = fields[3];
            let total: u64 = fields.iter().sum();
            let utilization = if total > 0 {
                ((total - idle) as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            Ok(utilization)
        } else {
            Err(anyhow!("Invalid CPU stats format"))
        }
    }

    /// Read memory usage from /proc/meminfo (Linux-specific)
    fn read_memory_usage_mb(&self) -> Result<f64> {
        let meminfo_content = fs::read_to_string("/proc/meminfo")?;
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        
        for line in meminfo_content.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        if total_kb > 0 {
            let used_mb = (total_kb - available_kb) as f64 / 1024.0;
            Ok(used_mb)
        } else {
            Err(anyhow!("Unable to parse memory information"))
        }
    }

    /// Get completed metrics for reporting
    pub fn get_completed_metrics(&self) -> Vec<EnergyConsumptionMetrics> {
        self.completed_metrics.lock().unwrap().clone()
    }

    /// Generate carbon footprint report
    pub fn generate_carbon_report(&self) -> CarbonFootprintReport {
        let metrics = self.get_completed_metrics();
        
        let total_energy_wh: f64 = metrics.iter().map(|m| m.energy_wh).sum();
        let total_co2_g: f64 = metrics.iter().map(|m| m.co2_emissions_g).sum();
        
        let mut workload_breakdown = HashMap::new();
        for metric in &metrics {
            let entry = workload_breakdown.entry(metric.workload_type.clone())
                .or_insert(WorkloadCarbonSummary {
                    workload_type: metric.workload_type.clone(),
                    total_energy_wh: 0.0,
                    total_co2_g: 0.0,
                    execution_count: 0,
                    avg_energy_per_execution: 0.0,
                });
            
            entry.total_energy_wh += metric.energy_wh;
            entry.total_co2_g += metric.co2_emissions_g;
            entry.execution_count += 1;
        }
        
        // Calculate averages
        for summary in workload_breakdown.values_mut() {
            summary.avg_energy_per_execution = 
                summary.total_energy_wh / summary.execution_count as f64;
        }
        
        CarbonFootprintReport {
            report_timestamp: Utc::now(),
            total_energy_wh,
            total_co2_emissions_g: total_co2_g,
            carbon_intensity_g_co2_kwh: self.config.carbon_intensity_g_co2_kwh,
            workload_breakdown: workload_breakdown.into_values().collect(),
            total_executions: metrics.len(),
            recommendations: self.generate_optimization_recommendations(&metrics),
        }
    }

    /// Generate optimization recommendations based on carbon metrics
    fn generate_optimization_recommendations(&self, metrics: &[EnergyConsumptionMetrics]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if metrics.is_empty() {
            return recommendations;
        }
        
        let avg_energy: f64 = metrics.iter().map(|m| m.energy_wh).sum::<f64>() / metrics.len() as f64;
        let avg_co2: f64 = metrics.iter().map(|m| m.co2_emissions_g).sum::<f64>() / metrics.len() as f64;
        
        if avg_energy > 10.0 {
            recommendations.push("Consider optimizing algorithm efficiency to reduce energy consumption per execution".to_string());
        }
        
        if avg_co2 > 5.0 {
            recommendations.push("High carbon footprint detected - consider running workloads during off-peak hours or in regions with cleaner energy".to_string());
        }
        
        // Check for genetic algorithm specific optimizations
        let ga_metrics: Vec<_> = metrics.iter()
            .filter(|m| m.workload_type == WorkloadType::GeneticAlgorithm)
            .collect();
        
        if !ga_metrics.is_empty() {
            let ga_avg_energy: f64 = ga_metrics.iter().map(|m| m.energy_wh).sum::<f64>() / ga_metrics.len() as f64;
            if ga_avg_energy > 5.0 {
                recommendations.push("Genetic algorithm consumes significant energy - consider reducing population size or generations".to_string());
            }
        }
        
        recommendations
    }

    /// Start background monitoring task
    pub async fn start_background_monitoring(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let interval = Duration::from_secs(self.config.sampling_interval_secs);
        
        loop {
            if let Ok((cpu_util, memory_mb)) = self.get_system_utilization() {
                let active_session_ids: Vec<String> = {
                    let sessions = self.active_sessions.lock().unwrap();
                    sessions.keys().cloned().collect()
                };
                
                for workload_id in active_session_ids {
                    if let Err(e) = self.record_power_sample(&workload_id, cpu_util, memory_mb) {
                        warn!("Failed to record power sample for {}: {}", workload_id, e);
                    }
                }
            }
            
            sleep(interval).await;
        }
    }
}

/// Summary of carbon metrics for a specific workload type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCarbonSummary {
    pub workload_type: WorkloadType,
    pub total_energy_wh: f64,
    pub total_co2_g: f64,
    pub execution_count: usize,
    pub avg_energy_per_execution: f64,
}

/// Comprehensive carbon footprint report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonFootprintReport {
    pub report_timestamp: DateTime<Utc>,
    pub total_energy_wh: f64,
    pub total_co2_emissions_g: f64,
    pub carbon_intensity_g_co2_kwh: f64,
    pub workload_breakdown: Vec<WorkloadCarbonSummary>,
    pub total_executions: usize,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_energy_tracking_session() {
        let config = CarbonAwarenessConfig::default();
        let collector = CarbonAwarenessCollector::new(config, None).unwrap();
        
        let workload_id = collector.start_tracking(WorkloadType::GeneticAlgorithm, None);
        assert!(!workload_id.is_empty());
        
        // Simulate some work with power samples
        collector.record_power_sample(&workload_id, 75.0, 1024.0).unwrap();
        collector.record_power_sample(&workload_id, 80.0, 1100.0).unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        let metrics = collector.stop_tracking(&workload_id).unwrap();
        assert_eq!(metrics.workload_type, WorkloadType::GeneticAlgorithm);
        assert!(metrics.energy_wh > 0.0);
        assert!(metrics.co2_emissions_g > 0.0);
    }

    #[test]
    fn test_power_estimation() {
        let config = CarbonAwarenessConfig::default();
        let collector = CarbonAwarenessCollector::new(config, None).unwrap();
        
        let power = collector.estimate_power_consumption(50.0, 1024.0);
        assert!(power > 50.0); // Should be base + CPU contribution
    }

    #[test]
    fn test_carbon_report_generation() {
        let config = CarbonAwarenessConfig::default();
        let collector = CarbonAwarenessCollector::new(config, None).unwrap();
        
        let report = collector.generate_carbon_report();
        assert_eq!(report.total_energy_wh, 0.0); // No metrics yet
        assert_eq!(report.workload_breakdown.len(), 0);
    }
}