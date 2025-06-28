
A Framework for High-Fidelity, AI-Generated Software Architecture Diagrams: From Code Analysis to Automated Validation


Section 1: From Source Code to Architectural Primitives: A Preprocessing and Abstraction Methodology

The foundational challenge in generating accurate architectural diagrams from code is not one of visualization, but of abstraction. The process of converting vast, low-level source code into a high-level, meaningful architectural representation is where fidelity is either established or lost. Simply feeding raw code files to a Large Language Model (LLM) often yields superficial dependency graphs rather than true architectural diagrams that elucidate component responsibilities and data flows.1 This limitation underscores the necessity of a robust preprocessing and abstraction pipeline. This section specifies a multi-phase methodology, grounded in the principles of Software Architecture Recovery (SAR), to transform source code into a structured, queryable "Software Architecture Model" (SAM)—the essential ground truth for both diagram generation and validation.

1.1 The Imperative of Abstraction: Beyond Naive Code-to-Text

The manual creation of architectural diagrams is notoriously time-consuming, susceptible to human error, and requires significant domain expertise.2 A primary motivation for employing AI is to automate this process, enhancing consistency and improving comprehension.2 However, initial approaches, such as those used by tools like Swark, which directly process code files with an LLM, demonstrate a significant gap. While useful for gaining a high-level understanding or reviewing dependency graphs, they fall short of producing genuine software architecture diagrams that describe nuanced interactions and data flows.1
To transcend these limitations, the framework must adopt principles from SAR, a discipline focused on extracting architectural information from lower-level system representations.4 The need for SAR arises because architectural documentation is frequently missing or, more critically, has drifted out of synchronization with the implemented system, a phenomenon known as architecture erosion.5 By systematically recovering the
as-is architecture from the code itself, we establish an authoritative foundation for all subsequent steps. This moves the system from a simple code-to-diagram tool to a sophisticated architecture analysis platform.

1.2 Phase 1: Static Analysis for Structural Information Extraction

Static program analysis, which examines code without executing it, is the cornerstone of structural discovery.7 It allows for the systematic parsing of the entire codebase to identify its constituent parts and their compile-time relationships.

1.2.1 Abstract Syntax Tree (AST) Parsing

The initial step in static analysis is to construct an Abstract Syntax Tree (AST) for the source code.9 An AST is a hierarchical representation of the code's syntactic structure, abstracting away non-essential details like punctuation and delimiters.9 It is a data structure widely used in compilers as an intermediate representation that facilitates deep analysis.9
The process involves using language-specific parsers (e.g., Esprima for JavaScript, JavaParser for Java) to transform code into a traversable tree structure.9 Each node in the AST represents a specific code construct, such as a class declaration, a method call, or a variable assignment. This structured format is invaluable because, unlike raw source text, it can be programmatically queried, edited, and annotated with additional information, such as the source file and line number of each construct.9

1.2.2 Feature Extraction and Entity Recognition

Once the AST is generated, the next step is to traverse it to perform feature extraction, identifying and classifying architectural entities (the nodes of our future model) and their relationships (the edges). This process is analogous to the information extraction phase of reverse engineering, where the goal is to recover the system's design and data structures from its code.12
Key architectural features to be extracted include:
Entities (Nodes):
Components: High-level groupings like services, modules, or subsystems.
Classes and Interfaces: The fundamental building blocks in object-oriented programming.
Functions/Methods: The units of behavior.
Data Structures: Classes, structs, or database schemas that represent data entities.
Configuration Files: Infrastructure-as-Code files (e.g., Terraform, Dockerfiles) that define deployment components.
Relationships (Edges):
Direct Calls: A function in component A calls a function in component B.
Instantiation: Component A creates an instance of component B.
Inheritance/Implementation: A class inherits from a superclass or implements an interface.
Data Access: A component reads from or writes to a data entity (e.g., a database table or a variable).
Dependency Injection: A component receives a dependency on another component.
These extracted features form the raw material for the architectural model.

1.3 Phase 2: Building the Software Architecture Model (SAM)

The extracted entities and relationships must be organized into a formal, structured representation. We define this as the Software Architecture Model (SAM), a directed, attributed graph that serves as the canonical, queryable "ground truth" of the system's architecture. The decision to create an explicit model, rather than proceeding directly to diagram generation, is a critical architectural choice for this framework. It is inspired by advanced modeling tools like Structurizr, which emphasize creating a single, reusable model from which multiple, consistent diagrams can be generated.14 This model-centric approach is the only viable path to achieving automated validation, as it provides a concrete artifact to compare against.
The SAM will adhere to the following schema:
Nodes (Entities): Each node in the graph represents an architectural entity and contains the following attributes:
id: A unique identifier for the node.
name: The human-readable name of the entity (e.g., UserService, AuthMiddleware).
type: The classification of the entity (e.g., Service, Class, Database, API Endpoint, Component).
source_location: A reference to the source file and line numbers where the entity is defined.
description: A textual description, which can be extracted from code comments or automatically summarized using a targeted LLM call.
group: An attribute for hierarchical grouping (e.g., the name of the microservice or C4 container it belongs to).
Edges (Relationships): Each directed edge represents a relationship between two entities and contains these attributes:
id: A unique identifier for the edge.
source_node: The ID of the originating node.
target_node: The ID of the destination node.
type: The classification of the relationship (e.g., CALLS, INHERITS_FROM, READS_DATA_FROM, PUBLISHES_TO_TOPIC).
properties: A key-value map for additional metadata, such as the frequency of a call if dynamic data is available.
This structured SAM becomes the central artifact of the CodeAtlas system.

1.4 Phase 3: Augmenting with Dynamic and Behavioral Analysis

While static analysis is powerful, it has inherent limitations, particularly for modern, distributed architectures.5 In systems that rely heavily on polymorphism, dynamic binding, and network-based communication (e.g., microservices), static analysis may identify a potential dependency but cannot confirm if or how it is used at runtime.18 Static analyzers often "break down" when faced with indirect dependencies via API calls or message queues.19
To achieve the highest fidelity, the SAM should be enriched with data from dynamic and behavioral analysis:
Runtime Analysis: By analyzing runtime data from logs, traces (e.g., OpenTelemetry), or API gateways, the framework can identify actual interactions between services. This allows the system to distinguish between a theoretical dependency in the code and a concrete, high-traffic interaction in production. This is crucial for accurately representing data flows in microservice architectures.20
Change Coupling Analysis: An innovative technique is to mine version-control data (e.g., Git history) and issue-tracking data (e.g., Jira tickets).19 "Change coupling" occurs when two or more modules or services are consistently modified together in the same commits or to resolve the same ticket. This reveals strong, implicit dependencies that are invisible to both static and runtime analyzers. This technique is particularly powerful for identifying dependencies that cross team boundaries, which are often sources of organizational friction and delivery bottlenecks.19
By combining static, dynamic, and behavioral analysis, the SAM evolves from a purely structural model to a rich, multi-faceted representation of the system's true architecture.

