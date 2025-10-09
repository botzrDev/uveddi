# Detector Calibration Setup Guide

**Created:** 2025-10-09
**Owner:** Austin
**Purpose:** Prepare for November 2025 detector calibration sprint (Risk R1 mitigation)

## Overview

This document outlines the test repositories, tooling, and methodology for the detector calibration sprint scheduled for 2025-11-03 → 2025-11-08. The goal is to validate and improve detector precision to meet the target of <85% false positive rate.

## Test Corpus

### Existing Test Repositories

Located in `test-codebases/`:

1. **rustlings** (Rust)
   - Small educational codebase
   - Good for Rust-specific detector testing
   - Expected patterns: simple functions, learning exercises

2. **express** (JavaScript)
   - Medium-sized web framework
   - Tests: god object detection, coupling analysis
   - Expected patterns: middleware, routing logic

3. **react** (JavaScript/JSX)
   - Large frontend library
   - Tests: long methods, complexity analysis
   - Expected patterns: component patterns, hooks

### Required Additional Repositories

Based on `archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md`, we need:

#### Rust Repositories
- [ ] **tokio** - async runtime (test concurrency patterns)
- [ ] **actix-web** - web framework (test API patterns)
- [ ] **rustc** or **rust-analyzer** subset - compiler code (test complex algorithms)

#### Python Repositories
- [ ] **django** - web framework (test MVC patterns)
- [ ] **flask** - micro-framework (test simplicity vs. complexity)
- [ ] **pandas** - data library (test performance-critical code)

#### JavaScript/TypeScript Repositories
- [ ] **vue** - frontend framework (test component patterns)
- [ ] Additional **express** apps - real-world usage examples

### Corpus Organization

```
test-codebases/
├── small/           # <1K LOC
│   ├── rust/
│   ├── python/
│   └── javascript/
├── medium/          # 1K-10K LOC
│   ├── rust/
│   ├── python/
│   └── javascript/
└── large/           # >10K LOC
    ├── rust/
    ├── python/
    └── javascript/
```

## Calibration Tooling

### 1. Automated Analysis Script

Create `scripts/calibrate_detectors.sh`:

```bash
#!/bin/bash
# Run comprehensive analysis on test corpus and collect metrics

set -e

CORPUS_DIR="${1:-test-codebases}"
OUTPUT_DIR="reports/calibration-$(date +%Y-%m-%d)"
mkdir -p "$OUTPUT_DIR"

# Function to analyze a codebase
analyze_repo() {
    local repo_path=$1
    local repo_name=$(basename "$repo_path")
    local output_file="$OUTPUT_DIR/${repo_name}-analysis.json"

    echo "Analyzing $repo_name..."
    cargo run --release -- analyze "$repo_path" \
        --output-format json \
        --output "$output_file" \
        --verbose

    echo "✓ Completed: $repo_name"
}

# Analyze all repos in corpus
for size in small medium large; do
    for lang in rust python javascript; do
        corpus_path="$CORPUS_DIR/$size/$lang"
        if [ -d "$corpus_path" ]; then
            for repo in "$corpus_path"/*; do
                if [ -d "$repo" ]; then
                    analyze_repo "$repo"
                fi
            done
        fi
    done
done

echo "All analyses complete. Results in: $OUTPUT_DIR"
```

### 2. Ground Truth Annotation Tool

Create `scripts/annotate_ground_truth.py`:

```python
#!/usr/bin/env python3
"""
Interactive tool for creating ground truth annotations.
Allows developers to mark true/false positives for detector calibration.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List

def load_analysis_results(results_file: Path) -> Dict:
    with open(results_file) as f:
        return json.load(f)

def annotate_issue(issue: Dict) -> Dict:
    """Prompt user to annotate an issue"""
    print(f"\n{'='*80}")
    print(f"File: {issue.get('file_path', 'unknown')}")
    print(f"Line: {issue.get('line_number', 'unknown')}")
    print(f"Type: {issue.get('issue_type', 'unknown')}")
    print(f"Severity: {issue.get('severity', 'unknown')}")
    print(f"Message: {issue.get('message', 'unknown')}")
    print(f"{'='*80}")

    while True:
        verdict = input("Verdict (tp/fp/skip/quit): ").lower().strip()
        if verdict in ['tp', 'fp', 'skip', 'quit']:
            break
        print("Invalid input. Please enter: tp (true positive), fp (false positive), skip, or quit")

    if verdict == 'quit':
        return None

    issue['annotation'] = {
        'verdict': verdict,
        'notes': input("Notes (optional): ").strip() if verdict != 'skip' else ''
    }

    return issue

def main():
    if len(sys.argv) < 2:
        print("Usage: annotate_ground_truth.py <analysis_results.json>")
        sys.exit(1)

    results_file = Path(sys.argv[1])
    results = load_analysis_results(results_file)

    issues = results.get('issues', [])
    print(f"Found {len(issues)} issues to annotate")

    annotated = []
    for i, issue in enumerate(issues, 1):
        print(f"\nIssue {i}/{len(issues)}")
        annotated_issue = annotate_issue(issue)
        if annotated_issue is None:
            break
        if annotated_issue.get('annotation', {}).get('verdict') != 'skip':
            annotated.append(annotated_issue)

    # Save ground truth
    output_file = results_file.parent / f"{results_file.stem}_ground_truth.json"
    with open(output_file, 'w') as f:
        json.dump({
            'source_file': str(results_file),
            'issues': annotated
        }, f, indent=2)

    print(f"\nGround truth saved to: {output_file}")
    print(f"Annotated {len(annotated)} issues")

if __name__ == '__main__':
    main()
```

### 3. Metrics Calculation Script

Create `scripts/calculate_metrics.py`:

```python
#!/usr/bin/env python3
"""
Calculate precision, recall, F1 score from ground truth annotations.
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

def calculate_metrics(ground_truth_file: Path):
    with open(ground_truth_file) as f:
        data = json.load(f)

    issues = data.get('issues', [])

    # Count by detector type
    metrics_by_detector = defaultdict(lambda: {'tp': 0, 'fp': 0, 'fn': 0})

    for issue in issues:
        detector = issue.get('issue_type', 'unknown')
        verdict = issue.get('annotation', {}).get('verdict')

        if verdict == 'tp':
            metrics_by_detector[detector]['tp'] += 1
        elif verdict == 'fp':
            metrics_by_detector[detector]['fp'] += 1

    # Calculate metrics
    print(f"\n{'Detector':<25} {'Precision':<12} {'Issues':<10} {'TP':<8} {'FP':<8}")
    print("="*75)

    overall_tp = 0
    overall_fp = 0

    for detector, counts in sorted(metrics_by_detector.items()):
        tp = counts['tp']
        fp = counts['fp']
        total = tp + fp

        precision = tp / total if total > 0 else 0.0

        print(f"{detector:<25} {precision:>6.2%}      {total:<10} {tp:<8} {fp:<8}")

        overall_tp += tp
        overall_fp += fp

    # Overall metrics
    overall_total = overall_tp + overall_fp
    overall_precision = overall_tp / overall_total if overall_total > 0 else 0.0

    print("="*75)
    print(f"{'OVERALL':<25} {overall_precision:>6.2%}      {overall_total:<10} {overall_tp:<8} {overall_fp:<8}")
    print(f"\nFalse Positive Rate: {overall_fp / overall_total * 100:.1f}%" if overall_total > 0 else "N/A")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: calculate_metrics.py <ground_truth.json>")
        sys.exit(1)

    calculate_metrics(Path(sys.argv[1]))
```

## Calibration Methodology

### Phase 1: Baseline Establishment (Day 1-2)

1. **Clone Required Repositories**
   ```bash
   # Rust
   git clone --depth 1 https://github.com/tokio-rs/tokio test-codebases/large/rust/tokio
   git clone --depth 1 https://github.com/actix/actix-web test-codebases/medium/rust/actix-web

   # Python
   git clone --depth 1 https://github.com/django/django test-codebases/large/python/django
   git clone --depth 1 https://github.com/pallets/flask test-codebases/medium/python/flask
   git clone --depth 1 https://github.com/pandas-dev/pandas test-codebases/large/python/pandas

   # JavaScript
   git clone --depth 1 https://github.com/vuejs/vue test-codebases/large/javascript/vue
   ```

