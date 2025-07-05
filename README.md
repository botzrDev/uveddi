# Uveddi: AI-Powered Architectural Analysis CLI

![Uveddi Logo/Banner (Placeholder - will add actual visual later)](./docs/img/uveddi-banner.png)

"Stop Just Linting Your Code. Start Analyzing Your Architecture."

Uveddi is an open source Command-Line Interface (CLI) tool designed to help developers maintain the health and integrity of their software codebases by performing high-level architectural analysis using local AI models. It goes beyond traditional static analysis to identify subtle but critical architectural anti-patterns that lead to technical debt and system complexity.

## 💡 Why Uveddi?

In today's fast-paced, AI-augmented software development landscape, architectural drift—the slow erosion of a system's design integrity—is a pervasive and costly problem. Manual architectural reviews are time-consuming, subjective, and don't scale. Existing static analysis tools are often too noisy and focus on line-level issues, while AI code generators like GitHub Copilot, while excellent for generation, lack a holistic understanding of project architecture and can even accelerate the introduction of architectural debt.

Uveddi fills this critical gap by providing:

* **A Singular Focus on Architecture:** We evaluate the foundational blueprint of your software, answering strategic questions like "Is this system structurally sound?" or "Are we introducing dangerous coupling?"
* **Local AI Analysis with Complete Privacy:** Analyze proprietary code privately and offline using high-performance, open-source LLMs (via Ollama) directly on your machine. Your code never leaves your system.
* **High-Quality, Actionable Reporting:** Get well-formatted markdown reports that make complex architectural issues and refactoring suggestions immediately understandable and actionable.
* **Community-Driven Development:** Open source with community contributions driving new features, language support, and anti-pattern detectors.

Uveddi acts as your **AI Architect**, providing essential architectural intelligence for modern development workflows.

## ✨ Key Features

* **Core Anti-pattern Detection:** Identifies essential architectural issues:
    * **God Objects:** Classes/structs with too many responsibilities
    * **Cyclic Dependencies:** Import/dependency cycles that create tight coupling
    * **Code Duplication:** Duplicate code blocks across your codebase
    * **Magic Values:** Hardcoded constants without explanation
    * **Tight Coupling:** Excessive dependencies between modules
* **Local AI Analysis:** Powered by Ollama for private, offline analysis with no data sharing
* **Multi-Language Support:** Currently supports Rust, Python, and JavaScript with more languages coming through community contributions
* **Local SQLite Database:** Store analysis results locally with no cloud dependencies

## 📚 Documentation

### Architecture & Design
* [📋 ARCHITECTURE.md](./docs/ARCHITECTURE.md) - Complete architectural documentation with layer definitions and boundaries
* [🏗️ C4_ARCHITECTURE.md](./docs/C4_ARCHITECTURE.md) - C4 model diagrams (Context, Container, Component, Code)
* [🔧 SAM.md](./docs/SAM.md) - Software Architecture Model (canonical source of truth)
* [📊 Architecture_Analysis_Report_2025-07-01.md](./docs/Architecture_Analysis_Report_2025-07-01.md) - Current architectural health assessment

### Development & Operations
* [🗄️ DATABASE_GUIDE.md](./docs/DATABASE_GUIDE.md) - Comprehensive guide for database setup, migrations, and deployment
* [⚙️ backend/README.md](./backend/README.md) - FastAPI backend documentation
* [🔌 plugins/DEVELOPER_GUIDE.md](./plugins/DEVELOPER_GUIDE.md) - Plugin development guide
* [🧪 tests/README.md](./tests/README.md) - Testing strategy and guidelines

### Project Management
* [📈 PRD.md](./docs/PRD.md) - Product Requirements Document
* [🎯 SPRINTS.md](./SPRINTS.md) - Sprint planning and progress tracking

## 🚀 Getting Started

### Quick Install

```bash
curl -sSL https://uveddi.dev/install.sh | bash
```

### Manual Installation

1.  **Install Prerequisites:**
    * **Rust Toolchain:** Install `rustup` from [rustup.rs](https://rustup.rs/)
    * **Ollama:** Install from [ollama.ai](https://ollama.ai) for local AI analysis

2.  **Clone and Build:**
    ```bash
    git clone https://github.com/botzrDev/uveddi.git
    cd uveddi
    cargo build --release
    ```

3.  **Set up Local AI (Optional):**
    ```bash
    # Install Ollama and pull a model
    ollama pull deepseek-coder:6.7b-instruct
    ```
    **Hardware Recommendation:** 16GB+ RAM recommended. GPU with 8GB+ VRAM optional but improves performance.

### Basic Usage

```bash
# Analyze current directory
uveddi analyze .

# Analyze with AI explanations (requires Ollama)
uveddi analyze . --enable-ai

# Save report to file
uveddi analyze . --output report.md
```

## 🐳 Docker Usage

Run Uveddi with local AI in a containerized environment:

```bash
# Build the image
docker build -t uveddi .

# Run analysis on your code
docker run -v $(pwd):/workspace uveddi analyze /workspace --enable-ai
```

The container includes Ollama and DeepSeek-Coder for complete local AI analysis.

## 🤝 Contributing

Uveddi is community-driven! We welcome contributions:

- **New Anti-Pattern Detectors:** Help identify more architectural issues
- **Language Support:** Add support for new programming languages  
- **Documentation:** Improve guides, examples, and API docs
- **Bug Reports:** Help us improve reliability and accuracy

See our [Contributing Guide](./docs/community/GUIDELINES.md) for details on how to get started.

## 📝 Documentation Standards

- All public APIs must have `///` doc comments
- Complex algorithms need explanatory comments
- Each module starts with `//!` module documentation
- See [`docs/commenting_documentation_checklist.md`](./docs/commenting_documentation_checklist.md) for full guidelines

---

# Uveddi

Uveddi is a Rust project for ...

## Build Instructions

```sh
cargo build
```

## Run

```sh
cargo run
```

## Test

```sh
cargo test
```

## Documentation

- See `docs/` for guides, API, and community info

## Contributing

- Please see `docs/community/GUIDELINES.md`
- Use GitHub Issues for bugs and feature requests

---

(Keep this file up to date with project changes.)