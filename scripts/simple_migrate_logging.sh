#!/bin/bash

# Simple script to migrate println!/eprintln! to tracing macros
# This handles the most common cases

echo "🔍 Migrating println!/eprintln! to tracing macros..."

# Count initial occurrences
initial_println=$(grep -r "println!" src/ --include="*.rs" | grep -v "tests/" | grep -v "bin/" | wc -l)
initial_eprintln=$(grep -r "eprintln!" src/ --include="*.rs" | grep -v "tests/" | grep -v "bin/" | wc -l)

echo "📊 Found $initial_println println! and $initial_eprintln eprintln! statements to migrate"

# Find all Rust files (excluding tests and binaries)
find src/ -name "*.rs" -type f | grep -v "tests/" | grep -v "bin/" | while read -r file; do
    # Check if file needs tracing import
    if grep -q "println!\|eprintln!" "$file" && ! grep -q "use tracing::" "$file"; then
        # Add tracing import after the first use statement or at the beginning
        if grep -q "^use " "$file"; then
            # Add after the last use statement
            sed -i '/^use /{ :a; n; /^use /ba; i\use tracing::{info, warn, error, debug};
            }' "$file"
        else
            # Add at the beginning after module docs
            sed -i '1,/^[^/]/{/^[^/]/i\use tracing::{info, warn, error, debug};\n
            }' "$file"
        fi
    fi
    
    # Simple replacements for common patterns
    # Replace eprintln! with error!
    sed -i 's/eprintln!/error!/g' "$file"
    
    # Replace println! for error patterns
    sed -i 's/println!("\(.*[Ee]rror\)/error!("\1/g' "$file"
    sed -i 's/println!("\(.*[Ff]ailed\)/error!("\1/g' "$file"
    sed -i 's/println!("\(.*[Cc]ould not\)/error!("\1/g' "$file"
    
    # Replace println! for warning patterns  
    sed -i 's/println!("\(.*[Ww]arning\)/warn!("\1/g' "$file"
    sed -i 's/println!("\(.*[Ss]kipping\)/warn!("\1/g' "$file"
    
    # Replace println! for info patterns (with emojis)
    sed -i 's/println!("\(.*📊\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*🌐\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*✅\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*💡\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*🚀\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*🔍\)/info!("\1/g' "$file"
    
    # Replace println! for starting/processing patterns
    sed -i 's/println!("\(.*[Ss]tarting\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*[Ii]nitializing\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*[Ll]oading\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*[Pp]rocessing\)/info!("\1/g' "$file"
    sed -i 's/println!("\(.*[Cc]ompleted\)/info!("\1/g' "$file"
done

# Count remaining occurrences
final_println=$(grep -r "println!" src/ --include="*.rs" | grep -v "tests/" | grep -v "bin/" | wc -l)
final_eprintln=$(grep -r "eprintln!" src/ --include="*.rs" | grep -v "tests/" | grep -v "bin/" | wc -l)

echo "✅ Migration complete!"
echo "📊 Remaining: $final_println println! and $final_eprintln eprintln! statements"
echo ""
echo "⚡ Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Check compilation: cargo check"
echo "  3. Run tests: cargo test"
echo "  4. Manually review remaining println! statements (may be intentional user output)"