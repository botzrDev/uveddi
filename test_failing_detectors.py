#!/usr/bin/env python3
"""
Minimal test cases for failing detectors in Uveddi.
This script creates simple test cases that should trigger each failing detector.
"""

import os
import tempfile
import subprocess
import json
from pathlib import Path

# Test cases for each failing detector
TEST_CASES = {
    "code_clones": {
        "description": "Code Clone Detector - should detect duplicate function implementations",
        "files": {
            "test_clones.rs": '''
fn calculate_total_price(items: &[i32], tax_rate: f64) -> f64 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    let subtotal = total as f64;
    let tax = subtotal * tax_rate;
    subtotal + tax
}

fn compute_final_amount(products: &[i32], tax_percentage: f64) -> f64 {
    let mut sum = 0;
    for product in products {
        sum += product;
    }
    let base_amount = sum as f64;
    let tax_amount = base_amount * tax_percentage;
    base_amount + tax_amount
}

fn main() {
    let items = vec![10, 20, 30];
    println!("Total: {}", calculate_total_price(&items, 0.08));
    println!("Amount: {}", compute_final_amount(&items, 0.08));
}
'''
        }
    },
    
    "long_methods": {
        "description": "Long Method Detector - should detect excessively long functions",
        "files": {
            "test_long_method.rs": '''
fn extremely_long_function(data: Vec<i32>) -> Vec<String> {
    let mut results = Vec::new();
    let mut processed_count = 0;
    
    // Process each item in multiple phases
    for item in data.iter() {
        let mut current_value = *item;
        
        // Phase 1: Basic validation
        if current_value < 0 {
            current_value = 0;
        }
        if current_value > 1000 {
            current_value = 1000;
        }
        
        // Phase 2: Complex calculations
        let mut intermediate_result = current_value * 2;
        if intermediate_result % 3 == 0 {
            intermediate_result += 5;
        } else if intermediate_result % 5 == 0 {
            intermediate_result += 3;
        } else {
            intermediate_result += 1;
        }
        
        // Phase 3: String formatting with multiple conditions
        let formatted_result = if intermediate_result < 10 {
            format!("SMALL_{}", intermediate_result)
        } else if intermediate_result < 100 {
            format!("MEDIUM_{}", intermediate_result)
        } else if intermediate_result < 1000 {
            format!("LARGE_{}", intermediate_result)
        } else {
            format!("XLARGE_{}", intermediate_result)
        };
        
        // Phase 4: Additional processing based on position
        let position_suffix = if processed_count % 10 == 0 {
            "_DECADE"
        } else if processed_count % 5 == 0 {
            "_FIVE"
        } else if processed_count % 2 == 0 {
            "_EVEN"
        } else {
            "_ODD"
        };
        
        // Phase 5: Final result construction
        let final_result = format!("{}{}", formatted_result, position_suffix);
        results.push(final_result);
        processed_count += 1;
        
        // Phase 6: Logging and validation
        if processed_count % 100 == 0 {
            println!("Processed {} items so far", processed_count);
        }
        
        // Phase 7: Memory management hints
        if results.len() > 10000 {
            results.reserve(results.len() * 2);
        }
    }
    
    // Final post-processing phase
    for result in results.iter_mut() {
        if result.contains("LARGE") {
            result.push_str("_FLAGGED");
        }
        if result.len() > 20 {
            *result = result[..20].to_string();
        }
    }
    
    println!("Processing complete. Total items: {}", processed_count);
    results
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    let results = extremely_long_function(data);
    println!("Results: {:?}", results);
}
'''
        }
    },
    
    "magic_values": {
        "description": "Magic Values Detector - should detect hardcoded literal values",
        "files": {
            "test_magic_values.rs": '''
fn calculate_score(base_score: f64, level: i32) -> f64 {
    let mut score = base_score;
    
    // Magic number: 42 (should be a named constant)
    if level > 42 {
        score *= 2.5;  // Magic number: 2.5
    }
    
    // Magic number: 100 (should be MAX_LEVEL or similar)
    if level >= 100 {
        score += 500.0;  // Magic number: 500.0
    }
    
    // Magic numbers in array indexing
    let multipliers = [1.0, 1.1, 1.2, 1.3, 1.4];
    if level < 5 {  // Magic number: 5
        score *= multipliers[level as usize];
    }
    
    // Magic string that should be a constant
    if score > 9999.99 {  // Magic number: 9999.99
        println!("Achievement unlocked: HIGH_SCORE");
    }
    
    score
}

fn validate_user_input(input: &str) -> bool {
    // Magic numbers for validation
    if input.len() < 3 || input.len() > 50 {  // Magic numbers: 3, 50
        return false;
    }
    
    // Magic number for character validation
    let valid_chars = input.chars().filter(|c| c.is_alphanumeric()).count();
    valid_chars >= 2  // Magic number: 2
}

fn main() {
    println!("Score: {}", calculate_score(100.0, 45));
    println!("Valid: {}", validate_user_input("test123"));
}
'''
        }
    }
}

