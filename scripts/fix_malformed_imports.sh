#!/bin/bash

echo "🔧 Fixing malformed tracing imports..."

# Find files with malformed imports (use statement in the middle of another use)
find src/ -name "*.rs" -type f | while read -r file; do
    # Create a temporary file
    tmpfile=$(mktemp)
    
    # Flag to track if we've added the tracing import
    added_tracing=false
    
    # Process the file line by line
    while IFS= read -r line; do
        # Check if this line has a malformed import (use inside use)
        if echo "$line" | grep -q "^use.*use tracing::"; then
            # Extract the original use statement part before the malformed insertion
            original_start=$(echo "$line" | sed 's/use tracing::.*//')
            # Add the tracing import first if not already added
            if [ "$added_tracing" = false ]; then
                echo "use tracing::{info, warn, error, debug};" >> "$tmpfile"
                added_tracing=true
            fi
            # Write the original use statement
            echo "$original_start" >> "$tmpfile"
        elif echo "$line" | grep -q "^use tracing::{info, warn, error, debug};$"; then
            # Check if this is a duplicate tracing import
            if [ "$added_tracing" = false ]; then
                echo "$line" >> "$tmpfile"
                added_tracing=true
            fi
            # Skip duplicates
        else
            echo "$line" >> "$tmpfile"
        fi
    done < "$file"
    
    # Replace the original file
    mv "$tmpfile" "$file"
done

# Remove duplicate consecutive tracing imports
find src/ -name "*.rs" -type f | while read -r file; do
    # Use awk to remove consecutive duplicate lines
    awk '!seen[$0]++ || $0 != prev; {prev=$0}' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
done

echo "✅ Fixed malformed imports"