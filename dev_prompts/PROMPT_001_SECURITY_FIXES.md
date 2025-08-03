# GPT Dev Prompt 001: Critical Security Vulnerabilities Fix

## 🚨 CRITICAL PRIORITY - Week 1 Implementation

### **Issue**: UV-103 - Critical Security Vulnerabilities
**Reference Document**: `SECURITY_FIXES_PLAN.md`  
**Jira Issue**: UV-103  
**Priority**: CRITICAL - Production Blocker  
**Estimated Time**: 3-5 days  

---

## **TASK OVERVIEW**

You need to fix critical security vulnerabilities that are blocking production deployment. The main issues are:

1. **RSA Timing Attack** (RUSTSEC-2023-0071) - Critical
2. **JWT Security Hardening** - High Priority  
3. **Unmaintained Dependencies** - Medium Priority

---

## **DETAILED REQUIREMENTS**

### **1. RSA Timing Attack Mitigation (Days 1-2)**

**Problem**: The `rsa` crate has a timing sidechannel vulnerability that could allow key recovery.

**Current Vulnerable Code Location**: 
- `src/security/authentication.rs` - JWT validation functions
- Any RSA cryptographic operations

**Required Implementation**:

```rust
// Add to src/security/authentication.rs
impl AuthenticationService {
    /// Secure JWT validation with timing attack protection
    pub async fn authenticate_jwt_secure(&self, token: &str) -> SecurityResult<AuthenticatedUser> {
        let start_time = std::time::Instant::now();
        
        // Perform actual validation
        let result = self.authenticate_jwt_internal(token).await;
        
        // Add consistent timing to prevent timing attacks
        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(50); // Minimum processing time
        
        if elapsed < target_duration {
            let delay = target_duration - elapsed;
            tokio::time::sleep(delay).await;
        }
        
        result
    }
    
    /// Constant-time string comparison for sensitive operations
    fn constant_time_compare(a: &str, b: &str) -> bool {
        use subtle::ConstantTimeEq;
        a.as_bytes().ct_eq(b.as_bytes()).into()
    }
}
```

**Tasks**:
- [ ] Implement timing randomization for JWT operations
- [ ] Add constant-time comparison for sensitive operations  
- [ ] Update all authentication calls to use secure variants
- [ ] Add timing attack monitoring metrics

### **2. JWT Security Hardening (Days 2-3)**

**Problem**: JWT configuration uses weak defaults and may have hardcoded secrets.

**Current Issues**:
- Default JWT secrets in development
- No JWT token rotation mechanism
- Missing JWT blacklisting capability

**Required Implementation**:

```rust
// Enhance src/security/authentication.rs
pub struct JwtManager {
    current_key: String,
    previous_key: Option<String>, // For graceful key rotation
    blacklisted_tokens: Arc<RwLock<HashSet<String>>>,
}

impl JwtManager {
    /// Rotate JWT signing key
    pub async fn rotate_key(&mut self) -> SecurityResult<()> {
        self.previous_key = Some(self.current_key.clone());
        self.current_key = self.generate_secure_key();
        
        // Schedule cleanup of previous key after rotation period
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_hours(24)).await;
            // Clean previous key
        });
        
        Ok(())
    }
    
    /// Blacklist a JWT token
    pub async fn blacklist_token(&self, token: &str) -> SecurityResult<()> {
        let jti = self.extract_jti(token)?;
        self.blacklisted_tokens.write().await.insert(jti);
        Ok(())
    }
    
    /// Check if token is blacklisted
    pub async fn is_blacklisted(&self, token: &str) -> bool {
        if let Ok(jti) = self.extract_jti(token) {
            self.blacklisted_tokens.read().await.contains(&jti)
        } else {
            true // Invalid tokens are considered blacklisted
        }
    }
}
```

**Tasks**:
- [ ] Implement JWT key rotation mechanism
- [ ] Add JWT token blacklisting
- [ ] Remove any hardcoded JWT secrets
- [ ] Enhance key derivation with stronger algorithms
- [ ] Add JWT expiration validation

### **3. Unmaintained Dependencies Fix (Days 3-4)**

**Problem**: Several dependencies are unmaintained and pose security risks.

**Dependencies to Replace**:
- `instant` → `std::time::Instant` (built-in)
- `paste` → `quote` + `proc-macro2`
- Update `arrow` dependencies to latest versions

**Required Changes**:

```toml
# Update Cargo.toml
[dependencies]
# Remove unmaintained crates
# instant = "0.1.13"  # REMOVE - use std::time::Instant
# paste = "1.0.15"    # REMOVE - replace with alternatives

# Updated maintained alternatives
quote = "1.0"
proc-macro2 = "1.0"
arrow = "52.2.0"  # Update to latest version

# Security patches
[patch.crates-io]
# Force updates for vulnerable dependencies
lexical-core = "0.8.6"  # Fix soundness issues
```

