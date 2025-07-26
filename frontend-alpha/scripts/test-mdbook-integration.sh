#!/bin/bash

# MDBook Integration Testing Script
# This script validates the mdbook integration and content

set -e

echo "🔍 Starting MDBook Integration Tests..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    print_error "mdbook is not installed. Please install it first."
    exit 1
fi

print_status "mdbook is installed"

# Build the documentation
echo "📚 Building documentation..."
cd ../docs
mdbook build
if [ $? -eq 0 ]; then
    print_status "Documentation built successfully"
else
    print_error "Failed to build documentation"
    exit 1
fi

# Copy to frontend public directory
echo "📋 Copying documentation to frontend..."
cp -r book/* ../frontend-alpha/public/
if [ $? -eq 0 ]; then
    print_status "Documentation copied to frontend"
else
    print_error "Failed to copy documentation"
    exit 1
fi

cd ..

# Validate that key files exist
echo "🔍 Validating documentation files..."
key_files=(
    "frontend-alpha/public/index.html"
    "frontend-alpha/public/book.js"
    "frontend-alpha/public/css/general.css"
    "frontend-alpha/public/css/chrome.css"
)

for file in "${key_files[@]}"; do
    if [ -f "$file" ]; then
        print_status "Found: $file"
    else
        print_warning "Missing: $file"
    fi
done

# Check for content in key documentation pages
echo "📖 Validating documentation content..."
content_checks=(
    "frontend-alpha/public/index.html:Uveddi"
    "frontend-alpha/public/01-getting-started/installation.html:installation"
    "frontend-alpha/public/02-user-guide/basic-concepts.html:concept"
    "frontend-alpha/public/04-architecture/overview.html:architecture"
)

for check in "${content_checks[@]}"; do
    file="${check%:*}"
    content="${check#*:}"
    
    if [ -f "$file" ]; then
        if grep -q "$content" "$file"; then
            print_status "Content validation passed: $file contains '$content'"
        else
            print_warning "Content validation failed: $file missing '$content'"
        fi
    else
        print_warning "File not found for content check: $file"
    fi
done

# Check for broken internal links
echo "🔗 Checking for broken internal links..."
cd frontend-alpha/public
find . -name "*.html" -exec grep -l "href=" {} \; | while read file; do
    # Extract internal links (not starting with http)
    grep -o 'href="[^"]*"' "$file" | grep -v 'href="http' | sed 's/href="//;s/"//' | while read link; do
        # Skip anchor links and javascript
        if [[ "$link" != "#"* && "$link" != "javascript:"* ]]; then
            # Convert relative path to absolute
            if [[ "$link" == "/"* ]]; then
                target_file=".${link}"
            else
                target_file="$(dirname "$file")/${link}"
            fi
            
            # Normalize path
            target_file=$(realpath -m "$target_file" 2>/dev/null || echo "$target_file")
            
            if [ ! -f "$target_file" ]; then
                print_warning "Broken link in $file: $link -> $target_file"
            fi
        fi
    done
done

cd ../..

# Run Cypress tests if available
if [ -f "frontend-alpha/cypress.config.js" ]; then
    echo "🧪 Running Cypress tests..."
    cd frontend-alpha
    
    # Check if Cypress is installed
    if [ -d "node_modules/.bin" ] && [ -f "node_modules/.bin/cypress" ]; then
        # Start the development server in background
        npm run dev &
        DEV_PID=$!
        
        # Wait for server to start
        echo "⏳ Waiting for development server to start..."
        sleep 10
        
        # Run the mdbook integration tests
        npx cypress run --spec "cypress/e2e/mdbook-*.cy.js" --headless
        CYPRESS_EXIT_CODE=$?
        
        # Kill the development server
        kill $DEV_PID 2>/dev/null || true
        
        if [ $CYPRESS_EXIT_CODE -eq 0 ]; then
            print_status "Cypress tests passed"
        else
            print_error "Cypress tests failed"
        fi
    else
        print_warning "Cypress not installed, skipping integration tests"
    fi
    
    cd ..
else
    print_warning "Cypress config not found, skipping integration tests"
fi

# Generate summary report
echo ""
echo "📊 MDBook Integration Test Summary"
echo "=================================="
echo "✅ Documentation build: PASSED"
echo "✅ File copy: PASSED"
echo "✅ Key files: CHECKED"
echo "✅ Content validation: CHECKED"
echo "✅ Link validation: CHECKED"

if [ -n "$CYPRESS_EXIT_CODE" ]; then
    if [ $CYPRESS_EXIT_CODE -eq 0 ]; then
        echo "✅ Integration tests: PASSED"
    else
        echo "❌ Integration tests: FAILED"
    fi
fi

print_status "MDBook integration testing completed!"

echo ""
echo "🚀 Next steps:"
echo "1. Start the frontend: cd frontend-alpha && npm run dev"
echo "2. Visit http://localhost:5173/dashboard"
echo "3. Verify the documentation loads correctly in the iframe"
echo "4. Test navigation and search functionality"