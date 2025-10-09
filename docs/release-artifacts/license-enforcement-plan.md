# Uveddi License Enforcement Implementation Plan

**Date:** 2025-10-09  
**Assignment:** A4 - Security & Compliance Readiness  
**Status:** Documented for Future Implementation  
**Priority:** Post-1.0.0 (Q1 2026)

## Executive Summary

Uveddi 1.0.0 will ship with **manual license verification** as the primary enforcement mechanism for the commercial release. This decision memo documents the current state, implementation plan for automated enforcement, and interim customer workflows.

---

## Current State Assessment

### Existing Implementation

**Status:** ❌ Not Implemented

**Current Code Search Results:**
- ✅ Environment variable placeholder documented in README.md
- ❌ No license key validation logic in codebase
- ❌ No `UVEDDI_LICENSE_KEY` environment variable handling
- ❌ No license enforcement at CLI startup
- ❌ No license check API or service

**Mentions in Codebase:**
1. `README.md` (Line 53): "If your deployment uses license keys, add `UVEDDI_LICENSE_KEY=<key>` to your environment. *License enforcement tooling ships separately.*"
2. `ASSIGNMENT-A4-SECURITY-COMPLIANCE.md`: Task requirement
3. API key authentication exists in `src/security/authentication.rs` (different purpose - API access, not license)

### API Key vs License Key Distinction

**Existing:** `src/security/authentication.rs` implements API key authentication
- Format: `uvd_` + 8 chars + `_` + 32 chars (total 45 chars)
- Purpose: Authenticating API requests (if API mode is enabled)
- Scope: Per-request authentication for server/API modes

**Required:** License key enforcement (not yet implemented)
- Purpose: Validate commercial subscription entitlement
- Scope: CLI startup validation for commercial use authorization
- Enforcement point: Before analysis execution

---

## Implementation Decision: Manual Verification (1.0.0)

### Rationale

**Why Defer Automated Enforcement:**

1. **Time to Market:** Automated enforcement would delay 1.0.0 release by ~2-3 weeks
2. **Customer Experience:** Early adopters are trusted enterprise customers with established relationships
3. **Support Burden:** Manual process allows personalized onboarding and support
4. **Architecture Maturity:** License server infrastructure not yet deployed
5. **Iteration Opportunity:** Gather customer feedback before committing to enforcement UX

**Risk Mitigation:**
- Limited distribution (sales@uveddi.com contact required)
- Small initial customer base (manageable manual process)
- Contract-based agreements (legal enforcement layer)
- Monitoring for misuse via support channels

**Approval:** Solo developer decision; would escalate to legal/product if team structure existed

---

## Interim Customer Workflow (1.0.0)

### Manual License Verification Process

**For Subscribers:**

1. **Purchase/Renewal:**
   - Contact sales@uveddi.com (assumed contact; update with actual)
   - Receive subscription confirmation email with:
     - Subscription tier (weekly $9 or annual $300)
     - Billing period
     - Support contact information
     - License terms document

2. **Installation:**
   - Clone repository or download signed binary (subscriber access)
   - Build from source: `cargo build --release --features cli-standard`
   - Verify binary signature (see signing checklist)

3. **Usage:**
   - No technical enforcement in 1.0.0
   - Honor system backed by contract agreement
   - Support tickets require valid subscription ID

4. **Support:**
   - Priority email: support@uveddi.com
   - Subscription verification during support intake
   - Access to updates/patches requires active subscription

### Documentation Updates Required

**README.md Section:** (Already contains placeholder, will enhance)
```markdown
## Installation & subscription

1. **Activate your subscription** – Contact sales (sales@uveddi.com) to provision a weekly or annual license.
2. **Install Rust** – Rust 1.70+ is required (`rustup` recommended).
3. **Build from source or request binaries** – Subscribers may request signed binaries, or build from source using `cargo build --release --features cli-standard`.
4. **License verification** – Your subscription is verified during support intake. Technical enforcement will be introduced in a future release.

**Commercial use requires an active subscription.** See LICENSE for terms.
```

