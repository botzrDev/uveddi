# Uveddi CLI v1.0.0 – Commercial Launch

Release date: 2025-10-08

## Overview

Uveddi 1.0.0 marks the first commercial release of the CLI-only product line. The focus of this milestone is reliability, deterministic analysis output, and a supportable experience for paid subscribers. All interactive web and TUI surfaces remain removed to streamline the footprint and simplify maintenance.

## Subscription plans

- **Weekly** subscription: $9 (billed every 7 days)
- **Annual** subscription: $300 (billed once per year)

Subscribers receive priority support, onboarding assistance, and early access to detector improvements. Licensing enforcement (if applicable) is delivered as a separate integration package.

## Highlights

- **Battle-tested analyze pipeline** – Persistent SQLite-backed orchestration, fault-tolerant timeout handling, and graceful degradation flows.
- **Security-first workflow** – SARIF export, OWASP category filtering, taint analysis toggles, and configurable severity thresholds.
- **AI-assisted analysis** – Optional Ollama integration for remediation guidance and architectural summaries.
- **Git automation** – Updated hooks command with install, list, test, and uninstall modes tuned for subscription deployments.
- **Improved messaging** – README, contributing guide, and legacy documents refreshed to reflect commercial positioning.

## Breaking changes

- Public community release artifacts, feature flags (`web`, `tui`, `community`), and Discord support references are deprecated. The crates remain but are unsupported.
- Contribution process now limited to approved partners; unsolicited pull requests may be closed without review.

## Upgrade guidance

1. Update your local checkout to the `release/cli-only` branch.
2. Rebuild using `cargo build --release --features cli-standard`.
3. Review `README.md` for updated quick-start instructions and support channels.
4. If you previously relied on web/TUI components, contact sales@uveddi.com to discuss roadmap alignment or custom support options.

## Known issues

- Diagram image rendering still requires the optional rendering service. Mermaid-only mode remains the default.
- AI features require a self-hosted Ollama deployment; SaaS-hosted AI is not bundled with the subscription yet.
- License enforcement tooling (if part of your contract) must be integrated separately.

## Looking ahead

The next milestones will expand security detections, add policy management, and improve insights around dependency health and modernization readiness. Subscribers will receive roadmap updates via the customer portal.
