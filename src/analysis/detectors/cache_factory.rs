//! Cache-aware detector factory
//!
//! Provides utilities for creating cache-enabled versions of detectors
//! to improve analysis performance through result caching.

#[cfg(feature = "analysis-cache")]
use super::{
    // Note: These detector imports need to be updated once the detector modules are properly exported
    // anti_patterns::{
    //     dead_code::detector::DeadCodeDetector,
    //     god_object::detector::GodObjectDetector,
    //     long_methods::detector::LongMethodsDetector,
    // },
    base::traits::Detector,
    cache_wrapper::CachedDetector,
    // cycle::CycleDetector,
    // security::detector::SecurityDetector,
};

#[cfg(feature = "analysis-cache")]
use crate::analysis::AnalysisError;

/// Factory for creating cache-aware detectors
#[cfg(feature = "analysis-cache")]
pub struct CacheAwareDetectorFactory;

#[cfg(feature = "analysis-cache")]
impl CacheAwareDetectorFactory {
    // TODO: Implement specific detector cache methods once detector types are properly exposed
    // These methods require concrete detector types to be available, which are currently
    // behind private modules. Once the detector modules are refactored to expose their
    // concrete types, these methods can be uncommented and implemented.
    //
    // Example implementation pattern:
    // pub fn create_cached_god_object_detector<D: GodObjectDetector>(
    //     detector: D,
    // ) -> CachedDetector<D> {
    //     CachedDetector::with_version(detector, "god_object:v1.2".to_string())
    // }

    /// Create a cached version of any detector with automatic versioning
    pub fn create_cached_detector<D>(detector: D) -> CachedDetector<D>
    where
        D: Detector + Send + Sync,
    {
        CachedDetector::new(detector)
    }

    /// Create a cached version of any detector with explicit version
    pub fn create_cached_detector_with_version<D>(detector: D, version: String) -> CachedDetector<D>
    where
        D: Detector + Send + Sync,
    {
        CachedDetector::with_version(detector, version)
    }
}

/// Migration helper for existing detector usage
#[cfg(feature = "analysis-cache")]
pub struct DetectorMigrationHelper;

#[cfg(feature = "analysis-cache")]
impl DetectorMigrationHelper {
    /// Migrate a collection of detectors to use caching
    pub fn migrate_to_cached<D>(detectors: Vec<D>) -> Vec<CachedDetector<D>>
    where
        D: Detector + Send + Sync,
    {
        detectors
            .into_iter()
            .map(|detector| CacheAwareDetectorFactory::create_cached_detector(detector))
            .collect()
    }

    /// Create a cache-enabled detector based on detector name
    /// This is useful for dynamic detector creation based on configuration
    pub fn create_by_name(
        name: &str,
    ) -> Result<Box<dyn Detector<Config = (), Output = ()>>, AnalysisError> {
        // This is a simplified implementation - in practice you'd need proper type handling
        match name {
            "god_object" => {
                // This would need proper configuration handling
                Err(AnalysisError::ConfigurationError {
                    field: "detector_type".to_string(),
                    value: "god_object".to_string(),
                    reason: "Not implemented".to_string(),
                })
            }
            "dead_code" => Err(AnalysisError::ConfigurationError {
                field: "detector_type".to_string(),
                value: "dead_code".to_string(),
                reason: "Not implemented".to_string(),
            }),
            "long_method" => Err(AnalysisError::ConfigurationError {
                field: "detector_type".to_string(),
                value: "long_method".to_string(),
                reason: "Not implemented".to_string(),
            }),
            "security" => Err(AnalysisError::ConfigurationError {
                field: "detector_type".to_string(),
                value: "security".to_string(),
                reason: "Not implemented".to_string(),
            }),
            "cycle" => Err(AnalysisError::ConfigurationError {
                field: "detector_type".to_string(),
                value: "cycle".to_string(),
                reason: "Not implemented".to_string(),
            }),
            _ => Err(AnalysisError::ConfigurationError {
                field: "detector_type".to_string(),
                value: name.to_string(),
                reason: format!("Unknown detector: {}", name),
            }),
        }
    }
}

