//! Progress reporting system for Uveddi
//!
//! Provides real-time progress feedback for long-running operations
//! including analysis, parsing, and report generation.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Phase of analysis progress
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisPhase {
    /// Discovering and cataloging source files
    Discovery,
    /// Parsing files into ASTs
    Parsing,
    /// Building dependency graphs
    DependencyAnalysis,
    /// Running detectors and analyzers
    Analysis,
    /// Generating AI insights (if enabled)
    AiAnalysis,
    /// Generating final reports
    ReportGeneration,
    /// Analysis completed
    Complete,
}

impl AnalysisPhase {
    /// Returns a human-readable description of the analysis phase
    pub fn description(&self) -> &'static str {
        match self {
            AnalysisPhase::Discovery => "Discovering source files",
            AnalysisPhase::Parsing => "Parsing source code",
            AnalysisPhase::DependencyAnalysis => "Analyzing dependencies",
            AnalysisPhase::Analysis => "Running analysis detectors",
            AnalysisPhase::AiAnalysis => "Generating AI insights",
            AnalysisPhase::ReportGeneration => "Generating reports",
            AnalysisPhase::Complete => "Analysis complete",
        }
    }

    /// Returns an emoji representing the analysis phase
    pub fn emoji(&self) -> &'static str {
        match self {
            AnalysisPhase::Discovery => "🔍",
            AnalysisPhase::Parsing => "📖",
            AnalysisPhase::DependencyAnalysis => "🕸️",
            AnalysisPhase::Analysis => "🔬",
            AnalysisPhase::AiAnalysis => "🤖",
            AnalysisPhase::ReportGeneration => "📄",
            AnalysisPhase::Complete => "✅",
        }
    }
}

/// Progress information for a specific analysis phase
#[derive(Debug, Clone)]
pub struct PhaseProgress {
    /// The current analysis phase
    pub phase: AnalysisPhase,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f32, // 0.0 to 1.0
    /// Currently processing item (file, module, etc.)
    pub current_item: Option<String>,
    /// Number of items processed so far
    pub items_processed: usize,
    /// Total number of items to process (if known)
    pub total_items: Option<usize>,
    /// Time elapsed in this phase
    pub elapsed_time: Duration,
    /// Estimated time remaining (if calculable)
    pub estimated_remaining: Option<Duration>,
}

impl PhaseProgress {
    /// Creates a new progress tracker for the given phase
    pub fn new(phase: AnalysisPhase) -> Self {
        Self {
            phase,
            progress: 0.0,
            current_item: None,
            items_processed: 0,
            total_items: None,
            elapsed_time: Duration::ZERO,
            estimated_remaining: None,
        }
    }

    /// Sets the total number of items to process
    pub fn with_total_items(mut self, total: usize) -> Self {
        self.total_items = Some(total);
        self
    }

    /// Updates progress information with current status
    pub fn update_progress(
        &mut self,
        processed: usize,
        current_item: Option<String>,
        start_time: Instant,
    ) {
        self.items_processed = processed;
        self.current_item = current_item;
        self.elapsed_time = start_time.elapsed();

        if let Some(total) = self.total_items {
            if total > 0 {
                self.progress = processed as f32 / total as f32;
            }

            // Estimate remaining time based on current throughput
            if processed > 0 && self.progress < 1.0 {
                let avg_time_per_item = self.elapsed_time / processed as u32;
                let remaining_items = total - processed;
                self.estimated_remaining = Some(avg_time_per_item * remaining_items as u32);
            }
        }
    }
}

/// Progress reporter trait for different output formats
pub trait ProgressReporter: Send + Sync {
    /// Report progress for the current phase
    fn report_phase(&self, progress: &PhaseProgress);

    /// Report overall progress across all phases
    fn report_overall(&self, phase: &AnalysisPhase, overall_progress: f32);

    /// Report completion
    fn report_complete(&self, total_time: Duration);

    /// Report error
    fn report_error(&self, phase: &AnalysisPhase, error: &str);
}

/// Terminal progress reporter with rich formatting
pub struct TerminalProgressReporter {
    show_details: bool,
    last_update: Arc<Mutex<Instant>>,
    spinner_index: Arc<Mutex<usize>>,
}

impl TerminalProgressReporter {
    /// Creates a new terminal progress reporter
    pub fn new(show_details: bool) -> Self {
        Self {
            show_details,
            last_update: Arc::new(Mutex::new(Instant::now())),
            spinner_index: Arc::new(Mutex::new(0)),
        }
    }

