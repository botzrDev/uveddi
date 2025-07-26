<div class="uveddi-logo-header">
  <img src="./assets/brand/logos/logo-blue.png" alt="Uveddi Logo" class="uveddi-logo" />
  <div class="uveddi-brand-text">
    <h1>Uveddi Documentation</h1>
    <p class="tagline">AI-Powered Code Analysis That Stays Private</p>
  </div>
</div>

Welcome to the comprehensive documentation for Uveddi, the privacy-first architectural analysis tool that helps developers identify anti-patterns and prevent architectural drift without compromising data security.

## What is Uveddi?

Uveddi is a sophisticated CLI tool that combines the power of AI with local-first architecture to provide:

- **Multi-language analysis** - Support for Rust, Python, and JavaScript
- **AI-powered insights** - Local and cloud AI integration for intelligent recommendations  
- **Privacy-focused** - All analysis happens locally, your code never leaves your machine
- **Comprehensive reporting** - Markdown, JSON, and interactive visual outputs
- **Tree-sitter enabled** - Advanced parsing for improved accuracy and performance

## Quick Start

Get up and running with Uveddi in minutes:

```bash
# Quick install
curl -sSL https://uveddi.dev/install.sh | bash

# Analyze your codebase
uveddi analyze /path/to/your/code --enable-ai

# Generate detailed report
uveddi analyze /path/to/your/code --output report.md
```

## Documentation Structure

Our documentation is organized to help you succeed, whether you're a new user or contributing to the project:

### Getting Started
- [**Installation**](./01-getting-started/installation.md) - Get Uveddi running on your system
- [**First Steps**](./01-getting-started/first-steps.md) - Your first analysis in 5 minutes
- [**Configuration**](./01-getting-started/configuration.md) - Customize Uveddi for your workflow

### User Guide  
- [**Basic Concepts**](./02-user-guide/basic-concepts.md) - Core concepts and terminology
- [**Common Use Cases**](./02-user-guide/common-use-cases.md) - Real-world scenarios and solutions
- [**Dashboard Guide**](./02-user-guide/dashboard-guide.md) - Navigate the interactive dashboard
- [**Troubleshooting**](./02-user-guide/troubleshooting.md) - Solutions to common issues

### Architecture & Development
- [**Architecture Overview**](./04-architecture/overview.md) - System design and components
- [**Developer Guide**](./05-development/DEVELOPER_GUIDE.md) - Contributing to Uveddi
- [**Detector Development**](./05-development/detector_development_guide.md) - Build custom detectors

### Security & Operations
- [**Security Model**](./04-architecture/security-model.md) - How we protect your data
- [**Deployment Guide**](./operations/deployment-guide.md) - Production deployment strategies
- [**Threat Model**](./security/threat-model.md) - Security considerations and mitigations

### Community
- [**Contributing Guidelines**](./09-community/CONTRIBUTING.md) - How to contribute
- [**Good First Issues**](./09-community/GOOD_FIRST_ISSUES.md) - Perfect for newcomers
- [**Code of Conduct**](./09-community/CODE_OF_CONDUCT.md) - Our community standards

## Key Features

### **Intelligent Analysis**
Uveddi uses advanced static analysis combined with AI to identify:
- **God Objects** - Classes that know too much or do too much
- **Tight Coupling** - Components that are overly dependent on each other  
- **Dead Code** - Unused functions, variables, and imports
- **Cyclic Dependencies** - Circular references that create maintenance issues
- **Large Classes** - Components that have grown beyond manageable size

### **AI-Enhanced Insights**
- **Local AI Support** - Use Ollama for completely private analysis
- **Cloud AI Integration** - Optional cloud providers for enhanced capabilities
- **Contextual Explanations** - AI-generated explanations for detected issues
- **Refactoring Suggestions** - Actionable recommendations for improvement

### **Rich Reporting**
- **Interactive Dashboards** - Visual exploration of your codebase
- **Mermaid Diagrams** - Architectural visualizations and dependency graphs
- **Multiple Formats** - JSON, Markdown, HTML, and custom templates
- **Trend Analysis** - Track improvements over time

### **Privacy by Design**
- **Local-First Architecture** - Your code stays on your machine
- **Zero Data Collection** - We don't collect or store your code
- **Transparent Processing** - Full visibility into what data is processed
- **Configurable Privacy** - Choose your comfort level with AI providers

## Interactive Examples

Try these examples to see Uveddi in action:

> **Tip**: All code examples in this documentation are interactive. Click the copy button to try them yourself!

```rust
// Example: Analyzing a Rust project
use uveddi::analysis::AnalysisEngine;

let engine = AnalysisEngine::new(config)?;
let results = engine.analyze_project("./src").await?;

for issue in results.anti_patterns {
    println!("Found {}: {}", issue.pattern_type, issue.description);
}
```

```bash
# Example: Generate architectural diagram
uveddi analyze ./src --output-format mermaid --save-diagram architecture.mmd

# Example: Focus on specific patterns
uveddi analyze ./src --detectors god-object,tight-coupling --severity high
```

## Why Choose Uveddi?

**Privacy-First**: Your code never leaves your machine - analyze with confidence

**Developer-Friendly**: Built by developers, for developers - intuitive CLI and rich documentation

**Performance-Focused**: Optimized for large codebases with intelligent caching and parallel processing

## Contributing

We welcome contributions from developers of all skill levels! Here's how to get involved:

1. **Report Issues** - Found a bug? [Open an issue](https://github.com/botzrDev/uveddi/issues)
2. **Suggest Features** - Have an idea? We'd love to hear it
3. **Improve Documentation** - Help make our docs even better
4. **Submit Code** - Check out our [good first issues](./09-community/GOOD_FIRST_ISSUES.md)

> **New to Open Source?** Check out our [Mentorship System](./09-community/MENTORSHIP_SYSTEM.md) for guidance and support.

## Support & Community

- **Documentation**: You're reading it! Comprehensive guides and references
- **Discussions**: [GitHub Discussions](https://github.com/botzrDev/uveddi/discussions) for questions and ideas
- **Issues**: [GitHub Issues](https://github.com/botzrDev/uveddi/issues) for bugs and feature requests
- **Discord**: [Join our Discord server](https://discord.gg/uveddi) for support and community discussions

## License

Uveddi is open source software licensed under the [MIT License](../LICENSE). This means you can use it freely in both personal and commercial projects.

---

<div style="text-align: center; margin: 2rem 0; padding: 2rem; background: var(--uveddi-bg-secondary); border-radius: 8px;">

**Ready to improve your code quality?**

[Get Started](./01-getting-started/installation.md) | [User Guide](./02-user-guide/basic-concepts.md) | [Contribute](./09-community/CONTRIBUTING.md)

</div>

*Built with care by the Uveddi community*