//! Language-Specific Anti-Pattern Knowledge Library
//!
//! This module extends the universal patterns with language-specific variations,
//! idioms, detection methods, and solutions tailored to each programming language.
//! It provides contextually rich, language-aware knowledge for precise AI-powered analysis.

use crate::ai::knowledge::compression::CompressedString;
use crate::ai::knowledge::schema::*;
use std::collections::HashMap;

/// Create comprehensive language-specific pattern libraries
pub fn create_language_specific_libraries() -> HashMap<SourceLanguage, LanguageKnowledge> {
    let mut libraries = HashMap::new();

    libraries.insert(SourceLanguage::Rust, create_rust_knowledge());
    libraries.insert(SourceLanguage::Python, create_python_knowledge());
    libraries.insert(SourceLanguage::JavaScript, create_javascript_knowledge());
    libraries.insert(SourceLanguage::TypeScript, create_typescript_knowledge());
    libraries.insert(SourceLanguage::Java, create_java_knowledge());

    libraries
}

/// Create comprehensive Rust-specific knowledge
fn create_rust_knowledge() -> LanguageKnowledge {
    LanguageKnowledge {
        language: SourceLanguage::Rust,
        patterns: create_rust_patterns(),
        idioms: create_rust_idioms(),
        frameworks: create_rust_frameworks(),
        stdlib_patterns: create_rust_stdlib_patterns(),
    }
}

