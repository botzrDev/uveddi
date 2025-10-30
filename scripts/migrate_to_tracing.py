#!/usr/bin/env python3
"""
Script to automatically migrate println!/eprintln! statements to tracing macros.
This ensures consistent, structured logging throughout the Uveddi codebase.
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple

# Files to exclude from migration (e.g., test files that need direct output)
EXCLUDE_PATTERNS = [
    "**/tests/**",
    "**/test_*.rs",
    "**/bin/*.rs",  # Binary files may need direct output
    "**/examples/**",
]

# Mapping of println!/eprintln! patterns to tracing macros
REPLACEMENTS = [
    # Error patterns with eprintln!
    (r'eprintln!\("([^"]+?):\s*\{\}"\s*,\s*(\w+)\)', r'error!("\1: {}", \2)'),
    (r'eprintln!\("([^"]+?)"\)', r'error!("\1")'),
    (r'eprintln!\(([^)]+)\)', r'error!(\1)'),
    
    # Info patterns with println!
    # Skip patterns that look like actual user output (reports, summaries)
    (r'println!\("(\[INFO\]|ℹ️|📊|🌐|✅|🔍|💡|🚀)([^"]*)"\)', r'info!("\1\2")'),
    (r'println!\("(\[WARN\]|⚠️|❌|⏰)([^"]*)"\)', r'warn!("\1\2")'),
    (r'println!\("(\[ERROR\]|❗)([^"]*)"\)', r'error!("\1\2")'),
    (r'println!\("(\[DEBUG\])([^"]*)"\)', r'debug!("\1\2")'),
    
    # Generic println! that are likely logging
    (r'println!\("(Starting|Initializing|Loading|Processing|Completed|Finished)([^"]*)"\)', r'info!("\1\2")'),
    (r'println!\("(Failed|Error|Could not|Unable to)([^"]*)"\)', r'error!("\1\2")'),
    (r'println!\("(Warning|Skipping)([^"]*)"\)', r'warn!("\1\2")'),
]

def should_exclude_file(file_path: Path) -> bool:
    """Check if a file should be excluded from migration."""
    file_str = str(file_path)
    
    # Exclude test files
    if "test" in file_str.lower() or file_str.endswith("_test.rs"):
        return True
    
    # Exclude bin files that might need console output
    if "/bin/" in file_str and file_str.endswith(".rs"):
        return True
    
    # Exclude example files
    if "/examples/" in file_str:
        return True
        
    return False

def needs_tracing_import(content: str) -> bool:
    """Check if file needs tracing import added."""
    # Check if already has tracing import
    if re.search(r'use\s+tracing::', content):
        return False
    
    # Check if file has any println!/eprintln! that will be migrated
    if re.search(r'(println!|eprintln!)', content):
        return True
        
    return False

def add_tracing_import(content: str) -> str:
    """Add tracing import to file if needed."""
    if not needs_tracing_import(content):
        return content
    
    # Find the best place to add the import
    # After other use statements
    use_pattern = re.compile(r'^use\s+', re.MULTILINE)
    matches = list(use_pattern.finditer(content))
    
    if matches:
        # Add after the last use statement
        last_use = matches[-1]
        # Find the end of the line
        newline_pos = content.find('\n', last_use.start())
        if newline_pos != -1:
            return (content[:newline_pos + 1] + 
                   "use tracing::{info, warn, error, debug};\n" + 
                   content[newline_pos + 1:])
    
    # If no use statements, add at the beginning after any module docs
    lines = content.split('\n')
    insert_pos = 0
    
    # Skip module documentation
    for i, line in enumerate(lines):
        if not line.startswith('//') and line.strip():
            insert_pos = i
            break
    
    lines.insert(insert_pos, "use tracing::{info, warn, error, debug};")
    return '\n'.join(lines)

def migrate_file(file_path: Path) -> Tuple[bool, int]:
    """
    Migrate a single file from println!/eprintln! to tracing.
    Returns (modified, replacement_count).
    """
    try:
        content = file_path.read_text()
        original_content = content
        replacement_count = 0
        
        # Apply replacements
        for pattern, replacement in REPLACEMENTS:
            new_content, count = re.subn(pattern, replacement, content)
            if count > 0:
                content = new_content
                replacement_count += count
        
        # Add tracing import if needed and file was modified
        if replacement_count > 0:
            content = add_tracing_import(content)
        
        # Write back if modified
        if content != original_content:
            file_path.write_text(content)
            return True, replacement_count
            
        return False, 0
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return False, 0

def main():
    """Main migration function."""
    # Find project root (where Cargo.toml is)
    current_dir = Path.cwd()
    while current_dir != current_dir.parent:
        if (current_dir / "Cargo.toml").exists():
            break
        current_dir = current_dir.parent
    else:
        print("Error: Could not find Cargo.toml. Run from within the Uveddi project.", file=sys.stderr)
        sys.exit(1)
    
    src_dir = current_dir / "src"
    if not src_dir.exists():
        print(f"Error: Source directory {src_dir} not found.", file=sys.stderr)
        sys.exit(1)
    
    print(f"🔍 Scanning for Rust files in {src_dir}...")
    
    # Find all Rust files
    rust_files = list(src_dir.rglob("*.rs"))
    
    # Filter out excluded files
    files_to_process = [f for f in rust_files if not should_exclude_file(f)]
    
    print(f"📝 Found {len(files_to_process)} files to process")
    
    total_modified = 0
    total_replacements = 0
    
    for file_path in files_to_process:
        modified, count = migrate_file(file_path)
        if modified:
            total_modified += 1
            total_replacements += count
            relative_path = file_path.relative_to(current_dir)
            print(f"  ✅ Modified {relative_path} ({count} replacements)")
    
    print(f"\n📊 Migration Summary:")
    print(f"  • Files modified: {total_modified}")
    print(f"  • Total replacements: {total_replacements}")
    print(f"  • Files unchanged: {len(files_to_process) - total_modified}")
    
    if total_modified > 0:
        print("\n⚡ Next steps:")
        print("  1. Review the changes with: git diff")
        print("  2. Run cargo check to ensure compilation")
        print("  3. Run cargo test to verify functionality")
        print("  4. Commit the changes")

if __name__ == "__main__":
    main()