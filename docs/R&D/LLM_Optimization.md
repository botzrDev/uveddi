
Optimizing LLM Context Management for Large-Scale Codebase Analysis


Executive Summary

The analysis of large-scale, real-world enterprise codebases using Large Language Models (LLMs) presents a formidable challenge, primarily constrained by finite context windows, prohibitive API costs, and performance bottlenecks. Naive approaches, such as feeding raw code snippets into an LLM, fail to scale and lack the architectural awareness necessary for meaningful analysis. This report presents a comprehensive strategy and a unified architectural blueprint designed to overcome these limitations, enabling CodeAtlas to perform deep, cost-effective, and scalable architectural analysis on codebases far exceeding the size of any single LLM context window.
The central recommendation of this report is the development of a multi-agent, graph-based Retrieval-Augmented Generation (RAG) system. This architecture treats code not as flat text, but as a rich, multi-modal data source, fusing its deterministic structural properties with its probabilistic semantic meaning. The core components of this strategy are:
Hierarchical Code Summarization: A hybrid algorithm is proposed, combining Abstract Syntax Tree (AST) based analysis for logical implementation details with Code Graph-based analysis for capturing architectural dependencies. This creates multi-level, information-dense summaries that drastically reduce token count while preserving architectural insights.
Structural-Semantic Fusion: Codebases are transformed into a dynamic, multi-layered Knowledge Graph. This graph serves as the "single source of truth," containing both deterministic structural facts (ASTs, call graphs, dependencies) and LLM-generated semantic enrichments (summaries, conceptual links). This grounds the LLM's reasoning in a verifiable data structure.
Code-Centric RAG: A modular RAG framework is specified, purpose-built for code. It employs structure-aware chunking, hybrid search (keyword + vector), and graph-aware relevance scoring to retrieve the most pertinent context. Post-retrieval refinement and corrective mechanisms ensure the context provided to the generator is maximally dense and accurate.
Multi-Agent Orchestration: For repository-wide analysis, a multi-agent architecture is proposed. An orchestrator agent decomposes complex analysis tasks and assigns them to specialized worker agents, each responsible for a specific module or function. These agents collaborate, sharing context via the central knowledge graph, thereby maintaining architectural coherence across module boundaries in a scalable, distributed manner.
This integrated approach is projected to meet and exceed the project's success criteria. Benchmarking against a naive baseline is expected to demonstrate the ability to analyze codebases at least 10 times larger than the LLM's context window, achieve a token consumption reduction of over 60%, and maintain or improve the quality of architectural insights. The report provides a detailed cost-performance analysis, a framework for measuring information density, and a phased implementation roadmap, offering CodeAtlas a clear and data-driven path to building a next-generation, production-ready code analysis platform.

Part I: Foundational Strategies for Context Compression and Representation

The primary obstacle to analyzing large codebases with LLMs is the sheer volume of text, which far exceeds the context window limitations of even the most advanced models.1 Directly feeding raw code is untenable due to both context size and exorbitant API costs. Therefore, the foundational step in any scalable solution is context compression: the transformation of vast amounts of code into compact, information-rich representations that can be efficiently processed by an LLM. This part of the report details the two pillars of this foundation. Section 1 explores hierarchical summarization techniques, the primary mechanism for creating multi-level abstractions of code. Section 2 presents a catalog of token optimization strategies, which provide a complementary, multi-layered approach to reducing token usage at every stage of the LLM interaction pipeline.

Section 1: Hierarchical Summarization for Architectural Insight

Hierarchical summarization is the process of creating multi-level abstractions of code at varying granularities, from individual functions to entire repositories. This is not merely about shortening code; it is about intelligently abstracting implementation details while preserving the essential architectural and business logic. Research has converged on two principal methodologies for achieving this: one rooted in the syntactic structure of code as defined by Abstract Syntax Trees (ASTs), and another rooted in the dependency structure as captured by code graphs. Each offers unique advantages, and a state-of-the-art system must leverage the strengths of both.

1.1 Abstract Syntax Tree (AST) Based Summarization: Capturing Implementation Logic

This approach leverages the formal grammatical structure of a programming language to decompose code into its constituent logical units. It excels at understanding the implementation details and logical containment within a single file or class.
Core Methodology
The process begins with parsing source code files using a language-specific parser to generate an Abstract Syntax Tree (AST).3 An AST is a tree representation of the code's syntactic structure, where each node corresponds to a construct in the code, such as a class declaration, a function definition, or a variable assignment.4 This structured representation allows a system to programmatically identify and isolate logical segments like functions, classes, constructors, variables, and enums.4
Once these segments are identified, a Large Language Model (LLM) is used to summarize each one individually. This is a critical step where naive prompting falls short. To generate high-quality, useful summaries, the LLM is guided by customized, structured prompts. For example, a prompt for a function might explicitly ask for its purpose, inputs, outputs, workflow, and side effects.7 Similarly, prompts for variables would focus on their role and scope within the application.6 This structured prompting ensures the LLM extracts the most relevant information for each type of code construct.7
Hierarchical Aggregation
The power of this method lies in its bottom-up hierarchical aggregation. Individual segment-level summaries serve as the building blocks for higher levels of abstraction. These granular summaries are first combined to generate a file-level summary, which describes the file's overall purpose and its role within the repository. Subsequently, these file-level summaries are aggregated to create package-level, and ultimately, repository-level summaries.6 This systematic aggregation ensures comprehensive coverage, addressing a common failure of other methods where significant objects or functionalities within large files are omitted from summaries.4 Each level of the hierarchy builds upon the one below it, creating a rich, multi-layered, and complete representation of the codebase.4
Business Context Grounding
A key innovation, particularly relevant for enterprise applications, is the concept of "grounding" summaries in business context.8 Standard code summarization often focuses on
what the code does technically, but for business applications, understanding why the code exists is paramount. This is achieved by enriching the LLM prompts with domain-specific and problem-specific context.4
For instance, when summarizing a codebase for a telecommunications Business Support System (BSS), the prompts are infused with knowledge about the telecom domain, its operational environment, and the specific business goals the software aims to achieve.6 This grounding enables the LLM to generate summaries that align with domain-specific language and concepts, capturing not just the technical implementation but also the code's higher-level business intent.6 Research from TCS has shown that this technique significantly enhances the quality of summaries for enterprise systems, improving domain relevance by over 7% and completeness by 13% compared to generic prompts.6 This makes the summaries far more valuable for tasks like developer onboarding, maintenance, and architectural review in a corporate setting.7 The use of local LLMs in this approach also addresses enterprise concerns about privacy and data security.7

1.2 Code Graph-Based Summarization: Capturing Architectural Relationships

While AST-based methods excel at understanding the contents of individual files, they can struggle to capture the intricate web of dependencies that defines a system's architecture. Code graph-based summarization directly addresses this by modeling the relationships between code elements across the entire repository.
Core Methodology
This approach, exemplified by the Hierarchical Code Graph Summarization (HCGS) system, begins by constructing a code graph of the entire project.9 In this graph, nodes represent code elements (files, classes, functions), and directed edges represent dependencies such as function calls, class inheritance, or module imports.9 This graph provides a holistic, structural map of the codebase.
A significant advantage of the HCGS approach is its language-agnosticism. Instead of relying on a multitude of language-specific parsers, it leverages the Language Server Protocol (LSP).9 The LSP provides a standardized interface for obtaining deep structural and semantic information from language-specific servers, which are maintained by language experts. This allows HCGS to robustly analyze heterogeneous codebases containing multiple programming languages by simply invoking the appropriate language server for each file.10
Contextual Propagation and Information Density
The fundamental limitation of traditional code analysis, including AST-based summarization, is that it often considers functions in isolation. This misses the critical context of how a function is used and what other functions it relies on.9 HCGS overcomes this by propagating information upward through the call graph.
The summarization process is bottom-up: the summaries of low-level, dependency-free functions are generated first. Then, when summarizing a higher-level function, the prompt provided to the LLM includes not only the source code of the function itself but also the previously generated summaries of all the functions it calls (its "children" in the dependency graph).9 This contextual propagation ensures that higher-level summaries are informed by the detailed behavior of their constituent components. The result is a set of embeddings and summaries with significantly higher information density and semantic accuracy, as they capture a function's true role within the broader system architecture.9
Parallelized Implementation for Scalability
Analyzing the dependency graph of a large codebase can be computationally intensive, especially with the presence of cyclic dependencies. To address this, HCGS employs a highly efficient parallel level-based algorithm.9 The dependency graph is first organized into levels, where each level contains functions that only depend on functions in lower levels. This transformation, which includes a mechanism for deterministically breaking cycles, allows all nodes within a single level to be processed independently and concurrently. A thread pool processes all functions in a given level in parallel, and the system moves to the next level only after the current one is complete. This approach dramatically improves performance and scalability, making it feasible to summarize massive codebases in a reasonable timeframe.9

1.3 Synthesis and Hybrid Algorithm Specification

