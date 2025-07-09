#!/bin/bash

# UV-81 Quick Fix Script for Immediate Error Reduction
# Run this script to fix the most critical compilation issues

echo "🚀 Starting Uveddi compilation fixes (UV-81)..."

# Phase 1: Fix tree-sitter stub duplicates (CRITICAL)
echo "📝 Phase 1: Removing duplicate tree-sitter method definitions..."

# Create backup
cp src/ast/tree_sitter/tree_sitter_stub.rs src/ast/tree_sitter/tree_sitter_stub.rs.backup

# Remove the first set of duplicate methods (lines 26-33)
sed -i '26,33d' src/ast/tree_sitter/tree_sitter_stub.rs

echo "✅ Removed duplicate method definitions"

# Phase 2: Quick field access fixes
echo "📝 Phase 2: Fixing common field access patterns..."

# Fix .display() calls on String fields (should be direct string access)
find src/analysis/detectors -name "*.rs" -exec sed -i 's/\.file_path\.display()/\.file_path/g' {} \;

# Fix .to_str() calls on String fields  
find src/analysis/detectors -name "*.rs" -exec sed -i 's/\.file_path\.to_str()\.unwrap_or("")\.to_string()/\.file_path\.clone()/g' {} \;

echo "✅ Fixed common field access patterns"

# Phase 3: Check progress
echo "📝 Phase 3: Checking compilation progress..."

ERROR_COUNT=$(cargo check --all-targets 2>&1 | grep -c "error\[")
echo "Current error count: $ERROR_COUNT"

if [ "$ERROR_COUNT" -lt 100 ]; then
    echo "🎉 Great progress! Error count reduced significantly."
    echo "💡 Next: Focus on remaining method implementation issues"
else
    echo "⚠️  Still many errors. Check tree-sitter stub for remaining duplicates."
fi

# Phase 4: Show remaining error summary
echo "📊 Remaining error types:"
cargo check --all-targets 2>&1 | grep "error\[" | cut -d: -f4 | sort | uniq -c | head -10

echo ""
echo "🔧 Next steps:"
echo "1. Review remaining tree-sitter duplicate errors"
echo "2. Add missing method implementations (LongMethodsDetector::analyze)"
echo "3. Fix configuration struct field access"
echo "4. Update type conversions (i32/u32, Option wrapping)"

echo ""
echo "📚 See COMPREHENSIVE_HANDOFF_REPORT.md for detailed guidance"
echo "✨ Happy coding!"
