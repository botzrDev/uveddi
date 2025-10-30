# Code Review Terminology Guidelines

This document provides comprehensive guidance for code reviewers to ensure consistent application of Uveddi's ubiquitous language standards during the review process.

## Review Philosophy

Code review serves as the primary enforcement mechanism for ubiquitous language standards. Every reviewer is responsible for:

1. **Terminology Consistency** - Ensuring standardized vocabulary usage
2. **Domain Alignment** - Verifying concepts map correctly to domain boundaries  
3. **Architectural Coherence** - Maintaining naming patterns that reflect system design
4. **Communication Clarity** - Promoting clear, unambiguous language

## Quick Reference for Reviewers

### Anti-Pattern Terminology Standards
| Context | Correct Form | Examples | Violations to Flag |
|---------|-------------|----------|------------------|
| Code Structures | `anti_pattern` | `anti_pattern_detector.rs`, `detect_anti_patterns()` | `antipattern`, `anti.pattern` |
| Type Names | `AntiPattern` | `AntiPatternType`, `AntiPatternConfig` | `Antipattern`, `Anti_Pattern` |
| Documentation | "anti-pattern" | "detects anti-patterns", "anti-pattern analysis" | "antipattern", "anti pattern" |
| URLs/IDs | "anti-pattern" | `/api/anti-patterns`, `anti-pattern-report` | "antipattern", "anti_pattern" |

### Component Architecture Patterns
| Suffix | Purpose | Correct Usage | Flag These Violations |
|--------|---------|---------------|---------------------|
| `Engine` | Core orchestrators | `AnalysisEngine`, `AiAnalysisEngine` | `WasmPluginEngine`, `ConfigEngine` |
| `Service` | Business logic | `ConfigurationService`, `AuthenticationService` | `ConfigurationEngine`, `AuthProvider` |
| `Provider` | External integrations | `OllamaProvider`, `DatabaseProvider` | `OllamaService`, `DatabaseEngine` |
| `Manager` | Resource management | `CacheManager`, `PluginLifecycleManager` | `CacheService`, `PluginEngine` |
| `Builder` | Object construction | `AnalysisEngineBuilder`, `ConfigBuilder` | `AnalysisFactory`, `ConfigCreator` |
| `Detector` | Issue identification | `GodObjectDetector`, `CyclicDependencyDetector` | `GodObjectFinder`, `CyclicChecker` |

### Spelling Consistency (US English)
| Correct (US) | Incorrect (UK) | Context |
|-------------|----------------|---------|
| `analyze` | `analyse` | Verb form in all contexts |
| `analysis` | `analysis` | Noun form (same in both) |
| `analyzer` | `analyser` | Agent noun |
| "configuration" | "config" | Documentation (full form) |
| `Config` | `Configuration` | Code suffixes (abbreviated) |

## Detailed Review Process

### 1. Initial Scan Phase

Before diving into detailed code review, perform a quick terminology scan:

**Command Line Check:**
```bash
# Quick terminology check on PR changes
git diff --name-only HEAD~1 | xargs grep -i "antipattern\|analyse\|analyser" 
```

**Visual Scan Checklist:**
- [ ] Look for inconsistent anti-pattern terminology
- [ ] Check component naming patterns align with architecture
- [ ] Verify US spelling in comments and documentation
- [ ] Identify any new abbreviations or domain terms

### 2. Structural Review Phase

#### 2.1 Type and Structure Naming

**✅ Correct Examples:**
```rust
// Proper anti-pattern detector naming
pub struct GodObjectDetector {
    config: DetectorConfig,
}

// Correct analysis result structure  
pub struct AnalysisResult {
    issues: Vec<ArchitecturalIssue>,
    metrics: PerformanceMetrics,
}

// Proper architectural component naming
pub struct AnalysisEngine {
    detectors: DetectorRegistry,
    config: AnalysisConfig,
}

pub struct ConfigurationService {
    loader: ConfigLoader,
    validator: ConfigValidator,
}
```

**❌ Flag These Violations:**
```rust
// Inconsistent anti-pattern terminology
pub struct GodObjectAntipattern;  // Should be GodObjectDetector

// Wrong component architecture pattern
pub struct ConfigurationEngine;   // Should be ConfigurationService
pub struct OllamaService;         // Should be OllamaProvider

// Inconsistent naming patterns
pub struct AnalyseFinder;         // Should be AnalyzeDetector (US spelling)
```

