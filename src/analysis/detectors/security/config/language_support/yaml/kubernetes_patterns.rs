//! Kubernetes security patterns for YAML configuration
//!
//! This module contains security patterns specific to Kubernetes
//! configurations including security contexts, capabilities, and resource limits.

use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use serde_yaml::Value as YamlValue;

/// Kubernetes security pattern checker
pub struct KubernetesPatternChecker;

impl KubernetesPatternChecker {
    /// Check Kubernetes security context configuration
    pub fn check_k8s_security_context(security_context: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Mapping(map) = security_context {
            // Check for runAsUser: 0 (root)
            if let Some(run_as_user) = map.get(&YamlValue::String("runAsUser".to_string())) {
                if run_as_user.as_u64() == Some(0) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::High,
                        "Container Running as Root",
                        "Container configured to run as root user (UID 0)",
                        None,
                        "Use a non-root user for container execution",
                        vec!["kubernetes".to_string(), "security-context".to_string()],
                    ).with_cwe(250));
                }
            }

            // Check for allowPrivilegeEscalation: true
            if let Some(allow_priv_esc) = map.get(&YamlValue::String("allowPrivilegeEscalation".to_string())) {
                if allow_priv_esc.as_bool() == Some(true) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::High,
                        "Privilege Escalation Allowed",
                        "Container allows privilege escalation",
                        None,
                        "Set allowPrivilegeEscalation to false",
                        vec!["kubernetes".to_string(), "privilege-escalation".to_string()],
                    ).with_cwe(250));
                }
            }

            // Check for runAsNonRoot: false
            if let Some(run_as_non_root) = map.get(&YamlValue::String("runAsNonRoot".to_string())) {
                if run_as_non_root.as_bool() == Some(false) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Medium,
                        "Container May Run as Root",
                        "Container not explicitly required to run as non-root",
                        None,
                        "Set runAsNonRoot to true to enforce non-root execution",
                        vec!["kubernetes".to_string(), "security-context".to_string()],
                    ).with_cwe(250));
                }
            }

            // Check for readOnlyRootFilesystem: false
            if let Some(read_only_fs) = map.get(&YamlValue::String("readOnlyRootFilesystem".to_string())) {
                if read_only_fs.as_bool() == Some(false) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Low,
                        "Writable Root Filesystem",
                        "Container root filesystem is writable",
                        None,
                        "Set readOnlyRootFilesystem to true where possible",
                        vec!["kubernetes".to_string(), "filesystem".to_string()],
                    ).with_cwe(732));
                }
            }
        }
    }

    /// Check volume mounts for security issues
    pub fn check_volume_mounts(volumes: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Sequence(volume_list) = volumes {
            for volume in volume_list {
                if let YamlValue::Mapping(volume_map) = volume {
                    // Check for sensitive host paths
                    if let Some(host_path) = volume_map.get(&YamlValue::String("hostPath".to_string())) {
                        if let YamlValue::Mapping(host_path_map) = host_path {
                            if let Some(path) = host_path_map.get(&YamlValue::String("path".to_string())) {
                                if let Some(path_str) = path.as_str() {
                                    let sensitive_paths = ["/", "/etc", "/proc", "/sys", "/var/run/docker.sock"];
                                    if sensitive_paths.iter().any(|&sensitive| path_str.starts_with(sensitive)) {
                                        issues.push(utils::create_config_issue(
                                            ConfigSeverity::Critical,
                                            "Sensitive Host Path Mount",
                                            format!("Container mounting sensitive host path: {}", path_str),
                                            None,
                                            "Avoid mounting sensitive host paths or use read-only mounts",
                                            vec!["container".to_string(), "volume".to_string(), "host-path".to_string()],
                                        ).with_cwe(250));
                                    }
                                }
                            }
                        }
                    }

                    // Check for writable volume mounts of sensitive paths
                    if let Some(mount_path) = volume_map.get(&YamlValue::String("mountPath".to_string())) {
                        if let Some(mount_path_str) = mount_path.as_str() {
                            let read_only = volume_map.get(&YamlValue::String("readOnly".to_string()))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            if !read_only && (mount_path_str.starts_with("/etc") || mount_path_str.starts_with("/proc")) {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::Medium,
                                    "Writable Sensitive Mount",
                                    format!("Writable mount of sensitive path: {}", mount_path_str),
                                    None,
                                    "Make sensitive path mounts read-only",
                                    vec!["container".to_string(), "volume".to_string()],
                                ).with_cwe(732));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check container capabilities
    pub fn check_capabilities(capabilities: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Mapping(cap_map) = capabilities {
            // Check for dangerous capabilities being added
            if let Some(add_caps) = cap_map.get(&YamlValue::String("add".to_string())) {
                if let YamlValue::Sequence(cap_list) = add_caps {
                    let dangerous_caps = ["SYS_ADMIN", "NET_ADMIN", "SYS_PTRACE", "SYS_MODULE", "DAC_OVERRIDE"];

                    for cap in cap_list {
                        if let Some(cap_str) = cap.as_str() {
                            if dangerous_caps.contains(&cap_str) {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::High,
                                    "Dangerous Capability Added",
                                    format!("Container has dangerous capability: {}", cap_str),
                                    None,
                                    "Remove dangerous capabilities or use more specific alternatives",
                                    vec!["container".to_string(), "capabilities".to_string()],
                                ).with_cwe(250));
                            }
                        }
                    }
                }
            }

            // Check if ALL capabilities are being dropped
            if let Some(drop_caps) = cap_map.get(&YamlValue::String("drop".to_string())) {
                if let YamlValue::Sequence(drop_list) = drop_caps {
                    let has_all = drop_list.iter().any(|cap| {
                        cap.as_str() == Some("ALL")
                    });

                    if !has_all {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::Low,
                            "Capabilities Not Dropped",
                            "Container not dropping all capabilities by default",
                            None,
                            "Consider dropping ALL capabilities and adding only required ones",
                            vec!["container".to_string(), "capabilities".to_string()],
                        ));
                    }
                }
            }
        }
    }

    /// Check resource limits configuration
    pub fn check_resource_limits(map: &serde_yaml::Mapping, issues: &mut Vec<ConfigIssue>) {
        // Look for resources section
        if let Some(resources) = map.get(&YamlValue::String("resources".to_string())) {
            if let YamlValue::Mapping(resources_map) = resources {
                // Check for missing limits
                let has_limits = resources_map.contains_key(&YamlValue::String("limits".to_string()));
                let has_requests = resources_map.contains_key(&YamlValue::String("requests".to_string()));

                if !has_limits {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Medium,
                        "Missing Resource Limits",
                        "Container has no resource limits defined",
                        None,
                        "Set CPU and memory limits to prevent resource exhaustion",
                        vec!["container".to_string(), "resources".to_string()],
                    ).with_cwe(770));
                }

                if !has_requests {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Low,
                        "Missing Resource Requests",
                        "Container has no resource requests defined",
                        None,
                        "Set CPU and memory requests for proper scheduling",
                        vec!["container".to_string(), "resources".to_string()],
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_user_detection() {
        let mut issues = Vec::new();
        let security_context = serde_yaml::from_str(r#"
runAsUser: 0
allowPrivilegeEscalation: true
"#).unwrap();

        KubernetesPatternChecker::check_k8s_security_context(&security_context, &mut issues);
        assert!(issues.iter().any(|i| i.title.contains("Running as Root")));
        assert!(issues.iter().any(|i| i.title.contains("Privilege Escalation")));
    }

    #[test]
    fn test_sensitive_host_path_detection() {
        let mut issues = Vec::new();
        let volumes = serde_yaml::from_str(r#"
- name: docker-sock
  hostPath:
    path: /var/run/docker.sock
"#).unwrap();

        KubernetesPatternChecker::check_volume_mounts(&volumes, &mut issues);
        assert!(issues.iter().any(|i| i.title.contains("Sensitive Host Path")));
    }

    #[test]
    fn test_dangerous_capability_detection() {
        let mut issues = Vec::new();
        let capabilities = serde_yaml::from_str(r#"
add:
- SYS_ADMIN
- NET_ADMIN
"#).unwrap();

        KubernetesPatternChecker::check_capabilities(&capabilities, &mut issues);
        assert!(issues.iter().any(|i| i.title.contains("Dangerous Capability")));
    }
}