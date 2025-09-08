# Logging Configuration Guide

## Overview

Uveddi uses the `tracing` crate for structured, consistent logging throughout the application. This provides better observability, debugging capabilities, and production monitoring compared to simple `println!` statements.

## Quick Start

### Setting Log Levels

Use the `RUST_LOG` environment variable to control log verbosity:

```bash
# Basic log levels
RUST_LOG=error uveddi analyze ./src    # Only errors
RUST_LOG=warn uveddi analyze ./src     # Warnings and errors
RUST_LOG=info uveddi analyze ./src     # Info, warnings, and errors (default)
RUST_LOG=debug uveddi analyze ./src    # Debug messages and above
RUST_LOG=trace uveddi analyze ./src    # Everything including trace

# Module-specific logging
RUST_LOG=uveddi=debug,tower_http=warn uveddi serve

# Target specific components
RUST_LOG=uveddi::analysis=debug,uveddi::api=trace uveddi analyze ./src
```

### Log Output Formats

Control the output format using the `LOG_FORMAT` environment variable:

```bash
# Compact format (default) - human-readable
LOG_FORMAT=compact uveddi analyze ./src

# JSON format - for log aggregation systems
LOG_FORMAT=json uveddi analyze ./src
```

## Configuration Options

### Environment Variables

| Variable | Description | Default | Options |
|----------|-------------|---------|---------|
| `RUST_LOG` | Log level filter | `info` | `error`, `warn`, `info`, `debug`, `trace` |
| `LOG_FORMAT` | Output format | `compact` | `compact`, `json` |
| `LOG_FILE` | Log file path (when supported) | None | Any valid file path |

### Programmatic Configuration

For development or testing, you can configure logging programmatically:

```rust
use uveddi::core::logging::unified::{LoggingConfig, LogFormat, LogOutput};

// Custom configuration
let config = LoggingConfig {
    level: "debug".to_string(),
    format: LogFormat::Json,
    output: LogOutput::Both("/var/log/uveddi.log".to_string()),
    include_timestamps: true,
    include_thread_info: true,
    include_location: true,
};

uveddi::core::logging::unified::init_with_config(config)?;
```

## Log Levels Guide

### Error Level
Use for recoverable errors that indicate something went wrong:
```rust
error!("Failed to parse configuration file: {}", e);
error!(error = ?e, file = %path, "Configuration parsing failed");
```

### Warn Level
Use for potentially problematic situations that don't prevent operation:
```rust
warn!("Deprecated feature used: {}", feature_name);
warn!(threshold = %limit, current = %value, "Approaching resource limit");
```

### Info Level
Use for significant events in normal operation:
```rust
info!("Analysis completed: {} files processed", file_count);
info!(duration_ms = %elapsed, "Operation completed");
```

### Debug Level
Use for detailed information useful during development:
```rust
debug!("Processing file: {}", file_path);
debug!(ast_nodes = %node_count, "AST parsing complete");
```

### Trace Level
Use for very detailed debugging information:
```rust
trace!("Entering function: analyze_module");
trace!(params = ?config, "Function parameters");
```

## Structured Logging

### Adding Context with Spans

Use spans to add context to a series of related log events:

```rust
use tracing::{info_span, info};

let span = info_span!("analyze", path = %file_path, request_id = %id);
let _enter = span.enter();

info!("Starting analysis");
// All logs within this scope include the span context
```

### Request Correlation

For tracking requests across the system:

```rust
use uveddi::core::logging::unified::generate_request_id;

let request_id = generate_request_id();
info!(request_id = %request_id, "Processing new request");
```

### Error Chains

Log complete error chains for better debugging:

```rust
use uveddi::log_error_with_context;

match operation() {
    Err(e) => {
        log_error_with_context!(e, "Failed to complete operation");
    }
    Ok(result) => {
        info!("Operation successful");
    }
}
```

## Production Configuration

### Recommended Settings

For production deployments:

```bash
# Production with file output
RUST_LOG=uveddi=info,tower_http=warn \
LOG_FORMAT=json \
uveddi serve --port 8888

# With external log aggregation
RUST_LOG=uveddi=info \
LOG_FORMAT=json \
uveddi serve | tee -a /var/log/uveddi/app.log
```

### Log Rotation

When writing to files, use external tools like `logrotate`:

```conf
# /etc/logrotate.d/uveddi
/var/log/uveddi/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 uveddi uveddi
    sharedscripts
    postrotate
        systemctl reload uveddi || true
    endscript
}
```

### Integration with Monitoring Systems

#### Prometheus Integration

Uveddi exposes metrics at `/metrics` when the API server is running:

