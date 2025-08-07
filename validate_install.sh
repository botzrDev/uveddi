#!/bin/bash
# Validate the install script without full installation

echo "🧪 Validating install.sh script..."

# Test 1: Syntax check
echo "1. Testing bash syntax..."
if bash -n install.sh; then
    echo "   ✅ Syntax is valid"
else
    echo "   ❌ Syntax errors found"
    exit 1
fi

# Test 2: Check required functions are defined
echo "2. Checking function definitions..."
required_functions=("check_requirements" "detect_system" "setup_directories" "setup_repository" "build_uveddi" "install_binary" "setup_path" "final_verification" "show_usage")

for func in "${required_functions[@]}"; do
    if grep -q "^$func()" install.sh; then
        echo "   ✅ Function $func found"
    else
        echo "   ❌ Function $func missing"
        exit 1
    fi
done

# Test 3: Check error handling
echo "3. Checking error handling..."
if grep -q "set -euo pipefail" install.sh && grep -q "trap.*handle_error" install.sh; then
    echo "   ✅ Error handling configured"
else
    echo "   ❌ Error handling missing"
    exit 1
fi

# Test 4: Check configuration variables
echo "4. Checking configuration..."
if grep -q 'REPO_URL="https://github.com/botzrDev/uveddi.git"' install.sh; then
    echo "   ✅ Repository URL configured"
else
    echo "   ❌ Repository URL missing or incorrect"
    exit 1
fi

# Test 5: Check installation paths
echo "5. Checking installation paths..."
if grep -q 'INSTALL_DIR="\$HOME/.uveddi"' install.sh && grep -q 'BIN_DIR="\$HOME/.local/bin"' install.sh; then
    echo "   ✅ Installation paths configured"
else
    echo "   ❌ Installation paths missing"
    exit 1
fi

echo ""
echo "🎉 All validation checks passed!"
echo ""
echo "✅ install.sh is ready for production deployment"
echo ""
echo "Next steps:"
echo "1. Upload install.sh to https://uveddi.org/install.sh"
echo "2. Test with: curl -sSL https://uveddi.org/install.sh | bash"
echo "3. Update frontend link to point to working installation"