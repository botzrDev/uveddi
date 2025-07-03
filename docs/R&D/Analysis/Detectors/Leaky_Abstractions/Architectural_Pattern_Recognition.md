
A Methodological Framework for Automated Recognition of Software Architectural Patterns and Conformance Verification


Introduction


Problem Statement

Software architecture serves as the fundamental blueprint for a system, defining its components, their relationships, and the principles governing its design and evolution. However, as software systems undergo continuous maintenance, adaptation, and feature enhancement, their implemented structure often deviates from the intended design. This phenomenon, known as architectural drift or erosion, is a pervasive challenge in software engineering.1 It occurs when development teams, often under pressure from deadlines or facing evolving requirements, make localized changes that violate the established architectural principles.3 Over time, these incremental deviations accumulate, leading to a system where the conceptual model in the code drifts apart from the original domain concepts.4
The consequences of unmanaged architectural drift are severe. They include the loss of critical non-functional qualities like extensibility and maintainability, an increase in system complexity, and the accumulation of architectural technical debt.5 This erosion renders architectural documentation unreliable, forcing developers to distrust it and instead rely on deciphering the complex and often convoluted source code itself.2 The process of understanding such legacy systems becomes akin to "deciphering ancient hieroglyphs" without a guide, making maintenance risky and innovation slow.6 To combat this degradation and enable effective architectural governance, a systematic and automated method for recovering the "as-is" architecture from the source code is not just beneficial, but essential.7

Objective

The primary objective of this report is to present a comprehensive, multi-signal methodological framework for the automated recognition of common software architectural patterns directly from source code artifacts. This framework moves beyond simple pattern matching to provide a robust, heuristic-based approach capable of handling the ambiguities and variations found in real-world codebases. The ultimate goal of this architectural recognition is to establish a foundation for automated architectural conformance verification, enabling the detection of abstraction boundary violations and the enforcement of architectural integrity throughout a system's lifecycle.

Methodology Overview

The methodology detailed herein is founded on the principle that no single indicator is sufficient to reliably identify a complex architectural pattern. Instead, a robust analysis must synthesize evidence from multiple sources within the code. The core of this approach relies on a combination of advanced static analysis techniques, primarily:
Dependency Graph Analysis: Constructing and analyzing the network of dependencies between code modules to understand the fundamental flow of control and data, which is the primary differentiator between architectural styles.
Abstract Syntax Tree (AST) Parsing: Performing granular inspection of the code's structure to extract nuanced details such as class inheritance, interface implementation, and naming conventions.
These techniques are used to build a set of heuristics for classifying codebases according to five major architectural pattern families: Layered (N-Tier), Clean Architecture, Hexagonal Architecture (Ports and Adapters), Model-View-Controller (MVC) / Model-View-Presenter (MVP) / Model-View-ViewModel (MVVM), and Domain-Driven Design (DDD) Bounded Contexts. The report will provide practical strategies and specific code examples for implementing these detection techniques in Rust, Python, and JavaScript, addressing the unique challenges and tooling available in each language ecosystem.

Section 1: Foundational Techniques for Architectural Recognition

The successful automation of architectural pattern detection hinges on a robust set of underlying analysis techniques. A system designed for this purpose cannot operate on a single source of information; the inherent flexibility and variability in how patterns are implemented demand a multi-faceted approach. This section establishes the foundational techniques that form the pillars of our proposed methodology: dependency graph analysis, Abstract Syntax Tree (AST) parsing, and a unified heuristic model that combines these signals to produce a probabilistic assessment of a codebase's architecture.

1.1 The Centrality of Dependency Analysis

The most fundamental and revealing characteristic of any software architecture is its dependency structure. The rules governing which parts of a system can communicate with or depend on other parts are what give an architecture its shape and enforce its constraints. Therefore, the construction and analysis of a dependency graph is the primary and most powerful technique for automated architectural recognition.9

Core Concept

A dependency graph is a directed graph where the nodes represent distinct code modules (e.g., files, classes, packages, or crates) and the directed edges represent dependencies between them (e.g., an import statement in Python, a use statement in Rust, or a function call). This graph makes the abstract flow of control and coupling within a system tangible and machine-readable. By analyzing the topology of this graph—the direction of edges, the presence of cycles, and the clustering of nodes—we can infer the high-level architectural pattern. For example, a strictly layered architecture will manifest as a Directed Acyclic Graph (DAG) with no "upward" pointing edges, whereas a Clean or Hexagonal architecture will show dependencies converging inward toward a central core.

Implementation

The process begins by traversing the codebase and identifying all code modules that will serve as nodes in the graph. For each module, static analysis is used to parse its source code and identify all explicit dependencies on other modules. These dependencies become the edges. For instance, the Python statement from services.user_service import UserService in a controller file would create a directed edge from the controller node to the user_service node.
Libraries and tools exist to facilitate this process. In Python, libraries like networkx or igraph are excellent for constructing and analyzing graph structures, while tools like pydeps can generate dependency visualizations.12 In Rust, the
cargo metadata command provides crate-level dependency information, which can be visualized with tools like krates, while intra-crate module dependencies must be parsed from the source code itself.13

1.2 Granular Inspection via Abstract Syntax Trees (ASTs)

While the dependency graph reveals that a dependency exists between two modules, it does not explain how or why it exists. To gain this deeper, more granular understanding, we must turn to Abstract Syntax Trees (ASTs). An AST is a tree representation of the abstract syntactic structure of source code, generated by a parser during the compilation process.15 Every element of the code—a variable declaration, a function call, a class definition—corresponds to a node in the tree.

Application in Heuristics

By traversing the AST, an automated tool can extract highly specific structural and semantic information that is crucial for building robust detection heuristics. This moves the analysis beyond simple import checking to a more sophisticated level of pattern recognition. Key information that can be extracted via ASTs includes:
Interface Implementation and Inheritance: Identifying which classes implement a specific interface (in languages that support them, like TypeScript or Java) or inherit from a base class (in Python or C++). This is the cornerstone of detecting the Ports and Adapters pattern, where an "Adapter" class implements a "Port" interface.18
Naming Conventions: Extracting the names of classes, functions, files, and variables. This data is a powerful input for heuristics that look for common naming patterns, such as files ending in _controller.py, classes named UserService, or folders named domain.19
Method Signatures and Calls: Analyzing the parameters and return types of functions, and identifying specific method calls. This helps in understanding data flow and can distinguish between different types of interactions, for example, differentiating a simple data query from a command that modifies state.
Annotation and Decorator Analysis: In languages like Python and TypeScript, decorators (or annotations) provide metadata that can be a strong architectural signal. For example, a @Controller() decorator in NestJS is a clear indicator of a controller component.
Each language ecosystem has mature libraries for AST manipulation. Rust provides the powerful syn crate for parsing Rust code into a manipulable syntax tree.18 The Python standard library includes the
ast module, which can parse any Python source file into an AST that can be traversed with visitors.19 For JavaScript and TypeScript,
Esprima is a foundational library, with modern alternatives like the Babel parser or Acorn providing ESTree-compliant ASTs.25

1.3 A Unified Heuristic Model: Combining Dependency, Naming, and Structural Analysis

The central thesis of this report is that automated architectural detection cannot be a deterministic process. Real-world software rarely adheres perfectly to a single pattern. Developers may use non-standard naming conventions, mix patterns, or allow architectural drift to erode the original design.1 A tool that relies on a single signal, such as checking for a folder named
controllers, will be brittle and prone to both false positives and false negatives.
This ambiguity necessitates a shift from a rule-based system to a probabilistic, heuristic-based scoring engine. Such an engine does not provide a binary "yes/no" answer but instead calculates a confidence score for each potential architectural pattern. This approach gracefully handles the messiness of real-world code and provides a more nuanced and useful analysis. This perspective is supported by academic research, which has shown that combining multiple analysis techniques, such as static and dynamic analysis, yields more accurate pattern detection results, especially in complex or legacy systems.31 While this report focuses on static analysis, we emulate this combined approach by using multiple, orthogonal static signals.
The proposed heuristic model is built on three pillars of evidence, weighted by their reliability:
Dependency Analysis (Strongest Signal): This is the most objective and reliable indicator. The analysis involves building the full dependency graph and verifying its topology against the strict dependency rules of each architectural pattern (e.g., "dependencies must point inwards" for Clean Architecture, "dependencies must flow downwards" for Layered Architecture). Violations of these rules strongly penalize a pattern's confidence score.
Naming Conventions (Strong Signal): This involves analyzing the names of directories, files, classes, and functions against a dictionary of conventional terms associated with each pattern. For example, the presence of folders like core, domain, application, and infrastructure is a strong heuristic for Clean or Hexagonal architecture.33 While powerful, this signal is less reliable than dependency analysis because conventions vary and can be misleading.
Structural Analysis (Supporting Signal): This involves using ASTs to identify other structural clues. Examples include the physical separation of interface definitions from their concrete implementations, the presence of Data Transfer Objects (DTOs) used for inter-layer communication, or the use of specific framework features (like dependency injection containers) that align with a particular pattern.
By combining scores from these three pillars, the system can produce a ranked list of potential patterns. For example, the output might be: "Confidence Score for Clean Architecture: 85%; Confidence Score for Layered Architecture: 40%." This probabilistic result is far more valuable to an architect than a simple, and likely incorrect, deterministic classification. It accurately reflects the state of the codebase, highlighting areas of strong conformance alongside areas of ambiguity or architectural drift.
The following table provides a high-level comparison of the dependency rules that form the core of the strongest analysis signal for each major architectural pattern. These rules are the fundamental constraints that the automated tool will seek to verify.
Pattern
Core Principle
Primary Dependency Rule
Example Violation
Layered (N-Tier)
Separation of Concerns into hierarchical layers.
Dependencies flow strictly downwards (e.g., Presentation → Business → Data). Higher layers must not depend on lower layers.
A Repository class (Data Layer) importing and using a Service class (Business Layer).
Clean Architecture
Isolation of business logic from external details.
Dependencies flow strictly inwards, toward the center (Entities). Outer layers depend on inner layers.
A UseCase (Application Layer) importing a specific database framework (e.g., mongoose) from the Frameworks layer.
Hexagonal (Ports/Adapters)
Isolation of the application core from infrastructure.
Dependencies flow inwards. Adapters depend on Ports defined in the core; the core does not depend on Adapters.
A DomainService (Core) directly importing and instantiating a PostgresRepository (Adapter).
MVC
Separation of UI concerns.
Controller handles input and manipulates the Model. View renders the Model. View and Model are decoupled.
The View component directly calling a method on the Model to update data, bypassing the Controller.
MVP
Complete decoupling of View and Model.
Presenter mediates all interaction. The View is passive and delegates all events to the Presenter.
The View component containing logic that directly manipulates data from the Model.
MVVM
Separation of UI state and logic from the UI.
View binds to the ViewModel. ViewModel is unaware of the View. ViewModel updates the Model.
The ViewModel holding a direct reference to a UI element or a View instance.