Neither the AST-based nor the graph-based approach is sufficient on its own for comprehensive architectural analysis. An AST-based approach captures the logical containment structure of code (e.g., a method is part of a class), which is vital for object-oriented analysis and understanding how a component is built. A graph-based approach captures the functional dependency network (e.g., a service calls a utility function), which is key to understanding data flow, runtime behavior, and the impact of changes. A truly powerful system must understand both.
Therefore, a hybrid algorithm is proposed, combining the strengths of both methodologies to create summaries that are rich in both implementation detail and architectural context.
Algorithm Specification: Hybrid Hierarchical Summarization
Phase 1: Multi-Modal Graph Construction.
Dependency Graph: Use an LSP-based tool, similar to HCGS, to build a global dependency graph for the entire codebase. This graph will capture call graphs, class inheritance hierarchies, and module import relationships across all supported languages.
Containment Graph (AST-based): For each file node in the dependency graph, use a language-specific AST parser to identify its internal logical segments (classes, functions, interfaces, etc.). Represent these as a "containment" subgraph linked to the file node. This creates a dual-graph representation of the codebase.
Phase 2: Bottom-Up, Context-Aware Summarization.
Traversal Order: Traverse the global dependency graph in a bottom-up, level-by-level fashion, as implemented in HCGS, to ensure all dependencies are summarized before the components that rely on them.
Hybrid Prompt Generation: For each node (e.g., a function F) being summarized, construct a rich prompt for the LLM that includes:
Target Source Code: The full source code of function F.
Internal Segment Summaries: If F is a class method, include the AST-derived summaries of its containing class's other methods and properties.
Dependency Summaries: Include the structured summaries of all functions that F directly calls, retrieved from the cache of previously generated summaries.
Business Context: Inject user-provided domain and problem context to ground the summary in business intent, as pioneered by the TCS Research approach.6
Phase 3: Hierarchical Aggregation.
Aggregate the detailed function and class summaries into file-level summaries.
Aggregate file-level summaries into package-level and repository-level summaries. At each stage of aggregation, the business context prompt should be re-applied to ensure the high-level summaries focus on the overall purpose and role of the component within the business application.
This hybrid approach ensures that the resulting summaries are maximally informative, capturing the "how" from the AST, the "what it depends on" from the graph, and the "why" from the business context. This provides a robust foundation for the advanced RAG and analysis systems detailed later in this report.
Technique
Core Principle
Granularity
Primary Strength
Primary Weakness
Best For
Key Research
AST-Based Summarization
Decompose code via syntactic structure (logical containment).
Segment, File, Package
High-fidelity representation of implementation logic; excellent for business context grounding.
Blind to cross-file dependencies and architectural flow.
Understanding the purpose and construction of individual components.
TCS Research 4
Graph-Based Summarization (HCGS)
Model codebase via dependency structure (functional relationships).
Function, Module, Repository
Captures architectural dependencies and call chains; language-agnostic via LSP.
Can miss fine-grained implementation details within a function.
Analyzing system-wide data flow, dependencies, and impact.
HCGS 9
Full Code Summarization
Use the entire code unit (file or module) as input.
File, Module
Provides complete context for smaller files, yielding high-quality summaries.
Does not scale; token limits are quickly exceeded; performs poorly for modules.
Summarizing small, self-contained files.
Sun et al. 11
Proposed Hybrid Approach
Fuse dependency graph traversal with AST-based segmentation and business context.
All Levels
Combines architectural awareness (graph) with implementation detail (AST) and business intent.
Higher initial processing complexity and cost.
Comprehensive, multi-faceted architectural analysis.
Synthesis of 6

Table 1: Comparison of Hierarchical Summarization Techniques. This table provides a clear, at-a-glance comparison of the trade-offs between the primary summarization methodologies, justifying the proposed hybrid approach.

1.4 Evaluating Summarization Strategies

Recent research has systematically studied different summarization strategies for higher-level code units.11 For file-level summarization, providing the full code to the LLM (if it fits within the context window) is the most effective approach. However, using a "reduced code" version (e.g., omitting method bodies) serves as a cost-efficient alternative. For module-level summarization, which involves multiple files, the hierarchical approach of summarizing individual files first and then summarizing those summaries becomes the most promising strategy, outperforming attempts to feed the entire module's code at once.11 These findings validate the core principle of the hierarchical approach proposed here: breaking down large artifacts and summarizing them in stages is essential for both quality and scalability.

Section 2: A Multi-Layered Approach to Token Optimization

While hierarchical summarization is the primary strategy for managing context size, it is complemented by a suite of token optimization techniques. Achieving the target of a >60% reduction in token consumption requires a holistic, "defense-in-depth" strategy that applies optimizations at every stage of the analysis pipeline: before the prompt is sent (input-side), within the model architecture itself (model-side), and after the response is generated (output-side). This section provides a catalog of these techniques. A particularly powerful finding is the direct link between source code quality and token consumption; cleaner, less complex code is cheaper to analyze, creating a new, quantifiable dimension of technical debt.12

2.1 Input-Side Optimization: Reducing the Prompt Footprint

These techniques focus on reducing the number of tokens sent to the LLM in the first place, which has a direct impact on both cost and latency.
Concise and Structured Prompting: The most direct method is to refine the prompt itself. Instead of verbose, open-ended questions, prompts should be concise and request only the essential information.14 A highly effective technique is to demand structured output, such as JSON. This forces the LLM to be precise and avoid conversational filler. For code analysis, this is ideal, as prompts can specify a schema with required fields like
purpose, inputs, outputs, workflow, and side_effects, ensuring a dense and predictable response format.7 Using one-shot examples within the prompt (
Structured Prompt+1S) can further guide the model to produce the desired format and level of detail.7
Code-Level Preprocessing: The content of the code itself can be optimized before being included in the prompt.
Automated Refactoring: Research has demonstrated a strong correlation between code "smells" (e.g., excessive complexity, long methods, deep nesting) and the token consumption required for an LLM to reason about that code.12 Code with high cyclomatic or Halstead complexity requires significantly more tokens to process.13 Therefore, a preprocessing step that automatically refactors smelly code to reduce its complexity can yield substantial token savings without altering its functionality. This reframing of technical debt into a direct, measurable API cost provides a powerful incentive for improving code quality.
Selective Pruning (Reduced Code): For certain high-level analysis tasks, not all code details are necessary. A "reduced code" view can be created by programmatically removing non-essential elements like comments, logging statements, or even entire method bodies, leaving only signatures and class structures.11 This approach offers a cost-efficient alternative to full-code analysis, significantly reducing token count with a manageable trade-off in summary quality.11
Consolidating Reference Data: When a task requires analyzing a large document (or code file) multiple times for different properties, it is inefficient to send the entire document in each prompt. A more efficient workflow is to use a powerful model once to extract all required information into a structured format like JSON. Subsequent, simpler tasks can then be performed by cheaper models that only receive the compact JSON object as input, drastically reducing token usage.16

2.2 Model-Side and Architectural Optimization

These strategies involve architectural choices about the models themselves and how they are used in the system.
Dynamic Model Selection: Not all tasks require the most powerful (and expensive) LLM. A key optimization is to build a workflow that routes tasks to different models based on their complexity.16 Simple, repetitive tasks like generating boilerplate code or summarizing a single, clean function can be handled by smaller, faster, and cheaper models. Complex, ambiguous tasks requiring deep reasoning, such as planning a multi-file refactoring, should be routed to state-of-the-art models like GPT-4 or Claude 3.5 Sonnet.15 This dynamic routing ensures that computational resources are used judiciously.
Knowledge Distillation: This technique involves training a smaller, more efficient "student" model to replicate the output distribution (logits) of a larger, more capable "teacher" model.17 The resulting student model can perform specific tasks with far fewer parameters and tokens, yet it retains a significant portion of the teacher's nuanced performance. This is an effective strategy for creating specialized, cost-effective models for high-volume tasks like code summarization.
Sequence Compression Models: For tasks that unavoidably require very long contexts, specialized model architectures can be employed. Models like Longformer and BigBird use sparse attention mechanisms, where each token only attends to a subset of other tokens, rather than the entire sequence.17 This changes the computational complexity of attention from quadratic to near-linear with respect to sequence length, enabling the model to handle much longer documents more efficiently. Hierarchical Attention Networks (HANs) achieve a similar effect by grouping tokens into sentences and sentences into paragraphs, operating on compressed representations at each level.17

2.3 Output-Side and Post-Processing Optimization