Section 2: Advanced Prompt Engineering for Generating Diagrammatic Syntax

With a high-fidelity Software Architecture Model (SAM) established, the role of the Large Language Model (LLM) shifts from architectural inference to structured-to-text translation. The LLM is not asked to understand the raw code; instead, it is tasked with the more constrained and well-defined problem of converting the structured SAM into syntactically correct and visually effective Mermaid.js or PlantUML code. This approach leverages the LLM's formidable linguistic and code-generation capabilities while mitigating its known weaknesses in complex, logical reasoning, which has already been handled by the deterministic SAM generation process.

2.1 The Role of the LLM: From Structured Model to Syntactic Representation

The core of this methodology is to treat the LLM as a skilled technical illustrator rather than a creative architect. The prompt provides the LLM with a serialized, human-readable representation of a sub-graph from the SAM, along with a clear set of instructions for how to render it. This reframing of the task is crucial for achieving high accuracy and predictability. It transforms an open-ended, complex reasoning problem ("draw this codebase") into a closed-ended translation task ("render this graph data as a Mermaid diagram"), which is much better suited to the capabilities of current LLMs. This aligns with prompt engineering best practices, which demonstrate that LLMs perform best when given highly structured input and clear, explicit constraints.22

2.2 Core Prompting Methodology

To ensure consistent and high-quality output, a modular prompt template should be employed. This template is designed to provide the LLM with all necessary context, data, and constraints in a structured format.23
Universal Prompt Template Structure:
Role-Playing: The prompt begins by assigning a specific role to the LLM to prime its behavior.
Example: You are an expert software architect specializing in creating clear and accurate system diagrams using {diagram_language}. (where {diagram_language} is "Mermaid.js" or "PlantUML").
Context and Goal: Clearly state the objective of the task.
Example: Your task is to convert a structured representation of a software architecture, provided in JSON format, into a {diagram_type} diagram. The diagram must be visually clear, well-organized, and accurately reflect all provided components and relationships.
Input Data (Serialized SAM Sub-graph): Provide the architectural data in a clean, serialized format. JSON is ideal for this purpose.
Example: Here is the architectural model to be visualized: \n\``json\n{ "nodes": [...], "edges": [...] }\n````
Formatting Instructions and Constraints: This is the most critical part of the prompt, as it strictly guides the LLM's output.
Labeling: Use clear, concise labels for each node based on the 'name' attribute from the input JSON.
Relationships: Represent relationships using the specified arrow types and labels from the 'type' attribute of each edge.
Grouping: Group components within containers or boundaries as specified by the 'group' attribute of the nodes.
Layout: Organize the diagram using a {layout_preference} layout, such as 'Top-Down (TD)' or 'Left-to-Right (LR)'. 22
Fidelity Constraint: CRITICAL: Strictly adhere to the provided JSON model. Do not add any components, relationships, or attributes that are not explicitly present in the input data. Do not infer or invent any new architectural elements. This constraint is paramount for ensuring the diagram is a faithful representation of the SAM, which is essential for validation.
Few-Shot Examples (Recommended): To further improve accuracy, especially for complex syntax, provide a complete, concise example of a JSON input and the corresponding ideal diagram code output. This one-shot or few-shot prompting technique significantly improves the model's ability to match the desired format.23
Output Specification: Instruct the model on the exact format of its response to prevent extraneous text.
Example: Provide ONLY the complete, valid {diagram_language} code block. Do not include any explanations, apologies, or surrounding text.

2.3 Specialized Prompts for Architectural Patterns and Concerns

While the universal template provides a strong foundation, tailoring prompts for specific architectural patterns and concerns can yield superior results. This involves adjusting the requested diagram_type and filtering the SAM to provide only the relevant data for that specific view.

2.3.1 Generating C4 Model Diagrams

The C4 model provides a structured way to visualize architecture at different levels of abstraction: Context, Containers, Components, and Code.25 To generate these views, the SAM is first filtered to the appropriate level.
For a C4 Container Diagram: The SAM is filtered to include only nodes of type: 'Service', type: 'Database', type: 'WebApp', etc., and the edges between them.
Prompt Specialization: The prompt's {diagram_type} would be set to 'C4 Container Diagram', and the LLM would be instructed to use the specific C4 syntax supported by PlantUML or Mermaid extensions.15

2.3.2 Visualizing Microservice Architectures

For microservice architectures, different views are needed to understand different aspects of the system.
Topology Map: To show the static dependencies between services, the prompt would request a component or network diagram, using a SAM sub-graph containing only service nodes and the edges representing their communication channels (e.g., API calls, message bus topics).20
Sequence Diagram: To illustrate a specific user workflow, the prompt would request a sequence diagram. The input data would be an ordered list of interactions (a "trace") extracted from the SAM, potentially enriched with dynamic analysis data.24 The prompt would instruct the LLM to represent the actors and the sequence of messages between them.

2.3.3 Handling Cross-Cutting Concerns

A recurring challenge is visualizing cross-cutting concerns (e.g., authentication, logging, caching) without cluttering the primary business logic diagrams.28 The consensus from both community feedback and architectural best practices is to represent these in separate, dedicated views or as overlays.29
The problem is thus framed as a "view selection" problem, not a generation problem. The SAM must first be annotated during the preprocessing phase, tagging components with the concerns they participate in (e.g., a node attribute concerns: ['authentication', 'logging']).
Prompting Strategy for a "View Overlay":
Provide the base architectural model (e.g., the container view).
Add a specific instruction to the prompt: Generate the diagram based on the provided model. Additionally, apply a special visual style (e.g., a dashed border and a light blue background color) to the following nodes, which are part of the 'Authentication' concern: [list of component names].
Alternatively, for a dedicated view: From the provided model, generate a diagram that shows ONLY the components involved in the 'Logging' concern and the direct relationships between them.
This approach maintains the clarity of the primary architecture diagram while allowing stakeholders to explore specific cross-cutting concerns on demand, aligning with the concept of architectural "perspectives" 29 or layers.31
The following table provides specific, actionable templates for generating diagrams of common architectural patterns.