2. **Run Baseline Analysis**
   ```bash
   ./scripts/calibrate_detectors.sh test-codebases
   ```

3. **Manual Annotation** (100 files per language)
   ```bash
   # Annotate samples from each language
   python3 scripts/annotate_ground_truth.py reports/calibration-*/tokio-analysis.json
   python3 scripts/annotate_ground_truth.py reports/calibration-*/django-analysis.json
   python3 scripts/annotate_ground_truth.py reports/calibration-*/vue-analysis.json
   ```

4. **Calculate Baseline Metrics**
   ```bash
   python3 scripts/calculate_metrics.py reports/calibration-*/tokio-analysis_ground_truth.json
   ```

### Phase 2: Threshold Adjustment (Day 3-4)

1. **Review Current Thresholds**
   - Long Methods: `src/analysis/detectors/long_methods.rs`
   - God Object: `src/analysis/detectors/god_object.rs`
   - Tight Coupling: `src/analysis/detectors/tight_coupling.rs`
   - Code Duplication: `src/analysis/detectors/code_duplication.rs`
   - Dead Code: `src/analysis/detectors/dead_code.rs`

2. **Implement Threshold Adjustments**
   - Update configuration in `src/analysis/detectors/config/`
   - Apply recommendations from DETECTOR_TOLERANCE_AUDIT.md

3. **Re-run Analysis**
   ```bash
   ./scripts/calibrate_detectors.sh test-codebases
   ```

4. **Compare Metrics**
   ```bash
   # Compare before/after
   python3 scripts/calculate_metrics.py reports/calibration-before/*_ground_truth.json
   python3 scripts/calculate_metrics.py reports/calibration-after/*_ground_truth.json
   ```

### Phase 3: Validation (Day 5)

1. **Cross-validation Testing**
   ```bash
   # Run on real-world projects not in training set
   cargo run -- analyze ../other-project --output-format json
   ```

2. **Performance Benchmarking**
   ```bash
   # Ensure no performance regression
   cargo bench detectors
   ```

3. **Documentation**
   - Create calibration report: `reports/detector-calibration-2025-11.md`
   - Update detector configurations
   - Update risk register R1

## Success Criteria

### Target Metrics (by 2025-11-08)

- **False Positive Rate:** < 15% (from current ~25%)
- **Precision:** > 85% (from current ~75%)
- **Coverage:** 100 files manually annotated per language
- **Performance:** < 10% analysis time regression

### Deliverables

1. Updated detector configuration files
2. Calibration report with before/after metrics
3. Ground truth dataset for future calibration
4. Updated documentation in detector modules
5. Risk register R1 status update

## Timeline

| Date | Activity | Owner | Deliverable |
|------|----------|-------|-------------|
| 2025-11-03 (Sun) | Clone repos, run baseline analysis | Austin | Baseline metrics |
| 2025-11-04 (Mon) | Manual annotation (Rust) | Austin | Rust ground truth |
| 2025-11-05 (Tue) | Manual annotation (Python/JS) | Austin | Python/JS ground truth |
| 2025-11-06 (Wed) | Threshold adjustments, re-analysis | Austin | Updated configs |
| 2025-11-07 (Thu) | Validation, benchmarking | Austin | Performance data |
| 2025-11-08 (Fri) | Documentation, report generation | Austin | Calibration report |

## Dependencies

- [ ] Rust toolchain (already installed)
- [ ] Python 3.8+ for annotation scripts
- [ ] Git for cloning test repositories
- [ ] Sufficient disk space (~5GB for test repos)
- [ ] Access to `archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md`

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Manual annotation takes longer than expected | Medium | Reduce sample size to 50 files/language if needed |
| Test repos too large to analyze in time | Medium | Use --max-files flag to limit analysis |
| Performance regression from threshold changes | Medium | Roll back changes if regression >15% |
| Insufficient improvement in metrics | Low | Document findings and plan follow-up sprint |

## Next Steps

1. **Week of 2025-10-13:** Finalize scripts and test on existing corpus
2. **Week of 2025-10-20:** Clone additional repos and verify tooling
3. **Week of 2025-10-27:** Dry-run calibration process
4. **Week of 2025-11-03:** Execute calibration sprint

---

**Status:** Planning Complete
**Last Updated:** 2025-10-09
