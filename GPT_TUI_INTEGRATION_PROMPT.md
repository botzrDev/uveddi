# GPT Development Prompt: TUI Integration and Isolated Testing Environment

## Context and Current State

You are working on **Uveddi**, a Rust-based code analysis tool that combines static analysis with AI-powered insights. The project currently has:

- **Main CLI Binary** (`uveddi`): Handles `analyze`, `config`, `ui`, `ci`, `serve` commands
- **Separate TUI Binary** (`tui_test`): Terminal User Interface for interactive analysis
- **Feature-Based Build System**: Uses extensive feature flags for modular compilation
- **Production vs Development Builds**: Optimized feature sets for different use cases

## Current TUI Integration Issues

### 1. **Architecture Problem**
- TUI exists as separate `tui_test` binary instead of being integrated into main `uveddi` binary
- Users expect `uveddi tui` or `uveddi ui tui` command, not a separate binary
- Current CLI structure doesn't expose TUI functionality through main interface

### 2. **CLI Structure Mismatch**
```rust
// Current: main.rs has limited commands, application/mod.rs has full CLI
// Problem: TUI not accessible through standard CLI interface

// Expected user experience:
uveddi tui                    // Launch TUI interface
uveddi ui tui                 // Alternative access path
uveddi analyze --tui          // Analyze with TUI display
```

### 3. **Feature Flag Issues**
- TUI requires `--features=tui` but may not be properly integrated into production feature set
- Build system complexity makes it unclear when TUI is available
- Production builds should include TUI by default

## Task 1: Integrate TUI into Main Binary

### Objective
Integrate the TUI functionality directly into the main `uveddi` binary so users can access it through standard CLI commands.

### Implementation Steps

#### Step 1: CLI Structure Unification
1. **Analyze Current CLI Architecture**:
   - `src/main.rs`: Basic CLI with limited commands
   - `src/application/mod.rs`: Full CLI with all commands
   - `src/cli/ui_command.rs`: Web UI commands (not TUI)
   - Need to create `src/cli/tui_command.rs` for TUI integration

2. **Create TUI Command Module**:
```rust
// src/cli/tui_command.rs
use clap::Args;

#[derive(Debug, Args)]
pub struct TuiCommand {
    /// Project path to analyze in TUI
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,
    
    /// Skip initial analysis and go straight to TUI
    #[arg(long)]
    pub skip_analysis: bool,
    
    /// Load existing analysis results
    #[arg(long)]
    pub load_results: Option<PathBuf>,
    
    /// Enable debug mode for TUI development
    #[arg(long)]
    pub debug: bool,
}

impl TuiCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        // Implementation to launch TUI
    }
}
```

3. **Update CLI Command Enum**:
```rust
// In application/mod.rs Commands enum
enum Commands {
    Analyze(Box<AnalyzeCommand>),
    Config(ConfigCommand),
    Ui(UiCommand),
    Ci(CiCommand),
    Tui(TuiCommand),  // <- Add this
    Serve { /* ... */ },
}
```

#### Step 2: TUI Module Integration
1. **Refactor TUI Module Structure**:
   - Move `src/bin/tui_test.rs` logic into `src/tui/mod.rs`
   - Create `src/tui/cli_integration.rs` for CLI bridge
   - Ensure TUI can be launched from main binary

2. **Handle Terminal Detection**:
```rust
// Improve terminal detection and provide better error messages
pub fn launch_tui(args: &TuiCommand) -> Result<(), TuiError> {
    if !atty::is(atty::Stream::Stdout) {
        return Err(TuiError::NonInteractive {
            suggestion: "Run in a terminal or use: uveddi analyze --output json".to_string()
        });
    }
    // ... rest of TUI launch logic
}
```

#### Step 3: Feature Flag Management
1. **Ensure TUI in Production Features**:
```toml
# Cargo.toml - Verify these settings:
production = ["tree-sitter", "security", "memory-optimization", "web-full", "tui"]
```

