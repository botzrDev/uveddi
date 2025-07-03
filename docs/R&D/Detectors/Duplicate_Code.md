
A Research Report on the Design and Implementation of a Scalable, Multi-Language Code Duplication Detector for the Uveddi Architectural Analysis Tool


Part 1: Foundational Theory and Algorithmic Landscape of Code Clone Detection

This section establishes the theoretical and algorithmic foundation for developing a sophisticated code duplication detector. It provides a definitive classification of code clone types, surveys the primary detection methodologies, offers a deep dive into key algorithms, analyzes existing industry implementations, and culminates in a recommended algorithm stack tailored for the Uveddi architectural analysis tool. A thorough understanding of these fundamentals is paramount for making sound architectural and implementation decisions.

1.1 A Definitive Taxonomy of Code Clones

The term "code clone" refers to code fragments that are identical or similar. The precise definition of "similar" is crucial, as it dictates the detection methodology and the nature of the findings. The academic and industry consensus categorizes clones into four primary types, which represent a spectrum from exact textual identity to pure functional equivalence.2 This taxonomy provides a critical shared vocabulary for discussing and implementing the detector.
Type-1 (Exact Clones): These are code fragments that are identical, with allowances only for variations in whitespace, comments, and layout.4 They are often the result of direct copy-and-paste operations. Despite potential differences in a text-based
diff, they are considered the strictest form of clone because they are textually identical after trivial normalization (e.g., removing comments and standardizing whitespace).
Type-2 (Renamed/Parameterized Clones): These fragments are structurally and syntactically identical, but differ in the names of identifiers (variables, types, functions) and literal values (numbers, strings).4 Detecting these clones requires a level of syntactic understanding beyond simple text comparison, typically involving a parsing step to abstract away specific names and values.5
Type-3 (Near-Miss/Gapped Clones): These are copied fragments that have been further modified. In addition to the variations allowed in Type-2 clones, Type-3 clones may contain added, removed, or altered statements.7 These are considered the most prevalent type of clone in many software systems and present a significant challenge for detection tools.7 To provide more nuance, researchers have further subdivided Type-3 clones based on syntactic similarity thresholds, as used in the BigCloneBench dataset 4:
Very-Strongly Type-3 (VST3): 90% to <100% similarity.
Strongly Type-3 (ST3): 70% to <90% similarity.
Moderately Type-3 (MT3): 50% to <70% similarity.
These fine-grained categories are essential for configuring meaningful detection thresholds and prioritizing results.
Type-4 (Semantic Clones): These are code fragments that are syntactically dissimilar but implement the same functionality.1 For example, an iterative and a recursive implementation of the same algorithm would be considered a Type-4 clone pair. Detecting these clones is the most challenging task, as it requires deep semantic analysis of the code's behavior, often involving techniques like Program Dependence Graph (PDG) analysis or advanced machine learning models.4

1.2 A Survey of Detection Methodologies

Corresponding to the clone taxonomy, several distinct methodologies have been developed, each with its own trade-offs in terms of performance, complexity, and the types of clones it can effectively identify.
Textual Approaches: These methods treat source code as raw text or a sequence of lines.9 They are simple to implement and computationally fast but are generally limited to detecting Type-1 clones. They are brittle and easily defeated by minor code changes like reformatting or renaming variables.9
Token-Based Approaches: These methods first use a lexer to transform the source code into a sequence of tokens.3 This token stream is then normalized (e.g., by replacing all identifiers with a placeholder) and scanned for duplicated subsequences. This approach is resilient to changes in whitespace, comments, and, with normalization, identifier names. It is effective for detecting Type-1 and Type-2 clones and some Type-3 clones.5 Prominent tools like PMD's CPD and CCFinder utilize this technique.3
Tree-Based (Syntactic) Approaches: These methods parse the source code into an Abstract Syntax Tree (AST).10 The problem of clone detection is then transformed into finding similar or identical subtrees within the ASTs of the codebase.3 This approach is robust against syntactic variations and is considered the most reliable method for detecting Type-2 and a wide range of Type-3 clones with high precision.7
Graph-Based (Semantic) Approaches: For detecting semantic clones, the code's control and data dependencies are modeled using a Program Dependence Graph (PDG) or similar graph structures.10 Clones are then identified by searching for isomorphic subgraphs, which represent functionally equivalent computations.1 While powerful enough to detect non-contiguous and Type-4 clones, these methods are computationally intensive and significantly more complex to implement and scale.12
Hybrid Approaches: Recognizing that no single method is optimal, modern, scalable tools often employ a hybrid approach.7 A common and effective pattern is to use a fast but less precise method (like token-based hashing) as a first-pass filter to generate a small set of candidate clones. These candidates are then verified using a slower but more accurate method (like AST comparison).7 This multi-stage process balances the need for both scalability and accuracy.

1.3 Algorithmic Deep Dive

A successful implementation requires a detailed understanding of the core algorithms that power these methodologies.

1.3.1 Fingerprinting with the Karp-Rabin Algorithm

The Karp-Rabin algorithm is a string-searching algorithm that uses hashing to find matches. Its application to clone detection involves treating a sequence of normalized code tokens as a string and using a rolling hash to efficiently compute fingerprints for code blocks.13
Description: The algorithm slides a window of a fixed size over the token stream. For each window, it computes a hash value. The key innovation is the "rolling hash," which allows the hash of the next window to be calculated in constant time from the previous window's hash, avoiding a full re-computation.14 Code blocks that produce the same set of hash values are considered strong candidates for being clones.
Pseudocode (Conceptual):
function find_clone_candidates(codebase, min_tokens):
  fingerprint_map = new map<hash, list<location>>
  for each file in codebase:
    tokens = tokenize_and_normalize(file)
    for each block in file (e.g., functions):
      token_sequence = get_tokens(block)
      if length(token_sequence) < min_tokens:
        continue

      // Generate fingerprints using a rolling hash
      rolling_hash = new RollingHash(window_size)
      for token in token_sequence:
        hash_value = rolling_hash.update(token)
        fingerprint_map[hash_value].append(block.location)

  // Post-process fingerprint_map to find blocks with high hash overlap
  candidates = find_blocks_with_shared_fingerprints(fingerprint_map)
  return candidates


Complexity Analysis:
Time: The expected time complexity for fingerprinting the entire codebase is O(T), where T is the total number of tokens. This is because the rolling hash operates in constant time per token. This makes it exceptionally well-suited for a fast, scalable filtering stage.14
Space: The space complexity is dominated by the fingerprint map, which can be large but is manageable. It is proportional to the number of unique fingerprints generated.
Implementation Difficulty: Moderate. Requires a solid implementation of a rolling hash function and a robust tokenization and normalization pipeline.

1.3.2 Structural Matching with Suffix Trees

Suffix trees are a powerful data structure for string processing that can be adapted for clone detection by finding repeated substrings in a concatenated token stream from the entire codebase.3
Description: All normalized token sequences from the codebase are concatenated into one large sequence. A generalized suffix tree is built from this sequence. Any internal node in the tree with multiple children represents a sequence that occurs more than once, i.e., a clone. The path from the root to that node spells out the cloned sequence.17
Complexity Analysis:
Time: With Ukkonen's algorithm, a suffix tree can be built in O(T) time, where T is the total length of the concatenated token stream.17
Space: The space complexity is also O(T). In practice, suffix trees can be very memory-intensive, often requiring 10-20 times the size of the input text, which can be a significant drawback for large codebases.19
Implementation Difficulty: High. Ukkonen's algorithm is notoriously complex to implement correctly. While powerful, the high implementation complexity and memory overhead make it less practical for this use case compared to hashing-based indexing approaches.20

1.3.3 Vector-Space Models and LSH (DECKARD)

The DECKARD tool introduced a novel approach for detecting near-miss (Type-3) clones by moving from direct tree comparison to a vector-space model.21
Description: The core idea is to transform AST subtrees into numerical vectors, called "characteristic vectors." Each dimension of the vector corresponds to the count of a specific type of AST node (e.g., if_statement, binary_expression) within that subtree. Once all relevant subtrees are converted into vectors, the problem of finding similar trees becomes one of finding clusters of nearby vectors in a high-dimensional space. To do this efficiently, DECKARD uses Locality-Sensitive Hashing (LSH), a technique that hashes similar items to the same bucket with high probability.21
Pseudocode (Conceptual):
function find_clones_with_deckard(codebase):
  all_vectors =
  for each file in codebase:
    ast = parse_file(file)
    for each subtree in ast:
      if size(subtree) > min_size:
        vector = generate_characteristic_vector(subtree)
        all_vectors.append(vector)

  // Index vectors using LSH
  lsh_index = new LSHIndex(dimensions, num_hash_tables)
  lsh_index.insert(all_vectors)

  // Query for clusters
  clone_clusters =
  for vector in all_vectors:
    neighbors = lsh_index.query(vector, similarity_threshold)
    if len(neighbors) > 1:
      clone_clusters.add(neighbors)

  return post_process(clone_clusters)


Complexity Analysis:
Time: Vector generation is linear in the number of AST nodes. The LSH-based clustering is sub-linear, significantly faster than the quadratic pairwise comparison of all vectors.21
Space: Requires space for the ASTs and the generated vectors, plus the LSH index data structures.
Implementation Difficulty: High. Requires a deep understanding of vectorization, LSH, and clustering algorithms. However, libraries for LSH are available, which can reduce the implementation burden.

1.3.4 Program Dependence Graphs (PDG) for Semantic Clones

PDGs capture the control and data flow dependencies within a program, providing a representation of its semantics.10
Description: To find semantic clones, PDGs are constructed for code fragments (e.g., functions). The problem then becomes finding isomorphic subgraphs between two PDGs. Two code fragments with isomorphic PDG subgraphs are considered semantic clones because they have equivalent data and control dependencies, even if their syntax is different.12
Complexity Analysis:
Time: PDG construction is complex and time-consuming. The subgraph isomorphism problem is NP-complete in the general case. While heuristics exist for program graphs, it remains a computationally very expensive operation.12
Space: PDGs are typically larger and more complex than ASTs.
Implementation Difficulty: Very High. Requires a full semantic analysis engine capable of performing control-flow and data-flow analysis for each language. This is significantly beyond the scope of a standard Tree-sitter-based analysis and is generally not considered scalable for large systems.12

1.4 Analysis of Industry Implementations