**Tasks**:
- [ ] Replace `instant` with `std::time::Instant` throughout codebase
- [ ] Replace `paste` macro usage with `quote` alternatives
- [ ] Update arrow dependencies to latest secure versions
- [ ] Run comprehensive security audit after changes
- [ ] Update all usage sites

---

## **IMPLEMENTATION STEPS**

### **Day 1: RSA Timing Attack Mitigation**
1. **Analyze Current RSA Usage**:
   ```bash
   grep -r "rsa::" src/ --include="*.rs"
   grep -r "authenticate_jwt" src/ --include="*.rs"
   ```

2. **Implement Timing Protection**:
   - Add timing randomization to authentication functions
   - Implement constant-time comparison utilities
   - Add timing metrics collection

3. **Test Implementation**:
   ```rust
   #[cfg(test)]
   mod timing_tests {
       #[tokio::test]
       async fn test_consistent_timing() {
           // Test that valid/invalid tokens have consistent timing
       }
   }
   ```

### **Day 2: JWT Security Enhancement**
1. **Implement JWT Manager**:
   - Create JwtManager struct with rotation capability
   - Add blacklisting functionality
   - Implement secure key generation

2. **Update Authentication Service**:
   - Replace direct JWT usage with JwtManager
   - Add token validation enhancements
   - Implement graceful key rotation

### **Day 3: Dependency Updates**
1. **Replace Unmaintained Crates**:
   - Update all `instant` usage to `std::time::Instant`
   - Replace `paste` macros with `quote` alternatives
   - Update arrow dependencies

2. **Security Validation**:
   ```bash
   cargo audit
   cargo deny check
   ```

### **Day 4: Testing and Validation**
1. **Comprehensive Security Tests**:
   - Timing attack resistance tests
   - JWT security validation tests
   - Dependency vulnerability scans

2. **Performance Validation**:
   - Ensure no significant performance regression
   - Validate security measures don't impact user experience

---

## **ACCEPTANCE CRITERIA**

### **Security Requirements**:
- [ ] Zero critical security vulnerabilities in `cargo audit`
- [ ] JWT timing attack resistance verified through testing
- [ ] All hardcoded secrets eliminated
- [ ] JWT rotation mechanism functional
- [ ] Token blacklisting capability implemented

### **Technical Requirements**:
- [ ] All tests pass including new security tests
- [ ] No performance regression >5%
- [ ] Documentation updated for new security features
- [ ] Security configuration properly externalized

### **Code Quality**:
- [ ] All security functions have comprehensive tests
- [ ] Error handling follows established patterns
- [ ] Code follows Rust security best practices
- [ ] Proper logging for security events

---

## **TESTING REQUIREMENTS**

### **Security Tests to Implement**:

```rust
// tests/security/timing_attack_tests.rs
#[tokio::test]
async fn test_jwt_timing_consistency() {
    // Verify consistent timing for valid/invalid tokens
}

#[tokio::test]
async fn test_constant_time_comparison() {
    // Verify constant-time string comparison
}

// tests/security/jwt_security_tests.rs
#[tokio::test]
async fn test_jwt_rotation() {
    // Test JWT key rotation functionality
}

#[tokio::test]
async fn test_jwt_blacklisting() {
    // Test token blacklisting capability
}
```

### **Security Validation Commands**:
```bash
# Run after implementation
cargo audit --deny warnings
cargo test --test security_tests
cargo bench --bench security_benchmarks
```

---

## **DOCUMENTATION UPDATES REQUIRED**

1. **Security Configuration Documentation**:
   - Update authentication configuration examples
   - Document JWT rotation procedures
   - Add security hardening guidelines

2. **API Documentation**:
   - Document new secure authentication methods
   - Add security considerations to API docs
   - Update deployment security guides

---

## **ROLLBACK PLAN**

If issues arise during implementation:

1. **Immediate Rollback**: Revert to commit before security changes
2. **Partial Rollback**: Disable new security features via feature flags
3. **Emergency Patch**: Apply minimal fixes to address critical issues

---

## **POST-IMPLEMENTATION VERIFICATION**

After implementation, run these verification steps:

```bash
# Security audit
cargo audit

# Comprehensive test suite
cargo test --all-features

# Performance benchmarks
cargo bench

# Security-specific tests
cargo test --test security_tests -- --nocapture
```

---

## **ADDITIONAL RESOURCES**

- **Reference**: `SECURITY_FIXES_PLAN.md` (created earlier)
- **Security Guide**: `docs/11-security/SECURITY_HARDENING_GUIDE.md`
- **Authentication Docs**: `src/security/authentication.rs` documentation
- **Jira Issue**: UV-103 for tracking and updates

---

## **SUCCESS METRICS**

Upon completion, you should achieve:
- ✅ **Zero critical security vulnerabilities**
- ✅ **JWT timing attack protection implemented**
- ✅ **All unmaintained dependencies replaced**
- ✅ **Comprehensive security test coverage**
- ✅ **Documentation updated and complete**

**This is a CRITICAL security fix - production deployment is blocked until completion.**
