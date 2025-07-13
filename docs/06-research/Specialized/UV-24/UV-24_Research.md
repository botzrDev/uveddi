
A Comprehensive Architectural Blueprint for Next-Generation Code Clone Detection (UV-24 Enhancement)


Executive Summary

This report presents a comprehensive architectural blueprint for advancing the UV-24 code clone detection system. The primary strategic objective is to evolve the system from its current, highly capable syntactic analysis foundation to a state-of-the-art platform incorporating deep semantic understanding. The existing system, which leverages a two-stage hybrid of Karp-Rabin hashing and Abstract Syntax Tree (AST) verification, demonstrates high proficiency in detecting Type-1, Type-2, and simple Type-3 clones across Rust, Python, and JavaScript. However, to address the more complex and insidious forms of code duplication that contribute significantly to technical debt and bug propagation, a fundamental architectural enhancement is required.
The proposed architecture introduces a multi-stage pipeline that layers advanced structural and semantic analysis techniques on top of the existing framework. The core enhancements include the integration of Control Flow Graph (CFG) analysis to capture logical structure, a sophisticated machine learning (ML) classification framework for nuanced clone prediction, and cutting-edge semantic detection capabilities to identify functionally equivalent code (Type-4 clones).
Key technical recommendations are central to this strategy. For structural analysis, a tiered approach is proposed, using scalable graph embeddings for initial candidate filtering followed by the more precise Weisfeiler-Lehman (WL) kernel for refined similarity scoring. For semantic analysis, the adoption of GraphCodeBERT, a pre-trained model that incorporates code's structural information (data flow), is recommended over alternatives like CodeBERT, as it provides a fundamentally richer representation of program logic. To ensure the validity and real-world applicability of these enhancements, this report underscores a critical methodological finding: standard public benchmarks like BigCloneBench, while useful for syntactic evaluation, are demonstrably flawed for validating semantic clone detection. Consequently, a key strategic recommendation is the development of a custom, high-quality, manually-validated benchmark dataset to accurately measure the system's performance on Type-4 clones.
The implementation is structured as a phased, 10–14 day roadmap, systematically integrating CFG parsing, ML model fine-tuning, full pipeline assembly, and rigorous benchmarking. The expected outcomes are a significant leap in detection accuracy for complex Type-3 and Type-4 clones, enhanced system adaptability through domain-specific tuning mechanisms, and the positioning of UV-24 as a market-leading, next-generation code analysis solution capable of delivering deep insights into large-scale, complex codebases.

The Evolving Landscape of Code Duplication and Analysis

The practice of code duplication, often through copy-and-paste programming, is a pervasive reality in software development.1 While it can accelerate development in the short term, it is a primary source of technical debt, leading to increased maintenance costs, code bloat, and the propagation of bugs.3 Effective code clone detection is therefore not merely a code quality utility but a critical component of sustainable software engineering. The challenge lies in the fact that not all clones are created equal; they exist on a spectrum of complexity that demands increasingly sophisticated analysis techniques.

A Formal Taxonomy of Code Clones: From Syntax to Semantics

To architect a robust detection system, it is essential to establish a formal classification of clone types. This taxonomy, widely accepted in academic and industry research, defines clones based on their level of similarity, moving from simple textual identity to deep functional equivalence.5 This classification directly informs the required analytical techniques at each level of complexity.
Type-1 (Exact Clones): These are identical code fragments where the only variations are in whitespace, comments, and layout.7 They are the most straightforward to detect and are often the result of direct copy-paste operations. The existing UV-24 system, with its hashing and AST comparison, is highly effective at identifying this type.
Type-2 (Syntactic/Renamed Clones): These clones are syntactically equivalent but have variations in identifiers (variable, function, or class names), literals, and data types, in addition to the differences allowed in Type-1.1 AST-based analysis is well-suited for this type, as it abstracts away specific names and focuses on the code's structural form.9
Type-3 (Near-Miss/Gapped Clones): These are syntactically similar fragments that have been modified further. Statements may have been added, deleted, or changed.7 These "near-miss" clones represent a significant challenge for purely syntactic methods. While AST comparison can tolerate minor changes, substantial modifications can render two fragments structurally dissimilar, even if their core logic remains intact. The concept of a similarity threshold becomes critical for detecting Type-3 clones, defining how much dissimilarity is permissible.1
Type-4 (Semantic Clones): These are code fragments that are functionally equivalent but are implemented with different syntax and potentially different algorithms.1 For example, one function might use a
for loop to iterate over a collection, while another uses a while loop or a recursive approach to achieve the same result. These clones are impossible to detect using purely textual or syntactic analysis, as they require an understanding of the code's behavior and intent. They are the most challenging to identify and are the primary target for the enhancements proposed in this report.
The progression from Type-1 to Type-4 represents a clear gradient of increasing analytical difficulty. While the current UV-24 system is proficient in handling clones defined by their textual and syntactic properties (Types 1, 2, and some 3), the most subtle and often dangerous forms of duplication—those that mask identical logic behind different structures (complex Type-3) or different implementations (Type-4)—remain undetected. These are precisely the clones that can lead to elusive bugs, where a fix in one location is not propagated to its functionally identical but syntactically different counterpart.3 This capability gap is the central motivation for evolving the UV-24 architecture. The need to incorporate CFG and ML analysis is not an arbitrary addition of features; it is a direct and necessary response to the challenge of climbing this ladder of clone complexity, moving the system's capabilities from syntactic pattern matching to true semantic understanding.

Critical Assessment of the Current UV-24 Architecture

