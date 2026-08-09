# Uveddi v1.0.0 — First Stable Release 🎉

**Release Date:** August 9, 2026
**Version:** 1.0.0
**Codename:** Foundation

---

## 🌟 Highlights

Uveddi 1.0.0 is the first stable release of Uveddi, a Rust-based architectural
analysis CLI that combines static code analysis with optional AI-assisted
insights to help engineering teams audit, modernize, and govern codebases.

This release promotes the CLI-focused product refined through the 0.0.x alpha
series to stable status. The focus of 1.0.0 is reliability of the core
analysis workflow: analyze a codebase, detect anti-patterns and security
issues, and produce actionable reports — locally or in CI.

### What's in the box

- **Multi-language analysis** — Rust, Python, JavaScript, and TypeScript via
  Tree-sitter in the default build; additional languages available through
  feature flags and license tiers
- **Anti-pattern detection** — god objects, large classes, dead code, code
  duplication, tight coupling, magic values, and related architectural smells
- **Security scanning** — built-in security mode with SARIF export for CI/CD
- **Comprehensive reporting** — Markdown, JSON, and HTML outputs with severity
  scoring and technical-debt metrics
- **Reliable storage** — persistent SQLite database with migration support
- **Tiered licensing** — core detectors and security scanning are free;
  advanced cyclic-dependency detection and extended language packs are
  Pro/Team-tier
- **Experimental extras (opt-in)** — WASM component-model plugin system and
  local AI insights via Ollama

---

## 📦 Installation

### From source (recommended)

```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
./deploy/install.sh
```

The installer builds the release binary, installs it to `~/.local/bin/uveddi`,
adds it to your PATH, and verifies the installation.

### With cargo

```bash
cargo install --git https://github.com/botzrDev/uveddi.git
```

### Pre-built binaries

Binaries for Linux (x86_64 gnu/musl, aarch64), macOS (x86_64, aarch64), and
Windows (x86_64) are published on the
[releases page](https://github.com/botzrDev/uveddi/releases) for each tagged
release.

---

## 🚀 Quick Start

```bash
# Initialize configuration for a project
uveddi init

# Analyze a codebase and produce a Markdown report
uveddi analyze /path/to/project --output-format markdown

# Security-focused analysis with SARIF export for CI
uveddi analyze /path/to/project --security --export-sarif --output-format json

# Health diagnostics
uveddi doctor

# License activation and status
uveddi license status
```

See the [CLI Reference](CLI_REFERENCE.md) for the full command surface:
`analyze`, `init`, `config`, `doctor`, `ci`, `hooks`, `migrate`, `license`,
`help`, and `plugin` (with the `wasm-plugins` feature).

---

## 🧩 Feature Tiers

| Tier | Languages | Detectors | Notes |
|------|-----------|-----------|-------|
| **Free** | JavaScript, TypeScript (default build also parses Rust & Python) | Core anti-pattern detectors + security scanning | No license required |
| **Pro** | 10 languages | All detectors incl. cyclic-dependency analysis | Single seat |
| **Team** | 14 languages | All detectors | 5 seats |
| **Enterprise** | 16+ languages | All + custom | Custom terms |

Activate with `uveddi license activate <key>`; activation is offline/local.

---

## ⚠️ Experimental Features

These ship in 1.0.0 behind opt-in feature flags and are not covered by
stability expectations:

- **WASM plugins** (`--features cli-plugins`) — WebAssembly component-model
  host with a WASI plugin template. The host is functional; the plugin
  ecosystem is young. See [plugin-development](plugin-development/).
- **AI insights** (`--features cli-ai`) — remediation guidance via a locally
  running [Ollama](https://ollama.com) instance. Quality depends on the local
  model.

---

## 🔎 Known Limitations

- TypeScript analysis handles common constructs; very complex type-level code
  may parse partially.
- Detector thresholds are tuned for typical codebases; calibration options
  are documented in the [configuration reference](CLI_REFERENCE.md).
- See [known-issues.md](known-issues.md) for the current list.

---

## 🔮 What's Next

- **1.0.x** — bug-fix patches from early stable feedback
- **1.1** — plugin ecosystem growth and detector calibration improvements
- **1.2+** — additional language tiers, IDE integrations, richer CI templates

The full release plan and gate results for 1.0.0 live in
[release-planning/v1.0.0-release-plan.md](release-planning/v1.0.0-release-plan.md).

---

## 📄 License

Uveddi is released under **CC-BY-NC-SA-4.0** (non-commercial). Commercial use
requires a paid tier — see `uveddi license --help` or contact
contact@botzr.com.
