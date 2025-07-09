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