Examining how leading industry tools implement clone detection provides invaluable practical insights.
SonarQube (sonar-cpd): SonarSource made a deliberate decision to replace the third-party PMD-CPD tool with their own sonar-cpd library. This was motivated by a need for better scalability, reduced resource consumption, and greater control over the detection algorithm to minimize false positives.23 Their approach is "statement-based," which appears to be a hybrid that uses language-specific knowledge to group tokens into meaningful statements before comparison. This allows them to effectively detect Type-2 and partial Type-3 clones while filtering out noise like blocks of
import statements.23 A critical lesson from their experience is the high volume of false positives generated in JavaScript code by ignoring string literals, which led to user complaints and the feature being effectively unusable for some teams.25 This underscores the importance of intelligent, context-aware normalization.
PMD (Copy/Paste Detector - CPD): PMD's CPD is a widely used, mature tool. It has evolved through several algorithms, ultimately settling on the efficient Karp-Rabin algorithm applied to a stream of tokens.11 Its key strength is its simple, robust model and extensive language support. Its command-line flags, such as
--ignore-literals and --ignore-identifiers, provide a clear, practical blueprint for how to implement Type-2 clone detection on top of a token-based system by normalizing the token stream before hashing and comparison.11
Simian: Simian is a commercial tool that also operates on a tokenized representation of the source code. It achieves language independence by ignoring language-specific syntax like curly braces and focusing on the sequence of significant tokens.27 Its primary features highlight the importance of user configurability, such as setting the minimum line threshold and options to ignore identifiers, literals, and character case.

1.5 Recommendation: Optimal Algorithm Stack for Uveddi

No single algorithm satisfies all of Uveddi's requirements for scalability, accuracy, and multi-language support. A naive approach of comparing every code block with every other block would have a time complexity of at least O(N2), where N is the number of blocks, which is computationally infeasible for 100k+ files. Therefore, the most critical architectural decision is to adopt a multi-stage, hybrid architecture that drastically reduces the number of required comparisons. The "algorithm" for a system of this scale is, in fact, the entire data processing pipeline.
The recommended architecture consists of two primary stages:
Stage 1: Candidate Generation (Fast Filtering)
The goal of this stage is to quickly and efficiently eliminate the vast majority of code block pairs that cannot possibly be clones, producing a small set of high-probability candidates.
Methodology: A token-based hashing approach is recommended.
Process:
Granularity: The analysis unit will be the function/method level. Tree-sitter queries will be used to extract these blocks from each source file.
Normalization: Each block's source text will be tokenized. A first level of normalization will occur here: comments and whitespace are ignored, and identifiers and literals are abstracted to placeholder tokens (e.g., _ID_, _LIT_). This makes the process resilient to Type-2 clone variations.
Fingerprinting: A rolling hash algorithm (e.g., Karp-Rabin) will be used to generate a set of hash fingerprints for the normalized token stream of each block.
Indexing: These fingerprints will be stored in a highly optimized, persistent index (e.g., an inverted index in the PostgreSQL database). The index will map a fingerprint hash to a list of code blocks that contain it.
Outcome: To find candidates for a given block, the system queries the index with its fingerprints. Any other blocks that share a number of fingerprints above a configurable threshold become part of a candidate clone pair. This reduces the search space from all possible pairs to a very small, manageable subset. This approach is inspired by the proven scalability of tools like SourcererCC.28
Stage 2: Candidate Verification (Accurate Comparison)
This stage takes the small set of candidate pairs from Stage 1 and performs a more computationally expensive but far more precise comparison to confirm the clone and eliminate false positives.
Methodology: An AST-based structural comparison is recommended.
Process:
Parsing: For each candidate pair, the original source code for the two blocks is parsed using Tree-sitter to generate their respective ASTs.
Normalization: The ASTs are normalized by pruning irrelevant nodes (e.g., punctuation) and abstracting identifiers and literals (if not already handled in the token stream).
Comparison: A structural hash is computed for each normalized AST subtree. If the hashes of the two root nodes are identical, they are a high-confidence Type-2 clone. For Type-3 clones, a more nuanced similarity metric is needed. Rather than a full tree edit distance, a more pragmatic approach is to compare the characteristic vectors of the two ASTs (as in DECKARD 21) and calculate a similarity score (e.g., Jaccard similarity or cosine similarity of the vectors).
Outcome: Each candidate pair is either confirmed as a clone (and assigned a similarity score and type) or rejected as a false positive.
This two-stage architecture provides the optimal balance. Stage 1 delivers the necessary speed and scalability to handle enterprise-sized codebases by avoiding a quadratic explosion in comparisons. Stage 2 provides the syntactic accuracy of AST analysis to ensure the final results are precise, actionable, and have a low false-positive rate, directly addressing the core requirements for the Uveddi tool.

Table 1: Comparative Analysis of Core Clone Detection Algorithms

Algorithm/Approach
Detects Clone Types
Time Complexity
Space Complexity
Implementation Difficulty
Key Strengths
Key Weaknesses
Line-based Matching
Type-1
O(L) (with hashing)
O(L)
Low
Very fast, simple to implement.
Brittle, fails with minor formatting or code changes. High false negatives.
Token Hashing (Karp-Rabin)
Type-1, Type-2, some Type-3
O(T) (expected)
O(T)
Moderate
Excellent scalability, resilient to formatting/naming changes.
Can miss structural clones (Type-3) with reordered statements.
Suffix Tree
Type-1, Type-2, some Type-3
O(T)
O(T)
High
Finds longest repeated sequences efficiently.
High memory consumption, complex to implement correctly.
AST Structural Hashing
Type-1, Type-2
O(N)
O(N)
Moderate
Very accurate for structural identity, fast comparison.
Does not handle near-misses (Type-3) without modifications.
DECKARD (AST Vectors + LSH)
Type-2, Type-3
Sub-linear query time
O(N⋅d)
High
Scalable detection of near-miss clones, robust to statement changes.
Complex to tune, vector representation can lose some structural info.
PDG Subgraph Isomorphism
Type-4, non-contiguous
NP-complete (worst-case)
O(V+E)
Very High
Can detect semantic clones, language-independent in theory.
Extremely poor scalability, very high implementation barrier.

L = Total lines of code, T = Total tokens, N = Total AST nodes, d = vector dimensions, V/E = vertices/edges in PDG.

Part 2: Strategies for Tree-sitter Integration and AST Comparison

Integrating Tree-sitter is central to the Uveddi detector's multi-language capabilities. However, effective clone detection requires more than just parsing; it demands a sophisticated strategy for normalizing and comparing the resulting syntax trees, especially in a polyglot environment. This section details the practical techniques for transforming language-specific parse trees into a comparable, canonical form.

2.1 The Challenge: From Concrete Syntax Tree to a Normalized Representation

A common misconception is that Tree-sitter provides a universal Abstract Syntax Tree (AST) out of the box. In reality, it produces a Concrete Syntax Tree (CST), which is a much more detailed representation of the source code, including all syntactic elements like parentheses, commas, and semicolons.29 Furthermore, each language has its own community-maintained grammar, resulting in different node types and tree structures for semantically similar constructs (e.g., a function declaration is a
function_declaration in tree-sitter-javascript but a function_item in tree-sitter-rust).30
This presents a fundamental challenge: to compare code for similarity, especially across languages, the Uveddi detector cannot work directly with the raw CSTs. It must first process them into an internal, normalized AST representation. This transformation involves two key steps:
Pruning: Ignoring "trivial" or "unnamed" nodes (like punctuation) that do not contribute to the code's logic.
Abstraction: Focusing on the "significant" or "named" nodes that represent the core structure of the program.29
This normalization process is not a one-time event but a spectrum of transformations. The level of abstraction applied determines the type of clones that can be detected. A shallow normalization might only abstract variable names to find Type-2 clones, while a deep, cross-language normalization might map different loop structures to a single canonical representation to find more abstract similarities. The architecture should therefore treat normalization as a configurable pipeline, allowing users to select an analysis profile that balances precision against the ability to find more abstract clones. For instance, a "Strict Refactoring" profile would use minimal normalization to find near-exact duplicates, whereas an "Architectural Pattern Discovery" profile could use aggressive normalization to find similar structures across different languages, accepting a higher rate of false positives.

2.2 Intra-Language Normalization for Type-2 and Type-3 Clones

To accurately detect clones within a single language, the AST must be normalized to remove superficial differences.

2.2.1 Identifier and Literal Abstraction

The defining characteristic of Type-2 clones is the renaming of identifiers and the changing of literal values. To detect them, the AST must be "anonymized."
Process: This involves a traversal of the AST. Whenever a node corresponding to an identifier (e.g., a variable name, function name) or a literal (e.g., a string, number, boolean) is encountered, its specific value is replaced with a generic placeholder. For example, all identifier nodes could be represented as _ID_ and all string_literal or number_literal nodes as _LIT_.
Implementation: This can be implemented as a recursive function that traverses the AST and builds a new, normalized tree structure. The logic of tools like PMD's CPD, which offers --ignore-identifiers and --ignore-literals flags, serves as a direct model for this functionality.11

2.2.2 Structural Hashing

