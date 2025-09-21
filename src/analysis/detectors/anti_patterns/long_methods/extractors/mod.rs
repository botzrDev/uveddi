//! Extraction modules for method analysis

pub mod match_processor;
pub mod method_extractor;
pub mod refactoring_suggester;

pub use match_processor::MatchProcessor;
pub use method_extractor::MethodExtractor;
pub use refactoring_suggester::RefactoringSuggester;
