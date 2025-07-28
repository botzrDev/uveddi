# Ubiquitous Language Enforcement Process

This document defines the comprehensive process for enforcing consistent use of Uveddi's ubiquitous language across all codebase components, documentation, and team communication.

## Enforcement Strategy Overview

The enforcement strategy operates on four levels:
1. **Automated Validation** - Tools and scripts that detect terminology inconsistencies
2. **Development Guidelines** - Clear rules for developers to follow
3. **Review Process Integration** - Code review checklists and standards
4. **Continuous Monitoring** - Ongoing validation and improvement

## 1. Automated Validation Tools

### 1.1 Terminology Linter (`scripts/terminology-linter.py`)

A custom linting tool that validates terminology usage across the codebase:

```python
#!/usr/bin/env python3
"""
Terminology Linter for Uveddi Ubiquitous Language Enforcement
Validates consistent use of standardized terminology across code and documentation.
"""

import re
import sys
from pathlib import Path
from typing import List, Dict, Tuple

# Standardized terminology rules
TERMINOLOGY_RULES = {
    # Anti-pattern terminology standardization
    "anti_pattern_rules": {
        "code_structures": r"anti_pattern",  # snake_case in code
        "type_names": r"AntiPattern",        # PascalCase in types
        "documentation": r"anti-pattern",    # hyphenated in docs
        "violations": [
            r"antipattern",      # single word (incorrect)
            r"anti\.pattern",    # dot notation (incorrect)
            r"AntiPattern(?!.*(?:Type|Config|Detector))",  # Wrong PascalCase usage
        ]
    },
    
    # Analysis/Analyze consistency (US spelling)
    "analysis_rules": {
        "verb_form": r"analyze",
        "noun_form": r"analysis",
        "agent_form": r"analyzer",
        "violations": [
            r"analyse",    # UK spelling
            r"analyser",   # UK spelling
        ]
    },
    
    # Configuration terminology
    "config_rules": {
        "code_suffix": r"Config",
        "documentation": r"configuration",
        "violations": [
            r"config(?!uration|\.toml|\.rs|\.json)",  # Abbreviated in docs
        ]
    },
    
    # Component type distinctions
    "component_rules": {
        "engines": r"Engine$",      # Core orchestrators
        "services": r"Service$",    # Internal business logic
        "providers": r"Provider$",  # External integrations
        "managers": r"Manager$",    # Resource management
        "builders": r"Builder$",    # Object construction
        "detectors": r"Detector$",  # Issue detection
    },
    
    # Forbidden overloaded terms
    "overload_prevention": {
        "engine_overuse": [
            r"WasmPluginEngine",           # Should be WasmPluginRuntime
            r"IncrementalAnalysisEngine",  # Should be IncrementalAnalyzer
        ]
    }
}

class TerminologyLinter:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.violations = []
        
    def lint_file(self, file_path: Path) -> List[Dict]:
        """Lint a single file for terminology violations."""
        violations = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.splitlines()
                
            # Check each rule category
            violations.extend(self._check_anti_pattern_rules(file_path, content, lines))
            violations.extend(self._check_analysis_rules(file_path, content, lines))
            violations.extend(self._check_config_rules(file_path, content, lines))
            violations.extend(self._check_component_rules(file_path, content, lines))
            violations.extend(self._check_overload_rules(file_path, content, lines))
            
        except Exception as e:
            violations.append({
                'file': str(file_path),
                'line': 0,
                'type': 'error',
                'message': f'Failed to process file: {e}'
            })
            
        return violations
    
    def _check_anti_pattern_rules(self, file_path: Path, content: str, lines: List[str]) -> List[Dict]:
        """Check anti-pattern terminology consistency."""
        violations = []
        rules = TERMINOLOGY_RULES["anti_pattern_rules"]
        
        # Check for violations
        for violation_pattern in rules["violations"]:
            for line_num, line in enumerate(lines, 1):
                matches = re.finditer(violation_pattern, line, re.IGNORECASE)
                for match in matches:
                    violations.append({
                        'file': str(file_path),
                        'line': line_num,
                        'column': match.start(),
                        'type': 'terminology',
                        'category': 'anti-pattern',
                        'message': f'Inconsistent anti-pattern terminology: "{match.group()}"',
                        'suggestion': self._get_anti_pattern_suggestion(match.group(), file_path)
                    })
        
        return violations
    
    def _get_anti_pattern_suggestion(self, violation: str, file_path: Path) -> str:
        """Provide context-appropriate suggestion for anti-pattern terminology."""
        if file_path.suffix in ['.rs', '.py', '.js', '.ts']:
            if violation.lower() in ['antipattern', 'anti.pattern']:
                return 'Use "anti_pattern" in code structures or "AntiPattern" in type names'
        elif file_path.suffix in ['.md', '.txt', '.rst']:
            return 'Use "anti-pattern" (hyphenated) in documentation'
        return 'Use standardized anti-pattern terminology'
    
    def _check_analysis_rules(self, file_path: Path, content: str, lines: List[str]) -> List[Dict]:
        """Check analysis/analyze consistency (US spelling)."""
        violations = []
        rules = TERMINOLOGY_RULES["analysis_rules"]
        
        for violation_pattern in rules["violations"]:
            for line_num, line in enumerate(lines, 1):
                matches = re.finditer(violation_pattern, line, re.IGNORECASE)
                for match in matches:
                    us_form = match.group().replace('se', 'ze')
                    violations.append({
                        'file': str(file_path),
                        'line': line_num,
                        'column': match.start(),
                        'type': 'terminology',
                        'category': 'spelling',
                        'message': f'Use US spelling: "{us_form}" instead of "{match.group()}"',
                        'suggestion': f'Replace with "{us_form}"'
                    })
        
        return violations
    
    def _check_config_rules(self, file_path: Path, content: str, lines: List[str]) -> List[Dict]:
        """Check configuration terminology consistency."""
        violations = []
        # Implementation details for configuration rule checking
        return violations
    
    def _check_component_rules(self, file_path: Path, content: str, lines: List[str]) -> List[Dict]:
        """Check architectural component naming consistency."""
        violations = []
        # Implementation details for component rule checking
        return violations
    
    def _check_overload_rules(self, file_path: Path, content: str, lines: List[str]) -> List[Dict]:
        """Check for overloaded terminology usage."""
        violations = []
        # Implementation details for overload rule checking
        return violations
    
    def lint_project(self) -> bool:
        """Lint the entire project for terminology violations."""
        # File patterns to check
        patterns = [
            "src/**/*.rs",
            "docs/**/*.md",
            "tests/**/*.rs",
            "examples/**/*.rs",
            "scripts/**/*.py",
            "frontend/**/*.ts",
            "frontend/**/*.tsx",
        ]
        
        all_violations = []
        
        for pattern in patterns:
            for file_path in self.project_root.glob(pattern):
                if file_path.is_file():
                    violations = self.lint_file(file_path)
                    all_violations.extend(violations)
        
        # Report results
        if all_violations:
            self._report_violations(all_violations)
            return False
        else:
            print("✅ No terminology violations found!")
            return True
    
    def _report_violations(self, violations: List[Dict]):
        """Report terminology violations in a structured format."""
        print(f"❌ Found {len(violations)} terminology violations:")
        print()
        
        # Group by category
        by_category = {}
        for violation in violations:
            category = violation.get('category', 'unknown')
            if category not in by_category:
                by_category[category] = []
            by_category[category].append(violation)
        
        for category, cat_violations in by_category.items():
            print(f"## {category.title()} Issues ({len(cat_violations)})")
            for v in cat_violations:
                print(f"  {v['file']}:{v['line']}:{v.get('column', 0)}")
                print(f"    {v['message']}")
                if 'suggestion' in v:
                    print(f"    Suggestion: {v['suggestion']}")
                print()


if __name__ == "__main__":
    project_root = Path.cwd()
    linter = TerminologyLinter(project_root)
    
    success = linter.lint_project()
    sys.exit(0 if success else 1)
```

