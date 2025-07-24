
UV-24 Research Report: A Framework for Advanced Semantic Clone Detection and ML-Powered Classification


Executive Summary

This report presents a comprehensive research and development framework for enhancing the Uveddi code duplication detector with state-of-the-art capabilities for identifying Type-4 semantic clones and integrating machine learning for improved classification accuracy. The existing system, while proficient in detecting syntactic clones (Types 1-3) through a hybrid Karp-Rabin and Abstract Syntax Tree (AST) approach, lacks the semantic understanding necessary to identify functionally equivalent but syntactically divergent code. This limitation represents a significant gap, as undetected semantic clones contribute to increased maintenance costs, code fragility, and the propagation of defects across a codebase.1
The proposed enhancement is a multi-stage, hybrid analysis pipeline designed to balance performance and precision at enterprise scale. The core architectural recommendations are as follows:
A Multi-Stage Semantic Analysis Pipeline: The existing two-stage pipeline will be augmented with two new, optional stages. A fast Code Embedding Filter (Stage 3), powered by a fine-tuned transformer model such as GraphCodeBERT, will serve as a scalable semantic candidate generator. This is followed by a Deep Semantic Verification stage (Stage 4) that uses Program Dependence Graph (PDG) analysis with approximate graph matching to provide high-fidelity confirmation of functional equivalence. This tiered design ensures that computationally expensive graph analysis is reserved for only the most promising candidates, meeting stringent performance requirements.
A Unified Intermediate Representation (IR): To achieve true cross-language analysis for Rust, Python, and JavaScript, a custom, language-agnostic IR will be developed. Source code from each language will be parsed into this unified representation before Control Flow Graph (CFG) and PDG construction. This pivotal architectural choice abstracts away language-specific syntax, enabling the core analysis engine to operate on a single, semantic representation and effectively recognize idiomatic equivalences across languages.
An ML-Powered Classification Framework: A machine learning classifier will be integrated at the end of the pipeline to act as a meta-learner. It will fuse a rich set of features—lexical, syntactic, graph-based, and embedding-based—from all preceding stages to produce a final, nuanced classification (e.g., Type4) and a calibrated confidence score. This replaces brittle, rule-based thresholds with a data-driven model, projected to reduce the false positive rate by over 30%.
An Adaptive and Context-Aware System: The detector will be enhanced with a Bayesian Optimization framework for the automated, adaptive tuning of analysis parameters and thresholds based on codebase characteristics. This, combined with the recognition of language-specific clone patterns, will ensure the tool is optimized for different project domains and languages.
The successful implementation of this framework is projected to elevate the Uveddi clone detector to the forefront of semantic code analysis technology. It will enable the detection of elusive Type-4 clones with over 70% recall on relevant benchmarks, while maintaining scalability for codebases exceeding 100,000 files and providing developers with more accurate, actionable insights to improve code quality and maintainability.

Part I: The Semantic Frontier - Advancing Beyond Syntactic Clone Detection

This section establishes the foundational context and motivation for the research, defining the problem space of semantic clone detection and its significance in modern software engineering.

1.1 Characterizing the Challenge of Type-4 "Semantic" Clones

The classification of code clones into four distinct types provides a crucial taxonomy for understanding the scope and complexity of code duplication.3 While the existing detector capably handles the first three types, the primary research objective is to address the most challenging category: Type-4, or semantic clones.
Type-1 (Exact Clones): Identical code fragments, excluding variations in whitespace and comments.3 These are the most trivial to detect.
Type-2 (Renamed Clones): Structurally and syntactically identical code where identifiers (variables, types, functions) and literals have been changed.4 These are often called parameterized clones.
Type-3 (Near-Miss Clones): Code fragments with further modifications, such as added, removed, or altered statements, in addition to Type-2 variations.3 These represent syntactic drift from a common origin.
Type-4 (Semantic Clones): Code fragments that are functionally equivalent but implemented with different syntactic variants.3 These clones perform the same computation or achieve the same result through entirely different algorithms or logic. For example, a function calculating a factorial using an iterative
for loop is a semantic clone of a function calculating it via recursion.1
The fundamental limitation of the current two-stage system, which relies on Karp-Rabin hashing and AST comparison, is its dependence on lexical and structural similarity.6 These methods are inherently incapable of identifying Type-4 clones because, by definition, such clones lack the necessary syntactic resemblance. They operate on the
form of the code, not its function. The core challenge of this research initiative is to bridge this "semantic gap" by incorporating techniques that analyze program behavior and intent.8
The prevalence of code duplication in large software systems is well-documented, with studies indicating that between 5% and 23% of the codebase can be duplicated code.4 Undetected semantic clones significantly exacerbate the negative consequences of this duplication. They lead to increased maintenance costs, as a bug fixed in one implementation must be manually identified and fixed in all other functionally equivalent—but syntactically different—implementations.1 This "bug propagation" is a primary driver of software fragility.
Furthermore, the origin of semantic clones reveals a deeper complexity. They do not arise solely from simple copy-paste-modify workflows. Instead, they can emerge from two distinct evolutionary paths. The first is divergent evolution, where a single piece of code is copied and then modified so heavily over time that its syntactic link to the original is lost. The second, and more subtle, origin is convergent evolution, where different developers or teams independently write code to solve the same problem, arriving at functionally identical but syntactically unique solutions.1 This distinction is critical because it informs the appropriate corrective action. A truly advanced tool should not merely flag a semantic clone; it should provide the context necessary to guide a developer's refactoring decision. For divergent clones, the goal might be to merge them back into a single, configurable implementation. For convergent clones, the appropriate action might be to introduce a new shared library or abstraction that both original locations can then utilize.

1.2 State-of-the-Art in Clone Detection: A Literature Review