These techniques focus on controlling and refining the LLM's generated output to minimize waste.
Limiting max_tokens: A simple yet crucial safeguard is to set the max_tokens parameter in the API call.14 This provides a predictable upper bound on the cost of each generation and prevents the model from producing excessively long or runaway responses, which is a common failure mode.
Post-processing and Filtering: The raw output from an LLM can often be verbose. A post-processing step can be applied to programmatically filter out extraneous words, remove conversational pleasantries, and condense the output into a more streamlined format.14 This ensures that the final stored or displayed result is as token-efficient as possible.
Advanced Token Filtering: Emerging research explores more advanced techniques like token filtering, where tokens that are unlikely to contribute meaningfully to the final output are identified and eliminated during the generation process itself.18 These methods can be categorized into
forward filtering (removing tokens during the forward pass) and backward filtering (removing tokens only during the backward pass in training), both of which aim to improve efficiency and utility by focusing computation on the most important tokens.18
The following table provides a catalog of these techniques, categorizing them by their application stage and estimating their potential impact.
Technique
Stage
Description
Est. Token Reduction
Comp. Overhead
Key Research
Concise Prompting
Input
Refining prompts to be direct and specific, removing filler words.
5-15%
Low
14
Structured JSON Output
Input/Output
Forcing the LLM to generate output in a strict JSON schema, eliminating verbosity.
20-40%
Low
7
Code Refactoring
Input
Pre-processing code to remove "smells" and reduce complexity before analysis.
15-30%
Medium
12
Reduced Code Pruning
Input
Removing non-essential code parts (e.g., method bodies, comments) for high-level analysis.
40-75%
Low
11
Dynamic Model Selection
Model
Routing tasks to the most cost-effective model based on complexity.
30-80%
Medium
15
Knowledge Distillation
Model
Training a smaller "student" model to mimic a larger "teacher" model for specific tasks.
50-90%
High (Training)
17
max_tokens Limiting
Output
Setting a hard limit on the number of generated tokens to prevent runaway responses.
Variable
None
14
Post-processing Filters
Output
Programmatically filtering and condensing the LLM's raw text output.
5-10%
Low
14

Table 2: Catalog of Token Optimization Techniques. This catalog provides a practical, actionable summary of available token optimization methods, allowing developers to select the most appropriate techniques for their specific use case.

Part II: Architecting for Advanced Code Intelligence

Having established foundational strategies for compressing code and optimizing token usage, this report now turns to the core system architecture. A truly intelligent code analysis platform must move beyond treating code as simple text. It requires an architecture that can deeply understand and fuse the two fundamental modalities of code: its deterministic, logical structure and its high-level, probabilistic semantic meaning. This part details the design of such an architecture. Section 3 presents the structural-semantic fusion paradigm, which transforms code into a rich, multi-modal knowledge graph. Section 4 then specifies how to build a Retrieval-Augmented Generation (RAG) system that is purpose-built to leverage this sophisticated representation for advanced code analysis.

Section 3: The Structural-Semantic Fusion Architecture

The central architectural thesis of this report is that code should be treated as multi-modal data. Traditional static analysis tools excel at understanding code's structure but fail to grasp its semantic intent. Conversely, LLMs excel at semantic understanding but lack the precision and logical rigor to reliably parse complex structures. A fusion architecture combines the strengths of both, creating a system that is more powerful, scalable, and verifiable than either approach in isolation. This is achieved by first transforming the codebase into a structured, queryable data source and then using an LLM to enrich and interact with that source.

3.1 The "Code-as-Data" Paradigm

This paradigm, exemplified by systems like CodeFuse-Query, reimagines static code analysis not as a text-processing task, but as a data computation task.19 Instead of feeding unstructured code files to a model, the entire codebase is first parsed and transformed into a structured database of "code facts."
Architecture of CodeFuse-Query
The CodeFuse-Query system provides a powerful blueprint for this approach.19 Its architecture is designed for massive scale, reportedly scanning over 10 billion lines of code daily across more than 300 different analysis tasks.19
COREF Schema: The core of the system is a standardized, two-tiered data model called COREF (Code Representation Framework). Source code from various languages is parsed into this schema, which captures multiple structural views of the code, including the Abstract Syntax Tree (AST), Abstract Semantic Graph (ASG), Control Flow Graph (CFG), Program Dependency Graph (PDG), Call Graph, and Class Hierarchy.21 This creates a rich, structured database representing the code's ground truth.
Declarative Querying with Datalog: To analyze this database, CodeFuse-Query employs a high-level, declarative language called Gödel, which is based on Datalog.19 Datalog is a logic-based query language well-suited for complex, recursive queries on graph-like data. This allows analysts to formulate sophisticated queries—such as finding all transitive dependencies of a function or performing taint analysis to trace data flow—that are extremely difficult or impossible to express reliably in a natural language prompt to an LLM.19 The declarative nature of the language means users describe
what they want, and the underlying engine optimizes the execution plan.21
Scalable, Service-Oriented Infrastructure: The system is built on a decoupled, service-oriented architecture with four distinct layers: an access layer for handling requests, a coordinator for managing tasks, a worker layer with stateless extractor and analysis nodes, and a distributed storage layer.19 This design enables elastic scaling, resource isolation, and incremental analysis, which are all critical for handling enterprise-scale codebases.19

3.2 The "Code-as-Graph" Paradigm and the Semantic Layer

The "Code-as-Data" approach provides the structural foundation. The next step is to fuse this with semantic understanding, which is best achieved by conceptualizing the codebase as a rich knowledge graph (KG).23
Multi-Modal Fusion
This fusion is a direct application of multi-modal learning, a field that combines information from different data types (e.g., text, image, audio) to achieve a more comprehensive understanding.25 In our context, the code's structure (the graph of dependencies, ASTs, etc.) is one modality, while its textual content and the semantic meaning embedded within it (e.g., in function names, comments, and the logic itself) is the second modality.
An effective fusion strategy does not simply treat these modalities independently. Instead, it uses one to enrich the other. For example, the FEAMDA framework for malware detection combines low-level structural patterns from bytecode images with high-level behavioral features from API call sequences by "textualizing" both into a structured prompt for an LLM.26 This principle can be applied directly to code analysis: a path through the dependency graph (a structural feature) can be described in text and combined with the semantic summaries of the nodes along that path to provide a rich, multi-modal context to the LLM.
The Semantic Layer
A crucial component in this fusion is the semantic layer.27 This layer acts as a "universal translator" between the raw, technical data structures of the code and the high-level business concepts an LLM needs to reason about.27 While an LLM might not understand the specific implementation of a function named
calculate_final_revenue, a semantic layer provides the explicit business context: "In our company, 'final revenue' is defined as gross sales minus discounts and returns."
This layer bridges the semantic gap between the code and the business domain. It ensures that when an LLM analyzes the code, it does so with a precise understanding of what key business terms like "active user" or "net revenue" actually mean within that specific codebase.27 This is indispensable for generating analysis that is not just technically correct but also aligned with business objectives, rectifying a key weakness of LLMs that may otherwise interpret terms in a general, context-free sense.27 The combination of a powerful LLM with a precise semantic layer creates a high-context analyst capable of delivering reliable insights.27

3.3 Proposed Fusion Architecture Design

To realize the full potential of structural-semantic fusion, a unified multi-modal architecture is proposed. This design integrates the "Code-as-Data" and "Code-as-Graph" paradigms into a cohesive system that is robust, verifiable, and semantically rich.
Architectural Blueprint
Data Layer: The Structural-Semantic Knowledge Graph.
At the foundation is a graph database (e.g., Neo4j, FalkorDB) that stores the entire codebase as a comprehensive knowledge graph.
Structural Backbone: The initial graph is built using deterministic static analysis, populating it with nodes (files, classes, functions) and edges representing the COREF schema: AST relationships, call graphs, dependency graphs, and class hierarchies.21 This forms the immutable, verifiable structural backbone of the system.
Semantic Enrichment: An LLM is then used to enrich this graph. Using the hierarchical summarization technique from Section 1, it generates a structured semantic summary for each node in the graph. These summaries are stored as node properties. Furthermore, the LLM can be used to infer and create new semantic edges between nodes, such as is_similar_to (connecting two functions with similar logic but different implementations) or implements_concept (linking a class to a business concept from the semantic layer). This creates a rich, multi-layered graph containing both deterministic structural links and probabilistic semantic links.
Query Layer: The Dual-Query Engine.
To interact with this hybrid graph, a dual-query engine is required.
Structural Query Engine: A Datalog or graph query (e.g., Cypher, GQL) engine provides precise, logical access to the structural backbone.19 This is used for tasks requiring deterministic analysis, such as "Find all functions that transitively depend on
lib_v1.so."
Semantic Query Engine: A vector search index is built over the semantic summaries stored on the graph nodes. This engine handles natural language, semantic queries, such as "Find code related to user authentication."
Fusion in RAG (Retrieval-Augmented Generation).
The RAG system, which serves as the primary interface for the LLM, utilizes this dual-query engine. When a user poses a natural language question, a query-planning LLM (as detailed in Section 4) first analyzes the query to determine its nature.28
If the query is primarily structural ("What functions does AuthService contain?"), it is routed to the structural query engine.
If the query is primarily semantic ("Show me examples of data validation logic"), it is routed to the semantic query engine.
For complex queries ("Which services would be impacted by a security vulnerability in our data validation logic?"), the planner can orchestrate a multi-step query that uses both engines: first a semantic search to identify "data validation logic," then a structural query to trace the dependencies of those functions.
The results from these queries—a combination of code snippets, summaries, and graph paths—are then fused into a final, highly-contextual prompt for the generator LLM.
This architecture creates a system that can reason with different levels of precision and abstraction. The structural data provides deterministic "ground truth," which serves to ground the LLM and mitigate its tendency for hallucination. The semantic data provides the high-level understanding and conceptual links that are missing from traditional static analysis. This fusion allows the system to answer a much broader and more sophisticated range of questions about a codebase, moving beyond simple code search to true architectural comprehension. It represents a shift from "LLMs for code analysis" to "a structured code analysis engine with an LLM-based natural language interface," a far more robust, scalable, and defensible technical approach.

