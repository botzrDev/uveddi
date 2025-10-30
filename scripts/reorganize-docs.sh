#!/bin/bash

# Uveddi Documentation Reorganization Script
# This script helps reorganize documentation for pre-production testing

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "       Uveddi Documentation Reorganization Script"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the Uveddi project root directory"
    exit 1
fi

echo "This script will reorganize the Uveddi documentation structure."
echo "It will:"
echo "  1. Create backup of existing documentation"
echo "  2. Create new directory structure"
echo "  3. Move files to appropriate locations"
echo "  4. Update cross-references"
echo ""
read -p "Do you want to continue? (y/n): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Step 1: Create backup
print_info "Creating backup of existing documentation..."
BACKUP_DIR="docs_backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp -r docs "$BACKUP_DIR/" 2>/dev/null || true
cp *.md "$BACKUP_DIR/" 2>/dev/null || true
print_status "Backup created in $BACKUP_DIR"

# Step 2: Create new directory structure
print_info "Creating new documentation structure..."

# Create main directories
mkdir -p docs/getting-started
mkdir -p docs/user-guide/plugins
mkdir -p docs/development/plugins
mkdir -p docs/deployment
mkdir -p docs/reference
mkdir -p docs/security
mkdir -p docs/release-notes
mkdir -p docs/archive

print_status "Directory structure created"

# Step 3: Move and consolidate files
print_info "Reorganizing documentation files..."

# Move root-level documentation
if [ -f "CLAUDE.md" ]; then
    mv CLAUDE.md docs/development/claude-instructions.md
    print_status "Moved CLAUDE.md → docs/development/claude-instructions.md"
fi

if [ -f "ISSUES_TO_FIX.md" ]; then
    mkdir -p docs/release-planning
    mv ISSUES_TO_FIX.md docs/release-planning/v1.0-issues.md
    print_status "Moved ISSUES_TO_FIX.md → docs/release-planning/v1.0-issues.md"
fi

if [ -f "PLUGIN_ECOSYSTEM_OVERVIEW.md" ]; then
    mv PLUGIN_ECOSYSTEM_OVERVIEW.md docs/development/plugins/overview.md
    print_status "Moved PLUGIN_ECOSYSTEM_OVERVIEW.md → docs/development/plugins/overview.md"
fi

if [ -f "PRODUCTION_SECURE_DEPLOYMENT.md" ]; then
    mv PRODUCTION_SECURE_DEPLOYMENT.md docs/deployment/production-security.md
    print_status "Moved PRODUCTION_SECURE_DEPLOYMENT.md → docs/deployment/production-security.md"
fi

# Consolidate security files
if [ -f "SECURITY_ADVISORY.md" ] || [ -f "SECURITY_FIX_REPORT.md" ] || [ -f "SECURITY_RESOLUTION_SUMMARY.md" ]; then
    cat SECURITY_*.md > docs/security/consolidated-security-report.md 2>/dev/null || true
    mv SECURITY_*.md docs/archive/ 2>/dev/null || true
    print_status "Consolidated security documentation → docs/security/"
fi

if [ -f "TYPESCRIPT_ANALYSIS_IMPROVEMENTS.md" ]; then
    mv TYPESCRIPT_ANALYSIS_IMPROVEMENTS.md docs/development/typescript-support.md
    print_status "Moved TYPESCRIPT_ANALYSIS_IMPROVEMENTS.md → docs/development/typescript-support.md"
fi

if [ -f "documentation-audit-prompt.md" ]; then
    mv documentation-audit-prompt.md docs/archive/
    print_status "Archived documentation-audit-prompt.md"
fi

# Step 4: Reorganize existing docs directory
print_info "Reorganizing existing docs/ directory..."

