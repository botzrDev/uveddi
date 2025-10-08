# Uveddi CLI-Only Alpha Testing Guide

## Mission
Comprehensive testing of the minimal CLI-only build to verify all core functionality works before community release.

## Build Information
- **Version:** 1.0.0-alpha (CLI-only release)
- **Build Profile:** `minimal` feature set
- **Binary Size:** 14MB (stripped release)
- **LOC:** ~208K lines of Rust code
- **Target:** CLI users who want fast, local code analysis

## Pre-Testing Setup

### 1. Build the Binary
```bash
# Clean build
cargo clean

# Build with minimal features (recommended for testing)
cargo build --release --features minimal

# Verify binary
./target/release/uveddi --version
# Expected: "uveddi 1.0.0"
```

### 2. Test Environment Setup
```bash
# Create test directory structure
mkdir -p ~/uveddi-testing/{small,medium,large,multi-lang}

# Prepare test codebases of different sizes:
# - small: 10-50 files, ~1K LOC
# - medium: 100-500 files, ~10K LOC  
# - large: 1000+ files, ~50K+ LOC
# - multi-lang: Mix of Rust, Python, JavaScript, TypeScript
```

## Testing Matrix

## Phase 1: CLI Interface & Help System

### Test 1.1: Basic Commands
```bash
# Version check
./target/release/uveddi --version

# Help output
./target/release/uveddi --help
./target/release/uveddi help

# Subcommand help
./target/release/uveddi analyze --help
./target/release/uveddi config --help
./target/release/uveddi doctor --help
./target/release/uveddi init --help
./target/release/uveddi hooks --help
./target/release/uveddi ci --help
```

**Expected Results:**
- [ ] Version displays correctly
- [ ] Help shows all available commands
- [ ] Each subcommand help is accessible
- [ ] No panic or crash on help requests
- [ ] Output is properly formatted for terminal

**Failures:** Record any missing commands, formatting issues, or crashes

---

### Test 1.2: Enhanced Help System
```bash
# Test enhanced help
./target/release/uveddi help analyze
./target/release/uveddi help config
./target/release/uveddi help examples
```

**Expected Results:**
- [ ] Enhanced help displays with examples
- [ ] Usage patterns are clear
- [ ] Code examples are properly formatted
- [ ] No broken formatting or missing sections

---

## Phase 2: Configuration Management

### Test 2.1: Configuration Initialization
```bash
cd ~/uveddi-testing/small

# Initialize config
uveddi init

# Verify config file created
ls -la uveddi.toml
cat uveddi.toml
```

**Expected Results:**
- [ ] `uveddi.toml` created in current directory
- [ ] Default configuration is valid TOML
- [ ] Contains reasonable defaults
- [ ] Comments explain each section

**Test:** Modify some values and re-run init
- [ ] Prompts before overwriting existing config

---

### Test 2.2: Configuration Commands
```bash
# Show current configuration
uveddi config show

# Get specific value
uveddi config get analysis.max_files
uveddi config get detectors.god_object.enabled

# Set values
uveddi config set analysis.max_files 1000
uveddi config set detectors.god_object.threshold 500

# Verify changes
uveddi config show | grep max_files
uveddi config show | grep god_object
```

**Expected Results:**
- [ ] `config show` displays full configuration
- [ ] `config get` retrieves specific values
- [ ] `config set` updates values correctly
- [ ] Changes persist across commands
- [ ] Invalid keys show helpful error messages

---

### Test 2.3: Configuration Validation
```bash
# Create invalid config
echo "invalid_toml = [" > uveddi.toml

# Try to use it
uveddi analyze ./src

# Create config with invalid values
cat > uveddi.toml << 'EOF'
[analysis]
max_files = -100
timeout = "not_a_number"
EOF

uveddi analyze ./src
```

**Expected Results:**
- [ ] Invalid TOML shows parse error with line number
- [ ] Invalid values show validation errors
- [ ] Error messages are helpful and actionable
- [ ] Application doesn't crash on bad config

---

## Phase 3: Analysis Engine - Core Functionality

