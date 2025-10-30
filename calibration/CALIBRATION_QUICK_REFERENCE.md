# Detector Calibration - Quick Reference Card

## At a Glance

### Confidence Levels (0.0 - 1.0)
| Level    | Range     | Meaning                | Action                    |
|----------|-----------|------------------------|---------------------------|
| Low      | 0.0 - 0.3 | Experimental/Noise     | Review manually           |
| Medium   | 0.3 - 0.5 | Possible issue         | Schedule for review       |
| High     | 0.5 - 0.7 | Likely issue           | Prioritize refactoring    |
| Critical | 0.7 - 0.9 | Definite issue         | Address in current sprint |
| Certain  | 0.9 - 1.0 | Confirmed issue        | Fix immediately           |

### Severity Scores (0 - 100)
| Level    | Range  | Color  | Priority |
|----------|--------|--------|----------|
| Info     | 0-25   | 🔵 Blue  | P4       |
| Low      | 25-50  | 🟢 Green | P3       |
| Medium   | 50-75  | 🟡 Yellow| P2       |
| High     | 75-90  | 🟠 Orange| P1       |
| Critical | 90-100 | 🔴 Red   | P0       |

### Context Multipliers
| Context      | Multiplier | Example Path               | Skip? |
|--------------|------------|----------------------------|-------|
| Production   | 1.0x       | src/main.rs                | No    |
| Test         | 1.5x       | tests/integration_test.rs  | No    |
| Generated    | 3.0x       | generated/api_pb2.py       | Yes*  |
| Framework    | 2.0x       | src/axum_handler.rs        | No    |
| Legacy       | 1.2x       | legacy/old_code.rs         | No    |
| ThirdParty   | 2.5x       | node_modules/lib/foo.js    | Yes*  |
| Build        | 2.0x       | scripts/build.sh           | No    |
| Documentation| 2.5x       | examples/tutorial.rs       | No    |

*Default behavior, configurable

### Profiles
| Profile   | Use Case              | Confidence | Severity | Skip Tests | Skip Generated |
|-----------|-----------------------|------------|----------|------------|----------------|
| Fast      | CI/CD, quick feedback | 0.7-0.9    | High+    | Yes        | Yes            |
| Balanced  | Daily development     | 0.5-0.8    | Medium+  | No         | Yes            |
| Thorough  | Pre-release analysis  | 0.3-0.7    | Low+     | No         | No             |
| Paranoid  | Security audits       | 0.0-0.5    | Info+    | No         | No             |
| Legacy    | Old codebases         | 0.5-0.8    | High+    | Yes        | Yes            |

## Updated Thresholds

### Long Methods (LOC)
```
Rust:       25 → 30  (+20%, allow match expressions)
Python:    100 → 60  (-40%, align with PEP 8)
JavaScript: 80 → 70  (-13%, account for callbacks)
TypeScript: 80 → 70  (same as JavaScript)
```

### God Object (Methods/Fields)
```
Rust:       30/20 (unchanged)
Python:     25/15 (unchanged)
JavaScript: 20/12 (unchanged)
TypeScript: 15/10 → 18/12 (+20%/+20%, harmonize with JS)
```

### Tight Coupling (FanOut/CBO/RFC Warning/Critical)
```
Rust:       3/7, 6/10, 15/25 (unchanged)
Python:     10/15, 8/12, 18/30 → 8/11, 6/9, 14/23 (-25%)
JavaScript: 12/18, 10/15, 20/35 → 10/14, 8/12, 16/28 (-20%)
```

### Code Duplication (Similarity/Tokens/Lines)
```
Fast:       0.9, 100, 20
Balanced:   0.8,  50, 10
Thorough:   0.6,  20,  5
```

## Common Patterns

### Basic Usage
```rust
use uveddi::analysis::detectors::calibration::*;

// Get adjusted threshold for context
let context = ContextAwareThresholds::default_config();
let threshold = context.apply_to_int_threshold(100, "tests/foo.rs");
// Result: 150 (test code gets 1.5x multiplier)
```

### Calculate Severity
```rust
let severity = SeverityScore::from_components(vec![
    SeverityComponent::new("complexity", 30, 0.4),  // 40% weight
    SeverityComponent::new("size", 25, 0.3),        // 30% weight
    SeverityComponent::new("coupling", 20, 0.3),    // 30% weight
]);
// Result: Score 75, Level::High
```