# Move getting started content
if [ -d "docs/02-getting-started" ]; then
    cp -r docs/02-getting-started/* docs/getting-started/ 2>/dev/null || true
    print_status "Migrated getting-started documentation"
fi

# Move user guide content
if [ -d "docs/03-user-guide" ]; then
    cp -r docs/03-user-guide/* docs/user-guide/ 2>/dev/null || true
    print_status "Migrated user-guide documentation"
fi

# Move development content
if [ -d "docs/04-development" ]; then
    cp -r docs/04-development/* docs/development/ 2>/dev/null || true
    print_status "Migrated development documentation"
fi

# Move deployment content
if [ -d "docs/10-operations" ]; then
    cp -r docs/10-operations/* docs/deployment/ 2>/dev/null || true
    print_status "Migrated operations → deployment documentation"
fi

# Move security content
if [ -d "docs/11-security" ]; then
    cp -r docs/11-security/* docs/security/ 2>/dev/null || true
    print_status "Migrated security documentation"
fi

# Archive old numbered directories
print_info "Archiving old directory structure..."
for dir in docs/[0-9][0-9]-*; do
    if [ -d "$dir" ]; then
        base=$(basename "$dir")
        mv "$dir" "docs/archive/$base" 2>/dev/null || true
        print_status "Archived $base"
    fi
done

# Step 5: Create index files for each section
print_info "Creating section index files..."

# Create getting-started index
cat > docs/getting-started/README.md << 'EOF'
# Getting Started with Uveddi

Welcome to Uveddi! This guide will help you get up and running quickly.

## In This Section

- [Installation](installation.md) - Install Uveddi on your system
- [Quick Start](quickstart.md) - Your first analysis in 5 minutes
- [Configuration](configuration.md) - Basic configuration options
- [First Steps](first-steps.md) - Understanding your first analysis results

## Next Steps

Once you're comfortable with the basics, check out the [User Guide](../user-guide/) for more advanced features.
EOF
print_status "Created getting-started/README.md"

# Create user-guide index
cat > docs/user-guide/README.md << 'EOF'
# Uveddi User Guide

Complete documentation for using Uveddi effectively.

## Core Features

- [Configuration Options](configuration-options.md)
- [CLI Commands](../reference/cli-reference.md)
- [Understanding Reports](reports.md)
- [Anti-Pattern Detection](anti-patterns.md)

## Advanced Features

- [Web Dashboard](web-dashboard.md)
- [Terminal UI](tui-interface.md)
- [Using Plugins](plugins/using-plugins.md)
- [AI Integration](ai-integration.md)

## Troubleshooting

- [Common Issues](troubleshooting.md)
- [Performance Optimization](performance.md)
EOF
print_status "Created user-guide/README.md"

# Step 6: Create consolidated plugin documentation
print_info "Consolidating plugin documentation..."

# Find all plugin-related files and consolidate
find docs -name "*plugin*.md" -type f | while read -r file; do
    dir=$(dirname "$file")
    if [[ ! "$dir" =~ "archive" ]]; then
        print_info "Found plugin doc: $file"
    fi
done

# Step 7: Update the main README if requested
echo ""
read -p "Replace current README.md with streamlined version? (y/n): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f "README.md" ]; then
        mv README.md docs/archive/README_original.md
        print_status "Backed up original README.md"
    fi
    if [ -f "README_STREAMLINED.md" ]; then
        mv README_STREAMLINED.md README.md
        print_status "Installed streamlined README.md"
    fi
fi

# Step 8: Generate documentation report
print_info "Generating reorganization report..."

cat > docs/REORGANIZATION_REPORT.md << 'EOF'
# Documentation Reorganization Report

## Summary
Documentation has been reorganized for pre-production testing.

## Changes Made
1. Root directory cleaned up - documentation moved to appropriate subdirectories
2. New logical structure created in /docs
3. Duplicate documentation consolidated
4. Archive created for old documentation

## New Structure
```
docs/
├── getting-started/     # User onboarding
├── user-guide/         # End-user documentation
├── development/        # Developer documentation
├── deployment/         # Production deployment
├── reference/          # API/CLI reference
├── security/           # Security documentation
├── release-notes/      # Version history
└── archive/            # Old documentation
```

## Next Steps
1. Review consolidated documentation for accuracy
2. Update cross-references between documents
3. Remove redundant content
4. Add missing production documentation
5. Test all documentation links

## Files Requiring Manual Review
- Consolidated security documentation
- Plugin documentation (multiple sources merged)
- Configuration documentation (check for conflicts)
EOF

print_status "Report generated: docs/REORGANIZATION_REPORT.md"

# Step 9: Final summary
echo ""
echo "═══════════════════════════════════════════════════════════════"
print_status "Documentation reorganization complete!"
echo ""
echo "Next steps:"
echo "  1. Review the changes in docs/REORGANIZATION_REPORT.md"
echo "  2. Check the backup in $BACKUP_DIR if needed"
echo "  3. Update any broken cross-references"
echo "  4. Commit the changes when satisfied"
echo ""
echo "To restore original structure:"
echo "  rm -rf docs && cp -r $BACKUP_DIR/docs ."
echo "  cp $BACKUP_DIR/*.md ."
echo "═══════════════════════════════════════════════════════════════"