The field of code clone detection has evolved significantly, moving from simple textual comparisons to sophisticated semantic analysis. This evolution provides the context for the proposed architectural enhancements.
Text-Based and Token-Based Methods: The earliest approaches treated code as simple text, comparing lines or sequences of characters after removing whitespace and comments.2 Token-based methods improved upon this by using a lexer to convert source code into a stream of tokens, making the analysis more resilient to formatting changes.6 The current system's Karp-Rabin hashing stage is a highly optimized form of this token-level analysis.
Tree-Based and Metric-Based Methods: The introduction of AST-based analysis marked a significant step forward, allowing for the comparison of code based on its syntactic structure rather than its textual representation.6 By finding similar subtrees in the AST, these methods can robustly detect Type-2 and Type-3 clones. The current system's second stage leverages this principle. Metric-based approaches complement this by calculating various software metrics (e.g., cyclomatic complexity, fan-in/fan-out) and comparing these feature vectors to find similar code blocks.6
Graph-Based Semantic Methods: To address Type-4 clones, research shifted towards graph-based representations that capture program semantics. Control Flow Graphs (CFGs) model the flow of execution, while Program Dependence Graphs (PDGs) make both data and control dependencies explicit.5 Tools like Duplex and Scorpio have demonstrated the power of PDGs for detecting semantic clones.6 These techniques form a cornerstone of this research proposal.
Machine Learning and Deep Learning Methods: More recently, the application of machine learning, particularly deep learning models pre-trained on vast code corpora, has revolutionized the field.7 Models like CodeBERT, GraphCodeBERT, and UniXcoder can learn to "understand" code semantics by converting code fragments into high-dimensional vector embeddings.13 In this vector space, semantic similarity is represented by proximity, allowing for the detection of Type-4 clones that are beyond the reach of traditional methods.7 The integration of these models is the second major pillar of the proposed enhancement.
The trajectory of the field clearly indicates that a state-of-the-art clone detector must move beyond purely syntactic analysis and embrace a hybrid approach that combines the strengths of graph-based representations and ML-driven semantic understanding.

Part II: Graph-Based Representations for Deep Semantic Analysis

To capture the functional equivalence of Type-4 clones, it is necessary to represent code in a form that abstracts away syntax and focuses on execution flow and data dependencies. Graph-based representations, specifically Control Flow Graphs (CFGs) and Program Dependence Graphs (PDGs), are the canonical structures for this purpose.

2.1 Control Flow Graph (CFG) Integration for Functional Equivalence

A CFG is a directed graph that models all possible execution paths through a code fragment. Nodes in the graph represent basic blocks (sequences of straight-line code), and directed edges represent control flow transfers (e.g., branches, loops).16 At a conceptual level, the structure of a CFG is language-agnostic, making it a viable candidate for semantic comparison.

2.1.1 Multi-Language CFG Construction

The first step is to generate CFGs for the target languages: Rust, Python, and JavaScript.
Rust: The Rust language provides clear control flow constructs like if, loop, while, and for.18 However, there is no standard library tool for direct CFG generation. The implementation will require leveraging a third-party crate, such as
graph-flow 19 (though it is designed for workflows, not static analysis), or, more likely, building a custom CFG generator that operates on the output of a Rust parser like
syn.
Python: The Python ecosystem offers mature libraries for this task. Tools like py2cfg 20 and
pycfg 21 can directly generate CFGs from Python source code, providing a straightforward path for integration.
JavaScript: The JavaScript landscape is more fragmented. Libraries such as esgraph 22 can produce a CFG from an ESTree-compliant AST, which is a standard output format for JavaScript parsers. This provides a viable, though potentially less maintained, option.

2.1.2 CFG Comparison and Normalization

Once generated, CFGs must be compared to identify clones. The canonical method for determining if two graphs are identical is solving the graph isomorphism problem. However, subgraph isomorphism is computationally expensive (NP-hard), making it impractical for large-scale analysis.23 More scalable, heuristic-based approaches are required:
Path-Based Similarity: This technique involves extracting and comparing features based on paths within the CFG. For example, one can count the occurrences of paths of length n (n-paths) and use these counts to compute a similarity score between two graphs.16
Node-Metric Similarity: A simpler approach involves comparing metrics derived from the nodes, such as the in-degree and out-degree of corresponding nodes in two graphs. This can provide a fast, albeit coarse, measure of structural similarity.24
The most significant challenge in using CFGs for cross-language clone detection is normalization. A CFG generated from Python's for item in list: loop will be structurally different from a C-style for (int i = 0;...) loop in JavaScript or Rust's for item in iterator construct. Directly comparing these graphs is meaningless. This necessitates the creation of a unified, abstract representation of control flow constructs.
This challenge reveals a critical architectural consideration. Rather than building three separate, language-specific CFG generators and then attempting the difficult task of normalizing their outputs, a more robust and scalable strategy is to first translate the source code of all languages into a unified, language-agnostic Intermediate Representation (IR). This IR would feature generic nodes like ConditionalBranch, PreConditionLoop, PostConditionLoop, and IteratorLoop. The CFG would then be constructed from this common IR, not from the language-specific ASTs. This approach, inspired by frameworks like Amazon's "MU" 25 and POLYCALL's use of WebAssembly 26, centralizes the language-specific logic into a "frontend" translation layer, making the core analysis engine truly language-agnostic and far more maintainable.

2.2 Program Dependence Graph (PDG) for Data and Control Dependencies

The Program Dependence Graph (PDG) offers a more powerful semantic representation than the CFG. A PDG contains the same nodes as a CFG but uses edges to explicitly represent both control dependencies (which statements control the execution of others) and data dependencies (the flow of data between statements, i.e., def-use chains).27 By abstracting away the strict, and often arbitrary, sequencing of statements present in a CFG, the PDG captures the essential computational relationships within a program.
This property makes PDGs uniquely suited for detecting non-contiguous and reordered clones.27 For example, if two functions perform the same calculation but the independent statements are in a different order, their ASTs and CFGs would differ significantly, but their PDGs could be isomorphic. This is a critical advantage for identifying sophisticated Type-3 and Type-4 clones that arise from complex refactoring.

2.2.1 Multi-Language PDG Generation

Generating a PDG is more complex than generating a CFG, as it requires data-flow analysis. The tooling landscape for multi-language PDG generation is sparse. While tools like Frama-C exist for C 29, and dependency analysis tools like
multilang-depends 30 and
madge for JavaScript 31 exist, a comprehensive, off-the-shelf PDG generator for Rust, Python, and JavaScript is not readily available.
Therefore, the recommended approach is to build a custom PDG generation pipeline. This pipeline would operate on the same unified, language-agnostic IR proposed for CFG generation. General-purpose graph libraries like petgraph in Rust 32 and
networkx in Python 33 provide the foundational data structures and algorithms needed to construct, store, and manipulate these graphs.

