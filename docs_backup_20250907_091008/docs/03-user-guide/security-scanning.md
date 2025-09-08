# Security Analysis Features Guide

## Overview

Uveddi provides comprehensive security analysis capabilities that detect vulnerabilities, insecure patterns, and compliance issues in your codebase. The security scanner implements OWASP Top 10 detection, secret scanning, and dependency vulnerability analysis.

## Security Detectors

### OWASP Top 10 Coverage

Uveddi detects vulnerabilities from the OWASP Top 10 2021 categories:

| Category | Detection Capabilities |
|----------|----------------------|
| **A01: Broken Access Control** | Missing authentication checks, improper authorization, path traversal |
| **A02: Cryptographic Failures** | Weak encryption, hardcoded keys, insecure random generators |
| **A03: Injection** | SQL injection, command injection, XSS vulnerabilities |
| **A04: Insecure Design** | Security anti-patterns, missing security controls |
| **A05: Security Misconfiguration** | Debug mode enabled, default credentials, verbose errors |
| **A06: Vulnerable Components** | Outdated dependencies, known CVEs, unpatched libraries |
| **A07: Authentication Failures** | Weak passwords, missing MFA, session issues |
| **A08: Data Integrity Failures** | Insecure deserialization, unsigned data |
| **A09: Security Logging Failures** | Missing audit logs, insufficient monitoring |
| **A10: SSRF** | Server-side request forgery vulnerabilities |

## Running Security Analysis

### Basic Security Scan

```bash
# Run security-focused analysis
uveddi analyze ./src --enable-security

# With specific security profile
uveddi analyze ./src --security-profile strict

# Output security report
uveddi analyze ./src \
  --enable-security \
  --output-format html \
  --output security-report.html
```

### Advanced Configuration

```bash
# Comprehensive security scan with all features
uveddi analyze ./src \
  --enable-security \
  --secret-scanning \
  --dependency-audit \
  --owasp-rules "A01,A02,A03" \
  --min-severity high \
  --include-dev-dependencies
```

## Security Profiles

### Available Profiles

| Profile | Description | Use Case |
|---------|------------|----------|
| `minimal` | Basic security checks | Quick scans, CI/CD |
| `standard` | OWASP Top 10 + secrets | Regular development |
| `strict` | All security checks | Pre-production |
| `compliance` | Regulatory compliance | Audit requirements |
| `custom` | User-defined rules | Specific needs |

### Profile Configuration

```toml
# uveddi.toml
[security]
profile = "strict"
enable_secret_scanning = true
enable_dependency_audit = true
enable_owasp_scanning = true

[security.owasp]
rules = ["A01", "A02", "A03", "A04", "A05"]
severity_threshold = "medium"

[security.secrets]
patterns = [
  "api[_-]?key",
  "secret[_-]?key",
  "password",
  "token",
  "credential"
]
entropy_threshold = 4.5

[security.dependencies]
check_dev_dependencies = false
vulnerability_database = "https://nvd.nist.gov/feeds"
max_cvss_score = 7.0
```

## Vulnerability Detection

### 1. Injection Vulnerabilities

#### SQL Injection Detection

```rust
// Detected: String concatenation in SQL
let query = format!("SELECT * FROM users WHERE id = {}", user_id);

// Suggested fix: Use parameterized queries
let query = sqlx::query("SELECT * FROM users WHERE id = ?")
    .bind(user_id);
```

#### Command Injection

```python
# Detected: User input in system command
os.system(f"ping {user_input}")

# Suggested fix: Use subprocess with array
subprocess.run(["ping", user_input], check=True)
```

### 2. Authentication & Authorization

#### Missing Authentication

```javascript
// Detected: Unprotected endpoint
app.get('/admin/users', (req, res) => {
  // No authentication check
  res.json(getAllUsers());
});

// Suggested fix: Add authentication middleware
app.get('/admin/users', authenticate, authorize('admin'), (req, res) => {
  res.json(getAllUsers());
});
```

#### Weak Password Requirements

```python
# Detected: Weak password validation
if len(password) >= 6:
    save_password(password)

# Suggested fix: Strong password requirements
if (len(password) >= 12 and 
    has_uppercase(password) and 
    has_lowercase(password) and 
    has_numbers(password) and 
    has_special_chars(password)):
    save_password(hash_password(password))
```

### 3. Cryptographic Issues

