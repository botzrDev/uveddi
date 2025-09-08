# Plugin Security and Permissions

## Overview

Uveddi's WASM plugin system implements a comprehensive security model designed to provide safe extensibility while maintaining system integrity. The security architecture uses capability-based permissions, resource limits, and sandboxed execution to ensure plugins cannot compromise the host system or access unauthorized resources.

## Security Architecture

### Sandboxed Execution Environment

All plugins execute within a WebAssembly sandbox that provides:

- **Memory Isolation**: Plugins cannot access host memory directly
- **CPU Isolation**: Resource usage monitoring and fuel-based execution limits  
- **File System Isolation**: No direct file system access (only through host functions)
- **Network Isolation**: No direct network access without explicit permission
- **System Call Isolation**: No ability to make system calls directly

### Capability-Based Security Model

Uveddi uses a capability-based security model where plugins must explicitly declare required permissions in their manifest, and the host validates and enforces these permissions at runtime.

```rust
// Core permission types
pub enum Permission {
    // File system access
    FileRead(PathBuf),          // Read specific files or directories
    FileWrite(PathBuf),         // Write to specific files or directories
    TempFileCreate,             // Create temporary files in sandboxed directory
    
    // Network access  
    NetworkConnect(String),     // Connect to specific URLs or domains
    NetworkListen(u16),         // Listen on specific ports (rarely granted)
    
    // Configuration access
    ConfigRead,                 // Read project configuration
    ConfigWrite,                // Modify project configuration
    
    // System interaction
    EnvRead(String),           // Read specific environment variables
    ProcessSpawn(String),       // Execute specific external commands
    
    // Uveddi services
    Logging,                   // Write to log output
    DatabaseRead,              // Read from analysis database
    DatabaseWrite,             // Write to analysis database
    
    // Advanced features
    PluginCommunication,       // Communicate with other plugins
    HostFunctionCall(String),  // Call additional host functions
}
```

## Permission System

### Permission Declaration

Plugins must declare all required permissions in their `plugin.toml` manifest:

```toml
[capabilities]
# Basic permissions
permissions = [
    "ConfigRead",           # Read project configuration
    "Logging",             # Write to logs
    "TempFileCreate"       # Create temp files
]

# File system permissions (path-restricted)
file_read_paths = [
    "./src/**/*.rs",       # Read Rust source files
    "./config.toml"        # Read specific config file
]

file_write_paths = [
    "./target/plugin-cache/**"  # Write to specific cache directory
]

# Network permissions (URL-restricted)
network_domains = [
    "api.example.com",     # Allowed domain
    "https://registry.npmjs.org"  # Specific HTTPS endpoints
]

# Environment variable access
env_read_vars = [
    "PLUGIN_CONFIG_*",     # Pattern-based access
    "CI"                   # Specific variables
]

# Resource limits
max_memory_mb = 32         # Maximum memory allocation
max_execution_seconds = 30  # Maximum execution time
fuel_limit = 2000000       # Maximum computational operations
max_file_operations = 100  # Maximum file I/O operations
max_network_requests = 10  # Maximum network requests
```

### Permission Validation

The host system validates permissions at multiple levels:

```rust
// Permission checking at runtime
pub struct SecurityValidator {
    granted_permissions: HashSet<Permission>,
    resource_limits: ResourceLimits,
    runtime_state: RuntimeSecurityState,
}

impl SecurityValidator {
    pub fn check_file_access(&self, path: &Path, operation: FileOperation) -> Result<(), SecurityError> {
        match operation {
            FileOperation::Read => {
                // Check if plugin has FileRead permission for this path
                for permission in &self.granted_permissions {
                    if let Permission::FileRead(allowed_path) = permission {
                        if path.starts_with(allowed_path) {
                            return Ok(());
                        }
                    }
                }
                Err(SecurityError::PermissionDenied("File read not permitted"))
            }
            FileOperation::Write => {
                // Similar validation for write operations
                self.validate_file_write(path)
            }
        }
    }
    
    pub fn check_network_access(&self, url: &str) -> Result<(), SecurityError> {
        for permission in &self.granted_permissions {
            if let Permission::NetworkConnect(allowed_domain) = permission {
                if url.contains(allowed_domain) {
                    return Ok(());
                }
            }
        }
        Err(SecurityError::PermissionDenied("Network access not permitted"))
    }
    
    pub fn check_resource_usage(&mut self, resource_type: ResourceType, amount: u64) -> Result<(), SecurityError> {
        match resource_type {
            ResourceType::Memory => {
                if self.runtime_state.memory_used + amount > self.resource_limits.max_memory {
                    return Err(SecurityError::ResourceLimitExceeded("Memory limit exceeded"));
                }
                self.runtime_state.memory_used += amount;
            }
            ResourceType::FileOperations => {
                if self.runtime_state.file_operations >= self.resource_limits.max_file_operations {
                    return Err(SecurityError::ResourceLimitExceeded("File operation limit exceeded"));
                }
                self.runtime_state.file_operations += 1;
            }
            ResourceType::NetworkRequests => {
                if self.runtime_state.network_requests >= self.resource_limits.max_network_requests {
                    return Err(SecurityError::ResourceLimitExceeded("Network request limit exceeded"));
                }
                self.runtime_state.network_requests += 1;
            }
        }
        Ok(())
    }
}
```

## Resource Limits

### Computational Limits

```rust
pub struct ComputationalLimits {
    pub fuel_limit: u64,              // Maximum WASM instructions
    pub execution_timeout: Duration,   // Maximum wall-clock time
    pub max_recursion_depth: usize,   // Stack overflow prevention
    pub max_loop_iterations: u64,     // Infinite loop prevention
}

// Default limits for different plugin types
impl ComputationalLimits {
    pub fn light_analysis() -> Self {
        Self {
            fuel_limit: 500_000,           // ~0.5M instructions
            execution_timeout: Duration::from_secs(10),
            max_recursion_depth: 100,
            max_loop_iterations: 100_000,
        }
    }
    
    pub fn heavy_analysis() -> Self {
        Self {
            fuel_limit: 5_000_000,         // ~5M instructions
            execution_timeout: Duration::from_secs(60),
            max_recursion_depth: 500,
            max_loop_iterations: 1_000_000,
        }
    }
    
    pub fn custom(fuel: u64, timeout_secs: u64) -> Self {
        Self {
            fuel_limit: fuel,
            execution_timeout: Duration::from_secs(timeout_secs),
            max_recursion_depth: 200,
            max_loop_iterations: fuel / 10,
        }
    }
}
```

### Memory Management

```rust
pub struct MemoryLimits {
    pub max_heap_size: usize,      // Maximum heap allocation
    pub max_stack_size: usize,     // Maximum stack size  
    pub max_globals: usize,        // Maximum global variables
    pub enable_memory_protection: bool, // Enable additional protections
}

// Memory monitoring and enforcement
pub struct MemoryMonitor {
    current_usage: usize,
    peak_usage: usize,
    allocation_count: u64,
    limits: MemoryLimits,
}

impl MemoryMonitor {
    pub fn allocate(&mut self, size: usize) -> Result<(), SecurityError> {
        if self.current_usage + size > self.limits.max_heap_size {
            return Err(SecurityError::ResourceLimitExceeded(
                format!("Memory allocation would exceed limit: {} + {} > {}",
                    self.current_usage, size, self.limits.max_heap_size)
            ));
        }
        
        self.current_usage += size;
        self.peak_usage = self.peak_usage.max(self.current_usage);
        self.allocation_count += 1;
        
        Ok(())
    }
    
    pub fn deallocate(&mut self, size: usize) {
        self.current_usage = self.current_usage.saturating_sub(size);
    }
    
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage: self.current_usage,
            peak_usage: self.peak_usage,
            allocation_count: self.allocation_count,
            limit: self.limits.max_heap_size,
        }
    }
}
```

### I/O and Network Limits

