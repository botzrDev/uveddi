# Uveddi Documentation Reorganization Plan
## Pre-Production Testing Preparation

### Phase 1: Root Directory Cleanup (Immediate)

#### Files to Keep in Root:
- `README.md` - Streamlined to 50-75 lines (project overview + quick start)
- `CONTRIBUTING.md` - Single authoritative contribution guide
- `CHANGELOG.md` - Version history
- `LICENSE` - Legal requirements

#### Files to Move:
- `CLAUDE.md` → `/docs/development/claude-instructions.md`
- `ISSUES_TO_FIX.md` → `/docs/release-planning/v1.0-issues.md`
- `PLUGIN_ECOSYSTEM_OVERVIEW.md` → `/docs/development/plugins/overview.md`
- `PRODUCTION_SECURE_DEPLOYMENT.md` → `/docs/deployment/production-security.md`
- `SECURITY_*.md` (all 3) → `/docs/security/` (consolidate)
- `TYPESCRIPT_ANALYSIS_IMPROVEMENTS.md` → `/docs/development/typescript-support.md`
- `documentation-audit-prompt.md` → Archive or remove

### Phase 2: New Documentation Structure

```
/docs/
├── getting-started/
│   ├── README.md (index)
│   ├── installation.md
│   ├── quickstart.md
│   └── first-analysis.md
│
├── user-guide/
│   ├── README.md (index)
│   ├── configuration.md
│   ├── cli-reference.md
│   ├── anti-patterns.md
│   ├── web-dashboard.md
│   ├── tui-interface.md
│   └── plugins/
│       ├── using-plugins.md
│       └── available-plugins.md
│
├── development/
│   ├── README.md (index)
│   ├── architecture.md
│   ├── building.md
│   ├── testing.md
│   ├── claude-instructions.md
│   └── plugins/
│       ├── development-guide.md
│       ├── api-reference.md
│       └── examples.md
│
├── deployment/
│   ├── README.md (index)
│   ├── pre-production-checklist.md
│   ├── production-deployment.md
│   ├── docker-deployment.md
│   ├── monitoring.md
│   ├── security-hardening.md
│   └── troubleshooting.md
│
├── reference/
│   ├── README.md (index)
│   ├── cli-reference.md
│   ├── api-reference.md
│   ├── configuration-reference.md
│   └── error-codes.md
│
├── security/
│   ├── README.md (index)
│   ├── security-policy.md
│   ├── vulnerability-disclosure.md
│   └── incident-response.md
│
└── release-notes/
    ├── README.md (index)
    ├── v1.0-alpha-status.md
    ├── known-issues.md
    └── changelog.md
```

### Phase 3: Content Consolidation Tasks

#### Plugin Documentation (14+ files → 3 files):
- **User perspective**: `/docs/user-guide/plugins/`
- **Developer perspective**: `/docs/development/plugins/`
- **API Reference**: `/docs/reference/plugin-api.md`

#### Configuration (10+ files → 2 files):
- **User Guide**: `/docs/user-guide/configuration.md`
- **Complete Reference**: `/docs/reference/configuration-reference.md`

#### Security (7+ files → 3 files):
- **Security Policy**: `/docs/security/security-policy.md`
- **Deployment Security**: `/docs/deployment/security-hardening.md`
- **Incident Response**: `/docs/security/incident-response.md`

### Phase 4: New Production Documentation

#### Critical Missing Documents to Create:
1. `/docs/deployment/pre-production-checklist.md`
2. `/docs/deployment/production-deployment.md`
3. `/docs/release-notes/v1.0-alpha-status.md`
4. `/docs/getting-started/quickstart.md` (5-minute guide)

### Phase 5: README.md Rewrite

New streamlined README structure (target: 75 lines):
```markdown
# Uveddi - Architectural Analysis Tool

One-paragraph description of what Uveddi does and why it matters.

## Features
- Bullet list of key features (5-7 items max)

## Quick Start
```bash
# Installation
cargo install uveddi

# Basic usage
uveddi analyze ./src
```

## Documentation
- [Getting Started](docs/getting-started/)
- [User Guide](docs/user-guide/)
- [Development](docs/development/)
- [API Reference](docs/reference/)

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md)

## License
[LICENSE](LICENSE)
```

### Implementation Timeline

#### Week 1 (Immediate - Pre-Production Critical):
- [ ] Streamline README.md
- [ ] Move root-level docs to proper locations
- [ ] Create pre-production checklist
- [ ] Consolidate security documentation

#### Week 2 (Structure):
- [ ] Reorganize docs/ directory structure
- [ ] Consolidate plugin documentation
- [ ] Merge duplicate configuration docs
- [ ] Create production deployment guide

#### Week 3 (Polish):
- [ ] Update all cross-references
- [ ] Add missing production docs
- [ ] Final review and cleanup
- [ ] Documentation testing

### Success Metrics
- Root directory has ≤4 markdown files
- No duplicate documentation topics
- Clear user journey from installation to production
- All production-critical documentation present
- Documentation build passes without warnings

### Notes
- Keep `.claude/agents/` directory as-is (specialized tooling)
- Archive old documentation in `/docs/archive/` before deletion
- Update CI/CD to validate documentation structure
- Consider mdBook or similar for documentation website