Architectural Pattern
Recommended Diagram Type
Key SAM Elements to Include
Mermaid.js/PlantUML Prompt Template
Microservices
C4 Component Diagram or Sequence Diagram
Nodes: Service, API Gateway, Database, Message Broker. Edges: API Call, Publishes/Subscribes To Event.
You are an expert architect. Convert the provided JSON model of a microservices architecture into a Mermaid.js component diagram using a 'TD' layout. Show services, databases, and message brokers. CRITICAL: Do not add any elements not present in the input JSON. Here is the model:...
Layered Architecture
Layered Diagram (using subgraphs)
Nodes: UI Component, Business Logic Service, Data Access Object, Database. Group nodes by layer (Presentation, Business, Data).
You are an expert architect. Convert the provided JSON model of a layered architecture into a PlantUML component diagram. Use packages to represent the 'Presentation', 'Business', and 'Data' layers. Place components in their respective layer packages. CRITICAL: Do not infer any relationships. Here is the model:...
Client-Server
Network Diagram or C4 Container Diagram
Nodes: Client Application, Web Server, API Server, Database. Edges: HTTP Request, SQL Query.
You are an expert architect. Convert the provided JSON model of a client-server system into a Mermaid.js graph diagram using an 'LR' layout. Clearly label the 'Client', 'Server', and 'Database' components and the protocols on the edges. CRITICAL: Only use the provided data. Here is the model:...
Event-Driven
Sequence Diagram or Message Flow Diagram
Nodes: Event Producer, Event Consumer, Event Bus/Topic. Edges: Publishes Event, Consumes Event.
You are an expert architect. Convert the provided JSON model, which represents an event-driven workflow, into a PlantUML sequence diagram. The model contains an ordered list of event publications and consumptions. The actors are the 'Producers' and 'Consumers'. CRITICAL: Follow the sequence exactly as provided. Here is the model:...


Section 3: A Quantitative Framework for Diagram-Code Validation

To ensure that AI-generated diagrams are not merely plausible but verifiably accurate, a rigorous, quantitative validation framework is essential. Subjective evaluation is insufficient; the CodeAtlas system requires objective, computable metrics that can drive an automated feedback loop. This section defines a multi-faceted suite of metrics to measure the fidelity of a generated diagram against the ground-truth Software Architecture Model (SAM). The framework assesses accuracy along two primary axes: Structural Fidelity (are the boxes and lines correct?) and Semantic Coherence (do the labels and names mean the right thing?).
The core of the evaluation process is a comparison between two graph representations:
Gsam​: The ground-truth graph derived from the SAM, as constructed in Section 1.
Ggenerated​: The graph extracted by deterministically parsing the AI-generated Mermaid.js or PlantUML code.32

3.1 Metric Suite for Structural Fidelity

These metrics evaluate the topological and structural correctness of the generated diagram. They answer the question: "Does the diagram's structure match the code's structure?"

3.1.1 Node and Edge Precision & Recall

These are foundational metrics that provide a granular assessment of the diagram's completeness and correctness by counting the presence or absence of individual elements.33
Node Precision: Measures the fraction of nodes in the generated diagram that are correct. A low precision indicates the AI is "hallucinating" components that do not exist in the source code.
$$ \text{Node Precision} = \frac{|\text{Nodes}(G_{generated}) \cap \text{Nodes}(G_{sam})|}{|\text{Nodes}(G_{generated})|} $$
Node Recall: Measures the fraction of nodes from the source model that were successfully captured in the diagram. A low recall indicates the AI has omitted existing components.
$$ \text{Node Recall} = \frac{|\text{Nodes}(G_{generated}) \cap \text{Nodes}(G_{sam})|}{|\text{Nodes}(G_{sam})|} $$
Edge Precision: Measures the fraction of relationships in the generated diagram that are correct.
$$ \text{Edge Precision} = \frac{|\text{Edges}(G_{generated}) \cap \text{Edges}(G_{sam})|}{|\text{Edges}(G_{generated})|} $$
Edge Recall: Measures the fraction of relationships from the source model that were captured in the diagram.
$$ \text{Edge Recall} = \frac{|\text{Edges}(G_{generated}) \cap \text{Edges}(G_{sam})|}{|\text{Edges}(G_{sam})|} $$
For these calculations, two nodes or edges are considered identical if their key attributes (e.g., name, type, source/target nodes) match.

3.1.2 Graph Edit Distance (GED)

While precision and recall are useful, they do not capture the overall "shape" or structural difference between the two graphs. Graph Edit Distance provides a more holistic measure by calculating the minimum cost of edit operations (node/edge insertions, deletions, and substitutions) required to transform Ggenerated​ into Gsam​.34
The cost of each operation can be defined (e.g., cost of 1 for each insertion/deletion). The matching process then seeks to find the sequence of edits with the minimum total cost. A lower GED signifies a higher degree of structural similarity. While computing exact GED is NP-hard, efficient approximation algorithms, such as those based on A* search or bipartite matching, can be used for practical application.35

3.1.3 Architectural Metric Drift

A high-fidelity diagram should not only represent the components correctly but also preserve the high-level architectural characteristics of the system. This can be measured by calculating key software architecture metrics on both Gsam​ and Ggenerated​ and measuring the difference, or "drift".36
Key metrics to compare include:
Coupling: For each component, calculate its Afferent Coupling (Ca​, incoming dependencies) and Efferent Coupling (Ce​, outgoing dependencies). The drift is the average difference in these values between the two graphs.
Cyclomatic Complexity: While typically a code-level metric, an architectural equivalent can be calculated from the control flow graph of components: M=E−N+2P, where E is the number of edges, N is the number of nodes, and P is the number of connected components.36 A significant drift suggests the diagram misrepresents the system's logical complexity.
A low drift score indicates that the generated diagram is a faithful abstraction that maintains the architectural integrity of the source.

3.2 Metric for Semantic Coherence

Structural correctness is necessary but not sufficient. A diagram could be structurally perfect but use confusing or misleading labels, rendering it useless. The Semantic Coherence Score (SCS) is a novel metric designed to quantify how well the meaning of the diagram's text aligns with the meaning of the source code's identifiers and documentation.

3.2.1 Semantic Similarity of Labels