Section 2: Detecting Inward-Flow Architectures: The Dependency Inversion Principle in Practice

A modern class of software architectures has emerged around a powerful central idea: the strict separation of core business logic from external concerns like databases, user interfaces, and third-party frameworks. This separation is achieved through the Dependency Inversion Principle (DIP), which dictates that high-level modules should not depend on low-level modules; both should depend on abstractions. Architectures like Hexagonal (Ports and Adapters), Clean, and Domain-Driven Design (DDD) all implement this principle, resulting in a characteristic dependency flow that points inwards toward the domain. This section details the methods for detecting these patterns by focusing on their shared dependency rule and their unique structural fingerprints.

2.1 The Common Principle: The Dependency Rule as the Core Differentiator

The single most important rule for identifying this family of architectures is The Dependency Rule: source code dependencies must only point inwards.9 The application's core, which contains the most stable and valuable business logic (Entities and Use Cases), must remain independent and ignorant of the volatile, external details that make the application function, such as the web framework, database library, or UI rendering engine.35
This inversion of traditional dependencies is accomplished by defining abstract interfaces, often called Ports, at the boundaries of the inner layers. These interfaces are contracts that specify what the core logic needs, such as "a way to save a user" or "a way to send a notification." The outer layers then provide concrete implementations of these interfaces, called Adapters, which handle the specific technology (e.g., a PostgreSQL implementation for saving a user, or an SMTP implementation for sending a notification). At runtime, these concrete adapters are injected into the core, allowing the business logic to function without ever holding a direct compile-time dependency on the infrastructure code.11
From an automated detection standpoint, this provides a clear, verifiable mandate. The primary strategy is to:
Partition the codebase into "core" and "infrastructure" modules using heuristics like directory and file naming.
Construct a dependency graph from all import, use, or require statements.
Systematically verify that no module classified as "core" has a direct dependency on any module classified as "infrastructure". Any such dependency is a direct violation of the architecture's foundational principle.

2.2 Hexagonal Architecture (Ports and Adapters)

Hexagonal Architecture, also known as Ports and Adapters, is the archetypal inward-flow pattern. It visualizes the application as a central hexagon (the core) that interacts with the outside world exclusively through ports, which are plugged into by adapters.42

2.2.1 Structural and Dependency Fingerprints

The key structural elements to identify are:
The Application Core (The Hexagon): A set of modules containing the application's domain logic and business rules, completely isolated from external technology.37
Ports: These are technology-agnostic interfaces defined inside the application core. They represent the boundary and define the contract for interaction.37 There are two types:
Primary/Driving Ports: APIs of the application core that are called by the outside world to drive the application (e.g., an interface for a use case like CreateOrderService).
Secondary/Driven Ports: Interfaces required by the application core to reach out to external services (e.g., a OrderRepository interface for persistence).
Adapters: These are the concrete implementations of the ports and reside outside the application core. They are the glue that connects a specific technology to a port.37
Primary/Driving Adapters: Code that drives the application by calling a primary port. A REST controller that handles an HTTP request and calls the CreateOrderService is a primary adapter.
Secondary/Driven Adapters: Code that is driven by the application by implementing a secondary port. A PostgresOrderRepository class that implements the OrderRepository interface is a secondary adapter.
The dependency flow is unambiguous: Adapters depend on Ports, but the Core (which defines the Ports) is completely independent of the Adapters.11

2.2.2 Directory and Naming Heuristics

While not strictly enforced, strong conventions have emerged for structuring hexagonal projects:
Directory Structure: Code is often organized into high-level directories that reflect the architecture. Common names for the core include domain, core, or app. The external components are typically placed in infrastructure, adapters, or technology-specific folders like web or persistence.33
File and Class Naming:
Ports: Since ports are interfaces or traits, they are often named with a suffix like Port, Service, or Repository. A highly descriptive convention is the For pattern, such as ForPlacingOrders (primary port) or ForStoringUsers (secondary port).47 More common, however, are names like
OrderService (for the interface) and UserRepository (for the interface).48
Adapters: Adapter names typically incorporate the technology they are adapting. For example, PlaceOrderApiController (primary adapter) or MySqlUserRepository (secondary adapter).48

2.2.3 Automated Detection Strategy with Language-Specific Examples

The strategy combines dependency validation with these structural and naming heuristics.
Rust Example:
Structure: A common Rust project structure separates the logic into a domain library crate and an infra binary or library crate.50
Detection:
Identify the domain and infra crates.
Verify that the [dependencies] section of the domain crate's Cargo.toml file does not list the infra crate.
Within the domain crate, use syn to parse the source and identify trait definitions. These are the Ports. Heuristics can look for trait names ending in Port or Repository.
Within the infra crate, use syn to find struct definitions that have an impl block for one of the port traits (e.g., impl UserRepository for PostgresRepo). These are the Adapters.
A web framework handler, for instance in Axum, acts as a primary adapter. It receives a request, calls a use case (defined by a primary port), and returns a response. The use case itself is injected into the Axum state as a trait object (Arc<dyn OrderService>).51 The analysis of the
fteychene/blueprint-hexagonal-rust repository confirms this structure, with a clear separation of domain (defining ports like TaskStoragePort) and infra (providing adapters like SqliteStorageAdapter and InMemoryStorageAdapter).50
Python Example:
Structure: Projects are often organized with a domain package and an infrastructure package.46
Detection:
Identify the domain and infrastructure directories.
Use ast.walk to traverse all Python files and build a dependency graph. Confirm that no file within the domain package contains an import statement that resolves to a module within the infrastructure package.
Within the domain package, use ast to find classes that inherit from abc.ABC. These are the Ports.53
Within the infrastructure package, find classes that inherit from these abstract base classes. These are the Adapters.
The analysis of serfer2/flask-hexagonal-architecture-api shows this pattern clearly: domain/repositories/report_repository_interface.py defines the port, and infrastructure/repositories/report_repository.py provides the concrete adapter implementation.46
JavaScript/TypeScript Example:
Structure: A typical structure includes a src/domain folder and a src/infrastructure folder.54
Detection:
Identify the domain and infrastructure directories.
Parse all .ts files using a parser like the Babel parser to create an AST.
Traverse the AST to find all ImportDeclaration nodes and build the dependency graph. Verify that no file in domain imports from infrastructure.
In the domain folder, identify interface declarations. These are the Ports.
In the infrastructure folder, identify class declarations that use an implements clause to implement one of the port interfaces. These are the Adapters.
The juanm4/hexagonal-architecture-frontend example demonstrates this by defining repository interfaces in domain/repositories and providing concrete implementations in infrastructure/repositories, with views in infrastructure/views acting as primary adapters.54

2.3 Clean Architecture

Clean Architecture can be seen as a more detailed and prescriptive evolution of Hexagonal Architecture. It adopts the same core principle of inward-pointing dependencies but further organizes the application core into a series of concentric, hierarchical layers.40

2.3.1 Differentiating from Hexagonal: Identifying Granular Inner Layers

The key difference that allows for automated differentiation between Clean and Hexagonal architecture is the presence of these well-defined inner layers. While a Hexagonal architecture might have a monolithic domain or core, a Clean Architecture will partition this core further.36 The canonical layers, from innermost to outermost, are:
Entities: These are the core business objects, encapsulating enterprise-wide business rules and data structures. They are the most stable part of the application and have zero dependencies on any other layer.34
Use Cases (or Interactors): This layer contains application-specific business logic. A use case orchestrates the flow of data to and from the Entities to achieve a specific application goal (e.g., RegisterUser). This layer depends only on the Entities layer.41
Interface Adapters: This layer acts as a set of converters. It transforms data from a format convenient for the Use Cases and Entities to a format convenient for external agencies like the database or the web, and vice versa. This layer includes presenters, conceptual controllers, and gateways (which are often repository implementations).55 This layer depends on the Use Cases layer.
Frameworks & Drivers: This is the outermost layer, containing all the details: the UI, the database, the web framework, etc. This layer is the most volatile.35
The detection of Clean Architecture is therefore a refinement of Hexagonal detection. After confirming the primary core/infrastructure split, the tool must then attempt to partition the "core" into these sub-layers and verify that the dependency rule holds between them as well (e.g., Use Cases depend on Entities, but Entities do not depend on Use Cases).

