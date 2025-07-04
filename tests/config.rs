use uveddi::config::Config;
use std::env;
use std::fs;
use tempfile::tempdir;

fn setup() {
    // Clean up any previous environment variables
    env::remove_var("OLLAMA_MODEL");
}

#[test]
fn test_config_from_env_with_ollama_model() {
    setup();
    
    // Set environment variable
    env::set_var("OLLAMA_MODEL", "llama2");
    
    let config = Config::from_env().unwrap();
    assert_eq!(config.ollama_model, Some("llama2".to_string()));
    
    // Clean up
    env::remove_var("OLLAMA_MODEL");
}

#[test]
fn test_config_from_env_without_ollama_model() {
    setup();
    
    let config = Config::from_env().unwrap();
    assert_eq!(config.ollama_model, None);
}

#[test]
fn test_config_from_file_valid_toml() {
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
    setup();
    
    let result = Config::from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_from_file_invalid_toml() {
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
    setup();
    
    let config = Config {
        ollama_model: Some("test_model".to_string()),
    };
    
    let cloned_config = config.clone();
    assert_eq!(config.ollama_model, cloned_config.ollama_model);
}

#[test]
fn test_config_debug_format() {
    setup();
    
    let config = Config {
        ollama_model: Some("test_model".to_string()),
    };
    
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("test_model"));
}