```rust
pub struct IOLimits {
    pub max_file_size_read: u64,      // Maximum file size for read operations
    pub max_file_size_write: u64,     // Maximum file size for write operations
    pub max_files_open: usize,        // Maximum concurrent open files
    pub max_file_operations: u64,     // Maximum total file operations
    pub allowed_file_extensions: Vec<String>, // Allowed file types
    pub blocked_paths: Vec<PathBuf>,   // Explicitly blocked paths
}

pub struct NetworkLimits {
    pub max_request_size: u64,        // Maximum request body size
    pub max_response_size: u64,       // Maximum response body size
    pub max_concurrent_connections: usize, // Maximum concurrent connections
    pub max_requests_per_minute: u64, // Rate limiting
    pub allowed_protocols: Vec<String>, // HTTP/HTTPS only
    pub connection_timeout: Duration,  // Connection timeout
    pub request_timeout: Duration,     // Request timeout
}
```

## Security Policies

### Policy Configuration

```rust
pub struct SecurityPolicy {
    pub trust_level: TrustLevel,
    pub permissions: PermissionSet,
    pub resource_limits: ResourceLimits,
    pub validation_rules: ValidationRules,
    pub monitoring_settings: MonitoringSettings,
}

pub enum TrustLevel {
    Untrusted,      // Minimal permissions, strict limits
    Basic,          // Standard permissions for community plugins
    Verified,       // Extended permissions for verified publishers
    Trusted,        // Full permissions for internal/trusted plugins
    Development,    // Relaxed limits for development environment
}

impl SecurityPolicy {
    pub fn for_trust_level(level: TrustLevel) -> Self {
        match level {
            TrustLevel::Untrusted => Self::untrusted_policy(),
            TrustLevel::Basic => Self::basic_policy(),
            TrustLevel::Verified => Self::verified_policy(),
            TrustLevel::Trusted => Self::trusted_policy(),
            TrustLevel::Development => Self::development_policy(),
        }
    }
    
    fn untrusted_policy() -> Self {
        Self {
            trust_level: TrustLevel::Untrusted,
            permissions: PermissionSet {
                allowed: vec![Permission::Logging, Permission::ConfigRead],
                denied: vec![
                    Permission::FileWrite(PathBuf::from("/")),
                    Permission::NetworkConnect("*".to_string()),
                    Permission::ProcessSpawn("*".to_string()),
                ],
            },
            resource_limits: ResourceLimits {
                max_memory_mb: 16,
                max_execution_seconds: 10,
                fuel_limit: 500_000,
                max_file_operations: 0,
                max_network_requests: 0,
            },
            validation_rules: ValidationRules::strict(),
            monitoring_settings: MonitoringSettings::verbose(),
        }
    }
    
    fn verified_policy() -> Self {
        Self {
            trust_level: TrustLevel::Verified,
            permissions: PermissionSet {
                allowed: vec![
                    Permission::Logging,
                    Permission::ConfigRead,
                    Permission::TempFileCreate,
                    Permission::DatabaseRead,
                ],
                denied: vec![
                    Permission::ProcessSpawn("*".to_string()),
                    Permission::ConfigWrite,
                ],
            },
            resource_limits: ResourceLimits {
                max_memory_mb: 64,
                max_execution_seconds: 60,
                fuel_limit: 2_000_000,
                max_file_operations: 100,
                max_network_requests: 10,
            },
            validation_rules: ValidationRules::standard(),
            monitoring_settings: MonitoringSettings::standard(),
        }
    }
}
```

### Runtime Policy Enforcement

