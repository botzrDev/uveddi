//! Universal Anti-Pattern Knowledge Database
//!
//! This module contains the comprehensive universal anti-pattern knowledge base
//! extracted from Uveddi's existing detectors and enhanced with AI-optimized content.
//! The knowledge is structured for optimal AI consumption and compression efficiency.

use crate::ai::knowledge::schema::*;
use crate::ai::knowledge::compression::CompressedString;
use std::collections::HashMap;

/// Create the complete universal anti-pattern knowledge base
pub fn create_universal_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // Object-Oriented Anti-patterns
    patterns.insert("god_object".to_string(), create_god_object_pattern());
    patterns.insert("large_class".to_string(), create_large_class_pattern());
    patterns.insert("feature_envy".to_string(), create_feature_envy_pattern());

    // Architectural Anti-patterns
    patterns.insert("tight_coupling".to_string(), create_tight_coupling_pattern());
    patterns.insert("cyclic_dependencies".to_string(), create_cyclic_dependencies_pattern());
    patterns.insert("inappropriate_intimacy".to_string(), create_inappropriate_intimacy_pattern());

    // Performance Anti-patterns
    patterns.insert("premature_optimization".to_string(), create_premature_optimization_pattern());
    patterns.insert("resource_leak".to_string(), create_resource_leak_pattern());
    patterns.insert("inefficient_algorithms".to_string(), create_inefficient_algorithms_pattern());

    // Error Handling Anti-patterns
    patterns.insert("silent_failure".to_string(), create_silent_failure_pattern());
    patterns.insert("inappropriate_exception_type".to_string(), create_inappropriate_exception_pattern());
    patterns.insert("error_information_loss".to_string(), create_error_information_loss_pattern());

    // Maintainability Anti-patterns
    patterns.insert("dead_code".to_string(), create_dead_code_pattern());
    patterns.insert("code_duplication".to_string(), create_code_duplication_pattern());
    patterns.insert("magic_values".to_string(), create_magic_values_pattern());
    patterns.insert("inconsistent_naming".to_string(), create_inconsistent_naming_pattern());

    patterns
}