2.2.2 Addressing the Scalability of PDG Comparison

The primary historical drawback of PDG-based clone detection is its computational complexity. As with CFGs, the canonical comparison method is subgraph isomorphism, an NP-hard problem that renders the approach non-scalable for enterprise codebases.11 This is not a minor hurdle but a fundamental blocker to practical application.
The solution lies in shifting from exact to approximate graph matching. This reframes the problem from "are these subgraphs identical?" to "how similar are these subgraphs?". This approach is not only more scalable but can also detect more clones, as it is resilient to minor structural differences.23
The state-of-the-art technique for scalable, approximate graph similarity is the Weisfeiler-Lehman (WL) Graph Kernel.23 The WL kernel is a highly efficient algorithm that computes a similarity score between two graphs by iteratively creating features that capture the topological structure around each node. In each iteration, a node's label is updated by hashing it with an aggregate of its neighbors' labels from the previous iteration. After several iterations, each node's label encodes rich information about its local neighborhood structure. The similarity between two graphs is then computed by comparing the histograms of these generated labels.34
The WL subtree kernel's runtime scales only linearly in the number of edges, making it exceptionally fast compared to isomorphism algorithms.34 This makes it feasible for large-scale analysis. The implementation can leverage existing Python libraries like
GraKeL 36 or be implemented directly within the Rust framework. This necessary shift from exact to approximate matching fundamentally changes the verification stage from a deterministic check to a probabilistic one, which aligns perfectly with the goal of integrating a machine learning classifier to weigh evidence from multiple sources.

Part III: Transformer Architectures and Code Embeddings for Semantic Similarity

In parallel to graph-based methods, a powerful approach to semantic understanding has emerged from the field of Natural Language Processing: large-scale transformer models pre-trained on code. These models learn to represent code fragments as dense numerical vectors, or embeddings, in a high-dimensional space where semantic similarity corresponds to geometric proximity.12

3.1 An Empirical Evaluation of Pre-trained Models for Code Intelligence

Several pre-trained models have demonstrated strong performance on code-related tasks, including clone detection. The selection of the base model is a critical first step.
CodeBERT: A bidirectional transformer model (based on RoBERTa) pre-trained on a massive corpus of paired natural language comments and source code from six programming languages.13 It has shown impressive results, with some studies reporting over 90% F1-score on clone detection benchmarks and particularly high recall for Type-4 semantic clones.13
GraphCodeBERT: An evolution of CodeBERT that explicitly incorporates semantic-level code structure, specifically data flow, into its pre-training tasks.14 By training the model to predict data flow edges, it learns a representation that is inherently more sensitive to the program's computational structure. This enhancement leads to state-of-the-art performance, outperforming CodeBERT on the BigCloneBench dataset.14
UniXcoder: A unified cross-modal model pre-trained to understand relationships between code, comments, and ASTs. Its architecture is designed to support both understanding and generation tasks and has been successfully adapted for cross-language clone detection.38 Its ability to map code from different languages into a shared vector space is highly relevant to the project's requirements.40
The table below summarizes the performance of leading models on the BigCloneBench dataset, a widely used (though imperfect) benchmark for Java clone detection.

Model
Base Architecture
Parameters
Key Feature
F1-Score (BigCloneBench)
Source(s)
GraphCodeBERT
BERT + Data Flow
125M
Incorporates data flow graph structure in pre-training.
0.950
14
PLBART
Seq2Seq (BART)
~140M
Pre-trained on a large corpus of Java and Python code.
0.905
7
Salesforce T5
Seq2Seq (T5)
220M
Fine-tuned T5 model for code-related tasks.
0.896
7
CodeBERT
BERT-based (RoBERTa)
125M
Pre-trained on code and natural language pairs.
0.941
15
UniXCoder
Encoder-Decoder
~125M
Unified model for code understanding and generation.
0.905
7

Based on this analysis, GraphCodeBERT is the recommended base model for this project. Its explicit use of data flow in pre-training provides a stronger semantic foundation than models that treat code as plain text, making it theoretically better suited for capturing functional equivalence.

3.2 Fine-Tuning Strategies for High-Fidelity Clone Classification

Off-the-shelf pre-trained models are not a silver bullet. Research demonstrates that while they perform well on benchmarks they were trained on, their ability to generalize to new, unseen functionalities can be poor, with recall dropping by as much as 15-40% on different datasets.15 Therefore,
fine-tuning on a task-specific dataset is not optional; it is critical for achieving production-level accuracy.
The fine-tuning process adapts the general-purpose pre-trained model for the specific task of clone classification. This involves:
Adding a Classification Head: A new, randomly initialized dense layer is added on top of the pre-trained model's output. This layer will be trained to map the code embedding to a classification decision (clone/non-clone).43
Defining a Loss Function: The model is trained using a loss function that encourages it to produce similar embeddings for clone pairs and dissimilar embeddings for non-clone pairs. For this task, Cosine Similarity Loss (based on Mean Squared Error) is a standard choice.44 For cross-language fine-tuning, a
contrastive loss objective is superior, as it explicitly trains the model to pull representations of positive pairs (clones) together while pushing negative pairs apart in the embedding space.40
Training on Labeled Data: The model is then trained on a labeled dataset of code pairs. Tutorials and frameworks from Hugging Face (Trainer API, SFTTrainer) provide a clear path for implementing this training loop.43
A significant challenge in this process is the quality of available training data. Public benchmarks like BigCloneBench are known to have issues, including class imbalance and "group leakage," where the training and test sets are not properly separated by functionality. This can lead to models that "memorize" problem types rather than learning general semantic similarity, resulting in inflated performance scores that do not translate to real-world use cases.42 A model trained on BigCloneBench might report a 94% F1-score, but when evaluated on a properly separated test set, this can plummet to below 50%.42
This observation leads to a crucial conclusion: we cannot solely rely on public benchmarks. A core component of the ML research track must be the creation and maintenance of a high-quality, internal validation dataset. This dataset must be curated from real-world codebases relevant to Uveddi's customers and must be carefully constructed to ensure a clean separation between training and testing functionalities. The success of the entire ML-based enhancement hinges on this rigorous, custom validation strategy.

