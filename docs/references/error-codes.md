# Error Code Reference

## Analysis Errors (1xxx)

| Code | Error | Description | Solution |
|------|-------|-------------|----------|
| 1001 | AnalysisTimeout | Analysis exceeded time limit | Increase timeout with `--timeout` flag |
| 1002 | MemoryLimitExceeded | Analysis used too much memory | Increase memory limit or reduce scope |
| 1003 | UnsupportedFileType | File type not supported | Exclude file or use `--force` flag |
| 1004 | ParseError | Could not parse source code | Check file syntax |

## Configuration Errors (2xxx)

| Code | Error | Description | Solution |
|------|-------|-------------|----------|
| 2001 | InvalidConfig | Configuration file error | Validate config with `uveddi validate-config` |
| 2002 | MissingProvider | No AI provider configured | Set provider in config or CLI |
| 2003 | InvalidRule | Rule configuration invalid | Check rule syntax in config |

## AI Integration Errors (3xxx)

| Code | Error | Description | Solution |
|------|-------|-------------|----------|
| 3001 | ProviderUnavailable | AI provider not responding | Check provider status/credentials |
| 3002 | ContextWindowExceeded | Prompt too large for model | Reduce context or use larger model |
| 3003 | InvalidResponse | AI returned malformed response | Retry or report to provider |

## Plugin Errors (4xxx)

| Code | Error | Description | Solution |
|------|-------|-------------|----------|
| 4001 | PluginLoadFailed | Could not load plugin | Check WASM compatibility |
| 4002 | PluginRuntimeError | Plugin crashed during execution | Check plugin logs |
| 4003 | InvalidPluginAPI | Plugin uses unsupported API | Update plugin version |

## Common Solutions

1. **Enable debug logging**:
   ```bash
   RUST_LOG=debug uveddi analyze ./project
   ```

2. **Check known issues**:
   ```bash
   uveddi known-issues
   ```

3. **Generate debug report**:
   ```bash
   uveddi debug-report > debug.log
   ```

4. **Update to latest version**:
   ```bash
   cargo install --force uveddi