The existing UV-24 system is built on a robust and efficient two-stage hybrid architecture designed for multi-language support (Rust, Python, JavaScript) and cross-file analysis.
Stage 1: Karp-Rabin Hashing: This initial stage serves as a high-performance filter. It converts code blocks into hash values, allowing for extremely fast comparisons to identify identical or near-identical fragments. This approach is highly effective at quickly reducing the search space by eliminating the vast majority of non-cloned code, which is essential for scalability in large codebases.
Stage 2: Abstract Syntax Tree (AST) Verification: For candidate pairs that pass the hashing stage, the system generates ASTs. An AST is a tree representation of the code's syntactic structure.9 By comparing these trees, the system can robustly verify Type-1 and Type-2 clones, as this representation is inherently immune to variations in variable names, formatting, and comments.9 It can also detect simple Type-3 clones where the structural differences between the ASTs are minimal and fall below a configurable threshold.
Strengths:
The primary strength of this architecture is its efficiency and accuracy within its defined scope. The combination of a fast hashing filter with a precise syntactic verifier provides excellent performance for detecting clones where the code's structure is largely preserved.10 This makes it a powerful tool for identifying straightforward duplication and enforcing coding style consistency.
Limitations:
The fundamental limitation of the current architecture is its reliance on syntactic structure. This reliance creates two critical blind spots:
Brittleness to Structural Refactoring (Complex Type-3 Clones): ASTs are sensitive to the specific syntactic constructs used. A developer could refactor a for loop into a while loop, or extract a block of code into a helper function. While the underlying logic or control flow might be identical, the resulting ASTs would be significantly different, causing the clone to be missed.
Inability to Detect Functional Equivalence (Type-4 Clones): The architecture has no mechanism to understand what the code does. It can only see how it is written. Two functions implementing the same algorithm with different approaches will have completely different ASTs and will never be identified as clones.11
To overcome these limitations, the system must be enhanced with components that can analyze code at a higher level of abstraction—capturing the logical flow independent of syntax, and understanding the semantic intent behind the code. This necessitates the integration of Control Flow Graph analysis and Machine Learning, which form the core of the proposed architectural evolution.

Enhancing Structural Similarity with Control Flow Graph Analysis

To transcend the limitations of AST-based analysis and detect clones with similar logic but different syntax (complex Type-3 and some Type-4), the integration of Control Flow Graph (CFG) analysis is paramount. A CFG represents the execution flow of a program, where nodes are basic blocks of straight-line code and directed edges represent jumps in the control flow (e.g., conditionals, loops).12 This representation abstracts away the specific syntactic constructs and focuses on the underlying logical structure, making it more resilient to the types of code modifications that break AST similarity.

Polyglot CFG Generation and Normalization

The first practical step is the reliable generation of CFGs for the three target languages: Rust, Python, and JavaScript. The strategy must account for the varying maturity of tooling in each ecosystem.
Python: The ecosystem provides a strong candidate in the py2cfg library. It is designed specifically for generating CFGs from Python 3 code and offers direct integration with visualization tools like Graphviz, which will be invaluable for debugging and analysis during development.13
JavaScript: The landscape for direct JavaScript-to-CFG generation is more fragmented. A survey of the npm registry reveals libraries like esgraph and styx, but their maintenance status appears less active, posing a potential risk for long-term support.14 A more robust strategy would be to build a custom CFG generator. This can be achieved by leveraging a mature AST parser like Esprima to get the initial code structure, and then using a general-purpose graph library like
graphlib to construct the CFG by traversing the AST and identifying control flow statements (e.g., if, for, while, switch).15 While this requires more initial development effort, it provides greater control and avoids reliance on potentially unmaintained third-party packages.
Rust: The Rust ecosystem, with its focus on safety and performance, provides powerful compiler internals. While off-the-shelf libraries for direct CFG generation are not as prevalent as in the Python world, the foundational blocks are available. The rustc compiler itself generates CFGs as part of its analysis, and its internal APIs could potentially be leveraged. Alternatively, frameworks like ctrl-flow or graph-flow provide abstractions for building graph-based systems, which could serve as the foundation for a custom CFG constructor.16 This path, similar to the JavaScript approach, involves more custom development but ensures deep integration with the language's specific features.
A critical component of this phase is normalization. To ensure that downstream analysis is language-agnostic, all three generators must produce CFGs that adhere to a unified, intermediate representation. This involves defining a standard schema for nodes (basic blocks) and edges (control flow transfers), abstracting away language-specific details while preserving the essential logical structure.

A Comparative Analysis of Graph Similarity Algorithms

Once CFGs are generated, the next challenge is to compare them efficiently and accurately. There is a fundamental trade-off between the precision of a graph similarity algorithm and its computational complexity. A successful architecture must intelligently manage this trade-off.
Classical (Exact) Methods: These methods provide precise measurements but are often too slow for large-scale application.
Graph Isomorphism (e.g., VF2, Ullmann): This determines if two graphs have the exact same structure. It is far too strict for clone detection, as any minor difference, such as an additional statement in a basic block, would result in a non-match.18
Graph Edit Distance (GED): This calculates the minimum cost of edit operations (node/edge insertion, deletion, substitution) required to transform one graph into another. While highly flexible and accurate, computing the exact GED is an NP-hard problem, making it computationally infeasible for the all-pairs comparison required in a large codebase.20
Modern (Approximate and Scalable) Methods: These methods offer a balance of performance and accuracy, making them suitable for real-world systems.
Weisfeiler-Lehman (WL) Kernel: This is a powerful and efficient graph kernel algorithm. It works by iteratively augmenting node labels with a hash of their neighbors' labels. After several iterations, each node's label represents the structure of its local neighborhood. The similarity between two graphs can then be computed by comparing the histograms of these final node labels.22 The WL kernel's runtime scales linearly with the number of edges, making it highly efficient, and it has been successfully applied in academic research for detecting high-level code clones.19
Graph Embedding Techniques: This is the most scalable approach. These methods learn to represent an entire graph or its nodes as low-dimensional vectors (embeddings) in a way that preserves structural properties.25 The expensive graph-to-graph comparison is thus transformed into a very cheap vector-to-vector comparison (e.g., using cosine similarity).
Node2Vec: This technique uses biased random walks to explore the local neighborhoods of nodes, learning embeddings that capture their structural roles within the graph.27 It is highly scalable and effective.
Graph Neural Networks (GNNs): Models like GraphSAGE or Graph Convolutional Networks (GCNs) learn node embeddings through a message-passing mechanism, where nodes aggregate information from their neighbors.29 This allows them to capture both graph structure and node features effectively.
The core challenge of CFG analysis is not to select a single "best" algorithm, but to design a pipeline that leverages the strengths of different approaches. A naive implementation using a single algorithm would force a compromise between scalability and precision. For instance, using only GED would be too slow, while using only embeddings might miss subtle clones. A more sophisticated architecture recognizes this trade-off and uses the right tool for each stage of the analysis.

