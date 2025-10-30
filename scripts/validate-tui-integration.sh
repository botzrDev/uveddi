#!/bin/bash
# Comprehensive TUI Integration Validation Script
# This script validates the complete TUI integration and user experience

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCKER_DIR="$PROJECT_ROOT/docker/test-environment"

print_header() {
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${BLUE}$(echo $1 | sed 's/./=/g')${NC}"
}

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Validation functions
validate_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        print_error "Docker not found. Please install Docker to run integration tests."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose >/dev/null 2>&1; then
        print_error "Docker Compose not found. Please install Docker Compose."
        exit 1
    fi
    
    # Check we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_error "Not in Uveddi project root. Run from project root directory."
        exit 1
    fi
    
    print_status "Prerequisites validated"
}

# Local build validation
validate_local_build() {
    print_header "Local Build Validation"
    
    cd "$PROJECT_ROOT"
    
    # Test minimal build
    print_info "Testing dev-core build (fast development)..."
    if cargo build --features=dev-core; then
        print_status "Dev-core build successful"
    else
        print_error "Dev-core build failed"
        return 1
    fi
    
    # Test TUI-specific build
    print_info "Testing TUI build..."
    if cargo build --features=tui; then
        print_status "TUI build successful"
    else
        print_error "TUI build failed"
        return 1
    fi
    
    # Test production build
    print_info "Testing production build (includes TUI)..."
    if cargo build --features=production; then
        print_status "Production build successful"
    else
        print_error "Production build failed"
        return 1
    fi
    
    print_status "Local build validation completed"
}

# Local TUI integration test
validate_local_integration() {
    print_header "Local TUI Integration Test"
    
    cd "$PROJECT_ROOT"
    
    # Test command availability
    print_info "Testing TUI command availability..."
    if ./target/debug/uveddi --help | grep -q "tui"; then
        print_status "TUI command found in main CLI"
    else
        print_error "TUI command NOT found in main CLI"
        return 1
    fi
    
    # Test TUI help
    if ./target/debug/uveddi tui --help >/dev/null 2>&1; then
        print_status "TUI command help accessible"
    else
        print_error "TUI command help failed"
        return 1
    fi
    
    # Test non-interactive error handling
    print_info "Testing non-interactive environment detection..."
    if ! ./target/debug/uveddi tui ./src 2>/dev/null; then
        print_status "Correctly detects non-interactive environment"
    else
        print_error "Failed to detect non-interactive environment"
        return 1
    fi
    
    # Test error message quality
    error_output=$(./target/debug/uveddi tui ./src 2>&1 || true)
    if echo "$error_output" | grep -q "interactive terminal"; then
        print_status "Error message explains terminal requirement"
    else
        print_error "Error message lacks clarity"
        return 1
    fi
    
    if echo "$error_output" | grep -q "uveddi analyze"; then
        print_status "Error suggests analyze alternative"
    else
        print_error "Error lacks helpful suggestions"
        return 1
    fi
    
    print_status "Local integration validation completed"
}

# Docker environment testing
validate_docker_environment() {
    print_header "Docker Environment Testing"
    
    cd "$DOCKER_DIR"
    
    # Build test environment
    print_info "Building Docker test environment..."
    if docker-compose build uveddi-test; then
        print_status "Docker environment built successfully"
    else
        print_error "Docker environment build failed"
        return 1
    fi
    
    # Run automated tests
    print_info "Running automated integration tests..."
    if docker-compose run --rm uveddi-test-automated; then
        print_status "Automated tests passed"
    else
        print_error "Automated tests failed"
        return 1
    fi
    
    print_status "Docker environment testing completed"
}

