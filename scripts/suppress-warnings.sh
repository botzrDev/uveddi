#!/bin/bash

# Quick and effective warning suppression for Uveddi
# This adds crate-level warning suppressions to eliminate build noise

set -e

echo "🔇 Suppressing Rust compiler warnings..."

# Add warning suppressions to lib.rs
if [[ -f "src/lib.rs" ]]; then
    echo "📝 Adding suppressions to src/lib.rs"
    if ! grep -q "#!\[allow(warnings)\]" src/lib.rs; then
        # Add comprehensive warning suppression at the top
        cat > src/lib.rs.tmp << 'EOF'
#![allow(warnings)]
#![allow(unused_variables, unused_imports, dead_code, unused_mut)]
#![allow(clippy::all)]

EOF
        cat src/lib.rs >> src/lib.rs.tmp
        mv src/lib.rs.tmp src/lib.rs
        echo "✅ Added warning suppressions to lib.rs"
    else
        echo "ℹ️ Warning suppressions already present in lib.rs"
    fi
fi

# Add warning suppressions to main.rs  
if [[ -f "src/main.rs" ]]; then
    echo "📝 Adding suppressions to src/main.rs"
    if ! grep -q "#!\[allow(warnings)\]" src/main.rs; then
        # Add comprehensive warning suppression at the top
        cat > src/main.rs.tmp << 'EOF'
#![allow(warnings)]
#![allow(unused_variables, unused_imports, dead_code, unused_mut)]
#![allow(clippy::all)]

EOF
        cat src/main.rs >> src/main.rs.tmp
        mv src/main.rs.tmp src/main.rs
        echo "✅ Added warning suppressions to main.rs"
    else
        echo "ℹ️ Warning suppressions already present in main.rs"
    fi
fi

# Test the result
echo "🧪 Testing build with suppressions..."
echo "Building with dev-core features..."

# Count warnings before and after
warning_count=$(cargo build --features=dev-core 2>&1 | grep -c "warning:" || true)

if [[ $warning_count -eq 0 ]]; then
    echo "✅ SUCCESS: All warnings suppressed!"
elif [[ $warning_count -lt 10 ]]; then
    echo "✅ GOOD: Most warnings suppressed (only $warning_count remaining)"
else
    echo "ℹ️ INFO: $warning_count warnings remaining (significantly reduced from 4262)"
fi

echo ""
echo "🎯 Quick suppression complete!"
echo "💡 To restore warnings later, remove the #![allow(...)] lines from src/lib.rs and src/main.rs"
echo ""
echo "🚀 Your build should now be much cleaner!"