```rust
pub struct PolicyEnforcer {
    policy: SecurityPolicy,
    runtime_state: RuntimeSecurityState,
    violation_handler: ViolationHandler,
}

impl PolicyEnforcer {
    pub fn enforce_operation(&mut self, operation: PluginOperation) -> Result<(), SecurityError> {
        // Pre-operation validation
        self.validate_operation(&operation)?;
        
        // Resource usage checking
        self.check_resource_usage(&operation)?;
        
        // Permission validation
        self.validate_permissions(&operation)?;
        
        // Rate limiting
        self.check_rate_limits(&operation)?;
        
        // Update runtime state
        self.update_state(&operation);
        
        Ok(())
    }
    
    fn validate_operation(&self, operation: &PluginOperation) -> Result<(), SecurityError> {
        match operation {
            PluginOperation::FileRead(path) => {
                // Validate file path
                if self.is_blocked_path(path) {
                    return Err(SecurityError::OperationBlocked("Path is blocked"));
                }
                
                // Check file extension
                if !self.is_allowed_extension(path) {
                    return Err(SecurityError::OperationBlocked("File type not allowed"));
                }
                
                // Check file size
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.len() > self.policy.resource_limits.max_file_size_read {
                        return Err(SecurityError::OperationBlocked("File too large"));
                    }
                }
            }
            PluginOperation::NetworkRequest(url) => {
                // Validate URL
                if !self.is_allowed_url(url) {
                    return Err(SecurityError::OperationBlocked("URL not allowed"));
                }
                
                // Check protocol
                if !self.is_allowed_protocol(url) {
                    return Err(SecurityError::OperationBlocked("Protocol not allowed"));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn handle_violation(&mut self, violation: SecurityViolation) {
        self.violation_handler.record_violation(violation);
        
        // Escalation based on severity
        match violation.severity {
            ViolationSeverity::Low => {
                // Log warning
                tracing::warn!("Security violation: {}", violation.description);
            }
            ViolationSeverity::Medium => {
                // Reduce plugin privileges temporarily  
                self.policy.resource_limits.reduce_limits(0.5);
                tracing::error!("Security violation, reducing limits: {}", violation.description);
            }
            ViolationSeverity::High => {
                // Terminate plugin execution
                self.runtime_state.terminated = true;
                tracing::error!("Critical security violation, terminating plugin: {}", violation.description);
            }
        }
    }
}
```

## Threat Model and Mitigations

### Identified Threats

| Threat | Risk Level | Mitigation |
|--------|------------|------------|
| **Malicious Code Execution** | High | WASM sandboxing, fuel limits, permission system |
| **Resource Exhaustion (DoS)** | High | Memory limits, execution timeouts, fuel consumption tracking |
| **Data Exfiltration** | Medium | Network restrictions, file system isolation, permission validation |
| **Privilege Escalation** | Medium | Capability-based security, runtime permission checking |
| **Host System Access** | High | Complete sandboxing, no direct system calls |
| **Plugin-to-Plugin Attacks** | Low | Process isolation, separate memory spaces |
| **Configuration Tampering** | Medium | Read-only configuration access by default |
| **Side-Channel Attacks** | Low | Resource monitoring, timing attack prevention |

### Defense in Depth

```rust
// Multi-layer security implementation
pub struct SecurityStack {
    // Layer 1: WASM Sandbox
    wasm_runtime: WasmRuntime,          // Fundamental isolation
    
    // Layer 2: Capability System
    permission_validator: PermissionValidator,
    
    // Layer 3: Resource Management
    resource_monitor: ResourceMonitor,
    
    // Layer 4: Policy Enforcement
    policy_enforcer: PolicyEnforcer,
    
    // Layer 5: Monitoring and Auditing
    security_monitor: SecurityMonitor,
    audit_logger: AuditLogger,
}

impl SecurityStack {
    pub fn execute_plugin_operation(&mut self, plugin_id: &str, operation: PluginOperation) -> Result<PluginResult, SecurityError> {
        // Layer 1: Basic WASM validation
        self.wasm_runtime.validate_operation(&operation)?;
        
        // Layer 2: Permission checking
        self.permission_validator.check_permissions(plugin_id, &operation)?;
        
        // Layer 3: Resource validation
        self.resource_monitor.check_resources(plugin_id, &operation)?;
        
        // Layer 4: Policy enforcement
        self.policy_enforcer.enforce_policy(plugin_id, &operation)?;
        
        // Layer 5: Audit logging
        self.audit_logger.log_operation(plugin_id, &operation);
        
        // Execute operation with monitoring
        let result = self.security_monitor.monitored_execution(|| {
            self.wasm_runtime.execute_operation(operation)
        })?;
        
        // Post-execution validation
        self.validate_result(&result)?;
        
        Ok(result)
    }
}
```

### Vulnerability Prevention