#### Hardcoded Secrets

```javascript
// Detected: Hardcoded API key
const API_KEY = "sk-1234567890abcdef";

// Suggested fix: Use environment variables
const API_KEY = process.env.API_KEY;
```

#### Weak Encryption

```python
# Detected: MD5 hashing
import hashlib
password_hash = hashlib.md5(password.encode()).hexdigest()

# Suggested fix: Use bcrypt or argon2
import bcrypt
password_hash = bcrypt.hashpw(password.encode(), bcrypt.gensalt())
```

### 4. Security Misconfiguration

#### Debug Mode in Production

```python
# Detected: Debug mode enabled
app.run(debug=True, host='0.0.0.0')

# Suggested fix: Disable debug in production
app.run(debug=False, host='127.0.0.1')
```

#### Verbose Error Messages

```javascript
// Detected: Exposing stack traces
app.use((err, req, res, next) => {
  res.status(500).json({
    error: err.message,
    stack: err.stack  // Exposes internal details
  });
});

// Suggested fix: Generic error messages in production
app.use((err, req, res, next) => {
  console.error(err);  // Log internally
  res.status(500).json({
    error: 'Internal server error'
  });
});
```

## Secret Scanning

### Detection Patterns

Uveddi scans for various secret patterns:

- **API Keys**: AWS, GCP, Azure, Stripe, etc.
- **Tokens**: JWT, OAuth, Bearer tokens
- **Passwords**: Hardcoded passwords, default credentials
- **Private Keys**: SSH, SSL, GPG keys
- **Database Credentials**: Connection strings, passwords

### Entropy Analysis

```python
# High entropy string detection
def calculate_entropy(string):
    """Calculate Shannon entropy to detect random secrets"""
    # Strings with entropy > 4.5 are flagged
    return shannon_entropy(string)

# Example detection:
# "sk_live_4242424242424242" - High entropy, likely secret
# "hello_world_example" - Low entropy, likely safe
```

### False Positive Reduction

```toml
[security.secrets]
# Whitelist patterns
whitelist = [
  "example_key",
  "test_token",
  "mock_password"
]

# Ignore files
ignore_files = [
  "*.test.js",
  "*.spec.ts",
  "testdata/*"
]

# Context-aware detection
check_variable_names = true
check_comments = false
```

## Dependency Vulnerability Scanning

### Vulnerability Database Integration

```bash
# Check dependencies against vulnerability databases
uveddi analyze ./src --dependency-audit

# Output includes:
# - CVE identifiers
# - CVSS scores
# - Affected versions
# - Remediation advice
```

### Dependency Analysis Report

```json
{
  "vulnerabilities": [
    {
      "package": "lodash",
      "version": "4.17.15",
      "vulnerability": "CVE-2021-23337",
      "severity": "HIGH",
      "cvss_score": 7.2,
      "description": "Command injection via template",
      "fixed_version": "4.17.21",
      "remediation": "Update to version 4.17.21 or later"
    }
  ],
  "statistics": {
    "total_dependencies": 150,
    "vulnerable_dependencies": 3,
    "critical_vulnerabilities": 1,
    "high_vulnerabilities": 2
  }
}
```

### Supply Chain Security

```toml
[security.supply_chain]
# Verify package integrity
verify_checksums = true
verify_signatures = true

# Restrict package sources
allowed_registries = [
  "https://registry.npmjs.org",
  "https://crates.io"
]

# License compliance
banned_licenses = ["GPL-3.0", "AGPL-3.0"]
```

## Security Reports

### Report Formats

#### HTML Security Report

```bash
uveddi analyze ./src \
  --enable-security \
  --output-format html \
  --template security \
  --output security-report.html
```

Features:
- Interactive vulnerability explorer
- Severity-based filtering
- Code snippets with remediation
- Compliance summary
- Executive summary

#### JSON Security Report

```bash
uveddi analyze ./src \
  --enable-security \
  --output-format json \
  --output security-report.json
```

Structure:
```json
{
  "summary": {
    "total_issues": 42,
    "critical": 2,
    "high": 8,
    "medium": 15,
    "low": 17
  },
  "vulnerabilities": [...],
  "compliance": {
    "owasp_top_10": "partial",
    "pci_dss": "fail",
    "hipaa": "pass"
  }
}
```

#### SARIF Format

