# Terminology Enforcement Process

## Overview

This document outlines the processes and tools for enforcing consistent terminology across the Uveddi codebase, ensuring adherence to our **Ubiquitous Language Glossary** and preventing architectural drift through inconsistent naming.

---

## Code Review Process

### Pre-Review Automated Checks

#### 1. Terminology Linting Rules

Create custom Clippy lints for common terminology violations:

```toml
# .clippy.toml
terminology-rules = [
    # Anti-pattern naming consistency
    { pattern = "AntiPatternType", replacement = "AntiPattern", message = "Use AntiPattern instead of AntiPatternType for consistency" },
    { pattern = "*_anti_pattern_type_*", replacement = "*_anti_pattern_*", message = "Use anti_pattern prefix consistently" },
    
    # Engine vs Manager vs Service distinction
    { pattern = "*AnalysisManager", replacement = "*AnalysisEngine", message = "Use Engine suffix for execution components" },
    { pattern = "*DatabaseEngine", replacement = "*DatabaseService", message = "Use Service suffix for interface abstractions" },
    
    # AI terminology standardization
    { pattern = "*AiSuggestion", replacement = "*AiInsight", message = "Use AiInsight for all AI-generated content" },
    { pattern = "*ai_explanation", replacement = "*ai_insight", message = "Use ai_insight consistently" },
    
    # Component vs Module distinction
    { pattern = "*ModuleAnalysis", replacement = "*ComponentAnalysis", message = "Use Component for architectural analysis units" },
]
```

#### 2. Automated Terminology Checker

```bash
#!/bin/bash
# scripts/check-terminology.sh

# Check for deprecated terminology in new changes
git diff --name-only HEAD~1 HEAD | grep -E '\.(rs|md)$' | while read file; do
    echo "Checking terminology in $file..."
    
    # Check for forbidden patterns
    if grep -n "AntiPatternType\|anti_pattern_type_id\|AiSuggestion" "$file"; then
        echo "❌ Deprecated terminology found in $file"
        echo "Please refer to docs/01-architecture/ubiquitous-language-glossary.md"
        exit 1
    fi
    
    # Check for inconsistent naming
    if grep -n -E "(Engine|Manager|Service)" "$file" | grep -v -E "(AnalysisEngine|PluginManager|.*Service)" ; then
        echo "⚠️  Review Engine/Manager/Service usage in $file"
        echo "Ensure proper distinction per glossary guidelines"
    fi
done
```

### Manual Review Checklist

#### Domain Terminology Checklist

```markdown
## Terminology Review Checklist

### Core Analysis Domain
- [ ] Uses `Detector` (not `Analyzer`) for issue detection
- [ ] Uses `Extractor` for data extraction logic  
- [ ] Uses `Engine` for execution/processing components
- [ ] Uses `AntiPattern` (not `AntiPatternType`)
- [ ] Uses `ArchitecturalIssue` for specific issue instances
- [ ] Uses `AnalysisRun` for analysis sessions

### Component Architecture  
- [ ] Uses `Component` for architectural analysis units
- [ ] Reserves `Module` for language-specific constructs
- [ ] Uses `Dependency` for inter-component relationships
- [ ] Uses `Architecture` for system-level structure

### AI & Intelligence
- [ ] Uses `AiProvider` for AI service implementations
- [ ] Uses `AiInsight` for AI-generated content
- [ ] Uses `Prompt` for AI input processing
- [ ] Uses `AiResponse` for raw AI output

### Security & Access Control
- [ ] Uses `User` for basic identity (no auth context)
- [ ] Uses `AuthenticatedUser` when auth is verified
- [ ] Uses `Session` for login sessions
- [ ] Uses `Role` for permission collections
- [ ] Uses `Permission` for specific access rights

### Plugin System
- [ ] Uses `Plugin` for extension code
- [ ] Uses `PluginEngine` for plugin execution
- [ ] Uses `PluginManager` for lifecycle coordination
- [ ] Uses `ResourceLimits` for execution constraints

### Configuration
- [ ] Uses `Config` for settings containers
- [ ] Uses `ConfigLoader` for loading logic
- [ ] Uses `ConfigBuilder` for construction logic
```

#### Naming Convention Checklist

```markdown
## Naming Convention Checklist

### Rust Conventions
- [ ] Structs use PascalCase
- [ ] Fields use snake_case  
- [ ] Enums use PascalCase for names and variants
- [ ] Functions use snake_case with verb-noun pattern
- [ ] Traits use descriptive names ending in purpose

### Anti-Patterns to Avoid
- [ ] No concept overloading (same term, different concepts)
- [ ] No synonym proliferation (multiple terms, same concept)
- [ ] No technical jargon mixing in domain layer
- [ ] No generic names without context
```

### Review Automation

#### 1. GitHub Actions Integration

