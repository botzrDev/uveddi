# Assignment 10: Eliminate Unused Feature Flags

## Priority: HIGH
## Estimated Time: 2-3 hours
## Dependencies: Assignment 09 (Feature Flag Audit)

## Objective
Remove unused, redundant, and unnecessary feature flags to reduce complexity.

## Input Required
Results from Assignment 09:
- feature_flags_audit.md
- feature_usage_matrix.csv
- List of unused/redundant features

## Tasks

### 1. Identify Features for Elimination
Based on audit results, categorize features:

#### A. Zero-Usage Features
Features with no conditional compilation usage:
- Document why they exist
- Verify they're truly unused
- Plan safe removal

#### B. Redundant Features
Features that duplicate functionality:
- Identify overlapping features
- Choose canonical feature to keep
- Plan merger/elimination

#### C. Always-Enabled Features
Features that are always used together:
- Consider making them part of default
- Evaluate if separate flag adds value

### 2. Create Elimination Plan
```markdown
# Feature Elimination Plan

## Phase 1: Zero-Usage Features (Safe to Remove)
- feature_name_1: No usage found, originally for deprecated functionality
- feature_name_2: Experimental feature never completed
- feature_name_3: Duplicate of existing functionality

## Phase 2: Redundant Features (Requires Merger)
- old_feature -> new_feature: Merge functionality
- deprecated_api -> modern_api: Update all usage

## Phase 3: Always-Enabled Features (Move to Default)
- core_feature: Used in 98% of builds, should be default
- essential_deps: Required by all other features
```

### 3. Remove Zero-Usage Features

For each unused feature:

#### Step 1: Verify No Hidden Usage
```bash
# Double-check usage across entire codebase
rg "feature.*FEATURE_NAME" . --type rust
rg "cfg.*FEATURE_NAME" . --type rust
grep -r "FEATURE_NAME" docs/ README.md CHANGELOG.md
```

#### Step 2: Remove from Cargo.toml
```toml
# Remove line from [features] section
# Example: Remove unused_feature = ["dep1", "dep2"]
```

#### Step 3: Remove Conditional Dependencies
```toml
# Remove from [dependencies] if only used by removed feature
# Example: Remove optional dependencies that are now unused
```

#### Step 4: Clean Up Conditional Compilation
```bash
# Remove any remaining #[cfg(feature = "unused_feature")] blocks
# Verify no dead code remains
```

### 4. Merge Redundant Features

For features to be merged:

#### Step 1: Choose Canonical Feature
- Pick the better-named feature
- Consider which is more widely used
- Maintain backward compatibility if possible

#### Step 2: Update Cargo.toml
```toml
# Before:
old_feature = ["dependency_a"]
new_feature = ["dependency_b"]

# After:
new_feature = ["dependency_a", "dependency_b"]
# Remove old_feature line
```

#### Step 3: Update Conditional Compilation
```rust
// Replace:
#[cfg(feature = "old_feature")]

// With:
#[cfg(feature = "new_feature")]
```

#### Step 4: Add Compatibility Alias (Temporary)
```toml
# Add temporary alias for backward compatibility
old_feature = ["new_feature"]  # Deprecated: use new_feature
```

### 5. Move Always-Enabled to Default

For features that should be default:

#### Step 1: Update Default Features
```toml
# Before:
default = ["basic"]

# After:
default = ["basic", "always_used_feature"]
```

#### Step 2: Remove Conditional Compilation
```rust
// Remove unnecessary feature gates for now-default features
// Before:
#[cfg(feature = "always_used")]
pub mod always_used;

// After:
pub mod always_used;
```

### 6. Update Documentation

#### Update README.md
- Remove documentation for eliminated features
- Update feature flag examples
- Add deprecation notices for temporary aliases

#### Update CHANGELOG.md
```markdown
## [Unreleased]
### Removed
- Unused feature flags: `unused_feature_1`, `unused_feature_2`
- Redundant feature flags merged: `old_feature` -> `new_feature`

### Changed
- Features now included in default: `always_used_feature`
```

#### Update Cargo.toml comments
```toml
[features]
# Core feature profiles
default = ["standard"]

# Language support - modular selection
rust-support = ["tree-sitter-rust"]
python-support = ["tree-sitter-python"]

# Deprecated features (will be removed in next major version)
old_feature_name = ["new_feature_name"]  # Use new_feature_name instead
```

### 7. Update CI/CD Pipeline

#### Update GitHub Actions
```yaml
# Remove build matrix entries for eliminated features
strategy:
  matrix:
    features:
      # Remove: unused_feature
      # Update: old_feature -> new_feature
      - minimal
      - standard
      - full
```

#### Update Build Scripts
```bash
# Update any scripts that reference removed features
# Update documentation generation scripts
# Update feature testing scripts
```

### 8. Create Migration Guide

```markdown
# Feature Flag Migration Guide

## Removed Features
- `unused_feature`: No longer needed, functionality removed
- `experimental_api`: Experiment completed, not continuing

## Merged Features
- `old_feature` -> `new_feature`: Update your Cargo.toml

## Now Default
- `core_functionality`: No longer needs explicit enabling

## Breaking Changes
- Some feature combinations no longer valid
- Update build scripts and CI/CD configurations
```

### 9. Test All Changes

#### Build Matrix Testing
```bash
# Test remaining feature combinations
cargo check --no-default-features
cargo check --features minimal
cargo check --features standard
cargo check --features full

# Test deprecated aliases still work (temporarily)
cargo check --features old_feature_name
```

#### Integration Testing
```bash
# Run full test suite with different feature combinations
cargo test --no-default-features
cargo test --features minimal
cargo test --all-features
```

### 10. Performance Impact Assessment
- Measure compilation time improvements
- Check binary size changes
- Verify no functionality regressions

## Success Criteria
- [ ] All unused features removed
- [ ] Redundant features merged
- [ ] Default features updated appropriately
- [ ] All tests pass with new configuration
- [ ] CI/CD pipeline updated
- [ ] Documentation updated
- [ ] Migration guide created

## Breaking Changes Documentation
Document all breaking changes:
- Removed features and their impact
- Changed feature names
- New default behavior

## Rollback Plan
- Keep backup of original Cargo.toml
- Document how to restore removed features if needed
- Plan for hotfix if critical issues found

## Verification Commands
```bash
# Count remaining features
grep -c "^[a-zA-Z]" Cargo.toml

# Test build matrix
./scripts/test-all-feature-combinations.sh

# Verify no orphaned conditional compilation
rg "#\[cfg\(feature.*\"(.+?)\"\)" src/ -o | sort | uniq > used_features.txt
grep "^[a-zA-Z-]" Cargo.toml | cut -d'=' -f1 | sort > defined_features.txt
comm -23 used_features.txt defined_features.txt  # Should be empty
```

## Completion Notes
_To be filled by AI developer:_
- Features removed: ___
- Features merged: ___
- Compilation time improvement: ___
- Breaking changes: ___
- Backward compatibility maintained for: ___