2.3.2 Directory and Naming Heuristics

Clean Architecture implementations often have very indicative directory and naming schemes:
Directory Structure: A common top-level structure is domain, application (or use_cases), and infrastructure. The domain directory might contain an entities sub-directory, and application would contain the use case files.33
File and Class Naming:
Entities: Named with simple nouns representing the domain concept, e.g., User.ts, Product.py.
Use Cases: Often explicitly named with a verb phrase and a suffix like UseCase or Interactor, e.g., CreateUserUseCase.ts, GetProductDetails.py.29
Interface Adapters: Repository interfaces are defined in the application layer (e.g., IUserRepository.ts), while their concrete implementations (e.g., MongoUserRepository.ts) reside in the infrastructure layer.

2.3.3 Automated Detection Strategy with Language-Specific Examples

The strategy is hierarchical: first, confirm a Hexagonal-style dependency inversion between a "core" and "infrastructure". Then, analyze the internal structure of the "core" for the Clean layers.
Rust Example:
Structure: A project might be organized into domain, application, and infrastructure crates.59
Detection:
Verify the crate dependencies in Cargo.toml: application depends on domain, infrastructure depends on application, and there are no outward dependencies.
Analyze the domain crate for simple structs with business logic (Entities).
Analyze the application crate for modules named use_cases or files with the UseCase suffix. Verify that these modules depend on the domain crate but not the infrastructure crate.
The flosse/clean-architecture-with-rust example shows a structure with domain, application, and adapter/infrastructure circles, which is a strong signal.60
Python Example:
Structure: A highly indicative structure would be src/domain/entities, src/domain/use_cases, src/adapter, and src/data.61
Detection:
Identify these packages.
Build the dependency graph and validate that modules in use_cases import from entities, but not vice-versa.
Confirm that no module in domain imports from adapter or data.
The heumsi/python-clean-architecture-example provides a perfect template for this, with a clear separation into domain (containing entity and use_case sub-packages), adapter, and data (for the repository implementation).61
JavaScript/TypeScript Example:
Structure: A common structure in front-end or Node.js applications includes domain (with entities, repositories interfaces, use-cases), data or infrastructure (with datasources, services), and presentation or delivery (with controllers, views).58
Detection:
Identify these directories and their sub-directories.
Verify the inward dependency flow across these layers.
In a NestJS application, this pattern is often implemented using its module and dependency injection system. A UseCase would be an @Injectable() service. Repository interfaces would be defined as injection tokens, and the concrete implementations (e.g., a MongoRepository) would be provided in an infrastructure module and injected into the use cases.64 The
askides/clean-architecture-react example adapts this for the front-end, separating domain (Models, Repositories, UseCases) from data (DataSources) and presentation (React components).62

2.4 Domain-Driven Design (DDD) Bounded Contexts

Detecting Domain-Driven Design (DDD) boundaries presents a unique and significant challenge for automated tools because Bounded Contexts are primarily logical constructs, not necessarily physical ones.67 A Bounded Context defines a boundary within which a particular domain model and its associated
Ubiquitous Language are consistent and have unambiguous meaning.68 A single monolithic repository can contain multiple Bounded Contexts. Therefore, detection cannot rely on simple structural markers alone but must infer these logical boundaries from deeper code relationships.

2.4.1 Heuristics for Context Detection: Cohesion, Coupling, and Ubiquitous Language Proxies

Since direct detection is difficult, we must rely on a set of strong heuristics that act as proxies for identifying Bounded Contexts.
High Cohesion and Loose Coupling: A well-designed Bounded Context should be internally highly cohesive, with its components densely interconnected. It should also be loosely coupled with other contexts, communicating only through well-defined, narrow interfaces.68 This property can be measured directly from the module dependency graph. A subgraph with high internal edge density and low external edge density is a strong candidate for a Bounded Context.
Directory Structure as a Proxy: In well-organized projects, developers often use the physical directory structure to represent logical boundaries. Top-level directories within the source folder frequently correspond to Bounded Contexts. For example, in an e-commerce application, one might find src/sales, src/shipping, and src/inventory directories, each representing a distinct context.69
Model Polysemy: A key concept in DDD is that the same real-world entity can have different models in different contexts. For instance, a "Product" in the Sales context might have attributes like price and discount, while the "Product" in the Inventory context has stockLevel and warehouseLocation.67 Detecting classes with the same or similar names (e.g.,
Product) located in different high-level candidate contexts is a powerful signal of a context boundary.
Anti-Corruption Layer (ACL): When two Bounded Contexts need to integrate, a common DDD pattern is the Anti-Corruption Layer. An ACL is a dedicated module whose sole responsibility is to translate data and concepts from one context into the language of another, preventing the "downstream" context's model from being corrupted by the "upstream" context's model.68 Identifying a module that explicitly depends on two or more candidate contexts and appears to be performing data transformation is a very strong indicator of a context boundary.

2.4.2 Automated Detection Strategy

A multi-step strategy is required to infer Bounded Contexts:
Graph-Based Clustering: First, construct the complete module-level dependency graph for the entire codebase. Apply a community detection algorithm (e.g., the Louvain method) to this graph. These algorithms are designed to find clusters of nodes that are densely connected internally but sparsely connected to other clusters. The resulting communities are strong first-pass candidates for Bounded Contexts.
Directory-Based Refinement: Analyze the top-level directory structure of the project. Use these directory names (e.g., billing, support) as candidate context names. Correlate the files within these directories with the clusters found in the previous step. A high overlap between a directory's contents and a graph cluster reinforces the hypothesis that it represents a Bounded Context.
Ubiquitous Language Analysis: For each candidate context, create a "lexicon" by extracting the names of all public classes, structs, and methods. Compare these lexicons across contexts. The detection of homonyms—classes with the same name but different structures or methods, like billing.Customer and support.Customer—is a powerful confirmation of a context boundary.
Anti-Corruption Layer Identification: Scan for modules that have dependencies on two or more distinct candidate contexts. Analyze the functions within these modules. If they primarily consist of data mapping and transformation logic (e.g., converting a billing.Order object into a shipping.ShipmentRequest object), flag the module as a potential Anti-Corruption Layer.
This combined approach allows the tool to build a "context map" of the application, identifying not just the boundaries but also the relationships between them, providing invaluable insight into the strategic design of the system.

Section 3: Detecting Hierarchical and Presentation-Centric Architectures

In contrast to the inward-flow patterns, a significant class of architectures is characterized by a more linear, hierarchical dependency structure. These patterns, including the traditional Layered (N-Tier) architecture and the family of presentation patterns (MVC, MVP, MVVM), prioritize a top-down separation of concerns, often with a specific focus on isolating the user interface from the underlying business logic and data. Detecting these patterns requires a different set of heuristics focused on identifying distinct layers and the specific interaction rules between their components.

3.1 Layered (N-Tier) Architecture

The Layered architecture is one of the most traditional and widely used software design patterns. It organizes a system into a stack of horizontal layers, each with a specific and distinct responsibility.72

3.1.1 Structural and Dependency Fingerprints

Key Structure: The pattern is defined by its hierarchical organization. A typical three-tier application consists of a Presentation Layer (handling UI and user interaction), a Business Logic Layer (containing business rules and processes), and a Data Access Layer (managing persistence and communication with the database).72 More complex systems may have additional layers (N-Tier), such as a dedicated Application Layer or Service Layer.76
Dependency Flow: The cardinal rule of Layered architecture is that dependencies must flow in a single direction: downwards. A layer can only depend on services provided by layers below it. A higher layer must never depend on a layer above it.72 This ensures that changes in a lower layer (e.g., switching the database) have a minimal impact on higher layers. The resulting dependency graph must be a Directed Acyclic Graph (DAG).
Closed vs. Open Layers: This is a key variation that affects the dependency graph's structure.
Closed Layer Architecture: This is the strict form where a layer can only call the layer immediately below it. For example, the Presentation layer can only call the Business layer, not the Data Access layer directly. This minimizes coupling between layers.76
Open Layer Architecture: This is a more relaxed form where a layer can call any layer below it. The Presentation layer could call the Business layer and also directly call the Data Access layer. This can improve performance by bypassing intermediate layers but increases coupling.76

3.1.2 Directory and Naming Heuristics

Layered architectures often exhibit very clear naming and structural conventions that serve as strong heuristics for detection.
Directory Structure: Projects are frequently organized into directories that directly map to the architectural layers. Common naming schemes include presentation, ui, or controllers for the top layer; business, logic, or services for the middle layer; and data, persistence, or repositories for the bottom layer.70
File and Class Naming: The names of files and classes often include a suffix that indicates their layer, such as UserController.java, ProductService.cs, or CustomerRepository.py. This convention makes the role of each component explicit.

3.1.3 Automated Detection Strategy