Recommended Integration Strategy for CFG Analysis

A multi-tiered approach is proposed for integrating CFG analysis into the UV-24 pipeline. This strategy optimizes the trade-off between speed and accuracy.
Preprocessing: For every function/method in the target codebase, generate and cache its CFG in the normalized, language-agnostic format. This is a one-time cost per analysis run.
Tier 1: Fast Candidate Filtering via Graph Embeddings: Use a pre-trained GNN or a Node2Vec model to generate a vector embedding for each cached CFG. Perform a fast, project-wide, all-pairs comparison using cosine similarity on these embeddings. This step acts as a high-throughput filter, identifying a manageable set of potential clone candidates and discarding the vast majority of structurally dissimilar pairs with minimal computational cost.
Tier 2: Refined Scoring via Graph Kernels: For the much smaller set of candidate pairs produced by Tier 1, apply the more computationally intensive but more precise Weisfeiler-Lehman (WL) kernel. This will yield a robust structural similarity score for each candidate pair.
Feature Output: The similarity scores from both the embedding comparison and the WL kernel, along with other graph-level metrics (e.g., node count, edge density, cyclomatic complexity), will be packaged as a feature vector. This vector will be passed as input to the machine learning classifier in the next stage of the main UV-24 pipeline.
This tiered approach allows UV-24 to scale to large codebases by using cheap, approximate methods for broad filtering, while reserving the more expensive, precise methods for the final validation of a small number of promising candidates.
Table 1: Comparison of CFG Similarity Algorithms
Algorithm
Computational Complexity
Precision
Scalability
Primary Use Case in UV-24
Graph Isomorphism
Sub-exponential, but NP-intermediate
Very High (Exact Match)
Very Low
Unsuitable for clone detection due to strictness.
Graph Edit Distance (GED)
NP-hard (O(n23n) in worst case)
High (Flexible)
Very Low
Unsuitable for large-scale comparison. Potentially useful for analyzing small, critical pairs offline.
Weisfeiler-Lehman (WL) Kernel
Near-linear ($O(h \cdot
E
)$)
High (Approximate)
Node2Vec / GNN Embedding
Near-linear ($O(
V
)$ for inference)
Moderate (Approximate)


A Machine Learning Framework for Intelligent Clone Classification

The integration of a machine learning (ML) framework marks the transition of UV-24 from a system that recognizes patterns to one that learns and predicts similarity. An ML classifier, trained on a diverse set of examples, can overcome the limitations of static thresholds and heuristic rules by learning the complex, non-linear relationships between various code features that signify a clone.3 This enables more accurate and adaptable detection, especially for nuanced Type-3 and semantic Type-4 clones.

Feature Engineering for Code Similarity

The performance of any ML model is fundamentally dependent on the quality and richness of its input features. The UV-24 enhancement will leverage a comprehensive feature set derived from multiple levels of code abstraction, providing the model with a holistic view of each code fragment.
Lexical Features: Extracted directly from the sequence of tokens. These include classic text-based features like term frequency-inverse document frequency (TF-IDF) of tokens and n-gram overlap, which capture surface-level similarities.
Syntactic Features: Derived from the Abstract Syntax Tree (AST). These features quantify structural similarity and include metrics like tree edit distance, the number of common subtree hashes, and statistics on the types of nodes and edges in the tree.9 These are provided by the existing UV-24 components.
Structural Features: Generated by the new CFG analysis pipeline (as detailed in Section 3). This includes the similarity scores from both the graph embedding comparison and the WL kernel, as well as graph-theoretic metrics like cyclomatic complexity, graph density, and in/out-degree distributions of nodes.31 These features describe the logical complexity and shape of the code's execution flow.
Semantic Features: These are high-dimensional vector representations (embeddings) of code fragments generated by large, pre-trained transformer models. These embeddings aim to capture the functional meaning of the code and are the most powerful features for detecting Type-4 clones.

Model Architecture and Selection: From CodeBERT to GraphCodeBERT

The choice of ML model is a critical architectural decision. While traditional models can provide a solid baseline, state-of-the-art performance in code understanding is now dominated by deep learning-based transformer models.
Traditional Models (Baseline): Models like Random Forests or Support Vector Machines (SVMs) are fast to train and relatively interpretable.3 They can perform well when fed the rich, handcrafted feature set described above. They will serve as an essential baseline to quantify the performance lift provided by more complex deep learning models.
CodeBERT: A powerful transformer model developed by Microsoft, CodeBERT is pre-trained on a massive corpus of both natural language text and source code from multiple programming languages.33 It learns to generate contextual embeddings for code tokens. For clone detection, two code snippets are fed into the model, and it outputs a similarity prediction.34 CodeBERT has demonstrated strong performance, particularly on Type-1 and Type-4 clones in the BigCloneBench dataset.33 However, its core architecture treats source code as a flat sequence of tokens, just like natural language. This ignores the inherent, rich structural information of code (e.g., which variable is used where, how control flows), and its performance has been shown to degrade significantly when applied to datasets and functionalities it has not seen during training.35
GraphCodeBERT (Recommended Model): This model represents a significant architectural evolution over CodeBERT and is the recommended choice for UV-24. GraphCodeBERT enhances the standard transformer architecture by incorporating structural information from the code in the form of a data flow graph (DFG) during its pre-training phase.36 A DFG connects variables to their points of use, explicitly modeling the "where-the-value-comes-from" relationship. By feeding both the token sequence and this structural information into the model, GraphCodeBERT's attention mechanism can learn not just the context of tokens in a sequence, but also their semantic relationships through the program's data flow.37 This makes it fundamentally better equipped to understand program logic and, therefore, to detect semantic clones.
The transition from CodeBERT to GraphCodeBERT reflects a pivotal paradigm shift in the field of code representation learning: the move from treating code as simple "text" to treating it as "structured logic." A model that only sees a sequence of tokens might learn that x, y, and z often appear together, but it doesn't know that z = x + y. GraphCodeBERT, by incorporating data flow, is designed to understand this crucial relationship. By selecting GraphCodeBERT, the UV-24 system aligns its core technology with this more advanced and logically sound representation of source code, bridging the critical gap between sequential and structural understanding that is essential for true semantic analysis.