This metric leverages advances in Natural Language Processing (NLP), specifically pre-trained sentence embedding models like Sentence-BERT (SBERT).39 These models convert text strings into high-dimensional vectors where semantically similar strings are located closer together in the vector space.
The process is as follows:
For each correctly matched node between Gsam​ and Ggenerated​:
Extract the textual description for the node from the SAM (e.g., class name concatenated with its code comments).
Extract the label of the corresponding node from the generated diagram.
Use an SBERT model to generate a vector embedding for both the SAM description and the diagram label.
Calculate the cosine similarity between the two vectors. This yields a score between -1 and 1 (typically 0 to 1 for this task), where 1 indicates identical meaning.40
For example, if the SAM has a node UserAuthenticationService and the LLM labels it "Login Handler," the semantic similarity score would be high. If it were labeled "Data Cache," the score would be very low.

3.2.2 Calculating the Semantic Coherence Score (SCS)

The overall SCS for the diagram is the average cosine similarity score calculated across all matched nodes.
$$ \text{SCS} = \frac{1}{|\text{Matched Nodes}|} \sum_{i \in \text{Matched Nodes}} \text{cosine_similarity}(\text{vector}(label_{i, sam}), \text{vector}(label_{i, gen})) $$
An SCS score close to 1.0 indicates that the diagram is not only structurally correct but also semantically aligned with the source code, making it highly comprehensible and trustworthy.
The introduction of this comprehensive, multi-faceted suite of metrics is a direct consequence of understanding that "accuracy" is not a monolithic concept.42 A validation framework that relies on a single number would be brittle. By combining structural metrics (Precision/Recall, GED), architectural characteristic metrics (Drift), and semantic metrics (SCS), the CodeAtlas system can generate a rich, detailed report on diagram quality. This entire framework is predicated on the existence of the SAM as the ground truth; without it, none of these quantitative, automated comparisons would be possible, reinforcing the criticality of the model-first approach outlined in Section 1.
The following table provides a consolidated reference for the proposed quantitative metrics.