def run_detective_test():
    """Run Uveddi analysis on test cases to check detector functionality."""
    print("🔍 Testing Failing Detectors in Uveddi")
    print("=" * 50)
    
    # Create temporary directory for test files
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        
        for detector_name, test_case in TEST_CASES.items():
            print(f"\n📋 Testing {detector_name.upper().replace('_', ' ')} DETECTOR")
            print(f"Description: {test_case['description']}")
            
            # Create test files
            test_files = []
            for filename, content in test_case['files'].items():
                file_path = temp_path / filename
                file_path.write_text(content)
                test_files.append(str(file_path))
                print(f"Created test file: {filename}")
            
            # Run Uveddi analysis
            print("Running Uveddi analysis...")
            try:
                # Try to find uveddi binary
                uveddi_paths = [
                    "../../target/release/uveddi",
                    "../../target/debug/uveddi", 
                    "target/release/uveddi",
                    "target/debug/uveddi",
                    "uveddi"
                ]
                
                uveddi_binary = None
                for path in uveddi_paths:
                    if os.path.exists(path):
                        uveddi_binary = path
                        break
                
                if not uveddi_binary:
                    print("❌ Uveddi binary not found. Please build the project first.")
                    continue
                
                # Run analysis with JSON output
                cmd = [
                    uveddi_binary, "analyze", 
                    str(temp_path),
                    "--output-format", "json",
                    "--output", str(temp_path / "analysis.json")
                ]
                
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
                
                if result.returncode == 0:
                    print("✅ Analysis completed successfully")
                    
                    # Check for analysis output
                    analysis_file = temp_path / "analysis.json"
                    if analysis_file.exists():
                        try:
                            with open(analysis_file) as f:
                                analysis_data = json.load(f)
                            
                            # Look for detections related to this detector
                            detections_found = False
                            if "issues" in analysis_data:
                                for issue in analysis_data["issues"]:
                                    detector_name_in_issue = issue.get("detector_name", "").lower()
                                    issue_desc = issue.get("description", "").lower()
                                    
                                    if (detector_name.replace("_", "") in detector_name_in_issue or
                                        detector_name.replace("_", " ") in issue_desc):
                                        print(f"✅ DETECTION FOUND: {issue.get('description', 'No description')}")
                                        detections_found = True
                            
                            if not detections_found:
                                print(f"❌ NO DETECTIONS: {detector_name} detector did not find expected patterns")
                                print("This indicates the detector is not working correctly.")
                        except json.JSONDecodeError:
                            print("❌ Failed to parse analysis JSON output")
                    else:
                        print("❌ No analysis output file generated")
                else:
                    print(f"❌ Analysis failed with return code {result.returncode}")
                    print(f"Error output: {result.stderr}")
                
            except subprocess.TimeoutExpired:
                print("❌ Analysis timed out")
            except Exception as e:
                print(f"❌ Error running analysis: {e}")

def print_debug_instructions():
    """Print instructions for debugging detector issues."""
    print("\n🔧 DEBUGGING INSTRUCTIONS")
    print("=" * 50)
    
    print("""
To debug detector issues:

1. ADD DEBUG LOGGING:
   Add these lines to detector implementations:
   ```rust
   use crate::core::logging::debug;
   debug!("Detector {} starting analysis of file: {}", self.get_detector_name(), parsed_file.file_path.display());
   ```

2. CHECK FEATURE FLAGS:
   Ensure detectors are compiled with correct features:
   ```bash
   cargo build --features=tree-sitter,rust-lang,python-lang,javascript-lang
   ```

3. VERIFY DETECTOR REGISTRATION:
   Check that detectors are registered in detector_factory.rs or detector_registry.rs

4. TEST INDIVIDUAL DETECTORS:
   Run unit tests for specific detectors:
   ```bash
   cargo test test_long_method_detection
   cargo test test_code_clone_detection
   ```

5. CHECK AST PARSING:
   Verify that tree-sitter is working correctly:
   ```bash
   cargo test --features=tree-sitter
   ```
""")

def print_findings_summary():
    """Print summary of findings from detector analysis."""
    print("\n📊 DETECTOR ANALYSIS SUMMARY")
    print("=" * 50)
    
    findings = {
        "Code Clone Detector": {
            "status": "❌ NOT WORKING",
            "issues": [
                "Implementation is complex but may have configuration issues",
                "Default thresholds might be too strict (min_tokens: 50, min_lines: 5)",
                "May require specific feature flags for tree-sitter parsing",
                "CFG and semantic analysis disabled by default for performance"
            ],
            "minimal_test": "Two nearly identical functions with different variable names"
        },
        
        "Long Method Detector": {
            "status": "❌ NOT WORKING", 
            "issues": [
                "Feature flag dependency on 'tree-sitter' may not be enabled",
                "Returns empty Vec when tree-sitter feature disabled",
                "Thresholds may be too high for small test cases",
                "Query parsing might fail silently"
            ],
            "minimal_test": "Function with 50+ lines, high complexity, deep nesting"
        },
        
        "Magic Values Detector": {
            "status": "❌ NOT IMPLEMENTED",
            "issues": [
                "Only a scaffold implementation exists",
                "detect_issues() always returns empty Vec",
                "get_anti_pattern_types() returns empty Vec", 
                "No actual detection logic implemented"
            ],
            "minimal_test": "Function with hardcoded numeric literals and strings"
        }
    }
    
    for detector, info in findings.items():
        print(f"\n{detector}:")
        print(f"  Status: {info['status']}")
        print(f"  Test Case: {info['minimal_test']}")
        print("  Issues:")
        for issue in info['issues']:
            print(f"    • {issue}")

if __name__ == "__main__":
    run_detective_test()
    print_debug_instructions()
    print_findings_summary()