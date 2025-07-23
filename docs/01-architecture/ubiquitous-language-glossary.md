# Uveddi Ubiquitous Language Glossary

## Overview

This document establishes the standardized terminology and ubiquitous language for the Uveddi project. It serves as the authoritative reference for all domain concepts, architectural components, and technical terms used across the codebase, documentation, and team communication.

**Last Updated:** January 2025  
**Status:** Living Document  
**Maintainers:** Architecture Team

---

## Core Analysis Domain

### Analysis & Detection

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Detector** | A component that identifies specific architectural issues or anti-patterns in code | Use for all issue detection implementations | `DeadCodeDetector`, `GodObjectDetector` |
| **Extractor** | A component that extracts structured data from source code | Use for data extraction logic | `ComponentExtractor`, `DependencyExtractor` |
| **Engine** | A component responsible for execution and processing of complex operations | Use for orchestration and execution logic | `AnalysisEngine`, `PluginEngine` |
| **AntiPattern** | A recurring architectural pattern that negatively impacts code quality | Always singular, consistent casing | `AntiPattern`, not `AntiPatternType` |
| **ArchitecturalIssue** | A specific instance of a problem found during code analysis | Use for concrete issue instances | `ArchitecturalIssue.severity` |
| **AnalysisRun** | A complete execution session of code analysis | Use for analysis session tracking | `AnalysisRun.start_time` |
| **Metric** | A quantitative measurement of code quality or architectural properties | Use for all measurements | `ComplexityMetric`, `CouplingMetric` |

### Component Architecture

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Component** | A logical unit of code with defined boundaries and responsibilities | Use for architectural analysis units | `Component.dependencies` |
| **Module** | A language-specific organizational unit (file, package, namespace) | Reserve for language constructs | `RustModule`, `JavaScriptModule` |
| **Dependency** | A relationship where one component relies on another | Use for all inter-component relationships | `Dependency.source_component` |
| **Architecture** | The overall structural organization of a codebase | Use for system-level structure | `Architecture.component_graph` |
| **Node** | A vertex in a graph representation (used only in graph contexts) | Restrict to graph algorithms | `graph.nodes()` |

---

## AI & Intelligence Domain

### AI Integration

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **AiProvider** | A service that provides AI capabilities and model access | Use for all AI service implementations | `OllamaProvider`, `OpenAiProvider` |
| **AiInsight** | AI-generated analysis, suggestion, or explanation | Use for all AI outputs | `AiInsight.explanation` |
| **Prompt** | Input text or template sent to an AI model | Use for AI input processing | `PromptTemplate.render()` |
| **AiResponse** | Raw response data returned from an AI model | Use for unprocessed AI output | `AiResponse.content` |
| **Intelligence** | The overall AI capabilities and smart features | Use for AI feature categories | `ArchitecturalIntelligence` |

---

## Security & Access Control Domain

### Identity & Authentication

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **User** | Basic user identity information | Use for user data without auth context | `User.email` |
| **AuthenticatedUser** | User identity with authentication context and permissions | Use when authentication is verified | `AuthenticatedUser.roles` |
| **Session** | An authenticated period of user interaction | Use for login sessions | `Session.expires_at` |
| **Role** | A collection of permissions that can be assigned to users | Use for access level definitions | `Role.permissions` |
| **Permission** | A specific right to perform an action on a resource | Use for granular access rights | `Permission.resource_type` |
| **Principal** | The authenticated entity performing an action | Use in authorization contexts | `Principal.has_permission()` |

### Security Operations

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Authentication** | The process of verifying user identity | Use for identity verification | `authenticate_user()` |
| **Authorization** | The process of determining user permissions | Use for access control checks | `authorize_action()` |
| **Token** | A cryptographic proof of authentication or authorization | Use for auth tokens | `JwtToken`, `ApiToken` |
| **Credential** | Authentication information (password, key, etc.) | Use for auth data | `UserCredential` |

---

## Plugin & Extension Domain

### Plugin System

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Plugin** | An extension that adds functionality to the core system | Use for extension code | `Plugin.metadata` |
| **PluginEngine** | The runtime environment for executing plugins | Use for plugin execution | `PluginEngine.execute()` |
| **PluginManager** | Coordinates plugin lifecycle and registration | Use for plugin coordination | `PluginManager.load_plugin()` |
| **Extension** | A generic term for system extensibility | Use in abstract contexts | `ExtensionPoint` |
| **ResourceLimits** | Constraints on plugin execution (memory, CPU, time) | Use for plugin security | `ResourceLimits.max_memory` |
| **HostContext** | The environment and services available to plugins | Use for plugin-host interface | `HostContext.file_system` |

---

## Configuration & Settings Domain

### Configuration Management

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Config** | A container for configuration settings | Use for settings data structures | `AnalysisConfig`, `SecurityConfig` |
| **ConfigLoader** | Responsible for loading configuration from sources | Use for config loading logic | `ConfigLoader.from_file()` |
| **ConfigBuilder** | Constructs configuration objects programmatically | Use for config construction | `ConfigBuilder.with_option()` |
| **Settings** | User-customizable preferences and options | Use for user preferences | `UserSettings.theme` |
| **Environment** | Runtime environment variables and context | Use for env-specific config | `Environment.is_development()` |

---