The detection process for a Layered architecture involves a three-step validation process:
Propose Layering Model: The first step is to hypothesize the layered structure of the application. This is done primarily through naming heuristics. The tool scans directory and file names for keywords like controller, service, repository, view, data, etc., and assigns each module to a proposed layer (e.g., all modules in a controllers folder are assigned to the Presentation layer).
Construct Dependency Graph: A full module-level dependency graph is constructed using the static analysis techniques described in Section 1.
Verify Unidirectionality and Acyclicity: The core of the detection is to validate the dependency graph against the rules of a layered architecture.
Check for Upward Dependencies: The tool iterates through every dependency edge in the graph. If an edge points from a lower-proposed layer to a higher-proposed layer (e.g., a module in the data layer depending on a module in the business layer), it is flagged as a violation.
Check for Cycles: The tool must detect any dependency cycles that span multiple layers. A cycle (e.g., Presentation -> Business -> Data -> Presentation) is a severe violation of the layered pattern.
Assign Confidence Score: The confidence score is determined by the number and severity of violations. A project with zero upward dependencies and no cross-layer cycles would receive a very high confidence score for being a Layered architecture. A few violations might indicate architectural drift, while numerous violations would suggest that the project does not follow this pattern. The tool can also distinguish between open and closed architectures by checking if dependencies skip layers.

3.2 Model-View-Controller (MVC), Model-View-Presenter (MVP), and Model-View-ViewModel (MVVM)

These three patterns are primarily concerned with separating the user interface from the underlying data and business logic. They are often considered "micro-architectures" that can exist within a single layer (typically the presentation layer) of a larger system. The key to differentiating them lies in analyzing the specific rules of interaction and dependency between their three core components.80
A critical insight for automated detection is that these patterns are frequently found within a larger architectural pattern. For example, a web application might be built using Clean Architecture, but its presentation layer (the web framework part) will almost certainly be organized using a pattern like MVC or MVVM.65 This hierarchical nature means an effective tool cannot simply classify an entire application as "MVC." It must first identify the macro-architecture (e.g., Layered, Clean) and then perform a more focused analysis within the presentation/UI-facing components to identify the specific UI pattern in use. The final report should reflect this hierarchy, for example: "The system is identified as Layered Architecture (Confidence: 90%). The Presentation Layer (located in
/src/web) is implemented using the MVC pattern (Confidence: 95%)."

3.2.1 Differentiating via Component Interaction Rules

Model-View-Controller (MVC):
Components: Model (data and business logic), View (UI representation), and Controller (handles user input).84
Interaction: The Controller is the central entry point. It receives user input, processes it by interacting with the Model, and then selects a View to render the response. The View and Model are generally decoupled. In classic implementations, the Model might notify the View of changes using the Observer pattern, but in many web frameworks, the Controller explicitly passes data from the Model to the View.80
Dependencies: View -> Controller; Controller -> Model; Controller -> View.
Model-View-Presenter (MVP):
Components: Model, View, and Presenter (mediator).84
Interaction: The Presenter acts as the "middle-man" and contains the UI logic. The View is entirely passive; it captures user events and delegates them to the Presenter. The Presenter retrieves data from the Model, formats it, and then explicitly updates the View. There is no direct communication between the Model and the View.81
Dependencies: View -> Presenter; Presenter -> Model; Presenter -> View (typically through an interface that the View implements). The one-to-one relationship between a View and its Presenter is a strong characteristic.84
Model-View-ViewModel (MVVM):
Components: Model, View, and ViewModel (an abstraction of the View).84
Interaction: The ViewModel exposes data properties and commands that the View can bind to. When the data in the ViewModel changes, the View updates automatically through this data-binding mechanism. The ViewModel pulls data from the Model and transforms it for display. Crucially, the ViewModel has no direct reference to the View.91
Dependencies: View -> ViewModel (via data binding); ViewModel -> Model. The absence of a dependency from the ViewModel to the View is a key differentiator.

3.2.2 Directory and Naming Heuristics

For these patterns, naming conventions are an exceptionally strong signal.
Directory Structure: Look for directories explicitly named models, views, and controllers (for MVC), presenters (for MVP), or viewmodels (for MVVM).79
File and Class Naming: File and class names often follow suit, e.g., UserModel.js, LoginView.jsx, UserController.ts, ProfilePresenter.rs, SettingsViewModel.cs.

3.2.3 Automated Detection Strategy with Language-Specific Examples

Rust: In a web framework like Axum, the handlers defined in the routing setup act as Controllers.95 Data structures (
structs) used for business logic and persistence are the Models. HTML templates (e.g., rendered with libraries like askama or tera) are the Views. To detect MVC, the tool would trace calls from the Axum handler (Controller) to functions that manipulate data structs (Model) and then render a template (View). For MVP, the handler (View) would hold a reference to a Presenter struct and delegate calls to it. For MVVM, this is less common in backend Rust but could be seen in GUI frameworks where the View binds to data exposed by a ViewModel struct.
Python: The Django framework is a prime example of a modified MVC pattern called Model-View-Template (MVT).97
Model: Defined in models.py files.
View (Controller in MVC terms): The logic is in views.py. These are Python functions or classes that take a web request and return a web response. They contain the controller logic.97
Template (View in MVC terms): The HTML files, typically located in a templates directory, are responsible for presentation.97
Detection: An automated tool would parse the project's urls.py file to map URL patterns to specific functions in views.py. It would then analyze those view functions to identify which models they interact with (by looking for imports from models.py and ORM calls) and which templates they render (by looking for calls to render()).
JavaScript (React):
View: A React component (.jsx or .tsx file) is unambiguously the View.
Model: This can be represented by data-fetching logic (e.g., functions using fetch or axios) or dedicated data classes.
Differentiating the Mediator:
MVC: Less common in modern React. A controller might be a separate utility function or class imported by the component, but the flow is often less distinct.
MVP: A Presenter class would be instantiated within the React component's lifecycle (e.g., in a useEffect hook or constructor). The component would pass its own methods (as an interface) to the presenter, e.g., const presenter = new UserPresenter(this).
MVVM: This is the most natural fit for modern React. The ViewModel is the component's state and the logic that updates it (e.g., using useState, useReducer, or state management libraries like Redux, MobX, or Zustand). The View "binds" to this state, and re-renders when it changes. The ViewModel logic (e.g., in a custom hook useUserViewModel) would call the Model (data-fetching functions).58 Detection would focus on identifying these state management patterns and the one-way data flow from state to the JSX.

Section 4: Handling Architectural Ambiguity and Complexity

Real-world software systems rarely exist as pure, textbook implementations of a single architectural pattern. They are complex ecosystems influenced by deployment strategies, framework conventions, and the inevitable process of evolution and decay. An effective automated analysis tool must therefore be equipped to handle this ambiguity. This section addresses three critical areas of complexity: the distinction between monoliths and microservices, the impact of framework-imposed structures, and the challenges of analyzing legacy systems suffering from architectural drift. The underlying theme is that the goal of automated analysis must shift from simple "pattern detection" to a more nuanced "architectural recovery and health assessment."

4.1 Architectural Style: Monolith vs. Microservices

The distinction between a monolithic application and a microservices architecture is fundamental, as it dictates the scope of analysis. A monolithic application is built as a single, unified, and deployable unit, whereas a microservices architecture is a collection of smaller, independently deployable services that communicate over a network.102

The Detection Challenge

From a static analysis perspective confined to a single code repository, distinguishing a small, well-structured monolith from a single microservice can be nearly impossible. The internal code structure of one microservice might look identical to that of a small monolith, as both could be built using, for example, Clean Architecture. The defining characteristics of microservices are often external to the source code itself, residing in the deployment and operational models.105

Heuristics for Differentiation

Despite the challenge, static analysis can identify strong indicators of a microservices architecture, especially within a monorepo context:
Repository Structure: The most obvious signal is a monorepo containing multiple, distinct, and seemingly independent applications. Each application would typically have its own entry point (main.rs, app.js), dependency manifest (Cargo.toml, package.json), and build configuration.
Inter-Service Communication Code: A monolith's components communicate through in-process function calls. In contrast, microservices communicate over the network.103 The presence of code for network communication is a powerful heuristic. This includes:
HTTP Clients: Extensive use of libraries like reqwest (Rust), requests (Python), or axios (JavaScript) to call other internal services.
Message Queue Integration: Code that produces or consumes messages from systems like RabbitMQ, Apache Kafka, or AWS SQS.
gRPC or other RPC Frameworks: The presence of gRPC client stubs or server definitions is a strong indicator of a service-oriented architecture.
Deployment Artifacts: The configuration files for deployment and containerization can reveal the architectural style. The presence of multiple Dockerfiles, Kubernetes manifests (e.g., deployment.yaml, service.yaml), or serverless framework configurations (serverless.yml, SAM templates) within a project strongly suggests that it is composed of multiple deployable units.104
Decentralized Data Management: Microservices often adhere to the principle of "one database per service".105 Finding configuration and connection logic for multiple, distinct databases (e.g., a
users-db and a products-db) within the codebase is another indicator that points away from a traditional monolith.

Impact on Analysis

If a microservices architecture is detected, the analysis strategy must adapt. The architectural pattern recognition process described in previous sections should not be run on the repository as a whole. Instead, it must be executed independently for each identified service. Each microservice is its own application and could be implemented using a completely different architectural pattern from its peers. The final report should present a service-by-service breakdown of the architecture.

4.2 The Influence of Framework-Imposed Structures

