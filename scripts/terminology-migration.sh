#!/bin/bash

# Terminology Migration Script
# Identifies and optionally fixes terminology inconsistencies in the Uveddi codebase

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REPORT_FILE="$PROJECT_ROOT/terminology-migration-report.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APPLY_FIXES=false
DRY_RUN=true
VERBOSE=false

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -f, --fix        Apply fixes automatically (default: dry run)"
    echo "  -v, --verbose    Show detailed output"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Dry run, generate report only"
    echo "  $0 --fix             # Apply fixes automatically"
    echo "  $0 --verbose         # Show detailed analysis"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--fix)
            APPLY_FIXES=true
            DRY_RUN=false
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Initialize report
init_report() {
    cat > "$REPORT_FILE" << 'EOF'
# Terminology Migration Report

Generated: $(date)
Project: Uveddi
Status: Analysis Complete

## Executive Summary

This report identifies terminology inconsistencies in the Uveddi codebase and provides migration recommendations based on the Ubiquitous Language Glossary.

## Findings

EOF
}

# Check for deprecated terminology patterns
check_deprecated_patterns() {
    log_info "Checking for deprecated terminology patterns..."
    
    local found_issues=0
    
    # Define deprecated patterns and their replacements
    declare -A deprecated_patterns=(
        ["AntiPatternType"]="AntiPattern"
        ["AiSuggestion"]="AiInsight" 
        ["ai_explanation"]="ai_insight"
        ["ai_refactoring_suggestion"]="ai_insight"
        ["ModuleAnalysis"]="ComponentAnalysis"
        ["anti_pattern_type_id"]="anti_pattern_id"
        ["DatabaseService"]="DatabaseRepository"
    )
    
    echo "### Deprecated Terminology Usage" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    for pattern in "${!deprecated_patterns[@]}"; do
        local replacement="${deprecated_patterns[$pattern]}"
        
        log_info "Checking for '$pattern' -> should be '$replacement'"
        
        # Search in Rust source files
        local matches=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -Hn "$pattern" {} \; 2>/dev/null || true)
        
        if [[ -n "$matches" ]]; then
            found_issues=$((found_issues + 1))
            
            echo "#### $pattern → $replacement" >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
            echo "**Priority:** High" >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
            echo "**Occurrences:**" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
            echo "$matches" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
            
            if [[ "$VERBOSE" == true ]]; then
                log_warn "Found $pattern in:"
                echo "$matches" | while IFS= read -r line; do
                    echo "  $line"
                done
            fi
            
            # Apply fixes if requested
            if [[ "$APPLY_FIXES" == true ]]; then
                log_info "Applying fix: $pattern -> $replacement"
                find "$PROJECT_ROOT/src" -name "*.rs" -exec sed -i "s/$pattern/$replacement/g" {} \;
                log_success "Fixed $pattern -> $replacement"
            fi
        fi
    done
    
    if [[ $found_issues -eq 0 ]]; then
        echo "✅ No deprecated terminology patterns found." >> "$REPORT_FILE"
        log_success "No deprecated terminology patterns found"
    else
        log_warn "Found $found_issues deprecated terminology patterns"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Check for naming convention violations
check_naming_conventions() {
    log_info "Checking naming convention compliance..."
    
    echo "### Naming Convention Violations" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    local violations=0
    
    # Check for struct names not in PascalCase
    log_info "Checking struct naming conventions..."
    local bad_structs=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -Hn "^struct [a-z]" {} \; 2>/dev/null || true)
    
    if [[ -n "$bad_structs" ]]; then
        violations=$((violations + 1))
        echo "#### Struct Names Not in PascalCase" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "**Issue:** Struct names should use PascalCase" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "$bad_structs" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
    
    # Check for field names not in snake_case  
    log_info "Checking field naming conventions..."
    local bad_fields=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -Hn "pub [A-Z][a-zA-Z]*:" {} \; 2>/dev/null || true)
    
    if [[ -n "$bad_fields" ]]; then
        violations=$((violations + 1))
        echo "#### Field Names Not in snake_case" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "**Issue:** Field names should use snake_case" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "$bad_fields" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
    
    if [[ $violations -eq 0 ]]; then
        echo "✅ No naming convention violations found." >> "$REPORT_FILE"
        log_success "No naming convention violations found"
    else
        log_warn "Found $violations naming convention violations"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Check for inconsistent terminology within the same domain