2. **Conditional Compilation**:
```rust
// Handle TUI availability gracefully
#[cfg(feature = "tui")]
pub fn execute_tui_command(cmd: &TuiCommand) -> Result<(), UveddiError> {
    // TUI implementation
}

#[cfg(not(feature = "tui"))]
pub fn execute_tui_command(_cmd: &TuiCommand) -> Result<(), UveddiError> {
    Err(UveddiError::feature_not_enabled("TUI", "cargo build --features=tui"))
}
```

## Task 2: Create Isolated Testing Environment

### Objective
Set up a complete virtual testing environment where you can:
1. Install release builds of Uveddi as if you were an end user
2. Test all functionality in isolation from the development environment  
3. Validate the complete user experience including installation and usage

### Implementation Requirements

#### Environment 1: Docker-Based Testing
```dockerfile
# Create: docker/test-environment/Dockerfile
FROM rust:1.70-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git curl build-essential pkg-config libssl-dev \
    tmux vim less tree htop \
    && rm -rf /var/lib/apt/lists/*

# Create test user (non-root)
RUN useradd -m -s /bin/bash testuser
USER testuser
WORKDIR /home/testuser

# Install Rust for test user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/testuser/.cargo/bin:${PATH}"

# Set up test workspace
RUN mkdir -p /home/testuser/test-workspace/projects
COPY --chown=testuser:testuser scripts/test-setup.sh /home/testuser/
RUN chmod +x /home/testuser/test-setup.sh

CMD ["/bin/bash"]
```

```bash
# scripts/test-setup.sh
#!/bin/bash
set -e

echo "🚀 Setting up Uveddi testing environment..."

# Clone sample codebases for testing
cd /home/testuser/test-workspace/projects

# Create various test projects
mkdir -p rust-sample python-sample js-sample

# Rust sample with various issues
cat > rust-sample/src/main.rs << 'EOF'
// Sample Rust code with anti-patterns for testing
pub struct GodObject {
    field1: String,
    field2: i32,
    field3: Vec<String>,
    field4: HashMap<String, i32>,
    // ... more fields to trigger large class detection
}

impl GodObject {
    pub fn do_everything(&mut self) {
        // Method that does too many things
        self.process_data();
        self.handle_network();
        self.manage_files();
        self.calculate_metrics();
    }
    
    // Dead code example
    fn unused_function(&self) {
        println!("This is never called");
    }
}

fn main() {
    println!("Hello, world!");
}
EOF

# Python sample
cat > python-sample/main.py << 'EOF'
# Python sample with anti-patterns
class GodClass:
    def __init__(self):
        self.data = []
        self.config = {}
        self.network = None
        self.files = []
        
    def do_everything(self):
        self.process_data()
        self.handle_network() 
        self.manage_files()
        
    def unused_method(self):
        """Dead code example"""
        pass

if __name__ == "__main__":
    print("Hello Python!")
EOF

echo "✅ Test projects created"
echo "📁 Projects available in: /home/testuser/test-workspace/projects/"
echo ""
echo "🔧 To test Uveddi:"
echo "  1. Install: cargo install --path /path/to/uveddi --features=production"
echo "  2. Test: uveddi analyze rust-sample/"
echo "  3. TUI: uveddi tui rust-sample/"
echo "  4. Web: uveddi serve --port 8080"
```

