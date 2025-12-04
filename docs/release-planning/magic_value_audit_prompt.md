# Uveddi Magic Value Detection Audit Prompt

## Objective

Objectively audit the `magic_values.rs` detector to identify the root cause of inaccurate results. The current detector has a ~60-75% false positive rate on magic value detection.

---

## Context

Uveddi is a Rust-based code analysis tool. The magic value detector (`src/analysis/detectors/anti_patterns/magic_values.rs`, 694 lines) is designed to identify unexplained literals (numbers and strings) that should be named constants.

### Current Performance Issues

From alpha testing feedback:
- **False Positive Rate:** 60-75%
- **515 magic values flagged** on a 9,094 LOC codebase, but only ~50-100 are legitimate
- **Severity miscalibration:** Many "High" severity flags are not actionable
- **Missing exclusions:** Test code, env vars, database column names, serde attributes

---

## Source Code to Audit

**File:** `src/analysis/detectors/anti_patterns/magic_values.rs`

Key components:
1. `MagicValuesConfig` (lines 26-70) - Configuration with allowlists
2. `is_integer_allowed()` (lines 132-180) - Integer filtering heuristics
3. `is_float_allowed()` (lines 183-206) - Float filtering heuristics
4. `is_string_allowed()` (lines 209-325) - String filtering heuristics (most complex)
5. `determine_context()` (lines 328-364) - AST context detection
6. `determine_severity()` (lines 376-390) - Severity classification

---

## Audit Tasks

### 1. Heuristic Completeness Analysis

Review the allowlist logic and identify gaps. For each category, answer:

**Integer Allowlist (`is_integer_allowed`):**
- Are small numbers (0-10) properly excluded? (Check lines 157-159)
- Are powers of 2 properly excluded? (Check line 139)
- Are timeout values (100, 500, 1000, etc.) properly excluded? (Check lines 163-165)
- Are HTTP status codes properly excluded? (Check lines 173-177)
- **Missing patterns to identify:** What common integer patterns are NOT covered?

**String Allowlist (`is_string_allowed`):**
- Are env var names excluded? (Check lines 247-249 - looks for uppercase+underscore)
- Are serde attribute strings excluded? (NOT FOUND - this is a gap)
- Are database column names excluded? (NOT FOUND - this is a gap)
- Are common framework strings excluded? (Check lines 253-268)
- **Missing patterns to identify:** What common string patterns are NOT covered?

### 2. Context Detection Accuracy

Review `determine_context()` (lines 328-364):
- Does it correctly detect `const` declarations?
- Does it correctly detect function arguments vs assignments?
- Does it differentiate between test code and production code? (Likely NOT - this is a gap)

### 3. Severity Calibration

Review `determine_severity()` (lines 376-390):
- Is "High" severity appropriate for function arguments?
- Is "High" severity appropriate for comparisons?
- Should test code findings have reduced severity?

### 4. Test Code Detection (MISSING FEATURE)

The detector does NOT currently detect test code. Identify:
- Where should test code detection be added?
- What heuristics would identify test code? (e.g., `#[test]`, `#[cfg(test)]`, `mod tests`)

### 5. Specific False Positive Patterns to Investigate

From the accuracy assessment, these patterns caused false positives:

| Pattern | Example | Expected Behavior |
|---------|---------|-------------------|
| Env var names | `env::var("MODE")` | Should be IGNORED |
| Serde attributes | `#[serde(rename = "snake_case")]` | Should be IGNORED |
| DB column names | `.bind("tool_name")` | Should be IGNORED |
| Test data | `let test_id = "test_agent"` in `#[test]` | Should be LOW or IGNORED |
| Self-documenting | `Duration::from_secs(30)` | Should be IGNORED |
| Single-use values | Used exactly once | Should be LOW, not HIGH |

---

## Questions to Answer

1. **Root cause priority:** Which missing heuristic contributes most to false positives?
   - Test code exclusion?
   - Serde attribute exclusion?
   - Env var detection gaps?
   - DB column name exclusion?

2. **Quick wins:** Which fixes would have highest impact with lowest code changes?

3. **Severity recalibration:** Should the default severity mapping be changed?

4. **Configuration options:** Should users be able to toggle test code analysis?

5. **Architectural issues:** Is the current approach (allowlist-based) fundamentally flawed, or just incomplete?

---

## Expected Output

Provide a structured report with:

1. **Top 3 root causes** of false positives, ranked by impact
2. **Specific code changes** needed to address each root cause
3. **Recommended severity mapping** changes
4. **Prioritized fix list** (Quick Win vs. Medium Effort vs. Major Refactor)

---

## Reference: Known False Positive Examples

From the `limits.rs` file (105 issues flagged, ~85% false positives):

```rust
// SHOULD BE IGNORED (test data):
#[cfg(test)]
mod tests {
    const TEST_CPU_LIMIT: u64 = 1_000_000;  // Flagged as magic number
    let test_agent = "test_agent";          // Flagged as magic string
}

// SHOULD BE IGNORED (const declaration):
const DEFAULT_CPU_FUEL: u64 = 1_000_000;    // Already in const - should be ignored

// SHOULD BE IGNORED (well-known pattern):
Duration::from_secs(30)                      // Self-documenting timeout

// SHOULD BE IGNORED (env var):
env::var("OLLAMA_API_URL")                   // Env var name

// SHOULD BE MEDIUM, not HIGH (single use):
if percentage > 90.0 { ... }                 // Used once, in context
```

---

## File Location

Audit target: `/home/austingreen/Documents/botzr/projects/uveddi/src/analysis/detectors/anti_patterns/magic_values.rs`

Supporting files:
- `/home/austingreen/Documents/botzr/projects/uveddi/Uveddi Accuracy assessment.md` (feedback)
- `/home/austingreen/Documents/botzr/projects/uveddi/Uveddi Accuracy Improvement Report.md` (recommendations)
