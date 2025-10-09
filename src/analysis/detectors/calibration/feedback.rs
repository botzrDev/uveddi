//! User feedback collection and auto-calibration system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// User verdict on a detected issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserVerdict {
    /// Issue is a true positive
    TruePositive,
    /// Issue is a false positive
    FalsePositive,
    /// User is uncertain
    NotSure,
    /// Issue is valid but severity is too high
    SeverityTooHigh,
    /// Issue is valid but severity is too low
    SeverityTooLow,
}

impl UserVerdict {
    /// Check if this is a positive verdict
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            Self::TruePositive | Self::SeverityTooHigh | Self::SeverityTooLow
        )
    }

    /// Check if this is a negative verdict
    pub fn is_negative(&self) -> bool {
        matches!(self, Self::FalsePositive)
    }

    /// Check if severity needs adjustment
    pub fn needs_severity_adjustment(&self) -> bool {
        matches!(self, Self::SeverityTooHigh | Self::SeverityTooLow)
    }
}

/// Context information about an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueContext {
    /// Issue ID
    pub issue_id: String,
    /// Detector that reported the issue
    pub detector: String,
    /// File path
    pub file_path: String,
    /// Line number
    pub line: u32,
    /// Reported confidence score
    pub confidence: f64,
    /// Reported severity score
    pub severity: u32,
    /// Issue description
    pub description: String,
    /// Code snippet
    pub code_snippet: String,
}

/// User feedback entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    /// Unique feedback ID
    pub id: String,
    /// Issue context
    pub issue: IssueContext,
    /// User's verdict
    pub verdict: UserVerdict,
    /// Optional user comment
    pub comment: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// User ID (optional)
    pub user_id: Option<String>,
}

impl FeedbackEntry {
    /// Create a new feedback entry
    pub fn new(issue: IssueContext, verdict: UserVerdict) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: format!("{}_{}", issue.issue_id, timestamp),
            issue,
            verdict,
            comment: None,
            timestamp,
            user_id: None,
        }
    }

    /// Add a comment
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Add user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

/// Aggregated feedback statistics for a detector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectorFeedbackStats {
    /// Detector name
    pub detector: String,
    /// Total feedback count
    pub total_feedback: usize,
    /// True positive count
    pub true_positives: usize,
    /// False positive count
    pub false_positives: usize,
    /// Uncertain count
    pub uncertain: usize,
    /// Severity too high count
    pub severity_too_high: usize,
    /// Severity too low count
    pub severity_too_low: usize,
    /// Current false positive rate
    pub false_positive_rate: f64,
}

impl DetectorFeedbackStats {
    /// Create new stats for a detector
    pub fn new(detector: impl Into<String>) -> Self {
        Self {
            detector: detector.into(),
            ..Default::default()
        }
    }

    /// Update stats with new feedback
    pub fn add_feedback(&mut self, feedback: &FeedbackEntry) {
        self.total_feedback += 1;

        match feedback.verdict {
            UserVerdict::TruePositive => self.true_positives += 1,
            UserVerdict::FalsePositive => self.false_positives += 1,
            UserVerdict::NotSure => self.uncertain += 1,
            UserVerdict::SeverityTooHigh => {
                self.severity_too_high += 1;
                self.true_positives += 1; // Still a valid issue
            }
            UserVerdict::SeverityTooLow => {
                self.severity_too_low += 1;
                self.true_positives += 1; // Still a valid issue
            }
        }

        // Recalculate false positive rate
        if self.total_feedback > 0 {
            self.false_positive_rate = self.false_positives as f64 / self.total_feedback as f64;
        }
    }

    /// Check if false positive rate is too high
    pub fn has_high_fp_rate(&self, threshold: f64) -> bool {
        self.false_positive_rate > threshold
    }

    /// Check if we have enough feedback for reliable calibration
    pub fn has_sufficient_feedback(&self, min_count: usize) -> bool {
        self.total_feedback >= min_count
    }

    /// Get precision estimate
    pub fn estimated_precision(&self) -> f64 {
        if self.total_feedback == 0 {
            return 0.0;
        }
        self.true_positives as f64 / self.total_feedback as f64
    }
}

/// Feedback collector and analyzer
#[derive(Debug, Clone)]
pub struct FeedbackCollector {
    /// Collected feedback entries
    pub feedback: Vec<FeedbackEntry>,
    /// Stats per detector
    pub detector_stats: HashMap<String, DetectorFeedbackStats>,
    /// Minimum feedback count for calibration
    pub min_feedback_count: usize,
    /// Maximum acceptable false positive rate
    pub max_fp_rate: f64,
}