check_domain_consistency() {
    log_info "Checking domain-specific terminology consistency..."
    
    echo "### Domain Terminology Consistency" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # AI Domain consistency
    echo "#### AI Domain" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    local ai_inconsistencies=0
    
    # Check for AI provider naming inconsistencies
    local provider_variants=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -o "Ai[A-Za-z]*Provider\|Llm[A-Za-z]*Provider" {} \; 2>/dev/null | sort | uniq || true)
    
    if [[ -n "$provider_variants" ]]; then
        ai_inconsistencies=$((ai_inconsistencies + 1))
        echo "**AI Provider Naming Variants:**" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "$provider_variants" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "**Recommendation:** Standardize on 'AiProvider' suffix" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
    
    # Analysis Domain consistency
    echo "#### Analysis Domain" >> "$REPORT_FILE"  
    echo "" >> "$REPORT_FILE"
    
    local analysis_inconsistencies=0
    
    # Check for engine/manager/service inconsistencies
    local engine_variants=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -o "[A-Za-z]*Engine\|[A-Za-z]*Manager\|[A-Za-z]*Service" {} \; 2>/dev/null | sort | uniq || true)
    
    if [[ -n "$engine_variants" ]]; then
        analysis_inconsistencies=$((analysis_inconsistencies + 1))
        echo "**Engine/Manager/Service Usage:**" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "$engine_variants" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "**Review needed:** Ensure proper Engine/Manager/Service distinction" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
    
    local total_inconsistencies=$((ai_inconsistencies + analysis_inconsistencies))
    
    if [[ $total_inconsistencies -eq 0 ]]; then
        echo "✅ No domain consistency issues found." >> "$REPORT_FILE"
        log_success "No domain consistency issues found"
    else
        log_warn "Found $total_inconsistencies potential domain consistency issues"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Check documentation consistency