### 1.2 Git Pre-commit Hook Integration

Add terminology validation to the pre-commit workflow:

```bash
#!/bin/bash
# .git/hooks/pre-commit
# Terminology validation pre-commit hook

echo "🔍 Running terminology validation..."

# Run terminology linter
python3 scripts/terminology-linter.py

if [ $? -ne 0 ]; then
    echo "❌ Terminology violations found. Please fix before committing."
    echo "Run 'python3 scripts/terminology-linter.py' for details."
    exit 1
fi

echo "✅ Terminology validation passed."
```

### 1.3 CI/CD Integration

Add terminology checking to GitHub Actions workflow:

```yaml
# .github/workflows/terminology-check.yml
name: Terminology Validation

on:
  pull_request:
    branches: [ master, develop ]
  push:
    branches: [ master, develop ]

jobs:
  terminology-check:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.9'
    
    - name: Run Terminology Linter
      run: |
        chmod +x scripts/terminology-linter.py
        python3 scripts/terminology-linter.py
        
    - name: Check Documentation Consistency
      run: |
        # Validate that glossary terms are used consistently in docs
        python3 scripts/validate-glossary-usage.py
```

## 2. Development Guidelines

### 2.1 Coding Standards

#### Naming Conventions
- **Structs/Enums:** PascalCase matching glossary terms exactly
- **Functions:** snake_case with descriptive verbs from glossary
- **Modules:** snake_case reflecting domain boundaries
- **Constants:** SCREAMING_SNAKE_CASE with domain prefixes

