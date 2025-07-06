# Large Classes Detection

The Large Classes Detection feature in Uveddi identifies classes, structs, and objects that have grown too large and violate the Single Responsibility Principle. This includes detection of God Objects, Blob anti-patterns, and other large class code smells across multiple programming languages.

## Features

- **Multi-language Support**: Rust, Python, and JavaScript
- **Multi-metric Analysis**: Size, complexity, and structural metrics
- **Severity Scoring**: Intelligent scoring from Info to Critical levels
- **Language-specific Thresholds**: Optimized defaults for each language
- **Configurable Detection**: Customize thresholds and ignore patterns
- **Detailed Reporting**: Comprehensive metrics breakdown and refactoring suggestions

## Detection Strategy

The detector uses a sophisticated three-pillar analysis framework:

### 1. Size Metrics (40% weight)
- **Logical Lines of Code (LLOC)**: Excludes comments and blank lines
- **Number of Methods (NOM)**: Count of functions/methods in the class
- **Number of Fields (NOF)**: Count of attributes/properties

### 2. Complexity Metrics (35% weight)
- **Cyclomatic Complexity (CC)**: Measures decision points and control flow
- **Cognitive Complexity**: Human-readable complexity assessment

### 3. Structural Metrics (25% weight)
- **Lack of Cohesion in Methods (LCOM)**: Measures how well methods work together
- **Coupling Between Objects (CBO)**: Measures external dependencies

## Severity Scoring

Issues are scored from 0-100 and categorized as:

- **Info (0-25)**: Slightly above thresholds, minor concern
- **Low (26-50)**: Moderate size, should be monitored
- **Medium (51-75)**: Clear anti-pattern, refactoring recommended
- **High (76-90)**: Significant design issues, refactoring needed
- **Critical (91-100)**: God Object, immediate attention required

## Configuration

### CLI Options

```bash
# Basic analysis with large classes detection
uveddi analyze ./src

# Custom thresholds
uveddi analyze ./src --large-classes-max-loc 500 --large-classes-max-methods 15

# Adjust severity threshold
uveddi analyze ./src --large-classes-min-severity 50

# Ignore specific patterns
uveddi analyze ./src --large-classes-ignore-patterns "test,spec,mock,generated"

# Complex configuration example
uveddi analyze ./src \
  --large-classes-max-loc 600 \
  --large-classes-max-methods 25 \
  --large-classes-max-fields 10 \
  --large-classes-max-complexity 40 \
  --large-classes-max-lcom 0.7 \
  --large-classes-min-severity 30
```

### Environment Variables

```bash
# Set maximum lines of code threshold
export LARGE_CLASSES_MAX_LOC=500

# Set maximum methods threshold
export LARGE_CLASSES_MAX_METHODS=15

# Set maximum fields threshold
export LARGE_CLASSES_MAX_FIELDS=8

# Set maximum complexity threshold
export LARGE_CLASSES_MAX_COMPLEXITY=40

# Set maximum cognitive complexity threshold
export LARGE_CLASSES_MAX_COGNITIVE_COMPLEXITY=35

# Set maximum LCOM score threshold
export LARGE_CLASSES_MAX_LCOM=0.7

# Set maximum coupling threshold
export LARGE_CLASSES_MAX_COUPLING=10

# Set ignore patterns (comma-separated)
export LARGE_CLASSES_IGNORE_PATTERNS="test,spec,mock,generated"
```

### Configuration File

Add to your `config.toml`:

```toml
[large_classes]
max_logical_loc = 500
max_methods = 15
max_fields = 8
max_cyclomatic_complexity = 40
max_cognitive_complexity = 35
max_lcom_score = 0.7
max_coupling = 10
ignore_patterns = ["test", "spec", "mock", "generated", "fixture"]

# Language-specific overrides
[large_classes.language_overrides.rust]
max_logical_loc = 400
max_methods = 20
max_fields = 15

[large_classes.language_overrides.python]
max_logical_loc = 1000
max_methods = 20
max_fields = 7

[large_classes.language_overrides.javascript]
max_logical_loc = 800
max_methods = 25
max_fields = 12
```

## Language-Specific Behavior

### Rust
- **Conservative thresholds** due to systems programming nature
- **Default limits**: 400 LOC, 20 methods, 15 fields
- **Struct + impl analysis**: Combines struct definition with implementation blocks
- **Visibility detection**: Considers `pub` keyword for export analysis

