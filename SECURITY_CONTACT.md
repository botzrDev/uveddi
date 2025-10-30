# Uveddi Security Contact Information

**Version:** 0.0.2  
**Last Updated:** 2025-10-09  
**Status:** Active

## 🔒 Reporting Security Vulnerabilities

If you discover a security vulnerability in Uveddi, please report it to our security team immediately. We take security seriously and will respond promptly to all legitimate reports.

### Primary Security Contact

**Email:** security@uveddi.com *(placeholder - update with production address)*  
**Response Time:** Within 24 hours (business days)  
**Escalation SLA:** Critical vulnerabilities escalated within 4 hours

### Alternative Contact Methods

1. **GitHub Security Advisories:** [https://github.com/botzrDev/uveddi/security/advisories](https://github.com/botzrDev/uveddi/security/advisories)
2. **Encrypted Communication:** PGP key available (see below)
3. **Emergency Hotline:** *(to be established for enterprise customers)*

---

## 📧 PGP/GPG Encryption

For sensitive vulnerability reports, use our PGP public key:

**Key ID:** 94156723
**Fingerprint:** `D592 AD0C 4CCC 5125 326F  7A12 B68A 7402 9415 6723`
**Key Type:** RSA 4096-bit
**Created:** 2025-10-09
**Expires:** Never
**Email:** security@uveddi.com
**Name:** Uveddi Security Team

**Public Key Location:** [`pgp/security@uveddi.com.asc`](./pgp/security@uveddi.com.asc)

**To verify and import the key:**
```bash
# Download and import from the repository
curl -O https://raw.githubusercontent.com/botzrDev/uveddi/main/pgp/security@uveddi.com.asc
gpg --import security@uveddi.com.asc

# Verify the fingerprint matches
gpg --fingerprint security@uveddi.com

# Expected output:
# pub   rsa4096 2025-10-09 [SC]
#       D592 AD0C 4CCC 5125 326F  7A12 B68A 7402 9415 6723
# uid           [ultimate] Uveddi Security Team <security@uveddi.com>
# sub   rsa4096 2025-10-09 [E]
```

**To encrypt a message:**
```bash
gpg --encrypt --armor --recipient security@uveddi.com your-report.txt
# Send the encrypted output (.asc file) to security@uveddi.com
```

---

## 🚨 What to Report

We are interested in security vulnerabilities in:

### In Scope

✅ **Code Execution:** Remote or local arbitrary code execution  
✅ **Authentication/Authorization:** Bypass of security controls  
✅ **Data Exposure:** Unauthorized access to sensitive data  
✅ **Injection Attacks:** SQL injection, command injection, path traversal  
✅ **Cryptographic Issues:** Weak encryption, insecure key storage  
✅ **Dependency Vulnerabilities:** Critical issues in third-party libraries  
✅ **Denial of Service:** Resource exhaustion, crash bugs  
✅ **WASM Plugin Security:** Sandbox escapes, host function abuse  

### Out of Scope

❌ **Social Engineering:** Phishing, pretexting (report to abuse@uveddi.com)  
❌ **Physical Security:** Data center access, hardware tampering  
❌ **Third-Party Services:** Issues in external dependencies (report directly to vendors)  
❌ **Denial of Service:** Low-impact DoS without demonstrated exploit  
❌ **Best Practice Deviations:** Non-exploitable security improvements (use GitHub issues)

---

## 📋 Reporting Guidelines

### What to Include

Please provide as much information as possible:

1. **Vulnerability Type:** (e.g., SQL injection, XSS, authentication bypass)
2. **Affected Component:** (e.g., CLI, API server, web UI, specific module)
3. **Version:** Uveddi version number (run `uveddi --version`)
4. **Environment:** Operating system, Rust version, deployment type
5. **Steps to Reproduce:** Detailed instructions to replicate the issue
6. **Proof of Concept:** Code, commands, or screenshots demonstrating the vulnerability
7. **Impact Assessment:** Your analysis of the potential impact
8. **Suggested Fix:** (optional) Proposed mitigation or patch

### Report Template

```markdown
### Vulnerability Summary
[Brief description of the vulnerability]

### Affected Versions
- Uveddi version: X.Y.Z
- Operating System: [Linux/macOS/Windows]
- Deployment: [Binary/Docker/Source build]

### Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Step 3]

### Proof of Concept
```bash
# Commands or code demonstrating the issue
uveddi analyze --malicious-payload ...
```

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens - security impact]

### Impact Assessment
- **Confidentiality:** [High/Medium/Low]
- **Integrity:** [High/Medium/Low]
- **Availability:** [High/Medium/Low]

### Suggested Mitigation
[Your recommendation for fixing the issue]

### Additional Context
[Any other relevant information]
```

---

## 🕐 Response Timeline & SLAs

We commit to the following response times:

| Severity | Initial Response | Status Update | Target Resolution |
|----------|------------------|---------------|-------------------|
| **Critical** (P0) | 4 hours | Daily | 7 days |
| **High** (P1) | 24 hours | Every 3 days | 30 days |
| **Medium** (P2) | 48 hours | Weekly | 90 days |
| **Low** (P3) | 5 business days | Bi-weekly | Next release |

### Severity Definitions

**Critical (P0):** Remote code execution, authentication bypass, data breach
- Immediate escalation to senior developer
- Emergency patch within 7 days
- Public disclosure coordinated with reporter

**High (P1):** Privilege escalation, significant data exposure, DoS
- High priority investigation
- Patch within 30 days
- Security advisory published

**Medium (P2):** Information disclosure, minor injection vulnerabilities
- Standard priority investigation
- Patch within 90 days or next release
- CVE assignment if applicable

**Low (P3):** Security misconfigurations, minor issues with limited impact
- Tracked in backlog
- Fixed in next release cycle
- No immediate public disclosure

---

## 👥 Security Team

### Primary Responders

**Austin Green** - Lead Developer  
- Email: austin@uveddi.com *(placeholder)*
- GitHub: @austingreen *(placeholder)*
- Responsibilities: Vulnerability triage, patch development, security coordination

### Escalation Path

1. **Tier 1:** Primary security contact (security@uveddi.com)
2. **Tier 2:** Lead developer (direct email)
3. **Tier 3:** Legal/Executive (for critical incidents)

**Note:** As a solo developer project transitioning to commercial product, escalation paths will be formalized with team growth.

---

## 🛡️ Responsible Disclosure Policy

We follow a **coordinated disclosure** approach:

### Our Commitments

1. **Acknowledgment:** We will acknowledge receipt within 24 hours (business days)
2. **Investigation:** We will investigate and validate reports promptly
3. **Communication:** We will keep you informed of progress
4. **Credit:** We will credit reporters in security advisories (unless you prefer anonymity)
5. **Fix Timeline:** We will provide a reasonable timeline for resolution
6. **Public Disclosure:** We will coordinate disclosure timing with you

### We Request

1. **Private Disclosure:** Please report vulnerabilities privately first
2. **Reasonable Time:** Allow us time to fix before public disclosure (typically 90 days)
3. **Good Faith:** Do not exploit vulnerabilities beyond proof-of-concept
4. **No Data Access:** Do not access, modify, or delete user data
5. **Coordination:** Work with us on disclosure timing and content

### Public Disclosure

After a fix is released:
- We will publish a security advisory
- CVE will be assigned for critical/high vulnerabilities
- Release notes will credit reporter (with permission)
- Technical details will be shared responsibly

**Embargo Period:** Typically 90 days from validated report, or until patch is widely deployed.

---

## 🏆 Security Acknowledgments

We publicly thank security researchers who help improve Uveddi:

### Hall of Fame

*No reports yet - be the first!*

**Want to be listed?** Report a valid security vulnerability and we'll credit you here (with your permission).

---

## 📚 Security Resources

### For Researchers

- **Security Policy:** [`SECURITY_POLICY.md`](./SECURITY_POLICY.md)
- **Threat Model:** [`docs/security/threat-model.md`](./docs/security/threat-model.md)
- **Security Hardening Guide:** [`docs/security/SECURITY_HARDENING_GUIDE.md`](./docs/security/SECURITY_HARDENING_GUIDE.md)
- **Known Issues:** [`docs/known-issues.md`](./docs/known-issues.md)

### For Users

- **Incident Response:** [`docs/security/INCIDENT_RESPONSE_RUNBOOKS.md`](./docs/security/INCIDENT_RESPONSE_RUNBOOKS.md)
- **Security Compliance:** [`docs/security/SECURITY_COMPLIANCE_CHECKLIST.md`](./docs/security/SECURITY_COMPLIANCE_CHECKLIST.md)
- **Vulnerability Disclosure:** [`docs/security/vulnerability-disclosure.md`](./docs/security/vulnerability-disclosure.md)

---

## 🔐 Security Best Practices

### For Users

1. **Keep Updated:** Always run the latest Uveddi version
2. **Verify Signatures:** Check binary signatures before installation
3. **Secure Configuration:** Follow security hardening guidelines
4. **Monitor Logs:** Review security-relevant log entries
5. **Report Issues:** If you see something, say something

### For Contributors

1. **Security Review:** All PRs undergo security review
2. **Dependency Audits:** Run `cargo audit` before submitting
3. **Input Validation:** Sanitize all user inputs
4. **Secrets Management:** Never commit secrets or API keys
5. **Secure Coding:** Follow Rust security best practices

---

## 📞 Other Security Contacts

### Bug Bounty Program

**Status:** 🚧 Not yet established  
**Target:** Q2 2026 (post-0.0.2 maturity)  
**Platform:** TBD (HackerOne, Bugcrowd, or self-hosted)

Interested in participating? Email security@uveddi.com to express interest.

### Security Advisory Mailing List

**Subscribe:** security-advisories@uveddi.com *(to be established)*  
**Frequency:** Only for security announcements  
**Content:** CVE notifications, critical updates, patch releases

### Emergency Support (Enterprise Customers)

**Availability:** 24/7 for P0/P1 incidents  
**Contact:** Via customer support portal (enterprise tier only)  
**SLA:** 1-hour response time for critical security incidents

---

## 📄 Legal & Compliance

### Safe Harbor

We will not pursue legal action against security researchers who:
- Act in good faith
- Follow responsible disclosure guidelines
- Do not cause harm or access data beyond proof-of-concept
- Coordinate disclosure timing

### DMCA Exemption

Security research activities are protected under DMCA Section 1201(j) exemptions for good faith security research.

### Compliance Contacts

**GDPR/Privacy:** privacy@uveddi.com *(placeholder)*  
**Legal Inquiries:** legal@uveddi.com *(placeholder)*  
**Abuse Reports:** abuse@uveddi.com *(placeholder)*

---

## 🔄 Document Maintenance

This document is reviewed and updated:
- Quarterly (minimum)
- After any security incident
- When contact information changes
- When processes are updated

**Last Review:** 2025-10-09  
**Next Review:** 2026-01-09  
**Owner:** Security Team Lead

---

## ✅ Verification

To verify this document's authenticity:

1. **Canonical Source:** https://github.com/botzrDev/uveddi/blob/main/SECURITY_CONTACT.md
2. **PGP Signature:** *(to be added after key generation)*
3. **Web Archive:** Timestamped at https://web.archive.org

---

**Thank you for helping keep Uveddi secure!** 🙏

Your responsible disclosure helps protect all Uveddi users and contributes to a safer software ecosystem.

---

**Document Control:**
- Version: 1.0
- Classification: Public
- Distribution: Public repository, website
- Format: Markdown (GitHub-rendered)
- Signatures: *(to be added)*