#### Documentation Requirements
- All public APIs must use standardized terminology
- Code comments must align with ubiquitous language
- Error messages must use consistent terminology
- Examples must demonstrate proper terminology usage

#### Anti-Pattern Terminology Rules
```rust
// ✅ Correct usage
pub struct AntiPatternDetector;
pub enum AntiPatternType;
let anti_pattern_config = AntiPatternConfig::new();

// ❌ Incorrect usage  
pub struct AntipatternDetector;  // Missing hyphen/underscore
pub enum AntiPattern;            // Missing Type suffix
let antipattern_config = ...;    // Inconsistent naming
```

#### Component Naming Rules
```rust
// ✅ Correct architectural component naming
pub struct AnalysisEngine;        // Core orchestrator
pub struct ConfigurationService;  // Internal business logic
pub struct OllamaProvider;        // External integration
pub struct CacheManager;          // Resource management
pub struct AnalysisEngineBuilder; // Object construction

// ❌ Incorrect usage - violates component type distinctions
pub struct ConfigurationEngine;   // Should be Service
pub struct OllamaService;         // Should be Provider
pub struct CacheService;          // Should be Manager
```

### 2.2 Documentation Standards

#### Glossary Integration
- Link to glossary terms in documentation using `[term](../09-community/glossary.md#term)`
- Use exact terminology from glossary definitions
- Avoid abbreviations unless defined in glossary
- Maintain consistent capitalization and hyphenation

#### Code Comment Standards
```rust
/// Detects god object anti-patterns in the analyzed codebase.
/// 
/// A [god object](../docs/09-community/glossary.md#god-object) is a class
/// or module that violates the Single Responsibility Principle by 
/// centralizing excessive functionality.
pub struct GodObjectDetector {
    // Configuration for detection thresholds
    config: DetectorConfig,
}
```

## 3. Review Process Integration

### 3.1 Code Review Checklist

Add terminology validation to pull request templates:

```markdown
## Terminology Review Checklist

- [ ] All new types use PascalCase matching glossary terms
- [ ] Function names follow snake_case with domain-appropriate verbs
- [ ] Anti-pattern terminology follows standardization (anti_pattern/AntiPattern/anti-pattern)
- [ ] Component types follow architectural patterns (Engine/Service/Provider/Manager/Builder)
- [ ] Documentation uses standardized terminology consistently
- [ ] No terminology overloading or concept conflicts introduced
- [ ] Error messages use ubiquitous language terms
- [ ] Comments align with domain vocabulary

### Specific Checks

**Anti-Pattern Terminology:**
- [ ] Code structures use `anti_pattern` (snake_case)
- [ ] Type names use `AntiPattern` prefix (PascalCase)
- [ ] Documentation uses "anti-pattern" (hyphenated)

**Analysis Terminology:**
- [ ] US spelling used consistently (analyze/analysis/analyzer)
- [ ] No UK spellings (analyse/analyser) present

**Component Architecture:**
- [ ] "Engine" reserved for core orchestrators
- [ ] "Service" used for internal business logic
- [ ] "Provider" used for external integrations
- [ ] "Manager" used for resource management
- [ ] "Builder" used for object construction
```

### 3.2 Automated Review Assistance

Create GitHub Actions bot comments for terminology issues:

```yaml
# .github/workflows/pr-terminology-review.yml
name: PR Terminology Review

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  terminology-review:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
      with:
        fetch-depth: 0
        
    - name: Check terminology in PR changes
      run: |
        # Get changed files
        git diff --name-only origin/master...HEAD > changed_files.txt
        
        # Run terminology linter on changed files only
        python3 scripts/terminology-linter.py --files-from changed_files.txt --output json > violations.json
        
    - name: Comment on PR with violations
      if: failure()
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const violations = JSON.parse(fs.readFileSync('violations.json', 'utf8'));
          
          if (violations.length > 0) {
            const comment = `## 📝 Terminology Review
            
Found ${violations.length} terminology issues in this PR:

${violations.map(v => `- **${v.file}:${v.line}**: ${v.message}`).join('\n')}

Please review the [Ubiquitous Language Glossary](docs/09-community/glossary.md) for standardized terminology.`;
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
          }
```

## 4. Continuous Monitoring

### 4.1 Terminology Metrics Dashboard

Track terminology consistency over time:

```python
# scripts/terminology-metrics.py
"""
Generate terminology consistency metrics for monitoring dashboard
"""

import json
from pathlib import Path
from datetime import datetime

class TerminologyMetrics:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        
    def generate_metrics(self) -> dict:
        """Generate comprehensive terminology metrics."""
        return {
            'timestamp': datetime.now().isoformat(),
            'consistency_score': self._calculate_consistency_score(),
            'violation_count': self._count_violations(),
            'coverage_metrics': self._calculate_coverage(),
            'domain_compliance': self._check_domain_compliance(),
            'trend_data': self._generate_trend_data()
        }
    
    def _calculate_consistency_score(self) -> float:
        """Calculate overall terminology consistency score (0-100)."""
        # Implementation for consistency scoring
        pass
    
    def _count_violations(self) -> dict:
        """Count violations by category."""
        # Implementation for violation counting
        pass
    
    def _calculate_coverage(self) -> dict:
        """Calculate terminology coverage across domains."""
        # Implementation for coverage calculation
        pass
```

### 4.2 Regular Validation Reports

Automated weekly terminology health reports:

```bash
#!/bin/bash
# scripts/weekly-terminology-report.sh

echo "📊 Generating Weekly Terminology Report..."

# Run comprehensive terminology analysis
python3 scripts/terminology-linter.py --detailed-report > terminology-report.md

# Generate metrics
python3 scripts/terminology-metrics.py > terminology-metrics.json

# Create summary report
cat << EOF > weekly-terminology-summary.md
# Weekly Terminology Health Report - $(date +%Y-%m-%d)

## Summary
$(python3 scripts/generate-summary.py)

## Detailed Analysis
$(cat terminology-report.md)

## Metrics
$(python3 scripts/format-metrics.py terminology-metrics.json)

## Action Items
$(python3 scripts/generate-action-items.py)
EOF

echo "✅ Weekly terminology report generated: weekly-terminology-summary.md"
```

## 5. Training and Onboarding

### 5.1 Developer Onboarding Checklist

