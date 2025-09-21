# 📋 PHASE 4 DETECTOR REFACTORING ASSIGNMENTS (04h-04n)

This document contains the complete development assignments for refactoring the remaining large detector files in the Uveddi codebase. Each assignment follows the established modular architecture pattern with strict file size limits and build verification requirements.

---

## **📋 ASSIGNMENT 04h: DEAD CODE DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/dead_code.rs`
- **Current Size**: 792 lines
- **Priority**: High (largest remaining security detector)

### **Objective**
Transform the monolithic dead code security detector into a modular, maintainable architecture that follows single responsibility principles and enables independent development of dead code detection components.

### **Required Modular Structure**
```
src/analysis/detectors/security/dead_code/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── detection/
│   ├── mod.rs                      (<100 lines)
│   ├── unreachable_code.rs         (<300 lines - unreachable code detection)
│   ├── unused_variables.rs         (<300 lines - unused variable detection)
│   ├── unused_functions.rs         (<300 lines - unused function detection)
│   ├── unused_imports.rs           (<300 lines - unused import detection)
│   └── unused_types.rs             (<300 lines - unused type detection)
├── analysis/
│   ├── mod.rs                      (<100 lines)
│   ├── control_flow.rs             (<300 lines - control flow analysis)
│   ├── data_flow.rs                (<300 lines - data flow analysis)
│   ├── dependency_graph.rs         (<300 lines - dependency tracking)
│   └── reachability.rs             (<300 lines - reachability analysis)
├── language_support/
│   ├── mod.rs                      (<100 lines)
│   ├── rust.rs                     (<300 lines - Rust-specific detection)
│   ├── python.rs                   (<300 lines - Python-specific detection)
│   └── javascript.rs               (<300 lines - JS/TS-specific detection)
└── metrics/
    ├── mod.rs                      (<100 lines)
    ├── coverage.rs                 (<300 lines - code coverage metrics)
    ├── complexity.rs               (<300 lines - complexity analysis)
    └── statistics.rs               (<300 lines - dead code statistics)
```

### **Functional Requirements**
1. **Dead Code Detection**: Maintain all existing dead code detection capabilities
2. **Multi-Language Support**: Preserve Rust, Python, and JavaScript/TypeScript analysis
3. **Control Flow Analysis**: Retain sophisticated control flow tracking
4. **Dependency Analysis**: Maintain dependency graph construction and analysis
5. **Metrics Collection**: Preserve code coverage and complexity metrics

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `dead_code.rs` after successful refactoring
- **Modular Architecture**: Clear separation of concerns across specialized modules
- **API Compatibility**: Maintain existing public interface for seamless integration

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📋 ASSIGNMENT 04i: CONFIG DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/config.rs`
- **Current Size**: 779 lines
- **Priority**: High (configuration security critical)

### **Objective**
Refactor the monolithic configuration security detector into a modular architecture that enables independent development of configuration parsing, validation, and security analysis components.

### **Required Modular Structure**
```
src/analysis/detectors/security/config/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── parsers/
│   ├── mod.rs                      (<100 lines)
│   ├── yaml_parser.rs              (<300 lines - YAML configuration parsing)
│   ├── json_parser.rs              (<300 lines - JSON configuration parsing)
│   ├── toml_parser.rs              (<300 lines - TOML configuration parsing)
│   ├── env_parser.rs               (<300 lines - environment variable parsing)
│   └── docker_parser.rs            (<300 lines - Docker configuration parsing)
├── validators/
│   ├── mod.rs                      (<100 lines)
│   ├── syntax_validator.rs         (<300 lines - configuration syntax validation)
│   ├── schema_validator.rs         (<300 lines - schema validation)
│   ├── value_validator.rs          (<300 lines - value range/type validation)
│   └── consistency_validator.rs    (<300 lines - cross-config consistency)
├── security_checks/
│   ├── mod.rs                      (<100 lines)
│   ├── secrets_detector.rs         (<300 lines - hardcoded secrets detection)
│   ├── permissions_analyzer.rs     (<300 lines - permission configuration analysis)
│   ├── network_security.rs         (<300 lines - network security settings)
│   ├── encryption_settings.rs      (<300 lines - encryption configuration)
│   └── authentication_config.rs    (<300 lines - auth configuration analysis)
└── language_support/
    ├── mod.rs                      (<100 lines)
    ├── rust.rs                     (<300 lines - Rust config analysis)
    ├── python.rs                   (<300 lines - Python config analysis)
    └── javascript.rs               (<300 lines - JS/TS config analysis)