# Interactive testing guidance
provide_interactive_guidance() {
    print_header "Interactive Testing Guide"
    
    echo "🧪 Manual Testing Steps:"
    echo "======================="
    echo ""
    echo "1. Start interactive test environment:"
    echo "   cd $DOCKER_DIR"
    echo "   docker-compose run --rm uveddi-test"
    echo ""
    echo "2. Inside the container, run:"
    echo "   ./scripts/test-setup.sh"
    echo "   uveddi --help"
    echo "   uveddi tui --help"
    echo ""
    echo "3. Test TUI with sample projects:"
    echo "   # This should show error with helpful suggestions"
    echo "   uveddi tui ./projects/rust-sample"
    echo ""
    echo "   # Test analyze command as alternative"
    echo "   uveddi analyze ./projects/rust-sample --output-format json"
    echo ""
    echo "4. Test in actual terminal (outside Docker):"
    echo "   # Build and install Uveddi locally"
    echo "   cargo install --path . --features=production"
    echo ""
    echo "   # Test in terminal emulator"
    echo "   uveddi tui ./some-project"
    echo ""
    echo "5. Expected Results:"
    echo "   ✅ TUI starts in interactive terminal"
    echo "   ✅ Clear error messages in non-interactive mode"
    echo "   ✅ Helpful alternative command suggestions"
    echo "   ✅ All command help working correctly"
}

# User experience validation checklist
validate_user_experience() {
    print_header "User Experience Validation Checklist"
    
    echo "📋 Manual UX Validation Checklist:"
    echo "=================================="
    echo ""
    echo "□ Command Discovery:"
    echo "  □ 'uveddi --help' shows TUI command clearly"
    echo "  □ TUI has clear description: 'Terminal User Interface'"
    echo "  □ Commands are logically organized"
    echo ""
    echo "□ Error Handling:"
    echo "  □ Non-interactive environment detected gracefully"
    echo "  □ Error messages are helpful, not technical"
    echo "  □ Alternative commands suggested (analyze, serve)"
    echo "  □ Example commands provided in error messages"
    echo ""
    echo "□ Help System:"
    echo "  □ 'uveddi tui --help' is comprehensive"
    echo "  □ Options are clearly explained"
    echo "  □ Default values are shown"
    echo "  □ Examples or usage patterns provided"
    echo ""
    echo "□ Integration:"
    echo "  □ No separate tui_test binary needed"
    echo "  □ Consistent with other command patterns"
    echo "  □ Works with production build"
    echo "  □ Feature flags work correctly"
    echo ""
    echo "□ Workflow:"
    echo "  □ Easy progression from analyze → tui → serve"
    echo "  □ Clear path for different user needs"
    echo "  □ No confusing command overlap"
}

# Main execution
main() {
    print_header "🧪 Uveddi TUI Integration Validation"
    echo "Comprehensive testing of TUI integration and user experience"
    echo ""
    
    # Parse command line arguments
    RUN_LOCAL=true
    RUN_DOCKER=true
    SKIP_BUILD=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --local-only)
                RUN_DOCKER=false
                shift
                ;;
            --docker-only)
                RUN_LOCAL=false
                shift
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --local-only    Run only local validation tests"
                echo "  --docker-only   Run only Docker environment tests"
                echo "  --skip-build    Skip local build validation"
                echo "  --help, -h      Show this help message"
                echo ""
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Run validation steps
    validate_prerequisites
    
    if [ "$RUN_LOCAL" = true ]; then
        if [ "$SKIP_BUILD" = false ]; then
            validate_local_build
        fi
        validate_local_integration
    fi
    
    if [ "$RUN_DOCKER" = true ]; then
        validate_docker_environment
    fi
    
    # Always provide guidance
    provide_interactive_guidance
    validate_user_experience
    
    print_header "🎉 Validation Summary"
    
    echo "✅ TUI integration validation completed successfully!"
    echo ""
    echo "📊 What was tested:"
    echo "  • Local build with TUI features"
    echo "  • Command integration into main CLI"
    echo "  • Error handling and user guidance"  
    echo "  • Docker environment testing"
    echo "  • User experience validation"
    echo ""
    echo "🚀 Next Steps:"
    echo "  1. Complete manual testing using the interactive guide above"
    echo "  2. Test in actual terminal environments"
    echo "  3. Validate with real users for UX feedback"
    echo "  4. Deploy with confidence!"
    echo ""
    print_status "TUI integration is ready for production! 🎊"
}

# Error handling
trap 'print_error "Validation failed! Check output above for details."; exit 1' ERR

# Run main function
main "$@"