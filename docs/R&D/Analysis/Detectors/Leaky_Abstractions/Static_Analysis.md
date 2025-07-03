
Static Analysis of Architectural Abstractions in Polyglot Systems: A Technical Blueprint


Section 1: Programmatic Identification of Module Boundaries

The foundational prerequisite for any architectural analysis tool is the ability to accurately identify the fundamental units of encapsulation and their public interfaces within a given language. These units, which we will refer to as "modules," form the nodes in a dependency graph. The public interface of a module defines the permissible points of connection for other modules, forming the basis for validating architectural rules. The mechanisms for defining these boundaries vary significantly across Rust, Python, and JavaScript, necessitating distinct analytical strategies for each.

1.1. Rust: Crates, Modules, and the Visibility System

Rust provides a uniquely powerful and explicit system for encapsulation, offering strong compile-time guarantees about which parts of a codebase are accessible. A static analysis tool must develop a sophisticated understanding of this system, which is composed of three primary concepts: crates, modules, and visibility.

1.1.1. Core Concepts

The Crate as the Ultimate Boundary: The crate is Rust's fundamental unit of compilation, linkage, and versioning. It is the highest-level abstraction boundary. A project, as defined by a Cargo.toml file, can contain a library crate (with its root at src/lib.rs) and one or more binary crates (e.g., src/main.rs, src/bin/my_bin.rs). Critically, even within the same project directory, the library crate and each binary crate are treated as separate, distinct crates.1 Any interaction between them is subject to the same public API rules as if they were external dependencies. For the analysis tool, the crate boundary is the primary demarcation between "internal" and "external" code.
The Module System (mod): Within a crate, the mod keyword is used to partition code into a hierarchical namespace, improving organization and controlling scope.2 A declaration like
mod services; instructs the compiler to include the contents of either services.rs or services/mod.rs as a child module. The entire module structure of a crate forms a tree, with the crate root (lib.rs or main.rs) at its apex. The analysis tool must parse these mod declarations to reconstruct this module tree accurately.
The Visibility System (pub): Rust's visibility system is the most critical component for identifying a module's public API. By default, all items (functions, structs, enums, etc.) in Rust are private to the module in which they are defined.3 An item's visibility must be explicitly declared using the
pub keyword. The analysis tool must parse and interpret several forms of visibility:
pub: When an item is marked pub, it becomes part of its module's public interface. It can be accessed by any other module that is allowed to access the parent module and all of its ancestors.3 An item marked
pub at the crate root is part of the crate's public API, visible to external crates.
pub(crate): This qualifier makes an item visible to any module within the current crate but keeps it private to the outside world.3 This is an essential feature for building complex libraries, as it allows the creation of a crate-internal API that helper modules can share without polluting the public, external-facing API.2
pub(in path) and pub(super): These provide more granular control, restricting visibility to a specific ancestor module (pub(in crate::outer_mod)) or the immediate parent module (pub(super)), respectively.3 These are used for fine-grained encapsulation within a complex module hierarchy.
Re-exports (pub use): A module can incorporate an item from another module into its own public API using a pub use statement. For example, pub use crate::internal::types::ImportantType; makes ImportantType directly accessible to consumers of the current module, effectively creating a clean, curated public facade from potentially complex internal structures.2 The analysis tool must treat these re-exports as part of the module's public interface.

1.1.2. Differentiating Public and Internal APIs

A naive analysis might treat Rust as having a simple binary public/private system. However, this would fail to capture a critical architectural pattern in Rust development: the distinction between the public API intended for external consumers and the crate-internal API used for implementation. Early in Rust's development, developers resorted to module-nesting tricks to simulate this "crate-level" visibility.4 The stabilization of
pub(crate) provided a first-class language feature for this exact purpose.3
Therefore, the static analysis tool must model at least two distinct levels of public interface for a Rust crate:
External Public API: Consists of all pub items reachable from the crate root. An access from an external crate to any item not in this set is an architectural violation.
Internal Public API: Consists of all pub(crate) items. These items are accessible from anywhere within the same crate. An access from an external crate to a pub(crate) item is a clear violation, but access between two modules within the same crate is a legitimate use of an internal, shared library.
This distinction is paramount. The tool cannot simply flag all cross-module dependencies; it must validate them against the declared visibility scope. A call to a pub(crate) function from another module in the same crate is architecturally sound, whereas a call to the same function from a different crate is a boundary violation.

1.2. Python: Packages, Modules, and the Import Mechanism

In contrast to Rust's explicit, compiler-enforced system, Python's module and package boundaries are defined by a combination of filesystem conventions and runtime import mechanics. This makes static analysis more challenging, as it must often rely on convention rather than rigid rules.

1.2.1. Core Concepts

Modules and Files: The simplest unit of encapsulation is the module. Any file with a .py extension is a module, and its name is the filename without the suffix.5
Packages and __init__.py: A directory is treated as a Python package only if it contains a file named __init__.py.5 This file serves two purposes: it signals to the interpreter that the directory is a package, and it can contain initialization code that is executed the first time any part of the package is imported. The presence of
__init__.py is the primary structural marker a static analysis tool must look for to identify package boundaries.
The import Statement: Dependencies are created via import statements. The analysis must parse all variants:
import my_package.my_module: An absolute import that brings the module into the current namespace.
from my_package import my_module: Imports the submodule directly.
from my_package.my_module import my_function: Imports a specific name from a module into the current namespace.
from. import sibling_module and from.. import parent_package_module: Relative imports, denoted by leading dots. These are crucial for understanding dependencies within a package and are resolved based on the current module's path.6
Defining the Public API (__all__): A package author can explicitly declare the public API by defining a list of strings called __all__ in the __init__.py file. When a user performs a wildcard import (from my_package import *), only the names listed in __all__ are imported into their namespace.5 While wildcard imports are often discouraged in production code, the presence of
__all__ is the strongest and most explicit signal of the author's intent for the package's public interface.7

1.2.2. The Ambiguity of Python's Public Interface

