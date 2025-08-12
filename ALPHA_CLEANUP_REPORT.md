# Uveddi Alpha Release Cleanup Report

**Generated:** $(date)  
**Jira Context:** UV-243 - Alpha Release Preparation  
**Purpose:** Identify files and directories to remove before alpha release

## 🎯 Executive Summary

This report identifies **critical files and directories that must be removed** before the Uveddi alpha release to ensure:
- **Security**: No sensitive development data exposed
- **Size Optimization**: Reduced distribution size (currently ~5.5GB)
- **Cleanliness**: Professional alpha release package
- **Performance**: Faster installation and startup

**Total Space to Reclaim:** ~5.6GB (95% reduction in repository size)

---

## 🚨 CRITICAL REMOVALS (Security & Size)

### 1. Build Artifacts - **5.4GB** 🔥
```bash
target/                           # Entire Rust build directory
├── debug/                        # Debug builds (not needed in release)
├── release/                      # Will be rebuilt for final alpha
├── doc/                          # Generated documentation
├── tmp/                          # Temporary build files
└── .rustc_info.json             # Compiler metadata
```
**Action:** `cargo clean` before packaging

### 2. Development Prompts - **~2MB** 🔥
```bash
dev_prompts/                      # Internal development instructions
├── PROMPT_001_SECURITY_FIXES.md
├── PROMPT_002_ARCHITECTURE_REFACTORING.md
├── PROMPT_003_CIRCULAR_DEPENDENCIES.md
├── PROMPT_004_TEST_STABILITY.md
├── PROMPT_005_DEPENDENCY_MANAGEMENT.md
└── PROMPT_MASTER_COORDINATION.md
```
**Reason:** Contains internal development strategy and sensitive architectural details

### 3. Test Reports & Artifacts - **149MB** 🔥
```bash
reports/                          # Generated test reports
├── reports/                      # Nested duplicate reports
├── cache/                        # Report generation cache
├── database_fix_test.html
├── debug_report.html
├── diagnostic_test.html
├── memory_aware_test.html
├── test_small_report.html
├── test_with_issues.html
├── tokio/                        # External project reports
└── uveddi_cache.db              # Test database artifacts
```

### 4. Development Test HTML Files 🔥
```bash
test_typescript_fix.html
test_typescript_fresh.html
test_typescript_verification.html
flask_ai_analysis_report.html
flask_analysis_report.html
rust_analysis_report.html
typescript_analysis_report.html
```

---

## ⚠️ HIGH PRIORITY REMOVALS

### 5. Test Repositories & Data - **Size varies**
```bash
test_repos/                       # Test codebases for development
test_small/                       # Sample small projects
test_with_issues/                 # Projects with known issues
alpha_testing/                    # Alpha testing artifacts
├── express-typescript/
├── requests/
└── zola/
```

### 6. Cache Directories
```bash
cache/                            # Development cache
├── ast/                          # AST cache files
└── zero_copy/                    # Memory optimization cache
.uveddi_cache/                    # Hidden cache directory
logs/                             # Development logs
└── service.pid                   # Process ID files
```

### 7. Temporary & Backup Files
```bash
src/report/image_renderer.rs.bak  # Backup file
reports/reports/django/tests/i18n/unchanged/locale/de/LC_MESSAGES/django.po.tmp
reports/reports/django/tests/i18n/unchanged/models.py.tmp
```

### 8. Development Scripts & Tools
```bash
simple_cycle_demo                 # Demo artifacts
true/                             # Unknown directory
sample.ts                         # Sample TypeScript file
dependency_analysis_report.txt    # Development analysis
```

---

## 🔧 MEDIUM PRIORITY REMOVALS

### 9. Development Documentation
```bash
INVESTIGATION_PROMPT.md           # Internal investigation guide
EXECUTIVE_ACTION_PLAN.md          # Development planning
SECURITY_FIXES_PLAN.md           # Internal security roadmap
TEST_STABILITY_PLAN.md           # Test improvement plans
ALPHA_READINESS_REPORT.md        # Internal readiness assessment
ALPHA_RELEASE_SETUP.md           # Internal setup guide
```

### 10. Test Infrastructure Files
```bash
tests/cache_performance_test.rs   # Performance testing
tests/memory_optimization_phase4.rs # Memory optimization tests
tests/simple_test_infrastructure.rs # Test utilities
tests/tui_performance.rs         # TUI performance tests
tests/unit/analysis/engine_tests_standalone.rs # Standalone tests
tests/test_*.rs                  # Various test files
```

### 11. Development Configuration
```bash
test_install.sh                  # Installation testing
validate_install.sh              # Validation scripts
zola_performance_test.json       # Performance test data
```

---

## ✅ FILES TO KEEP (Production Ready)

