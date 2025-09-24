//! Test module to verify engine modules compile independently
//! This can be removed after migration is complete.

#[cfg(test)]
mod tests {
    use super::{
        analysis::AnalysisContext,
        cache::{AnalysisCache, AstCache},
        knowledge_graph::{GraphBuilder, KnowledgeGraph, QueryBuilder},
        parsing::{AstBuilder, LanguageParser},
    };

    #[test]
    fn test_engine_modules_compile() {
        // Test that all our new types can be instantiated
        let _ast_cache = AstCache::new(100);
        let _analysis_cache = AnalysisCache::new(50);
        let _graph_builder = GraphBuilder::new();
        let _query_builder = QueryBuilder::new();

        // This test just verifies our types are properly defined
        assert!(true);
    }
}