**LICENSE File Update:**
Add commercial use clause:
```
Commercial Use Clause:
Commercial use of this software requires an active paid subscription from botzrDev.
Contact sales@uveddi.com for licensing inquiries.
```

---

## Automated Enforcement Implementation Plan

### Phase 1: License Server Infrastructure (Q1 2026)

**Deliverables:**
- [ ] License management SaaS or self-hosted service
- [ ] License key generation API
- [ ] License validation API endpoint
- [ ] Customer portal for license management
- [ ] Admin dashboard for sales/support

**Tech Stack (Proposed):**
- Backend: Rust (Axum) or Go
- Database: PostgreSQL
- Authentication: JWT tokens
- Hosting: AWS/GCP/Azure (TBD based on ops requirements)

**License Key Format:**
```
UVEDDI-{VERSION}-{CUSTOMER_ID}-{SIGNATURE}
Example: UVEDDI-1-ABC123XYZ-9f8e7d6c5b4a3

Components:
- VERSION: License schema version (1)
- CUSTOMER_ID: Base62-encoded customer identifier
- SIGNATURE: HMAC-SHA256 signature for validation
```

### Phase 2: CLI Integration (Q1 2026)

**Implementation Points:**

1. **Startup Validation** (`src/cli/mod.rs` or `src/main.rs`):
```rust
// Pseudocode - not implemented
async fn validate_license() -> Result<(), UveddiError> {
    // 1. Check for UVEDDI_LICENSE_KEY environment variable
    let license_key = env::var("UVEDDI_LICENSE_KEY")
        .or_else(|_| load_from_config_file())?;
    
    // 2. Parse license key format
    let license = LicenseKey::parse(&license_key)?;
    
    // 3. Validate signature (offline)
    license.verify_signature()?;
    
    // 4. Check expiration (offline)
    if license.is_expired() {
        return Err(UveddiError::LicenseExpired);
    }
    
    // 5. Optional: Online validation (with grace period)
    if should_phone_home() {
        validate_license_online(&license).await?;
    }
    
    Ok(())
}
```

2. **Grace Period Logic:**
   - Allow 30 days without online validation
   - Cache last successful validation
   - Offline mode for air-gapped environments (enterprise tier)

3. **User Experience:**
```bash
$ uveddi analyze ./project
ERROR: License key not found or expired

Your Uveddi license is not active. Please check:
1. Set UVEDDI_LICENSE_KEY environment variable
2. Ensure subscription is active: https://uveddi.com/account
3. Contact support: support@uveddi.com

Trial mode: Run 'uveddi trial start' for 14-day evaluation
```

**Configuration Options:**
- `~/.config/uveddi/license.toml` for persistent storage
- Environment variable override
- CI/CD-friendly (env var only, no interactive prompts)

### Phase 3: Enforcement Policies (Q1-Q2 2026)

**Policy Options:**

1. **Soft Enforcement** (Recommended Start):
   - Warning messages for expired/invalid licenses
   - Functional CLI with reduced features
   - Telemetry reporting (opt-in with consent)
   - Grace periods for renewals

2. **Hard Enforcement** (Future Enterprise):
   - Binary exits on invalid license
   - No analysis without valid key
   - Suitable for enterprise contracts with guaranteed uptime

3. **Tiered Enforcement:**
   - Weekly subscribers: 7-day grace period
   - Annual subscribers: 30-day grace period
   - Enterprise: Custom SLAs with offline support

**Feature Gating Examples:**
- Core analysis: Always available (limited without license)
- AI integrations: Requires valid license
- Security scanning: Requires valid license
- Plugin system: Requires valid license
- HTML reports: Requires valid license

---

## Technical Specifications (Future Implementation)

### License Key Validation

