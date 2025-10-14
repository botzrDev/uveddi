# Code Analysis Report

**Generated:** 2025-10-14 20:47:27 UTC

## Summary

- **Files Analyzed:** 1
- **Issues Found:** 3
- **Analysis Duration:** 0.05s

---

## Issues by Severity

### ⚪ Low (3 issues)

#### Long method 'complex_data_processing_algorithm_v1' detected: 67 lines, 89 statements, complexity 14

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 56

**Code:**
```
pub fn complex_data_processing_algorithm_v1(input_data: Vec<i32>) -> Result<ProcessedData, ProcessingError> {
    let mut result = ProcessedData::new();

    // Phase 1: Input Validation (15 lines)
    if input_data.is_empty() {
...
```

**Recommendation:** Consider breaking down 'complex_data_processing_algorithm_v1' into smaller, more focused methods. Current metrics: LOC=67, Statements=89, Complexity=14, Nesting=6

---

#### Long method 'complex_data_processing_algorithm_v2' detected: 67 lines, 89 statements, complexity 14

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 149

**Code:**
```
pub fn complex_data_processing_algorithm_v2(input_data: Vec<i32>) -> Result<ProcessedData, ProcessingError> {
    let mut result = ProcessedData::new();

    // Phase 1: Input Validation (15 lines)
    if input_data.is_empty() {
...
```

**Recommendation:** Consider breaking down 'complex_data_processing_algorithm_v2' into smaller, more focused methods. Current metrics: LOC=67, Statements=89, Complexity=14, Nesting=6

---

#### Long method 'analyze_numeric_dataset' detected: 67 lines, 89 statements, complexity 14

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 242

**Code:**
```
pub fn analyze_numeric_dataset(numbers: Vec<i32>) -> Result<ProcessedData, ProcessingError> {
    let mut output = ProcessedData::new();

    // Validation phase
    if numbers.is_empty() {
...
```

**Recommendation:** Consider breaking down 'analyze_numeric_dataset' into smaller, more focused methods. Current metrics: LOC=67, Statements=89, Complexity=14, Nesting=6

---