/// Rust-specific anti-pattern variations
fn create_rust_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in Rust context
    patterns.insert("god_object".to_string(), PatternKnowledge {
        id: "rust_god_object".to_string(),
        name: "God Object (Rust)".to_string(),
        definition: CompressedString::new(
            "A struct with too many fields, methods, or responsibilities that violates Rust's \
             ownership principles and makes code difficult to borrow-check and compose."
        ),
        symptoms: vec![
            CompressedString::new("Struct with 15+ fields (Rust prefers composition)"),
            CompressedString::new("impl block with 20+ methods"),
            CompressedString::new("Multiple unrelated trait implementations on single type"),
            CompressedString::new("Struct requires many lifetime parameters"),
            CompressedString::new("Difficult to derive common traits (Clone, Debug, etc.)"),
            CompressedString::new("Borrow checker conflicts when using the struct"),
            CompressedString::new("Large enum with many variants doing different things"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![
            DetectionMethod::MetricThreshold {
                metric_name: "field_count".to_string(),
                threshold: 12.0, // Lower for Rust due to ownership
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "method_count".to_string(),
                threshold: 20.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::MetricThreshold {
                metric_name: "trait_impl_count".to_string(),
                threshold: 8.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::StaticAnalysis {
                pattern: "lifetime_param_count".to_string(),
                confidence: 0.8,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "rust_composition_traits".to_string(),
                title: "Composition with Traits".to_string(),
                implementation: CompressedString::new(
                    "Use trait objects and composition instead of large structs. \
                     Leverage Rust's zero-cost abstractions and type system."
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Rust,
                        problem_code: CompressedString::new(
                            "// God struct - hard to use and compose\n\
                             struct UserManager {\n\
                                 db: Database,\n\
                                 email: EmailService,\n\
                                 auth: AuthService,\n\
                                 logger: Logger,\n\
                                 cache: CacheService,\n\
                                 metrics: MetricsCollector,\n\
                                 // ... many more fields\n\
                             }\n\
                             \n\
                             impl UserManager {\n\
                                 // 25+ methods mixing different responsibilities\n\
                                 fn create_user(&mut self) { /* ... */ }\n\
                                 fn send_email(&self) { /* ... */ }\n\
                                 fn authenticate(&self) { /* ... */ }\n\
                                 // ... many more methods\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "// Define focused traits for different responsibilities\n\
                             trait UserService {\n\
                                 fn create_user(&mut self, user: User) -> Result<(), UserError>;\n\
                                 fn delete_user(&mut self, id: UserId) -> Result<(), UserError>;\n\
                             }\n\
                             \n\
                             trait NotificationService {\n\
                                 fn send_email(&self, recipient: &str, message: &str) -> Result<(), NotificationError>;\n\
                             }\n\
                             \n\
                             // Small, focused implementations\n\
                             struct DatabaseUserService {\n\
                                 db: Database,\n\
                             }\n\
                             \n\
                             impl UserService for DatabaseUserService {\n\
                                 fn create_user(&mut self, user: User) -> Result<(), UserError> {\n\
                                     self.db.insert_user(user)\n\
                                 }\n\
                             }\n\
                             \n\
                             // Compose using trait objects for flexibility\n\
                             struct UserManager {\n\
                                 user_service: Box<dyn UserService>,\n\
                                 notification: Box<dyn NotificationService>,\n\
                             }\n\
                             \n\
                             // Or use enum dispatch for performance\n\
                             enum UserServiceType {\n\
                                 Database(DatabaseUserService),\n\
                                 Memory(MemoryUserService),\n\
                             }\n\
                             \n\
                             impl UserService for UserServiceType {\n\
                                 fn create_user(&mut self, user: User) -> Result<(), UserError> {\n\
                                     match self {\n\
                                         UserServiceType::Database(svc) => svc.create_user(user),\n\
                                         UserServiceType::Memory(svc) => svc.create_user(user),\n\
                                     }\n\
                                 }\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Replace the god struct with trait-based composition. This approach:\n\
                             • Improves ownership semantics and borrow-checking\n\
                             • Enables easy testing through trait mocking\n\
                             • Provides flexible composition options\n\
                             • Leverages Rust's zero-cost abstractions\n\
                             • Allows enum dispatch for performance-critical code"
                        ),
                        file_context: Some("user_management.rs".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec![
                    "Understanding of Rust traits and ownership".to_string(),
                    "Comprehensive test coverage before refactoring".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            },
        ],
        examples: CodeExamples {
            primary: vec![
                CodeExample {
                    language: SourceLanguage::Rust,
                    problem_code: CompressedString::new(
                        "struct ConfigManager<'a> {\n\
                         // Too many responsibilities and lifetimes\n\
                         db_config: &'a DatabaseConfig,\n\
                         server_config: &'a ServerConfig,\n\
                         cache_config: &'a CacheConfig,\n\
                         // ... 15+ fields\n\
                         }"
                    ),
                    solution_code: CompressedString::new(
                        "// Split into focused structs:\n\
                         struct DatabaseManager { config: DatabaseConfig }\n\
                         struct ServerManager { config: ServerConfig }\n\
                         struct CacheManager { config: CacheConfig }\n\
                         \n\
                         // Compose through dependency injection\n\
                         struct AppManager {\n\
                             database: DatabaseManager,\n\
                             server: ServerManager,\n\
                             cache: CacheManager,\n\
                         }"
                    ),
                    explanation: CompressedString::new(
                        "Transform monolithic struct into composable, ownership-friendly components"
                    ),
                    file_context: Some("config_management.rs".to_string()),
                }
            ],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "large_class".to_string(),
            "tight_coupling".to_string(),
            "resource_leak".to_string(),
        ],
        tags: vec![
            "rust".to_string(),
            "ownership".to_string(),
            "composition".to_string(),
            "traits".to_string(),
        ],
        frequency_score: 0.7,
        detection_confidence: 0.9,
    });

    // Resource Leak in Rust context
    patterns.insert("resource_leak".to_string(), PatternKnowledge {
        id: "rust_resource_leak".to_string(),
        name: "Resource Leak (Rust)".to_string(),
        definition: CompressedString::new(
            "Despite RAII, resources can still leak through manual memory management, \
             circular references, forgetting to join threads, or not awaiting async tasks."
        ),
        symptoms: vec![
            CompressedString::new("Manual memory management with Box::leak()"),
            CompressedString::new("Forgetting to close files/sockets explicitly"),
            CompressedString::new("Circular references with Rc<RefCell<T>>"),
            CompressedString::new("Thread handles not joined"),
            CompressedString::new("Async tasks spawned but not awaited"),
            CompressedString::new("Using mem::forget() inappropriately"),
            CompressedString::new("Custom Drop implementations that don't clean up"),
        ],
        impact: ImpactLevel::Critical,
        category: AntiPatternCategory::Resources,
        detection_methods: vec![
            DetectionMethod::StaticAnalysis {
                pattern: "manual_memory_management".to_string(),
                confidence: 0.95,
            },
            DetectionMethod::RegexPattern {
                pattern: r"Box::leak|mem::forget|Rc<RefCell".to_string(),
                context: "potential_leak".to_string(),
            },
            DetectionMethod::AstPattern {
                query: "missing_drop_impl".to_string(),
                node_types: vec!["struct_item".to_string()],
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "rust_raii_drop".to_string(),
                title: "RAII and Drop Trait".to_string(),
                implementation: CompressedString::new(
                    "Implement Drop trait for automatic resource cleanup and use RAII principles"
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Rust,
                        problem_code: CompressedString::new(
                            "struct ResourceManager {\n\
                                 file: File,\n\
                                 connection: TcpStream,\n\
                                 buffer: Vec<u8>,\n\
                             }\n\
                             \n\
                             // No Drop implementation - resources may not be cleaned up properly\n\
                             impl ResourceManager {\n\
                                 fn process(&mut self) {\n\
                                     // Manual cleanup attempted but error-prone\n\
                                     self.connection.shutdown(Shutdown::Both).unwrap();\n\
                                 }\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "struct ResourceManager {\n\
                                 file: File,\n\
                                 connection: TcpStream,\n\
                                 buffer: Vec<u8>,\n\
                             }\n\
                             \n\
                             impl Drop for ResourceManager {\n\
                                 fn drop(&mut self) {\n\
                                     // Guaranteed cleanup on scope exit\n\
                                     if let Err(e) = self.connection.shutdown(Shutdown::Both) {\n\
                                         eprintln!(\"Failed to shutdown connection: {}\", e);\n\
                                     }\n\
                                     // File automatically closed by its Drop impl\n\
                                 }\n\
                             }\n\
                             \n\
                             // Or use RAII wrapper types\n\
                             struct SafeResource<T> {\n\
                                 resource: T,\n\
                                 cleanup: Box<dyn FnOnce(&mut T)>,\n\
                             }\n\
                             \n\
                             impl<T> Drop for SafeResource<T> {\n\
                                 fn drop(&mut self) {\n\
                                     // Safe cleanup with custom function\n\
                                     (self.cleanup)(&mut self.resource);\n\
                                 }\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "RAII ensures guaranteed cleanup on scope exit, even with panics. \
                             Benefits: automatic cleanup, exception safety, no manual resource management"
                        ),
                        file_context: Some("resource_manager.rs".to_string()),
                    }
                ],
                effort_level: EffortLevel::Low,
                prerequisites: vec!["Understanding of Rust Drop trait".to_string()],
                expected_impact: ImpactLevel::High,
            },
            SolutionPattern {
                id: "rust_weak_references".to_string(),
                title: "Weak References for Cycles".to_string(),
                implementation: CompressedString::new(
                    "Use Weak<T> to break circular references in Rc<T> structures"
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Rust,
                        problem_code: CompressedString::new(
                            "use std::rc::Rc;\n\
                             use std::cell::RefCell;\n\
                             \n\
                             struct Node {\n\
                                 value: i32,\n\
                                 parent: Option<Rc<RefCell<Node>>>,\n\
                                 children: Vec<Rc<RefCell<Node>>>,\n\
                             }\n\
                             \n\
                             // Creates circular references - memory leak!\n\
                             let parent = Rc::new(RefCell::new(Node { \n\
                                 value: 1, parent: None, children: vec![] \n\
                             }));\n\
                             let child = Rc::new(RefCell::new(Node { \n\
                                 value: 2, parent: Some(parent.clone()), children: vec![] \n\
                             }));\n\
                             parent.borrow_mut().children.push(child.clone());"
                        ),
                        solution_code: CompressedString::new(
                            "use std::rc::{Rc, Weak};\n\
                             use std::cell::RefCell;\n\
                             \n\
                             struct Node {\n\
                                 value: i32,\n\
                                 parent: Option<Weak<RefCell<Node>>>, // Weak reference\n\
                                 children: Vec<Rc<RefCell<Node>>>,\n\
                             }\n\
                             \n\
                             // No circular references - proper cleanup\n\
                             let parent = Rc::new(RefCell::new(Node { \n\
                                 value: 1, parent: None, children: vec![] \n\
                             }));\n\
                             let child = Rc::new(RefCell::new(Node { \n\
                                 value: 2, \n\
                                 parent: Some(Rc::downgrade(&parent)), // Weak reference\n\
                                 children: vec![] \n\
                             }));\n\
                             parent.borrow_mut().children.push(child.clone());\n\
                             \n\
                             // Access parent through weak reference\n\
                             if let Some(parent) = child.borrow().parent.as_ref().and_then(|p| p.upgrade()) {\n\
                                 println!(\"Parent value: {}\", parent.borrow().value);\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Use Weak<T> for parent references to break cycles. \
                             Only children hold strong references, allowing proper cleanup."
                        ),
                        file_context: Some("tree_structure.rs".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec!["Understanding of Rc/Weak pattern".to_string()],
                expected_impact: ImpactLevel::High,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "god_object".to_string(),
            "inappropriate_intimacy".to_string(),
        ],
        tags: vec![
            "rust".to_string(),
            "memory".to_string(),
            "raii".to_string(),
            "drop".to_string(),
        ],
        frequency_score: 0.4,
        detection_confidence: 0.9,
    });

    patterns
}

/// Rust-specific idioms and conventions
fn create_rust_idioms() -> Vec<LanguageIdiom> {
    vec![
        LanguageIdiom {
            name: "Builder Pattern".to_string(),
            description: CompressedString::new(
                "Use builder pattern for structs with many optional fields, \
                 leveraging Rust's ownership and type system for compile-time guarantees"
            ),
            example: CodeExample {
                language: SourceLanguage::Rust,
                problem_code: CompressedString::new(
                    "// Constructor with many parameters - error prone\n\
                     struct Config {\n\
                         host: String,\n\
                         port: Option<u16>,\n\
                         timeout: Option<Duration>,\n\
                         retries: Option<u32>,\n\
                         ssl: Option<bool>,\n\
                     }\n\
                     \n\
                     impl Config {\n\
                         fn new(host: String, port: Option<u16>, timeout: Option<Duration>, \\\n\
                                retries: Option<u32>, ssl: Option<bool>) -> Self {\n\
                             Config { host, port, timeout, retries, ssl }\n\
                         }\n\
                     }"
                ),
                solution_code: CompressedString::new(
                    "struct Config {\n\
                         host: String,\n\
                         port: u16,\n\
                         timeout: Duration,\n\
                         retries: u32,\n\
                         ssl: bool,\n\
                     }\n\
                     \n\
                     struct ConfigBuilder {\n\
                         host: Option<String>,\n\
                         port: Option<u16>,\n\
                         timeout: Option<Duration>,\n\
                         retries: Option<u32>,\n\
                         ssl: Option<bool>,\n\
                     }\n\
                     \n\
                     impl ConfigBuilder {\n\
                         fn new() -> Self {\n\
                             ConfigBuilder {\n\
                                 host: None, port: None, timeout: None,\n\
                                 retries: None, ssl: None,\n\
                             }\n\
                         }\n\
                         \n\
                         fn host(mut self, host: impl Into<String>) -> Self {\n\
                             self.host = Some(host.into());\n\
                             self\n\
                         }\n\
                         \n\
                         fn port(mut self, port: u16) -> Self {\n\
                             self.port = Some(port);\n\
                             self\n\
                         }\n\
                         \n\
                         fn build(self) -> Result<Config, &'static str> {\n\
                             Ok(Config {\n\
                                 host: self.host.ok_or(\"host is required\")?,\n\
                                 port: self.port.unwrap_or(8080),\n\
                                 timeout: self.timeout.unwrap_or(Duration::from_secs(30)),\n\
                                 retries: self.retries.unwrap_or(3),\n\
                                 ssl: self.ssl.unwrap_or(false),\n\
                             })\n\
                         }\n\
                     }\n\
                     \n\
                     // Usage:\n\
                     let config = ConfigBuilder::new()\n\
                         .host(\"localhost\")\n\
                         .port(9000)\n\
                         .ssl(true)\n\
                         .build()?;"
                ),
                explanation: CompressedString::new(
                    "Builder pattern provides fluent API, compile-time validation, and clear defaults"
                ),
                file_context: Some("config.rs".to_string()),
            },
            when_to_use: CompressedString::new(
                "Structs with 4+ optional fields, complex configuration objects, \
                 or when you need validation during construction"
            ),
            alternatives: vec![
                "Default trait implementation".to_string(),
                "Constructor functions with Option parameters".to_string(),
                "derive_builder crate for automatic generation".to_string(),
            ],
        },
        LanguageIdiom {
            name: "Error Propagation with ?".to_string(),
            description: CompressedString::new(
                "Use ? operator for error propagation instead of unwrap() or manual matching"
            ),
            example: CodeExample {
                language: SourceLanguage::Rust,
                problem_code: CompressedString::new(
                    "fn process_file(path: &Path) -> Result<String, Error> {\n\
                         let content = match fs::read_to_string(path) {\n\
                             Ok(content) => content,\n\
                             Err(e) => return Err(Error::from(e)),\n\
                         };\n\
                         \n\
                         let processed = match transform_content(&content) {\n\
                             Ok(processed) => processed,\n\
                             Err(e) => return Err(Error::from(e)),\n\
                         };\n\
                         \n\
                         Ok(processed)\n\
                     }"
                ),
                solution_code: CompressedString::new(
                    "fn process_file(path: &Path) -> Result<String, Error> {\n\
                         let content = fs::read_to_string(path)?;\n\
                         let processed = transform_content(&content)?;\n\
                         Ok(processed)\n\
                     }\n\
                     \n\
                     // Or with custom error conversion\n\
                     fn process_file_detailed(path: &Path) -> Result<String, ProcessingError> {\n\
                         let content = fs::read_to_string(path)\n\
                             .map_err(ProcessingError::FileRead)?;\n\
                         \n\
                         let processed = transform_content(&content)\n\
                             .map_err(ProcessingError::Transform)?;\n\
                         \n\
                         Ok(processed)\n\
                     }"
                ),
                explanation: CompressedString::new(
                    "? operator automatically converts errors using From trait, \
                     making error handling concise and composable"
                ),
                file_context: Some("file_processor.rs".to_string()),
            },
            when_to_use: CompressedString::new(
                "Any fallible operation where you want to propagate errors up the call stack"
            ),
            alternatives: vec![
                "match expressions for specific error handling".to_string(),
                "unwrap_or_else for default values".to_string(),
                "map_err for error conversion".to_string(),
            ],
        },
        LanguageIdiom {
            name: "Iterator Adapters".to_string(),
            description: CompressedString::new(
                "Use iterator adapters for functional-style data processing \
                 instead of manual loops - more efficient and expressive"
            ),
            example: CodeExample {
                language: SourceLanguage::Rust,
                problem_code: CompressedString::new(
                    "// Imperative style - verbose and error-prone\n\
                     fn process_numbers(numbers: Vec<i32>) -> Vec<String> {\n\
                         let mut result = Vec::new();\n\
                         for num in numbers {\n\
                             if num % 2 == 0 {\n\
                                 let squared = num * num;\n\
                                 if squared > 10 {\n\
                                     result.push(format!(\"Even: {}\", squared));\n\
                                 }\n\
                             }\n\
                         }\n\
                         result\n\
                     }"
                ),
                solution_code: CompressedString::new(
                    "// Functional style - clear intent and efficient\n\
                     fn process_numbers(numbers: Vec<i32>) -> Vec<String> {\n\
                         numbers\n\
                             .into_iter()\n\
                             .filter(|&n| n % 2 == 0)      // Keep even numbers\n\
                             .map(|n| n * n)               // Square them\n\
                             .filter(|&n| n > 10)          // Keep if > 10\n\
                             .map(|n| format!(\"Even: {}\", n)) // Format as string\n\
                             .collect()\n\
                     }\n\
                     \n\
                     // Zero-allocation version with iterators\n\
                     fn process_numbers_lazy(numbers: &[i32]) -> impl Iterator<Item = String> + '_ {\n\
                         numbers\n\
                             .iter()\n\
                             .filter(|&&n| n % 2 == 0)\n\
                             .map(|&n| n * n)\n\
                             .filter(|&n| n > 10)\n\
                             .map(|n| format!(\"Even: {}\", n))\n\
                     }"
                ),
                explanation: CompressedString::new(
                    "Iterator adapters are zero-cost abstractions that compile to efficient code \
                     while expressing intent clearly"
                ),
                file_context: Some("data_processing.rs".to_string()),
            },
            when_to_use: CompressedString::new(
                "Data transformation, filtering, mapping operations - \
                 preferred over manual loops in most cases"
            ),
            alternatives: vec![
                "for loops for complex logic".to_string(),
                "while loops for stateful iteration".to_string(),
                "rayon for parallel iteration".to_string(),
            ],
        },
    ]
}

/// Rust framework-specific knowledge
fn create_rust_frameworks() -> HashMap<String, FrameworkKnowledge> {
    let mut frameworks = HashMap::new();

    frameworks.insert(
        "tokio".to_string(),
        FrameworkKnowledge {
            framework_name: "Tokio".to_string(),
            version_range: "1.0+".to_string(),
            specific_patterns: vec![
                "async_blocking_operations".to_string(),
                "unbounded_task_spawning".to_string(),
                "panic_in_async_task".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use spawn_blocking for CPU-intensive work"),
                CompressedString::new("Implement proper backpressure with bounded channels"),
                CompressedString::new("Use timeout for external operations"),
                CompressedString::new("Handle task panics with proper error propagation"),
                CompressedString::new("Use structured concurrency with JoinSet"),
                CompressedString::new("Implement graceful shutdown with CancellationToken"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Blocking operations in async context (use spawn_blocking)"),
                CompressedString::new(
                    "Creating too many tasks without bounds (use semaphores/pools)",
                ),
                CompressedString::new("Not handling task panics (tasks fail silently)"),
                CompressedString::new("Async recursion without Box<> (stack overflow)"),
                CompressedString::new("Holding locks across .await points (deadlocks)"),
            ],
        },
    );

    frameworks.insert(
        "serde".to_string(),
        FrameworkKnowledge {
            framework_name: "Serde".to_string(),
            version_range: "1.0+".to_string(),
            specific_patterns: vec![
                "primitive_obsession_serde".to_string(),
                "untrusted_deserialization".to_string(),
                "missing_field_validation".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use strong types instead of primitive obsession"),
                CompressedString::new("Implement custom Deserialize for validation"),
                CompressedString::new("Use #[serde(default)] for optional fields"),
                CompressedString::new("Validate data during deserialization, not after"),
                CompressedString::new("Use #[serde(with)] for custom serialization logic"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Deserializing untrusted data without validation"),
                CompressedString::new("Using String for all fields instead of proper types"),
                CompressedString::new("Not handling missing fields gracefully"),
                CompressedString::new("Infinite recursion in custom serialization"),
                CompressedString::new("Performance issues with large enums"),
            ],
        },
    );

    frameworks.insert(
        "clap".to_string(),
        FrameworkKnowledge {
            framework_name: "Clap".to_string(),
            version_range: "4.0+".to_string(),
            specific_patterns: vec![
                "argument_validation_missing".to_string(),
                "help_text_inconsistency".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use derive API for type safety"),
                CompressedString::new("Implement custom validation with value_parser"),
                CompressedString::new("Provide comprehensive help text"),
                CompressedString::new("Use subcommands for complex CLIs"),
                CompressedString::new("Handle argument conflicts explicitly"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Missing input validation"),
                CompressedString::new("Inconsistent help text"),
                CompressedString::new("Not handling argument conflicts"),
                CompressedString::new("Complex boolean logic in argument parsing"),
            ],
        },
    );

    frameworks
}

/// Rust standard library patterns
fn create_rust_stdlib_patterns() -> Vec<StdlibPattern> {
    vec![
        StdlibPattern {
            pattern_name: "Option and Result".to_string(),
            description: CompressedString::new(
                "Proper use of Option<T> and Result<T, E> for error handling and null safety",
            ),
            recommended_usage: CompressedString::new(
                "Use Option for values that may not exist, Result for operations that may fail. \
                 Combine with ? operator, map, and_then, etc. for ergonomic handling.",
            ),
            alternatives: vec![
                "unwrap() for prototyping only".to_string(),
                "expect() with descriptive messages".to_string(),
                "unwrap_or_default() for fallback values".to_string(),
            ],
        },
        StdlibPattern {
            pattern_name: "Vec vs slice".to_string(),
            description: CompressedString::new(
                "Choose between Vec<T> for owned data and &[T] for borrowed slices",
            ),
            recommended_usage: CompressedString::new(
                "Use &[T] in function parameters for maximum flexibility. \
                 Use Vec<T> when you need ownership or mutation. \
                 Consider Box<[T]> for immutable owned data.",
            ),
            alternatives: vec![
                "Arrays [T; N] for fixed-size data".to_string(),
                "VecDeque for double-ended operations".to_string(),
                "SmallVec for stack-allocated small vectors".to_string(),
            ],
        },
        StdlibPattern {
            pattern_name: "String vs str".to_string(),
            description: CompressedString::new(
                "Distinguish between owned String and borrowed &str for text handling",
            ),
            recommended_usage: CompressedString::new(
                "Use &str for function parameters and borrowed text. \
                 Use String for owned, mutable, or dynamically created text. \
                 Use Cow<str> when you might need either.",
            ),
            alternatives: vec![
                "OsString/OsStr for OS-specific strings".to_string(),
                "CString/CStr for C FFI".to_string(),
                "Cow<str> for clone-on-write semantics".to_string(),
            ],
        },
    ]
}

/// Create comprehensive Python-specific knowledge
fn create_python_knowledge() -> LanguageKnowledge {
    LanguageKnowledge {
        language: SourceLanguage::Python,
        patterns: create_python_patterns(),
        idioms: create_python_idioms(),
        frameworks: create_python_frameworks(),
        stdlib_patterns: create_python_stdlib_patterns(),
    }
}

fn create_python_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in Python context
    patterns.insert(
        "god_object".to_string(),
        PatternKnowledge {
            id: "python_god_object".to_string(),
            name: "God Object (Python)".to_string(),
            definition: CompressedString::new(
                "A class with too many methods and responsibilities that violates Python's \
             'Simple is better than complex' principle and duck typing philosophy.",
            ),
            symptoms: vec![
                CompressedString::new(
                    "Class with 25+ methods (Python allows more due to duck typing)",
                ),
                CompressedString::new("Class with 20+ instance variables"),
                CompressedString::new("__init__ method with 10+ parameters"),
                CompressedString::new("Multiple inheritance from unrelated classes"),
                CompressedString::new("Class that imports many unrelated modules"),
                CompressedString::new("Utility classes with only @staticmethod or @classmethod"),
                CompressedString::new("Classes that handle multiple data formats"),
            ],
            impact: ImpactLevel::High,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![
                DetectionMethod::MetricThreshold {
                    metric_name: "method_count".to_string(),
                    threshold: 25.0, // Higher for Python due to duck typing
                    operator: ComparisonOperator::GreaterThan,
                },
                DetectionMethod::MetricThreshold {
                    metric_name: "attribute_count".to_string(),
                    threshold: 20.0,
                    operator: ComparisonOperator::GreaterThan,
                },
                DetectionMethod::MetricThreshold {
                    metric_name: "import_count".to_string(),
                    threshold: 15.0,
                    operator: ComparisonOperator::GreaterThan,
                },
            ],
            solutions: vec![SolutionPattern {
                id: "python_composition_mixins".to_string(),
                title: "Composition with Mixins".to_string(),
                implementation: CompressedString::new(
                    "Use mixins and composition instead of large classes. \
                     Leverage Python's duck typing and multiple inheritance judiciously.",
                ),
                examples: vec![CodeExample {
                    language: SourceLanguage::Python,
                    problem_code: CompressedString::new(
                        "# God class - too many responsibilities\n\
                             class UserManager:\n\
                                 def __init__(self, db_url, email_config, cache_config):\n\
                                     self.db = Database(db_url)\n\
                                     self.email = EmailService(email_config)\n\
                                     self.cache = CacheService(cache_config)\n\
                                     # ... many more attributes\n\
                             \n\
                                 def create_user(self, user_data): ...\n\
                                 def delete_user(self, user_id): ...\n\
                                 def send_welcome_email(self, user): ...\n\
                                 def send_notification(self, user, message): ...\n\
                                 def cache_user_data(self, user): ...\n\
                                 def invalidate_cache(self, user_id): ...\n\
                                 def generate_report(self): ...\n\
                                 def export_users(self, format): ...\n\
                                 # ... 20+ more methods",
                    ),
                    solution_code: CompressedString::new(
                        "# Split into focused classes using composition\n\
                             class UserRepository:\n\
                                 \"\"\"Handles user data persistence.\"\"\"\n\
                                 def __init__(self, db):\n\
                                     self.db = db\n\
                             \n\
                                 def create_user(self, user_data): ...\n\
                                 def delete_user(self, user_id): ...\n\
                                 def find_user(self, user_id): ...\n\
                             \n\
                             class NotificationService:\n\
                                 \"\"\"Handles user notifications.\"\"\"\n\
                                 def __init__(self, email_service):\n\
                                     self.email = email_service\n\
                             \n\
                                 def send_welcome_email(self, user): ...\n\
                                 def send_notification(self, user, message): ...\n\
                             \n\
                             class UserCacheService:\n\
                                 \"\"\"Handles user data caching.\"\"\"\n\
                                 def __init__(self, cache):\n\
                                     self.cache = cache\n\
                             \n\
                                 def cache_user_data(self, user): ...\n\
                                 def invalidate_cache(self, user_id): ...\n\
                             \n\
                             # Compose services using dependency injection\n\
                             class UserService:\n\
                                 \"\"\"Orchestrates user operations.\"\"\"\n\
                                 def __init__(self, repository, notifications, cache):\n\
                                     self.repository = repository\n\
                                     self.notifications = notifications\n\
                                     self.cache = cache\n\
                             \n\
                                 def create_user(self, user_data):\n\
                                     user = self.repository.create_user(user_data)\n\
                                     self.cache.cache_user_data(user)\n\
                                     self.notifications.send_welcome_email(user)\n\
                                     return user\n\
                             \n\
                             # Factory for easy setup\n\
                             def create_user_service(config):\n\
                                 db = Database(config.db_url)\n\
                                 email = EmailService(config.email)\n\
                                 cache = CacheService(config.cache)\n\
                                     \n\
                                 repository = UserRepository(db)\n\
                                 notifications = NotificationService(email)\n\
                                 cache_service = UserCacheService(cache)\n\
                                 \n\
                                 return UserService(repository, notifications, cache_service)",
                    ),
                    explanation: CompressedString::new(
                        "Split the god class into focused, single-purpose classes. \
                             Benefits: better testability with dependency injection, \
                             clearer separation of concerns, easier to mock individual components.",
                    ),
                    file_context: Some("user_management.py".to_string()),
                }],
                effort_level: EffortLevel::Medium,
                prerequisites: vec![
                    "Understanding of Python's composition patterns".to_string(),
                    "Test coverage for refactoring safety".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            }],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec!["large_class".to_string(), "feature_envy".to_string()],
            tags: vec![
                "python".to_string(),
                "composition".to_string(),
                "duck-typing".to_string(),
            ],
            frequency_score: 0.75,
            detection_confidence: 0.85,
        },
    );

    patterns
}

fn create_python_idioms() -> Vec<LanguageIdiom> {
    vec![LanguageIdiom {
        name: "List Comprehensions".to_string(),
        description: CompressedString::new(
            "Use list comprehensions for concise, readable data transformations \
                 instead of explicit loops",
        ),
        example: CodeExample {
            language: SourceLanguage::Python,
            problem_code: CompressedString::new(
                "# Verbose imperative style\n\
                     result = []\n\
                     for item in items:\n\
                         if item.is_valid():\n\
                             processed = item.process()\n\
                             if processed is not None:\n\
                                 result.append(processed.upper())",
            ),
            solution_code: CompressedString::new(
                "# Concise list comprehension\n\
                     result = [\n\
                         processed.upper()\n\
                         for item in items\n\
                         if item.is_valid()\n\
                         for processed in [item.process()]\n\
                         if processed is not None\n\
                     ]\n\
                     \n\
                     # Or with generator for memory efficiency\n\
                     result = (\n\
                         processed.upper()\n\
                         for item in items\n\
                         if item.is_valid()\n\
                         for processed in [item.process()]\n\
                         if processed is not None\n\
                     )",
            ),
            explanation: CompressedString::new(
                "List comprehensions are more readable and often faster than equivalent loops",
            ),
            file_context: Some("data_processing.py".to_string()),
        },
        when_to_use: CompressedString::new("Simple transformations and filtering operations"),
        alternatives: vec![
            "map() and filter() for functional style".to_string(),
            "explicit loops for complex logic".to_string(),
            "generator expressions for memory efficiency".to_string(),
        ],
    }]
}

fn create_python_frameworks() -> HashMap<String, FrameworkKnowledge> {
    let mut frameworks = HashMap::new();

    frameworks.insert(
        "django".to_string(),
        FrameworkKnowledge {
            framework_name: "Django".to_string(),
            version_range: "3.0+".to_string(),
            specific_patterns: vec![
                "fat_models".to_string(),
                "n_plus_one_queries".to_string(),
                "circular_imports".to_string(),
            ],
            best_practices: vec![
                CompressedString::new(
                    "Use select_related and prefetch_related for query optimization",
                ),
                CompressedString::new("Keep models thin, use services for business logic"),
                CompressedString::new(
                    "Use Django's built-in security features (CSRF, XSS protection)",
                ),
                CompressedString::new("Organize apps by business domain, not technical layer"),
                CompressedString::new("Use custom managers for complex queries"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Fat models with business logic (use services instead)"),
                CompressedString::new("N+1 query problems (use select_related/prefetch_related)"),
                CompressedString::new("Circular imports between apps"),
                CompressedString::new("Not using database transactions properly"),
                CompressedString::new("Mixing business logic in views"),
            ],
        },
    );

    frameworks
}

fn create_python_stdlib_patterns() -> Vec<StdlibPattern> {
    vec![StdlibPattern {
        pattern_name: "Context Managers".to_string(),
        description: CompressedString::new(
            "Use context managers (with statement) for proper resource management",
        ),
        recommended_usage: CompressedString::new(
            "Always use 'with' for file operations, database connections, locks, etc. \
                 Create custom context managers with __enter__ and __exit__ methods.",
        ),
        alternatives: vec![
            "try/finally for manual cleanup".to_string(),
            "@contextmanager decorator for simple cases".to_string(),
            "contextlib.ExitStack for multiple resources".to_string(),
        ],
    }]
}

/// Create comprehensive JavaScript-specific knowledge
fn create_javascript_knowledge() -> LanguageKnowledge {
    LanguageKnowledge {
        language: SourceLanguage::JavaScript,
        patterns: create_javascript_patterns(),
        idioms: create_javascript_idioms(),
        frameworks: create_javascript_frameworks(),
        stdlib_patterns: create_javascript_stdlib_patterns(),
    }
}

fn create_javascript_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in JavaScript context
    patterns.insert("god_object".to_string(), PatternKnowledge {
        id: "javascript_god_object".to_string(),
        name: "God Object (JavaScript)".to_string(),
        definition: CompressedString::new(
            "A JavaScript object or class with too many properties and methods, \
             often created through prototype pollution or excessive monolithic design."
        ),
        symptoms: vec![
            CompressedString::new("Object with 30+ properties/methods (JS is more flexible)"),
            CompressedString::new("Constructor function with excessive parameters"),
            CompressedString::new("Prototype with many unrelated methods"),
            CompressedString::new("Global object pollution"),
            CompressedString::new("This context confusion in methods"),
            CompressedString::new("Module that exports everything"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![
            DetectionMethod::MetricThreshold {
                metric_name: "property_count".to_string(),
                threshold: 30.0,
                operator: ComparisonOperator::GreaterThan,
            },
            DetectionMethod::StaticAnalysis {
                pattern: "prototype_pollution".to_string(),
                confidence: 0.8,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "js_module_pattern".to_string(),
                title: "Module Pattern with Composition".to_string(),
                implementation: CompressedString::new(
                    "Use ES6 modules and class composition to break down large objects"
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::JavaScript,
                        problem_code: CompressedString::new(
                            "// God object - everything in one place\n\
                             class UserManager {\n\
                                 constructor(dbUrl, emailConfig, cacheConfig) {\n\
                                     this.db = new Database(dbUrl);\n\
                                     this.email = new EmailService(emailConfig);\n\
                                     this.cache = new CacheService(cacheConfig);\n\
                                     // ... many more properties\n\
                                 }\n\
                             \n\
                                 createUser(userData) { /* ... */ }\n\
                                 deleteUser(userId) { /* ... */ }\n\
                                 sendEmail(user, message) { /* ... */ }\n\
                                 validateEmail(email) { /* ... */ }\n\
                                 cacheUserData(user) { /* ... */ }\n\
                                 generateReport() { /* ... */ }\n\
                                 exportUsers(format) { /* ... */ }\n\
                                 // ... 25+ more methods\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "// Split into focused modules\n\
                             class UserRepository {\n\
                                 constructor(database) {\n\
                                     this.db = database;\n\
                                 }\n\
                             \n\
                                 async createUser(userData) {\n\
                                     return await this.db.users.create(userData);\n\
                                 }\n\
                             \n\
                                 async deleteUser(userId) {\n\
                                     return await this.db.users.delete(userId);\n\
                                 }\n\
                             }\n\
                             \n\
                             class NotificationService {\n\
                                 constructor(emailService) {\n\
                                     this.email = emailService;\n\
                                 }\n\
                             \n\
                                 async sendWelcomeEmail(user) {\n\
                                     const template = 'welcome';\n\
                                     return await this.email.send(user.email, template, { user });\n\
                                 }\n\
                             }\n\
                             \n\
                             class UserCacheService {\n\
                                 constructor(cache) {\n\
                                     this.cache = cache;\n\
                                 }\n\
                             \n\
                                 async cacheUser(user) {\n\
                                     const key = `user:${user.id}`;\n\
                                     return await this.cache.set(key, user, { ttl: 3600 });\n\
                                 }\n\
                             }\n\
                             \n\
                             // Compose services using dependency injection\n\
                             class UserService {\n\
                                 constructor({ repository, notifications, cache }) {\n\
                                     this.repository = repository;\n\
                                     this.notifications = notifications;\n\
                                     this.cache = cache;\n\
                                 }\n\
                             \n\
                                 async createUser(userData) {\n\
                                     const user = await this.repository.createUser(userData);\n\
                                     await this.cache.cacheUser(user);\n\
                                     await this.notifications.sendWelcomeEmail(user);\n\
                                     return user;\n\
                                 }\n\
                             }\n\
                             \n\
                             // Factory for dependency injection\n\
                             export function createUserService(config) {\n\
                                 const db = new Database(config.database);\n\
                                 const email = new EmailService(config.email);\n\
                                 const cache = new CacheService(config.cache);\n\
                             \n\
                                 return new UserService({\n\
                                     repository: new UserRepository(db),\n\
                                     notifications: new NotificationService(email),\n\
                                     cache: new UserCacheService(cache)\n\
                                 });\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Break the god object into focused, composable services. \
                             Use dependency injection for testability and ES6 modules for organization."
                        ),
                        file_context: Some("userService.js".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec![
                    "Understanding of ES6 modules and classes".to_string(),
                    "Dependency injection patterns in JavaScript".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "callback_hell".to_string(),
            "global_state".to_string(),
        ],
        tags: vec![
            "javascript".to_string(),
            "modules".to_string(),
            "composition".to_string(),
        ],
        frequency_score: 0.8,
        detection_confidence: 0.8,
    });

    patterns
}

fn create_javascript_idioms() -> Vec<LanguageIdiom> {
    vec![LanguageIdiom {
        name: "Async/Await over Promises".to_string(),
        description: CompressedString::new(
            "Use async/await syntax instead of .then() chains for better readability",
        ),
        example: CodeExample {
            language: SourceLanguage::JavaScript,
            problem_code: CompressedString::new(
                "// Promise chains - harder to read and debug\n\
                     function processUser(userId) {\n\
                         return fetchUser(userId)\n\
                             .then(user => validateUser(user))\n\
                             .then(validUser => enrichUserData(validUser))\n\
                             .then(enrichedUser => saveUser(enrichedUser))\n\
                             .then(savedUser => sendNotification(savedUser))\n\
                             .catch(error => {\n\
                                 console.error('Error processing user:', error);\n\
                                 throw error;\n\
                             });\n\
                     }",
            ),
            solution_code: CompressedString::new(
                "// Async/await - clear sequential flow\n\
                     async function processUser(userId) {\n\
                         try {\n\
                             const user = await fetchUser(userId);\n\
                             const validUser = await validateUser(user);\n\
                             const enrichedUser = await enrichUserData(validUser);\n\
                             const savedUser = await saveUser(enrichedUser);\n\
                             await sendNotification(savedUser);\n\
                             return savedUser;\n\
                         } catch (error) {\n\
                             console.error('Error processing user:', error);\n\
                             throw error;\n\
                         }\n\
                     }\n\
                     \n\
                     // For parallel operations\n\
                     async function processMultipleUsers(userIds) {\n\
                         try {\n\
                             const users = await Promise.all(\n\
                                 userIds.map(id => processUser(id))\n\
                             );\n\
                             return users;\n\
                         } catch (error) {\n\
                             console.error('Error processing users:', error);\n\
                             throw error;\n\
                         }\n\
                     }",
            ),
            explanation: CompressedString::new(
                "Async/await provides better error handling, debugging, and readability",
            ),
            file_context: Some("userProcessor.js".to_string()),
        },
        when_to_use: CompressedString::new(
            "Any asynchronous operation, especially sequential async calls",
        ),
        alternatives: vec![
            "Promise.then() for simple transformations".to_string(),
            "Promise.all() for parallel operations".to_string(),
            "Callbacks for event-based operations".to_string(),
        ],
    }]
}

fn create_javascript_frameworks() -> HashMap<String, FrameworkKnowledge> {
    let mut frameworks = HashMap::new();

    frameworks.insert(
        "react".to_string(),
        FrameworkKnowledge {
            framework_name: "React".to_string(),
            version_range: "16.8+".to_string(),
            specific_patterns: vec![
                "prop_drilling".to_string(),
                "unnecessary_renders".to_string(),
                "useeffect_dependencies".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use hooks instead of class components"),
                CompressedString::new("Memoize expensive computations with useMemo"),
                CompressedString::new("Use Context API to avoid prop drilling"),
                CompressedString::new("Include all dependencies in useEffect"),
                CompressedString::new("Split components into smaller, reusable pieces"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Prop drilling instead of context or state management"),
                CompressedString::new("Missing dependencies in useEffect"),
                CompressedString::new("Unnecessary re-renders due to object/array creation"),
                CompressedString::new("Mutating state directly instead of using setState"),
                CompressedString::new("Not cleaning up effects (memory leaks)"),
            ],
        },
    );

    frameworks.insert(
        "express".to_string(),
        FrameworkKnowledge {
            framework_name: "Express.js".to_string(),
            version_range: "4.0+".to_string(),
            specific_patterns: vec![
                "callback_hell_middleware".to_string(),
                "missing_error_handling".to_string(),
                "middleware_order_issues".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use async/await in route handlers"),
                CompressedString::new("Implement proper error handling middleware"),
                CompressedString::new("Validate input data with middleware"),
                CompressedString::new("Use helmet for security headers"),
                CompressedString::new("Implement request logging and monitoring"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Callback hell in middleware chains"),
                CompressedString::new("Missing error handling middleware"),
                CompressedString::new("Incorrect middleware order"),
                CompressedString::new("Not validating input data"),
                CompressedString::new("Blocking operations in request handlers"),
            ],
        },
    );

    frameworks
}

fn create_javascript_stdlib_patterns() -> Vec<StdlibPattern> {
    vec![StdlibPattern {
        pattern_name: "Array Methods".to_string(),
        description: CompressedString::new(
            "Use functional array methods (map, filter, reduce) instead of imperative loops",
        ),
        recommended_usage: CompressedString::new(
            "Prefer map() for transformations, filter() for selection, reduce() for aggregation. \
                 Chain methods for complex operations. Use forEach() only for side effects.",
        ),
        alternatives: vec![
            "for...of loops for complex logic".to_string(),
            "while loops for early termination".to_string(),
            "for loops for performance-critical code".to_string(),
        ],
    }]
}

/// Create comprehensive TypeScript-specific knowledge
fn create_typescript_knowledge() -> LanguageKnowledge {
    LanguageKnowledge {
        language: SourceLanguage::TypeScript,
        patterns: create_typescript_patterns(),
        idioms: create_typescript_idioms(),
        frameworks: create_typescript_frameworks(),
        stdlib_patterns: create_typescript_stdlib_patterns(),
    }
}

fn create_typescript_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // Any Type Abuse in TypeScript
    patterns.insert("any_type_abuse".to_string(), PatternKnowledge {
        id: "typescript_any_abuse".to_string(),
        name: "Any Type Abuse (TypeScript)".to_string(),
        definition: CompressedString::new(
            "Excessive use of 'any' type that defeats TypeScript's type safety, \
             often used as an escape hatch but undermines the benefits of static typing."
        ),
        symptoms: vec![
            CompressedString::new("Widespread use of 'any' type annotations"),
            CompressedString::new("Type assertions to 'any' to bypass compiler"),
            CompressedString::new("Function parameters and returns typed as 'any'"),
            CompressedString::new("Object properties typed as 'any'"),
            CompressedString::new("Array typed as 'any[]'"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::Maintainability,
        detection_methods: vec![
            DetectionMethod::RegexPattern {
                pattern: r": any\b|<any>|\bas any\b".to_string(),
                context: "type_annotation".to_string(),
            },
            DetectionMethod::StaticAnalysis {
                pattern: "any_type_usage".to_string(),
                confidence: 0.95,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "typescript_proper_typing".to_string(),
                title: "Proper Type Definitions".to_string(),
                implementation: CompressedString::new(
                    "Replace 'any' with specific types, interfaces, generics, or union types"
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::TypeScript,
                        problem_code: CompressedString::new(
                            "// Overuse of 'any' - loses type safety\n\
                             interface User {\n\
                                 id: number;\n\
                                 data: any; // Vague type\n\
                                 preferences: any; // Lost type information\n\
                             }\n\
                             \n\
                             function processData(input: any): any {\n\
                                 return input.map((item: any) => {\n\
                                     return {\n\
                                         ...item,\n\
                                         processed: transformValue(item.value as any)\n\
                                     };\n\
                                 });\n\
                             }\n\
                             \n\
                             // API response handling\n\
                             async function fetchUserData(userId: number): Promise<any> {\n\
                                 const response = await fetch(`/api/users/${userId}`);\n\
                                 return response.json(); // Could be anything\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "// Proper typing with interfaces and generics\n\
                             interface UserPreferences {\n\
                                 theme: 'light' | 'dark';\n\
                                 notifications: boolean;\n\
                                 language: string;\n\
                             }\n\
                             \n\
                             interface UserProfile {\n\
                                 firstName: string;\n\
                                 lastName: string;\n\
                                 email: string;\n\
                                 avatar?: string;\n\
                             }\n\
                             \n\
                             interface User {\n\
                                 id: number;\n\
                                 profile: UserProfile;\n\
                                 preferences: UserPreferences;\n\
                                 createdAt: Date;\n\
                             }\n\
                             \n\
                             // Generic function with proper constraints\n\
                             function processData<T extends { value: unknown }>(\n\
                                 input: T[]\n\
                             ): Array<T & { processed: string }> {\n\
                                 return input.map(item => ({\n\
                                     ...item,\n\
                                     processed: transformValue(item.value)\n\
                                 }));\n\
                             }\n\
                             \n\
                             // Strongly typed API response\n\
                             interface ApiResponse<T> {\n\
                                 data: T;\n\
                                 status: 'success' | 'error';\n\
                                 message?: string;\n\
                             }\n\
                             \n\
                             async function fetchUserData(userId: number): Promise<User> {\n\
                                 const response = await fetch(`/api/users/${userId}`);\n\
                                 const apiResponse: ApiResponse<User> = await response.json();\n\
                                 \n\
                                 if (apiResponse.status === 'error') {\n\
                                     throw new Error(apiResponse.message || 'Failed to fetch user');\n\
                                 }\n\
                                 \n\
                                 return apiResponse.data;\n\
                             }\n\
                             \n\
                             // Use unknown for truly unknown data, then narrow\n\
                             function processUnknownData(data: unknown): string {\n\
                                 if (typeof data === 'string') {\n\
                                     return data.toUpperCase();\n\
                                 }\n\
                                 if (typeof data === 'number') {\n\
                                     return data.toString();\n\
                                 }\n\
                                 return 'Invalid data';\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Use specific types, interfaces, and generics instead of 'any'. \
                             Benefits: compile-time error detection, better IDE support, \
                             self-documenting code, refactoring safety."
                        ),
                        file_context: Some("userService.ts".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec![
                    "Understanding of TypeScript type system".to_string(),
                    "Knowledge of generics and utility types".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "god_object".to_string(),
            "magic_values".to_string(),
        ],
        tags: vec![
            "typescript".to_string(),
            "types".to_string(),
            "safety".to_string(),
        ],
        frequency_score: 0.9,
        detection_confidence: 0.95,
    });

    patterns
}

fn create_typescript_idioms() -> Vec<LanguageIdiom> {
    vec![
        LanguageIdiom {
            name: "Type Guards".to_string(),
            description: CompressedString::new(
                "Use type guards to safely narrow types and avoid runtime errors"
            ),
            example: CodeExample {
                language: SourceLanguage::TypeScript,
                problem_code: CompressedString::new(
                    "// Unsafe type assumptions\n\
                     function processValue(value: unknown) {\n\
                         return (value as string).toUpperCase(); // Dangerous!\n\
                     }\n\
                     \n\
                     function handleApiResponse(response: any) {\n\
                         console.log(response.data.users[0].name); // Will crash if structure differs\n\
                     }"
                ),
                solution_code: CompressedString::new(
                    "// Safe type guards\n\
                     function isString(value: unknown): value is string {\n\
                         return typeof value === 'string';\n\
                     }\n\
                     \n\
                     function processValue(value: unknown): string {\n\
                         if (isString(value)) {\n\
                             return value.toUpperCase(); // TypeScript knows it's a string\n\
                         }\n\
                         return 'Invalid value';\n\
                     }\n\
                     \n\
                     // Complex type guard for API responses\n\
                     interface ApiResponse {\n\
                         data: {\n\
                             users: Array<{ name: string }>;\n\
                         };\n\
                     }\n\
                     \n\
                     function isValidApiResponse(response: unknown): response is ApiResponse {\n\
                         return (\n\
                             typeof response === 'object' &&\n\
                             response !== null &&\n\
                             'data' in response &&\n\
                             typeof (response as any).data === 'object' &&\n\
                             'users' in (response as any).data &&\n\
                             Array.isArray((response as any).data.users)\n\
                         );\n\
                     }\n\
                     \n\
                     function handleApiResponse(response: unknown) {\n\
                         if (isValidApiResponse(response)) {\n\
                             console.log(response.data.users[0]?.name || 'No name');\n\
                         } else {\n\
                             console.error('Invalid API response structure');\n\
                         }\n\
                     }"
                ),
                explanation: CompressedString::new(
                    "Type guards provide runtime type checking with compile-time type narrowing"
                ),
                file_context: Some("typeGuards.ts".to_string()),
            },
            when_to_use: CompressedString::new(
                "When working with unknown data, API responses, or user input"
            ),
            alternatives: vec![
                "Zod or Yup for schema validation".to_string(),
                "Type assertions for trusted data".to_string(),
                "Discriminated unions for known variants".to_string(),
            ],
        },
    ]
}

fn create_typescript_frameworks() -> HashMap<String, FrameworkKnowledge> {
    let mut frameworks = HashMap::new();

    frameworks.insert(
        "angular".to_string(),
        FrameworkKnowledge {
            framework_name: "Angular".to_string(),
            version_range: "12+".to_string(),
            specific_patterns: vec![
                "any_type_in_templates".to_string(),
                "untyped_observables".to_string(),
                "missing_interface_definitions".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use strict TypeScript configuration"),
                CompressedString::new("Define interfaces for all data structures"),
                CompressedString::new("Type RxJS observables properly"),
                CompressedString::new("Use generic types in services"),
                CompressedString::new("Implement proper error handling with typed errors"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Using 'any' in component templates"),
                CompressedString::new("Untyped HTTP client responses"),
                CompressedString::new("Missing type definitions for third-party libraries"),
                CompressedString::new("Not using strict mode compilation"),
            ],
        },
    );

    frameworks
}

fn create_typescript_stdlib_patterns() -> Vec<StdlibPattern> {
    vec![StdlibPattern {
        pattern_name: "Utility Types".to_string(),
        description: CompressedString::new(
            "Use TypeScript's built-in utility types for type transformations",
        ),
        recommended_usage: CompressedString::new(
            "Use Partial<T> for optional updates, Pick<T, K> for selecting properties, \
                 Omit<T, K> for excluding properties, Record<K, V> for key-value mappings.",
        ),
        alternatives: vec![
            "Custom mapped types for complex transformations".to_string(),
            "Conditional types for advanced logic".to_string(),
            "Template literal types for string manipulation".to_string(),
        ],
    }]
}

/// Create comprehensive Java-specific knowledge
fn create_java_knowledge() -> LanguageKnowledge {
    LanguageKnowledge {
        language: SourceLanguage::Java,
        patterns: create_java_patterns(),
        idioms: create_java_idioms(),
        frameworks: create_java_frameworks(),
        stdlib_patterns: create_java_stdlib_patterns(),
    }
}

fn create_java_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in Java context
    patterns.insert("god_object".to_string(), PatternKnowledge {
        id: "java_god_object".to_string(),
        name: "God Object (Java)".to_string(),
        definition: CompressedString::new(
            "A Java class with excessive methods, fields, and responsibilities that \
             violates Single Responsibility Principle and makes code difficult to maintain and test."
        ),
        symptoms: vec![
            CompressedString::new("Class with 25+ methods"),
            CompressedString::new("Class with 15+ instance variables"),
            CompressedString::new("Constructor with 8+ parameters"),
            CompressedString::new("Class imports from many different packages"),
            CompressedString::new("Multiple unrelated interfaces implemented"),
            CompressedString::new("Static utility methods mixed with instance methods"),
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
                metric_name: "field_count".to_string(),
                threshold: 15.0,
                operator: ComparisonOperator::GreaterThan,
            },
        ],
        solutions: vec![
            SolutionPattern {
                id: "java_extract_class".to_string(),
                title: "Extract Class with Dependency Injection".to_string(),
                implementation: CompressedString::new(
                    "Break down large classes using dependency injection frameworks like Spring"
                ),
                examples: vec![
                    CodeExample {
                        language: SourceLanguage::Java,
                        problem_code: CompressedString::new(
                            "// God class - too many responsibilities\n\
                             @Service\n\
                             public class UserManager {\n\
                                 private DatabaseConnection db;\n\
                                 private EmailService email;\n\
                                 private CacheService cache;\n\
                                 private LoggingService logger;\n\
                                 private MetricsCollector metrics;\n\
                                 // ... many more fields\n\
                             \n\
                                 public User createUser(UserRequest request) { /* ... */ }\n\
                                 public void deleteUser(Long userId) { /* ... */ }\n\
                                 public void sendWelcomeEmail(User user) { /* ... */ }\n\
                                 public void validateEmail(String email) { /* ... */ }\n\
                                 public void cacheUserData(User user) { /* ... */ }\n\
                                 public void generateUserReport() { /* ... */ }\n\
                                 public List<User> exportUsers(String format) { /* ... */ }\n\
                                 // ... 20+ more methods\n\
                             }"
                        ),
                        solution_code: CompressedString::new(
                            "// Split into focused services\n\
                             @Repository\n\
                             public class UserRepository {\n\
                                 private final JdbcTemplate jdbcTemplate;\n\
                             \n\
                                 public UserRepository(JdbcTemplate jdbcTemplate) {\n\
                                     this.jdbcTemplate = jdbcTemplate;\n\
                                 }\n\
                             \n\
                                 public User save(User user) {\n\
                                     // Database operations only\n\
                                     return jdbcTemplate.update(/* ... */);\n\
                                 }\n\
                             \n\
                                 public Optional<User> findById(Long id) {\n\
                                     return jdbcTemplate.queryForObject(/* ... */);\n\
                                 }\n\
                             }\n\
                             \n\
                             @Service\n\
                             public class NotificationService {\n\
                                 private final EmailService emailService;\n\
                             \n\
                                 public NotificationService(EmailService emailService) {\n\
                                     this.emailService = emailService;\n\
                                 }\n\
                             \n\
                                 public void sendWelcomeEmail(User user) {\n\
                                     EmailTemplate template = EmailTemplate.WELCOME;\n\
                                     emailService.send(user.getEmail(), template, Map.of(\"user\", user));\n\
                                 }\n\
                             }\n\
                             \n\
                             @Service\n\
                             public class UserCacheService {\n\
                                 private final CacheManager cacheManager;\n\
                             \n\
                                 public UserCacheService(CacheManager cacheManager) {\n\
                                     this.cacheManager = cacheManager;\n\
                                 }\n\
                             \n\
                                 @Cacheable(value = \"users\", key = \"#user.id\")\n\
                                 public void cacheUser(User user) {\n\
                                     // Caching handled by Spring's @Cacheable\n\
                                 }\n\
                             }\n\
                             \n\
                             // Orchestrating service using composition\n\
                             @Service\n\
                             @Transactional\n\
                             public class UserService {\n\
                                 private final UserRepository userRepository;\n\
                                 private final NotificationService notificationService;\n\
                                 private final UserCacheService cacheService;\n\
                             \n\
                                 public UserService(\n\
                                     UserRepository userRepository,\n\
                                     NotificationService notificationService,\n\
                                     UserCacheService cacheService\n\
                                 ) {\n\
                                     this.userRepository = userRepository;\n\
                                     this.notificationService = notificationService;\n\
                                     this.cacheService = cacheService;\n\
                                 }\n\
                             \n\
                                 public User createUser(UserRequest request) {\n\
                                     User user = User.fromRequest(request);\n\
                                     User savedUser = userRepository.save(user);\n\
                                     cacheService.cacheUser(savedUser);\n\
                                     notificationService.sendWelcomeEmail(savedUser);\n\
                                     return savedUser;\n\
                                 }\n\
                             }"
                        ),
                        explanation: CompressedString::new(
                            "Split the god class into focused services using Spring's dependency injection. \
                             Each service has a single responsibility and can be easily tested and maintained."
                        ),
                        file_context: Some("UserService.java".to_string()),
                    }
                ],
                effort_level: EffortLevel::Medium,
                prerequisites: vec![
                    "Understanding of Spring Framework".to_string(),
                    "Knowledge of dependency injection patterns".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            },
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![
            "tight_coupling".to_string(),
            "large_class".to_string(),
        ],
        tags: vec![
            "java".to_string(),
            "spring".to_string(),
            "dependency-injection".to_string(),
        ],
        frequency_score: 0.8,
        detection_confidence: 0.9,
    });

    patterns
}

fn create_java_idioms() -> Vec<LanguageIdiom> {
    vec![
        LanguageIdiom {
            name: "Builder Pattern".to_string(),
            description: CompressedString::new(
                "Use builder pattern for objects with many optional parameters"
            ),
            example: CodeExample {
                language: SourceLanguage::Java,
                problem_code: CompressedString::new(
                    "// Constructor with many parameters\n\
                     public class DatabaseConfig {\n\
                         public DatabaseConfig(\n\
                             String host, Integer port, String database,\n\
                             String username, String password, Boolean ssl,\n\
                             Integer timeout, Integer maxConnections\n\
                         ) {\n\
                             // ... initialization\n\
                         }\n\
                     }\n\
                     \n\
                     // Difficult to use and remember parameter order\n\
                     DatabaseConfig config = new DatabaseConfig(\n\
                         \"localhost\", 5432, \"mydb\", \"user\", \"pass\", true, 30, 10\n\
                     );"
                ),
                solution_code: CompressedString::new(
                    "// Builder pattern for complex objects\n\
                     public class DatabaseConfig {\n\
                         private final String host;\n\
                         private final Integer port;\n\
                         private final String database;\n\
                         private final String username;\n\
                         private final String password;\n\
                         private final Boolean ssl;\n\
                         private final Integer timeout;\n\
                         private final Integer maxConnections;\n\
                     \n\
                         private DatabaseConfig(Builder builder) {\n\
                             this.host = builder.host;\n\
                             this.port = builder.port;\n\
                             // ... assign all fields\n\
                         }\n\
                     \n\
                         public static class Builder {\n\
                             private String host = \"localhost\";\n\
                             private Integer port = 5432;\n\
                             private String database;\n\
                             private String username;\n\
                             private String password;\n\
                             private Boolean ssl = false;\n\
                             private Integer timeout = 30;\n\
                             private Integer maxConnections = 10;\n\
                     \n\
                             public Builder host(String host) {\n\
                                 this.host = host;\n\
                                 return this;\n\
                             }\n\
                     \n\
                             public Builder port(Integer port) {\n\
                                 this.port = port;\n\
                                 return this;\n\
                             }\n\
                     \n\
                             public Builder database(String database) {\n\
                                 this.database = database;\n\
                                 return this;\n\
                             }\n\
                     \n\
                             public DatabaseConfig build() {\n\
                                 if (database == null) {\n\
                                     throw new IllegalStateException(\"Database name is required\");\n\
                                 }\n\
                                 return new DatabaseConfig(this);\n\
                             }\n\
                         }\n\
                     \n\
                         public static Builder builder() {\n\
                             return new Builder();\n\
                         }\n\
                     }\n\
                     \n\
                     // Clear, readable usage\n\
                     DatabaseConfig config = DatabaseConfig.builder()\n\
                         .host(\"localhost\")\n\
                         .port(5432)\n\
                         .database(\"mydb\")\n\
                         .username(\"user\")\n\
                         .password(\"pass\")\n\
                         .ssl(true)\n\
                         .build();"
                ),
                explanation: CompressedString::new(
                    "Builder pattern provides fluent API and validates required fields"
                ),
                file_context: Some("DatabaseConfig.java".to_string()),
            },
            when_to_use: CompressedString::new(
                "Objects with 4+ parameters, especially with optional fields"
            ),
            alternatives: vec![
                "Lombok @Builder annotation".to_string(),
                "Factory methods with parameter objects".to_string(),
                "Telescoping constructor pattern".to_string(),
            ],
        },
    ]
}

fn create_java_frameworks() -> HashMap<String, FrameworkKnowledge> {
    let mut frameworks = HashMap::new();

    frameworks.insert(
        "spring".to_string(),
        FrameworkKnowledge {
            framework_name: "Spring Framework".to_string(),
            version_range: "5.0+".to_string(),
            specific_patterns: vec![
                "circular_bean_dependencies".to_string(),
                "overuse_of_autowired".to_string(),
                "missing_transaction_boundaries".to_string(),
            ],
            best_practices: vec![
                CompressedString::new("Use constructor injection over field injection"),
                CompressedString::new("Define clear transaction boundaries with @Transactional"),
                CompressedString::new("Use @Configuration classes for bean definitions"),
                CompressedString::new("Implement proper exception handling with @ControllerAdvice"),
                CompressedString::new("Use profiles for environment-specific configuration"),
            ],
            common_pitfalls: vec![
                CompressedString::new("Circular bean dependencies"),
                CompressedString::new("Overuse of @Autowired leading to tight coupling"),
                CompressedString::new("Missing or incorrect @Transactional annotations"),
                CompressedString::new("Not handling lazy initialization properly"),
                CompressedString::new("Mixing business logic in controllers"),
            ],
        },
    );

    frameworks
}

fn create_java_stdlib_patterns() -> Vec<StdlibPattern> {
    vec![StdlibPattern {
        pattern_name: "Optional".to_string(),
        description: CompressedString::new(
            "Use Optional<T> to handle null values safely and expressively",
        ),
        recommended_usage: CompressedString::new(
            "Use Optional for return types that may be empty. \
                 Use map(), flatMap(), filter() for transformations. \
                 Avoid Optional in fields and parameters.",
        ),
        alternatives: vec![
            "Null checks with defensive programming".to_string(),
            "@Nullable and @NonNull annotations".to_string(),
            "Default values with null coalescing".to_string(),
        ],
    }]
}
