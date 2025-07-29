//! Memory analysis reporting for performance monitoring and optimization

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Report containing memory usage metrics and optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysisReport {
    /// Target memory limit in MB
    pub memory_limit_mb: usize,
    
    /// Initial memory usage before analysis in MB
    pub initial_memory_mb: usize,
    
    /// Final memory usage after analysis in MB
    pub final_memory_mb: usize,
    
    /// Peak memory usage during analysis in MB
    pub peak_memory_mb: usize,
    
    /// Total analysis duration
    pub analysis_duration: Duration,
    
    /// Whether memory optimization was effective (peak < limit)
    pub memory_optimization_effective: bool,
    
    /// Whether memory limit was exceeded at any point
    pub memory_limit_exceeded: bool,
    
    /// Whether analysis scope was reduced due to memory constraints
    pub scope_reduced: bool,
    
    /// List of fallback strategies that were applied
    pub applied_strategies: Vec<String>,
    
    /// Memory optimization recommendations
    pub recommendations: Vec<String>,
    
    /// Detailed memory breakdowns by analysis phase
    pub phase_breakdowns: Vec<PhaseMemoryBreakdown>,
}

/// Memory usage breakdown for a specific analysis phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseMemoryBreakdown {
    /// Name of the analysis phase
    pub phase_name: String,
    
    /// Memory usage at start of phase in MB
    pub start_memory_mb: usize,
    
    /// Memory usage at end of phase in MB
    pub end_memory_mb: usize,
    
    /// Duration of this phase
    pub duration: Duration,
    
    /// Memory allocations during this phase in MB
    pub allocations_mb: usize,
    
    /// Memory deallocations during this phase in MB
    pub deallocations_mb: usize,
}

impl MemoryAnalysisReport {
    /// Create a new memory analysis report
    pub fn new(memory_limit_mb: usize) -> Self {
        Self {
            memory_limit_mb,
            initial_memory_mb: 0,
            final_memory_mb: 0,
            peak_memory_mb: 0,
            analysis_duration: Duration::from_secs(0),
            memory_optimization_effective: false,
            memory_limit_exceeded: false,
            scope_reduced: false,
            applied_strategies: Vec::new(),
            recommendations: Vec::new(),
            phase_breakdowns: Vec::new(),
        }
    }
    
    /// Add a phase memory breakdown
    pub fn add_phase_breakdown(&mut self, breakdown: PhaseMemoryBreakdown) {
        self.phase_breakdowns.push(breakdown);
    }
    
    /// Calculate memory efficiency percentage
    pub fn memory_efficiency_percentage(&self) -> f64 {
        if self.memory_limit_mb == 0 {
            return 0.0;
        }
        
        let efficiency = (self.memory_limit_mb as f64 - self.peak_memory_mb as f64) 
            / self.memory_limit_mb as f64 * 100.0;
        efficiency.max(0.0).min(100.0)
    }
    
    /// Get total memory delta (final - initial)
    pub fn memory_delta_mb(&self) -> i32 {
        self.final_memory_mb as i32 - self.initial_memory_mb as i32
    }
    
    /// Check if memory usage was within acceptable bounds
    pub fn is_memory_usage_acceptable(&self) -> bool {
        !self.memory_limit_exceeded && 
        self.peak_memory_mb <= self.memory_limit_mb &&
        !self.scope_reduced
    }
    
    /// Generate a summary string for logging
    pub fn summary(&self) -> String {
        format!(
            "Memory Analysis: {}MB initial → {}MB peak → {}MB final (limit: {}MB, efficiency: {:.1}%)",
            self.initial_memory_mb,
            self.peak_memory_mb,
            self.final_memory_mb,
            self.memory_limit_mb,
            self.memory_efficiency_percentage()
        )
    }
    
    /// Export report as JSON for external tools
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Load report from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl PhaseMemoryBreakdown {
    /// Create a new phase breakdown
    pub fn new(phase_name: String, start_memory_mb: usize) -> Self {
        Self {
            phase_name,
            start_memory_mb,
            end_memory_mb: start_memory_mb,
            duration: Duration::from_secs(0),
            allocations_mb: 0,
            deallocations_mb: 0,
        }
    }
    
    /// Finish the phase breakdown with end metrics
    pub fn finish(&mut self, end_memory_mb: usize, duration: Duration) {
        self.end_memory_mb = end_memory_mb;
        self.duration = duration;
        
        // Calculate allocations/deallocations
        if end_memory_mb > self.start_memory_mb {
            self.allocations_mb = end_memory_mb - self.start_memory_mb;
        } else {
            self.deallocations_mb = self.start_memory_mb - end_memory_mb;
        }
    }
    
    /// Get memory delta for this phase
    pub fn memory_delta_mb(&self) -> i32 {
        self.end_memory_mb as i32 - self.start_memory_mb as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_report_creation() {
        let report = MemoryAnalysisReport::new(1024);
        assert_eq!(report.memory_limit_mb, 1024);
        assert_eq!(report.initial_memory_mb, 0);
        assert!(!report.memory_optimization_effective);
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        let mut report = MemoryAnalysisReport::new(1000);
        report.peak_memory_mb = 800;
        
        let efficiency = report.memory_efficiency_percentage();
        assert_eq!(efficiency, 20.0); // (1000 - 800) / 1000 * 100
    }

    #[test]
    fn test_memory_delta() {
        let mut report = MemoryAnalysisReport::new(1000);
        report.initial_memory_mb = 100;
        report.final_memory_mb = 150;
        
        assert_eq!(report.memory_delta_mb(), 50);
    }

    #[test]
    fn test_phase_breakdown() {
        let mut phase = PhaseMemoryBreakdown::new("test_phase".to_string(), 100);
        phase.finish(150, Duration::from_secs(10));
        
        assert_eq!(phase.memory_delta_mb(), 50);
        assert_eq!(phase.allocations_mb, 50);
        assert_eq!(phase.deallocations_mb, 0);
    }

    #[test]
    fn test_json_serialization() {
        let report = MemoryAnalysisReport::new(1024);
        let json = report.to_json().unwrap();
        let deserialized = MemoryAnalysisReport::from_json(&json).unwrap();
        
        assert_eq!(report.memory_limit_mb, deserialized.memory_limit_mb);
    }
}