Once an AST is normalized, a hash can be computed for each subtree to serve as a unique fingerprint of its structure. This allows for extremely fast, O(1), comparison of two subtrees.
Algorithm: A common approach is a recursive, bottom-up hash calculation. The hash of a parent node is a function of its own type and the hashes of its children.
hash(node) = H(node.type |



| hash(node.child) |
| hash(node.child) ||...)
```
where H is a standard hashing algorithm (like SHA-256 or a faster non-cryptographic hash like xxHash) and || denotes concatenation.
Application: This technique is ideal for the "Verification" stage of the proposed architecture. After the fast filter identifies a candidate pair, their ASTs can be generated, normalized, and their structural hashes compared. A hash match indicates a very high probability of being a Type-1 or Type-2 clone.31

2.3 Cross-Language AST Mapping Strategies

Detecting clones between Rust, Python, and JavaScript requires a deeper level of normalization that bridges their syntactic differences. The goal is to create a Canonical Intermediate Representation (CIR) for common programming constructs. This involves mapping language-specific AST nodes to a universal, language-agnostic set of node types.

2.3.1 Universal Node Mapping

A mapping layer must be defined that translates nodes from different Tree-sitter grammars into a shared vocabulary. This is a non-trivial task that requires domain knowledge of each language's syntax.33
Example Mapping Table:
Canonical Node Type
Rust (tree-sitter-rust)
Python (tree-sitter-python)
JavaScript (tree-sitter-javascript)
CanonicalFunction
function_item
function_definition
function_declaration, arrow_function
CanonicalConditional
if_expression
if_statement
if_statement
CanonicalLoop
for_expression, while_expression
for_statement, while_statement
for_statement, while_statement, do_statement
CanonicalAssignment
let_declaration
assignment, augmented_assignment
variable_declaration, assignment_expression
CanonicalCall
call_expression
call
call_expression

This mapping creates a "unified" AST that represents the abstract structure of the code, independent of the source language's specific syntax.34

2.3.2 Handling Language-Specific Constructs

Not all language features have direct one-to-one equivalents. The strategy for handling these unique constructs is crucial for maintaining accuracy.
Rust: Features like lifetime annotations ('a), the borrow checker's syntax (&, &mut), and macro invocations (println!) are unique. For the purpose of detecting structural clones, lifetime annotations can often be pruned from the canonical AST, as they describe memory management rather than algorithmic logic. Macros, however, are more complex as they expand into arbitrary code; a robust solution would require analyzing the expanded code, which adds significant complexity. Initially, it is recommended to treat macro invocations as opaque LanguageSpecificConstruct nodes.
Python: Decorators (@my_decorator), with statements, and list comprehensions are idiomatic. Decorators can be represented as a wrapper node (CanonicalDecorator) around a CanonicalFunction. with statements can be mapped to a CanonicalResourceScope node. List comprehensions are a form of loop and can be mapped to CanonicalLoop.
JavaScript: Prototypal inheritance, async/await patterns, and closures present challenges. async/await maps well to Rust's async/await, but Python's is slightly different. These can all be mapped to a CanonicalAsyncFunction or similar construct. Prototypal patterns are more semantic and harder to capture syntactically.
The general principle is to map what can be mapped, prune what is non-essential noise for structural comparison, and represent truly unique, meaningful constructs as special nodes in the canonical tree.

2.4 Measuring Structural Similarity of ASTs

With normalized, canonical ASTs, the final step is to measure their similarity.
Tree Edit Distance: This metric calculates the minimum number of "edits" (insert, delete, relabel a node) required to transform one tree into another.21 Algorithms like APTED can compute this score, providing a robust measure for Type-3 clone similarity.36 However, their high computational complexity (e.g.,
O(n3)) makes them unsuitable for direct, large-scale comparison.33 Their use should be limited to secondary analysis, such as providing a more precise similarity score for a small number of already-confirmed clones for reporting purposes.
Vector Similarity: As recommended in Part 1, the most scalable approach is to convert the normalized canonical ASTs into characteristic vectors and compare them using a distance metric like cosine similarity or Euclidean distance.21 This transforms the complex tree comparison problem into a much more tractable geometric problem of finding close points in a vector space.

2.5 Implementation Guide: Tree-sitter Queries and Rust Patterns

This section provides actionable code patterns for the Uveddi implementation team.

2.5.1 Tree-sitter Queries for Extracting Comparison Units

Tree-sitter's query engine is a powerful, declarative way to find specific patterns in the AST. It will be used to identify the boundaries of the code blocks (functions, methods) to be compared.
Rust Query (queries/rust/blocks.scm):
Scheme
(function_item) @block
(impl_item (function_item) @block)


Python Query (queries/python/blocks.scm):
Scheme
(function_definition) @block
(class_definition (function_definition) @block)


JavaScript Query (queries/javascript/blocks.scm):
Scheme
(function_declaration) @block
(function_expression) @block
(arrow_function) @block
(method_definition) @block


These queries capture the nodes that will serve as the root for each code block to be fingerprinted and compared. The @block capture name allows the Rust code to easily retrieve the matched nodes.37

2.5.2 Rust Code Examples for AST Processing

The following Rust code snippets illustrate key implementation patterns using the tree-sitter and rayon crates.
Parallel Parsing of Files:
Rust
use rayon::prelude::*;
use std::path::Path;

fn parse_files_in_parallel(paths: &[&Path]) -> Vec<tree_sitter::Tree> {
    paths.par_iter()
       .filter_map(|path| {
            let code = std::fs::read_to_string(path).ok()?;
            let mut parser = tree_sitter::Parser::new();
            // Assume language is set correctly based on file extension
            let language = tree_sitter_python::language(); 
            parser.set_language(language).ok()?;
            parser.parse(code, None)
        })
       .collect()
}


AST Normalization and Structural Hashing:
Rust
use std::collections::HashMap;
use sha2::{Sha256, Digest};

// Represents our internal, normalized AST node
struct NormalizedNode {
    kind: String, // Could be a canonical kind
    children: Vec<NormalizedNode>,
    hash: [u8; 32],
}

fn normalize_and_hash(node: tree_sitter::Node, source_code: &str) -> NormalizedNode {
    let mut hasher = Sha256::new();
    let kind_str = match node.kind() {
        "identifier" => "_ID_",
        "string_literal" => "_LIT_",
        //... other normalizations and canonical mappings
        _ => node.kind(),
    };
    hasher.update(kind_str.as_bytes());

    let mut children = Vec::new();
    let mut cursor = node.walk();
    for child_node in node.children(&mut cursor) {
        // Filter out trivial nodes like punctuation
        if child_node.is_named() {
            let normalized_child = normalize_and_hash(child_node, source_code);
            hasher.update(&normalized_child.hash);
            children.push(normalized_child);
        }
    }

    NormalizedNode {
        kind: kind_str.to_string(),
        children,
        hash: hasher.finalize().into(),
    }
}


This combination of Tree-sitter queries and Rust processing patterns provides a concrete starting point for building the normalization and comparison engine, turning the theoretical strategies into practical, performant code.

Part 3: Architecting for Performance and Scalability

Building a clone detector that can analyze over 100,000 files in minutes, not hours, requires an architecture meticulously designed for performance and scalability. This section translates the algorithms from Part 1 into a concrete system design, addressing the challenges of large-scale data processing, memory management, and incremental analysis within the Uveddi ecosystem. The central theme is that for a tool of this magnitude, performance is not an afterthought but the primary driver of the architecture itself.

3.1 The Scalability Imperative: Learning from the Field

The performance of existing large-scale clone detectors sets a benchmark for what is achievable. Tools like CCFinderX and SourcererCC have demonstrated the ability to process hundreds of millions of lines of code (MLOC).28 More recent hybrid tools like TACC can analyze 100 MLOC in under four hours.7 These examples prove that large-scale analysis is feasible, but only with architectures that explicitly avoid naive, brute-force comparisons. A pairwise comparison of all functions in a 100k file repository could involve billions of comparisons, which is computationally prohibitive. Therefore, the Uveddi detector's architecture must be founded on sub-quadratic principles, primarily through efficient indexing and candidate reduction.

3.2 Indexing and Candidate Reduction Strategies

The cornerstone of a scalable architecture is a filtering mechanism that drastically reduces the number of potential clone pairs that need detailed comparison. This implements the "fast filtering" stage of the recommended hybrid model.

3.2.1 Inverted Index on Token Hashes

Inspired by the success of information retrieval systems and tools like SourcererCC, an inverted index is the recommended primary strategy for candidate generation.28
Architecture:
Unit of Analysis (Chunking): The codebase is first divided into logical "documents" or "blocks." The most effective granularity is the function or method level, as this aligns with logical units of code that are typically copied.28 Tree-sitter queries (as defined in Part 2) are used to extract these blocks.
Fingerprinting: Each block is tokenized and normalized. A rolling hash (like a polynomial rolling hash used in Karp-Rabin) is applied to the normalized token stream to generate a set of hash values (fingerprints) that characterize the block.
Indexing: An inverted index is constructed. This is a data structure that maps each unique fingerprint (hash value) to a list of all code blocks that contain it.
Querying: To find clone candidates for a given block B, the system retrieves the lists of blocks associated with each of B's fingerprints. Blocks that appear frequently across these lists (i.e., share many fingerprints with B) are considered strong candidates. A simple count-based threshold can be used to select these candidates for the next stage.
Scalability: This approach scales exceptionally well because it transforms the problem from code comparison to efficient set intersections on pre-computed lists, a task at which modern databases and search indices excel.

3.2.2 Locality-Sensitive Hashing (LSH) for Vector-based Analysis

If the primary representation chosen is AST-based characteristic vectors (as in the DECKARD model), then LSH is the appropriate indexing technique.21
Architecture:
Vectorization: Every relevant AST subtree is converted into a high-dimensional numerical vector.
LSH Hashing: A family of LSH functions is used to hash these vectors. The key property of LSH is that similar vectors are likely to produce the same hash value ("collide"), while dissimilar vectors are not.
Indexing: Vectors are placed into multiple hash tables, each using a different LSH function.
Querying: To find candidates for a vector v, the system looks up v's hash in all tables and collects all vectors found in the same buckets. This set of vectors constitutes the approximate nearest neighbors and thus the clone candidates.
Trade-offs: LSH is probabilistic and more complex to implement and tune than an inverted index. However, it is specifically designed for finding "near misses" in high-dimensional spaces, making it powerful for detecting Type-3 clones if a vector-based approach is pursued. For Uveddi's initial implementation, the inverted index on token hashes is recommended for its simplicity and proven scalability, with LSH being a potential future enhancement.

3.3 Memory and Concurrency Optimization in Rust

The Uveddi detector must be implemented with careful attention to resource utilization. Rust's features, combined with the existing parallel processing infrastructure, provide a strong foundation for an efficient implementation.
Parallel Processing with Rayon: The initial stages of the detection pipeline—reading files, parsing, and fingerprinting—are "embarrassingly parallel," as each file can be processed independently. The rayon crate, already in use by Uveddi, is perfectly suited for this. A par_iter() over the list of files to be analyzed will distribute the workload across all available CPU cores, dramatically reducing the wall-clock time for the initial data processing phase.39
Memory-Efficient Data Structures:
AST Representation: The tree-sitter crate is already efficient, but care must be taken when building the internal normalized AST. Instead of copying source text into the AST nodes, the nodes should store byte-range indices (Range<usize>) that refer back to the original source file content (&[u8]). This avoids duplicating string data and significantly reduces the memory footprint of each parsed file.
Index Implementation: A naive in-memory HashMap for the fingerprint index can consume vast amounts of memory. The most robust solution is to offload the index to a persistent store, as detailed below. This moves the memory bottleneck from the application's heap to the database, which is designed to manage large datasets efficiently.
Caching Mechanisms: Leveraging and extending Uveddi's existing caching infrastructure is critical for performance, especially for incremental analysis.
AST Cache: The existing cache will be used to avoid re-parsing unchanged files.
Fingerprint Cache: A new, persistent cache is required to store the fingerprints (and/or vectors) for each code block, keyed by a unique identifier for that block (e.g., file path + function name). This cache is the cornerstone of incremental analysis.
Result Cache: The final computed clone pairs, along with their similarity scores and metadata, will be cached to avoid re-running the verification stage and to speed up reporting.

3.4 Designing for Incremental Analysis in CI/CD

For integration into a CI/CD pipeline, the detector must provide feedback in seconds or minutes, not hours. This mandates an incremental, or "delta," analysis approach. A full re-analysis of the entire codebase on every commit is unacceptable.40
Change-Based Workflow: The analysis should be triggered by a set of changed files from the version control system.
Cache-Driven Incremental Logic:
Identify Changes: Given a set of added, modified, or deleted files.
Update Fingerprints:
For deleted files, remove all associated code blocks and their fingerprints from the persistent index.
For added or modified files, parse them and generate new fingerprints for all their code blocks.
Update Index: Atomically update the fingerprint index in the database: remove the old fingerprints for the modified blocks and insert the new ones.
Query for New Clones: The crucial optimization is to only query for new clones involving the changed blocks. The system does not need to re-check for clones between two unchanged files, as that result is already known (or can be assumed to be cached).
Verification: Run the AST-based verification stage only on the new candidate pairs generated in the previous step.
Benefits: This design ensures that the analysis time is proportional to the size of the code change, not the size of the entire repository. This is a far more practical and scalable approach than attempting to use complex updatable data structures like generalized suffix trees, which have significant memory overhead and are difficult to distribute.40

3.5 Proposed Scalable Architecture for Uveddi

The architecture must not be viewed as a single algorithm, but as a multi-stage data processing pipeline where the database is a core component of the algorithmic strategy. For a system of this scale, the fingerprint index cannot be reliably held in memory; it must reside in the database.
The Database-as-Index Architecture:
This design treats the PostgreSQL database not just as a place to store final results, but as the engine for the scalable candidate search itself.
Schema Design:
code_blocks table: Stores metadata for each analysis unit (e.g., id, file_path, function_name, start_line, end_line, content_hash).
block_fingerprints table: Stores the many-to-many relationship between blocks and their fingerprints (block_id, fingerprint_hash). A database index must be created on the fingerprint_hash column to enable fast lookups.
Analysis Flow:
Initial Scan:
The Rust application, using rayon, processes all files in parallel.
For each function, it generates a set of fingerprints.
It populates the code_blocks and block_fingerprints tables in the database. This is a one-time, expensive operation.
Finding Candidates: The candidate generation step is now a SQL query. To find candidates for a block with ID B_ID, the query would be:
SQL
SELECT T2.block_id, COUNT(*) as shared_fingerprints
FROM block_fingerprints T1
JOIN block_fingerprints T2 ON T1.fingerprint_hash = T2.fingerprint_hash
WHERE T1.block_id = <B_ID> AND T1.block_id!= T2.block_id
GROUP BY T2.block_id
HAVING COUNT(*) > <MIN_SHARED_FINGERPRINTS_THRESHOLD>
ORDER BY shared_fingerprints DESC;

This query leverages the database's highly optimized join and indexing engine to perform the candidate search efficiently, moving the heavy lifting from the Rust application to the database server.
Verification: The Rust application receives the candidate IDs from the database, fetches their source code, and performs the AST-based verification as described before.
Incremental Update: When a file changes, the Rust application updates the relevant rows in the code_blocks and block_fingerprints tables within a database transaction and then runs the candidate-finding query only for the newly updated block IDs.
This architecture is robust, scalable, and persistent. It leverages the strengths of both Rust (for CPU-intensive parallel parsing) and PostgreSQL (for memory-efficient, indexed data management), providing a solid foundation for an enterprise-grade clone detector.
Architectural Diagram:



+----------------+   (Rayon)   +-------------------+   (Parallel)   +----------------------+

| Source Files | --------> | File Parser | -------------> | Fingerprinter / |
| (100k+ files) | | (Tree-sitter) | | Vectorizer |
+----------------+           +-------------------+                +----------------------+
|
| (Batch Insert/Update)
                                                                           v
+------------------------------------------------------------------------------------------+

| PostgreSQL Database |
| |
| +-----------------------+      +--------------------------+      +-------------------+ |
| | code_blocks Table | | block_fingerprints Table | | results_cache | |
| | (id, path, name,...) |<---->| (block_id, fingerprint) | | (clone_pair_hash) | |
| +-----------------------+      +--------------------------+      +-------------------+ |
| (Indexed on fingerprint) |
+------------------------------------------------------------------------------------------+
      ^ | (SQL Query for Candidates)

| (Fetch block source)                 v
+----------------------+           +--------------------------+

| AST-based Verifier | <--------- | Candidate Pair Generator |
| (Rust Process) | | (Rust Process) |
+----------------------+           +--------------------------+
|
| (Store confirmed clones)
      v
+----------------------------------+

| Result Processing & Reporting |
| (False Positive Filter, AI Explainer, UI) |
+----------------------------------+



Part 4: A Multi-Layered Approach to False Positive Mitigation

A code duplication detector is only as valuable as its results are actionable. A high volume of false positives—findings that are technically duplicates but are uninteresting or benign—leads to alert fatigue and causes developers to lose trust in the tool.42 Therefore, a sophisticated, multi-layered filtering strategy is not an optional feature but a core requirement for success. The goal is not to achieve zero false positives, which is often impossible without also creating false negatives, but to reduce the noise to a manageable level where the reported issues are overwhelmingly relevant and actionable.43

4.1 A Taxonomy of False Positives

To effectively filter noise, one must first understand its sources. The following taxonomy categorizes common types of uninteresting duplication that should be filtered.
Boilerplate Code: This includes code that is required by the language or framework but carries little semantic weight. Prime examples are blocks of import or use statements, package declarations, and empty constructors or destructors.23 Tools like SonarQube specifically target these for exclusion.23
Generated Code: Many projects contain code that is not written by developers but is generated by tools. This can include UI designer files (e.g., in.NET or Qt), code generated from protobuf or OpenAPI specifications, or ORM entity classes. These files should almost always be excluded from analysis.
Test Code: Test suites often contain significant amounts of duplication by design. Test setup methods (beforeEach), mock object configurations, and assertion blocks frequently have repetitive structures. While this duplication can sometimes indicate a need for test utility functions, it is generally considered a different class of problem from duplication in production code and should be filtered separately.
Benign Idioms and Patterns: Short, common programming patterns are often flagged as clones but are not worth refactoring. This includes getter/setter pairs, simple null checks, or the implementation of standard interface methods like equals() and hashCode() in Java.
Framework-Mandated Repetition: Some frameworks require developers to implement specific patterns repeatedly. Examples include overriding lifecycle methods in Android/iOS development or implementing controller actions in web frameworks. These are often structurally identical but functionally distinct and represent necessary repetition rather than accidental duplication.
Configuration and Data Literals: Large data structures, such as arrays or dictionaries used for configuration, mappings, or test data, can be flagged as clones if they are structurally similar. This is a key area where ignoring literals is insufficient; the entire data structure should often be ignored. The issues reported by SonarQube users regarding duplicated JavaScript code differing only by string literals highlight this problem vividly.25

4.2 Heuristic and Contextual Filtering Strategies

A robust filtering system combines several techniques, from simple thresholds to contextual rules.
Size Thresholds: The most fundamental filter is to ignore clones below a certain size. This is typically configured by a minimum number of lines or tokens. Nearly all clone detection tools, including PMD and Simian, support this.26 This single setting is highly effective at eliminating a large volume of trivial, uninteresting micro-clones.
Syntactic Filtering: By leveraging Tree-sitter, the Uveddi detector can perform powerful syntactic filtering. This involves writing queries to identify and exclude specific AST node types from the analysis entirely. For example, a query can be written to ignore all import_declaration nodes or any function annotated with #[test] in Rust.
File and Path-Based Filtering: A standard and essential feature is the ability to exclude files and directories based on glob patterns.26 The configuration should allow users to specify a list of ignore patterns, such as
["**/test/**", "**/generated/**", "*.spec.js"], to completely remove test, generated, and other non-essential code from the analysis scope.
Comment-Based Suppression: To handle specific one-off cases where a developer intentionally wants to keep duplicated code, the tool should support in-line suppression comments. A developer could add a comment like // uveddi:ignore-duplication or // noclone directly above a code block to prevent it from being reported.

4.3 Statistical and Machine Learning-Based Filtering

For more advanced and adaptive filtering, statistical and machine learning techniques can be employed as a post-processing step on the initial set of detected clones.
Frequency and Distribution Analysis: A statistical heuristic can be applied to down-rank or filter clones based on their prevalence. If a particular clone class (a set of fragments all similar to each other) has a very high number of instances spread widely across the codebase, it is more likely to be a common, benign programming idiom than a harmful copy-paste error. Such highly frequent clones can be assigned a lower severity or hidden by default.
Machine Learning-Based Classification: This represents the state-of-the-art in false positive reduction.44 The process is as follows:
Feature Extraction: For each detected clone pair, extract a set of features. These can include:
Size Metrics: Lines of code, number of tokens, number of AST nodes.
Complexity Metrics: Cyclomatic complexity (can be estimated from the AST).
Structural Metrics: Number of loops, conditionals, function calls within the clone.
Location Metrics: Are the clones in the same file? Same module? Different modules?
Syntactic Metrics: Percentage of identifier/literal differences.
Training: A human expert manually labels a dataset of detected clones as either "Actionable" (true positive) or "Benign" (false positive).
Model Training: A classification model (e.g., a Decision Tree, Random Forest, or Gradient Boosted Trees) is trained on this labeled feature set.
Filtering: The trained model is then used as a filter. For each new clone pair detected, its features are extracted and fed to the model, which predicts whether it is actionable or benign. This allows the system to learn the specific patterns of what constitutes "noise" within a particular organization or codebase.44

4.4 User-Centric Configuration System Design

No single set of filtering rules will work for all users or all projects. Therefore, providing a powerful and intuitive configuration system is essential for user adoption and satisfaction. The detector's configuration, likely managed in a uveddi.toml file, should be comprehensive.

Ini, TOML


[analysis.detectors.duplication]
# Enable or disable the detector entirely
enabled = true

# The minimum number of tokens a block must have to be considered for duplication
minimum_token_count = 70

# The similarity threshold (e.g., 0.0 to 1.0) for reporting a clone
similarity_threshold = 0.85

# File and directory patterns to exclude from analysis
ignore_files = [
    "**/*.spec.js",
    "**/*_test.py",
    "**/tests/**",
    "**/generated/**",
    "**/vendor/**",
]

# Language-specific overrides
[analysis.detectors.duplication.javascript]
minimum_token_count = 100
# Acknowledge the issues seen with SonarQube; make this configurable
ignore_string_literals = false

# Heuristic filters to suppress common boilerplate patterns
[analysis.detectors.duplication.filters]
ignore_imports = true
ignore_getters_setters = true
# Filter clones where the only difference is the type annotation
ignore_type_only_diffs = true


A critical insight is that the concept of a "false positive" is highly context-dependent and subjective. A clone that is noise for a developer looking to refactor might be a critical finding for an architect auditing API consistency.44 A rigid, one-size-fits-all filter is therefore suboptimal.
A more advanced architecture should support different analysis profiles. Instead of a single set of global configurations, the user could select a profile tailored to their current task:
profile = "refactoring": This profile would use high similarity thresholds and aggressive filtering of tests, boilerplate, and common idioms to present a very clean list of high-impact refactoring opportunities.
profile = "convention_audit": This profile might use lower thresholds and focus specifically on clones that cross module or component boundaries, helping architects identify where similar logic has been implemented inconsistently.
profile = "plagiarism_check": This profile would use very low thresholds and minimal filtering to achieve the highest possible recall, suitable for academic or intellectual property investigations.
This profile-based approach transforms the tool from a simple detector into a versatile analysis assistant that adapts to the user's intent, making its output dramatically more valuable.

Part 5: Multi-Language Considerations

A key requirement for the Uveddi detector is its ability to operate effectively across a polyglot codebase of Rust, Python, and JavaScript, with an architecture that can be extended to other languages. This necessitates a design that can abstract away language-specific details while respecting the unique idioms and paradigms of each.

5.1 Language-Specific Normalization and Challenges

As discussed in Part 2, the core strategy for cross-language analysis is the creation of a Canonical Intermediate Representation (CIR) from the language-specific Tree-sitter ASTs. However, this process must carefully handle the unique features of each target language.
Rust:
Ownership and Lifetimes: Lifetime annotations ('a) and borrow syntax (&, &mut) are central to Rust's safety model but are typically noise for structural clone detection. The normalization process should prune these annotations from the canonical AST.
Macros: Rust's procedural and declarative macros are a significant challenge because they expand into arbitrary code before compilation. A simple syntactic analysis will only see the macro invocation (e.g., println!("hello")). A truly robust analysis would require expanding the macros, which is a complex process tightly coupled to the compiler. For an initial implementation, it is recommended to treat macro invocations as opaque nodes in the AST or to use heuristics to identify common ones.
Traits and Generics: These are Rust's primary abstraction mechanisms. Clones involving generic functions can be handled by treating generic type parameters as normalizable identifiers.
Python:
Dynamic Typing: The absence of static types simplifies the AST but means that semantic similarity is harder to infer. The analysis will be purely structural.
Decorators: Decorators (@decorator) are syntactic sugar for function wrapping. They should be represented in the CIR as a CanonicalDecorator node that wraps a CanonicalFunction node, preserving the structural relationship.
with Statements: The with statement for resource management is a common idiom. It can be mapped to a CanonicalResourceScope node in the CIR.
Duck Typing: The concept of "if it walks like a duck and quacks like a duck, it's a duck" is semantic, not syntactic. A purely AST-based approach will not be able to identify clones between two classes that are "duck-typed" to be interchangeable but do not share a common inheritance structure.
JavaScript:
Prototypal Inheritance: Unlike classical inheritance in Java or the trait system in Rust, JavaScript's prototypal inheritance is a dynamic, object-based system. Capturing this structurally is difficult. The analysis should focus on class-based syntax (class MyClass extends...) which can be mapped more directly to canonical inheritance nodes.
Asynchronous Patterns: Callbacks, Promises (.then()), and async/await are different ways to express asynchronous logic. async/await syntax in JS and Python can be mapped to a common CanonicalAsyncFunction representation. Promise chains are structurally distinct and would be more challenging to map to async/await clones.
Frameworks: Modern JavaScript is dominated by frameworks like React, Vue, and Angular. These frameworks introduce their own syntax (e.g., JSX) and patterns (e.g., React Hooks). Handling these requires either dedicated parsers (like tree-sitter-javascript with JSX support) and specific normalization rules to ignore framework boilerplate.

5.2 Cross-Language Equivalence and Paradigm Mapping

The ultimate goal of a cross-language detector is to find functionally equivalent code, which often means bridging different programming paradigms.33
Object-Oriented vs. Functional: A common pattern might be implemented as a class with methods in Python or JavaScript, but as a module with free functions operating on a struct in Rust. A sophisticated CIR would need to be able to represent both. For example, a method_call on an object and a function_call with the object as the first parameter could be normalized to the same canonical representation: CanonicalCall(target=object, function=name, args=...).
Data Structures: Mapping built-in data structures is essential. A Python dict, a JavaScript Object (used as a map), and a Rust HashMap all serve a similar purpose. The CIR should have a CanonicalMap type to represent these abstractly.
API Call Mapping: A more advanced technique involves mapping equivalent library or API calls across languages.47 For example, a file-reading operation might be
fs.readFileSync in Node.js and std::fs::read_to_string in Rust. This can be achieved by building a manual or semi-automated mapping dictionary of common APIs. This moves beyond purely structural similarity towards semantic similarity. Modern approaches leverage large language models (LLMs) trained on multilingual code to learn these mappings automatically by embedding code from different languages into a shared vector space.48

5.3 Polyglot Analysis Architecture

The Uveddi detector's architecture must be designed from the ground up to support polyglot analysis.
Language Dispatcher: The entry point of the pipeline receives a file and determines its language (e.g., from its extension). It then dispatches the file to the correct Tree-sitter parser.
Language-Specific Normalizer: After parsing, the CST is passed to a language-specific normalization module. This module is responsible for traversing the CST and converting it into the language-agnostic Canonical Intermediate Representation (CIR). This is where the mapping rules from section 5.2 are implemented.
Language-Agnostic Core: Once the code is represented as a CIR, the core clone detection logic—fingerprinting, indexing, and verification—operates on this universal representation. This core logic does not need to know whether the original code was Rust, Python, or JavaScript.
Extensibility: To add a new language (e.g., Go), the only components that need to be created are:
The Tree-sitter grammar for Go.
A new language-specific normalizer module that maps the Go CST to the CIR.
Language-specific Tree-sitter queries for block extraction.
The core detection engine remains unchanged, making the architecture highly extensible.
This architecture effectively decouples the language-specific parsing and normalization from the language-agnostic detection logic, which is the key to building a scalable and maintainable multi-language analysis tool. While creating a truly universal AST that captures all semantics is an open research problem 49, a pragmatic CIR focused on the most common structural constructs provides a powerful and achievable foundation for cross-language clone detection.

Part 6: Integration and Reporting Strategies

A clone detector's findings are only useful if they are communicated effectively to developers and architects. The integration of results into existing workflows, the design of clear reports and visualizations, and the ability to provide context for AI-powered explanations are as important as the detection algorithm itself. This section outlines strategies for making the Uveddi clone detection results actionable, insightful, and seamlessly integrated.

6.1 Integration with AI Explanation Systems

The Uveddi platform's existing AI explanation system presents a unique opportunity to go beyond simple clone reporting. To enable high-quality, automated explanations, the detector must provide the AI model (likely an LLM) with rich, structured context. Simply feeding the two code snippets to the model is insufficient.
Structured Data for LLM Prompting: The data passed from the clone detector to the AI system for a given clone pair should be a structured object (e.g., JSON) containing:
Code Fragments: The full source code of both cloned blocks.
Location Metadata: File paths, start/end line numbers, and function/class names for both fragments.
Clone Classification: The detected clone type (Type-1, Type-2, Type-3) and similarity score (e.g., 0.87).
Syntactic Differences: A structured diff of the normalized ASTs. This is crucial as it highlights what changed (e.g., "Identifier x renamed to y", "Statement z = 1 added").
Contextual Information: Information about the modules or components the clones reside in. A clone between a backend-api module and a data-processing module has different implications than a clone within a single file. This data can be sourced from Uveddi's existing dependency graph.
Example Prompt for AI Explanation:
Analyze the following code duplication finding.

**Clone Pair Details:**
- Clone Type: Type-3 (Near-Miss)
- Similarity Score: 88%
- Location A: file='src/user_service.rs', function='get_user_profile', lines=50-75
- Location B: file='src/admin_service.rs', function='get_admin_profile', lines=40-68

**Structural Differences:**
- In Location B, an additional 'if' statement checking for 'is_superuser' is present.
- In Location B, the function call 'fetch_user_data' is replaced with 'fetch_admin_data'.

**Architectural Context:**
- Location A is in the 'UserService' component.
- Location B is in the 'AdminService' component.

**Task:**
1. Provide a concise summary of the duplication.
2. Explain the potential risks (e.g., bug propagation, inconsistent logic).
3. Suggest a specific refactoring strategy (e.g., "Extract a generic 'get_profile' function into a shared 'profile_utils' module, parameterizing the data fetching call and the permission check.").


This structured approach enables the LLM to move beyond generic advice and provide specific, context-aware, and actionable recommendations, significantly enhancing the value of the Uveddi platform.51

6.2 Visualization Techniques for Communicating Findings

Effective visualization can transform raw data into immediate insight. The Uveddi UI should offer multiple views for exploring duplication.
High-Level Dashboard: A project-level dashboard should display key metrics at a glance:
Overall Duplication Percentage: The percentage of lines of code that are part of a clone.
Duplication by Language: A pie or bar chart showing the distribution of duplicated code across Rust, Python, and JavaScript.
Trend Graph: A line chart showing how the duplication percentage has changed over time, which is crucial for tracking technical debt.54
Interactive Scatterplot (Dot Plot): For a whole-system overview, a scatterplot is a powerful visualization.55
Axes: The X and Y axes represent the files in the codebase, concatenated together.
Dots: A dot at position (x, y) indicates that the code at line x is a clone of the code at line y.
Patterns: This visualization makes high-level patterns immediately obvious:
Diagonals: Indicate long, contiguous copy-pasted blocks.
Grids/Rectangles: Indicate repetitive code, such as boilerplate or switch statements.
Clusters: Show "hotspots" of duplication within or between modules.
The plot should be interactive, allowing users to zoom in and click on a dot to see the corresponding code.
Clone Cluster Graph: To visualize relationships between clones, a force-directed graph can be used.
Nodes: Each node represents a file or a function.
Edges: An edge connects two nodes if they contain cloned code. The thickness or color of the edge can represent the amount of duplication between them.
Insight: This view is excellent for identifying which modules are tightly coupled by duplication and are prime candidates for shared library extraction.57
Side-by-Side Diff Viewer: For examining a specific clone pair, a classic side-by-side diff view is essential. It should highlight the identical parts and the differences (added, removed, or changed lines/tokens) to make the similarity clear.

6.3 Prioritization and Ranking of Clones

Not all clones are created equal. To prevent overwhelming users, the detected clones must be prioritized by their potential impact and severity.
Severity Score Calculation: A severity score can be computed for each clone class based on a weighted combination of factors:
Size: Larger clones are generally more problematic.
Number of Instances: A clone class with many instances may represent a more widespread problem (or a benign idiom, requiring filtering).
Location (Coupling): Clones that span across major architectural boundaries (e.g., different microservices or high-level modules in the dependency graph) are more severe than clones within the same file or module.
Similarity (for Type-3): Clones with higher similarity are easier to refactor and may be prioritized.
Complexity: The cyclomatic complexity of the cloned code. Duplicating complex logic is riskier than duplicating simple logic.
Default Ranking: The results list should be sorted by this severity score by default, presenting the most critical issues to the user first.

6.4 Key Metrics for Developers and Architects

The detector should produce a set of clear, valuable metrics that can be tracked over time and used in quality gates.
Duplicated Lines: The absolute number of physical lines of code involved in duplication.
Duplication Density / Coverage: The percentage of duplicated lines relative to the total lines of code in the project (Duplicated Lines / Total Lines). This is the primary top-level health metric.
Number of Clone Classes: The total number of distinct duplication groups.
Refactoring Opportunities: An estimated number of high-severity clone classes that are strong candidates for refactoring. This provides a more actionable metric than just raw duplication numbers.

6.5 Integration with Development Workflows

To be effective, the detector must integrate into the daily workflows of developers.
CI/CD Integration: The detector should run as part of the continuous integration pipeline. A quality gate can be configured to fail the build if a pull request introduces new, high-severity duplication above a certain threshold. This prevents new duplication from entering the codebase.58
IDE Plugins: While a full IDE plugin is a larger effort, the results from the Uveddi platform should be easily accessible. A link from the CI/CD report should take the developer directly to the interactive visualization of the new clones in their branch.
Code Review Automation: The results can be posted automatically as comments on pull requests or merge requests, showing the developer exactly where the new duplication is and providing a link to the full Uveddi report.58
By focusing on rich context for AI, effective visualization, intelligent prioritization, and seamless workflow integration, the Uveddi clone detector can evolve from a simple analysis tool into an indispensable guide for improving code quality and architectural health.

Part 7: Implementation Roadmap

This section outlines a strategic, phased implementation plan for the Uveddi Code Duplication Detector. The roadmap is designed to deliver value incrementally, starting with a Minimum Viable Product (MVP) and progressively adding more sophisticated features. This approach allows for continuous feedback, risk mitigation, and alignment with the overall Uveddi development schedule. Each phase includes specific goals, testing strategies, and success criteria.

Phase 1: Foundational Architecture and Type-1/2 MVP

The primary goal of this phase is to build the scalable backbone of the detector and deliver an MVP capable of finding basic clones in a single language. This validates the core architecture before adding complexity.
Features:
Implement the core data processing pipeline: parallel file reading, Tree-sitter parsing, and function/block extraction.
Target a single language first, preferably Python due to its simpler syntax and lack of compilation requirements.
Implement the token-based hashing and indexing strategy (Stage 1). This includes tokenization, basic normalization (whitespace, comments), and fingerprinting using a rolling hash.
Implement the "database-as-index" architecture. Set up the PostgreSQL schema for code_blocks and block_fingerprints.
Implement the candidate generation logic via SQL queries against the fingerprint index.
For verification (Stage 2), implement a simple byte-for-byte comparison of the source text of candidate blocks to detect Type-1 clones.
Build a basic CLI to run the analysis and output clone pairs to the console or a simple JSON file.
Testing Strategy:
Unit tests for the tokenizer, hasher, and database interactions.
Integration tests that run the full pipeline on a small, controlled set of Python files with known Type-1 clones.
Initial performance testing on a medium-sized open-source project (e.g., 1k-5k files) to validate the scalability of the indexing approach.
Success Criteria:
The system can successfully analyze a codebase of at least 10k Python files and correctly identify all injected Type-1 clones.
The core database indexing schema is finalized and proven to be performant for candidate queries.
The end-to-end pipeline is functional and ready for further enhancement.

Phase 2: Advanced Syntactic and Near-Miss Clone Detection (Type-3)

This phase builds upon the MVP to add detection of more complex and common clone types (Type-2 and Type-3), significantly increasing the detector's practical value.
Features:
Enhance the normalization pipeline to include identifier and literal abstraction, enabling Type-2 detection.
Implement the AST-based verification stage (Stage 2). This involves:
Parsing candidate blocks to full ASTs.
Implementing the AST normalization logic (pruning trivial nodes, abstracting identifiers/literals).
Implementing a structural similarity algorithm. The recommended approach is to generate characteristic vectors for the normalized ASTs and compare them using cosine similarity. This provides a robust similarity score for Type-3 clones.
Introduce the user configuration system (uveddi.toml) for setting thresholds (e.g., minimum_token_count, similarity_threshold).
Develop the initial false positive filtering layer (size thresholds, file path exclusion).
Testing Strategy:
Unit tests for the AST normalization and vectorization logic.
Validate against a known benchmark. The BigCloneBench dataset is the industry standard for evaluating clone detection tools, especially for Type-3 and Type-4 clones.7 Although it is Java-based, the clone pairs can be used as a reference to create an equivalent test set in Python or to validate the logic conceptually. The
Mutation/Injection Framework approach can also be used to create a large set of artificial Type-2 and Type-3 clones for automated recall testing.61
Measure precision and recall against the chosen benchmark to tune the similarity threshold.
Success Criteria:
The detector achieves high precision (>95%) and recall (>80%) for Type-1, Type-2, and Strongly Type-3 clones on the validation dataset.
The system can successfully analyze a large Python codebase (e.g., 50k files) and produce meaningful, actionable results.
The configuration system is functional and allows users to tune the detector's sensitivity.

Phase 3: Multi-Language and Cross-Language Support

This phase expands the detector's reach to all target languages and introduces the capability for cross-language detection.
Features:
Add support for Rust and JavaScript. This involves implementing the language-specific Tree-sitter queries for block extraction and extending the dispatcher to use the correct parser.
Implement the Canonical Intermediate Representation (CIR) for ASTs.
Create the language-specific normalizer modules that transform Rust, Python, and JavaScript CSTs into the CIR. This includes handling language-specific idioms as discussed in Part 5.
The core detection logic (fingerprinting and verification) will now operate on the CIR, making it language-agnostic.
Enable a "cross-language" analysis mode that finds clones between files of different languages.
Testing Strategy:
Repeat the testing strategy from Phase 2 for Rust and JavaScript codebases individually.
For cross-language validation, use datasets like XLCoST or CodeNet, which contain functionally equivalent problems solved in multiple languages.63 Alternatively, create a custom benchmark by finding open-source projects that have been ported between the target languages (e.g., a tool rewritten from Python to Rust).65
Manually validate a sample of cross-language findings to assess precision.
Success Criteria:
The detector can analyze a mixed-language repository containing Rust, Python, and JavaScript.
The system successfully identifies known functional equivalents in the cross-language validation dataset.
The CIR and normalization pipeline are proven to be robust and extensible.

Phase 4: Advanced Filtering, Reporting, and Integration

This phase focuses on user experience, turning the powerful backend into a polished, integrated, and highly usable feature of the Uveddi platform.
Features:
Implement the advanced false positive filtering techniques: statistical frequency analysis and context-aware rules (e.g., down-ranking clones within test files).
Design and implement the user-facing visualizations: the high-level dashboard, the interactive scatterplot, and the clone cluster graph.
Integrate the detector with the Uveddi UI and reporting system.
Implement the full incremental analysis workflow for CI/CD integration, including the quality gate functionality.
Begin research and data collection for the ML-based false positive filter by allowing users to mark findings as "actionable" or "benign."
Testing Strategy:
Usability testing with internal developers to gather feedback on the UI and visualizations.
End-to-end testing of the CI/CD integration in a staging environment.
Performance testing of the incremental analysis on a large, actively developed repository to ensure fast feedback times.
Success Criteria:
The duplication analysis results are fully integrated into the Uveddi platform.
The CI/CD quality gate can successfully block pull requests that introduce severe duplication.
Developer feedback indicates that the results are actionable and the false positive rate is acceptably low.

Phase 5: Semantic Analysis and AI-Powered Explanations

This final phase pushes the detector to the cutting edge by exploring semantic clone detection and fully leveraging the AI integration.
Features:
Integrate the detector's structured output with the AI explanation system, as designed in Part 6.
Begin research and prototyping for Type-4 (semantic) clone detection. This should start with ML-based approaches that learn code embeddings. Pre-trained models like CodeBERT or UniXcoder can be used to generate vector representations of code blocks, which can then be compared using cosine similarity to find functionally similar code, even if the structure is different.48
Train and deploy the ML-based false positive classification model developed in Phase 4.
Testing Strategy:
Evaluate the quality of AI-generated explanations and refactoring suggestions through expert review.
Benchmark the semantic clone detection prototype against the Type-4 clones in BigCloneBench and other semantic similarity datasets.
Success Criteria:
The Uveddi platform can provide accurate, context-aware, AI-generated explanations and refactoring suggestions for detected clones.
The prototype for semantic clone detection demonstrates feasibility and shows promising results on standard benchmarks, paving the way for future productization.
This phased roadmap provides a clear and logical path to building a world-class code duplication detector. By prioritizing the scalable architecture first and then incrementally adding features, the Uveddi team can manage complexity, deliver value early, and build a robust, accurate, and highly performant tool that meets all of its strategic objectives.

References

Academic Papers & Theses: 1
Industry Implementations & Documentation: 11
Tooling & Library Documentation (Tree-sitter, etc.): 29
General Surveys & Articles: 2
Works cited
Review of Code Similarity and Plagiarism Detection Research Studies, accessed July 3, 2025, https://www.mdpi.com/2076-3417/13/20/11358
Analysis of Research Trends Towards Types of Code Clone Detection Techniques, accessed July 3, 2025, https://www.researchgate.net/publication/368801598_Analysis_of_Research_Trends_Towards_Types_of_Code_Clone_Detection_Techniques
(PDF) Various Code Clone Detection Techniques and Tools: A Comprehensive Survey, accessed July 3, 2025, https://www.researchgate.net/publication/312080887_Various_Code_Clone_Detection_Techniques_and_Tools_A_Comprehensive_Survey
Exploring the Boundaries Between LLM Code Clone Detection and Code Similarity Assessment on Human and AI-Generated Code - MDPI, accessed July 3, 2025, https://www.mdpi.com/2504-2289/9/2/41
A Systematic Review on Code Clone Detection - SciSpace, accessed July 3, 2025, https://scispace.com/pdf/a-systematic-review-on-code-clone-detection-51ueuzckrv.pdf
Survey of Research on Software Clones, accessed July 3, 2025, https://d-nb.info/991490185/34
Comparison and Evaluation of Clone Detection Techniques with Different Code Representations - Yueming Wu, accessed July 3, 2025, https://wu-yueming.github.io/Files/ICSE2023_TACC.pdf
A Survey of Software Clone Detection Techniques - International Journal of Computer Applications, accessed July 3, 2025, https://www.ijcaonline.org/research/volume137/number10/sheneamer-2016-ijca-908896.pdf
A Comprehensive Review of Code Clone Detection Techniques - ijltemas, accessed July 3, 2025, https://www.ijltemas.in/DigitalLibrary/Vol.4Issue12/43-47.pdf
(PDF) Survey on Software code clone detection - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/360015247_Survey_on_Software_code_clone_detection
Finding duplicated code with CPD | PMD Source Code Analyzer, accessed July 3, 2025, https://pmd.github.io/pmd/pmd_userdocs_cpd.html
Identifying Similar Code with Program Dependence Graphs | Request PDF - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/3919783_Identifying_Similar_Code_with_Program_Dependence_Graphs
PMD (software) - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/PMD_(software)
Rabin–Karp algorithm - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Rabin%E2%80%93Karp_algorithm
Rabin-Karp Algorithm for Pattern Searching - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/dsa/rabin-karp-algorithm-for-pattern-searching/
Using suffix trees and suffix arrays for duplicate code detection - Acadia Scholar, accessed July 3, 2025, https://scholar.acadiau.ca/node/2432
Understanding Suffix Trees and Arrays: Advanced Data Structures for String Processing, accessed July 3, 2025, https://algocademy.com/blog/understanding-suffix-trees-and-arrays-advanced-data-structures-for-string-processing/
Phoenix-Based Clone Detection Using Suffix Trees - Jeff Gray, accessed July 3, 2025, https://gray.cs.ua.edu/pubs/acmse-2006-robert.pdf
Suffix Trees, accessed July 3, 2025, https://web.stanford.edu/class/archive/cs/cs166/cs166.1146/lectures/10/Small10.pdf
Linear suffix tree : r/algorithms - Reddit, accessed July 3, 2025, https://www.reddit.com/r/algorithms/comments/92vb3n/linear_suffix_tree/
DECKARD: Scalable and accurate tree-based ... - InK@SMU.edu.sg, accessed July 3, 2025, https://ink.library.smu.edu.sg/cgi/viewcontent.cgi?article=2010&context=sis_research
A tiny code fragment demonstrating the concept of a program dependence graph, accessed July 3, 2025, https://www.researchgate.net/figure/A-tiny-code-fragment-demonstrating-the-concept-of-a-program-dependence-graph-the-listing_fig1_289645965
Manage Duplicated Code with Sonar | Sonar, accessed July 3, 2025, https://www.sonarsource.com/blog/manage-duplicated-code-with-sonar/
SonarQube Duplication Code Issue - Sonar Community, accessed July 3, 2025, https://community.sonarsource.com/t/sonarqube-duplication-code-issue/71208
SonarQube duplicate code detection - Sonar Community, accessed July 3, 2025, https://community.sonarsource.com/t/sonarqube-duplicate-code-detection/142587
pmd:cpd - Plugins - Apache Maven, accessed July 3, 2025, https://maven.apache.org/plugins/maven-pmd-plugin/cpd-mojo.html
Features - Simian Similarity Analyzer - Quandary Peak Research, accessed July 3, 2025, https://simian.quandarypeak.com/features/
SourcererCC: Scaling Code Clone Detection to Big-Code - Chanchal Roy, accessed July 3, 2025, https://clones.usask.ca/pubfiles/articles/SajnaniSourcererCCICSE2016.pdf
Core Concepts in ast-grep's Pattern, accessed July 3, 2025, https://ast-grep.github.io/advanced/core-concepts.html
For those who don't already know, this is built on tree-sitter (https://tree-sit... | Hacker News, accessed July 3, 2025, https://news.ycombinator.com/item?id=39779595
(PDF) Development of an algorithm for code clone detection in ..., accessed July 3, 2025, https://www.researchgate.net/publication/374320347_Development_of_an_algorithm_for_code_clone_detection_in_source_code_based_on_abstract_syntax_tree
Clone Detection Using Abstract Syntax Trees. - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/221307490_Clone_Detection_Using_Abstract_Syntax_Trees
Structural and Nominal Cross-Language Clone Detection - CS@UCSB, accessed July 3, 2025, https://sites.cs.ucsb.edu/~benh/research/papers/nichols19structural.pdf
Unified Abstract Syntax Tree Representation Learning for Cross-Language Program Classification - arXiv, accessed July 3, 2025, https://arxiv.org/pdf/2205.00424
cqfn/uast: Unified abstract syntax tree - GitHub, accessed July 3, 2025, https://github.com/cqfn/uast
Revisiting Code Similarity Evaluation with Abstract Syntax Tree Edit Distance - arXiv, accessed July 3, 2025, https://arxiv.org/html/2404.08817v1
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 3, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Scalable Code Clone Detection and Search based on Adaptive Prefix Filtering - Kostadin Damevski, accessed July 3, 2025, https://damevski.github.io/files/nishi_scalable_2017_preprint.pdf
Data Science in Action with Rust - Medium, accessed July 3, 2025, https://medium.com/@yuanli13/data-science-in-action-with-rust-fec669b19472
Index-Based Code Clone Detection: Incremental, Distributed, Scalable | Teamscale, accessed July 3, 2025, https://teamscale.com/publications/2010-index-based-code-clone-detection-incremental-distributed-scalable.pdf
An Overview of the Incremental Clone Detection Process - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/figure/An-Overview-of-the-Incremental-Clone-Detection-Process_fig2_221115244
How to reduce False Positive Alerts in Threat Detection: Sharpening Security Detections | by Tahir | Medium, accessed July 3, 2025, https://medium.com/@tahirbalarabe2/how-to-reduce-false-positive-alerts-in-threat-detection-sharpening-security-detections-d8382b93915a
Should automated code scanning target zero false positives? - GitGuardian Blog, accessed July 3, 2025, https://blog.gitguardian.com/should-we-target-zero-false-positives/
Reducing False Positives of Static Bug Detectors through Code Representation Learning - Xiang Gao, accessed July 3, 2025, https://gaoxiang9430.github.io/papers/saner24b.pdf
Improving Clone Detection Precision using Machine Learning Techniques - Chaiyong Ragkhitwetsagul, accessed July 3, 2025, https://cragkhit.github.io/publications/iwesep19_Vara.pdf
On the Use of Machine Learning Techniques Towards the Design of Cloud Based Automatic Code Clone Validation Tools - Chanchal Roy, accessed July 3, 2025, https://clones.usask.ca/pubfiles/articles/GMostaeenCodeCloneValidationPaper_SCAM18.pdf
CLCD-I: Cross-Language Clone Detection by Using Deep Learning with InferCode - MDPI, accessed July 3, 2025, https://www.mdpi.com/2073-431X/12/1/12
TCCCD: Triplet-Based Cross-Language Code Clone Detection - MDPI, accessed July 3, 2025, https://www.mdpi.com/2076-3417/13/21/12084
coala/coAST: Universal and language-independent abstract syntax tree - GitHub, accessed July 3, 2025, https://github.com/coala/coAST
Can a Abstract Syntax Tree be Compile by multiple Compiler or Interpreter? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/29326143/can-a-abstract-syntax-tree-be-compile-by-multiple-compiler-or-interpreter
Large Language Models (LLMs) for Source Code Analysis: applications, models and datasets - arXiv, accessed July 3, 2025, https://arxiv.org/html/2503.17502v1
LLM-powered Developer Automation - Mayfield, accessed July 3, 2025, https://www.mayfield.com/llm-powered-developer-automation/
AI/LLM Tools for Secure Coding | Benefits, Risks, Training - Security Journey, accessed July 3, 2025, https://www.securityjourney.com/ai/llm-tools-secure-coding
Analysing the Quantity of Duplicated Code in your Codebase - Matthew Cunningham, accessed July 3, 2025, https://aboutmatt.net/blog/2015/09/21/analysing-the-quantity-of-duplicated-code-in-your-codebase/
Visual Detection of Duplicated Code* - RMOD Files, accessed July 3, 2025, https://rmod-files.lille.inria.fr/Team/Texts/Papers/Rieg98aEcoopWorkshop.pdf
(PDF) Visual Detection of Duplicated Code - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/2866183_Visual_Detection_of_Duplicated_Code
Visualizing and Understanding Code Duplication in Large Software Systems - bac-lac.gc.ca, accessed July 3, 2025, https://dam-oclc.bac-lac.gc.ca/download?id=f22dea1e-ff51-450a-b6c3-5499d66402d2&fileName=thesis.pdf
LLMs in Automated Code Review: Transforming Software Development - Medium, accessed July 3, 2025, https://medium.com/@API4AI/llms-in-automated-code-review-transforming-software-development-7f4e4ab23db7
CloReCo: Benchmarking Platform for Code Clone Detection - SciTePress, accessed July 3, 2025, https://www.scitepress.org/publishedPapers/2025/136449/pdf/index.html
google ... - Hugging Face, accessed July 3, 2025, https://huggingface.co/datasets/google/code_x_glue_cc_clone_detection_big_clone_bench
Code Clone Benchmarks Overview - CEUR-WS, accessed July 3, 2025, https://ceur-ws.org/Vol-2217/paper-vis.pdf
Benchmarks for Software Clone Detection: A Ten-Year Retrospective, accessed July 3, 2025, https://www.cs.usask.ca/~croy/papers/2018/SANER2018/RoyCordySANER2018MIP.pdf
The Struggles of LLMs in Cross-Lingual Code Clone Detection - arXiv, accessed July 3, 2025, https://arxiv.org/html/2408.04430
Large Language Models for cross-language code clone detection - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/382971548_Large_Language_Models_for_cross-language_code_clone_detection
Repository-level Code Translation Benchmark Targeting Rust - arXiv, accessed July 3, 2025, https://arxiv.org/html/2411.13990v1
C4: Contrastive Cross-Language Code Clone Detection - Xing Hu's, accessed July 3, 2025, https://xing-hu.github.io/assets/papers/icpc-c8.pdf
A Survey of Software Clone Detection Techniques - International Journal of Computer Applications, accessed July 3, 2025, https://www.ijcaonline.org/archives/volume137/number10/24308-2016908896/
Clone Detection Using Abstract Syntax Trees - Leonardo de Moura, accessed July 3, 2025, https://leodemoura.github.io/files/ICSM98.pdf
Clone Detection Using Abstract Syntax Suffix Trees - SciSpace, accessed July 3, 2025, https://scispace.com/pdf/clone-detection-using-abstract-syntax-suffix-trees-2d8omteqb3.pdf
On the Impact of Multiple Source Code Representations on Software Engineering Tasks - An Empirical Study - arXiv, accessed July 3, 2025, https://arxiv.org/html/2106.10918v5
(PDF) Index-based code clone detection: Incremental, distributed, scalable - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/224185154_Index-based_code_clone_detection_Incremental_distributed_scalable
Gitor: Scalable Code Clone Detection by Building Global Sample Graph - arXiv, accessed July 3, 2025, https://arxiv.org/pdf/2311.08778
Large-Scale Code Clone Detection - eScholarship, accessed July 3, 2025, https://escholarship.org/uc/item/45r2308g
Incremental Clone Detection and Elimination for Erlang Programs - University of Kent, accessed July 3, 2025, https://kar.kent.ac.uk/id/document/23837
Incremental clone detection and elimination for erlang programs - SciSpace, accessed July 3, 2025, https://scispace.com/pdf/incremental-clone-detection-and-elimination-for-erlang-4aypg6tx36.pdf
CLONE DETECTION IN MODEL-BASED DESIGN - DiVA portal, accessed July 3, 2025, https://www.diva-portal.org/smash/get/diva2:1568943/FULLTEXT01.pdf
CLCDSA: Cross Language Code Clone Detection using Syntactical Features and API Documentation - Chanchal Roy, accessed July 3, 2025, https://clones.usask.ca/pubfiles/articles/Nafi_CLCDSAASE2019.pdf
Evaluating the Performance of Clone Detection Tools in Detecting Cloned Co-change Candidates - SciSpace, accessed July 3, 2025, https://scispace.com/pdf/evaluating-the-performance-of-clone-detection-tools-in-36r8ovj2.pdf
A Survey on the Evaluation of Clone Detection Performance and Benchmarking - Chanchal Roy, accessed July 3, 2025, https://clones.usask.ca/pubfiles/articles/SvajlenkoRoyBenchmarksSurvey.pdf
arXiv:1706.03934v1 [cs.SE] 13 Jun 2017, accessed July 3, 2025, https://arxiv.org/pdf/1706.03934
Cross-Language Clone Detection by Learning Over Abstract Syntax Trees - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/335496325_Cross-Language_Clone_Detection_by_Learning_Over_Abstract_Syntax_Trees
Large Language Models for cross-language code clone detection - arXiv, accessed July 3, 2025, https://arxiv.org/html/2408.04430v1
Towards Translating Real-World Code with LLMs: A Study of Translating to Rust - arXiv, accessed July 3, 2025, https://arxiv.org/pdf/2405.11514
Tools to Detect Clones in Batch mode and During Software Development - arXiv, accessed July 3, 2025, https://arxiv.org/pdf/1603.01661
Scalable Code Clone Detection and Search based on Adaptive Prefix Filtering, accessed July 3, 2025, https://www.researchgate.net/publication/321186002_Scalable_Code_Clone_Detection_and_Search_based_on_Adaptive_Prefix_Filtering
SourcererCC: Scaling Code Clone Detection to Big Code, accessed July 3, 2025, https://cs.uwaterloo.ca/~m2nagapp/courses/CS846/1171/papers/sajnani_icse16.pdf
GAST: A Generic AST Representation for Language-Independent Source Code Analysis, accessed July 3, 2025, https://www.redalyc.org/journal/5722/572276219003/html/
Multilingual Abstractions: Abstract Syntax Trees and Universal Dependencies - Gupea, accessed July 3, 2025, https://gupea.ub.gu.se/handle/2077/60331
AdaCCD: Adaptive Semantic Contrasts Discovery Based Cross Lingual Adaptation for Code Clone Detection, accessed July 3, 2025, https://ojs.aaai.org/index.php/AAAI/article/view/29749/31289
TransClone: A Language Agnostic Code Clone Detector | Request PDF - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/379176662_TransClone_A_Language_Agnostic_Code_Clone_Detector
SLACC: Simion-based Language Agnostic Code Clones - Chris Parnin, accessed July 3, 2025, https://www.chrisparnin.me/pdf/SLACC.pdf
Duplication detection - Google Groups, accessed July 3, 2025, https://groups.google.com/g/sonarqube/c/__KuV42aK4w
Question to 'Remove this clone implementation'-Code Smell Warning - Sonar Community, accessed July 3, 2025, https://community.sonarsource.com/t/question-to-remove-this-clone-implementation-code-smell-warning/109368
Clone with a reference was detected - warning - SonarQube Server / Community Build, accessed July 3, 2025, https://community.sonarsource.com/t/clone-with-a-reference-was-detected-warning/15060
GitLab Code Quality Duplication Analysis With PMD CPD - Aaron Goldenthal, accessed July 3, 2025, https://aarongoldenthal.com/posts/gitlab-code-quality-duplication-analysis-with-pmd-cpd/
CPD (PMD Core 6.55.0 API), accessed July 3, 2025, https://docs.pmd-code.org/apidocs/pmd-core/6.55.0/net/sourceforge/pmd/cpd/CPD.html
Why CPD Doesn't work for Same file duplicates? · pmd pmd · Discussion #5114 - GitHub, accessed July 3, 2025, https://github.com/pmd/pmd/discussions/5114
Rust Static Code Analysis - Sonar, accessed July 3, 2025, https://www.sonarsource.com/knowledge/languages/rust/
PMD, accessed July 3, 2025, https://pmd.github.io/
Tree-sitter: Introduction, accessed July 3, 2025, https://tree-sitter.github.io/
AST-T5: Structure-Aware Pretraining for Code Generation and Understanding - arXiv, accessed July 3, 2025, https://arxiv.org/html/2401.03003v2
Custom Language Support | ast-grep, accessed July 3, 2025, https://ast-grep.github.io/advanced/custom-language.html
Dossier: A tree-sitter based multi-language source code and docstring parser : r/rust - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/1980y0j/dossier_a_treesitter_based_multilanguage_source/
Getting Started with Tree-sitter: Syntax Trees and Express API Parsing - DEV Community, accessed July 3, 2025, https://dev.to/lovestaco/getting-started-with-tree-sitter-syntax-trees-and-express-api-parsing-5c2d
Using Tree Sitter to extract insights from your code and drive your development metrics, accessed July 3, 2025, https://colinwren.medium.com/using-tree-sitter-to-extract-insights-from-your-code-and-drive-your-development-metrics-8f52f95749d0
hydro-project/rust-sitter: Use Tree Sitter to parse your own languages in Rust - GitHub, accessed July 3, 2025, https://github.com/hydro-project/rust-sitter
Incremental Parsing Using Tree-sitter - Strumenta - Federico Tomassetti, accessed July 3, 2025, https://tomassetti.me/incremental-parsing-using-tree-sitter/
Dossier: a multi-language code and docstrings parser based on tree-sitter, accessed July 3, 2025, https://users.rust-lang.org/t/dossier-a-multi-language-code-and-docstrings-parser-based-on-tree-sitter/105354
Blog Post: Enabling low-latency, syntax-aware editing using Tree-sitter : r/rust - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/125zdyw/blog_post_enabling_lowlatency_syntaxaware_editing/
gaoya - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/gaoya
The Survey of the Code Clone Detection Techniques and Process with Types ( I , II , III and IV ) - Semantic Scholar, accessed July 3, 2025, https://www.semanticscholar.org/paper/The-Survey-of-the-Code-Clone-Detection-Techniques-(-Kaur-Sharma/f5600f495f863fd9f62ed29873d509939cd09ca0
Rabin–Karp algorithm - Semantic Scholar, accessed July 3, 2025, https://www.semanticscholar.org/topic/Rabin%E2%80%93Karp-algorithm/5414
Rabin-Karp Algorithm This following steps are the explanation for each... - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/figure/Rabin-Karp-Algorithm-This-following-steps-are-the-explanation-for-each-of-the-process-on_fig6_338363038
Source Code Clone Detection Using Unsupervised Similarity Measures - arXiv, accessed July 3, 2025, https://arxiv.org/html/2401.09885v1
Karp Rabin Algorithm, accessed July 3, 2025, https://cs.indstate.edu/~raddanki/pf.pdf
Approximate substring matching using a Suffix Tree - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/19368962/approximate-substring-matching-using-a-suffix-tree
A clone detector written in Rascal. - GitHub, accessed July 3, 2025, https://github.com/Nicasso/rascal-clone-detector
(PDF) DCCD: An Efficient and Scalable Distributed Code Clone Detection Technique for Big Code - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/328688945_DCCD_An_Efficient_and_Scalable_Distributed_Code_Clone_Detection_Technique_for_Big_Code
Secrets in Source Code: Reducing False Positives using Machine Learning - TU Wien, accessed July 3, 2025, https://secpriv.wien/fulltext/publik_302294.pdf
Automating detection of security vulnerabilities and bugs in CI/CD pipelines using Amazon CodeGuru Reviewer CLI | AWS DevOps & Developer Productivity Blog, accessed July 3, 2025, https://aws.amazon.com/blogs/devops/automating-detection-of-security-vulnerabilities-and-bugs-in-ci-cd-pipelines-using-amazon-codeguru-reviewer-cli/
A Language Independent Approach for Detecting Duplicated Code - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/2430208_A_Language_Independent_Approach_for_Detecting_Duplicated_Code
Rust library for multiple languages (JS, Python, etc) - help, accessed July 3, 2025, https://users.rust-lang.org/t/rust-library-for-multiple-languages-js-python-etc/93015
Kixiron/rust-langdev: Language development libraries for Rust - GitHub, accessed July 3, 2025, https://github.com/Kixiron/rust-langdev
Any example in other programming languages where values are cloned without obviously being seen? - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/183v1ey/any_example_in_other_programming_languages_where/
Heuristic analysis technology applied to anti-spam filtering solutions - Altospam, accessed July 3, 2025, https://www.altospam.com/en/glossary/heuristic-analysis/
Reducing false positive incidental findings with ensemble genotyping and logistic regression-based variant filtering methods, accessed July 3, 2025, https://pmc.ncbi.nlm.nih.gov/articles/PMC4112476/
Automated Code Generation with Large Language Models (LLMs) | by Sunny Patel, accessed July 3, 2025, https://medium.com/@sunnypatel124555/automated-code-generation-with-large-language-models-llms-0ad32f4b37c8
Using LLMs to Adjudicate Static-Analysis Alerts - YouTube, accessed July 3, 2025, https://www.youtube.com/watch?v=s3wgXgPTf4k
Code Duplication: Best Practices for Maintainable Programming - Metridev, accessed July 3, 2025, https://www.metridev.com/metrics/code-duplication-best-practices-for-maintainable-programming/
Duplicate Lines of Code - SciTools Blog, accessed July 3, 2025, https://blog.scitools.com/duplicate-lines-of-code/
[2506.10995] Evaluating Small-Scale Code Models for Code Clone Detection - arXiv, accessed July 3, 2025, https://arxiv.org/abs/2506.10995
abstract syntax tree for imperative languages - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/31350609/abstract-syntax-tree-for-imperative-languages
