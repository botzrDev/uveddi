# Uveddi Developer Guide

## Structured Logging Implementation

### Overview
The rendering service now uses structured logging with the following components:
- **Pino** logger for high-performance JSON logging
- Standardized severity levels and error categories
- Machine-readable log format for observability tools

### Log Severity Levels
- `fatal`: Critical errors causing service failure
- `error`: Runtime errors that affect functionality
- `warn`: Potentially problematic situations
- `info`: General operational messages
- `debug`: Detailed debugging information
- `trace`: Very detailed tracing

### Error Categories
- `RenderingFailure`: Errors during diagram rendering
- `CacheFailure`: Cache-related errors
- `WorkerPoolExhausted`: No available workers
- `WorkerPoolInitializationFailure`: Worker pool startup errors
- `BatchRenderingFailure`: Batch processing errors
- `UnhandledError`: Unexpected errors

### Example Log Formats
```json
{
  "level": "error",
  "time": "2025-07-09T12:34:56Z",
  "msg": "Rendering failed",
  "error": "SVG export failed: missing font resource",
  "severity": "Error",
  "category": "RenderingFailure",
  "stack": "...",
  "request_id": "abc123"
}
```

### Configuration
Configure logging via environment variables:
```bash
LOG_LEVEL=debug  # Set log level (default: info)
LOG_PRETTY=true  # Enable pretty-printing for development
```

### Best Practices
1. Always include:
   - Error message and stack trace for errors
   - Appropriate severity level
   - Relevant category
   - Contextual metadata

2. Use structured fields instead of string concatenation:
   ```javascript
   // Good
   logger.info({ renderTimeMs: 150 }, 'Rendering completed');

   // Avoid
   logger.info(`Rendering completed in ${150}ms`);
   ```

3. For errors, always include the error object:
   ```javascript
   try {
     // ...
   } catch (error) {
     logger.error({
       msg: 'Operation failed',
       error: error.message,
       stack: error.stack,
       severity: 'Error',
       category: 'OperationFailure'
     });
   }
   ```

4. Use appropriate severity levels:
   - `error` for operational failures
   - `warn` for recoverable issues
   - `info` for normal operations
   - `debug` for troubleshooting

### Adding New Logs
When adding new log statements:
1. Determine the appropriate severity level
2. Include all relevant context as structured fields
3. For errors, specify a clear category
4. Keep messages concise but descriptive

## Architectural Patterns

### Preventing Circular Dependencies

Uveddi uses a **shared types pattern** to prevent circular dependencies between modules. This is critical for maintaining clean architecture and enabling successful compilation.

#### Problem
Circular dependencies occur when two modules depend on each other directly or indirectly:

```rust
// ❌ Circular dependency
// src/api/mod.rs
use crate::service_orchestration::ServiceOrchestrator;

// src/service_orchestration/mod.rs  
use crate::api::{CombinedApiServer, RestApiConfig};
```

#### Solution: Shared Types Module

Create a dedicated types module that both dependent modules can use:

```rust
// ✅ src/api/types.rs - Shared types module
use std::path::PathBuf;
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct RestApiConfig {
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
    pub spa_assets_path: Option<PathBuf>,
    pub reports_storage_path: PathBuf,
    // ... other fields
}

pub trait ApiServer {
    fn start(
        self,
        database: Arc<Database>,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send;

    fn start_with_readiness(
        self,
        database: Arc<Database>,
        ready_tx: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send;
}
```

#### Implementation Pattern

1. **Extract shared types** into `src/api/types.rs`
2. **Update module exports** in `src/api/mod.rs`:
   ```rust
   pub mod types;
   pub use types::{ApiServer, RestApiConfig};
   ```
3. **Use shared types** in both modules:
   ```rust
   // src/api/rest.rs
   use crate::api::types::{ApiServer, RestApiConfig};

   // src/service_orchestration/orchestrator.rs
   use crate::api::{CombinedApiServer, RestApiConfig};
   ```

### Service Orchestration Patterns

#### Readiness Signaling
Use `tokio::sync::oneshot` channels for proper service readiness notification:

```rust
pub async fn start_with_readiness(
    self,
    database: Arc<Database>,
    ready_tx: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => {
            // Signal readiness immediately after successful bind
            let _ = ready_tx.send(Ok(()));
            listener
        }
        Err(e) => {
            let _ = ready_tx.send(Err(Box::new(e)));
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AddrInUse, 
                "Failed to bind to port"
            )));
        }
    };
    
    // Start serving after readiness notification
    axum::serve(listener, app).await?;
    Ok(())
}
```

#### Health Check Patterns
Implement exponential backoff with detailed error reporting:

```rust
async fn verify_services_health(&self, config: &OrchestratorConfig) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
        
    let mut retries = 0;
    const MAX_RETRIES: u32 = 15;
    let mut last_errors = Vec::new();

    while retries < MAX_RETRIES {
        // Attempt health checks...
        
        if all_healthy {
            return Ok(());
        }

        retries += 1;
        last_errors = errors; // Store errors from this attempt
        
        // Exponential backoff: 500ms → 1s → 2s → 2s...
        let wait_time = std::cmp::min(500 * (1 << std::cmp::min(retries, 2)), 2000);
        sleep(Duration::from_millis(wait_time as u64)).await;
    }

    Err(eyre!("Services failed to become healthy after {} retries. Last errors: {:?}", 
             MAX_RETRIES, last_errors.join(", ")))
}
```

### Key Principles

1. **Separation of Concerns**: Each module should have a single, well-defined responsibility
2. **Dependency Injection**: Use trait-based abstractions for testability
3. **Error Handling**: Provide detailed error context with proper error types
4. **Async Patterns**: Use proper readiness signaling and graceful shutdown
5. **Resource Management**: Always clean up background processes and handles