#### Environment 2: VM-Based Testing (Vagrant)
```ruby
# Create: vagrant/Vagrantfile
Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/jammy64"
  config.vm.hostname = "uveddi-test"
  
  # Resource allocation
  config.vm.provider "virtualbox" do |vb|
    vb.memory = "4096"
    vb.cpus = 2
    vb.name = "uveddi-testing-env"
  end
  
  # Port forwarding for web dashboard
  config.vm.network "forwarded_port", guest: 8080, host: 8080
  config.vm.network "forwarded_port", guest: 3001, host: 3001
  
  # Provision the VM
  config.vm.provision "shell", inline: <<-SHELL
    # Update system
    apt-get update
    apt-get install -y curl build-essential git vim tmux
    
    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    # Install Node.js for rendering service
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    apt-get install -y nodejs
    
    # Set up test user
    adduser --disabled-password --gecos "" testuser
    su - testuser -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
  SHELL
  
  # Copy test setup script
  config.vm.provision "file", source: "test-setup.sh", destination: "/tmp/test-setup.sh"
  config.vm.provision "shell", inline: "chmod +x /tmp/test-setup.sh && /tmp/test-setup.sh"
end
```

#### Environment 3: GitHub Codespaces Configuration
```json
// .devcontainer/test-environment.json
{
    "name": "Uveddi Testing Environment",
    "image": "mcr.microsoft.com/vscode/devcontainers/rust:1",
    "features": {
        "ghcr.io/devcontainers/features/node:1": {
            "version": "18"
        },
        "ghcr.io/devcontainers/features/git:1": {}
    },
    "postCreateCommand": "bash .devcontainer/setup-test-env.sh",
    "forwardPorts": [8080, 3001, 3000],
    "mounts": [
        "source=${localWorkspaceFolder},target=/workspace,type=bind,consistency=cached"
    ],
    "customizations": {
        "vscode": {
            "extensions": [
                "rust-lang.rust-analyzer",
                "ms-vscode.vscode-json"
            ]
        }
    }
}
```

### Testing Workflow Scripts

#### Comprehensive Test Suite
```bash
#!/bin/bash
# scripts/comprehensive-tui-test.sh

set -e

echo "🧪 Comprehensive Uveddi TUI Testing Suite"
echo "========================================"

# Build and install Uveddi
echo "📦 Building and installing Uveddi..."
cargo build --release --features=production
cargo install --path . --features=production --force

# Verify installation
echo "✅ Verifying installation..."
uveddi --version
uveddi --help

# Test basic commands
echo "🔍 Testing basic commands..."
uveddi analyze --help
uveddi config --help
uveddi ui --help
uveddi ci --help
uveddi serve --help

# Test TUI availability
echo "🖥️  Testing TUI integration..."
if uveddi --help | grep -q "tui"; then
    echo "✅ TUI command found in main CLI"
    uveddi tui --help
else
    echo "❌ TUI command NOT found in main CLI"
    echo "   Expected: uveddi tui [OPTIONS]"
    exit 1
fi

# Test TUI launch (with timeout for non-interactive)
echo "🚀 Testing TUI launch..."
timeout 5s uveddi tui test-projects/rust-sample/ || {
    exit_code=$?
    if [ $exit_code -eq 124 ]; then
        echo "✅ TUI launch detected (timed out as expected in non-interactive)"
    else
        echo "❌ TUI launch failed with exit code: $exit_code"
    fi
}

# Test analysis with different formats
echo "📊 Testing analysis outputs..."
uveddi analyze test-projects/rust-sample/ --output-format json --output results.json
uveddi analyze test-projects/python-sample/ --output-format html --output results.html
uveddi analyze test-projects/js-sample/ --output-format markdown --output results.md

echo "✅ All tests completed!"
echo ""
echo "📋 Test Results Summary:"
echo "  • CLI integration: $(uveddi --help | grep -q 'tui' && echo '✅ PASS' || echo '❌ FAIL')"
echo "  • TUI availability: $(uveddi tui --help &>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"
echo "  • Analysis working: $([ -f results.json ] && echo '✅ PASS' || echo '❌ FAIL')"
```

