#!/usr/bin/env python3
"""
Calculate precision, recall, F1 score from ground truth annotations.

Usage:
    python3 scripts/calculate_metrics.py <ground_truth.json> [--verbose]

Examples:
    python3 scripts/calculate_metrics.py reports/calibration-2025-11-03/tokio-analysis_ground_truth.json
    python3 scripts/calculate_metrics.py reports/calibration-2025-11-03/*_ground_truth.json --verbose
"""

import json
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

# Terminal colors
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
RESET = '\033[0m'


def load_ground_truth(file_path: Path) -> Dict:
    """Load ground truth annotations from file"""
    try:
        with open(file_path) as f:
            return json.load(f)
    except Exception as e:
        print(f"{RED}Error loading {file_path}: {e}{RESET}")
        return {'issues': []}


def calculate_detector_metrics(issues: List[Dict]) -> Dict[str, Dict[str, int]]:
    """Calculate TP, FP counts by detector type"""
    metrics = defaultdict(lambda: {'tp': 0, 'fp': 0, 'total': 0})

    for issue in issues:
        detector = issue.get('issue_type', 'unknown')
        verdict = issue.get('annotation', {}).get('verdict')

        if verdict in ['tp', 'fp']:
            metrics[detector]['total'] += 1
            metrics[detector][verdict] += 1

    return dict(metrics)


def calculate_precision(tp: int, fp: int) -> float:
    """Calculate precision: TP / (TP + FP)"""
    total = tp + fp
    return tp / total if total > 0 else 0.0


def calculate_severity_metrics(issues: List[Dict]) -> Dict[str, Dict[str, int]]:
    """Calculate metrics by severity level"""
    metrics = defaultdict(lambda: {'tp': 0, 'fp': 0, 'total': 0})

    for issue in issues:
        severity = issue.get('severity', 'unknown')
        verdict = issue.get('annotation', {}).get('verdict')

        if verdict in ['tp', 'fp']:
            metrics[severity]['total'] += 1
            metrics[severity][verdict] += 1

    return dict(metrics)


def print_detector_table(metrics: Dict[str, Dict[str, int]]):
    """Print formatted table of detector metrics"""
    print(f"\n{YELLOW}=== Metrics by Detector Type ==={RESET}")
    print(f"{'Detector':<30} {'Precision':>10} {'Issues':>8} {'TP':>6} {'FP':>6}")
    print("=" * 65)

    # Sort by detector name
    sorted_detectors = sorted(metrics.items(), key=lambda x: x[0])

    for detector, counts in sorted_detectors:
        tp = counts['tp']
        fp = counts['fp']
        total = counts['total']
        precision = calculate_precision(tp, fp)

        # Color code based on precision
        if precision >= 0.85:
            color = GREEN
        elif precision >= 0.70:
            color = YELLOW
        else:
            color = RED

        print(f"{detector:<30} {color}{precision:>9.1%}{RESET} {total:>8} {tp:>6} {fp:>6}")


def print_severity_table(metrics: Dict[str, Dict[str, int]]):
    """Print formatted table of severity metrics"""
    print(f"\n{YELLOW}=== Metrics by Severity Level ==={RESET}")
    print(f"{'Severity':<15} {'Precision':>10} {'Issues':>8} {'TP':>6} {'FP':>6}")
    print("=" * 50)

    # Sort by severity (Critical, High, Medium, Low, Info)
    severity_order = ['Critical', 'High', 'Medium', 'Low', 'Info', 'unknown']
    sorted_severities = sorted(
        metrics.items(),
        key=lambda x: severity_order.index(x[0]) if x[0] in severity_order else 99
    )

    for severity, counts in sorted_severities:
        tp = counts['tp']
        fp = counts['fp']
        total = counts['total']
        precision = calculate_precision(tp, fp)

        # Color code based on precision
        if precision >= 0.85:
            color = GREEN
        elif precision >= 0.70:
            color = YELLOW
        else:
            color = RED

        print(f"{severity:<15} {color}{precision:>9.1%}{RESET} {total:>8} {tp:>6} {fp:>6}")


