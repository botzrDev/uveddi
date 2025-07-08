# Documentation Reorganization Implementation Plan

## Phase 1: Create New Structure (30 minutes)

### 1. Create New Folder Structure
```bash
# Create numbered folders for logical ordering
mkdir -p docs/{01-getting-started,02-user-guide,03-api-reference,04-architecture,05-development,06-research,07-reports,08-examples,09-community,10-reference}

# Create research subfolders
mkdir -p docs/06-research/{anti-pattern-analysis,detector-research,visualization-research,technology-evaluation}

# Create reports subfolders  
mkdir -p docs/07-reports/{audit-reports,sprint-reports}
```

### 2. File Migrations and Renames

#### Getting Started (01-getting-started/)
- `getting-started/installation.md` → `01-getting-started/installation.md`
- `getting-started/configuration.md` → `01-getting-started/configuration.md`
- `getting-started/first-steps.md` → `01-getting-started/quick-start.md`
- `user-guide/troubleshooting.md` → `01-getting-started/troubleshooting.md`

#### User Guide (02-user-guide/)
- `user-guide/basic-concepts.md` → `02-user-guide/basic-concepts.md`
- `user-guide/common-use-cases.md` → `02-user-guide/analysis-workflow.md`
- `user-guide/configuration-options.md` → `02-user-guide/configuration-options.md`
- `visualization_guide.md` → `02-user-guide/visualization-system.md`
- NEW: `02-user-guide/anti-pattern-detection.md`

#### API Reference (03-api-reference/)
- `api/overview.md` → `03-api-reference/overview.md`
- NEW: `03-api-reference/analysis-engine.md`
- NEW: `03-api-reference/detector-api.md`
- NEW: `03-api-reference/ai-integration.md`
- NEW: `03-api-reference/plugin-system.md`

#### Architecture (04-architecture/)
- `Architecture/ARCHITECTURE.md` → `04-architecture/system-architecture.md`
- `Architecture/DATABASE_GUIDE.md` → `04-architecture/database-design.md`
- `Architecture/C4_ARCHITECTURE.md` → `04-architecture/overview.md`
- `ERD.md` → `04-architecture/database-design.md` (merge)
- `SAM.md` → `04-architecture/component-model.md`
- NEW: `04-architecture/security-model.md`

#### Development (05-development/)
- `development/architecture.md` → `05-development/system-overview.md`
- `development/contributing.md` → `05-development/contributing.md`
- `development/detector_development_guide.md` → `05-development/detector-development.md`
- `TESTING_STRATEGY.md` → `05-development/testing-strategy.md`
- NEW: `05-development/coding-standards.md`
- NEW: `05-development/release-process.md`

#### Research (06-research/)
- `R&D/Analysis/Anti_Patterns/Master_Anti_Patterns_List.md` → `06-research/anti-pattern-analysis/overview.md`
- `R&D/Analysis/Anti_Patterns/AntiPats_*.md` → `06-research/anti-pattern-analysis/language-specific-patterns.md`
- `R&D/Analysis/Detectors/Dead_Code/Dead_Code_Research.md` → `06-research/detector-research/dead-code-detection.md`
- `R&D/Analysis/Detectors/Duplicate_Code/` → `06-research/detector-research/code-duplication.md`
- `R&D/Reports_Visualization/` → `06-research/visualization-research/`
- `R&D/NEW_Research-2025-07-07/` → `06-research/technology-evaluation/`

#### Reports (07-reports/)
- `Dev_Reports/comprehensive_audit_report_07072025.md` → `07-reports/audit-reports/2025-01-07-comprehensive-audit.md`
- `Dev_Reports/Anti_Pattern_Detector_Audit_2025-01-07.md` → `07-reports/audit-reports/2025-01-07-detector-audit.md`
- `Dev_Reports/*Sprint*.md` → `07-reports/sprint-reports/`
- NEW: `07-reports/project-status.md`

#### Examples (08-examples/)
- `examples/basic-example.md` → `08-examples/basic-usage.md`
- `examples/advanced-example.md` → `08-examples/advanced-configuration.md`
- `examples/integration-example.md` → `08-examples/integration-examples.md`
- NEW: `08-examples/custom-detectors.md`

#### Community (09-community/)
- `community/` → `09-community/`
- `faq.md` → `09-community/faq.md`

