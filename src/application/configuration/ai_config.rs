//! AI integration configuration for LLM providers and analysis settings

use crate::error::UveddiError;

/// Configuration for AI-powered analysis features
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// A flag to enable or disable AI-powered analysis
    pub enable_ai: bool,
    /// AI provider configuration
    pub provider: AiProvider,
    /// Analysis settings for AI processing
    pub analysis_settings: AiAnalysisSettings,
}

/// AI provider configuration options
#[derive(Debug, Clone)]
pub enum AiProvider {
    /// Local Ollama API configuration
    Ollama {
        /// The URL of the Ollama API endpoint
        api_url: String,
        /// The name of the Ollama model to be used for analysis
        model: String,
    },
    /// OpenAI API configuration
    OpenAi {
        /// OpenAI API key
        api_key: String,
        /// Model name (e.g., "gpt-4", "gpt-3.5-turbo")
        model: String,
        /// API base URL (optional, for custom endpoints)
        base_url: Option<String>,
    },
    /// Anthropic Claude API configuration
    Anthropic {
        /// Anthropic API key
        api_key: String,
        /// Model name (e.g., "claude-3-sonnet", "claude-3-haiku")
        model: String,
    },
    /// Google Gemini API configuration
    Gemini {
        /// Google API key
        api_key: String,
        /// Model name (e.g., "gemini-pro", "gemini-pro-vision")
        model: String,
    },
    /// Mock provider for testing
    Mock,
}

/// AI analysis configuration settings
#[derive(Debug, Clone)]
pub struct AiAnalysisSettings {
    /// Maximum number of issues to analyze with AI
    pub max_issues_to_analyze: usize,
    /// Confidence threshold for AI suggestions (0.0 to 1.0)
    pub confidence_threshold: f64,
    /// Enable detailed analysis for critical issues
    pub enable_detailed_analysis: bool,
    /// Enable code snippet analysis
    pub analyze_code_snippets: bool,
    /// Maximum lines of code to include in analysis context
    pub max_context_lines: u32,
    /// Timeout for AI API calls in seconds
    pub api_timeout_seconds: u64,
    /// Maximum retries for failed AI calls
    pub max_retries: u32,
}

impl AiConfig {
    /// Create a new AI configuration with Ollama provider
    pub fn with_ollama(api_url: String, model: String) -> Self {
        Self {
            enable_ai: true,
            provider: AiProvider::Ollama { api_url, model },
            analysis_settings: AiAnalysisSettings::default(),
        }
    }

    /// Create a new AI configuration with OpenAI provider
    pub fn with_openai(api_key: String, model: String) -> Self {
        Self {
            enable_ai: true,
            provider: AiProvider::OpenAi {
                api_key,
                model,
                base_url: None,
            },
            analysis_settings: AiAnalysisSettings::default(),
        }
    }

    /// Create a new AI configuration with Anthropic provider
    pub fn with_anthropic(api_key: String, model: String) -> Self {
        Self {
            enable_ai: true,
            provider: AiProvider::Anthropic { api_key, model },
            analysis_settings: AiAnalysisSettings::default(),
        }
    }

    /// Create a new AI configuration with Gemini provider
    pub fn with_gemini(api_key: String, model: String) -> Self {
        Self {
            enable_ai: true,
            provider: AiProvider::Gemini { api_key, model },
            analysis_settings: AiAnalysisSettings::default(),
        }
    }

    /// Create a disabled AI configuration
    pub fn disabled() -> Self {
        Self {
            enable_ai: false,
            provider: AiProvider::Mock,
            analysis_settings: AiAnalysisSettings::default(),
        }
    }

    /// Validate the AI configuration
    pub fn validate(&self) -> Result<(), UveddiError> {
        if !self.enable_ai {
            return Ok(()); // Skip validation if AI is disabled
        }

        // Validate provider configuration
        match &self.provider {
            AiProvider::Ollama { api_url, model } => {
                if api_url.is_empty() {
                    return Err(UveddiError::config_error(
                        "Ollama API URL cannot be empty",
                        "AI configuration",
                    ));
                }
                if model.is_empty() {
                    return Err(UveddiError::config_error(
                        "Ollama model name cannot be empty",
                        "AI configuration",
                    ));
                }
                // Validate URL format
                if !api_url.starts_with("http://") && !api_url.starts_with("https://") {
                    return Err(UveddiError::config_error(
                        "Ollama API URL must start with http:// or https://",
                        "AI configuration",
                    ));
                }
            }
            AiProvider::OpenAi { api_key, model, .. } => {
                if api_key.is_empty() {
                    return Err(UveddiError::config_error(
                        "OpenAI API key cannot be empty",
                        "AI configuration",
                    ));
                }
                if model.is_empty() {
                    return Err(UveddiError::config_error(
                        "OpenAI model name cannot be empty",
                        "AI configuration",
                    ));
                }
            }
            AiProvider::Anthropic { api_key, model } => {
                if api_key.is_empty() {
                    return Err(UveddiError::config_error(
                        "Anthropic API key cannot be empty",
                        "AI configuration",
                    ));
                }
                if model.is_empty() {
                    return Err(UveddiError::config_error(
                        "Anthropic model name cannot be empty",
                        "AI configuration",
                    ));
                }
            }
            AiProvider::Gemini { api_key, model } => {
                if api_key.is_empty() {
                    return Err(UveddiError::config_error(
                        "Gemini API key cannot be empty",
                        "AI configuration",
                    ));
                }
                if model.is_empty() {
                    return Err(UveddiError::config_error(
                        "Gemini model name cannot be empty",
                        "AI configuration",
                    ));
                }
            }
            AiProvider::Mock => {
                // Mock provider is always valid
            }
        }

        // Validate analysis settings
        self.analysis_settings.validate()?;

        Ok(())
    }

