//! Python-specific SQL injection pattern aggregation.

use crate::analysis::detectors::security::sql_injection::patterns::{
    blind_injection, injection_patterns, time_based, union_attacks,
};
use crate::analysis::detectors::security::sql_injection::types::LanguagePatternSet;

pub fn patterns() -> LanguagePatternSet {
    let mut set = LanguagePatternSet::default();

    set.query_patterns = injection_patterns::python_query_patterns();
    set.parameter_patterns = blind_injection::boolean_based_patterns();
    set.dynamic_query_patterns = union_attacks::common_union_patterns();
    set.dynamic_query_patterns
        .extend(time_based::time_delay_patterns());
    set.orm_patterns = injection_patterns::python_nosql_patterns();

    set
}
