# Uveddi: AI-Powered Architectural Analysis CLI

![Uveddi Logo/Banner (Placeholder - will add actual visual later)](./docs/img/uveddi-banner.png)

"Stop Just Linting Your Code. Start Analyzing Your Architecture."

Uveddi is a powerful Command-Line Interface (CLI) tool designed to help Senior Developers, Tech Leads, and Software Architects maintain the health and integrity of their software codebases by performing high-level architectural analysis using a hybrid of local and API-based Artificial Intelligence models. It goes beyond traditional static analysis to identify subtle but critical architectural anti-patterns that lead to technical debt and system complexity.

## 💡 Why Uveddi?

In today's fast-paced, AI-augmented software development landscape, architectural drift—the slow erosion of a system's design integrity—is a pervasive and costly problem. Manual architectural reviews are time-consuming, subjective, and don't scale. Existing static analysis tools are often too noisy and focus on line-level issues, while AI code generators like GitHub Copilot, while excellent for generation, lack a holistic understanding of project architecture and can even accelerate the introduction of architectural debt.

Uveddi fills this critical gap by providing:

* **A Singular Focus on Architecture:** We evaluate the foundational blueprint of your software, answering strategic questions like "Is this system structurally sound?" or "Are we introducing dangerous coupling?"
* **Dual AI Model for Ultimate Flexibility & Privacy:**
    * **Local Model (Free Tier):** Analyze proprietary code privately and offline using high-performance, open-source LLMs directly on your machine.
    * **API-based Model (Paid Tier):** Leverage state-of-the-art commercial LLMs for the most complex, nuanced analysis and seamless CI/CD integration.
* **High-Quality, Actionable Reporting:** Get well-formatted markdown reports with embedded diagrams (Mermaid.js/PlantUML) that make complex architectural issues and refactoring suggestions immediately understandable and

Uveddi acts as your **AI Architect**, providing essential architectural intelligence and guardrails for code increasingly written by both humans and AI.

## ✨ Key Features

* **Architectural Anti-pattern Detection:** Identifies a robust taxonomy of high-level architectural smells:
    * **Dependency-Based:** Unstable Interface, Cyclic Dependency, Modularity Violation.
    * **Abstraction-Based:** The Blob/God Object, Leaky Abstraction, Violation of Inheritance Hierarchy.
    * **Microservice-Specific:** Insufficient Access Control, Hardcoded Endpoints, Shared Database.
* **Hybrid AI Analysis:** Seamlessly switches between local (Ollama-powered, private) and API-based (GPT-4, Claude 3, Gemini for enhanced accuracy) AI models.
* **Abstract Syntax Tree (AST) Powered:** Utilizes deep structural analysis via AST parsing (leveraging Tree-sitter) for accurate anti-pattern identification.
* **Dual Database Architecture:**
    * **Local SQLite Database:** Store analysis results locally in the Rust CLI.
    * **Cloud PostgreSQL Database:** Sync analysis data to a centralized backend for team collaboration and CI/CD integration.

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

## 🚀 Getting Started (For Developers & Contributors)

### Prerequisites

* **Rust Toolchain:** Uveddi is built with Rust for performance and safety.
    * Install `rustup` by following the instructions on [rustup.rs](https://rustup.rs/).
* **Git:** Required for cloning the repository.

### Local Development Setup

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/botzrDev/uveddi.git
    cd uveddi
    ```
2.  **Build the Project:**
    ```bash
    cargo build --release
    ```
    This will compile the Uveddi CLI binary and place it in `target/release/uveddi`.
3.  **Run Tests:**
    ```bash
    cargo test
    ```
4.  **Install Local AI Models (Optional, for Free Tier development):**
    For developing with local AI models, you'll need `Ollama`.
    ```bash
    # Run the Uveddi init command (once implemented)
    ./target/release/uveddi init-local-ai
    # This command will guide you through installing Ollama and downloading a default model (e.g., mistral:7b-instruct-v0.2-q4_K_M)
    ```
    **Hardware Recommendation for Local Models:** A minimum of 16GB RAM is required. For optimal performance, a GPU with at least 8-12GB VRAM (e.g., NVIDIA RTX 3060 or better) is highly recommended.

### Basic Usage (Once Built)

To run an analysis on your current directory (example placeholder):

```bash
./target/release/uveddi analyze . --output-file architectural_report.md
```

### Backend Setup (Optional)

For team collaboration and CI/CD integration, set up the backend:

```bash
cd backend

# Option 1: Using Docker Compose (recommended)
./docker-compose.sh up

# Option 2: Manual setup
cp .env.example .env
# Edit .env with your database credentials
make setup migrate
make run
```

See [DATABASE_GUIDE.md](./DATABASE_GUIDE.md) for detailed instructions.

## 🐳 Running Uveddi with Local AI in Docker

You can run Uveddi in a fully containerized environment with Ollama and DeepSeek-Coder for local AI analysis. No host setup required!

### Build the Docker image:

```bash
docker build -t uveddi-local-ai .
```

### Run the container (interactive shell):

```bash
docker run -it --rm uveddi-local-ai
```

This will:
- Start the Ollama server
- Pull the DeepSeek-Coder model
- Run all Uveddi tests (including AI integration)
- Drop you into a shell with the environment ready

### Run an analysis with local AI:

```bash
# From inside the container shell:
cargo run --release -- analyze . --enable-ai
```

You can also specify a different model or API URL:

```bash
cargo run --release -- analyze . --enable-ai --ollama-model deepseek-coder:6.7b-instruct-q4_0 --ollama-api-url http://localhost:11434
```

Or set environment variables:

```bash
export OLLAMA_MODEL=deepseek-coder:6.7b-instruct-q4_0
export OLLAMA_API_URL=http://localhost:11434
cargo run --release -- analyze . --enable-ai
```

All AI explanations in reports will be generated using the local DeepSeek model via Ollama.