Training, Validation, and Deployment Pipeline

The selected GraphCodeBERT model must be fine-tuned for the specific task of clone detection.
Training Data: The model will be trained on large, labeled datasets of code pairs.
BigCloneBench: This dataset can be used for initial model training and for benchmarking performance on syntactic clones (Type-1 to Type-3).11 However, as will be detailed in Section 6, it must be used with extreme caution and is not suitable for evaluating semantic clone performance due to significant labeling inaccuracies.39
POJ-104: This dataset, consisting of 104 different programming problems solved by multiple students, is a much better resource for training and validating semantic clone detection, as all solutions for a given problem are, by definition, functionally equivalent.40
Custom Dataset: A strategic priority will be the creation of a high-quality, internally-validated dataset to ensure the model is trained on accurate ground truth.
Integration: The fine-tuned GraphCodeBERT model will serve as the final classification stage in the UV-24 pipeline. It will receive a candidate pair of code snippets and their associated handcrafted features (lexical, syntactic, structural). The model will then output a final similarity score between 0 and 1, and a predicted clone type (e.g., Type-3, Type-4).
Performance Optimization: Running a large transformer model at scale requires performance considerations. The system will implement aggressive caching of code embeddings to avoid re-computation. The entire pipeline will be designed for parallel processing, allowing multiple code pairs to be analyzed concurrently to maximize throughput.3
Table 2: Machine Learning Model Comparison for Clone Detection
Model
Input Representation
Key Strength
Key Weakness
Suitability for Type-4 Clones
Implementation Complexity
Random Forest / SVM
Handcrafted feature vectors (lexical, syntactic, structural)
Fast, interpretable, strong baseline.
May not capture deep semantic nuances; relies on manual feature engineering.
Low to Moderate
Low
CodeBERT
Sequence of code tokens
Powerful semantic understanding from large-scale pre-training.
Ignores code's inherent structure; treats code as flat text. Performance drops on unseen data.
High
Moderate
GraphCodeBERT
Sequence of code tokens + Data Flow Graph
Combines semantic and structural understanding. More robust representation of program logic.
Higher computational cost than CodeBERT; more complex pre-processing.
Very High
High
Pure GNN
Graph structures (AST/CFG)
Excellent at learning from graph topology.
Lacks pre-training on massive code-text corpora; harder to integrate with non-graph features.
Moderate to High
High


Achieving Semantic Equivalence Detection (Type-4 Clones)

The detection of Type-4 clones—code fragments that are functionally equivalent but syntactically and structurally different—represents the frontier of code analysis. It requires a system to move beyond comparing what code looks like to understanding what code does. This is a profoundly challenging problem, rooted in the theoretical limits of computation.

The Semantic Frontier: Challenges of Functional Equivalence

The Halting Problem, a foundational result in computer science, proves that it is impossible to create a general algorithm that can determine, for all possible inputs, whether an arbitrary program will finish running or continue to run forever. A direct consequence of this is that proving the perfect functional equivalence of two arbitrary code fragments for all possible inputs is an undecidable problem. Therefore, any practical approach to Type-4 clone detection must be, by definition, a heuristic or an approximation. The goal is not to achieve impossible perfection but to build a highly accurate and scalable system that provides strong evidence of functional equivalence.
The most robust and practical architecture for this task is not a single monolithic tool but a hybrid system that combines the scalability of probabilistic machine learning with the rigor of deterministic formal methods. This approach allows the system to balance the competing demands of broad discovery and high-confidence verification.

Primary Strategy: Deep Code Embeddings with GraphCodeBERT

The core of the proposed semantic detection strategy relies on the deep semantic representations learned by the fine-tuned GraphCodeBERT model. This approach leverages the model's exposure to millions of code examples during pre-training to develop an intuitive understanding of program functionality.
The workflow for this primary strategy is as follows:
Input: Two code fragments to be compared, for example, functions f1 and f2.
Embedding Generation: Each code fragment is processed by the fine-tuned GraphCodeBERT model. The model outputs a high-dimensional vector, or embedding (e.g., v1 and v2), which serves as a dense numerical representation of the code's semantics.37
Similarity Computation: The similarity between the two code fragments is calculated as the cosine similarity of their embedding vectors. The formula for cosine similarity between two vectors A and B is similarity=∥A∥∥B∥A⋅B​. This value ranges from -1 (opposite) to 1 (identical), and is typically scaled to a 0-1 range for this application.
Classification: If the resulting similarity score exceeds a carefully tuned threshold, the pair is classified as a potential Type-4 (semantic) clone.
This method is powerful because it abstracts away implementation details. The model learns to map different implementations of, for instance, a "sorting algorithm" to similar regions in the high-dimensional embedding space. This makes it a scalable and effective heuristic for identifying semantic clones across a large codebase.

Confirmatory Strategy: Targeted Symbolic Execution

