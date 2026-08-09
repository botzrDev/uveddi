# Known Issues and Limitations

> **Version**: 1.0.0
> **Last Updated**: August 2026
> **Status**: Stable Release

This document lists known issues and limitations in the current stable
release. For the full release scope and gate results, see
[release-planning/v1.0.0-release-plan.md](release-planning/v1.0.0-release-plan.md).

> Historical note: earlier revisions of this document described the
> pre-CLI-refocus architecture (web dashboard, REST API server, TUI,
> Prometheus monitoring). Those components were removed before 1.0.0 and the
> issues that referred to them no longer apply.

---

## Limitations

### TypeScript analysis handles common constructs

**Impact**: Medium

TypeScript parsing and detection work for typical application code. Very
complex type-level code may parse partially:

- Deeply nested conditional and mapped types
- Unusual decorator patterns

**Workaround**: None needed for most codebases; issues surface as reduced
detection coverage, not analysis failures.

---

### WASM plugin system is experimental

**Impact**: Low (opt-in feature)

The plugin host, loader, and lifecycle manager are functional, but the
plugin ecosystem is young (template plus example plugins). Plugins require
building with `--features cli-plugins`.

- Plugin API may change in a future minor release
- Resource limits and sandbox policies are conservative by default

---

### AI insights require a local Ollama instance

**Impact**: Low (opt-in feature)

The `cli-ai` feature integrates with a locally running
[Ollama](https://ollama.com) server. Output quality depends on the local
model; suggestions should be reviewed before acting on them.

---

### Detector calibration

**Impact**: Medium

Detector thresholds (god object, long method, large class) are tuned for
typical codebases and may need per-project calibration via configuration.
Calibration data lives under `calibration/` and thresholds are configurable
in `uveddi.toml`.

---

### Large codebases

**Impact**: Medium

Very large repositories (>10k files) can take significant time and memory on
first analysis. Incremental analysis substantially reduces re-analysis cost.

**Workarounds**:

- Analyze subtrees (`uveddi analyze ./src`)
- Use `.gitignore`-style exclusions in configuration
- Rely on incremental re-analysis after the first full run

---

## Reporting Issues

- **Bugs**: [GitHub Issues](https://github.com/botzrDev/uveddi/issues)
- **Security**: see [SECURITY_CONTACT.md](../SECURITY_CONTACT.md)