3.3 A Unified Semantic Vector Space for Cross-Language Clone Detection

The ultimate objective of using code embeddings is to create a single, unified vector space where code from Rust, Python, and JavaScript can be compared directly. In this space, a Python function that sorts a list should have an embedding vector that is very close to a Rust function that does the same, despite their syntactic differences.
Models like UniXcoder 39 and techniques like C4 (CodeBERT with contrastive learning) 40 are explicitly designed to achieve this. By fine-tuning the model on cross-language clone pairs (e.g., a known Java clone and its Python equivalent), the contrastive learning objective forces the model to learn language-agnostic representations. It learns to ignore language-specific syntax and focus on the underlying algorithmic pattern, mapping both implementations to a similar region in the embedding space. This capability is essential for fulfilling the cross-language analysis requirement of the project.

Part IV: A Hybrid Architecture for Scalable, High-Precision Clone Detection

To meet the goals of high accuracy for Type-4 clones and scalability for enterprise codebases, a single analysis technique is insufficient. This section proposes a hybrid, multi-stage architecture that strategically combines the speed of embedding-based filtering with the precision of graph-based verification, all orchestrated by a final machine learning classifier.

4.1 Proposed Multi-Stage Semantic Analysis Pipeline

The proposed architecture extends the existing two-stage pipeline, preserving its performance for syntactic clones while adding new, optional stages for deep semantic analysis.
Stage 1: Fast Fingerprinting (Existing): The current Karp-Rabin rolling hash implementation remains the first line of defense. It efficiently identifies exact (Type-1) and near-exact matches, serving as a highly scalable filter for the most common types of duplication.
Stage 2: AST-Based Verification (Existing): Candidate pairs from Stage 1 are verified using normalized AST comparison. This robustly identifies Type-2 and Type-3 clones by comparing syntactic structure, ignoring identifier names and minor statement changes.
Stage 3: Semantic Candidate Filtering (New): This new stage addresses semantic clones. All code blocks (or those not matched in earlier stages) are passed through the fine-tuned GraphCodeBERT model to generate high-dimensional vector embeddings. These embeddings are stored and indexed in a specialized vector database (e.g., FAISS) that enables highly efficient Approximate Nearest Neighbor (ANN) search. This stage acts as a fast semantic filter, identifying pairs of code blocks that are semantically similar (i.e., close in the embedding space) even if they are syntactically disparate. This is computationally far cheaper than full graph analysis.
Stage 4: Deep Semantic Verification (New): Candidate pairs that pass a configurable similarity threshold from Stage 3 are promoted to this final, most rigorous verification stage. Here, the system constructs PDGs for the candidate pair and performs an approximate graph comparison using the Weisfeiler-Lehman (WL) kernel. This provides a high-confidence score based on deep structural and data-flow similarity.
This tiered pipeline design is the key to achieving both scalability and accuracy. The computationally expensive PDG analysis is only performed on a small, highly-qualified subset of code pairs that have already been identified as semantically promising by the much faster embedding-based filter. The following table provides a comparative analysis justifying this hybrid approach.
Technique
Core Principle
Strengths
Weaknesses
Computational Complexity
Scalability
CFG Isomorphism
Compares execution flow structure.
Captures control flow logic.
Ignores data flow; exact isomorphism is too rigid and slow.
NP-Hard
Low
PDG (Exact Match)
Compares data and control dependencies.
Detects non-contiguous & reordered clones; high precision.
Subgraph isomorphism is NP-Hard; misses near-miss semantic clones.
NP-Hard
Very Low
Code Embeddings
Vector similarity in a learned semantic space.
Extremely fast at scale (ANN search); good at finding Type-4 candidates; cross-language capable.
Can be imprecise; may group conceptually related but functionally different code.
O(1) per query (post-indexing)
Very High
PDG (Approx. Match)
Compares graph structure via kernels (e.g., WL kernel).
High precision; detects non-contiguous clones; robust to minor differences.
More expensive than embeddings, but far cheaper than exact matching.
O(h⋅m) (linear in edges)
High

This comparison makes the rationale for the hybrid pipeline clear: embeddings provide a scalable "search" capability, while approximate PDG matching provides a precise "verification" capability.

4.2 ML-Powered Classification and Confidence Scoring Framework

The final component of the architecture is a machine learning model that replaces simple, hard-coded thresholds with an intelligent classification system. This model acts as a meta-learner, fusing the diverse signals from all pipeline stages to make a final, informed decision. It learns the complex, non-linear relationships between different similarity metrics, allowing it to identify patterns that a rule-based system would miss. For example, it can learn that low syntactic similarity combined with high embedding and PDG similarity is a strong signal for a Type-4 clone.

4.2.1 Feature Engineering

The classifier will be trained on a rich, multi-modal feature set that provides a holistic view of code similarity 10:
Lexical & Syntactic Features (Stages 1 & 2):
Similarity score from the AST comparator.
Normalized tree edit distance between ASTs.48
Metrics like token count, line count, and cyclomatic complexity difference.6
Features from human-centric validation, such as fragment size differences and unmatched brace counts.9
Semantic Embedding Features (Stage 3):
Cosine similarity score between the code embeddings.
Graph-Based Features (Stage 4):
Similarity score from the PDG Weisfeiler-Lehman kernel comparison.23
Structural metrics from the CFG/PDG, such as node and edge counts, and node degree distributions.16
Metadata Features:
Programming language of the code fragments.
Project context information (if available).

4.2.2 Model Architecture and Output

Given the heterogeneous, tabular nature of the feature set, a complex deep learning model is not necessary for this classification task. A Gradient Boosted Decision Tree (GBDT) model, such as XGBoost or LightGBM, is highly recommended. GBDTs excel at handling such data, are highly performant, and offer good interpretability (feature importance). Alternatively, a small Artificial Neural Network (ANN) with one or two hidden layers provides a robust alternative that has been used successfully in prior work on clone validation.9
The model will be trained as a multi-class classifier. Its output will be an enhanced CloneType enum, providing both a classification and a calibrated confidence score:

Rust


// Enhanced CloneType with semantic classification and confidence
#
pub enum CloneType {
    Type1 { confidence: f64 },
    Type2 { confidence: f64, renamed_elements: Vec<String> },
    Type3 { confidence: f64, modifications: Vec<Modification> },
    Type4 { confidence: f64, semantic_similarity: SemanticSimilarity }, // NEW
}


This output is far more valuable to a developer than a simple binary decision, enabling features like filtering clones by confidence level.

4.3 Training Data Pipeline

A high-quality training data pipeline is essential for the success of the ML components.
Initial Data Sources: Public datasets like BigCloneBench 42 and SemanticCloneBench 13 will be used for initial model bootstrapping, with careful consideration of their known limitations.
Active Learning for Efficient Labeling: Manually labeling code clones is a significant bottleneck.9 To address this, an
active learning loop will be implemented.49 The system will identify clone pairs where the model is most uncertain (e.g., confidence score near 0.5) and present these to human experts for labeling. This strategy focuses human effort on the most informative examples, dramatically reducing the amount of data needed to train a high-performance model. Studies show that clustering-based acquisition functions, which select diverse and representative samples, are particularly effective for code-related tasks.49
Synthetic Data Generation: To augment the training set, especially for underrepresented clone patterns, synthetic data generation will be employed.50 This involves applying automated, semantics-preserving refactorings to existing code snippets to create new, valid Type-4 clone pairs. Examples include converting a
for loop to a while loop, changing a recursive implementation to an iterative one, or inlining a function call.

4.4 Performance, Scalability, and Incremental Analysis

To meet the stringent performance requirements for enterprise use, several optimization strategies are necessary.
ML Inference Optimization: The fine-tuned transformer model, while powerful, can be slow. To meet the <100ms per-pair inference target, it must be optimized:
Quantization: This technique reduces the precision of the model's weights (e.g., from 32-bit floats to 8-bit integers). This can lead to a 2-4x speedup and a 4x reduction in model size with minimal impact on accuracy.51
Pruning: This involves removing redundant or unimportant weights from the network, further reducing model size and computational cost.51
Knowledge Distillation: A highly effective technique is to train a much smaller "student" model (e.g., DistilBERT) to mimic the output distribution of the large, fine-tuned "teacher" model (GraphCodeBERT). This can produce a model that is 40% smaller and 60% faster while retaining over 95% of the original's performance.52
Incremental Analysis: For seamless CI/CD integration, the system must be able to analyze only the files that have changed since the last run.
The existing fingerprint index is already well-suited for this.
For the new semantic stages, a caching mechanism is required. Embeddings and graph representations (CFG/PDG) for unchanged code blocks will be cached. When a file is modified, only the graphs for the functions within that file will be incrementally reconstructed and updated in the analysis database.54

Part V: Domain-Specific and Adaptive Optimization

A generic clone detector, even a semantically aware one, may not perform optimally across all programming languages and project types. To maximize practical value, the system must be adaptive and context-aware.

5.1 Language-Specific Clone Pattern Recognition

Semantic equivalence is heavily influenced by the idioms and common patterns of a given programming language. A detector that is aware of these patterns can achieve higher accuracy. This awareness is a double-edged sword: language idioms are a primary source of syntactic divergence that creates semantic clones, but knowledge of these idiomatic transformations is also the key to detecting them. The proposed solution is to encode this knowledge into the normalization step of the unified IR generator. By mapping language-specific idioms to a canonical representation in the IR, the system can recognize their equivalence.
Rust: The analysis must account for patterns arising from Rust's unique ownership and borrowing system. For example, two functions may have identical logic but one operates on an owned value (T) while the other operates on a borrowed reference (&T). The IR normalizer should recognize these as semantically related. Other key patterns include idiomatic error handling (e.g., match on a Result vs. the ? operator) and the use of declarative macros (macro_rules!) for generating repetitive code, which is a form of intentional, structured duplication.56
Python: The dynamic nature of Python creates common clone patterns. Due to duck typing, functions are often written to operate on any object that supports a certain interface (e.g., has a .read() or __len__() method), leading to functional similarity across different types. The syntactic sugar of list comprehensions versus explicit for loops is another classic example of a Type-4 clone that the IR must normalize.59 Common Python code smells related to duplication often have well-known refactoring patterns that can inform the feature engineering process.61
JavaScript: Asynchronous programming is a major source of semantic clones in JavaScript. Logic implemented with callback chains, .then()-based Promise chains, and modern async/await syntax can all be functionally equivalent but are syntactically distinct. The IR normalizer must be able to canonicalize these different asynchronous patterns. Other patterns arise from the use of closures for state management and the historical shift from prototype-based inheritance to ES6 classes.63

5.2 Adaptive Threshold and Parameter Optimization

Hard-coded similarity thresholds are brittle and suboptimal. A threshold that works well for a mature, stable enterprise application may be too strict for a rapidly evolving prototype. The system's parameters must adapt to the characteristics of the codebase being analyzed.
The recommended approach for this is automated hyperparameter optimization. This treats the entire analysis pipeline as a black-box function whose inputs are configuration parameters (e.g., similarity thresholds, ML model settings) and whose output is a performance metric (e.g., F1-score on a validation set).
Bayesian Optimization (BO): This is the ideal technique for this problem. BO is a data-efficient global optimization strategy for functions that are expensive to evaluate, which perfectly describes running a full clone analysis pipeline.65 It works by building a probabilistic surrogate model (typically a Gaussian Process) of the objective function. It then uses an acquisition function (e.g., Expected Improvement) to intelligently select the next set of parameters to evaluate, balancing exploration of the parameter space with exploitation of known good regions. This allows it to find near-optimal configurations with far fewer evaluations than brute-force methods.65 Python libraries like
bayes_opt 68 or
scikit-optimize 67 provide ready-to-use implementations.
Grid Search and Random Search: These are simpler, less efficient methods that can serve as baselines for comparison. Grid search exhaustively tries all combinations of a discrete set of parameters, while random search samples them randomly. Both are viable but far less efficient than Bayesian Optimization for this use case.69
Reinforcement Learning (RL): As a more advanced, future-looking approach, reinforcement learning could be used to train an agent that learns an optimal policy for setting analysis parameters. The agent's state could include characteristics of the codebase (e.g., size, language distribution), and it would receive rewards based on developer feedback (e.g., a positive reward when a suggested clone is refactored, a negative reward when it is ignored).70 This would create a truly adaptive system that learns from user interaction over time.

