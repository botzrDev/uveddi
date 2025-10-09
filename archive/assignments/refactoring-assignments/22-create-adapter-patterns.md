# Assignment 22: Create Adapter Patterns

## Priority: MEDIUM
## Estimated Time: 3-4 hours
## Dependencies: Assignments 18-21 (Interfaces and Patterns)

## Objective
Implement adapter patterns to integrate external libraries and services seamlessly with the unified interface system.

## Current Problem
- Direct dependencies on external library APIs create tight coupling
- Different external services have incompatible interfaces
- Difficult to swap implementations or mock external services
- External API changes require widespread code modifications

## Tasks

### 1. Identify Adapter Opportunities

#### A. Audit External Dependencies:
```bash
# Find external library usage
rg "use (tokio|serde|clap|reqwest|sqlx)::" src/ --type rust | head -20

# Find direct external API calls
rg "\.await\?" src/ --type rust | grep -E "(reqwest|sqlx|redis)" | head -10

# Find external configuration usage
rg "Config::" src/ --type rust | head -10
```

#### B. Map External Integration Points:
```markdown
# External Integrations Needing Adapters

## High Priority:
1. **HTTP Client**: reqwest -> HttpClient trait
2. **Database**: sqlx -> DatabaseConnection trait
3. **Cache**: redis -> CacheProvider trait
4. **AI Services**: ollama/openai -> AiProvider trait

## Medium Priority:
5. **File System**: std::fs -> FileSystem trait
6. **JSON Processing**: serde_json -> JsonProcessor trait
7. **Logging**: tracing -> Logger trait
8. **Template Engine**: handlebars -> TemplateEngine trait

## Low Priority:
9. **Command Line**: clap -> CliProvider trait
10. **Configuration**: config crate -> ConfigLoader trait
```

### 2. Create HTTP Client Adapter