Python lacks a strict, language-enforced visibility system like Rust's pub. Instead, it relies on conventions. A name prefixed with a single underscore (e.g., _internal_helper) is conventionally treated as a private, implementation detail. A name with a double underscore prefix (e.g., __very_private) is subject to name mangling, but neither is truly private; they can still be accessed if the caller knows the name.
This ambiguity is reflected in the different philosophies for structuring a package's __init__.py file 7:
Blank __init__.py: This forces consumers to import submodules explicitly (e.g., from my_package.services import MyService). The package structure itself defines the API.
Wildcard Imports in __init__.py: The __init__.py file itself can use from.submodule import * to pull all submodule contents into the top-level package namespace. This can lead to namespace pollution.
Curated API in __init__.py: The __init__.py can selectively import key functions and classes from its submodules (e.g., from.services import MyService), presenting a clean, unified API to the consumer, who can then use from my_package import MyService.
This lack of a single, enforced standard means the static analysis tool must be flexible and configurable. It cannot rely on one simple rule to define a boundary. The analysis should proceed with the following hierarchy of evidence:
__all__ Variable: If __all__ is defined in __init__.py, it should be treated as the canonical public API. Any external import of a name not in __all__ can be flagged as a potential violation.
Names in __init__.py: In the absence of __all__, the names explicitly imported or defined in __init__.py can be considered the intended public API.
Underscore Convention: An import of any name beginning with an underscore (e.g., from my_package import _internal_module) should be flagged as a high-confidence boundary violation, as it breaks a widely respected community convention.
Direct Submodule Import: An import that bypasses the __init__.py to access a nested submodule (e.g., from my_package.internal.helpers import some_func) should be considered a potential violation, especially if the package follows the "Curated API" pattern.

1.3. JavaScript: A Tale of Two Module Systems

JavaScript's evolution has resulted in two dominant module systems, CommonJS (CJS) and ECMAScript Modules (ESM). They are not merely different syntaxes but represent fundamentally different philosophies of module loading and resolution, which has profound implications for static analysis.

1.3.1. Core Concepts

ECMAScript Modules (ESM): This is the modern, standardized module system for JavaScript, supported natively in browsers and Node.js.8
Syntax: Modules expose functionality using the export keyword and consume it using the import keyword. These can be named exports (export const myVar; import { myVar } from './file.js';) or a single default export (export default myFunction; import myFunc from './file.js';).8
Static Nature: The most crucial characteristic of ESM for static analysis is that import and export statements are static. They must be at the top level of the module (not inside functions or control flow blocks), and their module specifiers (the path strings) must be static string literals.10 This allows the entire dependency graph of an application to be determined by parsing the source code, without any execution.11 This static structure is what enables powerful optimizations like "tree-shaking," where bundlers can safely remove unused exports from the final code bundle.11
CommonJS (CJS): This was the original module system used by Node.js and remains prevalent in many existing projects and libraries.
Syntax: Modules are imported using the require() function. Functionality is exposed by assigning to the module.exports object or by adding properties to the exports object.9
Dynamic Nature: The key challenge with CJS is its dynamic nature. require() is a regular function that is executed synchronously at the point it appears in the code.11 The path passed to
require() can be a variable or a dynamically constructed string (e.g., require('./locales/' + userLocale + '.js')). This makes it impossible for a purely static analysis tool to determine all dependencies in the general case.12

1.3.2. Static vs. Dynamic Analysis Dictates Tooling Strategy

The dichotomy between ESM and CJS is the single most important factor for a JavaScript analysis tool. It is not possible to use the same analysis strategy for both.
For ESM: The tool can achieve high confidence. By parsing the AST for ImportDeclaration, ExportNamedDeclaration, and ExportDefaultDeclaration nodes, it can build a complete and accurate dependency graph. The analysis is deterministic and comprehensive.
For CJS: The tool's confidence is lower and its capabilities are limited. It can easily parse simple cases like const myModule = require('./my-module');. However, it cannot resolve a dynamic require(variablePath). This means the analysis will be incomplete.
The static analysis tool must therefore implement a multi-stage strategy for JavaScript:
Detect Module System: The first step is to determine which system a file is using. This can be done by checking for import/export keywords (indicating ESM) or require/module.exports (indicating CJS). Additionally, in Node.js, the "type": "module" field in the nearest package.json file signals that .js files should be treated as ESM.11
Apply Correct Analysis: Based on the detected system, the tool must apply the appropriate analysis engine.
Handle Ambiguity: For CJS, the tool should flag dynamic require calls as "unresolved dependencies" or "potential architectural violations" and may require manual configuration from the user to provide hints for resolving these dynamic paths.
This fundamental difference dictates that the tool's architecture cannot be monolithic. It must be designed to handle the varying levels of analytical certainty that arise from these two distinct module paradigms.
Language Boundary Heuristics Summary
Language
Rust


Python


JavaScript (ESM)
JavaScript (CJS)


Section 2: Deconstructing Abstraction Layers in Software Architecture

To automatically validate a software architecture, the analysis tool must first possess a formal model of what an architecture is. This involves translating high-level design principles into a concrete set of rules and definitions. This section deconstructs the concept of abstraction layers, defines a canonical set of layers, and shows how popular architectural patterns map onto this canonical model.

2.1. The Principle of Abstraction and Layering

At its core, an abstraction layer is a technique for hiding the implementation details of a subsystem, thereby managing complexity.14 It creates a deliberate separation between two or more parts of a program, allowing them to evolve independently.15 This is achieved through a well-defined interface that exposes only the necessary functionality while concealing the underlying mechanics. As famously stated by David Wheeler, "All problems in computer science can be solved by another level of indirection," and abstraction layers are a primary form of such indirection.15
Layering extends this principle into a formal hierarchy. A layered architecture organizes a system into a stack of layers, where each layer provides services to the layer above it and consumes services from the layer below it.14 This structure imposes a strict rule on dependencies: a layer can only depend on layers below it. An "upward" dependency, where a lower layer depends on a higher one, violates the hierarchy and undermines the goal of separation.16 This unidirectional dependency flow is the central principle that a static analysis tool for architectural validation must enforce.

2.2. Canonical Layers: Presentation, Application, Data, and Infrastructure

