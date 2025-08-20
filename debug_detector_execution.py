#!/usr/bin/env python3
"""
Debug script to trace detector execution in Uveddi.
This adds debug logging to help understand why detectors aren't finding issues.
"""

import os
import re
from pathlib import Path

def add_debug_logging_to_detector(detector_file_path, detector_name):
    """Add debug logging to a detector implementation."""
    print(f"Adding debug logging to {detector_name} detector: {detector_file_path}")
    
    if not os.path.exists(detector_file_path):
        print(f"❌ Detector file not found: {detector_file_path}")
        return False
    
    with open(detector_file_path, 'r') as f:
        content = f.read()
    
    # Add debug logging import if not present
    if 'use crate::core::logging::debug;' not in content and 'use crate::core::logging::{debug' not in content:
        # Find the imports section and add debug import
        import_pattern = r'(use crate::core::logging::[^;]+;)'
        if re.search(import_pattern, content):
            content = re.sub(import_pattern, r'\1\nuse crate::core::logging::debug;', content, count=1)
        else:
            # Add after other crate imports
            crate_import_pattern = r'(use crate::[^;]+;)'
            matches = list(re.finditer(crate_import_pattern, content))
            if matches:
                last_match = matches[-1]
                insert_pos = last_match.end()
                content = content[:insert_pos] + '\nuse crate::core::logging::debug;' + content[insert_pos:]
    
    # Add debug logging at the start of detect_issues method
    detect_issues_pattern = r'(async fn detect_issues\(\s*&self,\s*[^)]+\)\s*->[^{]+\{\s*)'
    if re.search(detect_issues_pattern, content):
        def add_debug_start(match):
            return match.group(1) + f'''
        debug!("🔍 {} detector starting analysis of file: {{}}", parsed_file.file_path.display());
        debug!("🔍 {} detector language: {{:?}}", parsed_file.language);
        debug!("🔍 {} detector AST available: {{}}", parsed_file.tree.is_some());
        '''.format(detector_name.title(), detector_name.title(), detector_name.title())
        
        content = re.sub(detect_issues_pattern, add_debug_start, content)
    
    # Add debug logging at the end of detect_issues method (before return)
    return_pattern = r'(\s+)(Ok\(([^)]+)\))\s*$'
    def add_debug_end(match):
        indent = match.group(1)
        return_statement = match.group(2)
        result_var = match.group(3)
        return f'''{indent}debug!("🔍 {detector_name.title()} detector found {{}} issues", {result_var}.len());
{indent}for (i, issue) in {result_var}.iter().enumerate() {{
{indent}    debug!("🔍 {detector_name.title()} issue {{}}: {{}} at line {{:?}}", i + 1, issue.description, issue.line_number);
{indent}}}
{indent}{return_statement}'''
    
    content = re.sub(return_pattern, add_debug_end, content, flags=re.MULTILINE)
    
    # Write the modified content back
    with open(detector_file_path, 'w') as f:
        f.write(content)
    
    print(f"✅ Added debug logging to {detector_name} detector")
    return True

def add_detector_factory_logging():
    """Add debug logging to detector factory to show which detectors are being created."""
    factory_path = "src/analysis/detector_factory.rs"
    
    if not os.path.exists(factory_path):
        print(f"❌ Detector factory not found: {factory_path}")
        return False
    
    with open(factory_path, 'r') as f:
        content = f.read()
    
    # Add debug import if not present
    if 'use crate::core::logging::debug;' not in content:
        # Add after first use statement
        use_pattern = r'(use [^;]+;)'
        content = re.sub(use_pattern, r'\1\nuse crate::core::logging::debug;', content, count=1)
    
    # Add logging to create_default_detectors
    create_default_pattern = r'(pub fn create_default_detectors\(\)[^{]+\{\s*)'
    if re.search(create_default_pattern, content):
        content = re.sub(create_default_pattern, 
                        r'\1        debug!("🏭 Creating default detector set");\n        ', content)
    
    # Add logging before each detector creation
    detector_creations = [
        (r'(Box::new\(GodObjectDetector::new\([^)]+\)\),)', "God Object"),
        (r'(Box::new\(CodeDuplicationDetector::new\(\)\),)', "Code Duplication"),
        (r'(Box::new\(DeadCodeDetector::with_default_config\(\)\),)', "Dead Code"),
        (r'(Box::new\(LargeClassDetector::with_default_config\(\)\),)', "Large Class"),
        (r'(Box::new\(TightCouplingDetector::default\(\)\),)', "Tight Coupling"),
        (r'(Box::new\(LongMethodsDetector::default\(\)\),)', "Long Methods"),
        (r'(Box::new\(MagicValuesDetector::default\(\)\),)', "Magic Values"),
    ]
    
    for pattern, name in detector_creations:
        replacement = f'        debug!("🏭 Creating {name} detector");\n            \\1'
        content = re.sub(pattern, replacement, content)
    
    with open(factory_path, 'w') as f:
        f.write(content)
    
    print("✅ Added debug logging to detector factory")
    return True