```yaml
# .github/workflows/terminology-check.yml
name: Terminology Compliance

on:
  pull_request:
    paths:
      - 'src/**/*.rs'
      - 'docs/**/*.md'

jobs:
  terminology-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 2
          
      - name: Check Terminology Compliance
        run: |
          chmod +x scripts/check-terminology.sh
          ./scripts/check-terminology.sh
          
      - name: Validate Glossary References
        run: |
          # Check that new domain terms are defined in glossary
          git diff --name-only HEAD~1 HEAD | xargs grep -l "struct\|enum\|trait" | while read file; do
            # Extract new type definitions
            git diff HEAD~1 HEAD "$file" | grep "^+.*\(struct\|enum\|trait\)" | \
            sed 's/^+.*\(struct\|enum\|trait\) \([A-Za-z][A-Za-z0-9]*\).*/\2/' | \
            while read type_name; do
              if ! grep -q "$type_name" docs/01-architecture/ubiquitous-language-glossary.md; then
                echo "⚠️  New type '$type_name' not found in glossary"
                echo "Consider adding to ubiquitous-language-glossary.md"
              fi
            done
          done
```

#### 2. Pre-commit Hooks

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Check terminology before commit
./scripts/check-terminology.sh

if [ $? -ne 0 ]; then
    echo "❌ Terminology check failed. Please fix issues before committing."
    echo "Refer to docs/01-architecture/ubiquitous-language-glossary.md"
    exit 1
fi

# Check for glossary updates
if git diff --cached --name-only | grep -q "src/.*\.rs$"; then
    echo "📚 Remember to update glossary if introducing new domain concepts"
fi
```

---

## Developer Tools

### IDE Configuration

#### VS Code Settings

```json
{
  "editor.codeActionsOnSave": {
    "source.fixAll.clippy": true
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-W", "clippy::terminology"],
  
  "cSpell.words": [
    "AntiPattern", "ArchitecturalIssue", "AnalysisRun",
    "AiProvider", "AiInsight", "AiResponse", 
    "ComponentExtractor", "DependencyExtractor",
    "PluginEngine", "PluginManager", "ResourceLimits",
    "AuthenticatedUser", "ConfigLoader", "ConfigBuilder"
  ],
  
  "cSpell.flagWords": [
    "AntiPatternType", "AiSuggestion", "ai_explanation",
    "ModuleAnalysis", "anti_pattern_type_id"
  ]
}
```

#### IntelliJ IDEA Settings

```xml
<!-- .idea/inspectionProfiles/terminology.xml -->
<profile>
  <inspection_tool class="SpellCheckingInspection">
    <option name="processCode" value="true" />
    <option name="processLiterals" value="true" />
    <option name="processComments" value="true" />
  </inspection_tool>
  
  <inspection_tool class="RegExpRedundantEscape">
    <terminology_rules>
      <rule pattern="AntiPatternType" replacement="AntiPattern" severity="ERROR" />
      <rule pattern="AiSuggestion" replacement="AiInsight" severity="WARNING" />
      <rule pattern="ModuleAnalysis" replacement="ComponentAnalysis" severity="WARNING" />
    </terminology_rules>
  </inspection_tool>
</profile>
```

### Custom Linting Tools

#### 1. Terminology Lint (Custom Clippy Lint)

```rust
// tools/terminology-lint/src/lib.rs
use clippy_utils::diagnostics::span_lint_and_sugg;
use rustc_lint::{EarlyLintPass, EarlyContext};
use rustc_ast::{Item, ItemKind};
use rustc_span::Span;

declare_clippy_lint! {
    pub DEPRECATED_TERMINOLOGY,
    style,
    "checks for deprecated terminology that should be updated"
}

impl EarlyLintPass for DeprecatedTerminology {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        match &item.kind {
            ItemKind::Struct(_, _) | ItemKind::Enum(_, _) | ItemKind::Trait(_, _) => {
                let name = item.ident.name.as_str();
                
                if name.contains("AntiPatternType") {
                    span_lint_and_sugg(
                        cx,
                        DEPRECATED_TERMINOLOGY,
                        item.span,
                        "deprecated terminology: use 'AntiPattern' instead of 'AntiPatternType'",
                        "consider renaming to",
                        name.replace("AntiPatternType", "AntiPattern"),
                        Applicability::MachineApplicable,
                    );
                }
                
                if name.contains("AiSuggestion") {
                    span_lint_and_sugg(
                        cx,
                        DEPRECATED_TERMINOLOGY,
                        item.span,
                        "use 'AiInsight' for AI-generated content",
                        "consider renaming to",
                        name.replace("AiSuggestion", "AiInsight"),
                        Applicability::MachineApplicable,
                    );
                }
            }
            _ => {}
        }
    }
}
```

#### 2. Glossary Validator

```rust
// tools/glossary-validator/src/main.rs
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn main() {
    let glossary_terms = extract_glossary_terms();
    let code_terms = extract_code_terms();
    
    // Find terms used in code but not in glossary
    let undocumented: Vec<_> = code_terms
        .difference(&glossary_terms)
        .collect();
    
    if !undocumented.is_empty() {
        println!("⚠️  Terms used in code but not documented in glossary:");
        for term in undocumented {
            println!("  - {}", term);
        }
        println!("\nConsider adding these to the ubiquitous language glossary.");
    }
    
    // Find deprecated terms still in use
    let deprecated_terms = vec!["AntiPatternType", "AiSuggestion", "ModuleAnalysis"];
    for term in deprecated_terms {
        if code_terms.contains(term) {
            println!("❌ Deprecated term '{}' still found in code", term);
        }
    }
}