#### 2.2 Function and Method Naming

**✅ Correct Examples:**
```rust
impl GodObjectDetector {
    /// Detects god object anti-patterns in the provided code
    pub fn detect_issues(&self, code: &ParsedCode) -> Vec<ArchitecturalIssue> {
        // Implementation
    }
    
    /// Analyzes class complexity metrics
    pub fn analyze_complexity(&self, class: &ClassNode) -> ComplexityMetrics {
        // Implementation
    }
}

impl ConfigurationService {
    /// Loads configuration from multiple sources
    pub fn load_configuration(&self) -> Result<Configuration, ConfigError> {
        // Implementation
    }
}
```

**❌ Flag These Violations:**
```rust
impl GodObjectDetector {
    // UK spelling violation
    pub fn analyse_complexity(&self, class: &ClassNode) -> ComplexityMetrics { }
    
    // Inconsistent terminology
    pub fn find_antipatterns(&self, code: &ParsedCode) -> Vec<Issue> { }
    
    // Wrong verb choice for domain
    pub fn discover_issues(&self, code: &ParsedCode) -> Vec<Issue> { }
}
```

### 3. Documentation Review Phase

#### 3.1 Code Comments

**✅ Correct Examples:**
```rust
/// Detects god object anti-patterns using complexity analysis.
/// 
/// A god object is a class that violates the Single Responsibility
/// Principle by centralizing too much functionality. This detector
/// analyzes method counts, dependency relationships, and complexity
/// metrics to identify potential violations.
pub struct GodObjectDetector {
    /// Configuration parameters for detection thresholds
    config: DetectorConfig,
    /// Analyzer for computing complexity metrics
    complexity_analyzer: ComplexityAnalyzer,
}
```

**❌ Flag These Violations:**
```rust
/// Finds god object antipatterns using analysis.  // Wrong terminology
/// 
/// A god object is a class that violates SRP by centralising too much  // UK spelling
/// functionality. This detector analyses method counts to identify  // UK spelling
/// potential violations.
pub struct GodObjectDetector {
    /// Config for detection thresholds  // Abbreviated in documentation
    config: DetectorConfig,
}
```

#### 3.2 API Documentation

**Review Checklist:**
- [ ] All public APIs documented with consistent terminology
- [ ] Domain concepts linked to glossary where appropriate
- [ ] Examples use standardized vocabulary
- [ ] Error descriptions use ubiquitous language
- [ ] Return types clearly documented with domain terms

**✅ Good API Documentation:**
```rust
impl AnalysisEngine {
    /// Executes comprehensive analysis on the provided codebase.
    /// 
    /// This method orchestrates multiple detectors to identify architectural
    /// issues including anti-patterns, design violations, and quality concerns.
    /// 
    /// # Arguments
    /// * `codebase` - The codebase to analyze
    /// * `config` - Analysis configuration parameters
    /// 
    /// # Returns
    /// * `AnalysisResult` - Comprehensive analysis results containing:
    ///   - Detected architectural issues with criticality levels
    ///   - Performance metrics and statistics  
    ///   - Recommendations for improvement
    /// 
    /// # Example
    /// ```rust
    /// let engine = AnalysisEngine::new();
    /// let result = engine.analyze_codebase(&codebase, &config)?;
    /// for issue in result.issues {
    ///     println!("Found {} anti-pattern: {}", issue.pattern_type, issue.description);
    /// }
    /// ```
    pub fn analyze_codebase(&self, codebase: &Codebase, config: &AnalysisConfig) 
        -> Result<AnalysisResult, AnalysisError> {
        // Implementation
    }
}
```

### 4. Domain Boundary Review

#### 4.1 Cross-Domain Interactions

**Review Questions:**
- Do interface definitions use consistent terminology across domains?
- Are domain concepts properly encapsulated without leaking implementation details?
- Do integration points maintain clear vocabulary boundaries?

**✅ Good Cross-Domain Integration:**
```rust
// Clear domain separation with consistent interfaces
pub trait LlmProvider {
    /// Analyzes code using AI capabilities
    fn analyze_with_ai(&self, context: &AnalysisContext) -> Result<AiInsights, AiError>;
}