def add_registry_logging():
    """Add debug logging to detector registry."""
    registry_path = "src/analysis/detector_registry.rs"
    
    if not os.path.exists(registry_path):
        print(f"❌ Detector registry not found: {registry_path}")
        return False
    
    with open(registry_path, 'r') as f:
        content = f.read()
    
    # Add debug import if not present
    if 'use crate::core::logging::debug;' not in content:
        use_pattern = r'(use [^;]+;)'
        content = re.sub(use_pattern, r'\1\nuse crate::core::logging::debug;', content, count=1)
    
    # Add logging to load_defaults
    load_defaults_pattern = r'(pub fn load_defaults\(&mut self\)\s*\{\s*)'
    if re.search(load_defaults_pattern, content):
        content = re.sub(load_defaults_pattern, 
                        r'\1        debug!("📋 Loading default detector set");\n        ', content)
    
    # Add logging to register method
    register_pattern = r'(pub fn register\([^{]+\{\s*)'
    if re.search(register_pattern, content):
        content = re.sub(register_pattern, 
                        r'\1        debug!("📋 Registering detector: {}", name);\n        ', content)
    
    with open(registry_path, 'w') as f:
        f.write(content)
    
    print("✅ Added debug logging to detector registry")
    return True

def add_engine_logging():
    """Add debug logging to analysis engine."""
    engine_path = "src/analysis/engine.rs"
    
    if not os.path.exists(engine_path):
        print(f"❌ Analysis engine not found: {engine_path}")
        return False
    
    with open(engine_path, 'r') as f:
        content = f.read()
    
    # Add debug import if not present
    if 'use crate::core::logging::debug;' not in content:
        use_pattern = r'(use [^;]+;)'
        content = re.sub(use_pattern, r'\1\nuse crate::core::logging::debug;', content, count=1)
    
    # Add logging to detector execution
    detector_loop_pattern = r'(for detector in &self\.detectors\s*\{\s*)'
    if re.search(detector_loop_pattern, content):
        content = re.sub(detector_loop_pattern, 
                        r'\1            debug!("🔧 Running detector: {}", detector.get_detector_name());\n            ', content)
    
    with open(engine_path, 'w') as f:
        f.write(content)
    
    print("✅ Added debug logging to analysis engine")
    return True

def print_usage_instructions():
    """Print instructions for using the debug output."""
    print("\n🔧 DEBUG USAGE INSTRUCTIONS")
    print("=" * 50)
    print("""
After running this script:

1. REBUILD WITH DEBUG LOGGING:
   ```bash
   cargo build --features=tree-sitter,rust-lang,python-lang,javascript-lang
   ```

2. RUN ANALYSIS WITH DEBUG OUTPUT:
   ```bash
   RUST_LOG=debug ./target/debug/uveddi analyze ./test_files --output-format json
   ```

3. CHECK FOR DETECTOR EXECUTION:
   Look for these log patterns:
   - 🏭 Creating [Detector Name] detector
   - 📋 Loading default detector set  
   - 📋 Registering detector: [name]
   - 🔧 Running detector: [DetectorName]
   - 🔍 [Detector] detector starting analysis
   - 🔍 [Detector] detector found X issues

4. IDENTIFY MISSING DETECTORS:
   If you don't see logs for a detector, it's either:
   - Not being created in the factory
   - Not being registered in the registry
   - Not being included in the default set

5. CHECK FEATURE FLAGS:
   If detectors return early, check that tree-sitter features are enabled:
   ```bash
   cargo build --features=production  # Full feature set
   ```
""")

def main():
    """Main debug enhancement function."""
    print("🔧 ADDING DEBUG LOGGING TO UVEDDI DETECTORS")
    print("=" * 50)
    
    # Change to the project root directory
    if os.path.exists("src/analysis"):
        os.chdir(".")
    elif os.path.exists("../src/analysis"):
        os.chdir("..")
    elif os.path.exists("../../src/analysis"):
        os.chdir("../..")
    else:
        print("❌ Could not find Uveddi project root. Please run from the project directory.")
        return
    
    # Add debug logging to failing detectors
    detectors_to_debug = [
        ("src/analysis/detectors/anti_patterns/code_duplication.rs", "code_duplication"),
        ("src/analysis/detectors/anti_patterns/long_methods.rs", "long_methods"),
        ("src/analysis/detectors/anti_patterns/magic_values.rs", "magic_values"),
    ]
    
    success_count = 0
    for detector_path, detector_name in detectors_to_debug:
        if add_debug_logging_to_detector(detector_path, detector_name):
            success_count += 1
    
    # Add logging to infrastructure components
    if add_detector_factory_logging():
        success_count += 1
    if add_registry_logging():
        success_count += 1
    if add_engine_logging():
        success_count += 1
    
    print(f"\n✅ Successfully added debug logging to {success_count} components")
    print_usage_instructions()

if __name__ == "__main__":
    main()