fn extract_glossary_terms() -> HashSet<String> {
    let glossary = fs::read_to_string("docs/01-architecture/ubiquitous-language-glossary.md")
        .expect("Failed to read glossary");
    
    // Extract terms from markdown table format
    glossary
        .lines()
        .filter(|line| line.starts_with("| **"))
        .map(|line| {
            line.split('|')
                .nth(1)
                .unwrap()
                .trim()
                .trim_start_matches("**")
                .trim_end_matches("**")
                .to_string()
        })
        .collect()
}

fn extract_code_terms() -> HashSet<String> {
    let mut terms = HashSet::new();
    
    // Walk through src directory
    for entry in walkdir::WalkDir::new("src") {
        let entry = entry.unwrap();
        if entry.path().extension() == Some(std::ffi::OsStr::new("rs")) {
            let content = fs::read_to_string(entry.path()).unwrap();
            
            // Extract struct, enum, trait names
            for line in content.lines() {
                if let Some(name) = extract_type_name(line) {
                    terms.insert(name);
                }
            }
        }
    }
    
    terms
}
```

---

## Migration Process

### Phase 1: Immediate Enforcement

1. **Enable Linting**: Add terminology checks to CI/CD
2. **Update Templates**: Ensure all templates use standardized terms  
3. **Train Team**: Share glossary and enforcement process

### Phase 2: Gradual Migration

1. **Create Type Aliases**: For backward compatibility
   ```rust
   // Deprecated but maintained for compatibility
   #[deprecated(since = "0.8.0", note = "use AntiPattern instead")]
   pub type AntiPatternType = AntiPattern;
   ```

2. **Update New Code**: All new code must use standardized terms
3. **Refactor Critical Paths**: Update public APIs first

### Phase 3: Complete Migration

1. **Remove Deprecated Terms**: After sufficient deprecation period
2. **Update Dependencies**: Ensure external APIs align
3. **Documentation Cleanup**: Remove all references to old terms

---

## Measuring Success

### Metrics

1. **Terminology Consistency**: Percentage of standardized vs deprecated terms
2. **Review Efficiency**: Time spent on terminology discussions in reviews
3. **Developer Onboarding**: Time to understand domain concepts
4. **Issue Resolution**: Reduction in communication-related bugs

### Monitoring

```bash
#!/bin/bash
# scripts/terminology-metrics.sh

echo "=== Terminology Compliance Report ==="

# Count occurrences of standardized terms
echo "✅ Standardized terms usage:"
grep -r "AntiPattern\|AiInsight\|ComponentAnalysis" src/ | wc -l

# Count deprecated terms
echo "❌ Deprecated terms still in use:"
deprecated_count=$(grep -r "AntiPatternType\|AiSuggestion\|ModuleAnalysis" src/ | wc -l)
echo "$deprecated_count instances found"

# Calculate compliance ratio
total_terms=$((standardized_count + deprecated_count))
if [ $total_terms -gt 0 ]; then
    compliance=$((standardized_count * 100 / total_terms))
    echo "📊 Compliance rate: ${compliance}%"
fi

# Check for new domain terms without glossary entries
echo "⚠️  Potential missing glossary entries:"
git log --since="1 week ago" --grep="struct\|enum\|trait" --oneline | wc -l
```

---

## Continuous Improvement

### Regular Reviews

1. **Monthly Glossary Review**: Update terms based on usage patterns
2. **Quarterly Compliance Audit**: Measure progress and identify gaps
3. **Annual Terminology Evolution**: Assess if language needs major updates

### Feedback Integration

1. **Developer Surveys**: Collect feedback on terminology clarity
2. **Issue Analysis**: Track problems caused by terminology confusion
3. **External Feedback**: Incorporate user and contributor suggestions

### Tool Evolution

1. **Enhanced Linting**: Add more sophisticated pattern detection
2. **IDE Plugins**: Develop custom plugins for better integration
3. **Documentation Tools**: Automate glossary maintenance

---

## Related Resources

- [Ubiquitous Language Glossary](../01-architecture/ubiquitous-language-glossary.md)
- [Code Review Guidelines](./code-review-standards.md)
- [Domain-Driven Design Principles](../02-design/domain-driven-design.md)
- [API Design Standards](../03-api/design-guidelines.md)

---

*This enforcement process ensures consistent terminology usage while maintaining development velocity and code quality.*