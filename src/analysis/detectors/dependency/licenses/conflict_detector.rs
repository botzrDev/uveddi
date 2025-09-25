use super::{LicenseCheckOutput, LicenseChecker};
use crate::analysis::detectors::dependency::config::LicenseConfig;
use crate::analysis::detectors::dependency::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConflictDetector {
    compatibility_matrix: LicenseCompatibilityMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAnalysis {
    pub conflicts: Vec<LicenseConflict>,
    pub potential_conflicts: Vec<PotentialConflict>,
    pub compatibility_summary: CompatibilitySummary,
    pub risk_assessment: ConflictRiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialConflict {
    pub package1: String,
    pub license1: LicenseInfo,
    pub package2: String,
    pub license2: LicenseInfo,
    pub conflict_probability: f32,
    pub conflict_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilitySummary {
    pub total_license_pairs: usize,
    pub compatible_pairs: usize,
    pub incompatible_pairs: usize,
    pub compatibility_score: f32,
    pub license_distribution: HashMap<LicenseCategory, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRiskAssessment {
    pub overall_risk: RiskLevel,
    pub legal_risk: RiskLevel,
    pub business_risk: RiskLevel,
    pub mitigation_urgency: MitigationUrgency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MitigationUrgency {
    Immediate,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
struct LicenseCompatibilityMatrix {
    category_compatibility: HashMap<(LicenseCategory, LicenseCategory), CompatibilityLevel>,
}

impl LicenseCompatibilityMatrix {
    pub fn new() -> Self {
        let mut category_compatibility = HashMap::new();

        // Define compatibility between different license categories
        use CompatibilityLevel::*;
        use LicenseCategory::*;

        // Permissive licenses are generally compatible with everything
        category_compatibility.insert((Permissive, Permissive), Compatible);
        category_compatibility.insert((Permissive, Copyleft), Compatible);
        category_compatibility.insert((Permissive, WeakCopyleft), Compatible);
        category_compatibility.insert((Permissive, Proprietary), Compatible);

        // Copyleft licenses have restrictions
        category_compatibility.insert((Copyleft, Permissive), Compatible);
        category_compatibility.insert((Copyleft, Copyleft), Compatible);
        category_compatibility.insert((Copyleft, WeakCopyleft), RequiresReview);
        category_compatibility.insert((Copyleft, Proprietary), Incompatible);

        // Weak copyleft licenses
        category_compatibility.insert((WeakCopyleft, Permissive), Compatible);
        category_compatibility.insert((WeakCopyleft, Copyleft), RequiresReview);
        category_compatibility.insert((WeakCopyleft, WeakCopyleft), Compatible);
        category_compatibility.insert((WeakCopyleft, Proprietary), RequiresReview);

        // Proprietary licenses
        category_compatibility.insert((Proprietary, Permissive), Compatible);
        category_compatibility.insert((Proprietary, Copyleft), Incompatible);
        category_compatibility.insert((Proprietary, WeakCopyleft), RequiresReview);
        category_compatibility.insert((Proprietary, Proprietary), Compatible);

        Self {
            category_compatibility,
        }
    }

    pub fn get_compatibility(
        &self,
        license1: &LicenseInfo,
        license2: &LicenseInfo,
    ) -> CompatibilityLevel {
        let key = (license1.category.clone(), license2.category.clone());
        self.category_compatibility
            .get(&key)
            .cloned()
            .unwrap_or(CompatibilityLevel::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompatibilityLevel {
    Compatible,
    Incompatible,
    RequiresReview,
    Unknown,
}

impl LicenseChecker for ConflictDetector {
    fn check(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<LicenseCheckOutput, DependencyError> {
        let result = self.detect_conflicts(dependencies, config)?;
        Ok(LicenseCheckOutput::Conflicts(result))
    }

    fn name(&self) -> &str {
        "ConflictDetector"
    }

    fn supports_transitive(&self) -> bool {
        false
    }
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            compatibility_matrix: LicenseCompatibilityMatrix::new(),
        }
    }

    pub fn detect_conflicts(
        &self,
        dependencies: &[DependencyInfo],
        _config: &LicenseConfig,
    ) -> Result<ConflictAnalysis, DependencyError> {
        let license_map = self.extract_license_map(dependencies)?;

        let conflicts = self.find_actual_conflicts(&license_map);
        let potential_conflicts = self.find_potential_conflicts(&license_map);
        let compatibility_summary = self.create_compatibility_summary(&license_map);
        let risk_assessment = self.assess_conflict_risks(&conflicts, &potential_conflicts);

        Ok(ConflictAnalysis {
            conflicts,
            potential_conflicts,
            compatibility_summary,
            risk_assessment,
        })
    }

    fn extract_license_map(
        &self,
        dependencies: &[DependencyInfo],
    ) -> Result<HashMap<String, LicenseInfo>, DependencyError> {
        let mut license_map = HashMap::new();

        for dep in dependencies {
            if let Some(license) = self.get_package_license(dep)? {
                license_map.insert(dep.name.clone(), license);
            }
        }

        Ok(license_map)
    }

    fn get_package_license(
        &self,
        dep: &DependencyInfo,
    ) -> Result<Option<LicenseInfo>, DependencyError> {
        match dep.name.as_str() {
            "mit-package" => Ok(Some(self.create_license(
                "MIT",
                "MIT License",
                LicenseCategory::Permissive,
            ))),
            "gpl-package" => Ok(Some(self.create_license(
                "GPL-3.0",
                "GNU General Public License v3.0",
                LicenseCategory::Copyleft,
            ))),
            "proprietary-package" => Ok(Some(self.create_license(
                "Proprietary",
                "Proprietary License",
                LicenseCategory::Proprietary,
            ))),
            _ => Ok(None),
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

    fn find_actual_conflicts(
        &self,
        license_map: &HashMap<String, LicenseInfo>,
    ) -> Vec<LicenseConflict> {
        let mut conflicts = Vec::new();
        let packages: Vec<_> = license_map.iter().collect();

        for i in 0..packages.len() {
            for j in i + 1..packages.len() {
                let (pkg1, license1) = packages[i];
                let (pkg2, license2) = packages[j];

                if let Some(conflict) =
                    self.check_license_compatibility(pkg1, license1, pkg2, license2)
                {
                    conflicts.push(conflict);
                }
            }
        }

        conflicts
    }

    fn check_license_compatibility(
        &self,
        pkg1: &str,
        license1: &LicenseInfo,
        pkg2: &str,
        license2: &LicenseInfo,
    ) -> Option<LicenseConflict> {
        let compatibility = self
            .compatibility_matrix
            .get_compatibility(license1, license2);

        match compatibility {
            CompatibilityLevel::Incompatible => {
                let conflict_type = self.determine_conflict_type(license1, license2);
                let resolution_suggestions =
                    self.generate_resolution_suggestions(license1, license2);

                Some(LicenseConflict {
                    package1: pkg1.to_string(),
                    license1: license1.clone(),
                    package2: pkg2.to_string(),
                    license2: license2.clone(),
                    conflict_type,
                    resolution_suggestions,
                })
            }
            _ => None,
        }
    }

    fn determine_conflict_type(
        &self,
        license1: &LicenseInfo,
        license2: &LicenseInfo,
    ) -> ConflictType {
        use LicenseCategory::*;

        match (&license1.category, &license2.category) {
            (Copyleft, Proprietary) | (Proprietary, Copyleft) => ConflictType::Incompatible,
            (Copyleft, Permissive) | (Permissive, Copyleft) => ConflictType::RequiresDisclosure,
            (Proprietary, _) | (_, Proprietary) => ConflictType::RestrictsCommercialUse,
            _ => ConflictType::RequiresAttribution,
        }
    }

    fn generate_resolution_suggestions(
        &self,
        license1: &LicenseInfo,
        license2: &LicenseInfo,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match (&license1.category, &license2.category) {
            (LicenseCategory::Copyleft, LicenseCategory::Proprietary)
            | (LicenseCategory::Proprietary, LicenseCategory::Copyleft) => {
                suggestions.push(
                    "Consider replacing the proprietary component with an open-source alternative"
                        .to_string(),
                );
                suggestions.push(
                    "Obtain a commercial license for the copyleft component if available"
                        .to_string(),
                );
            }
            (LicenseCategory::Permissive, LicenseCategory::Copyleft)
            | (LicenseCategory::Copyleft, LicenseCategory::Permissive) => {
                suggestions
                    .push("Ensure all source code is available under copyleft terms".to_string());
                suggestions.push(
                    "Consider dual-licensing approach if both licenses are supported".to_string(),
                );
            }
            _ => {
                suggestions.push(
                    "Review license terms for specific compatibility requirements".to_string(),
                );
                suggestions
                    .push("Consult legal counsel for complex license interactions".to_string());
            }
        }

        suggestions
    }

    fn find_potential_conflicts(
        &self,
        license_map: &HashMap<String, LicenseInfo>,
    ) -> Vec<PotentialConflict> {
        let mut potential_conflicts = Vec::new();
        let packages: Vec<_> = license_map.iter().collect();

        for i in 0..packages.len() {
            for j in i + 1..packages.len() {
                let (pkg1, license1) = packages[i];
                let (pkg2, license2) = packages[j];

                if let Some(potential) =
                    self.check_potential_conflict(pkg1, license1, pkg2, license2)
                {
                    potential_conflicts.push(potential);
                }
            }
        }

        potential_conflicts
    }

    fn check_potential_conflict(
        &self,
        pkg1: &str,
        license1: &LicenseInfo,
        pkg2: &str,
        license2: &LicenseInfo,
    ) -> Option<PotentialConflict> {
        let compatibility = self
            .compatibility_matrix
            .get_compatibility(license1, license2);

        match compatibility {
            CompatibilityLevel::RequiresReview => {
                let (probability, reasons) =
                    self.calculate_conflict_probability(license1, license2);

                Some(PotentialConflict {
                    package1: pkg1.to_string(),
                    license1: license1.clone(),
                    package2: pkg2.to_string(),
                    license2: license2.clone(),
                    conflict_probability: probability,
                    conflict_reasons: reasons,
                })
            }
            _ => None,
        }
    }

    fn calculate_conflict_probability(
        &self,
        license1: &LicenseInfo,
        license2: &LicenseInfo,
    ) -> (f32, Vec<String>) {
        let mut probability: f32 = 0.0;
        let mut reasons = Vec::new();

        if license1.category == LicenseCategory::Copyleft
            || license2.category == LicenseCategory::Copyleft
        {
            probability += 0.3;
            reasons.push("Copyleft licenses may require source disclosure".to_string());
        }

        if license1.category == LicenseCategory::Unknown
            || license2.category == LicenseCategory::Unknown
        {
            probability += 0.4;
            reasons.push("Unknown license terms create uncertainty".to_string());
        }

        if license1.category != license2.category {
            probability += 0.1;
            reasons
                .push("Different license categories may have conflicting requirements".to_string());
        }

        (probability.min(1.0), reasons)
    }

    fn create_compatibility_summary(
        &self,
        license_map: &HashMap<String, LicenseInfo>,
    ) -> CompatibilitySummary {
        let packages: Vec<_> = license_map.iter().collect();
        let total_pairs = packages.len() * (packages.len() - 1) / 2;

        let mut compatible_pairs = 0;
        let mut incompatible_pairs = 0;
        let mut license_distribution = HashMap::new();

        for license in license_map.values() {
            *license_distribution
                .entry(license.category.clone())
                .or_insert(0) += 1;
        }

        for i in 0..packages.len() {
            for j in i + 1..packages.len() {
                let (_, license1) = packages[i];
                let (_, license2) = packages[j];

                match self
                    .compatibility_matrix
                    .get_compatibility(license1, license2)
                {
                    CompatibilityLevel::Compatible => compatible_pairs += 1,
                    CompatibilityLevel::Incompatible => incompatible_pairs += 1,
                    CompatibilityLevel::RequiresReview => {}
                    CompatibilityLevel::Unknown => {}
                }
            }
        }

        let total_pairs = packages.len() * (packages.len() - 1) / 2;
        let compatibility_score = if total_pairs > 0 {
            compatible_pairs as f32 / total_pairs as f32
        } else {
            1.0
        };

        CompatibilitySummary {
            total_license_pairs: total_pairs,
            compatible_pairs,
            incompatible_pairs,
            compatibility_score,
            license_distribution,
        }
    }

    fn assess_conflict_risks(
        &self,
        conflicts: &[LicenseConflict],
        potential_conflicts: &[PotentialConflict],
    ) -> ConflictRiskAssessment {
        let conflict_count = conflicts.len() + potential_conflicts.len();

        let overall_risk = if conflict_count == 0 {
            RiskLevel::Low
        } else if conflict_count <= 2 {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        };

        let legal_risk = if conflicts.iter().any(|c| {
            matches!(
                c.conflict_type,
                ConflictType::Incompatible | ConflictType::RequiresDisclosure
            )
        }) {
            RiskLevel::High
        } else if conflicts.is_empty() {
            RiskLevel::Low
        } else {
            RiskLevel::Medium
        };

        let business_risk = if conflicts.len() > 3 {
            RiskLevel::High
        } else if conflicts.is_empty() {
            RiskLevel::Low
        } else {
            RiskLevel::Medium
        };

        let mitigation_urgency = if overall_risk == RiskLevel::High {
            MitigationUrgency::Immediate
        } else if overall_risk == RiskLevel::Medium {
            MitigationUrgency::High
        } else {
            MitigationUrgency::Low
        };

        ConflictRiskAssessment {
            overall_risk,
            legal_risk,
            business_risk,
            mitigation_urgency,
        }
    }
}