### Python
- **Standard OOP thresholds** based on Pylint defaults
- **Default limits**: 1000 LOC, 20 methods, 7 fields
- **Class-based analysis**: Analyzes class definitions and methods
- **Private detection**: Treats symbols starting with `_` as private

### JavaScript
- **Framework-aware thresholds** for React/Node.js patterns
- **Default limits**: 800 LOC, 25 methods, 12 fields
- **Class and prototype analysis**: Supports modern class syntax
- **Export detection**: Considers `export` and `module.exports`

## Example Output

```
Found 2 large class issues in src/models/user.rs:

High Severity - Lines 15-89:
Large class detected: 'UserManager' has grown beyond recommended thresholds (severity: 78%)

Metrics breakdown:
• Lines of code: 520 (threshold: 400)
• Methods: 28 (threshold: 20)
• Fields: 18 (threshold: 15)
• Cyclomatic complexity: 65 (threshold: 50)
• LCOM score: 0.85 (threshold: 0.80)
• Coupling: 15 (threshold: 12)

Suggested refactoring:
• Extract Class: Low cohesion suggests multiple responsibilities
• Extract Superclass: Consider inheritance hierarchy
• Dependency Injection: Reduce tight coupling

Code snippet:
pub struct UserManager {
    users: HashMap<UserId, User>,
    sessions: HashMap<SessionId, Session>,
    permissions: PermissionManager,
    ...
```

## Refactoring Suggestions

Based on the primary violations detected, the system provides targeted suggestions:

### Extract Class Pattern
- **When**: High LCOM score indicates low cohesion
- **Action**: Split class into multiple focused classes
- **Example**: Separate user management from session management

### Extract Superclass Pattern
- **When**: Too many methods suggest multiple concerns
- **Action**: Create inheritance hierarchy
- **Example**: Base class with specialized subclasses

### Dependency Injection Pattern
- **When**: High coupling count
- **Action**: Inject dependencies rather than creating them
- **Example**: Pass services through constructor

### Facade Pattern
- **When**: High complexity with many external interactions
- **Action**: Create simplified interface
- **Example**: Wrapper class that delegates to specialized services

## Best Practices

1. **Start with High Severity**: Begin with threshold 75+ to focus on critical issues
2. **Language-appropriate Limits**: Use different thresholds per language
3. **Ignore Test Code**: Exclude test files which naturally have different patterns
4. **Iterative Refactoring**: Address issues incrementally
5. **Monitor Trends**: Track metrics over time to prevent regression

## Integration with Build Systems

### CI/CD Pipeline
```yaml
# GitHub Actions example
- name: Run Large Classes Analysis
  run: |
    uveddi analyze ./src \
      --large-classes-min-severity 50 \
      --output-format json \
      --output large-classes-report.json
    
    # Fail build if critical issues found
    if grep -q '"severity": "Critical"' large-classes-report.json; then
      echo "Critical large class issues found!"
      exit 1
    fi
```

### Pre-commit Hook
```bash
#!/bin/bash
# Check for new large classes before commit
uveddi analyze ./src --large-classes-min-severity 75 --output-format json > /tmp/large-classes.json

if [ -s /tmp/large-classes.json ]; then
  echo "Large class issues detected. Please refactor before committing."
  cat /tmp/large-classes.json
  exit 1
fi
```

## Limitations

- **Single-file Analysis**: Current implementation analyzes files individually
- **Simplified LCOM**: Uses approximated cohesion calculation
- **Language Coverage**: Some language-specific patterns may not be detected
- **Dynamic Features**: Cannot analyze runtime-generated classes

## Future Enhancements

- Cross-file inheritance analysis
- More sophisticated LCOM algorithms
- Support for additional languages (C++, Java, Go)
- Integration with IDE plugins
- Historical trend analysis
- Automated refactoring suggestions

## Troubleshooting

### Common Issues

**Q: Why are my test classes being flagged?**
A: Add test patterns to ignore list: `--large-classes-ignore-patterns "test,spec"`

**Q: The thresholds seem too strict for my codebase**
A: Adjust thresholds based on your team's standards and gradually lower them

**Q: Some generated code is being flagged**
A: Add "generated" to ignore patterns and ensure generated files are properly marked

**Q: LCOM scores seem inaccurate**
A: Current implementation is simplified; consider this as a rough indicator rather than precise measurement