Part VI: Validation, Benchmarking, and Implementation Roadmap

This final section outlines the actionable plan for building, validating, and deploying the proposed enhancements, ensuring that the research translates into a robust and valuable product feature.

6.1 A Rigorous Validation and Benchmarking Protocol

A systematic and rigorous validation framework is essential to measure the success of the enhancements and to guide the development process.

6.1.1 Accuracy Measurement

The primary goal is to improve detection accuracy, especially for semantic clones, while reducing false positives.
Datasets: A multi-pronged dataset strategy will be used:
Standard Benchmarks: Initial development and baseline comparisons will use public datasets like BigCloneBench 42 and SemanticCloneBench.13
Internal Validation Set: A new, high-quality validation set will be created internally. This set will consist of curated examples of all four clone types from real-world, large-scale projects in Rust, Python, and JavaScript. This dataset is crucial for mitigating the known issues of public benchmarks and ensuring the model generalizes to production code.42
Metrics and Targets: Performance will be measured using standard classification metrics: Precision, Recall, and F1-score, calculated for each clone type individually and overall.2 The key success metrics for accuracy are:
Type-4 Detection: Achieve >70% recall on the Type-4 clone portion of the internal validation set.
False Positive Reduction: Reduce the overall false positive rate by >30% compared to the baseline, driven by the ML classifier.
Cross-Language Accuracy: Maintain >85% F1-score across all three supported languages.

6.1.2 Performance Benchmarking

The enhancements must not introduce prohibitive performance overhead.
Methodology: A dedicated performance benchmark suite will be created using a set of open-source and internal codebases of varying sizes (e.g., 10k, 50k, 100k, and 500k files).
Metrics and Targets:
Scalability: The full analysis (including semantic stages) on a 100k+ file codebase should complete within 2x the time of the current syntactic-only analysis. Semantic analysis should add <50% overhead.
Memory Efficiency: Total memory usage for large codebase analysis should remain under 2 GB.
Incremental Analysis: Processing a typical pull request (e.g., 10 modified files) should complete within 30 seconds.
Real-time Classification: ML model inference time must be <100ms per clone pair.

6.2 Phased Implementation and Integration Plan

A phased implementation approach is recommended to manage complexity and deliver value incrementally.
Phase 1: Graph Generation and Analysis Prototype
Objective: Build the core graph-based semantic analysis engine.
Tasks:
Design and implement the language-agnostic Intermediate Representation (IR).
Develop the "frontend" parsers to translate Rust, Python, and JavaScript ASTs into the unified IR.
Implement the custom PDG builder that operates on the IR.
Implement and validate the Weisfeiler-Lehman (WL) kernel for approximate PDG similarity scoring.
Validation: Benchmark the standalone PDG/WL component for accuracy on Type-4 clones and measure its performance characteristics.
Phase 2: Embedding Model Integration and Fine-Tuning
Objective: Develop the fast semantic filtering stage.
Tasks:
Select and acquire the base pre-trained model (GraphCodeBERT recommended).
Set up the fine-tuning pipeline using the internal labeled dataset and contrastive loss.
Apply inference optimization techniques (quantization, distillation) to the fine-tuned model.
Integrate the optimized model and a vector search index (e.g., FAISS) as Stage 3 of the detection pipeline.
Validation: Evaluate the effectiveness of the embedding filter in identifying semantic clone candidates and measure its end-to-end latency.
Phase 3: ML Classifier and Full System Integration
Objective: Unify all analysis signals and enable adaptive tuning.
Tasks:
Develop the feature engineering framework to extract signals from all four pipeline stages.
Train, validate, and tune the final GBDT or ANN classifier.
Integrate all stages into the final hybrid architecture, with the ML model making the final classification.
Implement the adaptive parameter tuning framework using Bayesian Optimization.
Validation: Conduct a full system benchmark against all accuracy and performance targets.
Phase 4: Production Rollout and Monitoring
Objective: Deploy the new capabilities to users and establish a feedback loop.
Tasks:
Deploy the enhanced semantic analysis as an optional, configurable feature to allow for gradual adoption.
Implement detailed monitoring and observability for the performance and accuracy of all new components.
Establish a feedback mechanism for developers to confirm or reject clone suggestions, feeding this data back into the active learning loop for continuous model improvement.

Conclusion and Future Research Directions

