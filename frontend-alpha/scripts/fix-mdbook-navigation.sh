#!/bin/bash

# Fix MDBook Navigation Issues
# This script addresses common navigation problems in mdbook integration

set -e

echo "🔧 Fixing MDBook Navigation Issues..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Rebuild documentation with proper configuration
echo "📚 Rebuilding documentation..."
cd ../docs

# Check if book.toml has proper configuration
if ! grep -q "git-repository-url" book.toml; then
    echo "🔧 Adding git repository URL to book.toml..."
    echo "" >> book.toml
    echo "[output.html]" >> book.toml
    echo "git-repository-url = \"https://github.com/botzrDev/uveddi\"" >> book.toml
    echo "edit-url-template = \"https://github.com/botzrDev/uveddi/edit/main/docs/{path}\"" >> book.toml
fi

# Ensure proper navigation structure
if ! grep -q "curly-quotes" book.toml; then
    echo "curly-quotes = true" >> book.toml
fi

# Build with clean slate
rm -rf book/
mdbook build

if [ $? -eq 0 ]; then
    print_status "Documentation rebuilt successfully"
else
    print_error "Failed to rebuild documentation"
    exit 1
fi

cd ..

# Fix relative paths in HTML files
echo "🔗 Fixing relative paths..."
cd docs/book

# Fix CSS and JS paths to be relative
find . -name "*.html" -exec sed -i 's|href="/css/|href="css/|g' {} \;
find . -name "*.html" -exec sed -i 's|src="/js/|src="js/|g' {} \;
find . -name "*.html" -exec sed -i 's|href="/book.js"|href="book.js"|g' {} \;
find . -name "*.html" -exec sed -i 's|src="/book.js"|src="book.js"|g' {} \;

# Fix FontAwesome and other external resources
find . -name "*.html" -exec sed -i 's|href="/FontAwesome/|href="FontAwesome/|g' {} \;

print_status "Fixed relative paths in HTML files"

cd ../..

# Copy fixed files to frontend
echo "📋 Copying fixed documentation to frontend..."
rm -rf frontend-alpha/public/css frontend-alpha/public/js frontend-alpha/public/*.html frontend-alpha/public/*.js
cp -r docs/book/* frontend-alpha/public/

print_status "Copied fixed documentation to frontend"

# Create a custom iframe navigation helper
echo "🔧 Creating iframe navigation helper..."
cat > frontend-alpha/public/iframe-helper.js << 'EOF'
// Iframe Navigation Helper for MDBook
(function() {
    'use strict';
    
    // Fix relative paths for iframe context
    function fixPaths() {
        // Fix all relative links
        const links = document.querySelectorAll('a[href]');
        links.forEach(link => {
            const href = link.getAttribute('href');
            if (href && !href.startsWith('http') && !href.startsWith('#') && !href.startsWith('javascript:')) {
                // Ensure links work within iframe context
                link.addEventListener('click', function(e) {
                    e.preventDefault();
                    window.location.href = href;
                });
            }
        });
        
        // Fix search functionality
        const searchInput = document.getElementById('searchbar');
        if (searchInput) {
            searchInput.addEventListener('keyup', function(e) {
                if (e.key === 'Enter') {
                    // Trigger search
                    const searchTerm = this.value;
                    if (window.search && typeof window.search.search === 'function') {
                        window.search.search(searchTerm);
                    }
                }
            });
        }
        
        // Fix mobile navigation
        const mobileToggle = document.querySelector('.mobile-nav-toggle');
        if (mobileToggle) {
            mobileToggle.addEventListener('click', function() {
                const sidebar = document.querySelector('.sidebar');
                if (sidebar) {
                    sidebar.classList.toggle('visible');
                }
            });
        }
    }
    
    // Run fixes when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', fixPaths);
    } else {
        fixPaths();
    }
    
    // Re-run fixes when navigation occurs
    window.addEventListener('popstate', fixPaths);
    
    // Fix for mdbook's navigation
    const originalPushState = history.pushState;
    history.pushState = function() {
        originalPushState.apply(history, arguments);
        setTimeout(fixPaths, 100);
    };
})();
EOF

# Inject the helper script into all HTML files
find frontend-alpha/public -name "*.html" -exec sed -i 's|</body>|<script src="iframe-helper.js"></script></body>|g' {} \;

print_status "Created and injected iframe navigation helper"

# Validate the fixes
echo "🔍 Validating navigation fixes..."

# Check that key files exist
key_files=(
    "frontend-alpha/public/index.html"
    "frontend-alpha/public/book.js"
    "frontend-alpha/public/css/general.css"
    "frontend-alpha/public/iframe-helper.js"
)

for file in "${key_files[@]}"; do
    if [ -f "$file" ]; then
        print_status "Validated: $file exists"
    else
        print_error "Missing: $file"
    fi
done

# Check that HTML files have proper structure
html_files=$(find frontend-alpha/public -name "*.html" | head -5)
for file in $html_files; do
    if grep -q "<nav" "$file" && grep -q "sidebar" "$file"; then
        print_status "Navigation structure found in: $(basename $file)"
    else
        print_warning "Navigation structure missing in: $(basename $file)"
    fi
done

echo ""
print_status "MDBook navigation fixes completed!"

echo ""
echo "🚀 To test the fixes:"
echo "1. Run: cd frontend-alpha && npm run dev"
echo "2. Visit: http://localhost:5173/dashboard"
echo "3. Test navigation within the documentation iframe"
echo "4. Run: ./scripts/test-mdbook-integration.sh for comprehensive testing"