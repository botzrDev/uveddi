# Code Analysis Report

**Generated:** 2025-10-14 20:16:12 UTC

## Summary

- **Files Analyzed:** 1
- **Issues Found:** 3
- **Analysis Duration:** 0.06s

---

## Issues by Severity

### 🔴 Critical (1 issues)

#### Long method 'extremely_long_and_complex_method' detected: 161 lines, 189 statements, complexity 92

- **File:** `tests/validation_test_cases.rs`
- **Line:** 66

**Code:**
```
pub fn extremely_long_and_complex_method(input: i32) -> Result<String, String> {
    let mut result = String::new();

    // Start deep nesting
    if input > 0 {
...
```

**Recommendation:** Consider breaking down 'extremely_long_and_complex_method' into smaller, more focused methods. Current metrics: LOC=161, Statements=189, Complexity=92, Nesting=22

---

### 🟡 Medium (1 issues)

#### God Object detected: 'MassiveGodObject' has 32 methods and 15 fields. (Thresholds: methods>30, fields>20) LCOM4 score: 1 (>1 indicates low cohesion)

- **File:** `tests/validation_test_cases.rs`
- **Line:** 11

**Code:**
```
pub struct MassiveGodObject {
    field1: String,
    field2: i32,
    field3: Vec<String>,
    field4: bool,
    field5: f64,
    field6: Option<String>,
    field7: Result<i32, String>,
    field8: Box<dyn std::error::Error>,
    field9: std::collections::HashMap<String, i32>,
    field10: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    field11: String,
    field12: i32,
    field13: bool,
    field14: f64,
    field15: String,
}
```

---

### ⚪ Low (1 issues)

#### Large class 'MassiveGodObject' detected: 0 LOC, 32 methods, 15 fields

- **File:** `tests/validation_test_cases.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