**Offline Validation:**
```rust
// Cryptographic signature verification
pub struct LicenseKey {
    pub version: u8,
    pub customer_id: String,
    pub tier: SubscriptionTier,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub features: Vec<Feature>,
    pub signature: Vec<u8>,
}

impl LicenseKey {
    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<(), LicenseError> {
        let payload = format!("{}{}{}{}", 
            self.version, self.customer_id, self.issued_at, self.expires_at);
        
        public_key.verify(payload.as_bytes(), &self.signature)
            .map_err(|_| LicenseError::InvalidSignature)?;
        
        Ok(())
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
```

**Online Validation:**
```rust
// Periodic phone-home validation
pub async fn validate_license_online(license: &LicenseKey) -> Result<LicenseStatus, LicenseError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.uveddi.com/v1/license/validate")
        .json(&json!({
            "license_key": license.to_string(),
            "machine_id": get_machine_id()?,
            "version": env!("CARGO_PKG_VERSION"),
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let status: LicenseStatus = response.json().await?;
    
    // Cache validation result
    cache_validation_result(&status).await?;
    
    Ok(status)
}
```

### Storage & Caching

**License Cache:**
- Location: `~/.cache/uveddi/license.dat`
- Format: Encrypted binary (ChaCha20-Poly1305)
- Contents: Last validation timestamp, grace period remaining
- Lifetime: 30 days max

**Configuration File:**
```toml
# ~/.config/uveddi/license.toml
[license]
key = "UVEDDI-1-ABC123XYZ-9f8e7d6c5b4a3"

[validation]
phone_home_interval = "7d"  # Weekly check
grace_period = "30d"
offline_mode = false  # Enterprise only

[telemetry]
enabled = false  # Opt-in
anonymous_usage = true
error_reporting = false
```

### Security Considerations

1. **Key Protection:**
   - Never log license keys
   - Redact from error messages
   - Secure storage with OS keychain integration (future)

2. **Anti-Tampering:**
   - Binary signature verification
   - License key embedded in binary hash (future)
   - Audit logging for suspicious activity

3. **Privacy:**
   - Minimal data collection
   - Clear opt-in for telemetry
   - GDPR/CCPA compliance considerations

---

## Migration Path (1.0.0 → 2.0.0)

### Customer Communication

**Timeline:**
1. **2025-11-01:** License enforcement announcement in release notes
2. **2025-12-01:** Beta testing with opt-in customers
3. **2026-01-15:** Enforcement enabled for new subscriptions
4. **2026-02-01:** Enforcement enabled for all subscriptions (with grace period)

**Communication Channels:**
- Release notes (detailed implementation guide)
- Email to all subscribers (90-day notice)
- In-app messages (version update prompts)
- Documentation updates

### Backwards Compatibility

**Versions Supporting Manual Flow:**
- 1.0.0 → 1.x.x: Manual verification only
- 2.0.0+: Automated enforcement (with graceful fallback)

**Deprecation Plan:**
- 1.x.x maintained for 12 months after 2.0.0 release
- Security patches only after 6 months
- End-of-life: 2026-12-31

---

## Cost-Benefit Analysis

### Automated Enforcement

**Benefits:**
- Reduced manual verification overhead
- Scalable for larger customer base
- Clear audit trail for compliance
- Automated renewal reminders

**Costs:**
- Development: ~3-4 weeks (license server + CLI integration)
- Infrastructure: $50-200/month (hosting, database, monitoring)
- Maintenance: Ongoing support burden
- Customer friction: Potential onboarding complaints

### Manual Verification (Chosen for 1.0.0)

**Benefits:**
- Immediate release (no development delay)
- Personalized customer relationships
- Flexible enforcement during early adoption
- Lower infrastructure costs

**Costs:**
- Manual support overhead (~2-5 hours/week for 10-20 customers)
- Risk of unauthorized use (mitigated by contract + small distribution)
- Scaling limitation (not viable beyond 100 customers)