### Test 3.1: Basic Analysis (Small Codebase)
```bash
cd ~/uveddi-testing/small

# Basic analysis with defaults
uveddi analyze .

# With specific output format
uveddi analyze . --output-format json
uveddi analyze . --output-format markdown
uveddi analyze . --output-format html

# With output file
uveddi analyze . --output-format json --output results.json
cat results.json | jq .
```

**Expected Results:**
- [ ] Analysis completes without errors
- [ ] Progress indicators show during analysis
- [ ] Results are printed to stdout (when no --output)
- [ ] JSON output is valid JSON
- [ ] Markdown output is properly formatted
- [ ] HTML output is valid HTML
- [ ] Output files are created at specified paths
- [ ] Results contain detected issues

**Metrics to verify:**
- [ ] Files analyzed count
- [ ] Total issues found
- [ ] Issues broken down by type
- [ ] Analysis completion time reported

---

### Test 3.2: Language-Specific Analysis
```bash
cd ~/uveddi-testing/multi-lang

# Rust-only analysis
uveddi analyze ./rust-code --language rust

# Python-only
uveddi analyze ./python-code --language python

# JavaScript
uveddi analyze ./js-code --language javascript

# TypeScript
uveddi analyze ./ts-code --language typescript

# Auto-detection (no --language flag)
uveddi analyze .
```

**Expected Results:**
- [ ] Each language is correctly analyzed
- [ ] Language auto-detection works
- [ ] Language-specific patterns are detected
- [ ] Unsupported language files are skipped gracefully
- [ ] Multi-language projects analyze all supported files

---

### Test 3.3: Detector Coverage
Test each anti-pattern detector individually:

```bash
# God Object Detection
uveddi analyze ./src --detectors god_object
# Expected: Finds large classes with many methods

# Code Duplication
uveddi analyze ./src --detectors code_duplication
# Expected: Finds duplicated code blocks

# Dead Code
uveddi analyze ./src --detectors dead_code
# Expected: Finds unused functions/methods

# Large Class
uveddi analyze ./src --detectors large_class
# Expected: Finds classes exceeding size thresholds

# Long Methods
uveddi analyze ./src --detectors long_methods
# Expected: Finds methods with too many lines

# Tight Coupling
uveddi analyze ./src --detectors tight_coupling
# Expected: Finds high coupling between modules

# Cyclic Dependencies
uveddi analyze ./src --detectors cyclic_dependencies
# Expected: Finds circular dependencies in imports

# Magic Values
uveddi analyze ./src --detectors magic_values
# Expected: Finds hardcoded numbers/strings

# All detectors (default)
uveddi analyze ./src
```

**For each detector verify:**
- [ ] Detector runs without crashing
- [ ] Issues are correctly identified
- [ ] False positive rate is acceptable
- [ ] Results include file location and line numbers
- [ ] Severity levels are appropriate
- [ ] Suggestions for fixes are helpful

---

### Test 3.4: Analysis Options & Flags

```bash
# Exclude paths
uveddi analyze ./src --exclude "tests/*" --exclude "target/*"

# Include patterns
uveddi analyze ./src --include "*.rs"

# Max files limit
uveddi analyze ./src --max-files 100

# Timeout
uveddi analyze ./src --timeout 60

# Verbose output
uveddi analyze ./src --verbose
uveddi analyze ./src -v

# Quiet mode
uveddi analyze ./src --quiet
uveddi analyze ./src -q

# Parallel processing control
uveddi analyze ./src --threads 4
uveddi analyze ./src --threads 1  # Single-threaded

# Incremental analysis
uveddi analyze ./src --incremental
# Modify a file
echo "// test" >> src/main.rs
uveddi analyze ./src --incremental
# Expected: Only re-analyzes changed files
```

**Expected Results:**
- [ ] Exclusion patterns work correctly
- [ ] Inclusion patterns filter properly
- [ ] Max files limit is respected
- [ ] Timeout prevents runaway analysis
- [ ] Verbose mode shows detailed progress
- [ ] Quiet mode suppresses non-essential output
- [ ] Thread count affects performance appropriately
- [ ] Incremental mode detects and skips unchanged files

---

### Test 3.5: Performance Testing

```bash
# Small codebase (baseline)
time uveddi analyze ~/uveddi-testing/small

# Medium codebase
time uveddi analyze ~/uveddi-testing/medium

# Large codebase
time uveddi analyze ~/uveddi-testing/large

# Memory usage during large analysis
/usr/bin/time -v uveddi analyze ~/uveddi-testing/large
```