### Check Confidence
```rust
let confidence = ConfidenceBand::from_score(0.75);
if confidence.should_report(0.7) {
    println!("{}", confidence.recommendation);
}
```

### Use Profile
```rust
let profile = ProfileConfig::thorough();
let should_analyze = profile.should_analyze_file("generated/foo.rs");
// Result: true (thorough analyzes everything)
```

## Environment Thresholds

### Development
```
Confidence: 0.5 (High)
Severity:   Info (0+)
Goal:       Catch issues early
```

### Staging
```
Confidence: 0.6 (between High/Critical)
Severity:   Low (25+)
Goal:       Filter noise before prod
```

### Production
```
Confidence: 0.7 (Critical)
Severity:   Medium (50+)
Goal:       Only real issues
```

### CI/CD
```
Confidence: 0.8 (near Certain)
Severity:   High (75+)
Goal:       Block only serious problems
```

## Feedback Verdicts

| Verdict           | Impact                      | Auto-Calibration Action        |
|-------------------|-----------------------------|--------------------------------|
| TruePositive      | Confirms detection          | None                           |
| FalsePositive     | Increases FP count          | Increase threshold after 100+  |
| NotSure           | Neutral                     | None                           |
| SeverityTooHigh   | Valid but over-scored       | Decrease severity multiplier   |
| SeverityTooLow    | Valid but under-scored      | Increase severity multiplier   |

## Auto-Calibration Triggers

| Condition                | Action                        | Confidence Required |
|--------------------------|-------------------------------|---------------------|
| FP rate > 30%            | Increase threshold by 15-20%  | 0.95                |
| FP rate > 20%            | Increase threshold by 10-15%  | 0.90                |
| 30%+ "Severity Too High" | Decrease severity by 10%      | 0.90                |
| 30%+ "Severity Too Low"  | Increase severity by 10%      | 0.90                |

## Testing Metrics

### Target Goals (3 months)
```
Precision:   > 85% (current ~75%)
Recall:      > 85% (current ~70%)
F1 Score:    > 0.80 (current ~0.65)
FP Rate:     < 15% (current ~25%)
```

### Acceptable Ranges
```
Precision:   80-90%
Recall:      80-90%
F1 Score:    0.75-0.85
FP Rate:     10-20%
```

## File Paths

```
Core Implementation:
src/analysis/detectors/calibration/
├── confidence_bands.rs     - Confidence standardization
├── severity_scoring.rs     - Severity calculation
├── context_multipliers.rs  - Context-aware adjustments
├── profiles.rs             - Profile configurations
├── testing.rs              - Testing infrastructure
└── feedback.rs             - Feedback collection

Documentation:
DETECTOR_TOLERANCE_AUDIT.md           - Original audit
CALIBRATION_IMPLEMENTATION_GUIDE.md   - Implementation guide
CALIBRATION_SUMMARY.md                - Executive summary
CALIBRATION_QUICK_REFERENCE.md        - This file

Tests:
tests/calibration_integration_test.rs - Integration tests
```

## CLI Usage (Future)

```bash
# Use specific profile
uveddi analyze --profile thorough src/

# Custom thresholds
uveddi analyze --confidence-threshold 0.8 src/

# Skip contexts
uveddi analyze --skip-generated --skip-tests src/

# Enable auto-calibration
uveddi analyze --enable-auto-calibration src/
```

## Configuration File (Future)

```toml
# .uveddi/calibration.toml
[profile]
type = "balanced"

[confidence]
development = 0.5
production = 0.7

[context]
skip_generated = true
legacy_multiplier = 1.5

[feedback]
enable_auto_calibration = false
```

## Next Steps Checklist

- [ ] Add `pub mod calibration;` to `src/analysis/detectors/mod.rs`
- [ ] Run `cargo test` to verify
- [ ] Migrate one detector to use calibration
- [ ] Collect baseline metrics
- [ ] A/B test old vs new thresholds
- [ ] Roll out to all detectors
- [ ] Enable user feedback
- [ ] Monitor and iterate

## Support

- **Documentation**: See CALIBRATION_IMPLEMENTATION_GUIDE.md
- **Issues**: GitHub Issues
- **Questions**: See inline code documentation

---

**Version**: 0.0.2
**Last Updated**: 2025-10-09
**Status**: Ready for integration