**Decision:** Manual verification is appropriate for 1.0.0 launch with planned automation in Q1 2026.

---

## Compliance & Legal Considerations

### License Terms (Contract-Based)

**Current Enforcement:**
- MIT license for source code
- Commercial use clause added to LICENSE file
- Subscription agreement (separate legal document)

**Required Legal Review:**
- [ ] Commercial use clause wording (legal approval)
- [ ] Subscription agreement template
- [ ] Terms of service for license server
- [ ] Privacy policy for telemetry (if implemented)

### Audit Trail

**Manual Process:**
- Subscription records in CRM/billing system
- Support ticket system for verification
- Git access logs for repository cloning

**Automated Process (Future):**
- License validation logs
- Usage analytics (opt-in)
- Renewal notification history

---

## Testing Plan (Automated Enforcement)

### Unit Tests

```rust
#[cfg(test)]
mod license_tests {
    #[test]
    fn test_license_key_parsing() {
        let key = "UVEDDI-1-ABC123-9f8e7d6c";
        let license = LicenseKey::parse(key).unwrap();
        assert_eq!(license.version, 1);
        assert_eq!(license.customer_id, "ABC123");
    }
    
    #[test]
    fn test_license_expiration() {
        let expired_license = LicenseKey {
            expires_at: Utc::now() - Duration::days(1),
            ..Default::default()
        };
        assert!(expired_license.is_expired());
    }
    
    #[test]
    fn test_signature_verification() {
        let key_pair = generate_test_keypair();
        let license = create_signed_license(&key_pair);
        assert!(license.verify_signature(&key_pair.public_key).is_ok());
    }
}
```

### Integration Tests

- [ ] Valid license allows analysis
- [ ] Expired license shows error message
- [ ] Missing license prompts for key
- [ ] Grace period logic works correctly
- [ ] Offline mode functions properly
- [ ] License caching persists across runs

### Manual QA Scenarios

1. **First-time user:** No license → helpful error message
2. **Expired subscription:** Grace period → renewal prompt
3. **Offline environment:** Cached validation → 30-day window
4. **Invalid key:** Immediate rejection with support contact
5. **Key rotation:** New key acceptance without disruption

---

## Action Items

### Immediate (Pre-1.0.0 Release)

- [x] Document current state (no implementation)
- [ ] Update README.md with manual verification workflow
- [ ] Add commercial use clause to LICENSE file
- [ ] Create subscriber onboarding email template
- [ ] Define support ticket intake process (subscription verification)

### Q1 2026 (Automated Enforcement)

- [ ] Design license key format specification
- [ ] Implement license server MVP
- [ ] Develop CLI integration with validation logic
- [ ] Create customer portal for license management
- [ ] Beta testing with 5-10 customers
- [ ] Documentation: migration guide for customers

### Q2 2026 (Rollout & Optimization)

- [ ] Gradual rollout with telemetry monitoring
- [ ] Customer feedback integration
- [ ] Performance optimization (caching, offline mode)
- [ ] Security audit of license system
- [ ] Automated renewal notifications

---

## References

- **Related Issues:** UV-7 (License Enforcement), UV-10 (Binary Signing)
- **Dependencies:** RISK_REGISTER.md (R7, R10)
- **External Resources:**
  - [Software License Enforcement Best Practices](https://example.com/license-enforcement)
  - [Rust Licensing Libraries](https://crates.io) - `license`, `licensor` crates

---

## Approval & Sign-Off

**Author:** GitHub Copilot (Assignment A4)  
**Date:** 2025-10-09  
**Review Required:** Legal (commercial clause), Product (customer workflow)  
**Implementation Target:** Q1 2026  
**Status:** ✅ Approved for 1.0.0 Manual Verification Approach

---

**Document Control:**
- Version: 1.0
- Classification: Internal/Commercial Confidential
- Next Review: 2025-11-15 (post-release retrospective)
- Owner: Engineering Lead