```

### **Functional Requirements**
1. **Multi-Format Support**: Maintain YAML, JSON, TOML, environment variable parsing
2. **Security Analysis**: Preserve all security vulnerability detection in configurations
3. **Schema Validation**: Retain configuration schema validation capabilities
4. **Secret Detection**: Maintain hardcoded secret and credential detection
5. **Cross-Reference Analysis**: Preserve configuration consistency checking

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `config.rs` after successful refactoring
- **Modular Architecture**: Clear separation of parsing, validation, and security analysis
- **API Compatibility**: Maintain existing public interface

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📋 ASSIGNMENT 04j: OWASP DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/owasp.rs`
- **Current Size**: 719 lines
- **Priority**: High (OWASP Top 10 security critical)

### **Objective**
Transform the monolithic OWASP security detector into a modular architecture that organizes OWASP Top 10 vulnerability detection by category and enables independent development of specific vulnerability scanners.

### **Required Modular Structure**
```
src/analysis/detectors/security/owasp/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── categories/
│   ├── mod.rs                      (<100 lines)
│   ├── injection.rs                (<300 lines - A01 Injection vulnerabilities)
│   ├── broken_auth.rs              (<300 lines - A02 Broken Authentication)
│   ├── sensitive_data.rs           (<300 lines - A03 Sensitive Data Exposure)
│   ├── xxe.rs                      (<300 lines - A04 XML External Entities)
│   ├── broken_access.rs            (<300 lines - A05 Broken Access Control)
│   ├── security_misconfig.rs       (<300 lines - A06 Security Misconfiguration)
│   ├── xss.rs                      (<300 lines - A07 Cross-Site Scripting)
│   ├── insecure_deserialization.rs (<300 lines - A08 Insecure Deserialization)
│   └── vulnerable_components.rs    (<300 lines - A09 Vulnerable Components)
├── vulnerabilities/
│   ├── mod.rs                      (<100 lines)
│   ├── sql_injection.rs            (<300 lines - SQL injection detection)
│   ├── command_injection.rs        (<300 lines - command injection detection)
│   ├── path_traversal.rs           (<300 lines - path traversal detection)
│   ├── csrf.rs                     (<300 lines - CSRF vulnerability detection)
│   └── session_management.rs       (<300 lines - session security analysis)
├── scanners/
│   ├── mod.rs                      (<100 lines)
│   ├── static_scanner.rs           (<300 lines - static analysis scanner)
│   ├── pattern_scanner.rs          (<300 lines - pattern-based scanning)
│   ├── flow_scanner.rs             (<300 lines - data flow analysis scanner)
│   └── dependency_scanner.rs       (<300 lines - dependency vulnerability scanner)
└── language_support/
    ├── mod.rs                      (<100 lines)
    ├── rust.rs                     (<300 lines - Rust OWASP analysis)
    ├── python.rs                   (<300 lines - Python OWASP analysis)
    └── javascript.rs               (<300 lines - JS/TS OWASP analysis)
```