#### A. Define HTTP Client Interface:
```rust
// src/adapters/http_client.rs
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<HttpResponse>;
    async fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse>;
    async fn put(&self, url: &str, body: &[u8]) -> Result<HttpResponse>;
    async fn delete(&self, url: &str) -> Result<HttpResponse>;

    async fn get_with_headers(&self, url: &str, headers: &HttpHeaders) -> Result<HttpResponse>;
    async fn post_json<T: serde::Serialize + Send>(&self, url: &str, data: &T) -> Result<HttpResponse>;

    fn with_timeout(&self, timeout: Duration) -> Box<dyn HttpClient>;
    fn with_header(&self, key: &str, value: &str) -> Box<dyn HttpClient>;
    fn with_bearer_token(&self, token: &str) -> Box<dyn HttpClient>;
}

pub type HttpHeaders = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone()).map_err(|e| {
            crate::error::UveddiError::Network(crate::error::NetworkError::InvalidUrl {
                url: format!("Invalid UTF-8 in response: {}", e),
            })
        })
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(|e| {
            crate::error::UveddiError::Network(crate::error::NetworkError::HttpError {
                url: "response".to_string(),
                source: Box::new(e),
            })
        })
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }
}

/// Adapter for reqwest HTTP client
pub struct ReqwestAdapter {
    client: reqwest::Client,
    base_headers: HttpHeaders,
    timeout: Option<Duration>,
}

impl ReqwestAdapter {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Uveddi/1.0")
            .build()
            .map_err(|e| crate::error::UveddiError::Network(
                crate::error::NetworkError::HttpError {
                    url: "client_creation".to_string(),
                    source: Box::new(e),
                }
            ))?;

        Ok(Self {
            client,
            base_headers: HashMap::new(),
            timeout: Some(Duration::from_secs(30)),
        })
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            base_headers: HashMap::new(),
            timeout: None,
        }
    }

    fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.request(method, url);

        // Add base headers
        for (key, value) in &self.base_headers {
            request = request.header(key, value);
        }

        // Add timeout if specified
        if let Some(timeout) = self.timeout {
            request = request.timeout(timeout);
        }

        request
    }

    async fn execute_request(&self, request: reqwest::RequestBuilder) -> Result<HttpResponse> {
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                crate::error::UveddiError::Network(crate::error::NetworkError::Timeout {
                    url: e.url().map(|u| u.to_string()).unwrap_or_default(),
                })
            } else {
                crate::error::UveddiError::Network(crate::error::NetworkError::HttpError {
                    url: e.url().map(|u| u.to_string()).unwrap_or_default(),
                    source: Box::new(e),
                })
            }
        })?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect();

        let body = response.bytes().await.map_err(|e| {
            crate::error::UveddiError::Network(crate::error::NetworkError::HttpError {
                url: "response_body".to_string(),
                source: Box::new(e),
            })
        })?.to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

#[async_trait]
impl HttpClient for ReqwestAdapter {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        let request = self.build_request(reqwest::Method::GET, url);
        self.execute_request(request).await
    }

    async fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        let request = self.build_request(reqwest::Method::POST, url)
            .body(body.to_vec());
        self.execute_request(request).await
    }

    async fn put(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        let request = self.build_request(reqwest::Method::PUT, url)
            .body(body.to_vec());
        self.execute_request(request).await
    }

    async fn delete(&self, url: &str) -> Result<HttpResponse> {
        let request = self.build_request(reqwest::Method::DELETE, url);
        self.execute_request(request).await
    }

    async fn get_with_headers(&self, url: &str, headers: &HttpHeaders) -> Result<HttpResponse> {
        let mut request = self.build_request(reqwest::Method::GET, url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        self.execute_request(request).await
    }

    async fn post_json<T: serde::Serialize + Send>(&self, url: &str, data: &T) -> Result<HttpResponse> {
        let json_body = serde_json::to_vec(data).map_err(|e| {
            crate::error::UveddiError::Network(crate::error::NetworkError::HttpError {
                url: url.to_string(),
                source: Box::new(e),
            })
        })?;

        let request = self.build_request(reqwest::Method::POST, url)
            .header("Content-Type", "application/json")
            .body(json_body);

        self.execute_request(request).await
    }

    fn with_timeout(&self, timeout: Duration) -> Box<dyn HttpClient> {
        Box::new(Self {
            client: self.client.clone(),
            base_headers: self.base_headers.clone(),
            timeout: Some(timeout),
        })
    }

    fn with_header(&self, key: &str, value: &str) -> Box<dyn HttpClient> {
        let mut headers = self.base_headers.clone();
        headers.insert(key.to_string(), value.to_string());

        Box::new(Self {
            client: self.client.clone(),
            base_headers: headers,
            timeout: self.timeout,
        })
    }

    fn with_bearer_token(&self, token: &str) -> Box<dyn HttpClient> {
        self.with_header("Authorization", &format!("Bearer {}", token))
    }
}

/// Mock HTTP client for testing
#[cfg(test)]
pub struct MockHttpClient {
    responses: std::sync::Mutex<std::collections::HashMap<String, HttpResponse>>,
    request_log: std::sync::Mutex<Vec<MockRequest>>,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockRequest {
    pub method: String,
    pub url: String,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

#[cfg(test)]
impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: std::sync::Mutex::new(std::collections::HashMap::new()),
            request_log: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn expect_get(&self, url: &str, response: HttpResponse) {
        self.responses.lock().unwrap().insert(
            format!("GET {}", url),
            response,
        );
    }

    pub fn expect_post(&self, url: &str, response: HttpResponse) {
        self.responses.lock().unwrap().insert(
            format!("POST {}", url),
            response,
        );
    }

    pub fn get_requests(&self) -> Vec<MockRequest> {
        self.request_log.lock().unwrap().clone()
    }
}

#[cfg(test)]
#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request_log.lock().unwrap().push(MockRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        });

        self.responses
            .lock()
            .unwrap()
            .get(&format!("GET {}", url))
            .cloned()
            .ok_or_else(|| crate::error::UveddiError::Network(
                crate::error::NetworkError::HttpError {
                    url: url.to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Mock response not configured"
                    )),
                }
            ))
    }

    async fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        self.request_log.lock().unwrap().push(MockRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: body.to_vec(),
        });

        self.responses
            .lock()
            .unwrap()
            .get(&format!("POST {}", url))
            .cloned()
            .ok_or_else(|| crate::error::UveddiError::Network(
                crate::error::NetworkError::HttpError {
                    url: url.to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Mock response not configured"
                    )),
                }
            ))
    }

    // ... implement other methods similarly
}
```

### 3. Create Database Adapter

