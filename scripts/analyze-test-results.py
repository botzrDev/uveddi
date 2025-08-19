#!/usr/bin/env python3
"""
Advanced Test Result Analysis for Uveddi v1.0 Community Core
Analyzes automated testing results and generates insights for v1.0 release readiness
"""

import json
import os
import sys
import argparse
from pathlib import Path
from datetime import datetime
from collections import defaultdict, Counter
import statistics
import yaml

class TestResultAnalyzer:
    def __init__(self, results_dir: str):
        self.results_dir = Path(results_dir)
        self.metrics_dir = self.results_dir / "metrics"
        self.reports_dir = self.results_dir / "reports"
        self.logs_dir = self.results_dir / "logs"
        
        self.load_test_expectations()
        self.analysis_data = {
            'repositories': [],
            'summary': {},
            'language_breakdown': defaultdict(list),
            'pattern_detection': defaultdict(int),
            'performance_metrics': [],
            'failure_analysis': [],
            'recommendations': []
        }
    
    def load_test_expectations(self):
        """Load expected patterns and thresholds from configuration"""
        config_file = self.results_dir.parent / "test_repositories.yaml"
        if config_file.exists():
            with open(config_file, 'r') as f:
                config = yaml.safe_load(f)
                self.expectations = config.get('detection_expectations', {})
        else:
            # Default expectations
            self.expectations = {
                'minimum_success_rate': 0.75,
                'minimum_issues_per_repo': 3,
                'maximum_analysis_time': 60,
                'by_language': {
                    'python': {'expected_success_rate': 0.85, 'expected_avg_issues': 8},
                    'javascript': {'expected_success_rate': 0.80, 'expected_avg_issues': 6},
                    'typescript': {'expected_success_rate': 0.80, 'expected_avg_issues': 7},
                    'rust': {'expected_success_rate': 0.90, 'expected_avg_issues': 4}
                }
            }
    
    def analyze_repository_results(self):
        """Analyze results for each repository"""
        print("🔍 Analyzing repository results...")
        
        if not self.metrics_dir.exists():
            print(f"❌ Metrics directory not found: {self.metrics_dir}")
            return
        
        for metrics_file in self.metrics_dir.glob("*.json"):
            try:
                with open(metrics_file, 'r') as f:
                    metrics = json.load(f)
                
                repo_data = self.process_repository_metrics(metrics)
                self.analysis_data['repositories'].append(repo_data)
                
                # Categorize by language
                language = metrics.get('language', 'unknown')
                self.analysis_data['language_breakdown'][language].append(repo_data)
                
            except Exception as e:
                print(f"⚠️ Error processing {metrics_file}: {e}")
    
    def process_repository_metrics(self, metrics: dict) -> dict:
        """Process individual repository metrics"""
        repo_name = metrics.get('repository', 'unknown')
        language = metrics.get('language', 'unknown')
        analysis = metrics.get('analysis', {})
        results = metrics.get('results', {})
        codebase = metrics.get('codebase', {})
        
        repo_data = {
            'name': repo_name,
            'language': language,
            'status': analysis.get('status', 'unknown'),
            'duration': analysis.get('duration_seconds', 0),
            'issues_found': results.get('issues_found', 0),
            'detectors_run': results.get('detectors_run', 0),
            'file_count': codebase.get('file_count', 0),
            'timestamp': analysis.get('timestamp', ''),
            'patterns_detected': []
        }
        
        # Load detailed analysis if available
        report_file = self.reports_dir / f"{repo_name}_analysis.json"
        if report_file.exists():
            try:
                with open(report_file, 'r') as f:
                    detailed_analysis = json.load(f)
                    repo_data['patterns_detected'] = self.extract_detected_patterns(detailed_analysis)
            except Exception as e:
                print(f"⚠️ Could not load detailed analysis for {repo_name}: {e}")
        
        # Performance assessment
        repo_data['performance_rating'] = self.assess_performance(repo_data)
        
        return repo_data
    
    def extract_detected_patterns(self, analysis: dict) -> list:
        """Extract detected anti-patterns from detailed analysis"""
        patterns = []
        
        issues = analysis.get('issues', [])
        for issue in issues:
            pattern_type = issue.get('detector', '').lower()
            if pattern_type:
                patterns.append(pattern_type)
                self.analysis_data['pattern_detection'][pattern_type] += 1
        
        return list(set(patterns))  # Remove duplicates
    
    def assess_performance(self, repo_data: dict) -> str:
        """Assess repository analysis performance"""
        duration = repo_data['duration']
        issues = repo_data['issues_found']
        status = repo_data['status']
        
        if status != 'success':
            return 'failed'
        elif duration > 90:
            return 'slow'
        elif duration > 60:
            return 'acceptable'
        elif issues >= 5:
            return 'excellent'
        elif issues >= 3:
            return 'good'
        else:
            return 'limited'
    
    def generate_summary_statistics(self):
        """Generate overall summary statistics"""
        print("📊 Generating summary statistics...")
        
        repositories = self.analysis_data['repositories']
        if not repositories:
            print("❌ No repository data found")
            return
        
        total_repos = len(repositories)
        successful_repos = len([r for r in repositories if r['status'] == 'success'])
        
        # Basic statistics
        success_rate = successful_repos / total_repos if total_repos > 0 else 0
        
        # Performance statistics
        successful_analyses = [r for r in repositories if r['status'] == 'success']
        if successful_analyses:
            durations = [r['duration'] for r in successful_analyses]
            issues_found = [r['issues_found'] for r in successful_analyses]
            
            avg_duration = statistics.mean(durations)
            avg_issues = statistics.mean(issues_found)
            median_duration = statistics.median(durations)
            median_issues = statistics.median(issues_found)
        else:
            avg_duration = avg_issues = median_duration = median_issues = 0
        
        self.analysis_data['summary'] = {
            'total_repositories': total_repos,
            'successful_analyses': successful_repos,
            'failed_analyses': total_repos - successful_repos,
            'success_rate': success_rate,
            'average_duration': avg_duration,
            'median_duration': median_duration,
            'average_issues_found': avg_issues,
            'median_issues_found': median_issues,
            'total_issues_found': sum(r['issues_found'] for r in repositories),
            'total_files_analyzed': sum(r['file_count'] for r in repositories)
        }
    
    def analyze_language_performance(self):
        """Analyze performance by language"""
        print("🗣️ Analyzing language-specific performance...")
        
        for language, repos in self.analysis_data['language_breakdown'].items():
            total = len(repos)
            successful = len([r for r in repos if r['status'] == 'success'])
            
            language_stats = {
                'total_repositories': total,
                'successful_analyses': successful,
                'success_rate': successful / total if total > 0 else 0,
                'performance_ratings': Counter(r['performance_rating'] for r in repos),
                'common_patterns': []
            }
            
            # Calculate averages for successful analyses
            successful_repos = [r for r in repos if r['status'] == 'success']
            if successful_repos:
                language_stats.update({
                    'average_duration': statistics.mean(r['duration'] for r in successful_repos),
                    'average_issues': statistics.mean(r['issues_found'] for r in successful_repos),
                    'total_issues': sum(r['issues_found'] for r in successful_repos)
                })
                
                # Find most common patterns
                all_patterns = []
                for repo in successful_repos:
                    all_patterns.extend(repo['patterns_detected'])
                language_stats['common_patterns'] = Counter(all_patterns).most_common(5)
            
            # Compare against expectations
            expected = self.expectations.get('by_language', {}).get(language, {})
            language_stats['meets_expectations'] = self.evaluate_language_expectations(
                language_stats, expected
            )
            
            self.analysis_data['language_breakdown'][language] = language_stats
    
    def evaluate_language_expectations(self, stats: dict, expected: dict) -> dict:
        """Evaluate if language performance meets expectations"""
        evaluation = {}
        
        if 'expected_success_rate' in expected:
            actual_rate = stats.get('success_rate', 0)
            expected_rate = expected['expected_success_rate']
            evaluation['success_rate'] = {
                'actual': actual_rate,
                'expected': expected_rate,
                'meets_expectation': actual_rate >= expected_rate,
                'difference': actual_rate - expected_rate
            }
        
        if 'expected_avg_issues' in expected:
            actual_issues = stats.get('average_issues', 0)
            expected_issues = expected['expected_avg_issues']
            evaluation['average_issues'] = {
                'actual': actual_issues,
                'expected': expected_issues,
                'meets_expectation': actual_issues >= expected_issues * 0.8,  # 80% threshold
                'difference': actual_issues - expected_issues
            }
        
        return evaluation
    
    def identify_failure_patterns(self):
        """Identify common failure patterns"""
        print("🔍 Identifying failure patterns...")
        
        failed_repos = [r for r in self.analysis_data['repositories'] if r['status'] != 'success']
        
        if not failed_repos:
            print("✅ No failures to analyze")
            return
        
        # Analyze failure patterns
        failure_by_language = Counter(r['language'] for r in failed_repos)
        failure_reasons = []
        
        # Check log files for failure reasons
        for repo in failed_repos:
            log_file = self.logs_dir / f"{repo['name']}_analysis.log"
            if log_file.exists():
                try:
                    with open(log_file, 'r') as f:
                        log_content = f.read()
                        failure_reason = self.categorize_failure(log_content)
                        failure_reasons.append(failure_reason)
                except Exception as e:
                    print(f"⚠️ Could not read log for {repo['name']}: {e}")
        
        self.analysis_data['failure_analysis'] = {
            'total_failures': len(failed_repos),
            'failures_by_language': dict(failure_by_language),
            'failure_reasons': Counter(failure_reasons),
            'failed_repositories': [r['name'] for r in failed_repos]
        }
    
    def categorize_failure(self, log_content: str) -> str:
        """Categorize failure reason from log content"""
        log_lower = log_content.lower()
        
        if 'timeout' in log_lower:
            return 'timeout'
        elif 'memory' in log_lower or 'oom' in log_lower:
            return 'memory_issue'
        elif 'parse' in log_lower or 'syntax' in log_lower:
            return 'parsing_error'
        elif 'permission' in log_lower or 'access' in log_lower:
            return 'permission_error'
        elif 'network' in log_lower or 'connection' in log_lower:
            return 'network_error'
        elif 'panic' in log_lower or 'crash' in log_lower:
            return 'application_crash'
        else:
            return 'unknown_error'
    
    def generate_recommendations(self):
        """Generate recommendations based on analysis"""
        print("💡 Generating recommendations...")
        
        summary = self.analysis_data['summary']
        recommendations = []
        
        # Overall success rate
        success_rate = summary.get('success_rate', 0)
        if success_rate < 0.60:
            recommendations.append({
                'priority': 'HIGH',
                'category': 'Stability',
                'issue': f'Low success rate ({success_rate:.1%})',
                'recommendation': 'Investigate and fix common failure patterns before v1.0 release'
            })
        elif success_rate < 0.75:
            recommendations.append({
                'priority': 'MEDIUM',
                'category': 'Stability',
                'issue': f'Below target success rate ({success_rate:.1%})',
                'recommendation': 'Improve error handling and edge case coverage'
            })
        
        # Performance issues
        avg_duration = summary.get('average_duration', 0)
        if avg_duration > 90:
            recommendations.append({
                'priority': 'MEDIUM',
                'category': 'Performance',
                'issue': f'Slow analysis speed ({avg_duration:.1f}s average)',
                'recommendation': 'Optimize analysis algorithms and consider parallelization'
            })
        
        # Issue detection
        avg_issues = summary.get('average_issues_found', 0)
        if avg_issues < 3:
            recommendations.append({
                'priority': 'MEDIUM',
                'category': 'Detection',
                'issue': f'Low issue detection rate ({avg_issues:.1f} issues/repo)',
                'recommendation': 'Tune detector sensitivity and expand pattern coverage'
            })
        
        # Language-specific recommendations
        for language, stats in self.analysis_data['language_breakdown'].items():
            if isinstance(stats, dict) and 'meets_expectations' in stats:
                expectations = stats['meets_expectations']
                
                for metric, evaluation in expectations.items():
                    if not evaluation.get('meets_expectation', True):
                        recommendations.append({
                            'priority': 'MEDIUM',
                            'category': f'{language.title()} Support',
                            'issue': f'{metric} below expectations for {language}',
                            'recommendation': f'Improve {language} analysis capabilities'
                        })
        
        # Failure pattern recommendations
        failure_analysis = self.analysis_data.get('failure_analysis', {})
        common_failures = failure_analysis.get('failure_reasons', {})
        
        for failure_type, count in common_failures.most_common(3):
            if count > 1:  # Only if multiple occurrences
                recommendations.append({
                    'priority': 'HIGH' if count > 3 else 'MEDIUM',
                    'category': 'Reliability',
                    'issue': f'Multiple {failure_type} failures ({count} occurrences)',
                    'recommendation': f'Address {failure_type} handling in codebase'
                })
        
        self.analysis_data['recommendations'] = recommendations
    
    def generate_release_readiness_assessment(self) -> dict:
        """Generate overall v1.0 release readiness assessment"""
        print("🎯 Assessing v1.0 release readiness...")
        
        summary = self.analysis_data['summary']
        success_rate = summary.get('success_rate', 0)
        avg_issues = summary.get('average_issues_found', 0)
        high_priority_issues = len([r for r in self.analysis_data['recommendations'] 
                                   if r['priority'] == 'HIGH'])
        
        # Scoring system
        scores = {
            'stability': min(100, success_rate * 100),
            'detection': min(100, (avg_issues / 5) * 100),  # Assume 5 issues is excellent
            'reliability': max(0, 100 - (high_priority_issues * 20))  # Each high issue -20 points
        }
        
        overall_score = sum(scores.values()) / len(scores)
        
        # Readiness determination
        if overall_score >= 80:
            readiness = 'READY'
            message = 'v1.0 community core ready - good stability and detection rates'
        elif overall_score >= 65:
            readiness = 'READY_WITH_CAVEATS'
            message = 'v1.0 community core possible with known limitations'
        elif overall_score >= 50:
            readiness = 'NEEDS_IMPROVEMENT'
            message = 'Significant issues need addressing before v1.0 release'
        else:
            readiness = 'NOT_READY'
            message = 'Major stability or functionality issues require resolution'
        
        return {
            'readiness': readiness,
            'overall_score': overall_score,
            'scores': scores,
            'message': message,
            'assessment_date': datetime.now().isoformat()
        }
    
    def export_results(self, output_file: str):
        """Export analysis results to JSON"""
        print(f"💾 Exporting results to {output_file}...")
        
        # Add readiness assessment
        self.analysis_data['release_readiness'] = self.generate_release_readiness_assessment()
        
        # Add metadata
        self.analysis_data['metadata'] = {
            'analysis_date': datetime.now().isoformat(),
            'uveddi_version': '1.0.0-community',
            'analyzer_version': '1.0.0'
        }
        
        with open(output_file, 'w') as f:
            json.dump(self.analysis_data, f, indent=2, default=str)
    
    def generate_markdown_report(self, output_file: str):
        """Generate comprehensive markdown report"""
        print(f"📄 Generating markdown report: {output_file}...")
        
        summary = self.analysis_data['summary']
        readiness = self.analysis_data.get('release_readiness', {})
        
        with open(output_file, 'w') as f:
            f.write("# Uveddi v1.0 Community Core Analysis Report\n\n")
            f.write(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
            
            # Executive Summary
            f.write("## Executive Summary\n\n")
            f.write(f"- **Release Readiness**: {readiness.get('readiness', 'UNKNOWN')} ({readiness.get('overall_score', 0):.1f}/100)\n")
            f.write(f"- **Success Rate**: {summary.get('success_rate', 0):.1%}\n")
            f.write(f"- **Repositories Tested**: {summary.get('total_repositories', 0)}\n")
            f.write(f"- **Total Issues Found**: {summary.get('total_issues_found', 0)}\n")
            f.write(f"- **Average Analysis Time**: {summary.get('average_duration', 0):.1f}s\n\n")
            
            f.write(f"**Assessment**: {readiness.get('message', 'No assessment available')}\n\n")
            
            # Detailed Statistics
            f.write("## Detailed Statistics\n\n")
            f.write("| Metric | Value |\n")
            f.write("|--------|-------|\n")
            for key, value in summary.items():
                if isinstance(value, float):
                    if 'rate' in key:
                        f.write(f"| {key.replace('_', ' ').title()} | {value:.1%} |\n")
                    else:
                        f.write(f"| {key.replace('_', ' ').title()} | {value:.1f} |\n")
                else:
                    f.write(f"| {key.replace('_', ' ').title()} | {value} |\n")
            
            # Language Breakdown
            f.write("\n## Language Performance\n\n")
            for language, stats in self.analysis_data['language_breakdown'].items():
                if isinstance(stats, dict):
                    f.write(f"### {language.title()}\n\n")
                    f.write(f"- **Success Rate**: {stats.get('success_rate', 0):.1%}\n")
                    f.write(f"- **Average Issues**: {stats.get('average_issues', 0):.1f}\n")
                    f.write(f"- **Average Duration**: {stats.get('average_duration', 0):.1f}s\n")
                    
                    common_patterns = stats.get('common_patterns', [])
                    if common_patterns:
                        f.write(f"- **Common Patterns**: {', '.join(p[0] for p in common_patterns[:3])}\n")
                    f.write("\n")
            
            # Most Detected Patterns
            f.write("## Most Detected Anti-Patterns\n\n")
            f.write("| Pattern | Detections |\n")
            f.write("|---------|------------|\n")
            for pattern, count in Counter(self.analysis_data['pattern_detection']).most_common(10):
                f.write(f"| {pattern.replace('_', ' ').title()} | {count} |\n")
            
            # Recommendations
            f.write("\n## Recommendations\n\n")
            recommendations = self.analysis_data.get('recommendations', [])
            
            for priority in ['HIGH', 'MEDIUM', 'LOW']:
                priority_recs = [r for r in recommendations if r['priority'] == priority]
                if priority_recs:
                    f.write(f"### {priority} Priority\n\n")
                    for rec in priority_recs:
                        f.write(f"**{rec['category']}**: {rec['issue']}\n")
                        f.write(f"- *Recommendation*: {rec['recommendation']}\n\n")
            
            # Failure Analysis
            failure_analysis = self.analysis_data.get('failure_analysis', {})
            if failure_analysis.get('total_failures', 0) > 0:
                f.write("## Failure Analysis\n\n")
                f.write(f"**Total Failures**: {failure_analysis['total_failures']}\n\n")
                
                f.write("### Failure Reasons\n\n")
                for reason, count in failure_analysis.get('failure_reasons', {}).items():
                    f.write(f"- **{reason.replace('_', ' ').title()}**: {count} occurrences\n")
                f.write("\n")
                
                f.write("### Failed Repositories\n\n")
                for repo in failure_analysis.get('failed_repositories', []):
                    f.write(f"- {repo}\n")
            
            f.write("\n---\n")
            f.write("*Generated by Uveddi Alpha Testing Analysis Framework*\n")
    
    def run_analysis(self):
        """Run complete analysis"""
        print("🚀 Starting comprehensive test result analysis...")
        
        self.analyze_repository_results()
        self.generate_summary_statistics()
        self.analyze_language_performance()
        self.identify_failure_patterns()
        self.generate_recommendations()
        
        print("✅ Analysis complete!")

def main():
    parser = argparse.ArgumentParser(description='Analyze Uveddi automated test results')
    parser.add_argument('results_dir', help='Directory containing test results')
    parser.add_argument('--output', '-o', default='analysis_results.json', 
                       help='Output JSON file for analysis results')
    parser.add_argument('--report', '-r', default='analysis_report.md',
                       help='Output markdown report file')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Verbose output')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.results_dir):
        print(f"❌ Results directory not found: {args.results_dir}")
        sys.exit(1)
    
    analyzer = TestResultAnalyzer(args.results_dir)
    analyzer.run_analysis()
    
    # Export results
    analyzer.export_results(args.output)
    analyzer.generate_markdown_report(args.report)
    
    # Print readiness assessment
    readiness = analyzer.analysis_data.get('alpha_readiness', {})
    print(f"\n🎯 Alpha Readiness: {readiness.get('readiness', 'UNKNOWN')}")
    print(f"📊 Overall Score: {readiness.get('overall_score', 0):.1f}/100")
    print(f"📝 Assessment: {readiness.get('message', 'No assessment available')}")
    
    print(f"\n📋 Results exported to: {args.output}")
    print(f"📄 Report generated: {args.report}")

if __name__ == "__main__":
    main()