```rust
pub struct VulnerabilityPrevention {
    input_validator: InputValidator,
    output_sanitizer: OutputSanitizer,
    timing_guard: TimingGuard,
    memory_guard: MemoryGuard,
}

impl VulnerabilityPrevention {
    pub fn validate_plugin_input(&self, input: &[u8]) -> Result<(), SecurityError> {
        // Size validation
        if input.len() > MAX_INPUT_SIZE {
            return Err(SecurityError::InputTooLarge);
        }
        
        // Format validation  
        self.input_validator.validate_json_structure(input)?;
        
        // Content validation
        self.input_validator.validate_content_safety(input)?;
        
        // Encoding validation
        if !self.input_validator.is_valid_utf8(input) {
            return Err(SecurityError::InvalidEncoding);
        }
        
        Ok(())
    }
    
    pub fn sanitize_plugin_output(&self, output: &mut Vec<u8>) -> Result<(), SecurityError> {
        // Size limits
        if output.len() > MAX_OUTPUT_SIZE {
            output.truncate(MAX_OUTPUT_SIZE);
        }
        
        // Remove potentially dangerous content
        self.output_sanitizer.remove_script_tags(output);
        self.output_sanitizer.escape_html_entities(output);
        
        // Validate JSON structure
        self.output_sanitizer.validate_json_output(output)?;
        
        Ok(())
    }
    
    pub fn prevent_timing_attacks(&self, operation: &PluginOperation) -> TimingGuard {
        // Add random delays to prevent timing analysis
        TimingGuard::new(operation)
    }
}
```

## Security Monitoring

### Real-Time Monitoring

```rust
pub struct SecurityMonitor {
    violation_detector: ViolationDetector,
    anomaly_detector: AnomalyDetector,
    performance_monitor: PerformanceMonitor,
    audit_trail: AuditTrail,
}

impl SecurityMonitor {
    pub fn monitor_plugin_execution(&mut self, plugin_id: &str) -> MonitoringSession {
        MonitoringSession::new(plugin_id, vec![
            Box::new(ResourceUsageMonitor::new()),
            Box::new(PermissionUsageMonitor::new()),
            Box::new(BehaviorMonitor::new()),
            Box::new(PerformanceMonitor::new()),
        ])
    }
    
    pub fn detect_anomalies(&mut self, metrics: &PluginMetrics) -> Vec<SecurityAnomaly> {
        let mut anomalies = Vec::new();
        
        // Unusual resource usage patterns
        if self.anomaly_detector.detect_memory_anomaly(&metrics.memory_usage) {
            anomalies.push(SecurityAnomaly::UnusualMemoryPattern);
        }
        
        // Suspicious file access patterns
        if self.anomaly_detector.detect_file_access_anomaly(&metrics.file_operations) {
            anomalies.push(SecurityAnomaly::SuspiciousFileAccess);
        }
        
        // Unexpected network activity
        if self.anomaly_detector.detect_network_anomaly(&metrics.network_activity) {
            anomalies.push(SecurityAnomaly::UnexpectedNetworkActivity);
        }
        
        // Permission usage patterns
        if self.anomaly_detector.detect_permission_anomaly(&metrics.permission_usage) {
            anomalies.push(SecurityAnomaly::UnusualPermissionUsage);
        }
        
        anomalies
    }
}

#[derive(Debug)]
pub enum SecurityAnomaly {
    UnusualMemoryPattern,
    SuspiciousFileAccess,
    UnexpectedNetworkActivity,
    UnusualPermissionUsage,
    ExcessiveResourceConsumption,
    AnomalousExecutionPattern,
}
```

### Audit and Compliance

```rust
pub struct AuditLogger {
    log_writer: Box<dyn Write + Send>,
    compliance_checker: ComplianceChecker,
    retention_policy: RetentionPolicy,
}

impl AuditLogger {
    pub fn log_security_event(&mut self, event: SecurityEvent) {
        let audit_record = AuditRecord {
            timestamp: SystemTime::now(),
            event_type: event.event_type,
            plugin_id: event.plugin_id,
            operation: event.operation,
            result: event.result,
            risk_level: event.risk_level,
            context: event.context,
        };
        
        // Write to audit log
        if let Err(e) = self.write_audit_record(&audit_record) {
            tracing::error!("Failed to write audit record: {}", e);
        }
        
        // Check compliance requirements
        self.compliance_checker.validate_event(&audit_record);
        
        // Apply retention policy
        self.retention_policy.apply(&audit_record);
    }
    
    fn write_audit_record(&mut self, record: &AuditRecord) -> std::io::Result<()> {
        let json = serde_json::to_string(record)?;
        writeln!(self.log_writer, "{}", json)?;
        self.log_writer.flush()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: SystemTime,
    pub event_type: SecurityEventType,
    pub plugin_id: String,
    pub operation: String,
    pub result: OperationResult,
    pub risk_level: RiskLevel,
    pub context: HashMap<String, String>,
}
```

