
Architecting a High-Fidelity Reasoning Engine for Code Analysis: A Technical Blueprint for Mitigating Hallucination


Section I: Foundational Layer: Advanced Retrieval-Augmented Generation for Codebases

The efficacy of any Large Language Model (LLM) based code analysis tool is fundamentally constrained by the quality and relevance of the context it is provided. For a system like Uveddi, designed to perform nuanced architectural analysis and anti-pattern detection, a simple Retrieval-Augmented Generation (RAG) pipeline is demonstrably insufficient. The state-of-the-art necessitates a paradigm shift away from treating code as unstructured text towards a model that comprehends and leverages the intricate structural relationships inherent in a software repository. This section outlines the architecture for such an advanced, structure-aware RAG system, forming the foundational layer upon which all subsequent reasoning will be built.

1.1. The Inadequacy of Conventional RAG for Code Analysis

Conventional RAG systems, while highly effective for natural language processing tasks, exhibit critical flaws when applied to source code. These systems typically pair semantic vector search with naive chunking strategies, a combination that fails to respect the syntactic and semantic integrity of code.1
The most common chunking methods, such as fixed-size or simple recursive character splitting, divide documents based on character or token counts.3 When applied to source code, this approach is destructive, leading to severe context fragmentation. A function can be bisected from its signature, a class definition can be separated from its methods, or a logical block can be split mid-statement. The resulting chunks are often syntactically invalid and semantically incomplete, providing a corrupted and misleading context to the LLM. This degradation of input quality is a primary driver of model hallucination; the LLM is forced to "fill in the blanks" for missing information, leading to erroneous conclusions.5
Furthermore, relying solely on semantic similarity for retrieval is a flawed premise for architectural analysis. A query for a concept like "database connection pooling" might retrieve a function with semantically similar variable names (e.g., pool, connection, db_client), but this function could be a simple test utility or a mock object, entirely irrelevant to the core application architecture. This semantic ambiguity provides the LLM with context that, while factually correct in isolation, is architecturally misleading and poisons the reasoning process.

1.2. The Paradigm Shift: Graph-Based RAG for Code

To overcome these limitations, the most advanced systems are moving towards a Graph-RAG approach, which models the entire codebase as an interconnected knowledge graph.6 This represents a fundamental architectural evolution: from treating code as a linear stream of text to understanding it as a structured graph of entities and their relationships.
A robust graph schema is the cornerstone of this approach. In this model, nodes represent distinct code entities, while edges represent the relationships between them. A comprehensive schema, inspired by projects like CodeRAG and code-graph-rag, would include 6:
Node Types: Project, Package, Module (file), Class, Function, Method, Struct, Enum, ExternalPackage (dependency).
Relationship Types (Edges): CONTAINS (hierarchical, e.g., Module contains Class), DEFINES (e.g., Module defines Function), CALLS (function-to-function invocation), INHERITS (class-to-class inheritance), IMPLEMENTS (class-to-interface), DEPENDS_ON (project-to-external-package).
The CodeRAG framework exemplifies this by constructing a DS-code graph to capture not only direct dependencies but also semantic similarity and indirect relationships, allowing it to retrieve a holistic context that includes invoked APIs and related code snippets.8 The open-source
code-graph-rag project provides a concrete, multi-language implementation blueprint using the tree-sitter parsing library to populate a Memgraph graph database.6
This graph structure transforms the retrieval process. Instead of a simple vector search, a user's natural language query can be translated by an LLM agent into a formal graph query language (e.g., Cypher for Memgraph or Neo4j).6 This allows the system to execute complex, multi-hop traversals to retrieve entire subgraphs of related entities. For a query about a specific service, the system can retrieve the service's class definition, all the methods it contains, the other services it calls, the classes it inherits from, and its external dependencies. This rich, structured context far surpasses the capabilities of text-based RAG and provides the LLM with a high-fidelity snapshot of the local architecture, mirroring how a human developer navigates and understands a codebase.

1.3. Structure-Aware Chunking: The Primacy of AST-Based Strategies

The quality of the knowledge graph—or any code-based vector index—is directly dependent on the intelligence of the chunking strategy used to populate it. For code, chunking must be structure-aware.
The most effective and recommended approach is AST-based chunking. This method leverages an Abstract Syntax Tree (AST), a tree representation of the code's syntactic structure, to identify logical boundaries. By splitting code at the level of functions, classes, methods, or other syntactic units, it ensures that every chunk is a syntactically valid and semantically self-contained unit of code.5 This avoids the context fragmentation that plagues simpler methods.
A state-of-the-art algorithm in this domain is cAST (chunking via Abstract Syntax Trees).9 This language-agnostic method, which relies on
tree-sitter parsers, employs a recursive, split-then-merge algorithm. It performs a top-down traversal of the AST, attempting to fit large nodes (like an entire class definition) into a single chunk. If a node exceeds the configured size limit (measured in non-whitespace characters for consistency), it is recursively split. To prevent the creation of overly small, low-context chunks, a subsequent greedy merging step combines adjacent small sibling nodes. This process maximizes the information density of each chunk while preserving syntactic integrity.
The choice of chunking granularity is not one-size-fits-all; it must be tailored to the analysis task. A comprehensive tool like Uveddi should support a multi-level chunking strategy:
Function/Method-Level Chunking: Essential for analyzing implementation details, such as algorithmic complexity, violations of the single-responsibility principle within a method, or identifying bugs.
Class/File-Level Chunking: Necessary for understanding class-level concerns like cohesion (how well the methods of a class belong together), coupling (dependencies between classes), and detecting patterns like the God Object.
Component-Level Chunking: For high-level architectural analysis, such as inter-service communication or module dependencies, chunks can represent entire directories or modules. These larger chunks should include metadata summarizing the component's public API, its dependency manifest (e.g., Cargo.toml), and a high-level description of its purpose.
Python packages such as ASTSnowballSplitter and astchunk offer practical, open-source implementations that can serve as a reference for building this capability in Rust.10

1.4. Embedding Complex Code Artifacts

The embedding strategy for code must capture not only the semantic meaning of the code's text but also its structural role within the broader architecture. A simple embedding of raw source text is insufficient.
Research indicates that a multi-representation embedding approach yields superior performance.12 Rather than embedding a single representation, the model should be fed a composite of different structural views of the code. For example, a single function could be represented by a collection of paths extracted from its Abstract Syntax Tree (AST), its Control Flow Graph (CFG), and its Program Dependency Graph (PDG).12 This enriches the resulting vector with syntactic, control flow, and data flow information.
In the context of a Graph-RAG system, the unit of embedding is the node within the code graph. The embedding for a Function node, for instance, should be generated from a "document" that synthesizes multiple facets of that function: its source code, its signature (including parameter types and return type), its docstring, the name of its parent class or module, and a list of its immediate dependencies (functions it calls, types it uses).
The process for embedding is as follows:
Parse the entire codebase into a structured knowledge graph using a robust parser like tree-sitter.9
For each relevant node in the graph (e.g., each function, class, and module), generate a composite text document containing its source code and critical metadata.
Utilize a code-specialized embedding model, such as CodeT5 or other transformer-based models trained on code, to generate a dense vector embedding for this composite document.5
Store this vector in a vector database, indexed by the unique identifier of its corresponding node in the code graph. This linkage is crucial for retrieving both the semantic embedding and the full structural context during a query.

1.5. Hybrid Retrieval: Combining Sparse and Dense Search

No single retrieval method is optimal for all code analysis queries. The most robust RAG systems employ a hybrid search strategy that combines the strengths of both dense and sparse retrieval techniques.
Dense Retrieval (Vector Search): This method excels at finding semantically and conceptually related code. It can understand that a query for "user session management" is related to code that implements JWT generation, cookie handling, and database lookups, even if the exact keywords are not present.2 This is powered by the vector embeddings generated in the previous step.
Sparse Retrieval (Keyword Search): This method, typically implemented with algorithms like BM25, is essential for matching exact, literal strings. It guarantees that a search for a specific function name (calculate_user_permissions), a library import (import org.springframework.security.crypto.bcrypt.BCryptPasswordEncoder), or a unique error code (ERR_DATABASE_CONNECTION_FAILED) will find the documents containing those exact terms, which dense search might overlook.2
A hybrid approach executes the query against both the dense vector index and the sparse keyword index simultaneously. The results from both searches are then fused and re-ranked to produce a final, unified list of results. This re-ranking often involves a weighted combination, using a parameter (commonly denoted as alpha) to balance the influence of semantic relevance versus keyword matching.14 This is non-negotiable for a tool like Uveddi. To analyze a potential anti-pattern in the
UserAuthenticationService class, the system must be able to retrieve that exact class (a task for sparse search) while also understanding the broader architectural concepts of authentication and security (a task for dense search).