Section 4: A Code-Centric RAG Implementation Strategy

With a rich structural-semantic knowledge graph as the foundation, the next step is to build a Retrieval-Augmented Generation (RAG) system capable of effectively leveraging it. A generic RAG system designed for plain text will fail to capture the nuances of code. A state-of-the-art, code-centric RAG system must be purpose-built, with specialized components for each stage of the pipeline: pre-retrieval (indexing), retrieval (searching), and post-retrieval (refining). The most advanced systems evolve beyond a static pipeline into a modular, agentic reasoning loop that dynamically adapts its strategy to the user's query.

4.1 Pre-Retrieval: Structure-Aware Indexing and Chunking

The quality of retrieval is fundamentally limited by the quality of the index. For code, this means the way code is chunked and indexed must respect its inherent structure.
The Failure of Naive Chunking
Simple chunking strategies, such as splitting text into fixed-size segments of N tokens or lines, are highly detrimental to code analysis.28 Code is not a flat sequence of words; it has a rigid syntactic and semantic structure. Naive chunking will inevitably break functions, classes, or logical blocks in arbitrary places, destroying the very context the LLM needs to understand the code's purpose. A chunk containing the end of one function and the beginning of another is effectively useless.
Structure-Aware Chunking with cAST
To address this, a structure-aware chunking method is essential. The cAST (Chunking via Abstract Syntax Trees) algorithm offers a robust solution.30 Instead of operating on raw text,
cAST traverses the Abstract Syntax Tree of the code. Its goal is to create chunks that align with complete syntactic units. The algorithm works as follows:
Top-Down Traversal: It recursively traverses the AST, attempting to fit entire nodes (like a function definition or a class block) into a single chunk, as long as they fit within a predefined token budget.
Greedy Merging: If a node is too large and must be split, the algorithm descends to its children. It then performs a greedy merging step, combining adjacent smaller sibling nodes (e.g., multiple consecutive statements) into a single chunk to maximize information density and avoid creating an excessive number of tiny, low-context chunks.30
This AST-guided approach ensures that every chunk is a syntactically coherent, self-contained unit. Experiments have shown that this method significantly improves RAG performance, boosting retrieval recall and code generation pass rates compared to line-based chunking.30
Hierarchical Indexing
In addition to better chunking, the index itself should be hierarchical.28 This mirrors the summarization strategy from Section 1. The system should maintain multiple indices at different levels of abstraction:
Level 1 (Repository/Package): An index of the high-level summaries of packages and modules.
Level 2 (File/Class): An index of file and class summaries.
Level 3 (Function/Chunk): An index of the detailed function summaries and the cAST-generated code chunks.
A query can first be run against the highest-level index to identify relevant modules. The search can then be narrowed to the indices for those specific modules, creating a highly efficient, "zoom-in" retrieval process.

4.2 Retrieval: Adapting Relevance for Code

The retrieval stage is responsible for finding the most relevant information in the index to answer a user's query. For code, this requires a multi-faceted approach to search and relevance.
Hybrid Search
The most effective strategy is a hybrid search that combines the strengths of keyword-based (sparse) search and semantic (dense) vector search.28
Sparse Search (e.g., BM25): This is excellent at finding exact matches for specific identifiers, such as function names (getUser), variable names, or library-specific keywords. This is critical for code, where precise names matter.
Dense Search (Vector Search): This excels at finding semantically similar concepts, even if the wording is different. For example, it can match the query "logic for validating user credentials" to a function named authenticate_user_password.
By combining the results of both search types, the system can satisfy queries that depend on both specific technical terms and broader semantic intent.
Code-Specific Relevance Scoring
Standard relevance scoring is based on textual similarity. For code, relevance is more complex. The retrieval system's scoring function must be adapted to prioritize architecturally significant components. This can be achieved by augmenting the standard similarity score with boosts from several sources:
Graph Proximity: In the structural-semantic knowledge graph, code snippets that are "closer" to already-retrieved context (e.g., directly called by, or contained within, another relevant function) should have their relevance scores boosted.
Structural Similarity: Code fragments with similar AST structures can be considered relevant, even if their text is different. This can help find analogous implementations of a design pattern.
Code-Specific Metrics: The system can boost the scores of code snippets that are identified as architecturally significant. For example, a function with a high degree of centrality in the call graph (i.e., it is called by many other parts of the system) is likely more important than a leaf-node helper function.
Business Concept Alignment: Code that implements a core business concept, as identified by the semantic layer, should be prioritized.
Advanced Retrieval Frameworks: Query Transformation and CodeRAG
The most advanced RAG systems use an LLM to actively participate in the retrieval process itself.
Query Routing and Transformation: An LLM can act as a "decider" or "router" that first analyzes the user's query to determine the best retrieval strategy.28 It can rewrite the query for better performance or even generate a hypothetical document or code snippet (a technique known as HyDE, or Hypothetical Document Embeddings) and search for embeddings that match the generated example rather than the original query.28
The CodeRAG Framework: The CodeRAG framework represents a state-of-the-art approach specifically for repo-level coding.32 Instead of just searching for textually similar code, it operates on a "bigraph" composed of a
requirement graph and a code graph. It first retrieves nodes with similar functional requirements (mined from docstrings or generated by an LLM). These requirement nodes are then mapped to their corresponding code nodes in the code graph. These code nodes serve as initial "anchors" for a deeper, agentic reasoning process that traverses the code graph to find all necessary supportive code. This approach, which retrieves based on functional and dependency similarity, has been shown to dramatically improve code generation accuracy, achieving a 40.9% increase in Pass@1 on the DevEval benchmark compared to no RAG.32

4.3 Post-Retrieval: Refining Context for the Generator

The output of the retrieval stage is a list of potentially relevant code chunks. Simply concatenating these and feeding them to the generator LLM is suboptimal. A post-retrieval refinement stage is crucial for maximizing the quality and density of the final prompt.
Reranking: The initial list of retrieved documents should be passed to a dedicated reranker model.28 These models are specifically trained to take a query and a list of documents and re-order them with higher precision than the initial retrieval search. This ensures the most relevant snippets appear at the top of the context.
Contextual Compression: After reranking, it is often the case that the retrieved context still contains redundant or less important information. A "compressor" LLM can be used to read through the retrieved snippets and synthesize a condensed version that removes this noise before it is passed to the final generator LLM.28
Corrective RAG (CRAG): CRAG introduces a self-reflection step into the pipeline.31 It uses a lightweight retrieval evaluator to score the relevance of the retrieved documents against the query. If the overall relevance score is below a certain threshold, it indicates a poor retrieval. In this case, the system can trigger a corrective action, such as performing a web search for additional information or using a different retrieval strategy, to augment or replace the initial context before proceeding to the generation phase.31
The combination of these techniques transforms RAG from a simple retrieve-then-generate pipeline into a sophisticated, multi-stage, and adaptive system. The architecture is modular, allowing for different strategies to be composed and dynamically selected by a query planner. This approach is far more robust and powerful for the complex domain of code analysis.
Stage
Component
Recommended Implementation
Key Function
Supporting Research
Pre-Retrieval
Chunking
cAST (Structure-Aware AST Chunking)
Divides code into syntactically coherent, self-contained units, preserving logical structure.
30


Indexing
Hybrid Hierarchical Index
Creates multi-level indices (summaries, chunks) for efficient, "zoom-in" retrieval.
28
Retrieval
Query Planner
LLM-based Router
Analyzes user query to select the optimal retrieval strategy (vector, graph, keyword).
28


Retriever
Hybrid Search + Graph Traversal
Combines sparse (keyword), dense (semantic), and graph-based retrieval for comprehensive results.
28
Post-Retrieval
Reranker
Dedicated Reranker Model
Precisely re-orders retrieved snippets based on relevance to the specific query.
28


Refinement
Corrective RAG (CRAG)
Evaluates retrieved context quality and triggers corrective actions (e.g., web search) if low.
31

Table 3: Code-Specific RAG Architecture Components. This table provides a clear blueprint of a modular, code-specific RAG system, breaking it down into its constituent components and recommended implementations.

Part III: Enabling Dynamic and Scalable Repository-Wide Analysis

The strategies outlined in Parts I and II provide the building blocks for analyzing isolated code components or small sets of files. However, true architectural analysis requires understanding the entire codebase as a cohesive, evolving system. This presents two major challenges: managing context incrementally as an analysis task progresses, and maintaining architectural coherence across thousands of files and module boundaries. The solution lies in moving away from a static, single-LLM paradigm towards a dynamic, agentic architecture that mimics how a human engineering team collaborates to understand and modify a large system.

Section 5: Incremental and Cross-Module Context Management

It is computationally infeasible and prohibitively expensive to load the context of an entire enterprise codebase into an LLM at once. Therefore, context must be built and managed dynamically and incrementally.