#### A. SQLx Database Adapter:
```rust
// src/adapters/database.rs
use crate::interfaces::DatabaseConnection;
use crate::error::{Result, DatabaseError, UveddiError};
use async_trait::async_trait;
use sqlx::{Pool, Sqlite, Postgres, Row};
use serde_json::Value;

/// Adapter for SQLx database operations
pub struct SqlxAdapter<DB> {
    pool: Pool<DB>,
}

impl SqlxAdapter<Sqlite> {
    pub async fn new_sqlite(connection_string: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(connection_string).await
            .map_err(|e| UveddiError::Database(DatabaseError::ConnectionFailed {
                details: e.to_string(),
            }))?;

        Ok(Self { pool })
    }
}

impl SqlxAdapter<Postgres> {
    pub async fn new_postgres(connection_string: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(connection_string).await
            .map_err(|e| UveddiError::Database(DatabaseError::ConnectionFailed {
                details: e.to_string(),
            }))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseConnection for SqlxAdapter<Sqlite> {
    async fn execute(&self, query: &str) -> Result<()> {
        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| UveddiError::Database(DatabaseError::QueryFailed {
                query: query.to_string(),
                source: Box::new(e),
            }))?;

        Ok(())
    }

    async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| UveddiError::Database(DatabaseError::QueryFailed {
                query: query.to_string(),
                source: Box::new(e),
            }))?;

        let mut results = Vec::new();
        for row in rows {
            // Convert SQLx row to JSON, then deserialize to T
            let json_value = self.row_to_json(&row)?;
            let item: T = serde_json::from_value(json_value)
                .map_err(|e| UveddiError::Database(DatabaseError::QueryFailed {
                    query: query.to_string(),
                    source: Box::new(e),
                }))?;
            results.push(item);
        }

        Ok(results)
    }

    async fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&dyn DatabaseConnection) -> Result<R> + Send,
    {
        let mut tx = self.pool.begin().await
            .map_err(|e| UveddiError::Database(DatabaseError::TransactionFailed(Box::new(e))))?;

        // Create a transaction adapter
        let tx_adapter = SqlxTransactionAdapter { tx: &mut tx };

        match f(&tx_adapter) {
            Ok(result) => {
                tx.commit().await
                    .map_err(|e| UveddiError::Database(DatabaseError::TransactionFailed(Box::new(e))))?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(|rollback_e| UveddiError::Database(DatabaseError::TransactionFailed(Box::new(rollback_e))))?;
                Err(e)
            }
        }
    }
}

impl SqlxAdapter<Sqlite> {
    fn row_to_json(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Value> {
        let mut json_obj = serde_json::Map::new();

        // This is a simplified implementation
        // In a real implementation, you'd iterate over columns and convert types
        for i in 0..row.len() {
            let column = row.column(i);
            let column_name = column.name().to_string();

            // Handle different SQLite types
            if let Ok(value) = row.try_get::<String, _>(i) {
                json_obj.insert(column_name, Value::String(value));
            } else if let Ok(value) = row.try_get::<i64, _>(i) {
                json_obj.insert(column_name, Value::Number(value.into()));
            } else if let Ok(value) = row.try_get::<f64, _>(i) {
                json_obj.insert(column_name, Value::Number(
                    serde_json::Number::from_f64(value).unwrap_or(0.into())
                ));
            } else if let Ok(value) = row.try_get::<bool, _>(i) {
                json_obj.insert(column_name, Value::Bool(value));
            } else {
                json_obj.insert(column_name, Value::Null);
            }
        }

        Ok(Value::Object(json_obj))
    }
}

/// Transaction adapter for SQLx
struct SqlxTransactionAdapter<'a> {
    tx: &'a mut sqlx::Transaction<'a, Sqlite>,
}

#[async_trait]
impl DatabaseConnection for SqlxTransactionAdapter<'_> {
    async fn execute(&self, query: &str) -> Result<()> {
        sqlx::query(query)
            .execute(&mut **self.tx)
            .await
            .map_err(|e| UveddiError::Database(DatabaseError::QueryFailed {
                query: query.to_string(),
                source: Box::new(e),
            }))?;

        Ok(())
    }

    async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        // Similar implementation to the main adapter
        todo!("Implement transaction query")
    }

    async fn transaction<F, R>(&self, _f: F) -> Result<R>
    where
        F: FnOnce(&dyn DatabaseConnection) -> Result<R> + Send,
    {
        // Nested transactions not supported in this adapter
        Err(UveddiError::Database(DatabaseError::TransactionFailed(
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Nested transactions not supported"
            ))
        )))
    }
}
```

### 4. Create AI Service Adapters