    /// Merge with another AI configuration
    pub fn merge_with(mut self, other: AiConfig) -> Self {
        if other.enable_ai {
            self.enable_ai = other.enable_ai;
            self.provider = other.provider;
        }
        self.analysis_settings = self.analysis_settings.merge_with(other.analysis_settings);
        self
    }

    /// Get the provider name as a string
    pub fn provider_name(&self) -> &str {
        match &self.provider {
            AiProvider::Ollama { .. } => "ollama",
            AiProvider::OpenAi { .. } => "openai",
            AiProvider::Anthropic { .. } => "anthropic",
            AiProvider::Gemini { .. } => "gemini",
            AiProvider::Mock => "mock",
        }
    }

    /// Check if the provider requires an API key
    pub fn requires_api_key(&self) -> bool {
        matches!(
            &self.provider,
            AiProvider::OpenAi { .. } | AiProvider::Anthropic { .. } | AiProvider::Gemini { .. }
        )
    }

    /// Get the model name for the current provider
    pub fn model_name(&self) -> Option<&str> {
        match &self.provider {
            AiProvider::Ollama { model, .. }
            | AiProvider::OpenAi { model, .. }
            | AiProvider::Anthropic { model, .. }
            | AiProvider::Gemini { model, .. } => Some(model),
            AiProvider::Mock => None,
        }
    }
}

impl AiAnalysisSettings {
    /// Create new analysis settings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the analysis settings
    pub fn validate(&self) -> Result<(), UveddiError> {
        if self.max_issues_to_analyze == 0 {
            return Err(UveddiError::config_error(
                "Maximum issues to analyze must be greater than 0",
                "AI analysis settings",
            ));
        }

        if self.confidence_threshold < 0.0 || self.confidence_threshold > 1.0 {
            return Err(UveddiError::config_error(
                "Confidence threshold must be between 0.0 and 1.0",
                "AI analysis settings",
            ));
        }

        if self.max_context_lines == 0 {
            return Err(UveddiError::config_error(
                "Maximum context lines must be greater than 0",
                "AI analysis settings",
            ));
        }

        if self.api_timeout_seconds == 0 {
            return Err(UveddiError::config_error(
                "API timeout must be greater than 0 seconds",
                "AI analysis settings",
            ));
        }

        Ok(())
    }

    /// Merge with another analysis settings configuration
    pub fn merge_with(mut self, other: AiAnalysisSettings) -> Self {
        if other.max_issues_to_analyze != 10 { // 10 is default
            self.max_issues_to_analyze = other.max_issues_to_analyze;
        }
        if (other.confidence_threshold - 0.75).abs() > f64::EPSILON { // 0.75 is default
            self.confidence_threshold = other.confidence_threshold;
        }
        if other.enable_detailed_analysis {
            self.enable_detailed_analysis = other.enable_detailed_analysis;
        }
        if other.analyze_code_snippets {
            self.analyze_code_snippets = other.analyze_code_snippets;
        }
        if other.max_context_lines != 100 { // 100 is default
            self.max_context_lines = other.max_context_lines;
        }
        if other.api_timeout_seconds != 30 { // 30 is default
            self.api_timeout_seconds = other.api_timeout_seconds;
        }
        if other.max_retries != 3 { // 3 is default
            self.max_retries = other.max_retries;
        }
        self
    }

    /// Configure for performance-focused analysis
    pub fn configure_performance(&mut self) -> &mut Self {
        self.max_issues_to_analyze = 5;
        self.enable_detailed_analysis = false;
        self.analyze_code_snippets = false;
        self.max_context_lines = 50;
        self.api_timeout_seconds = 15;
        self.max_retries = 1;
        self
    }

    /// Configure for comprehensive analysis
    pub fn configure_comprehensive(&mut self) -> &mut Self {
        self.max_issues_to_analyze = 25;
        self.enable_detailed_analysis = true;
        self.analyze_code_snippets = true;
        self.max_context_lines = 200;
        self.api_timeout_seconds = 60;
        self.max_retries = 5;
        self
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enable_ai: false,
            provider: AiProvider::Ollama {
                api_url: "http://localhost:11434".to_string(),
                model: "deepseek-coder:6.7b".to_string(),
            },
            analysis_settings: AiAnalysisSettings::default(),
        }
    }
}

impl Default for AiAnalysisSettings {
    fn default() -> Self {
        Self {
            max_issues_to_analyze: 10,
            confidence_threshold: 0.75,
            enable_detailed_analysis: true,
            analyze_code_snippets: true,
            max_context_lines: 100,
            api_timeout_seconds: 30,
            max_retries: 3,
        }
    }
}