## Data & Persistence Domain

### Data Management

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Repository** | An abstraction for data access and persistence | Use for data access patterns | `UserRepository.find_by_id()` |
| **Service** | A business logic abstraction that coordinates operations | Use for business logic coordination | `AnalysisService.run_analysis()` |
| **Entity** | A domain object with identity and lifecycle | Use for core business objects | `AnalysisEntity.id` |
| **ValueObject** | An immutable object defined by its attributes | Use for data without identity | `EmailAddress`, `FilePath` |
| **Aggregate** | A cluster of entities treated as a single unit | Use for transaction boundaries | `AnalysisAggregate` |

---

## Performance & Monitoring Domain

### Observability

| Term | Definition | Usage | Code Examples |
|------|------------|-------|---------------|
| **Metric** | A quantitative measurement for monitoring | Use for observability data | `ResponseTimeMetric` |
| **Telemetry** | Automated collection of performance and usage data | Use for monitoring systems | `TelemetryCollector` |
| **Trace** | A record of execution flow through the system | Use for distributed tracing | `TraceContext` |
| **Event** | A significant occurrence in the system | Use for event-driven patterns | `AnalysisCompletedEvent` |
| **Monitor** | A component that observes system behavior | Use for monitoring logic | `PerformanceMonitor` |

---

## Naming Conventions

### Rust-Specific Conventions

1. **Structs**: Use PascalCase for all struct names
   - ✅ `AnalysisEngine`, `ComponentExtractor`
   - ❌ `analysis_engine`, `componentExtractor`

2. **Fields**: Use snake_case for all field names
   - ✅ `component_id`, `analysis_result`
   - ❌ `componentId`, `analysisResult`

3. **Enums**: Use PascalCase for enum names and variants
   - ✅ `enum AntiPatternType { GodObject, TightCoupling }`
   - ❌ `enum anti_pattern_type { god_object, tight_coupling }`

4. **Traits**: Use descriptive names ending in purpose
   - ✅ `AnalysisDetector`, `ConfigurationLoader`
   - ❌ `Detector`, `Loader` (too generic)

5. **Functions**: Use snake_case with verb-noun pattern
   - ✅ `detect_anti_patterns()`, `extract_components()`
   - ❌ `detect()`, `extract()` (too generic)

### File and Module Conventions

1. **Module Names**: Use snake_case matching primary functionality
   - ✅ `anti_patterns.rs`, `dependency_graph.rs`
   - ❌ `antiPatterns.rs`, `dependencyGraph.rs`

2. **Directory Structure**: Organize by domain, not by technical layer
   - ✅ `analysis/detectors/`, `security/auth/`
   - ❌ `services/`, `models/` (generic technical grouping)

---

## Architectural Principles

### Domain-Driven Design Alignment

1. **Bounded Contexts**: Each major domain should use consistent terminology within its boundaries
2. **Context Mapping**: When crossing context boundaries, use explicit translation layers
3. **Shared Kernel**: Core concepts like `Component` and `AnalysisRun` are shared across contexts

### Anti-Patterns to Avoid

1. **Concept Overloading**: Don't use the same term for different concepts
   - ❌ Using "Service" for both HTTP services and business services
   - ✅ Use "HttpService" and "BusinessService" for clarity

2. **Synonym Proliferation**: Don't use multiple terms for the same concept
   - ❌ "Issue", "Problem", "Defect" for the same thing
   - ✅ Standardize on "ArchitecturalIssue"

3. **Technical Jargon Mixing**: Keep domain terms separate from technical implementation terms
   - ❌ "DatabaseUser" as a domain concept
   - ✅ "User" (domain) with "UserRepository" (technical)

---

## Enforcement Guidelines

### Code Review Checklist

- [ ] New types follow established naming conventions
- [ ] Domain terms align with glossary definitions
- [ ] No new synonyms introduced for existing concepts
- [ ] Technical terms don't leak into domain layer
- [ ] Comments use standardized terminology

### IDE Integration

Configure your IDE to:
1. Highlight non-standard terminology
2. Suggest standardized alternatives
3. Enforce naming conventions during development

### Documentation Standards

1. **API Documentation**: Must use glossary terms consistently
2. **README Files**: Reference glossary for domain concepts
3. **Architecture Decisions**: Justify any new terminology additions

---

## Evolution Process

### Adding New Terms

1. **Proposal**: Submit RFC with rationale and usage examples
2. **Review**: Architecture team evaluates consistency and necessity
3. **Trial**: Use in limited scope with team feedback
4. **Adoption**: Update glossary and begin enforcement

### Deprecating Terms

1. **Mark as deprecated** with migration path
2. **Create type aliases** for backward compatibility
3. **Update documentation** with timeline
4. **Remove** after deprecation period

### Version Control

- **Major Changes**: Require team consensus
- **Minor Additions**: Architecture team approval
- **Corrections**: Can be made directly with notification

---

## Related Resources

- [Domain-Driven Design Reference](../02-design/domain-driven-design.md)
- [API Design Guidelines](../03-api/design-guidelines.md)
- [Code Review Standards](../05-development/code-review-standards.md)
- [Architecture Decision Records](../04-decisions/)

---

*This glossary is a living document that evolves with the Uveddi project. All team members are responsible for maintaining consistency and suggesting improvements.*