#!/usr/bin/env python3
"""
UV-60: Rust Benchmark Report Generator
Processes Criterion benchmark results and generates comprehensive performance reports.
"""

import json
import sys
import os
import re
from pathlib import Path
from typing import Dict, Any, List, Optional
from datetime import datetime
import statistics

class RustReportGenerator:
    def __init__(self, results_file: str = "rust-benchmark-results.json"):
        self.results_file = results_file
        self.report_file = "rust-performance-report.json"
        self.summary_file = "performance-summary.md"
        
    def parse_criterion_results(self, file_path: str) -> Dict[str, Any]:
        """Parse Criterion benchmark results into structured format."""
        report = {
            "timestamp": datetime.utcnow().isoformat(),
            "source_file": file_path,
            "benchmarks": {},
            "summary": {},
            "metadata": {
                "criterion_version": "unknown",
                "features": ["tree-sitter"],
                "rust_version": "unknown"
            }
        }
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
            
            # Parse Criterion output
            lines = content.split('\n')
            current_group = None
            current_benchmark = None
            
            for line in lines:
                line = line.strip()
                
                # Group detection
                if 'group:' in line or 'Benchmarking' in line:
                    if 'group:' in line:
                        current_group = line.split('group:')[1].strip()
                    elif 'Benchmarking' in line:
                        current_group = line.split('Benchmarking')[1].strip().split('/')[0]
                    
                    if current_group not in report["benchmarks"]:
                        report["benchmarks"][current_group] = {}
                
                # Individual benchmark detection
                elif current_group and ('time:' in line or 'Found' in line):
                    if 'Found' in line and 'outliers' in line:
                        # Extract outlier information
                        pass
                    elif 'time:' in line:
                        # Extract timing information
                        time_match = re.search(r'time:\s*([\d.]+)\s*(ns|μs|ms|s)', line)
                        if time_match:
                            time_val = float(time_match.group(1))
                            unit = time_match.group(2)
                            
                            # Convert to microseconds for consistency
                            if unit == 'ns':
                                time_us = time_val / 1000
                            elif unit == 'μs':
                                time_us = time_val
                            elif unit == 'ms':
                                time_us = time_val * 1000
                            elif unit == 's':
                                time_us = time_val * 1000000
                            else:
                                time_us = time_val
                            
                            # Use line content to determine benchmark name
                            bench_name = f"benchmark_{len(report['benchmarks'][current_group])}"
                            report["benchmarks"][current_group][bench_name] = {
                                "execution_time_us": time_us,
                                "execution_time_ms": time_us / 1000,
                                "unit": unit,
                                "raw_value": time_val
                            }
                
                # Throughput detection
                elif 'throughput:' in line:
                    throughput_match = re.search(r'throughput:\s*([\d.]+)\s*(\w+)', line)
                    if throughput_match and current_group:
                        throughput_val = float(throughput_match.group(1))
                        throughput_unit = throughput_match.group(2)
                        
                        # Add throughput to last benchmark in current group
                        if report["benchmarks"][current_group]:
                            last_bench = list(report["benchmarks"][current_group].keys())[-1]
                            report["benchmarks"][current_group][last_bench]["throughput"] = {
                                "value": throughput_val,
                                "unit": throughput_unit
                            }
        
        except Exception as e:
            print(f"Error parsing Criterion results: {e}")
            return report
        
        # Calculate summary statistics
        self.calculate_summary_stats(report)
        return report
    
    def calculate_summary_stats(self, report: Dict[str, Any]) -> None:
        """Calculate summary statistics across all benchmarks."""
        all_times = []
        all_throughputs = []
        benchmark_count = 0
        
        for group_name, group_benchmarks in report["benchmarks"].items():
            group_times = []
            group_throughputs = []
            
            for bench_name, bench_data in group_benchmarks.items():
                benchmark_count += 1
                exec_time = bench_data.get("execution_time_us", 0)
                all_times.append(exec_time)
                group_times.append(exec_time)
                
                if "throughput" in bench_data:
                    throughput = bench_data["throughput"]["value"]
                    all_throughputs.append(throughput)
                    group_throughputs.append(throughput)
            
            # Group-level statistics
            if group_times:
                report["benchmarks"][group_name]["_group_stats"] = {
                    "count": len(group_times),
                    "mean_time_us": statistics.mean(group_times),
                    "median_time_us": statistics.median(group_times),
                    "min_time_us": min(group_times),
                    "max_time_us": max(group_times),
                    "std_dev_us": statistics.stdev(group_times) if len(group_times) > 1 else 0
                }
                
                if group_throughputs:
                    report["benchmarks"][group_name]["_group_stats"]["mean_throughput"] = statistics.mean(group_throughputs)
        
        # Overall summary
        report["summary"] = {
            "total_benchmarks": benchmark_count,
            "total_groups": len(report["benchmarks"]),
            "overall_stats": {}
        }
        
        if all_times:
            report["summary"]["overall_stats"] = {
                "mean_execution_time_us": statistics.mean(all_times),
                "median_execution_time_us": statistics.median(all_times),
                "min_execution_time_us": min(all_times),
                "max_execution_time_us": max(all_times),
                "total_execution_time_us": sum(all_times),
                "std_dev_us": statistics.stdev(all_times) if len(all_times) > 1 else 0
            }
            
        if all_throughputs:
            report["summary"]["overall_stats"]["mean_throughput"] = statistics.mean(all_throughputs)
    
    def calculate_performance_grade(self, report: Dict[str, Any]) -> str:
        """Calculate overall performance grade based on execution times."""
        overall_stats = report.get("summary", {}).get("overall_stats", {})
        mean_time_ms = overall_stats.get("mean_execution_time_us", 0) / 1000
        
        # Performance grading based on mean execution time
        if mean_time_ms < 1:      # < 1ms
            return "A"
        elif mean_time_ms < 10:   # < 10ms
            return "B"  
        elif mean_time_ms < 100:  # < 100ms
            return "C"
        elif mean_time_ms < 1000: # < 1s
            return "D"
        else:
            return "F"
    
    def generate_markdown_summary(self, report: Dict[str, Any]) -> str:
        """Generate Markdown summary for PR comments."""
        summary = report.get("summary", {})
        overall_stats = summary.get("overall_stats", {})
        performance_grade = self.calculate_performance_grade(report)
        
        md = "## 🦀 Rust Benchmark Results\n\n"
        md += f"**Performance Grade:** `{performance_grade}`\n\n"
        
        md += "### Summary\n"
        md += f"- **Total Benchmarks:** {summary.get('total_benchmarks', 0)}\n"
        md += f"- **Benchmark Groups:** {summary.get('total_groups', 0)}\n"
        
        if overall_stats:
            mean_ms = overall_stats.get("mean_execution_time_us", 0) / 1000
            md += f"- **Mean Execution Time:** {mean_ms:.2f}ms\n"
            md += f"- **Median Execution Time:** {overall_stats.get('median_execution_time_us', 0) / 1000:.2f}ms\n"
            
            total_time_s = overall_stats.get("total_execution_time_us", 0) / 1000000
            md += f"- **Total Execution Time:** {total_time_s:.2f}s\n"
        
        md += "\n### Performance by Group\n"
        md += "| Group | Benchmarks | Mean Time (ms) | Min/Max (ms) |\n"
        md += "|-------|------------|----------------|---------------|\n"
        
        for group_name, group_data in report.get("benchmarks", {}).items():
            if "_group_stats" in group_data:
                stats = group_data["_group_stats"]
                count = stats.get("count", 0)
                mean_ms = stats.get("mean_time_us", 0) / 1000
                min_ms = stats.get("min_time_us", 0) / 1000
                max_ms = stats.get("max_time_us", 0) / 1000
                md += f"| {group_name} | {count} | {mean_ms:.2f} | {min_ms:.2f}/{max_ms:.2f} |\n"
        
        md += f"\n---\n"
        md += f"*Generated by UV-60 Benchmark Integration • {report.get('timestamp', 'unknown')}*"
        
        return md
    
    def generate_report(self) -> bool:
        """Generate comprehensive performance report."""
        print(f"📊 Generating Rust performance report from {self.results_file}...")
        
        # Parse benchmark results
        report = self.parse_criterion_results(self.results_file)
        
        if not report.get("benchmarks"):
            print("⚠️  No benchmark data found to process")
            return False
        
        # Add performance grade
        report["summary"]["performance_grade"] = self.calculate_performance_grade(report)
        
        # Save JSON report
        with open(self.report_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        # Generate Markdown summary
        markdown_summary = self.generate_markdown_summary(report)
        with open(self.summary_file, 'w') as f:
            f.write(markdown_summary)
        
        print(f"✅ Reports generated:")
        print(f"   - JSON: {self.report_file}")
        print(f"   - Markdown: {self.summary_file}")
        print(f"   - Performance Grade: {report['summary']['performance_grade']}")
        
        return True

def main():
    """Main entry point for report generation."""
    if len(sys.argv) > 1:
        results_file = sys.argv[1]
    else:
        results_file = "rust-benchmark-results.json"
    
    generator = RustReportGenerator(results_file)
    
    if not os.path.exists(results_file):
        print(f"❌ Results file not found: {results_file}")
        sys.exit(1)
    
    success = generator.generate_report()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()