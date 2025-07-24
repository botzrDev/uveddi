# Detailed Documentation Cleanup Plan for Community Release

## 📋 **Phase 1: Structural Consolidation (Priority: Critical)**

### **Action 1.1: Eliminate Duplicate Documentation Trees**

**Primary Structure (Keep):** `docs/`
**Duplicate Structure (Remove):** `docs/src/`

**File-by-File Actions:**

```bash
# Files to MERGE (keep best content from both):
docs/01-getting-started/installation.md ← docs/src/01-getting-started/installation.md
docs/01-getting-started/configuration.md ← docs/src/01-getting-started/configuration.md
docs/01-getting-started/first-steps.md ← docs/src/01-getting-started/first-steps.md

# Files to CONSOLIDATE (multiple developer guides):
docs/05-development/DEVELOPER_GUIDE.md (KEEP - most comprehensive)
docs/02-user-guide/DEVELOPER_GUIDE.md (REMOVE - redundant)
docs/src/05-development/DEVELOPER_GUIDE.md (REMOVE - stub)

# Files to CONSOLIDATE (multiple deployment guides):
docs/operations/deployment-guide.md (KEEP - production focused)
docs/02-user-guide/deployment_guide.md (REMOVE - outdated)
docs/src/operations/deployment-guide.md (REMOVE - stub)
```

**Complete Removal List:**
```bash
# Remove entire duplicate structure
rm -rf docs/src/
rm -rf docs/book/ # Generated mdbook output
```

### **Action 1.2: Remove Internal Development Content**

**Directories to REMOVE (Archive separately):**
```bash
# Internal development artifacts
docs/06-research/ → archive/research/
docs/07-reports/ → archive/reports/
docs/11-prompts/ → archive/prompts/

# Task-specific documentation (154 UV- files)
find docs -name "*UV-*" -o -name "*uv-*" → archive/tasks/

# Performance and verification reports
docs/performance/ → archive/performance/
verification_reports/ → archive/verification/
```

**Files to REMOVE from root docs:**
```bash
docs/alert-system-uv248.md
docs/carbon-awareness-feasibility-report.md
docs/risk-briefing-UV-243.md
```

## 📋 **Phase 2: Content Standardization (Priority: High)**

### **Action 2.1: Update Core Documentation Files**

**README.md Updates:**
```markdown
# REMOVE these sections:
- "Extensible architecture: Plugin system for custom detectors"
- References to enterprise features
- Complex AI provider setup

# UPDATE these sections:
- Installation (community-focused)
- Features (community edition only)
- Documentation links (new structure)
```

**docs/COMMUNITY_RELEASE_NOTES.md:**
- Move to root as primary release documentation
- Update installation URLs and commands
- Remove enterprise migration references

### **Action 2.2: Community Documentation Completion**

**Missing Files to CREATE:**
```bash
docs/09-community/CONTRIBUTING.md (referenced but missing)
docs/09-community/CODE_OF_CONDUCT.md
docs/09-community/SECURITY.md
docs/09-community/CHANGELOG.md
```

**Files to UPDATE:**
```bash
docs/09-community/GUIDELINES.md (currently stub)
docs/09-community/support.md (remove enterprise references)
docs/09-community/faq.md (update for community features only)
```

### **Action 2.3: Navigation Structure Redesign**

**New docs/SUMMARY.md Structure:**
```markdown
# Summary

[Introduction](./README.md)

# Getting Started
- [Installation](./01-getting-started/installation.md)
- [First Steps](./01-getting-started/first-steps.md)
- [Configuration](./01-getting-started/configuration.md)

# User Guide
- [Basic Concepts](./02-user-guide/basic-concepts.md)
- [Common Use Cases](./02-user-guide/common-use-cases.md)
- [Configuration Options](./02-user-guide/configuration-options.md)
- [Troubleshooting](./02-user-guide/troubleshooting.md)

# API Reference
- [OpenAPI Specification](./api/openapi.yaml)
- [Rust Documentation](./api/rust-documentation.md)

# Development
- [Developer Guide](./05-development/DEVELOPER_GUIDE.md)
- [Contributing](./05-development/contributing.md)
- [Detector Development](./05-development/detector_development_guide.md)

# Operations
- [Deployment Guide](./operations/deployment-guide.md)

# Community
- [Guidelines](./09-community/GUIDELINES.md)
- [Contributors](./09-community/CONTRIBUTORS.md)
- [Support](./09-community/support.md)
- [FAQ](./09-community/faq.md)

# Examples
- [Basic Example](./08-examples/basic-example.md)
- [Advanced Example](./08-examples/advanced-example.md)

# Reference
- [Error Codes](./10-reference/error-codes.md)
```