### Core Application
```bash
src/                             # Core source code
Cargo.toml                       # Package manifest
Cargo.lock                       # Dependency lock file (debatable)
build.rs                         # Build script
```

### Documentation
```bash
docs/                            # User documentation
README.md                        # User guide
CHANGELOG.md                     # Version history
CONTRIBUTING.md                  # Contribution guidelines
```

### Configuration
```bash
config/                          # Application configuration
├── benchmark-config.toml
├── deny.toml
├── test_repositories.yaml      # May need review
└── security/
```

### Infrastructure
```bash
.github/                         # CI/CD workflows
k8s/                            # Kubernetes deployment
docker-compose.yml              # Production Docker setup
docker-compose.dev.yml          # Development Docker (review needed)
Dockerfile                      # Container definition
migrations/                     # Database migrations
```

### Assets & Templates
```bash
assets/                         # Web assets
templates/                      # Report templates
wit/                           # WASM interface types
plugins/                       # Plugin system
```

---

## 🚀 RECOMMENDED CLEANUP COMMANDS

### Step 1: Clean Build Artifacts
```bash
cargo clean
rm -rf target/
```

### Step 2: Remove Development Files
```bash
rm -rf dev_prompts/
rm -rf reports/
rm -rf test_repos/
rm -rf test_small/
rm -rf test_with_issues/
rm -rf alpha_testing/
rm -rf cache/
rm -rf .uveddi_cache/
rm -rf logs/
rm -f uveddi_cache.db
```

### Step 3: Remove Test HTML Files
```bash
rm -f test_typescript_*.html
rm -f *_analysis_report.html
rm -f flask_*.html
rm -f rust_analysis_report.html
rm -f typescript_analysis_report.html
```

### Step 4: Remove Temporary Files
```bash
find . -name "*.tmp" -delete
find . -name "*.bak" -delete
find . -name "*.orig" -delete
find . -name "service.pid" -delete
```

### Step 5: Remove Development Documentation
```bash
rm -f INVESTIGATION_PROMPT.md
rm -f EXECUTIVE_ACTION_PLAN.md
rm -f SECURITY_FIXES_PLAN.md
rm -f TEST_STABILITY_PLAN.md
rm -f ALPHA_READINESS_REPORT.md
rm -f ALPHA_RELEASE_SETUP.md
```

---

## 📋 POST-CLEANUP VERIFICATION

1. **Verify core functionality:**
   ```bash
   cargo build --release
   cargo test --release
   ```

2. **Check repository size:**
   ```bash
   du -sh .
   # Should be < 100MB after cleanup
   ```

3. **Verify essential files remain:**
   ```bash
   ls -la src/ docs/ config/
   ```

4. **Test basic operations:**
   ```bash
   ./target/release/uveddi --version
   ./target/release/uveddi analyze --help
   ```

---

## ⚡ AUTOMATION SCRIPT

Create `scripts/alpha-cleanup.sh`:
```bash
#!/bin/bash
# Alpha Release Cleanup Script (UV-243)

set -e

echo "🧹 Starting Uveddi Alpha Cleanup..."

# Clean build artifacts
echo "Cleaning build artifacts..."
cargo clean

# Remove development directories
echo "Removing development files..."
rm -rf dev_prompts/ reports/ test_repos/ test_small/ test_with_issues/
rm -rf alpha_testing/ cache/ .uveddi_cache/ logs/ true/

# Remove test files
echo "Removing test artifacts..."
rm -f test_typescript_*.html *_analysis_report.html
rm -f dependency_analysis_report.txt sample.ts
rm -f test_install.sh validate_install.sh zola_performance_test.json

# Remove temporary files
echo "Removing temporary files..."
find . -name "*.tmp" -delete
find . -name "*.bak" -delete
find . -name "*.orig" -delete
find . -name "service.pid" -delete

# Remove development documentation
echo "Removing development documentation..."
rm -f INVESTIGATION_PROMPT.md EXECUTIVE_ACTION_PLAN.md
rm -f SECURITY_FIXES_PLAN.md TEST_STABILITY_PLAN.md
rm -f ALPHA_READINESS_REPORT.md ALPHA_RELEASE_SETUP.md

echo "✅ Cleanup complete!"
echo "📊 Repository size after cleanup:"
du -sh .
```

---

## 🎯 JIRA INTEGRATION

**Related Issues:**
- UV-243: Alpha release preparation
- UV-81: Build system fixes (referenced in cleanup)
- UV-96: Visualization improvements (may affect report templates)

**Next Steps:**
1. Review this report with team
2. Execute cleanup script
3. Test alpha package
4. Update UV-243 with results

---

**Generated by:** Uveddi Alpha Cleanup Analysis  
**Contact:** Development Team  
**Last Updated:** $(date)
