# Uveddi Security Incident Response Runbooks

## Overview

This document provides step-by-step runbooks for responding to various security incidents in the Uveddi environment. These runbooks ensure consistent, effective responses to security threats and minimize impact on operations.

## Table of Contents

1. [General Incident Response Process](#general-incident-response-process)
2. [Runbook 1: Data Breach Response](#runbook-1-data-breach-response)
3. [Runbook 2: Container Security Compromise](#runbook-2-container-security-compromise)
4. [Runbook 3: Dependency Vulnerability Response](#runbook-3-dependency-vulnerability-response)
5. [Runbook 4: Authentication System Compromise](#runbook-4-authentication-system-compromise)
6. [Runbook 5: Injection Attack Response](#runbook-5-injection-attack-response)
7. [Runbook 6: DDoS Attack Response](#runbook-6-ddos-attack-response)
8. [Runbook 7: Configuration Drift Response](#runbook-7-configuration-drift-response)
9. [Runbook 8: Certificate/TLS Issues](#runbook-8-certificatetls-issues)
10. [Communication Templates](#communication-templates)
11. [Post-Incident Activities](#post-incident-activities)

## General Incident Response Process

### Phase 1: Detection and Analysis (0-15 minutes)
1. **Alert Receipt**: Security monitoring system or human report
2. **Initial Triage**: Determine severity and impact
3. **Team Activation**: Notify appropriate response team members
4. **Preliminary Assessment**: Gather initial evidence

### Phase 2: Containment (15-30 minutes)
1. **Immediate Containment**: Stop ongoing attack/damage
2. **System Isolation**: Isolate affected systems if necessary
3. **Preserve Evidence**: Maintain forensic integrity
4. **Stakeholder Notification**: Inform key stakeholders

### Phase 3: Investigation and Recovery (30 minutes - hours)
1. **Detailed Analysis**: Conduct thorough investigation
2. **Root Cause Analysis**: Identify attack vectors and vulnerabilities
3. **System Recovery**: Restore services safely
4. **Validation**: Confirm systems are clean and secure

### Phase 4: Post-Incident Activities (Days-Weeks)
1. **Lessons Learned**: Document improvements
2. **Policy Updates**: Update procedures based on findings
3. **Monitoring Enhancement**: Improve detection capabilities
4. **Training Updates**: Update team training materials

---

## Runbook 1: Data Breach Response

### **Incident Type**: Confirmed or suspected unauthorized access to sensitive data

### **Severity Classification**:
- **P0 (Critical)**: Confirmed breach with customer data exposure
- **P1 (High)**: Suspected breach or internal data exposure
- **P2 (Medium)**: Potential breach with limited scope

### **Immediate Response (0-15 minutes)**

#### Step 1: Initial Assessment
```bash
# Check system logs for suspicious access patterns
tail -n 1000 /app/logs/audit.log | grep -i "unauthorized\|failed\|breach"

# Verify user access logs
grep "authentication" /app/logs/audit.log | tail -50

# Check container status
docker ps -a
docker logs uveddi_app_1 | tail -100
```

#### Step 2: Alert Key Personnel
- **Security Team**: security@uveddi.dev
- **Incident Commander**: incident-commander@uveddi.dev
- **Legal Team**: legal@uveddi.dev
- **Executive Team**: executives@uveddi.dev

#### Step 3: Preserve Evidence
```bash
# Create forensic snapshot
sudo dd if=/dev/sda of=/forensics/disk-$(date +%Y%m%d-%H%M%S).img bs=4096

# Capture memory dump
sudo gcore -o /forensics/memory-$(date +%Y%m%d-%H%M%S) $(pgrep uveddi)

# Copy critical logs
cp -r /app/logs /forensics/logs-$(date +%Y%m%d-%H%M%S)
cp -r /var/log /forensics/system-logs-$(date +%Y%m%d-%H%M%S)
```

### **Containment (15-30 minutes)**

#### Step 4: Immediate Containment
```bash
# If breach is confirmed, isolate affected systems
docker-compose down

# Block suspicious IP addresses (example)
iptables -A INPUT -s SUSPICIOUS_IP -j DROP

# Disable compromised user accounts
# (Implementation depends on authentication system)
```

#### Step 5: Damage Assessment
```bash
# Check for data exfiltration
grep -r "SELECT\|COPY\|EXPORT" /app/logs/ | grep -v "normal_operation"

# Verify database integrity
sqlite3 /app/data/uveddi.db ".backup /forensics/db-backup-$(date +%Y%m%d-%H%M%S).db"

# Check file system changes
find /app -type f -mtime -1 -ls > /forensics/recent-changes-$(date +%Y%m%d-%H%M%S).log
```

### **Investigation and Recovery (30 minutes - 4 hours)**

#### Step 6: Detailed Investigation
1. **Access Pattern Analysis**:
   - Review authentication logs
   - Identify unauthorized access attempts
   - Map data access patterns

2. **System Compromise Assessment**:
   - Check for malware or backdoors
   - Verify system integrity
   - Review configuration changes

3. **Data Impact Assessment**:
   - Identify affected data types
   - Quantify scope of exposure
   - Assess data sensitivity levels

#### Step 7: System Recovery
```bash
# Clean system recovery
git checkout HEAD -- .  # Reset to known good state
docker-compose build --no-cache
docker-compose up -d

# Verify system integrity
cargo test --test comprehensive_security_validation
cargo test --test compliance_validation

# Update all credentials
# - Generate new API keys
# - Rotate database passwords
# - Update TLS certificates
```

### **Communication Requirements**

#### Internal Communication (Immediate)
- Executive briefing within 1 hour
- All-hands notification (if public-facing)
- Customer support team briefing

#### External Communication (As Required)
- **Regulatory Notification**: 72 hours (GDPR) or as required by jurisdiction
- **Customer Notification**: Based on impact assessment
- **Public Disclosure**: As required by regulations and company policy

### **Documentation Requirements**
- Incident timeline
- Impact assessment
- Response actions taken
- Evidence preservation log
- Communication log

---

## Runbook 2: Container Security Compromise

### **Incident Type**: Container escape, privilege escalation, or container compromise

### **Severity Classification**:
- **P0 (Critical)**: Container escape with host access
- **P1 (High)**: Container compromise with sensitive data access
- **P2 (Medium)**: Container compromise without data access

### **Immediate Response (0-15 minutes)**

#### Step 1: Detect Container Compromise
```bash
# Check container status and resource usage
docker stats --no-stream

# Review container security events
docker events --filter container=uveddi --since="1h"

# Check for privilege escalation
docker exec uveddi ps aux | grep -E "(root|sudo|su)"

# Verify container user
docker exec uveddi id
```

#### Step 2: Immediate Containment
```bash
# Stop compromised container
docker stop uveddi

# Prevent container restart
docker update --restart=no uveddi

# Isolate container network
docker network disconnect bridge uveddi
```

### **Investigation (15-60 minutes)**

#### Step 3: Forensic Analysis
```bash
# Export container filesystem
docker export uveddi > /forensics/container-$(date +%Y%m%d-%H%M%S).tar

# Examine container logs
docker logs uveddi > /forensics/container-logs-$(date +%Y%m%d-%H%M%S).log

# Check container image integrity
docker history uveddi:latest
docker inspect uveddi:latest | jq '.RootFS'
```

#### Step 4: Host System Check
```bash
# Check for host compromise
sudo netstat -tulpn | grep LISTEN
sudo ps aux | grep -v "\[.*\]" | sort

# Verify file system integrity
sudo find /var/lib/docker -type f -mtime -1 -ls

# Check for suspicious processes
sudo lsof | grep deleted
```

### **Recovery (1-2 hours)**

#### Step 5: Clean Recovery
```bash
# Remove compromised container and images
docker rm -f uveddi
docker rmi $(docker images -q uveddi)

# Rebuild from clean source
git status  # Ensure clean repository state
docker-compose build --no-cache
docker-compose up -d

# Verify security controls
docker exec uveddi id  # Should be UID 1001
docker exec uveddi ls -la /proc/1/  # Verify process isolation
```

#### Step 6: Security Hardening
```bash
# Run security validation
cargo test --test comprehensive_security_validation

# Update container security scanning
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image uveddi:latest

# Verify docker-compose security constraints
grep -E "(no-new-privileges|read_only|cap_drop)" docker-compose.yml
```

---

## Runbook 3: Dependency Vulnerability Response

### **Incident Type**: Critical vulnerability discovered in project dependencies

### **Severity Classification**:
- **P0 (Critical)**: Remote code execution vulnerabilities
- **P1 (High)**: Privilege escalation or data exposure vulnerabilities
- **P2 (Medium)**: Denial of service or information disclosure

### **Immediate Response (0-30 minutes)**

#### Step 1: Vulnerability Assessment
```bash
# Run dependency audit
cargo audit

# Check for specific vulnerabilities
cargo audit --json | jq '.vulnerabilities[] | select(.severity == "critical")'

# Verify current dependency versions
cargo tree | grep -E "(tokio|serde|reqwest|openssl)"
```

#### Step 2: Impact Analysis
```bash
# Check if vulnerable code paths are used
grep -r "vulnerable_function" src/
rg "use.*vulnerable_crate" src/

# Review recent commits using vulnerable dependencies
git log --oneline --since="30 days ago" -- Cargo.toml Cargo.lock
```

### **Mitigation (30 minutes - 2 hours)**

#### Step 3: Immediate Mitigation
```bash
# Update vulnerable dependencies
cargo update --package vulnerable_crate

# If no update available, check for alternatives
cargo search alternative_crate

# Build and test with updates
cargo build --release
cargo test
```

#### Step 4: Validation
```bash
# Re-run security tests
cargo test --test comprehensive_security_validation

# Verify vulnerability is resolved
cargo audit

# Check for regression issues
cargo test --all-features
```

### **Deployment (2-4 hours)**

#### Step 5: Emergency Deployment
```bash
# Build new container image
docker build -t uveddi:security-patch-$(date +%Y%m%d) .

# Test container security
docker run --rm uveddi:security-patch-$(date +%Y%m%d) cargo audit

# Deploy to production
docker-compose up -d

# Verify deployment
curl -f https://api.uveddi.com/health
```

---

## Runbook 4: Authentication System Compromise

### **Incident Type**: Authentication bypass, credential theft, or session hijacking

### **Immediate Response (0-15 minutes)**

#### Step 1: Detect Authentication Issues
```bash
# Check for suspicious authentication patterns
grep -i "authentication.*failed" /app/logs/audit.log | tail -50
grep -i "session.*invalid" /app/logs/audit.log | tail -50

# Look for unusual access patterns
awk '/authentication/ {print $4, $7}' /app/logs/audit.log | sort | uniq -c | sort -nr
```

#### Step 2: Immediate Protection
```bash
# Disable compromised accounts (if identified)
# Implementation depends on authentication system

# Invalidate all active sessions
# Clear session storage/cache

# Enable enhanced monitoring
# Reduce session timeout temporarily
```

### **Investigation and Recovery (15 minutes - 2 hours)**

#### Step 3: Session and Credential Analysis
- Review all active sessions
- Check for credential stuffing attacks
- Verify MFA bypass attempts
- Analyze geographic access patterns

#### Step 4: System Hardening
```bash
# Force password resets for affected accounts
# Update authentication configurations
# Implement additional monitoring rules
# Review and update access controls
```

---

## Runbook 5: Injection Attack Response

### **Incident Type**: SQL injection, command injection, or code injection attempts

### **Immediate Response (0-15 minutes)**

#### Step 1: Detect Injection Attempts
```bash
# Check for SQL injection patterns
grep -iE "(union select|drop table|insert into|'; --)" /app/logs/audit.log

# Check for command injection
grep -iE "(\||\&|;|`|\$\()" /app/logs/audit.log

# Review input validation failures
grep "input_validation.*failed" /app/logs/audit.log
```

#### Step 2: Block Malicious Requests
```bash
# Block suspicious IP addresses
iptables -A INPUT -s MALICIOUS_IP -j DROP

# Rate limit if needed
# Enable DDoS protection
```

### **Analysis and Hardening (15 minutes - 1 hour)**

#### Step 3: Input Validation Review
```bash
# Run injection prevention tests
cargo test input_validation -- --nocapture
cargo test sql_injection -- --nocapture
cargo test command_injection -- --nocapture

# Review code for new injection vectors
rg "execute|query|command" src/ --type rust
```

#### Step 4: Database Security Check
```bash
# Check database logs for successful injections
# Verify database user permissions
# Review parameterized query usage
```

---

## Runbook 6: DDoS Attack Response

### **Incident Type**: Distributed Denial of Service attack

### **Immediate Response (0-5 minutes)**

#### Step 1: Detect DDoS Attack
```bash
# Check connection counts
netstat -an | grep :8080 | wc -l

# Monitor resource usage
top -n 1 | head -20
free -m
df -h

# Check request patterns
tail -1000 /app/logs/access.log | awk '{print $1}' | sort | uniq -c | sort -nr | head -20
```

#### Step 2: Immediate Mitigation
```bash
# Enable rate limiting
# Block top attacking IPs
for ip in $(tail -1000 /app/logs/access.log | awk '{print $1}' | sort | uniq -c | sort -nr | head -10 | awk '$1 > 100 {print $2}'); do
    iptables -A INPUT -s $ip -j DROP
done

# Scale resources if possible
docker-compose up --scale uveddi=3
```

---

## Runbook 7: Configuration Drift Response

### **Incident Type**: Unauthorized configuration changes or security misconfiguration

### **Immediate Response (0-30 minutes)**

#### Step 1: Detect Configuration Drift
```bash
# Check for unauthorized changes
git status
git diff HEAD

# Review configuration files
diff docker-compose.yml docker-compose.yml.backup
diff src/security/config.rs.backup src/security/config.rs

# Check container configuration
docker inspect uveddi | jq '.[] | {User, SecurityOpt, ReadonlyRootfs}'
```

#### Step 2: Revert to Secure Configuration
```bash
# Revert unauthorized changes
git checkout HEAD -- .

# Rebuild with secure configuration
docker-compose down
docker-compose build --no-cache
docker-compose up -d

# Validate security controls
cargo test --test compliance_validation
```

---

## Runbook 8: Certificate/TLS Issues

### **Incident Type**: Certificate expiration, compromise, or TLS configuration issues

### **Immediate Response (0-30 minutes)**

#### Step 1: Certificate Status Check
```bash
# Check certificate expiration
openssl x509 -in /path/to/cert.pem -noout -dates

# Test TLS configuration
curl -I https://api.uveddi.com
openssl s_client -connect api.uveddi.com:443 -servername api.uveddi.com

# Verify certificate chain
openssl verify -CAfile ca-bundle.crt server.crt
```

#### Step 2: Certificate Renewal/Replacement
```bash
# Generate new certificate (if needed)
# Update certificate in configuration
# Restart services with new certificate
docker-compose restart

# Verify TLS functionality
cargo test tls_validation -- --nocapture
```

---

## Communication Templates

### **Initial Alert Template**
```
SECURITY INCIDENT ALERT

Incident ID: INC-YYYY-MMDD-XXXX
Detected At: [Timestamp]
Severity: [P0/P1/P2]
Incident Type: [Breach/Compromise/Attack]
Status: [Active/Contained/Resolved]

Initial Assessment:
- [Brief description of incident]
- [Affected systems/services]
- [Potential impact]

Response Actions:
- [Immediate containment measures]
- [Investigation status]
- [ETA for next update]

Incident Commander: [Name/Contact]
Next Update: [Timestamp]
```

### **Executive Briefing Template**
```
EXECUTIVE SECURITY BRIEFING

Incident: [Brief description]
Business Impact: [High/Medium/Low]
Customer Impact: [Yes/No - Details]
Data Involved: [Type and scope]

Current Status: [Active response/Contained/Resolved]
Resolution ETA: [Timestamp]

Required Actions:
- [ ] Customer notification
- [ ] Regulatory notification
- [ ] Public disclosure
- [ ] Legal review

Key Personnel:
- Incident Commander: [Name]
- Security Lead: [Name]
- Business Owner: [Name]
```

### **Customer Communication Template**
```
Subject: Important Security Notice - [Company Name]

Dear [Customer Name],

We are writing to inform you of a security incident that may have affected your account/data. We take the security and privacy of your information very seriously, and we want to provide you with details about what happened and what we are doing about it.

What Happened:
[Clear, non-technical description of the incident]

Information Involved:
[Specific types of data that may have been affected]

What We Are Doing:
[Actions taken to address the incident and prevent future occurrences]

What You Can Do:
[Specific recommendations for customers]

Contact Information:
[Support contact details]

We sincerely apologize for this incident and any inconvenience it may cause.
```

---

## Post-Incident Activities

### **Immediate Post-Incident (24-48 hours)**

#### 1. Incident Documentation
- Complete incident timeline
- Document all response actions
- Collect evidence and logs
- Record lessons learned

#### 2. System Validation
```bash
# Comprehensive security testing
cargo test --test comprehensive_security_validation
cargo test --test compliance_validation

# Vulnerability scanning
cargo audit
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock aquasec/trivy image uveddi:latest

# Security configuration review
./scripts/security-config-check.sh
```

#### 3. Communication Wrap-up
- Final stakeholder notification
- Customer follow-up (if applicable)
- Regulatory compliance documentation
- Internal team debrief

### **Short-term Follow-up (1-2 weeks)**

#### 1. Root Cause Analysis
- Technical analysis completion
- Process improvement identification
- Training needs assessment
- Policy update requirements

#### 2. Security Enhancement
- Implement additional monitoring
- Update security controls
- Enhance detection capabilities
- Update response procedures

### **Long-term Improvements (1-3 months)**

#### 1. Process Enhancement
- Update incident response procedures
- Enhance security training programs
- Implement lessons learned
- Update compliance documentation

#### 2. Technology Improvements
- Upgrade security tools
- Implement new monitoring capabilities
- Enhance automation
- Update security architecture

---

## Emergency Contacts

### **24/7 Security Response Team**
- **Primary**: +1-XXX-XXX-XXXX
- **Secondary**: +1-XXX-XXX-XXXX
- **Email**: security-emergency@uveddi.dev

### **Key Personnel**
- **CISO**: ciso@uveddi.dev / +1-XXX-XXX-XXXX
- **Security Architect**: security-arch@uveddi.dev
- **Incident Commander**: incident-commander@uveddi.dev
- **Legal Counsel**: legal@uveddi.dev
- **Executive Team**: executives@uveddi.dev

### **External Contacts**
- **Law Enforcement**: [Local FBI Cyber Crime Unit]
- **Legal Counsel**: [External legal firm]
- **Insurance**: [Cyber insurance provider]
- **Forensics**: [External forensics partner]

---

**Document Version**: 1.0  
**Created**: 2024-01-24  
**Last Updated**: 2024-01-24  
**Next Review**: 2024-07-24

**Approved by**: ___________________  
**Security Team Lead**: ____________  
**Date**: _________________________