## Best Practices for Plugin Developers

### Secure Development Guidelines

1. **Minimal Permissions**: Request only the minimum permissions required for functionality
2. **Input Validation**: Validate all inputs thoroughly before processing
3. **Output Sanitization**: Sanitize all outputs to prevent injection attacks
4. **Error Handling**: Implement comprehensive error handling without leaking sensitive information
5. **Resource Management**: Implement efficient resource usage and cleanup
6. **Secure Dependencies**: Use only necessary dependencies and keep them updated
7. **Testing**: Include security-focused testing in your test suite

### Security Checklist

```markdown
## Plugin Security Checklist

### Permissions
- [ ] Requested permissions are minimal and necessary
- [ ] File access is restricted to required directories
- [ ] Network access is limited to necessary domains
- [ ] No unnecessary system-level permissions requested

### Input Handling
- [ ] All inputs are validated for size and format
- [ ] JSON parsing includes proper error handling
- [ ] No unsafe operations on untrusted input
- [ ] Path traversal attacks prevented

### Output Generation
- [ ] Output size is limited appropriately
- [ ] No sensitive information leaked in outputs
- [ ] HTML/script content properly escaped
- [ ] JSON structure validated before serialization

### Resource Management
- [ ] Memory usage is bounded and efficient
- [ ] File operations are limited and cleaned up
- [ ] Network requests include timeouts
- [ ] No infinite loops or excessive recursion

### Error Handling
- [ ] Errors don't leak sensitive system information
- [ ] Graceful degradation on security violations
- [ ] Proper logging without sensitive data exposure
- [ ] Recovery mechanisms for partial failures

### Dependencies
- [ ] Minimal dependency footprint
- [ ] All dependencies are audited and up-to-date
- [ ] No dependencies with known vulnerabilities
- [ ] License compatibility verified

### Testing
- [ ] Security-focused unit tests included
- [ ] Integration tests with security scenarios
- [ ] Fuzzing tests for input validation
- [ ] Performance tests under resource constraints
```

### Secure Coding Examples

```rust
// ✅ GOOD: Secure input validation
#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    // Validate input size
    if file_data.len() > MAX_INPUT_SIZE {
        return create_error_response("Input too large");
    }
    
    // Safe JSON parsing
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => {
            log_error!("JSON parse error: {}", e);
            return create_error_response("Invalid input format");
        }
    };
    
    // Validate file path
    if input.file_path.contains("..") || input.file_path.starts_with('/') {
        return create_error_response("Invalid file path");
    }
    
    // Validate content length
    if input.content.len() > MAX_CONTENT_SIZE {
        return create_error_response("Content too large");
    }
    
    // Safe processing with bounds checking
    let issues = analyze_content_safely(&input);
    
    // Sanitize output
    let result = PluginAnalysisResult {
        plugin_name: "secure-analyzer".to_string(),
        issues: sanitize_issues(issues),
        metadata: create_safe_metadata(&input),
    };
    
    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_response("Serialization failed")
    })
}

// ❌ BAD: Unsafe input handling
#[export_name = "analyze_file"]  
pub fn analyze_file_unsafe(file_data: &[u8]) -> Vec<u8> {
    // No input validation!
    let input: PluginFileInput = serde_json::from_slice(file_data).unwrap(); // Panic on invalid input!
    
    // No path validation - vulnerable to path traversal!
    let content = std::fs::read_to_string(&input.file_path).unwrap(); // System file access!
    
    // No bounds checking - vulnerable to DoS!
    let mut issues = Vec::new();
    for line in content.lines() {
        // Potentially infinite loop
        loop {
            issues.push(create_issue(line));
        }
    }
    
    // No output sanitization
    serde_json::to_vec(&issues).unwrap() // Panic on serialization error!
}
```

This comprehensive security model ensures that Uveddi's plugin system provides safe extensibility while maintaining strong security guarantees for the host system and user data.