The research and development plan outlined in this report provides a robust, evidence-based roadmap for transforming the Uveddi code duplication detector into a cutting-edge semantic analysis platform. By adopting a hybrid, multi-stage architecture that combines the scalability of code embeddings with the precision of Program Dependence Graph analysis, the system can effectively and efficiently identify elusive Type-4 semantic clones. The integration of a machine learning classifier as a meta-learner, trained on a rich set of heterogeneous features and refined through active learning, will deliver superior accuracy and reduce false positives. Furthermore, the development of a unified, language-agnostic Intermediate Representation and an adaptive parameter tuning framework ensures that the solution is scalable, maintainable, and optimized for diverse software projects.
Upon successful implementation, this enhanced system will provide developers with significantly more powerful and actionable insights, enabling them to improve code quality, reduce maintenance burdens, and build more robust software.
Future research can extend this work in several promising directions:
Expansion to More Languages: The unified IR architecture is designed for extensibility. Adding support for new languages like Go, C++, or TypeScript would primarily involve developing a new "frontend" translator for that language.
Automated Refactoring Suggestions: The system could be extended beyond detection to suggest specific refactoring actions based on the type and context of the identified clones.
Temporal Clone Analysis: Analyzing the evolution of clones over a project's history could reveal important insights into technical debt accumulation and architectural decay.
Deeper IDE Integration: Real-time feedback within the IDE, flagging potential semantic clones as they are being written, could prevent duplication before it enters the codebase.
Works cited
To Enhance Type 4 Clone Detection in Clone Testing, accessed July 13, 2025, https://www.ijcsit.com/docs/Volume%207/vol7issue2/ijcsit20160702121.pdf
A Comprehensive Review of Code Clone Detection Techniques - ijltemas, accessed July 13, 2025, https://www.ijltemas.in/DigitalLibrary/Vol.4Issue12/43-47.pdf
Comparison and Evaluation of Code Clone Detection ... - CiteSeerX, accessed July 13, 2025, https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=fb35f65339c4f1bbe9c0bd25f09f83e65e436d51
The Survey of the Code Clone Detection Techniques and Process With Types (I, II, III And IV) - CORE, accessed July 13, 2025, https://core.ac.uk/download/539895020.pdf
Clone-Types 1 to Type 4 | Download Scientific Diagram - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/figure/Clone-Types-1-to-Type-4_fig2_335152710
(PDF) Survey on Software code clone detection - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/360015247_Survey_on_Software_code_clone_detection
Evaluating Small-Scale Code Models for Code Clone Detection - arXiv, accessed July 13, 2025, https://www.arxiv.org/pdf/2506.10995
Fine-Tuning Pre-Trained CodeBERT for Code Search in Smart Contract, accessed July 13, 2025, https://wujns.edpsciences.org/articles/wujns/full_html/2023/03/wujns-1007-1202-2023-03-0237-09/wujns-1007-1202-2023-03-0237-09.html
CloneCognition: Machine Learning Based Code Clone Validation Tool - Chanchal Roy, accessed July 13, 2025, https://clones.usask.ca/pubfiles/articles/MostaeenCloneCognitionFSE2019.pdf
Deep Learning Code Fragments for Code Clone Detection - Michele Tufano, accessed July 13, 2025, https://tufanomichele.com/publications/C5.pdf
Identifying Similar Code with Program Dependence Graphs | Request PDF - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/3919783_Identifying_Similar_Code_with_Program_Dependence_Graphs
Deep Learning for Code Intelligence: Survey, Benchmark and Toolkit - arXiv, accessed July 13, 2025, https://arxiv.org/html/2401.00288v1
Interpreting CodeBERT for semantic code clone detection - InK@SMU.edu.sg, accessed July 13, 2025, https://ink.library.smu.edu.sg/cgi/viewcontent.cgi?article=10313&context=sis_research
graphcodebert: pre-training code represen, accessed July 13, 2025, https://arxiv.org/abs/2009.08366
CodeBERT for code clone detection: A ... - InK@SMU.edu.sg, accessed July 13, 2025, https://ink.library.smu.edu.sg/cgi/viewcontent.cgi?article=11175&context=sis_research
Enhancing code clone detection using control flow graphs - SciSpace, accessed July 13, 2025, https://scispace.com/pdf/enhancing-code-clone-detection-using-control-flow-graphs-4h8tqyavnt.pdf
Control-flow graph - Wikipedia, accessed July 13, 2025, https://en.wikipedia.org/wiki/Control-flow_graph
Control Flow - The Rust Programming Language, accessed July 13, 2025, https://doc.rust-lang.org/book/ch03-05-control-flow.html
graph-flow - crates.io: Rust Package Registry, accessed July 13, 2025, https://crates.io/crates/graph-flow
py2cfg·PyPI, accessed July 13, 2025, https://pypi.org/project/py2cfg/
Draw Control Flow Graph using pycfg | Python - GeeksforGeeks, accessed July 13, 2025, https://www.geeksforgeeks.org/python/draw-control-flow-graph-using-pycfg-python/
control flow graph - npm search, accessed July 13, 2025, https://www.npmjs.com/search?q=control%20flow%20graph
CCGraph: a PDG-based code clone detector with approximate graph matching - Yinxing Xue's Homepage, accessed July 13, 2025, https://yinxingxue.github.io/papers/ase2020_CCGraph%20A%20PDG%20based%20Code%20Clone%20Detector%20With%20Approximate%20Graph%20Matching.pdf
Code Clone Detection using Graphs and Adjacency Structures - ijltemas, accessed July 13, 2025, https://www.ijltemas.in/DigitalLibrary/Vol.7Issue1/191-195.pdf
A language-agnostic framework for mining static analysis rules from ..., accessed July 13, 2025, https://www.amazon.science/publications/a-language-agnostic-framework-for-mining-static-analysis-rules-from-code-changes
Effective Call Graph Construction for Multilingual Programs, accessed July 13, 2025, https://csslab-ustc.github.io/publications/2023/call-graph.pdf
Tool Demonstration: Finding Duplicated Code Using ... - CiteSeerX, accessed July 13, 2025, https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=aff3b6fee24553b99930dd11ba35f7b749b52bfd
The Program Dependence Graph and Its Use in Optimization - UT Computer Science, accessed July 13, 2025, https://www.cs.utexas.edu/~pingali/CS395T/2009fa/papers/ferrante87.pdf
which free tools can I use to generate the program dependence graph for c codes - Stack Overflow, accessed July 13, 2025, https://stackoverflow.com/questions/9804591/which-free-tools-can-i-use-to-generate-the-program-dependence-graph-for-c-codes
multilang-depends/depends: Depends is a fast, comprehensive code dependency analysis tool - GitHub, accessed July 13, 2025, https://github.com/multilang-depends/depends
pahen/madge: Create graphs from your CommonJS, AMD ... - GitHub, accessed July 13, 2025, https://github.com/pahen/madge
petgraph - Rust - Docs.rs, accessed July 13, 2025, https://docs.rs/petgraph/
Build a dependency graph in python - Stack Overflow, accessed July 13, 2025, https://stackoverflow.com/questions/14242295/build-a-dependency-graph-in-python
Weisfeiler-Lehman Graph Kernels - Journal of Machine Learning ..., accessed July 13, 2025, https://www.jmlr.org/papers/volume12/shervashidze11a/shervashidze11a.pdf
Unlocking Weisfeiler-Lehman Graph Kernels - Number Analytics, accessed July 13, 2025, https://www.numberanalytics.com/blog/weisfeiler-lehman-graph-kernels-topological-machine-learning
Weisfeiler Lehman Framework — GraKeL 0.1a7 documentation, accessed July 13, 2025, https://ysig.github.io/GraKeL/0.1a7/kernels/weisfeiler_lehman.html
ysig/GraKeL: A scikit-learn compatible library for graph kernels - GitHub, accessed July 13, 2025, https://github.com/ysig/GraKeL
AdaCCD: Adaptive Semantic Contrasts Discovery Based Cross ..., accessed July 13, 2025, https://arxiv.org/pdf/2311.07277
The Struggles of LLMs in Cross-Lingual Code Clone Detection - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/392855604_The_Struggles_of_LLMs_in_Cross-Lingual_Code_Clone_Detection
C4: Contrastive Cross-Language Code Clone Detection - Xing Hu's, accessed July 13, 2025, https://xing-hu.github.io/assets/papers/icpc-c8.pdf
CCT: Cross-consistency training for Clone Detection and Code Search Tasks | OpenReview, accessed July 13, 2025, https://openreview.net/forum?id=aLgoMwjNDsi
(PDF) Generalizability of Code Clone Detection on CodeBERT, accessed July 13, 2025, https://www.researchgate.net/publication/363052432_Generalizability_of_Code_Clone_Detection_on_CodeBERT
Fine-tuning - Hugging Face, accessed July 13, 2025, https://huggingface.co/docs/transformers/training
Fine-tuning BERT for Semantic Textual Similarity with Transformers ..., accessed July 13, 2025, https://thepythoncode.com/article/finetune-bert-for-semantic-textual-similarity-in-python
Finetune Transformers - George Mihaila, accessed July 13, 2025, https://gmihaila.github.io/tutorial_notebooks/finetune_transformers_pytorch/
huggingface/trl: Train transformer language models with reinforcement learning. - GitHub, accessed July 13, 2025, https://github.com/huggingface/trl
Supervised Deep Features for Software Functional Clone Detection by Exploiting Lexical and Syntactical Information in Source Code - IJCAI, accessed July 13, 2025, https://www.ijcai.org/proceedings/2017/0423.pdf
Revisiting Code Similarity Evaluation with Abstract Syntax Tree Edit Distance - arXiv, accessed July 13, 2025, https://arxiv.org/html/2404.08817v1
Active code learning: Benchmarking sample-efficient training of code models - InK@SMU.edu.sg, accessed July 13, 2025, https://ink.library.smu.edu.sg/cgi/viewcontent.cgi?article=9698&context=sis_research
What is Synthetic Data Generation? A Practical Guide - K2view, accessed July 13, 2025, https://www.k2view.com/what-is-synthetic-data-generation/
Ultimate Guide to LLM Inference Optimization - Ghost, accessed July 13, 2025, https://latitude-blog.ghost.io/blog/ultimate-guide-to-llm-inference-optimization/
DistilBERT in Natural Language Processing - GeeksforGeeks, accessed July 13, 2025, https://www.geeksforgeeks.org/nlp/distilbert-in-natural-language-processing/
Autocorrelation Matrix Knowledge Distillation: A Task-Specific Distillation Method for BERT Models - MDPI, accessed July 13, 2025, https://www.mdpi.com/2076-3417/14/20/9180
accessed December 31, 1969, https.bing.com/search?q=incremental+control+flow+graph+analysis
[POPL'25] An Incremental Algorithm for Algebraic Program Analysis - YouTube, accessed July 13, 2025, https://www.youtube.com/watch?v=o_PCPFO2M6s
Design Patterns in Rust - Reddit, accessed July 13, 2025, https://www.reddit.com/r/rust/comments/1aol909/design_patterns_in_rust/
Duplicate v0.2.2: Easy code duplication : r/rust - Reddit, accessed July 13, 2025, https://www.reddit.com/r/rust/comments/gmrjqv/duplicate_v022_easy_code_duplication/
Don't duplicate common properties - help - The Rust Programming Language Forum, accessed July 13, 2025, https://users.rust-lang.org/t/dont-duplicate-common-properties/53982
Overcoming Code Smell A Python Developer's Guide to Refactoring - MoldStud, accessed July 13, 2025, https://moldstud.com/articles/p-overcoming-code-smell-a-python-developers-guide-to-refactoring
Code Style - The Hitchhiker's Guide to Python, accessed July 13, 2025, https://docs.python-guide.org/writing/style/
Duplicate Code - Refactoring.Guru, accessed July 13, 2025, https://refactoring.guru/smells/duplicate-code
How to do Code Smells Refactoring in Python the Right Way - Bito AI, accessed July 13, 2025, https://bito.ai/blog/how-to-do-code-smells-refactoring-in-python/
Code duplication anti-pattern, Diagnosis and remediation | by Ahmed Kooli - Medium, accessed July 13, 2025, https://medium.com/@kooliahmd/code-duplication-anti-pattern-diagnosis-and-treatment-44f8c1555382
How To Remove Duplicate Code Properly From Your App - JavaScript in Plain English, accessed July 13, 2025, https://javascript.plainenglish.io/how-to-use-the-dry-principle-properly-21fd354b48c3
Bayesian optimization - Wikipedia, accessed July 13, 2025, https://en.wikipedia.org/wiki/Bayesian_optimization
Integrating Bayesian Optimization and Machine Learning for the Optimal Configuration of Cloud Systems - IEEE Computer Society, accessed July 13, 2025, https://www.computer.org/csdl/journal/cc/2024/01/10418550/1Ubl1i0d0Zy
A Comprehensive Guide to Practical Bayesian Optimization ..., accessed July 13, 2025, https://www.numberanalytics.com/blog/comprehensive-guide-practical-bayesian-optimization-implementation
bayesian-optimization/BayesianOptimization: A Python implementation of global optimization with gaussian processes. - GitHub, accessed July 13, 2025, https://github.com/bayesian-optimization/BayesianOptimization
Hyperparameter optimization - Wikipedia, accessed July 13, 2025, https://en.wikipedia.org/wiki/Hyperparameter_optimization
DeepSWE: Training a Fully Open-sourced, State-of-the-Art Coding Agent by Scaling RL, accessed July 13, 2025, https://www.together.ai/blog/deepswe
