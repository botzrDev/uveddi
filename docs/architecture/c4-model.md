# Uveddi Architecture - C4 Model

## Table of Contents

1. [Introduction](#introduction)
2. [Level 1: System Context](#level-1-system-context)
3. [Level 2: Container Diagram](#level-2-container-diagram)
4. [Level 3: Component Diagrams](#level-3-component-diagrams)
5. [Level 4: Code Diagrams](#level-4-code-diagrams)
6. [Deployment Diagrams](#deployment-diagrams)
7. [Architecture Decision Records](#architecture-decision-records)

## Introduction

This document describes the Uveddi system architecture using the C4 model (Context, Containers, Components, and Code). The C4 model provides a hierarchical way to visualize software architecture at different levels of abstraction.

### Key Architectural Principles

- **Microservices Architecture**: Loosely coupled, independently deployable services
- **Event-Driven Design**: Asynchronous communication between components
- **Domain-Driven Design**: Clear separation of business domains
- **CQRS Pattern**: Separation of read and write operations
- **Hexagonal Architecture**: Clean separation of business logic from external concerns

## Level 1: System Context

The system context diagram shows how Uveddi fits into the broader ecosystem.

```mermaid
graph TB
    subgraph "External Systems"
        User[Developer/Analyst]
        Admin[System Administrator]
        CI[CI/CD Pipeline<br/>GitHub Actions, Jenkins]
        VCS[Version Control<br/>Git, GitHub, GitLab]
        IDE[Development Environment<br/>VS Code, IntelliJ, Vim]
        Alerting[Alerting System<br/>PagerDuty, Slack]
    end
    
    subgraph "AI Ecosystem"
        Ollama[Ollama<br/>Local AI Provider]
        OpenAI[OpenAI<br/>Cloud AI Provider]
        HuggingFace[Hugging Face<br/>Models & Datasets]
    end
    
    Uveddi[Uveddi System<br/>Static Code Analysis Platform<br/>with AI-Powered Insights]
    
    User --> Uveddi
    Admin --> Uveddi
    CI --> Uveddi
    Uveddi --> VCS
    IDE --> Uveddi
    Uveddi --> Alerting
    
    Uveddi --> Ollama
    Uveddi --> OpenAI
    Uveddi --> HuggingFace
    
    style Uveddi fill:#e1f5fe
    style User fill:#fff3e0
    style Admin fill:#fff3e0
    style CI fill:#e8f5e8
    style VCS fill:#e8f5e8
    style IDE fill:#e8f5e8
    style Alerting fill:#fce4ec
```

### System Context Relationships

| Actor/System | Relationship | Description |
|--------------|-------------|-------------|
| **Developer/Analyst** | Uses | Runs code analysis, views reports, configures settings |
| **System Administrator** | Manages | Deploys, monitors, maintains the Uveddi system |
| **CI/CD Pipeline** | Integrates | Automated code quality checks in build pipelines |
| **Version Control** | Reads from | Accesses source code repositories for analysis |
| **Development Environment** | Integrates | IDE plugins provide real-time analysis feedback |
| **Alerting System** | Sends to | Notifications for critical issues and system alerts |
| **AI Providers** | Consumes | AI-powered explanations and refactoring suggestions |

## Level 2: Container Diagram

The container diagram shows the high-level technology choices and how responsibilities are distributed.

```mermaid
graph TB
    subgraph "User Interfaces"
        WebUI[Web Dashboard<br/>React/TypeScript<br/>Port 3000]
        TUI[Terminal UI<br/>Rust/Ratatui<br/>CLI Application]
        CLI[Command Line<br/>Rust Binary<br/>CLI Tool]
    end
    
    subgraph "API Layer"
        APIGateway[API Gateway<br/>Rust/Axum<br/>Port 8080]
        WSGateway[WebSocket Gateway<br/>Real-time Updates<br/>Port 8080]
    end
    
    subgraph "Core Services"
        AnalysisEngine[Analysis Engine<br/>Rust Core<br/>Multi-threaded]
        AIService[AI Service<br/>Integration Layer<br/>Provider Abstraction]
        PluginEngine[Plugin Engine<br/>WebAssembly Runtime<br/>Sandboxed Execution]
    end
    
    subgraph "Supporting Services"
        RenderingService[Rendering Service<br/>Node.js/Puppeteer<br/>Port 3001]
        CacheService[Cache Service<br/>In-Memory + Disk<br/>Redis Compatible]
        MetricsService[Metrics Service<br/>Prometheus Compatible<br/>Port 9090]
    end
    
    subgraph "Data Layer"
        Database[(Primary Database<br/>PostgreSQL/SQLite<br/>Port 5432)]
        FileSystem[(File System<br/>Source Code<br/>Analysis Cache)]
        MessageQueue[(Message Queue<br/>Analysis Jobs<br/>Redis/RabbitMQ)]
    end
    
    subgraph "External Services"
        OllamaProvider[Ollama Provider<br/>Local AI Service<br/>Port 11434]
        OpenAIProvider[OpenAI Provider<br/>Cloud AI Service<br/>HTTPS API]
        SMTPService[SMTP Service<br/>Email Notifications<br/>Port 587]
    end
    
    %% User Interface Connections
    WebUI --> APIGateway
    WebUI --> WSGateway
    WebUI --> RenderingService
    TUI --> AnalysisEngine
    CLI --> AnalysisEngine
    
    %% API Gateway Connections
    APIGateway --> AnalysisEngine
    APIGateway --> MetricsService
    APIGateway --> Database
    
    %% WebSocket Gateway
    WSGateway --> MetricsService
    WSGateway --> AnalysisEngine
    
    %% Core Service Connections
    AnalysisEngine --> AIService
    AnalysisEngine --> PluginEngine
    AnalysisEngine --> CacheService
    AnalysisEngine --> Database
    AnalysisEngine --> FileSystem
    AnalysisEngine --> MessageQueue
    
    %% AI Service Connections
    AIService --> OllamaProvider
    AIService --> OpenAIProvider
    
    %% Supporting Service Connections
    RenderingService --> CacheService
    MetricsService --> Database
    MetricsService --> SMTPService
    
    %% Styling
    style WebUI fill:#e3f2fd
    style TUI fill:#e3f2fd
    style CLI fill:#e3f2fd
    style APIGateway fill:#fff3e0
    style WSGateway fill:#fff3e0
    style AnalysisEngine fill:#e8f5e8
    style AIService fill:#e8f5e8
    style PluginEngine fill:#e8f5e8
    style RenderingService fill:#fce4ec
    style CacheService fill:#fce4ec
    style MetricsService fill:#fce4ec
    style Database fill:#f3e5f5
    style FileSystem fill:#f3e5f5
    style MessageQueue fill:#f3e5f5
```

### Container Responsibilities

#### User Interfaces
- **Web Dashboard**: Modern React-based SPA for analysis management and visualization
- **Terminal UI**: Interactive terminal interface using Ratatui for local usage
- **Command Line**: Traditional CLI for scripting and automation

#### API Layer
- **API Gateway**: RESTful API server handling HTTP requests and authentication
- **WebSocket Gateway**: Real-time bidirectional communication for live updates

#### Core Services
- **Analysis Engine**: Multi-threaded Rust engine for code analysis and orchestration
- **AI Service**: Abstraction layer for multiple AI providers with fallback strategies
- **Plugin Engine**: WebAssembly-based plugin system for extensibility

#### Supporting Services
- **Rendering Service**: Node.js service for generating diagrams and visual reports
- **Cache Service**: High-performance caching for ASTs, analysis results, and artifacts
- **Metrics Service**: Prometheus-compatible metrics collection and alerting

#### Data Layer
- **Primary Database**: Persistent storage for analysis results, configuration, and metadata
- **File System**: Source code storage and analysis cache management
- **Message Queue**: Asynchronous job processing and task distribution

## Level 3: Component Diagrams

### Analysis Engine Components

```mermaid
graph TB
    subgraph "Analysis Engine Container"
        subgraph "Core Components"
            EngineOrchestrator[Engine Orchestrator<br/>Analysis Workflow<br/>State Management]
            DetectorRegistry[Detector Registry<br/>Plugin Management<br/>Lifecycle Control]
            ParserCoordinator[Parser Coordinator<br/>Multi-language Support<br/>AST Generation]
        end
        
        subgraph "Detection Framework"
            AntiPatternDetectors[Anti-Pattern Detectors<br/>God Objects, Dead Code<br/>Tight Coupling, etc.]
            DependencyAnalyzer[Dependency Analyzer<br/>Graph Construction<br/>Cycle Detection]
            MetricsCalculator[Metrics Calculator<br/>Complexity, Quality<br/>Technical Debt]
        end
        
        subgraph "Processing Pipeline"
            FileIngestion[File Ingestion<br/>Async Walking<br/>Filtering & Validation]
            ASTCache[AST Cache<br/>Zero-Copy Serialization<br/>Compression & Storage]
            ResultAggregator[Result Aggregator<br/>Cross-cutting Analysis<br/>Report Generation]
        end
        
        subgraph "Memory Management"
            ArenaAllocator[Arena Allocator<br/>Pool Management<br/>GC Optimization]
            DetectorPool[Detector Pool<br/>Resource Management<br/>Concurrency Control]
            StreamProcessor[Stream Processor<br/>Large Codebase Support<br/>Backpressure Handling]
        end
        
        subgraph "External Interfaces"
            AIIntegration[AI Integration<br/>Context Building<br/>Response Processing]
            PluginInterface[Plugin Interface<br/>WASM Runtime<br/>Security Sandbox]
            CacheInterface[Cache Interface<br/>Storage Abstraction<br/>Consistency Management]
        end
    end
    
    %% Component Relationships
    EngineOrchestrator --> DetectorRegistry
    EngineOrchestrator --> ParserCoordinator
    EngineOrchestrator --> FileIngestion
    EngineOrchestrator --> ResultAggregator
    
    DetectorRegistry --> AntiPatternDetectors
    DetectorRegistry --> DependencyAnalyzer
    DetectorRegistry --> MetricsCalculator
    DetectorRegistry --> PluginInterface
    
    ParserCoordinator --> ASTCache
    ParserCoordinator --> StreamProcessor
    
    AntiPatternDetectors --> AIIntegration
    DependencyAnalyzer --> ResultAggregator
    MetricsCalculator --> ResultAggregator
    
    FileIngestion --> StreamProcessor
    ASTCache --> CacheInterface
    ResultAggregator --> CacheInterface
    
    ArenaAllocator --> DetectorPool
    DetectorPool --> AntiPatternDetectors
    DetectorPool --> DependencyAnalyzer
    DetectorPool --> MetricsCalculator
    
    AIIntegration --> EngineOrchestrator
    
    style EngineOrchestrator fill:#e8f5e8
    style DetectorRegistry fill:#e8f5e8
    style ParserCoordinator fill:#e8f5e8
    style AntiPatternDetectors fill:#fff3e0
    style DependencyAnalyzer fill:#fff3e0
    style MetricsCalculator fill:#fff3e0
    style FileIngestion fill:#e3f2fd
    style ASTCache fill:#e3f2fd
    style ResultAggregator fill:#e3f2fd
    style ArenaAllocator fill:#fce4ec
    style DetectorPool fill:#fce4ec
    style StreamProcessor fill:#fce4ec
```

### AI Service Components

```mermaid
graph TB
    subgraph "AI Service Container"
        subgraph "Provider Management"
            ProviderRegistry[Provider Registry<br/>Multi-provider Support<br/>Failover Logic]
            OllamaAdapter[Ollama Adapter<br/>Local AI Integration<br/>Model Management]
            OpenAIAdapter[OpenAI Adapter<br/>Cloud API Integration<br/>Rate Limiting]
            HuggingFaceAdapter[HuggingFace Adapter<br/>Model Hub Integration<br/>Custom Models]
        end
        
        subgraph "Context Processing"
            ContextBuilder[Context Builder<br/>Analysis Context<br/>Code Embeddings]
            PromptTemplates[Prompt Templates<br/>Template Management<br/>Dynamic Generation]
            ResponseParser[Response Parser<br/>Structured Output<br/>Validation & Cleanup]
        end
        
        subgraph "Intelligence Features"
            ExplanationEngine[Explanation Engine<br/>Issue Analysis<br/>Natural Language]
            SuggestionEngine[Suggestion Engine<br/>Refactoring Ideas<br/>Best Practices]
            PatternRecognition[Pattern Recognition<br/>Architectural Patterns<br/>Code Smells]
        end
        
        subgraph "Quality & Reliability"
            ResponseCache[Response Cache<br/>LRU Strategy<br/>Cost Optimization]
            FallbackHandler[Fallback Handler<br/>Provider Switching<br/>Graceful Degradation]
            QualityAssurance[Quality Assurance<br/>Response Validation<br/>Confidence Scoring]
        end
    end
    
    %% Component Relationships
    ProviderRegistry --> OllamaAdapter
    ProviderRegistry --> OpenAIAdapter
    ProviderRegistry --> HuggingFaceAdapter
    ProviderRegistry --> FallbackHandler
    
    ContextBuilder --> PromptTemplates
    PromptTemplates --> OllamaAdapter
    PromptTemplates --> OpenAIAdapter
    PromptTemplates --> HuggingFaceAdapter
    
    OllamaAdapter --> ResponseParser
    OpenAIAdapter --> ResponseParser
    HuggingFaceAdapter --> ResponseParser
    
    ResponseParser --> ExplanationEngine
    ResponseParser --> SuggestionEngine
    ResponseParser --> PatternRecognition
    ResponseParser --> QualityAssurance
    
    ExplanationEngine --> ResponseCache
    SuggestionEngine --> ResponseCache
    PatternRecognition --> ResponseCache
    
    QualityAssurance --> FallbackHandler
    
    style ProviderRegistry fill:#e8f5e8
    style OllamaAdapter fill:#fff3e0
    style OpenAIAdapter fill:#fff3e0
    style HuggingFaceAdapter fill:#fff3e0
    style ContextBuilder fill:#e3f2fd
    style PromptTemplates fill:#e3f2fd
    style ResponseParser fill:#e3f2fd
    style ExplanationEngine fill:#fce4ec
    style SuggestionEngine fill:#fce4ec
    style PatternRecognition fill:#fce4ec
    style ResponseCache fill:#f3e5f5
    style FallbackHandler fill:#f3e5f5
    style QualityAssurance fill:#f3e5f5
```

### Web Dashboard Components

```mermaid
graph TB
    subgraph "Web Dashboard Container"
        subgraph "Application Shell"
            AppRouter[App Router<br/>React Router<br/>Route Management]
            AuthProvider[Auth Provider<br/>JWT Management<br/>Session Handling]
            StateManager[State Manager<br/>Redux/Zustand<br/>Global State]
        end
        
        subgraph "Analysis Management"
            AnalysisForm[Analysis Form<br/>Configuration UI<br/>Validation]
            ProgressTracker[Progress Tracker<br/>Real-time Updates<br/>WebSocket Integration]
            ResultsViewer[Results Viewer<br/>Issue Browser<br/>Filtering & Search]
        end
        
        subgraph "Visualization Components"
            DependencyGraph[Dependency Graph<br/>D3.js/Cytoscape<br/>Interactive Visualization]
            MetricsDashboard[Metrics Dashboard<br/>Charts & Graphs<br/>Real-time Updates]
            ReportGenerator[Report Generator<br/>Export Functionality<br/>Multiple Formats]
        end
        
        subgraph "AI Integration UI"
            AIInsightsPanel[AI Insights Panel<br/>Explanation Display<br/>Suggestion Interface]
            ContextViewer[Context Viewer<br/>Code Highlighting<br/>Cross-references]
            FeedbackSystem[Feedback System<br/>Quality Ratings<br/>User Preferences]
        end
        
        subgraph "System Management"
            ConfigEditor[Config Editor<br/>Settings Management<br/>Validation]
            UserManagement[User Management<br/>RBAC Interface<br/>Permissions]
            SystemMonitor[System Monitor<br/>Health Dashboard<br/>Alert Display]
        end
    end
    
    %% Component Relationships
    AppRouter --> AuthProvider
    AuthProvider --> StateManager
    StateManager --> AnalysisForm
    StateManager --> ResultsViewer
    StateManager --> MetricsDashboard
    StateManager --> ConfigEditor
    
    AnalysisForm --> ProgressTracker
    ProgressTracker --> ResultsViewer
    ResultsViewer --> DependencyGraph
    ResultsViewer --> AIInsightsPanel
    
    AIInsightsPanel --> ContextViewer
    AIInsightsPanel --> FeedbackSystem
    
    MetricsDashboard --> SystemMonitor
    ConfigEditor --> UserManagement
    
    ReportGenerator --> ResultsViewer
    ReportGenerator --> DependencyGraph
    ReportGenerator --> AIInsightsPanel
    
    style AppRouter fill:#e8f5e8
    style AuthProvider fill:#e8f5e8
    style StateManager fill:#e8f5e8
    style AnalysisForm fill:#fff3e0
    style ProgressTracker fill:#fff3e0
    style ResultsViewer fill:#fff3e0
    style DependencyGraph fill:#e3f2fd
    style MetricsDashboard fill:#e3f2fd
    style ReportGenerator fill:#e3f2fd
    style AIInsightsPanel fill:#fce4ec
    style ContextViewer fill:#fce4ec
    style FeedbackSystem fill:#fce4ec
    style ConfigEditor fill:#f3e5f5
    style UserManagement fill:#f3e5f5
    style SystemMonitor fill:#f3e5f5
```

## Level 4: Code Diagrams

### Anti-Pattern Detector Code Structure

```mermaid
classDiagram
    class AnalysisDetector {
        <<interface>>
        +detect_issues(context: AnalysisContext) ArchitecturalIssue[]
        +name() string
        +version() string
        +supported_languages() Language[]
    }
    
    class GodObjectDetector {
        -config: GodObjectConfig
        -complexity_calculator: ComplexityCalculator
        -metrics_collector: MetricsCollector
        +new(config: GodObjectConfig) GodObjectDetector
        +detect_issues(context: AnalysisContext) ArchitecturalIssue[]
        -analyze_class_metrics(class_node: ASTNode) ClassMetrics
        -calculate_complexity_score(metrics: ClassMetrics) f64
        -generate_issue(class_node: ASTNode, metrics: ClassMetrics) ArchitecturalIssue
    }
    
    class DeadCodeDetector {
        -config: DeadCodeConfig
        -symbol_table: GlobalSymbolTable
        -call_graph: CallGraph
        +new(config: DeadCodeConfig) DeadCodeDetector
        +detect_issues(context: AnalysisContext) ArchitecturalIssue[]
        -build_symbol_table(files: Vec~FileInfo~) GlobalSymbolTable
        -analyze_usage_patterns(symbol_table: GlobalSymbolTable) UsageReport
        -identify_unused_symbols(usage_report: UsageReport) Vec~UnusedSymbol~
    }
    
    class TightCouplingDetector {
        -config: TightCouplingConfig
        -dependency_analyzer: DependencyAnalyzer
        +new(config: TightCouplingConfig) TightCouplingDetector
        +detect_issues(context: AnalysisContext) ArchitecturalIssue[]
        -analyze_coupling_metrics(dep_graph: DependencyGraph) CouplingMetrics
        -calculate_coupling_strength(from: Module, to: Module) f64
        -identify_tight_coupling(metrics: CouplingMetrics) Vec~CouplingIssue~
    }
    
    class DetectorRegistry {
        -detectors: HashMap~String, Box~dyn AnalysisDetector~~
        -plugin_manager: PluginManager
        +new() DetectorRegistry
        +register_detector(detector: Box~dyn AnalysisDetector~) Result~()~
        +get_detector(name: String) Option~&dyn AnalysisDetector~
        +run_all_detectors(context: AnalysisContext) Vec~ArchitecturalIssue~
        +load_plugin_detectors(plugin_dir: Path) Result~()~
    }
    
    class AnalysisContext {
        +files: Vec~FileInfo~
        +ast_provider: Box~dyn ASTProvider~
        +symbol_table: GlobalSymbolTable
        +dependency_graph: DependencyGraph
        +config: AnalysisConfig
        +ai_service: Option~AIService~
    }
    
    class ArchitecturalIssue {
        +id: String
        +issue_type: AntiPatternType
        +severity: Severity
        +file_path: String
        +line_number: Option~u32~
        +description: String
        +ai_explanation: Option~String~
        +ai_suggestion: Option~String~
        +metadata: HashMap~String, Value~
        +confidence_score: f64
    }
    
    AnalysisDetector <|-- GodObjectDetector
    AnalysisDetector <|-- DeadCodeDetector
    AnalysisDetector <|-- TightCouplingDetector
    DetectorRegistry --> AnalysisDetector
    AnalysisDetector --> AnalysisContext
    AnalysisDetector --> ArchitecturalIssue
```

### AI Service Integration Code Structure

```mermaid
classDiagram
    class AIProvider {
        <<interface>>
        +generate_explanation(context: AIContext) Result~String~
        +generate_suggestion(issue: ArchitecturalIssue) Result~String~
        +health_check() Result~HealthStatus~
        +get_provider_info() ProviderInfo
    }
    
    class OllamaProvider {
        -client: OllamaClient
        -model: String
        -config: OllamaConfig
        +new(config: OllamaConfig) Result~OllamaProvider~
        +generate_explanation(context: AIContext) Result~String~
        +generate_suggestion(issue: ArchitecturalIssue) Result~String~
        -build_prompt(template: PromptTemplate, context: AIContext) String
        -parse_response(response: String) Result~AIResponse~
    }
    
    class OpenAIProvider {
        -client: OpenAIClient
        -api_key: String
        -model: String
        -config: OpenAIConfig
        +new(config: OpenAIConfig) Result~OpenAIProvider~
        +generate_explanation(context: AIContext) Result~String~
        +generate_suggestion(issue: ArchitecturalIssue) Result~String~
        -handle_rate_limiting(error: OpenAIError) Result~()~
    }
    
    class AIService {
        -providers: Vec~Box~dyn AIProvider~~
        -fallback_strategy: FallbackStrategy
        -response_cache: ResponseCache
        -context_builder: ContextBuilder
        +new(config: AIServiceConfig) AIService
        +explain_issue(issue: ArchitecturalIssue) Result~String~
        +suggest_refactoring(issue: ArchitecturalIssue) Result~String~
        +recognize_patterns(code: String) Result~Vec~Pattern~~
        -try_providers(request: AIRequest) Result~AIResponse~
        -cache_response(request: AIRequest, response: AIResponse)
    }
    
    class ContextBuilder {
        -project_metadata: ProjectMetadata
        -coding_standards: CodingStandards
        +build_context(issue: ArchitecturalIssue, code: String) AIContext
        -extract_surrounding_code(file: String, line: u32) String
        -build_dependency_context(issue: ArchitecturalIssue) DependencyContext
        -generate_context_summary(context: AIContext) String
    }
    
    class PromptTemplate {
        -template: String
        -variables: HashMap~String, String~
        +load(template_name: String) Result~PromptTemplate~
        +render(context: AIContext) String
        -substitute_variables(template: String, context: AIContext) String
        -validate_template(template: String) Result~()~
    }
    
    class ResponseCache {
        -cache: LruCache~String, AIResponse~
        -ttl: Duration
        +new(capacity: usize, ttl: Duration) ResponseCache
        +get(key: String) Option~AIResponse~
        +insert(key: String, response: AIResponse)
        +invalidate(pattern: String)
        -generate_cache_key(request: AIRequest) String
    }
    
    class FallbackStrategy {
        +handle_provider_failure(error: AIError, providers: Vec~Box~dyn AIProvider~~) Option~Box~dyn AIProvider~~
        +should_retry(error: AIError) bool
        +get_retry_delay(attempt: u32) Duration
        +use_cached_response(request: AIRequest, cache: ResponseCache) Option~AIResponse~
    }
    
    AIProvider <|-- OllamaProvider
    AIProvider <|-- OpenAIProvider
    AIService --> AIProvider
    AIService --> ContextBuilder
    AIService --> ResponseCache
    AIService --> FallbackStrategy
    ContextBuilder --> PromptTemplate
```

## Deployment Diagrams

### Production Deployment Architecture

```mermaid
graph TB
    subgraph "Load Balancer Tier"
        LB[Load Balancer<br/>HAProxy/NGINX<br/>SSL Termination]
    end
    
    subgraph "Application Tier"
        subgraph "App Server 1"
            API1[Uveddi API<br/>Port 8080]
            WS1[WebSocket<br/>Port 8080]
            Metrics1[Metrics<br/>Port 9090]
        end
        
        subgraph "App Server 2"
            API2[Uveddi API<br/>Port 8080]
            WS2[WebSocket<br/>Port 8080]
            Metrics2[Metrics<br/>Port 9090]
        end
        
        subgraph "App Server 3"
            API3[Uveddi API<br/>Port 8080]
            WS3[WebSocket<br/>Port 8080]
            Metrics3[Metrics<br/>Port 9090]
        end
    end
    
    subgraph "Service Tier"
        Rendering[Rendering Service<br/>Node.js Cluster<br/>Ports 3001-3004]
        
        subgraph "AI Services"
            Ollama1[Ollama Instance 1<br/>llama3.2:latest]
            Ollama2[Ollama Instance 2<br/>codellama:latest]
        end
    end
    
    subgraph "Data Tier"
        subgraph "Database Cluster"
            DBPrimary[(PostgreSQL Primary<br/>Port 5432)]
            DBReplica1[(PostgreSQL Replica 1<br/>Port 5432)]
            DBReplica2[(PostgreSQL Replica 2<br/>Port 5432)]
        end
        
        Redis[(Redis Cluster<br/>Cache & Queue<br/>Ports 6379-6381)]
        
        Storage[("Shared Storage<br/>NFS/GlusterFS<br/>Analysis Cache")]
    end
    
    subgraph "Monitoring Tier"
        Prometheus[Prometheus<br/>Metrics Collection<br/>Port 9090]
        Grafana[Grafana<br/>Visualization<br/>Port 3000]
        AlertManager[Alert Manager<br/>Notification<br/>Port 9093]
    end
    
    %% Load Balancer Connections
    Internet([Internet]) --> LB
    LB --> API1
    LB --> API2
    LB --> API3
    
    %% Application Tier Connections
    API1 --> Rendering
    API2 --> Rendering
    API3 --> Rendering
    
    API1 --> Ollama1
    API1 --> Ollama2
    API2 --> Ollama1
    API2 --> Ollama2
    API3 --> Ollama1
    API3 --> Ollama2
    
    %% Database Connections
    API1 --> DBPrimary
    API2 --> DBPrimary
    API3 --> DBPrimary
    
    API1 --> DBReplica1
    API2 --> DBReplica1
    API3 --> DBReplica2
    
    DBPrimary --> DBReplica1
    DBPrimary --> DBReplica2
    
    %% Cache Connections
    API1 --> Redis
    API2 --> Redis
    API3 --> Redis
    Rendering --> Redis
    
    %% Storage Connections
    API1 --> Storage
    API2 --> Storage
    API3 --> Storage
    
    %% Monitoring Connections
    Metrics1 --> Prometheus
    Metrics2 --> Prometheus
    Metrics3 --> Prometheus
    Prometheus --> Grafana
    Prometheus --> AlertManager
    
    style LB fill:#e8f5e8
    style API1 fill:#e3f2fd
    style API2 fill:#e3f2fd
    style API3 fill:#e3f2fd
    style Rendering fill:#fff3e0
    style Ollama1 fill:#fce4ec
    style Ollama2 fill:#fce4ec
    style DBPrimary fill:#f3e5f5
    style DBReplica1 fill:#f3e5f5
    style DBReplica2 fill:#f3e5f5
    style Redis fill:#fff8e1
    style Storage fill:#e8f5e8
```

### Container Orchestration (Kubernetes)

```mermaid
graph TB
    subgraph "Kubernetes Cluster"
        subgraph "Ingress"
            Ingress[NGINX Ingress<br/>SSL/TLS Termination<br/>Load Balancing]
        end
        
        subgraph "Application Namespace"
            subgraph "Uveddi API Deployment"
                APIPod1[Uveddi API Pod 1<br/>CPU: 2 cores<br/>Memory: 4Gi]
                APIPod2[Uveddi API Pod 2<br/>CPU: 2 cores<br/>Memory: 4Gi]
                APIPod3[Uveddi API Pod 3<br/>CPU: 2 cores<br/>Memory: 4Gi]
            end
            
            APIService[Uveddi API Service<br/>ClusterIP<br/>Port 8080]
            
            subgraph "Rendering Deployment"
                RenderPod1[Rendering Pod 1<br/>CPU: 1 core<br/>Memory: 2Gi]
                RenderPod2[Rendering Pod 2<br/>CPU: 1 core<br/>Memory: 2Gi]
            end
            
            RenderService[Rendering Service<br/>ClusterIP<br/>Port 3001]
        end
        
        subgraph "AI Namespace"
            subgraph "Ollama Deployment"
                OllamaPod1[Ollama Pod 1<br/>GPU: 1x NVIDIA T4<br/>Memory: 8Gi]
                OllamaPod2[Ollama Pod 2<br/>GPU: 1x NVIDIA T4<br/>Memory: 8Gi]
            end
            
            OllamaService[Ollama Service<br/>ClusterIP<br/>Port 11434]
        end
        
        subgraph "Data Namespace"
            subgraph "PostgreSQL StatefulSet"
                PostgresPod[PostgreSQL Pod<br/>CPU: 4 cores<br/>Memory: 8Gi<br/>Storage: 500Gi]
            end
            
            PostgresService[PostgreSQL Service<br/>ClusterIP<br/>Port 5432]
            
            subgraph "Redis Deployment"
                RedisPod[Redis Pod<br/>CPU: 2 cores<br/>Memory: 4Gi<br/>Storage: 100Gi]
            end
            
            RedisService[Redis Service<br/>ClusterIP<br/>Port 6379]
        end
        
        subgraph "Monitoring Namespace"
            PrometheusPod[Prometheus Pod<br/>CPU: 2 cores<br/>Memory: 8Gi<br/>Storage: 200Gi]
            GrafanaPod[Grafana Pod<br/>CPU: 1 core<br/>Memory: 2Gi]
            
            PrometheusService[Prometheus Service<br/>ClusterIP<br/>Port 9090]
            GrafanaService[Grafana Service<br/>ClusterIP<br/>Port 3000]
        end
        
        subgraph "Storage"
            PVC1[PVC: postgres-data<br/>ReadWriteOnce<br/>500Gi]
            PVC2[PVC: redis-data<br/>ReadWriteOnce<br/>100Gi]
            PVC3[PVC: prometheus-data<br/>ReadWriteOnce<br/>200Gi]
            PVC4[PVC: uveddi-cache<br/>ReadWriteMany<br/>1Ti]
        end
    end
    
    %% External Traffic
    Internet([Internet]) --> Ingress
    
    %% Ingress Routing
    Ingress --> APIService
    Ingress --> RenderService
    Ingress --> GrafanaService
    
    %% Service to Pod Connections
    APIService --> APIPod1
    APIService --> APIPod2
    APIService --> APIPod3
    
    RenderService --> RenderPod1
    RenderService --> RenderPod2
    
    OllamaService --> OllamaPod1
    OllamaService --> OllamaPod2
    
    %% Cross-Service Communications
    APIPod1 --> OllamaService
    APIPod2 --> OllamaService
    APIPod3 --> OllamaService
    
    APIPod1 --> PostgresService
    APIPod2 --> PostgresService
    APIPod3 --> PostgresService
    
    APIPod1 --> RedisService
    APIPod2 --> RedisService
    APIPod3 --> RedisService
    
    RenderPod1 --> RedisService
    RenderPod2 --> RedisService
    
    %% Storage Connections
    PostgresPod --> PVC1
    RedisPod --> PVC2
    PrometheusPod --> PVC3
    APIPod1 --> PVC4
    APIPod2 --> PVC4
    APIPod3 --> PVC4
    
    %% Monitoring Connections
    PrometheusPod --> APIService
    PrometheusPod --> RenderService
    PrometheusPod --> PostgresService
    PrometheusPod --> RedisService
    
    style Ingress fill:#e8f5e8
    style APIPod1 fill:#e3f2fd
    style APIPod2 fill:#e3f2fd
    style APIPod3 fill:#e3f2fd
    style RenderPod1 fill:#fff3e0
    style RenderPod2 fill:#fff3e0
    style OllamaPod1 fill:#fce4ec
    style OllamaPod2 fill:#fce4ec
    style PostgresPod fill:#f3e5f5
    style RedisPod fill:#fff8e1
    style PrometheusPod fill:#e8f5e8
    style GrafanaPod fill:#e8f5e8
```

## Architecture Decision Records

### ADR-001: Multi-Language AST Parsing with Tree-sitter

**Status**: Accepted

**Context**: Need to parse multiple programming languages (Rust, Python, JavaScript, TypeScript) with consistent AST representation.

**Decision**: Use Tree-sitter for parsing with language-specific grammars.

**Consequences**:
- ✅ Consistent parsing across languages
- ✅ High performance and accuracy
- ✅ Active community and grammar support
- ❌ Additional dependency complexity
- ❌ Grammar updates required for language changes

### ADR-002: WebAssembly Plugin System

**Status**: Accepted

**Context**: Need extensible architecture for custom detectors while maintaining security.

**Decision**: Implement WebAssembly-based plugin system with sandboxing.

**Consequences**:
- ✅ Language-agnostic plugin development
- ✅ Strong security isolation
- ✅ Near-native performance
- ❌ Additional runtime complexity
- ❌ Limited system API access

### ADR-003: AI Provider Abstraction

**Status**: Accepted

**Context**: Support multiple AI providers (local and cloud) with fallback strategies.

**Decision**: Create abstraction layer with pluggable providers and intelligent fallback.

**Consequences**:
- ✅ Provider independence
- ✅ Graceful degradation
- ✅ Cost optimization options
- ❌ Additional abstraction complexity
- ❌ Provider-specific feature limitations

### ADR-004: Event-Driven Analysis Pipeline

**Status**: Accepted

**Context**: Handle large codebases efficiently with progress tracking and cancellation.

**Decision**: Implement event-driven pipeline with async processing and backpressure.

**Consequences**:
- ✅ Scalable processing
- ✅ Real-time progress updates
- ✅ Resource efficiency
- ❌ Increased complexity
- ❌ Debugging challenges

### ADR-005: Microservices Architecture

**Status**: Accepted

**Context**: Enable independent scaling and technology choices for different concerns.

**Decision**: Separate core analysis, rendering, and AI services with clear interfaces.

**Consequences**:
- ✅ Independent deployment and scaling
- ✅ Technology diversity
- ✅ Fault isolation
- ❌ Network latency
- ❌ Distributed system complexity

## Conclusion

The Uveddi architecture follows modern software design principles with clear separation of concerns, scalability considerations, and maintainability focus. The C4 model provides a hierarchical view that helps both technical and non-technical stakeholders understand the system structure and relationships.

Key architectural strengths:
- **Modularity**: Clear component boundaries with well-defined interfaces
- **Scalability**: Horizontal and vertical scaling support
- **Extensibility**: Plugin system for custom functionality
- **Resilience**: Fallback strategies and graceful degradation
- **Observability**: Comprehensive monitoring and logging

This architecture supports Uveddi's mission to provide intelligent, scalable code analysis with AI-powered insights while maintaining performance, security, and operational excellence.