```bash
# Standard format for security tools
uveddi analyze ./src \
  --enable-security \
  --output-format sarif \
  --output results.sarif
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Security Scan
on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Run Uveddi Security Scan
        run: |
          uveddi analyze ./src \
            --enable-security \
            --output-format sarif \
            --output results.sarif
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
      
      - name: Fail on Critical
        run: |
          uveddi ci check ./src \
            --security-profile strict \
            --fail-on-severity critical
```

### GitLab CI

```yaml
security_scan:
  stage: test
  script:
    - uveddi analyze ./src --enable-security --output-format json --output security.json
    - uveddi ci check ./src --max-critical-issues 0
  artifacts:
    reports:
      security: security.json
```

## Remediation Guidance

### Automated Fixes

Some issues can be automatically fixed:

```bash
# Apply automated security fixes
uveddi fix --security-only --dry-run
uveddi fix --security-only --apply

# Fixes include:
# - Updating vulnerable dependencies
# - Adding input validation
# - Escaping output
# - Adding authentication checks
```

### Manual Remediation

For complex issues, Uveddi provides detailed guidance:

```markdown
## Issue: SQL Injection Vulnerability

**Location**: src/database/queries.rs:45
**Severity**: Critical
**OWASP**: A03 - Injection

### Vulnerable Code:
```rust
let query = format!("SELECT * FROM users WHERE id = {}", user_id);
```

### Recommended Fix:
Use parameterized queries with your database library:

```rust
use sqlx::query;

let user = query!("SELECT * FROM users WHERE id = ?", user_id)
    .fetch_one(&pool)
    .await?;
```

### Additional Steps:
1. Audit all database queries for similar patterns
2. Implement input validation
3. Use an ORM or query builder
4. Enable SQL query logging for monitoring
```

## Security Best Practices

### 1. Defense in Depth

```yaml
# Multiple layers of security
layers:
  - input_validation
  - authentication
  - authorization
  - encryption
  - audit_logging
  - monitoring
```

### 2. Secure by Default

```toml
[security.defaults]
require_authentication = true
encrypt_data_at_rest = true
use_secure_cookies = true
enable_csrf_protection = true
strict_transport_security = true
```

### 3. Regular Scanning

```bash
# Automated daily scans
0 2 * * * uveddi analyze /app \
  --enable-security \
  --email-report security@company.com
```

### 4. Security Training

Uveddi provides educational output:

```bash
# Educational mode with detailed explanations
uveddi analyze ./src \
  --enable-security \
  --educational \
  --verbose
```

## Compliance Checking

### Supported Standards

- **OWASP Top 10**: Web application security
- **CWE Top 25**: Most dangerous software weaknesses
- **PCI DSS**: Payment card industry standards
- **HIPAA**: Healthcare data protection
- **GDPR**: Data privacy requirements
- **SOC 2**: Service organization controls

### Compliance Report

```bash
uveddi analyze ./src \
  --compliance-check \
  --standards "owasp,pci-dss,gdpr" \
  --output compliance-report.pdf
```

## Advanced Features

### Custom Security Rules

```yaml
# custom-rules.yaml
rules:
  - id: company-auth-check
    pattern: |
      function($FUNC, ...) {
        ...
        !~"requireAuth"
        ...
      }
    message: "All endpoints must have authentication"
    severity: high
    
  - id: no-eval
    pattern: eval($ANY)
    message: "eval() is prohibited"
    severity: critical
```

### Security Metrics

```bash
# Generate security metrics
uveddi analyze ./src --security-metrics

# Output:
# - Security debt score
# - Mean time to remediation
# - Vulnerability density
# - Security coverage percentage
```

## Troubleshooting

### Common Issues

#### False Positives

```toml
# Configure false positive suppression
[security.suppression]
rules = [
  { id = "hardcoded-secret", file = "test/*" },
  { id = "sql-injection", line = 45, reason = "False positive - using ORM" }
]
```

#### Performance

```bash
# Optimize for large codebases
uveddi analyze ./src \
  --enable-security \
  --parallel-jobs 8 \
  --incremental \
  --cache-results
```

## See Also

- [CLI Commands Reference](../09-reference/cli-complete-reference.md)
- [Configuration Guide](configuration-options.md)
- [API Reference](../08-api/rest-api-reference.md)
- [Security Detector Development](../04-development/security-detector-development.md)