5.1 Progressive and Incremental Context Building

The core principle of incremental context building is to start with a high-level, low-token view of the system and load more detailed context on demand, a process akin to a developer "zooming in" on a specific area of interest.33
Techniques for Dynamic Context Management
Several techniques enable this dynamic behavior:
Sliding Window with Summarization: A common approach for managing conversational context that is also applicable to code analysis. The system maintains a "sliding window" of the most recent interactions or analyzed code files in the active context.34 As the analysis proceeds, older or less relevant context is pushed out of the window. However, instead of being discarded, it is summarized by an LLM and stored in an external memory (such as a vector database or the knowledge graph). This summary can be retrieved later if the analysis needs to refer back to that context, providing a balance between a focused active context and long-term memory.34
Semi-Automated Context Curation: Modern AI-powered IDEs and tools like Cursor and Aider provide mechanisms for developers to manually or semi-automatically manage the context.36 A developer can explicitly add a set of relevant files to the chat context. More advanced systems can automate this by performing a preliminary search for relevant files based on the user's initial prompt and suggesting them for inclusion.36 This gives the user fine-grained control over what the LLM "sees."
The llms.txt Standard: An emerging proposal to standardize initial context gathering is the llms.txt file.38 Similar in spirit to
robots.txt or sitemap.xml, this is a markdown file placed in the root of a repository. It acts as a "cheat sheet" for an LLM, providing a high-level project summary and direct links to crucial documentation, architectural diagrams, and key source files. This allows an automated system to quickly ingest the most important context for a codebase without having to guess or scan the entire repository, greatly improving the quality of initial interactions.39
Automated Context Updating and the Virtuous Cycle: The most sophisticated pattern involves creating a feedback loop where the LLM updates its own context.38 When a developer confirms that a piece of code generated by the LLM is correct, or that an analysis provided by the LLM is insightful, they can prompt the model to "update its context" with this new, validated information. For example, the developer could say, "That new
AuthService component is correct. Please update your llms.txt file with a link to this new file and a summary of its purpose." This creates a virtuous cycle where the system's knowledge base becomes progressively more accurate and complete over time. The DocAgent system formalizes this concept into a multi-agent framework that uses a topological sort of the code's dependency graph to ensure that components are documented only after their dependencies are, enabling a truly incremental and coherent context-building process.40

5.2 Maintaining Architectural Coherence with Agentic Frameworks

A single LLM, even with a large context window, cannot maintain a coherent mental model of a million-line codebase. The problem is one of scale and complexity. The solution is to distribute the cognitive load, moving from a single-LLM architecture to a Multi-Agent Architecture (MAA).41 This approach is the digital analogue of a human software engineering team.
Multi-Agent Architecture (MAA) for Code Analysis
In an MAA, a complex task is decomposed and handled by multiple, specialized AI agents that collaborate to achieve a common goal.42
Core Concept: An Orchestrator Agent receives a high-level analysis query from the user (e.g., "Assess the performance implications of migrating our database"). It then breaks this complex query down into a series of smaller, manageable subtasks.41 These subtasks are then dispatched to a team of specialized
Worker Agents.
Agent Specialization: For codebase analysis, a team of agents could be designed with specific roles:
ModuleAgent: An expert on a specific module or service in the codebase. It holds the deep, local context for that component, including its source code, summaries, and dependencies.
DependencyAgent: Responsible for understanding and resolving cross-module dependencies. When a ModuleAgent needs to understand how another module works, it queries the DependencyAgent, which in turn communicates with the relevant ModuleAgent for that other module.
SecurityAgent: A specialist agent fine-tuned and prompted to identify security vulnerabilities, such as SQL injection or insecure API patterns.
DocAgent: An agent specialized in reading existing documentation and generating new documentation, as seen in the DocAgent framework.40
TestAgent: An agent responsible for generating, executing, and interpreting unit and integration tests.
Communication and Coordination: The agents collaborate by exchanging messages through well-defined communication protocols, much like developers interacting via APIs or messaging systems.42 The Orchestrator agent manages the overall workflow, synthesizes the results from the worker agents, and can even include conflict resolution mechanisms (e.g., voting or confidence scoring) if different agents produce conflicting information.42 Frameworks like AutoGen, LangChain, and CrewAI provide the tools to build and manage these complex agentic workflows.42
This multi-agent approach provides inherent scalability, modularity, and robustness. New agents with new specializations can be added without disrupting the system, and the analysis of the codebase can be parallelized across many agents, each focusing on a small, manageable piece of the puzzle.42

5.3 Long-Term Memory for Architectural Context

For agents to collaborate effectively and maintain coherence over long-running analysis sessions, they require access to a shared long-term memory (LTM) that persists beyond any single interaction.41
Architectures for Long-Term Memory
The Knowledge Graph as LTM: The structural-semantic knowledge graph, as designed in Section 3, serves as the ideal LTM for this system. It is a persistent, structured, and queryable representation of the entire codebase's architecture and semantics.23 Agents can read from and write to this graph, allowing them to share a common understanding of the system's state.
Memory-Augmented Architectures: These architectures formalize the process of interacting with LTM. They typically consist of a memory store and a retrieval network.48 When an agent needs information, the retrieval network queries the memory store (the knowledge graph) to pull the most relevant past interactions, analysis results, or code summaries into the agent's active context. A key feature of advanced systems is
relevance-based pruning. Instead of simple Least Recently Used (LRU) eviction, these systems use semantic relevance to decide which memories to keep in the active context, ensuring that critical but infrequently accessed information is not lost.48
Cache-Augmented Generation (CAG): For parts of the context that are frequently accessed but relatively static (e.g., the API definition of a stable core library), the Cache-Augmented Generation (CAG) technique offers a significant performance boost.50 In this approach, the Key-Value (KV) pairs from the LLM's attention mechanism, which represent the model's processed understanding of that context, are pre-computed and stored in a cache. During subsequent inferences, instead of re-processing the raw text, the model can load the pre-computed KV cache directly. This dramatically reduces latency and computational cost by eliminating redundant processing of stable context.50
By combining a multi-agent framework with a robust long-term memory system built upon a knowledge graph, it becomes possible to analyze a massive codebase in a way that is both scalable and coherent. This architecture transforms the problem from an impossible task of fitting an entire codebase into a single model's memory into a manageable, distributed problem of coordinating specialized agents that share a common, persistent understanding of the system.

Part IV: Evaluation Framework and Implementation Roadmap

The final part of this report provides the necessary tools and plans to translate the proposed architecture into a successful production system. Section 6 introduces a framework for quantitatively measuring the performance of different context management strategies, focusing on the crucial trade-off between token count and information quality. Section 7 synthesizes all preceding concepts into a single, unified architecture and presents a cost-performance analysis against a baseline. Finally, Section 8 lays out a practical, phased implementation roadmap and a set of best practices for development and deployment.

Section 6: A Framework for Measuring Information Density and Performance

To make informed decisions about which context management strategies to implement, a robust evaluation framework is required. This framework must go beyond simple metrics like token count and instead quantify the quality and density of the information being processed. The central challenge is to measure the trade-off between the cost of context (token count) and the value of the analysis it enables.

6.1 Quantifying the Information Density vs. Token Count Trade-off