While specific applications may have unique layering schemes, a canonical set of layers is found in most enterprise and web applications. The analysis tool should use this canonical model as its default framework, allowing users to map their project's structure onto it.
Presentation Layer (UI/API Endpoints): This is the outermost layer, responsible for all interactions with the outside world. This is not limited to a graphical user interface (GUI); it also includes command-line interfaces (CLIs), and, crucially for modern systems, web API endpoints that serve other applications.17 The Presentation Layer's primary job is to receive requests, translate them into calls to the Application Layer, and then format the results for the user or client system. It should contain minimal to no business logic.18 In an MVC pattern, this layer typically encompasses the View and the Controller.19
Application Layer (Business Logic/Use Cases): This layer represents the heart of the application's functionality. It contains the application-specific business rules and orchestrates the system's use cases.18 For example, in an e-commerce application, a "Place Order" use case would reside here. It would coordinate fetching the user, validating the product inventory, and directing the domain entities to perform their operations. This layer acts as a crucial intermediary, decoupling the Presentation Layer from the intricacies of the domain and data access.18 It should be independent of any specific UI or database technology.
Data Access Layer (Persistence/Repositories): This layer's sole responsibility is to provide an abstract interface for data persistence. It hides the specific details of the data storage mechanism (e.g., whether it's a SQL database, a NoSQL document store, or a remote API) from the rest of the application, particularly the Application Layer.22 It typically implements the Repository pattern, exposing methods like
findById, save, and delete. By abstracting data access, this layer allows the underlying database technology to be changed without requiring modifications to the business logic.18
Infrastructure Layer: This layer contains the concrete implementations of everything external to the application. This includes web frameworks (like Express or ASP.NET Core), database drivers (like node-postgres or Entity Framework Core), clients for third-party services, and other low-level technical details.19 The key principle is that the rest of the application communicates with this layer only through abstractions (interfaces or traits) defined in the higher layers. For example, the Application Layer defines an
IUserRepository interface, and the Infrastructure Layer provides a PostgresUserRepository that implements it.

2.3. Architectural Patterns as Layering Strategies

Well-known architectural patterns like MVC, Hexagonal Architecture, and Clean Architecture are not mutually exclusive alternatives but rather different strategies for implementing the canonical layers and enforcing their separation. Understanding their mapping is crucial for configuring the analysis tool correctly for a given project.
Model-View-Controller (MVC): MVC is one of the oldest and most common patterns for structuring applications with user interfaces.20
Model: Traditionally represents the application's data and business logic. In many modern web frameworks, the Model has been reduced to simple data structures (Data Transfer Objects or ORM entities), with the business logic moving into "service" classes. This component often blurs the Application and Data Access layers.25
View: Responsible for rendering the UI. This maps directly to the Presentation Layer.20
Controller: Receives user input from the View, interacts with the Model, and selects the next View to display. The Controller acts as the entry point and orchestrator, mapping to the request-handling part of the Presentation Layer and sometimes containing logic that belongs in the Application Layer.20 A common architectural violation in MVC applications is placing business or data access logic directly within the Controller, bypassing any service or repository layer.
Hexagonal Architecture (Ports and Adapters): This pattern provides a more robust model for separation by focusing on the boundary between the application's core and the outside world.28
Inside (The Hexagon): This is the application's core, containing the business logic and domain entities. It corresponds to the Application and Domain/Data Access (abstractions only) layers. It is completely isolated from any specific technology.28
Ports: These are the interfaces defined by the core application. They define the contracts for how the outside world can interact with the application (e.g., an OrderService port) or how the application expects to interact with the outside world (e.g., an OrderRepository port). These ports are the boundaries of the Application Layer.19
Adapters: These are the concrete implementations of the ports and reside in the Infrastructure Layer. A "driving adapter" could be a web controller that calls a port, while a "driven adapter" could be a PostgreSQL repository that implements a repository port.19
Clean Architecture: Proposed by Robert C. Martin, this pattern formalizes and extends the principles of Hexagonal Architecture into a more prescriptive model based on concentric circles and a strict dependency rule.19
Layers (from inside out):
Entities: Enterprise-wide business rules and data structures.
Use Cases: Application-specific business logic that orchestrates the flow of data to and from Entities.
Interface Adapters: Adapters that convert data between the format convenient for Use Cases/Entities and the format for external agencies (e.g., Presenters, Gateways, Controllers).
Frameworks and Drivers: The outermost layer containing all the details: the UI, the database, the web framework, etc..21
The Dependency Rule: This is the central tenet of Clean Architecture. Source code dependencies must only point inwards. An inner circle must not know anything about an outer circle. For example, a Use Case cannot have a dependency on a web framework or a concrete database class.21 This is achieved through the Dependency Inversion Principle, where inner layers define interfaces (abstractions) that outer layers implement.
A crucial realization for building an effective analysis tool is that these patterns are convergent. They all strive to achieve the same goal—separation of concerns and dependency inversion—using slightly different terminology and levels of prescription.28 Clean Architecture can be seen as a unification of the principles found in Hexagonal and other similar patterns.31 This means the analysis tool does not need a unique, complex model for each pattern. It can be built around the core concepts of Clean Architecture: a configurable set of layers and a strict, unidirectional dependency rule. The tool's configuration would then simply map a project's directories (e.g.,
/app/controllers, /app/use_cases) to the tool's canonical layers (presentation, application), allowing it to analyze any of these architectural styles effectively.
Architectural Pattern Layer Mapping
Canonical Layer
Presentation
Application (Business Logic)
Data Access
Infrastructure


Section 3: Algorithmic Detection of Architectural Violations

With a formal model of language-specific boundaries and architectural layers, the core logic of the static analysis tool can be designed. This involves a multi-step process: first, constructing a comprehensive dependency graph from the source code; second, checking this graph against user-defined architectural rules to find violations; and third, handling the necessary exceptions for legitimate cross-layer dependencies.

3.1. Building a Multi-Language Dependency Graph via AST Analysis

The foundational data structure for architectural analysis is a directed graph representing the dependencies between all modules in a codebase.
Graph Structure:
Nodes: Each node in the graph represents a single, addressable module. In Python and JavaScript, this is typically a source file. In Rust, this is a module within the crate's module tree.
Edges: A directed edge from Node A to Node B (A -> B) exists if the source code of module A contains a statement that creates a dependency on module B.
Algorithm: The construction of this graph requires parsing the entire codebase.
File Discovery: Recursively scan the project directory to find all relevant source files (e.g., .rs, .py, .js, .mjs, .cjs).
AST Parsing: For each discovered file, use a language-appropriate parser to generate an Abstract Syntax Tree (AST). Recommended libraries include syn for Rust, Python's built-in ast module, and @babel/parser for JavaScript.32 These libraries convert raw source text into a traversable tree structure representing the code's syntax.35
Dependency Extraction: Traverse each AST to find all dependency-creating nodes. This includes use statements in Rust, import and import from statements in Python, and import/export from statements (ESM) or require() calls (CJS) in JavaScript.
Path Resolution: For each dependency found, resolve the import path (e.g., '../services/user_service') to an absolute file path within the project. This step can be complex, as it needs to account for language-specific resolution rules (e.g., Python's sys.path, JavaScript's node_modules resolution).
Graph Construction: For each resolved dependency from file A to file B, add a directed edge from the node representing A to the node representing B in the dependency graph. The result is a complete, project-wide map of inter-module dependencies.36