def print_overall_summary(all_tp: int, all_fp: int):
    """Print overall metrics summary"""
    total = all_tp + all_fp
    precision = calculate_precision(all_tp, all_fp)
    fp_rate = (all_fp / total * 100) if total > 0 else 0.0

    print(f"\n{BLUE}{'=' * 65}{RESET}")
    print(f"{YELLOW}=== Overall Summary ==={RESET}")
    print(f"{BLUE}{'=' * 65}{RESET}")

    # Color code based on precision
    if precision >= 0.85:
        color = GREEN
        status = "✓ Target Met"
    elif precision >= 0.70:
        color = YELLOW
        status = "⚠ Approaching Target"
    else:
        color = RED
        status = "✗ Below Target"

    print(f"\nTotal Issues Annotated: {total}")
    print(f"True Positives: {GREEN}{all_tp}{RESET}")
    print(f"False Positives: {RED}{all_fp}{RESET}")
    print(f"\n{color}Precision: {precision:.1%} {status}{RESET}")
    print(f"False Positive Rate: {fp_rate:.1f}%")

    # Target comparison
    print(f"\n{YELLOW}Target Metrics:{RESET}")
    print(f"  Precision: ≥85%")
    print(f"  False Positive Rate: <15%")


def print_false_positives(issues: List[Dict], limit: int = 10):
    """Print details of false positive issues"""
    fps = [i for i in issues if i.get('annotation', {}).get('verdict') == 'fp']

    if not fps:
        return

    print(f"\n{YELLOW}=== False Positive Examples (first {min(limit, len(fps))}) ==={RESET}")

    for i, issue in enumerate(fps[:limit], 1):
        print(f"\n{i}. {issue.get('issue_type', 'unknown')} in {issue.get('file_path', 'unknown')}")
        print(f"   Line: {issue.get('line_number', 'unknown')}")
        print(f"   Message: {issue.get('message', 'No message')[:100]}...")

        notes = issue.get('annotation', {}).get('notes', '')
        if notes:
            print(f"   Notes: {notes}")


def main():
    if len(sys.argv) < 2:
        print(f"{RED}Usage: calculate_metrics.py <ground_truth.json> [--verbose]{RESET}")
        sys.exit(1)

    verbose = '--verbose' in sys.argv or '-v' in sys.argv
    file_paths = [arg for arg in sys.argv[1:] if not arg.startswith('--')]

    # Support glob patterns
    all_files = []
    for pattern in file_paths:
        path = Path(pattern)
        if '*' in pattern:
            # Glob pattern
            parent = path.parent
            pattern_name = path.name
            all_files.extend(parent.glob(pattern_name))
        else:
            all_files.append(path)

    if not all_files:
        print(f"{RED}No ground truth files found{RESET}")
        sys.exit(1)

    print(f"{GREEN}Analyzing {len(all_files)} ground truth file(s)...{RESET}\n")

    # Aggregate metrics across all files
    all_issues = []
    for file_path in all_files:
        if file_path.exists():
            print(f"Loading: {file_path}")
            data = load_ground_truth(file_path)
            all_issues.extend(data.get('issues', []))

    if not all_issues:
        print(f"{RED}No annotated issues found{RESET}")
        sys.exit(0)

    # Calculate metrics
    detector_metrics = calculate_detector_metrics(all_issues)
    severity_metrics = calculate_severity_metrics(all_issues)

    # Calculate overall stats
    all_tp = sum(m['tp'] for m in detector_metrics.values())
    all_fp = sum(m['fp'] for m in detector_metrics.values())

    # Print results
    print_detector_table(detector_metrics)
    print_severity_table(severity_metrics)
    print_overall_summary(all_tp, all_fp)

    if verbose:
        print_false_positives(all_issues)

    # Exit with appropriate code
    total = all_tp + all_fp
    precision = calculate_precision(all_tp, all_fp)

    if precision >= 0.85:
        print(f"\n{GREEN}✓ Calibration target achieved!{RESET}")
        sys.exit(0)
    elif precision >= 0.70:
        print(f"\n{YELLOW}⚠ Approaching calibration target{RESET}")
        sys.exit(0)
    else:
        print(f"\n{RED}✗ Further calibration needed{RESET}")
        sys.exit(1)


if __name__ == '__main__':
    main()