### **Functional Requirements**
1. **OWASP Top 10 Coverage**: Maintain complete OWASP Top 10 vulnerability detection
2. **Injection Detection**: Preserve SQL, NoSQL, OS command, and LDAP injection detection
3. **Authentication Analysis**: Retain broken authentication and session management detection
4. **Access Control**: Maintain broken access control vulnerability detection
5. **Data Protection**: Preserve sensitive data exposure and encryption analysis

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `owasp.rs` after successful refactoring
- **Modular Architecture**: Clear separation by OWASP categories and vulnerability types
- **API Compatibility**: Maintain existing public interface

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📋 ASSIGNMENT 04k: VALIDATION DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/validation.rs`
- **Current Size**: 717 lines
- **Priority**: High (input validation security critical)

### **Objective**
Refactor the monolithic validation security detector into a modular architecture that separates input validation, output validation, and sanitization concerns for maintainable security analysis.

### **Required Modular Structure**
```
src/analysis/detectors/security/validation/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── input_validation/
│   ├── mod.rs                      (<100 lines)
│   ├── parameter_validation.rs     (<300 lines - parameter validation analysis)
│   ├── form_validation.rs          (<300 lines - form input validation)
│   ├── api_validation.rs           (<300 lines - API input validation)
│   ├── file_validation.rs          (<300 lines - file upload validation)
│   └── data_type_validation.rs     (<300 lines - data type validation)
├── output_validation/
│   ├── mod.rs                      (<100 lines)
│   ├── response_validation.rs      (<300 lines - response validation)
│   ├── encoding_validation.rs      (<300 lines - output encoding validation)
│   ├── content_validation.rs       (<300 lines - content validation)
│   └── header_validation.rs        (<300 lines - HTTP header validation)
├── sanitizers/
│   ├── mod.rs                      (<100 lines)
│   ├── html_sanitizer.rs           (<300 lines - HTML sanitization analysis)
│   ├── sql_sanitizer.rs            (<300 lines - SQL sanitization analysis)
│   ├── path_sanitizer.rs           (<300 lines - path sanitization analysis)
│   ├── command_sanitizer.rs        (<300 lines - command sanitization analysis)
│   └── custom_sanitizer.rs         (<300 lines - custom sanitization patterns)
└── language_support/
    ├── mod.rs                      (<100 lines)
    ├── rust.rs                     (<300 lines - Rust validation analysis)
    ├── python.rs                   (<300 lines - Python validation analysis)
    └── javascript.rs               (<300 lines - JS/TS validation analysis)
```

### **Functional Requirements**
1. **Input Validation**: Maintain comprehensive input validation vulnerability detection
2. **Output Encoding**: Preserve output encoding and sanitization analysis
3. **Injection Prevention**: Retain injection prevention validation analysis
4. **Data Sanitization**: Maintain data sanitization pattern detection
5. **Framework Integration**: Preserve framework-specific validation analysis

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `validation.rs` after successful refactoring
- **Modular Architecture**: Clear separation of input, output, and sanitization concerns
- **API Compatibility**: Maintain existing public interface

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📋 ASSIGNMENT 04l: DEPENDENCY DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/dependency.rs`
- **Current Size**: 695 lines
- **Priority**: High (supply chain security critical)

### **Objective**
Transform the monolithic dependency security detector into a modular architecture that organizes dependency analysis, vulnerability detection, and license compliance checking into specialized components.

### **Required Modular Structure**
```
src/analysis/detectors/security/dependency/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── analyzers/
│   ├── mod.rs                      (<100 lines)
│   ├── dependency_graph.rs         (<300 lines - dependency graph analysis)
│   ├── version_analyzer.rs         (<300 lines - version analysis)
│   ├── circular_deps.rs            (<300 lines - circular dependency detection)
│   ├── outdated_deps.rs            (<300 lines - outdated dependency detection)
│   └── transitive_analyzer.rs      (<300 lines - transitive dependency analysis)
├── vulnerabilities/
│   ├── mod.rs                      (<100 lines)
│   ├── cve_scanner.rs              (<300 lines - CVE vulnerability scanning)
│   ├── advisory_scanner.rs         (<300 lines - security advisory scanning)
│   ├── malware_scanner.rs          (<300 lines - malware detection in dependencies)
│   ├── supply_chain.rs             (<300 lines - supply chain attack detection)
│   └── integrity_checker.rs        (<300 lines - package integrity verification)
├── licenses/
│   ├── mod.rs                      (<100 lines)
│   ├── license_analyzer.rs         (<300 lines - license compatibility analysis)
│   ├── compliance_checker.rs       (<300 lines - license compliance checking)
│   ├── conflict_detector.rs        (<300 lines - license conflict detection)
│   └── policy_enforcer.rs          (<300 lines - license policy enforcement)
└── language_support/
    ├── mod.rs                      (<100 lines)
    ├── rust.rs                     (<300 lines - Cargo.toml analysis)
    ├── python.rs                   (<300 lines - requirements.txt, setup.py analysis)
    └── javascript.rs               (<300 lines - package.json, yarn.lock analysis)
```

