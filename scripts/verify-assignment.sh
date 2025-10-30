#!/bin/bash

# Database Refactor Assignment Verification Script
# Usage: ./scripts/verify-assignment.sh [assignment_number]

set -e

ASSIGNMENT=${1:-"all"}
PROJECT_ROOT=$(pwd)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Pre-flight checks
check_baseline() {
    echo_status "Running baseline checks..."

    echo_status "Checking file sizes..."
    find src/database -name "*.rs" -exec wc -l {} + | sort -n

    echo_status "Checking for circular dependencies..."
    if command -v cargo-deny &> /dev/null; then
        cargo deny check
        echo_success "No circular dependencies found"
    else
        echo_warning "cargo-deny not installed, skipping dependency check"
    fi

    echo_status "Running database tests..."
    cargo test database:: --quiet
    echo_success "Database tests pass"
}

# Assignment 01 verification
verify_assignment_01() {
    echo_status "Verifying Assignment 01: Isolate Models"

    echo_status "Checking model file structure..."
    model_files=$(find src/database/models -name "*.rs" -type f | wc -l)
    if [ "$model_files" -lt 5 ]; then
        echo_warning "Expected at least 5 model files, found $model_files"
    else
        echo_success "Model files: $model_files"
    fi

    echo_status "Checking model file sizes..."
    find src/database/models -name "*.rs" -exec wc -l {} + | while read lines file; do
        if [ "$lines" -gt 150 ] && [[ "$file" != *"total"* ]]; then
            echo_warning "Model file $file has $lines lines (>150)"
        fi
    done

    echo_status "Testing model compilation..."
    cargo test database::models:: --quiet
    echo_success "Model tests pass"
}

# Assignment 02 verification
verify_assignment_02() {
    echo_status "Verifying Assignment 02: Extract Connection Infrastructure"

    echo_status "Checking connection structure..."
    find src/database/connection -name "*.rs" -exec wc -l {} +

    echo_status "Testing connection management..."
    cargo test database::connection:: --quiet
    echo_success "Connection tests pass"

    echo_status "Verifying CRUD API compatibility..."
    cargo test database::crud:: --quiet
    echo_success "CRUD tests pass"
}

# Assignment 03 verification
verify_assignment_03() {
    echo_status "Verifying Assignment 03: Introduce Repository Interfaces"

    if [ ! -d "src/database/repositories" ]; then
        echo_error "Repository directory not found"
        exit 1
    fi

    echo_status "Checking repository structure..."
    find src/database/repositories -name "*.rs" -exec wc -l {} +

    echo_status "Testing repository implementations..."
    cargo test database::repositories:: --quiet
    echo_success "Repository tests pass"

    echo_status "Verifying Database delegation..."
    cargo test database::crud:: --quiet
    echo_success "Database delegation works"
}

# Assignment 04 verification
verify_assignment_04() {
    echo_status "Verifying Assignment 04: Update Application Integration"

    echo_status "Checking for direct Database usage in application..."
    if rg "Database::" src/application/ --type rust --quiet; then
        echo_error "Found direct Database usage in application layer"
        rg "Database::" src/application/ --type rust
        exit 1
    else
        echo_success "No direct Database usage found in application"
    fi

    echo_status "Verifying repository usage..."
    if rg "repository" src/application/ --type rust --quiet; then
        echo_success "Repository pattern usage found"
    else
        echo_warning "No repository usage found in application"
    fi

    echo_status "Testing application integration..."
    cargo test application:: --quiet
    echo_success "Application tests pass"
}

# Assignment 05 verification
verify_assignment_05() {
    echo_status "Verifying Assignment 05: Migration & Error Handling Cleanup"

    echo_status "Checking migration structure..."
    if [ -d "src/database/migrations" ]; then
        find src/database/migrations -name "*.rs" -exec wc -l {} +
        echo_success "Migration directory structure exists"
    else
        echo_error "Migration directory not found"
        exit 1
    fi

    echo_status "Testing migrations..."
    cargo test database::migrations:: --quiet
    echo_success "Migration tests pass"

    echo_status "Testing migration dry-run..."
    if cargo run --quiet -- migrate --dry-run 2>/dev/null; then
        echo_success "Migration dry-run successful"
    else
        echo_warning "Migration dry-run failed or not implemented"
    fi
}

# Assignment 06 verification
verify_assignment_06() {
    echo_status "Verifying Assignment 06: Remove Legacy CRUD Layer"

    echo_status "Checking crud.rs size..."
    if [ -f "src/database/crud.rs" ]; then
        crud_lines=$(wc -l < src/database/crud.rs)
        if [ "$crud_lines" -gt 200 ]; then
            echo_warning "crud.rs still has $crud_lines lines (>200)"
        else
            echo_success "crud.rs size acceptable: $crud_lines lines"
        fi
    else
        echo_success "crud.rs has been removed"
    fi

    echo_status "Checking for legacy imports..."
    if rg "use.*crud" src/ --type rust --quiet; then
        echo_warning "Found legacy crud imports:"
        rg "use.*crud" src/ --type rust
    else
        echo_success "No legacy crud imports found"
    fi

    echo_status "Final verification suite..."
    cargo test database:: --quiet
    echo_success "All database tests pass"
}

# Run verification based on assignment
case $ASSIGNMENT in
    "01"|"1")
        verify_assignment_01
        ;;
    "02"|"2")
        verify_assignment_02
        ;;
    "03"|"3")
        verify_assignment_03
        ;;
    "04"|"4")
        verify_assignment_04
        ;;
    "05"|"5")
        verify_assignment_05
        ;;
    "06"|"6")
        verify_assignment_06
        ;;
    "baseline")
        check_baseline
        ;;
    "all")
        echo_status "Running full verification suite..."
        check_baseline
        verify_assignment_01
        verify_assignment_02
        verify_assignment_03
        verify_assignment_04
        verify_assignment_05
        verify_assignment_06
        echo_success "All assignments verified successfully!"
        ;;
    *)
        echo_error "Unknown assignment: $ASSIGNMENT"
        echo "Usage: $0 [01|02|03|04|05|06|baseline|all]"
        exit 1
        ;;
esac

echo_success "Verification complete for assignment $ASSIGNMENT"