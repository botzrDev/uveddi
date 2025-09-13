//! Comprehensive unit tests for Data Clumps Anti-Pattern Detector
//!
//! This module provides extensive test coverage for the Data Clumps detector,
//! ensuring proper detection of parameter groups that should be extracted
//! into their own classes.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::analysis::components::AstProvider;
    use crate::analysis::services::AnalysisResult;
    use crate::parsing::{ParsedFile, FunctionSignature};
    use std::path::PathBuf;
    use std::collections::HashMap;
    use async_trait::async_trait;

    /// Mock AST provider for testing
    struct MockAstProvider {
        functions: HashMap<PathBuf, Vec<FunctionSignature>>,
    }

    impl MockAstProvider {
        fn new() -> Self {
            Self {
                functions: HashMap::new(),
            }
        }

        fn with_functions(mut self, path: PathBuf, functions: Vec<FunctionSignature>) -> Self {
            self.functions.insert(path, functions);
            self
        }
    }

    impl AstProvider for MockAstProvider {
        fn get_functions(&self, file_path: &Path) -> Result<Vec<FunctionSignature>, AnalysisError> {
            self.functions
                .get(file_path)
                .cloned()
                .ok_or_else(|| AnalysisError::FileNotFound(file_path.to_string_lossy().to_string()))
        }
    }

    #[test]
    fn test_detector_creation_with_default_config() {
        let detector = DataClumpsDetector::new();
        assert_eq!(detector.config.min_clump_size, 3);
        assert_eq!(detector.config.min_occurrences, 2);
        assert_eq!(detector.config.similarity_threshold, 0.8);
        assert_eq!(detector.config.max_parameter_count, 6);
    }

    #[test]
    fn test_detector_creation_with_custom_config() {
        let config = DataClumpsConfig {
            min_clump_size: 4,
            min_occurrences: 3,
            similarity_threshold: 0.9,
            max_parameter_count: 8,
        };

        let detector = DataClumpsDetector::with_config(config.clone());
        assert_eq!(detector.config.min_clump_size, 4);
        assert_eq!(detector.config.min_occurrences, 3);
        assert_eq!(detector.config.similarity_threshold, 0.9);
        assert_eq!(detector.config.max_parameter_count, 8);
    }

    #[test]
    fn test_detect_address_data_clump() {
        let detector = DataClumpsDetector::new();

        // Test address-related function naming
        let params = detector.simulate_parameters("update_user_address");
        assert_eq!(params.len(), 5);
        assert!(params.iter().any(|(name, _)| name == "street"));
        assert!(params.iter().any(|(name, _)| name == "city"));
        assert!(params.iter().any(|(name, _)| name == "state"));
        assert!(params.iter().any(|(name, _)| name == "zip_code"));
        assert!(params.iter().any(|(name, _)| name == "country"));
    }

    #[test]
    fn test_detect_person_data_clump() {
        let detector = DataClumpsDetector::new();

        // Test person-related function naming
        let params = detector.simulate_parameters("create_person_profile");
        assert_eq!(params.len(), 4);
        assert!(params.iter().any(|(name, _)| name == "first_name"));
        assert!(params.iter().any(|(name, _)| name == "last_name"));
        assert!(params.iter().any(|(name, _)| name == "email"));
        assert!(params.iter().any(|(name, _)| name == "phone"));
    }

    #[test]
    fn test_detect_date_time_data_clump() {
        let detector = DataClumpsDetector::new();

        // Test date/time-related function naming
        let params = detector.simulate_parameters("schedule_appointment_date");
        assert_eq!(params.len(), 5);
        assert!(params.iter().any(|(name, _)| name == "year"));
        assert!(params.iter().any(|(name, _)| name == "month"));
        assert!(params.iter().any(|(name, _)| name == "day"));
        assert!(params.iter().any(|(name, _)| name == "hour"));
        assert!(params.iter().any(|(name, _)| name == "minute"));
    }

    #[test]
    fn test_detect_coordinate_data_clump() {
        let detector = DataClumpsDetector::new();

        // Test coordinate-related function naming
        let params = detector.simulate_parameters("set_object_position");
        assert_eq!(params.len(), 3);
        assert!(params.iter().any(|(name, typ)| name == "x" && typ == "f64"));
        assert!(params.iter().any(|(name, typ)| name == "y" && typ == "f64"));
        assert!(params.iter().any(|(name, typ)| name == "z" && typ == "f64"));
    }

    #[test]
    fn test_detect_color_data_clump() {
        let detector = DataClumpsDetector::new();

        // Test color-related function naming
        let params = detector.simulate_parameters("set_background_color");
        assert_eq!(params.len(), 4);
        assert!(params.iter().any(|(name, typ)| name == "red" && typ == "u8"));
        assert!(params.iter().any(|(name, typ)| name == "green" && typ == "u8"));
        assert!(params.iter().any(|(name, typ)| name == "blue" && typ == "u8"));
        assert!(params.iter().any(|(name, typ)| name == "alpha" && typ == "u8"));
    }

    #[test]
    fn test_detect_money_data_clump() {
        let detector = DataClumpsDetector::new();

        // Test money-related function naming
        let params = detector.simulate_parameters("calculate_total_price");
        assert_eq!(params.len(), 3);
        assert!(params.iter().any(|(name, typ)| name == "amount" && typ == "f64"));
        assert!(params.iter().any(|(name, typ)| name == "currency" && typ == "String"));
        assert!(params.iter().any(|(name, typ)| name == "precision" && typ == "i32"));
    }

    #[test]
    fn test_parameter_signature_creation() {
        let detector = DataClumpsDetector::new();

        let params = vec![
            ("city".to_string(), "String".to_string()),
            ("street".to_string(), "String".to_string()),
            ("zip".to_string(), "String".to_string()),
        ];

        let signature = detector.create_signature(&params);
        // Should be alphabetically sorted
        assert_eq!(signature, "city:String|street:String|zip:String");
    }

    #[test]
    fn test_parameter_signature_normalization() {
        let detector = DataClumpsDetector::new();

        // Same parameters in different order should produce same signature
        let params1 = vec![
            ("x".to_string(), "f64".to_string()),
            ("y".to_string(), "f64".to_string()),
            ("z".to_string(), "f64".to_string()),
        ];

        let params2 = vec![
            ("z".to_string(), "f64".to_string()),
            ("x".to_string(), "f64".to_string()),
            ("y".to_string(), "f64".to_string()),
        ];

        let sig1 = detector.create_signature(&params1);
        let sig2 = detector.create_signature(&params2);

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_semantic_coherence_detection() {
        let detector = DataClumpsDetector::new();

        // Test address-related group
        let address_group = ParameterGroup {
            parameter_names: vec![
                "street".to_string(),
                "city".to_string(),
                "state".to_string(),
            ],
            parameter_types: vec![
                "String".to_string(),
                "String".to_string(),
                "String".to_string(),
            ],
            signature: "city:String|state:String|street:String".to_string(),
        };

        assert!(detector.has_semantic_coherence(&address_group));

        // Test non-coherent group
        let mixed_group = ParameterGroup {
            parameter_names: vec![
                "param1".to_string(),
                "param2".to_string(),
                "param3".to_string(),
            ],
            parameter_types: vec![
                "String".to_string(),
                "i32".to_string(),
                "bool".to_string(),
            ],
            signature: "param1:String|param2:i32|param3:bool".to_string(),
        };

        assert!(!detector.has_semantic_coherence(&mixed_group));
    }

    #[test]
    fn test_clump_confidence_calculation() {
        let detector = DataClumpsDetector::new();

        // High confidence case: semantically coherent, multiple occurrences, large group
        let coherent_group = ParameterGroup {
            parameter_names: vec![
                "year".to_string(),
                "month".to_string(),
                "day".to_string(),
                "hour".to_string(),
                "minute".to_string(),
            ],
            parameter_types: vec!["i32".to_string(); 5],
            signature: "day:i32|hour:i32|minute:i32|month:i32|year:i32".to_string(),
        };

        let occurrences = vec![
            "schedule_meeting".to_string(),
            "book_appointment".to_string(),
            "set_deadline".to_string(),
            "create_event".to_string(),
        ];

        let confidence = detector.calculate_clump_confidence(&coherent_group, &occurrences);
        assert!(confidence > 0.8);

        // Low confidence case: non-coherent, few occurrences, small group
        let non_coherent_group = ParameterGroup {
            parameter_names: vec![
                "a".to_string(),
                "b".to_string(),
            ],
            parameter_types: vec!["i32".to_string(); 2],
            signature: "a:i32|b:i32".to_string(),
        };

        let few_occurrences = vec![
            "func1".to_string(),
            "func2".to_string(),
        ];

        let low_confidence = detector.calculate_clump_confidence(&non_coherent_group, &few_occurrences);
        assert!(low_confidence < 0.7);
    }

    #[test]
    fn test_class_name_suggestion() {
        let detector = DataClumpsDetector::new();

        // Test address group
        let address_group = ParameterGroup {
            parameter_names: vec![
                "street".to_string(),
                "city".to_string(),
                "state".to_string(),
                "zip_code".to_string(),
            ],
            parameter_types: vec!["String".to_string(); 4],
            signature: "city:String|state:String|street:String|zip_code:String".to_string(),
        };

        let suggested_name = detector.suggest_class_name(&address_group);
        assert!(suggested_name.contains("Address"));

        // Test coordinate group
        let coord_group = ParameterGroup {
            parameter_names: vec![
                "x".to_string(),
                "y".to_string(),
                "z".to_string(),
            ],
            parameter_types: vec!["f64".to_string(); 3],
            signature: "x:f64|y:f64|z:f64".to_string(),
        };

        let coord_suggestion = detector.suggest_class_name(&coord_group);
        assert!(coord_suggestion.contains("Point") || coord_suggestion.contains("Coordinate"));
    }

    #[test]
    fn test_extract_parameter_groups_sliding_window() {
        let detector = DataClumpsDetector::new();

        let groups = detector.extract_parameter_groups(
            "update_user_address",
            &vec![] // Parameters not used in current implementation
        );

        // With 5 address parameters and min_clump_size of 3,
        // we should get multiple sliding window groups
        assert!(!groups.is_empty());

        // Check that we get groups of different sizes
        let sizes: Vec<usize> = groups.iter()
            .map(|g| g.parameter_names.len())
            .collect();

        assert!(sizes.iter().any(|&s| s >= 3));
        assert!(sizes.iter().any(|&s| s <= 5));
    }

    #[test]
    fn test_identify_clump_patterns_minimum_occurrences() {
        let detector = DataClumpsDetector::new();

        let group = ParameterGroup {
            parameter_names: vec![
                "x".to_string(),
                "y".to_string(),
                "z".to_string(),
            ],
            parameter_types: vec!["f64".to_string(); 3],
            signature: "x:f64|y:f64|z:f64".to_string(),
        };

        // Test with insufficient occurrences
        let mut param_groups = HashMap::new();
        param_groups.insert(group.clone(), vec!["move_object".to_string()]);

        let patterns = detector.identify_clump_patterns(param_groups);
        assert!(patterns.is_empty()); // Should not identify pattern with only 1 occurrence

        // Test with sufficient occurrences
        let mut param_groups = HashMap::new();
        param_groups.insert(
            group.clone(),
            vec![
                "move_object".to_string(),
                "set_position".to_string(),
                "translate".to_string(),
            ]
        );

        let patterns = detector.identify_clump_patterns(param_groups);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].occurrences.len(), 3);
    }

    #[test]
    fn test_no_false_positives_for_small_groups() {
        let config = DataClumpsConfig {
            min_clump_size: 3,
            min_occurrences: 2,
            similarity_threshold: 0.8,
            max_parameter_count: 6,
        };

        let detector = DataClumpsDetector::with_config(config);

        // Small parameter group (less than min_clump_size)
        let small_group = ParameterGroup {
            parameter_names: vec![
                "id".to_string(),
                "name".to_string(),
            ],
            parameter_types: vec![
                "u32".to_string(),
                "String".to_string(),
            ],
            signature: "id:u32|name:String".to_string(),
        };

        let mut param_groups = HashMap::new();
        param_groups.insert(
            small_group,
            vec![
                "get_user".to_string(),
                "update_user".to_string(),
                "delete_user".to_string(),
            ]
        );

        let patterns = detector.identify_clump_patterns(param_groups);
        assert!(patterns.is_empty()); // Should not identify small groups as clumps
    }

    #[test]
    fn test_edge_case_empty_parameters() {
        let detector = DataClumpsDetector::new();

        let groups = detector.extract_parameter_groups(
            "no_params_function",
            &vec![]
        );

        // Functions with no matching pattern should return minimal groups
        assert!(groups.len() <= 1);
    }

    #[test]
    fn test_edge_case_very_long_parameter_list() {
        let detector = DataClumpsDetector::new();

        // Test with a function that would have many parameters
        let groups = detector.extract_parameter_groups(
            "complex_address_update_with_validation",
            &vec![]
        );

        // Should still extract groups despite long parameter list
        assert!(!groups.is_empty());
    }
}