### **Functional Requirements**
1. **Vulnerability Detection**: Maintain CVE and security advisory vulnerability scanning
2. **License Compliance**: Preserve license compatibility and compliance analysis
3. **Dependency Graph**: Retain dependency graph construction and circular dependency detection
4. **Supply Chain Security**: Maintain supply chain attack and integrity checking
5. **Multi-Ecosystem**: Preserve support for Cargo, npm, pip, and other package managers

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `dependency.rs` after successful refactoring
- **Modular Architecture**: Clear separation of analysis, vulnerability, and license concerns
- **API Compatibility**: Maintain existing public interface

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📋 ASSIGNMENT 04m: CRYPTO DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/crypto.rs`
- **Current Size**: 611 lines
- **Priority**: High (cryptographic security critical)

### **Objective**
Refactor the monolithic cryptographic security detector into a modular architecture that organizes cryptographic algorithm analysis, implementation security, and vulnerability detection into specialized components.

### **Required Modular Structure**
```
src/analysis/detectors/security/crypto/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── algorithms/
│   ├── mod.rs                      (<100 lines)
│   ├── symmetric.rs                (<300 lines - symmetric encryption analysis)
│   ├── asymmetric.rs               (<300 lines - asymmetric encryption analysis)
│   ├── hashing.rs                  (<300 lines - hashing algorithm analysis)
│   ├── key_derivation.rs           (<300 lines - key derivation function analysis)
│   └── random_generation.rs        (<300 lines - random number generation analysis)
├── implementations/
│   ├── mod.rs                      (<100 lines)
│   ├── tls_analysis.rs             (<300 lines - TLS/SSL implementation analysis)
│   ├── certificate_analysis.rs     (<300 lines - certificate validation analysis)
│   ├── key_management.rs           (<300 lines - key management analysis)
│   ├── padding_analysis.rs         (<300 lines - padding scheme analysis)
│   └── mode_analysis.rs            (<300 lines - encryption mode analysis)
├── vulnerabilities/
│   ├── mod.rs                      (<100 lines)
│   ├── weak_crypto.rs              (<300 lines - weak cryptography detection)
│   ├── timing_attacks.rs           (<300 lines - timing attack vulnerability)
│   ├── side_channel.rs             (<300 lines - side-channel attack analysis)
│   ├── entropy_analysis.rs         (<300 lines - entropy and randomness analysis)
│   └── crypto_misuse.rs            (<300 lines - cryptographic misuse detection)
└── language_support/
    ├── mod.rs                      (<100 lines)
    ├── rust.rs                     (<300 lines - Rust crypto analysis)
    ├── python.rs                   (<300 lines - Python crypto analysis)
    └── javascript.rs               (<300 lines - JS/TS crypto analysis)
