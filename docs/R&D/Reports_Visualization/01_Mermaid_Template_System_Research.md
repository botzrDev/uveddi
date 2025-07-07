
Technical Specification for a Rust-Based Mermaid.js Code Analysis Visualization System


Section 1: Template Engine Architecture and Recommendation

This section provides a comprehensive analysis of the available template engine options for the Rust-based Mermaid.js generation system. It evaluates the fundamental architectural trade-offs between compiled and interpreted engines, conducts a comparative analysis of suitable runtime engines, and examines their performance and composition capabilities. The section culminates in a definitive engine recommendation and a practical integration guide, ensuring the selected technology aligns with the system's core requirements for dynamic, configuration-driven diagram generation.

1.1 The Interpreted vs. Compiled Engine Dichotomy: An Architectural Constraint

The primary architectural decision for the templating subsystem revolves around the choice between a compiled and an interpreted engine. This choice carries significant implications for both development workflow and runtime performance. A compiled template engine, such as Sailfish, Askama, or Maud, processes templates during the application's compilation phase (cargo build), converting them into highly optimized, native Rust code.1 This approach yields maximum runtime performance, with some benchmarks showing compiled engines like Sailfish to be orders of magnitude faster than their interpreted counterparts.
However, the system's core requirement is to generate diagrams based on definitions provided in a runtime configuration file, such as diagrams.toml. This file will specify which templates to use for which data sources. Consequently, the application must be capable of discovering, loading, and parsing these template files at runtime, after the main binary has already been compiled. This dynamic loading requirement fundamentally disqualifies compiled template engines. Their operational model is predicated on the template source code being known and available at compile time, a condition that cannot be met in this configuration-driven architecture.
Therefore, the system architecture mandates the use of an interpreted template engine. Interpreted engines, such as Handlebars, Tera, and Liquid, parse template files from strings or the filesystem at runtime.3 This provides the necessary flexibility to build a system where diagram generation logic is decoupled from the application's compiled code. The performance cost associated with runtime parsing is a necessary and acceptable trade-off to achieve the desired dynamic and configurable behavior. While benchmarks indicate a significant performance gap, the absolute performance of Rust-based interpreted engines is more than sufficient for the task of generating text-based Mermaid.js definitions, which is not a computationally intensive operation on the scale of high-throughput web request processing.1

1.2 Comparative Analysis of Runtime Template Engines

With the architectural constraint established, the selection process narrows to Rust's mature, interpreted template engines. The leading candidates are Handlebars, Tera, and Liquid, each with a distinct philosophy and feature set.
Handlebars (handlebars-rust)
The handlebars-rust crate is a Rust implementation of the popular Handlebars templating system, which originated in the JavaScript ecosystem.4 Its guiding philosophy is to maintain a strict separation of concerns by keeping application logic out of the templates. It provides a minimal but essential set of built-in control structures, primarily
#if for conditional rendering and #each for iteration.7 More complex logic is intended to be encapsulated within custom "helpers," which can be defined in Rust and registered with the engine.7 This approach promotes cleaner templates but can increase the amount of Rust code required for complex conditional logic.
For data handling, handlebars-rust is designed to work with JSON-like data structures. In practice, this means any data passed to a template for rendering must implement the serde::Serialize trait. The crate internally converts these Rust types into serde_json::Value for processing within the template context.6 The crate is considered rock-solid and is used in production by high-profile projects like the official Rust language website,
rust-lang.org.4 It enjoys robust support across the Rust web framework ecosystem, with integrations available for Axum, Rocket, Warp, and others.7
Tera
Tera is a powerful template engine explicitly inspired by the Jinja2 (Python) and Django template languages.4 Unlike Handlebars, Tera is designed to be more feature-rich within the template itself, allowing for more complex expressions and logic without requiring custom Rust helpers. It supports mathematical operations, logical comparisons, and a powerful system of filters and tests that can modify or evaluate data directly in the template.4 For example, a template can contain expressions like
{{ items | length }} or {% if user.age > 18 %}.
Tera uses its own tera::Context struct to hold data for rendering, but it provides a convenient Context::from_serialize method to create this context from any type that implements serde::Serialize, making it just as easy to use with standard Rust structs as Handlebars.11 Tera is also well-supported in the ecosystem and is used in various web frameworks and for scaffolding projects.13 Its documentation is widely regarded as excellent and comprehensive.11
Liquid (liquid-rust)
The liquid crate is a Rust port of the Liquid template language, originally developed in Ruby and famously used by the Shopify e-commerce platform.4 The crate's explicit goal is to achieve 100% compatibility with the Shopify/Liquid specification.4 While it is a functional and stable engine, its feature set is generally considered less powerful than that of Jinja2-inspired engines like Tera, particularly concerning advanced features like template inheritance.15 In the context of the Rust ecosystem, it has a smaller community footprint and fewer downloads compared to Handlebars and Tera.3 Given the advanced composition and dynamic styling requirements of this project, focusing the evaluation on the two more feature-rich and widely adopted engines, Handlebars and Tera, is the most pragmatic approach.

1.3 Performance Evaluation: Justifying the Trade-Off