#### Reference (10-reference/)
- `references/error-codes.md` → `10-reference/error-codes.md`
- `glossary.md` → `10-reference/glossary.md`
- NEW: `10-reference/cli-reference.md`
- NEW: `10-reference/configuration-schema.md`

## Phase 2: Update Content (60 minutes)

### 1. Standardize Titles
- Convert all titles to "Title Case"
- Add consistent subtitle format
- Standardize section headers

### 2. Update Cross-References
- Update all internal links to new paths
- Fix broken references
- Update SUMMARY.md navigation

### 3. Consolidate Duplicate Content
- Merge similar architecture docs
- Combine related research files
- Remove outdated duplicates

## Phase 3: Update Navigation (15 minutes)

### 1. Update SUMMARY.md
```markdown
# Summary

- [Introduction](README.md)

## Getting Started
- [Installation](01-getting-started/installation.md)
- [Quick Start](01-getting-started/quick-start.md)
- [Configuration](01-getting-started/configuration.md)
- [Troubleshooting](01-getting-started/troubleshooting.md)

## User Guide
- [Basic Concepts](02-user-guide/basic-concepts.md)
- [Analysis Workflow](02-user-guide/analysis-workflow.md)
- [Anti-Pattern Detection](02-user-guide/anti-pattern-detection.md)
- [Visualization System](02-user-guide/visualization-system.md)
- [Configuration Options](02-user-guide/configuration-options.md)

## API Reference
- [Overview](03-api-reference/overview.md)
- [Analysis Engine](03-api-reference/analysis-engine.md)
- [Detector API](03-api-reference/detector-api.md)
- [AI Integration](03-api-reference/ai-integration.md)
- [Plugin System](03-api-reference/plugin-system.md)

## Architecture
- [Overview](04-architecture/overview.md)
- [System Architecture](04-architecture/system-architecture.md)
- [Database Design](04-architecture/database-design.md)
- [Component Model](04-architecture/component-model.md)
- [Security Model](04-architecture/security-model.md)

## Development
- [Contributing](05-development/contributing.md)
- [Detector Development](05-development/detector-development.md)
- [Testing Strategy](05-development/testing-strategy.md)
- [Coding Standards](05-development/coding-standards.md)
- [Release Process](05-development/release-process.md)

## Research
- [Anti-Pattern Analysis](06-research/anti-pattern-analysis/overview.md)
- [Detector Research](06-research/detector-research/)
- [Visualization Research](06-research/visualization-research/)
- [Technology Evaluation](06-research/technology-evaluation/)

## Reports
- [Project Status](07-reports/project-status.md)
- [Audit Reports](07-reports/audit-reports/)
- [Sprint Reports](07-reports/sprint-reports/)

## Examples
- [Basic Usage](08-examples/basic-usage.md)
- [Advanced Configuration](08-examples/advanced-configuration.md)
- [Custom Detectors](08-examples/custom-detectors.md)
- [Integration Examples](08-examples/integration-examples.md)

## Community
- [Guidelines](09-community/guidelines.md)
- [Support](09-community/support.md)
- [Resources](09-community/resources.md)
- [FAQ](09-community/faq.md)

## Reference
- [Error Codes](10-reference/error-codes.md)
- [Glossary](10-reference/glossary.md)
- [CLI Reference](10-reference/cli-reference.md)
- [Configuration Schema](10-reference/configuration-schema.md)
```

### 2. Update README.md
- Add clear navigation to main sections
- Include quick links to most important docs
- Add contribution guidelines link

## Phase 4: Cleanup (15 minutes)

### 1. Remove Old Structure
```bash
# After confirming all content is migrated
rm -rf docs/Architecture
rm -rf docs/Dev_Reports  
rm -rf docs/R\&D
rm -rf docs/Checklists
rm -rf docs/daily_notes
```

### 2. Update .gitignore
- Add any temporary files
- Ensure no important docs are ignored

### 3. Validate Links
- Run link checker
- Fix any broken internal references
- Update external link references

## Success Criteria

✅ All documentation follows consistent naming convention
✅ Logical folder structure with numbered ordering  
✅ No duplicate or outdated content
✅ All internal links work correctly
✅ SUMMARY.md provides clear navigation
✅ Professional appearance suitable for external users
✅ Easy to find information by category
✅ Consistent title and header formatting

## Estimated Time: 2 hours total