Software frameworks like Django, Ruby on Rails, NestJS, and Axum provide structure and conventions that significantly accelerate development. However, these same conventions can either reinforce or obscure the underlying architectural pattern, presenting a challenge for a generic detection tool.107

The Framework-as-a-Heuristic Strategy

Instead of viewing frameworks as a complication, an intelligent tool should treat their presence as a powerful heuristic. The detection process should include a preliminary phase to identify the framework being used, typically by inspecting dependency files (Cargo.toml, package.json, requirements.txt). Once a framework is identified, the tool can load a set of framework-specific rules and assumptions to guide the subsequent pattern analysis.
Prescriptive Frameworks (e.g., Django): Django strongly imposes its Model-View-Template (MVT) structure, which is a variant of MVC.97 Upon detecting Django, the tool should immediately assume an MVT pattern and focus its analysis on verifying the roles of
models.py, views.py, and the templates directory, rather than searching for other patterns from scratch.
Flexible Frameworks with Strong Conventions (e.g., NestJS): NestJS is built around modules, controllers, and an injectable services model that heavily utilizes dependency injection and decorators.83 While it can be used for simple MVC-like applications, it is exceptionally well-suited for implementing Clean or Hexagonal architectures.64 For a NestJS project, the tool should specifically look for signs of Dependency Inversion. For example, it can check if controllers depend on use-case services, which in turn depend on abstract repository interfaces (
@Inject('IUserRepository')), with the concrete repository implementation being provided in a separate infrastructure module. This use of the DI container to invert dependencies is a key signal of a Clean/Hexagonal design.66
Modular/Library-Based Frameworks (e.g., Axum): Axum is less of a full-fledged framework and more of a modular set of libraries built on top of Tokio and Tower.95 It is unopinionated about overall application structure. Its use of extractors, state, and handlers can be adapted to any architectural pattern.110 For an Axum project, the framework itself provides fewer clues, so the tool must rely more heavily on the other signals, such as directory structure and naming conventions, to form its hypothesis.

4.3 Legacy Systems: Architectural Recovery and Drift Analysis

Legacy systems represent the most significant challenge. They are often characterized by a lack of documentation, outdated technologies, tangled dependencies, and years of unmanaged architectural drift.4 In such systems, patterns are rarely pure; they are more likely to be mixed, partially implemented, or heavily violated.111
This reality requires a fundamental shift in the objective of the analysis. For legacy systems, the goal is not merely to assign a pattern label but to perform architectural recovery—the process of extracting high-level architectural information from low-level artifacts like source code.7 This aligns with the academic field of software architecture recovery, which uses techniques like source code clustering and dependency analysis to reconstruct a system's "as-implemented" architecture because its "as-designed" architecture is lost or obsolete.112

Detection Strategy for Legacy Systems

