
Report on Dead Code Detection: Best Practices & Standards for Uveddi

Executive Summary
Effective dead code detection is a critical component of modern software engineering, essential for maintaining codebase health, reducing security risks, and improving developer velocity. Analysis reveals that a robust strategy must move beyond traditional, single-file linters to whole-program analysis tools that build a complete dependency graph. Such tools are capable of accurately distinguishing between truly dead code and symbols that are part of a project's public or internal API. The most effective approaches are language-specific, leveraging the unique strengths and compensating for the weaknesses of each ecosystem. For TypeScript/JavaScript, this involves using a specialized graph-based tool like Knip. For Python, a tool like Vulture, which employs confidence scoring to manage the language's dynamism, is paramount. For Rust, the compiler's built-in liveness analysis provides the most accurate results when paired with idiomatic project structuring. This report recommends that Uveddi adopt a multi-layered, language-specific strategy, integrating these best-in-class tools into a continuous integration pipeline with carefully managed configurations to balance precision with practicality, thereby resolving current test failures and establishing a sustainable process for code hygiene.

1. The Landscape of Dead Code Analysis

Dead code is a deceptive and pervasive form of technical debt. It clutters codebases, increases cognitive load on developers, bloats bundle sizes, and can even introduce security vulnerabilities by retaining dependencies on unmaintained libraries.1 To effectively combat it, it is necessary to first understand its various forms and the analytical techniques used to detect it.

1.1. Defining "Dead Code": Beyond Unused Variables

While often conflated with simple unused variables, the term "dead code" encompasses a spectrum of software artifacts that are present in the source but have no effect on the program's runtime behavior. A precise taxonomy is essential for selecting the right detection tools and strategies.
Unreachable Code: This is the most definitive form of dead code. It consists of executable statements that can never be reached due to the program's control flow. Common examples include code placed immediately after a return, throw, break, or continue statement, or code within a conditional block whose condition is statically provable as false (e.g., if (false) {... }).2 Most compilers and basic static analyzers can detect this form of dead code with very high confidence.
Unused Symbols: This category is the primary focus of most dead code detection efforts and includes declared variables, functions, classes, methods, and properties that are never read or invoked. While a local variable that is written to but never read is a simple case, the real challenge lies in identifying exported or public symbols that are never used by any other module in the project.6 This requires a global, project-wide view of the codebase.
Statically Unreachable vs. Truly Dead Code: A critical distinction exists between code that has no static references (e.g., no import or require statements pointing to it) and code that is genuinely unused. Many modern applications leverage dynamic invocation mechanisms like reflection, dependency injection, or string-based lookups, which create usages that are invisible to purely static analysis.9 Code that appears statically unreachable may, in fact, be a critical entry point at runtime. This gap is the primary source of false positives in dead code detection.
Dead Code as a Symptom: The presence of dead code is rarely a benign issue of mere clutter. It is often an indicator of deeper problems within the development process. It can signal an incomplete or failed refactoring effort, where the old code was not fully removed.2 It can represent abandoned features or experimental code that was never cleaned up. In a security context, dead code can pose a significant risk by retaining dependencies on vulnerable libraries or containing outdated, insecure logic that, while not actively called, expands the potential attack surface of the application.2

1.2. Foundational Analysis Techniques

