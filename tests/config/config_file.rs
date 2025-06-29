//! tests/config/config_file.rs

use codeatlas::config::Config;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_load_config_from_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("codeatlas.toml");

    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "openai_api_key = 'test_key'").unwrap();
    writeln!(file, "ollama_model = 'test_model'").unwrap();

    let config = Config::from_file(file_path.to_str().unwrap()).unwrap();

    assert_eq!(config.openai_api_key, Some("test_key".to_string()));
    assert_eq!(config.ollama_model, Some("test_model".to_string()));
    assert!(config.anthropic_api_key.is_none());
}