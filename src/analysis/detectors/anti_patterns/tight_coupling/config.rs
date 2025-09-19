use super::types::CouplingThresholds;

/// Configuration for tight coupling detection
#[derive(Debug, Clone)]
pub struct TightCouplingConfig {
    pub rust_thresholds: CouplingThresholds,
    pub python_thresholds: CouplingThresholds,
    pub javascript_thresholds: CouplingThresholds,
    pub enable_cross_file_analysis: bool,
    pub exclude_patterns: Vec<String>,
    pub include_test_files: bool,
    pub visualization_enabled: bool,
    pub generate_dependency_graph: bool,
    pub generate_coupling_matrix: bool,
    pub max_components_for_visualization: usize,
}

impl Default for TightCouplingConfig {
    fn default() -> Self {
        Self {
            rust_thresholds: CouplingThresholds {
                fan_out_warning: 3,
                fan_out_critical: 7,
                cbo_warning: 6,
                cbo_critical: 10,
                rfc_warning: 15,
                rfc_critical: 25,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            python_thresholds: CouplingThresholds {
                fan_out_warning: 10,
                fan_out_critical: 15,
                cbo_warning: 8,
                cbo_critical: 12,
                rfc_warning: 18,
                rfc_critical: 30,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            javascript_thresholds: CouplingThresholds {
                fan_out_warning: 12,
                fan_out_critical: 18,
                cbo_warning: 10,
                cbo_critical: 15,
                rfc_warning: 20,
                rfc_critical: 35,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            enable_cross_file_analysis: true,
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            include_test_files: false,
            visualization_enabled: false,
            generate_dependency_graph: false,
            generate_coupling_matrix: false,
            max_components_for_visualization: 100,
        }
    }
}

impl TightCouplingConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable visualization features
    pub fn with_visualization(mut self) -> Self {
        self.visualization_enabled = true;
        self.generate_dependency_graph = true;
        self.generate_coupling_matrix = true;
        self
    }

    /// Configure for strict coupling detection (lower thresholds)
    pub fn with_strict_thresholds(mut self) -> Self {
        // Reduce all thresholds by 30%
        self.rust_thresholds.fan_out_warning = (self.rust_thresholds.fan_out_warning as f64 * 0.7) as usize;
        self.rust_thresholds.fan_out_critical = (self.rust_thresholds.fan_out_critical as f64 * 0.7) as usize;
        self.rust_thresholds.cbo_warning = (self.rust_thresholds.cbo_warning as f64 * 0.7) as usize;
        self.rust_thresholds.cbo_critical = (self.rust_thresholds.cbo_critical as f64 * 0.7) as usize;

        self.python_thresholds.fan_out_warning = (self.python_thresholds.fan_out_warning as f64 * 0.7) as usize;
        self.python_thresholds.fan_out_critical = (self.python_thresholds.fan_out_critical as f64 * 0.7) as usize;
        self.python_thresholds.cbo_warning = (self.python_thresholds.cbo_warning as f64 * 0.7) as usize;
        self.python_thresholds.cbo_critical = (self.python_thresholds.cbo_critical as f64 * 0.7) as usize;

        self.javascript_thresholds.fan_out_warning = (self.javascript_thresholds.fan_out_warning as f64 * 0.7) as usize;
        self.javascript_thresholds.fan_out_critical = (self.javascript_thresholds.fan_out_critical as f64 * 0.7) as usize;
        self.javascript_thresholds.cbo_warning = (self.javascript_thresholds.cbo_warning as f64 * 0.7) as usize;
        self.javascript_thresholds.cbo_critical = (self.javascript_thresholds.cbo_critical as f64 * 0.7) as usize;

        self
    }

    /// Configure for lenient coupling detection (higher thresholds)
    pub fn with_lenient_thresholds(mut self) -> Self {
        // Increase all thresholds by 50%
        self.rust_thresholds.fan_out_warning = (self.rust_thresholds.fan_out_warning as f64 * 1.5) as usize;
        self.rust_thresholds.fan_out_critical = (self.rust_thresholds.fan_out_critical as f64 * 1.5) as usize;
        self.rust_thresholds.cbo_warning = (self.rust_thresholds.cbo_warning as f64 * 1.5) as usize;
        self.rust_thresholds.cbo_critical = (self.rust_thresholds.cbo_critical as f64 * 1.5) as usize;

        self.python_thresholds.fan_out_warning = (self.python_thresholds.fan_out_warning as f64 * 1.5) as usize;
        self.python_thresholds.fan_out_critical = (self.python_thresholds.fan_out_critical as f64 * 1.5) as usize;
        self.python_thresholds.cbo_warning = (self.python_thresholds.cbo_warning as f64 * 1.5) as usize;
        self.python_thresholds.cbo_critical = (self.python_thresholds.cbo_critical as f64 * 1.5) as usize;

        self.javascript_thresholds.fan_out_warning = (self.javascript_thresholds.fan_out_warning as f64 * 1.5) as usize;
        self.javascript_thresholds.fan_out_critical = (self.javascript_thresholds.fan_out_critical as f64 * 1.5) as usize;
        self.javascript_thresholds.cbo_warning = (self.javascript_thresholds.cbo_warning as f64 * 1.5) as usize;
        self.javascript_thresholds.cbo_critical = (self.javascript_thresholds.cbo_critical as f64 * 1.5) as usize;

        self
    }

    /// Include test files in analysis
    pub fn with_test_files(mut self) -> Self {
        self.include_test_files = true;
        self
    }

    /// Disable cross-file analysis for faster processing
    pub fn with_single_file_analysis(mut self) -> Self {
        self.enable_cross_file_analysis = false;
        self
    }

    /// Add custom exclude patterns
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns.extend(patterns);
        self
    }

    /// Set maximum components for visualization
    pub fn with_max_visualization_components(mut self, max_components: usize) -> Self {
        self.max_components_for_visualization = max_components;
        self
    }
}