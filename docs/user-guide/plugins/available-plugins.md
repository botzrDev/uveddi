# Available Plugins

This document lists all officially supported and community plugins available for Uveddi. These plugins extend the analysis capabilities beyond the built-in detectors.

## Official Plugins

### Performance Analysis Plugins

#### Performance Analyzer Plugin

**Plugin ID**: `performance-analyzer`  
**Version**: `1.0.0`  
**Status**: Production Ready

**Description**: Identifies performance bottlenecks and optimization opportunities in your codebase.

**Key Features**:
- Detects nested loops and complexity issues
- Identifies memory allocation patterns in hot paths
- Analyzes string concatenation inefficiencies
- Flags synchronous I/O operations
- Calculates performance scores and hotspot metrics

**Technologies**:
- Advanced AST analysis with tree-sitter queries
- Complex metrics calculation (Cyclomatic complexity, performance scoring)
- Host function usage for AST parsing
- Comprehensive error handling

**Detection Rules**:
- `PERF001`: Nested loops detection
- `PERF002`: Memory allocation in loops
- `PERF003`: String concatenation in loops
- `PERF004`: Synchronous file I/O detection

**Supported Languages**: Rust, JavaScript, TypeScript, Python, Java, Go

**Installation**:
```bash
uveddi plugin install performance-analyzer
```

**Configuration Example**:
```toml
[plugins.performance-analyzer]
complexity_threshold = 15
memory_analysis = true
io_analysis = true
```

---

### Security Plugins

#### Security Vulnerability Scanner

**Plugin ID**: `security-scanner`  
**Version**: `2.1.0`  
**Status**: Production Ready

**Description**: Comprehensive security analysis and vulnerability detection across multiple languages.

**Key Features**:
- Pattern-based vulnerability detection (SQL injection, XSS, etc.)
- Hardcoded secrets and API key detection
- Weak cryptography algorithm identification
- Dangerous function call analysis
- Dependency vulnerability scanning with network integration

**Technologies**:
- Network access permissions for CVE database lookups
- Database caching for vulnerability data
- Multi-language security pattern matching
- Integration with external security APIs
- Advanced regex pattern matching

**Detection Rules**:
- `SEC001-SEC010`: Complete OWASP-aligned security rule set
- CWE mapping for industry compliance
- Vulnerability severity scoring

**Supported Languages**: All supported languages

**Installation**:
```bash
uveddi plugin install security-scanner
```

**Configuration Example**:
```toml
[plugins.security-scanner]
strict_mode = true
cve_database_url = "https://api.cve.org"
ignore_patterns = ["**/test/**"]
severity_threshold = "medium"
```

---

### Code Quality Plugins

#### Code Complexity Analyzer

**Plugin ID**: `complexity-analyzer`  
**Version**: `1.2.0`  
**Status**: Production Ready

**Description**: Multi-metric complexity analysis with industry-standard calculations.

**Key Features**:
- Cyclomatic complexity calculation
- Cognitive complexity assessment
- Halstead metrics computation
- Function length and parameter count analysis
- Maintainability index calculation

**Technologies**:
- Sophisticated AST traversal algorithms
- Mathematical metric calculations
- Multi-dimensional analysis scoring
- Function-level granular reporting
- Statistical analysis and aggregation

**Detection Rules**:
- `COMP001`: High cyclomatic complexity
- `COMP002`: High cognitive complexity
- `COMP003`: Long function detection
- `COMP004`: Too many parameters
- `COMP005`: Deep nesting analysis

**Supported Languages**: Rust, JavaScript, TypeScript, Python, Java, Go

**Installation**:
```bash
uveddi plugin install complexity-analyzer
```

**Configuration Example**:
```toml
[plugins.complexity-analyzer]
cyclomatic_threshold = 10
cognitive_threshold = 15
max_function_length = 50
max_parameters = 5
```

---

### Dependency Management Plugins

#### Dependency Vulnerability Tracker

**Plugin ID**: `dependency-tracker`  
**Version**: `1.3.0`  
**Status**: Production Ready

**Description**: Cross-platform dependency management and vulnerability tracking.

**Key Features**:
- Multi-package manager support (Cargo, npm, pip, Go modules)
- Real-time vulnerability database integration
- License compliance checking
- Outdated dependency detection
- Software Bill of Materials (SBOM) generation

**Technologies**:
- Network integration with multiple package registries
- Complex data parsing (TOML, JSON, requirements files)
- Caching strategies for performance
- Asynchronous processing patterns
- Supply chain security analysis

**Detection Rules**:
- `DEP001`: Vulnerable dependencies
- `DEP002`: Outdated packages
- `DEP003`: Deprecated dependencies
- `DEP004`: License compliance violations
- `DEP005`: Supply chain risks

**Supported Package Managers**: Cargo, npm, pip, Go modules, Maven, Gradle

**Installation**:
```bash
uveddi plugin install dependency-tracker
```

**Configuration Example**:
```toml
[plugins.dependency-tracker]
check_vulnerabilities = true
check_licenses = true
allowed_licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"]
vulnerability_threshold = "medium"
```

---

### Documentation Plugins

#### Documentation Analyzer

**Plugin ID**: `documentation-analyzer`  
**Version**: `1.0.0`  
**Status**: Production Ready

**Description**: Documentation coverage and quality analysis.

**Key Features**:
- Multi-language documentation pattern recognition
- Coverage calculation for functions, classes, modules
- Documentation quality scoring
- Missing documentation identification
- README and project documentation validation

**Technologies**:
- File system operations with host functions
- Multi-language parsing strategies
- Quality metric algorithms
- Documentation standard compliance
- Text analysis and pattern recognition

