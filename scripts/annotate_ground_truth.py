#!/usr/bin/env python3
"""
Interactive tool for creating ground truth annotations.
Allows developers to mark true/false positives for detector calibration.

Usage:
    python3 scripts/annotate_ground_truth.py <analysis_results.json>

Examples:
    python3 scripts/annotate_ground_truth.py reports/calibration-2025-11-03/tokio-analysis.json
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Optional

# Terminal colors
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
RESET = '\033[0m'


def load_analysis_results(results_file: Path) -> Dict:
    """Load analysis results from JSON file"""
    try:
        with open(results_file) as f:
            return json.load(f)
    except Exception as e:
        print(f"{RED}Error loading file: {e}{RESET}")
        sys.exit(1)


def print_issue_details(issue: Dict, index: int, total: int):
    """Pretty-print issue details for review"""
    print(f"\n{BLUE}{'='*80}{RESET}")
    print(f"{YELLOW}Issue {index}/{total}{RESET}")
    print(f"{BLUE}{'='*80}{RESET}")

    file_path = issue.get('file_path', 'unknown')
    line_num = issue.get('line_number', 'unknown')
    issue_type = issue.get('issue_type', 'unknown')
    severity = issue.get('severity', 'unknown')
    message = issue.get('message', 'No message')

    print(f"{GREEN}File:{RESET} {file_path}")
    print(f"{GREEN}Line:{RESET} {line_num}")
    print(f"{GREEN}Type:{RESET} {issue_type}")
    print(f"{GREEN}Severity:{RESET} {severity}")
    print(f"\n{YELLOW}Message:{RESET}")
    print(f"  {message}")

    # Print code snippet if available
    if 'code_snippet' in issue:
        print(f"\n{YELLOW}Code Snippet:{RESET}")
        snippet = issue['code_snippet']
        if isinstance(snippet, str):
            for line in snippet.split('\n')[:10]:  # Limit to 10 lines
                print(f"  {line}")

    print(f"{BLUE}{'='*80}{RESET}")


def get_annotation_input() -> Optional[str]:
    """Prompt user for annotation verdict"""
    while True:
        print(f"\n{YELLOW}Commands:{RESET}")
        print("  tp  - True Positive (this is a real issue)")
        print("  fp  - False Positive (not an issue, incorrect detection)")
        print("  s   - Skip (uncertain, come back later)")
        print("  q   - Quit and save")
        print("  h   - Help")

        verdict = input(f"\n{BLUE}Verdict:{RESET} ").lower().strip()

        if verdict in ['tp', 'fp', 's', 'q', 'h']:
            return verdict

        print(f"{RED}Invalid input. Please enter: tp, fp, s, q, or h{RESET}")


def annotate_issue(issue: Dict, index: int, total: int) -> Optional[Dict]:
    """Annotate a single issue"""
    print_issue_details(issue, index, total)

    while True:
        verdict = get_annotation_input()

        if verdict == 'h':
            print(f"\n{GREEN}=== Help ==={RESET}")
            print("True Positive (tp): The detector correctly identified a real issue")
            print("False Positive (fp): The detector incorrectly flagged something as an issue")
            print("Skip (s): You're not sure - skip this issue for now")
            print("Quit (q): Save progress and exit")
            continue

        if verdict == 'q':
            return None

        if verdict == 's':
            return issue  # Return without annotation

        # For tp/fp, get optional notes
        notes = input(f"{BLUE}Notes (optional):{RESET} ").strip()

        issue['annotation'] = {
            'verdict': verdict,
            'notes': notes
        }

        return issue


def save_ground_truth(output_file: Path, data: Dict):
    """Save ground truth annotations to file"""
    try:
        with open(output_file, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"\n{GREEN}✓ Ground truth saved to:{RESET} {output_file}")
    except Exception as e:
        print(f"\n{RED}Error saving file: {e}{RESET}")
        sys.exit(1)


def print_summary(annotated: List[Dict]):
    """Print summary of annotations"""
    tp_count = sum(1 for i in annotated if i.get('annotation', {}).get('verdict') == 'tp')
    fp_count = sum(1 for i in annotated if i.get('annotation', {}).get('verdict') == 'fp')

    print(f"\n{GREEN}=== Annotation Summary ==={RESET}")
    print(f"Total annotated: {len(annotated)}")
    print(f"True Positives: {tp_count}")
    print(f"False Positives: {fp_count}")

    if len(annotated) > 0:
        precision = tp_count / len(annotated) * 100
        print(f"\nPrecision (preliminary): {precision:.1f}%")


def main():
    if len(sys.argv) < 2:
        print(f"{RED}Usage: annotate_ground_truth.py <analysis_results.json>{RESET}")
        sys.exit(1)

    results_file = Path(sys.argv[1])

    if not results_file.exists():
        print(f"{RED}File not found: {results_file}{RESET}")
        sys.exit(1)

    print(f"{GREEN}Loading analysis results from: {results_file}{RESET}")
    results = load_analysis_results(results_file)

    issues = results.get('issues', [])
    print(f"\n{YELLOW}Found {len(issues)} issues to annotate{RESET}")

    if len(issues) == 0:
        print(f"{RED}No issues found in analysis results{RESET}")
        sys.exit(0)

    # Check if ground truth file already exists
    output_file = results_file.parent / f"{results_file.stem}_ground_truth.json"
    existing_annotations = {}

    if output_file.exists():
        print(f"\n{YELLOW}Found existing ground truth file{RESET}")
        response = input("Continue with existing annotations? (y/n): ").lower().strip()
        if response == 'y':
            with open(output_file) as f:
                existing_data = json.load(f)
                existing_issues = existing_data.get('issues', [])
                # Create a map of file_path:line_number to annotation
                for issue in existing_issues:
                    if 'annotation' in issue:
                        key = f"{issue.get('file_path')}:{issue.get('line_number')}"
                        existing_annotations[key] = issue['annotation']

    # Annotate issues
    annotated = []
    for i, issue in enumerate(issues, 1):
        # Check if already annotated
        key = f"{issue.get('file_path')}:{issue.get('line_number')}"
        if key in existing_annotations:
            issue['annotation'] = existing_annotations[key]
            annotated.append(issue)
            continue

        annotated_issue = annotate_issue(issue, i, len(issues))

        if annotated_issue is None:
            # User quit
            break

        if 'annotation' in annotated_issue:
            annotated.append(annotated_issue)

        # Auto-save every 10 annotations
        if len(annotated) % 10 == 0:
            ground_truth_data = {
                'source_file': str(results_file),
                'timestamp': str(Path(__file__).stat().st_mtime),
                'issues': annotated
            }
            save_ground_truth(output_file, ground_truth_data)
            print(f"{GREEN}Auto-saved progress{RESET}")

    # Final save
    ground_truth_data = {
        'source_file': str(results_file),
        'issues': annotated
    }
    save_ground_truth(output_file, ground_truth_data)

    print_summary(annotated)

    print(f"\n{YELLOW}Next step:{RESET}")
    print(f"  python3 scripts/calculate_metrics.py {output_file}")


if __name__ == '__main__':
    main()
