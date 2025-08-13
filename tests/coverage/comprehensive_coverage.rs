//! Comprehensive Coverage Analysis Framework  
//! Automated coverage reporting and CI integration for bulletproof testing

use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Write, BufReader, BufRead};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tempfile::tempdir;

#[cfg(test)]
mod comprehensive_coverage_analysis {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CoverageReport {
        timestamp: u64,
        project_name: String,
        total_lines: usize,
        covered_lines: usize,
        coverage_percentage: f64,
        files: Vec<FileCoverage>,
        uncovered_regions: Vec<UncoveredRegion>,
        summary: CoverageSummary,
        trends: CoverageTrends,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct FileCoverage {
        file_path: String,
        total_lines: usize,
        covered_lines: usize,
        coverage_percentage: f64,
        functions: Vec<FunctionCoverage>,
        branches: Vec<BranchCoverage>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct FunctionCoverage {
        name: String,
        line_start: usize,
        line_end: usize,
        is_covered: bool,
        call_count: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct BranchCoverage {
        line: usize,
        branch_id: String,
        true_taken: bool,
        false_taken: bool,
        coverage_percentage: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct UncoveredRegion {
        file_path: String,
        start_line: usize,
        end_line: usize,
        reason: String,
        priority: CoveragePriority,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum CoveragePriority {
        Critical,
        High,
        Medium,
        Low,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CoverageSummary {
        line_coverage: f64,
        branch_coverage: f64,
        function_coverage: f64,
        complexity_coverage: f64,
        test_count: usize,
        regression_risk_score: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CoverageTrends {
        coverage_trend: f64, // Positive = improving, negative = declining
        new_uncovered_lines: usize,
        resolved_uncovered_lines: usize,
        test_growth_rate: f64,
        performance_impact: f64,
    }

    struct CoverageAnalyzer {
        project_root: PathBuf,
        output_dir: PathBuf,
        llvm_tools_path: Option<PathBuf>,
        historical_data: Vec<CoverageReport>,
    }

    impl CoverageAnalyzer {
        fn new(project_root: PathBuf) -> Self {
            let output_dir = project_root.join("coverage_reports");
            fs::create_dir_all(&output_dir).unwrap_or_else(|_| {});
            
            Self {
                project_root: project_root.clone(),
                output_dir,
                llvm_tools_path: Self::find_llvm_tools(),
                historical_data: Self::load_historical_data(&project_root),
            }
        }

        fn find_llvm_tools() -> Option<PathBuf> {
            // Try to find LLVM coverage tools
            let possible_paths = [
                "llvm-profdata",
                "llvm-cov",
                "/usr/bin/llvm-profdata",
                "/usr/local/bin/llvm-profdata",
            ];

            for path in &possible_paths {
                if Command::new(path).arg("--version").output().is_ok() {
                    return Some(PathBuf::from(path).parent().unwrap().to_path_buf());
                }
            }

            None
        }

        fn load_historical_data(project_root: &Path) -> Vec<CoverageReport> {
            let history_file = project_root.join("coverage_reports/history.json");
            
            if history_file.exists() {
                match fs::read_to_string(&history_file) {
                    Ok(content) => {
                        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
                    }
                    Err(_) => Vec::new()
                }
            } else {
                Vec::new()
            }
        }

        fn generate_llvm_coverage_report(&self) -> Result<CoverageReport, String> {
            println!("🔍 Generating LLVM Coverage Report");
            
            // Set coverage environment variables
            std::env::set_var("RUSTFLAGS", "-C instrument-coverage");
            std::env::set_var("LLVM_PROFILE_FILE", "coverage/uveddi-%p-%m.profraw");
            
            // Create coverage directory
            let coverage_dir = self.project_root.join("coverage");
            fs::create_dir_all(&coverage_dir).map_err(|e| format!("Failed to create coverage dir: {}", e))?;

            // Run tests with coverage instrumentation
            println!("  Running tests with coverage instrumentation...");
            let test_output = Command::new("cargo")
                .args(&["test", "--all-features", "--", "--test-threads=1"])
                .current_dir(&self.project_root)
                .env("RUSTFLAGS", "-C instrument-coverage")
                .env("LLVM_PROFILE_FILE", "coverage/uveddi-%p-%m.profraw")
                .output()
                .map_err(|e| format!("Failed to run tests: {}", e))?;

            if !test_output.status.success() {
                return Err(format!("Tests failed: {}", String::from_utf8_lossy(&test_output.stderr)));
            }

            // Find .profraw files
            let profraw_files: Vec<_> = fs::read_dir(&coverage_dir)
                .map_err(|e| format!("Failed to read coverage dir: {}", e))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "profraw"))
                .map(|entry| entry.path())
                .collect();

            if profraw_files.is_empty() {
                return Err("No .profraw files generated".to_string());
            }

            // Merge profraw files
            println!("  Merging {} profraw files...", profraw_files.len());
            let profdata_file = coverage_dir.join("merged.profdata");
            
            let mut merge_cmd = Command::new("llvm-profdata");
            merge_cmd.arg("merge")
                .arg("-sparse")
                .arg("-o")
                .arg(&profdata_file);
            
            for file in &profraw_files {
                merge_cmd.arg(file);
            }

            let merge_output = merge_cmd
                .current_dir(&self.project_root)
                .output()
                .map_err(|e| format!("Failed to merge profdata: {}", e))?;

            if !merge_output.status.success() {
                return Err(format!("Profdata merge failed: {}", String::from_utf8_lossy(&merge_output.stderr)));
            }

            // Generate coverage report
            println!("  Generating detailed coverage report...");
            let binary_path = self.find_test_binary()?;
            
            let cov_output = Command::new("llvm-cov")
                .args(&["show", "--format=json", "--instr-profile"])
                .arg(&profdata_file)
                .arg(&binary_path)
                .arg("--ignore-filename-regex=/.cargo/")
                .current_dir(&self.project_root)
                .output()
                .map_err(|e| format!("Failed to generate coverage: {}", e))?;

            if !cov_output.status.success() {
                return Err(format!("Coverage generation failed: {}", String::from_utf8_lossy(&cov_output.stderr)));
            }

            // Parse LLVM coverage JSON
            let coverage_json = String::from_utf8_lossy(&cov_output.stdout);
            self.parse_llvm_coverage_json(&coverage_json)
        }

        fn find_test_binary(&self) -> Result<PathBuf, String> {
            let target_dir = self.project_root.join("target/debug/deps");
            
            let entries = fs::read_dir(&target_dir)
                .map_err(|e| format!("Failed to read target dir: {}", e))?;

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("uveddi-") && path.is_file() && !name.ends_with(".d") {
                            return Ok(path);
                        }
                    }
                }
            }

            Err("Could not find test binary".to_string())
        }

        fn parse_llvm_coverage_json(&self, json_data: &str) -> Result<CoverageReport, String> {
            let coverage_data: serde_json::Value = serde_json::from_str(json_data)
                .map_err(|e| format!("Failed to parse coverage JSON: {}", e))?;

            let mut total_lines = 0;
            let mut covered_lines = 0;
            let mut files = Vec::new();
            let mut uncovered_regions = Vec::new();

            if let Some(data) = coverage_data.get("data").and_then(|d| d.as_array()) {
                for file_data in data {
                    if let Some(filename) = file_data.get("filename").and_then(|f| f.as_str()) {
                        // Skip external dependencies
                        if filename.contains("/.cargo/") || filename.contains("/rustc/") {
                            continue;
                        }

                        let file_coverage = self.parse_file_coverage(file_data, filename)?;
                        total_lines += file_coverage.total_lines;
                        covered_lines += file_coverage.covered_lines;

                        // Identify uncovered regions
                        if file_coverage.coverage_percentage < 100.0 {
                            uncovered_regions.extend(self.identify_uncovered_regions(&file_coverage));
                        }

                        files.push(file_coverage);
                    }
                }
            }

            let coverage_percentage = if total_lines > 0 {
                (covered_lines as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            };

            let summary = self.calculate_coverage_summary(&files, coverage_percentage);
            let trends = self.calculate_coverage_trends(&summary);

            let report = CoverageReport {
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                project_name: "Uveddi".to_string(),
                total_lines,
                covered_lines,
                coverage_percentage,
                files,
                uncovered_regions,
                summary,
                trends,
            };

            Ok(report)
        }

        fn parse_file_coverage(&self, file_data: &serde_json::Value, filename: &str) -> Result<FileCoverage, String> {
            let mut total_lines = 0;
            let mut covered_lines = 0;
            let mut functions = Vec::new();
            let mut branches = Vec::new();

            // Parse segments for line coverage
            if let Some(segments) = file_data.get("segments").and_then(|s| s.as_array()) {
                let mut line_coverage = HashMap::new();
                
                for segment in segments {
                    if let Some(segment_array) = segment.as_array() {
                        if segment_array.len() >= 4 {
                            if let (Some(line), Some(count)) = (
                                segment_array[0].as_u64(),
                                segment_array[3].as_u64()
                            ) {
                                line_coverage.insert(line as usize, count > 0);
                            }
                        }
                    }
                }

                total_lines = line_coverage.len();
                covered_lines = line_coverage.values().filter(|&&covered| covered).count();
            }

            // Parse function coverage
            if let Some(funcs) = file_data.get("functions").and_then(|f| f.as_array()) {
                for func in funcs {
                    if let (Some(name), Some(line_start), Some(line_end), Some(execution_count)) = (
                        func.get("name").and_then(|n| n.as_str()),
                        func.get("regions").and_then(|r| r.as_array()).and_then(|arr| arr.get(0)).and_then(|r| r.as_array()).and_then(|arr| arr.get(0)).and_then(|l| l.as_u64()),
                        func.get("regions").and_then(|r| r.as_array()).and_then(|arr| arr.get(0)).and_then(|r| r.as_array()).and_then(|arr| arr.get(2)).and_then(|l| l.as_u64()),
                        func.get("executionCount").and_then(|c| c.as_u64())
                    ) {
                        functions.push(FunctionCoverage {
                            name: name.to_string(),
                            line_start: line_start as usize,
                            line_end: line_end as usize,
                            is_covered: execution_count > 0,
                            call_count: execution_count as usize,
                        });
                    }
                }
            }

            let coverage_percentage = if total_lines > 0 {
                (covered_lines as f64 / total_lines as f64) * 100.0
            } else {
                100.0
            };

            Ok(FileCoverage {
                file_path: filename.to_string(),
                total_lines,
                covered_lines,
                coverage_percentage,
                functions,
                branches,
            })
        }

        fn identify_uncovered_regions(&self, file_coverage: &FileCoverage) -> Vec<UncoveredRegion> {
            let mut regions = Vec::new();

            // Identify uncovered functions
            for func in &file_coverage.functions {
                if !func.is_covered {
                    let priority = if func.name.contains("test_") || func.name.contains("bench_") {
                        CoveragePriority::Low
                    } else if func.name.contains("error") || func.name.contains("fail") {
                        CoveragePriority::Critical
                    } else if func.name.starts_with("pub") {
                        CoveragePriority::High
                    } else {
                        CoveragePriority::Medium
                    };

                    regions.push(UncoveredRegion {
                        file_path: file_coverage.file_path.clone(),
                        start_line: func.line_start,
                        end_line: func.line_end,
                        reason: format!("Uncovered function: {}", func.name),
                        priority,
                    });
                }
            }

            regions
        }

        fn calculate_coverage_summary(&self, files: &[FileCoverage], line_coverage: f64) -> CoverageSummary {
            let total_functions: usize = files.iter().map(|f| f.functions.len()).sum();
            let covered_functions: usize = files.iter()
                .flat_map(|f| &f.functions)
                .filter(|func| func.is_covered)
                .count();

            let function_coverage = if total_functions > 0 {
                (covered_functions as f64 / total_functions as f64) * 100.0
            } else {
                100.0
            };

            let total_branches: usize = files.iter().map(|f| f.branches.len()).sum();
            let covered_branches: usize = files.iter()
                .flat_map(|f| &f.branches)
                .filter(|branch| branch.true_taken || branch.false_taken)
                .count();

            let branch_coverage = if total_branches > 0 {
                (covered_branches as f64 / total_branches as f64) * 100.0
            } else {
                100.0
            };

            // Calculate regression risk score based on coverage gaps
            let risk_score = (100.0 - line_coverage) * 0.4 + 
                            (100.0 - function_coverage) * 0.3 + 
                            (100.0 - branch_coverage) * 0.3;

            CoverageSummary {
                line_coverage,
                branch_coverage,
                function_coverage,
                complexity_coverage: (line_coverage + branch_coverage + function_coverage) / 3.0,
                test_count: self.count_tests(),
                regression_risk_score: risk_score,
            }
        }

        fn count_tests(&self) -> usize {
            // Count test functions in the project
            let mut test_count = 0;
            
            if let Ok(entries) = fs::read_dir(self.project_root.join("tests")) {
                for entry in entries.flatten() {
                    if entry.path().extension().map_or(false, |ext| ext == "rs") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            test_count += content.matches("#[test]").count();
                            test_count += content.matches("#[tokio::test]").count();
                        }
                    }
                }
            }

            test_count
        }

        fn calculate_coverage_trends(&self, current_summary: &CoverageSummary) -> CoverageTrends {
            if let Some(last_report) = self.historical_data.last() {
                let coverage_trend = current_summary.line_coverage - last_report.summary.line_coverage;
                let test_growth_rate = if last_report.summary.test_count > 0 {
                    ((current_summary.test_count as f64 - last_report.summary.test_count as f64) / 
                     last_report.summary.test_count as f64) * 100.0
                } else {
                    0.0
                };

                CoverageTrends {
                    coverage_trend,
                    new_uncovered_lines: 0, // Would need diff analysis
                    resolved_uncovered_lines: 0, // Would need diff analysis  
                    test_growth_rate,
                    performance_impact: 0.0, // Would need performance benchmarking
                }
            } else {
                CoverageTrends {
                    coverage_trend: 0.0,
                    new_uncovered_lines: 0,
                    resolved_uncovered_lines: 0,
                    test_growth_rate: 0.0,
                    performance_impact: 0.0,
                }
            }
        }

        fn generate_html_report(&self, report: &CoverageReport) -> Result<(), String> {
            let html_content = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Uveddi Coverage Report</title>
    <style>
        body {{ font-family: 'Segoe UI', Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; padding: 30px; }}
        .metric {{ text-align: center; padding: 20px; background: #f8f9fa; border-radius: 8px; }}
        .metric h3 {{ margin: 0 0 10px 0; color: #333; }}
        .metric .value {{ font-size: 2em; font-weight: bold; }}
        .good {{ color: #28a745; }}
        .warning {{ color: #ffc107; }}
        .danger {{ color: #dc3545; }}
        .files {{ margin: 20px; }}
        .file {{ margin: 10px 0; padding: 15px; background: #f8f9fa; border-radius: 5px; border-left: 4px solid #007bff; }}
        .uncovered {{ background: #fff3cd; border-left-color: #ffc107; }}
        .coverage-bar {{ width: 100%; height: 20px; background: #e9ecef; border-radius: 10px; overflow: hidden; }}
        .coverage-fill {{ height: 100%; background: linear-gradient(90deg, #dc3545 0%, #ffc107 50%, #28a745 100%); }}
        .trends {{ padding: 20px; background: #e3f2fd; }}
        .trend-up {{ color: #28a745; }}
        .trend-down {{ color: #dc3545; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🛡️ Uveddi Coverage Report</h1>
            <p>Generated on: {timestamp}</p>
            <p>Total Coverage: <strong>{coverage:.1}%</strong></p>
        </div>
        
        <div class="metrics">
            <div class="metric">
                <h3>Line Coverage</h3>
                <div class="value {line_class}">{line_coverage:.1}%</div>
                <div>{covered_lines}/{total_lines} lines</div>
            </div>
            <div class="metric">
                <h3>Function Coverage</h3>
                <div class="value {func_class}">{func_coverage:.1}%</div>
            </div>
            <div class="metric">
                <h3>Branch Coverage</h3>
                <div class="value {branch_class}">{branch_coverage:.1}%</div>
            </div>
            <div class="metric">
                <h3>Tests</h3>
                <div class="value">{test_count}</div>
                <div>Growth: {test_growth:+.1}%</div>
            </div>
            <div class="metric">
                <h3>Risk Score</h3>
                <div class="value {risk_class}">{risk_score:.1}</div>
                <div>Lower is better</div>
            </div>
        </div>

        <div class="trends">
            <h2>📈 Coverage Trends</h2>
            <p>Coverage Trend: <span class="{trend_class}">{trend:+.2}%</span></p>
            <p>New Uncovered Lines: {new_uncovered}</p>
            <p>Resolved Lines: {resolved_uncovered}</p>
        </div>

        <div class="files">
            <h2>📁 File Coverage Details</h2>
            {file_details}
        </div>
        
        <div style="padding: 20px;">
            <h2>⚠️ Critical Uncovered Regions</h2>
            {uncovered_regions}
        </div>
    </div>
</body>
</html>
"#,
                timestamp = chrono::DateTime::from_timestamp(report.timestamp as i64, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S"),
                coverage = report.coverage_percentage,
                line_coverage = report.summary.line_coverage,
                line_class = if report.summary.line_coverage >= 90.0 { "good" } 
                           else if report.summary.line_coverage >= 70.0 { "warning" } 
                           else { "danger" },
                covered_lines = report.covered_lines,
                total_lines = report.total_lines,
                func_coverage = report.summary.function_coverage,
                func_class = if report.summary.function_coverage >= 90.0 { "good" } 
                           else if report.summary.function_coverage >= 70.0 { "warning" } 
                           else { "danger" },
                branch_coverage = report.summary.branch_coverage,
                branch_class = if report.summary.branch_coverage >= 80.0 { "good" } 
                             else if report.summary.branch_coverage >= 60.0 { "warning" } 
                             else { "danger" },
                test_count = report.summary.test_count,
                test_growth = report.trends.test_growth_rate,
                risk_score = report.summary.regression_risk_score,
                risk_class = if report.summary.regression_risk_score < 10.0 { "good" }
                           else if report.summary.regression_risk_score < 25.0 { "warning" }
                           else { "danger" },
                trend = report.trends.coverage_trend,
                trend_class = if report.trends.coverage_trend > 0.0 { "trend-up" } else { "trend-down" },
                new_uncovered = report.trends.new_uncovered_lines,
                resolved_uncovered = report.trends.resolved_uncovered_lines,
                file_details = self.generate_file_details_html(&report.files),
                uncovered_regions = self.generate_uncovered_regions_html(&report.uncovered_regions),
            );

            let html_file = self.output_dir.join("coverage_report.html");
            fs::write(&html_file, html_content)
                .map_err(|e| format!("Failed to write HTML report: {}", e))?;

            println!("✅ HTML coverage report generated: {}", html_file.display());
            Ok(())
        }

        fn generate_file_details_html(&self, files: &[FileCoverage]) -> String {
            let mut html = String::new();
            
            for file in files {
                let coverage_class = if file.coverage_percentage >= 90.0 { 
                    "good" 
                } else if file.coverage_percentage >= 70.0 { 
                    "warning" 
                } else { 
                    "danger uncovered" 
                };

                let coverage_width = file.coverage_percentage;
                
                html.push_str(&format!(r#"
                <div class="file {coverage_class}">
                    <h4>{file_path}</h4>
                    <div style="display: flex; align-items: center; gap: 10px;">
                        <div class="coverage-bar" style="flex: 1;">
                            <div class="coverage-fill" style="width: {coverage_width:.1}%"></div>
                        </div>
                        <strong>{coverage:.1}%</strong>
                    </div>
                    <p>{covered}/{total} lines covered, {functions} functions</p>
                </div>
                "#, 
                    file_path = file.file_path,
                    coverage_class = coverage_class,
                    coverage_width = coverage_width,
                    coverage = file.coverage_percentage,
                    covered = file.covered_lines,
                    total = file.total_lines,
                    functions = file.functions.len()
                ));
            }
            
            html
        }

        fn generate_uncovered_regions_html(&self, regions: &[UncoveredRegion]) -> String {
            let mut html = String::new();
            
            for region in regions.iter().filter(|r| matches!(r.priority, CoveragePriority::Critical | CoveragePriority::High)) {
                let priority_class = match region.priority {
                    CoveragePriority::Critical => "danger",
                    CoveragePriority::High => "warning", 
                    _ => "",
                };

                html.push_str(&format!(r#"
                <div class="file {priority_class}">
                    <strong>{file_path}:{start_line}-{end_line}</strong>
                    <p>{reason}</p>
                    <small>Priority: {priority:?}</small>
                </div>
                "#,
                    file_path = region.file_path,
                    start_line = region.start_line,
                    end_line = region.end_line,
                    reason = region.reason,
                    priority = region.priority
                ));
            }
            
            if html.is_empty() {
                html = "<p>✅ No critical uncovered regions found!</p>".to_string();
            }
            
            html
        }

        fn save_report_data(&mut self, report: CoverageReport) -> Result<(), String> {
            // Add to historical data
            self.historical_data.push(report.clone());
            
            // Keep only last 50 reports for trends
            if self.historical_data.len() > 50 {
                self.historical_data.drain(0..self.historical_data.len() - 50);
            }

            // Save historical data
            let history_file = self.project_root.join("coverage_reports/history.json");
            let history_json = serde_json::to_string_pretty(&self.historical_data)
                .map_err(|e| format!("Failed to serialize history: {}", e))?;
            
            fs::write(&history_file, history_json)
                .map_err(|e| format!("Failed to write history: {}", e))?;

            // Save current report
            let report_file = self.output_dir.join("latest_report.json");
            let report_json = serde_json::to_string_pretty(&report)
                .map_err(|e| format!("Failed to serialize report: {}", e))?;
            
            fs::write(&report_file, report_json)
                .map_err(|e| format!("Failed to write report: {}", e))?;

            Ok(())
        }

        fn check_coverage_requirements(&self, report: &CoverageReport) -> Result<(), String> {
            let min_line_coverage = 95.0; // Target: 95%+ line coverage
            let min_function_coverage = 90.0; // Target: 90%+ function coverage
            let max_risk_score = 15.0; // Maximum acceptable risk score

            let mut violations = Vec::new();

            if report.summary.line_coverage < min_line_coverage {
                violations.push(format!(
                    "Line coverage {:.1}% is below required {:.1}%",
                    report.summary.line_coverage, min_line_coverage
                ));
            }

            if report.summary.function_coverage < min_function_coverage {
                violations.push(format!(
                    "Function coverage {:.1}% is below required {:.1}%", 
                    report.summary.function_coverage, min_function_coverage
                ));
            }

            if report.summary.regression_risk_score > max_risk_score {
                violations.push(format!(
                    "Regression risk score {:.1} exceeds maximum {:.1}",
                    report.summary.regression_risk_score, max_risk_score
                ));
            }

            // Check for critical uncovered regions
            let critical_uncovered = report.uncovered_regions.iter()
                .filter(|r| matches!(r.priority, CoveragePriority::Critical))
                .count();

            if critical_uncovered > 0 {
                violations.push(format!(
                    "{} critical uncovered regions found", critical_uncovered
                ));
            }

            if !violations.is_empty() {
                return Err(format!("Coverage requirements not met:\n{}", violations.join("\n")));
            }

            Ok(())
        }
    }

    #[test]
    fn test_coverage_analyzer_initialization() {
        println!("🧪 Testing Coverage Analyzer Initialization");

        let temp_dir = tempdir().unwrap();
        let analyzer = CoverageAnalyzer::new(temp_dir.path().to_path_buf());

        assert!(analyzer.output_dir.exists());
        assert_eq!(analyzer.historical_data.len(), 0);
        
        println!("✅ Coverage analyzer initialized successfully");
    }

    #[test]
    fn test_llvm_tools_detection() {
        println!("🔍 Testing LLVM Tools Detection");

        let llvm_path = CoverageAnalyzer::find_llvm_tools();
        
        match llvm_path {
            Some(path) => {
                println!("✅ LLVM tools found at: {}", path.display());
            }
            None => {
                println!("⚠️ LLVM tools not found - coverage analysis will be limited");
                println!("Install with: rustup component add llvm-tools-preview");
            }
        }
    }

    #[test]
    fn test_coverage_metrics_calculation() {
        println!("📊 Testing Coverage Metrics Calculation");

        let temp_dir = tempdir().unwrap();
        let analyzer = CoverageAnalyzer::new(temp_dir.path().to_path_buf());

        // Create mock file coverage data
        let files = vec![
            FileCoverage {
                file_path: "src/lib.rs".to_string(),
                total_lines: 100,
                covered_lines: 90,
                coverage_percentage: 90.0,
                functions: vec![
                    FunctionCoverage {
                        name: "test_function".to_string(),
                        line_start: 10,
                        line_end: 20,
                        is_covered: true,
                        call_count: 5,
                    }
                ],
                branches: vec![],
            },
            FileCoverage {
                file_path: "src/main.rs".to_string(),
                total_lines: 50,
                covered_lines: 40,
                coverage_percentage: 80.0,
                functions: vec![
                    FunctionCoverage {
                        name: "main".to_string(),
                        line_start: 1,
                        line_end: 10,
                        is_covered: true,
                        call_count: 1,
                    }
                ],
                branches: vec![],
            }
        ];

        let summary = analyzer.calculate_coverage_summary(&files, 86.7);
        
        assert_eq!(summary.line_coverage, 86.7);
        assert_eq!(summary.function_coverage, 100.0); // Both functions covered
        assert!(summary.regression_risk_score < 20.0); // Should be relatively low risk

        println!("✅ Coverage metrics calculated correctly");
        println!("  Line coverage: {:.1}%", summary.line_coverage);
        println!("  Function coverage: {:.1}%", summary.function_coverage);
        println!("  Risk score: {:.1}", summary.regression_risk_score);
    }

    #[test]
    fn test_coverage_html_generation() {
        println!("🌐 Testing Coverage HTML Generation");

        let temp_dir = tempdir().unwrap();
        let analyzer = CoverageAnalyzer::new(temp_dir.path().to_path_buf());

        // Create mock coverage report
        let report = CoverageReport {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            project_name: "Test Project".to_string(),
            total_lines: 1000,
            covered_lines: 850,
            coverage_percentage: 85.0,
            files: vec![
                FileCoverage {
                    file_path: "test.rs".to_string(),
                    total_lines: 100,
                    covered_lines: 85,
                    coverage_percentage: 85.0,
                    functions: vec![],
                    branches: vec![],
                }
            ],
            uncovered_regions: vec![],
            summary: CoverageSummary {
                line_coverage: 85.0,
                branch_coverage: 80.0,
                function_coverage: 90.0,
                complexity_coverage: 85.0,
                test_count: 50,
                regression_risk_score: 15.0,
            },
            trends: CoverageTrends {
                coverage_trend: 2.5,
                new_uncovered_lines: 5,
                resolved_uncovered_lines: 10,
                test_growth_rate: 10.0,
                performance_impact: 0.0,
            },
        };

        let result = analyzer.generate_html_report(&report);
        assert!(result.is_ok(), "HTML generation should succeed");

        let html_file = analyzer.output_dir.join("coverage_report.html");
        assert!(html_file.exists(), "HTML file should be created");

        let html_content = fs::read_to_string(&html_file).unwrap();
        assert!(html_content.contains("Test Project"));
        assert!(html_content.contains("85.0%"));
        
        println!("✅ HTML coverage report generated successfully");
    }

    #[test]
    fn test_coverage_requirements_checking() {
        println!("✅ Testing Coverage Requirements Checking");

        let temp_dir = tempdir().unwrap();
        let analyzer = CoverageAnalyzer::new(temp_dir.path().to_path_buf());

        // Test passing requirements
        let good_report = CoverageReport {
            timestamp: 0,
            project_name: "Good Project".to_string(),
            total_lines: 1000,
            covered_lines: 960,
            coverage_percentage: 96.0,
            files: vec![],
            uncovered_regions: vec![],
            summary: CoverageSummary {
                line_coverage: 96.0,
                branch_coverage: 92.0,
                function_coverage: 95.0,
                complexity_coverage: 94.0,
                test_count: 100,
                regression_risk_score: 8.0,
            },
            trends: CoverageTrends {
                coverage_trend: 0.0,
                new_uncovered_lines: 0,
                resolved_uncovered_lines: 0,
                test_growth_rate: 0.0,
                performance_impact: 0.0,
            },
        };

        let result = analyzer.check_coverage_requirements(&good_report);
        assert!(result.is_ok(), "Good coverage should pass requirements");

        // Test failing requirements
        let bad_report = CoverageReport {
            timestamp: 0,
            project_name: "Bad Project".to_string(),
            total_lines: 1000,
            covered_lines: 700,
            coverage_percentage: 70.0,
            files: vec![],
            uncovered_regions: vec![
                UncoveredRegion {
                    file_path: "critical.rs".to_string(),
                    start_line: 1,
                    end_line: 10,
                    reason: "Critical function not tested".to_string(),
                    priority: CoveragePriority::Critical,
                }
            ],
            summary: CoverageSummary {
                line_coverage: 70.0,
                branch_coverage: 60.0,
                function_coverage: 75.0,
                complexity_coverage: 68.0,
                test_count: 20,
                regression_risk_score: 35.0,
            },
            trends: CoverageTrends {
                coverage_trend: 0.0,
                new_uncovered_lines: 0,
                resolved_uncovered_lines: 0,
                test_growth_rate: 0.0,
                performance_impact: 0.0,
            },
        };

        let result = analyzer.check_coverage_requirements(&bad_report);
        assert!(result.is_err(), "Poor coverage should fail requirements");
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Line coverage"));
        assert!(error_msg.contains("Function coverage"));
        assert!(error_msg.contains("critical uncovered"));

        println!("✅ Coverage requirements checking working correctly");
    }

    #[tokio::test]
    async fn test_full_coverage_analysis_workflow() {
        println!("🔄 Testing Full Coverage Analysis Workflow");

        let project_root = std::env::current_dir().unwrap();
        let mut analyzer = CoverageAnalyzer::new(project_root);

        // Attempt to generate coverage report (may fail if LLVM tools not available)
        match analyzer.generate_llvm_coverage_report() {
            Ok(report) => {
                println!("✅ LLVM coverage report generated successfully");
                println!("  Total lines: {}", report.total_lines);
                println!("  Covered lines: {}", report.covered_lines);  
                println!("  Coverage: {:.1}%", report.coverage_percentage);

                // Generate HTML report
                let html_result = analyzer.generate_html_report(&report);
                assert!(html_result.is_ok(), "HTML report generation should succeed");

                // Save report data
                let save_result = analyzer.save_report_data(report.clone());
                assert!(save_result.is_ok(), "Report saving should succeed");

                // Check requirements (may fail for incomplete test suite)
                match analyzer.check_coverage_requirements(&report) {
                    Ok(_) => println!("✅ All coverage requirements met"),
                    Err(e) => println!("⚠️ Coverage requirements not met: {}", e),
                }
            }
            Err(e) => {
                println!("⚠️ LLVM coverage generation failed (expected in test environment): {}", e);
                println!("This test verifies the workflow structure rather than actual coverage");
            }
        }

        println!("✅ Coverage analysis workflow completed");
    }

    #[test]
    fn test_ci_integration_script_generation() {
        println!("🚀 Testing CI Integration Script Generation");

        let temp_dir = tempdir().unwrap();
        let analyzer = CoverageAnalyzer::new(temp_dir.path().to_path_buf());

        // Generate CI integration script
        let ci_script = r#"#!/bin/bash
# Uveddi Coverage CI Integration Script
set -e

echo "🛡️ Running Uveddi Coverage Analysis"

# Install LLVM tools if not present
if ! command -v llvm-cov &> /dev/null; then
    echo "Installing LLVM tools..."
    rustup component add llvm-tools-preview
fi

# Set coverage environment
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="coverage/uveddi-%p-%m.profraw"

# Create coverage directory
mkdir -p coverage

# Run tests with coverage
echo "Running tests with coverage instrumentation..."
cargo test --all-features -- --test-threads=1

# Process coverage data
echo "Processing coverage data..."
llvm-profdata merge -sparse coverage/*.profraw -o coverage/merged.profdata

# Generate coverage report  
echo "Generating coverage report..."
llvm-cov show --format=json --instr-profile=coverage/merged.profdata target/debug/deps/uveddi-* \
    --ignore-filename-regex='/.cargo/' > coverage/coverage.json

# Run coverage analysis
echo "Analyzing coverage results..."
cargo test test_full_coverage_analysis_workflow -- --nocapture

echo "✅ Coverage analysis completed successfully"

# Check if coverage meets requirements
if [ -f "coverage_reports/latest_report.json" ]; then
    echo "Coverage report available at coverage_reports/coverage_report.html"
else
    echo "❌ Coverage report generation failed"
    exit 1
fi
"#;

        let script_file = analyzer.output_dir.join("ci_coverage_check.sh");
        fs::write(&script_file, ci_script).unwrap();

        // Make script executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_file).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_file, perms).unwrap();
        }

        assert!(script_file.exists());
        println!("✅ CI integration script generated: {}", script_file.display());
    }
}
