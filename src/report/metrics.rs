//! Report metrics utilities (technical debt, severity summaries)
//!
//! Lightweight helpers to compute derived metrics from analysis findings.

use std::collections::HashMap;
use crate::database::models::ArchitecturalIssue;

/// Compute issues by severity (critical/high/medium/low) using case-insensitive matching.
pub fn compute_issues_by_severity(issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
    let mut map = HashMap::from([
        ("critical".to_string(), 0usize),
        ("high".to_string(), 0usize),
        ("medium".to_string(), 0usize),
        ("low".to_string(), 0usize),
    ]);

    for i in issues {
        let sev = i.severity.to_lowercase();
        if let Some(v) = map.get_mut(&sev) {
            *v += 1;
        } else {
            // Unknown severity -> treat as medium by default
            *map.get_mut("medium").unwrap() += 1;
        }
    }
    map
}

/// Compute a simple 0-100 technical debt score (higher is worse).
///
/// Weights per issue by severity:
/// - critical: 8
/// - high:     5
/// - medium:   3
/// - low:      1
/// Score is min(100, sum(weights) * scaling).
/// Scaling chosen so ~10 medium issues ~ 30 points, ~5 high ~ 25, 1 critical ~ 8.
pub fn compute_debt_score(issues: &[ArchitecturalIssue]) -> u32 {
    let sev = compute_issues_by_severity(issues);
    let critical = *sev.get("critical").unwrap_or(&0) as f64;
    let high = *sev.get("high").unwrap_or(&0) as f64;
    let medium = *sev.get("medium").unwrap_or(&0) as f64;
    let low = *sev.get("low").unwrap_or(&0) as f64;

    let weighted = critical * 8.0 + high * 5.0 + medium * 3.0 + low * 1.0;
    let scaled = (weighted * 2.0).round(); // simple scaling factor
    scaled.min(100.0) as u32
}

/// Count unique files in issues list.
pub fn count_unique_files(issues: &[ArchitecturalIssue]) -> usize {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    for i in issues {
        set.insert(i.file_path.clone());
    }
    set.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mk_issue(severity: &str, file: &str) -> ArchitecturalIssue {
        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: file.to_string(),
            start_line: Some(1),
            end_line: Some(1),
            line_number: Some(1),
            column_number: None,
            message: String::new(),
            metadata: "{}".to_string(),
            detector_name: "test".to_string(),
            created_at: Utc::now(),
            severity: severity.to_string(),
            description: String::new(),
            code_snippet: None,
            ai_explanation: None,
        }
    }

    #[test]
    fn test_compute_issues_by_severity() {
        let issues = vec![
            mk_issue("critical", "a.rs"),
            mk_issue("High", "b.rs"),
            mk_issue("MEDIUM", "c.rs"),
            mk_issue("low", "d.rs"),
            mk_issue("unknown", "e.rs"),
        ];
        let sev = compute_issues_by_severity(&issues);
        assert_eq!(*sev.get("critical").unwrap(), 1);
        assert_eq!(*sev.get("high").unwrap(), 1);
        assert_eq!(*sev.get("medium").unwrap(), 2); // unknown treated as medium
        assert_eq!(*sev.get("low").unwrap(), 1);
    }

    #[test]
    fn test_compute_debt_score_bounds() {
        // 1 critical => 8 * 2 = 16
        let issues = vec![mk_issue("critical", "a")];
        assert!(compute_debt_score(&issues) >= 16);

        // Many issues cap at 100
        let many: Vec<_> = (0..200).map(|i| mk_issue("high", &format!("f{i}"))).collect();
        assert_eq!(compute_debt_score(&many), 100);
    }

    #[test]
    fn test_count_unique_files() {
        let issues = vec![
            mk_issue("low", "x.rs"),
            mk_issue("low", "x.rs"),
            mk_issue("low", "y.rs"),
        ];
        assert_eq!(count_unique_files(&issues), 2);
    }
}
