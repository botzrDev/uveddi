//! Malicious pattern signatures and type definitions
//!
//! This module contains the core malicious pattern definitions, threat level classifications,
//! and the comprehensive signature database used for detecting malicious code patterns.

use crate::analysis::detectors::security::types::SecuritySeverity;
use serde::{Deserialize, Serialize};

/// Malicious pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousPattern {
    pub name: String,
    pub description: String,
    pub pattern_type: MaliciousPatternType,
    pub signatures: Vec<String>,
    pub indicators: Vec<String>,
    pub severity: SecuritySeverity,
    pub confidence_base: f64,
    pub threat_level: ThreatLevel,
}

/// Types of malicious patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaliciousPatternType {
    Backdoor,
    Keylogger,
    DataExfiltration,
    RemoteAccess,
    Persistence,
    Evasion,
    Rootkit,
    Botnet,
}

/// Threat levels for malicious patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ToString for MaliciousPatternType {
    fn to_string(&self) -> String {
        match self {
            MaliciousPatternType::Backdoor => "Backdoor".to_string(),
            MaliciousPatternType::Keylogger => "Keylogger".to_string(),
            MaliciousPatternType::DataExfiltration => "DataExfiltration".to_string(),
            MaliciousPatternType::RemoteAccess => "RemoteAccess".to_string(),
            MaliciousPatternType::Persistence => "Persistence".to_string(),
            MaliciousPatternType::Evasion => "Evasion".to_string(),
            MaliciousPatternType::Rootkit => "Rootkit".to_string(),
            MaliciousPatternType::Botnet => "Botnet".to_string(),
        }
    }
}

/// Load default malicious pattern signatures
pub fn load_default_patterns() -> Vec<MaliciousPattern> {
    vec![
        MaliciousPattern {
            name: "backdoor_signature".to_string(),
            description: "Known backdoor signatures and patterns".to_string(),
            pattern_type: MaliciousPatternType::Backdoor,
            signatures: vec![
                "backdoor".to_string(),
                "remote_shell".to_string(),
                "reverse_shell".to_string(),
                "cmd_shell".to_string(),
                "hidden_access".to_string(),
            ],
            indicators: vec![
                "bind_shell".to_string(),
                "connect_back".to_string(),
                "shell_code".to_string(),
            ],
            severity: SecuritySeverity::Critical,
            confidence_base: 0.9,
            threat_level: ThreatLevel::Critical,
        },
        MaliciousPattern {
            name: "keylogger_pattern".to_string(),
            description: "Keylogging and input capture patterns".to_string(),
            pattern_type: MaliciousPatternType::Keylogger,
            signatures: vec![
                "keylog".to_string(),
                "keystroke".to_string(),
                "input_capture".to_string(),
                "keyboard_hook".to_string(),
            ],
            indicators: vec![
                "key_press".to_string(),
                "input_monitor".to_string(),
                "keyboard_spy".to_string(),
            ],
            severity: SecuritySeverity::High,
            confidence_base: 0.85,
            threat_level: ThreatLevel::High,
        },
        MaliciousPattern {
            name: "data_exfiltration".to_string(),
            description: "Data theft and exfiltration patterns".to_string(),
            pattern_type: MaliciousPatternType::DataExfiltration,
            signatures: vec![
                "exfiltrate".to_string(),
                "data_theft".to_string(),
                "steal_data".to_string(),
                "upload_stolen".to_string(),
            ],
            indicators: vec![
                "collect_files".to_string(),
                "gather_info".to_string(),
                "transmit_data".to_string(),
            ],
            severity: SecuritySeverity::Critical,
            confidence_base: 0.8,
            threat_level: ThreatLevel::Critical,
        },
        MaliciousPattern {
            name: "remote_access_tool".to_string(),
            description: "Remote access trojan (RAT) patterns".to_string(),
            pattern_type: MaliciousPatternType::RemoteAccess,
            signatures: vec![
                "remote_access".to_string(),
                "rat_client".to_string(),
                "remote_control".to_string(),
                "admin_panel".to_string(),
            ],
            indicators: vec![
                "remote_desktop".to_string(),
                "vnc_server".to_string(),
                "remote_cmd".to_string(),
            ],
            severity: SecuritySeverity::Critical,
            confidence_base: 0.9,
            threat_level: ThreatLevel::Critical,
        },
        MaliciousPattern {
            name: "persistence_mechanism".to_string(),
            description: "System persistence and startup mechanisms".to_string(),
            pattern_type: MaliciousPatternType::Persistence,
            signatures: vec![
                "auto_start".to_string(),
                "registry_persist".to_string(),
                "startup_folder".to_string(),
                "service_install".to_string(),
            ],
            indicators: vec![
                "cron_job".to_string(),
                "scheduled_task".to_string(),
                "systemd_service".to_string(),
            ],
            severity: SecuritySeverity::High,
            confidence_base: 0.75,
            threat_level: ThreatLevel::High,
        },
        MaliciousPattern {
            name: "evasion_technique".to_string(),
            description: "Anti-analysis and evasion techniques".to_string(),
            pattern_type: MaliciousPatternType::Evasion,
            signatures: vec![
                "anti_debug".to_string(),
                "vm_detect".to_string(),
                "sandbox_evasion".to_string(),
                "obfuscation".to_string(),
            ],
            indicators: vec![
                "sleep_evasion".to_string(),
                "timing_check".to_string(),
                "env_check".to_string(),
            ],
            severity: SecuritySeverity::High,
            confidence_base: 0.8,
            threat_level: ThreatLevel::High,
        },
    ]
}