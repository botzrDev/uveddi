# Code Analysis Report

**Generated:** 2025-10-14 20:54:27 UTC

## Summary

- **Files Analyzed:** 1
- **Issues Found:** 65
- **Analysis Duration:** 0.05s

---

## Issues by Severity

### 🟠 High (19 issues)

#### Magic value '10000' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 68

---

#### Magic value '2.0' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 92

---

#### Magic value '10000' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 161

---

#### Magic value '2.0' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 185

---

#### Magic value '10000' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 254

---

#### Magic value '2.0' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 277

---

#### Magic value '18' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 373

---

#### Magic value '100.50' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 378

---

#### Magic value '"administrator"' used in comparison operation (high). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 383

---

#### Magic value '"guest"' used in comparison operation (high). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 388

---

#### Magic value '300' passed as function argument (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 394

---

#### Magic value '"localhost"' passed as function argument (high). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 397

---

#### Magic value '5432' passed as function argument (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 397

---

#### Magic value '100' used in comparison operation (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 456

---

#### Magic value '25' passed as function argument (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 464

---

#### Magic value '"debug"' used in comparison operation (high). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 484

---

#### Magic value '25' passed as function argument (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 528

---

#### Magic value '200.0' passed as function argument (high). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 528

---

#### Magic value '"user"' passed as function argument (high). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 528

---

### ⚪ Low (46 issues)

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

#### Magic value '100' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 111

---

#### Magic value '101' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 114

---

#### Magic value '500' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 114

---

#### Magic value '501' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 117

---

#### Magic value '10000' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 117

---

#### Magic value '100' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 204

---

#### Magic value '101' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 207

---

#### Magic value '500' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 207

---

#### Magic value '501' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 210

---

#### Magic value '10000' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 210

---

#### Magic value '100' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 296

---

#### Magic value '101' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 299

---

#### Magic value '500' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 299

---

#### Magic value '501' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 302

---

#### Magic value '10000' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 302

---

#### Magic value '"Admin access granted"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 384

---

#### Magic value '"Registered user"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 389

---

#### Magic value '5' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 406

---

#### Magic value '3600' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 409

---

#### Magic value '500' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 412

---

#### Magic value '0.75' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 415

---

#### Magic value '"production"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 418

---

#### Magic value '10' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 444

---

#### Magic value '20' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 444

---

#### Magic value '30' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 444

---

#### Magic value '40' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 444

---

#### Magic value '50' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 444

---

#### Magic value '50' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 461

---

#### Magic value '60' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 468

---

#### Magic value '3' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 469

---

#### Magic value '8080' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 470

---

#### Magic value '"Success"' used in field initialization (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 473

---

#### Magic value '"Debug mode enabled"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 485

---

#### Magic value '"https://api.example.com/v1/users"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 489

---

#### Magic value '"production"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 493

---

#### Magic value '"prod"' used in field initialization (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 493

---

#### Magic value '"staging"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 494

---

#### Magic value '"stage"' used in field initialization (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 494

---

#### Magic value '"development"' used in field initialization (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 495

---

#### Magic value '3' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 505

---

#### Magic value '5' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 505

---

#### Magic value '3' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 521

---

#### Magic value '5' found in code (low). Consider extracting this magic number into a named constant with a descriptive name.

- **File:** `tests/detector_validation_extended.rs`
- **Line:** 521

---

