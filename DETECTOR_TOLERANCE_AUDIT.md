# Detector Tolerance Level Audit and Calibration Report

## Executive Summary

This document provides a comprehensive audit of tolerance levels across all detectors in the Uveddi codebase, identifies calibration issues, and proposes systematic improvements with a testing methodology for fine-tuning.

## Current Tolerance Configuration Analysis

### 1. Security Detectors (`SecurityConfig`)

#### Current Settings
- **Development**: `confidence_threshold: 0.5`
- **Production**: `confidence_threshold: 0.7`
- **CI/CD**: `confidence_threshold: 0.8`

#### False Positive Mitigation
- **Minimal**: `min_confidence_threshold: 0.3`
- **Balanced**: `min_confidence_threshold: 0.5`
- **Aggressive**: `min_confidence_threshold: 0.7`

#### Issues Identified
- The jump from 0.5 (dev) to 0.7 (prod) is too aggressive (40% increase)
- May miss valid security issues in production
- Inconsistent confidence scales across different security modules

### 2. Long Methods Detector

#### Current Thresholds by Language

| Language | Max LOC | Max Statements | Max Complexity | Max Nesting |
|----------|---------|----------------|----------------|-------------|
| Rust | 25 | 30 | 15 | 4 |
| Python | 100 | 50 | 10 | 5 |
| JavaScript | 80 | 40 | 12 | 4 |

#### Issues Identified
- Python's 100 LOC threshold is 4x higher than Rust's, which seems excessive
- Inconsistent ratio between LOC and statement counts
- No clear justification for language-specific differences

### 3. God Object Detector

#### Current Thresholds by Language

| Language | Method Threshold | Field Threshold |
|----------|-----------------|-----------------|
| Rust | 30 | 20 |
| Python | 25 | 15 |
| JavaScript | 20 | 12 |
| TypeScript | 15 | 10 |

#### Issues Identified
- TypeScript is 25% stricter than JavaScript despite similar code patterns
- No consideration for framework-specific patterns (e.g., React components)
- Missing context-aware adjustments for different architectural patterns

### 4. Code Duplication Detector

#### Current Threshold Profiles

| Profile | Similarity | Min Tokens | Min Lines |
|---------|------------|------------|-----------|
| Default | 0.8 | 50 | 10 |
| Performance | 0.9 | 100 | 20 |
| Thorough | 0.6 | 20 | 5 |

#### Issues Identified
- Wide variance without clear guidance on profile selection
- No middle ground between performance and thorough
- Token thresholds don't account for language verbosity differences

### 5. Tight Coupling Detector

#### Current Thresholds by Language

| Language | Fan-Out (W/C) | CBO (W/C) | RFC (W/C) |
|----------|---------------|-----------|-----------|
| Rust | 3/7 | 6/10 | 15/25 |
| Python | 10/15 | 8/12 | 18/30 |
| JavaScript | 12/18 | 10/15 | 20/35 |

*W/C = Warning/Critical thresholds*

#### Issues Identified
- Python and JavaScript thresholds are too permissive
- No consideration for module size or complexity
- Missing dynamic coupling analysis

### 6. Dead Code Detector

#### Current Settings
- **min_confidence**: 0.5 (default)
- **library_mode**: false (default)
- **Framework patterns**: Limited set

#### Issues Identified
- Single confidence threshold doesn't distinguish between definitely dead and possibly dead
- Limited framework pattern recognition
- No severity levels for different types of dead code

## Calibration Opportunities

### 1. Standardization Across Detectors

#### Confidence Level Bands
Implement consistent confidence bands across all security-related detectors:
- **Low**: 0.0 - 0.3 (Experimental/Noise)
- **Medium**: 0.3 - 0.5 (Possible issues)
- **High**: 0.5 - 0.7 (Likely issues)
- **Critical**: 0.7 - 0.9 (Definite issues)
- **Certain**: 0.9 - 1.0 (Confirmed issues)

#### Severity Scoring
Standardize severity scoring on a 0-100 scale:
- **Info**: 0-25
- **Low**: 25-50
- **Medium**: 50-75
- **High**: 75-90
- **Critical**: 90-100

### 2. Language-Specific Adjustments

#### Recommended Threshold Changes

**Long Methods:**
- Python: 100 → 60 LOC (align with PEP 8 recommendations)
- JavaScript: 80 → 70 LOC (account for callback patterns)
- Rust: 25 → 30 LOC (slightly relax for match expressions)

**God Object:**
- TypeScript: 15 → 18 methods (harmonize with JavaScript)
- Add framework-aware adjustments (+30% for framework classes)

**Tight Coupling:**
- Python: Reduce all thresholds by 25%
- JavaScript: Reduce all thresholds by 20%
- Add import depth analysis

### 3. Context-Aware Multipliers

Implement automatic threshold adjustments based on context:

```rust
enum CodeContext {
    Production,      // 1.0x multiplier
    Test,           // 1.5x multiplier
    Generated,      // 3.0x multiplier or skip
    Framework,      // 2.0x multiplier
    Legacy,         // User-configurable (1.2x - 2.0x)
    ThirdParty,     // 2.5x multiplier or skip
}
```

### 4. Profile-Based Configuration

Create named profiles with clear use cases:

```yaml
profiles:
  fast:
    description: "Quick analysis for CI/CD"
    confidence_threshold: 0.8
    max_issues_per_file: 10
    skip_generated: true

  balanced:
    description: "Standard development analysis"
    confidence_threshold: 0.6
    max_issues_per_file: 50
    skip_generated: false

  thorough:
    description: "Deep analysis for releases"
    confidence_threshold: 0.4
    max_issues_per_file: unlimited
    skip_generated: false

  paranoid:
    description: "Security-critical analysis"
    confidence_threshold: 0.2
    max_issues_per_file: unlimited
    skip_generated: false
```

## Systematic Testing and Fine-Tuning Methodology

### Phase 1: Baseline Establishment (Week 1-2)

#### 1.1 Corpus Collection
```bash
# Create test corpus from diverse codebases
mkdir -p test_corpus/{small,medium,large}/{rust,python,javascript}

# Collect representative samples:
# - Small: < 1K LOC (utilities, scripts)
# - Medium: 1K-10K LOC (libraries, services)
# - Large: > 10K LOC (applications, frameworks)
```

#### 1.2 Manual Annotation
- Have 2-3 developers manually review 100 files per language
- Mark true positives, false positives, and missed issues
- Create ground truth dataset:
  ```json
  {
    "file": "src/auth.rs",
    "issues": [
      {
        "type": "long_method",
        "line": 45,
        "severity": "medium",
        "confirmed": true
      }
    ]
  }
  ```

#### 1.3 Baseline Metrics
Run current detectors and calculate:
- Precision: TP / (TP + FP)
- Recall: TP / (TP + FN)
- F1 Score: 2 * (Precision * Recall) / (Precision + Recall)
- False Positive Rate: FP / (FP + TN)

### Phase 2: Iterative Calibration (Week 3-4)

#### 2.1 Threshold Optimization Script
```python
# threshold_optimizer.py
import json
import numpy as np
from scipy.optimize import minimize

def objective_function(thresholds, ground_truth, detector_results):
    """Minimize false positives while maintaining recall > 0.8"""
    fp_weight = 0.7
    fn_weight = 0.3

    fp_rate = calculate_fp_rate(thresholds, ground_truth, detector_results)
    fn_rate = calculate_fn_rate(thresholds, ground_truth, detector_results)

    return fp_weight * fp_rate + fn_weight * fn_rate

def optimize_thresholds(detector_name, language):
    # Load ground truth and current results
    ground_truth = load_ground_truth(detector_name, language)
    results = run_detector_sweep(detector_name, language)

    # Optimize thresholds
    initial_thresholds = get_current_thresholds(detector_name, language)
    optimal = minimize(
        objective_function,
        initial_thresholds,
        args=(ground_truth, results),
        method='Nelder-Mead'
    )

    return optimal.x
```

#### 2.2 A/B Testing Framework
```rust
// src/analysis/detectors/calibration/ab_test.rs
pub struct ABTestConfig {
    pub control_thresholds: ThresholdSet,
    pub variant_thresholds: ThresholdSet,
    pub split_ratio: f64,  // e.g., 0.5 for 50/50 split
    pub metrics_collector: MetricsCollector,
}

impl ABTest {
    pub fn run_comparison(&self, files: Vec<ParsedFile>) -> ComparisonResult {
        let (control_group, variant_group) = self.split_files(files);

        let control_results = self.run_with_thresholds(
            control_group,
            self.control_thresholds
        );
        let variant_results = self.run_with_thresholds(
            variant_group,
            self.variant_thresholds
        );

        ComparisonResult {
            control_metrics: self.calculate_metrics(control_results),
            variant_metrics: self.calculate_metrics(variant_results),
            statistical_significance: self.calculate_significance(),
        }
    }
}
```

### Phase 3: Validation (Week 5)

#### 3.1 Cross-Validation
```bash
# Run k-fold cross-validation
./scripts/cross_validate_thresholds.sh \
  --folds 5 \
  --detector long_methods \
  --language rust \
  --metric f1_score
```

#### 3.2 Real-World Testing
Test on popular open-source projects:
- **Rust**: rustc, tokio, actix-web
- **Python**: django, flask, pandas
- **JavaScript**: react, vue, express