impl FeedbackCollector {
    /// Create a new feedback collector
    pub fn new() -> Self {
        Self {
            feedback: Vec::new(),
            detector_stats: HashMap::new(),
            min_feedback_count: 100,
            max_fp_rate: 0.2,
        }
    }

    /// Add feedback
    pub fn add_feedback(&mut self, feedback: FeedbackEntry) {
        let detector = feedback.issue.detector.clone();

        // Update stats
        let stats = self
            .detector_stats
            .entry(detector.clone())
            .or_insert_with(|| DetectorFeedbackStats::new(detector));
        stats.add_feedback(&feedback);

        // Store feedback
        self.feedback.push(feedback);
    }

    /// Get feedback for a specific detector
    pub fn get_detector_feedback(&self, detector: &str) -> Vec<&FeedbackEntry> {
        self.feedback
            .iter()
            .filter(|f| f.issue.detector == detector)
            .collect()
    }

    /// Get stats for a detector
    pub fn get_detector_stats(&self, detector: &str) -> Option<&DetectorFeedbackStats> {
        self.detector_stats.get(detector)
    }

    /// Check if detector needs threshold adjustment
    pub fn needs_adjustment(&self, detector: &str) -> bool {
        if let Some(stats) = self.get_detector_stats(detector) {
            stats.has_sufficient_feedback(self.min_feedback_count)
                && stats.has_high_fp_rate(self.max_fp_rate)
        } else {
            false
        }
    }

    /// Get all detectors needing adjustment
    pub fn detectors_needing_adjustment(&self) -> Vec<String> {
        self.detector_stats
            .keys()
            .filter(|d| self.needs_adjustment(d))
            .cloned()
            .collect()
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-calibration engine
#[derive(Debug, Clone)]
pub struct AutoCalibrator {
    /// Minimum feedback count required
    pub min_feedback_count: usize,
    /// Confidence threshold for making adjustments (0.0-1.0)
    pub confidence_threshold: f64,
    /// Maximum adjustment magnitude (0.0-1.0)
    pub max_adjustment: f64,
    /// Target false positive rate
    pub target_fp_rate: f64,
}

impl AutoCalibrator {
    /// Create a new auto-calibrator with conservative defaults
    pub fn new() -> Self {
        Self {
            min_feedback_count: 100,
            confidence_threshold: 0.95,
            max_adjustment: 0.1, // 10% max adjustment
            target_fp_rate: 0.15,
        }
    }

    /// Propose threshold adjustments based on feedback
    pub fn propose_adjustments(
        &self,
        collector: &FeedbackCollector,
    ) -> Vec<ThresholdProposal> {
        let mut proposals = Vec::new();

        for (detector, stats) in &collector.detector_stats {
            if !stats.has_sufficient_feedback(self.min_feedback_count) {
                continue;
            }

            // Calculate proposed adjustment
            if let Some(proposal) = self.calculate_adjustment(detector, stats) {
                proposals.push(proposal);
            }
        }

        proposals
    }

    /// Calculate adjustment for a detector
    fn calculate_adjustment(
        &self,
        detector: &str,
        stats: &DetectorFeedbackStats,
    ) -> Option<ThresholdProposal> {
        // Too many false positives - increase threshold
        if stats.false_positive_rate > self.target_fp_rate * 1.5 {
            let adjustment = self.calculate_adjustment_magnitude(stats.false_positive_rate);
            return Some(ThresholdProposal {
                detector: detector.to_string(),
                current_threshold: 0.7, // Would come from config
                proposed_threshold: 0.7 * (1.0 + adjustment),
                adjustment_type: AdjustmentType::IncreaseThreshold,
                reason: format!(
                    "False positive rate {:.1}% exceeds target {:.1}%",
                    stats.false_positive_rate * 100.0,
                    self.target_fp_rate * 100.0
                ),
                confidence: self.calculate_confidence(stats),
                supporting_feedback: stats.total_feedback,
            });
        }

        // Severity calibration issues
        if stats.severity_too_high as f64 / stats.total_feedback as f64 > 0.3 {
            return Some(ThresholdProposal {
                detector: detector.to_string(),
                current_threshold: 0.7,
                proposed_threshold: 0.7,
                adjustment_type: AdjustmentType::DecreaseSeverity,
                reason: "Over 30% of issues marked as too severe".to_string(),
                confidence: self.calculate_confidence(stats),
                supporting_feedback: stats.total_feedback,
            });
        }

        if stats.severity_too_low as f64 / stats.total_feedback as f64 > 0.3 {
            return Some(ThresholdProposal {
                detector: detector.to_string(),
                current_threshold: 0.7,
                proposed_threshold: 0.7,
                adjustment_type: AdjustmentType::IncreaseSeverity,
                reason: "Over 30% of issues marked as not severe enough".to_string(),
                confidence: self.calculate_confidence(stats),
                supporting_feedback: stats.total_feedback,
            });
        }

        None
    }

    /// Calculate adjustment magnitude based on FP rate
    fn calculate_adjustment_magnitude(&self, fp_rate: f64) -> f64 {
        let excess = fp_rate - self.target_fp_rate;
        (excess * 0.5).min(self.max_adjustment) // Conservative scaling
    }

    /// Calculate confidence in the adjustment
    fn calculate_confidence(&self, stats: &DetectorFeedbackStats) -> f64 {
        // Confidence increases with more feedback
        let count_factor = (stats.total_feedback as f64 / 200.0).min(1.0);

        // Confidence increases with clearer signal
        let signal_strength = if stats.total_feedback > 0 {
            (stats.false_positives.abs_diff(stats.true_positives) as f64
                / stats.total_feedback as f64)
                .min(1.0)
        } else {
            0.0
        };

        (count_factor * 0.5 + signal_strength * 0.5).min(1.0)
    }
}

impl Default for AutoCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Proposed threshold adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdProposal {
    /// Detector name
    pub detector: String,
    /// Current threshold
    pub current_threshold: f64,
    /// Proposed new threshold
    pub proposed_threshold: f64,
    /// Type of adjustment
    pub adjustment_type: AdjustmentType,
    /// Reason for adjustment
    pub reason: String,
    /// Confidence in the proposal (0.0-1.0)
    pub confidence: f64,
    /// Number of feedback entries supporting this
    pub supporting_feedback: usize,
}

impl ThresholdProposal {
    /// Check if this proposal is high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.9
    }

    /// Get percentage change
    pub fn percent_change(&self) -> f64 {
        ((self.proposed_threshold - self.current_threshold) / self.current_threshold) * 100.0
    }
}

/// Type of threshold adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentType {
    IncreaseThreshold,
    DecreaseThreshold,
    IncreaseSeverity,
    DecreaseSeverity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_verdict() {
        assert!(UserVerdict::TruePositive.is_positive());
        assert!(UserVerdict::FalsePositive.is_negative());
        assert!(UserVerdict::SeverityTooHigh.needs_severity_adjustment());
    }

