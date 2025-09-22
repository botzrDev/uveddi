//! JavaScript/TypeScript-specific SQL injection pattern aggregation.

use crate::analysis::detectors::security::sql_injection::patterns::{
    blind_injection, error_based, injection_patterns, time_based, union_attacks,
};
use crate::analysis::detectors::security::sql_injection::types::LanguagePatternSet;

pub fn patterns() -> LanguagePatternSet {
    let mut set = LanguagePatternSet::default();

    set.query_patterns = injection_patterns::javascript_query_patterns();
    set.parameter_patterns = blind_injection::boolean_based_patterns();
    set.dynamic_query_patterns = union_attacks::common_union_patterns();
    set.dynamic_query_patterns
        .extend(time_based::time_delay_patterns());
    set.stored_procedure_patterns = error_based::error_probe_patterns();
    set.orm_patterns = injection_patterns::javascript_nosql_patterns();

    set
}