## 📋 **Phase 3: Content Updates (Priority: Medium)**

### **Action 3.1: Remove Enterprise References**

**Files requiring enterprise content removal:**
```bash
docs/02-user-guide/security-deployment-guide.md → REMOVE (enterprise only)
docs/02-user-guide/security-implementation-guide.md → REMOVE (enterprise only)
docs/security/ → REMOVE (enterprise only)
docs/09-community/support.md → UPDATE (remove premium support)
docs/09-community/faq.md → UPDATE (remove enterprise features)
```

**Search and replace operations:**
```bash
# Remove references to:
- "Enterprise customers"
- "Premium support"
- "WASM plugins"
- "PostgreSQL" (except as optional)
- "Team collaboration"
- "Multi-tier pricing"
- "FastAPI backend"
- "User authentication system"
```

### **Action 3.2: Update Feature Documentation**

**docs/02-user-guide/ Updates:**
```bash
basic-concepts.md → UPDATE (community features only)
common-use-cases.md → UPDATE (local analysis focus)
configuration-options.md → UPDATE (simplified config)
feature-flags.md → REVIEW (remove enterprise flags)
```

**docs/api/ Updates:**
```bash
openapi.yaml → UPDATE (community API only)
rust-documentation.md → UPDATE (community modules only)
```

## 📋 **Phase 4: Quality Assurance (Priority: Medium)**

### **Action 4.1: Link Validation**

**Automated checks to implement:**
```bash
# Check for broken internal links
find docs -name "*.md" -exec grep -l "\[.*\](.*\.md)" {} \;

# Check for references to removed files
grep -r "06-research\|07-reports\|11-prompts" docs/

# Validate external links
grep -r "http" docs/ | grep -v "github.com/botzrDev/uveddi"
```

### **Action 4.2: Content Consistency**

