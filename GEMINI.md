# Gemini Agent Project Guide: uveddi

## About This File

This document provides guidance for the Gemini AI agent to effectively understand and interact with the `uveddi` project. It outlines the project's architecture, tech stack, key commands, and development workflows.

## Project Overview

`uveddi` is a comprehensive software analysis and visualization platform designed for developers. It performs static analysis on source code to identify anti-patterns (like God Objects, tight coupling), detect dead code, and visualize software architecture. The project consists of a powerful Rust-based core analysis engine, a command-line interface (CLI), a terminal UI (TUI), and a web-based frontend for rich visualizations. A separate Node.js service appears to handle rendering of diagrams.

## Tech Stack

- **Backend:** Rust
  - **CLI:** `clap`
  - **TUI:** `ratatui` (likely)
  - **Web Server:** `axum` or `actix-web` (likely)
  - **Async Runtime:** `tokio`
- **Frontend:** TypeScript, Vite, React (likely), Tailwind CSS
- **Rendering Service:** Node.js
- **Database:** SQLite (for local/embedded use) and PostgreSQL (for production/server use)
- **Containerization:** Docker and Docker Compose
- **CI/CD:** GitHub Actions
- **Code Quality:** `pre-commit` hooks, `clippy` (Rust), `eslint` (TypeScript)

## Project Structure

- `src/`: Core Rust backend logic.
  - `src/main.rs`: Main entry point for the CLI application.
  - `src/analysis/`: The core static analysis engine and anti-pattern detectors.
  - `src/tui/`: Terminal User Interface implementation.
  - `src/server/`: Web server implementation.
  - `src/cli/`: Command-line argument parsing and handling.
- `frontend/`: The web application source code.
- `rendering-service/`: A microservice dedicated to generating diagrams and visualizations.
- `tests/`: Rust integration and unit tests.
- `benches/`: Rust performance benchmarks.
- `docs/`: Project documentation.
- `scripts/`: Helper scripts for development, testing, and deployment.
- `templates/`: Tera templates used for generating reports and diagrams.
- `migrations/`: SQL database schema migrations.

## Key Commands

### Rust Backend (Run from project root)

- **Build:** `cargo build --release`
- **Run CLI:** `cargo run -- [args]`
- **Run Tests:** `cargo test`
- **Check Formatting:** `cargo fmt --check`
- **Apply Formatting:** `cargo fmt`
- **Lint:** `cargo clippy --all-targets`

### Frontend (Run from `frontend/` directory)

- **Install Dependencies:** `npm install`
- **Run Dev Server:** `npm run dev`
- **Build for Production:** `npm run build`
- **Run Tests:** `npx cypress run`

### Docker (Run from project root)

- **Start all services (production):** `docker-compose up -d`
- **Start all services (development):** `docker-compose -f docker-compose.dev.yml up -d`
- **Stop all services:** `docker-compose down`

## Development Workflow

1.  **Setup:** Install `pre-commit` hooks to ensure code quality before committing.
    ```bash
    pre-commit install
    ```
2.  **Code Changes:**
    - Make changes to the Rust backend in `src/`.
    - Make changes to the web frontend in `frontend/src/`.
3.  **Testing:**
    - Add corresponding tests for any new features or bug fixes in the `tests/` directory for backend changes or within the `frontend/` directory for frontend changes.
    - Run `cargo test` and `npm test` (or Cypress tests) to ensure changes haven't broken existing functionality.
4.  **Committing:** Write clear and concise commit messages. Check `git log` for examples of the prevailing style. The pre-commit hooks will run automatically.