#### A. Multi-Provider AI Adapter:
```rust
// src/adapters/ai_service.rs
use crate::interfaces::AiService;
use crate::types::*;
use crate::error::Result;
use crate::adapters::HttpClient;
use async_trait::async_trait;
use std::sync::Arc;

/// Configuration for AI service providers
#[derive(Debug, Clone)]
pub struct AiServiceConfig {
    pub provider: AiProvider,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub enum AiProvider {
    Ollama,
    OpenAI,
    Anthropic,
    Mock,
}

/// Unified AI service adapter that supports multiple providers
pub struct UnifiedAiAdapter {
    provider: Box<dyn AiService>,
}

impl UnifiedAiAdapter {
    pub async fn new(config: AiServiceConfig, http_client: Arc<dyn HttpClient>) -> Result<Self> {
        let provider: Box<dyn AiService> = match config.provider {
            AiProvider::Ollama => Box::new(OllamaAdapter::new(config, http_client).await?),
            AiProvider::OpenAI => Box::new(OpenAiAdapter::new(config, http_client).await?),
            AiProvider::Anthropic => Box::new(AnthropicAdapter::new(config, http_client).await?),
            AiProvider::Mock => Box::new(MockAiAdapter::new()),
        };

        Ok(Self { provider })
    }
}

#[async_trait]
impl AiService for UnifiedAiAdapter {
    async fn analyze_with_ai(&self, analysis: &AnalysisResult) -> Result<AiInsights> {
        self.provider.analyze_with_ai(analysis).await
    }

    async fn get_recommendations(&self, findings: &[Finding]) -> Result<Vec<Recommendation>> {
        self.provider.get_recommendations(findings).await
    }

    fn available_models(&self) -> &[String] {
        self.provider.available_models()
    }
}

/// Ollama API adapter
struct OllamaAdapter {
    config: AiServiceConfig,
    http_client: Arc<dyn HttpClient>,
}

impl OllamaAdapter {
    async fn new(config: AiServiceConfig, http_client: Arc<dyn HttpClient>) -> Result<Self> {
        let adapter = Self { config, http_client };

        // Test connection
        adapter.health_check().await?;

        Ok(adapter)
    }

    async fn health_check(&self) -> Result<()> {
        let response = self.http_client
            .get(&format!("{}/api/tags", self.config.api_url))
            .await?;

        if !response.is_success() {
            return Err(crate::error::UveddiError::Network(
                crate::error::NetworkError::HttpError {
                    url: self.config.api_url.clone(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "Ollama service not available"
                    )),
                }
            ));
        }

        Ok(())
    }

    async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let request_body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false
        });

        let response = self.http_client
            .post_json(&format!("{}/api/generate", self.config.api_url), &request_body)
            .await?;

        if !response.is_success() {
            return Err(crate::error::UveddiError::Network(
                crate::error::NetworkError::HttpError {
                    url: self.config.api_url.clone(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Ollama API error: {}", response.status)
                    )),
                }
            ));
        }

        let response_data: serde_json::Value = response.json()?;
        let completion = response_data["response"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        Ok(completion)
    }
}

#[async_trait]
impl AiService for OllamaAdapter {
    async fn analyze_with_ai(&self, analysis: &AnalysisResult) -> Result<AiInsights> {
        let prompt = format!(
            "Analyze this code analysis result and provide insights:\n\n\
            Files analyzed: {}\n\
            Issues found: {}\n\
            Top issues:\n{}\n\n\
            Please provide:\n\
            1. Overall code quality assessment\n\
            2. Priority recommendations\n\
            3. Potential risks",
            analysis.summary.files_analyzed,
            analysis.findings.len(),
            analysis.findings.iter()
                .take(5)
                .map(|f| format!("- {}: {}", f.severity, f.message))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let ai_response = self.generate_completion(&prompt).await?;

        Ok(AiInsights {
            summary: ai_response.clone(),
            quality_score: self.extract_quality_score(&ai_response),
            risk_level: self.extract_risk_level(&ai_response),
            recommendations: self.extract_recommendations(&ai_response),
        })
    }

    async fn get_recommendations(&self, findings: &[Finding]) -> Result<Vec<Recommendation>> {
        let findings_text = findings.iter()
            .take(10) // Limit to prevent prompt overflow
            .map(|f| format!("- {}: {}", f.severity, f.message))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Given these code analysis findings, provide specific actionable recommendations:\n\n{}\n\n\
            Format each recommendation as:\n\
            PRIORITY: [High/Medium/Low]\n\
            ACTION: [specific action to take]\n\
            RATIONALE: [why this matters]",
            findings_text
        );

        let ai_response = self.generate_completion(&prompt).await?;
        Ok(self.parse_recommendations(&ai_response))
    }

    fn available_models(&self) -> &[String] {
        // This would be dynamically fetched from Ollama API
        &[self.config.model.clone()].leak()
    }
}

impl OllamaAdapter {
    fn extract_quality_score(&self, response: &str) -> f64 {
        // Simple heuristic to extract quality score from response
        if response.contains("excellent") || response.contains("high quality") {
            0.9
        } else if response.contains("good") || response.contains("acceptable") {
            0.7
        } else if response.contains("poor") || response.contains("needs improvement") {
            0.4
        } else {
            0.6 // Default
        }
    }

    fn extract_risk_level(&self, response: &str) -> RiskLevel {
        if response.contains("critical") || response.contains("high risk") {
            RiskLevel::High
        } else if response.contains("moderate") || response.contains("medium risk") {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn extract_recommendations(&self, response: &str) -> Vec<String> {
        // Simple parsing of numbered recommendations
        response.lines()
            .filter(|line| line.trim().starts_with(char::is_numeric))
            .map(|line| line.trim().to_string())
            .collect()
    }

    fn parse_recommendations(&self, response: &str) -> Vec<Recommendation> {
        // Parse structured recommendations from AI response
        let mut recommendations = Vec::new();
        let sections: Vec<&str> = response.split("PRIORITY:").collect();

        for section in sections.iter().skip(1) { // Skip first empty section
            if let Some(recommendation) = self.parse_single_recommendation(section) {
                recommendations.push(recommendation);
            }
        }

        recommendations
    }

    fn parse_single_recommendation(&self, section: &str) -> Option<Recommendation> {
        let lines: Vec<&str> = section.lines().collect();
        if lines.len() < 3 {
            return None;
        }

        let priority_line = lines[0].trim();
        let priority = if priority_line.contains("High") {
            Priority::High
        } else if priority_line.contains("Medium") {
            Priority::Medium
        } else {
            Priority::Low
        };

        let action = lines.iter()
            .find(|line| line.trim().starts_with("ACTION:"))
            .map(|line| line.trim().strip_prefix("ACTION:").unwrap_or("").trim())?;

        let rationale = lines.iter()
            .find(|line| line.trim().starts_with("RATIONALE:"))
            .map(|line| line.trim().strip_prefix("RATIONALE:").unwrap_or("").trim())
            .unwrap_or("");

        Some(Recommendation {
            priority,
            action: action.to_string(),
            rationale: rationale.to_string(),
            estimated_effort: EstimatedEffort::Unknown,
        })
    }
}

/// OpenAI API adapter
struct OpenAiAdapter {
    config: AiServiceConfig,
    http_client: Arc<dyn HttpClient>,
}

impl OpenAiAdapter {
    async fn new(config: AiServiceConfig, http_client: Arc<dyn HttpClient>) -> Result<Self> {
        Ok(Self { config, http_client })
    }
}

#[async_trait]
impl AiService for OpenAiAdapter {
    async fn analyze_with_ai(&self, analysis: &AnalysisResult) -> Result<AiInsights> {
        let client = self.http_client
            .with_bearer_token(self.config.api_key.as_ref().unwrap())
            .with_header("Content-Type", "application/json");

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [{
                "role": "system",
                "content": "You are a code analysis expert. Analyze the provided code analysis results and provide insights."
            }, {
                "role": "user",
                "content": format!("Analyze this code analysis result: {:?}", analysis.summary)
            }],
            "max_tokens": 1000
        });

        let response = client
            .post_json(&format!("{}/v1/chat/completions", self.config.api_url), &request_body)
            .await?;

        let response_data: serde_json::Value = response.json()?;
        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(AiInsights {
            summary: content.clone(),
            quality_score: 0.7, // Would be extracted from response
            risk_level: RiskLevel::Medium,
            recommendations: vec![content],
        })
    }

    async fn get_recommendations(&self, _findings: &[Finding]) -> Result<Vec<Recommendation>> {
        // Similar implementation to Ollama but using OpenAI API format
        todo!("Implement OpenAI recommendations")
    }

    fn available_models(&self) -> &[String] {
        &["gpt-3.5-turbo", "gpt-4"].leak()
    }
}

/// Mock AI adapter for testing
struct MockAiAdapter {
    responses: std::collections::HashMap<String, AiInsights>,
}

impl MockAiAdapter {
    fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }
}

#[async_trait]
impl AiService for MockAiAdapter {
    async fn analyze_with_ai(&self, _analysis: &AnalysisResult) -> Result<AiInsights> {
        Ok(AiInsights {
            summary: "Mock AI analysis completed".to_string(),
            quality_score: 0.8,
            risk_level: RiskLevel::Low,
            recommendations: vec!["Mock recommendation".to_string()],
        })
    }

    async fn get_recommendations(&self, _findings: &[Finding]) -> Result<Vec<Recommendation>> {
        Ok(vec![Recommendation {
            priority: Priority::Medium,
            action: "Mock recommendation action".to_string(),
            rationale: "Mock rationale".to_string(),
            estimated_effort: EstimatedEffort::Low,
        }])
    }

    fn available_models(&self) -> &[String] {
        &["mock-model"].leak()
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct AiInsights {
    pub summary: String,
    pub quality_score: f64,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub priority: Priority,
    pub action: String,
    pub rationale: String,
    pub estimated_effort: EstimatedEffort,
}

#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum EstimatedEffort {
    Low,      // < 1 day
    Medium,   // 1-3 days
    High,     // 1-2 weeks
    Unknown,
}
```

