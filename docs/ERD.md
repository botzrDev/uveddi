Engineering Requirements Document (ERD) for CodeAtlas  
  
**1. Introduction**  
  
This Engineering Requirements Document (ERD) details the technical specifications and engineering considerations for the development of CodeAtlas, an AI-powered Command-Line Interface (CLI) tool for high-level architectural analysis of software codebases. It translates the product requirements outlined in the PRD into actionable engineering tasks and technical design decisions.  
  
**1.1. Purpose**  
The purpose of this document is to guide the engineering team in the design, development, and testing of CodeAtlas, ensuring that the implemented solution meets the defined functional, non-functional, and performance requirements.  
  
**1.2. Scope**  
This ERD covers the technical architecture, component design, data models, integration points, and performance targets for the initial release of CodeAtlas. It focuses on the core analysis engine, AI model integration, reporting mechanisms, and the plugin system.  
  
**1.3. Definitions, Acronyms, and Abbreviations**

- **AI:** Artificial Intelligence
- **API:** Application Programming Interface
- **AST:** Abstract Syntax Tree
- **CI/CD:** Continuous Integration/Continuous Deployment
- **CLI:** Command-Line Interface
- **CPU:** Central Processing Unit
- **ERD:** Engineering Requirements Document (this document)
- **GPU:** Graphics Processing Unit
- **LLM:** Large Language Model
- **PRD:** Product Requirements Document
- **RAG:** Retrieval-Augmented Generation
- **RAM:** Random Access Memory
- **USP:** Unique Selling Proposition
- **VRAM:** Video Random Access Memory
- **WASM:** WebAssembly (for plugin sandboxing)

**2. System Architecture**  
  
CodeAtlas will implement a hybrid architecture to balance performance, privacy, and the power of advanced AI models.  
  
**2.1. High-Level System Diagram**

```
graph TD
    A[User/CI/CD] --> B{CodeAtlas CLI}
    B --> C[Codebase Files]
    C -- .archlintignore --> D[File Ingestion Module]
    D --> E[AST Parsing Module (Tree-sitter)]
    E --> F[Dependency Graph Builder]
    F --> G[Deterministic Anti-pattern Detector]
    G -- Relevant Context/Snippets --> H{AI Reasoning Engine}
    H -- Local Model Selector --> I[Ollama (Local LLM)]
    H -- API Model Selector --> J[External LLM APIs (OpenAI, Anthropic, Google)]
    H -- AI-Generated Insights --> K[Report Generator]
    K --> L[Markdown Report (with Diagrams)]
    B -- Plugin API --> M[Plugin System]
    M --> N[Community Plugins]
    
    subgraph Core Analysis Pipeline
        D --> E
        E --> F
        F --> G
        G --> H
    end
    
    subgraph AI Model Layer
        H --> I
        H --> J
    end
    
    subgraph Reporting Layer
        K --> L
    end
    
    subgraph Extensibility Layer
        B --> M
        M --> N
    end
```

**2.2. Component Breakdown**

- **CLI Core:** The main entry point for user interaction, command parsing, and orchestrating other modules.
- **Configuration Manager:** Handles loading and managing configurations (e.g., `.archlintignore`, API keys, analysis scopes).
- **File Ingestion Module:** Scans the target codebase, filters files based on configuration, and prepares them for parsing.
- **AST Parsing Module:** Utilizes Tree-sitter to generate Abstract Syntax Trees for supported programming languages.
- **Dependency Graph Builder:** Constructs and maintains an in-memory (or cached) dependency graph of modules and components.
- **Deterministic Anti-pattern Detector:** Implements rule-based and graph-traversal algorithms to identify common anti-patterns based on ASTs and dependency graphs. This provides the "fact-based" grounding for AI.
- **AI Reasoning Engine:** Orchestrates calls to either local or API-based LLMs, injecting relevant code snippets and structural context (RAG).
- **Ollama Integration:** Manages interaction with the local Ollama instance for local LLM inference.
- **External LLM API Clients:** Clients for interacting with OpenAI, Anthropic, and Google Gemini APIs.
- **Report Generator:** Compiles analysis findings, AI explanations, and diagram syntax into a structured Markdown report.
- **Caching Layer:** Stores parsed ASTs and intermediate analysis results to speed up subsequent runs.
- **Plugin System:** Provides an API and runtime environment for loading and executing community-contributed scanners.

