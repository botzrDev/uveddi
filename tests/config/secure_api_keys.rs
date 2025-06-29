//! tests/config/secure_api_keys.rs

use codeatlas::config::Config;
use std::env;

#[test]
fn test_load_config_from_env() {
    env::set_var("OPENAI_API_KEY", "env_key");
    env::set_var("ANTHROPIC_API_KEY", "anthropic_key");

    let config = Config::from_env().unwrap();

    assert_eq!(config.openai_api_key, Some("env_key".to_string()));
    assert_eq!(config.anthropic_api_key, Some("anthropic_key".to_string()));
    assert!(config.gemini_api_key.is_none());

    env::remove_var("OPENAI_API_KEY");
    env::remove_var("ANTHROPIC_API_KEY");
}