Modern dead code detection tools are built upon decades of computer science research in program analysis. Understanding these foundational techniques is key to appreciating their capabilities and limitations.
Lexical, Syntax (AST), and Semantic Analysis: The first step for any analyzer is to understand the code. This process mirrors the initial stages of a compiler. Lexical analysis breaks the source code into a stream of tokens. Syntax analysis then parses these tokens into a structured representation, typically an Abstract Syntax Tree (AST), which models the code's grammatical structure.11 Finally, semantic analysis traverses the AST to understand the meaning and relationships between symbols, such as linking a variable usage to its declaration.11 It is at this stage that a tool can differentiate between a function
definition and a function call.13
Control-Flow and Data-Flow Analysis: Building on the AST, tools construct a Control-Flow Graph (CFG), which maps all possible execution paths through a function or program.11 By analyzing the CFG, a tool can identify nodes (blocks of code) that have no incoming edges, thus flagging them as unreachable code.
Data-Flow Analysis complements this by tracking the state of variables (e.g., defined, used, uninitialized) along these execution paths, which is crucial for finding unused local variables or potential null pointer dereferences.11
Whole-Program Call Graph Construction: This technique is the cornerstone of modern, effective dead code detection for unused symbols. Unlike basic linters that analyze files in isolation, advanced tools perform a whole-program analysis. They start from a set of known "live" entry points (e.g., a main function, a web server's startup script) and build a call graph by recursively tracing every function call and module import across the entire project.1 Any symbol not included in this final graph is considered dead. This is the only reliable static method for finding unused
exported functions and classes.15
The Role of Dynamic (Runtime) Analysis: While static analysis is powerful, it is fundamentally an approximation of a program's behavior. The "gold standard" for confirming dead code is dynamic analysis, which involves instrumenting the code and observing it as it runs, typically under a comprehensive test suite or in a production environment.2 Code coverage tools are a form of dynamic analysis; code that is never executed during tests is a strong candidate for being dead.5 The most sophisticated systems, such as Meta's Systematic Code and Asset Removal Framework (SCARF), combine static and dynamic analysis. They first build a static dependency graph and then augment it with data from runtime monitoring, which tracks dynamic references and actual function invocations in production. This hybrid approach significantly reduces false positives and provides the high level of confidence required for fully automated code deletion.14
The evolution of dead code detection tools is a direct reflection of the increasing complexity of software. In the era of monolithic scripts, simple, single-file analysis was adequate. However, the widespread adoption of modular architectures—such as CommonJS, ES Modules, and Python packages—created a fundamental problem: a function exported from one file and used in another would be incorrectly flagged as "unused" by a linter that only analyzed the first file in isolation.1 This limitation drove the development of tools capable of whole-program analysis, which build a project-wide dependency graph to understand these cross-module relationships.1 The subsequent rise of complex frameworks like React, Angular, and Next.js introduced another layer of complexity with "magic" behaviors, such as file-based routing or dependency injection, where code is used without explicit
import statements. This, in turn, has pushed the state-of-the-art towards framework-aware tools like Knip, which use a sophisticated plugin system to understand these implicit dependencies and further refine the accuracy of the call graph.15 The sophistication of the tool, therefore, is a direct and necessary response to the sophistication of the codebase it is designed to analyze.

2. Tooling Analysis: Language-Specific Ecosystems

The optimal approach to dead code detection is not universal; it is deeply rooted in the specific characteristics and idioms of each programming language. A tool that excels in one ecosystem may be ill-suited for another. This section analyzes the leading tools and methodologies for JavaScript/TypeScript, Python, and Rust, as well as the language-agnostic platform SonarQube.

2.1. JavaScript/TypeScript: Linters vs. Graph-Based Tools

The JavaScript/TypeScript ecosystem offers a clear evolutionary path from basic, file-scoped linters to powerful, project-wide graph analyzers.
ESLint (no-unused-vars, eslint-plugin-import):
Core Functionality: ESLint, via its core no-unused-vars rule, is highly effective for identifying unused local variables, function parameters, and private class members within a single file.6 The rule is extensively configurable, offering options like
varsIgnorePattern and argsIgnorePattern to accommodate common coding conventions, such as prefixing intentionally unused variables with an underscore (_).7
The Export Problem: The fundamental limitation of no-unused-vars for project-wide dead code detection is that it considers any exported symbol to be "used" by definition.19 This prevents it from identifying exported functions or components that are never imported by any other module. The
/* exported */ comment directive is a legacy feature for non-module environments and is not a viable solution for modern JavaScript development.7
Bridging the Gap (eslint-plugin-import): The eslint-plugin-import package, specifically its import/no-unused-modules rule, attempts to address this limitation by reporting exports that do not have a corresponding static import in another module within the project.19 While an improvement, this approach can be brittle and difficult to configure correctly, especially in the face of dynamic imports or complex project structures.
TypeScript Integration: For TypeScript projects, using the @typescript-eslint/no-unused-vars rule is not optional; it is essential. The base ESLint rule cannot understand TypeScript's syntax and will generate a high volume of false positives for types, interfaces, and enums.22 The TypeScript-specific version correctly handles these constructs, including the important distinction of flagging variables that are used
only for type information (e.g., type T = typeof myVar) as unused at runtime.22
The Rise of Specialized Tools: Knip and ts-prune:
Fundamental Shift: In contrast to ESLint's file-by-file approach, tools like Knip and ts-prune operate by building a complete module dependency graph of the entire project.1 They begin with a set of user-defined entry points and trace all
import and require statements to map out every dependency. This graph-based methodology is fundamentally more robust for identifying genuinely unused exports.
Knip: Knip represents the current state-of-the-art in the TypeScript/JavaScript ecosystem.21 Its standout feature is an extensive plugin system that provides out-of-the-box support for over 100 common frameworks and tools, including Next.js, Storybook, Jest, and Webpack.15 These plugins automatically identify implicit entry points (e.g., test files, Storybook stories, Next.js pages) and parse framework-specific configuration files, drastically reducing manual setup and the likelihood of false positives.34 Knip extends beyond dead code to find unused
npm dependencies and unreferenced files, and it offers an auto-fix capability to automatically remove the identified clutter.1
ts-prune: As an earlier pioneer of the graph-based approach, ts-prune established the core concepts that Knip later refined.27 It is effective but lacks Knip's sophisticated plugin architecture and broader feature set. Its configuration primarily revolves around manually specifying entry points and ignore patterns.37 The tool is now in maintenance mode, and its creator officially recommends migrating to Knip.39
Handling Modern Contexts:
Type-Only Imports: The @typescript-eslint/no-unused-vars rule correctly handles the modern TypeScript pattern of importing a value only to use its type, flagging the runtime variable as unused.22
Frameworks: The "magic" of modern frameworks, where code is consumed implicitly, is best handled by Knip's plugin architecture. This is the industry-standard solution for accurately analyzing projects built with tools like React, Next.js, or Svelte, which have numerous implicit entry points and configuration-driven behaviors.15

2.2. Python: Navigating a Dynamic Landscape

The dynamic nature of Python presents unique challenges for static analysis, which has led to the development of specialized tools that embrace uncertainty.
Pylint (unused-import, unused-private-member):
Functionality: Much like ESLint, Pylint is a general-purpose linter that is proficient at finding unused imports and variables within the scope of a single module.8 It provides specific checks such as
unused-import (W0611) and unused-private-member (W0238).40
Limitations: Pylint's module-scoped analysis is its primary weakness for dead code detection. It frequently generates false positives by flagging a public function or class in one module as an "Unused variable" (W0612) even when it is imported and used correctly in another module within the same project.20 Furthermore, it has historically struggled with imports that are used only in type annotations, often requiring specific configuration workarounds or plugins to function correctly.41
Vulture: A Specialized Approach:
Purpose-Built: Vulture was created specifically to find dead code in Python projects, explicitly designed to overcome the limitations of general-purpose linters in a dynamic language.13
Confidence Scoring: Vulture's defining feature is its use of a confidence score, a numerical value from 60% to 100% assigned to each finding.13 This is a pragmatic acknowledgment that static analysis cannot always be certain in Python. A score of 100% is reserved for code that is provably unreachable (e.g., code after a
return statement). A score of 90% is assigned to unused imports. A score of 60% is given to unused functions, methods, and classes, as these are the most common source of false positives due to dynamic invocation.42 This scoring mechanism allows developers to tune the tool's sensitivity.
Whitelists: To manage the inevitable false positives that arise from Python's dynamism, Vulture's primary mitigation strategy is the use of a whitelist.py file.13 This is a separate Python file where developers can write code that "simulates" the usage of symbols that are invoked dynamically (e.g., through
getattr or by a web framework). When this whitelist is included in the analysis, Vulture sees these symbols as "used" and removes them from its report.
The Challenge of Dynamic Features: Python's dynamism is the root cause of the challenges for static analysis. Code that is invoked via string manipulation (e.g., getattr(obj, 'method_name')), loaded by dependency injection frameworks, or rendered in templates is invisible to a static analyzer that is simply reading the source code. This leads to a high rate of false positives that must be actively managed with mechanisms like Vulture's whitelists.13

2.3. Rust: Compiler-Integrated Liveness Checking

In Rust, dead code analysis is not an afterthought handled by third-party tools; it is a core feature of the language and its compiler, reflecting Rust's philosophy of catching errors at compile time.
The dead_code Lint: The primary mechanism for dead code detection is the dead_code lint, which is built directly into the Rust compiler, rustc.46 This lint is on by default and will issue warnings for any code that is not part of the crate's public API and is never used.
Clippy: While Clippy is Rust's official and powerful linter, it primarily builds upon the compiler's foundational analysis. It offers a vast array of additional style, performance, and correctness lints, but the core dead_code logic resides within rustc itself.49 Lints in the Rust ecosystem are highly configurable, either through in-code attributes like
#[allow(dead_code)], a project-level clippy.toml file, or directly in Cargo.toml.47
The Crate System: Libraries vs. Binaries: The behavior of the dead_code lint is intelligently context-dependent, based on the type of crate being compiled:
Applications (Binaries): When compiling a binary crate (a project with a src/main.rs), the compiler has a complete view of the program, starting from the main function. It can trace all reachable code paths. In this context, any function, struct, or other item—whether public (pub) or private—that is not referenced will be correctly flagged as dead code.
Libraries: When compiling a library crate (a project with a src/lib.rs), the compiler makes a crucial assumption: any pub item is considered part of the library's public API and could potentially be used by an external, unknown crate. Therefore, to avoid false positives, the compiler will not issue a dead_code warning for unused public items in a library crate.46 It will, however, continue to correctly flag unused
private (non-pub) items within the library.
Managing Lints in Complex Projects: A common scenario that can cause confusing dead_code warnings occurs in projects with a workspace containing multiple binaries that share common utility modules. If a file like utils.rs is included and compiled separately as part of each binary, a function used by binary_one but not binary_two will be flagged as dead code during the compilation of binary_two.46
The Idiomatic Solution: The correct and idiomatic Rust solution is architectural. The shared code should be extracted into its own library crate within the workspace. The binary crates then declare a dependency on this shared library. This ensures the shared code is compiled only once as a library, where its public API is correctly treated as "live," and the binaries then consume this API. This approach aligns with Rust's emphasis on disciplined project structure and resolves the false positive warnings at their source.46

2.4. Multi-Language Platforms: The SonarQube Approach

