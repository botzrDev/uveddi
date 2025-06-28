use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub openai_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        Ok(Config { openai_api_key })
    }
}