#### User Experience Validation
```bash
#!/bin/bash
# scripts/user-experience-test.sh

echo "👤 User Experience Testing"
echo "========================="

# Simulate first-time user experience
echo "🆕 Simulating first-time user experience..."

# 1. Installation from release
echo "📦 Testing installation from release build..."
# This would test actual installation process

# 2. Help discovery
echo "❓ Testing help discovery..."
uveddi --help | head -20
echo ""
echo "User should be able to easily find:"
echo "  - analyze: for code analysis"  
echo "  - tui: for interactive terminal interface"
echo "  - serve: for web dashboard"

# 3. Basic workflow
echo "🔄 Testing basic user workflow..."

# Quick analysis
uveddi analyze . --output-format markdown | head -10

# TUI accessibility test
echo "🖥️  Testing TUI accessibility..."
echo "Expected: User runs 'uveddi tui' and gets interactive interface"
echo "Fallback: If no TTY, suggest alternative commands"

# 4. Feature discovery
echo "🔍 Testing feature discovery..."
uveddi analyze --help | grep -E "(ai|tui|format)"

echo "✅ User experience validation complete"
```

## Integration Checklist

### Pre-Integration Verification
- [ ] Current TUI works as separate binary (`./target/debug/tui_test`)
- [ ] Main CLI has all commands except TUI (`analyze`, `config`, `ui`, `ci`, `serve`)
- [ ] Production feature set includes `tui` feature
- [ ] TUI code compiles without errors when integrated

### Integration Steps
- [ ] Create `src/cli/tui_command.rs` with proper CLI args
- [ ] Add `Tui(TuiCommand)` to Commands enum in `application/mod.rs`
- [ ] Move TUI launch logic from `bin/tui_test.rs` to main library
- [ ] Add command dispatch in `run_app()` function
- [ ] Update help text and documentation
- [ ] Remove separate `tui_test` binary from Cargo.toml

### Testing Environment Setup
- [ ] Create Docker testing environment
- [ ] Set up Vagrant VM configuration  
- [ ] Configure GitHub Codespaces for testing
- [ ] Create comprehensive test scripts
- [ ] Set up sample projects with known anti-patterns

### Validation Criteria
- [ ] `uveddi tui` launches TUI interface
- [ ] `uveddi --help` shows TUI command
- [ ] TUI works in interactive terminals
- [ ] Graceful error in non-interactive environments
- [ ] Production build includes TUI by default
- [ ] Installation and user experience match expectations

## Expected Outcomes

### User Experience After Integration
```bash
# Primary access method
uveddi tui                          # Launch TUI for current directory
uveddi tui /path/to/project         # Launch TUI for specific project
uveddi tui --load-results report.json  # Launch TUI with existing results

# Alternative access (if implemented)
uveddi analyze --tui               # Analyze and show in TUI
uveddi ui tui                      # Access through UI subcommand

# Error handling
uveddi tui  # In non-interactive environment
# Error: TUI not available: running in non-interactive environment
# Suggestion: Use 'uveddi analyze --output json' for programmatic access
#            or run in a terminal for interactive TUI
```

### File Structure After Integration
```
src/
├── cli/
│   ├── analyze_command.rs
│   ├── config_command.rs  
│   ├── ui_command.rs          # Web UI commands
│   ├── tui_command.rs         # New: TUI CLI integration
│   ├── ci_command.rs
│   └── mod.rs
├── tui/
│   ├── mod.rs                 # Main TUI module
│   ├── app.rs                 # TUI application logic  
│   ├── cli_integration.rs     # New: CLI bridge
│   └── terminal.rs            # Terminal handling
├── application/mod.rs         # Updated: includes TUI command
└── main.rs                    # Simplified: just calls run_app()
```

## Success Metrics

1. **Integration Success**: `uveddi tui` works exactly like current `./target/debug/tui_test`
2. **User Experience**: New users can discover and use TUI through standard CLI
3. **Build System**: Production builds include TUI without extra configuration  
4. **Testing**: Isolated environments validate complete user experience
5. **Documentation**: Clear instructions for TUI usage and troubleshooting

This prompt provides complete guidance for both TUI integration and comprehensive testing infrastructure setup.