    fn should_update(&self) -> bool {
        let mut last = self.last_update.lock().unwrap();
        let now = Instant::now();
        if now.duration_since(*last) > Duration::from_millis(100) {
            *last = now;
            true
        } else {
            false
        }
    }

    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        if total_seconds < 60 {
            format!("{}s", total_seconds)
        } else if total_seconds < 3600 {
            format!("{}m {}s", total_seconds / 60, total_seconds % 60)
        } else {
            format!("{}h {}m", total_seconds / 3600, (total_seconds % 3600) / 60)
        }
    }

    fn format_progress_bar(progress: f32, width: usize) -> String {
        let filled = (progress * width as f32) as usize;
        let empty = width.saturating_sub(filled);

        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

impl ProgressReporter for TerminalProgressReporter {
    fn report_phase(&self, progress: &PhaseProgress) {
        // Simple spinner animation
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        let mut idx = self.spinner_index.lock().unwrap();
        *idx = (*idx + 1) % spinner_frames.len();
        let spinner = spinner_frames[*idx];

        eprint!("\r  {} {} {}...", progress.phase.emoji(), spinner, progress.phase.description());

        use std::io::{self, Write};
        let _ = io::stderr().flush();
    }

    fn report_overall(&self, _phase: &AnalysisPhase, _overall_progress: f32) {
        // Skip overall progress to keep it simple
    }

    fn report_complete(&self, _total_time: Duration) {
        eprintln!("\r  ✅ Analysis complete!                                             ");
    }

    fn report_error(&self, phase: &AnalysisPhase, error: &str) {
        eprintln!("\r  ❌ Error in {}: {}                    ", phase.description(), error);
    }
}

/// Silent progress reporter (no output)
pub struct SilentProgressReporter;

impl ProgressReporter for SilentProgressReporter {
    fn report_phase(&self, _progress: &PhaseProgress) {}
    fn report_overall(&self, _phase: &AnalysisPhase, _overall_progress: f32) {}
    fn report_complete(&self, _total_time: Duration) {}
    fn report_error(&self, _phase: &AnalysisPhase, _error: &str) {}
}

/// JSON progress reporter for programmatic consumption
pub struct JsonProgressReporter;

impl ProgressReporter for JsonProgressReporter {
    fn report_phase(&self, progress: &PhaseProgress) {
        let json = serde_json::json!({
            "type": "phase_progress",
            "phase": format!("{:?}", progress.phase),
            "progress": progress.progress,
            "current_item": progress.current_item,
            "items_processed": progress.items_processed,
            "total_items": progress.total_items,
            "elapsed_seconds": progress.elapsed_time.as_secs(),
            "estimated_remaining_seconds": progress.estimated_remaining.map(|d| d.as_secs())
        });
        println!("{}", json);
    }

    fn report_overall(&self, phase: &AnalysisPhase, overall_progress: f32) {
        let json = serde_json::json!({
            "type": "overall_progress",
            "phase": format!("{:?}", phase),
            "progress": overall_progress
        });
        println!("{}", json);
    }

    fn report_complete(&self, total_time: Duration) {
        let json = serde_json::json!({
            "type": "complete",
            "total_time_seconds": total_time.as_secs()
        });
        println!("{}", json);
    }

    fn report_error(&self, phase: &AnalysisPhase, error: &str) {
        let json = serde_json::json!({
            "type": "error",
            "phase": format!("{:?}", phase),
            "error": error
        });
        println!("{}", json);
    }
}

/// Progress tracker that manages phase progression and reporting
pub struct ProgressTracker {
    current_phase: AnalysisPhase,
    phase_start_time: Instant,
    analysis_start_time: Instant,
    reporter: Box<dyn ProgressReporter>,
    sender: Option<watch::Sender<PhaseProgress>>,
}

impl ProgressTracker {
    /// Creates a new progress tracker with the given reporter
    pub fn new(reporter: Box<dyn ProgressReporter>) -> Self {
        let now = Instant::now();
        Self {
            current_phase: AnalysisPhase::Discovery,
            phase_start_time: now,
            analysis_start_time: now,
            reporter,
            sender: None,
        }
    }

    /// Adds a watch channel for external progress monitoring
    pub fn with_watch_channel(mut self) -> (Self, watch::Receiver<PhaseProgress>) {
        let (tx, rx) = watch::channel(PhaseProgress::new(AnalysisPhase::Discovery));
        self.sender = Some(tx);
        (self, rx)
    }

    /// Starts tracking progress for a new analysis phase
    pub fn start_phase(&mut self, phase: AnalysisPhase, total_items: Option<usize>) {
        self.current_phase = phase.clone();
        self.phase_start_time = Instant::now();

        let mut progress = PhaseProgress::new(self.current_phase.clone());
        if let Some(total) = total_items {
            progress = progress.with_total_items(total);
        }

        self.reporter.report_phase(&progress);

        if let Some(sender) = &self.sender {
            let _ = sender.send(progress);
        }
    }

    /// Updates progress for the current phase
    pub fn update_progress(&mut self, processed: usize, current_item: Option<String>) {
        let mut progress = PhaseProgress::new(self.current_phase.clone());
        progress.update_progress(processed, current_item, self.phase_start_time);

        self.reporter.report_phase(&progress);

        if let Some(sender) = &self.sender {
            let _ = sender.send(progress);
        }
    }

    /// Updates progress with specific file information
    pub fn update_file_progress(&mut self, file_path: &Path, current: usize, total: usize) {
        self.update_progress(current, Some(file_path.display().to_string()));

        // Update overall progress across phases
        let overall_progress = self.calculate_overall_progress(current, total);
        self.reporter
            .report_overall(&self.current_phase, overall_progress);
    }

    fn calculate_overall_progress(&self, current: usize, total: usize) -> f32 {
        // Weight different phases based on typical time distribution
        let phase_weights = match self.current_phase {
            AnalysisPhase::Discovery => 0.05,
            AnalysisPhase::Parsing => 0.25,
            AnalysisPhase::DependencyAnalysis => 0.15,
            AnalysisPhase::Analysis => 0.35,
            AnalysisPhase::AiAnalysis => 0.15,
            AnalysisPhase::ReportGeneration => 0.05,
            AnalysisPhase::Complete => 1.0,
        };

        let phase_progress = if total > 0 {
            current as f32 / total as f32
        } else {
            0.0
        };

        // Calculate cumulative progress based on completed phases
        let completed_phases_weight = match self.current_phase {
            AnalysisPhase::Discovery => 0.0,
            AnalysisPhase::Parsing => 0.05,
            AnalysisPhase::DependencyAnalysis => 0.30,
            AnalysisPhase::Analysis => 0.45,
            AnalysisPhase::AiAnalysis => 0.80,
            AnalysisPhase::ReportGeneration => 0.95,
            AnalysisPhase::Complete => 1.0,
        };

        completed_phases_weight + (phase_progress * phase_weights)
    }

    /// Marks the analysis as complete
    pub fn complete(&self) {
        let total_time = self.analysis_start_time.elapsed();
        self.reporter.report_complete(total_time);

        if let Some(sender) = &self.sender {
            let progress = PhaseProgress {
                phase: AnalysisPhase::Complete,
                progress: 1.0,
                current_item: None,
                items_processed: 0,
                total_items: None,
                elapsed_time: total_time,
                estimated_remaining: None,
            };
            let _ = sender.send(progress);
        }
    }

    /// Reports an error during analysis
    pub fn error(&self, error: &str) {
        self.reporter.report_error(&self.current_phase, error);
    }
}

/// Create a progress reporter based on output format preference
pub fn create_progress_reporter(format: &str, show_details: bool) -> Box<dyn ProgressReporter> {
    match format {
        "json" => Box::new(JsonProgressReporter),
        "silent" | "quiet" => Box::new(SilentProgressReporter),
        _ => Box::new(TerminalProgressReporter::new(show_details)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestProgressReporter {
        reports: Arc<Mutex<Vec<String>>>,
    }

    impl TestProgressReporter {
        fn new() -> (Self, Arc<Mutex<Vec<String>>>) {
            let reports = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    reports: reports.clone(),
                },
                reports,
            )
        }
    }

    impl ProgressReporter for TestProgressReporter {
        fn report_phase(&self, progress: &PhaseProgress) {
            self.reports.lock().unwrap().push(format!(
                "phase: {:?} {}%",
                progress.phase,
                (progress.progress * 100.0) as u8
            ));
        }

        fn report_overall(&self, phase: &AnalysisPhase, overall_progress: f32) {
            self.reports.lock().unwrap().push(format!(
                "overall: {:?} {}%",
                phase,
                (overall_progress * 100.0) as u8
            ));
        }

        fn report_complete(&self, total_time: Duration) {
            self.reports
                .lock()
                .unwrap()
                .push(format!("complete: {}s", total_time.as_secs()));
        }

        fn report_error(&self, phase: &AnalysisPhase, error: &str) {
            self.reports
                .lock()
                .unwrap()
                .push(format!("error: {:?} {}", phase, error));
        }
    }

    #[test]
    fn test_progress_tracking() {
        let (reporter, reports) = TestProgressReporter::new();
        let mut tracker = ProgressTracker::new(Box::new(reporter));

        tracker.start_phase(AnalysisPhase::Parsing, Some(10));
        tracker.update_progress(5, Some("test.rs".to_string()));

        let reports = reports.lock().unwrap();
        assert!(reports.len() >= 2);
        assert!(reports[0].contains("Parsing"));
    }

    #[test]
    fn test_phase_progression() {
        let progress = PhaseProgress::new(AnalysisPhase::Analysis);
        assert_eq!(progress.phase.description(), "Running analysis detectors");
        assert_eq!(progress.phase.emoji(), "🔬");
    }
}