```markdown
# Uveddi Terminology Onboarding

## Required Reading
- [ ] [Ubiquitous Language Glossary](docs/09-community/glossary.md)
- [ ] [Terminology Mapping](docs/05-development/terminology-mapping.md)
- [ ] [Enforcement Process](docs/05-development/ubiquitous-language-enforcement.md)

## Setup Tasks
- [ ] Install terminology linter: `pip install -r scripts/requirements.txt`
- [ ] Configure pre-commit hooks: `./scripts/setup-pre-commit.sh`
- [ ] Run initial terminology check: `python3 scripts/terminology-linter.py`

## Knowledge Verification
- [ ] Can identify correct anti-pattern terminology usage
- [ ] Understands component type distinctions (Engine/Service/Provider/Manager)
- [ ] Knows when to use US vs UK spelling conventions
- [ ] Can map ubiquitous language terms to code structures

## Practical Exercise
- [ ] Review a sample PR for terminology compliance
- [ ] Fix terminology violations in practice codebase
- [ ] Write documentation using standardized terminology
```

### 5.2 Team Training Materials

Create comprehensive training resources:

- **Terminology Quick Reference Card** - Printable cheat sheet
- **Interactive Glossary** - Searchable web interface  
- **Video Tutorials** - Domain-specific terminology explanations
- **Practice Exercises** - Hands-on terminology application

## 6. Tool Integration

### 6.1 IDE Integration

Provide IDE-specific terminology assistance:

```json
// .vscode/settings.json
{
  "cSpell.words": [
    "AntiPattern", "GodObject", "CyclicDependency", 
    "TightCoupling", "AnalysisEngine", "DetectorConfig"
  ],
  "cSpell.flagWords": [
    "antipattern", "analyse", "analyser", "config"
  ]
}
```

### 6.2 Documentation Generation

Integrate terminology validation into documentation builds:

```bash
# docs/build.sh
#!/bin/bash

echo "📚 Building documentation with terminology validation..."

# Validate terminology in all markdown files
python3 ../scripts/terminology-linter.py docs/

# Build documentation
mdbook build

# Generate terminology index
python3 ../scripts/generate-terminology-index.py

echo "✅ Documentation built with terminology validation"
```

## 7. Maintenance and Evolution

### 7.1 Glossary Update Process

1. **Proposal Phase** - New terms proposed via GitHub issues
2. **Review Phase** - Team discussion and domain expert validation
3. **Implementation Phase** - Update glossary and related documentation
4. **Migration Phase** - Update existing code to use new terminology
5. **Validation Phase** - Verify consistency across entire codebase

### 7.2 Version Control

Track terminology evolution:
- Maintain glossary versioning with semantic versioning
- Document terminology changes in CHANGELOG
- Provide migration guides for major terminology updates
- Archive deprecated terms with explanations

## Implementation Priority

### Phase 1 (Immediate - 1-2 weeks)
- [ ] Implement basic terminology linter
- [ ] Create pre-commit hook integration
- [ ] Update existing glossary with standardized definitions
- [ ] Create developer onboarding materials

### Phase 2 (Short-term - 1 month)
- [ ] Integrate CI/CD validation
- [ ] Develop comprehensive review checklists
- [ ] Create IDE integration tools
- [ ] Implement automated PR review assistance

### Phase 3 (Medium-term - 2-3 months)
- [ ] Build terminology metrics dashboard
- [ ] Develop advanced violation detection
- [ ] Create interactive training materials
- [ ] Implement automated report generation

### Phase 4 (Long-term - 6+ months)
- [ ] Advanced semantic analysis of terminology usage
- [ ] Machine learning-based terminology suggestion
- [ ] Integration with external documentation systems
- [ ] Community contribution tools for terminology evolution

## Success Metrics

- **Consistency Score:** >95% terminology consistency across codebase
- **Violation Reduction:** <10 terminology violations per 1000 lines of code
- **Review Efficiency:** Terminology issues caught in <24 hours during PR review
- **Team Adoption:** 100% team compliance with terminology standards
- **Documentation Quality:** All public APIs using standardized terminology

This enforcement process ensures that Uveddi's ubiquitous language remains consistent, clear, and effective in facilitating communication and reducing architectural drift through precise, shared vocabulary.