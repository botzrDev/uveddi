//! Configuration options for the SQL injection detector.

#[derive(Debug, Clone)]
pub struct SqlInjectionConfig {
    pub enable_query_analysis: bool,
    pub enable_parameter_analysis: bool,
    pub enable_dynamic_query_analysis: bool,
    pub enable_stored_procedure_analysis: bool,
    pub enable_orm_analysis: bool,
}

impl SqlInjectionConfig {
    pub fn with_disabled_parameter_analysis(mut self) -> Self {
        self.enable_parameter_analysis = false;
        self
    }
}

impl Default for SqlInjectionConfig {
    fn default() -> Self {
        Self {
            enable_query_analysis: true,
            enable_parameter_analysis: true,
            enable_dynamic_query_analysis: true,
            enable_stored_procedure_analysis: true,
            enable_orm_analysis: true,
        }
    }
}
