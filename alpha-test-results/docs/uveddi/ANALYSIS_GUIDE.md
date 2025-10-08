# General Code Analysis Guide

## Overview

This guide covers general code analysis principles and best practices that apply across all programming languages and project types.

## Universal Analysis Areas

### 1. Dead Code Detection

Dead code appears in all languages and includes:

- **Unused functions, methods, and variables**
- **Unreachable code** (code after returns, breaks, or exceptions)
- **Commented-out code** blocks
- **Obsolete conditional branches**
- **Legacy compatibility code**

**Universal Configuration:**
```toml
[dead_code]
confidence_threshold = 0.75  # Balanced threshold
library_mode = false         # Adjust based on project type
ignore_patterns = [
    "test/**",
    "tests/**",
    "spec/**",
    "docs/**",
    "examples/**",
    "benchmarks/**",
    "scripts/**"
]
```

### 2. Large Classes/Modules Detection

Regardless of language, watch for:
- **God objects** - Classes/modules doing too much
- **Long functions** - Functions with too many lines
- **Complex parameter lists** - Too many parameters
- **Deep nesting** - Too many indentation levels

**Universal Thresholds:**
- Max logical LOC: 300
- Max methods/functions: 30
- Max parameters: 8
- Max nesting depth: 4

### 3. Architectural Anti-Patterns

**Common Cross-Language Anti-Patterns:**
- **Circular dependencies** between modules
- **Tight coupling** between components
- **Magic numbers and strings** throughout code
- **Feature envy** - using other classes more than own
- **Data clumps** - groups of data always used together
- **Shotgun surgery** - changes require editing many files

## Code Quality Principles

### 1. Single Responsibility Principle
Every module, class, or function should have one reason to change.

```
// Bad: Multiple responsibilities
function processUserData(userData) {
    // Validation
    // Database operations
    // Email sending
    // Logging
    // Report generation
}

// Good: Single responsibility
function validateUserData(userData) { /* ... */ }
function saveUser(userData) { /* ... */ }
function sendWelcomeEmail(user) { /* ... */ }
```

### 2. Don't Repeat Yourself (DRY)
Avoid code duplication by extracting common functionality.

```
// Bad: Repeated logic
function calculateTaxForOrder(order) {
    return order.total * 0.08;
}

function calculateTaxForInvoice(invoice) {
    return invoice.amount * 0.08;
}

// Good: Extracted common logic
const TAX_RATE = 0.08;
function calculateTax(amount) {
    return amount * TAX_RATE;
}
```

### 3. Keep It Simple, Stupid (KISS)
Prefer simple solutions over complex ones.

```
// Bad: Overly complex
function isEven(number) {
    return number % 2 === 0 ? true : false;
}

// Good: Simple and clear
function isEven(number) {
    return number % 2 === 0;
}
```

### 4. You Aren't Gonna Need It (YAGNI)
Don't implement functionality until you need it.

## Universal Best Practices

### Code Organization
```
project/
├── src/                    # Source code
│   ├── core/              # Core business logic
│   ├── utils/             # Utility functions
│   ├── services/          # External service integrations
│   └── models/            # Data models
├── tests/                 # Test files
├── docs/                  # Documentation
├── scripts/               # Build and utility scripts
└── config/                # Configuration files
```

### Naming Conventions
- **Use descriptive names** for variables, functions, and classes
- **Be consistent** with naming patterns within your codebase
- **Avoid abbreviations** unless they're widely understood
- **Use searchable names** for important concepts

```
// Bad: Unclear names
function calc(u, o) {
    return u.reduce((t, i) => t + i.p * i.q, 0) + o;
}

// Good: Descriptive names
function calculateOrderTotal(items, shippingCost) {
    const itemsTotal = items.reduce((total, item) => {
        return total + item.price * item.quantity;
    }, 0);
    return itemsTotal + shippingCost;
}
```

### Error Handling
- **Fail fast** - Detect and report errors early
- **Use specific exceptions** rather than generic ones
- **Log errors appropriately** for debugging
- **Clean up resources** in finally blocks or equivalents

### Documentation
- **Code should be self-documenting** through good naming
- **Add comments for complex business logic**
- **Document public APIs** and interfaces
- **Keep documentation up to date** with code changes

## Analysis Workflow

### 1. Initial Assessment
```bash
# Run basic analysis to get overview
uveddi analyze src/

# Check system health first
uveddi doctor

# Validate configuration
uveddi config validate
```

### 2. Focused Analysis
```bash
# Focus on specific anti-patterns
uveddi analyze src/ --detectors god-object,dead-code

# Analyze architectural issues
uveddi analyze src/ --detectors circular-dependencies,tight-coupling

# Generate detailed report
uveddi analyze src/ --output-format html --enable-ai
```

