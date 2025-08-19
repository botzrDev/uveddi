#!/usr/bin/env bash

# Simple test of Uveddi on a small Rust sample

set -e

# Create a temporary directory
TEST_DIR="/tmp/uveddi-simple-test"
mkdir -p "$TEST_DIR"

# Create a sample Rust file with known issues
cat > "$TEST_DIR/main.rs" << 'EOF'
// Sample Rust file with anti-patterns for Uveddi testing

// Large class anti-pattern - too many fields and methods
struct LargeStruct {
    field1: i32,
    field2: String,
    field3: Vec<i32>,
    field4: Option<String>,
    field5: bool,
    field6: f64,
    field7: Vec<String>,
    field8: u32,
    field9: u64,
    field10: String,
    field11: i32,
    field12: String,
}

impl LargeStruct {
    fn method1(&self) -> i32 { self.field1 }
    fn method2(&self) -> String { self.field2.clone() }
    fn method3(&self) -> Vec<i32> { self.field3.clone() }
    fn method4(&self) -> Option<String> { self.field4.clone() }
    fn method5(&self) -> bool { self.field5 }
    fn method6(&self) -> f64 { self.field6 }
    fn method7(&self) -> Vec<String> { self.field7.clone() }
    fn method8(&self) -> u32 { self.field8 }
    fn method9(&self) -> u64 { self.field9 }
    fn method10(&self) -> String { self.field10.clone() }
    fn method11(&self) -> i32 { self.field11 }
    fn method12(&self) -> String { self.field12.clone() }
}

// Dead code anti-pattern
fn unused_function() {
    println!("This function is never called");
}

// Magic number anti-pattern
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height * 0.3048 // Magic number - hardcoded conversion factor
}

// Main function that uses only part of the code
fn main() {
    let large = LargeStruct {
        field1: 1,
        field2: "test".to_string(),
        field3: vec![1, 2, 3],
        field4: Some("test".to_string()),
        field5: true,
        field6: 1.0,
        field7: vec!["test".to_string()],
        field8: 1,
        field9: 1,
        field10: "test".to_string(),
        field11: 1,
        field12: "test".to_string(),
    };
    
    println!("Value: {}", large.method1());
}
EOF

echo "📊 Created test file at $TEST_DIR/main.rs"

# Run Uveddi analysis
cd "$(dirname "$(dirname "$0")")"

echo "🔍 Building Uveddi..."
cargo build

echo "🔍 Running Uveddi analysis..."
RUST_BACKTRACE=1 ./target/debug/uveddi analyze "$TEST_DIR" --output-format markdown --output "$TEST_DIR/report.md" --open-dashboard

echo "✅ Analysis complete!"
echo "📝 Report saved to $TEST_DIR/report.md"
echo "📊 Dashboard should be open in your browser"

# Display the markdown report
echo "📄 Analysis Report Contents:"
echo "------------------------"
cat "$TEST_DIR/report.md"
echo "------------------------"
