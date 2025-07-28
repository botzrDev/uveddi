# Uveddi Alpha Release Notes

## Version: 0.9.0
## Release Date: July 28, 2025

We're excited to announce the alpha release of Uveddi, an AI-powered architectural analysis CLI tool. This release marks a significant milestone in our development journey, and we welcome your feedback to help us improve.

## Major Features

- **Architectural Anti-Pattern Detection**: Identifies common architectural issues like cyclic dependencies, god objects, and leaky abstractions
- **Microservice-Specific Analysis**: Detects microservice smells including shared databases and hardcoded service endpoints
- **LLM API Integrations**: Supports OpenAI (GPT-4), Anthropic (Claude 3), Google Gemini, and local Ollama
- **Knowledge Library**: Provides contextual insights and solution recommendations
- **Plugin Architecture**: Extensible system for adding custom detectors and analysis capabilities
- **Reporting & Visualization**: Generates Mermaid.js diagrams and PlantUML visualizations
- **Performance Monitoring**: Built-in metrics for retrieval times, cache efficiency, and resource usage

## Known Limitations

1. **Limited Language Support**: Currently supports Rust, Python, and JavaScript analysis. Support for additional languages is planned.
2. **Plugin Sandboxing**: WebAssembly (WASM) sandboxing for plugins is planned for future release (UV-456).
3. **Enterprise Scalability**: Performance degrades with very large codebases (>1M LOC) (UV-512).
4. **UI Polish**: The TUI interface requires additional refinement for better user experience (UV-478).
5. **Documentation Gaps**: Some advanced features lack comprehensive documentation (UV-523).

## Missing Features

- Cloud-native architecture analysis patterns
- Integration with Jira for automatic ticket creation
- Advanced visualization export options (PNG, SVG)
- Custom rule creation through CLI
- Historical analysis and trend reporting

## How to Report Issues

Please report any bugs or unexpected behavior using the following methods:

1. **GitHub Issues**: 
   - Visit our [Issues page](https://github.com/botzrDev/uveddi/issues)
   - Use the "Bug Report" template
   - Include the issue key "UV-XXX" if applicable

2. **Email Support**:
   - Send to: support@uveddi.com
   - Subject: [Alpha Bug] - Brief description
   - Include: OS version, Uveddi version, reproduction steps

3. **Community Forums**:
   - Join our [Discord server](https://discord.gg/uveddi)
   - Post in the #alpha-feedback channel

When reporting issues, please include:
- The command you were running
- Expected vs actual behavior
- Any relevant error messages
- Steps to reproduce the issue
- Environment details (OS, shell, etc.)

## Feedback Welcome

We value your input! Please share your experience with:
- Feature requests
- Documentation improvements
- Usability suggestions
- Performance observations

Your feedback will directly influence our development priorities and help shape Uveddi's future.

## Thank You

We appreciate your participation in our alpha program. Together, we're building better tools for architectural analysis and software quality.