### 5. Create Cache Provider Adapter

#### A. Multi-Backend Cache Adapter:
```rust
// src/adapters/cache.rs
use crate::interfaces::CacheRepository;
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configuration for cache providers
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub provider: CacheProvider,
    pub connection_string: Option<String>,
    pub max_size: usize,
    pub default_ttl: Duration,
}

#[derive(Debug, Clone)]
pub enum CacheProvider {
    Memory,
    Redis,
    File,
    None,
}

/// Unified cache adapter supporting multiple backends
pub struct UnifiedCacheAdapter {
    backend: Box<dyn CacheRepository>,
}

impl UnifiedCacheAdapter {
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let backend: Box<dyn CacheRepository> = match config.provider {
            CacheProvider::Memory => Box::new(MemoryCacheAdapter::new(config)),
            CacheProvider::Redis => Box::new(RedisCacheAdapter::new(config).await?),
            CacheProvider::File => Box::new(FileCacheAdapter::new(config)?),
            CacheProvider::None => Box::new(NullCacheAdapter::new()),
        };

        Ok(Self { backend })
    }
}

#[async_trait]
impl CacheRepository for UnifiedCacheAdapter {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.backend.get(key).await
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.backend.set(key, value, ttl).await
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.backend.delete(key).await
    }

    async fn clear(&self) -> Result<()> {
        self.backend.clear().await
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.backend.exists(key).await
    }
}

/// In-memory cache adapter
struct MemoryCacheAdapter {
    store: Arc<Mutex<HashMap<String, CacheEntry>>>,
    max_size: usize,
    default_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<std::time::Instant>,
}

impl MemoryCacheAdapter {
    fn new(config: CacheConfig) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_size: config.max_size,
            default_ttl: config.default_ttl,
        }
    }

    fn cleanup_expired(&self) {
        let mut store = self.store.lock().unwrap();
        let now = std::time::Instant::now();

        store.retain(|_, entry| {
            entry.expires_at.map_or(true, |expires| expires > now)
        });
    }

    fn evict_if_needed(&self) {
        let mut store = self.store.lock().unwrap();

        while store.len() > self.max_size && !store.is_empty() {
            // Simple LRU: remove first entry (in practice, you'd track access times)
            if let Some(key) = store.keys().next().cloned() {
                store.remove(&key);
            }
        }
    }
}

#[async_trait]
impl CacheRepository for MemoryCacheAdapter {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.cleanup_expired();

        let store = self.store.lock().unwrap();

        if let Some(entry) = store.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if expires_at <= std::time::Instant::now() {
                    return Ok(None);
                }
            }

            let value: T = serde_json::from_slice(&entry.data)
                .map_err(|e| crate::error::UveddiError::Internal {
                    message: format!("Cache deserialization error: {}", e),
                })?;

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: serde::Serialize,
    {
        let data = serde_json::to_vec(value)
            .map_err(|e| crate::error::UveddiError::Internal {
                message: format!("Cache serialization error: {}", e),
            })?;

        let expires_at = ttl.or(Some(self.default_ttl))
            .map(|duration| std::time::Instant::now() + duration);

        let entry = CacheEntry { data, expires_at };

        {
            let mut store = self.store.lock().unwrap();
            store.insert(key.to_string(), entry);
        }

        self.evict_if_needed();
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.clear();
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.cleanup_expired();
        let store = self.store.lock().unwrap();

        if let Some(entry) = store.get(key) {
            if let Some(expires_at) = entry.expires_at {
                Ok(expires_at > std::time::Instant::now())
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

/// Redis cache adapter
struct RedisCacheAdapter {
    client: redis::Client,
    default_ttl: Duration,
}

impl RedisCacheAdapter {
    async fn new(config: CacheConfig) -> Result<Self> {
        let connection_string = config.connection_string
            .unwrap_or_else(|| "redis://localhost:6379".to_string());

        let client = redis::Client::open(connection_string)
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        // Test connection
        let mut conn = client.get_async_connection().await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        // Test with a ping
        redis::cmd("PING").query_async::<_, String>(&mut conn).await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        Ok(Self {
            client,
            default_ttl: config.default_ttl,
        })
    }
}

#[async_trait]
impl CacheRepository for RedisCacheAdapter {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        let data: Option<Vec<u8>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::QueryFailed {
                    query: format!("GET {}", key),
                    source: Box::new(e),
                }
            ))?;

        if let Some(bytes) = data {
            let value: T = serde_json::from_slice(&bytes)
                .map_err(|e| crate::error::UveddiError::Internal {
                    message: format!("Redis cache deserialization error: {}", e),
                })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: serde::Serialize,
    {
        let data = serde_json::to_vec(value)
            .map_err(|e| crate::error::UveddiError::Internal {
                message: format!("Redis cache serialization error: {}", e),
            })?;

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        let ttl_seconds = ttl.unwrap_or(self.default_ttl).as_secs();

        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(data)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::QueryFailed {
                    query: format!("SETEX {} {}", key, ttl_seconds),
                    source: Box::new(e),
                }
            ))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::QueryFailed {
                    query: format!("DEL {}", key),
                    source: Box::new(e),
                }
            ))?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        redis::cmd("FLUSHDB")
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::QueryFailed {
                    query: "FLUSHDB".to_string(),
                    source: Box::new(e),
                }
            ))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::ConnectionFailed {
                    details: e.to_string(),
                }
            ))?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::UveddiError::Database(
                crate::error::DatabaseError::QueryFailed {
                    query: format!("EXISTS {}", key),
                    source: Box::new(e),
                }
            ))?;

        Ok(exists)
    }
}

/// File-based cache adapter
struct FileCacheAdapter {
    cache_dir: std::path::PathBuf,
    default_ttl: Duration,
}

impl FileCacheAdapter {
    fn new(config: CacheConfig) -> Result<Self> {
        let cache_dir = std::path::PathBuf::from("./cache");
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            default_ttl: config.default_ttl,
        })
    }

    fn get_cache_path(&self, key: &str) -> std::path::PathBuf {
        // Use hash of key to avoid filesystem issues
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        self.cache_dir.join(format!("{:x}.cache", hash))
    }
}

#[async_trait]
impl CacheRepository for FileCacheAdapter {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let cache_path = self.get_cache_path(key);

        if !cache_path.exists() {
            return Ok(None);
        }

        // Check if file is expired based on modification time
        let metadata = tokio::fs::metadata(&cache_path).await?;
        let modified = metadata.modified()?;
        let age = modified.elapsed().unwrap_or(Duration::MAX);

        if age > self.default_ttl {
            tokio::fs::remove_file(&cache_path).await.ok(); // Ignore errors
            return Ok(None);
        }

        let data = tokio::fs::read(&cache_path).await?;
        let value: T = serde_json::from_slice(&data)
            .map_err(|e| crate::error::UveddiError::Internal {
                message: format!("File cache deserialization error: {}", e),
            })?;

        Ok(Some(value))
    }

    async fn set<T>(&self, key: &str, value: &T, _ttl: Option<Duration>) -> Result<()>
    where
        T: serde::Serialize,
    {
        let cache_path = self.get_cache_path(key);
        let data = serde_json::to_vec(value)
            .map_err(|e| crate::error::UveddiError::Internal {
                message: format!("File cache serialization error: {}", e),
            })?;

        tokio::fs::write(&cache_path, data).await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let cache_path = self.get_cache_path(key);
        tokio::fs::remove_file(&cache_path).await.ok(); // Ignore if file doesn't exist
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.cache_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "cache") {
                tokio::fs::remove_file(entry.path()).await.ok();
            }
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let cache_path = self.get_cache_path(key);
        Ok(cache_path.exists())
    }
}

/// Null cache adapter (no-op)
struct NullCacheAdapter;

impl NullCacheAdapter {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CacheRepository for NullCacheAdapter {
    async fn get<T>(&self, _key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(None)
    }

    async fn set<T>(&self, _key: &str, _value: &T, _ttl: Option<Duration>) -> Result<()>
    where
        T: serde::Serialize,
    {
        Ok(())
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        Ok(())
    }

    async fn exists(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }
}
```

