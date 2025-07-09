use crate::error::RenderingServiceError;
use std::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct FallbackConfig {
    pub fallback_timeout: Duration,
    pub recovery_check_interval: Duration,
    pub max_fallback_duration: Duration,
    pub enable_automatic_recovery: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            fallback_timeout: Duration::from_secs(30),
            recovery_check_interval: Duration::from_secs(60),
            max_fallback_duration: Duration::from_secs(3600), // 1 hour
            enable_automatic_recovery: true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FallbackMode {
    Normal,
    MermaidOnly { triggered_at: Instant, reason: String },
}

pub struct FallbackManager {
    config: FallbackConfig,
    mode: FallbackMode,
    consecutive_fallback_triggers: u32,
}

impl FallbackManager {
    pub fn new(config: FallbackConfig) -> Self {
        Self {
            config,
            mode: FallbackMode::Normal,
            consecutive_fallback_triggers: 0,
        }
    }

    pub fn should_trigger_fallback(&mut self, error: &RenderingServiceError) -> bool {
        // If already in fallback mode, don't trigger again
        if self.is_in_fallback_mode() {
            return false;
        }

        // Check if this error should trigger fallback
        if error.should_trigger_fallback() {
            self.consecutive_fallback_triggers += 1;
            
            // Trigger fallback after 3 consecutive critical errors
            if self.consecutive_fallback_triggers >= 3 {
                let reason = format!("Triggered by consecutive errors: {}", error);
                self.trigger_fallback(reason);
                return true;
            }
        }
        false
    }

    pub fn trigger_fallback(&mut self, reason: String) {
        // Only trigger if not already in fallback mode
        if !self.is_in_fallback_mode() {
            warn!(
                "Entering fallback mode: {}. Consecutive triggers: {}",
                reason, self.consecutive_fallback_triggers
            );
            self.mode = FallbackMode::MermaidOnly {
                triggered_at: Instant::now(),
                reason,
            };
        }
    }

    pub fn attempt_recovery(&mut self) -> bool {
        // Only attempt recovery if in fallback mode
        if let FallbackMode::MermaidOnly { triggered_at, reason } = &self.mode {
            let fallback_duration = triggered_at.elapsed();
            
            // Check if we've exceeded max fallback duration
            if fallback_duration > self.config.max_fallback_duration {
                info!(
                    "Recovering from fallback mode (max duration exceeded): {}",
                    reason
                );
                self.mode = FallbackMode::Normal;
                self.consecutive_fallback_triggers = 0;
                return true;
            }
            
            // Check if recovery interval has passed and automatic recovery is enabled
            if self.config.enable_automatic_recovery && fallback_duration > self.config.recovery_check_interval {
                info!("Recovering from fallback mode: {}", reason);
                self.mode = FallbackMode::Normal;
                self.consecutive_fallback_triggers = 0;
                return true;
            }
        }
        false
    }

    pub fn is_in_fallback_mode(&self) -> bool {
        matches!(self.mode, FallbackMode::MermaidOnly { .. })
    }

    pub fn get_fallback_reason(&self) -> Option<String> {
        match &self.mode {
            FallbackMode::MermaidOnly { reason, .. } => Some(reason.clone()),
            FallbackMode::Normal => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn create_critical_error() -> RenderingServiceError {
        RenderingServiceError::ServiceUnavailable
    }

    fn create_non_critical_error() -> RenderingServiceError {
        RenderingServiceError::InvalidMermaidSyntax {
            line: Some(1),
            details: "Test input".to_string(),
        }
    }

    #[test]
    fn test_should_trigger_fallback() {
        let config = FallbackConfig::default();
        let mut manager = FallbackManager::new(config);

        // Non-critical errors shouldn't trigger fallback
        let non_critical = create_non_critical_error();
        assert!(!manager.should_trigger_fallback(&non_critical));
        assert!(!manager.is_in_fallback_mode());

        // First critical error
        let critical = create_critical_error();
        assert!(!manager.should_trigger_fallback(&critical));
        assert!(!manager.is_in_fallback_mode());
        assert_eq!(manager.consecutive_fallback_triggers, 1);

        // Second critical error
        assert!(!manager.should_trigger_fallback(&critical));
        assert_eq!(manager.consecutive_fallback_triggers, 2);

        // Third critical error triggers fallback
        assert!(manager.should_trigger_fallback(&critical));
        assert!(manager.is_in_fallback_mode());
        assert_eq!(manager.consecutive_fallback_triggers, 3);
    }

    #[test]
    fn test_trigger_fallback() {
        let config = FallbackConfig::default();
        let mut manager = FallbackManager::new(config);

        let reason = "Test reason".to_string();
        manager.trigger_fallback(reason.clone());

        assert!(manager.is_in_fallback_mode());
        assert_eq!(manager.get_fallback_reason(), Some(reason));
    }

    #[test]
    fn test_attempt_recovery() {
        let mut config = FallbackConfig::default();
        config.recovery_check_interval = Duration::from_millis(50);
        config.max_fallback_duration = Duration::from_millis(100);
        
        let mut manager = FallbackManager::new(config);
        manager.trigger_fallback("Test recovery".to_string());

        // Shouldn't recover immediately
        assert!(!manager.attempt_recovery());
        assert!(manager.is_in_fallback_mode());

        // Wait longer than recovery interval
        thread::sleep(Duration::from_millis(75));
        assert!(manager.attempt_recovery());
        assert!(!manager.is_in_fallback_mode());
        assert_eq!(manager.consecutive_fallback_triggers, 0);

        // Test max duration recovery
        manager.trigger_fallback("Test max duration".to_string());
        thread::sleep(Duration::from_millis(110));
        assert!(manager.attempt_recovery());
    }

    #[test]
    fn test_no_recovery_when_disabled() {
        let mut config = FallbackConfig::default();
        config.enable_automatic_recovery = false;
        config.recovery_check_interval = Duration::from_millis(50);
        
        let mut manager = FallbackManager::new(config);
        manager.trigger_fallback("Test disabled recovery".to_string());

        thread::sleep(Duration::from_millis(75));
        assert!(!manager.attempt_recovery());
        assert!(manager.is_in_fallback_mode());
    }
}
