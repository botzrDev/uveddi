# Dead Code Detection

The Dead Code Detection feature in Uveddi identifies potentially unused functions, variables, classes, and other symbols across multiple programming languages. This feature has been fully integrated into the main analysis engine and provides configurable detection with confidence scoring.

## Features

- **Multi-language Support**: Rust, Python, and JavaScript
- **Confidence Scoring**: Reduces false positives with intelligent scoring
- **Configurable Thresholds**: Customize detection sensitivity
- **Library Mode**: Special handling for exported symbols in libraries
- **Pattern-based Filtering**: Ignore test files and keep essential symbols alive

## Configuration

### CLI Options

```bash
# Basic analysis with dead code detection
uveddi analyze ./src

# Custom confidence threshold (0.0 to 1.0)
uveddi analyze ./src --dead-code-confidence 0.8

# Enable library mode for exported symbols
uveddi analyze ./src --dead-code-library-mode

# Ignore specific patterns
uveddi analyze ./src --dead-code-ignore-patterns "test,spec,mock,generated"

# Keep specific symbols alive
uveddi analyze ./src --dead-code-keep-alive "main,init,setup,teardown"
```

### Environment Variables

```bash
# Set confidence threshold
export DEAD_CODE_CONFIDENCE_THRESHOLD=0.7

# Enable library mode
export DEAD_CODE_LIBRARY_MODE=true

# Set ignore patterns (comma-separated)
export DEAD_CODE_IGNORE_PATTERNS="test,spec,mock"

# Set keep-alive patterns (comma-separated)
export DEAD_CODE_KEEP_ALIVE_PATTERNS="main,init,setup"
```

### Configuration File

Add to your `config.toml`:

```toml
[dead_code]
confidence_threshold = 0.7
library_mode = false
ignore_patterns = ["test", "spec", "mock", "generated"]
keep_alive_patterns = ["main", "init", "setup", "teardown"]
```

## Detection Strategy

The dead code detector uses a simplified mark-and-sweep algorithm:

1. **Symbol Collection**: Extract all function/variable definitions using Tree-sitter queries
2. **Usage Analysis**: Find all references and calls to these symbols
3. **Entry Point Detection**: Identify main functions, exports, and public APIs
4. **Reachability Analysis**: Mark symbols as live if they're reachable from entry points
5. **Dead Code Reporting**: Report symbols that remain unmarked as potentially dead

## Confidence Scoring

The detector assigns confidence scores to reduce false positives:

- **High (0.8-1.0)**: Private/internal symbols with no references
- **Medium (0.5-0.7)**: Exported symbols in applications with no apparent usage
- **Low (0.2-0.4)**: Symbols in files with dynamic features (eval, decorators, etc.)

## Language-Specific Behavior

### Rust
- Detects unused functions, structs, enums, and constants
- Considers `pub` visibility for export detection
- Higher confidence for private symbols

### Python
- Detects unused functions, classes, and variables
- Treats symbols starting with `_` as private
- Lower confidence for special methods (`__init__`, etc.)
- Reduced confidence in test files

### JavaScript
- Detects unused functions, classes, and variables
- Considers `export` and `module.exports` for export detection
- Lower confidence for React components (capitalized names)
- Reduced confidence in test files

## Integration

The Dead Code Detector is automatically included in the analysis engine. No additional setup is required beyond configuration.

## Example Output

```
Found 3 potential dead code issues in src/utils.rs:

High Severity - Line 15:
Potentially dead code: function 'unused_helper' is not used in this file (confidence: 90.0%)
Code snippet: fn unused_helper() { ... }

Medium Severity - Line 42:
Potentially dead code: struct 'UnusedStruct' is not used in this file (confidence: 60.0%)
Code snippet: pub struct UnusedStruct { ... }
```

## Best Practices

1. **Start with High Confidence**: Begin with a threshold of 0.8 to minimize false positives
2. **Use Library Mode**: Enable library mode when analyzing library code to avoid flagging public APIs
3. **Configure Ignore Patterns**: Add patterns for test files, generated code, and other special cases
4. **Review Results**: Always review detected dead code before removal, especially for exported symbols
5. **Iterative Cleanup**: Remove dead code in small batches and test thoroughly

## Limitations

- **Single-file Analysis**: Current implementation analyzes files individually, which may miss cross-file usage
- **Dynamic Usage**: Cannot detect usage through reflection, eval, or other dynamic mechanisms
- **External Dependencies**: May not detect usage by external packages or tools

## Future Enhancements

- Cross-file reachability analysis
- Integration with build systems to detect entry points
- Support for more languages (C++, Java, Go)
- Advanced pattern matching for dynamic usage detection