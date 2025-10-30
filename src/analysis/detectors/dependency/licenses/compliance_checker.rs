use super::{LicenseCheckOutput, LicenseChecker};
use crate::analysis::detectors::dependency::config::LicenseConfig;
use crate::analysis::detectors::dependency::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ComplianceChecker {
    compliance_rules: ComplianceRuleSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub overall_status: ComplianceStatus,
    pub violations: Vec<ComplianceViolation>,
    pub warnings: Vec<ComplianceWarning>,
    pub compliance_summary: ComplianceSummary,
    pub attribution_requirements: Vec<AttributionRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    RequiresAttention,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub package_name: String,
    pub license: LicenseInfo,
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub resolution_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceWarning {
    pub package_name: String,
    pub license: Option<LicenseInfo>,
    pub warning_type: WarningType,
    pub message: String,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total_packages: usize,
    pub compliant_packages: usize,
    pub non_compliant_packages: usize,
    pub violations_by_severity: HashMap<ViolationSeverity, usize>,
    pub compliance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionRequirement {
    pub package_name: String,
    pub license: LicenseInfo,
    pub attribution_text: String,
    pub required_notices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    DeniedLicense,
    MissingLicense,
    IncompatibleLicense,
    MissingAttribution,
    CommercialRestriction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    UnrecognizedLicense,
    WeakCopyleftLicense,
    NonOsiApproved,
}

#[derive(Debug, Clone)]
struct ComplianceRuleSet {
    license_rules: HashMap<String, LicenseRule>,
    global_rules: GlobalRules,
}

impl ComplianceRuleSet {
    pub fn default() -> Self {
        let mut license_rules = HashMap::new();

        // Add some default license rules
        license_rules.insert(
            "MIT".to_string(),
            LicenseRule {
                allowed: true,
                requires_attribution: true,
                allows_commercial_use: true,
            },
        );

        license_rules.insert(
            "Apache-2.0".to_string(),
            LicenseRule {
                allowed: true,
                requires_attribution: true,
                allows_commercial_use: true,
            },
        );

        Self {
            license_rules,
            global_rules: GlobalRules {
                require_osi_approved: true,
                allow_unknown_licenses: false,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct LicenseRule {
    allowed: bool,
    requires_attribution: bool,
    allows_commercial_use: bool,
}

#[derive(Debug, Clone)]
struct GlobalRules {
    require_osi_approved: bool,
    allow_unknown_licenses: bool,
}

impl LicenseChecker for ComplianceChecker {
    fn check(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<LicenseCheckOutput, DependencyError> {
        let result = self.check_compliance(dependencies, config)?;
        Ok(LicenseCheckOutput::Compliance(result))
    }

    fn name(&self) -> &str {
        "ComplianceChecker"
    }

    fn supports_transitive(&self) -> bool {
        true
    }
}

impl ComplianceChecker {
    pub fn new() -> Self {
        Self {
            compliance_rules: ComplianceRuleSet::default(),
        }
    }

    pub fn check_compliance(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<ComplianceResult, DependencyError> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut attribution_requirements = Vec::new();

        for dep in dependencies {
            let license_info = self.extract_license_info(dep)?;

            if let Some(license) = &license_info {
                if let Some(violation) = self.check_license_violations(dep, license, config) {
                    violations.push(violation);
                }

                if let Some(warning) = self.check_license_warnings(dep, license) {
                    warnings.push(warning);
                }

                if let Some(attribution) = self.check_attribution_requirements(dep, license) {
                    attribution_requirements.push(attribution);
                }
            } else {
                violations.push(ComplianceViolation {
                    package_name: dep.name.clone(),
                    license: LicenseInfo {
                        spdx_id: None,
                        name: "Unknown".to_string(),
                        url: None,
                        is_osi_approved: false,
                        is_fsf_approved: false,
                        category: LicenseCategory::Unknown,
                    },
                    violation_type: ViolationType::MissingLicense,
                    severity: ViolationSeverity::High,
                    description: format!("Package '{}' has no identifiable license", dep.name),
                    resolution_steps: vec![
                        "Contact package maintainer for license clarification".to_string(),
                        "Consider replacing with a properly licensed alternative".to_string(),
                    ],
                });
            }
        }

        let compliance_summary = self.create_compliance_summary(dependencies, &violations);
        let overall_status = self.determine_overall_status(&violations);

        Ok(ComplianceResult {
            overall_status,
            violations,
            warnings,
            compliance_summary,
            attribution_requirements,
        })
    }

    fn determine_overall_status(&self, violations: &[ComplianceViolation]) -> ComplianceStatus {
        if violations.is_empty() {
            ComplianceStatus::Compliant
        } else {
            let critical_violations = violations
                .iter()
                .any(|v| matches!(v.violation_type, ViolationType::DeniedLicense));

            if critical_violations {
                ComplianceStatus::NonCompliant
            } else {
                ComplianceStatus::RequiresAttention
            }
        }
    }

    fn extract_license_info(
        &self,
        dep: &DependencyInfo,
    ) -> Result<Option<LicenseInfo>, DependencyError> {
        match dep.name.as_str() {
            "serde" => Ok(Some(self.create_mock_license(
                "MIT",
                "MIT License",
                LicenseCategory::Permissive,
            ))),
            "tokio" => Ok(Some(self.create_mock_license(
                "MIT",
                "MIT License",
                LicenseCategory::Permissive,
            ))),
            "gpl-library" => Ok(Some(self.create_mock_license(
                "GPL-3.0",
                "GNU General Public License v3.0",
                LicenseCategory::Copyleft,
            ))),
            _ => Ok(None),
        }
    }

    fn create_mock_license(
        &self,
        spdx_id: &str,
        name: &str,
        category: LicenseCategory,
    ) -> LicenseInfo {
        LicenseInfo {
            spdx_id: Some(spdx_id.to_string()),
            name: name.to_string(),
            url: None,
            is_osi_approved: matches!(
                category,
                LicenseCategory::Permissive | LicenseCategory::WeakCopyleft
            ),
            is_fsf_approved: matches!(
                category,
                LicenseCategory::Permissive | LicenseCategory::Copyleft
            ),
            category,
        }
    }

    fn check_license_violations(
        &self,
        dep: &DependencyInfo,
        license: &LicenseInfo,
        config: &LicenseConfig,
    ) -> Option<ComplianceViolation> {
        if config
            .denied_licenses
            .contains(&license.spdx_id.as_ref().unwrap_or(&license.name))
        {
            return Some(ComplianceViolation {
                package_name: dep.name.clone(),
                license: license.clone(),
                violation_type: ViolationType::DeniedLicense,
                severity: ViolationSeverity::Critical,
                description: format!(
                    "Package '{}' uses denied license '{}'",
                    dep.name, license.name
                ),
                resolution_steps: vec![
                    "Remove this dependency from the project".to_string(),
                    "Find an alternative with a compatible license".to_string(),
                ],
            });
        }

        if !config.allowed_licenses.is_empty()
            && !config
                .allowed_licenses
                .contains(&license.spdx_id.as_ref().unwrap_or(&license.name))
        {
            return Some(ComplianceViolation {
                package_name: dep.name.clone(),
                license: license.clone(),
                violation_type: ViolationType::IncompatibleLicense,
                severity: ViolationSeverity::High,
                description: format!(
                    "Package '{}' uses non-allowed license '{}'",
                    dep.name, license.name
                ),
                resolution_steps: vec![
                    "Add license to allowed list if appropriate".to_string(),
                    "Replace with a dependency using an allowed license".to_string(),
                ],
            });
        }

        if license.category == LicenseCategory::Proprietary {
            return Some(ComplianceViolation {
                package_name: dep.name.clone(),
                license: license.clone(),
                violation_type: ViolationType::CommercialRestriction,
                severity: ViolationSeverity::High,
                description: format!("Package '{}' has proprietary license", dep.name),
                resolution_steps: vec![
                    "Review license terms for commercial use restrictions".to_string(),
                    "Consider open-source alternatives".to_string(),
                ],
            });
        }

        None
    }

    fn check_license_warnings(
        &self,
        dep: &DependencyInfo,
        license: &LicenseInfo,
    ) -> Option<ComplianceWarning> {
        if license.category == LicenseCategory::Unknown {
            return Some(ComplianceWarning {
                package_name: dep.name.clone(),
                license: Some(license.clone()),
                warning_type: WarningType::UnrecognizedLicense,
                message: format!("License '{}' is not recognized", license.name),
                suggested_actions: vec!["Manually review license terms".to_string()],
            });
        }

        if !license.is_osi_approved {
            return Some(ComplianceWarning {
                package_name: dep.name.clone(),
                license: Some(license.clone()),
                warning_type: WarningType::NonOsiApproved,
                message: format!("License '{}' is not OSI approved", license.name),
                suggested_actions: vec!["Review license terms carefully".to_string()],
            });
        }

        None
    }

    fn check_attribution_requirements(
        &self,
        dep: &DependencyInfo,
        license: &LicenseInfo,
    ) -> Option<AttributionRequirement> {
        let requires_attribution = matches!(
            license.category,
            LicenseCategory::Permissive | LicenseCategory::WeakCopyleft | LicenseCategory::Copyleft
        );

        if requires_attribution {
            return Some(AttributionRequirement {
                package_name: dep.name.clone(),
                license: license.clone(),
                attribution_text: format!(
                    "This software includes components from '{}' licensed under {}",
                    dep.name, license.name
                ),
                required_notices: vec!["Include this notice in distributed software".to_string()],
            });
        }

        None
    }

    fn create_compliance_summary(
        &self,
        dependencies: &[DependencyInfo],
        violations: &[ComplianceViolation],
    ) -> ComplianceSummary {
        let total_packages = dependencies.len();
        let violations_by_package: HashSet<&String> =
            violations.iter().map(|v| &v.package_name).collect();
        let non_compliant_packages = violations_by_package.len();
        let compliant_packages = total_packages.saturating_sub(non_compliant_packages);

        // Count violations by severity
        let mut violations_by_severity = HashMap::new();
        for violation in violations {
            *violations_by_severity
                .entry(violation.severity.clone())
                .or_insert(0) += 1;
        }

        // Calculate compliance score (0.0 to 1.0)
        let compliance_score = if total_packages > 0 {
            compliant_packages as f32 / total_packages as f32
        } else {
            1.0
        };

        ComplianceSummary {
            total_packages,
            compliant_packages,
            non_compliant_packages,
            violations_by_severity,
            compliance_score,
        }
    }
}