A raw token count is a poor proxy for the value of a piece of context. A 100-token summary that omits a critical architectural dependency is far less valuable than a 200-token summary that captures it. The goal is to maximize information density: the amount of meaningful, task-relevant information conveyed per token.52
Defining and Measuring Information in Code
Information in source code is multi-faceted, containing both semantic meaning and structural complexity. Therefore, a composite metric is needed to capture its density.
Semantic Content: The semantic information in a summary or code snippet can be quantified by decomposing it into a set of unique, verifiable "claims" or "propositions." Techniques from Natural Language Inference (NLI) can then be used to evaluate these claims. An NLI model can check for entailment (the claim is fully supported by the source code), contradiction (the claim conflicts with the source), or neutrality.53 A high-quality summary should have a high number of entailed claims and zero contradictions. Frameworks like InfoLossQA provide a structured way to characterize information loss in summarization by generating question-answer pairs that highlight deleted or oversimplified information.54
Structural Content: The architectural significance of a piece of code can be approximated using well-established software metrics that correlate with complexity and importance.
Cyclomatic Complexity: This metric measures the number of linearly independent paths through a piece of code, providing a quantitative measure of its logical complexity. Higher complexity often indicates a more information-dense and critical component.55
Halstead Complexity Suite: These metrics are derived from the number of distinct operators and operands in the code, providing another view on its complexity.13
Graph-based Metrics: The importance of a code element can be inferred from its position in the dependency graph. Metrics like node degree (number of incoming/outgoing calls) or PageRank centrality can quantify a function's architectural significance.
Information-Theoretic Measures: These provide a more theoretical lens on complexity.
Shannon Entropy: Measures the uncertainty or randomness in a distribution. For code, this can be applied to the distribution of tokens. Code with lower entropy may be more repetitive and less information-dense.56
Kolmogorov Complexity: Theoretically defined as the length of the shortest program that can generate a piece of data. While not directly computable, it can be approximated by compressibility (e.g., using gzip). Less compressible code is, in a sense, more complex and information-dense.56
Proposed Metric: ArchIS (Architectural Information Score)
To combine these dimensions into a single, quantifiable metric for evaluating context management strategies, the Architectural Information Score (ArchIS) is proposed:
ArchIS=TokenCount(w1​×SemanticClaims)+(w2​×StructuralComplexity)​
SemanticClaims: The number of unique, entailed claims extracted from the context.
StructuralComplexity: A normalized score combining metrics like cyclomatic complexity and graph centrality.
TokenCount: The total number of tokens in the context.
w1, w2: Weights that can be tuned based on the specific analysis task. For example, a query about architectural impact might use a higher w2, while a query about functionality might use a higher w1.
This metric provides a way to directly compare different summarization or chunking strategies by measuring the amount of relevant architectural information they pack per token. A key realization is that the definition of "information" is task-dependent. For a junior developer asking about functionality, a semantic explanation is informative. For an architect assessing impact, the dependency graph is informative. Therefore, the evaluation framework itself should be dynamic, adjusting the weights of the ArchIS metric based on the user's inferred intent, creating a more nuanced and meaningful measurement of performance.
Information Type
Measurement Method
Example Metric
Rationale
Supporting Research
Semantic Content
Claim Extraction & NLI
Entailed Claim Count
Measures how much factually correct information from the source is preserved in the summary.
53
Structural Complexity (Logical)
Static Code Analysis
Cyclomatic Complexity
Quantifies the logical branching and path complexity, a proxy for implementation intricacy.
55
Structural Connectivity (Architectural)
Graph Analysis
Node Centrality (PageRank)
Measures the importance of a code element within the overall dependency structure of the codebase.
9
Information-Theoretic Complexity
Statistical Analysis
Shannon Entropy / Compressibility
Measures the randomness/incompressibility of the code, providing a theoretical measure of information content.
56

Table 4: Information Density Measurement Framework. This table provides a clear, actionable framework for evaluating the core trade-off between context size and information quality.

6.2 Establishing a Benchmark Dataset and Protocol

To rigorously test and compare different strategies, a standardized benchmark is essential.
Benchmark Dataset: A new benchmark dataset should be curated, composed of several large, open-source, enterprise-grade codebases (e.g., from the Apache Software Foundation, or projects like Django, Kubernetes). For a subset of these, human experts should create "gold standard" architectural documentation and identify key components and dependencies to serve as ground truth.
Benchmark Tasks: The evaluation should be based on a suite of representative analysis tasks that test different facets of code understanding. This suite should draw from and extend existing state-of-the-art code benchmarks:
SWE-bench: This benchmark consists of real-world software engineering problems from GitHub issues, requiring models to generate patches to resolve them. Success requires a deep understanding of the existing codebase and the ability to make precise, multi-file changes.57
CrossCodeEval: This benchmark specifically evaluates a model's ability to handle cross-file dependencies for code completion. It is an excellent test of cross-module understanding.60
CodeRAG-Bench: This is a holistic benchmark designed specifically for evaluating Retrieval-Augmented Code Generation. It includes a diverse set of tasks (basic programming, open-domain, repository-level) and a large, multi-source datastore for retrieval, making it an ideal framework for our evaluation.63
Evaluation Metrics: A multi-faceted evaluation approach is required.
Task Performance: For code generation or repair tasks, the primary metric is execution-based correctness, measured by Pass@k (the probability that at least one of k generated solutions passes the unit tests).66
Summarization Quality: To evaluate generated summaries, a combination of metrics should be used.
Reference-based metrics like BLEU, ROUGE, and METEOR provide a baseline by comparing n-gram overlap with a human-written reference summary.4
Reference-free, LLM-as-judge metrics like G-Eval are more sophisticated, using a powerful LLM to score summaries on dimensions like coherence, consistency, fluency, and relevance.67
Code-specific metrics like SIDE use contrastive learning to directly measure the semantic alignment between a generated summary and the source code itself, independent of any reference summary, which is a powerful way to address the issue of low-quality or outdated reference comments.72
Retrieval Quality: The performance of the RAG system's retrieval component should be measured using standard information retrieval metrics like nDCG (Normalized Discounted Cumulative Gain), Precision@k, and Recall.66

Section 7: Integrated System Design and Cost-Performance Analysis

This section synthesizes the strategies from the preceding parts into a unified architectural design for the CodeAtlas CDAT-2 system and presents a quantitative analysis of its expected performance and cost compared to a naive baseline.

7.1 The Unified CodeAtlas-CDAT-2 Architecture

The proposed end-to-end architecture integrates all the key components discussed in this report into a cohesive, scalable system.
Ingestion Pipeline: When a new codebase is added, it passes through an automated ingestion pipeline. This pipeline uses LSP and AST parsers to construct the structural-semantic knowledge graph (as detailed in Section 3). Simultaneously, it runs the hybrid hierarchical summarization algorithm (Section 1) to generate multi-level summaries, which are stored as properties on the graph nodes.
Analysis Core (Multi-Agent System): The core of the analysis engine is a multi-agent system (Section 5). A central Orchestrator Agent receives user queries. It decomposes the query into subtasks and dispatches them to a pool of specialized Worker Agents (e.g., ModuleAgent, SecurityAgent).
Context Management (Code-Centric RAG): The Orchestrator and Worker Agents do not hold the full codebase context. Instead, they query the code-centric RAG engine (Section 4) to retrieve necessary context from the knowledge graph on demand. This RAG engine uses structure-aware chunking, hybrid search, and graph-aware relevance scoring to provide maximally dense and relevant context for each subtask.
Memory and State: The knowledge graph serves as the long-term memory for the entire system. Agents use a combination of short-term memory (for the current task) and LTM access to maintain coherence. Stable, frequently used context can be optimized using Cache-Augmented Generation (CAG) to improve performance.
Token Optimization: Token optimization strategies (Section 2) are applied throughout the system: prompts are structured, models are dynamically selected based on task complexity, and output is controlled and filtered.

7.2 Comparative Analysis: Baseline vs. Proposed Architecture

To quantify the benefits of this architecture, a comparative analysis will be conducted against a naive baseline using the benchmark protocol defined in Section 6.
Baseline (Naive Approach): This system represents the simplest possible implementation. Code files are chunked into fixed-size, overlapping segments and embedded into a vector database. A user query triggers a simple vector search, and the top-k retrieved text chunks are concatenated and stuffed into a single LLM prompt for generation.
Proposed Architecture: The full, integrated CDAT-2 system as described above.
The following key performance indicators (KPIs) will be measured for both systems across the benchmark tasks:
Task Success Rate: Measured by Pass@1 on SWE-bench and CrossCodeEval tasks.
Average Token Consumption per Task: The total number of input and output tokens used to complete a task.
API Cost per Task: Calculated based on token consumption and public pricing for the LLM models used.15
End-to-End Latency: The time from query submission to final response generation.
Context Quality: Measured by the ArchIS score of the context provided to the final generator LLM.

7.3 Cost-Performance Analysis

The results of the comparative analysis will be presented to provide a clear, quantitative cost-benefit trade-off. The proposed architecture has a higher upfront computational cost during the initial ingestion and graph-building phase. However, it is expected to yield significantly lower operational costs per query and substantially higher analysis quality. The analysis will demonstrate the achievement of the project's primary success criteria.

Strategy
Avg. Tokens per Query
API Cost per Query ($)
Analysis Quality (Pass@1)
Max Codebase Size
Latency (s)
Naive Snippet Extraction
~200,000
High
Low (< 5%)
< LLM Context Window
High
Hierarchical Summarization Only
~40,000
Medium
Medium (~10%)
~5x Context Window
Medium
RAG with Structural Fusion
~15,000
Low
High (~20-25%)
~10x Context Window
Medium-Low
Full Proposed Architecture
~10,000
Lowest
Highest (> 30%)
> 10x Context Window
Low (with Caching)

Table 5: Cost-Performance Analysis of Context Management Strategies. This table presents the projected outcomes of the comparative analysis, providing a data-driven justification for the development of the proposed architecture. Values are illustrative estimates based on reported gains in the literature.

Section 8: Production Implementation Plan and Recommendations

This final section provides a practical, phased roadmap for developing and deploying the CDAT-2 architecture, along with key best practices and future research directions.

8.1 Phased Implementation Roadmap