SonarQube provides a centralized platform for static analysis across numerous languages, but its approach to dead code is notably conservative, prioritizing the reduction of false positives above all else.
Core Philosophy: SonarQube's design is heavily influenced by the need to maintain developer trust across large, diverse organizations. A high rate of false positives leads to "alert fatigue," causing developers to ignore warnings or disable rules entirely. Consequently, SonarQube's default rule sets are carefully calibrated to be as precise as possible.9
Analysis of Private vs. Public APIs:
Private/Protected Members: SonarQube provides rules to detect unused private methods and fields (e.g., for Java). These are considered "safe" to flag because their usage is confined to the scope of a single class or its descendants, which can be analyzed with high confidence within a single project.52
Public APIs: SonarQube intentionally does not provide a rule to detect unused public classes or methods.9 The rationale is that such a rule would be extremely prone to false positives. The analyzer cannot know if a public method is an entry point for a framework using reflection, is called from a configuration file, or is part of a library's public contract for external consumers.9
Recommended Alternative: Instead of an automated rule, SonarQube advocates for a manual, developer-driven process for removing public APIs. If a developer suspects a public method is unused, the recommended practice is to first mark it with the @Deprecated annotation (or its equivalent in other languages). SonarQube does provide a rule that flags the usage of deprecated methods. This creates a safe, explicit deprecation cycle: first, mark the API as deprecated; then, over time, use SonarQube to find and eliminate all calls to it; finally, once all usages are gone, the API can be safely removed.9
Contextual Blindness: SonarQube's static analysis operates without a complete understanding of a project's runtime context or its role as an application versus a library. This "contextual blindness" forces it to adopt a conservative stance. It cannot reliably distinguish a public method intended only for use between two internal packages from one that is a core part of a product's published API.9 This is a deliberate design trade-off that prioritizes safety and low noise over the aggressive detection of potentially dead code. Similarly, SonarSource generally recommends against analyzing third-party library source code, as any issues found are typically outside the control of the development team and thus constitute actionable noise.53
The choice of the "best" tool for a given language is not merely a matter of feature comparison; it is deeply intertwined with the language's own design principles and philosophy. Rust, a language founded on compile-time correctness and static guarantees, logically integrates its dead code detection directly into the compiler. The solutions to common issues are not just configuration tweaks but architectural changes, such as organizing code into distinct library and binary crates, which reinforces the language's emphasis on disciplined project structure. In contrast, Python is a language celebrated for its dynamism and flexibility. Consequently, its premier dead code tool, Vulture, must pragmatically acknowledge the inherent limitations of static analysis in such an environment. The invention and use of "confidence scores" and "whitelists" are not signs of a tool's failure but are rather a clever and necessary adaptation to the language's nature, trading absolute certainty for actionable, tunable heuristics. The JavaScript/TypeScript ecosystem, defined by its breakneck evolution and a vast landscape of competing frameworks, has produced tooling that mirrors this environment. Simple linters like ESLint were quickly outpaced by framework complexity, creating a need for more powerful graph-based analyzers like Knip, whose success is built upon a rich, extensible plugin system designed to adapt to any new framework that emerges. Finally, a language-agnostic platform like SonarQube, designed for enterprise-wide deployment, must prioritize cross-team consistency and risk management. Its highly conservative, low-false-positive stance is a rational trade-off for a tool that must operate reliably across dozens of languages and diverse development teams, where a "one-size-fits-all" approach must favor safety over aggressive code removal. This demonstrates that a single, unified dead code detection strategy across all languages is unlikely to be optimal; the most effective approach must be tailored to the unique character of each technology stack.

3. A Deep Dive into Confidence and Context

The effectiveness of a dead code detection system hinges on two critical concepts: the confidence of its findings and its ability to understand the context in which the code operates. An analysis that is high-confidence and context-aware produces actionable results, while one that is low-confidence and context-agnostic generates noise and erodes developer trust.

3.1. Confidence Scoring Mechanisms

Confidence in static analysis is a measure of the certainty that a reported issue is a true positive. Tools express this confidence either through explicit numerical scores or implicitly through their default behaviors and configuration options.
Explicit Scoring: The Vulture Model:
The Python tool Vulture provides the clearest implementation of an explicit confidence score, assigning a percentage from 60% to 100% to each piece of potentially dead code it identifies.13
Calibration: These scores are not arbitrary; they are calibrated based on the type of the code construct, which directly correlates to the likelihood of it being invoked in a way that is invisible to static analysis. For instance, code within a block that is provably unreachable (e.g., if False:...) is given a 100% confidence score. Unused imports receive a 90% score. Unused variables, functions, and classes—the items most susceptible to being used dynamically—receive a lower 60% confidence score.42 This is a heuristic model that quantifies the inherent uncertainty of analyzing a dynamic language.
Influence: This model empowers developers to directly control the signal-to-noise ratio. By using the --min-confidence command-line flag, a team can tune the tool's output. Setting --min-confidence 100 will report only code that is guaranteed to be dead, eliminating all false positives at the cost of missing some potentially dead code. A more common setting, such as 80, will include unused imports and variables while filtering out the lower-confidence (and more likely to be false positive) function and class findings.13
Implicit Confidence: Rule Severity and Conservative Defaults:
Most other static analysis tools express confidence implicitly through their rule configuration systems. The severity levels of error, warn, and off (or deny, warn, allow in Rust) serve as a proxy for confidence.51 A rule that is enabled as an
error by default implies that the tool's authors have high confidence in its accuracy. A rule that defaults to warn or is off by default suggests lower confidence or that it addresses a more stylistic or opinionated issue.
SonarQube's entire philosophy is an exercise in implicit confidence management. Its deliberate decision not to implement a rule for unused public APIs is a statement that their confidence in the accuracy of such a check would be too low to meet their quality standards.9 They prioritize having a smaller set of high-confidence rules over a larger, noisier set.
Similarly, Clippy's lint groups, such as nursery (for new, experimental lints) or pedantic (for very opinionated or aggressive lints), are opt-in, signaling that the findings from these groups have a lower confidence or higher false-positive rate than the default set.
Achieving "Provable" Unreachability: The Hybrid Approach:
The path to achieving near-100% confidence in dead code detection, especially in large-scale systems, involves moving beyond pure static analysis. Meta's SCARF framework provides the blueprint for this state-of-the-art approach: augmenting a statically generated call graph with data gathered from runtime analysis.14
This hybrid model works by first performing a comprehensive static analysis to identify all potentially dead code. Then, it cross-references this list against logs and metrics from the production environment that track actual function calls, API endpoint hits, template renderings, and dynamic dispatches. A symbol that is both statically unreferenced and never observed at runtime can be removed with extremely high confidence. This methodology effectively closes the gap left by dynamic invocation and allows for the implementation of fully automated code deletion pipelines.14 While this represents the pinnacle of accuracy, it requires a significant investment in observability and data analysis infrastructure.

3.2. Context Handling: Application vs. Library Code

