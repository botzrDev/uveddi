# Security Policy

**Product:** Uveddi CLI v0.0.2  
**Effective Date:** 2025-10-09  
**Last Updated:** 2025-10-09  
**Policy Version:** 1.0

## 🔒 Our Commitment to Security

Uveddi is committed to maintaining the highest standards of security for our commercial code analysis platform. This document outlines our security policies, supported versions, and vulnerability reporting procedures.

---

## 📦 Supported Versions

We provide security updates for the following Uveddi versions:

| Version | Supported          | End of Support |
| ------- | ------------------ | -------------- |
| 1.0.x   | :white_check_mark: | TBD (Active)   |
| 0.9.x (Alpha) | :x: End of Life | 2025-10-09 |
| < 0.9   | :x: End of Life | 2025-09-01 |

### Support Policy

- **Active Support:** Latest minor version (1.0.x) receives all security patches
- **Security-Only Updates:** Previous minor versions may receive critical security fixes for 90 days after new minor release
- **End of Life:** Versions no longer supported will not receive security updates

**Current Supported Version:** 0.0.2  
**Next Minor Release:** 1.1.0 (Planned Q2 2026)

---

## 🚨 Reporting a Vulnerability

### How to Report

**DO NOT** create public GitHub issues for security vulnerabilities.

Instead, please report security vulnerabilities through one of these channels:

1. **Email:** security@uveddi.com *(preferred, placeholder address)*
2. **GitHub Security Advisories:** [Report privately](https://github.com/botzrDev/uveddi/security/advisories/new)
3. **Encrypted Email:** Use our PGP key (see [SECURITY_CONTACT.md](./SECURITY_CONTACT.md))

### What to Expect

When you report a vulnerability:

✅ **Acknowledgment within 24 hours** (business days)  
✅ **Initial assessment within 48 hours**  
✅ **Regular updates** on investigation progress  
✅ **Coordinated disclosure** timing  
✅ **Public credit** in security advisories (if desired)

### Response Timeline

| Severity | Initial Response | Investigation | Patch Release | Public Disclosure |
|----------|------------------|---------------|---------------|-------------------|
| **Critical** | 4 hours | 24-48 hours | 7 days | After patch deployment |
| **High** | 24 hours | 3-5 days | 30 days | After patch deployment |
| **Medium** | 48 hours | 1-2 weeks | 90 days | With next release |
| **Low** | 5 days | 2-4 weeks | Next release | With release notes |

---

## 🛡️ Security Architecture

### Defense in Depth

Uveddi implements multiple layers of security:

1. **Input Validation**
   - All user inputs are sanitized
   - Path traversal prevention
   - Command injection protection
   - SQL injection prevention (parameterized queries)

2. **Secure Defaults**
   - Minimal permissions required
   - No network access unless explicitly enabled
   - Secure temporary file handling
   - Safe deserialization practices

3. **Dependency Management**
   - Regular `cargo audit` scans
   - Automated dependency updates via Dependabot
   - License compliance verification
   - Supply chain security monitoring

4. **WASM Plugin Sandboxing** (when enabled)
   - Resource limits enforced
   - Host function whitelist
   - Memory isolation
   - No direct system access

5. **Secure Communication**
   - TLS 1.3 for all network traffic
   - Certificate validation
   - No insecure protocols (HTTP, FTP, etc.)

### Security Features

- **No Telemetry by Default:** Privacy-first design
- **Local Execution:** Analysis runs entirely locally unless AI features enabled
- **Database Encryption:** SQLite database uses secure permissions
- **Secrets Redaction:** Automatic redaction of sensitive data in logs
- **Audit Logging:** Security-relevant events logged for compliance

---

## 🔐 Authentication & Authorization

### CLI Security Model

**Uveddi CLI operates in these security contexts:**

1. **Local Analysis Mode** (Default)
   - No authentication required
   - Operates with user's file system permissions
   - No network communication

2. **AI-Enhanced Mode** (Optional)
   - Requires API keys for external services (OpenAI, Anthropic, etc.)
   - Keys stored in secure configuration files or environment variables
   - Never transmitted to Uveddi servers

3. **API Server Mode** (Optional)
   - JWT-based authentication
   - Role-based access control (RBAC)
   - API key rotation support
   - Rate limiting and throttling

### Credential Storage

**Best Practices:**
- Use environment variables for API keys
- Never commit credentials to version control
- Use OS-level keychains when available (future feature)
- Rotate keys regularly

**Configuration File Security:**
- `~/.config/uveddi/config.toml` should be `chmod 600` (user-only access)
- Automatic permission validation on startup
- Warning if insecure permissions detected

---

## 🔍 Vulnerability Disclosure Policy

### Coordinated Disclosure

We follow responsible disclosure principles:

1. **Private Reporting:** Researchers report vulnerabilities privately
2. **Investigation:** We investigate and develop a patch
3. **Embargo Period:** Typically 90 days for patch development
4. **Coordinated Release:** Public disclosure after patch is available
5. **Credit:** Researcher credited in security advisory

### Security Advisories

Published security advisories include:

- **CVE Identifier:** For qualifying vulnerabilities
- **CVSS Score:** Severity rating (v3.1 methodology)
- **Affected Versions:** Clear version ranges
- **Mitigation Steps:** Immediate actions users can take
- **Patch Information:** How to update to fixed version
- **Technical Details:** After responsible disclosure period

**Advisory Locations:**
- GitHub Security Advisories
- Release notes
- Security mailing list
- RustSec Advisory Database (when applicable)

---

## 📋 Security Compliance

### Standards & Frameworks

Uveddi aligns with industry security standards:

- **OWASP Top 10:** Application security risks mitigated
- **CWE Top 25:** Most dangerous software weaknesses addressed
- **Rust Security Guidelines:** Memory safety, no unsafe code in critical paths
- **NIST Cybersecurity Framework:** Risk management approach
- **GDPR/CCPA:** Privacy-by-design for data handling

### Secure Development Lifecycle

1. **Design Phase**
   - Threat modeling
   - Security requirements definition
   - Architecture security review

2. **Development Phase**
   - Secure coding standards (Rust idioms)
   - Code review (security-focused)
   - Static analysis (`cargo clippy`, `cargo audit`)
   - Dependency vulnerability scanning

3. **Testing Phase**
   - Security-specific test cases
   - Fuzzing (where applicable)
   - Penetration testing (pre-release)
   - Regression testing for known vulnerabilities

4. **Release Phase**
   - Binary signing
   - SBOM (Software Bill of Materials) generation
   - Security release notes
   - Vulnerability disclosure coordination

5. **Maintenance Phase**
   - Continuous monitoring
   - Incident response procedures
   - Regular security audits
   - Patch management

---

## 🚦 Incident Response

### Security Incident Classification

**P0 - Critical:**
- Active exploitation in the wild
- Remote code execution
- Authentication bypass
- Data breach

**Response:** Immediate escalation, emergency patch within 24-72 hours

**P1 - High:**
- Privilege escalation
- Significant data exposure
- Denial of service with high impact

**Response:** High priority, patch within 1-2 weeks

**P2 - Medium:**
- Information disclosure (limited)
- Cross-site scripting
- Minor injection vulnerabilities

**Response:** Standard priority, patch within 30-90 days

**P3 - Low:**
- Security misconfigurations
- Minor issues with minimal impact

**Response:** Next release cycle

### Incident Response Team

- **Incident Commander:** Lead Developer (Austin Green)
- **Technical Lead:** Core development team
- **Communications:** Customer support (when established)
- **Legal:** External counsel (as needed)

### Post-Incident Process

After resolving a security incident:

1. **Root Cause Analysis:** Identify how vulnerability was introduced
2. **Remediation Verification:** Confirm fix addresses vulnerability
3. **Process Improvement:** Update development practices to prevent recurrence
4. **Communication:** Notify affected users and stakeholders
5. **Documentation:** Update security documentation and runbooks

---

## 🔬 Security Testing

### Automated Testing

**Continuous Security Checks:**
```bash
# Dependency vulnerability scanning
cargo audit

# Security-focused linting
cargo clippy -- -W clippy::all

# Static analysis for unsafe code patterns
cargo geiger

# License compliance
cargo deny check
```

**CI/CD Integration:**
- GitHub Actions runs security checks on every PR
- Automated dependency updates via Dependabot
- SARIF output for code scanning integration

### Manual Testing

**Pre-Release Security Review:**
- Threat model updates
- Code review for security-critical changes
- Penetration testing (external, when budget allows)
- Fuzz testing for parsers and input handling

**Ongoing Monitoring:**
- RustSec advisory monitoring
- CVE database tracking
- Security mailing lists
- Bug bounty program (planned Q2 2026)

---

## 📚 Security Resources

### For Users

- **Security Contact:** [SECURITY_CONTACT.md](./SECURITY_CONTACT.md)
- **Security Hardening:** [`docs/security/SECURITY_HARDENING_GUIDE.md`](./docs/security/SECURITY_HARDENING_GUIDE.md)
- **Incident Response:** [`docs/security/INCIDENT_RESPONSE_RUNBOOKS.md`](./docs/security/INCIDENT_RESPONSE_RUNBOOKS.md)
- **Compliance Checklist:** [`docs/security/SECURITY_COMPLIANCE_CHECKLIST.md`](./docs/security/SECURITY_COMPLIANCE_CHECKLIST.md)

### For Developers

- **Threat Model:** [`docs/security/threat-model.md`](./docs/security/threat-model.md)
- **Secure Coding Guidelines:** [`docs/development/secure-coding.md`](./docs/development/secure-coding.md) *(to be created)*
- **Security Testing Guide:** [`docs/development/security-testing.md`](./docs/development/security-testing.md) *(to be created)*

### For Researchers

- **Bug Bounty Program:** (Planned Q2 2026)
- **Security Hall of Fame:** [SECURITY_CONTACT.md](./SECURITY_CONTACT.md#hall-of-fame)
- **Responsible Disclosure Guidelines:** [SECURITY_CONTACT.md](./SECURITY_CONTACT.md#responsible-disclosure-policy)

---

## 🔄 Policy Updates

This security policy is reviewed and updated:

- **Quarterly:** Routine review and updates
- **After Incidents:** Post-incident improvements
- **On Major Releases:** Alignment with new features
- **On Request:** Community or customer feedback

**Policy Version History:**

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-10-09 | Initial policy for 0.0.2 release |

**Next Scheduled Review:** 2026-01-09

---

## 📞 Contact Information

**Security Team:** security@uveddi.com  
**General Support:** support@uveddi.com  
**Bug Reports:** https://github.com/botzrDev/uveddi/issues  
**Private Vulnerability Reports:** https://github.com/botzrDev/uveddi/security/advisories

For detailed contact information, see [SECURITY_CONTACT.md](./SECURITY_CONTACT.md).

---

## ⚖️ Legal & Compliance

### Safe Harbor Statement

Uveddi commits to:
- Not pursuing legal action against good faith security researchers
- Protecting researchers who follow responsible disclosure
- Coordinating public disclosure timing
- Recognizing researcher contributions publicly (with permission)

### Scope

This security policy applies to:
- ✅ Uveddi CLI (core product)
- ✅ Official Uveddi plugins
- ✅ Official Docker images
- ✅ Official documentation
- ❌ Third-party integrations (report to respective vendors)
- ❌ User configurations (unless default is insecure)
- ❌ Infrastructure outside our control

---

## 🙏 Acknowledgments

We thank the security research community for helping keep Uveddi secure. Special recognition to:

- **RustSec Advisory Database:** For dependency vulnerability tracking
- **Rust Security Response WG:** For language-level security guidance
- **GitHub Security Lab:** For code scanning and advisory infrastructure

*Security researchers who report valid vulnerabilities will be listed in [SECURITY_CONTACT.md](./SECURITY_CONTACT.md#hall-of-fame).*

---

## ✅ Compliance Certifications

**Current Status (0.0.2 Release):**
- ✅ MIT License (open source code)
- ✅ Dependency license compliance
- ✅ No known critical vulnerabilities
- ⏳ SOC 2 Type II (planned for enterprise tier, 2026)
- ⏳ ISO 27001 (future consideration)

---

**Thank you for helping us maintain a secure product!** 🔒

If you have questions about this security policy, please contact security@uveddi.com.

---

**Document Control:**
- Version: 1.0
- Classification: Public
- Distribution: Public repository, website, product documentation
- Owner: Security Team Lead
- Approval: Lead Developer
- Effective Date: 2025-10-09
