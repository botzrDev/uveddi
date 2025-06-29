use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub ollama_model: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let anthropic_api_key = env::var("ANTHROPIC_API_KEY").ok();
        let gemini_api_key = env::var("GEMINI_API_KEY").ok();
        let ollama_model = env::var("OLLAMA_MODEL").ok();
        Ok(Config {
            openai_api_key,
            anthropic_api_key,
            gemini_api_key,
            ollama_model,
        })
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
