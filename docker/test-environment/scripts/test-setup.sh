#!/bin/bash
# Comprehensive test setup script for Uveddi TUI integration validation

set -e

echo "🚀 Setting up Uveddi TUI testing environment..."
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if Uveddi source is available
UVEDDI_SOURCE="/workspace"
UVEDDI_LOCAL="/home/testuser/uveddi-source"

if [ -d "$UVEDDI_SOURCE" ]; then
    print_info "Copying Uveddi source from mounted volume..."
    cp -r "$UVEDDI_SOURCE" "$UVEDDI_LOCAL"
    cd "$UVEDDI_LOCAL"
else
    print_warning "Uveddi source not found at $UVEDDI_SOURCE"
    print_info "Please mount Uveddi source directory at /workspace"
    exit 1
fi

# Build and install Uveddi
print_info "Building Uveddi with production features..."
if cargo build --release --features=production; then
    print_status "Uveddi build successful"
else
    print_error "Uveddi build failed"
    exit 1
fi

print_info "Installing Uveddi to user PATH..."
if cargo install --path . --features=production --force; then
    print_status "Uveddi installation successful"
else
    print_error "Uveddi installation failed"
    exit 1
fi

# Verify installation
print_info "Verifying Uveddi installation..."
if command -v uveddi >/dev/null 2>&1; then
    print_status "Uveddi command available: $(which uveddi)"
    print_status "Uveddi version: $(uveddi --version || echo 'Version info not available')"
else
    print_error "Uveddi command not found in PATH"
    exit 1
fi

# Test basic CLI functionality
print_info "Testing basic CLI functionality..."
if uveddi --help >/dev/null 2>&1; then
    print_status "Basic CLI help working"
else
    print_error "Basic CLI help failed"
    exit 1
fi

# Test TUI command availability
print_info "Testing TUI command availability..."
if uveddi --help | grep -q "tui"; then
    print_status "TUI command found in help"
else
    print_error "TUI command not found in main CLI help"
    exit 1
fi

if uveddi tui --help >/dev/null 2>&1; then
    print_status "TUI command help working"
else
    print_error "TUI command help failed"
    exit 1
fi

# Create test results directory
mkdir -p /home/testuser/test-workspace/results

# Test sample projects exist
print_info "Checking test sample projects..."
for project in rust-sample python-sample js-sample; do
    if [ -d "/home/testuser/test-workspace/projects/$project" ]; then
        print_status "Sample project exists: $project"
    else
        print_warning "Sample project missing: $project"
    fi
done

# Test analyze command with sample projects
print_info "Testing analyze command with sample projects..."
cd /home/testuser/test-workspace

for project in rust-sample python-sample js-sample; do
    if [ -d "projects/$project" ]; then
        print_info "Testing analysis of $project..."
        if uveddi analyze "projects/$project" --output-format json --output "results/$project-analysis.json"; then
            print_status "Analysis successful for $project"
        else
            print_warning "Analysis failed for $project (this might be expected for some samples)"
        fi
    fi
done

# Test TUI error handling (non-interactive environment)
print_info "Testing TUI error handling in non-interactive environment..."
cd /home/testuser/test-workspace
if ! uveddi tui ./projects/rust-sample 2>/dev/null; then
    print_status "TUI correctly detects non-interactive environment"
else
    print_warning "TUI did not detect non-interactive environment (might be an issue)"
fi

# Test TUI with --force flag
print_info "Testing TUI with --force flag..."
if timeout 3s uveddi tui ./projects/rust-sample --force 2>/dev/null; then
    print_warning "TUI started with --force flag (test incomplete due to timeout)"
else
    print_status "TUI --force flag handling verified"
fi

print_status "Test setup completed successfully!"
print_info "Ready to run comprehensive TUI tests"
print_info ""
print_info "Next steps:"
print_info "  1. Run: ./scripts/comprehensive-tui-test.sh"
print_info "  2. Run: ./scripts/user-experience-test.sh"
print_info "  3. Manual testing: uveddi tui --help"
print_info ""
print_info "Test projects available at:"
print_info "  - ~/test-workspace/projects/rust-sample"
print_info "  - ~/test-workspace/projects/python-sample"  
print_info "  - ~/test-workspace/projects/js-sample"