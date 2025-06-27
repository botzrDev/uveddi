# CodeAtlas: AI-Powered Architectural Analysis CLI

![CodeAtlas Logo/Banner (Placeholder - will add actual visual later)](./docs/img/codeatlas-banner.png)

"Stop Just Linting Your Code. Start Analyzing Your Architecture."

CodeAtlas is a powerful Command-Line Interface (CLI) tool designed to help Senior Developers, Tech Leads, and Software Architects maintain the health and integrity of their software codebases by performing high-level architectural analysis using a hybrid of local and API-based Artificial Intelligence models. It goes beyond traditional static analysis to identify subtle but critical architectural anti-patterns that lead to technical debt and system complexity.

## 💡 Why CodeAtlas?

In today's fast-paced, AI-augmented software development landscape, architectural drift—the slow erosion of a system's design integrity—is a pervasive and costly problem. Manual architectural reviews are time-consuming, subjective, and don't scale. Existing static analysis tools are often too noisy and focus on line-level issues, while AI code generators like GitHub Copilot, while excellent for generation, lack a holistic understanding of project architecture and can even accelerate the introduction of architectural debt.

CodeAtlas fills this critical gap by providing:

* **A Singular Focus on Architecture:** We evaluate the foundational blueprint of your software, answering strategic questions like "Is this system structurally sound?" or "Are we introducing dangerous coupling?"
* **Dual AI Model for Ultimate Flexibility & Privacy:**
    * **Local Model (Free Tier):** Analyze proprietary code privately and offline using high-performance, open-source LLMs directly on your machine.
    * **API-based Model (Paid Tier):** Leverage state-of-the-art commercial LLMs for the most complex, nuanced analysis and seamless CI/CD integration.
* **High-Quality, Actionable Reporting:** Get well-formatted markdown reports with embedded diagrams (Mermaid.js/PlantUML) that make complex architectural issues and refactoring suggestions immediately understandable and shareable.

CodeAtlas acts as your **AI Architect**, providing essential architectural intelligence and guardrails for code increasingly written by both humans and AI.

## ✨ Key Features

* **Architectural Anti-pattern Detection:** Identifies a robust taxonomy of high-level architectural smells:
    * **Dependency-Based:** Unstable Interface, Cyclic Dependency, Modularity Violation.
    * **Abstraction-Based:** The Blob/God Object, Leaky Abstraction, Violation of Inheritance Hierarchy.
    * **Microservice-Specific:** Insufficient Access Control, Hardcoded Endpoints, Shared Database.
* **Hybrid AI Analysis:** Seamlessly switches between local (Ollama-powered, private) and API-based (GPT-4, Claude 3, Gemini for enhanced accuracy) AI models.
* **Abstract Syntax Tree (AST) Powered:** Utilizes deep structural analysis via AST parsing (leveraging Tree-sitter) for accurate anti-pattern identification.
* **Comprehensive Markdown Reports:** Generates human-readable reports with clear explanations, relevant code snippets, AI-generated refactoring suggestions, severity indicators, and integrated diagrams.
* **CI/CD Ready:** Designed for headless execution and provides meaningful exit codes, making it perfect for integration into your Continuous Integration/Continuous Deployment pipelines.
* **Extensible Plugin System:** A robust plugin system allows the community to contribute custom scanners for new architectural patterns, languages, or specific frameworks.
* **Targeted Analysis:** Analyze specific directories or modules for faster, more focused scans.

## 🚀 Getting Started (For Developers & Contributors)

### Prerequisites

* **Rust Toolchain:** CodeAtlas is built with Rust for performance and safety.
    * Install `rustup` by following the instructions on [rustup.rs](https://rustup.rs/).
* **Git:** Required for cloning the repository.

### Local Development Setup

1.  **Clone the Repository:**
    ```bash
    git clone [https://github.com/your-org/codeatlas-cli.git](https://github.com/your-org/codeatlas-cli.git)
    cd codeatlas-cli
    ```
2.  **Build the Project:**
    ```bash
    cargo build --release
    ```
    This will compile the CodeAtlas CLI binary and place it in `target/release/codeatlas`.
3.  **Run Tests:**
    ```bash
    cargo test
    ```
4.  **Install Local AI Models (Optional, for Free Tier development):**
    For developing with local AI models, you'll need `Ollama`.
    ```bash
    # Run the CodeAtlas init command (once implemented)
    ./target/release/codeatlas init-local-ai
    # This command will guide you through installing Ollama and downloading a default model (e.g., mistral:7b-instruct-v0.2-q4_K_M)
    ```
    **Hardware Recommendation for Local Models:** A minimum of 16GB RAM is required. For optimal performance, a GPU with at least 8-12GB VRAM (e.g., NVIDIA RTX 3060 or better) is highly recommended.

### Basic Usage (Once Built)

To run an analysis on your current directory (example placeholder):

```bash
./target/release/codeatlas analyze . --output-file architectural_report.md