While deep code embeddings provide a powerful and scalable heuristic, they are probabilistic and do not constitute a formal proof of functional equivalence. For scenarios requiring the highest degree of confidence—such as security vulnerability analysis or intellectual property compliance—a deterministic verification method is needed. Symbolic execution provides this capability, but at a significant computational cost.
Methodology:
Symbolic execution is a static analysis technique that explores a program's execution paths without running it with concrete inputs. Instead, it uses symbolic values (e.g., x, y) for inputs.43 As it traverses the code, it builds a set of logical formulas representing the path conditions (the constraints on the inputs required to traverse a specific path) and the resulting output in terms of the symbolic inputs.44 By generating these symbolic input-output mappings for two different functions, it is possible to formally check if they are equivalent for all inputs that satisfy the path conditions.43
Proposed Use within UV-24:
Given its high computational cost, particularly the problem of "path explosion" where the number of paths to analyze grows exponentially with conditional branches and loops, symbolic execution is entirely unsuitable for general, all-pairs clone detection.44
Instead, it should be implemented as an optional, on-demand, high-confidence verification stage. The workflow would be:
The primary detection pipeline (ending with GraphCodeBERT) identifies a set of likely semantic clones.
For a small, user-defined subset of these clones—perhaps those with a similarity score in a critical range (e.g., 0.95-0.99), or those located in security-sensitive modules—the system can invoke a symbolic execution engine.
The engine attempts to formally prove the functional equivalence of the pair.
This creates a tiered system of confidence. The GraphCodeBERT stage provides a "likely semantic clone" result, which is sufficient for most code maintenance tasks. The optional symbolic execution stage provides a "verified semantic clone" result, which is invaluable for high-stakes applications. This hybrid of probabilistic discovery and deterministic verification allows UV-24 to be both broad in its reach and deep in its certainty where it matters most, providing a practical solution to an otherwise intractable problem.

System-Wide Optimization and Empirical Evaluation

The introduction of advanced analytical techniques necessitates a corresponding focus on system-wide optimization and, most importantly, a rigorous and honest framework for empirical evaluation. The success of the enhanced UV-24 system depends not only on the power of its algorithms but also on its adaptability to real-world code and the validity of the benchmarks used to measure its performance.

Domain-Specific Tuning and Adaptive Thresholds

