# Uveddi CLI

Uveddi is a commercial-grade architectural analysis CLI that combines static code analysis with AI-assisted insights to help engineering teams audit, modernize, and govern large codebases.

**Current Version**: `1.0.0` &nbsp;|&nbsp; **Availability**: Commercial release

**Pricing**: $9 per week or $300 per year (billed annually)

## What’s included

- **Multi-language coverage** – Deep Rust, Python, JavaScript, and TypeScript analysis powered by Tree-sitter
- **Anti-pattern detection** – Catch God Objects, dead code, circular dependencies, large classes, and dozens of architectural smells
- **Security scanning** – Optional security mode with SARIF export for CI/CD pipelines
- **AI insights (optional)** – Integrate with an on-prem Ollama instance to generate remediation guidance and summaries
- **Comprehensive reporting** – Markdown, JSON, and HTML outputs with interactive diagrams, dark/light themes, severity scoring, and debt metrics
- **CI automation** – Threshold-based quality gates for pull requests and release builds
- **Enterprise-ready reliability** – Persistent SQLite storage, deterministic caching, detailed progress reporting, and structured logging
- **Complete documentation** – [CLI Reference](docs/CLI_REFERENCE.md) and [Troubleshooting Guide](docs/TROUBLESHOOTING.md) for self-service success

> Web UI and TUI experiences have been removed for the CLI-only release to streamline support and provide a focused commercial product.

## Quick start

```bash
# Clone the source (subscription required for commercial use)
git clone https://github.com/botzrDev/uveddi.git
cd uveddi

# Build the CLI with the recommended commercial feature set
cargo build --release --features cli-standard

# Run your first analysis
./target/release/uveddi analyze /path/to/project \
	--output-format markdown \
	--output reports/audit.md

# Generate security-focused SARIF output (optional)
./target/release/uveddi analyze /path/to/project \
	--security --export-sarif \
	--output-format json

# Enable AI guidance when Ollama is available
OLLAMA_API_URL=http://localhost:11434 \
./target/release/uveddi analyze /path/to/project \
	--enable-ai --ollama-model deepseek-coder:6.7b
```

## Installation & subscription

1. **Activate your subscription** – Contact sales (sales@uveddi.com) to provision a weekly or annual license. *Assumed contact channel; replace with your organization’s preferred sales workflow.*
2. **Install Rust** – Rust 1.70+ is required (`rustup` recommended).
3. **Build from source or request binaries** – Subscribers may request signed binaries, or build from source using `cargo build --release --features cli-standard`.
4. **Apply license key (optional)** – If your deployment uses license keys, add `UVEDDI_LICENSE_KEY=<key>` to your environment. *License enforcement tooling ships separately.*

Optional components:
- **Ollama** for on-device AI explanations
- **Rendering service** if you intend to produce image-based diagrams instead of Mermaid output

## CLI highlights

- `uveddi analyze` – Full codebase audits with configurable detectors, memory profiles, and timeout handling
- `uveddi config` – Validate and tune analysis defaults for your organization
- `uveddi doctor` – Environment diagnostics and auto-fix routines for parsers, AI connectivity, and system resources
- `uveddi init` – Project bootstrapper that generates tuned configuration files for new repositories
- `uveddi hooks` – Git hook automation to enforce analysis before commits or pushes
- `uveddi ci` – CI-centric quality gates with customizable debt and severity thresholds

Run `uveddi --help` or `uveddi <command> --help` for detailed usage information. See the [CLI Reference](docs/CLI_REFERENCE.md) for comprehensive command documentation and usage examples.

## Recommended feature flags

- `cli-standard` *(default)* – Balanced profile with engine integrations, caching, and tree-sitter language packs
- `cli-ai` – Adds AI workflows on top of `cli-standard`
- `cli-plugins` – Enables WebAssembly plugin execution when your audit requires bespoke detectors

**Security scanning** is always available via the `--security` CLI flag (no build feature required).

> **Note (A3 cleanup):** The `security` feature flag has been removed as it was empty/deprecated. Security detectors are now part of the standard CLI build.

Legacy web/TUI flags remain in the manifest for backwards compatibility but are unsupported in the commercial build.

## Support & success

Commercial subscribers receive:
- Priority email support: support@uveddi.com *(placeholder – update with production address)*
- Onboarding guidance and environment validation scripts
- Early access to detector updates and private roadmap briefings
- Escalation path for critical audit findings

If you encounter an issue, first consult the [Troubleshooting Guide](docs/TROUBLESHOOTING.md) for common solutions, then open a ticket via the subscriber portal or email support. Public GitHub issues remain available for transparency but may have slower response times for non-subscribers.

## Security & privacy

Uveddi follows industry-standard security practices and privacy-first design principles:

- **Security Policy:** See [SECURITY_POLICY.md](SECURITY_POLICY.md) for supported versions and vulnerability reporting
- **Report Vulnerabilities:** Contact security@uveddi.com or use [GitHub Security Advisories](https://github.com/botzrDev/uveddi/security/advisories)
- **PGP Encryption:** For sensitive reports, use our [PGP public key](pgp/security@uveddi.com.asc) (Fingerprint: `D592 AD0C 4CCC 5125 326F 7A12 B68A 7402 9415 6723`)
- **Privacy First:** No telemetry or data collection by default; all analysis runs locally unless AI features explicitly enabled
- **Compliance:** GDPR and CCPA compliant; see our [Privacy Policy](PRIVACY.md)

For detailed security contact information and responsible disclosure guidelines, see [SECURITY_CONTACT.md](SECURITY_CONTACT.md).

## Roadmap snapshot

- Q4 2025: Expanded security rule packs, dependency hygiene scoring, SaaS-based reporting backend (opt-in)
- Q1 2026: Managed policy library, organization-wide baselines, enhanced plugin marketplace

## License & attribution

The source code is distributed under the MIT license (see [LICENSE](LICENSE)). Commercial use of the official Uveddi CLI distribution requires an active subscription agreement.

### Third-party licenses

Uveddi incorporates open source components from the Rust ecosystem and other projects. See:
- [THIRD_PARTY_LICENSES.txt](THIRD_PARTY_LICENSES.txt) – Complete license texts for all 441 dependencies
- [NOTICE](NOTICE) – Third-party acknowledgements and trademarks

These files are included in all distribution packages and document our compliance with open source license obligations.

---

© 2025 Uveddi. Crafted for engineering teams that need trustworthy code intelligence.