```

### **Functional Requirements**
1. **Algorithm Analysis**: Maintain analysis of symmetric, asymmetric, and hashing algorithms
2. **Implementation Security**: Preserve TLS, certificate, and key management analysis
3. **Vulnerability Detection**: Retain weak cryptography and side-channel attack detection
4. **Entropy Analysis**: Maintain random number generation and entropy analysis
5. **Standards Compliance**: Preserve cryptographic standards compliance checking

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `crypto.rs` after successful refactoring
- **Modular Architecture**: Clear separation of algorithms, implementations, and vulnerabilities
- **API Compatibility**: Maintain existing public interface

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📋 ASSIGNMENT 04n: SQL INJECTION DETECTOR REFACTORING**

### **Target File**
- **File**: `/src/analysis/detectors/security/sql_injection.rs`
- **Current Size**: 589 lines
- **Priority**: High (injection attack critical)

### **Objective**
Transform the monolithic SQL injection security detector into a modular architecture that organizes injection detection, pattern analysis, and sanitization validation into specialized components.

### **Required Modular Structure**
```
src/analysis/detectors/security/sql_injection/
├── mod.rs                          (<100 lines - public API & re-exports)
├── detector.rs                     (<300 lines - main detector implementation)
├── config.rs                       (<50 lines - configuration types)
├── types.rs                        (<300 lines - core data structures)
├── detection/
│   ├── mod.rs                      (<100 lines)
│   ├── query_analyzer.rs           (<300 lines - SQL query analysis)
│   ├── parameter_analyzer.rs       (<300 lines - parameter injection analysis)
│   ├── dynamic_query.rs            (<300 lines - dynamic query construction analysis)
│   ├── stored_procedure.rs         (<300 lines - stored procedure analysis)
│   └── orm_analyzer.rs             (<300 lines - ORM injection analysis)
├── patterns/
│   ├── mod.rs                      (<100 lines)
│   ├── injection_patterns.rs       (<300 lines - injection pattern detection)
│   ├── union_attacks.rs            (<300 lines - UNION-based attack detection)
│   ├── blind_injection.rs          (<300 lines - blind injection detection)
│   ├── time_based.rs               (<300 lines - time-based injection detection)
│   └── error_based.rs              (<300 lines - error-based injection detection)
├── sanitizers/
│   ├── mod.rs                      (<100 lines)
│   ├── parameterization.rs         (<300 lines - parameterized query analysis)
│   ├── escape_analysis.rs          (<300 lines - escape sequence analysis)
│   ├── whitelist_validation.rs     (<300 lines - whitelist validation analysis)
│   ├── input_validation.rs         (<300 lines - input validation analysis)
│   └── output_encoding.rs          (<300 lines - output encoding analysis)
└── language_support/
    ├── mod.rs                      (<100 lines)
    ├── rust.rs                     (<300 lines - Rust SQL analysis)
    ├── python.rs                   (<300 lines - Python SQL analysis)
    └── javascript.rs               (<300 lines - JS/TS SQL analysis)
```

### **Functional Requirements**
1. **Injection Detection**: Maintain comprehensive SQL injection vulnerability detection
2. **Pattern Analysis**: Preserve injection pattern and attack vector analysis
3. **Query Analysis**: Retain dynamic query construction and parameter analysis
4. **Sanitization Validation**: Maintain sanitization and parameterization analysis
5. **Framework Support**: Preserve ORM and framework-specific injection detection

### **Quality Standards**
- **File Size Limit**: ALL files must be <300 lines (mod.rs <100 lines, config.rs <50 lines)
- **Build Success**: Must compile without errors using `cargo check --features dev-core`
- **Original File Deletion**: Remove original `sql_injection.rs` after successful refactoring
- **Modular Architecture**: Clear separation of detection, patterns, and sanitization
- **API Compatibility**: Maintain existing public interface

### **Deliverables**
1. Complete modular directory structure as specified above
2. Successful compilation with zero errors
3. Verification that all files meet size requirements
4. Confirmation of original file deletion
5. Basic functionality tests within modules

---

## **📊 ASSIGNMENT SUMMARY**

### **Remaining Assignments**: 8 detector refactoring tasks
### **Total Lines to Refactor**: ~5,501 lines
### **Expected Modules**: ~160 specialized modules
### **Completion Timeline**: Sequential execution after Assignment 04g resolution

### **Success Criteria for All Assignments**
1. ✅ **File Size Compliance**: 100% of files under 300 lines
2. ✅ **Build Success**: Zero compilation errors
3. ✅ **Modular Architecture**: Clear separation of concerns
4. ✅ **API Compatibility**: Maintained public interfaces
5. ✅ **Original File Cleanup**: All original files deleted

**Status**: Ready for sequential execution upon Assignment 04g completion and approval.