### 6. Create Adapter Registry

#### A. Centralized Adapter Management:
```rust
// src/adapters/mod.rs
pub mod http_client;
pub mod database;
pub mod ai_service;
pub mod cache;

use crate::error::Result;
use std::sync::Arc;

pub use http_client::{HttpClient, ReqwestAdapter};
pub use database::SqlxAdapter;
pub use ai_service::{UnifiedAiAdapter, AiServiceConfig};
pub use cache::{UnifiedCacheAdapter, CacheConfig};

/// Central registry for creating and managing adapters
pub struct AdapterFactory {
    http_client: Arc<dyn HttpClient>,
}

impl AdapterFactory {
    pub fn new() -> Result<Self> {
        let http_client = Arc::new(ReqwestAdapter::new()?);

        Ok(Self { http_client })
    }

    pub fn http_client(&self) -> Arc<dyn HttpClient> {
        self.http_client.clone()
    }

    pub async fn create_database_adapter(&self, connection_string: &str) -> Result<impl crate::interfaces::DatabaseConnection> {
        if connection_string.starts_with("sqlite:") {
            SqlxAdapter::new_sqlite(connection_string).await
        } else if connection_string.starts_with("postgresql:") {
            SqlxAdapter::new_postgres(connection_string).await
        } else {
            Err(crate::error::UveddiError::Validation(
                crate::error::ValidationError::FieldError {
                    field: "connection_string".to_string(),
                    details: "Unsupported database type".to_string(),
                }
            ))
        }
    }

    pub async fn create_ai_adapter(&self, config: AiServiceConfig) -> Result<UnifiedAiAdapter> {
        UnifiedAiAdapter::new(config, self.http_client.clone()).await
    }

    pub async fn create_cache_adapter(&self, config: CacheConfig) -> Result<UnifiedCacheAdapter> {
        UnifiedCacheAdapter::new(config).await
    }

    #[cfg(test)]
    pub fn with_mock_http_client(http_client: Arc<dyn HttpClient>) -> Self {
        Self { http_client }
    }
}

impl Default for AdapterFactory {
    fn default() -> Self {
        Self::new().expect("Failed to create adapter factory")
    }
}
```