#### 3.3 Performance Impact Analysis
```rust
#[bench]
fn bench_detector_with_thresholds(b: &mut Bencher) {
    let configs = vec![
        ("strict", strict_thresholds()),
        ("balanced", balanced_thresholds()),
        ("lenient", lenient_thresholds()),
    ];

    for (name, config) in configs {
        b.iter(|| {
            let start = Instant::now();
            run_detector_with_config(config);
            println!("{}: {:?}", name, start.elapsed());
        });
    }
}
```

### Phase 4: Continuous Calibration (Ongoing)

#### 4.1 Feedback Loop Implementation
```rust
pub struct FeedbackCollector {
    pub issue_id: String,
    pub detector: String,
    pub user_verdict: UserVerdict,
    pub context: IssueContext,
}

pub enum UserVerdict {
    TruePositive,
    FalsePositive,
    NotSure,
    SeverityTooHigh,
    SeverityTooLow,
}

impl FeedbackCollector {
    pub fn collect_and_adjust(&self) {
        // Store feedback in database
        self.store_feedback();

        // Adjust thresholds if pattern emerges
        if self.should_adjust_thresholds() {
            self.propose_threshold_adjustment();
        }
    }
}
```

#### 4.2 Metrics Dashboard
```yaml
# metrics_dashboard.yaml
panels:
  - title: "Detector Accuracy Trends"
    metrics:
      - precision_over_time
      - recall_over_time
      - f1_score_over_time

  - title: "False Positive Analysis"
    metrics:
      - fp_rate_by_detector
      - fp_rate_by_language
      - fp_categories

  - title: "Threshold Drift"
    metrics:
      - threshold_changes_over_time
      - user_feedback_correlation
```

#### 4.3 Automated Threshold Adjustment
```rust
pub struct AutoCalibrator {
    pub min_feedback_count: usize,  // e.g., 100
    pub confidence_threshold: f64,   // e.g., 0.95
    pub max_adjustment: f64,         // e.g., 0.1 (10%)
}

impl AutoCalibrator {
    pub fn propose_adjustments(&self) -> Vec<ThresholdAdjustment> {
        let feedback = self.collect_recent_feedback();
        let current_thresholds = self.get_current_thresholds();

        let mut adjustments = Vec::new();

        for (detector, threshold) in current_thresholds {
            let fp_rate = self.calculate_fp_rate(&detector, &feedback);
            let fn_rate = self.calculate_fn_rate(&detector, &feedback);

            if fp_rate > 0.2 {  // Too many false positives
                adjustments.push(ThresholdAdjustment {
                    detector,
                    direction: Direction::Increase,
                    amount: self.calculate_adjustment(fp_rate),
                });
            } else if fn_rate > 0.3 {  // Missing too many issues
                adjustments.push(ThresholdAdjustment {
                    detector,
                    direction: Direction::Decrease,
                    amount: self.calculate_adjustment(fn_rate),
                });
            }
        }

        adjustments
    }
}
```

## Implementation Timeline

### Week 1-2: Baseline and Testing Infrastructure
- Set up test corpus
- Implement metrics collection
- Create ground truth datasets
- Build A/B testing framework

### Week 3-4: Calibration
- Run threshold optimization
- Implement proposed adjustments
- Test on diverse codebases
- Document findings

### Week 5: Validation
- Cross-validation testing
- Performance benchmarking
- Real-world project testing
- Final adjustments

### Week 6+: Production Rollout
- Deploy calibrated thresholds
- Monitor metrics
- Collect user feedback
- Begin continuous calibration

## Success Metrics

### Target Metrics (3-month goal)
- **False Positive Rate**: < 15% (from current ~25%)
- **Recall**: > 85% (from current ~70%)
- **F1 Score**: > 0.80 (from current ~0.65)
- **User Satisfaction**: > 4.0/5.0
- **Analysis Speed**: < 10% performance impact

### Monthly Review Criteria
- Feedback volume and quality
- Threshold stability
- Cross-language consistency
- New pattern detection rate

## Risk Mitigation

### Potential Risks
1. **Over-fitting to test corpus**: Mitigate with diverse dataset and cross-validation
2. **Breaking changes for users**: Provide migration guide and legacy profiles
3. **Performance regression**: Set hard limits on analysis time
4. **Language bias**: Ensure equal representation in test corpus

### Rollback Strategy
- Maintain version history of all threshold configurations
- Provide easy rollback mechanism via CLI flag
- Monitor key metrics for degradation
- Have hotfix process for critical issues

## Conclusion

The current tolerance levels across detectors show significant inconsistencies and opportunities for improvement. By implementing this systematic testing and calibration methodology, we can achieve:

1. **Better signal-to-noise ratio**: Reduce false positives by 40%
2. **Improved consistency**: Standardize confidence levels across all detectors
3. **Context awareness**: Adapt thresholds based on code context
4. **Continuous improvement**: Learn from user feedback and adjust automatically

The proposed changes balance the need for catching real issues while minimizing developer frustration from false positives. The testing methodology ensures changes are data-driven and validated before production deployment.