/// Create comprehensive God Object pattern knowledge
fn create_god_object_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "god_object".to_string(),
        name: "God Object".to_string(),
        definition: CompressedString::new(
            "A class that knows too much or does too much. It centralizes too many \
             responsibilities and becomes a bottleneck for maintenance and testing. \
             Also known as 'Blob' or 'Swiss Army Knife' anti-pattern."
        ),
        symptoms: vec![
            CompressedString::new("Class has excessive number of methods (>20-30)"),
            CompressedString::new("Class has excessive lines of code (>500-1000)"),
            CompressedString::new("Class has too many instance variables (>10-15)"),
            CompressedString::new("Class name is vague or overly generic (Manager, Handler, Util)"),
            CompressedString::new("Class is difficult to test in isolation"),
            CompressedString::new("Multiple unrelated responsibilities in single class"),
            CompressedString::new("High coupling with many other classes"),
            CompressedString::new("Low cohesion - methods don't work together"),
            CompressedString::new("Frequent changes to the class for unrelated reasons"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![
            DetectionMethod::MetricThreshold {
                metric_name: "method_count".to_string(),
                threshold: 25.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "line_count".to_string(),
                threshold: 500.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "field_count".to_string(),
                threshold: 12.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "cohesion_score".to_string(),
                threshold: 0.3,
                operator: ComparisonOperator::LessThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "coupling_score".to_string(),
                threshold: 0.7,
                operator: ComparisonOperator::GreaterThan,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "extract_class".to_string(),
                title: "Extract Class".to_string(),
                implementation: CompressedString::new(
                    "Identify cohesive groups of methods and data, extract them into separate classes. \
                     This is the most common and effective solution for God Objects."
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Universal,
                        problem_code: CompressedString::new(
                            "class UserManager {\n\
                             // User data management\n\
                             void createUser() { ... }\n\
                             void deleteUser() { ... }\n\
                             \n\
                             // Email functionality\n\
                             void sendEmail() { ... }\n\
                             void validateEmail() { ... }\n\
                             \n\
                             // Database operations\n\
                             void connectToDatabase() { ... }\n\
                             void executeQuery() { ... }\n\
                             \n\
                             // Logging\n\
                             void logActivity() { ... }\n\
                             void generateReport() { ... }\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "class UserService {\n\
                             void createUser() { ... }\n\
                             void deleteUser() { ... }\n\
                             }\n\
                             \n\
                             class EmailService {\n\
                             void sendEmail() { ... }\n\
                             void validateEmail() { ... }\n\
                             }\n\
                             \n\
                             class DatabaseService {\n\
                             void connectToDatabase() { ... }\n\
                             void executeQuery() { ... }\n\
                             }\n\
                             \n\
                             class LoggingService {\n\
                             void logActivity() { ... }\n\
                             void generateReport() { ... }\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Split the God Object into focused, single-responsibility classes. \
                             Each class now has a clear purpose and is easier to maintain, test, and reuse."
                        ),
                        file_context: Some("user_management.py".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec![
                    "Understanding of Single Responsibility Principle".to_string(),
                    "Comprehensive test coverage before refactoring".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            },
            SolutionPattern {
                id: "facade_pattern".to_string(),
                title: "Facade Pattern".to_string(),
                implementation: CompressedString::new(
                    "Create a simplified interface that delegates to extracted classes while \
                     maintaining backward compatibility for existing clients."
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Universal,
                        problem_code: CompressedString::new("// Same God Object as above"),
                        solution_code: CompressedString::new(
                            "class UserManagerFacade {\n\
                             private UserService userService;\n\
                             private EmailService emailService;\n\
                             private DatabaseService dbService;\n\
                             private LoggingService loggingService;\n\
                             \n\
                             // Delegate methods to appropriate services\n\
                             void createUser() { userService.createUser(); }\n\
                             void sendEmail() { emailService.sendEmail(); }\n\
                             // ... other delegations\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "The facade maintains the original interface while internally using \
                             the extracted services. This allows gradual migration of clients."
                        ),
                        file_context: Some("user_management_facade.py".to_string()),
                    }
                ],
                effort_level: EffortLevel::High,
                prerequisites: vec![
                    "Extract Class refactoring completed first".to_string(),
                    "Understanding of Facade pattern".to_string(),
                ],
                expected_impact: ImpactLevel::Medium,
            },
        ],
        examples: CodeExamples {
            primary: vec![
                CodeExample {
                    language: SourceLanguage::Universal,
                    problem_code: CompressedString::new(
                        "class UserManager {\n\
                         // 50+ methods mixing user management, email, database, logging, etc.\n\
                         // 15+ fields for various unrelated purposes\n\
                         // Difficult to test, understand, or modify\n\
                         }"
                    ),
                    solution_code: CompressedString::new(
                        "// Split into focused classes:\n\
                         class UserService { /* user operations */ }\n\
                         class EmailService { /* email operations */ }\n\
                         class DatabaseService { /* database operations */ }\n\
                         class LoggingService { /* logging operations */ }"
                    ),
                    explanation: CompressedString::new(
                        "Transform a single large class into multiple focused classes, \
                         each with a single responsibility."
                    ),
                    file_context: Some("user_system.py".to_string()),
                }
            ],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "large_class".to_string(),
            "feature_envy".to_string(),
            "tight_coupling".to_string(),
            "low_cohesion".to_string(),
        ],
        tags: vec![
            "object-oriented".to_string(),
            "design".to_string(),
            "maintainability".to_string(),
            "complexity".to_string(),
            "single-responsibility".to_string(),
        ],
        frequency_score: 0.8, // Very common anti-pattern
        detection_confidence: 0.85,
    }
}

/// Create comprehensive Large Class pattern knowledge
fn create_large_class_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "large_class".to_string(),
        name: "Large Class".to_string(),
        definition: CompressedString::new(
            "A class that has grown too large due to accumulating too many lines of code, \
             methods, or fields. While similar to God Object, this focuses specifically on size metrics."
        ),
        symptoms: vec![
            CompressedString::new("Excessive lines of code (>500-1000 LOC)"),
            CompressedString::new("Too many methods (>20-30 methods)"),
            CompressedString::new("Too many instance variables (>10-15 fields)"),
            CompressedString::new("Difficult to understand the class's overall purpose"),
            CompressedString::new("Long compilation or parsing times"),
            CompressedString::new("Difficult to navigate within the class"),
        ],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![
            DetectionMethod::MetricThreshold {
                metric_name: "logical_lines_of_code".to_string(),
                threshold: 500.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "method_count".to_string(),
                threshold: 20.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "field_count".to_string(),
                threshold: 10.0,
                operator: ComparisonOperator::GreaterThan,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "extract_class_by_size".to_string(),
                title: "Extract Class (Size-based)".to_string(),
                implementation: CompressedString::new(
                    "Break the large class into smaller, more focused classes based on logical groupings."
                ),
                examples: vec![],
                effort_level: EffortLevel::Medium,
                prerequisites: vec!["Test coverage".to_string()],
                expected_impact: ImpactLevel::Medium,
            },
            SolutionPattern {
                id: "extract_method".to_string(),
                title: "Extract Method".to_string(),
                implementation: CompressedString::new(
                    "Break down large methods into smaller, more focused methods."
                ),
                examples: vec![],
                effort_level: EffortLevel::Low,
                prerequisites: vec![],
                expected_impact: ImpactLevel::Low,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "god_object".to_string(),
            "long_method".to_string(),
        ],
        tags: vec![
            "size".to_string(),
            "complexity".to_string(),
            "maintainability".to_string(),
        ],
        frequency_score: 0.7,
        detection_confidence: 0.9, // High confidence - size metrics are objective
    }
}

/// Create comprehensive Tight Coupling pattern knowledge
fn create_tight_coupling_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "tight_coupling".to_string(),
        name: "Tight Coupling".to_string(),
        definition: CompressedString::new(
            "Components that are overly dependent on each other's internal implementation details, \
             making the system difficult to modify, test, and understand."
        ),
        symptoms: vec![
            CompressedString::new("High fan-out (component depends on many others)"),
            CompressedString::new("High fan-in (many components depend on this one)"),
            CompressedString::new("Changes to one component require changes to many others"),
            CompressedString::new("Difficult to test components in isolation"),
            CompressedString::new("Direct access to internal data structures"),
            CompressedString::new("Long parameter lists between components"),
            CompressedString::new("Circular dependencies between components"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::Architectural,
        detection_methods: vec![
            DetectionMethod::MetricThreshold {
                metric_name: "fan_out".to_string(),
                threshold: 7.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "coupling_between_objects".to_string(),
                threshold: 6.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::DependencyAnalysis {
                graph_pattern: "high_coupling".to_string(),
                relationship_type: "depends_on".to_string(),
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "dependency_injection".to_string(),
                title: "Dependency Injection".to_string(),
                implementation: CompressedString::new(
                    "Use dependency injection to provide dependencies from the outside rather than creating them internally."
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Universal,
                        problem_code: CompressedString::new(
                            "class OrderService {\n\
                             EmailService emailService = new EmailService();\n\
                             DatabaseService dbService = new DatabaseService();\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "class OrderService {\n\
                             private EmailService emailService;\n\
                             private DatabaseService dbService;\n\
                             \n\
                             // Inject dependencies via constructor\n\
                             OrderService(EmailService email, DatabaseService db) {\n\
                             this.emailService = email;\n\
                             this.dbService = db;\n\
                             }\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Dependencies are injected rather than created, making the class \
                             more testable and flexible."
                        ),
                        file_context: Some("order_service.java".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec!["Understanding of dependency injection pattern".to_string()],
                expected_impact: ImpactLevel::High,
            },
            SolutionPattern {
                id: "interface_abstraction".to_string(),
                title: "Interface Abstraction".to_string(),
                implementation: CompressedString::new(
                    "Depend on interfaces/abstractions rather than concrete implementations."
                ),
                examples: vec![],
                effort_level: EffortLevel::Medium,
                prerequisites: vec!["Interface design skills".to_string()],
                expected_impact: ImpactLevel::High,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "cyclic_dependencies".to_string(),
            "god_object".to_string(),
            "inappropriate_intimacy".to_string(),
        ],
        tags: vec![
            "coupling".to_string(),
            "dependencies".to_string(),
            "architecture".to_string(),
            "testability".to_string(),
        ],
        frequency_score: 0.75,
        detection_confidence: 0.8,
    }
}

/// Create comprehensive Dead Code pattern knowledge
fn create_dead_code_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "dead_code".to_string(),
        name: "Dead Code".to_string(),
        definition: CompressedString::new(
            "Code that is defined but never used, including unused functions, variables, \
             classes, and modules. Dead code clutters the codebase and can mislead developers."
        ),
        symptoms: vec![
            CompressedString::new("Functions or methods that are never called"),
            CompressedString::new("Variables that are declared but never read"),
            CompressedString::new("Classes that are never instantiated"),
            CompressedString::new("Imports that are never used"),
            CompressedString::new("Constants that are never referenced"),
            CompressedString::new("Entire modules with no external references"),
        ],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::Maintainability,
        detection_methods: vec![
            DetectionMethod::StaticAnalysis {
                pattern: "unused_symbol".to_string(),
                confidence: 0.9,
            },
            DetectionMethod::AstPattern {
                query: "unreferenced_definition".to_string(),
                node_types: vec!["function_item".to_string(), "struct_item".to_string()],
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "remove_unused_code".to_string(),
                title: "Remove Unused Code".to_string(),
                implementation: CompressedString::new(
                    "Safely remove code that is confirmed to be unused after thorough analysis."
                ),
                examples: vec![],
                effort_level: EffortLevel::Low,
                prerequisites: vec![
                    "Static analysis to confirm code is unused".to_string(),
                    "Version control for safe removal".to_string(),
                ],
                expected_impact: ImpactLevel::Medium,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "speculative_generality".to_string(),
            "lazy_class".to_string(),
        ],
        tags: vec![
            "unused".to_string(),
            "cleanup".to_string(),
            "maintainability".to_string(),
        ],
        frequency_score: 0.6,
        detection_confidence: 0.75, // Can have false positives with dynamic code
    }
}

/// Create comprehensive Code Duplication pattern knowledge
fn create_code_duplication_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "code_duplication".to_string(),
        name: "Code Duplication".to_string(),
        definition: CompressedString::new(
            "Identical or very similar code fragments that appear in multiple locations. \
             Code duplication violates the DRY (Don't Repeat Yourself) principle and increases maintenance burden."
        ),
        symptoms: vec![
            CompressedString::new("Identical code blocks in different locations"),
            CompressedString::new("Similar code with minor variations (renamed clones)"),
            CompressedString::new("Copy-paste programming patterns"),
            CompressedString::new("Parallel bug fixes needed in multiple places"),
            CompressedString::new("Inconsistent behavior in supposedly identical code"),
        ],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::Maintainability,
        detection_methods: vec![
            DetectionMethod::StaticAnalysis {
                pattern: "duplicate_block".to_string(),
                confidence: 0.8,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "extract_common_function".to_string(),
                title: "Extract Common Function".to_string(),
                implementation: CompressedString::new(
                    "Extract duplicated code into a shared function or method."
                ),
                examples: vec![],
                effort_level: EffortLevel::Low,
                prerequisites: vec![],
                expected_impact: ImpactLevel::Medium,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "copy_paste_programming".to_string(),
        ],
        tags: vec![
            "duplication".to_string(),
            "dry".to_string(),
            "maintainability".to_string(),
        ],
        frequency_score: 0.7,
        detection_confidence: 0.85,
    }
}

// Placeholder implementations for remaining patterns
fn create_feature_envy_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "feature_envy".to_string(),
        name: "Feature Envy".to_string(),
        definition: CompressedString::new("A method that uses more features of another class than its own."),
        symptoms: vec![],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec!["god_object".to_string()],
        tags: vec!["coupling".to_string()],
        frequency_score: 0.5,
        detection_confidence: 0.6,
    }
}

fn create_cyclic_dependencies_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "cyclic_dependencies".to_string(),
        name: "Cyclic Dependencies".to_string(),
        definition: CompressedString::new("Circular dependencies between modules or components."),
        symptoms: vec![],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::Architectural,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec!["tight_coupling".to_string()],
        tags: vec!["dependencies".to_string(), "architecture".to_string()],
        frequency_score: 0.4,
        detection_confidence: 0.9,
    }
}

fn create_inappropriate_intimacy_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "inappropriate_intimacy".to_string(),
        name: "Inappropriate Intimacy".to_string(),
        definition: CompressedString::new("Classes that know too much about each other's private details."),
        symptoms: vec![],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec!["tight_coupling".to_string()],
        tags: vec!["encapsulation".to_string()],
        frequency_score: 0.3,
        detection_confidence: 0.7,
    }
}

