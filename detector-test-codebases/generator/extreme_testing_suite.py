#!/usr/bin/env python3
"""
Extreme Testing Suite for Uveddi Detector Validation
Comprehensive test runner for validating all detector categories with extreme test cases
"""

import json
import os
import sys
import time
import subprocess
import statistics
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Tuple
import argparse

class ExtremeTestSuite:
    def __init__(self, uveddi_path: str = None):
        self.uveddi_path = uveddi_path or self._find_uveddi_binary()
        self.test_results = {}
        self.performance_metrics = {}
        self.baseline_metrics = {}
        self.start_time = time.time()
        
        # Define detector categories and their expected patterns
        self.detector_categories = {
            'god_object': {
                'description': 'God Object / Large Class Detection',
                'expected_patterns': ['god object', 'large class', 'excessive responsibilities', 'god_object', 'large_class']
            },
            'dead_code': {
                'description': 'Dead Code Detection',
                'expected_patterns': ['dead code', 'unused code', 'unreachable code', 'dead_code', 'unused_code']
            },
            'code_clones': {
                'description': 'Code Clone Detection',
                'expected_patterns': ['code duplication', 'duplicate code', 'similar code', 'code_duplication', 'duplicate_code']
            },
            'tight_coupling': {
                'description': 'Tight Coupling Detection',
                'expected_patterns': ['tight coupling', 'high coupling', 'dependency violation', 'tight_coupling', 'high_coupling']
            },
            'long_methods': {
                'description': 'Long Method Detection',
                'expected_patterns': ['long method', 'excessive method length', 'long_method', 'excessive_method_length']
            },
            'magic_values': {
                'description': 'Magic Values Detection',
                'expected_patterns': ['magic number', 'magic string', 'hardcoded value', 'magic_number', 'magic_string']
            }
        }

    def _find_uveddi_binary(self) -> str:
        """Find the Uveddi binary in the workspace"""
        possible_paths = [
            './target/release/uveddi',
            './target/debug/uveddi',
            'uveddi',
            '../target/release/uveddi',
            '../target/debug/uveddi'
        ]
        
        for path in possible_paths:
            if os.path.exists(path) or subprocess.run(['which', path], capture_output=True).returncode == 0:
                return path
        
        raise FileNotFoundError("Uveddi binary not found. Please build the project first.")

    def get_extreme_test_files(self) -> Dict[str, List[str]]:
        """Get all extreme test files organized by language"""
        extreme_suite_path = Path("../benchmark-codebases/extreme_suite")
        test_files = {}
        
        if extreme_suite_path.exists():
            for lang_dir in extreme_suite_path.iterdir():
                if lang_dir.is_dir():
                    lang_name = lang_dir.name.replace('_extreme', '')
                    test_files[lang_name] = []
                    
                    for test_file in lang_dir.rglob('*'):
                        if test_file.is_file() and not test_file.name.startswith('.'):
                            test_files[lang_name].append(str(test_file))
        
        return test_files

    def run_analysis(self, file_path: str, timeout: int = 60) -> Dict[str, Any]:
        """Run Uveddi analysis on a single file with performance tracking"""
        start_time = time.time()
        
        try:
            cmd = [self.uveddi_path, "analyze", file_path, "--output-format", "json"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            
            analysis_time = time.time() - start_time
            
            if result.returncode == 0 and result.stdout.strip():
                try:
                    # Extract JSON from output
                    output = result.stdout.strip()
                    json_start = output.find('{')
                    if json_start == -1:
                        raise json.JSONDecodeError("No JSON found in output", output, 0)
                    
                    # Find matching closing brace
                    brace_count = 0
                    json_end = json_start
                    
                    for i in range(json_start, len(output)):
                        if output[i] == '{':
                            brace_count += 1
                        elif output[i] == '}':
                            brace_count -= 1
                            if brace_count == 0:
                                json_end = i + 1
                                break
                    
                    json_text = output[json_start:json_end]
                    data = json.loads(json_text)
                    
                    return {
                        'success': True,
                        'issues': data.get('issues', []),
                        'analysis_time': analysis_time,
                        'metadata': data.get('metadata', {}),
                        'summary': data.get('summary', {}),
                        'file_size': os.path.getsize(file_path),
                        'errors': []
                    }
                except json.JSONDecodeError as e:
                    return {
                        'success': False,
                        'issues': [],
                        'analysis_time': analysis_time,
                        'errors': [f'JSON parse error: {str(e)}'],
                        'file_size': os.path.getsize(file_path)
                    }
            else:
                return {
                    'success': False,
                    'issues': [],
                    'analysis_time': analysis_time,
                    'errors': [f'Analysis failed: {result.stderr}'],
                    'file_size': os.path.getsize(file_path)
                }
                
        except subprocess.TimeoutExpired:
            return {
                'success': False,
                'issues': [],
                'analysis_time': timeout,
                'errors': [f'Analysis timeout ({timeout}s)'],
                'file_size': os.path.getsize(file_path)
            }
        except Exception as e:
            return {
                'success': False,
                'issues': [],
                'analysis_time': time.time() - start_time,
                'errors': [f'Unexpected error: {str(e)}'],
                'file_size': os.path.getsize(file_path) if os.path.exists(file_path) else 0
            }

    def validate_detector_categories(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Validate that all detector categories are working correctly"""
        validation_results = {}
        
        for category, config in self.detector_categories.items():
            validation_results[category] = {
                'description': config['description'],
                'detected': False,
                'detection_count': 0,
                'files_with_detections': [],
                'performance_metrics': {
                    'avg_analysis_time': 0,
                    'total_detections': 0
                }
            }
        
        # Analyze results for each category
        for lang, lang_results in results.items():
            for file_path, file_result in lang_results.items():
                if file_result['success']:
                    issues = file_result['issues']
                    
                    for issue in issues:
                        issue_type = issue.get('issue_type', '').lower()
                        anti_pattern_type = issue.get('antiPatternType', '').lower()
                        issue_description = issue.get('description', '').lower()
                        
                        # Check which detector category this issue belongs to
                        for category, config in self.detector_categories.items():
                            for pattern in config['expected_patterns']:
                                if (pattern in issue_type or 
                                    pattern in anti_pattern_type or 
                                    pattern in issue_description):
                                    validation_results[category]['detected'] = True
                                    validation_results[category]['detection_count'] += 1
                                    if file_path not in validation_results[category]['files_with_detections']:
                                        validation_results[category]['files_with_detections'].append(file_path)
                                    break
        
        return validation_results

    def benchmark_performance(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Generate performance benchmarks and baseline metrics"""
        performance_data = {
            'total_files_analyzed': 0,
            'total_analysis_time': 0,
            'average_analysis_time': 0,
            'files_per_second': 0,
            'total_file_size': 0,
            'throughput_mb_per_second': 0,
            'memory_usage': {},
            'by_language': {},
            'by_file_size': {
                'small_files': {'count': 0, 'avg_time': 0, 'size_range': '< 10KB'},
                'medium_files': {'count': 0, 'avg_time': 0, 'size_range': '10KB - 100KB'},
                'large_files': {'count': 0, 'avg_time': 0, 'size_range': '> 100KB'}
            }
        }
        
        all_times = []
        all_sizes = []
        
        for lang, lang_results in results.items():
            lang_stats = {
                'file_count': 0,
                'total_time': 0,
                'avg_time': 0,
                'total_size': 0,
                'successful_analyses': 0,
                'failed_analyses': 0
            }
            
            for file_path, file_result in lang_results.items():
                performance_data['total_files_analyzed'] += 1
                lang_stats['file_count'] += 1
                
                analysis_time = file_result['analysis_time']
                file_size = file_result['file_size']
                
                all_times.append(analysis_time)
                all_sizes.append(file_size)
                
                performance_data['total_analysis_time'] += analysis_time
                performance_data['total_file_size'] += file_size
                
                lang_stats['total_time'] += analysis_time
                lang_stats['total_size'] += file_size
                
                if file_result['success']:
                    lang_stats['successful_analyses'] += 1
                else:
                    lang_stats['failed_analyses'] += 1
                
                # Categorize by file size
                if file_size < 10 * 1024:  # < 10KB
                    performance_data['by_file_size']['small_files']['count'] += 1
                elif file_size < 100 * 1024:  # 10KB - 100KB
                    performance_data['by_file_size']['medium_files']['count'] += 1
                else:  # > 100KB
                    performance_data['by_file_size']['large_files']['count'] += 1
            
            if lang_stats['file_count'] > 0:
                lang_stats['avg_time'] = lang_stats['total_time'] / lang_stats['file_count']
            
            performance_data['by_language'][lang] = lang_stats
        
        # Calculate overall metrics
        if performance_data['total_files_analyzed'] > 0:
            performance_data['average_analysis_time'] = performance_data['total_analysis_time'] / performance_data['total_files_analyzed']
            performance_data['files_per_second'] = performance_data['total_files_analyzed'] / performance_data['total_analysis_time']
        
        if performance_data['total_file_size'] > 0:
            performance_data['throughput_mb_per_second'] = (performance_data['total_file_size'] / (1024 * 1024)) / performance_data['total_analysis_time']
        
        # Calculate file size category averages
        for category in performance_data['by_file_size'].values():
            if category['count'] > 0:
                category_times = [t for t, s in zip(all_times, all_sizes) 
                                if self._get_size_category(s) == category['size_range']]
                if category_times:
                    category['avg_time'] = statistics.mean(category_times)
        
        return performance_data

    def _get_size_category(self, size: int) -> str:
        """Get size category for a file size"""
        if size < 10 * 1024:
            return '< 10KB'
        elif size < 100 * 1024:
            return '10KB - 100KB'
        else:
            return '> 100KB'

    def generate_comprehensive_report(self, results: Dict[str, Any], validation_results: Dict[str, Any], 
                                    performance_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate comprehensive test report"""
        total_runtime = time.time() - self.start_time
        
        report = {
            'test_suite_info': {
                'name': 'Extreme Detector Validation Suite',
                'version': '1.0.0',
                'timestamp': datetime.now().isoformat(),
                'total_runtime_seconds': total_runtime,
                'uveddi_binary': self.uveddi_path
            },
            'test_execution_summary': {
                'total_files_tested': sum(len(lang_results) for lang_results in results.values()),
                'successful_analyses': sum(
                    sum(1 for file_result in lang_results.values() if file_result['success'])
                    for lang_results in results.values()
                ),
                'failed_analyses': sum(
                    sum(1 for file_result in lang_results.values() if not file_result['success'])
                    for lang_results in results.values()
                ),
                'languages_tested': list(results.keys())
            },
            'detector_validation': validation_results,
            'performance_benchmarks': performance_data,
            'detailed_results': results,
            'baseline_metrics': self._generate_baseline_metrics(performance_data, validation_results)
        }
        
        return report

    def _generate_baseline_metrics(self, performance_data: Dict[str, Any], 
                                 validation_results: Dict[str, Any]) -> Dict[str, Any]:
        """Generate baseline metrics for future comparisons"""
        return {
            'performance_baseline': {
                'avg_analysis_time_ms': performance_data['average_analysis_time'] * 1000,
                'throughput_files_per_second': performance_data['files_per_second'],
                'throughput_mb_per_second': performance_data['throughput_mb_per_second'],
                'memory_efficiency_score': self._calculate_memory_efficiency_score(performance_data)
            },
            'detection_baseline': {
                'detector_coverage_percentage': (
                    sum(1 for result in validation_results.values() if result['detected']) / 
                    len(validation_results) * 100
                ),
                'total_detections': sum(result['detection_count'] for result in validation_results.values()),
                'detectors_working': [
                    category for category, result in validation_results.items() 
                    if result['detected']
                ],
                'detectors_not_working': [
                    category for category, result in validation_results.items() 
                    if not result['detected']
                ]
            },
            'quality_metrics': {
                'success_rate_percentage': (
                    performance_data['total_files_analyzed'] - 
                    sum(lang['failed_analyses'] for lang in performance_data['by_language'].values())
                ) / performance_data['total_files_analyzed'] * 100 if performance_data['total_files_analyzed'] > 0 else 0,
                'reliability_score': self._calculate_reliability_score(performance_data, validation_results)
            }
        }

    def _calculate_memory_efficiency_score(self, performance_data: Dict[str, Any]) -> float:
        """Calculate a memory efficiency score (placeholder - would need actual memory measurements)"""
        # This is a simplified score based on throughput
        return min(100.0, performance_data['throughput_mb_per_second'] * 10)

    def _calculate_reliability_score(self, performance_data: Dict[str, Any], 
                                   validation_results: Dict[str, Any]) -> float:
        """Calculate overall reliability score"""
        success_rate = (
            performance_data['total_files_analyzed'] - 
            sum(lang['failed_analyses'] for lang in performance_data['by_language'].values())
        ) / performance_data['total_files_analyzed'] if performance_data['total_files_analyzed'] > 0 else 0
        
        detection_rate = sum(1 for result in validation_results.values() if result['detected']) / len(validation_results)
        
        return (success_rate + detection_rate) / 2 * 100

    def save_report(self, report: Dict[str, Any], output_file: str = None):
        """Save the comprehensive report to file"""
        if output_file is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            output_file = f"extreme_test_report_{timestamp}.json"
        
        with open(output_file, 'w') as f:
            json.dump(report, f, indent=2, default=str)
        
        print(f"✅ Comprehensive report saved to: {output_file}")
        return output_file

    def print_summary(self, report: Dict[str, Any]):
        """Print a summary of the test results"""
        print("\n" + "="*80)
        print("🚀 EXTREME DETECTOR VALIDATION SUITE - RESULTS SUMMARY")
        print("="*80)
        
        summary = report['test_execution_summary']
        print(f"📊 Total Files Tested: {summary['total_files_tested']}")
        print(f"✅ Successful Analyses: {summary['successful_analyses']}")
        print(f"❌ Failed Analyses: {summary['failed_analyses']}")
        print(f"🌍 Languages Tested: {', '.join(summary['languages_tested'])}")
        
        print(f"\n⏱️  Total Runtime: {report['test_suite_info']['total_runtime_seconds']:.2f} seconds")
        
        print("\n" + "-"*50)
        print("🔍 DETECTOR VALIDATION RESULTS")
        print("-"*50)
        
        for category, result in report['detector_validation'].items():
            status = "✅ WORKING" if result['detected'] else "❌ NOT DETECTED"
            print(f"{result['description']}: {status}")
            if result['detected']:
                print(f"   └─ Detections: {result['detection_count']} in {len(result['files_with_detections'])} files")
        
        print("\n" + "-"*50)
        print("⚡ PERFORMANCE BENCHMARKS")
        print("-"*50)
        
        perf = report['performance_benchmarks']
        print(f"Average Analysis Time: {perf['average_analysis_time']:.3f} seconds")
        print(f"Throughput: {perf['files_per_second']:.2f} files/second")
        print(f"Data Throughput: {perf['throughput_mb_per_second']:.2f} MB/second")
        
        print("\n" + "-"*50)
        print("📈 BASELINE METRICS")
        print("-"*50)
        
        baseline = report['baseline_metrics']
        print(f"Detector Coverage: {baseline['detection_baseline']['detector_coverage_percentage']:.1f}%")
        print(f"Success Rate: {baseline['quality_metrics']['success_rate_percentage']:.1f}%")
        print(f"Reliability Score: {baseline['quality_metrics']['reliability_score']:.1f}/100")
        
        print("\n" + "="*80)

    def run_extreme_test_suite(self, output_file: str = None) -> str:
        """Run the complete extreme test suite"""
        print("🚀 Starting Extreme Detector Validation Suite...")
        print(f"📍 Using Uveddi binary: {self.uveddi_path}")
        
        # Get all extreme test files
        test_files = self.get_extreme_test_files()
        if not test_files:
            raise RuntimeError("No extreme test files found. Please ensure the benchmark-codebases/extreme_suite directory exists.")
        
        print(f"📁 Found test files in {len(test_files)} languages: {', '.join(test_files.keys())}")
        
        # Run analysis on all files
        results = {}
        total_files = sum(len(files) for files in test_files.values())
        current_file = 0
        
        for lang, files in test_files.items():
            print(f"\n🔍 Analyzing {lang} files...")
            results[lang] = {}
            
            for file_path in files:
                current_file += 1
                print(f"  [{current_file}/{total_files}] {os.path.basename(file_path)}...", end=" ")
                
                result = self.run_analysis(file_path)
                results[lang][file_path] = result
                
                if result['success']:
                    print(f"✅ ({result['analysis_time']:.2f}s, {len(result['issues'])} issues)")
                else:
                    print(f"❌ ({result['analysis_time']:.2f}s, {len(result['errors'])} errors)")
        
        # Validate detector categories
        print("\n🔍 Validating detector categories...")
        validation_results = self.validate_detector_categories(results)
        
        # Benchmark performance
        print("⚡ Generating performance benchmarks...")
        performance_data = self.benchmark_performance(results)
        
        # Generate comprehensive report
        print("📊 Generating comprehensive report...")
        report = self.generate_comprehensive_report(results, validation_results, performance_data)
        
        # Save report
        report_file = self.save_report(report, output_file)
        
        # Print summary
        self.print_summary(report)
        
        return report_file


def main():
    parser = argparse.ArgumentParser(description='Extreme Detector Validation Suite')
    parser.add_argument('--uveddi-path', help='Path to Uveddi binary')
    parser.add_argument('--output', help='Output file for the report')
    parser.add_argument('--timeout', type=int, default=60, help='Analysis timeout in seconds')
    
    args = parser.parse_args()
    
    try:
        suite = ExtremeTestSuite(args.uveddi_path)
        report_file = suite.run_extreme_test_suite(args.output)
        
        print(f"\n🎉 Extreme test suite completed successfully!")
        print(f"📄 Full report available at: {report_file}")
        
        return 0
        
    except Exception as e:
        print(f"\n💥 Extreme test suite failed: {str(e)}")
        return 1


if __name__ == "__main__":
    sys.exit(main())