Prioritize Dependency Analysis: In legacy code, naming conventions are often inconsistent or misleading. The dependency graph, however, represents the undeniable ground truth of how the components interact. This signal should be weighted most heavily.
Focus on Violations and "Architectural Smells": Rather than searching for a perfect pattern match, a more fruitful approach is to search for common violations, often referred to as architectural smells. These are common design problems that indicate architectural decay.115 The tool should be configured to detect and report on smells such as:
Cyclic Dependencies: A set of modules that depend on each other in a cycle. This is a classic sign of a "big ball of mud" and a severe violation of any layered or inward-flow pattern.
Unstable Dependencies: A module that is intended to be stable (e.g., a core domain model) depending on a volatile module (e.g., a specific UI library).
Leaky Abstractions: A high-level module (e.g., a UI controller) bypassing its intended layer and directly depending on a low-level module (e.g., a database driver or ORM entity).
Architectural Conformance Checking: The most powerful application for legacy systems is to move from discovery to conformance checking. In this mode, the architect provides the tool with a simplified "to-be" or "intended" architecture. This could be a simple mapping file that assigns modules (identified by file paths or regex) to logical layers (e.g., src/web/** is Presentation, src/services/** is Business). The tool then uses the dependency graph to check the implementation against this intended architecture and reports every single dependency that violates the specified rules.119 This provides the development team with a concrete, actionable list of architectural debt to be addressed.
Acknowledge Limitations: It is crucial to recognize that static analysis has limits, especially with legacy code that makes heavy use of dynamic dispatch, reflection, or other dynamic language features. A complete architectural picture in these cases may require dynamic analysis (runtime tracing) to observe actual object interactions, a point well-established in academic research.7
By reframing the tool as an "architectural health assessment" engine that performs recovery, smell detection, and conformance checking, its utility for an architect managing a complex or legacy system is magnified immensely. It becomes a diagnostic instrument for identifying and prioritizing the most critical areas for refactoring and paying down architectural technical debt.

Section 5: A Practical Toolkit for Implementation

This section provides a practical guide to the specific libraries and techniques necessary to build the proposed architectural analysis engine. The implementation is broken down by language—Rust, Python, and JavaScript/TypeScript—and by the core tasks of building the dependency graph and parsing source code with Abstract Syntax Trees (ASTs) for heuristic data.

5.1 Building and Analyzing the Dependency Graph


5.1.1 In Rust

Tooling:
Inter-Crate Dependencies: The cargo metadata command is the starting point. It outputs a JSON structure detailing all crates in a workspace and their dependencies. The krates crate can parse this output to build a crate-level dependency graph.14
Intra-Crate Dependencies: For module-level dependencies within a single crate, the syn crate is essential for parsing Rust source code.122
Graph Representation: A library like petgraph can be used to store and analyze the constructed graph.
Process:
Crate Graph: Execute cargo metadata and parse the JSON output to build an initial graph of crate dependencies. This identifies high-level relationships within a workspace.
Module Graph: For each crate, recursively find all .rs files.
Parse Files: Use syn::parse_file to parse each Rust source file into a syn::File AST.
Extract use Statements: Implement a visitor that walks the AST using syn::visit::Visit. The visitor's visit_item_use method will be called for every use statement (syn::ItemUse).
Resolve Paths: Extract the path from each use statement. This path needs to be resolved to a fully qualified module path relative to the crate root. This can be complex due to Rust's module system (mod declarations, super, crate).
Construct Graph: Add nodes to the graph for each module (file) and directed edges for each resolved use path.

5.1.2 In Python

Tooling:
AST Parsing: The built-in ast module is the standard for parsing Python code.23
Graph Manipulation: The networkx library is a powerful and widely used tool for creating, manipulating, and studying complex networks. It is ideal for representing and analyzing the dependency graph.12
Static Analysis Helpers: The pyan library provides a higher-level abstraction for generating call dependency graphs, which can complement the module import graph.124
Process:
File Traversal: Recursively walk the project directory to find all .py files.
AST Parsing: For each file, read its content and parse it into an AST using ast.parse().
Import Extraction: Create a custom visitor class that inherits from ast.NodeVisitor. Implement the visit_Import and visit_ImportFrom methods to capture all import statements.23 These methods are triggered as the visitor walks the AST.
Extract Module Names:
For ast.Import nodes, iterate through the names attribute to get the imported module names.
For ast.ImportFrom nodes, get the base module from the module attribute and the specific imported names from the names attribute. Handle relative imports by inspecting the level attribute.
Graph Construction: For each Python file, add a node to a networkx.DiGraph. For each import found, add a directed edge from the current file's node to the node of the imported module.

5.1.3 In JavaScript/TypeScript

Tooling:
AST Parsing: While Esprima is a classic choice, more modern and actively maintained parsers like @babel/parser (for JavaScript and TypeScript) or acorn are recommended. They produce an AST compliant with the ESTree specification.25
AST Traversal: The estraverse library provides a generic visitor pattern for traversing any ESTree-compliant AST. Alternatively, many parsers come with their own traversal utilities.127
Graph Representation: A library like graphlib can be used, or a simple dictionary/map structure can suffice for representing the graph.
Process:
File Traversal: Find all .js, .jsx, .ts, and .tsx files in the project.
AST Parsing: Parse each file into an AST. For modules, use the parser's module-aware parsing mode (e.g., esprima.parseModule or sourceType: 'module' in Babel).
Import/Require Extraction: Traverse the AST and look for specific node types:
ES Modules: Identify ImportDeclaration nodes. The dependency is the string value of the source property.
CommonJS: Identify CallExpression nodes. Check if the callee is an Identifier with the name require. If so, the first argument to the call is the dependency path.
Path Resolution: The extracted module paths need to be resolved to actual file paths, taking into account node_modules, path aliases (from tsconfig.json or Webpack config), and relative paths.
Graph Construction: Add nodes for each file and directed edges for each resolved dependency.

5.2 Parsing Source Code with ASTs for Heuristic Inputs


5.2.1 In Rust using syn

Objective: Identify trait definitions (Ports) and their implementations (Adapters) to detect Dependency Inversion.
Technique:
Parse a file using syn::parse_file.
Use a syn::visit::Visit implementation to walk the AST.
Find Ports: In the visit_item_trait method, capture the syn::ItemTrait node. Its ident field gives the name of the trait (the Port).
Find Adapters: In the visit_item_impl method, inspect the syn::ItemImpl node. This node represents an impl block. The trait_ field is an Option<(Option<Token!(!_]>, Path, Token![for])>. If this is Some, it's a trait implementation. The Path within this tuple is the path to the trait being implemented. This directly links the implementing struct (the Adapter) to the trait (the Port).122

5.2.2 In Python using ast

Objective: Identify class definitions, inheritance hierarchies, and function signatures.
Technique:
Parse a file using ast.parse.
Create a ast.NodeVisitor and walk the tree.
Find Classes and Inheritance: Implement the visit_ClassDef method. The node.name attribute provides the class name. The node.bases attribute is a list of nodes representing the parent classes. This is key for identifying adapter-port relationships where the adapter inherits from an abstract port class.19
Find Functions: Implement visit_FunctionDef to get function names (node.name) and argument details (node.args). This helps in analyzing data flow and identifying method signatures.

5.2.3 In JavaScript/TypeScript using esprima (or Babel)

Objective: Identify class definitions, interface implementations (implements clause), and function calls.
Technique:
Parse a TypeScript file into an ESTree-compliant AST.
Traverse the AST.
Find Ports and Adapters (TypeScript): Look for TSInterfaceDeclaration nodes to identify Ports. Look for ClassDeclaration nodes and inspect their implements property. This property will be an array of TSExpressionWithTypeArguments nodes, which links the class (Adapter) to the interface it implements (Port).
Analyze Data Flow: Look for CallExpression nodes. By inspecting the callee (the function being called) and arguments, the tool can trace data flow. For example, it can determine which service method is being called by a controller, providing evidence for architectural conformance.25
The following table provides a summary of the recommended libraries and tools for implementing the static analysis engine in each target language.
Language
Task
Recommended Library/Tool
Key Features
Considerations
Rust
AST Parsing
syn
Full Rust syntax tree representation, robust parsing, feature-gated for performance.
Primarily for procedural macros but works for general analysis. Can be complex due to Rust's rich syntax.


Dependency Graph
cargo metadata + krates (inter-crate), syn (intra-crate), petgraph (graph structure)
Direct access to workspace dependency info. syn provides precise module-level dependencies.
Requires handling Rust's complex module resolution system (mod, super, etc.).
Python
AST Parsing
ast (built-in)
Standard library, no external dependencies, covers all Python syntax.
Provides a raw AST; requires a visitor pattern (ast.NodeVisitor) for traversal.


Dependency Graph
networkx
Industry-standard for graph analysis in Python. Rich set of algorithms for cycle detection, clustering, etc.
Excellent for analysis, but the graph must be built manually by parsing imports with the ast module.
JavaScript / TypeScript
AST Parsing
@babel/parser or acorn
ESTree-compliant, actively maintained, excellent support for modern JS/TS features.
esprima is a classic but may lag on newest language features. Babel is a robust choice.


Dependency Graph
dependency-cruiser or custom traversal with estraverse
dependency-cruiser is a high-level tool for dependency analysis. estraverse is a generic AST walker.
Requires careful handling of module resolution (aliases, node_modules, etc.).


Section 6: Conclusion and Recommendations


Synthesis of Methodology

This report has detailed a methodological framework for the automated recognition of software architectural patterns and the subsequent verification of architectural conformance. The core of this framework is a departure from simplistic, deterministic rule-matching. Instead, it advocates for a multi-signal, probabilistic heuristic model that is resilient to the inherent ambiguity and drift found in real-world software projects.
The proposed methodology rests on three pillars of static analysis:
Dependency Graph Analysis: As the primary and most reliable signal, the dependency graph provides the ground truth of how system components interact. Verifying this graph against the fundamental dependency rules of each pattern (e.g., inward flow for Clean/Hexagonal, downward flow for Layered) is the most critical step.
Naming Convention Analysis: As a strong secondary signal, the names of directories, files, and classes provide powerful, human-centric clues about the intended architecture.
Structural Analysis via ASTs: As a supporting signal, parsing Abstract Syntax Trees allows for the extraction of granular details like interface implementation and inheritance, which are essential for differentiating between similar patterns (e.g., MVP vs. MVVM) and confirming dependency inversion.
By combining and weighting evidence from these three sources, the resulting analysis engine can produce a confidence-based assessment, correctly identifying not only pure patterns but also mixed patterns and areas of architectural erosion. This approach reframes the problem from "pattern detection" to the more practical and valuable goal of architectural recovery and health assessment.

Path to Implementation

Building a tool based on this framework is a significant undertaking. A phased, incremental approach is recommended to manage complexity and deliver value early.
Phase 1: Dependency Visualization and Basic Violation Detection. The initial version of the tool should focus exclusively on the strongest signal: the dependency graph. The goal is to parse a codebase, construct the graph, and visualize it. This phase should also include a basic rule engine to detect fundamental architectural smells like cyclic dependencies between high-level modules.
Phase 2: Heuristic-Based Pattern Classification. Incorporate the naming and structural analysis heuristics. Develop a dictionary of common architectural terms and use AST parsing to identify structural features. Implement a scoring engine that combines these signals with the dependency analysis to produce a confidence-scored classification for the major architectural patterns.
Phase 3: Framework-Specific Analyzers. Develop specialized modules for popular and prescriptive frameworks (e.g., Django, NestJS). These modules would contain framework-specific heuristics that leverage the framework's conventions to achieve higher accuracy in pattern detection.
Phase 4: Architectural Conformance and Drift Monitoring. The most advanced stage involves shifting the tool's focus from discovery to conformance. Allow an architect to define a "target architecture" by mapping code modules to logical layers. The tool would then function as a conformance checker, comparing the as-implemented architecture against this target and generating a detailed report of all violations.

Final Recommendation

The ultimate value of such a tool is not in a one-time analysis of a codebase but in its integration into the daily development workflow. The final recommendation is to evolve the tool from a reactive analysis instrument into a proactive architectural governance engine.
By integrating the architectural conformance checking capabilities into the Continuous Integration/Continuous Delivery (CI/CD) pipeline, the tool can analyze every pull request or commit for architectural violations.131 When a developer introduces a change that creates an illicit dependency—for example, making a
repository call a service—the CI build would fail, providing immediate feedback. This prevents architectural drift before it can take root in the codebase.
This proactive approach transforms the tool from a means of identifying existing technical debt into a mechanism for preventing its accumulation. It empowers development teams to maintain architectural integrity as the system evolves, ensuring that the software remains maintainable, scalable, and aligned with its intended design for the long term.
Works cited
Detecting Architectural Gaps with Automation - GlobalLogic, accessed July 3, 2025, https://www.globallogic.com/wp-content/uploads/2023/06/Detecting-Architectural-Gaps.pdf
Assessing architectural drift in commercial software development: a case study - Lero, accessed July 3, 2025, https://lero.ie/sites/default/files/files/Assessing%20architectural%20drift%20in%20commercial%20software%20development%20-%20a%20case%20study.pdf
[Part 1] Delving into Architectural Drift - DEV Community, accessed July 3, 2025, https://dev.to/vladi-stevanovic/delving-into-architectural-drift-939
Working on Legacy Software: Rewriting technique, experience and lessons, accessed July 3, 2025, https://dev.to/olivermensahdev/working-on-legacy-software-rewriting-technique-experience-and-lessons-29
Monolith vs microservices: Comparing architectures for software delivery - Chronosphere, accessed July 3, 2025, https://chronosphere.io/learn/comparing-monolith-and-microservice-architectures-for-software-delivery/
Mapping legacy code: Techniques for better architecture understanding | by Gilad Navot, accessed July 3, 2025, https://overcast.blog/mapping-legacy-code-techniques-for-better-architecture-understanding-4479e8e3d864
Software architecture recovery - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Software_architecture_recovery
A Software Architecture Reconstruction Method, accessed July 3, 2025, https://users.ece.utexas.edu/~perry/prof/wicsa1/final/goa.pdf
Main and visibly apparent difference between n-tier and clean architecture - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/56804075/main-and-visibly-apparent-difference-between-n-tier-and-clean-architecture
Everything You Need to Know About Clean Architecture | Bitloops Docs, accessed July 3, 2025, https://bitloops.com/docs/bitloops-language/learning/software-architecture/clean-architecture
Hexagonal Architecture – What Is It? Why Use It? - HappyCoders.eu, accessed July 3, 2025, https://www.happycoders.eu/software-craftsmanship/hexagonal-architecture/
Build a dependency graph in python - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/14242295/build-a-dependency-graph-in-python
Item 25: Manage your dependency graph - Effective Rust, accessed July 3, 2025, https://effective-rust.com/dep-graph.html
Visualization — list of Rust libraries/crates // Lib.rs, accessed July 3, 2025, https://lib.rs/visualization
Abstract Syntax Tree (AST) and Interpreter - Create Your Own Programming Language with Rust, accessed July 3, 2025, https://createlang.rs/01_calculator/ast.html
Syntax and the AST - Rust Compiler Development Guide, accessed July 3, 2025, https://rustc-dev-guide.rust-lang.org/syntax-intro.html
Static Analysis using ASTs | by Hootsuite Engineering - Medium, accessed July 3, 2025, https://medium.com/hootsuite-engineering/static-analysis-using-asts-ebcd170c955e
The Abstract Syntax Tree - Create a Static Analyser in Rust, accessed July 3, 2025, https://www.michaelfbryan.com/static-analyser-in-rust/book/parse/ast.html
Extracting the Module and Function Names from Python ASTs - Arumoy Shome, accessed July 3, 2025, https://arumoy.me/blogs/python-ast-extract-module-method-names/
How to extract docstrings from Python files with ast - Gabriele Lanaro, accessed July 3, 2025, https://gabrielelanaro.github.io/blog/2014/12/12/extract-docstrings.html
Abstract Syntax Tree | Write a JavaScript Parser in Rust - Oxc, accessed July 3, 2025, https://oxc-project.github.io/javascript-parser-in-rust/docs/ast/
ast - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/sap-ast
ast — Abstract Syntax Trees — Python 3.13.5 documentation, accessed July 3, 2025, https://docs.python.org/3/library/ast.html
Deciphering Python: How to use Abstract Syntax Trees (AST) to understand code, accessed July 3, 2025, https://www.mattlayman.com/blog/2018/decipher-python-ast/
Chapter 2. Syntactic Analysis (Parsing) — Esprima master documentation, accessed July 3, 2025, https://esprima.readthedocs.io/en/latest/syntactic-analysis.html
Chapter 1. Getting Started — Esprima master documentation, accessed July 3, 2025, https://esprima.readthedocs.io/en/latest/getting-started.html
esprima - NPM, accessed July 3, 2025, https://www.npmjs.com/package/esprima
The Layer Naming Dilemma. Designers and developers who have… | by Taras Savytskyi | Medium, accessed July 3, 2025, https://medium.com/@TarasSavytskyi/the-layer-naming-dilemma-3b5b14c59db2
What naming convention should I use in Clean Architecture?, accessed July 3, 2025, https://softwareengineering.stackexchange.com/questions/423308/what-naming-convention-should-i-use-in-clean-architecture
Mastering Software Architecture: Patterns, Challenges, and Best Practices - Tarun Telang, accessed July 3, 2025, https://taruntelang.medium.com/mastering-software-architecture-patterns-challenges-and-best-practices-c8e92602473d
(PDF) Automatic design pattern detection - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/4015789_Automatic_design_pattern_detection
Automatic Design Pattern Detection - Welf Löwe, accessed July 3, 2025, http://welf.se/files/HHHL03.pdf
Clean DDD lessons: project structure and naming conventions | by George | Technical blog from UNIL engineering teams | Medium, accessed July 3, 2025, https://medium.com/unil-ci-software-engineering/clean-ddd-lessons-project-structure-and-naming-conventions-00d0b9c57610
Clean Architecture with Dependency Rule | by Mehmet Ozkaya - Medium, accessed July 3, 2025, https://medium.com/design-microservices-architecture-with-patterns/clean-architecture-with-dependency-rule-dff96d479a60
Complete Guide to Clean Architecture - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/system-design/complete-guide-to-clean-architecture/
Clean Architecture — Everything You Need to Know - CodiLime, accessed July 3, 2025, https://codilime.com/blog/clean-architecture/
Hexagonal architecture pattern - AWS Prescriptive Guidance, accessed July 3, 2025, https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html
thecodest.co, accessed July 3, 2025, https://thecodest.co/blog/the-power-of-hexagonal-architecture/#:~:text=Dependency%20rule%20is%20a%20fundamental,or%20any%20other%20external%20agency.
The Power of Hexagonal Architecture - The Codest, accessed July 3, 2025, https://thecodest.co/blog/the-power-of-hexagonal-architecture/
Clean Architecture: A Deep Dive into Structured Software Design | Spaceteams, accessed July 3, 2025, https://www.spaceteams.de/en/insights/clean-architecture-a-deep-dive-into-structured-software-design
The Clean Architecture — Beginner's Guide | by Bharath - Better Programming, accessed July 3, 2025, https://betterprogramming.pub/the-clean-architecture-beginners-guide-e4b7058c1165
Hexagonal architecture (software) - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)
Everything You Need to Know About Hexagonal Architecture: Kernel, Ports, Adapters, accessed July 3, 2025, https://scalastic.io/en/hexagonal-architecture/
Hexagonal Architecture: Ports and Adapters Pattern | by Milad Fahmy - Medium, accessed July 3, 2025, https://miladezzat.medium.com/hexagonal-architecture-ports-and-adapters-pattern-5ad2421802ec
Hexagonal Architecture: Structure Example | by Alessandro Traversi - Medium, accessed July 3, 2025, https://medium.com/@alessandro.traversi/hexagonal-architecture-structure-example-7ea1d998954e
serfer2/flask-hexagonal-architecture-api: Simple example ... - GitHub, accessed July 3, 2025, https://github.com/serfer2/flask-hexagonal-architecture-api
Project Structure - An Implementation Guide - Hexagonal Architecture, accessed July 3, 2025, https://jmgarridopaz.github.io/content/hexagonalarchitecture-ig/chapter2.html
Understanding Hexagonal Architecture - DEV Community, accessed July 3, 2025, https://dev.to/xoubaman/understanding-hexagonal-architecture-3gk
The Ultimate Guide to Mastering Hexagonal Architecture: Focus on the Domain, accessed July 3, 2025, https://scalastic.io/en/hexagonal-architecture-domain/
fteychene/blueprint-hexagonal-rust: Hexagonal architecture example in Rust - GitHub, accessed July 3, 2025, https://github.com/fteychene/blueprint-hexagonal-rust
Hexagonal architecture in Rust. Tutorial index | by Luca Corsetti | Medium, accessed July 3, 2025, https://medium.com/@lucorset/hexagonal-architecture-in-rust-72f8958eb26d
Master Hexagonal Architecture in Rust - How To Code It, accessed July 3, 2025, https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust
Hexagonal Architecture implemented in Python - Workflows.guru, accessed July 3, 2025, https://www.workflows.guru/resources/hexagonal-architecture-implemented-in-python
juanm4/hexagonal-architecture-frontend: How to implement ... - GitHub, accessed July 3, 2025, https://github.com/juanm4/hexagonal-architecture-frontend
Clean Architecture - Do you know the main principles? | SSW.Rules, accessed July 3, 2025, https://www.ssw.com.au/rules/clean-architecture/
panagiop/node.js-clean-architecture - GitHub, accessed July 3, 2025, https://github.com/panagiop/node.js-clean-architecture
Your "Clean Architecture" is still layered! - DEV Community, accessed July 3, 2025, https://dev.to/nikolicbojan/your-clean-architecture-is-still-layered-3god
arbazpirwani/React-Clean-MVVM-Architecture - GitHub, accessed July 3, 2025, https://github.com/arbazpirwani/React-Clean-MVVM-Architecture
frederikhors/rust-clean-architecture-with-db-transactions - GitHub, accessed July 3, 2025, https://github.com/frederikhors/rust-clean-architecture-with-db-transactions
flosse/clean-architecture-with-rust: Full-Stack Clean ... - GitHub, accessed July 3, 2025, https://github.com/flosse/clean-architecture-with-rust
heumsi/python-clean-architecture-example: A example ... - GitHub, accessed July 3, 2025, https://github.com/heumsi/python-clean-architecture-example
askides/clean-architecture-react: Clean Architecture ... - GitHub, accessed July 3, 2025, https://github.com/askides/clean-architecture-react
Example of clean architecture in front-end (Next.js) - GitHub, accessed July 3, 2025, https://github.com/dimitridumont/clean-architecture-front-end
Implementing a Clean Architecture with Nest.JS (Part 2) - Mantra Labs, accessed July 3, 2025, https://www.mantralabsglobal.com/blog/implementing-a-clean-architecture-with-nest-js-part-2/
Clean Node.js Architecture —With NestJs and TypeScript | by Royi Benita, Senior Full Stack Developer At Armis | Better Programming, accessed July 3, 2025, https://betterprogramming.pub/clean-node-js-architecture-with-nestjs-and-typescript-34b9398d790f
Clean architecture with NestJS - Medium, accessed July 3, 2025, https://medium.com/@mohitkumarsingh907/clean-architecture-with-nestjs-632437e699a7
Bounded Context - Martin Fowler, accessed July 3, 2025, https://martinfowler.com/bliki/BoundedContext.html
Blog: From Good to Excellent in DDD: Understanding Bounded Contexts in Domain-Driven Design - 8/10 - Kranio, accessed July 3, 2025, https://www.kranio.io/en/blog/de-bueno-a-excelente-en-ddd-comprender-bounded-contexts-en-domain-driven-design---8-10
Explain me like I'm 5 what „The bounded context“ means : r/microservices - Reddit, accessed July 3, 2025, https://www.reddit.com/r/microservices/comments/1bms6dh/explain_me_like_im_5_what_the_bounded_context/
How to define and enforce a code structure in layered architecture | by Razvan Dubau, accessed July 3, 2025, https://medium.com/@razvandubau/how-to-define-and-a-enforce-a-code-structure-in-layered-architecture-338afc28121d
Domain analysis for microservices - Azure Architecture Center | Microsoft Learn, accessed July 3, 2025, https://learn.microsoft.com/en-us/azure/architecture/microservices/model/domain-analysis
Layered Architecture Essentials - Number Analytics, accessed July 3, 2025, https://www.numberanalytics.com/blog/layered-architecture-essentials
Layered (N-Layer) Architecture with SOLID Design Principles | by Mehmet Ozkaya - Medium, accessed July 3, 2025, https://medium.com/design-microservices-architecture-with-patterns/layered-n-layer-architecture-with-solid-design-principles-15967a518ff1
Layered Architecture: Building Scalable & Maintainable Software Systems | Bitloops Docs, accessed July 3, 2025, https://bitloops.com/docs/bitloops-language/learning/software-architecture/layered-architecture
Layered Architecture in Software Development: A Comprehensive Guide - Exatosoftware, accessed July 3, 2025, https://exatosoftware.com/layered-architecture-in-software-development-a-comprehensive-guide/
N-tier architecture style - Azure Architecture Center | Microsoft Learn, accessed July 3, 2025, https://learn.microsoft.com/en-us/azure/architecture/guide/architecture-styles/n-tier
Multitier architecture - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Multitier_architecture
How to Differentiate Business and Service Layers in Layered Architecture - Level Up Coding, accessed July 3, 2025, https://levelup.gitconnected.com/how-to-differentiate-business-and-service-layers-in-layered-architecture-912123b2ccf1
Using MVC directory structure - Vue.js Developers, accessed July 3, 2025, https://vuejsdevelopers.com/lessons/using-mvc-directory-structure/
Comparing Software Architecture Patterns MVC Vs. MVVM Vs. MVP - Masai School, accessed July 3, 2025, https://www.masaischool.com/blog/comparing-software-architecture-patterns/
Architecture Patterns for Beginners: MVC, MVP, and MVVM - DEV Community, accessed July 3, 2025, https://dev.to/chiragagg5k/architecture-patterns-for-beginners-mvc-mvp-and-mvvm-2pe7
Architecture Patterns for Beginners: MVC, MVP, and MVVM | HackerNoon, accessed July 3, 2025, https://hackernoon.com/architecture-patterns-for-beginners-mvc-mvp-and-mvvm
Design Patterns in NestJS - DEV Community, accessed July 3, 2025, https://dev.to/geampiere/design-patterns-in-nestjs-9h0
Difference Between MVC, MVP and MVVM Architecture Pattern in Android - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/android/difference-between-mvc-mvp-and-mvvm-architecture-pattern-in-android/
MVC Design Pattern - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/system-design/mvc-design-pattern/
MVC Architecture Explained: Model, View, Controller - Codecademy, accessed July 3, 2025, https://www.codecademy.com/article/mvc-architecture-model-view-controller
Understanding MVC, MVP and MVVM Design Patterns - ScholarHat, accessed July 3, 2025, https://www.scholarhat.com/tutorial/designpatterns/understanding-mvc-mvp-and-mvvm-design-patterns
Understanding Model-View-Presenter (MVP) Architecture in Android: A Complete Guide with Example | by Manish Kumar | Medium, accessed July 3, 2025, https://medium.com/@manishkumar_75473/understanding-model-view-presenter-mvp-architecture-in-android-a-complete-guide-with-example-fa8e7cecb0e7
MVP (Model View Presenter) Architecture Pattern in Android with Example - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/mvp-model-view-presenter-architecture-pattern-in-android-with-example/
Model–view–presenter - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93presenter
Model-View-ViewModel (MVVM) - Learn Microsoft, accessed July 3, 2025, https://learn.microsoft.com/en-us/dotnet/architecture/maui/mvvm
Which Android Architecture Is Best: MVC vs MVP vs MVVM - WebMob Technologies, accessed July 3, 2025, https://webmobtech.com/blog/mvc-mvp-mvvm/
How to organize MVVM files in solution - Software Engineering Stack Exchange, accessed July 3, 2025, https://softwareengineering.stackexchange.com/questions/181948/how-to-organize-mvvm-files-in-solution
Nickforall/Iron-MVC: Simple MVC framework in Rust using Iron - GitHub, accessed July 3, 2025, https://github.com/Nickforall/Iron-MVC
axum - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/axum/latest/axum/
tokio-rs/axum: Ergonomic and modular web framework built with Tokio, Tower, and Hyper, accessed July 3, 2025, https://github.com/tokio-rs/axum
Django Project MVT Structure - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/python/django-project-mvt-structure/
MVC Pattern with Python Django - machinesintheclouds, accessed July 3, 2025, https://machinesintheclouds.com/mvc-pattern-with-python-django
Django Project MVT Structure - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/django-project-mvt-structure/
KishorNaik/Sol_MVVM_React: Example of MVVM pattern in React.js - GitHub, accessed July 3, 2025, https://github.com/KishorNaik/Sol_MVVM_React
thirafidide/react-mvvm-example - GitHub, accessed July 3, 2025, https://github.com/thirafidide/react-mvvm-example
www.atlassian.com, accessed July 3, 2025, https://www.atlassian.com/microservices/microservices-architecture/microservices-vs-monolith#:~:text=A%20monolithic%20application%20is%20built,of%20smaller%2C%20independently%20deployable%20services.
Monolithic vs Microservices - Difference Between Software Development Architectures, accessed July 3, 2025, https://aws.amazon.com/compare/the-difference-between-monolithic-and-microservices-architecture/
Microservices vs. monolithic architecture - Atlassian, accessed July 3, 2025, https://www.atlassian.com/microservices/microservices-architecture/microservices-vs-monolith
Microservices - Martin Fowler, accessed July 3, 2025, https://martinfowler.com/articles/microservices.html
Monolithic vs. Microservices Architecture - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/monolithic-vs-microservices-architecture/
Challenges and Considerations of Architectural Patterns for Payment Solutions - Diva, accessed July 3, 2025, https://mau.diva-portal.org/smash/get/diva2:1884031/FULLTEXT02.pdf
A Complete Guide to Nest Design Pattern | by xxkeith - Stackademic, accessed July 3, 2025, https://blog.stackademic.com/a-complete-guide-to-nest-design-pattern-4d6a6f4fccbd
Exploring NestJS: Architecture and Design Principles - Tekos Interactive, accessed July 3, 2025, https://tekos.net/articles/product-development/exploring-nestjs-architecture-and-design-principles/
Rust Axum Full Course - Web Development - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/12n1vdc/rust_axum_full_course_web_development/
Exploring Software Design Patterns with AI: Future Trends - Zencoder, accessed July 3, 2025, https://zencoder.ai/blog/software-design-patterns-with-ai-future-trends
Software architecture recovery – Knowledge and References - Taylor & Francis, accessed July 3, 2025, https://taylorandfrancis.com/knowledge/Engineering_and_technology/Computer_science/Software_architecture_recovery/
Recover and Optimize Software Architecture Based on Source Code and Directory Hierarchies - KSI Research, accessed July 3, 2025, https://ksiresearch.org/seke/seke19paper/seke19paper_45.pdf
a systematic analysis on software architecture recovery techniques - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/384241240_A_SYSTEMATIC_ANALYSIS_ON_SOFTWARE_ARCHITECTURE_RECOVERY_TECHNIQUES
Integrated Formal Tools for Software Architecture Smell Detection - World Scientific Publishing, accessed July 3, 2025, https://www.worldscientific.com/doi/10.1142/S0218194020400057
Toward a Catalogue of Architectural Bad Smells - Joshua Garcia @ UCI, accessed July 3, 2025, https://jgarcia.ics.uci.edu/wp-content/uploads/10.1.1.183.9958.pdf
Taxonomy of Architecture Maintainability Smells - Universität Hamburg, accessed July 3, 2025, https://www.edit.fis.uni-hamburg.de/ws/files/45000699/APSEC2023.pdf
Arcan: a Tool for Architectural Smells Detection - Milano-Bicocca, accessed July 3, 2025, https://boa.unimib.it/retrieve/e39773b3-de57-35a3-e053-3a05fe0aac26/PID4705339.pdf
Static Architecture-Conformance Checking: An Illustrative Overview - DCC/UFMG, accessed July 3, 2025, https://www.dcc.ufmg.br/~mtov/pub/2010_ieeesw.pdf
Evaluation of an architectural conformance checking software service - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/311622080_Evaluation_of_an_architectural_conformance_checking_software_service
Using code analysis tools for architectural conformance checking - SciSpace, accessed July 3, 2025, https://scispace.com/pdf/using-code-analysis-tools-for-architectural-conformance-4wxqjrssl6.pdf
syn - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/syn
syn - Parser for Rust source code - Crates.io, accessed July 3, 2025, https://crates.io/crates/syn
davidfraser/pyan: pyan is a Python module that performs ... - GitHub, accessed July 3, 2025, https://github.com/davidfraser/pyan
Using Python imports to inject more relevant context into code generation prompts - Medium, accessed July 3, 2025, https://medium.com/@rdefauw/using-python-imports-to-inject-more-relevant-context-into-code-generation-prompts-cf6d9791a276
jquery/esprima: ECMAScript parsing infrastructure for multipurpose analysis - GitHub, accessed July 3, 2025, https://github.com/jquery/esprima
ast-types - NPM, accessed July 3, 2025, https://www.npmjs.com/package/ast-types
Fun with Esprima and Static Analysis - Toby Ho, accessed July 3, 2025, https://tobyho.com/2013/12/02/fun-with-esprima/
Rust Syn Crate Tutorial: Automate Builder Patterns with Custom Macros - Packet and Pine, accessed July 3, 2025, https://packetandpine.com/blog/rust-syn-crate-tutorial/
Modifying JavaScript AST with Yeoman | by Robert Vogt | smartive - Medium, accessed July 3, 2025, https://medium.com/smartive/modifying-javascript-ast-with-yeoman-1182dcd6cb0a
Perforce Klocwork, accessed July 3, 2025, https://www.perforce.com/products/klocwork
Static Code Analysis Methodology and Best Practices - Veracode, accessed July 3, 2025, https://www.veracode.com/security/static-code-analysis/
