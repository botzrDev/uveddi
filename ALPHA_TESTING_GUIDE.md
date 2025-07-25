# Uveddi Alpha Testing Guide

This guide provides step-by-step instructions for installing and testing the Uveddi alpha release on your local system.

## System Requirements

### Operating System
- **Linux**: Ubuntu 20.04+, RHEL 8+, or equivalent
- **macOS**: 10.15+ (Catalina or later)
- **Windows**: Windows 10/11 with WSL2 recommended

### Prerequisites
- **Rust toolchain**: 1.70.0 or later
- **Git**: For source code management
- **C/C++ compiler**: Required for native dependencies
- **Memory**: At least 4GB RAM (8GB+ recommended for large projects)
- **Storage**: 2GB free space for build artifacts

## Installation Instructions

### Step 1: Install Rust (if not already installed)

#### Linux/macOS:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Windows:
Download and run [rustup-init.exe](https://rustup.rs/)

### Step 2: Install System Dependencies

#### Ubuntu/Debian:
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git
```

#### RHEL/CentOS/Fedora:
```bash
sudo dnf install -y gcc g++ openssl-devel git pkg-config
# OR for older systems:
# sudo yum install -y gcc gcc-c++ openssl-devel git pkgconfig
```

#### macOS:
```bash
# Install Xcode command line tools
xcode-select --install

# Or install via Homebrew
brew install git pkg-config openssl
```

#### Windows (WSL2):
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git
```

### Step 3: Clone and Build Uveddi

```bash
# Clone the repository
git clone https://github.com/botzrDev/uveddi.git
cd uveddi

# Switch to the alpha branch (if not already on version-0.9-pre-release)
git checkout version-0.9-pre-release

# Build the alpha release
cargo build --release --features="alpha"
```

**Build Time**: Expect 5-15 minutes depending on your system performance.

### Step 4: Verify Installation

```bash
# Check if the binary was built successfully
ls -la target/release/uveddi

# Test basic functionality
./target/release/uveddi --help
./target/release/uveddi analyze --help
```

Expected output should show the help information with all command-line options.

## Alpha Testing Scenarios

### Test 1: Basic CLI Functionality

```bash
# Create a simple test project
mkdir -p ~/uveddi_test/simple_rust
cat > ~/uveddi_test/simple_rust/main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
}

#[allow(dead_code)]
fn unused_function() {
    println!("This function is never used");
}
EOF

# Test basic analysis
./target/release/uveddi analyze ~/uveddi_test/simple_rust --output-format=json
```

**Expected Result**: Should output JSON with analysis results or show a structured error message.

### Test 2: Help System Validation

```bash
# Test all help commands
./target/release/uveddi --help
./target/release/uveddi analyze --help
./target/release/uveddi config --help
```

**Expected Result**: All commands should display comprehensive help information without errors.

### Test 3: Complex Project Analysis

```bash
# Create a more complex test project
mkdir -p ~/uveddi_test/complex_rust
cat > ~/uveddi_test/complex_rust/main.rs << 'EOF'
use std::collections::HashMap;

pub struct LargeClass {
    field1: String,
    field2: i32,
    field3: f64,
    field4: bool,
    field5: Vec<String>,
    field6: HashMap<String, i32>,
    field7: Option<String>,
    field8: Result<i32, String>,
    field9: u64,
    field10: char,
}

impl LargeClass {
    pub fn new() -> Self {
        Self {
            field1: String::new(),
            field2: 0,
            field3: 0.0,
            field4: false,
            field5: Vec::new(),
            field6: HashMap::new(),
            field7: None,
            field8: Ok(0),
            field9: 0,
            field10: 'a',
        }
    }
    
    pub fn method1(&self) -> String { self.field1.clone() }
    pub fn method2(&self) -> i32 { self.field2 }
    pub fn method3(&self) -> f64 { self.field3 }
    pub fn method4(&self) -> bool { self.field4 }
    pub fn method5(&self) -> Vec<String> { self.field5.clone() }
    
    #[allow(dead_code)]
    fn unused_method(&self) {
        println!("This method is never called");
    }
}

#[allow(dead_code)]
fn unused_function() {
    println!("This function is never used");
}

pub fn main() {
    let instance = LargeClass::new();
    println!("{}", instance.method1());
}
EOF

# Test with various options
./target/release/uveddi analyze ~/uveddi_test/complex_rust \
  --output-format=json \
  --dead-code-confidence=0.8 \
  --large-classes-max-methods=3 \
  --large-classes-max-fields=5
```

**Expected Result**: Should detect large class and dead code issues.

### Test 4: Output Format Testing

```bash
# Test markdown output
./target/release/uveddi analyze ~/uveddi_test/complex_rust --output-format=markdown

# Test file output
./target/release/uveddi analyze ~/uveddi_test/complex_rust \
  --output-format=json \
  --output=~/uveddi_test/analysis_report.json

# Verify output file was created
cat ~/uveddi_test/analysis_report.json
```

### Test 5: Error Handling

```bash
# Test with non-existent path
./target/release/uveddi analyze /path/that/does/not/exist

# Test with invalid format
./target/release/uveddi analyze ~/uveddi_test/simple_rust --output-format=invalid

# Test with invalid confidence value
./target/release/uveddi analyze ~/uveddi_test/simple_rust --dead-code-confidence=1.5
```

**Expected Result**: Should show clear, helpful error messages without crashing.

## Running Automated Tests

### Test Suite Execution

```bash
# Run the comprehensive test suite
./scripts/run-automated-tests.sh

# Run specific test categories
./scripts/run-automated-tests.sh cli
./scripts/run-automated-tests.sh unit
./scripts/run-automated-tests.sh functional
```

### Individual Test Execution

```bash
# CLI integration tests
cargo test --features="alpha" --test cli_integration

# TUI component tests (if compilation succeeds)
cargo test --features="alpha" --test tui_component_automation

# Basic unit tests
cargo test --features="alpha" --lib
```

## Known Issues & Limitations (Alpha)

### Current Limitations
1. **Analysis Engine**: Core analysis logic is in development; may show "Analysis execution failed" errors
2. **TUI Mode**: Interactive TUI is implemented but may not be fully functional
3. **Language Support**: Primary focus is Rust; Python/JavaScript support is basic
4. **Performance**: Not yet optimized for very large codebases (>10k files)

### Expected Behaviors
- **CLI Interface**: ✅ Fully functional with comprehensive help and argument parsing
- **Build System**: ✅ Compiles successfully with alpha features
- **Error Handling**: ✅ Graceful error messages and help information
- **Output Formats**: ✅ JSON and Markdown format support implemented
- **Configuration**: ⚠️ Basic validation; advanced analysis may show errors

### Troubleshooting

#### Build Issues
```bash
# Clean build if you encounter issues
cargo clean
cargo build --release --features="alpha"

# Update Rust if build fails
rustup update stable
```

#### Permission Issues (Linux/macOS)
```bash
# Make sure the binary is executable
chmod +x target/release/uveddi
```

#### Memory Issues
```bash
# If build runs out of memory, try:
export CARGO_BUILD_JOBS=1
cargo build --release --features="alpha"
```

## Performance Testing

### Small Project (< 100 files)
```bash
time ./target/release/uveddi analyze ~/uveddi_test/complex_rust --output-format=json
```
**Expected**: < 5 seconds

### Medium Project (100-1000 files)
Create a larger test project and measure performance:
```bash
# Create multiple files
for i in {1..50}; do
    cp ~/uveddi_test/complex_rust/main.rs ~/uveddi_test/complex_rust/file$i.rs
done

time ./target/release/uveddi analyze ~/uveddi_test/complex_rust --output-format=json
```
**Expected**: < 30 seconds

## Feedback and Bug Reporting

### What to Test
1. **CLI Argument Parsing**: Try various command combinations
2. **Error Handling**: Test invalid inputs and edge cases  
3. **Output Formats**: Verify JSON and Markdown outputs are well-formed
4. **Performance**: Test with projects of different sizes
5. **Cross-Platform**: Test on different operating systems

### Reporting Issues
When reporting issues, please include:
- Operating system and version
- Rust version (`rustc --version`)
- Exact command that failed
- Full error output
- Project structure being analyzed (if relevant)

### Success Criteria for Alpha
✅ **CLI builds and executes without crashes**  
✅ **Help system provides comprehensive information**  
✅ **Basic argument validation works correctly**  
✅ **Error messages are clear and helpful**  
✅ **Output formats are properly structured**  
⚠️ **Analysis results may be limited (development in progress)**

## Next Steps

After alpha testing, the roadmap includes:
1. **Analysis Engine Completion**: Full pattern detection implementation
2. **TUI Finalization**: Complete interactive terminal interface
3. **Performance Optimization**: Large codebase support
4. **Beta Release**: Public testing with community feedback
5. **v1.0 Release**: Production-ready with full feature set

## Contact

For alpha testing support:
- **GitHub Issues**: [https://github.com/botzrDev/uveddi/issues](https://github.com/botzrDev/uveddi/issues)
- **Discussions**: [https://github.com/botzrDev/uveddi/discussions](https://github.com/botzrDev/uveddi/discussions)

---

**Note**: This is alpha software under active development. While the CLI infrastructure is robust and ready for demonstration, the analysis engine is still being refined. The comprehensive automated testing framework ensures reliability of the core infrastructure.