**Performance benchmarks to record:**
- [ ] Small (<1K LOC): < 5 seconds
- [ ] Medium (~10K LOC): < 30 seconds
- [ ] Large (~50K LOC): < 2 minutes
- [ ] Memory usage stays reasonable (< 500MB for large)
- [ ] CPU usage is reasonable (doesn't peg all cores)
- [ ] No memory leaks over time

---

## Phase 4: Reporting & Output Formats

### Test 4.1: JSON Output Validation
```bash
uveddi analyze ./src --output-format json --output results.json

# Validate JSON structure
cat results.json | jq .
cat results.json | jq '.issues | length'
cat results.json | jq '.summary'
cat results.json | jq '.files_analyzed'

# Verify schema completeness
cat results.json | jq 'keys'
```

**Expected JSON structure:**
```json
{
  "summary": {
    "total_issues": number,
    "files_analyzed": number,
    "analysis_time_ms": number
  },
  "issues": [
    {
      "type": "string",
      "severity": "high|medium|low",
      "file": "path/to/file",
      "line": number,
      "message": "string",
      "suggestion": "string"
    }
  ],
  "detectors_run": ["detector1", "detector2"],
  "metadata": {}
}
```

**Verify:**
- [ ] Valid JSON (parseable by jq)
- [ ] All required fields present
- [ ] Data types are correct
- [ ] Line numbers are accurate
- [ ] File paths are correct

---

### Test 4.2: Markdown Output
```bash
uveddi analyze ./src --output-format markdown --output report.md

# View the report
cat report.md
```

**Expected Markdown structure:**
```markdown
# Uveddi Analysis Report

## Summary
- Files Analyzed: X
- Total Issues: Y
- Analysis Time: Z

## Issues by Severity
### High Severity (N issues)
...

### Medium Severity (N issues)
...

## Detailed Findings
### God Object: ClassName
...
```

**Verify:**
- [ ] Valid Markdown syntax
- [ ] Headings properly formatted
- [ ] Tables render correctly
- [ ] Code blocks are formatted
- [ ] Links work (if any)
- [ ] Human-readable and well-organized

---

### Test 4.3: HTML Output
```bash
uveddi analyze ./src --output-format html --output report.html

# Open in browser
xdg-open report.html  # Linux
open report.html      # macOS
```

**Verify:**
- [ ] Valid HTML5
- [ ] CSS styling applied
- [ ] Responsive layout
- [ ] Syntax highlighting for code
- [ ] Interactive elements work (if any)
- [ ] Navigation works
- [ ] Renders in major browsers

---

## Phase 5: Git Hooks Integration

### Test 5.1: Pre-commit Hook Installation
```bash
cd ~/uveddi-testing/small
git init

# Install pre-commit hook
uveddi hooks install --hook-type pre-commit

# Verify installation
ls -la .git/hooks/pre-commit
cat .git/hooks/pre-commit
```

**Expected Results:**
- [ ] Hook file created at `.git/hooks/pre-commit`
- [ ] Hook is executable (`chmod +x`)
- [ ] Hook contains uveddi analysis command
- [ ] Hook has proper shebang

---

### Test 5.2: Pre-commit Hook Execution
```bash
# Create a file with issues
cat > bad_code.rs << 'EOF'
fn god_object() {
    // 100 lines of code with multiple responsibilities
    let x = 42;  // magic value
    let y = 100; // magic value
    // ... many more lines
}
EOF

git add bad_code.rs
git commit -m "test commit"
# Expected: Hook should run and potentially block commit
```

**Expected Results:**
- [ ] Hook runs automatically on commit
- [ ] Analysis results are displayed
- [ ] Commit blocked if issues exceed threshold
- [ ] Commit proceeds if issues are acceptable
- [ ] Clear messaging about why commit blocked/allowed

---

### Test 5.3: Hook Management
```bash
# List installed hooks
uveddi hooks list

# Uninstall hook
uveddi hooks uninstall --hook-type pre-commit

# Verify uninstallation
ls .git/hooks/pre-commit
# Expected: File removed or no longer executable
```

**Expected Results:**
- [ ] List shows all installed hooks
- [ ] Uninstall removes hook correctly
- [ ] Re-install works after uninstall

---

## Phase 6: Doctor Command (Health Diagnostics)

### Test 6.1: Basic Health Check
```bash
# Run doctor without arguments
uveddi doctor

# Run with specific checks
uveddi doctor --check dependencies
uveddi doctor --check configuration
uveddi doctor --check tree-sitter
```

**Expected Results:**
- [ ] Shows system information
- [ ] Checks Rust installation
- [ ] Verifies tree-sitter grammars installed
- [ ] Validates configuration file
- [ ] Reports dependency status
- [ ] Color-coded output (green/yellow/red)

---

### Test 6.2: Auto-fix Capabilities
```bash
# Create broken config
echo "bad_toml = [" > uveddi.toml

# Run doctor with --fix
uveddi doctor --fix

# Verify fix
cat uveddi.toml
```

**Expected Results:**
- [ ] Identifies fixable issues
- [ ] Offers to fix problems
- [ ] Successfully repairs common issues
- [ ] Reports what was fixed
- [ ] Doesn't break working configurations

---

## Phase 7: CI/CD Integration

### Test 7.1: CI Command Basic Usage
```bash
# Run in CI mode
uveddi ci ./src

# Exit code should be non-zero if issues found
uveddi ci ./src
echo $?
```

**Expected Results:**
- [ ] Runs analysis in CI-friendly mode
- [ ] No interactive prompts
- [ ] Exit code 0 for success (no critical issues)
- [ ] Exit code non-zero for failures
- [ ] Output suitable for CI logs

---

### Test 7.2: CI Thresholds & Failure Conditions
```bash
# Set failure threshold
uveddi ci ./src --fail-on high
uveddi ci ./src --fail-on medium
uveddi ci ./src --max-issues 10

# Generate CI reports
uveddi ci ./src --output-format json --output ci-report.json
```

**Expected Results:**
- [ ] Respects failure threshold settings
- [ ] Counts issues correctly
- [ ] Fails build appropriately
- [ ] CI report format is machine-readable
- [ ] Summary statistics are accurate

---

## Phase 8: Edge Cases & Error Handling

### Test 8.1: Invalid Inputs
```bash
# Non-existent path
uveddi analyze /path/that/does/not/exist
# Expected: Clear error message

# Permission denied
mkdir no_access
chmod 000 no_access
uveddi analyze no_access
# Expected: Permission error, graceful handling

# Empty directory
mkdir empty
uveddi analyze empty
# Expected: Reports no files analyzed

# Corrupted source files
echo "fn invalid { { { " > broken.rs
uveddi analyze broken.rs
# Expected: Parse error reported, doesn't crash
```

**Verify:**
- [ ] Errors are user-friendly
- [ ] No panics or stack traces shown to user
- [ ] Suggestions for fixing issues provided
- [ ] Application exits gracefully

---

### Test 8.2: Resource Limits
```bash
# Very large file
dd if=/dev/zero of=huge.rs bs=1M count=100
uveddi analyze huge.rs
# Expected: Handles large files or skips with warning

# Very deep directory structure
mkdir -p a/b/c/d/e/f/g/h/i/j/k/l/m/n/o
uveddi analyze a
# Expected: Handles deep recursion

# Files with special characters in names
touch "file with spaces.rs"
touch "file'with'quotes.rs"
uveddi analyze .
# Expected: Processes files with special names
```

**Verify:**
- [ ] Large files handled appropriately
- [ ] Deep directories don't cause stack overflow
- [ ] Special characters in filenames work
- [ ] Memory doesn't grow unbounded

---

### Test 8.3: Interrupt Handling
```bash
# Start long-running analysis and interrupt
uveddi analyze ~/uveddi-testing/large &
PID=$!
sleep 5
kill -SIGINT $PID

# Verify graceful shutdown
```

**Expected Results:**
- [ ] Catches SIGINT (Ctrl+C)
- [ ] Cleans up resources
- [ ] Shows partial results if available
- [ ] Doesn't corrupt cache or output files

---

## Phase 9: Cache System

### Test 9.1: AST Cache Functionality
```bash
# First run (cold cache)
time uveddi analyze ./src

# Second run (warm cache)
time uveddi analyze ./src
# Expected: Second run significantly faster

# Modify a file
echo "// change" >> src/main.rs

# Third run (partial cache)
time uveddi analyze ./src
# Expected: Only re-parses modified file
```

**Expected Results:**
- [ ] Cache speeds up subsequent analyses
- [ ] Cache invalidation works for modified files
- [ ] Cache doesn't use excessive disk space
- [ ] Cache location is configurable

---

### Test 9.2: Cache Management
```bash
# Check cache location
ls ~/.cache/uveddi/  # or configured location

# Clear cache
uveddi config set cache.enabled false
uveddi analyze ./src
# Expected: Doesn't use cache

# Re-enable cache
uveddi config set cache.enabled true
```

**Verify:**
- [ ] Cache directory created automatically
- [ ] Cache can be disabled
- [ ] Cache persists across runs
- [ ] Old cache entries are cleaned up

---

## Phase 10: Multi-language Support

### Test 10.1: Rust Projects
```bash
# Analyze Rust crate
uveddi analyze /path/to/rust/crate

# Workspace analysis
uveddi analyze /path/to/cargo/workspace
```

**Expected Results:**
- [ ] Detects `Cargo.toml`
- [ ] Analyzes all crates in workspace
- [ ] Handles `mod.rs` and file-based modules
- [ ] Understands Rust-specific patterns

---

### Test 10.2: Python Projects
```bash
# Analyze Python project
uveddi analyze /path/to/python/project

# With virtual environment
uveddi analyze ./venv --exclude venv
```

**Expected Results:**
- [ ] Parses Python 3.x syntax
- [ ] Detects Python-specific anti-patterns
- [ ] Skips `__pycache__` automatically
- [ ] Handles `.py` files in subdirectories

---

### Test 10.3: JavaScript/TypeScript Projects
```bash
# JavaScript
uveddi analyze /path/to/js/project

# TypeScript
uveddi analyze /path/to/ts/project

# Mixed JS/TS
uveddi analyze /path/to/mixed/project
```

**Expected Results:**
- [ ] Parses modern JavaScript (ES6+)
- [ ] Handles TypeScript syntax
- [ ] Detects JS-specific patterns
- [ ] Works with JSX/TSX files

---

## Phase 11: Stress Testing

### Test 11.1: Large Codebase Analysis
```bash
# Clone and analyze a large open-source project
git clone https://github.com/rust-lang/rust.git
uveddi analyze rust/compiler/rustc --max-files 1000
```

**Monitor:**
- [ ] Memory usage (should stay under 1GB)
- [ ] CPU usage (reasonable core utilization)
- [ ] Completion time (under 5 minutes for 1000 files)
- [ ] No crashes or hangs

---

### Test 11.2: Concurrent Analyses
```bash
# Run multiple analyses in parallel
uveddi analyze ./project1 &
uveddi analyze ./project2 &
uveddi analyze ./project3 &
wait
```

**Verify:**
- [ ] No race conditions
- [ ] No cache corruption
- [ ] Each analysis completes successfully
- [ ] Results are accurate

---

## Phase 12: Documentation & User Experience

### Test 12.1: Error Messages
Intentionally trigger errors and evaluate messages:

```bash
# Missing required argument
uveddi analyze
# Expected: Clear message about missing path

# Invalid flag
uveddi analyze ./src --invalid-flag
# Expected: Suggestion for correct flags

# Type mismatch
uveddi config set analysis.max_files "not_a_number"
# Expected: Type validation error with example
```

**Rate each error message:**
- [ ] Clearly explains what went wrong
- [ ] Provides actionable solution
- [ ] Shows correct syntax/example
- [ ] Appropriate tone (helpful, not condescending)

---

### Test 12.2: Progress Indicators
```bash
# Long-running analysis
uveddi analyze ~/uveddi-testing/large --verbose
```

**Verify:**
- [ ] Progress bar or indicator shown
- [ ] Percentage completion displayed
- [ ] Current file being analyzed shown
- [ ] Estimated time remaining (optional)
- [ ] Updates don't flicker or spam terminal

---

## Test Results Template

Use this template to record results for each test:

```markdown
## Test: [Test Number and Name]
**Date:** YYYY-MM-DD
**Tester:** [Your Name]
**Build:** uveddi 1.0.0-alpha (minimal features)

### Setup
- OS: 
- Rust version:
- Test codebase: [describe]

### Results
- [ ] PASS / [ ] FAIL / [ ] PARTIAL

### Issues Found
1. [Description of issue]
   - Severity: Critical / High / Medium / Low
   - Steps to reproduce:
   - Expected behavior:
   - Actual behavior:
   - Error messages/logs:

### Performance Metrics
- Analysis time: 
- Memory usage:
- CPU usage:
- Files processed:

### Notes
[Any additional observations]
```

## Critical Issues Checklist

Mark as blockers for release:

### Functionality
- [ ] Application crashes or panics
- [ ] Data corruption or data loss
- [ ] Incorrect analysis results
- [ ] Security vulnerabilities
- [ ] Cannot analyze common codebases

### Performance
- [ ] Analysis takes >10x longer than expected
- [ ] Memory leaks or unbounded memory growth
- [ ] Hangs or infinite loops

### User Experience
- [ ] Confusing or missing error messages
- [ ] Loss of work (no save/recovery)
- [ ] Inaccessible features (broken commands)

### Compatibility
- [ ] Doesn't work on Linux
- [ ] Doesn't work on macOS
- [ ] Requires unavailable dependencies

## Success Criteria

Alpha testing is successful if:

1. **Core Functionality (100% required)**
   - [ ] All analysis commands work
   - [ ] All detectors run without crashing
   - [ ] All output formats generate valid files
   - [ ] Configuration system works

2. **Stability (90% required)**
   - [ ] No crashes on valid inputs
   - [ ] Graceful error handling on invalid inputs
   - [ ] Resource usage stays reasonable

3. **Performance (80% acceptable)**
   - [ ] Small codebases: < 10 seconds
   - [ ] Medium codebases: < 1 minute
   - [ ] Large codebases: < 5 minutes
   - [ ] Memory: < 500MB for typical use

4. **User Experience (80% acceptable)**
   - [ ] Error messages are helpful
   - [ ] Documentation is adequate
   - [ ] CLI interface is intuitive
   - [ ] Progress feedback is clear

## Reporting Issues

### Issue Template
```markdown
**Title:** [Brief description]

**Type:** Bug / Enhancement / Question

**Severity:** Critical / High / Medium / Low

**Environment:**
- OS: [Linux/macOS/Windows + version]
- Uveddi version: 1.0.0-alpha
- Rust version: [rustc --version]

**Steps to Reproduce:**
1. 
2. 
3. 

**Expected Behavior:**
[What should happen]

**Actual Behavior:**
[What actually happens]

**Error Messages/Logs:**
```
[Paste logs here]
```

**Sample Code/Project:**
[Link to or paste minimal reproducible example]

**Workaround:**
[If known]
```

### Where to Report
- GitHub Issues: https://github.com/botzrDev/uveddi/issues
- Include `[ALPHA-TEST]` prefix in issue title
- Tag with appropriate labels

## Post-Testing Cleanup

After completing all tests:

```bash
# Clean up test artifacts
rm -rf ~/uveddi-testing

# Remove any test configurations
rm -f uveddi.toml

# Clear cache
rm -rf ~/.cache/uveddi

# Uninstall git hooks
cd ~/.git/hooks && rm pre-commit post-commit
```

## Final Alpha Testing Report

Complete this summary after all testing:

```markdown
# Uveddi Alpha Testing Summary

## Test Coverage
- Total tests attempted: X
- Tests passed: Y
- Tests failed: Z
- Tests partially passed: W

## Critical Issues Found
[List P0/P1 issues that block release]

## Recommendations
- [ ] Ready for beta release
- [ ] Needs fixes before beta (list critical items)
- [ ] Needs major rework

## Overall Assessment
[2-3 paragraphs summarizing findings, strengths, and weaknesses]

## Testing Team
- Lead Tester:
- Contributors:
- Date: YYYY-MM-DD
```

---

**Thank you for alpha testing Uveddi! Your feedback is crucial for making this tool production-ready.**