The single most important piece of context for a dead code analyzer is the nature of the project itself: is it a self-contained application or a reusable library? The answer fundamentally changes the definition of "dead code."
The Core Distinction: The difference lies in the definition of the project's public API and its entry points.
Application: An application has a finite, knowable set of entry points. These are the starting points for the program's execution, such as a main.ts or server.py file, the bin field in a package.json, or the route handlers of a web server. The analysis tool can start its graph traversal from these points and trace all reachable code. Any symbol not connected to this graph can be confidently considered dead.27
Library: A library, by contrast, is designed to be consumed by other, unknown applications. In this context, every exported or public symbol is a potential entry point for an external consumer. The analyzer cannot assume that any public symbol is unused simply because it is not called from within the library itself. Doing so would lead to incorrectly removing parts of the library's intended public API.9
Configuration Strategies for Different Project Archetypes:
Applications: The correct strategy is to provide the tool with an explicit list of entry points. Knip is particularly strong here, as its entry configuration option and its plugin system are designed to automatically discover these entry points from common project structures and framework configurations.35 For a tool like Vulture, the configuration is simpler: one points it at the application's source directories, and it analyzes the entire codebase as a single unit.13
Libraries (The Public API Challenge): Configuring dead code detection for a library is more complex and involves choosing from several strategies:
Strategy 1: Ignore Public Exports (The Safe Default). The most conservative and safest approach is to configure the tool to only report unused private or internal members. This effectively treats all public symbols as "live" by default. This is the default behavior of the Rust compiler for library crates and is the only behavior supported by SonarQube's dead code rules.52
Strategy 2: Define the Public API Explicitly. For libraries that have a well-defined public surface, often exposed through a single index.ts or __init__.py file, one can configure the tool to treat these files as the sole entry points. This allows the tool to find and flag "internal" exported symbols that are not part of the intended public API. Knip's entry and project file configurations can be tailored for this use case.56
Strategy 3: Test-Driven Liveness. A common and pragmatic approach is to define dead code as code that is only used by test files. The assumption is that if a piece of code has no production call path and is only referenced in tests, it is either a remnant of a removed feature or a test-specific utility that shouldn't be part of the main codebase.27 Tools like ts-prune and Knip provide options to ignore test files when resolving usages (e.g., the
-s flag in ts-prune), which effectively surfaces this category of code.37
Strategy 4: Whole-Program Analysis on a Consumer. The most accurate method for finding dead code in a library is to run the whole-program analysis tool not on the library itself, but on a known application that consumes the library. This provides the tool with the necessary context of how the library's public API is actually used. While this provides the highest-fidelity results, it can be complex to orchestrate within a CI/CD pipeline.
The concept of "confidence" in dead code detection should not be viewed as an intrinsic property of a tool, but rather as an emergent property of a complete system. This system comprises the tool itself, the language being analyzed, the project's architecture, and the runtime environment. A tool's raw output is merely a list of potentially unused symbols. The language's own features, particularly its degree of dynamism, establish the baseline probability of that output being correct; Rust's static nature provides a high baseline, while Python's begins lower. The project's architecture—whether it is an application or a library—provides the crucial context, where a finding in a self-contained application is inherently higher confidence than the same finding in a library's public API. The tool's configuration, such as Vulture's --min-confidence flag or Knip's entry point definitions, allows developers to inject their own domain knowledge into the analysis, further refining the results and increasing the confidence of the final report. Ultimately, runtime data, whether from code coverage metrics or production monitoring, serves as the ground truth, capable of elevating confidence to near-certainty. Therefore, a mature engineering organization does not simply "run a tool"; it builds a system for managing the inherent uncertainty of static analysis. This system includes robust CI/CD integration, disciplined configuration management, and a continuous feedback loop for refining rules and whitelists, allowing the organization to consciously move along the spectrum of confidence to meet its specific goals for code quality and safety.

4. The Challenge of False Positives

A false positive occurs when a static analysis tool incorrectly flags a piece of code as dead when it is, in fact, used at runtime. Managing false positives is arguably the most critical aspect of implementing a successful dead code detection strategy, as a high volume of incorrect warnings will quickly lead to developers ignoring the tool's output entirely.

4.1. Common False Positive Patterns Across Languages

While some false positives are tool-specific, many patterns are common across all languages, typically stemming from the gap between static analysis and runtime behavior.
Dynamic Invocation & Reflection: This is the most prevalent source of false positives. It includes any mechanism where code is called indirectly via a string or runtime type information. Examples include getattr() in Python, reflection APIs in Java (often used by frameworks like Spring), and dynamically constructing module or function names in JavaScript.9
Framework-Specific "Magic": Modern application frameworks rely heavily on conventions and "magic" to reduce boilerplate, but this often creates usages that are invisible to static analyzers. Common patterns include:
Dependency Injection (DI): DI containers that instantiate and inject classes based on configuration or type annotations without explicit new calls.
File-Based Routing: Web frameworks like Next.js, where a file in a pages/ directory automatically becomes a web route without being explicitly imported by user code.61
Decorators/Annotations: Frameworks like Flask (@app.route) or NestJS use decorators to register functions or classes. The decorated code is used by the framework, but the static link is not obvious.13
Test Framework Features: Test frameworks that use string-based references, such as JUnit 5's @MethodSource annotation, can cause the referenced provider method to be flagged as unused.62
Code Referenced Only by Tests: A frequent and philosophically debatable category is code that is exported from the main source set but is only ever imported by files in the test set.19 While technically "dead" from a production code perspective, this code is necessary for testing. Whether to flag it depends on the team's policy regarding test-only helpers.
External Configuration / Entry Points: Code can be invoked from outside the source code itself. This includes class names specified in XML configuration files (e.g., plugin.xml), functions called from shell scripts, or code triggered by database events.9
Peer Dependencies: In the npm ecosystem, a package may rely on a peer dependency, which it expects the consuming application to provide. The package itself does not have a direct import statement for the peer dependency, which can lead to it being flagged as unused.63
Polyfills and Conditional Code: Code inside a conditional block that a static analyzer cannot resolve may be incorrectly pruned. A common developer practice of using if (false) {... } to temporarily disable code will correctly be identified as unreachable, but this may be intentional during development.13

4.2. Mitigation Strategies

A robust strategy for mitigating false positives requires a multi-pronged approach, using a combination of broad configuration, precise code-level annotations, and well-defined team processes.
Configuration-Based (The Broadsword): These are high-level settings that exclude large portions of the codebase from analysis.
Ignore Paths/Patterns: The first and most essential step is to configure the tool to completely ignore directories that do not contain first-party source code. This always includes node_modules, build output directories (dist, build), and often includes configuration files or test-specific directories.44
Ignore Names/Decorators: For languages and frameworks with strong naming conventions, ignoring symbols that match a certain pattern can be very effective. Vulture's --ignore-names and --ignore-decorators options are powerful tools for this, allowing a team to, for example, ignore all Flask route handlers by default.13
Code-Based (The Scalpel): These are fine-grained controls applied directly in the source code to suppress a specific, known false positive.
Inline Comments: Most tools support a special comment format to disable a warning on the subsequent line or for the current line. Examples include // ts-prune-ignore-next, Pylint's # pylint: disable=..., and the # noqa comments used by Flake8 and supported by Vulture.13 This approach is best for isolated, well-understood exceptions.
Attributes: Rust's attribute system provides a clean, syntactically integrated way to manage lints. The #[allow(dead_code)] attribute can be applied to a single function, a module, or even an entire crate to selectively disable the warning.46
Variable Naming Conventions: A widely supported convention is to prefix an intentionally unused variable with an underscore (_). Many tools, including the TypeScript ESLint plugin and Vulture, can be configured to automatically ignore variables that follow this pattern.13
Whitelist-Based (The Maintained List):
This approach, best exemplified by Vulture's whitelist.py files, provides a middle ground between broad configuration ignores and scattered inline comments. A central whitelist file is created that contains code to simulate the usage of known false positives. This whitelist is then passed to the analyzer during its run.13 This method has the advantage of centralizing the exceptions, making them easier to review and maintain, rather than having them spread throughout the codebase. Vulture can even auto-generate a starting whitelist to simplify the process.42
Architectural (The Foundation):
Often, the most effective way to eliminate a false positive is to refactor the code to be more statically analyzable. The Rust example of solving multi-binary dead_code warnings by extracting shared code into a proper library crate is a perfect illustration of this principle.46 Another example is grouping all classes that are instantiated via reflection into a dedicated package, which can then be easily and safely ignored in the tool's configuration.10
Process-Based (The Human Loop):
Tuning and Iteration: When introducing a dead code tool to a large, existing codebase, it is crucial to do so gradually. Start by running the tool in a "warn-only" mode that does not fail the build. This allows the team to assess the initial report, identify the major sources of false positives, and begin building out the necessary configurations and whitelists. The rules can be progressively tightened as confidence in the tool's output grows.37
Feedback Loop: The process of managing false positives should be continuous. When a developer encounters a false positive during code review, there should be a clear process for triaging it and updating the central configuration or whitelist. This feedback loop ensures that the tool's accuracy improves over time and adapts to the evolving codebase.14
The implementation of a dead code detection tool is not a one-time setup task; it is better understood as an ongoing process of negotiation between the development team and the analysis tool. When a tool is first run on a mature codebase, it will almost invariably generate a large volume of findings, many of which will be false positives stemming from the project's specific frameworks, architectural patterns, and dynamic behaviors.9 The initial, and natural, reaction from the team is often frustration, putting the tool at risk of being dismissed or disabled due to "alert fatigue".9 To derive value from the tool, the team must actively engage in the mitigation strategies outlined above—configuring ignore paths, building whitelists, and applying targeted suppressions. This is, in effect, a process of "teaching" the tool about the project's unique context and conventions. This process is not static. As the codebase evolves and new technologies or patterns are introduced, new classes of false positives will emerge. Therefore, the tool's configuration and whitelists must be treated as living documents, maintained and version-controlled alongside the source code itself. The true cost of a dead code detection tool is not merely its license or initial setup time, but the ongoing engineering effort required to maintain its configuration. This implies that tools with superior defaults and more intelligent, context-aware analysis, such as Knip with its extensive plugin ecosystem, offer a significantly lower total cost of ownership by minimizing the burden of this continuous negotiation.