check_documentation_consistency() {
    log_info "Checking documentation terminology consistency..."
    
    echo "### Documentation Consistency" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    local doc_issues=0
    
    # Check if README uses standardized terminology
    if [[ -f "$PROJECT_ROOT/README.md" ]]; then
        local readme_deprecated=$(grep -n "AntiPatternType\|AiSuggestion\|ModuleAnalysis" "$PROJECT_ROOT/README.md" || true)
        
        if [[ -n "$readme_deprecated" ]]; then
            doc_issues=$((doc_issues + 1))
            echo "#### README.md Terminology Issues" >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
            echo "$readme_deprecated" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
        fi
    fi
    
    # Check other documentation files
    local doc_deprecated=$(find "$PROJECT_ROOT/docs" -name "*.md" -exec grep -Hn "AntiPatternType\|AiSuggestion\|ModuleAnalysis" {} \; 2>/dev/null || true)
    
    if [[ -n "$doc_deprecated" ]]; then
        doc_issues=$((doc_issues + 1))
        echo "#### Documentation Files with Deprecated Terms" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "$doc_deprecated" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        
        if [[ "$APPLY_FIXES" == true ]]; then
            log_info "Updating documentation terminology..."
            find "$PROJECT_ROOT/docs" -name "*.md" -exec sed -i 's/AntiPatternType/AntiPattern/g' {} \;
            find "$PROJECT_ROOT/docs" -name "*.md" -exec sed -i 's/AiSuggestion/AiInsight/g' {} \;
            find "$PROJECT_ROOT/docs" -name "*.md" -exec sed -i 's/ModuleAnalysis/ComponentAnalysis/g' {} \;
            log_success "Updated documentation terminology"
        fi
    fi
    
    if [[ $doc_issues -eq 0 ]]; then
        echo "✅ No documentation consistency issues found." >> "$REPORT_FILE"
        log_success "No documentation consistency issues found"
    else
        log_warn "Found $doc_issues documentation consistency issues"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Generate migration recommendations
generate_recommendations() {
    log_info "Generating migration recommendations..."
    
    cat >> "$REPORT_FILE" << 'EOF'
## Migration Recommendations

### Immediate Actions (High Priority)

1. **Update Deprecated Terms**
   - Replace all instances of `AntiPatternType` with `AntiPattern`
   - Replace `AiSuggestion` with `AiInsight` 
   - Replace `ModuleAnalysis` with `ComponentAnalysis`

2. **Fix Database Schema**
   - Rename `anti_pattern_type_id` to `anti_pattern_id`
   - Update foreign key references accordingly

3. **API Consistency**
   - Ensure public APIs use standardized terminology
   - Update API documentation and examples

### Medium Priority Actions

1. **Code Comments**
   - Review and update inline documentation
   - Ensure comments use glossary terminology

2. **Test Files**
   - Update test names and documentation
   - Ensure test code follows naming conventions

3. **Configuration Files**
   - Review TOML and YAML configurations
   - Standardize configuration key names

### Long-term Actions (Low Priority)

1. **External Documentation**
   - Update user guides and tutorials
   - Review blog posts and examples

2. **Development Tools**
   - Update IDE configurations
   - Create custom lint rules

3. **Community Resources**
   - Update contributor documentation
   - Review issue templates

## Implementation Plan

### Phase 1: Critical Path (Week 1)
- [ ] Update core analysis types
- [ ] Fix public API inconsistencies  
- [ ] Update database schema

### Phase 2: Internal Consistency (Week 2-3)
- [ ] Update internal documentation
- [ ] Fix naming convention violations
- [ ] Update test terminology

### Phase 3: Documentation & Tools (Week 4)
- [ ] Update external documentation
- [ ] Implement automated checks
- [ ] Update development tooling

## Success Metrics

- **Terminology Consistency**: >95% standardized term usage
- **Review Efficiency**: Reduce terminology-related PR comments by 80%
- **Developer Onboarding**: Reduce domain concept confusion time by 50%

EOF
}

# Generate compliance summary
generate_compliance_summary() {
    log_info "Generating compliance summary..."
    
    local total_rust_files=$(find "$PROJECT_ROOT/src" -name "*.rs" | wc -l)
    local total_doc_files=$(find "$PROJECT_ROOT/docs" -name "*.md" | wc -l 2>/dev/null || echo "0")
    
    # Count standardized vs deprecated terms
    local standardized_count=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -o "AntiPattern\|AiInsight\|ComponentAnalysis" {} \; 2>/dev/null | wc -l || echo "0")
    local deprecated_count=$(find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -o "AntiPatternType\|AiSuggestion\|ModuleAnalysis" {} \; 2>/dev/null | wc -l || echo "0")
    
    local total_terms=$((standardized_count + deprecated_count))
    local compliance_percentage=0
    
    if [[ $total_terms -gt 0 ]]; then
        compliance_percentage=$((standardized_count * 100 / total_terms))
    fi
    
    cat >> "$REPORT_FILE" << EOF

## Compliance Summary

- **Rust Source Files Analyzed**: $total_rust_files
- **Documentation Files Analyzed**: $total_doc_files
- **Standardized Terms Found**: $standardized_count
- **Deprecated Terms Found**: $deprecated_count
- **Current Compliance Rate**: ${compliance_percentage}%

## Next Steps

1. **Review this report** and prioritize fixes based on impact
2. **Run with --fix flag** to apply automatic corrections
3. **Update CI/CD pipeline** to prevent future terminology drift
4. **Train development team** on ubiquitous language standards

---

*Generated by: $0*
*Report location: $REPORT_FILE*

EOF
}

# Main execution
main() {
    log_info "Starting terminology migration analysis..."
    log_info "Project root: $PROJECT_ROOT"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "Running in DRY RUN mode - no changes will be applied"
    else
        log_warn "APPLYING FIXES - changes will be made to source files"
    fi
    
    # Initialize report
    init_report
    
    # Run all checks
    check_deprecated_patterns
    check_naming_conventions  
    check_domain_consistency
    check_documentation_consistency
    
    # Generate recommendations and summary
    generate_recommendations
    generate_compliance_summary
    
    log_success "Analysis complete! Report generated: $REPORT_FILE"
    
    if [[ "$APPLY_FIXES" == true ]]; then
        log_success "Automatic fixes have been applied"
        log_info "Please review changes and run tests before committing"
    else
        log_info "To apply fixes automatically, run: $0 --fix"
    fi
}

# Execute main function
main "$@"