#!/bin/bash

# Files that need the StreamingIterator import
files=(
    "src/analysis/detectors/anti_patterns/dead_code/language_support/rust.rs"
    "src/analysis/detectors/anti_patterns/dead_code/language_support/python.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/typescript/classes.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/typescript/namespaces.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/typescript/interfaces.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/typescript/patterns.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/python.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/rust/implementations.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/rust/structs.rs"
    "src/analysis/detectors/anti_patterns/god_object/languages/rust/patterns.rs"
    "src/analysis/detectors/anti_patterns/god_object/metrics.rs"
    "src/analysis/detectors/anti_patterns/tight_coupling/analysis/import_detector.rs"
    "src/analysis/detectors/anti_patterns/tight_coupling/analysis/wildcard_analyzer.rs"
    "src/analysis/detectors/anti_patterns/long_methods/extractors/method_extractor.rs"
    "src/analysis/detectors/anti_patterns/large_classes.rs"
    "src/analysis/detectors/dependency/detector.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        # Look for an existing tree_sitter import and add the StreamingIterator import after it
        if grep -q "use crate::ast::tree_sitter::" "$file"; then
            # Add the import after the tree_sitter import
            sed -i '/use crate::ast::tree_sitter::/a use streaming_iterator::StreamingIterator;' "$file"
        elif grep -q "use.*tree_sitter" "$file"; then
            # Add after any other tree_sitter import
            sed -i '/use.*tree_sitter/a use streaming_iterator::StreamingIterator;' "$file"
        else
            # Add after the first use statement block
            sed -i '/^use /a use streaming_iterator::StreamingIterator;' "$file"
        fi
        echo "Added StreamingIterator import to $file"
    else
        echo "File not found: $file"
    fi
done