// Plugin domain properly isolated
pub trait WasmPluginRuntime {  // Note: Runtime, not Engine
    /// Executes detector plugin on analysis target
    fn execute_detector(&self, plugin: &Plugin, target: &AnalysisTarget) 
        -> Result<DetectionResult, PluginError>;
}
```

**❌ Flag These Issues:**
```rust
// Terminology leakage between domains  
pub trait AiService {  // Should be LlmProvider
    fn do_analysis(&self, stuff: &SomeStuff) -> Result<AiStuff, Error>;  // Vague terminology
}

// Incorrect component classification
pub trait WasmPluginEngine {  // Should be WasmPluginRuntime
    fn run_plugin(&self, plugin: &Plugin) -> PluginResult;  // Inconsistent naming
}
```

#### 4.2 Module Structure Alignment

**Review Checklist:**
- [ ] Module names reflect domain boundaries clearly
- [ ] File names use consistent terminology patterns
- [ ] Directory structure aligns with ubiquitous language concepts
- [ ] No terminology conflicts between modules

### 5. Error Handling and Messages

#### 5.1 Error Type Naming

**✅ Correct Error Naming:**
```rust
#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Detector configuration invalid: {reason}")]
    DetectorConfigurationInvalid { reason: String },
    
    #[error("Anti-pattern detection failed: {detector_name}")]
    AntiPatternDetectionFailed { detector_name: String },
    
    #[error("Analysis engine initialization failed")]
    AnalysisEngineInitializationFailed,
}
```

**❌ Flag These Violations:**
```rust
#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Detector config invalid: {reason}")]  // Abbreviated "config"
    DetectorConfigInvalid { reason: String },
    
    #[error("Antipattern detection failed: {detector}")]  // Wrong terminology
    AntipatternDetectionFailed { detector: String },
}
```

#### 5.2 User-Facing Messages

**Review Criteria:**
- Error messages use standardized terminology
- User guidance references correct domain concepts
- Help text maintains vocabulary consistency
- Status messages align with ubiquitous language

### 6. Test Code Review

#### 6.1 Test Naming Standards

**✅ Good Test Names:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_god_object_detector_identifies_excessive_methods() {
        // Test implementation
    }
    
    #[test]
    fn test_analysis_engine_aggregates_detector_results() {
        // Test implementation
    }
    
    #[test]
    fn test_anti_pattern_detection_respects_configuration_thresholds() {
        // Test implementation
    }
}
```