A well-documented weakness of many clone detection tools is their reliance on a single, global similarity threshold. Research has shown that the optimal threshold for identifying meaningful clones can vary significantly between different programming languages, different projects, and even different functionalities within the same project.45 A one-size-fits-all approach inevitably leads to a compromise between precision and recall.
To address this, the UV-24 system will incorporate a multi-faceted framework for domain-specific tuning:
Configurable Thresholds and Weights: The system will expose key parameters through a user-friendly configuration system (e.g., project-level YAML or JSON files). This will allow users to define language-specific and project-specific similarity thresholds for the various detection stages (AST, CFG, semantic). Furthermore, the final similarity score will be a weighted combination of features from different analysis levels (lexical, syntactic, structural, semantic), and these weights will also be tunable.
ML-Powered Adaptation: The most powerful mechanism for adaptation is the machine learning model itself. Instead of relying on a single, manually-tuned threshold, the ML classifier learns a complex, multi-dimensional decision boundary in the feature space. This is, in effect, a highly sophisticated and adaptive threshold. By fine-tuning the GraphCodeBERT model on datasets specific to a particular domain (e.g., a company's internal codebase, a specific framework like Android), the system can learn the unique characteristics of clones in that context, leading to significantly higher accuracy.47
Continuous Improvement via Feedback Loop: To facilitate this adaptation, the system can incorporate a feedback mechanism. When UV-24 presents a potential clone, the user can validate it as a "true clone" or invalidate it as a "false positive." This feedback can be collected and used to periodically re-train and fine-tune the ML model, creating a continuous improvement cycle that adapts the system to the user's specific needs and perspective on what constitutes a meaningful clone.3

A Rigorous Performance Benchmarking Protocol

Validating the performance of the enhanced UV-24 system is a mission-critical task. The benchmarking protocol must be comprehensive, employing appropriate datasets and metrics to provide an unbiased assessment of the system's capabilities.
Dataset Selection and Critique:
The choice of dataset is the most critical factor in benchmarking. A flawed dataset will produce misleading results, regardless of the quality of the tool being tested.
BigCloneBench (BCB): This is a large-scale, widely-used benchmark containing millions of validated clone pairs from open-source Java projects.50 It has become a de facto standard for evaluating clone detection tools. It is a
valid and essential benchmark for evaluating the detection of syntactic clones (Type-1, Type-2, and Type-3). UV-24 must demonstrate strong performance on BCB for these clone types. However, recent, critical research has demonstrated that BigCloneBench is fundamentally flawed and unsuitable for evaluating semantic (Type-4) clone detection.39 A detailed manual analysis revealed that as many as 93% of the pairs labeled as "Weakly Type-3/Type-4" (the category used for semantic evaluation) are not functionally similar and are therefore mislabeled. Training or evaluating a semantic model on BCB will lead to the model overfitting on dataset-specific artifacts and reporting deceptively high performance metrics that do not translate to the real world. This is a critical risk that must be addressed.
POJ-104: This dataset consists of C++ source code submissions for 104 different programming problems from an online judge.52 For any given problem, all correct solutions are, by definition, functionally equivalent (i.e., Type-4 clones). This makes POJ-104 an
excellent dataset for evaluating the core capability of a system to detect semantic clones in a controlled, algorithmic context.40
Custom Validation Set (Strategic Priority): Given the identified flaws in public datasets for semantic analysis, the most significant risk to the UV-24 enhancement project is not a technical failure but a methodological one: optimizing against an invalid benchmark. To mitigate this, a key strategic recommendation is for the project team to invest resources in creating a small but high-quality, manually-validated internal ground truth dataset. This dataset should include examples of complex Type-3 and Type-4 clones specific to the target languages (Rust, Python, JavaScript) and potentially relevant to the business domains of key users. While resource-intensive, this is the only way to reliably measure and validate the true semantic detection performance of the enhanced system.
Metrics for Evaluation:
The evaluation will use a standard set of metrics to assess both accuracy and performance.
Accuracy Metrics: Precision, Recall, and F1-score are the standard metrics for evaluating classification quality.53 These will be calculated for the overall dataset and also on a per-clone-type basis to understand the system's strengths and weaknesses.31
Performance Metrics: To ensure the tool is practical for enterprise use, runtime (processing time per 1M LOC), memory consumption, and scalability will be systematically measured.55
Benchmarking Workflow:
The evaluation will proceed in a structured, multi-step workflow:
Baseline Measurement: Run the current UV-24 system against the benchmarks to establish a performance baseline.
Incremental Testing: Evaluate each new major component (e.g., the CFG analysis pipeline, the ML classifier) in isolation to quantify its individual contribution.
Integrated Pipeline Testing: Test the full, end-to-end enhanced system to measure its overall performance.
Regression Testing: Ensure that the new enhancements for complex clones have not negatively impacted the accuracy or performance of detecting simpler (Type-1, Type-2) clones.
Scalability Testing: Execute the tool on a series of increasingly large, real-world codebases to analyze its performance characteristics under load.
Table 3: Benchmarking Datasets for Code Clone Detection

Dataset
Primary Clone Types
Ground Truth Quality (Syntactic)
Ground Truth Quality (Semantic)
Recommended Use in UV-24
BigCloneBench
Type-1, Type-2, Type-3, Type-4
High for Types 1-3
Very Low / Flawed for Type-4 39
Primary benchmark for syntactic clones (Types 1-3). Use for regression testing. DO NOT USE for validating semantic (Type-4) performance.
POJ-104
Type-4
N/A
High (within algorithmic domain)
Primary public benchmark for semantic (Type-4) clone detection. Used to train and validate the core semantic capabilities of the ML model.
Custom Internal Set
Complex Type-3, Type-4
Very High (by definition)
Very High (by definition)
Strategic Priority. The definitive ground truth for final validation, domain-specific tuning, and reporting trustworthy performance metrics for semantic detection.


Final Architectural Blueprint and Implementation Roadmap

Synthesizing the analysis from the preceding sections, this final section presents the unified architectural design for the enhanced UV-24 system and a practical, phased roadmap for its implementation. This blueprint provides a clear path to transforming UV-24 into a state-of-the-art code analysis platform.

The Unified Multi-Stage Detection Pipeline

The proposed architecture is a modular, multi-stage pipeline that progressively applies more sophisticated and computationally intensive analysis techniques. Each stage acts as a filter, refining the set of candidate clone pairs and enriching them with features for subsequent stages.
Diagram of the Proposed UV-24 Pipeline:



|
       v
+---------------------------------+

| Stage 1: Fast Candidate Filter |
| (Karp-Rabin Hashing) | ---->
+---------------------------------+

| (Syntactic Candidates)
       v
+---------------------------------+

| Stage 2: Syntactic Verification |
| (AST Analysis) | ---->
+---------------------------------+

| (Structural Candidates)
       v
+---------------------------------+

| Stage 3: Structural Analysis |
| (CFG Pipeline) |
| - Tier 1: Embedding Filter | ---->
| - Tier 2: WL-Kernel Scorer |
+---------------------------------+

| (Semantic Candidates + All Features)
       v
+---------------------------------+

| Stage 4: Semantic Classifier |
| (Fine-tuned GraphCodeBERT) | ---->
+---------------------------------+

| (High-Importance Clones)
       v
+---------------------------------+

| Stage 5: (Optional) Verification|
| (Targeted Symbolic Execution) | ----> [Verified Functional Equivalence]
+---------------------------------+


Stage Descriptions:
Stage 1: Fast Candidate Filtering (Existing): The existing Karp-Rabin hashing mechanism serves as the entry point, rapidly identifying textually identical blocks and filtering out the vast majority of dissimilar code to reduce the load on subsequent stages.
Stage 2: Syntactic Verification (Existing): The existing AST comparison engine validates Type-1 and Type-2 clones and identifies simple Type-3 clones. It generates a set of syntactic features (e.g., tree similarity) for its candidate pairs.
Stage 3: Structural Similarity Analysis (New): This new stage processes candidates that are structurally ambiguous. It implements the two-tiered CFG analysis pipeline: fast filtering with graph embeddings followed by refined scoring with the Weisfeiler-Lehman kernel. It outputs a rich set of structural features.
Stage 4: Semantic Classification (New): This is the core of the new system. The fine-tuned GraphCodeBERT model acts as the final arbiter. It takes candidate pairs from all previous stages, along with the complete set of lexical, syntactic, and structural features. It computes a final, unified similarity score and predicts the most likely clone type (e.g., "Moderately Type-3," "Weakly Type-3/Type-4").
Stage 5: High-Confidence Verification (Optional, New): This stage is not part of the default automated flow. It is an API hook that allows a user or an automated security tool to submit a specific, high-importance clone pair (identified by Stage 4) for formal verification of functional equivalence using a symbolic execution engine.

Phased Implementation Roadmap

The implementation is planned in a series of focused phases, with an estimated total duration of 10–14 days. This timeline is an aggressive but achievable target for an experienced engineering team and accounts for the complexities identified in this report.
Phase
Key Activities
Estimated Duration
Key Dependencies & Risks
1. CFG Integration
- Implement/integrate CFG parsers for Python, JavaScript, and Rust.
- Define and implement the unified, language-agnostic CFG schema.
- Implement the two-tiered similarity pipeline: GNN/Node2Vec for embedding and filtering, followed by the WL-Kernel for scoring.
- Implement caching for generated CFGs and embeddings.
3–4 days
- Risk: Rust and JavaScript CFG generation may require significant custom development if suitable libraries are not found.
- Dependency: A clear, finalized schema for the unified CFG representation.
2. ML Model Development
- Set up the Python environment with PyTorch and Transformers.
- Develop the data loading and pre-processing pipeline for GraphCodeBERT.
- Fine-tune an initial GraphCodeBERT model on the POJ-104 dataset.
- Begin development of the custom internal benchmark dataset. This is a parallel effort.
3–4 days
- Dependency: Access to GPU resources for model fine-tuning.
- Risk: Initial model performance may be lower than expected, requiring hyperparameter tuning.
3. Semantic & Pipeline Integration
- Integrate the fine-tuned model from Phase 2 into the main UV-24 application.
- Implement the logic for passing candidate pairs and their feature vectors between all stages of the pipeline.
- Develop the external API for invoking the optional Symbolic Execution stage (Stage 5).
2–3 days
- Risk: Performance bottlenecks may arise when passing large feature vectors between stages, requiring optimization.
4. Tuning & Benchmarking
- Implement the domain-specific configuration system (YAML/JSON) for thresholds and weights.
- Execute the full benchmarking protocol (syntactic on BCB, semantic on POJ-104 and the new internal dataset).
- Analyze results, tune similarity thresholds, and potentially perform a second round of model fine-tuning based on benchmark performance.
2–3 days
- Dependency: The first version of the custom internal benchmark must be ready for use.
- Risk: Benchmarking may reveal unexpected weaknesses that require revisiting earlier phases.
Total Estimated Duration


10–14 days




Conclusion: Positioning UV-24 as a State-of-the-Art Solution

The implementation of this comprehensive architectural blueprint will transform the UV-24 system. By moving beyond its strong but limited syntactic foundation, UV-24 will gain the ability to perform deep structural and semantic analysis. The integration of a multi-tiered CFG pipeline, a state-of-the-art GraphCodeBERT classification model, and a rigorous, honest benchmarking methodology will provide a durable competitive advantage. This enhancement will not only deliver a significant leap in accuracy for detecting the most complex and harmful types of code duplication but will also establish a flexible, extensible platform prepared for future advancements in code intelligence. The resulting system will empower developers and organizations to manage technical debt more effectively, improve software quality and security, and ultimately build more maintainable and robust systems, positioning UV-24 as a leading solution in the critical field of code analysis.
Works cited
The Survey of the Code Clone Detection Techniques and ... - CORE, accessed July 13, 2025, https://core.ac.uk/download/539895020.pdf
SOFTWARE CODE CLONE DETECTION MODEL USING HYBRID APPROACH - CiteSeerX, accessed July 13, 2025, https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=52860def9bff455db68d98ad6dfa0dfa8acd82da
CloneCognition: Machine Learning Based Code Clone Validation Tool - Chanchal Roy, accessed July 13, 2025, https://clones.usask.ca/pubfiles/articles/MostaeenCloneCognitionFSE2019.pdf
A Systematic Review on Code Clone Detection - SciSpace, accessed July 13, 2025, https://scispace.com/pdf/a-systematic-review-on-code-clone-detection-51ueuzckrv.pdf
Clone-Types 1 to Type 4 | Download Scientific Diagram, accessed July 13, 2025, https://www.researchgate.net/figure/Clone-Types-1-to-Type-4_fig2_335152710
Comparison and Evaluation of Code Clone Detection ... - CiteSeerX, accessed July 13, 2025, https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=fb35f65339c4f1bbe9c0bd25f09f83e65e436d51
Code Clones, accessed July 13, 2025, https://courses.cs.vt.edu/cs5704/spring16/handouts/5704-10-CodeClones.pdf
Understanding the Evolution of Type-3 Clones: An Exploratory Study - University of Saskatchewan, accessed July 13, 2025, https://www.cs.usask.ca/~croy/papers/2013/Saha_MSR2013_Type3Evolution.pdf
(PDF) Survey on Software code clone detection - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/360015247_Survey_on_Software_code_clone_detection
Design and Analysis of a Hybrid Technique for Code Clone Detection - ijarcce, accessed July 13, 2025, http://www.ijarcce.com/upload/2016/november-16/IJARCCE%2081.pdf
Deep learning application on code clone detection: A review of current knowledge, accessed July 13, 2025, https://www.researchgate.net/publication/356057132_Deep_learning_application_on_code_clone_detection_A_review_of_current_knowledge
Comparison and Evaluation of Clone Detection Techniques with Different Code Representations - Yueming Wu, accessed July 13, 2025, https://wu-yueming.github.io/Files/ICSE2023_TACC.pdf
py2cfg·PyPI, accessed July 13, 2025, https://pypi.org/project/py2cfg/
control flow graph - npm search, accessed July 13, 2025, https://www.npmjs.com/search?q=control%20flow%20graph
Is there any JavaScript libraries for graph operations and algorithms? - Stack Overflow, accessed July 13, 2025, https://stackoverflow.com/questions/14483473/is-there-any-javascript-libraries-for-graph-operations-and-algorithms
a-agmon/rs-graph-llm: High-performance framework for building interactive multi-agent workflow systems in Rust - GitHub, accessed July 13, 2025, https://github.com/a-agmon/rs-graph-llm
ctrl-flow - crates.io: Rust Package Registry, accessed July 13, 2025, https://crates.io/crates/ctrl-flow
SimilaR: R Code Clone and Plagiarism Detection - UNL Digital Commons, accessed July 13, 2025, https://digitalcommons.unl.edu/cgi/viewcontent.cgi?article=1176&context=r-journal
Weisfeiler-Lehman Graph Kernels - Journal of Machine Learning Research, accessed July 13, 2025, https://www.jmlr.org/papers/volume12/shervashidze11a/shervashidze11a.pdf
Graph Edit Distance Learning via Modeling Optimum Matchings with Constraints - IJCAI, accessed July 13, 2025, https://www.ijcai.org/proceedings/2021/0212.pdf
Computing Graph Edit Distance via Neural Graph Matching - VLDB Endowment, accessed July 13, 2025, https://www.vldb.org/pvldb/vol16/p1817-cheng.pdf
Enriched Weisfeiler-Lehman Kernel for Improved Graph Clustering of Source Code, accessed July 13, 2025, https://www.researchgate.net/publication/340843416_Enriched_Weisfeiler-Lehman_Kernel_for_Improved_Graph_Clustering_of_Source_Code
Detecting Similar Programs via the Weisfeiler-Leman Graph Kernel - Martin Schäf, accessed July 13, 2025, https://www.martinschaef.de/papers/icsr2016.pdf
Design and implementation of a high level code clone detection method, accessed July 13, 2025, http://joces.nudt.edu.cn/EN/abstract/abstract16391.shtml
Using Graph Embedding Techniques in Process-Oriented Case-Based Reasoning - MDPI, accessed July 13, 2025, https://www.mdpi.com/1999-4893/15/2/27
Understanding graph embedding methods and their applications - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/347300725_Understanding_graph_embedding_methods_and_their_applications
node2vec: Scalable Feature Learning for Networks - Stanford University, accessed July 13, 2025, https://snap.stanford.edu/node2vec/
node2vec: Scalable Feature Learning for Networks - CS Stanford, accessed July 13, 2025, https://cs.stanford.edu/~jure/pubs/node2vec-kdd16.pdf
What is Graph Embedding? A Practical Guide for Developers - PuppyGraph, accessed July 13, 2025, https://www.puppygraph.com/blog/graph-embedding
Machine Learning-Based Methods for Code Smell Detection: A Survey - MDPI, accessed July 13, 2025, https://www.mdpi.com/2076-3417/14/14/6149
Code Clone Detection and Analysis Using Software Metrics and Neural Network-A Literature Review - IJCST, accessed July 13, 2025, https://www.ijcstjournal.org/volume-3/issue-2/IJCST-V3I2P26.pdf
Code Clone Detection using Graphs and Adjacency ... - ijltemas, accessed July 13, 2025, https://www.ijltemas.in/DigitalLibrary/Vol.7Issue1/191-195.pdf
CodeBERT for Code Clone Detection: A Replication Study, accessed July 13, 2025, https://www.computer.org/csdl/proceedings-article/iwsc/2022/844700a039/1J6hKOqToVG
CodeXGLUE/Code-Code/Clone-detection-BigCloneBench/README.md at main - GitHub, accessed July 13, 2025, https://github.com/microsoft/CodeXGLUE/blob/main/Code-Code/Clone-detection-BigCloneBench/README.md
Interpreting CodeBERT for Semantic Code Clone Detection - my.SMU, accessed July 13, 2025, http://www.mysmu.edu/faculty/lxjiang/papers/apsec23interpretCodeBERT.pdf
GRAPHCODEBERT: PRE-TRAINING CODE REPRESEN- TATIONS WITH DATA FLOW - OpenReview, accessed July 13, 2025, https://openreview.net/pdf?id=jLoC4ez43PZ
Augmenting the Interpretability of GraphCodeBERT for Code Similarity Tasks - Jorge Martinez-Gil, accessed July 13, 2025, https://www.jorgemar.com/papers/Interpretability.pdf
Augmenting the Interpretability of GraphCodeBERT for Code Similarity Tasks, accessed July 13, 2025, https://www.worldscientific.com/doi/10.1142/S0218194025500160
How the Misuse of a Dataset Harmed Semantic Clone ... - arXiv, accessed July 13, 2025, https://arxiv.org/abs/2505.04311
CodeXGLUE - POJ-104 Benchmark (Clone Detection) - Papers With Code, accessed July 13, 2025, https://paperswithcode.com/sota/clone-detection-on-codexglue-poj-104
Evaluating Small-Scale Code Models for Code Clone Detection - arXiv, accessed July 13, 2025, https://arxiv.org/pdf/2506.10995
Evaluating few shot and Contrastive learning Methods for Code Clone Detection - M Shehata, accessed July 13, 2025, https://mshehata.ok.ubc.ca/publications/2022_kcGALyvb7RWbsd8.pdf
Applying Symbolic Execution to Semantic Code ... - KSI Research, accessed July 13, 2025, https://ksiresearch.org/seke/seke23paper/paper070.pdf
ARDiff: Scaling Program Equivalence Checking via Iterative Abstraction and Refinement of Common Code, accessed July 13, 2025, https://www3.ntu.edu.sg/home/yi_li/files/Badihi2020ASP.pdf
Threshold-free Code Clone Detection for a Large-scale Heterogeneous Java Repository, accessed July 13, 2025, https://seal-queensu.github.io/publications/pdf/SANER-Iman-2015.pdf
Multi-threshold token-based code clone detection - arXiv, accessed July 13, 2025, https://arxiv.org/pdf/2002.05204
Zero-Shot Code Representation Learning via Prompt Tuning - arXiv, accessed July 13, 2025, https://arxiv.org/html/2404.08947v1
Exploring the Boundaries Between LLM Code Clone Detection and Code Similarity Assessment on Human and AI-Generated Code - MDPI, accessed July 13, 2025, https://www.mdpi.com/2504-2289/9/2/41
Threshold-free code clone detection for a large-scale heterogeneous Java repository, accessed July 13, 2025, https://www.researchgate.net/publication/282680345_Threshold-free_code_clone_detection_for_a_large-scale_heterogeneous_Java_repository
clonebench/BigCloneBench - GitHub, accessed July 13, 2025, https://github.com/clonebench/BigCloneBench
Evaluating Clone Detection Tools with BigCloneBench - Chanchal Roy, accessed July 13, 2025, https://clones.usask.ca/pubfiles/articles/SvajlenkoEvaluatingToolsICSME2015.pdf
microsoft/CodeXGLUE - GitHub, accessed July 13, 2025, https://github.com/microsoft/CodeXGLUE
Code Clone Detection Using Metrics Based Technique and Classification using Neural Network, accessed July 13, 2025, http://nebula.wsimg.com/024b63c0168bed76b6923a7b7feb70af?AccessKeyId=DFB1BA3CED7E7997D5B1&disposition=0&alloworigin=1
Evaluating the performance of clone detection tools in detecting cloned co-change candidates - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/358121496_Evaluating_the_performance_of_clone_detection_tools_in_detecting_cloned_co-change_candidates
Benchmarking Software: Mastering the Art of Data Analysis - Metridev, accessed July 13, 2025, https://www.metridev.com/metrics/benchmarking-software-mastering-the-art-of-data-analysis/
Benchmarking workflows to assess performance and suitability of germline variant calling pipelines in clinical diagnostic assays, accessed July 13, 2025, https://pmc.ncbi.nlm.nih.gov/articles/PMC7903625/