    #[test]
    fn test_feedback_stats() {
        let mut stats = DetectorFeedbackStats::new("test_detector");

        let issue = IssueContext {
            issue_id: "1".to_string(),
            detector: "test_detector".to_string(),
            file_path: "test.rs".to_string(),
            line: 10,
            confidence: 0.8,
            severity: 75,
            description: "Test issue".to_string(),
            code_snippet: "fn test() {}".to_string(),
        };

        let feedback = FeedbackEntry::new(issue, UserVerdict::FalsePositive);
        stats.add_feedback(&feedback);

        assert_eq!(stats.total_feedback, 1);
        assert_eq!(stats.false_positives, 1);
        assert_eq!(stats.false_positive_rate, 1.0);
    }

    #[test]
    fn test_feedback_collector() {
        let mut collector = FeedbackCollector::new();
        collector.min_feedback_count = 2;
        collector.max_fp_rate = 0.5;

        let issue = IssueContext {
            issue_id: "1".to_string(),
            detector: "test_detector".to_string(),
            file_path: "test.rs".to_string(),
            line: 10,
            confidence: 0.8,
            severity: 75,
            description: "Test issue".to_string(),
            code_snippet: "fn test() {}".to_string(),
        };

        // Add two false positives
        collector.add_feedback(FeedbackEntry::new(issue.clone(), UserVerdict::FalsePositive));
        collector.add_feedback(FeedbackEntry::new(issue, UserVerdict::FalsePositive));

        assert!(collector.needs_adjustment("test_detector"));
    }

    #[test]
    fn test_auto_calibrator() {
        let calibrator = AutoCalibrator::new();
        calibrator.min_feedback_count = 2;

        let mut collector = FeedbackCollector::new();

        let issue = IssueContext {
            issue_id: "1".to_string(),
            detector: "test_detector".to_string(),
            file_path: "test.rs".to_string(),
            line: 10,
            confidence: 0.8,
            severity: 75,
            description: "Test issue".to_string(),
            code_snippet: "fn test() {}".to_string(),
        };

        // Add feedback with high FP rate
        for i in 0..10 {
            let mut issue = issue.clone();
            issue.issue_id = i.to_string();
            collector.add_feedback(FeedbackEntry::new(issue, UserVerdict::FalsePositive));
        }

        let proposals = calibrator.propose_adjustments(&collector);
        assert!(!proposals.is_empty());

        if let Some(proposal) = proposals.first() {
            assert_eq!(proposal.adjustment_type, AdjustmentType::IncreaseThreshold);
            assert!(proposal.proposed_threshold > proposal.current_threshold);
        }
    }
}