### 7. Integration with Service Container

#### A. Update Service Container to Use Adapters:
```rust
// Update src/container/mod.rs
impl ServiceContainer {
    pub async fn register_adapters(&mut self) -> Result<()> {
        let adapter_factory = AdapterFactory::new()?;

        // Register HTTP client
        self.register::<dyn HttpClient>(adapter_factory.http_client());

        // Register database adapter
        let db_config = self.get_config::<DatabaseConfig>("database");
        let db_adapter = adapter_factory.create_database_adapter(&db_config.connection_string).await?;
        self.register::<dyn DatabaseConnection>(db_adapter);

        // Register cache adapter
        let cache_config = self.get_config::<CacheConfig>("cache");
        let cache_adapter = adapter_factory.create_cache_adapter(cache_config).await?;
        self.register::<dyn CacheRepository>(cache_adapter);

        // Register AI adapter if configured
        #[cfg(feature = "ai-integration")]
        if let Ok(ai_config) = self.get_config::<AiServiceConfig>("ai") {
            let ai_adapter = adapter_factory.create_ai_adapter(ai_config).await?;
            self.register::<dyn AiService>(ai_adapter);
        }

        Ok(())
    }
}
```

### 8. Create Testing Support

#### A. Mock Adapter Factory:
```rust
// src/adapters/testing.rs
#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockAdapterFactory {
        http_client: Arc<dyn HttpClient>,
    }

    impl MockAdapterFactory {
        pub fn new() -> Self {
            Self {
                http_client: Arc::new(http_client::MockHttpClient::new()),
            }
        }

        pub fn with_http_responses(mut self, responses: Vec<(String, http_client::HttpResponse)>) -> Self {
            let mock_client = http_client::MockHttpClient::new();
            for (url, response) in responses {
                mock_client.expect_get(&url, response);
            }
            self.http_client = Arc::new(mock_client);
            self
        }
    }

    impl AdapterFactory {
        pub fn for_testing() -> Self {
            Self {
                http_client: Arc::new(http_client::MockHttpClient::new()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mocks::*;

    #[tokio::test]
    async fn test_adapter_factory() {
        let factory = AdapterFactory::for_testing();
        let http_client = factory.http_client();

        // Test adapter creation
        assert!(http_client.get("http://example.com").await.is_err()); // No mock response configured
    }
}
```

## Success Criteria
- [ ] External library dependencies abstracted behind adapters
- [ ] Consistent interfaces for similar functionality across different providers
- [ ] Easy to swap implementations (e.g., SQLite ↔ PostgreSQL)
- [ ] Comprehensive mock implementations for testing
- [ ] Adapters integrated with dependency injection system
- [ ] Clear separation between external API and internal interfaces
- [ ] Adapter configuration through builders

## External Integration Quality
- [ ] Adapters handle provider-specific error cases
- [ ] Consistent error handling across all adapters
- [ ] Provider-specific optimizations preserved
- [ ] Graceful degradation when services unavailable
- [ ] Connection pooling and resource management

## Verification Commands
```bash
# Test adapter functionality
cargo test adapters::

# Test external integrations
cargo test --test integration_adapters

# Test mock adapters
cargo test adapter_mocks

# Check adapter error handling
cargo test adapter_errors
```

## Completion Notes
_To be filled by AI developer:_
- Adapters implemented: ___
- External dependencies abstracted: ___
- Mock implementations created: ___
- Integration complexity reduced: ___
- Testing coverage: ___