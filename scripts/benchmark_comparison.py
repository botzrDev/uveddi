#!/usr/bin/env python3
"""
UV-60: Benchmark Result Comparison and Trending Analysis
Compares benchmark results across commits and generates performance trends.
"""

import json
import sys
import os
import argparse
from pathlib import Path
from typing import Dict, Any, List, Optional, Tuple
from datetime import datetime, timedelta
import statistics
import math

class BenchmarkComparator:
    def __init__(self):
        self.rust_results_dir = "benchmark-results/rust-benchmark-results"
        self.js_results_dir = "benchmark-results/js-benchmark-report"
        self.history_dir = "performance-history"
        self.output_dir = "performance-analysis"
        
        # Ensure output directory exists
        os.makedirs(self.output_dir, exist_ok=True)
        os.makedirs(self.history_dir, exist_ok=True)
    
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
    
    def save_json(self, file_path: str, data: Dict[str, Any]) -> bool:
        """Save JSON data to file."""
        try:
            os.makedirs(os.path.dirname(file_path), exist_ok=True)
            with open(file_path, 'w') as f:
                json.dump(data, f, indent=2)
            return True
        except Exception as e:
            print(f"Error saving {file_path}: {e}")
            return False
    
    def normalize_metrics(self, results: Dict[str, Any], benchmark_type: str) -> Dict[str, float]:
        """Extract and normalize key metrics from benchmark results."""
        normalized = {
            "timestamp": datetime.utcnow().isoformat(),
            "benchmark_type": benchmark_type,
            "git_commit": os.environ.get("GITHUB_SHA", "unknown")[:8],
            "git_branch": os.environ.get("GITHUB_REF_NAME", "unknown")
        }
        
        if benchmark_type == "rust":
            summary = results.get("summary", {})
            overall_stats = summary.get("overall_stats", {})
            
            normalized.update({
                "total_benchmarks": summary.get("total_benchmarks", 0),
                "mean_execution_time_us": overall_stats.get("mean_execution_time_us", 0),
                "median_execution_time_us": overall_stats.get("median_execution_time_us", 0),
                "max_execution_time_us": overall_stats.get("max_execution_time_us", 0),
                "total_execution_time_us": overall_stats.get("total_execution_time_us", 0),
                "performance_grade": self.grade_to_numeric(summary.get("performance_grade", "F"))
            })
            
        elif benchmark_type == "javascript":
            summary = results.get("summary", {})
            cache_performance = results.get("cache_performance", {})
            
            normalized.update({
                "overall_success_rate": summary.get("overall_success_rate", 0),
                "uv78_target_compliance": summary.get("uv78_target_compliance", 0),
                "performance_grade": self.grade_to_numeric(summary.get("performance_grade", "F")),
                "cache_speedup": cache_performance.get("speedup_factor", 0),
                "cache_cold_ms": cache_performance.get("cold_cache_ms", 0),
                "cache_warm_ms": cache_performance.get("warm_cache_ms", 0)
            })
            
            # Extract test-specific metrics
            for test_name, test_data in results.get("tests", {}).items():
                stats = test_data.get("stats", {})
                normalized[f"{test_name}_mean_ms"] = stats.get("mean", 0)
                normalized[f"{test_name}_p95_ms"] = stats.get("p95", 0)
                normalized[f"{test_name}_success_rate"] = test_data.get("success_rate", 0)
        
        return normalized
    
    def grade_to_numeric(self, grade: str) -> float:
        """Convert letter grade to numeric value for comparison."""
        grade_map = {"A": 4.0, "B": 3.0, "C": 2.0, "D": 1.0, "F": 0.0}
        return grade_map.get(grade, 0.0)
    
    def calculate_percentage_change(self, current: float, baseline: float) -> float:
        """Calculate percentage change between current and baseline values."""
        if baseline == 0:
            return 0.0
        return ((current - baseline) / baseline) * 100
    
    def calculate_trend(self, values: List[float], window_size: int = 5) -> Dict[str, Any]:
        """Calculate trend analysis for a series of values."""
        if len(values) < 2:
            return {"trend": "insufficient_data", "slope": 0, "confidence": 0}
        
        # Use linear regression to calculate trend
        n = len(values)
        x_values = list(range(n))
        
        # Calculate slope using least squares
        x_mean = statistics.mean(x_values)
        y_mean = statistics.mean(values)
        
        numerator = sum((x_values[i] - x_mean) * (values[i] - y_mean) for i in range(n))
        denominator = sum((x_values[i] - x_mean) ** 2 for i in range(n))
        
        if denominator == 0:
            slope = 0
        else:
            slope = numerator / denominator
        
        # Calculate R-squared for confidence
        if len(values) > 2:
            predicted = [y_mean + slope * (x - x_mean) for x in x_values]
            ss_res = sum((values[i] - predicted[i]) ** 2 for i in range(n))
            ss_tot = sum((values[i] - y_mean) ** 2 for i in range(n))
            r_squared = 1 - (ss_res / ss_tot) if ss_tot != 0 else 0
        else:
            r_squared = 0
        
        # Determine trend direction
        if abs(slope) < 0.01:
            trend = "stable"
        elif slope > 0:
            trend = "improving" if self.is_improvement_metric(values) else "degrading"
        else:
            trend = "degrading" if self.is_improvement_metric(values) else "improving"
        
        return {
            "trend": trend,
            "slope": slope,
            "confidence": r_squared,
            "recent_average": statistics.mean(values[-window_size:]) if len(values) >= window_size else statistics.mean(values)
        }
    
    def is_improvement_metric(self, values: List[float]) -> bool:
        """Determine if higher values indicate improvement for this metric."""
        # This is a simplified heuristic - in practice, you'd want to know the metric type
        # For most performance metrics, lower is better (execution time, memory usage)
        # For some metrics, higher is better (success rate, cache hit rate)
        return False  # Default assumption: lower is better
    
    def compare_with_baseline(self, current: Dict[str, float], baseline: Dict[str, float]) -> Dict[str, Any]:
        """Compare current metrics with baseline and identify regressions/improvements."""
        comparison = {
            "timestamp": datetime.utcnow().isoformat(),
            "baseline_commit": baseline.get("git_commit", "unknown"),
            "current_commit": current.get("git_commit", "unknown"),
            "changes": {},
            "summary": {
                "total_metrics": 0,
                "regressions": 0,
                "improvements": 0,
                "stable": 0
            }
        }
        
        # Performance regression thresholds
        thresholds = {
            "execution_time": 15.0,    # 15% slower
            "memory_usage": 20.0,      # 20% more memory
            "success_rate": 5.0,       # 5% lower success rate
            "performance_grade": 0.5,  # Half grade drop
            "cache_performance": 15.0   # 15% worse caching
        }
        
        for metric in current:
            if metric in baseline and isinstance(current[metric], (int, float)):
                current_val = current[metric]
                baseline_val = baseline[metric]
                
                if baseline_val != 0:
                    change_percent = self.calculate_percentage_change(current_val, baseline_val)
                    
                    # Determine if this is a regression based on metric type
                    is_regression = self.is_regression(metric, change_percent, thresholds)
                    severity = self.calculate_severity(change_percent, thresholds.get(metric, 10.0))
                    
                    comparison["changes"][metric] = {
                        "current": current_val,
                        "baseline": baseline_val,
                        "change_percent": change_percent,
                        "is_regression": is_regression,
                        "severity": severity,
                        "status": "regression" if is_regression else ("improvement" if change_percent < -5 else "stable")
                    }
                    
                    # Update summary
                    comparison["summary"]["total_metrics"] += 1
                    if is_regression:
                        comparison["summary"]["regressions"] += 1
                    elif change_percent < -5:  # 5% improvement threshold
                        comparison["summary"]["improvements"] += 1
                    else:
                        comparison["summary"]["stable"] += 1
        
        return comparison
    
    def is_regression(self, metric: str, change_percent: float, thresholds: Dict[str, float]) -> bool:
        """Determine if a metric change represents a performance regression."""
        threshold = thresholds.get(metric, 10.0)
        
        # Metrics where higher is better
        improvement_metrics = ["success_rate", "performance_grade", "cache_speedup", "uv78_target_compliance"]
        
        if any(improvement_metric in metric.lower() for improvement_metric in improvement_metrics):
            return change_percent < -threshold  # Decrease is bad
        else:
            return change_percent > threshold   # Increase is bad (execution time, memory, etc.)
    
    def calculate_severity(self, change_percent: float, threshold: float) -> str:
        """Calculate severity level based on change percentage."""
        abs_change = abs(change_percent)
        
        if abs_change < threshold:
            return "none"
        elif abs_change < threshold * 1.5:
            return "low"
        elif abs_change < threshold * 2.5:
            return "medium"
        elif abs_change < threshold * 4:
            return "high"
        else:
            return "critical"
    
    def analyze_historical_trends(self, metric_name: str, history: List[Dict[str, Any]], window_size: int = 10) -> Dict[str, Any]:
        """Analyze historical trends for a specific metric."""
        values = []
        timestamps = []
        
        for entry in history:
            if metric_name in entry and isinstance(entry[metric_name], (int, float)):
                values.append(entry[metric_name])
                timestamps.append(entry.get("timestamp", ""))
        
        if len(values) < 2:
            return {"status": "insufficient_data"}
        
        trend_analysis = self.calculate_trend(values, window_size)
        
        return {
            "status": "success",
            "metric": metric_name,
            "data_points": len(values),
            "time_range": {
                "start": timestamps[0] if timestamps else "unknown",
                "end": timestamps[-1] if timestamps else "unknown"
            },
            "trend_analysis": trend_analysis,
            "statistics": {
                "mean": statistics.mean(values),
                "median": statistics.median(values),
                "std_dev": statistics.stdev(values) if len(values) > 1 else 0,
                "min": min(values),
                "max": max(values),
                "latest": values[-1]
            }
        }
    
    def generate_trend_report(self, history: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Generate comprehensive trend analysis report."""
        report = {
            "timestamp": datetime.utcnow().isoformat(),
            "analysis_period": {
                "start": history[0].get("timestamp", "") if history else "",
                "end": history[-1].get("timestamp", "") if history else "",
                "total_entries": len(history)
            },
            "metrics_analysis": {},
            "summary": {
                "improving_metrics": [],
                "degrading_metrics": [],
                "stable_metrics": []
            }
        }
        
        # Analyze key metrics
        key_metrics = [
            "mean_execution_time_us", "total_execution_time_us", "performance_grade",
            "overall_success_rate", "uv78_target_compliance", "cache_speedup"
        ]
        
        for metric in key_metrics:
            analysis = self.analyze_historical_trends(metric, history)
            if analysis.get("status") == "success":
                report["metrics_analysis"][metric] = analysis
                
                trend = analysis["trend_analysis"]["trend"]
                if trend == "improving":
                    report["summary"]["improving_metrics"].append(metric)
                elif trend == "degrading":
                    report["summary"]["degrading_metrics"].append(metric)
                else:
                    report["summary"]["stable_metrics"].append(metric)
        
        return report
    
    def run_comparison_analysis(self) -> bool:
        """Run complete benchmark comparison and trending analysis."""
        print("📊 Starting UV-60 benchmark comparison and trending analysis...")
        
        # Load current benchmark results
        rust_results = self.load_json(f"{self.rust_results_dir}/rust-performance-report.json")
        js_results = self.load_json(f"{self.js_results_dir}/benchmark-report.json")
        
        if not rust_results and not js_results:
            print("❌ No benchmark results found to analyze")
            return False
        
        # Load or initialize historical data
        history_file = f"{self.history_dir}/benchmark-history.json"
        history_data = self.load_json(history_file) or {"entries": []}
        
        # Normalize and add current results to history
        current_entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "git_commit": os.environ.get("GITHUB_SHA", "unknown")[:8],
            "git_branch": os.environ.get("GITHUB_REF_NAME", "unknown")
        }
        
        if rust_results:
            rust_metrics = self.normalize_metrics(rust_results, "rust")
            current_entry.update(rust_metrics)
        
        if js_results:
            js_metrics = self.normalize_metrics(js_results, "javascript")
            current_entry.update(js_metrics)
        
        history_data["entries"].append(current_entry)
        
        # Limit history size
        max_entries = 200
        if len(history_data["entries"]) > max_entries:
            history_data["entries"] = history_data["entries"][-max_entries:]
        
        # Save updated history
        self.save_json(history_file, history_data)
        
        # Generate trend analysis
        trend_report = self.generate_trend_report(history_data["entries"])
        self.save_json(f"{self.output_dir}/trend-analysis.json", trend_report)
        
        # Compare with baseline if available
        if len(history_data["entries"]) >= 2:
            baseline = history_data["entries"][-2]  # Previous entry as baseline
            current = history_data["entries"][-1]
            
            comparison = self.compare_with_baseline(current, baseline)
            self.save_json(f"{self.output_dir}/baseline-comparison.json", comparison)
            
            # Print summary
            print(f"✅ Analysis complete:")
            print(f"   - Historical entries: {len(history_data['entries'])}")
            print(f"   - Trend analysis: {len(trend_report['metrics_analysis'])} metrics")
            print(f"   - Baseline comparison: {comparison['summary']['total_metrics']} metrics compared")
            print(f"   - Regressions detected: {comparison['summary']['regressions']}")
            print(f"   - Improvements found: {comparison['summary']['improvements']}")
        else:
            print(f"✅ Initial analysis complete - baseline will be established with more data")
        
        return True

def main():
    """Main entry point for benchmark comparison."""
    parser = argparse.ArgumentParser(description="UV-60 Benchmark Comparison and Trending")
    parser.add_argument("--window-size", type=int, default=10, help="Trend analysis window size")
    parser.add_argument("--history-limit", type=int, default=200, help="Maximum history entries to keep")
    
    args = parser.parse_args()
    
    comparator = BenchmarkComparator()
    success = comparator.run_comparison_analysis()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()