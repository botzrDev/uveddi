//! GeminiProvider: Google Gemini API integration stub
//
// This is a placeholder for Google Gemini API integration.

pub struct GeminiProvider {
    pub api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    pub async fn infer(&self, prompt: &str) -> Result<String, String> {
        // TODO: Implement HTTP call to Google Gemini API
        Ok("[Gemini stub]".to_string())
    }
}