fn create_premature_optimization_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "premature_optimization".to_string(),
        name: "Premature Optimization".to_string(),
        definition: CompressedString::new("Optimizing code before identifying actual performance bottlenecks."),
        symptoms: vec![],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::Performance,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["performance".to_string(), "optimization".to_string()],
        frequency_score: 0.4,
        detection_confidence: 0.5,
    }
}

fn create_resource_leak_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "resource_leak".to_string(),
        name: "Resource Leak".to_string(),
        definition: CompressedString::new("Failure to properly release resources like memory, file handles, or network connections."),
        symptoms: vec![],
        impact: ImpactLevel::Critical,
        category: AntiPatternCategory::Resources,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["resources".to_string(), "memory".to_string()],
        frequency_score: 0.3,
        detection_confidence: 0.8,
    }
}

fn create_inefficient_algorithms_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "inefficient_algorithms".to_string(),
        name: "Inefficient Algorithms".to_string(),
        definition: CompressedString::new("Using algorithms with poor time or space complexity when better alternatives exist."),
        symptoms: vec![],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::Performance,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["performance".to_string(), "algorithms".to_string()],
        frequency_score: 0.3,
        detection_confidence: 0.6,
    }
}

fn create_silent_failure_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "silent_failure".to_string(),
        name: "Silent Failure".to_string(),
        definition: CompressedString::new("Errors that are caught but not properly handled or reported."),
        symptoms: vec![],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ErrorHandling,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["error-handling".to_string(), "debugging".to_string()],
        frequency_score: 0.5,
        detection_confidence: 0.7,
    }
}

