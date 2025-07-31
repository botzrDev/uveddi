# Quickstart: Get Up and Running in 5 Minutes

Welcome to Uveddi! This guide will help you get started with your first analysis in just a few minutes.

## 1. Install Uveddi

```bash
curl -sSL https://uveddi.dev/install.sh | bash
```

## 2. Verify Installation

```bash
uveddi --version
```

## 3. Analyze Your First Project

```bash
# Basic analysis (memory optimization enabled by default)
uveddi analyze /path/to/your/code

# With AI-powered insights
uveddi analyze /path/to/your/code --enable-ai
```

## 4. View the Report

By default, the report will be printed to your terminal. To save it to a file:

```bash
uveddi analyze /path/to/your/code --output report.md
```

> **Performance Note**: Uveddi automatically optimizes memory usage based on your system. No configuration needed!

## 5. Next Steps

- Explore more options with `uveddi --help`
- Read the [User Guide](../02-user-guide/basic-concepts.md) for deeper usage
- Join our [Discord](https://discord.gg/uveddi) for help and feedback

---

> **Tip:** If you encounter any issues, check the [Troubleshooting Guide](../02-user-guide/troubleshooting.md) or [report a bug](../reporting-bugs.md).
