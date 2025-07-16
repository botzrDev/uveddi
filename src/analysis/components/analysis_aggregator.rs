//! Analysis aggregator implementation
//!
//! Collects and aggregates analysis results from multiple sources.

use super::traits::{AnalysisAggregator as AnalysisAggregatorTrait, AggregationStats};
use crate::database::models::ArchitecturalIssue;

use std::collections::HashMap;
use std::sync::Mutex;

/// Analysis result aggregator implementation
pub struct AnalysisAggregator {
    findings: Mutex<Vec<ArchitecturalIssue>>,
    stats: Mutex<InternalStats>,
}

#[derive(Debug, Default)]
struct InternalStats {
    files_processed: usize,
    findings_by_type: HashMap<String, usize>,
}

impl AnalysisAggregator {
    /// Creates a new analysis aggregator
    pub fn new() -> Self {
        Self {
            findings: Mutex::new(Vec::new()),
            stats: Mutex::new(InternalStats::default()),
        }
    }
    
    /// Updates internal statistics when a finding is recorded
    fn update_stats(&self, issues: &[ArchitecturalIssue]) {
        let mut stats = self.stats.lock().unwrap();
        
        for issue in issues {
            let issue_type = match issue.anti_pattern_type_id {
                1 => "god_object",
                2 => "dead_code", 
                3 => "cyclic_dependency",
                _ => "unknown",
            }.to_string();
            
            *stats.findings_by_type.entry(issue_type).or_insert(0) += 1;
        }
    }
    
    /// Records that a file has been processed
    pub fn record_file_processed(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.files_processed += 1;
    }
    
    /// Gets the current count of findings
    pub fn get_findings_count(&self) -> usize {
        self.findings.lock().unwrap().len()
    }
    
    /// Gets findings of a specific type (by anti_pattern_type_id)
    pub fn get_findings_by_type_id(&self, type_id: i64) -> Vec<ArchitecturalIssue> {
        self.findings.lock().unwrap()
            .iter()
            .filter(|issue| issue.anti_pattern_type_id == type_id)
            .cloned()
            .collect()
    }
    
    /// Gets findings of a specific type (by type name - helper for tests)
    pub fn get_findings_by_type(&self, issue_type: &str) -> Vec<ArchitecturalIssue> {
        let type_id = match issue_type {
            "god_object" => 1,
            "dead_code" => 2,
            "cyclic_dependency" => 3,
            _ => 99,
        };
        self.get_findings_by_type_id(type_id)
    }
    
    /// Gets findings for a specific file
    pub fn get_findings_for_file(&self, file_path: &str) -> Vec<ArchitecturalIssue> {
        self.findings.lock().unwrap()
            .iter()
            .filter(|issue| issue.file_path == file_path)
            .cloned()
            .collect()
    }
    
    /// Filters findings based on severity (using numeric level)
    pub fn get_findings_by_severity(&self, min_severity: i32) -> Vec<ArchitecturalIssue> {
        self.findings.lock().unwrap()
            .iter()
            .filter(|issue| {
                let severity_level = match issue.severity.as_str() {
                    "low" => 1,
                    "medium" => 2,
                    "high" => 3,
                    "critical" => 4,
                    _ => 0,
                };
                severity_level >= min_severity
            })
            .cloned()
            .collect()
    }
}

impl AnalysisAggregatorTrait for AnalysisAggregator {
    fn record_finding(&self, issue: ArchitecturalIssue) {
        self.update_stats(&[issue.clone()]);
        self.findings.lock().unwrap().push(issue);
    }
    
    fn record_findings(&self, issues: Vec<ArchitecturalIssue>) {
        if issues.is_empty() {
            return;
        }
        
        self.update_stats(&issues);
        self.findings.lock().unwrap().extend(issues);
    }
    
    fn get_findings(&self) -> Vec<ArchitecturalIssue> {
        self.findings.lock().unwrap().clone()
    }
    
    fn clear_findings(&self) {
        self.findings.lock().unwrap().clear();
        let mut stats = self.stats.lock().unwrap();
        stats.findings_by_type.clear();
        stats.files_processed = 0;
    }
    
    fn record_file_processed(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.files_processed += 1;
    }
    
    fn get_stats(&self) -> AggregationStats {
        let findings = self.findings.lock().unwrap();
        let stats = self.stats.lock().unwrap();
        
        AggregationStats {
            total_findings: findings.len(),
            findings_by_type: stats.findings_by_type.clone(),
            files_processed: stats.files_processed,
        }
    }
}

