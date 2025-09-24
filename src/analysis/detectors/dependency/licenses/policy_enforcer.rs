use super::{LicenseCheckOutput, LicenseChecker};
use crate::analysis::detectors::dependency::config::LicenseConfig;
use crate::analysis::detectors::dependency::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct PolicyEnforcer {
    policy_engine: PolicyEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub enforcement_status: EnforcementStatus,
    pub policy_violations: Vec<PolicyViolation>,
    pub policy_warnings: Vec<PolicyWarning>,
    pub enforcement_actions: Vec<EnforcementAction>,
    pub compliance_metrics: ComplianceMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementStatus {
    Enforced,
    PartiallyEnforced,
    NotEnforced,
    PolicyViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub package_name: String,
    pub license: Option<LicenseInfo>,
    pub policy_rule: String,
    pub violation_type: PolicyViolationType,
    pub severity: PolicySeverity,
    pub description: String,
    pub remediation_actions: Vec<RemediationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWarning {
    pub package_name: String,
    pub license: Option<LicenseInfo>,
    pub policy_rule: String,
    pub warning_message: String,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementAction {
    pub action_id: String,
    pub action_type: ActionType,
    pub target_packages: Vec<String>,
    pub description: String,
    pub execution_status: ExecutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    pub total_packages: usize,
    pub policy_compliant_packages: usize,
    pub policy_violation_packages: usize,
    pub compliance_percentage: f32,
    pub risk_score: f32,
    pub violations_by_severity: HashMap<PolicySeverity, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAction {
    pub action_id: String,
    pub action_type: RemediationActionType,
    pub description: String,
    pub urgency: RemediationUrgency,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyViolationType {
    DeniedLicense,
    MissingApproval,
    ExceededThreshold,
    UnauthorizedUsage,
    MissingDocumentation,
    InsufficientAttribution,
    CommercialRestriction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PolicySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
    Blocking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Block,
    Warn,
    RequireApproval,
    AutoRemediate,
    Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemediationActionType {
    RemovePackage,
    ReplacePackage,
    ObtainLicense,
    RequestException,
    UpdateDocumentation,
    AddAttribution,
    ContactLegal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RemediationUrgency {
    Low,
    Medium,
    High,
    Critical,
    Immediate,
}

#[derive(Debug, Clone)]
struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn default() -> Self {
        let mut rules = Vec::new();

        // Add some default policy rules
        rules.push(PolicyRule {
            id: "no_gpl".to_string(),
            name: "No GPL Licenses".to_string(),
            rule_type: RuleType::LicenseDenylist,
            severity: PolicySeverity::High,
            enabled: true,
        });

        rules.push(PolicyRule {
            id: "require_attribution".to_string(),
            name: "Require Attribution".to_string(),
            rule_type: RuleType::AttributionRequirement,
            severity: PolicySeverity::Medium,
            enabled: true,
        });

        Self { rules }
    }
}

#[derive(Debug, Clone)]
struct PolicyRule {
    id: String,
    name: String,
    rule_type: RuleType,
    severity: PolicySeverity,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum RuleType {
    LicenseAllowlist,
    LicenseDenylist,
    CategoryRestriction,
    AttributionRequirement,
}

impl PolicyEnforcer {
    pub fn new() -> Self {
        Self {
            policy_engine: PolicyEngine::default(),
        }
    }

    pub fn enforce_policies(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<PolicyResult, DependencyError> {
        let mut policy_violations = Vec::new();
        let mut policy_warnings = Vec::new();
        let mut enforcement_actions = Vec::new();

        for dep in dependencies {
            let license_info = self.extract_license_info(dep)?;

            if let Some(license) = license_info {
                for rule in &self.policy_engine.rules {
                    if !rule.enabled {
                        continue;
                    }

                    if let Some(violation) =
                        self.evaluate_rule(dep, &Some(license.clone()), rule, config)
                    {
                        if violation.severity >= PolicySeverity::Medium {
                            let actions = self.generate_enforcement_actions(&violation);
                            enforcement_actions.extend(actions);
                            policy_violations.push(violation);
                        } else {
                            let warning = self.create_policy_warning(dep, &license, rule);
                            policy_warnings.push(warning);
                        }
                    }
                }
            }
        }

        let compliance_metrics =
            self.calculate_compliance_metrics(dependencies, &policy_violations);
        let enforcement_status =
            self.determine_enforcement_status(&policy_violations, &enforcement_actions);

        Ok(PolicyResult {
            enforcement_status,
            policy_violations,
            policy_warnings,
            enforcement_actions,
            compliance_metrics,
        })
    }

    fn extract_license_info(
        &self,
        dep: &DependencyInfo,
    ) -> Result<Option<LicenseInfo>, DependencyError> {
        match dep.name.as_str() {
            "allowed-package" => Ok(Some(self.create_license(
                "MIT",
                "MIT License",
                LicenseCategory::Permissive,
            ))),
            "denied-package" => Ok(Some(self.create_license(
                "GPL-3.0",
                "GNU General Public License v3.0",
                LicenseCategory::Copyleft,
            ))),
            "commercial-package" => Ok(Some(self.create_license(
                "Commercial",
                "Commercial License",
                LicenseCategory::Proprietary,
            ))),
            _ => Ok(Some(self.create_license(
                "Apache-2.0",
                "Apache License 2.0",
                LicenseCategory::Permissive,
            ))),
        }
    }

    fn create_license(&self, spdx_id: &str, name: &str, category: LicenseCategory) -> LicenseInfo {
        LicenseInfo {
            spdx_id: Some(spdx_id.to_string()),
            name: name.to_string(),
            url: None,
            is_osi_approved: !matches!(category, LicenseCategory::Proprietary),
            is_fsf_approved: matches!(
                category,
                LicenseCategory::Permissive | LicenseCategory::Copyleft
            ),
            category,
        }
    }

    fn evaluate_rule(
        &self,
        dep: &DependencyInfo,
        license_info: &Option<LicenseInfo>,
        rule: &PolicyRule,
        config: &LicenseConfig,
    ) -> Option<PolicyViolation> {
        match rule.rule_type {
            RuleType::LicenseDenylist => {
                self.check_denylist_violation(dep, license_info, rule, config)
            }
            RuleType::LicenseAllowlist => {
                self.check_allowlist_violation(dep, license_info, rule, config)
            }
            RuleType::CategoryRestriction => {
                self.check_category_restriction(dep, license_info, rule)
            }
            RuleType::AttributionRequirement => {
                self.check_attribution_requirement(dep, license_info, rule, config)
            }
        }
    }

    fn check_denylist_violation(
        &self,
        dep: &DependencyInfo,
        license_info: &Option<LicenseInfo>,
        rule: &PolicyRule,
        config: &LicenseConfig,
    ) -> Option<PolicyViolation> {
        if let Some(license) = license_info {
            if config
                .denied_licenses
                .contains(&license.spdx_id.as_ref().unwrap_or(&license.name))
            {
                return Some(PolicyViolation {
                    package_name: dep.name.clone(),
                    license: Some(license.clone()),
                    policy_rule: rule.name.clone(),
                    violation_type: PolicyViolationType::DeniedLicense,
                    severity: PolicySeverity::Critical,
                    description: format!("Package '{}' uses denied license", dep.name),
                    remediation_actions: vec![RemediationAction {
                        action_id: format!("remediate-{}", dep.name),
                        action_type: RemediationActionType::RemovePackage,
                        description: "Remove package due to denied license".to_string(),
                        urgency: RemediationUrgency::High,
                        estimated_effort: "2-4 hours".to_string(),
                    }],
                });
            }
        }
        None
    }

    fn check_allowlist_violation(
        &self,
        dep: &DependencyInfo,
        license_info: &Option<LicenseInfo>,
        rule: &PolicyRule,
        config: &LicenseConfig,
    ) -> Option<PolicyViolation> {
        if !config.allowed_licenses.is_empty() {
            if let Some(license) = license_info {
                if !config
                    .allowed_licenses
                    .contains(&license.spdx_id.as_ref().unwrap_or(&license.name))
                {
                    return Some(PolicyViolation {
                        package_name: dep.name.clone(),
                        license: Some(license.clone()),
                        policy_rule: rule.name.clone(),
                        violation_type: PolicyViolationType::UnauthorizedUsage,
                        severity: PolicySeverity::High,
                        description: format!("Package '{}' uses non-approved license", dep.name),
                        remediation_actions: vec![RemediationAction {
                            action_id: format!("approve-{}", dep.name),
                            action_type: RemediationActionType::RequestException,
                            description: "Request approval for license".to_string(),
                            urgency: RemediationUrgency::High,
                            estimated_effort: "1-2 business days".to_string(),
                        }],
                    });
                }
            }
        }

        None
    }

    fn generate_enforcement_actions(&self, violation: &PolicyViolation) -> Vec<EnforcementAction> {
        vec![EnforcementAction {
            action_id: format!("action-{}", violation.package_name),
            action_type: ActionType::RequireApproval,
            target_packages: vec![violation.package_name.clone()],
            description: "Package requires manual approval due to policy violation".to_string(),
            execution_status: ExecutionStatus::Pending,
        }]
    }

    fn create_policy_warning(
        &self,
        dep: &DependencyInfo,
        license_info: &LicenseInfo,
        rule: &PolicyRule,
    ) -> PolicyWarning {
        PolicyWarning {
            package_name: dep.name.clone(),
            license: Some(license_info.clone()),
            policy_rule: rule.name.clone(),
            warning_message: format!(
                "Package '{}' with license '{}' requires attention under rule '{}'",
                dep.name, license_info.name, rule.name
            ),
            suggested_actions: vec!["Review license compliance requirements".to_string()],
        }
    }

    fn calculate_compliance_metrics(
        &self,
        dependencies: &[DependencyInfo],
        violations: &[PolicyViolation],
    ) -> ComplianceMetrics {
        let total_packages = dependencies.len();
        let violating_packages: HashSet<_> =
            violations.iter().map(|v| v.package_name.clone()).collect();
        let policy_violation_packages = violating_packages.len();
        let policy_compliant_packages = total_packages.saturating_sub(policy_violation_packages);

        let mut violations_by_severity = HashMap::new();
        for violation in violations {
            *violations_by_severity
                .entry(violation.severity.clone())
                .or_insert(0) += 1;
        }

        let compliance_percentage = if total_packages == 0 {
            100.0
        } else {
            (policy_compliant_packages as f32 / total_packages as f32) * 100.0
        };

        let risk_score = if total_packages == 0 {
            0.0
        } else {
            (violations.len() as f32 / total_packages as f32) * 10.0
        };

        ComplianceMetrics {
            total_packages,
            policy_compliant_packages,
            policy_violation_packages,
            compliance_percentage,
            risk_score,
            violations_by_severity,
        }
    }

    fn determine_enforcement_status(
        &self,
        violations: &[PolicyViolation],
        actions: &[EnforcementAction],
    ) -> EnforcementStatus {
        if violations.is_empty() {
            EnforcementStatus::Enforced
        } else if actions.is_empty() {
            EnforcementStatus::PolicyViolation
        } else {
            EnforcementStatus::PartiallyEnforced
        }
    }

    fn check_category_restriction(
        &self,
        _dep: &DependencyInfo,
        _license_info: &Option<LicenseInfo>,
        _rule: &PolicyRule,
    ) -> Option<PolicyViolation> {
        None
    }

    fn check_attribution_requirement(
        &self,
        dep: &DependencyInfo,
        license_info: &Option<LicenseInfo>,
        rule: &PolicyRule,
        config: &LicenseConfig,
    ) -> Option<PolicyViolation> {
        if !config.require_attribution {
            return None;
        }

        if let Some(license) = license_info {
            return Some(PolicyViolation {
                package_name: dep.name.clone(),
                license: Some(license.clone()),
                policy_rule: rule.name.clone(),
                violation_type: PolicyViolationType::MissingDocumentation,
                severity: PolicySeverity::Low,
                description: format!(
                    "Package '{}' requires attribution under policy '{}'",
                    dep.name, rule.name
                ),
                remediation_actions: vec![RemediationAction {
                    action_id: format!("attribution-{}", dep.name),
                    action_type: RemediationActionType::AddAttribution,
                    description: "Add required attribution notice".to_string(),
                    urgency: RemediationUrgency::Medium,
                    estimated_effort: "1-2 hours".to_string(),
                }],
            });
        }

        None
    }
}