**Standardization tasks:**
```bash
# Ensure consistent titles
grep -r "^# " docs/ | grep -v "Guide\|Documentation\|Reference"

# Check for TODO/FIXME markers
grep -r "TODO\|FIXME\|WIP\|DRAFT" docs/

# Validate code examples
find docs -name "*.md" -exec grep -l "```" {} \;
```

## 📋 **Phase 5: Final Organization (Priority: Low)**

### **Action 5.1: File Naming Standardization**

**Rename for consistency:**
```bash
docs/02-user-guide/UVEDDI_COMPREHENSIVE_MANUAL.md → docs/02-user-guide/comprehensive-manual.md
docs/02-user-guide/UVEDDI_EXECUTIVE_REPORT.md → REMOVE (internal)
docs/04-architecture/ANALYSIS_ENGINE_DEEP_DIVE.md → docs/04-architecture/analysis-engine.md
docs/04-architecture/ARCHITECTURE.md → docs/04-architecture/overview.md
```

### **Action 5.2: Directory Cleanup**

**Remove empty/unnecessary directories:**
```bash
docs/12-tui-design/ → REMOVE (internal development)
docs/development/ → MERGE into docs/05-development/
docs/architecture/ → MERGE into docs/04-architecture/
docs/runbooks/ → REMOVE (operational, not community)
```

## 🎯 **Implementation Priority Matrix**

| Phase | Priority | Effort | Impact | Dependencies |
|-------|----------|--------|--------|--------------|
| Phase 1 | Critical | High | High | None |
| Phase 2 | High | Medium | High | Phase 1 |
| Phase 3 | Medium | High | Medium | Phase 2 |
| Phase 4 | Medium | Low | Medium | Phase 3 |
| Phase 5 | Low | Low | Low | Phase 4 |

## 📊 **Success Metrics**

**Before Cleanup:**
- 390 markdown files
- 154 UV-task files
- Duplicate documentation trees
- Mixed enterprise/community content

**After Cleanup Target:**
- <50 markdown files
- 0 UV-task files in user docs
- Single documentation tree
- 100% community-focused content

## 🚀 **Execution Timeline**

**Week 1:** Phase 1 (Structural Consolidation)
**Week 2:** Phase 2 (Content Standardization) 
**Week 3:** Phase 3 (Content Updates)
**Week 4:** Phase 4 & 5 (QA and Final Organization)

---

## 📋 **Detailed File Inventory**

### **Files to Keep and Update**
```
docs/
├── 01-getting-started/
│   ├── installation.md (MERGE with src version)
│   ├── configuration.md (MERGE with src version)
│   └── first-steps.md (MERGE with src version)
├── 02-user-guide/
│   ├── basic-concepts.md (UPDATE - remove enterprise)
│   ├── common-use-cases.md (UPDATE - community focus)
│   ├── configuration-options.md (UPDATE - simplified)
│   ├── troubleshooting.md (KEEP)
│   └── feature-flags.md (REVIEW - remove enterprise flags)
├── 04-architecture/
│   ├── overview.md (RENAME from ARCHITECTURE.md)
│   ├── analysis-engine.md (RENAME from ANALYSIS_ENGINE_DEEP_DIVE.md)
│   └── database-guide.md (RENAME from DATABASE_GUIDE.md)
├── 05-development/
│   ├── DEVELOPER_GUIDE.md (KEEP - primary version)
│   ├── contributing.md (KEEP)
│   └── detector_development_guide.md (KEEP)
├── 08-examples/
│   ├── basic-example.md (KEEP)
│   ├── advanced-example.md (KEEP)
│   └── integration-example.md (KEEP)
├── 09-community/
│   ├── CONTRIBUTORS.md (KEEP)
│   ├── GUIDELINES.md (UPDATE - complete stub)
│   ├── support.md (UPDATE - remove enterprise)
│   ├── faq.md (UPDATE - community only)
│   ├── CONTRIBUTING.md (CREATE - missing)
│   ├── CODE_OF_CONDUCT.md (CREATE)
│   └── SECURITY.md (CREATE)
├── 10-reference/
│   ├── error-codes.md (KEEP)
│   └── testing_strategy.md (KEEP)
├── api/
│   ├── openapi.yaml (UPDATE - community API)
│   └── rust-documentation.md (UPDATE - community modules)
├── operations/
│   └── deployment-guide.md (KEEP - primary version)
├── book.toml (KEEP)
├── SUMMARY.md (REWRITE - new structure)
└── README.md (CREATE - docs introduction)
```

### **Files/Directories to Remove**
```
docs/src/ (ENTIRE DIRECTORY - duplicate)
docs/book/ (ENTIRE DIRECTORY - generated)
docs/06-research/ (ARCHIVE - internal development)
docs/07-reports/ (ARCHIVE - internal development)
docs/11-prompts/ (ARCHIVE - internal development)
docs/12-tui-design/ (ARCHIVE - internal development)
docs/performance/ (ARCHIVE - internal development)
docs/security/ (REMOVE - enterprise only)
docs/development/ (MERGE into 05-development/)
docs/architecture/ (MERGE into 04-architecture/)
docs/runbooks/ (REMOVE - operational)
docs/user-guides/ (MERGE into 02-user-guide/)

# Specific files to remove:
docs/alert-system-uv248.md
docs/carbon-awareness-feasibility-report.md
docs/risk-briefing-UV-243.md
docs/02-user-guide/DEVELOPER_GUIDE.md (duplicate)
docs/02-user-guide/deployment_guide.md (duplicate)
docs/02-user-guide/security-deployment-guide.md (enterprise)
docs/02-user-guide/security-implementation-guide.md (enterprise)
docs/02-user-guide/UVEDDI_EXECUTIVE_REPORT.md (internal)
```

## 🔧 **Automation Scripts Needed**

### **Script 1: Duplicate Detection**
```bash
#!/bin/bash
# find_duplicates.sh
echo "Finding duplicate documentation files..."
find docs -name "*.md" -exec basename {} \; | sort | uniq -d
```

### **Script 2: Enterprise Reference Scanner**
```bash
#!/bin/bash
# scan_enterprise_refs.sh
echo "Scanning for enterprise references..."
grep -r -i "enterprise\|premium\|wasm\|postgresql\|fastapi" docs/ --include="*.md"
```

### **Script 3: Link Validator**
```bash
#!/bin/bash
# validate_links.sh
echo "Validating internal links..."
find docs -name "*.md" -exec grep -l "\[.*\](.*\.md)" {} \; | while read file; do
    echo "Checking links in: $file"
    grep -o "\[.*\](.*\.md)" "$file"
done
```

---

**Next Steps:**
1. Review and approve this cleanup plan
2. Create archive directory for removed content
3. Begin Phase 1 implementation
4. Set up automation scripts for quality assurance

**Estimated Total Effort:** 2-3 weeks for complete cleanup and reorganization