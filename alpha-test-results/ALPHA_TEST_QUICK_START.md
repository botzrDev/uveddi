# Uveddi Alpha Test - Quick Start (30 Minutes)

## Setup (5 min)
```bash
# Build
cargo build --release --features minimal

# Add to PATH
export PATH="$PWD/target/release:$PATH"

# Verify
uveddi --version
```

## Essential Tests (25 min)

### 1. Basic Analysis (5 min)
```bash
# Analyze this project
uveddi analyze ./src --output-format json --output test-results.json

# Check results
cat test-results.json | jq '.summary'
```
✅ **Check:** Analysis completes, JSON is valid, issues are found

---

### 2. All Output Formats (5 min)
```bash
uveddi analyze ./src --output-format markdown --output report.md
uveddi analyze ./src --output-format html --output report.html
xdg-open report.html
```
✅ **Check:** All formats generate valid files, HTML renders properly

---

### 3. Configuration (3 min)
```bash
uveddi init
cat uveddi.toml
uveddi config show
uveddi config set analysis.max_files 500
uveddi config get analysis.max_files
```
✅ **Check:** Config created, values can be read/written

---

### 4. Detectors (5 min)
```bash
# Test each detector
uveddi analyze ./src --detectors god_object
uveddi analyze ./src --detectors code_duplication
uveddi analyze ./src --detectors dead_code
uveddi analyze ./src --detectors long_methods
```
✅ **Check:** Each detector runs without errors, finds relevant issues

---

### 5. Doctor Command (2 min)
```bash
uveddi doctor
uveddi doctor --check dependencies
```
✅ **Check:** Shows system info, validates installation

---

### 6. Git Hooks (3 min)
```bash
git init /tmp/test-repo
cd /tmp/test-repo
uveddi hooks install --hook-type pre-commit
cat .git/hooks/pre-commit
```
✅ **Check:** Hook installed and executable

---

### 7. Error Handling (2 min)
```bash
# Test error cases
uveddi analyze /nonexistent/path
uveddi analyze --invalid-flag
uveddi config set bad.key value
```
✅ **Check:** Errors are clear and helpful, no crashes

---

## Report Critical Issues

If you find:
- ❌ **Crashes/Panics** → Report immediately as P0
- ❌ **Incorrect Results** → Report as P1
- ⚠️ **Slow Performance** → Note in report
- ⚠️ **Confusing Errors** → Note in report

## Quick Issue Template
```
**Test:** [Test name]
**Issue:** [What went wrong]
**Expected:** [What should happen]
**Logs:** [Error message]
```

## Success = All ✅ Checked

Your quick test is successful if:
- All 7 essential tests pass
- No crashes or panics
- Output files are valid
- Commands work as expected