3.2. Enforcing the Dependency Rule: Detecting Illegal "Upward" Dependencies

The most common architectural violation is an improper dependency that breaks the layered hierarchy, often called an "upward" or "inward" dependency violation.16
Algorithm:
Layer Definition (Configuration): The tool must be configured with the project's architectural layout. This is best achieved with a configuration file (e.g., architecture.yml) where the user maps file path patterns (glob patterns) to named layers. This approach is used by commercial tools like SonarQube.39
YAML
layers:
  - name: "presentation"
    level: 1
    paths:
      - "src/api/**"
      - "src/controllers/**"
  - name: "application"
    level: 2
    paths:
      - "src/use_cases/**"
      - "src/services/**"
  - name: "domain"
    level: 3
    paths:
      - "src/domain/**"
  - name: "infrastructure"
    level: 4
    paths:
      - "src/db/**"
      - "src/messaging/**"


Rule Definition (Configuration): The user defines the allowed dependency flow between layers, typically based on their assigned levels. The fundamental rule is that a layer can only depend on layers with an equal or higher level number (i.e., layers "below" it).
YAML
rules:
  - from: "presentation"
    to: ["application"]
  - from: "application"
    to: ["domain", "infrastructure"] # Application layer can use domain objects and infrastructure abstractions


Violation Check:
Iterate through every edge (U -> V) in the module dependency graph.
For both module U and module V, determine their respective layers (LayerU, LayerV) and levels (LevelU, LevelV) by matching their file paths against the configured patterns.
Violation Condition: A dependency rule violation occurs if LevelU < LevelV (an upward dependency).
Alternatively, if using explicit from/to rules, a violation occurs if there is no rule allowing a dependency from LayerU to LayerV.
When a violation is found, report it with detailed context: source file (U), dependency file (V), the invalid layer transition (LayerU -> LayerV), and the line number of the import statement.

3.3. Detecting Circular Dependencies Between Architectural Layers

A circular dependency is a sequence of dependencies that starts and ends at the same module or layer (e.g., A -> B -> C -> A). Cycles at the module level are often caught by compilers or loaders. However, cycles at the architectural layer level are more insidious and represent a severe breakdown of separation of concerns.16 For example, if the Presentation layer depends on the Application layer, which in turn depends on the Presentation layer, the two layers effectively collapse into one large, tightly coupled component.
Algorithm (Depth-First Search for Cycle Detection):
Build Layer Graph: Create a new, higher-level directed graph where each node is an architectural layer defined in the configuration.
Populate Layer Edges: Iterate through the module dependency graph. For every edge (U -> V) where Layer(U) is different from Layer(V), add a directed edge from Layer(U) to Layer(V) in the layer graph. Duplicate edges can be ignored.
Detect Cycles with DFS: Perform a cycle detection algorithm on the layer graph. A standard approach uses a Depth-First Search (DFS) and three sets of nodes to track visitation state 40:
white_set: All nodes not yet visited.
gray_set: Nodes currently in the recursion stack of the DFS traversal.
black_set: Nodes that have been completely visited (including all their descendants).
Traversal Logic:
For each node L in the graph, start a DFS traversal if L is in the white_set.
Move L from white_set to gray_set.
For each neighbor N of L:
If N is in the gray_set, a back edge has been found. This indicates a cycle. Report the cycle and terminate.
If N is in the white_set, recursively call DFS on N.
Once all neighbors of L have been visited, move L from gray_set to black_set.
If the algorithm completes without finding a back edge, the architecture is acyclic. If a cycle is found, the path can be reconstructed from the recursion stack to provide a clear error message (e.g., "Circular dependency detected: presentation -> application -> infrastructure -> presentation").

3.4. Acknowledging Reality: Handling Legitimate Cross-Cutting Concerns

A rigid layer dependency checker will fail on any real-world application due to the existence of cross-cutting concerns. These are aspects of a program, such as logging, authentication, caching, data validation, and transaction management, that are not confined to a single layer but must be applied across many of them.41 For example, both the Presentation and Application layers might need to write to a log file. A naive check would flag the dependency from these layers to a logging utility as a violation.
Treating these concerns as first-class architectural elements, rather than messy exceptions, is crucial for the tool's utility. They are orthogonal to the primary layering of application functionality.42 The tool must provide mechanisms to handle them gracefully.
Pattern 1: Orthogonal Layers/Whitelisted Modules: The most robust solution is to allow the user to define special "orthogonal" layers or whitelist specific modules in the configuration. Any module in any standard layer is permitted to depend on a module within an orthogonal layer.
YAML
# architecture.yml
layers:
  #... standard layers...
orthogonal_layers:
  - name: "logging"
    paths: ["src/utils/logger.py"]
  - name: "security"
    paths: ["src/auth/**"]