**3. Functional Requirements**  
  
**3.1. Core Analysis Engine**

- **ER-F-001: Codebase Scanning:** The CLI SHALL recursively scan a specified directory, respecting exclusion rules defined in a `.archlintignore` file (or equivalent).
- **ER-F-002: Language Parsing (AST Generation):** The tool SHALL support parsing ASTs for Rust, Python, and JavaScript in the initial release. Future releases will expand language support via plugins.
- **ER-F-003: Dependency Graph Construction:** The tool SHALL build a directed graph representing inter-module/component dependencies based on AST analysis.
- **ER-F-004: Anti-pattern Detection (Deterministic):** The tool SHALL implement deterministic detection algorithms for:
    - Cyclic Dependency (via graph cycle detection).
    - The Blob/God Object (via AST node count heuristics).
    - Unstable Interface (via dependency graph fan-in and change frequency heuristics).
    - Modularity Violation (via co-change analysis heuristics - _stretch goal for V1, may be V2_).
- **ER-F-005: Contextual Code Snippet Extraction:** For each detected anti-pattern, the tool SHALL extract relevant code snippets and surrounding context from the AST.

**3.2. AI Model Integration**

- **ER-F-006: Local LLM Integration:** The tool SHALL integrate with Ollama to utilize locally hosted LLMs for analysis.
    - **ER-F-006.1:** The CLI SHALL provide a command to facilitate Ollama setup and model download (e.g., `codeatlas init-local-ai`).
    - **ER-F-006.2:** The tool SHALL default to a specified local model (e.g., `mistral:7b-instruct-v0.2-q4_K_M`) if available and configured.
- **ER-F-007: API-based LLM Integration:** The tool SHALL support integration with OpenAI (GPT-4), Anthropic (Claude 3), and Google (Gemini) LLM APIs.
    - **ER-F-007.1:** API keys and model preferences SHALL be configurable via environment variables or a secure configuration file.
- **ER-F-008: Smart Prompting/RAG:** The AI Reasoning Engine SHALL construct prompts that embed contextual code snippets and structural information from the AST analysis.
- **ER-F-009: AI-Generated Explanations & Suggestions:** The AI SHALL generate human-readable explanations of architectural issues and propose refactoring suggestions.
- **ER-F-010: Hallucination Mitigation (Prompt Engineering):** Prompts SHALL include instructions to constrain AI output, define format, and specify uncertainty handling (e.g., "state if unsure").

**3.3. Reporting**

- **ER-F-011: Markdown Report Generation:** The tool SHALL generate a comprehensive analysis report in Markdown format.
- **ER-F-012: Diagram Integration:** The report SHALL include Mermaid.js syntax for visual representation of architectural issues (e.g., cyclic dependencies, class relationships for God Objects).
    - **ER-F-012.1:** The AI SHALL be capable of generating valid Mermaid.js syntax for relevant diagrams.
- **ER-F-013: Issue Categorization:** The report SHALL categorize issues by anti-pattern type (e.g., Dependency-Based, Abstraction-Based, Microservice-Specific).
- **ER-F-014: Issue Detail:** Each issue entry in the report SHALL include:
    - Anti-pattern name.
    - File path and line numbers.
    - Severity (e.g., Critical, High, Medium, Low).
    - AI-generated title and description.
    - Relevant code snippets.
    - AI-generated refactoring suggestions.
    - Optional Mermaid.js diagram.

**3.4. Command-Line Interface (CLI)**

- **ER-F-015: Command Structure:** The CLI SHALL provide subcommands for core functionalities (