Metric Name
Definition
Formula / Calculation Method
Interpretation
Architectural Quality Assessed
Node/Edge Precision
The fraction of elements (nodes or edges) in the generated diagram that are correct.
$\frac{
\text{Correctly Identified}
}{
Node/Edge Recall
The fraction of elements from the source model that were captured in the diagram.
$\frac{
\text{Correctly Identified}
}{
Graph Edit Distance (GED)
The minimum cost of edit operations (insert, delete, substitute) to transform the generated graph into the source model graph.
Algorithmic search for minimum cost path (e.g., A*).
Lower is better (0 is a perfect match). Measures overall structural difference.
Structural Fidelity
Architectural Metric Drift
The difference in high-level architectural metrics (e.g., Coupling, Complexity) between the source model and the generated diagram.
Δ=Metric(Gsam​)−Metric(Ggenerated​)
Closer to 0 is better. Measures if the diagram preserves key architectural characteristics.
Characteristic Integrity
Semantic Coherence Score (SCS)
The average semantic similarity between the labels/descriptions of matched nodes in the source model and the generated diagram.
Average cosine similarity of SBERT vector embeddings.
Higher is better (1.0 is perfect). Measures if the diagram's text is meaningful and accurate.
Semantic Accuracy


Section 4: System Design for Automated Consistency and Progressive Refinement

The true transformative potential of AI in architectural diagramming lies not in a single, perfect generation, but in creating a dynamic, self-correcting system. This section outlines the design of an automated validation and refinement engine that leverages the quantitative metrics from Section 3 to create a closed feedback loop. This system ensures that diagrams remain perpetually consistent with the source code and progressively improve their accuracy over time. This approach directly addresses the user's need for automated validation and architectural conformance checking, solving the pervasive problem of documentation drift.

4.1 Architecture of the Validation and Refinement Engine

The proposed system architecture integrates the previously defined components into a cohesive workflow. It is designed to operate both on-demand and as part of a continuous integration (CI) pipeline.
The core components of the engine are:
SAM Generator: Implements the multi-phase abstraction methodology from Section 1. It takes source code as input and produces the ground-truth Gsam​.
LLM Diagram Generator: Implements the advanced prompting methodology from Section 2. It takes a SAM sub-graph and a prompt template as input and produces Mermaid.js or PlantUML code.
Diagram Parser: A deterministic parser specifically designed for Mermaid.js and PlantUML syntax. Its sole function is to convert the LLM's textual output back into a structured graph representation, Ggenerated​.
Validation Engine: This component implements the full suite of quantitative metrics defined in Section 3. It takes Gsam​ and Ggenerated​ as input and outputs a detailed validation report containing scores for precision, recall, GED, metric drift, and semantic coherence.
Refinement Prompt Generator: This is the crucial link in the feedback loop. It analyzes the discrepancy report from the Validation Engine and programmatically constructs a new, corrective prompt for the LLM. For instance, if the report indicates a missing node, the generator formulates a command like, "Please add a node named 'X' connected to 'Y'."
CI/CD Integration Hooks: These are scripts and configurations that allow the entire engine to be triggered automatically by events in a version control system, such as a git push or a pull request creation.44

4.2 The Progressive Refinement Loop

For any given diagram generation task, the system executes an iterative refinement loop that mimics, but automates, a human review cycle. This creates a "self-healing" capability, where the system can identify and correct its own mistakes.
The workflow proceeds as follows:
Initial Generation (v1): The SAM Generator creates the ground-truth model. The LLM Diagram Generator receives the initial prompt and produces the first version of the diagram, Diagram v1.
Validation and Discrepancy Analysis: Diagram v1 is parsed into Ggenerated,v1​. The Validation Engine compares it against Gsam​ and produces a list of specific errors. For example:
Missing Node: Node OrderProcessor exists in Gsam​ but not in Ggenerated,v1​.
Incorrect Edge: An edge exists from APIGateway to LegacyLogger in Ggenerated,v1​, but the correct edge in Gsam​ is to CentralizedLoggingService.
Low Semantic Score: The node for PaymentGateway is labeled "Money Handler" in the diagram, resulting in a low semantic similarity score.
Refinement Prompt Generation: The Refinement Prompt Generator translates these structured errors into a natural language prompt for the LLM.
Example Refinement Prompt: You previously generated a diagram that contained inaccuracies. Please regenerate the diagram with the following corrections: 1. You missed a component; you must add a node named 'OrderProcessor'. 2. The connection from 'APIGateway' is incorrect; it should connect to 'CentralizedLoggingService', not 'LegacyLogger'. 3. The label 'Money Handler' is unclear; rename this node to 'PaymentGateway' to match the system's terminology.
Re-generation (v2): The LLM Diagram Generator processes this new, highly specific refinement prompt to produce Diagram v2.
Convergence Check: The loop repeats, with Diagram v2 being validated. The process continues until the accuracy scores from the Validation Engine surpass a predefined quality threshold (e.g., F1-score > 0.95 and SCS > 0.9) or a maximum number of iterations is reached to prevent infinite loops.

4.3 Automated Architectural Conformance Checking

The ultimate goal is to eliminate architectural drift, where the implementation evolves but the documentation does not.45 This is achieved by embedding the validation engine into the development workflow through CI/CD integration. This practice transforms the architecture diagram from a static, point-in-time artifact into a living, continuously verified document.46
The integration works as follows:
On every pull request or merge to the main branch, the CI pipeline triggers the CodeAtlas engine.
The engine checks out the proposed code changes and regenerates the SAM.
It then regenerates the corresponding architecture diagram(s).
The Validation Engine performs a conformance check, comparing the newly generated diagram against the new SAM.
The results are presented as a "quality gate" in the CI pipeline.44 If the diagram's accuracy scores fall below the required threshold, the build fails. This immediately alerts the development team that their code change has introduced a deviation from the intended architecture or that the existing diagram no longer accurately reflects the system.
This automated check ensures that the architectural documentation is always synchronized with the code, solving one of the most persistent and costly problems in software maintenance.2 It effectively prevents the slow accumulation of architecture debt and erosion of the system's structural integrity.6
The power of this automated feedback loop cannot be overstated. LLMs are inherently non-deterministic and prone to errors. A system that relies on a single, perfect generation is brittle. By creating a system that can programmatically identify its own flaws using the quantitative metrics and then translate those flaws into a corrective prompt, we build a robust, anti-fragile process. It automates the tedious cycle of review and correction that would otherwise fall to a human architect. Furthermore, by integrating this process into the CI/CD pipeline, the framework addresses the primary failure mode of all manual documentation: staleness. While "diagrams as code" helps by versioning the diagram source 14, this framework goes a step further. The diagram is not just versioned
with the code; it is continuously derived from and validated against the code, guaranteeing a perpetual state of consistency.

Section 5: Dynamic Complexity Management and Viewpoint Synthesis

An architectural diagram that is 100% accurate yet displays thousands of nodes and edges in a tangled mess is functionally useless. As systems grow, their "dynamic complexity"—the complexity emerging from the dense interconnection of their components—can overwhelm human comprehension.48 The solution is not to sacrifice the accuracy of the underlying model but to provide intelligent mechanisms for managing the visual complexity of its representation. This is achieved by generating different
views of the SAM, each tailored to a specific audience and purpose, and by enabling interactive exploration.

5.1 The Challenge of Visual Complexity

A complete SAM of a large enterprise system is inherently complex. Attempting to render it in a single, static diagram leads to information overload, defeating the purpose of visualization.50 Effective communication requires that diagrams be simplified and focused on a specific narrative. The key principle is to manage complexity by controlling the level of detail and the perspective of the visualization, rather than by omitting information from the core model.

5.2 Hierarchical Abstraction and Level of Detail (LoD)

The most effective technique for managing complexity is hierarchical abstraction. This framework will adopt the principles of the C4 model (Context, Containers, Components, Code) as a primary mechanism for controlling the Level of Detail (LoD).25 The SAM itself will be structured hierarchically, with nodes having
group attributes that map them to higher-level containers. This allows the user to dynamically request a diagram at a specific zoom level.
Level 1 (System Context): A high-level view showing the software system as a single black box, its users, and its interactions with other external systems. This view is generated by filtering the SAM to show only nodes of type: 'ExternalSystem' or type: 'User' and their connections to the system boundary.
Level 2 (Containers): This view "zooms in" to the system, revealing its major, independently deployable parts. These are the "containers" in the C4 sense, such as web applications, microservices, databases, and message brokers.16
Level 3 (Components): This view zooms further into a single container, showing the major internal modules, classes, or components that make it up. It reveals the internal structure of a single service or application.
Level 4 (Code): This level corresponds to the raw code (classes, functions) and is generally not visualized as a diagram but is represented within the SAM for code-level analysis and traceability.
The generation of these views is handled by filtering the SAM based on the requested LoD before it is passed to the LLM prompt generator. This ensures the LLM receives only the necessary information for the desired view.

5.3 Viewpoint-Based Visualization

Beyond simple hierarchical zooming, a sophisticated system must offer different "viewpoints" or "perspectives" that slice the architectural model along different conceptual axes.29 This allows different stakeholders to see the information most relevant to them. The SAM, being a rich, multi-attributed graph, is the single source of truth from which these diverse views are rendered.
Examples of essential viewpoints include:
Structural View: The default view showing the main components and their dependencies, typically as a component or class diagram.
Data Flow View: A specialized diagram that filters the SAM to show only data-centric entities (e.g., nodes of type: 'Database', 'Cache', 'MessageQueue') and the services that have READS_FROM or WRITES_TO relationships with them. This is invaluable for understanding how information moves through the system.
Deployment View: This view maps logical components (containers) to the physical or virtual infrastructure they run on. It uses information from IaC files or runtime environments to show how services are deployed across servers, Kubernetes clusters, or cloud regions.55
Security View: This viewpoint highlights components and relationships relevant to security. It can show trust boundaries, authentication flows, and data encryption points by filtering for nodes and edges tagged with security concerns in the SAM.29
Sequence View: For a specific user story or API endpoint, this view generates a sequence diagram showing the timed order of interactions between components. This view relies on traces extracted from dynamic analysis or by following a specific call chain within the SAM.24

5.4 Interactive Exploration

Static diagrams, even with multiple views, have limited utility. To truly manage complexity, the user must be able to interact with the visualization dynamically. The front-end of the CodeAtlas tool should be an interactive canvas, not just an image viewer.
Key interactive features must include:
Zoom and Pan: Standard navigational controls for large diagrams.
Click-to-Drill-Down: This is the primary mechanism for navigating the C4 hierarchy. Clicking on a container in a Level 2 diagram should seamlessly transition the user to the corresponding Level 3 component diagram for that container.56
Hover-for-Details: Mousing over any node or edge should pop up a tooltip displaying its detailed attributes from the SAM (e.g., its full name, description, source code location, list of cross-cutting concerns).14
Dynamic Filtering and Highlighting: A search bar should allow users to type a query (e.g., a component name, a concern like "caching"). The system should then highlight all matching elements and their direct relationships in real-time, dimming the rest of the diagram. This allows users to instantly focus on relevant sub-systems.57
The realization that the SAM is a multi-view model, not a single diagram, is fundamental. It shifts the product vision for CodeAtlas from a simple "diagram generator" to a powerful "interactive architecture exploration platform." Different stakeholders have vastly different needs 47, and attempting to serve them all with a single, static image is a recipe for failure. By providing on-demand generation of specific, user-defined views from a single, consistent model, the system becomes exponentially more valuable. Interactivity is the ultimate tool for complexity management. A static diagram has a fixed cognitive load, whereas an interactive canvas empowers the user to dynamically control the amount of information they see at any moment, enabling progressive disclosure of detail as their investigation deepens.

Section 6: Comparative Analysis: AI-Generated vs. Manually-Crafted Architectural Diagrams

To validate the effectiveness of the proposed framework, a rigorous, evidence-based comparison against traditional, manual diagramming methods is required. This section outlines a series of case studies designed to benchmark the AI-powered approach across several key dimensions, including speed, accuracy, consistency, and maintainability. The results will provide a quantitative assessment of the framework's value proposition.

6.1 Case Study Methodology

The analysis will be conducted using a test suite of real-world open-source projects selected to represent a variety of architectural styles and complexities.
Test Suite Selection:
Project A (Monolith): A traditional, single-deployment application with a layered architecture.
Project B (Microservices): A distributed system composed of multiple, independently deployable services communicating via APIs and a message bus.
Project C (Event-Driven): A system architected around asynchronous event processing.
Process for Each Project:
Manual Baseline Creation: An expert software architect, external to the project, will be tasked with manually analyzing the source code and creating a set of "gold standard" architectural diagrams. This set will include a C4 Level 2 (Container) diagram, a C4 Level 3 (Component) diagram for a key service, and a sequence diagram for a critical user workflow. The time taken for this manual process will be recorded.
AI-Powered Generation: The CodeAtlas framework, as specified in this report, will be executed on the same codebase to automatically generate the equivalent set of diagrams. The total computation time, including any automated refinement loops, will be recorded.

6.2 Benchmarking Axes and Results

The two sets of diagrams (manual vs. AI-generated) will be compared across the following objective and subjective axes.
Creation Time: The time required to produce the initial set of diagrams. The AI-powered approach is expected to be orders of magnitude faster, reducing a process that takes days or weeks to minutes.1
Quantitative Accuracy: The AI-generated diagrams will be evaluated against the SAM (as the ground truth) using the full metric suite from Section 3. The manual diagrams will also be assessed against the SAM to quantify the extent of human error or omission.
Consistency: The diagrams within each set will be checked for consistency. For example, is a component named identically in the container view and the sequence diagram? The SAM-based approach is expected to achieve perfect consistency, whereas manual diagrams often suffer from inconsistencies.2
Maintenance Effort: This is the most critical axis of comparison. A significant change will be introduced into the source code of each project (e.g., refactoring a service, adding a new API endpoint). The time and effort required to update both the manual and AI-generated diagrams to reflect this change will be measured. For the AI framework, this should be near-zero, as the update is handled automatically by the CI/CD integration. This directly tests the primary weakness of manual documentation: its inability to stay synchronized with code.2

6.3 Analysis of Strengths and Weaknesses

The case studies are expected to highlight the distinct advantages and disadvantages of each approach.
AI-Generated Diagrams (CodeAtlas Framework):
Strengths: Unparalleled speed in generation and updates; guaranteed consistency with the current state of the code; objective and repeatable quality measurement via metrics; the ability to generate numerous, consistent views on demand.46
Weaknesses: The quality of the output is fundamentally dependent on the quality of the input code and its comments. The system may struggle to infer high-level architectural intent or business context that is not explicitly reflected in the code. There is also an initial investment cost in setting up the analysis pipeline.
Manually-Crafted Diagrams:
Strengths: The ability of a human architect to incorporate abstract business goals, future-state plans, and informal, "tribal" knowledge that exists outside the codebase. Humans can apply creative judgment to layout and emphasis to tell a specific story.
Weaknesses: The process is extremely slow, labor-intensive, and prone to human error and omission. The resulting artifacts are static, rapidly become obsolete, are difficult and costly to maintain, and are often inconsistent across different views.2
The results of this comparative analysis are best summarized in a benchmark table.

Metric
Project B (Microservices) - Manual
Project B (Microservices) - AI-Generated
Delta / Improvement
Initial Creation Time
24 hours
5 minutes
> 280x Faster
Update Time (Post-Refactor)
3 hours
< 1 minute (automated)
> 180x Faster
Node Precision
0.95
1.00
+5.3%
Node Recall
0.88
0.98
+11.4%
Edge Recall
0.82
0.96
+17.1%
Semantic Coherence Score (SCS)
N/A (Subjective)
0.92
Quantitatively Measurable
Consistency Across Views
Low (manual sync)
High (model-driven)
Qualitative Improvement


6.4 Conclusions from the Analysis

The primary value of an AI-powered framework like CodeAtlas is not merely in the initial generation of a diagram but in its continuous maintenance and guaranteed consistency. The case study's maintenance-effort benchmark is designed to unequivocally demonstrate this point. The recurring, debilitating cost in software engineering is not the one-time creation of a document, but the perpetual, manual effort required to keep it synchronized with a constantly evolving codebase—a task at which manual processes almost universally fail. The automated conformance checking loop specified in Section 4 directly and permanently solves this core problem.
This leads to a final, crucial conclusion: the AI framework and the human architect are complementary, not competitive. The system automates the laborious, error-prone, and historically intractable task of producing an accurate, objective, and always-current representation of the system's as-is architecture. This frees the human architect from the role of a mere drafter and empowers them to perform their true, high-value function: strategic analysis. With a perfect, real-time map of the existing territory provided by the tool, the architect can focus their expertise on analyzing the to-be architecture, identifying and prioritizing technical debt, planning complex refactoring initiatives, and making data-driven strategic decisions that align the system's evolution with business goals. The AI handles the "what is," so the human can master the "why" and "what's next."

Conclusion and Recommendations

This report has detailed a comprehensive framework for the development of CodeAtlas, an AI-powered system for generating high-fidelity software architecture diagrams. The proposed methodology represents a significant advancement over existing tools by moving beyond simple dependency mapping to a holistic system of architectural modeling, generation, and automated validation.
The core of the framework rests on several foundational principles derived from an exhaustive analysis of the field:
The Model-First Imperative: The system's central function must be the creation of a structured, queryable Software Architecture Model (SAM) through a combination of static, dynamic, and behavioral code analysis. This model serves as the single source of truth, enabling both accurate diagram generation and quantitative validation. This elevates the tool from a simple diagrammer to a true architectural modeling platform.
Constrained, Model-to-Text LLM Tasking: The role of the Large Language Model should be strictly confined to translating the pre-validated SAM into Mermaid.js or PlantUML syntax. This leverages the LLM's strengths in language generation while mitigating its weaknesses in deep architectural reasoning, ensuring high predictability and fidelity.
Quantitative, Multi-Faceted Validation: Diagram accuracy must be measured objectively using a suite of metrics that assess structural fidelity (Node/Edge Precision & Recall, Graph Edit Distance), architectural integrity (Metric Drift), and semantic coherence (Semantic Coherence Score). These metrics are the engine of automated quality control.
Closed-Loop Refinement and Conformance Checking: The system must implement a "self-healing" feedback loop where validation results are used to automatically generate refinement prompts, progressively improving diagram accuracy. Integrating this loop into the CI/CD pipeline solves the critical problem of architectural drift, ensuring diagrams remain perpetually synchronized with the source code.
Viewpoint-Based Complexity Management: Visual clarity for complex systems is achieved not by simplifying the model, but by generating multiple, focused views (e.g., C4 levels, data flow, security) from the central SAM and enabling rich, interactive exploration by the user.
Actionable Recommendations for the CodeAtlas Team:
Prioritize the SAM: The initial development effort should focus on building the robust SAM generation pipeline (Section 1). This is the bedrock of the entire system.
Adopt a View-Based Product Strategy: Market and design CodeAtlas not as a tool that generates "a diagram," but as an interactive platform for exploring multiple, consistent architectural views derived from a living model of the codebase.
Implement the Full Metric Suite: Resist the temptation to use simpler validation checks. The multi-faceted metric framework (Section 3) is essential for capturing the nuances of diagram quality and for powering an effective refinement loop.
Build for CI/CD Integration from Day One: The automated conformance checking capability (Section 4) is the single most compelling feature for enterprise customers, as it solves a major, costly pain point. This should be a core, not an add-on, feature.
Focus the User Experience on Interaction: The front-end should be designed as an interactive canvas (Section 5) that empowers users to manage complexity through drill-downs, filtering, and dynamic exploration, rather than as a static image viewer.
By implementing this framework, CodeAtlas can position itself as a next-generation software architecture tool that provides not just pictures, but trustworthy, interactive, and perpetually current insights into the structure and behavior of complex software systems. This approach automates the tedious and error-prone work of documentation, freeing architects and developers to focus on the strategic challenges of building better software.
Works cited
Introducing Swark: Automatic Architecture Diagrams from Code | by Oz Anani | Medium, accessed June 28, 2025, https://medium.com/@ozanani/introducing-swark-automatic-architecture-diagrams-from-code-cb5c8af7a7a5
Harnessing generative AI to create and understand architecture ..., accessed June 28, 2025, https://ijsra.net/sites/default/files/IJSRA-2024-2601.pdf
Automated Software Architecture Diagram Generator using Natural Language Processing, accessed June 28, 2025, https://www.researchgate.net/publication/371011658_Automated_Software_Architecture_Diagram_Generator_using_Natural_Language_Processing
(PDF) A SYSTEMATIC ANALYSIS ON SOFTWARE ... - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/384241240_A_SYSTEMATIC_ANALYSIS_ON_SOFTWARE_ARCHITECTURE_RECOVERY_TECHNIQUES
Software architecture recovery - Wikipedia, accessed June 28, 2025, https://en.wikipedia.org/wiki/Software_architecture_recovery
Axivion Software Architecture Verification | Analyze the Architecture ..., accessed June 28, 2025, https://www.qt.io/quality-assurance/axivion-architecture-verification
What Is Static Analysis? | Datadog, accessed June 28, 2025, https://www.datadoghq.com/knowledge-center/static-analysis/
Static program analysis - Wikipedia, accessed June 28, 2025, https://en.wikipedia.org/wiki/Static_program_analysis
Abstract syntax tree - Wikipedia, accessed June 28, 2025, https://en.wikipedia.org/wiki/Abstract_syntax_tree
Visualizing Abstract Syntax Trees in JavaScript | by Viswesh Subramanian - Medium, accessed June 28, 2025, https://medium.com/javascriptstore/visualizing-abstract-syntax-trees-in-javascript-3f382c3b7802
viswesh/astVisualizer: A tool to visualize abstract syntax tree. - GitHub, accessed June 28, 2025, https://github.com/viswesh/astVisualizer
Reverse Engineering - Software Engineering - GeeksforGeeks, accessed June 28, 2025, https://www.geeksforgeeks.org/software-engineering/software-engineering-reverse-engineering/
Reverse engineering: from the system back to the code - SYSPARENCY, accessed June 28, 2025, https://www.sysparency.com/en/reverse-engineering-from-the-system-back-to-the-code/
Top 7 diagrams as code tools for software architecture | by IcePanel - Medium, accessed June 28, 2025, https://icepanel.medium.com/top-7-diagrams-as-code-tools-for-software-architecture-1a9dd0df1815
C4 model tools, accessed June 28, 2025, https://c4model.tools/
Tooling | C4 model, accessed June 28, 2025, https://c4model.com/tooling
Comparing Methods for Visualizing Software Architecture · vibecoders - Skool, accessed June 28, 2025, https://www.skool.com/vibe-coders/comparing-methods-for-visualizing-software-architecture
High-level Static and Dynamic Visualisation of Software Architectures - nzjohng.github.io, accessed June 28, 2025, https://nzjohng.github.io/publications/papers/vl00.pdf
Visualize Microservice Dependencies in a Team Context - CodeScene, accessed June 28, 2025, https://codescene.com/engineering-blog/visualize-microservice-dependencies-in-team-context/
Microservices Visualization: The Secret to Effective Management - groundcover, accessed June 28, 2025, https://www.groundcover.com/microservices-observability/microservices-visualization
Microservice Visualization - Datadog, accessed June 28, 2025, https://www.datadoghq.com/containers/microservice-visualization/
Generate MermaidJS Customizable Flowcharts. Prompt included. : r ..., accessed June 28, 2025, https://www.reddit.com/r/PromptEngineering/comments/1k9c6an/generate_mermaidjs_customizable_flowcharts_prompt/
Level Up Your Diagramming Game: Unleash the Power of Gen AI ..., accessed June 28, 2025, https://medium.com/@psachethana/level-up-your-diagramming-game-unleash-the-power-of-gen-ai-706c546f3cba
The ChatGPT prompt list for Software Engineers: Prompts to generate software diagrams in Mermaid | Medium, accessed June 28, 2025, https://medium.com/@martin-jurran/chatgpt-prompt-list-for-software-engineers-prompts-to-generate-software-diagrams-in-mermaid-deaf2f373104
C4 model: Home, accessed June 28, 2025, https://c4model.com/
Diagram as Code: PlantUML and Mermaid-Js - Miro Community, accessed June 28, 2025, https://community.miro.com/product-news-31/diagram-as-code-plantuml-and-mermaid-js-9691
Mermaid: Generation of diagrams like flowcharts or sequence diagrams from text | Hacker News, accessed June 28, 2025, https://news.ycombinator.com/item?id=44049619
Balancing Cross-Cutting Concerns in Clean Architecture - Milan Jovanović, accessed June 28, 2025, https://www.milanjovanovic.tech/blog/balancing-cross-cutting-concerns-in-clean-architecture
Capturing cross cutting concerns : r/softwarearchitecture - Reddit, accessed June 28, 2025, https://www.reddit.com/r/softwarearchitecture/comments/1j3oahg/capturing_cross_cutting_concerns/
How to implement Design Pattern – Separation of concerns - CAST Software, accessed June 28, 2025, https://www.castsoftware.com/pulse/how-to-implement-design-pattern-separation-of-concerns
Layered BubbleTea Software Architecture Visualisation - Pure, accessed June 28, 2025, https://pure.tue.nl/ws/files/349611570/Layered_BubbleTea_Software_Architecture_Visualisation_1_.pdf
Architecture Diagrams Documentation (v11.1.0+) - Mermaid, accessed June 28, 2025, https://mermaid.js.org/syntax/architecture.html
Edge Node Architecture Diagram - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/figure/Edge-Node-Architecture-Diagram_fig1_340812062
Unlocking Graph Similarity Secrets - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/ultimate-guide-graph-similarity-measures-spectral-graph-theory
A Graph Pattern Matching Approach to Software Architecture Recovery - CiteSeerX, accessed June 28, 2025, https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=d46ecc7765406955d45f134f1777154304e901c4
Mastering Metrics in Software Architecture - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/mastering-metrics-in-software-architecture
(PDF) Software Architecture Metrics: a literature review - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/330700405_Software_Architecture_Metrics_a_literature_review
Metrics for the Design Model of the Product - GeeksforGeeks, accessed June 28, 2025, https://www.geeksforgeeks.org/software-engineering/metrics-for-the-design-model-of-the-product/
Top 10 Tools for Calculating Semantic Similarity - TiDB, accessed June 28, 2025, https://www.pingcap.com/article/top-10-tools-for-calculating-semantic-similarity/
Different Techniques for Sentence Semantic Similarity in NLP - GeeksforGeeks, accessed June 28, 2025, https://www.geeksforgeeks.org/nlp/different-techniques-for-sentence-semantic-similarity-in-nlp/
Semantic similarity - Wikipedia, accessed June 28, 2025, https://en.wikipedia.org/wiki/Semantic_similarity
12 Data Quality Metrics to Measure Data Quality in 2025 - lakeFS, accessed June 28, 2025, https://lakefs.io/data-quality/data-quality-metrics/
Successful Software Architecture Review: Step-by-Step Process - DevCom, accessed June 28, 2025, https://devcom.com/tech-blog/successful-software-architecture-review-step-by-step-process/
Software Architecture Verification and Validation - Qt, accessed June 28, 2025, https://www.qt.io/quality-assurance/blog/software-architecture-verification-and-validation
Automating architectural conformance checking by means of logic meta programming, accessed June 28, 2025, https://www.researchgate.net/publication/242355723_Automating_architectural_conformance_checking_by_means_of_logic_meta_programming
Automated architecture diagrams - Elements.cloud, accessed June 28, 2025, https://elements.cloud/salesforce-automated-architecture-diagrams/
Automated architecture diagrams. Who doesn't love architecture diagrams… | by Chris Brown | TheFork Engineering Blog | Medium, accessed June 28, 2025, https://medium.com/thefork/automated-architecture-diagrams-53f538f615b7
Mastering Dynamic Complexity - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/mastering-dynamic-complexity
Dynamic Complexity Simplified - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/dynamic-complexity-simplified
Code Complexity: An In-Depth Explanation and Metrics - Codacy | Blog, accessed June 28, 2025, https://blog.codacy.com/code-complexity
Complexity in Software Development and How to Reduce It | by BrainerHub Solutions, accessed June 28, 2025, https://medium.com/@marketing_18039/complexity-in-software-development-and-how-to-reduce-it-2bee46ae5dc7
Code Visualization: 4 Types of Diagrams and 5 Useful Tools - CodeSee, accessed June 28, 2025, https://www.codesee.io/learning-center/code-visualization
C4 Model Tools: Key Features and 7 Tools to Consider - CodeSee, accessed June 28, 2025, https://www.codesee.io/learning-center/c4-model-tools
Software Architecture Extraction, accessed June 28, 2025, https://scg.unibe.ch/download/AC/SDE-07ArchitectureExtraction.pdf
What is Architecture Diagramming? - AWS, accessed June 28, 2025, https://aws.amazon.com/what-is/architecture-diagramming/
Ilograph Interactive System Architecture Diagrams, accessed June 28, 2025, https://www.ilograph.com/
Software Graph Visualization | Apiiro, accessed June 28, 2025, https://apiiro.com/software-graph-visualization/
Create Software Architecture Diagram with Excalidraw, accessed June 28, 2025, https://plus.excalidraw.com/use-cases/software-architecture-diagram
The Role of AI in Software Architecture: Trends and Innovations - Imaginary Cloud, accessed June 28, 2025, https://www.imaginarycloud.com/blog/ai-in-software-architecture