```bash
# Start with metrics enabled
uveddi serve --port 8888

# Metrics available at
curl http://localhost:8888/metrics
```

#### Log Aggregation (ELK Stack)

For Elasticsearch/Logstash/Kibana integration:

```bash
# Output JSON logs for Logstash
LOG_FORMAT=json uveddi serve 2>&1 | \
  logstash -f /etc/logstash/conf.d/uveddi.conf
```

Example Logstash configuration:

```ruby
input {
  stdin {
    codec => json
  }
}

filter {
  if [level] == "ERROR" {
    mutate {
      add_tag => [ "error", "alert" ]
    }
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "uveddi-%{+YYYY.MM.dd}"
  }
}
```

## Development Tips

### Debugging Specific Modules

```bash
# Debug only the analysis module
RUST_LOG=uveddi::analysis=debug uveddi analyze ./src

# Trace API calls
RUST_LOG=uveddi::api=trace uveddi serve

# Multiple module debugging
RUST_LOG=uveddi::analysis=debug,uveddi::report=trace uveddi analyze ./src
```

### Performance Profiling

Enable timing logs:

```rust
use uveddi::log_timing;
use std::time::Instant;

let start = Instant::now();
perform_operation();
log_timing!("operation_name", start.elapsed());
```

### Testing with Logs

During tests, capture logs for assertions:

```rust
#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_with_logging() {
        info!("Test message");
        // Logs are captured and can be asserted
        assert!(logs_contain("Test message"));
    }
}
```

## Troubleshooting

### Common Issues

1. **No logs appearing**
   - Check `RUST_LOG` environment variable
   - Ensure logging is initialized early in `main()`
   - Verify no conflicting logging initialization

2. **Too many logs**
   - Adjust log level: `RUST_LOG=warn`
   - Filter specific modules: `RUST_LOG=uveddi=info,tokio=error`

3. **Log file not created**
   - Check file permissions
   - Ensure directory exists
   - Verify disk space

4. **Performance impact**
   - Use appropriate log levels in production
   - Consider async logging for high-throughput scenarios
   - Use structured logging for efficient filtering

### Debug Logging Configuration

To debug the logging system itself:

```bash
RUST_LOG=tracing=debug uveddi analyze ./src
```

## Migration from println!/eprintln!

If you're migrating from `println!`/`eprintln!` statements:

1. **Run the migration script**:
   ```bash
   ./scripts/simple_migrate_logging.sh
   ```

2. **Review remaining println! statements** - they may be intentional user output

3. **Update imports**:
   ```rust
   // Add at the top of files
   use tracing::{info, warn, error, debug};
   ```

4. **Replace patterns**:
   - `println!` → `info!` for general output
   - `eprintln!` → `error!` for errors
   - Add context with structured fields

## Best Practices

1. **Use structured logging**: Include relevant context as fields rather than formatting into strings
2. **Be consistent**: Use the same field names across the codebase
3. **Avoid sensitive data**: Never log passwords, tokens, or PII
4. **Use appropriate levels**: Don't use `error!` for warnings or `debug!` in production code paths
5. **Add request IDs**: For distributed tracing and correlation
6. **Include error chains**: Log the full error context for debugging
7. **Performance considerations**: Avoid expensive operations in log statements that may not be printed

## Examples

### Basic Usage

```rust
use tracing::{info, error, warn, debug};

// Simple messages
info!("Starting analysis");
warn!("Cache miss, regenerating");
error!("Failed to open file");
debug!("Parsing AST node");

// With structured fields
info!(
    files_analyzed = 42,
    duration_ms = 1500,
    "Analysis completed"
);

// With error context
if let Err(e) = process_file(&path) {
    error!(
        error = ?e,
        path = %path.display(),
        "Failed to process file"
    );
}
```

### Advanced Usage

```rust
use tracing::{instrument, Span, Level};
use uveddi::log_with_context;

#[instrument(level = "debug", skip(config))]
async fn analyze_codebase(
    path: &Path,
    config: &Config,
) -> Result<Report> {
    let request_id = generate_request_id();
    
    log_with_context!(
        Level::INFO,
        request_id,
        "Starting codebase analysis",
        path = %path.display(),
        config_profile = %config.profile
    );
    
    // Analysis logic here
    
    Ok(report)
}
```

## Additional Resources

- [Tracing Documentation](https://docs.rs/tracing)
- [Tracing Subscriber Guide](https://docs.rs/tracing-subscriber)
- [Structured Logging Best Practices](https://www.honeycomb.io/blog/structured-logging-best-practices)
- [OpenTelemetry Integration](https://opentelemetry.io/docs/instrumentation/rust/)