impl Default for AnalysisAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{ArchitecturalIssue, AntiPatternType};
    use chrono::Utc;
    
    fn create_test_issue(issue_type: &str, file_path: &str, severity_level: i32) -> ArchitecturalIssue {
        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: match issue_type {
                "god_object" => 1,
                "dead_code" => 2,
                "cyclic_dependency" => 3,
                _ => 99,
            },
            file_path: file_path.to_string(),
            start_line: Some(10),
            end_line: Some(15),
            severity: match severity_level {
                1 => "low".to_string(),
                2 => "medium".to_string(),
                3 => "high".to_string(),
                4 => "critical".to_string(),
                _ => "medium".to_string(),
            },
            description: format!("Test {} issue in {}", issue_type, file_path),
            code_snippet: None,
            ai_explanation: None,
        }
    }
    
    #[test]
    fn test_aggregator_basic_operations() {
        let aggregator = AnalysisAggregator::new();
        
        // Initially empty
        assert_eq!(aggregator.get_findings_count(), 0);
        
        // Record a finding
        let issue = create_test_issue("god_object", "src/main.rs", 3);
        aggregator.record_finding(issue);
        
        assert_eq!(aggregator.get_findings_count(), 1);
        
        // Record multiple findings
        let issues = vec![
            create_test_issue("dead_code", "src/lib.rs", 2),
            create_test_issue("cyclic_dependency", "src/mod.rs", 4),
        ];
        aggregator.record_findings(issues);
        
        assert_eq!(aggregator.get_findings_count(), 3);
    }
    
    #[test]
    fn test_aggregator_statistics() {
        let aggregator = AnalysisAggregator::new();
        
        // Record various findings
        aggregator.record_finding(create_test_issue("god_object", "src/main.rs", 3));
        aggregator.record_finding(create_test_issue("god_object", "src/lib.rs", 4));
        aggregator.record_finding(create_test_issue("dead_code", "src/utils.rs", 1));
        
        // Record file processing
        aggregator.record_file_processed();
        aggregator.record_file_processed();
        aggregator.record_file_processed();
        
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 3);
        assert_eq!(stats.files_processed, 3);
        assert_eq!(stats.findings_by_type.get("god_object"), Some(&2));
        assert_eq!(stats.findings_by_type.get("dead_code"), Some(&1));
    }
    
    #[test]
    fn test_aggregator_filtering() {
        let aggregator = AnalysisAggregator::new();
        
        // Add test data
        aggregator.record_finding(create_test_issue("god_object", "src/main.rs", 3));
        aggregator.record_finding(create_test_issue("dead_code", "src/main.rs", 1));
        aggregator.record_finding(create_test_issue("god_object", "src/lib.rs", 4));
        
        // Test filtering by type
        let god_object_issues = aggregator.get_findings_by_type("god_object");
        assert_eq!(god_object_issues.len(), 2);
        
        // Test filtering by file
        let main_rs_issues = aggregator.get_findings_for_file("src/main.rs");
        assert_eq!(main_rs_issues.len(), 2);
        
        // Test filtering by severity
        let high_severity_issues = aggregator.get_findings_by_severity(3);
        assert_eq!(high_severity_issues.len(), 2); // Severity 3 and 4
        
        let very_high_severity_issues = aggregator.get_findings_by_severity(4);
        assert_eq!(very_high_severity_issues.len(), 1); // Only severity 4
    }
    
    #[test]
    fn test_aggregator_clear() {
        let aggregator = AnalysisAggregator::new();
        
        // Add some data
        aggregator.record_finding(create_test_issue("god_object", "src/main.rs", 3));
        aggregator.record_file_processed();
        
        assert_eq!(aggregator.get_findings_count(), 1);
        
        // Clear and verify
        aggregator.clear_findings();
        assert_eq!(aggregator.get_findings_count(), 0);
        
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 0);
        assert_eq!(stats.files_processed, 0);
        assert!(stats.findings_by_type.is_empty());
    }
    
    #[test]
    fn test_aggregator_concurrent_access() {
        use std::sync::Arc;
        use std::thread;
        
        let aggregator = Arc::new(AnalysisAggregator::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to record findings concurrently
        for i in 0..10 {
            let aggregator_clone = Arc::clone(&aggregator);
            let handle = thread::spawn(move || {
                let issue = create_test_issue("test_issue", &format!("file_{}.rs", i), 1);
                aggregator_clone.record_finding(issue);
                aggregator_clone.record_file_processed();
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all findings were recorded
        assert_eq!(aggregator.get_findings_count(), 10);
        let stats = aggregator.get_stats();
        assert_eq!(stats.files_processed, 10);
    }
}