**❌ Flag These Issues:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_god_object_finder_works() {  // Wrong terminology, vague
        // Test implementation
    }
    
    #[test]
    fn test_analyser_aggregates_results() {  // UK spelling
        // Test implementation
    }
}
```

### 7. Configuration and Documentation Files

#### 7.1 Configuration File Review

**Review Configuration Files:**
- `config/*.toml` - Ensure parameter names use standardized terminology
- `Cargo.toml` - Verify feature names align with domain concepts
- `docs/*.md` - Check for consistent vocabulary usage

**✅ Good Configuration:**
```toml
# config/analysis.toml
[anti_pattern_detection]
god_object_threshold = 50
cyclic_dependency_detection_enabled = true

[analysis_engine]
parallel_detector_execution = true
ai_enhancement_enabled = false
```

**❌ Flag These Issues:**
```toml
# config/analysis.toml
[antipattern_detection]     # Wrong terminology
god_object_limit = 50       # Inconsistent parameter naming
cycle_detection = true      # Abbreviated concept name
```

## Review Communication Guidelines

### 1. Providing Feedback

#### Constructive Terminology Feedback

**✅ Good Review Comments:**
```
**Terminology**: This should use "anti-pattern" (hyphenated) in documentation 
to match our ubiquitous language standards. See the [glossary](docs/09-community/glossary.md#anti-pattern) 
for the standardized definition.

**Component Architecture**: Consider renaming `ConfigurationEngine` to `ConfigurationService` 
to align with our architectural patterns where "Engine" is reserved for core orchestrators. 
See [terminology mapping](docs/05-development/terminology-mapping.md#configuration-domain-mapping).

**Spelling Consistency**: Please use US spelling "analyze" instead of "analyse" 
to maintain consistency across the codebase.
```

**❌ Avoid These Review Comments:**
```
This is wrong, fix the naming.
Bad terminology here.
Use the right words.
```

#### Suggesting Improvements

**Template for Terminology Suggestions:**
```
**Terminology Suggestion**: 
- Current: `[current_usage]`
- Suggested: `[suggested_usage]`
- Reason: [explanation with reference to standards]
- Reference: [link to glossary or documentation]
```

### 2. Escalation Process

#### When to Escalate Terminology Issues

**Escalate if:**
- Multiple terminology violations across large PR
- Fundamental disagreement about domain concepts
- New terminology not covered in existing glossary
- Cross-cutting architectural terminology concerns

**Escalation Template:**
```
**Terminology Escalation Needed**

**Issue**: [Description of terminology concern]
**Impact**: [Potential confusion or inconsistency impact]
**Suggestion**: [Proposed resolution]
**Domain Expert Needed**: [Which domain expert should review]

cc: @domain-expert @tech-lead
```

## Tools and Automation Support

### 1. Automated Review Assistance

**Before Manual Review:**
```bash
# Run terminology linter on PR changes
python3 scripts/terminology-linter.py --pr-mode

# Check for common terminology violations  
grep -r "antipattern\|analyse\|analyser" --include="*.rs" src/

# Validate configuration terminology
python3 scripts/validate-config-terminology.py
```

### 2. Review Efficiency Tools

**Browser Extensions/Scripts:**
- Glossary quick-lookup for review comments
- Terminology highlighting in GitHub PR interface
- Quick links to domain documentation

### 3. Review Quality Metrics

**Track Review Effectiveness:**
- Terminology issues caught per review
- Time to identify terminology violations
- Consistency improvement over time
- Team adoption of terminology standards

## Common Review Scenarios

### Scenario 1: New Anti-Pattern Detector

**Review Focus:**
- Detector naming follows `{Pattern}Detector` pattern
- Anti-pattern terminology consistently applied
- Documentation uses domain vocabulary
- Test names reflect standardized concepts

**Quick Checklist:**
- [ ] Struct name ends with `Detector`
- [ ] Uses `anti_pattern` in code, "anti-pattern" in docs
- [ ] Implements `AnalysisDetector` trait
- [ ] Returns `ArchitecturalIssue` with proper `CriticalityLevel`

### Scenario 2: AI Integration Feature

**Review Focus:**
- AI-related terminology matches glossary definitions
- Provider vs Service distinction maintained
- Context building uses standardized vocabulary
- Prompt templates align with domain language

**Quick Checklist:**
- [ ] AI providers named with `Provider` suffix
- [ ] Uses "analysis" not "analyse" in all contexts
- [ ] Context builders use domain-appropriate terminology
- [ ] AI insights align with existing issue classification

### Scenario 3: Configuration Changes

**Review Focus:**
- Configuration parameter names use full terminology
- Documentation spells out "configuration"
- Code uses `Config` suffix appropriately
- Schema definitions match domain concepts

**Quick Checklist:**
- [ ] No abbreviated "config" in user documentation
- [ ] Parameter names reflect domain concepts clearly
- [ ] Configuration validation uses proper terminology
- [ ] Help text maintains vocabulary consistency

## Reviewer Training and Support

### 1. New Reviewer Onboarding

**Required Knowledge:**
- Complete glossary familiarity
- Understanding of architectural component patterns
- Knowledge of domain boundaries and terminology mapping
- Experience with common terminology violations

### 2. Ongoing Education

**Monthly Team Activities:**
- Terminology consistency review sessions
- Discussion of new domain concepts
- Review of terminology-related PRs as learning exercises
- Updates on glossary changes and new standards

### 3. Expert Support

**When to Consult Domain Experts:**
- Complex architectural terminology decisions
- Cross-domain interface naming questions
- New concept introduction requiring standardization
- Significant terminology refactoring proposals

## Success Metrics for Reviews

**Quantitative Metrics:**
- % of PRs with terminology violations decreasing over time
- Average time to identify terminology issues during review
- Number of terminology-related revisions per PR
- Team compliance rate with terminology standards

**Qualitative Metrics:**
- Improved clarity of code communication
- Reduced ambiguity in architectural discussions
- Enhanced onboarding experience for new team members
- Stronger alignment between code and documentation

This comprehensive review process ensures that Uveddi's ubiquitous language standards are consistently maintained and continuously improved through collaborative code review practices.