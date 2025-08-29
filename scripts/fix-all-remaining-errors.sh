#!/bin/bash

# Fix all remaining compilation errors after removing overly broad warning suppressions

set -e

echo "🔧 Fixing all remaining compilation errors..."

# Fix missing imports across multiple files
echo "📝 Adding missing imports..."

# 1. Fix leaky_abstraction.rs - add SourceLanguage import
if grep -q "SourceLanguage::Rust" src/analysis/detectors/anti_patterns/leaky_abstraction.rs; then
    echo "  • Adding SourceLanguage import to leaky_abstraction.rs"
    # Add the import after the existing use statements
    sed -i '/use crate::analysis::detectors::anti_patterns::AntiPatternType;/a use crate::ast::SourceLanguage;' \
        src/analysis/detectors/anti_patterns/leaky_abstraction.rs
fi

# 2. Fix extractors.rs - add missing imports
if grep -q "SymbolKind::Function" src/analysis/extractors.rs; then
    echo "  • Adding SymbolKind imports to extractors.rs"
    sed -i '1i use crate::analysis::symbols::{SymbolKind, CanonicalSymbol, SourceLocation};' src/analysis/extractors.rs
    sed -i '2i use std::sync::atomic::Ordering;' src/analysis/extractors.rs
fi

# 3. Fix ast_provider.rs - make language_map mutable
echo "  • Fixing mutability in ast_provider.rs"
sed -i 's/let language_map = HashMap::new();/let mut language_map = HashMap::new();/g' src/analysis/components/ast_provider.rs

# 4. Fix detector_factory.rs - make detectors mutable  
echo "  • Fixing mutability in detector_factory.rs"
sed -i 's/let detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>> = vec!\[/let mut detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>> = vec![/g' src/analysis/detector_factory.rs

# 5. Fix engine.rs - make anti_pattern_types mutable
echo "  • Fixing mutability in engine.rs" 
sed -i 's/let anti_pattern_types = vec!\[/let mut anti_pattern_types = vec![/g' src/analysis/engine.rs

# 6. Fix tree_sitter_impl.rs - make parsers mutable
echo "  • Fixing mutability in tree_sitter_impl.rs"
sed -i 's/let parsers = HashMap::new();/let mut parsers = HashMap::new();/g' src/ast/tree_sitter_impl.rs

echo "✅ All fixes applied!"

# Test compilation
echo "🧪 Testing compilation..."
if cargo build --features=production >/dev/null 2>&1; then
    echo "✅ SUCCESS: Production build now compiles!"
else
    echo "⚠️ Still some errors remaining, showing details:"
    cargo build --features=production 2>&1 | head -20
fi