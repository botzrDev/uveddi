//! Markdown formatting for God Object reports

use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use super::utils::{calculate_severity_distribution, get_severity_icon, extract_class_name, generate_recommendations};

/// Markdown formatter for God Object reports
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    /// Format issues as Markdown report
    pub fn format(issues: &[ArchitecturalIssue]) -> String {
        let mut markdown = String::new();

        markdown.push_str("# God Object Detection Report\n\n");
        markdown.push_str(&format!("**Total Issues Found:** {}\n\n", issues.len()));

        if issues.is_empty() {
            markdown.push_str("✅ **No God Objects detected!** Your code follows good separation of concerns.\n");
            return markdown;
        }

        // Severity distribution
        let distribution = calculate_severity_distribution(issues);
        markdown.push_str("## Severity Distribution\n\n");
        for (severity, count) in &distribution {
            let icon = get_severity_icon(severity);
            markdown.push_str(&format!("- {} **{}**: {} issue(s)\n", icon, severity, count));
        }
        markdown.push_str("\n");

        // Group issues by severity
        let mut by_severity: HashMap<String, Vec<&ArchitecturalIssue>> = HashMap::new();
        for issue in issues {
            by_severity
                .entry(issue.severity.clone())
                .or_insert_with(Vec::new)
                .push(issue);
        }

        // Output issues by severity (Critical first)
        let severity_order = ["Critical", "High", "Medium", "Low"];
        for severity in &severity_order {
            if let Some(severity_issues) = by_severity.get(*severity) {
                markdown.push_str(&format!("## {} Issues\n\n", severity));

                for (i, issue) in severity_issues.iter().enumerate() {
                    markdown.push_str(&Self::format_issue(issue, severity, i + 1));
                }
            }
        }

        // Add general recommendations
        markdown.push_str(&Self::get_general_recommendations());

        markdown
    }

    /// Format individual issue
    fn format_issue(issue: &ArchitecturalIssue, severity: &str, index: usize) -> String {
        let mut markdown = String::new();

        markdown.push_str(&format!("### {}.{} {}\n\n", severity, index, extract_class_name(&issue.description)));
        markdown.push_str(&format!("**File:** `{}`\n", issue.file_path));

        if let Some(line) = issue.start_line {
            markdown.push_str(&format!("**Line:** {}\n", line));
        }

        markdown.push_str(&format!("**Severity:** {}\n\n", issue.severity));
        markdown.push_str(&format!("**Description:** {}\n\n", issue.description));

        if let Some(snippet) = &issue.code_snippet {
            let truncated_snippet = Self::truncate_snippet(snippet);
            markdown.push_str("**Code Snippet:**\n");
            markdown.push_str("```\n");
            markdown.push_str(&truncated_snippet);
            markdown.push_str("\n```\n\n");
        }

        markdown.push_str("**Recommendations:**\n");
        markdown.push_str(&generate_recommendations(&issue.description));
        markdown.push_str("\n---\n\n");

        markdown
    }

    /// Truncate code snippet if too long
    fn truncate_snippet(snippet: &str) -> String {
        if snippet.len() > 1000 {
            format!("{}...\n\n[Code snippet truncated - {} characters total]", &snippet[..1000], snippet.len())
        } else {
            snippet.to_string()
        }
    }

    /// Get general recommendations section
    fn get_general_recommendations() -> String {
        let mut markdown = String::new();

        markdown.push_str("## General Recommendations\n\n");
        markdown.push_str("1. **Single Responsibility Principle**: Each class should have only one reason to change\n");
        markdown.push_str("2. **Extract Classes**: Break large classes into smaller, focused ones\n");
        markdown.push_str("3. **Composition over Inheritance**: Use composition to combine behaviors\n");
        markdown.push_str("4. **Interface Segregation**: Create focused interfaces instead of large ones\n");
        markdown.push_str("5. **Dependency Injection**: Use DI to manage complex dependencies\n\n");

        markdown.push_str("## Tools and Techniques\n\n");
        markdown.push_str("- **Extract Method**: Break down large methods into smaller ones\n");
        markdown.push_str("- **Extract Class**: Move related methods and fields to new classes\n");
        markdown.push_str("- **Extract Interface**: Define contracts for specific behaviors\n");
        markdown.push_str("- **Replace Data Value with Object**: Create value objects for complex data\n");
        markdown.push_str("- **Strategy Pattern**: Extract varying behaviors into separate strategies\n");

        markdown
    }

    /// Generate a summary report for multiple files
    pub fn generate_summary(all_issues: &[ArchitecturalIssue]) -> String {
        let mut summary = String::new();

        summary.push_str("# God Object Detection Summary\n\n");

        if all_issues.is_empty() {
            summary.push_str("✅ **Excellent!** No God Objects detected across the analyzed codebase.\n");
            return summary;
        }

        let total_issues = all_issues.len();
        let distribution = calculate_severity_distribution(all_issues);

        summary.push_str(&format!("📊 **Total Issues:** {}\n\n", total_issues));

        // Files with issues
        let files_with_issues = Self::get_files_with_issues(all_issues);
        summary.push_str(&format!("📁 **Files Affected:** {}\n\n", files_with_issues.len()));

        // Severity breakdown with percentages
        summary.push_str("## Severity Breakdown\n\n");
        for (severity, count) in &distribution {
            let percentage = (*count as f64 / total_issues as f64) * 100.0;
            let icon = get_severity_icon(severity);
            summary.push_str(&format!(
                "- {} **{}**: {} issues ({:.1}%)\n",
                icon, severity, count, percentage
            ));
        }

        summary.push_str("\n## Most Problematic Files\n\n");
        let mut sorted_files: Vec<_> = files_with_issues.iter().collect();
        sorted_files.sort_by(|a, b| b.1.cmp(a.1));

        for (file, count) in sorted_files.iter().take(10) {
            summary.push_str(&format!("- `{}`: {} issue(s)\n", file, count));
        }

        summary.push_str("\n## Next Steps\n\n");
        summary.push_str("1. 🎯 **Start with Critical issues** - These require immediate attention\n");
        summary.push_str("2. 📋 **Prioritize by file** - Focus on files with multiple issues\n");
        summary.push_str("3. 🔄 **Refactor incrementally** - Break down one responsibility at a time\n");
        summary.push_str("4. ✅ **Add tests** - Ensure behavior is preserved during refactoring\n");
        summary.push_str("5. 📈 **Track progress** - Re-run analysis to measure improvement\n");

        summary
    }

    /// Get files with issue counts
    fn get_files_with_issues(all_issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
        let mut files_with_issues: HashMap<String, usize> = HashMap::new();
        for issue in all_issues {
            *files_with_issues.entry(issue.file_path.clone()).or_insert(0) += 1;
        }
        files_with_issues
    }
}