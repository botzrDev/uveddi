//! HTML Report Generator Module
//!
//! This module handles all HTML report generation functionality, including:
//! - HTML structure generation
//! - CSS styling and theming
//! - JavaScript interactivity
//! - Section rendering (header, navigation, content)

use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::report::ReportGenerator;
use std::collections::HashMap;

impl ReportGenerator {
    /// Generate complete HTML report
    pub fn generate_html_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let header = self.generate_report_header(analysis_run);
        let html_head = self.generate_html_head();
        let html_header = self.generate_html_header(analysis_run);
        let navigation = self.generate_html_navigation();
        let executive_summary = self.generate_html_executive_summary(analysis_run, issues);
        let severity_dashboard = self.generate_html_severity_dashboard(issues);
        let detailed_issues = self.generate_html_detailed_issues(issues);

        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
{html_head}
</head>
<body>
    <div class="container">
        {html_header}
        {navigation}
        {executive_summary}
        {severity_dashboard}
        {detailed_issues}
    </div>

    <script>
        // Theme toggle functionality
        function toggleTheme() {{
            const body = document.body;
            const currentTheme = body.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            body.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
        }}

        // Load saved theme
        document.addEventListener('DOMContentLoaded', function() {{
            const savedTheme = localStorage.getItem('theme') || 'light';
            document.body.setAttribute('data-theme', savedTheme);
        }});
    </script>
</body>
</html>"#
        );

        Ok(full_html)
    }

    /// Generate HTML head section with meta tags, styles, and scripts
    pub fn generate_html_head(&self) -> String {
        format!(
            r#"
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Uveddi Architectural Analysis Report</title>

    <!-- Mermaid.js for diagram rendering -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mermaid/10.6.1/mermaid.min.js"></script>

    <!-- Font Awesome for icons -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">

    <style>
        /* Modern CSS Variables for theming */
        :root {{
            --primary-color: #2563eb;
            --secondary-color: #64748b;
            --success-color: #059669;
            --warning-color: #d97706;
            --error-color: #dc2626;
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #e2e8f0;
            --text-primary: #0f172a;
            --text-secondary: #475569;
            --border-color: #e2e8f0;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            --radius: 8px;
        }}

        [data-theme="dark"] {{
            --primary-color: #3b82f6;
            --secondary-color: #94a3b8;
            --success-color: #10b981;
            --warning-color: #f59e0b;
            --error-color: #ef4444;
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-tertiary: #334155;
            --text-primary: #f1f5f9;
            --text-secondary: #cbd5e1;
            --border-color: #475569;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
        }}

        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: var(--bg-secondary);
            color: var(--text-primary);
            line-height: 1.6;
            transition: background-color 0.3s ease, color 0.3s ease;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}

        /* Header Styles */
        .header {{
            background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
            color: white;
            padding: 2rem;
            border-radius: var(--radius);
            margin-bottom: 2rem;
            position: relative;
            overflow: hidden;
        }}

        .header::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: linear-gradient(45deg, transparent 49%, rgba(255,255,255,0.1) 50%, transparent 51%);
            animation: shimmer 3s infinite;
        }}

        @keyframes shimmer {{
            0% {{ transform: translateX(-100%); }}
            100% {{ transform: translateX(100%); }}
        }}

        .header h1 {{
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            position: relative;
            z-index: 1;
        }}

        .header .subtitle {{
            font-size: 1.1rem;
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }}

        /* Navigation Styles */
        .nav-tabs {{
            display: flex;
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 0.5rem;
            margin-bottom: 2rem;
            box-shadow: var(--shadow);
        }}

        .nav-tab {{
            flex: 1;
            padding: 0.75rem 1rem;
            background: transparent;
            border: none;
            border-radius: calc(var(--radius) - 4px);
            cursor: pointer;
            font-weight: 500;
            color: var(--text-secondary);
            transition: all 0.2s ease;
        }}

        .nav-tab.active,
        .nav-tab:hover {{
            background: var(--primary-color);
            color: white;
            transform: translateY(-1px);
        }}

        /* Card Styles */
        .card {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            box-shadow: var(--shadow);
            border: 1px solid var(--border-color);
        }}

        .card h2 {{
            color: var(--primary-color);
            margin-bottom: 1rem;
            font-size: 1.5rem;
            font-weight: 600;
        }}

        /* Severity Badge Styles */
        .severity-badge {{
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        .severity-critical {{
            background: #fef2f2;
            color: #991b1b;
            border: 1px solid #fca5a5;
        }}

        .severity-high {{
            background: #fef3c7;
            color: #92400e;
            border: 1px solid #fcd34d;
        }}

        .severity-medium {{
            background: #ecfdf5;
            color: #065f46;
            border: 1px solid #86efac;
        }}

        .severity-low {{
            background: #f0f9ff;
            color: #0c4a6e;
            border: 1px solid #7dd3fc;
        }}

        [data-theme="dark"] .severity-critical {{
            background: #450a0a;
            color: #fca5a5;
        }}

        [data-theme="dark"] .severity-high {{
            background: #451a03;
            color: #fcd34d;
        }}

        [data-theme="dark"] .severity-medium {{
            background: #052e16;
            color: #86efac;
        }}

        [data-theme="dark"] .severity-low {{
            background: #0c2d48;
            color: #7dd3fc;
        }}

        /* Theme Toggle */
        .theme-toggle {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: var(--primary-color);
            color: white;
            border: none;
            border-radius: 50%;
            width: 50px;
            height: 50px;
            font-size: 1.2rem;
            cursor: pointer;
            box-shadow: var(--shadow);
            transition: transform 0.2s ease;
            z-index: 1000;
        }}

        .theme-toggle:hover {{
            transform: scale(1.1);
        }}

        /* Responsive Design */
        @media (max-width: 768px) {{
            .container {{
                padding: 10px;
            }}

            .header h1 {{
                font-size: 2rem;
            }}

            .nav-tabs {{
                flex-direction: column;
            }}
        }}
    </style>
"#
        )
    }

    /// Generate HTML header section
    pub fn generate_html_header(&self, analysis_run: &AnalysisRun) -> String {
        format!(
            r#"
    <div class="header">
        <h1><i class="fas fa-chart-line"></i> Uveddi Analysis Report</h1>
        <p class="subtitle">Architectural Analysis • Generated {}</p>
    </div>

    <button class="theme-toggle" onclick="toggleTheme()" title="Toggle Dark Mode">
        <i class="fas fa-moon"></i>
    </button>
"#,
            analysis_run.run_timestamp.format("%B %d, %Y at %H:%M UTC")
        )
    }

    /// Generate navigation tabs
    pub fn generate_html_navigation(&self) -> String {
        r#"
    <div class="nav-tabs">
        <button class="nav-tab active" onclick="showSection('summary')">
            <i class="fas fa-chart-pie"></i> Executive Summary
        </button>
        <button class="nav-tab" onclick="showSection('severity')">
            <i class="fas fa-exclamation-triangle"></i> Issues Dashboard
        </button>
        <button class="nav-tab" onclick="showSection('detailed')">
            <i class="fas fa-list-ul"></i> Detailed Analysis
        </button>
    </div>
"#
        .to_string()
    }

    /// Generate executive summary section
    pub fn generate_html_executive_summary(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> String {
        let severity_counts = self.count_issues_by_severity(issues);
        let total_issues = issues.len();

        format!(
            r#"
    <div id="summary" class="card">
        <h2><i class="fas fa-chart-pie"></i> Executive Summary</h2>

        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin: 1rem 0;">
            <div style="text-align: center; padding: 1rem; background: var(--bg-secondary); border-radius: var(--radius);">
                <div style="font-size: 2rem; font-weight: bold; color: var(--primary-color);">{}</div>
                <div style="color: var(--text-secondary);">Total Issues</div>
            </div>
            <div style="text-align: center; padding: 1rem; background: var(--bg-secondary); border-radius: var(--radius);">
                <div style="font-size: 2rem; font-weight: bold; color: var(--error-color);">{}</div>
                <div style="color: var(--text-secondary);">Critical</div>
            </div>
            <div style="text-align: center; padding: 1rem; background: var(--bg-secondary); border-radius: var(--radius);">
                <div style="font-size: 2rem; font-weight: bold; color: var(--warning-color);">{}</div>
                <div style="color: var(--text-secondary);">High Priority</div>
            </div>
            <div style="text-align: center; padding: 1rem; background: var(--bg-secondary); border-radius: var(--radius);">
                <div style="font-size: 2rem; font-weight: bold; color: var(--success-color);">{}</div>
                <div style="color: var(--text-secondary);">Files Analyzed</div>
            </div>
        </div>

        <p style="margin-top: 1rem; color: var(--text-secondary);">
            Analysis completed on <strong>{}</strong> covering <strong>{}</strong> files.
            This report identifies architectural issues and anti-patterns to help improve code quality.
        </p>
    </div>
"#,
            total_issues,
            severity_counts.get("Critical").unwrap_or(&0),
            severity_counts.get("High").unwrap_or(&0),
            analysis_run.files_analyzed,
            analysis_run.run_timestamp.format("%B %d, %Y"),
            analysis_run.files_analyzed
        )
    }

    /// Generate severity dashboard section
    pub fn generate_html_severity_dashboard(&self, issues: &[ArchitecturalIssue]) -> String {
        let severity_counts = self.count_issues_by_severity(issues);

        let mut severity_sections = String::new();
        for (severity, count) in &severity_counts {
            let severity_class = severity.to_lowercase();
            let icon = match severity.as_str() {
                "Critical" => "fas fa-exclamation-circle",
                "High" => "fas fa-exclamation-triangle",
                "Medium" => "fas fa-exclamation",
                "Low" => "fas fa-info-circle",
                _ => "fas fa-question-circle",
            };

            severity_sections.push_str(&format!(
                r#"
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.75rem; background: var(--bg-secondary); border-radius: var(--radius); margin-bottom: 0.5rem;">
                <div style="display: flex; align-items: center; gap: 0.5rem;">
                    <i class="{}" style="color: var(--{}-color);"></i>
                    <span style="font-weight: 500;">{} Priority</span>
                </div>
                <span class="severity-badge severity-{}">{} issues</span>
            </div>
"#,
                icon,
                severity_class,
                severity,
                severity_class,
                count
            ));
        }

        format!(
            r#"
    <div id="severity" class="card" style="display: none;">
        <h2><i class="fas fa-exclamation-triangle"></i> Issues Dashboard</h2>
        <div style="margin-top: 1rem;">
            {}
        </div>
    </div>
"#,
            severity_sections
        )
    }

    /// Generate detailed issues section
    pub fn generate_html_detailed_issues(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut issues_html = String::new();

        for issue in issues {
            let severity_class = issue.severity.to_lowercase();
            issues_html.push_str(&format!(
                r#"
            <div style="border: 1px solid var(--border-color); border-radius: var(--radius); padding: 1rem; margin-bottom: 1rem; background: var(--bg-primary);">
                <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 0.5rem;">
                    <h3 style="color: var(--primary-color); margin: 0;">{}</h3>
                    <span class="severity-badge severity-{}">{}</span>
                </div>
                <p style="color: var(--text-secondary); margin-bottom: 0.5rem;">{}</p>
                <div style="font-size: 0.875rem; color: var(--text-secondary);">
                    <i class="fas fa-file"></i> {}
                    <span style="margin-left: 1rem;"><i class="fas fa-map-marker-alt"></i> Line {}</span>
                </div>
            </div>
"#,
                issue.issue_type,
                severity_class,
                issue.severity,
                issue.description,
                issue.file_path,
                issue.line_number.unwrap_or(0)
            ));
        }

        format!(
            r#"
    <div id="detailed" class="card" style="display: none;">
        <h2><i class="fas fa-list-ul"></i> Detailed Analysis</h2>
        <div style="margin-top: 1rem;">
            {}
        </div>
    </div>

    <script>
        function showSection(sectionId) {{
            // Hide all sections
            document.getElementById('summary').style.display = 'none';
            document.getElementById('severity').style.display = 'none';
            document.getElementById('detailed').style.display = 'none';

            // Show selected section
            document.getElementById(sectionId).style.display = 'block';

            // Update tab states
            const tabs = document.querySelectorAll('.nav-tab');
            tabs.forEach(tab => tab.classList.remove('active'));
            event.target.classList.add('active');
        }}
    </script>
"#,
            issues_html
        )
    }

    /// Helper method to count issues by severity
    fn count_issues_by_severity(&self, issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for issue in issues {
            *counts.entry(issue.severity.clone()).or_insert(0) += 1;
        }
        counts
    }
}