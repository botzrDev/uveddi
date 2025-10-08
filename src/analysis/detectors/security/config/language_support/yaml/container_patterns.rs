//! Container security patterns for YAML/JSON configuration
//!
//! This module contains security patterns specific to container
//! configurations (Docker, Kubernetes) in YAML and JSON files.

use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use super::kubernetes_patterns::KubernetesPatternChecker;
use serde_yaml::Value as YamlValue;

/// Container security pattern checker
pub struct ContainerPatternChecker;

impl ContainerPatternChecker {
    /// Check for container security issues
    pub fn check_container_security(value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        Self::check_container_recursive(value, &mut issues);
        issues
    }

    fn check_container_recursive(value: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Check for Kubernetes securityContext
                if let Some(security_context) =
                    map.get(&YamlValue::String("securityContext".to_string()))
                {
                    KubernetesPatternChecker::check_k8s_security_context(security_context, issues);
                }

                // Check for Docker privileged mode
                if let Some(privileged) = map.get(&YamlValue::String("privileged".to_string())) {
                    if privileged.as_bool() == Some(true) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::Critical,
                                "Privileged Container",
                                "Container running in privileged mode",
                                None,
                                "Remove privileged mode and use specific capabilities instead",
                                vec!["container".to_string(), "privilege-escalation".to_string()],
                            )
                            .with_cwe(250),
                        );
                    }
                }

                // Check for host network mode
                if let Some(host_network) = map.get(&YamlValue::String("hostNetwork".to_string())) {
                    if host_network.as_bool() == Some(true) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::High,
                                "Host Network Mode",
                                "Container using host network mode",
                                None,
                                "Use container networking instead of host network",
                                vec!["container".to_string(), "network".to_string()],
                            )
                            .with_cwe(250),
                        );
                    }
                }

                // Check for host PID mode
                if let Some(host_pid) = map.get(&YamlValue::String("hostPID".to_string())) {
                    if host_pid.as_bool() == Some(true) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::High,
                                "Host PID Mode",
                                "Container sharing host PID namespace",
                                None,
                                "Use isolated PID namespace instead of host PID",
                                vec!["container".to_string(), "isolation".to_string()],
                            )
                            .with_cwe(250),
                        );
                    }
                }

                // Check for host IPC mode
                if let Some(host_ipc) = map.get(&YamlValue::String("hostIPC".to_string())) {
                    if host_ipc.as_bool() == Some(true) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::Medium,
                                "Host IPC Mode",
                                "Container sharing host IPC namespace",
                                None,
                                "Use isolated IPC namespace instead of host IPC",
                                vec!["container".to_string(), "isolation".to_string()],
                            )
                            .with_cwe(250),
                        );
                    }
                }

                // Check for volume mounts
                if let Some(volumes) = map.get(&YamlValue::String("volumes".to_string())) {
                    KubernetesPatternChecker::check_volume_mounts(volumes, issues);
                }

                // Check for capabilities
                if let Some(capabilities) = map.get(&YamlValue::String("capabilities".to_string()))
                {
                    KubernetesPatternChecker::check_capabilities(capabilities, issues);
                }

                // Check for resource limits
                KubernetesPatternChecker::check_resource_limits(map, issues);

                // Recurse into nested mappings
                for (_, val) in map {
                    Self::check_container_recursive(val, issues);
                }
            }
            YamlValue::Sequence(seq) => {
                for item in seq {
                    Self::check_container_recursive(item, issues);
                }
            }
            _ => {}
        }
    }

    // Kubernetes-specific checks are handled by KubernetesPatternChecker
}

#[cfg(test)]
mod tests {
    use crate::analysis::detectors::security::config::language_support::yaml::YamlParser;
    use super::*;

    #[test]
    fn test_privileged_container_detection() {
        let yaml_content = r#"
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: test
    privileged: true
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = ContainerPatternChecker::check_container_security(&parsed);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Privileged Container")));
    }

    #[test]
    fn test_host_network_detection() {
        let yaml_content = r#"
apiVersion: v1
kind: Pod
spec:
  hostNetwork: true
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = ContainerPatternChecker::check_container_security(&parsed);
        assert!(issues.iter().any(|i| i.title.contains("Host Network")));
    }
}