### 3. Iterative Improvement
```bash
# Track changes over time
uveddi analyze src/ --baseline previous-analysis.json

# Focus on changed files only
uveddi analyze src/ --changed-files

# Set quality gates
uveddi analyze src/ --fail-on critical --max-issues 10
```

## Common Refactoring Patterns

### Extract Method
```
// Before: Long function
function processOrder(order) {
    // 50 lines of validation
    // 100 lines of calculation
    // 30 lines of database operations
    // 20 lines of email sending
}

// After: Extracted methods
function processOrder(order) {
    validateOrder(order);
    const total = calculateOrderTotal(order);
    saveOrder(order, total);
    sendOrderConfirmation(order);
}
```

### Extract Class
```
// Before: God class with multiple responsibilities
class OrderManager {
    validateOrder() { /* ... */ }
    calculateTotal() { /* ... */ }
    saveToDatabase() { /* ... */ }
    sendEmail() { /* ... */ }
    generateReport() { /* ... */ }
    auditLog() { /* ... */ }
}

// After: Separated responsibilities
class OrderValidator { /* ... */ }
class OrderCalculator { /* ... */ }
class OrderRepository { /* ... */ }
class EmailService { /* ... */ }
class ReportGenerator { /* ... */ }
class AuditLogger { /* ... */ }
```

### Replace Magic Numbers
```
// Before: Magic numbers
if (user.age > 18 && user.score > 750) {
    approveApplication(user);
}

// After: Named constants
const MINIMUM_AGE = 18;
const MINIMUM_CREDIT_SCORE = 750;

if (user.age > MINIMUM_AGE && user.score > MINIMUM_CREDIT_SCORE) {
    approveApplication(user);
}
```

## Metrics and Thresholds

### Code Complexity Metrics
- **Cyclomatic Complexity**: < 10 per function
- **Cognitive Complexity**: < 15 per function
- **Lines of Code**: < 50 per function, < 500 per class
- **Parameter Count**: < 5 per function
- **Nesting Depth**: < 4 levels

### Coupling and Cohesion
- **Afferent Coupling (Ca)**: Incoming dependencies
- **Efferent Coupling (Ce)**: Outgoing dependencies
- **Instability (I)**: Ce / (Ca + Ce)
- **LCOM (Lack of Cohesion)**: < 0.5 for good cohesion

### Quality Gates
```toml
[quality_gates]
max_cyclomatic_complexity = 10
max_cognitive_complexity = 15
max_function_lines = 50
max_class_lines = 500
max_parameters = 5
max_nesting_depth = 4
min_test_coverage = 80
max_critical_issues = 0
max_major_issues = 5
```

## Language-Agnostic Tools Integration

### Version Control Integration
```bash
# Git hooks for automated analysis
uveddi init --git-hooks

# Pre-commit analysis
git add . && uveddi analyze --changed-files

# Continuous integration
uveddi analyze src/ --output-format junit --output results.xml
```

### IDE Integration
Most IDEs can integrate with Uveddi through:
- **Language Server Protocol (LSP)** extensions
- **Command palette** integration
- **Problem matcher** configurations
- **Task runner** integration

### CI/CD Integration
```yaml
# Generic CI/CD pipeline
- name: Code Analysis
  run: |
    uveddi analyze src/ --output-format json --output analysis.json
    uveddi analyze src/ --quality-gates --fail-on critical
```

## Troubleshooting Common Issues

### Performance Problems
```bash
# Large codebases
uveddi analyze src/ --parallel --memory-limit 4GB

# Selective analysis
uveddi analyze src/core/ --exclude-patterns "tests/**"

# Incremental analysis
uveddi analyze src/ --incremental --cache-dir .uveddi-cache
```

### Configuration Issues
```bash
# Validate configuration
uveddi config validate

# Reset to defaults
uveddi config reset

# Debug configuration loading
uveddi analyze src/ --verbose --debug-config
```

### Analysis Accuracy
```bash
# Adjust confidence thresholds
uveddi analyze src/ --confidence-threshold 0.8

# Include/exclude specific patterns
uveddi analyze src/ --include-patterns "src/**" --exclude-patterns "generated/**"

# Language-specific settings
uveddi analyze src/ --language rust --strict-mode
```

## Quality Improvement Process

### 1. Assessment Phase
- Run comprehensive analysis
- Identify critical issues first
- Prioritize by impact and effort
- Set realistic improvement goals

### 2. Planning Phase
- Create refactoring roadmap
- Estimate time and resources
- Plan incremental improvements
- Set up monitoring and metrics

### 3. Implementation Phase
- Start with highest-impact issues
- Refactor incrementally
- Maintain test coverage
- Monitor quality metrics

### 4. Validation Phase
- Re-run analysis after changes
- Verify improvements in metrics
- Update documentation
- Share results with team

For language-specific guides, see the individual analysis guides for Rust, Python, JavaScript, and TypeScript.