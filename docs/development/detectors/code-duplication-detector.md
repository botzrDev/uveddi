# Code Duplication Detector

The Code Duplication Detector identifies similar or identical code blocks across your codebase, helping you maintain DRY (Don't Repeat Yourself) principles and reduce technical debt.

## Overview

| Feature | Details |
|---------|---------|
| **Detector Name** | `CodeDuplicationDetector` |
| **Registry Key** | `code_duplication` |
| **Status** | ✅ Fully Integrated |
| **Clone Types** | Type-1 through Type-4 |

## Supported Clone Types

| Type | Description | Detection Method |
|------|-------------|------------------|
| **Type-1** | Exact clones (identical code) | Token fingerprinting |
| **Type-2** | Renamed clones (different identifiers/literals) | Normalized token comparison |
| **Type-3** | Near-miss clones (small modifications) | Similarity threshold matching |
| **Type-4** | Semantic clones (different syntax, same behavior) | CFG/semantic analysis (optional) |

## Configuration

### Default Configuration

```rust
use uveddi::analysis::detectors::anti_patterns::code_duplication::DuplicationConfig;

let config = DuplicationConfig::new(); // Default settings
```

### Key Configuration Options

| Parameter | Default | Description |
|-----------|---------|-------------|
| `min_tokens` | 50 | Minimum tokens to consider a block |
| `min_lines` | 10 | Minimum lines to consider a block |
| `similarity_threshold` | 0.8 | Required similarity for detection (0.0-1.0) |
| `ignore_identifiers` | true | Normalize variable names for Type-2 |
| `ignore_literals` | true | Normalize literal values for Type-2 |

### Preset Configurations

```rust
// For fast scanning (fewer, more confident matches)
let config = DuplicationConfig::performance_optimized();

// For thorough analysis (more matches, lower thresholds)
let config = DuplicationConfig::thorough_analysis();

// Custom configuration with builder pattern
let config = DuplicationConfig::new()
    .with_min_tokens(20)
    .with_similarity_threshold(0.7);
```

## Integration

The detector is **automatically included** in the default detector set:

```rust
// Automatically included when using default detectors
let detectors = DetectorFactory::create_default_detectors();

// Or explicitly create
let detector = CodeDuplicationDetector::new();

// Or with custom config
let detector = CodeDuplicationDetector::with_config(config);
```

### CLI Usage

```bash
# Run with all default detectors (includes code duplication)
uveddi analyze ./src --output-format json

# Target specific detector
uveddi analyze ./src --detector code_duplication
```

## Test Coverage

The detector has comprehensive integration tests:

| Test | Purpose |
|------|---------|
| `test_code_duplication_detector_instantiation` | Verifies detector setup |
| `test_code_duplication_exact_match` | Tests Type-1 clone detection |
| `test_code_duplication_similar_blocks` | Tests Type-2 clone detection |
| `test_code_duplication_negative_case` | Verifies no false positives |
| `test_code_duplication_threshold_comparison` | Tests threshold configuration |
| `test_code_duplication_rust_patterns` | Tests Rust-specific patterns |

Run tests with:
```bash
cargo test --features tree-sitter --test code_duplication_detection
```

## Example Detection

**Input: Duplicated Functions**
```rust
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    if result > 100 {
        println!("Result is large: {}", result);
    }
    result
}

fn compute_total(x: i32, y: i32) -> i32 {  // Clone of calculate_sum
    let result = x + y;
    if result > 100 {
        println!("Result is large: {}", result);
    }
    result
}
```

**Output: Detected Issue**
```
Duplicate code detected: 85% similarity between lines 1-7 and 9-15
Suggestion: Consider extracting the common logic into a shared function
```

## Supported Languages

- Rust
- Python  
- JavaScript
- TypeScript

## Architecture

```
src/analysis/detectors/anti_patterns/code_duplication/
├── mod.rs           # Module exports
├── detector.rs      # Core detector implementation
├── config.rs        # DuplicationConfig
├── algorithms.rs    # Clone detection algorithms
├── types.rs         # CodeBlock, ClonePair, CloneType
├── metrics.rs       # Similarity scoring
└── language_support.rs  # Language-specific handling
```

## Performance Notes

- Uses rolling hash fingerprints for efficient matching
- Token-based normalization for Type-2 detection
- Optional CFG analysis for Type-4 (disabled by default for performance)
- Scales well on codebases up to 100k+ LOC