A phased approach is recommended to manage complexity, deliver value incrementally, and de-risk the project.
Phase 1: Foundational Data Layer (Months 1-6).
Objective: Build the core data ingestion and representation pipeline.
Key Activities:
Implement the LSP and AST-based parsers for target languages.
Set up the graph database and define the COREF-inspired schema.
Develop and test the hybrid hierarchical summarization algorithm.
Implement the dual-query engine (structural + semantic).
Deliverable: An internal "Smart Code Search" tool that allows engineers to perform advanced structural and semantic queries on codebases.
Phase 2: Advanced RAG and Single-Agent System (Months 7-12).
Objective: Build the specialized RAG engine and a single-agent analysis system.
Key Activities:
Implement the code-centric RAG pipeline, including cAST chunking, hybrid search, and code-specific reranking.
Develop a single-agent application that can use the RAG engine to answer complex questions about a single module or a small, well-defined set of files.
Deliverable: A prototype analysis tool capable of deep analysis on a limited scope (e.g., a single microservice).
Phase 3: Multi-Agent Orchestration and Scalability (Months 13-18).
Objective: Scale the system to handle entire repositories by implementing the multi-agent architecture.
Key Activities:
Develop the Orchestrator Agent and the task decomposition logic.
Implement specialized Worker Agents (ModuleAgent, DependencyAgent, etc.).
Define and implement the communication protocols between agents.
Integrate the multi-agent system with the RAG engine and the knowledge graph LTM.
Deliverable: The full CDAT-2 platform, capable of performing scalable, cross-module architectural analysis.
Phase 4: Continuous Learning and Optimization (Ongoing).
Objective: Enhance the system's intelligence and efficiency over time.
Key Activities:
Implement feedback loops in the UI to capture user corrections and ratings. Use this data for fine-tuning models and improving the knowledge graph.
Develop the "LLM updates its own context" capability.
Explore continual learning techniques to keep the models updated with new programming languages, frameworks, and patterns without full retraining.77

8.2 Best Practices and Recommendations

Data Curation is Paramount: The quality of the entire system depends on the quality of the initial parsing and summarization. Invest in creating high-quality, business-context-aware prompts and validation checks for the ingestion pipeline.
Embrace a Multi-Model Strategy: Avoid vendor lock-in and optimize for cost and performance by building the architecture to support a portfolio of LLMs. Use routing to select the best model for each subtask.15
Rigorous Monitoring and Evaluation: Continuously monitor the system in production using the evaluation framework from Section 6. Track not only system metrics (latency, cost) but also model quality metrics (accuracy, drift, ArchIS) to detect performance degradation over time.
Prioritize Human-in-the-Loop: Design the system with humans in the loop. The platform should be an assistive tool, not a fully autonomous one. Interfaces should allow developers to easily review, correct, and override the AI's analysis, with their feedback captured to improve the system.

8.3 Future Research Directions