1.6. Rust Vector Database Recommendations

For an application built in Rust, selecting a Rust-native vector database offers significant advantages in terms of performance, memory safety, binary size, and ease of integration. The choice between a server-based or an embedded database depends on the desired scalability and operational complexity of Uveddi.
Table 1: Comparative Analysis of Rust Vector Databases
Feature
Qdrant 15
SahomeDB 17
LanceDB 18
Milvus 20
Primary Language
Rust
Rust
Rust
C++, Go
Architecture
Server-based (Client-Server)
Embedded (SQLite-style)
Embedded (SQLite-style)
Server-based
Core Indexing
HNSW, Scalar Quantization
HNSW
IVF-PQ
HNSW, IVF, and others
Performance
Very high RPS & low latency; excels in filtered search
Fast search (0.15 ms for 10k vectors); memory-intensive
Fast, but potentially lower recall than HNSW; memory-efficient
Fast indexing, but can lag in RPS/latency with high dimensions
Key Features
Advanced metadata filtering, on-disk storage, replication, sharding
Optional persistence (Sled), incremental ops, flexible metadata
Zero-copy access from object storage, versioning, SQL queries
Distributed architecture, multi-tenancy, multiple index types
Best Use Case
Production-grade, scalable service requiring high performance and advanced filtering.
Simple, high-performance embedded use cases where HNSW speed is critical.
Embedded use cases where memory efficiency and direct object storage access are priorities.
Large-scale, distributed deployments where C++/Go stack is acceptable.

Recommendation:
For a production-ready and scalable version of Uveddi, Qdrant is the superior choice. Its implementation in Rust ensures performance and safety, while its client-server architecture, advanced filtering capabilities, and proven benchmarks make it suitable for handling complex queries on large codebases.15 The ability to perform fine-grained filtering on metadata is essential for the proposed Graph-RAG architecture, allowing the system to narrow down searches based on node types, relationships, or other code attributes before performing the vector search.
For scenarios demanding a simpler deployment model, such as a standalone desktop version of Uveddi or for rapid prototyping, an embedded database is preferable. In this case, the choice is between SahomeDB and LanceDB. SahomeDB, using HNSW, offers higher search performance at the cost of greater memory usage.17 LanceDB, using IVF-PQ, is more memory-efficient and offers unique features like zero-copy access from object storage, making it an excellent choice if resource consumption is a primary concern.18
The evolution from text-based RAG to graph-based RAG is not merely an incremental improvement; it is a necessary correction driven by the fundamental nature of source code. Early attempts to apply standard NLP techniques to code failed because they relied on semantic similarity, a concept that is secondary to structural relationships in software architecture. A function's meaning is defined less by the words in its name and more by what calls it and what it calls. This mismatch between the retrieval mechanism (semantic similarity) and the source of truth (code structure) was a primary source of irrelevant context, which in turn fueled LLM hallucinations. The model was being fed facts that were true in isolation but false within the specific architectural context of the query. Therefore, the adoption of a graph-based RAG architecture is a critical step to align the retrieval process with the ground truth of the codebase. The primary goal of the RAG system is not just to find relevant code snippets but to reconstruct a high-fidelity, query-specific architectural subgraph that can serve as a trustworthy foundation for the LLM's reasoning.
This perspective reveals that the chunking strategy is not a mundane preprocessing step but is, in fact, the ontological definition of the system's understanding of the codebase. The granularity of chunks determines the vocabulary of the knowledge graph and thus sets the upper bound on the analytical depth of the entire system. If the codebase is chunked at the file level, the system can reason about inter-file dependencies but is blind to the function calls occurring within those files. Conversely, adopting an AST-based, function-level chunking strategy populates the graph with function and class nodes, unlocking a far deeper level of reasoning about call chains, coupling, and cohesion. Consequently, Uveddi must implement a hierarchical chunking capability, allowing the reasoning engine to select the appropriate level of granularity—function, class, or component—based on the specific architectural anti-pattern being investigated. Analyzing a "God Object" requires class-level context, whereas tracing an "N+1 Query" anti-pattern demands a fine-grained view of function-level call chains.

Section II: The Reasoning Core: Structuring Prompts and Chains for Architectural Analysis

Once a high-quality, architecturally-aware context has been retrieved, the focus shifts to structuring the interaction with the LLM. This section details the prompt engineering patterns and reasoning frameworks required to guide the model toward accurate, verifiable architectural insights and away from the pitfalls of hallucination. The objective is to transform the LLM from a generic text generator into a specialized architectural analyst.

2.1. Advanced Prompt Engineering for Code Architecture

Effective prompting for code analysis goes far beyond simple questions. It requires a multi-faceted approach that provides deep context, sets clear expectations, and guides the model's behavior.
Providing Rich Context: Every prompt must be grounded in the comprehensive context retrieved by the Graph-RAG system. This includes not only the relevant code snippets but also crucial metadata: the specific problem domain (e.g., "e-commerce backend," "real-time data pipeline"), known architectural patterns already in use (e.g., "this system uses event sourcing"), technical constraints ("must be stateless," "database calls must be non-blocking"), and non-functional requirements like performance or security standards.21
The Persona Pattern: A highly effective technique is to instruct the LLM to adopt a specific expert persona. This primes the model to access the most relevant patterns and knowledge from its training data. Instead of a generic request, the prompt should begin with a clear role assignment, such as: "Act as a senior systems architect specializing in distributed systems and fault tolerance," or "Assume the role of a database performance expert".23 This simple instruction significantly improves the focus and quality of the generated analysis.
Specificity and Referencing: Ambiguity is the enemy of reliable analysis. Prompts must be precise and unambiguous, explicitly referencing the code entities under investigation. Vague requests like "analyze this code for issues" will yield generic and often useless results. A high-quality prompt will be specific: "Analyze the DataManager class located in the file src/core/data.rs for potential violations of the Liskov Substitution Principle in its inheritance hierarchy. Compare its structure to the EventManager class in src/core/events.rs, which is a good implementation example".21 The graph context retrieved in Section I makes this level of specific referencing possible.
Defining the Output Format: To ensure the LLM's output is consistent, machine-readable, and verifiable, the prompt must explicitly define the desired output structure. This can range from simple formatting instructions ("Provide your analysis in a Markdown table with columns for 'Issue', 'File', 'Line Number', and 'Suggestion'") to complex schema definitions ("Generate your response in JSON format conforming to the provided JSON Schema").24

2.2. Eliciting Deeper Insights: Chain-of-Thought and Beyond

For complex tasks like architectural analysis, prompting the LLM to "show its work" is essential for both improving accuracy and enabling verification of its reasoning process.
Chain-of-Thought (CoT) Prompting: CoT is a technique that guides the model to break down a complex problem into a sequence of intermediate, logical steps before arriving at a final conclusion.25 This transparency is invaluable for debugging the LLM's logic and building trust in its outputs. For anti-pattern detection, a CoT prompt would look like this: "To determine if the
OrderController class is a 'God Object', follow these steps: 1. List all distinct responsibilities handled by the class. 2. Evaluate the conceptual cohesion of these responsibilities. 3. Based on this evaluation, provide a final conclusion on whether it constitutes a God Object anti-pattern, explaining your reasoning for each step".22
Tree-of-Thoughts (ToT) Prompting: ToT is a more advanced generalization of CoT that empowers the model to explore and evaluate multiple reasoning paths simultaneously, akin to a tree search.25 This is particularly powerful for architectural tasks that involve strategy and trade-offs. For instance, when a "Spaghetti Code" anti-pattern is detected, a ToT prompt can ask the model to: "Propose three distinct refactoring strategies to improve the modularity of the
process_order function. For each strategy, evaluate its pros and cons regarding implementation effort, performance impact, and risk of introducing new bugs. Finally, recommend the optimal strategy and justify your choice".28 This deliberate exploration of alternatives mirrors the thought process of an experienced human architect and leads to more robust and well-considered recommendations.
Uncertainty-Aware Chain-of-Thought (UnCert-CoT): A key challenge with advanced reasoning techniques is their computational cost (in tokens and latency). The UnCert-CoT approach addresses this by dynamically deciding when to engage in deep reasoning.29 The system uses confidence-based uncertainty measures, such as the entropy of the model's output token probabilities, to gauge its own certainty. If the model is highly confident about a simple finding (e.g., a clear linting violation), it can output the result directly. If it is uncertain about a complex and ambiguous architectural smell, it triggers a more expensive CoT or ToT process. This adaptive strategy allows Uveddi to allocate its computational resources efficiently, focusing deep reasoning only where it is most needed.