**Detection Rules**:
- `DOC001-DOC008`: Comprehensive documentation rule set
- Coverage thresholds and quality gates
- Style guide enforcement

**Supported Languages**: Rust, JavaScript, TypeScript, Python, Java, Go

**Installation**:
```bash
uveddi plugin install documentation-analyzer
```

**Configuration Example**:
```toml
[plugins.documentation-analyzer]
coverage_threshold = 80
require_examples = true
check_readme = true
style_guide = "standard"
```

---

## Community Plugins

### Framework-Specific Plugins

#### React Analyzer
- **Plugin ID**: `react-analyzer`
- **Description**: React-specific patterns and best practices
- **Features**: Hook usage, component structure, performance patterns
- **Status**: Community Maintained

#### Vue.js Analyzer
- **Plugin ID**: `vue-analyzer`
- **Description**: Vue.js specific analysis and patterns
- **Features**: Template analysis, composition API patterns
- **Status**: Community Maintained

#### Angular Analyzer
- **Plugin ID**: `angular-analyzer`
- **Description**: Angular-specific patterns and architecture analysis
- **Features**: Dependency injection, service patterns, component analysis
- **Status**: Community Maintained

### Language-Specific Plugins

#### Go Advanced Analyzer
- **Plugin ID**: `go-advanced`
- **Description**: Advanced Go-specific analysis beyond built-in support
- **Features**: Goroutine analysis, channel patterns, interface compliance
- **Status**: Community Maintained

#### Python Data Science Analyzer
- **Plugin ID**: `python-datascience`
- **Description**: Analysis for data science and ML code patterns
- **Features**: Pandas usage patterns, NumPy optimizations, ML model structure
- **Status**: Community Maintained

### Enterprise Plugins

#### Enterprise Compliance Checker
- **Plugin ID**: `enterprise-compliance`
- **Description**: Enterprise-specific compliance and governance rules
- **Features**: Custom policy enforcement, audit trails, reporting
- **Status**: Premium/Enterprise

#### GDPR Compliance Analyzer
- **Plugin ID**: `gdpr-compliance`
- **Description**: GDPR compliance checking for data handling
- **Features**: Data flow analysis, privacy impact assessment
- **Status**: Premium/Enterprise

## Plugin Installation and Usage

### Installation Commands

```bash
# Install from official registry
uveddi plugin install <plugin-id>

# Install specific version
uveddi plugin install <plugin-id>@<version>

# Install from URL
uveddi plugin install https://plugins.uveddi.io/<plugin-id>.wasm

# Install from local file
uveddi plugin install ./path/to/plugin.wasm
```

### Usage in Analysis

```bash
# Use all enabled plugins
uveddi analyze ./src

# Use specific plugins
uveddi analyze ./src --plugins performance-analyzer,security-scanner

# Exclude specific plugins
uveddi analyze ./src --exclude-plugins deprecated-detector
```

## Plugin Compatibility Matrix

| Plugin | Rust | JavaScript | TypeScript | Python | Java | Go | Status |
|--------|------|------------|------------|--------|------|----|--------|
| performance-analyzer | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Stable |
| security-scanner | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Stable |
| complexity-analyzer | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Stable |
| dependency-tracker | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Stable |
| documentation-analyzer | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Stable |
| react-analyzer | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | Beta |
| vue-analyzer | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | Beta |
| angular-analyzer | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | Beta |
| go-advanced | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Beta |
| python-datascience | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | Beta |

## Plugin Categories

### By Function
- **Security**: Vulnerability detection, compliance checking
- **Performance**: Bottleneck detection, optimization suggestions
- **Quality**: Code metrics, maintainability analysis
- **Documentation**: Coverage analysis, quality checking
- **Dependencies**: Vulnerability tracking, license compliance

### By Scope
- **Language-Specific**: Tailored for specific programming languages
- **Framework-Specific**: Specialized for particular frameworks
- **Domain-Specific**: Focused on specific problem domains
- **Enterprise**: Organizational policies and compliance
- **General Purpose**: Broad applicability across projects

## Plugin Development

Interested in creating your own plugin? See:

- [Plugin Development Guide](../../development/plugins/development-guide.md)
- [API Reference](../../development/plugins/api-reference.md)
- [Plugin Examples](../../development/plugins/examples.md)

## Support and Community

### Getting Help
- **Documentation**: Each plugin includes comprehensive documentation
- **Community Forum**: Join discussions about specific plugins
- **GitHub Issues**: Report bugs or request features
- **Plugin Registry**: Browse and contribute to the plugin ecosystem

### Contributing
- **Plugin Submission**: Submit your plugin to the community registry
- **Bug Reports**: Help improve existing plugins
- **Feature Requests**: Suggest improvements for existing plugins
- **Documentation**: Help improve plugin documentation

## Roadmap

### Upcoming Official Plugins
- **AI Code Quality Linter**: Advanced AI-powered code analysis
- **Custom Rule Engine**: User-defined rules and policies
- **Advanced Metrics Visualization**: Enhanced reporting and visualization
- **Database Schema Analyzer**: Database design pattern analysis
- **API Design Analyzer**: REST API design best practices

### Community Requests
- **Kotlin Support**: Kotlin-specific analysis patterns
- **Swift Analyzer**: iOS/macOS development patterns  
- **Docker/Container Analysis**: Container best practices
- **Terraform Analyzer**: Infrastructure-as-code analysis
- **GraphQL Schema Analyzer**: GraphQL API analysis

For the most up-to-date information about available plugins, visit the [Plugin Registry](https://plugins.uveddi.io) or use:

```bash
uveddi plugin search --all
```