The violation checker would then permit any dependency where the target module (V) belongs to an orthogonal layer.
Pattern 2: Recognizing AOP and Middleware: Advanced analysis can be configured to recognize common patterns for implementing cross-cutting concerns that use dependency inversion, such as decorators (Python), attributes (C#), or middleware pipelines (ASP.NET Core, Express.js).44 In these cases, the business logic component does not directly depend on the cross-cutting concern; the framework or runtime injects the behavior. The tool can be taught to ignore the apparent dependency introduced by the decorator/attribute itself.
Pattern 3: Violation Suppression: For cases that cannot be modeled cleanly, the tool must support suppression mechanisms. This allows developers to acknowledge and document an accepted deviation from the rules.
In-code Comments: A special comment on the line of an import statement (e.g., // arch-check-ignore: logging is a cross-cutting concern) can suppress a violation for that specific line.46
Configuration-based Suppression: The configuration file can contain a list of specific violations to ignore, identified by the source module, target module, and rule being violated. This is a common feature in tools like NDepend and SonarQube.47
By combining these strategies, the analysis tool can enforce strict architectural rules while remaining practical and adaptable to the complexities of real-world software development.
Cross-Cutting Concern Implementation Strategies
Concern
Logging & Tracing
Authentication & Authorization
Caching
Validation
Transaction Management


Section 4: Practical Implementation and Code Pattern Analysis

This section provides concrete code examples of proper and improper architectural boundaries, along with detailed, language-specific algorithms for extracting dependency information using Abstract Syntax Tree (AST) analysis. These examples serve as a practical guide for implementing the core logic of the static analysis tool.

4.1. Code Patterns: Proper vs. Improper Abstraction Boundaries

The following examples illustrate common architectural violations and their corrected, well-architected counterparts. The analysis tool should be designed to detect the "improper" patterns and validate the "proper" ones.

4.1.1. Python: Bypassing the Application Layer

A frequent violation in web applications is having the Presentation Layer (e.g., a Flask or Django controller) directly access the Data Access Layer (e.g., a repository), bypassing the Application Layer where business logic should reside.
Improper Pattern: Controller Directly Accesses Repository
Python
# my_app/controllers/user_controller.py (Presentation Layer)
from flask import render_template
# VIOLATION: Presentation layer depends directly on Data Access layer.
from my_app.repositories.user_repo import UserRepo

def get_user_profile(user_id):
    """
    This controller fetches user data directly and contains presentation logic.
    It violates the dependency rule by importing from the 'repositories' module.
    """
    repo = UserRepo()
    user_entity = repo.find_by_id(user_id) # Data access logic in controller.

    if not user_entity:
        return "User not found", 404

    # Presentation formatting logic mixed with control flow.
    return render_template('user_profile.html', user=user_entity)

This code is brittle because any change to the UserRepo or the database schema could force a change in the controller. Furthermore, business logic (like handling a "not found" case) is tangled with presentation concerns.
Proper Pattern: Controller -> Service -> Repository
Python
# my_app/controllers/user_controller.py (Presentation Layer)
from flask import render_template
# CORRECT: Presentation depends on Application layer.
from my_app.services.user_service import UserService

def get_user_profile(user_id):
    """
    This controller delegates business logic to the UserService.
    Its only concerns are handling the request and rendering the view.
    """
    user_service = UserService() # In a real app, this would be injected.
    user_dto = user_service.get_user_for_profile(user_id)

    if not user_dto:
        return "User not found", 404

    return render_template('user_profile.html', user=user_dto)

# my_app/services/user_service.py (Application Layer)
from my_app.repositories.user_repo import IUserRepo # Depends on an abstraction.
from my_app.db.concrete_repo import UserRepo # Concrete repo for instantiation.

class UserService:
    def __init__(self, repo: IUserRepo = UserRepo()): # Dependency Inversion.
        self.repo = repo

    def get_user_for_profile(self, user_id):
        """
        Contains the business logic for fetching and preparing user data.
        """
        user_entity = self.repo.find_by_id(user_id)
        if user_entity and user_entity.is_active:
            # Business logic: e.g., transform entity to a Data Transfer Object (DTO).
            return {'id': user_entity.id, 'name': user_entity.name}
        return None

This corrected version adheres to the Dependency Rule. The controller only knows about the UserService. The UserService contains the business logic and depends on an abstraction (IUserRepo), not a concrete implementation, following the Dependency Inversion Principle.50 This makes the system modular, testable, and flexible.

4.1.2. Rust: Violating the Clean Architecture Dependency Rule

In Rust, the strong type and module system makes it easier to enforce boundaries, but violations are still possible. A common error is having an inner layer (like Use Cases) depend directly on a concrete implementation from an outer layer (like Frameworks).
Improper Pattern: Use Case Depends on Concrete Framework
Rust
// my_crate/src/use_cases.rs (Inner "Use Cases" Layer)
use crate::domain::User;
// VIOLATION: Inner layer depends on a concrete type from the outer "Frameworks" layer.
use crate::frameworks::database::PostgresConnection;

pub fn create_user_use_case(name: &str) -> Result<User, String> {
    // Direct instantiation of a framework detail. Tightly coupled.
    let mut conn = PostgresConnection::connect("db_url")
       .map_err(|e| e.to_string())?;

    let new_user = User::new(name.to_string());
    // SQL logic is leaked into the use case.
    conn.execute("INSERT INTO users (id, name) VALUES ($1, $2)",
                 &[&new_user.id, &new_user.name])
       .map_err(|e| e.to_string())?;

    Ok(new_user)
}

This code violates the Dependency Rule.21 The
use_cases module, which should only contain pure business logic, now knows about PostgresConnection and even SQL syntax. Swapping the database would require rewriting this entire use case.
Proper Pattern: Use Case Depends on Abstract Trait
Rust
// my_crate/src/application/repositories.rs (Defines abstraction in Application Layer)
use crate::domain::User;
// The "port" in Hexagonal terms. An abstract contract.
pub trait UserRepository {
    fn save(&self, user: &User) -> Result<(), String>;
}

// my_crate/src/application/use_cases.rs (Inner "Use Cases" Layer)
use crate::domain::User;
// CORRECT: Depends only on the abstract trait from its own layer or an inner one.
use crate::application::repositories::UserRepository;

pub fn create_user_use_case(repo: &impl UserRepository, name: &str) -> Result<User, String> {
    let new_user = User::new(name.to_string());
    repo.save(&new_user)?;
    Ok(new_user)
}

// my_crate/src/infrastructure/postgres_repo.rs (Outer "Infrastructure" Layer)
use crate::application::repositories::UserRepository; // Implements the trait.
use crate::domain::User;
use crate::frameworks::database::PostgresConnection; // Uses framework detail.

pub struct PostgresUserRepository {
    conn: PostgresConnection,
}

impl UserRepository for PostgresUserRepository {
    fn save(&self, user: &User) -> Result<(), String> {
        // Implementation details are hidden here.
        self.conn.execute("...", &[&user.id, &user.name]).map_err(|e| e.to_string())?;
        Ok(())
    }
}

This version correctly inverts the dependency.52 The use case depends only on the
UserRepository trait (the abstraction). The concrete PostgresUserRepository lives in the infrastructure layer and implements that trait. The business logic is now completely decoupled from the database implementation, making it easy to test and maintain.

4.2. AST Traversal Algorithms for Dependency Extraction

The core of the analysis tool is its ability to walk the AST of a source file and extract dependency information.

4.2.1. Rust (syn crate)

The syn crate is the de-facto standard for parsing Rust code into an AST, especially within procedural macros, but it is equally useful for standalone analysis tools.54
Algorithm:
Read the Rust source file into a string.
Use syn::parse_file(&source_code_string) to generate a syn::File object, which is the root of the AST.55
Create a visitor struct that implements the syn::visit::Visit trait. This trait provides methods for visiting each type of node in the AST.
Implement the visit_item_use(&mut self, i: &'ast syn::ItemUse) method on your visitor. This method is automatically called for every use statement in the file.
Inside visit_item_use, the i.tree field contains a syn::UseTree. This is a recursive structure that represents the path of the import (e.g., std::collections::HashMap). Recursively walk this UseTree to reconstruct the full import path as a string.
Store the collected import paths. These are the module's dependencies.
Example (Conceptual Code):
Rust
use syn::{visit::{self, Visit}, ItemUse, File};
use std::collections::HashSet;

struct DependencyVisitor {
    dependencies: HashSet<String>,
}

impl<'ast> Visit<'ast> for DependencyVisitor {
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        // A recursive function to flatten the UseTree into a string path
        fn path_to_string(tree: &syn::UseTree) -> String {
            match tree {
                syn::UseTree::Path(p) => format!("{}::{}", p.ident, path_to_string(&p.tree)),
                syn::UseTree::Name(n) => n.ident.to_string(),
                syn::UseTree::Glob(_) => "*".to_string(),
                // Handle other cases like Group, Rename
                _ => "".to_string(),
            }
        }
        self.dependencies.insert(path_to_string(&i.tree));
    }
}

pub fn find_rust_dependencies(source_code: &str) -> HashSet<String> {
    let ast: File = syn::parse_file(source_code).unwrap();
    let mut visitor = DependencyVisitor { dependencies: HashSet::new() };
    visitor.visit_file(&ast);
    visitor.dependencies
}



4.2.2. Python (ast module)

Python's built-in ast module provides all the necessary tools for parsing and traversing Python code.33
Algorithm:
Read the Python source file into a string.
Use ast.parse(&source_code_string) to generate an AST node, which is the root of the tree.33
Create a visitor class that inherits from ast.NodeVisitor.57
Implement visit_Import(&self, node: ast.Import) and visit_ImportFrom(&self, node: ast.ImportFrom) methods.
In visit_Import, iterate through node.names. Each element is an alias object with a name attribute representing the imported module.
In visit_ImportFrom, the node.module attribute gives the name of the module being imported from. The node.level attribute indicates the level of relative import (0 for absolute, 1 for ., 2 for .., etc.). node.names contains the specific items being imported.
To identify function calls (for more granular analysis), implement visit_Call(&self, node: ast.Call). The node.func attribute describes the function being called. This can be a simple ast.Name (for a local function) or a nested ast.Attribute (for a method call like module.service.do_work()).58
Example (Conceptual Code):
Python
import ast
from typing import Set

class DependencyVisitor(ast.NodeVisitor):
    def __init__(self):
        self.dependencies = set()

    def visit_Import(self, node: ast.Import):
        for alias in node.names:
            self.dependencies.add(alias.name)
        self.generic_visit(node)

    def visit_ImportFrom(self, node: ast.ImportFrom):
        # Handle relative imports by prepending dots
        relative_prefix = '.' * node.level
        if node.module:
            self.dependencies.add(f"{relative_prefix}{node.module}")
        else: # e.g., from. import foo
            self.dependencies.add(relative_prefix)
        self.generic_visit(node)

def find_python_dependencies(source_code: str) -> Set[str]:
    tree = ast.parse(source_code)
    visitor = DependencyVisitor()
    visitor.visit(tree)
    return visitor.dependencies



4.2.3. JavaScript (@babel/parser and @babel/traverse)

For JavaScript, the Babel toolchain provides robust libraries for parsing modern syntax (including JSX and TypeScript) into an ESTree-compatible AST and traversing it.34
Algorithm:
Read the JavaScript/TypeScript source file into a string.
Use @babel/parser's parse method to generate the AST. Ensure you pass appropriate options to enable plugins for syntax like TypeScript or JSX.
Use @babel/traverse to walk the AST with a visitor object. The visitor is a simple object with keys corresponding to AST node types.62
For ESM: The visitor should have a method for ImportDeclaration(path). The dependency is found in path.node.source.value. The same applies to ExportNamedDeclaration and ExportAllDeclaration if they have a source property.
For CJS: The visitor needs a method for CallExpression(path). Inside, it must check if path.node.callee.name is equal to 'require'. If it is, the dependency is typically the first argument: path.node.arguments.value. This only works for static, string-literal requires.64
Exports (CJS): To find exports, the visitor needs a method for AssignmentExpression(path) and must check if the left-hand side of the assignment (path.node.left) corresponds to module.exports or exports.someName.
Example (Conceptual Code):
JavaScript
const parser = require('@babel/parser');
const traverse = require('@babel/traverse').default;

function findJsDependencies(sourceCode) {
    const dependencies = new Set();
    const ast = parser.parse(sourceCode, {
        sourceType: 'module', // or 'script' for CJS
        plugins: ['typescript'] // example plugin
    });

    const visitor = {
        // Visitor for ES6 Imports
        ImportDeclaration(path) {
            dependencies.add(path.node.source.value);
        },
        // Visitor for CommonJS requires
        CallExpression(path) {
            if (
                path.node.callee.type === 'Identifier' &&
                path.node.callee.name === 'require' &&
                path.node.arguments.length > 0 &&
                path.node.arguments.type === 'StringLiteral'
            ) {
                dependencies.add(path.node.arguments.value);
            }
        }
    };

    traverse(ast, visitor);
    return dependencies;
}



Conclusions and Recommendations

This report has provided a comprehensive technical blueprint for developing a static analysis tool capable of identifying and validating architectural abstraction boundaries in multi-language Rust, Python, and JavaScript codebases. The analysis reveals that while the high-level goal is uniform—to enforce architectural integrity—the implementation details are profoundly language-specific. A successful tool must be built on a nuanced understanding of each language's module system and a flexible, configurable model of software architecture.
The primary conclusions and recommendations are as follows:
Language-Specific Parsers are Non-Negotiable: The mechanisms for defining module boundaries, public APIs, and dependencies are fundamentally different across the three languages.
Rust's explicit visibility system (pub, pub(crate)) provides strong, compile-time signals that must be precisely interpreted to differentiate between external and internal APIs.
Python's system relies on filesystem conventions (__init__.py) and community conventions (_private, __all__), requiring a heuristic-based approach to identify intended boundaries.
JavaScript's ESM/CJS dichotomy necessitates two distinct analysis strategies: a high-confidence static analysis for ESM and a lower-confidence, more limited analysis for the dynamic nature of CJS.
Adopt a Unified Architectural Model: Despite the varied terminology of patterns like MVC, Hexagonal, and Clean Architecture, they converge on the core principles of Separation of Concerns and the Dependency Inversion Principle. It is recommended that the tool be built around a single, canonical layer model inspired by Clean Architecture. This model, centered on a strict, unidirectional Dependency Rule, is general enough to represent all these patterns. The tool's configuration should focus on mapping a project's physical layout (directories and files) to these logical layers (e.g., Presentation, Application, Data Access, Infrastructure).
Prioritize a Robust Dependency Graph: The cornerstone of the analysis engine is a complete, project-wide dependency graph. This requires robust AST parsing and path resolution for each language. Investment in this foundational component will enable all subsequent architectural checks.
Implement Cycle Detection at the Layer Level: While module-level cycles are problematic, architectural decay often manifests as cycles between high-level layers. Implementing a DFS-based cycle detection algorithm on the abstracted layer graph is critical for identifying severe architectural breakdowns that tightly couple major components of the system.
Treat Cross-Cutting Concerns as First-Class Citizens: A naive dependency checker will be impractical due to ubiquitous cross-cutting concerns like logging, authentication, and caching. The tool's design must treat these not as exceptions to be ignored, but as a fundamental part of the architecture. It is strongly recommended to implement a configuration mechanism for defining "orthogonal layers" or whitelisted modules. This allows the tool to enforce the primary architectural layering while explicitly permitting these necessary, legitimate cross-layer dependencies. This approach is superior to ad-hoc, in-code suppression comments, as it keeps the architectural definition centralized and explicit.
In summary, the development of the proposed static analysis tool is a feasible but complex undertaking. Success hinges on embracing the heterogeneity of the target languages while building upon a unified and powerful architectural model. By combining language-specific AST analysis with a configurable engine based on the principles of layered architecture and the Dependency Rule, it is possible to create a tool that provides invaluable, automated feedback to maintain architectural integrity and prevent design erosion in large, polyglot systems.
Works cited
How do I keep a mod private and use it in another module within the same project in Rust?, accessed July 3, 2025, https://stackoverflow.com/questions/74974435/how-do-i-keep-a-mod-private-and-use-it-in-another-module-within-the-same-project
It's dangerous to go alone, `pub` `mod` `use` this.rs - Schneems, accessed July 3, 2025, https://schneems.com/2023/06/14/its-dangerous-to-go-alone-pub-mod-use-thisrs/
Visibility and privacy - The Rust Reference, accessed July 3, 2025, https://doc.rust-lang.org/reference/visibility-and-privacy.html
crate level visibility for modules in Rust? - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/2ls452/crate_level_visibility_for_modules_in_rust/
6. Modules — Python 3.13.5 documentation, accessed July 3, 2025, https://docs.python.org/3/tutorial/modules.html
What does '__init__.py' do in Python? - YouTube, accessed July 3, 2025, https://www.youtube.com/watch?v=VEbuZox5qC4&pp=0gcJCfwAo7VqN5tD
What's your opinion on what to include in __init__.py ? : r/Python - Reddit, accessed July 3, 2025, https://www.reddit.com/r/Python/comments/1bbbwk/whats_your_opinion_on_what_to_include_in_init_py/
JavaScript modules - JavaScript | MDN, accessed July 3, 2025, https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Modules
Documentation - Modules - TypeScript, accessed July 3, 2025, https://www.typescriptlang.org/docs/handbook/2/modules.html
16. Modules - Exploring JS, accessed July 3, 2025, https://exploringjs.com/es6/ch_modules.html
CommonJS vs. ES Modules | Better Stack Community, accessed July 3, 2025, https://betterstack.com/community/guides/scaling-nodejs/commonjs-vs-esm/
ES Modules and CommonJS: An Overview - DEV Community, accessed July 3, 2025, https://dev.to/costamatheus97/es-modules-and-commonjs-an-overview-1i4b
Explain the differences between CommonJS modules and ES modules in JavaScript | Quiz Interview Questions with Solutions - GreatFrontEnd, accessed July 3, 2025, https://www.greatfrontend.com/questions/quiz/explain-the-differences-between-commonjs-modules-and-es-modules
Abstraction layer - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Abstraction_layer
Abstraction Layers in Programming: An Overview – BMC Software | Blogs, accessed July 3, 2025, https://www.bmc.com/blogs/abstraction-layers/
Validating your application architecture - OutSystems 11 Documentation, accessed July 3, 2025, https://success.outsystems.com/documentation/11/app_architecture/designing_the_architecture_of_your_outsystems_applications/validating_your_application_architecture/
Global Three Layer Application Architecture - Tengiz Tutisani ..., accessed July 3, 2025, https://www.tutisani.com/software-architecture/global-three-layer-architecture.html
Layers in software architecture - Medium, accessed July 3, 2025, https://medium.com/@sagar.hudge/layers-in-software-architecture-c8cc16329ff6
Hexagonal vs. Clean architecture. Pros and cons, and demonstrate ..., accessed July 3, 2025, https://medium.com/@araxis/hexagonal-vs-clean-architecture-b11a6833136e
MVC Architecture Explained: Model, View, Controller | Codecademy, accessed July 3, 2025, https://www.codecademy.com/article/mvc-architecture-model-view-controller
The Clean Architecture Dependency Rule - InformIT, accessed July 3, 2025, https://www.informit.com/articles/article.aspx?p=2832399
Data Access Layer: Overview, Architecture & How to Build One - CData Software, accessed July 3, 2025, https://www.cdata.com/blog/data-access-layer
How to implement a program layered architecture? - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/post/How_to_implement_a_program_layered_architecture
Model–view–controller - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller
MVC Design Pattern - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/system-design/mvc-design-pattern/
Demystifying MVC: Understanding the Model-View-Controller Architecture - Medium, accessed July 3, 2025, https://medium.com/@finzyphinzy/demystifying-mvc-understanding-the-model-view-controller-architecture-85c88a558951
Understanding the MVC Pattern for Clean Code Architecture | by Rivan Prawira - Medium, accessed July 3, 2025, https://medium.com/@prawiraa.rivan/understanding-the-mvc-pattern-for-clean-code-architecture-0f37e55aa817
Understanding Hexagonal and Clean Architectures | by ISHII (石井) | May, 2025 | Medium, accessed July 3, 2025, https://schimizu.com/understanding-hexagonal-and-clean-architectures-df64f597ca79
Breaking the MVC Mold: Clean & Hexagonal Architecture | by Luc Lekene | Medium, accessed July 3, 2025, https://medium.com/@cedric-lekene/breaking-the-mvc-mold-clean-hexagonal-architecture-ea7b5a30e32a
Everything You Need to Know About Clean Architecture | Bitloops Docs, accessed July 3, 2025, https://bitloops.com/docs/bitloops-language/learning/software-architecture/clean-architecture
Hexagonal vs. Clean Architecture: Same Thing Different Name? : r/programming - Reddit, accessed July 3, 2025, https://www.reddit.com/r/programming/comments/1l7vun6/hexagonal_vs_clean_architecture_same_thing/
rusty-ast - crates.io: Rust Package Registry, accessed July 3, 2025, https://crates.io/crates/rusty-ast
ast — Abstract Syntax Trees — Python 3.13.5 documentation, accessed July 3, 2025, https://docs.python.org/3/library/ast.html
Understanding Abstract Syntax Tree (AST) in Node.js | by Interviewer Live - Medium, accessed July 3, 2025, https://medium.com/@interviewer.live/understanding-abstract-syntax-tree-ast-in-node-js-893652fa0e4
rust_code_analysis - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/rust-code-analysis/*/rust_code_analysis/
ast-scope - PyPI, accessed July 3, 2025, https://pypi.org/project/ast-scope/
Building a Dependency Graph of Our Python Codebase | Our Success Stories, accessed July 3, 2025, https://www.python.org/success-stories/building-a-dependency-graph-of-our-python-codebase/
Graph My Code 1: Creating a Graph of Function Dependencies in Python - ψML, accessed July 3, 2025, https://simonstolarczyk.com/posts/graph/Graph_My_Code_1.html
Design and Architecture overview - SonarQube Docs, accessed July 3, 2025, https://docs.sonarsource.com/sonarqube-server/latest/design-and-architecture/overview/
Detecting Cycles and Ordering Dependencies: Graph Algorithms in Kotlin - Medium, accessed July 3, 2025, https://medium.com/@chetanshingare2991/detecting-cycles-and-ordering-dependencies-graph-algorithms-in-kotlin-a3807cf8a57c
Cross-cutting concern - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Cross-cutting_concern
Cross cutting concern example - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/23700540/cross-cutting-concern-example
Separation of concerns - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Separation_of_concerns
Balancing Cross-Cutting Concerns in Clean Architecture - Milan Jovanović, accessed July 3, 2025, https://www.milanjovanovic.tech/blog/balancing-cross-cutting-concerns-in-clean-architecture
Cross-Cutting Concerns - Ten Approaches - Jesse Builds Software, accessed July 3, 2025, https://jessemcdowell.ca/2024/05/Cross-Cutting-Concerns/
How do I exclude certain code from certain rules? - SonarQube Server / Community Build, accessed July 3, 2025, https://community.sonarsource.com/t/how-do-i-exclude-certain-code-from-certain-rules/108933
Narrowing the Focus | SonarQube Docs, accessed July 3, 2025, https://scm.thm.de/sonar/documentation/project-administration/narrowing-the-focus/
How can we ignore some SonarQube rules in Java? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/39109228/how-can-we-ignore-some-sonarqube-rules-in-java
Suppress NDepend Issues, accessed July 3, 2025, https://www.ndepend.com/docs/suppress-issues
Dependency Inversion Principle in Python | by shailesh jadhav ..., accessed July 3, 2025, https://blog.nonstopio.com/dependency-inversion-principle-in-python-18bc0165e6f1
Dependency Inversion in Python - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/61358683/dependency-inversion-in-python
Clean Architecture in .NET: A Practical Guide with Examples | by ..., accessed July 3, 2025, https://medium.com/@roshikanayanadhara/clean-architecture-in-net-a-practical-guide-with-examples-817568b3f42e
Rules to Better Clean Architecture - SSW - Enterprise Software Development, accessed July 3, 2025, https://www.ssw.com.au/rules/rules-to-better-clean-architecture/
syn - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/syn
syn::parse - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/syn/latest/syn/parse/index.html
Analyzing Python Code with Python - Rotem Tamir, accessed July 3, 2025, https://rotemtam.com/2020/08/13/python-ast/
Analyzing Python Codes with AST (1) | by Ken Maeda | Medium, accessed July 3, 2025, https://medium.com/@whiteking64/analyzing-python-codes-with-ast-1-cd15e97dd79a
What is ast.Call(func,args,keywords,starargs,kwargs) in Python? - Educative.io, accessed July 3, 2025, https://www.educative.io/answers/what-is-astcallfuncargskeywordsstarargskwargs-in-python
Extracting the Module and Function Names from Python ASTs - Arumoy Shome, accessed July 3, 2025, https://arumoy.me/blogs/python-ast-extract-module-method-names/
babel/parser, accessed July 3, 2025, https://babeljs.io/docs/babel-parser
babel/traverse, accessed July 3, 2025, https://babeljs.io/docs/babel-traverse
How to find what is the dependency of a function, class, or variable in ES6 via AST, accessed July 3, 2025, https://dev.to/jennieji/find-what-is-affected-by-a-declaration-in-javascript-2d5c
babel-handbook/translations/hu/plugin-handbook.md at master - GitHub, accessed July 3, 2025, https://github.com/jamiebuilds/babel-handbook/blob/master/translations/hu/plugin-handbook.md
How do I traverse the scope of a Path in a babel plugin - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/44309639/how-do-i-traverse-the-scope-of-a-path-in-a-babel-plugin
Top 10 Examples of babel-traverse code in Javascript - CloudDefense.AI, accessed July 3, 2025, https://www.clouddefense.ai/code/javascript/example/babel-traverse