2.3. Enforcing Reliability with Structured Outputs

For the analysis generated by Uveddi to be programmatically useful—for integration into CI/CD pipelines, reporting dashboards, or automated refactoring tools—its output must adhere to a strict, predictable schema.
JSON Schema for Reliability: JSON Schema is the industry-standard vocabulary for defining the structure, data types, required fields, and validation constraints of JSON data.30 It serves as an unambiguous contract that the LLM's output must follow. By defining a schema for anti-pattern findings, we ensure that every piece of information is correctly typed and placed, facilitating automated processing and validation.31
The Superiority of Structured Output APIs: Modern LLM providers like OpenAI now offer Structured Output capabilities that can guarantee the model's response will conform to a supplied JSON Schema.37 This is a critical advancement over the older "JSON mode," which only ensured the output was syntactically valid JSON but did
not guarantee adherence to a specific schema. For a high-reliability tool like Uveddi, using these schema-enforcing APIs is non-negotiable. It eliminates an entire class of errors related to malformed or incomplete outputs and simplifies downstream parsing logic.
Trade-offs Between Output Formats: While JSON is the most robust choice, it's important to understand the trade-offs with other formats.
Table 2: Trade-Offs of Structured Output Formats
Format
Token Efficiency
Ease of Parsing (Rust)
Schema Enforcement Reliability
Supports Nesting/Complexity
Primary Trade-off
JSON (with Schema API)
Low (verbose syntax) 38
High (excellent library support, e.g., serde_json)
Very High (guaranteed by modern APIs) 37
Yes
Reliability vs. Cost/Latency
YAML
Medium (less verbose than JSON)
Medium (requires external crates like serde_yaml)
Medium (prone to formatting inconsistencies from LLMs) 38
Yes
Readability vs. Reliability
Custom (e.g., TSV)
High (minimal syntax)
Low-Medium (requires custom parsing logic)
Low (no standard for schema enforcement)
No (typically flat data only)
Efficiency vs. Expressiveness

Recommendation: For Uveddi, the only viable choice for reliable, verifiable, and machine-parseable analysis is JSON with JSON Schema enforcement via a supported API. The increased token cost and latency associated with JSON's verbosity are a necessary price to pay for the guarantee of structural correctness and reliability.37
The selection of a reasoning technique like CoT versus ToT represents a direct trade-off between the depth of analysis and the associated computational cost. CoT provides a single, linear path of logic, which is highly effective for explaining how a conclusion was reached and is relatively efficient.27 It is well-suited for identifying anti-patterns with clear, sequential detection criteria. In contrast, ToT's exploration of multiple reasoning branches is computationally more expensive but is fundamentally necessary for tasks that require strategic thinking, comparison of alternatives, or creative problem-solving—all hallmarks of high-level architectural analysis.28 A simple "long method" detection might only require CoT, but a task like "propose and evaluate three refactoring options for this tightly coupled module" inherently demands the exploratory power of ToT. This implies that the Uveddi
AIReasoningEngine should not be a monolithic entity but a strategic orchestrator. It must possess the capability to dynamically select the appropriate reasoning pattern based on the complexity and nature of the analysis, potentially guided by an uncertainty-aware mechanism like UnCert-CoT to optimize performance and cost.29
Furthermore, it is crucial to recognize that structured output is not merely a post-processing formatting step; it is a core mechanism for mitigating hallucinations. An LLM generating free-form text has a vast space for plausible-sounding but factually ungrounded statements. By forcing the model to populate a rigid JSON Schema, we fundamentally constrain its output space. The task shifts from the open-ended "write a report" to the highly constrained "fill out this form." The model cannot simply invent a narrative; it must find or deduce specific pieces of information from the provided context that fit into the schema's required fields, such as fileName, className, lineNumber, antiPatternType, and evidence. If the model cannot locate a specific piece of evidence, it is far more likely to leave a field null or empty—a verifiable and useful signal—than to hallucinate a value. This makes the design of the JSON schema itself a critical component of prompt engineering. A well-designed schema acts as a scaffold for the LLM's reasoning process, compelling it to ground its output in the verifiable facts of the provided context.

Section III: Architecting for Reliability: Multi-Stage Verification and Self-Correction

A single-pass analysis from an LLM, no matter how sophisticated the prompt, is insufficient for a mission-critical tool like Uveddi. To build a system that produces trustworthy and factually grounded results, it is essential to architect a pipeline that treats the initial LLM output as a hypothesis to be rigorously verified and refined. This section details multi-stage architectures designed to scrutinize, validate, and self-correct AI-generated architectural insights.

3.1. The Primary-Critic Model: An Architecture for Verifiable Analysis

The foundational pattern for verification is the Primary-Critic model. This architecture decomposes the analysis process into two distinct roles, performed by separate LLM agents, to introduce a layer of critical review.40
The Primary Agent: This agent is the initial analyst. It receives the comprehensive context from the Graph-RAG system and the task-specific prompt. Its function is to perform the core analysis and generate its findings in a structured JSON format, as defined by the system's schema. This is the "generator" of the initial hypothesis.
The Critic Agent: This agent acts as an automated peer reviewer. It receives the same original prompt and context as the Primary agent, along with the Primary's structured JSON output. The Critic's sole purpose is to evaluate the Primary's work. Its prompt is explicitly designed to be skeptical, asking it to perform tasks such as:
"Verify that every claim in this analysis is directly supported by the provided code context."
"Check for logical inconsistencies in the reasoning."
"Identify any potential hallucinations or statements not grounded in evidence."
"Act as a skeptical senior software architect reviewing a pull request. Is this analysis accurate and actionable?"
The communication between the two agents is mediated by the structured data format. The Critic's output is also structured, providing either a validation score, a simple boolean pass/fail, or, most usefully, a detailed list of identified errors and suggested corrections, which can then be used to trigger a correction loop.41

3.2. Simulating a Software Team: Multi-Agent Collaboration

The Primary-Critic model can be extended into a more sophisticated multi-agent workflow that mimics the division of labor in a high-functioning software development team. This approach leverages specialized agents, each with a focused role and prompt, to improve the overall quality and robustness of the final output.42
An effective architecture, inspired by the ACT (Analyst-Coder-Tester) model, would involve the following agents 42:
The Analyst Agent: This agent acts as a project manager or team lead. It receives the high-level user query (e.g., "Analyze the repository for performance-related anti-patterns") and decomposes it into a concrete, actionable plan. It identifies the key areas of the codebase to investigate and generates specific sub-tasks for the other agents.
The Coder (Primary) Agent: This agent executes the plan created by the Analyst. It receives a specific sub-task (e.g., "Analyze the DataAccessLayer module for N+1 query patterns") and the relevant code context, and performs the detailed analysis, generating the initial findings.
The Tester (Critic) Agent: This agent takes the findings from the Coder agent and validates them. Its role is to run checks, both deterministic and AI-based, to confirm the accuracy of the findings. It might cross-reference claims against the code graph, run a static analyzer, or even generate and execute a unit test to verify a behavioral claim. It then produces a feedback report.
This collaborative structure allows each agent to be highly specialized, leading to a more thorough and reliable analysis than a single, monolithic agent could achieve.

3.3. Grounding: Verification Against Deterministic Sources

