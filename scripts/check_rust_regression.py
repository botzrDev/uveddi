#!/usr/bin/env python3
"""
UV-60: Rust Benchmark Regression Detection
Compares current Rust benchmark results against baseline to detect performance regressions.
"""

import json
import sys
import os
from pathlib import Path
from typing import Dict, Any, Optional, List
import statistics

class RustRegressionChecker:
    def __init__(self, results_file: str = "rust-benchmark-results.json"):
        self.results_file = results_file
        self.baseline_file = "benchmarks/baselines/rust-benchmark-results.json"
        self.report_file = "rust-regression-report.json"
        
        # Regression thresholds (configurable)
        self.thresholds = {
            "performance_degradation": 0.15,  # 15% slower is significant
            "memory_increase": 0.20,          # 20% more memory usage
            "throughput_decrease": 0.10,      # 10% less throughput
            "error_rate_increase": 0.05       # 5% more errors
        }
        
    def load_json(self, file_path: str) -> Optional[Dict[str, Any]]:
        """Load JSON file with error handling."""
        try:
            if not os.path.exists(file_path):
                return None
            with open(file_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error loading {file_path}: {e}")
            return None
    
    def parse_criterion_output(self, file_path: str) -> Dict[str, Any]:
        """Parse Criterion benchmark output into structured format."""
        results = {"benchmarks": {}, "summary": {}}
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
                
            # Parse Criterion output (basic implementation)
            # In a real implementation, you'd parse the JSON output from Criterion
            lines = content.split('\n')
            current_group = None
            
            for line in lines:
                line = line.strip()
                if 'group:' in line:
                    current_group = line.split('group:')[1].strip()
                    results["benchmarks"][current_group] = {}
                elif 'time:' in line and current_group:
                    # Extract timing information
                    parts = line.split()
                    for i, part in enumerate(parts):
                        if part == 'time:' and i + 1 < len(parts):
                            time_str = parts[i + 1]
                            # Parse time (ms, μs, ns)
                            if 'ms' in time_str:
                                time_val = float(time_str.replace('ms', '')) * 1000
                            elif 'μs' in time_str:
                                time_val = float(time_str.replace('μs', ''))
                            elif 'ns' in time_str:
                                time_val = float(time_str.replace('ns', '')) / 1000
                            else:
                                time_val = float(time_str)
                            
                            results["benchmarks"][current_group]["execution_time_us"] = time_val
                            break
                            
        except Exception as e:
            print(f"Error parsing Criterion output: {e}")
            
        return results
    
    def calculate_regression_metrics(self, current: Dict[str, Any], baseline: Dict[str, Any]) -> Dict[str, Any]:
        """Calculate regression metrics between current and baseline results."""
        regressions = {
            "performance_regressions": [],
            "memory_regressions": [],
            "throughput_regressions": [],
            "summary": {
                "total_benchmarks": 0,
                "regressed_benchmarks": 0,
                "improved_benchmarks": 0,
                "stable_benchmarks": 0
            }
        }
        
        current_benchmarks = current.get("benchmarks", {})
        baseline_benchmarks = baseline.get("benchmarks", {})
        
        for bench_name in current_benchmarks:
            if bench_name not in baseline_benchmarks:
                continue
                
            regressions["summary"]["total_benchmarks"] += 1
            current_bench = current_benchmarks[bench_name]
            baseline_bench = baseline_benchmarks[bench_name]
            
            # Check execution time regression
            current_time = current_bench.get("execution_time_us", 0)
            baseline_time = baseline_bench.get("execution_time_us", 0)
            
            if baseline_time > 0:
                time_change = (current_time - baseline_time) / baseline_time
                
                if time_change > self.thresholds["performance_degradation"]:
                    regressions["performance_regressions"].append({
                        "benchmark": bench_name,
                        "current_time_us": current_time,
                        "baseline_time_us": baseline_time,
                        "change_percent": time_change * 100,
                        "severity": "high" if time_change > 0.3 else "medium"
                    })
                    regressions["summary"]["regressed_benchmarks"] += 1
                elif time_change < -0.05:  # 5% improvement
                    regressions["summary"]["improved_benchmarks"] += 1
                else:
                    regressions["summary"]["stable_benchmarks"] += 1
        
        return regressions
    
    def generate_alert_summary(self, regressions: Dict[str, Any]) -> str:
        """Generate human-readable alert summary."""
        summary = regressions["summary"]
        total = summary["total_benchmarks"]
        regressed = summary["regressed_benchmarks"]
        improved = summary["improved_benchmarks"]
        
        if regressed == 0:
            return f"✅ No performance regressions detected ({total} benchmarks checked)"
        
        severity_counts = {}
        for reg in regressions["performance_regressions"]:
            severity = reg["severity"]
            severity_counts[severity] = severity_counts.get(severity, 0) + 1
        
        alert = f"❌ Performance regressions detected:\n"
        alert += f"   - {regressed}/{total} benchmarks regressed\n"
        alert += f"   - {improved} benchmarks improved\n"
        
        if severity_counts:
            alert += f"   - Severity breakdown: {severity_counts}\n"
        
        alert += "\nTop regressions:\n"
        for reg in sorted(regressions["performance_regressions"], 
                         key=lambda x: x["change_percent"], reverse=True)[:5]:
            alert += f"   - {reg['benchmark']}: +{reg['change_percent']:.1f}% slower\n"
        
        return alert
    
    def check_regression(self) -> bool:
        """Main regression check function. Returns False if regressions detected."""
        print("🔍 Checking Rust benchmark regressions...")
        
        # Load current results
        current_results = self.load_json(self.results_file)
        if not current_results:
            # Try to parse Criterion output
            if os.path.exists(self.results_file):
                current_results = self.parse_criterion_output(self.results_file)
            else:
                print(f"❌ Current results file not found: {self.results_file}")
                return False
        
        # Load baseline results
        baseline_results = self.load_json(self.baseline_file)
        if not baseline_results:
            print(f"⚠️  No baseline found at {self.baseline_file}. Skipping regression check.")
            print("   This is normal for the first run. Results will be stored as baseline.")
            return True
        
        # Calculate regressions
        regressions = self.calculate_regression_metrics(current_results, baseline_results)
        
        # Save regression report
        with open(self.report_file, 'w') as f:
            json.dump(regressions, f, indent=2)
        
        # Generate and print summary
        alert_summary = self.generate_alert_summary(regressions)
        print(alert_summary)
        
        # Return True if no significant regressions
        return regressions["summary"]["regressed_benchmarks"] == 0

def main():
    """Main entry point for regression checking."""
    checker = RustRegressionChecker()
    
    # Check for environment variable overrides
    if "RUST_RESULTS_FILE" in os.environ:
        checker.results_file = os.environ["RUST_RESULTS_FILE"]
    
    # Override thresholds from environment if available
    for key, default_value in checker.thresholds.items():
        env_key = f"THRESHOLD_{key.upper()}"
        if env_key in os.environ:
            try:
                checker.thresholds[key] = float(os.environ[env_key])
            except ValueError:
                print(f"⚠️  Invalid threshold value for {env_key}, using default")
    
    # Perform regression check
    no_regressions = checker.check_regression()
    
    if not no_regressions:
        print("\n💡 Regression mitigation suggestions:")
        print("   - Review recent changes for performance impact")
        print("   - Run profiler to identify bottlenecks")
        print("   - Consider algorithmic optimizations")
        print("   - Check for memory leaks or excessive allocations")
        sys.exit(1)
    
    sys.exit(0)

if __name__ == "__main__":
    main()