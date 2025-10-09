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