/// Detector collection that supports both cached and uncached detectors
#[cfg(feature = "analysis-cache")]
pub struct MixedDetectorCollection {
    /// Cached detectors for better performance
    cached_detectors: Vec<Box<dyn Detector<Config = (), Output = ()>>>,
    /// Uncached detectors (for detectors that don't benefit from caching)
    uncached_detectors: Vec<Box<dyn Detector<Config = (), Output = ()>>>,
}

#[cfg(feature = "analysis-cache")]
impl MixedDetectorCollection {
    /// Create a new mixed collection
    pub fn new() -> Self {
        Self {
            cached_detectors: Vec::new(),
            uncached_detectors: Vec::new(),
        }
    }

    /// Add a cached detector to the collection
    pub fn add_cached<D>(&mut self, detector: D)
    where
        D: Detector<Config = (), Output = ()> + Send + Sync + 'static,
    {
        let cached = CacheAwareDetectorFactory::create_cached_detector(detector);
        self.cached_detectors.push(Box::new(cached));
    }

    /// Add an uncached detector to the collection
    pub fn add_uncached<D>(&mut self, detector: D)
    where
        D: Detector<Config = (), Output = ()> + Send + Sync + 'static,
    {
        self.uncached_detectors.push(Box::new(detector));
    }

    /// Get total number of detectors
    pub fn total_count(&self) -> usize {
        self.cached_detectors.len() + self.uncached_detectors.len()
    }

    /// Get number of cached detectors
    pub fn cached_count(&self) -> usize {
        self.cached_detectors.len()
    }

    /// Get number of uncached detectors
    pub fn uncached_count(&self) -> usize {
        self.uncached_detectors.len()
    }
}

#[cfg(feature = "analysis-cache")]
impl Default for MixedDetectorCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for cache-aware detector behavior
#[derive(Debug, Clone)]
pub struct CacheConfiguration {
    /// Whether to enable result caching
    pub enable_caching: bool,
    /// Cache size limit
    pub cache_size_limit: usize,
    /// Whether to force cache refresh
    pub force_refresh: bool,
    /// Detector-specific cache settings
    pub detector_settings: std::collections::HashMap<String, DetectorCacheSettings>,
}

/// Cache settings for individual detectors
#[derive(Debug, Clone)]
pub struct DetectorCacheSettings {
    /// Whether this specific detector should use caching
    pub enabled: bool,
    /// Custom cache expiry time in seconds
    pub expiry_seconds: Option<u64>,
    /// Custom version string for cache invalidation
    pub version: Option<String>,
}

impl Default for CacheConfiguration {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_size_limit: 5000,
            force_refresh: false,
            detector_settings: std::collections::HashMap::new(),
        }
    }
}

impl DetectorCacheSettings {
    /// Create default cache settings
    pub fn default() -> Self {
        Self {
            enabled: true,
            expiry_seconds: None,
            version: None,
        }
    }

    /// Create settings with custom expiry
    pub fn with_expiry(expiry_seconds: u64) -> Self {
        Self {
            enabled: true,
            expiry_seconds: Some(expiry_seconds),
            version: None,
        }
    }

    /// Create settings with custom version
    pub fn with_version(version: String) -> Self {
        Self {
            enabled: true,
            expiry_seconds: None,
            version: Some(version),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_configuration_default() {
        let config = CacheConfiguration::default();
        assert!(config.enable_caching);
        assert_eq!(config.cache_size_limit, 5000);
        assert!(!config.force_refresh);
        assert!(config.detector_settings.is_empty());
    }

    #[test]
    fn test_detector_cache_settings() {
        let default_settings = DetectorCacheSettings::default();
        assert!(default_settings.enabled);
        assert!(default_settings.expiry_seconds.is_none());
        assert!(default_settings.version.is_none());

        let expiry_settings = DetectorCacheSettings::with_expiry(3600);
        assert!(expiry_settings.enabled);
        assert_eq!(expiry_settings.expiry_seconds, Some(3600));

        let version_settings = DetectorCacheSettings::with_version("v2.0".to_string());
        assert!(version_settings.enabled);
        assert_eq!(version_settings.version, Some("v2.0".to_string()));
    }

    #[cfg(feature = "analysis-cache")]
    #[test]
    fn test_mixed_detector_collection() {
        let collection = MixedDetectorCollection::new();
        assert_eq!(collection.total_count(), 0);
        assert_eq!(collection.cached_count(), 0);
        assert_eq!(collection.uncached_count(), 0);
    }
}
