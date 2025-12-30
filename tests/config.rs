use std::env;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;
use uveddi::config::Config;

// Mutex to ensure environment variable tests don't interfere with each other
// Use unwrap_or_else to recover from poisoned mutex
static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn acquire_env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

fn setup() {
    // Clean up any previous environment variables
    env::remove_var("OLLAMA_MODEL");
}

#[test]
fn test_config_from_env_with_ollama_model() {
    let _lock = acquire_env_lock();
    setup();

    // Set environment variable
    env::set_var("OLLAMA_MODEL", "llama2");

    let config = Config::from_env().unwrap();

    // Clean up first to prevent interference
    env::remove_var("OLLAMA_MODEL");

    assert_eq!(config.ollama_model, Some("llama2".to_string()));
}

#[test]
fn test_config_from_env_without_ollama_model() {
    let _lock = acquire_env_lock();
    setup();

    let config = Config::from_env().unwrap();
    assert_eq!(config.ollama_model, None);
}

#[test]
fn test_config_from_file_valid_toml() {
    let _lock = acquire_env_lock();
    setup();

    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("config.toml");

    // Create a valid TOML config file
    let config_content = r#"
ollama_model = "llama2"
"#;
    fs::write(&config_file, config_content).unwrap();

    let config = Config::from_file(config_file.to_str().unwrap()).unwrap();
    assert_eq!(config.ollama_model, Some("llama2".to_string()));
}

#[test]
fn test_config_from_file_minimal_toml() {
    let _lock = acquire_env_lock();
    setup();

    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("config.toml");

    // Create a minimal TOML config file (no ollama_model specified)
    let config_content = r#"
# Empty config file
"#;
    fs::write(&config_file, config_content).unwrap();

    let config = Config::from_file(config_file.to_str().unwrap()).unwrap();
    assert_eq!(config.ollama_model, None);
}

#[test]
fn test_config_from_file_nonexistent_file() {
    let _lock = acquire_env_lock();
    setup();

    let result = Config::from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_from_file_invalid_toml() {
    let _lock = acquire_env_lock();
    setup();

    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("config.toml");

    // Create an invalid TOML file
    let config_content = r#"
invalid toml content [[[
"#;
    fs::write(&config_file, config_content).unwrap();

    let result = Config::from_file(config_file.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_config_clone() {
    let _lock = acquire_env_lock();
    setup();

    let config = Config {
        ollama_model: Some("test_model".to_string()),
        dead_code: None,
        large_classes: None,
    };

    let cloned_config = config.clone();
    assert_eq!(config.ollama_model, cloned_config.ollama_model);
}

#[test]
fn test_config_debug_format() {
    let _lock = acquire_env_lock();
    setup();

    let config = Config {
        ollama_model: Some("test_model".to_string()),
        dead_code: None,
        large_classes: None,
    };

    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("test_model"));
}

/// Integration test for config show/set workflow (A3 requirement)
#[test]
fn test_config_show_set_workflow() {
    let _lock = acquire_env_lock();
    setup();

    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("uveddi.toml");

    // Step 1: Create initial config with set operation
    let initial_config = Config {
        ollama_model: Some("initial-model".to_string()),
        dead_code: None,
        large_classes: None,
    };

    let toml_content = toml::to_string_pretty(&initial_config).unwrap();
    fs::write(&config_file, toml_content).unwrap();

    // Step 2: Verify show can read the config
    let loaded_config = Config::from_file(config_file.to_str().unwrap()).unwrap();
    assert_eq!(
        loaded_config.ollama_model,
        Some("initial-model".to_string())
    );

    // Step 3: Update config (simulating set operation)
    let updated_config = Config {
        ollama_model: Some("updated-model".to_string()),
        dead_code: None,
        large_classes: None,
    };

    let updated_toml = toml::to_string_pretty(&updated_config).unwrap();
    fs::write(&config_file, updated_toml).unwrap();

    // Step 4: Verify show reflects the update
    let final_config = Config::from_file(config_file.to_str().unwrap()).unwrap();
    assert_eq!(final_config.ollama_model, Some("updated-model".to_string()));
}

/// Integration test for config persistence across operations (A3 requirement)
#[test]
fn test_config_persistence_across_operations() {
    let _lock = acquire_env_lock();
    setup();

    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("uveddi.toml");

    // Create config with all fields
    let config = Config {
        ollama_model: Some("test-model".to_string()),
        dead_code: None,
        large_classes: None,
    };

    let toml = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_file, &toml).unwrap();

    // Load and verify
    let loaded = Config::from_file(config_file.to_str().unwrap()).unwrap();
    assert_eq!(loaded.ollama_model, config.ollama_model);

    // Update one field (simulating set operation)
    let updated = Config {
        ollama_model: Some("new-model".to_string()),
        dead_code: loaded.dead_code.clone(),
        large_classes: loaded.large_classes.clone(),
    };

    let updated_toml = toml::to_string_pretty(&updated).unwrap();
    fs::write(&config_file, updated_toml).unwrap();

    // Verify persistence
    let final_loaded = Config::from_file(config_file.to_str().unwrap()).unwrap();
    assert_eq!(final_loaded.ollama_model, Some("new-model".to_string()));
}

/// Test config show with missing file falls back correctly (A3 requirement)
#[test]
fn test_config_show_missing_file_fallback() {
    let _lock = acquire_env_lock();
    setup();

    let temp_dir = tempdir().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.toml");

    // Verify file doesn't exist
    assert!(!nonexistent_file.exists());

    // Attempt to load should error
    let result = Config::from_file(nonexistent_file.to_str().unwrap());
    assert!(result.is_err());

    // But loading from env should still work
    env::set_var("OLLAMA_MODEL", "env-fallback-model");
    let env_config = Config::from_env().unwrap();

    // Clean up first
    env::remove_var("OLLAMA_MODEL");

    assert_eq!(
        env_config.ollama_model,
        Some("env-fallback-model".to_string())
    );
}