Dynamic Knowledge Graph Construction: The current proposal relies on periodic re-indexing. Future work should explore techniques for dynamically updating the knowledge graph in real-time as the codebase evolves, for instance by processing git commits incrementally.79
Automated and Personalized Context Management: Research should investigate systems that can automatically learn user-specific or domain-specific context management patterns, moving beyond generic rules to a personalized analysis experience.39
Truly Multi-Modal Code Analysis: The definition of "modality" can be expanded beyond structure and text. A future system could integrate runtime data (performance profiles, logs), user interaction data (analytics), and even visual data (UI screenshots, design mockups) to build a truly holistic, 360-degree understanding of a software system.
Works cited
LLMs with largest context windows - Codingscape, accessed June 28, 2025, https://codingscape.com/blog/llms-with-largest-context-windows
What is LLM's Context Window?:Understanding and Working with the Context Window | by Tahir | Medium, accessed June 28, 2025, https://medium.com/@tahirbalarabe2/what-is-llms-context-window-understanding-and-working-with-the-context-window-641b6d4f811f
How Abstract Syntax Trees Unlock LLM's Code Understanding | by Danilka AKarawita, accessed June 28, 2025, https://medium.com/@nishandanilka/how-abstract-syntax-trees-unlock-llms-code-understanding-5fa88877123a
[Literature Review] Hierarchical Repository-Level Code Summarization for Business Applications Using Local LLMs - Moonlight | AI Colleague for Research Papers, accessed June 28, 2025, https://www.themoonlight.io/en/review/hierarchical-repository-level-code-summarization-for-business-applications-using-local-llms
Slack Combines ASTs with Large Language Models to Automatically Convert 80% of 15,000 Unit Tests - InfoQ, accessed June 28, 2025, https://www.infoq.com/news/2024/06/slack-automatic-test-conversion/
Towards Smarter Code Comprehension: Hierarchical ..., accessed June 28, 2025, https://www.marktechpost.com/2025/01/25/towards-smarter-code-comprehension-hierarchical-summarization-with-business-relevance/
Hierarchical Repository-Level Code Summarization for Business Applications Using Local LLMs - arXiv, accessed June 28, 2025, https://arxiv.org/html/2501.07857v1
Hierarchical Repository-Level Code Summarization for ... - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2501.07857
Hierarchical Graph-Based Code Summarization for Enhanced Context Retrieval - arXiv, accessed June 28, 2025, https://arxiv.org/html/2504.08975v1
[2504.08975] Code-Craft: Hierarchical Graph-Based Code Summarization for Enhanced Context Retrieval - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2504.08975
Commenting Higher-level Code Unit: Full Code, Reduced Code, or ..., accessed June 28, 2025, https://arxiv.org/abs/2503.10737
Optimizing Token Consumption in LLMs: A Nano Surge Approach for Code Reasoning Efficiency * Corresponding authors - arXiv, accessed June 28, 2025, https://arxiv.org/html/2504.15989v2
[2504.15989] Optimizing Token Consumption in LLMs: A Nano Surge Approach for Code Reasoning Efficiency - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2504.15989
Reducing Unnecessary Token Generation in LLM Responses ..., accessed June 28, 2025, https://prosperasoft.com/blog/artificial-intelligence/reducing-unnecessary-token-generation-in-llm-responses/
Top 6 Strategies to Optimize Token Costs for ChatGPT and LLM APIs - TypingMind Blog, accessed June 28, 2025, https://blog.typingmind.com/optimize-token-costs-for-chatgpt-and-llm-api/
How to Optimise Token Efficiency - V7 Go Resources & Documentation, accessed June 28, 2025, https://docs.go.v7labs.com/docs/how-to-optimise-token-efficiency
Token Efficiency and Compression Techniques in Large Language Models: Navigating Context-Length Limits” | by Arash Nicoomanesh | Medium, accessed June 28, 2025, https://medium.com/@anicomanesh/token-efficiency-and-compression-techniques-in-large-language-models-navigating-context-length-05a61283412b
Enhancing Token Filtering Efficiency in Large Language Model Training with Collider - arXiv, accessed June 28, 2025, https://arxiv.org/html/2502.00340v1
CodeFuse-Query: A Data-Centric Static Code Analysis System for Large-Scale Organizations - arXiv, accessed June 28, 2025, https://arxiv.org/html/2401.01571v1
[2401.01571] CodeFuse-Query: A Data-Centric Static Code Analysis System for Large-Scale Organizations - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2401.01571
codefuse-ai/CodeFuse-Query: Query-Based Code Analysis Engine - GitHub, accessed June 28, 2025, https://github.com/codefuse-ai/CodeFuse-Query
CodeFuse-Query/doc/2_introduction.en.md at main - GitHub, accessed June 28, 2025, https://github.com/codefuse-ai/CodeFuse-Query/blob/main/doc/2_introduction.en.md
Deep Dive into Knowledge Graph Components for LLM RAG Applications — With a Real World Example | by Gaurav Nigam | aingineer | Medium, accessed June 28, 2025, https://medium.com/aingineer/deep-dive-into-knowledge-graph-components-for-llm-rag-applications-with-a-real-world-example-9f2a7c585015
Graph-Based Codebase Management System (GBCMS) - GitHub, accessed June 28, 2025, https://github.com/peytontolbert/GCBMS
Multimodal Deep Learning | Papers With Code, accessed June 28, 2025, https://paperswithcode.com/task/multimodal-deep-learning
FEAMDA: Fusion-based Explainable Android Malware Detection Agent with LLM Support, accessed June 28, 2025, https://figshare.com/articles/dataset/FAMDA_Fusion-based_Android_Malware_Detection_Agent_with_LLM_Support/29146082
How LLMs and Semantic Layers Are Revolutionising Self-Service Analytics - Veezoo, accessed June 28, 2025, https://veezoo.com/blog/llms-and-semantic-layers/
Advanced RAG Techniques: What They Are & How to Use Them, accessed June 28, 2025, https://www.falkordb.com/blog/advanced-rag/
Advanced RAG: Architecture, Techniques, Applications and Use ..., accessed June 28, 2025, https://www.leewayhertz.com/advanced-rag/
cAST: Enhancing Code Retrieval-Augmented Generation ... - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2506.15655
8 Retrieval Augmented Generation (RAG) Architectures You Should Know in 2025, accessed June 28, 2025, https://humanloop.com/blog/rag-architectures
CodeRAG: Supportive Code Retrieval on Bigraph for Real ... - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2504.10046
Generating Code with LLMs: A Developer's Guide — Part 2 | by Mayuresh K - Medium, accessed June 28, 2025, https://mskadu.medium.com/generating-code-with-llms-a-developers-guide-part-1-c274dc0e4eec
LLM Context Windows: Why They Matter and 5 Solutions for Context Limits - Kolena, accessed June 28, 2025, https://www.kolena.com/guides/llm-context-windows-why-they-matter-and-5-solutions-for-context-limits/
The Art of LLM Context Management: Optimizing AI Agents for App Development - Medium, accessed June 28, 2025, https://medium.com/@ravikhurana_38440/the-art-of-llm-context-management-optimizing-ai-agents-for-app-development-e5ef9fcf8f75
Context control for local LLMs: How do you handle coding workflows? - Reddit, accessed June 28, 2025, https://www.reddit.com/r/ChatGPTCoding/comments/1jnkhjw/context_control_for_local_llms_how_do_you_handle/
Large Codebases - Cursor Docs, accessed June 28, 2025, https://docs.cursor.com/guides/advanced/large-codebases
How I Code With LLMs These Days - Honeycomb, accessed June 28, 2025, https://www.honeycomb.io/blog/how-i-code-with-llms-these-days
Context is king: tools for feeding your code and website to LLMs - WorkOS, accessed June 28, 2025, https://workos.com/blog/context-is-king-tools-for-feeding-your-code-and-website-to-llms
DocAgent: A Multi-Agent System for Automated Code Documentation Generation - arXiv, accessed June 28, 2025, https://arxiv.org/html/2504.08725v2
Agentic LLM Architecture: How It Works, Types, Key Applications | SaM Solutions, accessed June 28, 2025, https://sam-solutions.com/blog/llm-agent-architecture/
LLM Multi-Agent Architecture: How AI Teams Work Together | SaM Solutions, accessed June 28, 2025, https://sam-solutions.com/blog/llm-multi-agent-architecture/
Introduction to Multi-Agent Architecture for LLM-Based Applications - Reply, accessed June 28, 2025, https://www.reply.com/aim-reply/en/content/introduction-to-multi-agent-architecture-for-llm-based-applications
Understanding the Architecture of LLM Agents - Ema, accessed June 28, 2025, https://www.ema.co/additional-blogs/addition-blogs/understanding-the-architecture-of-llm-agents
From Human Memory to AI Memory: A Survey on Memory Mechanisms in the Era of LLMs - arXiv, accessed June 28, 2025, https://arxiv.org/html/2504.15965v1
Memory and State in LLM Applications - Arize AI, accessed June 28, 2025, https://arize.com/blog/memory-and-state-in-llm-applications/
Why LLMs Need Better Context? - Memgraph, accessed June 28, 2025, https://memgraph.com/blog/why-llms-need-context
Memory-Augmented Architecture for Long-Term Context Handling in Large Language Models - arXiv, accessed June 28, 2025, https://www.arxiv.org/pdf/2506.18271
Memory-Augmented Architecture for Long-Term Context Handling in Large Language Models - arXiv, accessed June 28, 2025, https://arxiv.org/html/2506.18271v1
Optimizing LLMs with cache augmented generation - IBM Developer, accessed June 28, 2025, https://developer.ibm.com/articles/awb-llms-cache-augmented-generation/
Architectural Strategies for External Knowledge Integration in LLMs: A Comparative Analysis of RAG and CAG - DEV Community, accessed June 28, 2025, https://dev.to/foxgem/architectural-strategies-for-external-knowledge-integration-in-llms-a-comparative-analysis-of-rag-23d6
information density - PKC - Obsidian Publish, accessed June 28, 2025, https://publish.obsidian.md/pkc/Hub/Theory/Sciences/information+density
Quantifying Fairness in LLMs Beyond Tokens: A Semantic and Statistical Perspective - arXiv, accessed June 28, 2025, https://arxiv.org/html/2506.19028v1
InfoLossQA: Characterizing and Recovering Information Loss in Text Simplification - arXiv, accessed June 28, 2025, https://arxiv.org/html/2401.16475v1
Code Complexity: An In-Depth Explanation and Metrics - Codacy | Blog, accessed June 28, 2025, https://blog.codacy.com/code-complexity
The Ultimate Guide to Complexity in Info Theory - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/ultimate-guide-complexity-information-theory
SWE-bench [Multimodal]: Can Language Models Resolve Real-world Github Issues?, accessed June 28, 2025, https://github.com/SWE-bench/SWE-bench
swe-bench: can language models resolve real-world github issues?, accessed June 28, 2025, https://r.jordan.im/download/language-models/jimenez2023.pdf
SWE-bench: Can Language Models Resolve Real-world Github Issues? | OpenReview, accessed June 28, 2025, https://openreview.net/forum?id=VTF8yNQM66
[2310.11248] CrossCodeEval: A Diverse and Multilingual Benchmark for Cross-File Code Completion - ar5iv, accessed June 28, 2025, https://ar5iv.labs.arxiv.org/html/2310.11248
CROSSCODEEVAL: A Diverse and Multilingual Benchmark for Cross-File Code Completion, accessed June 28, 2025, https://proceedings.neurips.cc/paper_files/paper/2023/file/920f2dced7d32ab2ba2f1970bc306af6-Paper-Datasets_and_Benchmarks.pdf
CrossCodeEval: A Diverse and Multilingual Benchmark for Cross-File Code Completion, accessed June 28, 2025, https://crosscodeeval.github.io/
arxiv.org, accessed June 28, 2025, https://arxiv.org/html/2406.14497v1
CodeRAG-Bench: Can Retrieval Augment Code Generation? - arXiv, accessed June 28, 2025, https://arxiv.org/html/2406.14497v2
CodeRAG-Bench: Can Retrieval Augment Code Generation? - ACL Anthology, accessed June 28, 2025, https://aclanthology.org/2025.findings-naacl.176/
CODERAG-BENCH: Can Retrieval Augment Code Generation? - ACL Anthology, accessed June 28, 2025, https://aclanthology.org/2025.findings-naacl.176.pdf
Evaluation Metrics for Summarization - DEV Community, accessed June 28, 2025, https://dev.to/espoir/evaluation-metrics-for-summarization-3amo
Which traditional language generation metrics are applicable for evaluating RAG-generated answers, and what aspect of quality does each (BLEU, ROUGE, METEOR) capture? - Milvus, accessed June 28, 2025, https://milvus.io/ai-quick-reference/which-traditional-language-generation-metrics-are-applicable-for-evaluating-raggenerated-answers-and-what-aspect-of-quality-does-each-bleu-rouge-meteor-capture
Evaluating NLP Models: A Comprehensive Guide to ROUGE, BLEU, METEOR, and BERTScore Metrics - In Plain English, accessed June 28, 2025, https://plainenglish.io/blog/evaluating-nlp-models-a-comprehensive-guide-to-rouge-bleu-meteor-and-bertscore-metrics-d0f1b1
9 (A)- Automatic evaluation metrics (BLEU, ROUGE, METEOR), accessed June 28, 2025, https://corejava25hours.com/2024/06/15/9-a-automatic-evaluation-metrics-bleu-rouge-meteor/
Evaluating the performance of LLM summarization prompts with G-Eval | Microsoft Learn, accessed June 28, 2025, https://learn.microsoft.com/en-us/ai/playbook/technology-guidance/generative-ai/working-with-llms/evaluation/g-eval-metric-for-summarization
antonio-mastropaolo/code-summarization-metric - GitHub, accessed June 28, 2025, https://github.com/antonio-mastropaolo/code-summarization-metric
[2312.15475] Evaluating Code Summarization Techniques: A New Metric and an Empirical Characterization - arXiv, accessed June 28, 2025, https://arxiv.org/abs/2312.15475
Evaluation Metrics for Retrieval-Augmented Generation (RAG) Systems - GeeksforGeeks, accessed June 28, 2025, https://www.geeksforgeeks.org/nlp/evaluation-metrics-for-retrieval-augmented-generation-rag-systems/
Best Practices in RAG Evaluation: A Comprehensive Guide - Qdrant, accessed June 28, 2025, https://qdrant.tech/blog/rag-evaluation-guide/
What do tokens per second ranges in provisioned throughput mean? - Azure Databricks, accessed June 28, 2025, https://learn.microsoft.com/en-us/azure/databricks/machine-learning/foundation-model-apis/prov-throughput-tokens
Continual Learning for Large Language Models: A Survey - arXiv, accessed June 28, 2025, https://arxiv.org/html/2402.01364v1
10 Benefits and 10 Challenges of Applying Large Language Models to DoD Software Acquisition - SEI Blog, accessed June 28, 2025, https://insights.sei.cmu.edu/blog/10-benefits-and-10-challenges-of-applying-large-language-models-to-dod-software-acquisition/
Creating a Dynamic Knowledge Graph Generator with Python and NLP | by Dinesh Ram, accessed June 28, 2025, https://medium.com/@dineshramdsml/creating-a-dynamic-knowledge-graph-generator-with-python-and-nlp-eaf0ca7974b5
getzep/graphiti: Build Real-Time Knowledge Graphs for AI Agents - GitHub, accessed June 28, 2025, https://github.com/getzep/graphiti
Learning Dynamic Context Management in LLMs through Human-in-the-Loop Curation: A Proposed Framework: From Basic Tools to Hierarchical Abstraction | by Micheal Bee | Medium, accessed June 28, 2025, https://medium.com/@mbonsign/learning-dynamic-context-management-in-llms-through-human-in-the-loop-curation-a-proposed-0029a4e9d06e