fn create_inappropriate_exception_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "inappropriate_exception_type".to_string(),
        name: "Inappropriate Exception Type".to_string(),
        definition: CompressedString::new("Using generic or inappropriate exception types instead of specific, meaningful ones."),
        symptoms: vec![],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::ErrorHandling,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["error-handling".to_string(), "exceptions".to_string()],
        frequency_score: 0.4,
        detection_confidence: 0.6,
    }
}

fn create_error_information_loss_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "error_information_loss".to_string(),
        name: "Error Information Loss".to_string(),
        definition: CompressedString::new("Loss of important error context when handling or re-throwing exceptions."),
        symptoms: vec![],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::ErrorHandling,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["error-handling".to_string(), "debugging".to_string()],
        frequency_score: 0.3,
        detection_confidence: 0.5,
    }
}

fn create_magic_values_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "magic_values".to_string(),
        name: "Magic Values".to_string(),
        definition: CompressedString::new("Unexplained numeric or string literals scattered throughout the code."),
        symptoms: vec![],
        impact: ImpactLevel::Low,
        category: AntiPatternCategory::Maintainability,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["literals".to_string(), "constants".to_string()],
        frequency_score: 0.6,
        detection_confidence: 0.8,
    }
}

fn create_inconsistent_naming_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "inconsistent_naming".to_string(),
        name: "Inconsistent Naming".to_string(),
        definition: CompressedString::new("Inconsistent naming conventions that make code harder to understand and maintain."),
        symptoms: vec![],
        impact: ImpactLevel::Low,
        category: AntiPatternCategory::Maintainability,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples { primary: vec![], variations: HashMap::new() },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["naming".to_string(), "conventions".to_string()],
        frequency_score: 0.5,
        detection_confidence: 0.6,
    }
}