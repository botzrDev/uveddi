# Code Analysis Report

**Generated:** 2025-10-14 21:41:08 UTC

## Summary

- **Files Analyzed:** 1
- **Issues Found:** 11
- **Analysis Duration:** 0.07s

---

## Issues by Severity

### 🟡 Medium (1 issues)

#### Consider extracting the duplicated code into a shared function or module. Type-3 clone with 84.5% similarity.

- **File:** `src/main.rs`
- **Line:** 62

**Code:**
```
fn get_version_string() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        " (", env!("GIT_HASH"), "-", env!("GIT_DIRTY"), ")"
    )
}
```

---

### ⚪ Low (10 issues)

#### Magic value '"\n"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 72

---

#### Magic value '"\n"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 73

---

#### Magic value '"\n"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 75

---

#### Magic value '"\n"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 77

---

#### Magic value '"\n"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 78

---

#### Magic value '"\n"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 79

---

#### Magic value '"uveddi"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 86

---

#### Magic value '"wasm-plugins"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 122

---

#### Magic value '"compact"' used in field initialization (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 143

---

#### Magic value '"wasm-plugins"' found in code (low). Consider extracting this magic string into a named constant to improve maintainability.

- **File:** `src/main.rs`
- **Line:** 202

---

