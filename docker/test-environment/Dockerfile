# Docker testing environment for Uveddi TUI integration validation
# This environment simulates an end-user installation experience

FROM rust:1.70-slim

# Install system dependencies for building and testing
RUN apt-get update && apt-get install -y \
    git curl build-essential pkg-config libssl-dev \
    tmux vim less tree htop procps \
    # Required for TUI testing
    xterm screen \
    # Node.js for rendering service testing
    nodejs npm \
    && rm -rf /var/lib/apt/lists/*

# Create non-root test user for realistic testing
RUN useradd -m -s /bin/bash testuser && \
    usermod -aG sudo testuser && \
    echo "testuser ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Switch to test user
USER testuser
WORKDIR /home/testuser

# Install Rust for test user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/testuser/.cargo/bin:${PATH}"

# Create test workspace structure
RUN mkdir -p /home/testuser/test-workspace/{projects,results,scripts}

# Copy test setup scripts
COPY --chown=testuser:testuser scripts/test-setup.sh /home/testuser/scripts/
COPY --chown=testuser:testuser scripts/comprehensive-tui-test.sh /home/testuser/scripts/
COPY --chown=testuser:testuser scripts/user-experience-test.sh /home/testuser/scripts/

# Make scripts executable
RUN chmod +x /home/testuser/scripts/*.sh

# Set up test projects with known anti-patterns
RUN mkdir -p /home/testuser/test-workspace/projects/{rust-sample,python-sample,js-sample,mixed-project}

# Create Rust sample project with anti-patterns
RUN cat > /home/testuser/test-workspace/projects/rust-sample/Cargo.toml << 'EOF'
[package]
name = "rust-sample"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
reqwest = "0.11"
EOF

RUN mkdir -p /home/testuser/test-workspace/projects/rust-sample/src

RUN cat > /home/testuser/test-workspace/projects/rust-sample/src/main.rs << 'EOF'
// Rust sample with multiple anti-patterns for Uveddi testing
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// God Object anti-pattern - too many responsibilities
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationManager {
    // Data management
    pub user_data: HashMap<String, String>,
    pub config_data: HashMap<String, String>,
    pub cache_data: HashMap<String, Vec<u8>>,
    
    // Network management
    pub api_endpoints: Vec<String>,
    pub connection_pool: Vec<String>,
    pub retry_config: HashMap<String, u32>,
    
    // File management
    pub file_paths: Vec<String>,
    pub temp_directories: Vec<String>,
    pub log_files: Vec<String>,
    
    // Business logic
    pub processing_rules: Vec<String>,
    pub validation_rules: Vec<String>,
    pub transformation_rules: Vec<String>,
    
    // UI state
    pub current_screen: String,
    pub dialog_state: String,
    pub form_data: HashMap<String, String>,
}

impl ApplicationManager {
    // Long method anti-pattern
    pub fn initialize_everything(&mut self) -> Result<(), String> {
        // Data initialization
        self.user_data.insert("admin".to_string(), "password123".to_string());
        self.user_data.insert("user".to_string(), "user456".to_string());
        self.config_data.insert("theme".to_string(), "dark".to_string());
        self.config_data.insert("language".to_string(), "en".to_string());
        
        // Network setup
        self.api_endpoints.push("https://api.example.com/v1".to_string());
        self.api_endpoints.push("https://api.example.com/v2".to_string());
        self.connection_pool.push("pool1".to_string());
        self.connection_pool.push("pool2".to_string());
        
        // File system setup
        self.file_paths.push("/tmp/app".to_string());
        self.file_paths.push("/var/log/app".to_string());
        self.temp_directories.push("/tmp/session1".to_string());
        self.temp_directories.push("/tmp/session2".to_string());
        
        // Business rules setup
        self.processing_rules.push("rule1: validate input".to_string());
        self.processing_rules.push("rule2: transform data".to_string());
        self.validation_rules.push("email must be valid".to_string());
        self.validation_rules.push("password min 8 chars".to_string());
        
        // UI initialization
        self.current_screen = "main".to_string();
        self.dialog_state = "closed".to_string();
        
        // This method is way too long and does too many things
        // More initialization code...
        for i in 0..50 {
            self.cache_data.insert(
                format!("cache_key_{}", i),
                vec![i as u8; 100]
            );
        }
        
        Ok(())
    }
    
    // Code duplication - similar validation logic
    pub fn validate_user_input(&self, input: &str) -> bool {
        if input.is_empty() {
            return false;
        }
        if input.len() < 3 {
            return false;
        }
        if input.len() > 100 {
            return false;
        }
        if input.chars().any(|c| c.is_control()) {
            return false;
        }
        true
    }
    
    // Code duplication - very similar validation logic
    pub fn validate_config_value(&self, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        if value.len() < 3 {
            return false;
        }
        if value.len() > 100 {
            return false;
        }
        if value.chars().any(|c| c.is_control()) {
            return false;
        }
        true
    }
    
    // Magic numbers anti-pattern
    pub fn calculate_timeout(&self) -> u64 {
        let base_timeout = 5000; // milliseconds
        let retry_multiplier = 2;
        let max_retries = 3;
        
        base_timeout * retry_multiplier * max_retries
    }
}

// Dead code - unused function
fn unused_helper_function() -> String {
    "This function is never called".to_string()
}

// Dead code - unused struct
#[derive(Debug)]
struct UnusedConfiguration {
    setting1: String,
    setting2: i32,
}

impl UnusedConfiguration {
    fn new() -> Self {
        Self {
            setting1: "default".to_string(),
            setting2: 42,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = ApplicationManager {
        user_data: HashMap::new(),
        config_data: HashMap::new(),
        cache_data: HashMap::new(),
        api_endpoints: Vec::new(),
        connection_pool: Vec::new(),
        retry_config: HashMap::new(),
        file_paths: Vec::new(),
        temp_directories: Vec::new(),
        log_files: Vec::new(),
        processing_rules: Vec::new(),
        validation_rules: Vec::new(),
        transformation_rules: Vec::new(),
        current_screen: String::new(),
        dialog_state: String::new(),
        form_data: HashMap::new(),
    };
    
    app.initialize_everything().map_err(|e| e)?;
    
    println!("Application initialized with {} users", app.user_data.len());
    println!("Timeout calculation: {} ms", app.calculate_timeout());
    
    Ok(())
}
EOF

# Create Python sample project
RUN cat > /home/testuser/test-workspace/projects/python-sample/main.py << 'EOF'
# Python sample with anti-patterns for testing
import json
import time
from typing import Dict, List, Any

class DataProcessor:
    """God object that handles too many responsibilities"""
    
    def __init__(self):
        # Data storage
        self.user_data: Dict[str, Any] = {}
        self.config_data: Dict[str, Any] = {}
        self.cache_data: Dict[str, Any] = {}
        
        # Network configuration
        self.api_endpoints: List[str] = []
        self.connection_settings: Dict[str, Any] = {}
        
        # File management
        self.file_paths: List[str] = []
        self.log_settings: Dict[str, Any] = {}
        
        # Business logic
        self.processing_rules: List[str] = []
        self.validation_rules: List[str] = []
    
    def process_everything(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Long method that does too many things"""
        # Input validation
        if not data:
            raise ValueError("Data cannot be empty")
        
        # Data transformation
        transformed_data = {}
        for key, value in data.items():
            if isinstance(value, str):
                transformed_data[key] = value.strip().lower()
            elif isinstance(value, (int, float)):
                transformed_data[key] = value * 1.1  # Magic number
            else:
                transformed_data[key] = str(value)
        
        # Caching logic
        cache_key = f"processed_{hash(str(data))}"
        self.cache_data[cache_key] = transformed_data
        
        # Logging
        self.log_operation("process_everything", len(data))
        
        # Business rule application
        for rule in self.processing_rules:
            transformed_data = self.apply_rule(rule, transformed_data)
        
        # Final validation
        if not self.validate_result(transformed_data):
            raise ValueError("Processing failed validation")
        
        return transformed_data
    
    # Code duplication - similar validation logic
    def validate_user_input(self, input_data: str) -> bool:
        if not input_data:
            return False
        if len(input_data) < 3:
            return False
        if len(input_data) > 100:
            return False
        if any(ord(c) < 32 for c in input_data):
            return False
        return True
    
    # Code duplication - very similar validation
    def validate_config_input(self, config_value: str) -> bool:
        if not config_value:
            return False
        if len(config_value) < 3:
            return False
        if len(config_value) > 100:
            return False
        if any(ord(c) < 32 for c in config_value):
            return False
        return True
    
    def apply_rule(self, rule: str, data: Dict[str, Any]) -> Dict[str, Any]:
        # Simplified rule application
        return data
    
    def validate_result(self, result: Dict[str, Any]) -> bool:
        return len(result) > 0
    
    def log_operation(self, operation: str, data_size: int):
        timestamp = time.time()
        print(f"[{timestamp}] {operation}: processed {data_size} items")

# Dead code - unused function
def unused_utility_function():
    """This function is never called"""
    return "unused"

# Dead code - unused class
class UnusedHelper:
    def __init__(self):
        self.data = {}
    
    def helper_method(self):
        return "not used"

def main():
    processor = DataProcessor()
    
    # Sample data processing
    sample_data = {
        "name": "  Test User  ",
        "age": 25,
        "score": 85.5,
        "active": True
    }
    
    try:
        result = processor.process_everything(sample_data)
        print(f"Processing result: {json.dumps(result, indent=2)}")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
EOF

# Create JavaScript sample project
RUN cat > /home/testuser/test-workspace/projects/js-sample/package.json << 'EOF'
{
  "name": "js-sample",
  "version": "1.0.0",
  "description": "JavaScript sample with anti-patterns",
  "main": "main.js",
  "scripts": {
    "start": "node main.js"
  },
  "dependencies": {
    "lodash": "^4.17.21"
  }
}
EOF

RUN cat > /home/testuser/test-workspace/projects/js-sample/main.js << 'EOF'
// JavaScript sample with anti-patterns for testing
const _ = require('lodash');

// God object anti-pattern
class ApplicationController {
    constructor() {
        // Data management
        this.userData = new Map();
        this.configData = new Map();
        this.cacheData = new Map();
        
        // Network management
        this.apiEndpoints = [];
        this.connectionPools = [];
        this.retrySettings = {};
        
        // File management
        this.filePaths = [];
        this.logConfiguration = {};
        
        // UI state
        this.currentView = '';
        this.formData = {};
        this.modalState = {};
        
        // Business logic
        this.processingRules = [];
        this.validationRules = [];
    }
    
    // Long method with too many responsibilities
    initializeApplication() {
        // User data setup
        this.userData.set('admin', { role: 'administrator', permissions: ['read', 'write', 'delete'] });
        this.userData.set('user', { role: 'user', permissions: ['read'] });
        
        // Configuration setup
        this.configData.set('theme', 'dark');
        this.configData.set('language', 'en');
        this.configData.set('timeout', 5000);
        this.configData.set('retries', 3);
        
        // API endpoints
        this.apiEndpoints.push('https://api.example.com/v1/users');
        this.apiEndpoints.push('https://api.example.com/v1/data');
        this.apiEndpoints.push('https://api.example.com/v2/analytics');
        
        // File paths
        this.filePaths.push('/tmp/app-data');
        this.filePaths.push('/var/log/application.log');
        this.filePaths.push('/etc/app/config.json');
        
        // Business rules
        this.processingRules.push('validate-input');
        this.processingRules.push('transform-data');
        this.processingRules.push('apply-business-logic');
        this.processingRules.push('generate-output');
        
        // Cache initialization
        for (let i = 0; i < 50; i++) {
            this.cacheData.set(`cache-${i}`, { data: new Array(100).fill(i) });
        }
        
        // UI initialization
        this.currentView = 'dashboard';
        this.formData = { form1: {}, form2: {}, form3: {} };
        this.modalState = { isOpen: false, type: '', data: null };
        
        console.log('Application initialized with', this.userData.size, 'users');
        return true;
    }
    
    // Code duplication - similar validation logic
    validateUserInput(input) {
        if (!input) return false;
        if (input.length < 3) return false;
        if (input.length > 100) return false;
        if (/[\x00-\x1F]/.test(input)) return false;
        return true;
    }
    
    // Code duplication - very similar validation
    validateConfigValue(value) {
        if (!value) return false;
        if (value.length < 3) return false;
        if (value.length > 100) return false;
        if (/[\x00-\x1F]/.test(value)) return false;
        return true;
    }
    
    // Magic numbers anti-pattern
    calculateProcessingDelay() {
        const baseDelay = 1000; // milliseconds
        const loadFactor = 2.5;
        const priorityMultiplier = 3;
        
        return baseDelay * loadFactor * priorityMultiplier;
    }
    
    // Long parameter list anti-pattern
    processUserData(userId, userName, userEmail, userRole, userPermissions, userPreferences, 
                   userSettings, userHistory, userMetadata, creationDate, lastLogin) {
        // Processing logic with too many parameters
        console.log('Processing user:', userName);
        return {
            id: userId,
            processed: true,
            timestamp: Date.now()
        };
    }
}

// Dead code - unused functions
function unusedHelperFunction() {
    return "This function is never called";
}

function anotherUnusedFunction(param1, param2) {
    const result = param1 + param2;
    return result * 2;
}

// Dead code - unused class
class UnusedDataHandler {
    constructor() {
        this.data = [];
    }
    
    addData(item) {
        this.data.push(item);
    }
    
    getData() {
        return this.data;
    }
}

// Main execution
function main() {
    const app = new ApplicationController();
    app.initializeApplication();
    
    // Demonstrate some functionality
    console.log('User validation:', app.validateUserInput('test-user'));
    console.log('Config validation:', app.validateConfigValue('some-config'));
    console.log('Processing delay:', app.calculateProcessingDelay());
    
    // Process sample user
    const processedUser = app.processUserData(
        1, 'John Doe', 'john@example.com', 'user', 
        ['read'], {theme: 'dark'}, {notifications: true}, 
        [], {created: '2023-01-01'}, '2023-01-01', '2024-01-01'
    );
    
    console.log('Processed user:', processedUser);
}

// Run the application
main();
EOF

# Set default work directory and environment
WORKDIR /home/testuser/test-workspace

# Add helpful aliases and environment setup
RUN echo 'alias ll="ls -la"' >> ~/.bashrc && \
    echo 'alias uveddi-help="echo \"Uveddi TUI Testing Commands:\n  uveddi --help         - Show all commands\n  uveddi tui --help     - Show TUI options\n  uveddi tui ./projects/rust-sample  - Test TUI with Rust project\n  ./scripts/comprehensive-tui-test.sh - Run full test suite\""' >> ~/.bashrc && \
    echo 'export PATH="/home/testuser/.cargo/bin:$PATH"' >> ~/.bashrc

# Create a welcome message
RUN cat > /home/testuser/WELCOME.md << 'EOF'
# Uveddi TUI Testing Environment

Welcome to the isolated Uveddi testing environment! This container provides a clean
environment to test Uveddi as an end user would experience it.

## Available Test Projects

- **rust-sample/**: Rust project with God Object, dead code, and duplication
- **python-sample/**: Python project with similar anti-patterns  
- **js-sample/**: JavaScript project with long methods and code duplication

## Testing Commands

1. **Build and Install Uveddi**:
   ```bash
   # Copy Uveddi source (mounted at /workspace)
   cp -r /workspace /home/testuser/uveddi-source
   cd /home/testuser/uveddi-source
   
   # Build with production features
   cargo build --release --features=production
   
   # Install to user path
   cargo install --path . --features=production
   ```

2. **Test TUI Integration**:
   ```bash
   # Check help system
   uveddi --help
   uveddi tui --help
   
   # Test with sample projects
   uveddi tui ./projects/rust-sample
   uveddi tui ./projects/python-sample --debug
   uveddi tui ./projects/js-sample --skip-analysis
   ```

3. **Run Comprehensive Tests**:
   ```bash
   ./scripts/comprehensive-tui-test.sh
   ./scripts/user-experience-test.sh
   ```

## Terminal Testing

The environment includes multiple terminal emulators for TUI testing:
- Use `tmux` for session management
- Use `screen` for alternative terminal session testing
- Test terminal resizing and responsiveness

## Expected Test Results

✅ **Success Criteria**:
- `uveddi tui` command available and documented in help
- Graceful error handling in non-interactive environments
- Proper terminal size validation and warnings
- TUI launches correctly in interactive terminals
- All anti-patterns detected in sample projects

❌ **Failure Scenarios**:
- TUI command not found in main CLI
- Poor error messages for environment issues
- TUI crashes or hangs during startup
- Anti-patterns not detected in sample code

Happy testing! 🚀
EOF

CMD ["/bin/bash"]
EOF