The most powerful technique for eliminating hallucinations is to ground the LLM's claims in verifiable, deterministic sources of truth. The verification stage must not rely solely on another LLM's opinion; it must fact-check against the code itself.
Fact-Checking Loop: When the Primary agent makes a specific, verifiable claim (e.g., "Method getUserOrders in OrderService.java iterates through a list of users and executes a separate database query for each user, indicating an N+1 anti-pattern"), the verification system must programmatically confirm this claim.
Verification Techniques:
Code Graph Traversal: The fundamental claims about code structure, such as "Class A inherits from B" or "Function X calls function Y," can be instantly and deterministically verified by querying the code knowledge graph built in Section I. If the corresponding edge, such as (X)-->(Y), does not exist in the graph, the claim is a hallucination and is immediately flagged.
Static Analysis Tool Integration: Traditional static analysis tools (SAST) can be integrated as specialized verifiers. The LLM can propose a potential issue (e.g., a possible SQL injection vulnerability), and the system can then invoke a targeted scan from a tool like SonarQube on that specific code segment to confirm or deny the finding.1
Sandboxed Code Execution: For claims about runtime behavior (e.g., "this code will panic if the input is null"), the system can use an LLM to generate a minimal unit test that reproduces the condition. This test is then executed in a secure, sandboxed environment to verify the behavior. This approach is particularly effective for confirming bugs and runtime errors.8

3.4. Implementing Self-Correction Loops

When the Critic agent or a deterministic verifier identifies a flaw in the initial analysis, the system should not simply discard the result and fail. Instead, it should initiate a self-correction loop to iteratively refine the output.41
The process is as follows:
The Primary agent generates its initial analysis.
The verification stage (comprising the Critic agent and deterministic checks) reviews the analysis and produces a structured feedback report detailing the specific errors, logical fallacies, or ungrounded claims.
This feedback report is then passed back to the Primary agent. A new prompt is constructed that includes the original request, the original context, and the specific corrections required. For example: "Your previous analysis was reviewed and found to contain the following errors: [Insert structured feedback]. Please regenerate your analysis, ensuring you correct these specific issues and ground all claims in the provided evidence."
This refinement loop can be configured to run for a fixed number of iterations (e.g., up to three times, as suggested in some research 42) or until the analysis successfully passes all verification checks.
In Rust, this iterative workflow can be elegantly managed using a state machine pattern or a simple loop that passes the analysis state between the Primary and Critic functions until a terminal condition (success or max retries) is met. The IterativeCodeAgent described in research provides a strong conceptual model for this implementation.41
The introduction of a multi-agent or Primary-Critic architecture is more than just a simple verification step; it is a mechanism for instilling structured, adversarial reasoning into the system. A single LLM, even when prompted with CoT, operates within its own self-consistent logical bubble. It can produce a line of reasoning that is internally coherent but based on a flawed premise, leading to a plausible but incorrect conclusion. The introduction of a separate Critic agent, explicitly prompted to be skeptical and to find flaws, shatters this bubble.40 This adversarial dynamic forces the Primary agent to produce an analysis that is robust enough to withstand scrutiny. The process compels the system to externalize and resolve its own internal uncertainties. If the Primary agent's reasoning is weak, the Critic's role is to expose it, and the self-correction loop provides the mechanism to resolve that exposed uncertainty.41 The effectiveness of this entire architecture, therefore, hinges on the "persona" and instructions given to the Critic. A lenient critic is useless. The Critic must be prompted to be relentlessly rigorous, to demand concrete evidence for every claim, and to default to flagging an issue for human review rather than accepting any unverified assertion.
This highlights that the most resilient verification systems arise from the synthesis of LLM-based critics and deterministic tools, as they address different classes of hallucinations. LLM critics excel at identifying logical fallacies, reasoning gaps, and subtle misinterpretations of intent within the generated text—they check the coherence of the analysis.40 In contrast, deterministic tools like graph queries, compilers, or static analyzers are masters of verifying atomic facts—they check the
correctness of the foundational claims upon which the reasoning is built.1 A hallucination can be perfectly coherent but factually incorrect (e.g., a flawless argument based on the false premise that "Function A calls Function B"). An LLM critic alone might miss this if it doesn't re-verify the premise. Conversely, an analysis can be a collection of true facts that are woven into an illogical or nonsensical architectural conclusion, a flaw that a deterministic tool would miss. Therefore, a truly robust verification pipeline must be layered. First, it must use deterministic tools to fact-check every verifiable claim in the LLM's output against the ground truth of the code graph and source code. Second, it must use an LLM critic to evaluate the logic, coherence, and relevance of the
remaining, now fact-checked, analysis. This two-layer approach ensures both factual grounding and logical soundness, leading to a dramatic reduction in hallucinations.

Section IV: Implementation Blueprint in Rust

Translating the preceding architectural concepts into a high-performance, reliable, and maintainable system requires careful consideration of the Rust language and its ecosystem. This section provides a concrete implementation blueprint for the Uveddi AI Reasoning Engine, addressing specific questions about traits, libraries, resource management, and error handling.

4.1. Designing the AIReasoningEngine Trait

A modular and testable design begins with a well-defined central abstraction. The AIReasoningEngine trait will serve as the primary interface for all analysis operations, decoupling the core application logic from the specific implementation details of the reasoning pipeline. This allows for different engine implementations (e.g., a simple single-pass engine for quick scans, a full multi-agent engine for deep audits) to be used interchangeably.
A recommended structure for this trait and its associated data types is as follows:

Rust


use std::path::PathBuf;
use async_trait::async_trait;

/// Represents a single, structured finding of an architectural anti-pattern.
/// This struct is designed to be serializable to/from JSON.
#
pub struct AntiPatternFinding {
    pub pattern_type: String, // e.g., "GodObject", "NPlusOneQuery"
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    #
    pub evidence_snippet: String, // The exact code snippet that constitutes evidence.
    pub explanation: String, // The LLM's reasoning for why this is an anti-pattern.
    pub suggestion: String, // A proposed refactoring or fix.
    #
    pub uncertainty_score: f32, // A score from 0.0 (certain) to 1.0 (uncertain).
}

/// Contains all the necessary context for a single analysis task,
/// retrieved by the RAG system.
#
pub struct AnalysisContext {
    pub retrieved_code_chunks: Vec<String>,
    // A representation of the relevant code graph, e.g., a subgraph serialized
    // into a format like Graphviz DOT or JSON Graph Format.
    pub relevant_graph_data: String,
}

/// The core trait for the AI Reasoning Engine.
/// It is generic over the error type to allow for different implementations.
#[async_trait]
pub trait AIReasoningEngine {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Performs an analysis for architectural anti-patterns based on the provided
    /// context and a high-level task description.
    async fn analyze_for_anti_patterns(
        &self,
        context: AnalysisContext,
        task_prompt: &str
    ) -> Result<Vec<AntiPatternFinding>, Self::Error>;
}


This design establishes a clear contract. The AntiPatternFinding struct enforces a consistent, structured output for all findings, including a crucial uncertainty_score. The AnalysisContext encapsulates the rich, multi-modal context provided by the Graph-RAG system. The async_trait allows the engine to perform I/O-bound operations, such as calling LLM APIs, asynchronously.

4.2. Leveraging the Rust LLM Ecosystem

The Rust ecosystem for LLM integration is mature and offers several high-quality client libraries. The choice of client is critical for implementing the flexible, multi-provider architecture recommended in this report.
Table 3: Evaluation of Rust LLM Client Crates
Crate
Supported Backends
Key Features
Maturity/Popularity
Recommendation for Uveddi
llm 49
OpenAI, Anthropic, Ollama, Google, Groq, etc.
Unified traits, multi-step chains, structured output (JSON Schema), parallel evaluation, agentic features.
High (actively developed, comprehensive feature set).
Strongly Recommended. Provides the most complete and flexible feature set required for the proposed hybrid, multi-agent architecture.
simple-llm-client 50
Perplexity, OpenAI
Streaming, markdown formatting, citation handling.
Medium (more focused scope).
Suitable for simpler applications or prototypes focused on a limited set of providers.
rusty_ollama 51
Ollama only
Streaming, context management, sync/async interfaces.
Medium (specific to Ollama).
A good choice only if Uveddi will exclusively use local Ollama models, limiting its capabilities.
openai-api-rs 52
OpenAI only
Unofficial but comprehensive client for the OpenAI API.
High (popular for OpenAI-specific projects).
Too restrictive; locks the architecture into a single cloud provider.

