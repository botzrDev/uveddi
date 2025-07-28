# CLI Commands Reference

This section documents all available CLI commands for Uveddi.

## Basic Usage

```bash
uveddi analyze <path> [options]
uveddi known-issues
uveddi debug-report
```

## Commands

- `analyze` — Analyze a codebase for architectural issues
  - `--lang <language>`: Specify language (rust, python, javascript)
  - `--output <file>`: Save report to file
  - `--output-format <format>`: Output as markdown, json, mermaid, etc.
  - `--detectors <list>`: Comma-separated list of detectors to run
  - `--jobs <n>`: Limit parallel jobs
  - `--ai-provider <provider>`: Select AI backend
  - `--timeout <seconds>`: Set analysis timeout
  - `--max-file-size <size>`: Exclude files above size

- `known-issues` — List current known issues
- `debug-report` — Generate a debug report for support
- `validate-config` — Validate your configuration file

## Examples

```bash
uveddi analyze ./src --lang rust --output report.md
uveddi analyze ./src --detectors god-object,tight-coupling
uveddi known-issues
uveddi debug-report > debug.log
```

---

> **Tip:** Run `uveddi --help` for the most up-to-date list of commands and options.
