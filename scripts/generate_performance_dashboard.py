#!/usr/bin/env python3
"""
UV-60: Performance Dashboard Generator
Creates comprehensive performance dashboard combining Rust and JavaScript benchmarks.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional
from datetime import datetime
import statistics

class PerformanceDashboard:
    def __init__(self):
        self.rust_results_file = "benchmark-results/rust-benchmark-results/rust-benchmark-results.json"
        self.rust_report_file = "benchmark-results/rust-benchmark-results/rust-performance-report.json"
        self.js_results_file = "benchmark-results/js-benchmark-report/benchmark-report.json"
        self.dashboard_file = "performance-dashboard.html"
        self.trends_file = "performance-trends.png"
        self.summary_file = "performance-summary.md"
        
    def load_json(self, file_path: str) -> Optional[Dict[str, Any]]:
        """Load JSON file with error handling."""
        try:
            if not os.path.exists(file_path):
                print(f"⚠️  File not found: {file_path}")
                return None
            with open(file_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error loading {file_path}: {e}")
            return None
    
    def analyze_rust_performance(self, rust_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze Rust benchmark performance."""
        if not rust_data:
            return {"status": "no_data", "analysis": "No Rust benchmark data available"}
        
        summary = rust_data.get("summary", {})
        overall_stats = summary.get("overall_stats", {})
        
        analysis = {
            "status": "success",
            "total_benchmarks": summary.get("total_benchmarks", 0),
            "performance_grade": summary.get("performance_grade", "Unknown"),
            "mean_execution_time_ms": overall_stats.get("mean_execution_time_us", 0) / 1000,
            "total_execution_time_s": overall_stats.get("total_execution_time_us", 0) / 1000000,
            "groups": {}
        }
        
        # Analyze by group
        for group_name, group_data in rust_data.get("benchmarks", {}).items():
            if "_group_stats" in group_data:
                stats = group_data["_group_stats"]
                analysis["groups"][group_name] = {
                    "count": stats.get("count", 0),
                    "mean_time_ms": stats.get("mean_time_us", 0) / 1000,
                    "efficiency_score": self.calculate_efficiency_score(stats.get("mean_time_us", 0))
                }
        
        return analysis
    
    def analyze_js_performance(self, js_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze JavaScript rendering service performance."""
        if not js_data:
            return {"status": "no_data", "analysis": "No JavaScript benchmark data available"}
        
        summary = js_data.get("summary", {})
        
        analysis = {
            "status": "success",
            "performance_grade": summary.get("performance_grade", "Unknown"),
            "uv78_compliance": summary.get("uv78_target_compliance", 0),
            "overall_success_rate": summary.get("overall_success_rate", 0),
            "cache_speedup": js_data.get("cache_performance", {}).get("speedup_factor", 0),
            "rendering_tests": {}
        }
        
        # Analyze rendering tests
        for test_name, test_data in js_data.get("tests", {}).items():
            analysis["rendering_tests"][test_name] = {
                "success_rate": test_data.get("success_rate", 0),
                "under_100ms_rate": test_data.get("under_100ms_rate", 0),
                "mean_time_ms": test_data.get("stats", {}).get("mean", 0),
                "p95_time_ms": test_data.get("stats", {}).get("p95", 0)
            }
        
        return analysis
    
    def calculate_efficiency_score(self, time_us: float) -> str:
        """Calculate efficiency score based on execution time."""
        time_ms = time_us / 1000
        if time_ms < 1:
            return "Excellent"
        elif time_ms < 10:
            return "Good"
        elif time_ms < 100:
            return "Fair"
        else:
            return "Poor"
    
    def calculate_overall_score(self, rust_analysis: Dict[str, Any], js_analysis: Dict[str, Any]) -> str:
        """Calculate overall performance score."""
        rust_grade = rust_analysis.get("performance_grade", "F")
        js_grade = js_analysis.get("performance_grade", "F")
        
        # Convert grades to numeric scores
        grade_scores = {"A": 4, "B": 3, "C": 2, "D": 1, "F": 0}
        rust_score = grade_scores.get(rust_grade, 0)
        js_score = grade_scores.get(js_grade, 0)
        
        # Weight JavaScript slightly higher as it's user-facing
        overall_score = (rust_score * 0.4 + js_score * 0.6)
        
        if overall_score >= 3.5:
            return "A"
        elif overall_score >= 2.5:
            return "B"
        elif overall_score >= 1.5:
            return "C"
        elif overall_score >= 0.5:
            return "D"
        else:
            return "F"
    
    def generate_html_dashboard(self, rust_analysis: Dict[str, Any], js_analysis: Dict[str, Any]) -> str:
        """Generate HTML performance dashboard."""
        overall_score = self.calculate_overall_score(rust_analysis, js_analysis)
        timestamp = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")
        
        html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>UV-60 Performance Dashboard</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .header {{ text-align: center; margin-bottom: 30px; }}
        .grade {{ font-size: 3em; font-weight: bold; margin: 10px 0; }}
        .grade.A {{ color: #22c55e; }}
        .grade.B {{ color: #84cc16; }}
        .grade.C {{ color: #eab308; }}
        .grade.D {{ color: #f97316; }}
        .grade.F {{ color: #ef4444; }}
        .section {{ margin: 20px 0; }}
        .metrics-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; }}
        .metric-card {{ background: #f8fafc; padding: 15px; border-radius: 6px; border-left: 4px solid #3b82f6; }}
        .metric-value {{ font-size: 2em; font-weight: bold; color: #1e40af; }}
        .metric-label {{ color: #64748b; font-size: 0.9em; }}
        .status-indicator {{ display: inline-block; width: 12px; height: 12px; border-radius: 50%; margin-right: 8px; }}
        .status-success {{ background-color: #22c55e; }}
        .status-warning {{ background-color: #eab308; }}
        .status-error {{ background-color: #ef4444; }}
        .benchmarks-table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
        .benchmarks-table th, .benchmarks-table td {{ padding: 8px 12px; text-align: left; border-bottom: 1px solid #e2e8f0; }}
        .benchmarks-table th {{ background-color: #f1f5f9; font-weight: 600; }}
        .footer {{ text-align: center; margin-top: 30px; color: #64748b; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 UV-60 Performance Dashboard</h1>
            <div class="grade {overall_score}">{overall_score}</div>
            <p>Overall Performance Grade</p>
            <p><em>Generated: {timestamp}</em></p>
        </div>
        
        <div class="section">
            <h2>📊 Performance Overview</h2>
            <div class="metrics-grid">
"""
        
        # Rust metrics
        if rust_analysis.get("status") == "success":
            html += f"""
                <div class="metric-card">
                    <div class="metric-value">{rust_analysis.get('performance_grade', 'N/A')}</div>
                    <div class="metric-label">Rust Performance Grade</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{rust_analysis.get('total_benchmarks', 0)}</div>
                    <div class="metric-label">Rust Benchmarks</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{rust_analysis.get('mean_execution_time_ms', 0):.2f}ms</div>
                    <div class="metric-label">Mean Rust Execution Time</div>
                </div>
"""
        
        # JavaScript metrics
        if js_analysis.get("status") == "success":
            html += f"""
                <div class="metric-card">
                    <div class="metric-value">{js_analysis.get('performance_grade', 'N/A')}</div>
                    <div class="metric-label">JS Rendering Grade</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{js_analysis.get('uv78_compliance', 0):.1f}%</div>
                    <div class="metric-label">UV-78 Compliance (&lt;100ms)</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{js_analysis.get('cache_speedup', 0):.1f}x</div>
                    <div class="metric-label">Cache Speedup Factor</div>
                </div>
"""
        
        html += """
            </div>
        </div>
"""
        
        # Rust benchmarks detail
        if rust_analysis.get("status") == "success" and rust_analysis.get("groups"):
            html += """
        <div class="section">
            <h2>🦀 Rust Benchmark Details</h2>
            <table class="benchmarks-table">
                <thead>
                    <tr>
                        <th>Benchmark Group</th>
                        <th>Count</th>
                        <th>Mean Time (ms)</th>
                        <th>Efficiency</th>
                    </tr>
                </thead>
                <tbody>
"""
            for group_name, group_data in rust_analysis["groups"].items():
                efficiency = group_data.get("efficiency_score", "Unknown")
                status_class = {
                    "Excellent": "status-success",
                    "Good": "status-success", 
                    "Fair": "status-warning",
                    "Poor": "status-error"
                }.get(efficiency, "status-warning")
                
                html += f"""
                    <tr>
                        <td>{group_name}</td>
                        <td>{group_data.get('count', 0)}</td>
                        <td>{group_data.get('mean_time_ms', 0):.2f}</td>
                        <td><span class="status-indicator {status_class}"></span>{efficiency}</td>
                    </tr>
"""
            html += """
                </tbody>
            </table>
        </div>
"""
        
        # JavaScript rendering details
        if js_analysis.get("status") == "success" and js_analysis.get("rendering_tests"):
            html += """
        <div class="section">
            <h2>🎨 Rendering Performance Details</h2>
            <table class="benchmarks-table">
                <thead>
                    <tr>
                        <th>Test Type</th>
                        <th>Success Rate</th>
                        <th>&lt;100ms Rate</th>
                        <th>Mean Time (ms)</th>
                        <th>P95 Time (ms)</th>
                    </tr>
                </thead>
                <tbody>
"""
            for test_name, test_data in js_analysis["rendering_tests"].items():
                html += f"""
                    <tr>
                        <td>{test_name}</td>
                        <td>{test_data.get('success_rate', 0):.1f}%</td>
                        <td>{test_data.get('under_100ms_rate', 0):.1f}%</td>
                        <td>{test_data.get('mean_time_ms', 0):.2f}</td>
                        <td>{test_data.get('p95_time_ms', 0):.2f}</td>
                    </tr>
"""
            html += """
                </tbody>
            </table>
        </div>
"""
        
        html += f"""
        <div class="footer">
            <p>UV-60: Automated Performance Monitoring • <a href="https://github.com/your-org/uveddi">Uveddi Project</a></p>
        </div>
    </div>
</body>
</html>"""
        
        return html
    
    def generate_markdown_summary(self, rust_analysis: Dict[str, Any], js_analysis: Dict[str, Any]) -> str:
        """Generate comprehensive Markdown summary for PR comments."""
        overall_score = self.calculate_overall_score(rust_analysis, js_analysis)
        
        md = "## 🚀 Performance Dashboard Summary\n\n"
        md += f"**Overall Performance Grade:** `{overall_score}`\n\n"
        
        # Status indicators
        rust_status = "✅" if rust_analysis.get("status") == "success" else "❌"
        js_status = "✅" if js_analysis.get("status") == "success" else "❌"
        
        md += "### Component Status\n"
        md += f"- {rust_status} **Rust Benchmarks:** {rust_analysis.get('performance_grade', 'N/A')} grade\n"
        md += f"- {js_status} **Rendering Service:** {js_analysis.get('performance_grade', 'N/A')} grade\n\n"
        
        # Key metrics
        md += "### Key Metrics\n"
        
        if rust_analysis.get("status") == "success":
            md += f"**Rust Performance:**\n"
            md += f"- {rust_analysis.get('total_benchmarks', 0)} benchmarks executed\n"
            md += f"- {rust_analysis.get('mean_execution_time_ms', 0):.2f}ms mean execution time\n"
            md += f"- {rust_analysis.get('total_execution_time_s', 0):.2f}s total execution time\n\n"
        
        if js_analysis.get("status") == "success":
            md += f"**Rendering Performance:**\n"
            md += f"- {js_analysis.get('uv78_compliance', 0):.1f}% UV-78 compliance (<100ms target)\n"
            md += f"- {js_analysis.get('overall_success_rate', 0):.1f}% overall success rate\n"
            md += f"- {js_analysis.get('cache_speedup', 0):.1f}x cache speedup factor\n\n"
        
        md += "---\n"
        md += f"*Full dashboard: [performance-dashboard.html](performance-dashboard.html)*"
        
        return md
    
    def generate_dashboard(self) -> bool:
        """Generate complete performance dashboard."""
        print("📊 Generating comprehensive performance dashboard...")
        
        # Load benchmark data
        rust_data = self.load_json(self.rust_report_file)
        if not rust_data:
            rust_data = self.load_json(self.rust_results_file)
        
        js_data = self.load_json(self.js_results_file)
        
        # Analyze performance
        rust_analysis = self.analyze_rust_performance(rust_data)
        js_analysis = self.analyze_js_performance(js_data)
        
        # Generate HTML dashboard
        html_content = self.generate_html_dashboard(rust_analysis, js_analysis)
        with open(self.dashboard_file, 'w') as f:
            f.write(html_content)
        
        # Generate Markdown summary
        markdown_summary = self.generate_markdown_summary(rust_analysis, js_analysis)
        with open(self.summary_file, 'w') as f:
            f.write(markdown_summary)
        
        print(f"✅ Performance dashboard generated:")
        print(f"   - HTML Dashboard: {self.dashboard_file}")
        print(f"   - Summary: {self.summary_file}")
        print(f"   - Overall Grade: {self.calculate_overall_score(rust_analysis, js_analysis)}")
        
        return True

def main():
    """Main entry point for dashboard generation."""
    dashboard = PerformanceDashboard()
    success = dashboard.generate_dashboard()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()