Recommendation: The llm crate is the clear choice for Uveddi. Its multi-backend support through a unified API is essential for implementing the recommended hybrid local/cloud strategy. Furthermore, its built-in support for structured output via JSON Schema, multi-step chains, and parallel evaluation directly maps to the advanced architectural patterns discussed in Sections II and III, significantly reducing implementation complexity.49

4.3. Balancing Local (Ollama) and Cloud LLMs

A production-grade system should not be exclusively reliant on either local or cloud-based LLMs. A hybrid strategy provides the best balance of performance, cost, privacy, and reasoning power.
Local LLMs (via Ollama):
Advantages: Unparalleled data privacy as source code never leaves the user's environment; zero per-call API costs; extremely low latency for smaller, quantized models.
Disadvantages: Requires powerful end-user hardware (CPU, RAM, and ideally a GPU); smaller open-weight models, while rapidly improving, still lag behind the largest proprietary models in complex reasoning tasks.
Use Cases in Uveddi: Ideal for "fast path" or high-volume tasks: generating embeddings, performing initial code chunk summarization, running simple CoT analyses for well-defined anti-patterns, or translating natural language queries into graph queries.
Cloud LLMs (OpenAI, Anthropic, Google):
Advantages: Access to state-of-the-art models (e.g., GPT-4o, Claude 3.5 Sonnet) with superior reasoning, abstraction, and instruction-following capabilities; no hardware burden on the user.
Disadvantages: Significant per-call API costs; potential data privacy concerns (mitigated by providers' data policies, but still a consideration); network latency.
Use Cases in Uveddi: Reserved for "slow path" or high-value tasks requiring deep reasoning: performing ToT analysis to compare refactoring strategies, acting as the high-fidelity Critic agent in the verification stage, or tackling novel and highly ambiguous architectural problems.
The llm crate's provider-agnostic design makes implementing this hybrid approach straightforward. The system can be configured to route different tasks to different providers based on predefined rules, cost-benefit analysis, or user settings.

4.4. Context and Memory Management for Large Codebases

Even with the increasing context windows of modern LLMs, feeding an entire multi-million-line codebase into a prompt is impossible. Therefore, intelligent context and memory management are paramount.
RAG as the Primary Context Manager: The Graph-RAG system described in Section I is the primary mechanism for context management. Its purpose is to distill a massive codebase down to a minimal, yet highly relevant, AnalysisContext for each specific task.
Token Optimization Techniques:
Summarization Pre-pass: For a very large and complex file that must be included in the context, a "pre-pass" can be performed where a fast, local LLM is used to summarize the file's purpose and public API. This much smaller summary is then included in the main context instead of the full file content.
Sliding Window for Agentic History: In multi-agent systems involving conversational loops (like self-correction), the history of interactions can quickly consume the context window. A sliding window approach should be implemented to retain only the most recent k messages, ensuring the context remains relevant without overflowing.53 Rust's
VecDeque data structure is highly efficient for implementing this.
Accurate Token Counting: Before sending any request to an LLM, the exact number of tokens must be calculated to avoid API errors. Using a fast, local tokenizer library written in Rust, such as tiktoken-rs (for OpenAI models) or the core tokenizers crate from Hugging Face, is essential for this pre-flight check.52
Leveraging Rust's Memory Management: Rust's unique ownership and borrowing system is a significant advantage when processing the large data structures involved in code analysis, such as ASTs and dependency graphs.54
Zero-Cost Abstractions: Utilize Rust's powerful iterators and closures to process large collections of code chunks or graph nodes without incurring runtime performance penalties.57
Smart Pointers for Shared Data: When multiple threads need to access the same large, read-only data structure (like the central code graph or a configuration object), use an Arc<T> (Atomically Reference-Counted pointer). This allows for safe, concurrent read access without expensive data duplication.58 For large, heap-allocated data with a single owner,
Box<T> is the appropriate choice.

4.5. Parallelization and Error Handling

Analyzing a large codebase is often an embarrassingly parallel problem, and Rust provides best-in-class tools for harnessing modern multi-core processors safely.
Data Parallelism with rayon: For CPU-bound tasks like parsing files or running analyses on multiple components simultaneously, the rayon crate is the idiomatic solution. By simply changing a call from .iter() to .par_iter(), it is possible to distribute the workload across all available CPU cores with guaranteed data-race freedom.60 The analysis of a project can be parallelized at the file or component level, with each thread running an instance of the
AIReasoningEngine.
I/O Concurrency with tokio: For I/O-bound tasks, particularly making numerous concurrent API calls to cloud-based LLMs, an asynchronous runtime like tokio is essential. The llm crate and other modern Rust HTTP clients are built on tokio, enabling the system to handle thousands of concurrent requests efficiently without blocking threads.49
Robust Error Handling and Uncertainty:
Transparency and Propagation: The system must never fail silently. All potential errors—from LLM API failures (e.g., rate limits, server errors), to parsing errors, to verification failures—must be explicitly handled. This is achieved by using Rust's Result<T, E> enum throughout the codebase. Define a comprehensive custom error enum for the application that can encapsulate all possible failure modes, providing clear and transparent error reporting.
Graceful Degradation: The architecture should be designed for resilience. If a complex, multi-stage analysis fails (e.g., the Critic agent's API call times out), the system should be able to gracefully degrade. For example, it could fall back to providing the result from the Primary agent's analysis, but with a high uncertainty score and a warning that the result is unverified. If a cloud LLM is unavailable, the system could automatically retry the request with a local Ollama model.
Representing Uncertainty: As defined in the AntiPatternFinding struct, every finding must have an associated uncertainty_score.29 This score is a critical piece of metadata. It can be derived from multiple signals: the confidence score from the LLM itself (if the API provides it), the outcome of the Critic's review (a failed review significantly increases uncertainty), or the failure of a deterministic verification check. Findings with an uncertainty score above a certain threshold should be explicitly flagged in the UI as requiring mandatory human review.
The decision to support both local and cloud LLMs is not merely a configuration choice but a fundamental architectural principle that enables a tiered reasoning system. A system designed only for local models will be constrained by the reasoning ceiling of current open-weight models, limiting its ability to tackle novel or highly complex architectural challenges.51 Conversely, a cloud-only system faces limitations of cost, latency, and data privacy, making it unsuitable for certain users or use cases. A hybrid architecture, abstracted by a crate like
llm, is the most powerful and resilient approach.49 This implies that the
AIReasoningEngine must be more than a simple wrapper around an LLM client; it must incorporate a strategic routing layer. This layer's responsibility is to dynamically select the appropriate LLM backend (local or cloud) and reasoning pattern (e.g., simple generation, CoT, ToT) for a given task, based on its complexity, the user's configuration (e.g., "privacy mode" vs. "maximum quality mode"), and cost-benefit analysis. This transforms the engine from a static processor into an active, runtime decision-maker.

Section V: A Framework for Rigorous Evaluation and Testing

Building a reliable AI reasoning engine requires a testing and evaluation framework as rigorous as the architecture itself. This final section outlines a comprehensive methodology for measuring performance, quantifying and minimizing hallucinations, and developing a high-quality test suite to ensure Uveddi is not only functional but also consistently accurate and trustworthy.

5.1. Measuring and Minimizing Hallucinations

Hallucinations in code analysis are outputs that appear plausible but are factually incorrect, deviate from the user's intent, or are not grounded in the provided source code. A systematic approach to measuring them is crucial.
Establishing a Hallucination Taxonomy: The first step is to define and categorize the types of hallucinations relevant to code analysis. The taxonomy developed for the HALLUCODE benchmark provides an excellent foundation 63:
Factual Contradiction: The analysis makes a claim that is demonstrably false based on the code (e.g., stating a function is called when it is not, misidentifying a variable's type).
Intent Deviation: The analysis correctly describes the code but misinterprets the architectural intent or identifies an anti-pattern that is not relevant to the project's context.
Internal Inconsistency: The analysis contradicts itself, for example, by identifying an issue in one section and then making a recommendation that ignores that same issue.
Advanced Evaluation Frameworks:
LLM-as-a-Judge: This has become a state-of-the-art technique for evaluating generative models. It involves using a powerful, independent LLM (e.g., GPT-4o or Claude 3.5 Sonnet) as an impartial evaluator. The judge LLM is given the source code context, the analysis generated by Uveddi, and a detailed evaluation rubric (a checklist). The rubric prompts the judge to score the output on multiple dimensions, such as Factual Correctness, Relevance, Coherence, and Faithfulness to the source code.64 The G-Eval framework provides a structured method for this, where the judge first generates its evaluation steps based on the rubric before providing a final score, improving consistency.64
Claim Extraction and Verification: This is a highly effective, semi-deterministic method. The process involves:
Using an LLM to parse the natural language analysis output and extract every distinct, verifiable "claim" it makes (e.g., "Class A depends on class B," "This loop makes a network call," "The variable x is never used").
For each extracted claim, the system attempts to verify it against a deterministic source of truth. A claim about dependencies is checked against the code graph. A claim about variable usage is checked with a linter or compiler warning.
The hallucination rate can be quantified as the ratio of unverified claims to the total number of claims made.64

5.2. Developing a High-Value Test Suite

The lack of large, publicly available datasets of codebases labeled with architectural anti-patterns is a significant challenge. Therefore, a core competency of the Uveddi team must be the creation and curation of a robust evaluation test suite.
Curating Real-World Examples: The test suite should begin with a curated set of real-world open-source projects that are known to contain well-documented examples of specific anti-patterns. Projects from sources like the Anti-Pattern-Analysis GitHub repository or academic papers that define patterns like Spaghetti Code, God Object, or Boat Anchor can serve as an initial seed set.65
Programmatic Synthetic Data Generation: To achieve broad coverage and test for a diverse range of anti-patterns, the team must generate synthetic code. This can be done in two ways:
Direct Generation: Use an LLM to programmatically generate code examples that are designed from the ground up to exhibit a specific anti-pattern. The prompt would be an instruction like: "Generate a complete Java class for a UserService that demonstrates the 'God Object' anti-pattern. It should handle user registration, profile updates, password management, sending notification emails, and logging user activity, all within the single class".67
Anti-Pattern Injection: This more controlled technique, used in the creation of the HALLUCODE benchmark, involves taking a clean, well-structured piece of code and using an LLM to intentionally inject a specific flaw.63 For example: "Given this clean
OrderProcessor class, refactor it to introduce a 'Spaghetti Code' anti-pattern by merging several methods and adding complex nested conditional logic." This method provides a clean "before" (ground truth) and "after" (test case) pair, allowing for precise evaluation of the detection engine's accuracy.

5.3. Leveraging Existing Benchmarks

While benchmarks specifically for architectural anti-patterns are rare, several existing benchmarks are invaluable for assessing the underlying capabilities of the reasoning engine in code understanding, generation, and quality assessment.
CodeXGLUE: A comprehensive benchmark suite that includes tasks relevant to anti-pattern detection, such as Defect Detection and Clone Detection.69 Success on these tasks is a strong indicator of the model's ability to identify problematic code.
HumanEval & MBPP: These are the standard benchmarks for evaluating the functional correctness of code generation.40 While not directly related to architecture, a model that performs well on these benchmarks demonstrates a strong fundamental understanding of code logic.
Deep-Bench & DS-1000: These benchmarks focus on data science and deep learning code, but their methodology—providing function-level problems with docstrings and unit tests—is a model for how to construct high-quality evaluation instances.63
Awareness of Benchmark Quality: It is crucial to be aware of the limitations of existing benchmarks. The "The Fault in our Stars" paper provides a critical analysis of popular code generation benchmarks, revealing quality issues like unclear prompts, spelling errors, and a lack of contextual dependencies.71 This reinforces the need for careful curation and the creation of custom, high-quality test cases.
CodeSearchNet: This dataset, containing millions of (comment, code) pairs, is the standard for evaluating the code retrieval component of the RAG system.74

5.4. Performance Metrics: Speed vs. Accuracy Trade-offs

The evaluation framework must capture both the quality of the analysis and the performance of the system, as there is an inherent trade-off between them.
Accuracy Metrics:
Precision, Recall, and F1-Score: These are the standard metrics for detection tasks. For anti-pattern detection:
Precision: Of all the anti-patterns Uveddi identified, what percentage were actual anti-patterns? (TP/(TP+FP)
)
Recall: Of all the actual anti-patterns present in the code, what percentage did Uveddi find? (TP/(TP+FN)
)
Pass@k: For tasks where a fix or refactoring is generated, this metric measures the probability that at least one of k generated code solutions is functionally correct (i.e., compiles and passes unit tests).
LLM-as-a-Judge Score: The scores from the evaluation rubric (e.g., faithfulness, relevance) provide a nuanced measure of quality beyond simple correctness.
Performance Metrics:
End-to-End Latency: The wall-clock time from submitting an analysis request to receiving the final, verified results.
Throughput (RPS): The number of analyses the system can process per second, which is critical for scaling.
The core architectural tension lies in the trade-off between these metrics. The multi-stage, multi-agent, and self-correcting architectures that deliver the highest accuracy and lowest hallucination rates (high precision and recall) will invariably have the highest latency and computational cost. The evaluation framework must therefore measure these metrics across a spectrum of configurations—from a fast, single-pass analysis to a deep, multi-iteration architectural audit. This data will empower the Uveddi team to make informed product decisions, potentially offering different tiers of analysis to their users (e.g., a "Quick Scan" optimized for speed versus a "Deep Architectural Review" optimized for thoroughness).
The landscape of code analysis benchmarks reveals a significant gap: while numerous benchmarks exist for code generation (HumanEval, MBPP) and general code intelligence (CodeXGLUE), there is a clear absence of a large-scale, standardized benchmark specifically for architectural anti-pattern detection.69 The closest analogues are defect detection datasets or repositories of known anti-pattern examples, which are often small-scale or not systematically labeled.65 This absence of a standard "ruler" makes it exceedingly difficult to objectively compare different analysis approaches and to measure progress in the field. This situation leads to a critical conclusion for the Uveddi project: the team cannot simply download an off-the-shelf benchmark to validate their system. Instead, a core engineering priority and strategic investment must be the programmatic generation and curation of a proprietary, high-quality evaluation suite. Drawing on methodologies from synthetic data generation 67 and the error-injection techniques used to create the HALLUCODE benchmark 63, the team must build a robust pipeline for generating a diverse and challenging set of codebases containing a wide array of labeled anti-patterns. This internal benchmark will become a key strategic asset, enabling rigorous testing, driving model improvements, and ultimately serving as the definitive measure of Uveddi's accuracy and reliability.

Conclusions and Recommendations

This report has detailed a comprehensive technical blueprint for building Uveddi's AI Reasoning Engine, a system designed for high-fidelity architectural analysis with robust hallucination mitigation. The analysis leads to a set of core architectural recommendations and implementation strategies.

1. Recommended Architecture: A Multi-Stage, Graph-Aware System

The optimal architecture for Uveddi is a multi-stage pipeline that moves beyond simplistic text-based analysis and embraces the structural complexity of code.
Foundation: Graph-Based RAG: The system must begin by parsing the entire codebase into a knowledge graph using tree-sitter. This graph, representing code entities as nodes and their relationships as edges, serves as the ground truth for all subsequent operations. Retrieval should be a hybrid process, combining graph traversal, sparse keyword search (BM25), and dense vector search to construct a rich, architecturally-aware context for each analysis task. For the underlying vector database, Qdrant is recommended for its performance, scalability, and advanced filtering capabilities, which are essential for querying the metadata associated with the code graph.
Reasoning Core: Strategic and Structured: The reasoning engine should employ a tiered approach to prompting. Simple, well-defined analyses can use Chain-of-Thought (CoT) prompting for efficiency and transparency. Complex architectural evaluations that require exploring trade-offs must use Tree-of-Thoughts (ToT) to simulate strategic decision-making. All outputs must be constrained to a rigorously defined JSON Schema using modern LLM APIs that guarantee schema adherence. This is a non-negotiable requirement for reliability and programmatic integration.
Verification Layer: Adversarial and Deterministic: Trust is built through verification. A Primary-Critic multi-agent architecture should be implemented, where a Primary agent generates the initial analysis and a skeptical Critic agent reviews it for factual errors and logical fallacies. This AI-based review must be augmented with a deterministic grounding loop that fact-checks all verifiable claims against the code graph and external tools (e.g., compilers, linters). Findings that fail verification trigger a self-correction loop, where the Primary agent refines its output based on the specific feedback.

2. Implementation in Rust: Key Choices

The Rust ecosystem is well-equipped to build this system.
AIReasoningEngine Trait: A central, async trait should be defined to abstract the reasoning logic, allowing for modular and interchangeable engine implementations.
LLM Client: The llm crate is the recommended choice due to its unified support for multiple backends (enabling a hybrid local/cloud strategy), structured output capabilities, and agentic features.
Parallelism: Use rayon for CPU-bound parallel tasks like analyzing multiple files and tokio for I/O-bound concurrency like making simultaneous calls to cloud LLM APIs.
Memory and Context: Leverage Rust's ownership model and smart pointers (Arc<T>) for efficient, safe management of large data structures like the code graph. Implement context management strategies like sliding windows for agent history and use Rust-native tokenizers like tiktoken-rs for accurate pre-flight token counting.
Error Handling and Uncertainty: Use Rust's Result<T, E> for transparent error propagation. Every analysis finding must include an uncertainty score, derived from the verification process, to clearly communicate the system's confidence to the end-user.

3. Workflow: From Deterministic Analysis to LLM Reasoning

The overall workflow should prioritize deterministic analysis first, only escalating to more expensive LLM-based reasoning when necessary.
Initial Parsing: The process begins by parsing the entire codebase into the knowledge graph and generating vector embeddings. This is a one-time (or incremental) deterministic step.
Task Decomposition: A user query is received by an Analyst Agent (which can be a lightweight, local LLM) that breaks the task down and identifies the target code areas.
Context Retrieval: The Graph-RAG system retrieves the relevant architectural subgraph and code chunks for the specific task.
Primary Analysis: The Primary Agent (potentially a powerful cloud LLM) receives the context and prompt, performs the analysis using the appropriate reasoning pattern (CoT or ToT), and generates a structured JSON output.
Verification and Correction: The output is passed to the Verification Layer. Deterministic checks are run against the code graph. A Critic Agent reviews the logic. If errors are found, the process loops back to the Primary Agent with corrective feedback.
Final Output: Once the analysis passes verification, the structured, uncertainty-scored findings are presented to the user.

4. Final Strategic Imperative: Build the Benchmark

The most significant challenge and opportunity for the Uveddi project is the lack of a standardized benchmark for architectural anti-pattern detection. A core strategic priority must be the development of a proprietary, high-quality evaluation suite. This involves both curating real-world examples and, crucially, building a pipeline to programmatically generate synthetic code with injected anti-patterns. This internal benchmark will be the single most important asset for measuring progress, comparing different architectural approaches, and proving the reliability and superiority of the Uveddi tool.
By adopting this comprehensive, multi-layered approach, Uveddi can move beyond the limitations of current-generation AI code tools and deliver a truly robust, reliable, and insightful architectural analysis engine.
Works cited
Augmenting Large Language Models with Static Code Analysis for Automated Code Quality Improvements - arXiv, accessed June 27, 2025, https://arxiv.org/html/2506.10330v1
Enhancing RAG Applications with Hybrid Search | by Sukalp Tripathi - Medium, accessed June 27, 2025, https://sukalp.medium.com/enhancing-rag-applications-with-hybrid-search-8baf6b582062
7 Chunking Strategies in RAG You Need To Know - F22 Labs, accessed June 27, 2025, https://www.f22labs.com/blogs/7-chunking-strategies-in-rag-you-need-to-know/
Chunking strategies for RAG applications - Amazon Bedrock Recipes - GitHub Pages, accessed June 27, 2025, https://aws-samples.github.io/amazon-bedrock-samples/rag/open-source/chunking/rag_chunking_strategies_langchain_bedrock/
Enhancing LLM Code Generation with RAG and AST-Based Chunking | by VXRL - Medium, accessed June 27, 2025, https://vxrl.medium.com/enhancing-llm-code-generation-with-rag-and-ast-based-chunking-5b81902ae9fc
vitali87/code-graph-rag: Search Monorepos and get ... - GitHub, accessed June 27, 2025, https://github.com/vitali87/code-graph-rag
Graph RAG vs Vector RAG: A Comprehensive Tutorial with Code Examples, accessed June 27, 2025, https://ragaboutit.com/graph-rag-vs-vector-rag-a-comprehensive-tutorial-with-code-examples/
arxiv.org, accessed June 27, 2025, https://arxiv.org/html/2504.10046v1
cAST: Enhancing Code Retrieval-Augmented Generation with Structural Chunking via Abstract Syntax Tree - arXiv, accessed June 27, 2025, https://arxiv.org/html/2506.15655
AST Snowball Splitter - GitHub, accessed June 27, 2025, https://github.com/ilanaliouchouche/ASTSnowballSplitter
ASTChunk is a Python toolkit for code chunking using Abstract Syntax Trees (ASTs), designed to create structurally sound and meaningful code segments. - GitHub, accessed June 27, 2025, https://github.com/yilinjz/astchunk
arxiv.org, accessed June 27, 2025, https://arxiv.org/html/2106.10918v5
Code Refactoring Using Graphs - Graphlytic, accessed June 27, 2025, https://graphlytic.com/blog/code-refactoring-using-graphs
Hybrid Search a method to Optimize RAG implementation | by Akash Chandrasekar, accessed June 27, 2025, https://medium.com/@csakash03/hybrid-search-is-a-method-to-optimize-rag-implementation-98d9d0911341
Which Vector Database Should You Use? Choosing the Best One for Your Needs | by Plaban Nayak | The AI Forum | Medium, accessed June 27, 2025, https://medium.com/the-ai-forum/which-vector-database-should-you-use-choosing-the-best-one-for-your-needs-5108ec7ba133
Vector Database Benchmarks - Qdrant, accessed June 27, 2025, https://qdrant.tech/benchmarks/
sahomedb - Rust - Docs.rs, accessed June 27, 2025, https://docs.rs/sahomedb
Top Vector Databases for Rust in 2025 - Slashdot, accessed June 27, 2025, https://slashdot.org/software/vector-databases/for-rust-language/
LanceDB vs Qdrant - by Sergei Petrov - Medium, accessed June 27, 2025, https://medium.com/@plaggy/lancedb-vs-qdrant-caf01c89965a
The 7 Best Vector Databases in 2025 - DataCamp, accessed June 27, 2025, https://www.datacamp.com/blog/the-top-5-vector-databases
How to write good prompts for generating code from LLMs : r/PromptEngineering - Reddit, accessed June 27, 2025, https://www.reddit.com/r/PromptEngineering/comments/1jqitv9/how_to_write_good_prompts_for_generating_code/
How to write good prompts for generating code from LLMs - GitHub, accessed June 27, 2025, https://github.com/potpie-ai/potpie/wiki/How-to-write-good-prompts-for-generating-code-from-LLMs
Creating Prompt Templates: Standardizing Annotation Instructions for LLMs - Labelvisor, accessed June 27, 2025, https://www.labelvisor.com/creating-prompt-templates-standardizing-annotation-instructions-for-llms/
Prompt Engineering for Architects: Making AI Speak Architecture | by Dave Patten | Medium, accessed June 27, 2025, https://medium.com/@dave-patten/prompt-engineering-for-architects-making-ai-speak-architecture-d812648cf755
What is Prompt Engineering? - AI Prompt Engineering Explained - AWS, accessed June 27, 2025, https://aws.amazon.com/what-is/prompt-engineering/
Prompt Engineering in Code Generation: Creating AI-Assisted Solutions for Developers, accessed June 27, 2025, https://hyqoo.com/artificial-intelligence/prompt-engineering-in-code-generation-creating-ai-assisted-solutions-for-developers
What is chain of thought (CoT) prompting? - IBM, accessed June 27, 2025, https://www.ibm.com/think/topics/chain-of-thoughts
Tree of Thoughts (ToT) - Prompt Engineering Guide, accessed June 27, 2025, https://www.promptingguide.ai/techniques/tot
[2503.15341] Uncertainty-Guided Chain-of-Thought for Code Generation with LLMs - arXiv, accessed June 27, 2025, https://arxiv.org/abs/2503.15341
How JSON Schema Works for LLM Tools & Structured Outputs - PromptLayer, accessed June 27, 2025, https://blog.promptlayer.com/how-json-schema-works-for-structured-outputs-and-tool-integration/
Miscellaneous Examples - JSON Schema, accessed June 27, 2025, https://json-schema.org/learn/miscellaneous-examples
JSON Schema examples, accessed June 27, 2025, https://json-schema.org/learn/json-schema-examples
JsonSchema.Net Basics - json-everything, accessed June 27, 2025, https://docs.json-everything.net/schema/basics/
How to Validate Your JSON Using JSON Schema | by Sivan Biham | TDS Archive - Medium, accessed June 27, 2025, https://medium.com/towards-data-science/how-to-validate-your-json-using-json-schema-f55f4b162dce
Opis JSON Schema: A simple form validation example? : r/PHPhelp - Reddit, accessed June 27, 2025, https://www.reddit.com/r/PHPhelp/comments/1aoi89v/opis_json_schema_a_simple_form_validation_example/
fge/sample-json-schemas - GitHub, accessed June 27, 2025, https://github.com/fge/sample-json-schemas
Structured Outputs - OpenAI API, accessed June 27, 2025, https://platform.openai.com/docs/guides/structured-outputs
LLM Output Formats: Why JSON Costs More Than TSV | by David ..., accessed June 27, 2025, https://david-gilbertson.medium.com/llm-output-formats-why-json-costs-more-than-tsv-ebaf590bd541
Is it time to stop requesting YAML from GPT? | Sophia Willows, accessed June 27, 2025, https://sophiabits.com/blog/is-it-time-to-stop-requesting-yaml-from-gpt
CodeCriticBench: A Holistic Code Critique Benchmark for Large Language Models - arXiv, accessed June 27, 2025, https://arxiv.org/html/2502.16614v1
Self-correcting Code Generation Using Multi-Step Agent ..., accessed June 27, 2025, https://deepsense.ai/resource/self-correcting-code-generation-using-multi-step-agent/
Enhancing LLM Code Generation: A Systematic Evaluation of Multi-Agent Collaboration and Runtime Debugging for Improved Accuracy, Reliability, and Latency - arXiv, accessed June 27, 2025, https://arxiv.org/html/2505.02133v1
System Design: Multi-Agent LLM Legal Analysis System (Coding) - Oleh Dubetcky - Medium, accessed June 27, 2025, https://oleg-dubetcky.medium.com/system-design-multi-agent-llm-legal-analysis-system-coding-18fcb0c8d4bf
LLM Powered Autonomous Agents | Lil'Log, accessed June 27, 2025, https://lilianweng.github.io/posts/2023-06-23-agent/
LLM Multi-Agent Architecture: How AI Teams Work Together | SaM Solutions, accessed June 27, 2025, https://sam-solutions.com/blog/llm-multi-agent-architecture/
LLM-Based Multi-Agent Systems for Software Engineering: Vision and the Road Ahead, accessed June 27, 2025, https://arxiv.org/html/2404.04834v1
Self-Correction in Large Language Models - Communications of the ACM, accessed June 27, 2025, https://cacm.acm.org/news/self-correction-in-large-language-models/
LLM-Powered Code Review: Top Benefits & Key Advantages - Medium, accessed June 27, 2025, https://medium.com/@API4AI/llm-powered-code-review-top-benefits-key-advantages-6feb5f887592
llm - crates.io: Rust Package Registry, accessed June 27, 2025, https://crates.io/crates/llm
simple-llm-client - crates.io: Rust Package Registry, accessed June 27, 2025, https://crates.io/crates/simple-llm-client
rusty_ollama — Rust HTTP client // Lib.rs, accessed June 27, 2025, https://lib.rs/crates/rusty_ollama
Machine learning — list of Rust libraries/crates // Lib.rs, accessed June 27, 2025, https://lib.rs/science/ml
Strategies and Techniques for Managing the Size of the Context Window When Using LLM (Large Language Models) - Mohammed Al Salboukh, accessed June 27, 2025, https://mohdmus99.medium.com/strategies-and-techniques-for-managing-the-size-of-the-context-window-when-using-llm-large-3c2dbc5dcc3a
Understanding Memory Management in Rust: A Comparative Insight with C++ and Java/Kotlin | by Cicero Hellmann | Medium, accessed June 27, 2025, https://medium.com/@cicerohellmann/understanding-memory-management-in-rust-a-comparative-insight-with-c-and-java-kotlin-0b2102020ae7
Mastering Rust Memory Management: The Ultimate Guide for 2024 - Rapid Innovation, accessed June 27, 2025, https://www.rapidinnovation.io/post/rusts-memory-management-and-ownership-model
Memory management in Rust Programming | by Sarathraj - Medium, accessed June 27, 2025, https://medium.com/@sarathraj2008/memory-management-in-rust-programming-d2fc8f9ae96e
Ultimate Rust Performance Optimization Guide 2024: Basics to Advanced - Rapid Innovation, accessed June 27, 2025, https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust
Rust Patterns for Lifetime Management - It's Your Code!, accessed June 27, 2025, https://sigwait.com/rust-patterns-for-lifetime-management
An experimental study of memory management in Rust programming for big data processing, accessed June 27, 2025, https://open.bu.edu/handle/2144/41789
Build Lightning-Fast Data Processing in Rust: From Single Thread to Parallel Performance, accessed June 27, 2025, https://dev.to/ivansing/build-lightning-fast-data-processing-in-rust-from-single-thread-to-parallel-performance-gd8
Optimization adventures: making a parallel Rust workload even faster with data-oriented design (and other tricks) | Blog | Guillaume Endignoux, accessed June 27, 2025, https://gendignoux.com/blog/2024/12/02/rust-data-oriented-design.html
mini_ollama_client - crates.io: Rust Package Registry, accessed June 27, 2025, https://crates.io/crates/mini_ollama_client
[Literature Review] Exploring and Evaluating Hallucinations in LLM ..., accessed June 27, 2025, https://www.themoonlight.io/en/review/exploring-and-evaluating-hallucinations-in-llm-powered-code-generation
LLM Evaluation Metrics: The Ultimate LLM Evaluation Guide - Confident AI, accessed June 27, 2025, https://www.confident-ai.com/blog/llm-evaluation-metrics-everything-you-need-for-llm-evaluation
DigitalProductInnovationAndDevelopment/Anti-Pattern-Analysis: This project aims to provide tools and methodologies for identifying and addressing common anti-patterns in software development. - GitHub, accessed June 27, 2025, https://github.com/DigitalProductInnovationAndDevelopment/Anti-Pattern-Analysis
How to Detect and Prevent Anti-Patterns in Software Development - Digma AI, accessed June 27, 2025, https://digma.ai/how-to-detect-and-prevent-anti-patterns/
Build an enterprise synthetic data strategy using Amazon Bedrock - AWS, accessed June 27, 2025, https://aws.amazon.com/blogs/machine-learning/build-an-enterprise-synthetic-data-strategy-using-amazon-bedrock/
A Systematic Review of Synthetic Data Generation Techniques Using Generative AI - MDPI, accessed June 27, 2025, https://www.mdpi.com/2079-9292/13/17/3509
CodeXGLUE Dataset - Papers With Code, accessed June 27, 2025, https://paperswithcode.com/dataset/codexglue
Deep-Bench: Deep Learning Benchmark Dataset for Code Generation - arXiv, accessed June 27, 2025, https://arxiv.org/pdf/2502.18726
s2e-lab/Datasets-Quality: An empirical investigation of quality in code benchmark datasets; Accepted at SCAM 2024. - GitHub, accessed June 27, 2025, https://github.com/s2e-lab/Datasets-Quality
The Fault in our Stars: Quality Assessment of Code Generation Benchmarks - S2E Lab, accessed June 27, 2025, https://s2e-lab.github.io/preprints/scam24-benchmarks-preprint.pdf
[2404.10155] The Fault in our Stars: Quality Assessment of Code Generation Benchmarks, accessed June 27, 2025, https://arxiv.org/abs/2404.10155
github/CodeSearchNet: Datasets, tools, and benchmarks for representation learning of code., accessed June 27, 2025, https://github.com/github/CodeSearchNet