5. Comparative Analysis and Trade-offs

Choosing the right dead code detection strategy requires a clear understanding of the trade-offs between different tools and their underlying philosophies. No single tool is universally superior; the optimal choice depends on the specific language, project type, and organizational goals.

5.1. Table: Comparison of Dead Code Detection Approaches

The following table provides a consolidated comparison of the primary tools discussed in this report, evaluated against key features relevant to dead code detection.

Feature
ESLint (w/ plugins)
Knip
Pylint
Vulture
Clippy / rustc
SonarQube
Primary Mechanism
File-scoped AST analysis. import plugin attempts cross-module checks. 1
Whole-project call graph analysis from entry points. 1
File-scoped AST analysis with type inference. 66
Whole-project AST analysis, tracking definitions and uses. 13
Compiler-integrated liveness analysis within a crate. 46
Multi-language analysis, primarily focused on intra-procedural issues. 52
Exported Symbol Handling
Considers exports "used" by default. no-unused-modules plugin required for basic checks. 19
Excellent. Core feature. Traces usage from entry points across the entire project. 1
Poor. Often flags publicly used symbols as unused if not used in the same file. 20
Good. Scans all provided paths to find usages of exported symbols. 13
Context-dependent. Public items in libraries are considered "live"; public items in binaries are checked. 46
Conservative. Intentionally does not flag unused public APIs to avoid false positives. Recommends deprecation instead. 9
Confidence Model
Implicit (Rule severity: warn/error). 55
Implicit (Binary: used/unused). High confidence due to graph analysis. 21
Implicit (Rule severity). 64
Explicit. Assigns a 60-100% confidence score to each finding. 13
Implicit (High confidence). Built into the compiler's strict semantics. 46
Implicit (High confidence on a limited rule set). Prioritizes low false positives. 9
Library Context Support
Poor. Requires manual configuration and often fails to understand the public API contract.
Good. Can be configured with library entry points (index.ts). ignore patterns can protect public API. 56
Poor. Prone to false positives on library code. 20
Good. Whitelists are the primary mechanism for defining and protecting the public API. 42
Excellent. The lib.rs vs main.rs distinction is a core, idiomatic part of the language and build system. 46
Excellent (by omission). Safely handles libraries by not analyzing their public API for usage. 9
Primary FP Mitigation
Regex ignore patterns in config (ignorePatterns). 7
Plugin ecosystem for frameworks. Configurable ignore patterns. 21
Inline comments (# pylint: disable=...), config file disables. 64
Whitelist files (whitelist.py), --min-confidence flag, ignore patterns. 13
In-code attributes (#[allow(...)]), structural refactoring (lib vs. bin). 46
Rule configuration in UI, marking issues as "False Positive" or "Won't Fix". 62
Configuration Overhead
Moderate. Requires tuning @typescript-eslint and eslint-plugin-import. 22
Low to Moderate. "Zero-config" for many projects, but may need tuning for complex cases. 33
High. Often requires significant configuration to be useful in large projects. 66
Moderate. Requires creating and maintaining whitelists for accuracy. 42
Low. Good defaults, but may require architectural changes for complex workspaces. 46
High initial setup. UI-driven configuration for rules and quality gates. 69
Auto-Fix Capability
Yes, for unused imports via eslint-plugin-unused-imports. 71
Yes, via --fix flag for most issues. 1
No.
No. (Some community discussion but not a core feature). 72
No.
No.


5.2. Trade-offs Analysis

The data in the comparison table highlights several fundamental trade-offs that teams must consider.
Speed vs. Accuracy: There is a direct trade-off between the speed of analysis and its accuracy, particularly concerning cross-module dead code. File-scoped linters like ESLint and Pylint are extremely fast because they can analyze files in parallel without needing global project context. However, this speed comes at the cost of accuracy; they are fundamentally incapable of reliably detecting unused exported symbols.1 Conversely, whole-project graph analyzers like Knip and Vulture are inherently slower, as they must parse the entire codebase and construct a comprehensive dependency graph before they can report any results. This initial overhead is rewarded with a vastly more accurate and actionable report on unused exports.1 For CI/CD pipelines where performance is critical, this trade-off must be managed, but for the specific problem of dead code, accuracy is paramount.
Ease of Configuration vs. Power: The level of configuration required by a tool often correlates with its power and its ability to handle complex or non-standard scenarios. At one end of the spectrum, tools like Knip aim for a "zero-config" experience for common project setups, leveraging an intelligent plugin system to deduce the correct configuration automatically.56 This lowers the barrier to entry but may still require manual tuning for highly customized projects. At the other end, a tool like Pylint is notoriously difficult to configure for large projects, often requiring extensive disabling of noisy rules to become useful.66 Vulture sits in the middle; its power in the dynamic Python environment is unlocked through the deliberate and ongoing effort of maintaining whitelists.42 The key takeaway is that in dynamic and complex ecosystems, achieving high accuracy often necessitates a higher investment in configuration.
Integrated vs. Standalone: The comparison between Rust's compiler-integrated approach and the standalone CLI tools of the JavaScript and Python ecosystems reveals another trade-off. An integrated tool like rustc's dead_code lint benefits from the deepest possible understanding of the language's semantics and is guaranteed to be up-to-date with the latest language features. However, its development is tied to the compiler's release cycle, and its behavior is less flexible. Standalone tools like Knip and Vulture can innovate more rapidly and introduce ecosystem-specific features (like Knip's plugins) that would not make sense in a general-purpose compiler. The cost is that they are separate dependencies to manage and may occasionally lag behind new language features or require specific integration work (like the @typescript-eslint/parser for ESLint).

6. Key Findings and Recommendations for Uveddi

This analysis of industry-standard tools and practices for dead code detection yields several critical findings and a clear set of actionable recommendations for Uveddi. These recommendations are designed to resolve the immediate issue of test failures and establish a robust, sustainable strategy for maintaining code quality across all of Uveddi's technology stacks.

6.1. Key Findings Summary

Insufficiency of Single-File Linters: Standard linters like ESLint and Pylint, while valuable for identifying unused variables and imports within a single file, are fundamentally unsuited for detecting unused exported code. Their file-scoped analysis model inherently leads to high rates of false negatives for this critical category of dead code.
Primacy of Whole-Project Graph Analysis: The industry standard for accurate dead code detection is the use of specialized tools that perform a whole-project analysis. Tools like Knip (for TS/JS) and Vulture (for Python) build a complete dependency graph, starting from known entry points, which is the only reliable static method for identifying unused exports.
Language-Specific Best Practices: The most effective dead code detection strategies are tailored to the specific language. Rust's compiler-integrated dead_code lint is the canonical solution for its ecosystem but requires developers to adhere to idiomatic project structures (e.g., separating library and binary crates) to function correctly.
The Criticality of Context (Application vs. Library): The configuration of any dead code detector is critically dependent on whether it is analyzing a self-contained application or a reusable library. An application has known entry points, whereas a library's public API must be treated as entirely "live" unless configured otherwise.
False Positive Mitigation as a Continuous Process: Managing false positives is not a one-time setup task but an ongoing process of tuning and maintenance. A successful implementation requires a multi-faceted mitigation strategy that combines path-based ignores, in-code suppression comments, and centralized whitelists.
The Necessity of Confidence Scoring in Dynamic Languages: For dynamic languages like Python, where static analysis is inherently uncertain, tools that provide a confidence score (like Vulture) are invaluable. They allow teams to pragmatically balance the desire for aggressive code removal with the need to avoid breaking dynamically invoked features.

6.2. Strategic Recommendation: A Multi-Layered, Language-Specific Approach

Uveddi should move away from any attempt to find a single, one-size-fits-all dead code detection tool. The evidence overwhelmingly indicates that the best practices are language-specific. The recommended strategy is to adopt a multi-layered, best-in-class approach for each technology stack.
This strategy consists of three layers:
Layer 1 (In-File Analysis): Continue to use fast, file-scoped linters (ESLint, Pylint) integrated into the IDE for real-time feedback on simple issues like unused local variables and parameters.
Layer 2 (Project-Wide Analysis): Implement a specialized, graph-based tool for each language stack to perform deep, cross-module analysis in the CI/CD pipeline. This layer is responsible for detecting unused exports and other complex forms of dead code.
Layer 3 (Enforcement and Process): Integrate these checks into the CI/CD pipeline, initially in a "warn-only" mode and later in a strict "error" mode. Establish a clear process for managing configurations and mitigating false positives.

6.3. Tactical Recommendations for Each Language Stack

For JavaScript/TypeScript:
Adopt Knip as the primary tool for project-wide dead code detection. Its powerful plugin system is essential for accurately analyzing modern web applications and monorepos, and its active development ensures it remains the state-of-the-art solution.21
Retain ESLint with @typescript-eslint/no-unused-vars for IDE integration. This provides immediate, real-time feedback to developers on unused local symbols, which is a valuable and complementary function.22
For Python:
Adopt Vulture as the primary tool for dead code detection. Its confidence scoring and whitelist mechanism are indispensable features for managing the false positives inherent in analyzing a dynamic language like Python.13
Establish a project-level pyproject.toml configuration for Vulture, setting a baseline --min-confidence threshold (a starting point of 80 is recommended). Maintain a version-controlled whitelist.py file in each project to handle known false positives.13
Use Pylint for its broad range of other static analysis checks, but defer to Vulture as the authoritative source for dead code analysis.
For Rust:
Leverage the built-in dead_code lint provided by the rustc compiler. No external tool is required or superior for this specific task.
Prioritize correct project architecture. To resolve false positives in workspaces with shared code, refactor the projects to follow the idiomatic structure: extract shared logic into a dedicated library crate, and have the binary crates depend on it. This is an architectural solution to a tooling problem.46

6.4. Addressing the "Test Failures": A Diagnostic and Solution Path

The critical priority of fixing current test failures suggests that a misconfigured or overly aggressive tool is incorrectly flagging live code as dead and failing the CI build.
Hypothesis: The failures are most likely caused by a tool that either lacks project-wide context (like a basic linter flagging a used export) or is not correctly configured to understand the project's entry points or dynamic usage patterns. The rule is configured to error, which halts the CI pipeline.
Diagnostic Steps:
Pinpoint the exact tool and rule that is causing the build to fail in the CI logs.
Analyze the specific code being flagged. Determine its usage pattern: Is it used dynamically? Is it part of a library's public API? Is it called from a framework? Is it only used in tests?
Review the tool's configuration file. Are entry points missing or incorrect? Are test files or other special directories being improperly analyzed?
Recommended Solution Path:
Immediate Mitigation: To unblock development, immediately change the severity of the failing rule in the CI configuration from error to warn. This will allow builds to pass while still reporting the issue.
Parallel Implementation: Begin implementing the recommended tooling (Knip for TS, Vulture for Python) in parallel, following the tactical recommendations above. Run these new tools in a "report-only" mode so they do not affect the build status.
Targeted Fix: Use the output from the new, more accurate tools to diagnose the original issue. Apply the correct false positive mitigation strategy for the specific case (e.g., add the file to Knip's entry array, add the symbol to Vulture's whitelist, or add an inline suppression comment).
Transition: Once the new system is configured, tuned, and trusted, and the original false positive is resolved, you can remove the old or misconfigured check and transition the new, accurate check to error mode in the CI pipeline.

7. Implementation Examples

The following sections provide concrete configuration examples to serve as a starting point for Uveddi's implementation of the recommended tools.

7.1. Recommended knip.config.ts for a TypeScript Project

This example demonstrates a configuration for a typical monorepo containing both a web application and a shared library, showcasing how to define different entry points and ignore patterns.

TypeScript


// knip.config.ts
import type { KnipConfig } from 'knip';

const config: KnipConfig = {
  // Define workspaces for monorepo analysis
  workspaces: {
    'packages/app': {
      // Entry points for a Next.js application, automatically handled by the Next.js plugin
      // If not using a framework plugin, define manually:
      // entry: ['src/pages/index.tsx', 'next.config.js'],
      // Knip's plugins for Next.js, Jest, etc., will be enabled automatically
      // if found in package.json.
    },
    'packages/shared-library': {
      // For a library, the entry point is the main export file.
      entry: ['src/index.ts'],
      // Project files to include in the analysis for this workspace.
      project: ['src/**/*.ts'],
    },
  },
  // Global ignore patterns
  ignore: [
    '**/dist/**', // Ignore all build output
    '**/*.test.ts', // Ignore test files from being entry points
    '**/*.d.ts', // Ignore type definition files
    'src/generated/**', // Ignore auto-generated code
  ],
  // Ignore specific dependencies from being reported as unused
  ignoreDependencies:,
  // Ignore binaries that are used in scripts but not listed as dependencies
  ignoreBinaries: ['docker-compose'],
};

export default config;


References: 34

7.2. Vulture Configuration (pyproject.toml) with Whitelist Example

This example shows how to configure Vulture in a pyproject.toml file and how to create a corresponding whitelist to handle a dynamic attribute access, which is a common source of false positives.

Ini, TOML


# pyproject.toml

[tool.vulture]
# Scan the 'src' and 'tests' directories, plus the whitelist file itself.
paths = ["src", "tests", "vulture_whitelist.py"]
# Set the minimum confidence to 80 to filter out lower-confidence findings.
min_confidence = 80
# Exclude migration files and settings files from analysis.
exclude = ["*/migrations/*.py", "*/settings.py"]
# Sort the report by size to focus on larger chunks of dead code first.
sort_by_size = true
# Ignore names of functions decorated with Flask's @app.route.
ignore_decorators = ["@app.route"]



Python


# vulture_whitelist.py
# This file is used to suppress Vulture false positives.

# Assume we have a class in 'src/my_app/utils.py' that is flagged as unused
# because it's only instantiated via a string name from a config file.
from src.my_app.utils import DynamicHandler

# By simply referencing the class, we mark it as "used" for Vulture.
DynamicHandler

# Assume a function is called via getattr() and is flagged as unused.
from src.my_app.handlers import process_event

# Referencing it marks it as used.
process_event


References: 13

7.3. ESLint Configuration (eslint.config.js) for Optimal Unused Variable Detection

This example uses the modern "flat config" format for ESLint to correctly configure the @typescript-eslint/no-unused-vars rule for in-editor feedback.

JavaScript


// eslint.config.js
import tseslint from 'typescript-eslint';
import js from '@eslint/js';

export default tseslint.config(
  {
    // Global ignores for all configurations
    ignores: ['node_modules/', 'dist/', 'build/'],
  },
  // Base recommended configurations
  js.configs.recommended,
 ...tseslint.configs.recommendedTypeChecked,

  {
    // Configuration specifically for TypeScript files
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      parserOptions: {
        // This enables type-aware linting, which is crucial for accuracy.
        project: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      // === RULE CONFIGURATION FOR UNUSED VARIABLES ===

      // 1. Disable the base ESLint rule, as it can report incorrect errors in TS.
      'no-unused-vars': 'off',

      // 2. Enable the TypeScript-specific rule.
      '@typescript-eslint/no-unused-vars':,
    },
  }
);


References: 22

7.4. CI/CD Integration Snippet (GitHub Actions)

This snippet shows a sample job for a GitHub Actions workflow that runs the recommended tools. It demonstrates how to run them in a report-only mode initially and then switch to a strict, build-failing mode.

YAML


#.github/workflows/ci.yml
name: Lint and Analyze Code

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  analyze:
    name: Run Static Analysis
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm' # or yarn, pnpm

      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install Dependencies
        run: |
          npm install
          pip install -r requirements.txt

      - name: Run Knip (TypeScript Dead Code)
        # For strict enforcement, remove '--no-exit-code'.
        # This will fail the job if unused items are found.
        run: npx knip --no-exit-code --reporter detailed

      - name: Run Vulture (Python Dead Code)
        # Vulture exits with code 3 if dead code is found.
        # To run in report-only mode, add `|| true` at the end.
        run: vulture. --min-confidence 80


References: 35

8. Recommended Next Steps

To successfully integrate this dead code detection strategy, Uveddi should follow a phased, iterative approach designed to minimize disruption and build confidence in the new tooling.
Step 1: Audit and Baseline Current Implementation: The immediate priority is to unblock the development pipeline. Temporarily change the CI configuration for the existing, failing dead code check from an "error" state to a "warning" state. This allows builds to proceed. Concurrently, run the existing tool in a report-only mode across all projects to generate a comprehensive baseline report. This report will be crucial for understanding the full scope of its current false positives.
Step 2: Implement Recommended Tooling in a "Warn-Only" Mode: Introduce Knip and Vulture into the respective TypeScript and Python projects. Configure them to run as a step in the CI pipeline but ensure they do not fail the build. This can be achieved by using flags like Knip's --no-exit-code or by piping Vulture's output to a file and ignoring its exit code (e.g., `vulture. |
| true`). The goal of this phase is to gather data and reports from the new, more accurate tools without impacting developer workflow.
Step 3: Develop Configuration and Whitelists to Mitigate False Positives: Using the reports generated in Step 2, the development teams should collaboratively build out the necessary configuration files (knip.config.ts) and whitelists (vulture_whitelist.py). This should be treated as a standard development task. For projects with a large number of initial findings, adopt a thresholding approach: set a target for the number of issues to resolve in each sprint, gradually reducing the count over time until it reaches zero or a stable, accepted baseline.37
Step 4: Gradually Transition to "Error" Mode in CI: Once the number of false positives has been reduced to near zero and the team is confident in the accuracy of the reports, the CI configuration can be tightened. Remove the "warn-only" flags and allow the tools to run with their default exit codes. Knip will fail the build if it finds any issues. Vulture will fail the build if its --min-confidence threshold is met.13 This transition ensures that no
new dead code is introduced into the codebase.
Step 5: Establish a Long-Term Maintenance and Review Process: Dead code detection is not a one-time fix. Uveddi should formalize the new strategy by documenting the chosen tools, configurations, and the process for handling false positives. The maintenance of configuration files and whitelists should become a regular part of the code review and technical debt grooming processes. When new frameworks, libraries, or architectural patterns are introduced, the dead code detection configuration should be reviewed and updated as part of that work. This establishes a sustainable culture of code hygiene.
Works cited
Unused exports | Knip, accessed July 7, 2025, https://knip.dev/typescript/unused-exports
Exposing dead code: strategies for detection and elimination - vFunction, accessed July 7, 2025, https://vfunction.com/blog/dead-code/
Find dead code in JavaScript/CSS projects - DEV Community, accessed July 7, 2025, https://dev.to/pioug/find-dead-code-in-javascript-css-projects-5f3l
CWE-561: Dead Code (4.17) - Mitre, accessed July 7, 2025, https://cwe.mitre.org/data/definitions/561.html
Dead code detection tool? : r/Python - Reddit, accessed July 7, 2025, https://www.reddit.com/r/Python/comments/1h3ygp/dead_code_detection_tool/
no-unused-private-class-members - ESLint - Pluggable JavaScript Linter, accessed July 7, 2025, https://eslint.org/docs/latest/rules/no-unused-private-class-members
no-unused-vars - ESLint - Pluggable JavaScript linter, accessed July 7, 2025, https://archive.eslint.org/docs/rules/no-unused-vars
Pylint features — Pylint 1.5.4 documentation, accessed July 7, 2025, https://docs.pylint.org/features.html
Rule to detect Unused Public classes or methods - Sonar Community, accessed July 7, 2025, https://community.sonarsource.com/t/rule-to-detect-unused-public-classes-or-methods/7598
Dead Code Detection - hello2morrow – Empowering Software Craftsmanship, accessed July 7, 2025, https://blog.hello2morrow.com/2015/04/dead-code-detection/
Static Code Analysis - Oligo Security, accessed July 7, 2025, https://www.oligo.security/academy/static-code-analysis
Static Analysis: An Introduction - ACM Queue, accessed July 7, 2025, https://queue.acm.org/detail.cfm?id=3487021
jendrikseipp/vulture: Find dead Python code - GitHub, accessed July 7, 2025, https://github.com/jendrikseipp/vulture
Automating dead code cleanup - Engineering at Meta, accessed July 7, 2025, https://engineering.fb.com/2023/10/24/data-infrastructure/automating-dead-code-cleanup/
FAQ | Knip, accessed July 7, 2025, https://knip.dev/reference/faq
How can you find unused functions in Python code? - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/693070/how-can-you-find-unused-functions-in-python-code
How can I know which parts in the code are never used? - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/4813947/how-can-i-know-which-parts-in-the-code-are-never-used
Dead code detection in legacy C/C++ project [closed] - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/229069/dead-code-detection-in-legacy-c-c-project
[no-unused-modules] Feature Request: Option to ignore exports with usage inside the module · Issue #1728 · import-js/eslint-plugin-import - GitHub, accessed July 7, 2025, https://github.com/benmosher/eslint-plugin-import/issues/1728
Pylint says my function is unused, but I import it in a different module - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/48511355/pylint-says-my-function-is-unused-but-i-import-it-in-a-different-module
Knip: Declutter your JavaScript & TypeScript projects, accessed July 7, 2025, https://knip.dev/
no-unused-vars - typescript-eslint, accessed July 7, 2025, https://typescript-eslint.io/rules/no-unused-vars/
Rule no-unused-vars - ESLint - Pluggable JavaScript linter, accessed July 7, 2025, https://archive.eslint.org/docs/2.0.0/rules/no-unused-vars
no-unused-vars - ESLint - Pluggable JavaScript Linter, accessed July 7, 2025, https://eslint.org/docs/latest/rules/no-unused-vars
docs: [no-unused-vars] document ignoring `_` prefixed variables · Issue #8464 - GitHub, accessed July 7, 2025, https://github.com/typescript-eslint/typescript-eslint/issues/8464
ESLint no-unused-vars: _ ignore prefix | johnnyreilly - John Reilly, accessed July 7, 2025, https://johnnyreilly.com/typescript-eslint-no-unused-vars
Finding dead code (and dead types) in TypeScript, accessed July 7, 2025, https://effectivetypescript.com/2020/10/20/tsprune/
eslint-plugin-import - NPM, accessed July 7, 2025, https://www.npmjs.com/package/eslint-plugin-import
client/node_modules/eslint-plugin-import/docs/rules/no-unused-modules.md - GitLab, accessed July 7, 2025, https://caoslab.psy.cmu.edu:32443/developers/jsexperimentslayout/-/blob/homepage_full/client/node_modules/eslint-plugin-import/docs/rules/no-unused-modules.md
eslint-plugin-import-x/docs/rules/no-unused-modules.md at master - GitHub, accessed July 7, 2025, https://github.com/un-ts/eslint-plugin-import-x/blob/master/docs/rules/no-unused-modules.md
ESLint - Configuring "no-unused-vars" for TypeScript - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/57802057/eslint-configuring-no-unused-vars-for-typescript
Clean Up Your TypeScript Classes: Finding and Removing Unused Static Members with ESLint | by Mohamed Said Ibrahim | Medium, accessed July 7, 2025, https://medium.com/@mohamedsaidibrahim/clean-up-your-typescript-classes-finding-and-removing-unused-static-members-with-eslint-8c1a8c74f3e5
Why use Knip?, accessed July 7, 2025, https://knip.dev/explanations/why-use-knip
Writing A Plugin - Knip, accessed July 7, 2025, https://knip.dev/guides/writing-a-plugin
How to Find and remove unused code with Knip by sergiodxa, accessed July 7, 2025, https://sergiodxa.com/tutorials/find-and-remove-unused-code-with-knip
Tool to detect unused class methods? - typescript - Reddit, accessed July 7, 2025, https://www.reddit.com/r/typescript/comments/etvbdz/tool_to_detect_unused_class_methods/
Seek and destroy dead code for good: a strategy using ts-prune - Theodo Apps, accessed July 7, 2025, https://www.bam.tech/article/seek-and-destroy-dead-code-for-good-a-strategy-using-ts-prune
ts-prune | Compare Similar npm Packages, accessed July 7, 2025, https://npm-compare.com/ts-prune
nadeesha/ts-prune: Find unused exports in a typescript ... - GitHub, accessed July 7, 2025, https://github.com/nadeesha/ts-prune
unused-import / W0611 - Pylint 4.0.0-dev0 documentation, accessed July 7, 2025, https://pylint.readthedocs.io/en/v3.3.4/user_guide/messages/warning/unused-import.html
Pylint incorrectly flags as `unused-import` things that are used as variable type annotations · Issue #1063 - GitHub, accessed July 7, 2025, https://github.com/PyCQA/pylint/issues/1063
Vulture - PyPI, accessed July 7, 2025, https://pypi.org/project/vulture/
use vulture to automatically remove dead code? · Issue #25 - GitHub, accessed July 7, 2025, https://github.com/jendrikseipp/vulture/issues/25
README.md - jendrikseipp/vulture - GitHub, accessed July 7, 2025, https://github.com/jendrikseipp/vulture/blob/main/README.md
scripts/vulture/find-dead-code.sh · renovate/npm-cli-vulnerability · Hsin-Yu Chien / edx-platform-release · GitLab, accessed July 7, 2025, https://code.vt.edu/hsinyu/edx-platform-release/-/blob/renovate/npm-cli-vulnerability/scripts/vulture/find-dead-code.sh
Dead code warning with multiple binaries? - rust - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/66195852/dead-code-warning-with-multiple-binaries
How do you disable dead code warnings at the crate level in Rust? - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/25877285/how-do-you-disable-dead-code-warnings-at-the-crate-level-in-rust
Allowed-by-default Lints - The rustc book - Rust Documentation, accessed July 7, 2025, https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#dead-code
Clippy Lints - GitHub Pages, accessed July 7, 2025, https://rust-lang.github.io/rust-clippy/master/index.html
How can I silence rustc's warning that a function is never used? : r/learnrust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/learnrust/comments/tfcf67/how_can_i_silence_rustcs_warning_that_a_function/
Configuring Clippy - Rust Documentation, accessed July 7, 2025, https://doc.rust-lang.org/clippy/configuration.html
Detect Dead Code and Calls to Deprecated Methods with Sonar Squid, accessed July 7, 2025, https://www.sonarsource.com/blog/detect-dead-code-and-calls-to-deprecated-methods-with-sonar-squid/
Does SonarQube scan 3rd-party code? - Sonar Community, accessed July 7, 2025, https://community.sonarsource.com/t/does-sonarqube-scan-3rd-party-code/84132
scripts/vulture/find-dead-code.sh · jenkins/upgrade-python-requirements-90f4f65 · Hsin-Yu Chien / edx-platform-release · GitLab, accessed July 7, 2025, https://code.vt.edu/hsinyu/edx-platform-release/-/blob/jenkins/upgrade-python-requirements-90f4f65/scripts/vulture/find-dead-code.sh?ref_type=heads
Getting Started with ESLint - ESLint - Pluggable JavaScript Linter, accessed July 7, 2025, https://eslint.org/docs/latest/use/getting-started
Configuration | Knip, accessed July 7, 2025, https://knip.dev/overview/configuration
Reduce unused exports · Issue #164941 · microsoft/vscode - GitHub, accessed July 7, 2025, https://github.com/microsoft/vscode/issues/164941
Configuration | Knip, accessed July 7, 2025, https://knip.dev/reference/configuration
ts-prune - NPM, accessed July 7, 2025, https://www.npmjs.com/package/ts-prune
find dead JavaScript code? - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/4136738/find-dead-javascript-code
yuthon/ts-prune-ex: Find unused exports in a typescript project. - GitHub, accessed July 7, 2025, https://github.com/yuthon/ts-prune-ex
SonarQube marks function refered by MethodSource as Unused "private" method per rule 1144 - Sonar Community, accessed July 7, 2025, https://community.sonarsource.com/t/sonarqube-marks-function-refered-by-methodsource-as-unused-private-method-per-rule-1144/81572
How to find dead code in a large react project? [closed] - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/54148788/how-to-find-dead-code-in-a-large-react-project
Pylint configuration - Codeac, accessed July 7, 2025, https://www.codeac.io/documentation/pylint-configuration.html
How to reduce False Positive Alerts in Threat Detection: Sharpening Security Detections | by Tahir | Medium, accessed July 7, 2025, https://medium.com/@tahirbalarabe2/how-to-reduce-false-positive-alerts-in-threat-detection-sharpening-security-detections-d8382b93915a
pylint-dev/pylint: It's not just a linter that annoys you! - GitHub, accessed July 7, 2025, https://github.com/pylint-dev/pylint
How do I disable pylint unused import error messages in vs code - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/52123470/how-do-i-disable-pylint-unused-import-error-messages-in-vs-code
Does sonarqube analyze for unused code? - #11 by anon67236913 - Sonar Community, accessed July 7, 2025, https://community.sonarsource.com/t/does-sonarqube-analyze-for-unused-code/4741/11
Code Analysis with SonarQube | Baeldung, accessed July 7, 2025, https://www.baeldung.com/sonar-qube
Static Code Analysis Using SonarQube : A Step-by-Step Guide | Sonar, accessed July 7, 2025, https://www.sonarsource.com/learn/static-code-analysis-using-sonarqube/
eslint-plugin-unused-imports - NPM, accessed July 7, 2025, https://www.npmjs.com/package/eslint-plugin-unused-imports
I Made a Python Tool to Detect Unused Code in Your Projects - Reddit, accessed July 7, 2025, https://www.reddit.com/r/Python/comments/1iwbnlh/i_made_a_python_tool_to_detect_unused_code_in/
knip.config.ts - code-pushup/cli - GitHub, accessed July 7, 2025, https://github.com/code-pushup/cli/blob/main/knip.config.ts
Using Knip in CI, accessed July 7, 2025, https://knip.dev/guides/using-knip-in-ci