Performance is a critical consideration, but it must be evaluated within the context of the architectural trade-offs. As established, interpreted engines are inherently slower than their compiled counterparts due to the overhead of parsing and evaluating templates at runtime. Benchmark suites, such as the one maintained by the Askama project, provide quantitative data on these differences.16
These benchmarks typically involve two types of workloads: one focused on raw data throughput (e.g., generating a large HTML table) and another focused on rendering logic (e.g., processing a structure with conditional statements and text that requires HTML escaping).16 In these tests, compiled engines like Sailfish demonstrate performance that can be over 100 times faster than interpreted engines like Handlebars.
However, when comparing the relevant interpreted engines, Handlebars and Tera exhibit broadly similar performance characteristics. The differences between them are minor and unlikely to be a deciding factor for this project's use case. The generation of Mermaid.js markup is a text-processing task that is not on the critical path of a high-frequency, low-latency system. The performance of either Handlebars or Tera, running as compiled Rust code, will be more than adequate and will not constitute a system bottleneck. The broader performance discussion often compares Rust to other languages like Node.js, where the outcome is highly dependent on the specific workload and implementation quality.5 Within the Rust ecosystem, the choice between interpreted engines should be driven by features, ergonomics, and maintainability rather than marginal performance differences.

1.4 Advanced Composition Patterns: Inheritance and Partials

