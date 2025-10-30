pub mod dynamic_query;
pub mod orm_analyzer;
pub mod parameter_analyzer;
pub mod query_analyzer;
pub mod stored_procedure;

use super::config::SqlInjectionDetectorConfig;
use super::types::{DetectionContext, DetectionFinding, SanitizationStatus, SqlInjectionPattern};

/// Execute all enabled detection passes and aggregate findings.
pub fn analyze(
    context: &DetectionContext,
    patterns: &[SqlInjectionPattern],
    sanitization: &SanitizationStatus,
    config: &SqlInjectionDetectorConfig,
) -> Vec<DetectionFinding> {
    let mut findings = Vec::new();

    if config.enable_pattern_analysis {
        findings.extend(query_analyzer::analyze(context, patterns, sanitization));
        findings.extend(dynamic_query::analyze(context, patterns, sanitization));
        findings.extend(orm_analyzer::analyze(context, patterns, sanitization));
        findings.extend(stored_procedure::analyze(context, patterns, sanitization));
    }

    if config.enable_parameter_analysis {
        findings.extend(parameter_analyzer::analyze(context, patterns, sanitization));
    }

    findings
}