A key requirement for building a maintainable and scalable templating system is the ability to reuse and compose templates. Both Handlebars and Tera provide robust mechanisms for this, primarily through template inheritance and partials (includes).
Handlebars Composition: The handlebars-rust crate facilitates template inheritance through a combination of built-in helpers. The {{> partial_name }} helper is used to include one template within another, which is suitable for simple composition.18 For more complex inheritance patterns, Handlebars uses a system of
{{#block}} and {{#partial}} helpers. A base template defines named {{#block "section_name"}}...{{/block}} sections, which act as placeholders with default content. A child template can then use {{#> partial "section_name"}}...{{/partial}} to override the content of the corresponding block in the parent.19 The canonical example demonstrating this pattern is found in the
examples/partials.rs file within the crate's source repository.6
Tera Composition: Tera implements a more direct and expressive inheritance model that will be immediately familiar to developers with experience in Django or Jinja2. A child template declares its parent using the {% extends "base.html" %} tag, which must be the first statement in the file.12 The parent template defines overridable sections using
{% block block_name %}...{% endblock block_name %}. The child template then overrides these blocks by defining its own blocks with the same names.11 Tera also provides a
{{ super() }} function, which allows a child block to render the content of its parent's block, enabling additive composition.12 This multi-level inheritance is parsed and validated by Tera when the templates are loaded, ensuring that inheritance chains are sound and free of circular dependencies.20
While both engines provide the necessary functionality, Tera's approach is more idiomatic and powerful for creating complex, multi-level template layouts. Its syntax is less verbose and more aligned with established best practices from other popular web development ecosystems.

1.5 Final Recommendation and Integration Guide

Based on the comprehensive analysis of features, performance, and composition patterns, this specification provides a definitive recommendation.
Recommendation: Tera is the recommended template engine for this system.
Justification:
Superior In-Template Logic: The project's core requirement is to dynamically style diagrams based on severity levels, which necessitates robust conditional logic within the templates. Tera's native support for filters (e.g., | lower), tests (e.g., is odd), and complex logical and mathematical expressions is far better suited for this task than Handlebars' more constrained, helper-centric approach. This allows for cleaner, more self-contained, and more maintainable templates that do not require extensive custom Rust code for simple presentation logic.4
Expressive and Idiomatic Inheritance: Tera’s {% extends %} and {% block %} syntax is a well-understood, powerful, and elegant pattern for template inheritance. It makes the creation and maintenance of complex, multi-level template hierarchies more straightforward and intuitive than the Handlebars equivalent.12
Excellent Developer Experience: The Jinja2-like syntax is familiar to a vast number of developers, significantly reducing the learning curve. Furthermore, the official Tera documentation is exceptionally clear, comprehensive, and provides numerous examples for all its features, which will accelerate development and onboarding.11
Integration Example (Rust):
The following code demonstrates a canonical integration pattern for Tera within a Rust application. It establishes a globally accessible, thread-safe Tera instance, which is a recommended best practice to avoid the cost of re-parsing templates on every request.12

Rust


use serde::Serialize;
use std::sync::{Arc, RwLock};
use tera::{Context, Tera};

// Represents the data structure passed to a template.
// It must derive `serde::Serialize`.
#
struct DiagramData {
    severity: String,
    nodes: Vec<String>,
    // Other relevant fields for diagram generation...
}

// A global, thread-safe, lazily-initialized Tera instance.
// The glob pattern "templates/**/*.tera" instructs Tera to load all files
// with a.tera extension from the 'templates' directory and its subdirectories.
pub static TERA: once_cell::sync::Lazy<Arc<RwLock<Tera>>> =
    once_cell::sync::Lazy::new(|| {
        let tera = match Tera::new("templates/**/*.tera") {
            Ok(t) => t,
            Err(e) => {
                println!("FATAL: Tera parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        Arc::new(RwLock::new(tera))
    });

/// Generates a diagram from a named template and its corresponding data.
///
/// # Arguments
/// * `template_name` - The name of the template file (e.g., "component_diagram.tera").
/// * `data` - A reference to the data structure to be rendered.
///
/// # Returns
/// A `Result` containing the rendered string or a `tera::Error`.
fn generate_diagram(
    template_name: &str,
    data: &DiagramData,
) -> Result<String, tera::Error> {
    // Create a Tera context from the serializable data structure.
    let context = Context::from_serialize(data)?;
    // Acquire a read lock on the global Tera instance.
    let tera_lock = TERA.read().unwrap();
    // Render the specified template with the given context.
    tera_lock.render(template_name, &context)
}


This implementation pattern provides a robust and performant foundation for the templating subsystem.
Table 1: Feature Comparison of Runtime Template Engines

Feature
Handlebars
Tera
Liquid
Base Syntax
Mustache-like {{ variable }}
Jinja2-like {{ variable }}
Shopify Liquid {{ variable }}
In-Template Logic
Minimal (#if, #each); logic deferred to Rust helpers 7
Extensive (math, comparisons, logic operators) 4
Moderate; less powerful than Tera 15
Extensibility
Custom Helpers (Rust functions or Rhai script) 7
Custom Filters, Functions, and Tests (Rust functions) 11
Custom Tags and Filters
Template Inheritance
{{> block}} and {{#> partial}} helpers 18
{% extends %} and {% block %} tags 12
Limited; no built-in block inheritance 15
Strict Mode
Opt-in via set_strict_mode(true) 6
Opt-in via configuration 11
Varies by implementation
Hot-Reloading
Built-in dev_mode feature 6
Via full_reload() method and external watcher 12
Not a standard feature
Ecosystem
Very High (1.9M downloads) 3
High (594K downloads) 3
Moderate (103K downloads) 3

Table 2: Summary of Template Engine Performance Benchmarks
Engine
Type
Big Table Benchmark (ns/iter)
Teams Benchmark (ns/iter)
Sailfish
Compiled
~1,000
~2,500
Askama
Compiled
~19,000
~45,000
Tera
Interpreted
~150,000
~70,000
Handlebars
Interpreted
~200,000
~80,000

Note: Performance figures are illustrative, based on relative data from sources like , , and 16, and can vary based on hardware and workload. They serve to demonstrate the performance difference between compiled and interpreted engines.

Section 2: Diagram Specifications for Code Analysis

This section provides the detailed specifications for the four required Mermaid.js diagram types. Each specification includes the intended purpose, the conceptual data schema required to drive the template, and a complete, production-ready Tera template. A unified styling framework is defined first to ensure a consistent visual language for severity across all diagrams.

2.1 Unified Styling Framework for Severity Highlighting

To provide clear and immediate visual feedback on code analysis results, a standardized styling system based on issue severity is required. This will be implemented using Mermaid's classDef syntax, which allows for the definition of reusable CSS-like style sets.21 A master template or partial will contain these definitions, which can then be applied dynamically within other templates based on a
severity field in the input data.
Severity Class Definitions:
The following classDef statements will be included in a base template (e.g., _base.tera) from which all diagram-specific templates will inherit. This ensures consistency and centralizes style management.

Code snippet


{# templates/_base.tera #}
...
{# --- Mermaid Style Definitions --- #}
classDef sev-critical fill:#8B0000,stroke:#FFFFFF,stroke-width:2px,color:#FFFFFF
classDef sev-high fill:#FF4500,stroke:#000000,stroke-width:2px,color:#000000
classDef sev-medium fill:#FFD700,stroke:#000000,stroke-width:1px,color:#000000
classDef sev-low fill:#90EE90,stroke:#000000,stroke-width:1px,color:#000000
classDef sev-info fill:#ADD8E6,stroke:#000000,stroke-width:1px,color:#000000
classDef link-critical stroke:#8B0000,stroke-width:4px
classDef link-high stroke:#FF4500,stroke-width:2px


Dynamic Application in Templates:
Child templates will apply these classes using Mermaid's ::: operator. Tera's lower filter will be used to normalize the severity string from the data source (e.g., "Critical", "HIGH") to match the class names (e.g., sev-critical, sev-high).

Code snippet


{# Example of applying a style to a node #}
{% for node in analysis_results.nodes %}
  {{ node.id }}[{{ node.label | escape }}]:::sev-{{ node.severity | lower }}
{% endfor %}


This combination of a centralized classDef block and dynamic application via Tera filters creates a robust, maintainable, and powerful styling framework. The escape filter is used on labels to prevent characters that conflict with Mermaid syntax from breaking the diagram.

2.2 Architectural Visualization: Component Diagrams

Purpose: To generate high-level visualizations of system architecture, showing the relationships between services, libraries, and logical groupings such as cloud environments or deployment zones.
Mermaid Type: The architecture-beta diagram type is specified for this purpose. It is semantically richer and more visually appropriate for architectural representation than a generic flowchart, offering distinct icons and grouping constructs.24
Conceptual Data Schema: The input data for this diagram should be structured to represent architectural elements.
groups: An array of objects, where each object defines a logical container.
id: String - A unique identifier for the group.
label: String - The human-readable name of the group.
icon: String - The name of a Mermaid or Iconify icon (e.g., cloud, logos:aws-ec2).24
parent: Option<String> - The ID of a parent group for nesting.
services: An array of objects representing individual components.
id: String - A unique identifier for the service.
label: String - The human-readable name of the service.
icon: String - The icon name.
group: String - The ID of the group this service belongs to.
severity: String - The severity level for styling (e.g., "info", "high").
edges: An array of objects defining connections.
from: String - The originating service ID, with optional side specifier (e.g., server:R).
to: String - The destination service ID, with optional side specifier (e.g., db:L).
label: Option<String> - An optional label for the edge.
Template Specification (component_diagram.tera):
Code snippet
{% extends "_base.tera" %}

{% block diagram %}
architecture-beta
    title {{ diagram_title | default(value="System Architecture Diagram") }}

    {# Define all groups first #}
    {% for group in groups %}
    group {{ group.id }}({{ group.icon }})["{{ group.label }}"]{% if group.parent %} in {{ group.parent }}{% endif %}
    {% endfor %}

    {# Define all services within their groups and apply severity styling #}
    {% for service in services %}
    service {{ service.id }}({{ service.icon }})["{{ service.label }}"] in {{ service.group }} :::sev-{{ service.severity | lower }}
    {% endfor %}

    {# Define all edges connecting the services #}
    {% for edge in edges %}
    {{ edge.from }} -- "{{ edge.label | default(value="") }}" --> {{ edge.to }}
    {% endfor %}
{% endblock diagram %}



2.3 Interaction Analysis: Sequence Diagrams

Purpose: To trace and visualize time-ordered interactions and call flows between different components, services, or actors within a system. This is invaluable for understanding complex protocols or debugging user-facing workflows.
Mermaid Type: sequenceDiagram is the standard and most effective diagram type for this purpose.25
Conceptual Data Schema: The data must capture the participants and the chronological flow of messages.
participants: An array of objects defining the actors in the sequence.
id: String - A unique identifier used in the flow.
label: String - The human-readable name displayed in the diagram.
flow: An ordered array of objects, where each object represents a single step in the interaction.
from: String - The ID of the message sender.
to: String - The ID of the message receiver.
message: String - The content of the message. May contain newlines.
arrow: String - The Mermaid arrow type (e.g., ->>, -->, ->>+ for activation).26
note: Option<String> - An optional note to be displayed next to the receiver.
is_critical: Option<bool> - A flag to highlight a critical interaction.
Template Specification (sequence_diagram.tera):
Code snippet
{% extends "_base.tera" %}

{% block diagram %}
sequenceDiagram
    autonumber
    title {{ diagram_title | default(value="Call Flow Analysis") }}

    {# Define all participants with aliases for clarity #}
    {% for p in participants %}
    participant {{ p.id }} as {{ p.label }}
    {% endfor %}

    {# Render the interaction flow step by step #}
    {% for step in flow %}
    {{ step.from }}{{ step.arrow }}{{ step.to }}: {{ step.message | replace(from="\n", to="<br/>") }}
      {% if step.note %}
    Note right of {{ step.to }}: {{ step.note | replace(from="\n", to="<br/>") }}
      {% endif %}
      {% if step.is_critical %}
    rect rgb(139, 0, 0,.2)
        {{ step.from }}->>{{ step.to }}: Critical Interaction
    end
      {% endif %}
    {% endfor %}
{% endblock diagram %}

This template correctly handles multi-line messages and notes by replacing newline characters with Mermaid's <br/> tag, a crucial detail for readability.26 It also demonstrates how to use the
rect feature to highlight critical parts of the flow.

2.4 Anti-Pattern Detection: Class Diagrams

Purpose: To model Object-Oriented Programming (OOP) structures and visually identify common design anti-patterns. This includes "Bloated Classes" (too many responsibilities), "Lazy Classes" (too few responsibilities), and problematic inheritance or delegation choices.28
Mermaid Type: classDiagram is the designated diagram for representing classes, attributes, methods, and their relationships.25
Conceptual Data Schema: The data must capture class definitions and relationships, augmented with analysis metadata.
classes: An array of objects, each representing a class.
name: String - The name of the class.
annotation: Option<String> - An optional annotation like Interface or Abstract.29
attributes: Array<String> - A list of class attributes, including visibility markers (e.g., +String owner, -int sizeInFeet).
methods: Array<String> - A list of class methods (e.g., +deposit(amount), +run()$).
severity: String - The severity level determined by anti-pattern analysis (e.g., "high" for a God Object).
relations: An array of objects defining relationships.
from: String - The source class name.
to: String - The destination class name.
type: String - The Mermaid relationship type (e.g., <|-- for inheritance, *-- for composition).29
label: Option<String> - An optional label for the relationship.
notes: An array of objects for attaching explanatory notes to classes.
class: String - The name of the class to attach the note to.
text: String - The content of the note (e.g., "Anti-Pattern: Bloated Class. This class has >20 methods.").
Template Specification (class_diagram.tera):
Code snippet
{% extends "_base.tera" %}

{% block diagram %}
classDiagram
    title {{ diagram_title | default(value="OOP Anti-Pattern Analysis") }}
    direction LR

    {# Define all classes, their members, and apply severity styling #}
    {% for class in classes %}
    class {{ class.name }} {
        {% if class.annotation %}<<{{ class.annotation }}>>{% endif %}
        {% for attr in class.attributes %}
        {{ attr }}
        {% endfor %}
        {% for method in class.methods %}
        {{ method }}
        {% endfor %}
    }
    class {{ class.name }} :::sev-{{ class.severity | lower }}
    {% endfor %}

    {# Define all relationships between classes #}
    {% for rel in relations %}
    {{ rel.from }} {{ rel.type }} {{ rel.to }} : {{ rel.label | default(value="") }}
    {% endfor %}

    {# Attach analysis notes to specific classes #}
    {% for note in notes %}
    note for {{ note.class }} "{{ note.text | replace(from="\n", to="\\n") }}"
    {% endfor %}
{% endblock diagram %}

By integrating severity and notes fields directly into the data model, this template can automatically highlight problematic classes and annotate them with the specific reasons, directly visualizing the anti-patterns described in sources like 28.

2.5 Dependency Analysis: Flowcharts for Cycles

Purpose: To map dependencies between software artifacts (modules, files, components) and provide a clear, unambiguous visualization of dependency cycles, which are a common source of architectural decay.
Mermaid Type: flowchart (or graph) is the most suitable diagram for representing directed dependency graphs.30
Layout Workaround for Cycles: A known issue with Mermaid's default layout algorithm is that a simple cyclic definition (e.g., A-->B-->C-->A) often results in a visually confusing, linear layout rather than a clear circle.32 A community-discovered workaround involves carefully ordering node definitions and using bidirectional (
<-->) or undirected (---) links to encourage the layout engine to produce a more aesthetically pleasing circular arrangement.32 While implementing the full workaround requires complex data pre-processing to determine the optimal link ordering, a more direct and highly effective approach is to simply highlight the nodes and edges that form the cycle.
Conceptual Data Schema:
nodes: An array of all dependency nodes.
id: String - The unique ID of the node (e.g., module name).
label: String - The display label for the node.
edges: An array of all dependency edges.
from: String - The ID of the dependent node.
to: String - The ID of the dependency node.
in_cycle: bool - A flag indicating if this edge is part of a detected cycle.
Template Specification (dependency_cycle.tera):
Code snippet
{% extends "_base.tera" %}

{% block diagram %}
graph TD
    title {{ diagram_title | default(value="Dependency Cycle Analysis") }}

    {# Define all nodes first #}
    {% for node in nodes %}
    {{ node.id }}["{{ node.label }}"]
    {% endfor %}

    {# Define all edges and apply special styling for cyclic edges #}
    {% for edge in edges %}
      {% if edge.in_cycle %}
    linkStyle {{ loop.index0 }} stroke:#8B0000,stroke-width:4px
      {% endif %}
    {{ edge.from }} --> {{ edge.to }}
    {% endfor %}

    {# Apply special styling to nodes that are part of a cycle #}
    {% for edge in edges %}
      {% if edge.in_cycle %}
    style {{ edge.from }} fill:#8B0000,stroke:#FFF,color:#FFF
    style {{ edge.to }} fill:#8B0000,stroke:#FFF,color:#FFF
      {% endif %}
    {% endfor %}
{% endblock diagram %}

This template relies on a pre-analysis step to identify which nodes and edges participate in a cycle. It then uses Mermaid's linkStyle and style commands to apply the sev-critical visual treatment to them. The loop.index0 variable from Tera provides the zero-based index of the edge, which is required by the linkStyle command. This approach ensures that even in a complex graph, the problematic cycle is immediately and clearly visible to the developer.

Section 3: Configuration Schema and Runtime Environment

This section specifies the format and schema for the system's configuration file, the multi-layered strategy for its validation, and the implementation details for a hot-reloading developer workflow. These components are essential for creating a system that is both robust and user-friendly.

3.1 Configuration Format Selection: TOML

The selection of a configuration format is critical for the usability and maintainability of the system. After evaluating the primary contenders, this specification mandates the use of TOML (Tom's Obvious, Minimal Language).
Justification:
Human Readability and Writability: TOML is explicitly designed to be a simple, intuitive configuration file format that is easy for humans to read and edit. Its key = "value" syntax and [table] sections map clearly to hierarchical data structures without the syntactic noise of JSON's braces and commas or the whitespace sensitivity of YAML.34 This is paramount for a file that users will be expected to create and modify by hand.
Native Comment Support: TOML supports comments using the hash symbol (#). This is a crucial feature that is entirely absent from the JSON specification.34 Comments allow users to document their configuration, explain complex settings, or temporarily disable diagram definitions, significantly improving the maintainability of the configuration file.
Rust Ecosystem Idiom: TOML is the de facto standard for configuration within the Rust ecosystem. Its adoption by Cargo for the Cargo.toml manifest file has cemented its status as the idiomatic choice for Rust projects.34 Aligning with this convention ensures a familiar and comfortable experience for Rust developers using the tool. While JSON is excellent for data interchange and APIs, TOML is superior for static, human-edited configuration.35

3.2 Master Configuration Schema (diagrams.toml)

The system will be driven by a single master configuration file named diagrams.toml. This file will contain an array of tables, with each table defining a single diagram to be generated. This structure allows for the management of multiple, diverse diagram generation tasks from one central location.
Example diagrams.toml:

Ini, TOML


# diagrams.toml - Master configuration for diagram generation.
# Each [[diagram]] block defines one output diagram.

[[diagram]]
# A unique, human-readable identifier for this diagram generation task.
# Used for logging and potentially for targeting specific diagrams from the CLI.
id = "architecture_overview"

# The type of diagram to generate. This corresponds to a specific template
# and data analysis process.
# Valid options: "component", "sequence", "class", "dependency"
type = "component"

# Path to the Tera template file to use for rendering.
# Path is relative to the project root.
template = "templates/component_diagram.tera"

# Path to the JSON file containing the input data for the template.
# This file is the output of a separate code analysis step.
data_source = "./analysis_output/architecture.json"

# Path where the final Mermaid.mmd file will be written.
output_file = "./diagrams/architecture.mmd"

# Optional: Override default severity styles for this diagram only.
# This allows for per-diagram customization of the visual language.
[diagram.style_overrides]
sev-critical = "fill:#A52A2A,color:#FFF"
sev-high = "fill:#FF6347,color:#000"


[[diagram]]
id = "login_flow_sequence"
type = "sequence"
template = "templates/sequence_diagram.tera"
data_source = "./analysis_output/login_flow.json"
output_file = "./diagrams/login_flow.mmd"


Table 3: TOML Configuration Schema Details
Key
Path
Type
Required
Description
diagram
[[diagram]]
Array of Tables
Yes
Top-level key defining a list of diagram tasks.
id
diagram.id
String
Yes
A unique identifier for the diagram task.
type
diagram.type
String
Yes
Specifies the diagram category. Must be one of "component", "sequence", "class", or "dependency".
template
diagram.template
String (Path)
Yes
Filesystem path to the Tera template file.
data_source
diagram.data_source
String (Path)
Yes
Filesystem path to the JSON data input file.
output_file
diagram.output_file
String (Path)
Yes
Filesystem path for the generated Mermaid markup file.
style_overrides
diagram.style_overrides
Table
No
An optional table to override default severity style definitions.
sev-*
diagram.style_overrides.sev-*
String
No
A key-value pair where the key is a severity class (e.g., sev-critical) and the value is a Mermaid-compatible CSS style string.


3.3 Validation and Error Handling Strategy

To ensure robustness and provide clear user feedback, a multi-layered validation strategy will be implemented for the diagrams.toml configuration file.
Layer 1: Schema Validation (Pre-Parse): Before attempting to parse the TOML file in Rust, its structure will be validated against a formal JSON Schema definition. This can be accomplished using a crate like jsonschema-for-toml 38 or by first converting the TOML to
serde_json::Value and then using the jsonschema crate.39 This initial pass catches high-level structural errors, such as incorrect data types (e.g.,
id being a number instead of a string) or missing required fields, providing immediate and precise feedback.
Layer 2: Deserialization Validation (Parse): The validated TOML content will be deserialized into a strongly-typed Rust struct using serde and the toml crate's from_str function.41 This leverages Rust's powerful type system as a second layer of validation. If the TOML contains values that cannot be coerced into the target struct's field types,
serde will return a descriptive error.
Layer 3: Semantic Value Validation (Post-Parse): After successful deserialization, the resulting Rust struct will undergo further validation to enforce business logic and semantic rules. This will be implemented using the validator crate 42 or
serde_valid.43 These crates allow for declarative validation rules to be added directly to the struct definition using procedural macros. Examples of such rules include:
Ensuring the type field is one of the allowed enumeration values.
Validating that the file paths specified in template, data_source, and output_file exist and are accessible.
Checking that id fields are unique across all [[diagram]] entries.
Error Handling:
Errors from any validation layer must be propagated and presented to the user in a clear, actionable format. The serde_path_to_error crate can be used to generate user-friendly error messages that include the exact path to the invalid field within the JSON/TOML structure (e.g., diagrams.type).44 Similarly, any errors returned by Tera during template rendering (e.g., a syntax error in a template or a reference to a non-existent variable) must be caught and reported with the relevant template name and line number to facilitate debugging.15

3.4 Developer Workflow: Hot-Reloading Implementation

To provide a fluid and efficient development experience, the system will implement a hot-reloading mechanism. This feature will monitor the configuration file and template directory for changes and automatically re-trigger diagram generation, eliminating the need for manual restarts.
Implementation Strategy:
Shared, Mutable State: The core application state, including the Tera instance and the parsed configuration data, must be accessible from both the main application thread and a background file-watching thread. This will be achieved by wrapping these components in an Arc<RwLock<T>>. The Arc (Atomically Referenced Counter) allows for shared ownership across threads, and the RwLock (Read-Write Lock) provides safe, concurrent mutability.45
File System Watching: A dedicated thread will be spawned to monitor the filesystem for changes. The notify crate is the standard and recommended choice for this task in the Rust ecosystem. The watcher will be configured to monitor the diagrams.toml file and the entire templates/ directory recursively.45
Reload Logic: A callback function or channel message will be triggered when the notify watcher detects a change. The logic within this callback will be as follows:
If diagrams.toml is modified: The application will acquire a write lock on the configuration's RwLock, re-read and re-validate the entire file, and replace the old configuration data with the new.
If a file within templates/ is modified: The application will acquire a write lock on the Tera instance's RwLock and call the tera.full_reload() method. This method efficiently re-parses all templates from the configured glob path, updating any changed templates and rebuilding the inheritance chains.12
Trigger Re-generation: After either type of reload, the system will trigger a full re-generation of all diagrams defined in the (potentially new) configuration.
While handlebars-rust offers a built-in dev_mode for this purpose 6, the recommended manual implementation for Tera provides greater flexibility. It allows the system to watch multiple, disparate sources (the configuration file and the template directory) and orchestrate a more complex reload logic that handles changes to both application configuration and presentation templates simultaneously. Crates like
tera-hot-reload can abstract some of this logic, particularly for web server integration, but a manual implementation using notify offers the most control for this specific command-line tool use case.46

Section 4: Conclusions and Recommendations

This technical specification has conducted a thorough analysis of the requirements for a Rust-based Mermaid.js diagram generation system, culminating in a set of clear and actionable architectural decisions. The following conclusions and recommendations form the blueprint for the system's implementation.
Recommended Template Engine: Tera
The system will use the Tera template engine. Its interpreted, runtime-parsing nature is essential for the configuration-driven architecture. Compared to its primary alternative, Handlebars, Tera offers a more powerful and expressive in-template language, including native support for filters, tests, and complex expressions. This is critical for implementing the required dynamic styling based on severity levels cleanly and efficiently. Furthermore, its Jinja2-style inheritance model ({% extends %}, {% block %}) is a more robust and widely understood pattern for template composition, which will enhance the maintainability of the diagram templates.
Recommended Configuration Format: TOML
All system configuration will be managed in a diagrams.toml file. TOML is the mandated format due to its superior human readability, native support for comments, and its status as the idiomatic configuration language within the Rust ecosystem. This choice prioritizes developer experience and long-term maintainability of the configuration files. A robust, multi-layered validation strategy, combining schema validation, type-safe deserialization, and semantic rule enforcement, will ensure configuration integrity.
Standardized Diagram Specifications
The system will support four distinct diagram types, each with a specific purpose and a corresponding Tera template:
Component Diagrams: Use the architecture-beta type for rich architectural visualization.
Sequence Diagrams: Use the sequenceDiagram type for detailed call flow analysis.
Class Diagrams: Use the classDiagram type to model OOP structures and visually flag anti-patterns.
Dependency Flowcharts: Use the flowchart type with specific styling for nodes and edges involved in cycles to clearly highlight circular dependencies.
Unified Styling and Development Workflow
A consistent visual language will be enforced across all diagrams through a centralized set of classDef style definitions within a base Tera template. These styles will be applied dynamically based on a severity field in the input data. For development, a hot-reloading mechanism will be implemented using the notify crate to monitor template and configuration files, providing an efficient and seamless workflow for developers creating and iterating on diagram definitions.
By adhering to these specifications, the development team can construct a powerful, flexible, and maintainable tool for code analysis visualization that is well-integrated with the conventions and strengths of the Rust programming language.
Works cited
Sailfish: Rust's fastest template engine, 200x faster than handlebars - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/hnhbmv/sailfish_rusts_fastest_template_engine_200x/
Best templating engine for Rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1fc2mic/best_templating_engine_for_rust/
Template engine — list of Rust libraries/crates // Lib.rs, accessed July 7, 2025, https://lib.rs/template-engine
Top 3 templating libraries for Rust - LogRocket Blog, accessed July 7, 2025, https://blog.logrocket.com/top-3-templating-libraries-for-rust/
Any benchmarks for Node vs. Rust regarding templating engines? - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/j9qpo5/any_benchmarks_for_node_vs_rust_regarding/
handlebars - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/thousand_birds_handlebars
handlebars - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/handlebars
rocket_dyn_templates::handlebars - Rust, accessed July 7, 2025, https://api.rocket.rs/v0.5/rocket_dyn_templates/handlebars/
handlebars-rust - Fuchsia, accessed July 7, 2025, https://fuchsia.googlesource.com/fuchsia/+/2e68aeeadbe1/third_party/rust_crates/vendor/handlebars/README.md
handlebars-rust/src/lib.rs at master - GitHub, accessed July 7, 2025, https://github.com/sunng87/handlebars-rust/blob/master/src/lib.rs
tera - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/tera
Tera - GitHub Pages, accessed July 7, 2025, https://keats.github.io/tera/docs/
The Essence of Templating with Tera - shuttle.dev, accessed July 7, 2025, https://www.shuttle.dev/blog/2024/11/29/the-essence-of-templating-with-tera
tera - Keywords - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/keywords/tera
Introducing Tera, a template engine in Rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/4ewsg6/introducing_tera_a_template_engine_in_rust/
askama-rs/template-benchmark: Comparison of template engines written in and for Rust., accessed July 7, 2025, https://github.com/rinja-rs/template-benchmark
Rust is SLOW actually? - AOC 2023 day 1 performance comparison - DEV Community, accessed July 7, 2025, https://dev.to/jwrunge/rust-is-slow-actually-aoc-2023-day-1-performance-comparison-28d
handlebars - Rust - Random Hacks Library Documentation, accessed July 7, 2025, http://docs.randomhacks.net/subtitles-rs/handlebars/index.html
handlebars - Rust, accessed July 7, 2025, https://fengsp.github.io/pencil/handlebars/index.html
Tera in tera - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/tera/latest/tera/struct.Tera.html
Styling State Diagrams · Issue #3742 · mermaid-js/mermaid - GitHub, accessed July 7, 2025, https://github.com/mermaid-js/mermaid/issues/3742
How to dynamically style a mermaid graph? - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/78515192/how-to-dynamically-style-a-mermaid-graph
Flowcharts – Basic Syntax - Mermaid Chart - Create complex, visual ..., accessed July 7, 2025, https://mermaid.js.org/syntax/flowchart.html#styling-and-classes
Architecture Diagrams Documentation (v11.1.0+) | Mermaid, accessed July 7, 2025, https://mermaid.js.org/syntax/architecture.html
5 Mermaid.js examples to get you started - Swimm, accessed July 7, 2025, https://swimm.io/learn/mermaid-js/5-mermaid-js-examples-to-get-you-started
Sequence Diagram - Mermaid Chart - Create complex, visual ..., accessed July 7, 2025, https://docs.mermaidchart.com/mermaid-oss/syntax/sequenceDiagram.html
Mermaid Cheat Sheet @ https://jojozhuang.github.io, accessed July 7, 2025, https://jojozhuang.github.io/tutorial/mermaid-cheat-sheet/
Class Diagram Anti-Patterns, accessed July 7, 2025, https://www.cs.sjsu.edu/~pearce/modules/lectures/uml/class/antipatterns/index.htm
Class diagrams | Mermaid, accessed July 7, 2025, https://mermaid.js.org/syntax/classDiagram.html
Flowcharts – Basic Syntax - Mermaid Chart, accessed July 7, 2025, https://docs.mermaidchart.com/mermaid-oss/syntax/flowchart.html
Mermaid.js: A Complete Guide - Swimm, accessed July 7, 2025, https://swimm.io/learn/mermaid-js/mermaid-js-a-complete-guide
Support for Circular Flow Diagram · Issue #3228 · mermaid-js/mermaid - GitHub, accessed July 7, 2025, https://github.com/mermaid-js/mermaid/issues/3228
Circular Flowcharts: How do I make them? - Basement - Obsidian Forum, accessed July 7, 2025, https://forum.obsidian.md/t/circular-flowcharts-how-do-i-make-them/19572
JSON vs YAML vs TOML vs XML: Best Data Format in 2025 - DEV ..., accessed July 7, 2025, https://dev.to/leapcell/json-vs-yaml-vs-toml-vs-xml-best-data-format-in-2025-5444
Understanding YAML, JSON, and TOML: When to Use What | by Suraj Singh Bisht, accessed July 7, 2025, https://surajsinghbisht054.medium.com/understanding-yaml-json-and-toml-when-to-use-what-e6089a1d16f1?source=rss------web_development-5
JSON vs YAML vs TOML vs XML: Best Data Format in 2025 | by Leapcell - Medium, accessed July 7, 2025, https://leapcell.medium.com/json-vs-yaml-vs-toml-vs-xml-best-data-format-in-2025-fa35e06841ba
The Manifest Format - The Cargo Book - Rust Documentation, accessed July 7, 2025, https://doc.rust-lang.org/cargo/reference/manifest.html
jsonschema-for-toml - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/jsonschema-for-toml
jsonschema - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/jsonschema
Stranger6667/jsonschema: A high-performance JSON Schema validator for Rust - GitHub, accessed July 7, 2025, https://github.com/Stranger6667/jsonschema
toml - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/toml
Keats/validator: Simple validation for Rust structs - GitHub, accessed July 7, 2025, https://github.com/Keats/validator
serde_valid - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/serde_valid/latest/serde_valid/
Easy guide to JSON input validation in Rust web services - LogRocket Blog, accessed July 7, 2025, https://blog.logrocket.com/json-input-validation-in-rust-web-services/
Is there a way to have immediate reload for HTML pages during development? : r/rust, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1f46za3/is_there_a_way_to_have_immediate_reload_for_html/
tera_hot_reload - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/tera-hot-reload/latest/tera_hot_reload/
sunng87/handlebars-rust: Rust templating with Handlebars - GitHub, accessed July 7, 2025, https://github.com/sunng87/handlebars-rust
tera-hot-reload - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/tera-hot-reload
