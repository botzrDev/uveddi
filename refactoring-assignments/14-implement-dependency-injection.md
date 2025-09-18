# Assignment 14: Implement Dependency Injection System

## Priority: HIGH
## Estimated Time: 4-6 hours
## Dependencies: Assignment 13 (Eliminate Circular Dependencies)

## Objective
Implement a comprehensive dependency injection system to reduce coupling and improve testability.

## Current Problem
- Components directly instantiate their dependencies (tight coupling)
- Difficult to test components in isolation
- Hard to swap implementations (e.g., mock database for testing)
- Circular dependencies not fully resolved without DI

## Tasks

### 1. Design Dependency Injection Architecture

#### A. Choose DI Pattern
Implement Service Locator pattern with trait objects for Rust compatibility:

```rust
// src/container/mod.rs
use std::sync::Arc;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct ServiceContainer {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(Arc::new(service)));
    }

    pub fn get<T: 'static>(&self) -> Option<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|service| service.downcast_ref::<Arc<T>>())
            .cloned()
    }

    pub fn resolve<T: 'static>(&self) -> Arc<T> {
        self.get::<T>()
            .expect(&format!("Service {} not registered", std::any::type_name::<T>()))
    }
}
```

### 2. Create Service Interfaces

#### A. Define Core Service Traits:
```rust
// src/services/traits.rs
use async_trait::async_trait;
use crate::types::*;

#[async_trait]
pub trait AnalysisService: Send + Sync {
    async fn analyze_project(&self, config: &AnalysisConfig) -> Result<AnalysisResult>;
    async fn analyze_file(&self, file_path: &Path) -> Result<FileAnalysis>;
    fn supported_languages(&self) -> &[Language];
}

#[async_trait]
pub trait DatabaseService: Send + Sync {
    async fn save_analysis(&self, analysis: &AnalysisResult) -> Result<String>;
    async fn load_analysis(&self, id: &str) -> Result<AnalysisResult>;
    async fn list_analyses(&self, filter: &AnalysisFilter) -> Result<Vec<AnalysisMetadata>>;
    async fn delete_analysis(&self, id: &str) -> Result<()>;
}

#[async_trait]
pub trait ReportService: Send + Sync {
    async fn generate_report(&self, analysis: &AnalysisResult, format: ReportFormat) -> Result<String>;
    async fn export_report(&self, report: &str, path: &Path) -> Result<()>;
    fn supported_formats(&self) -> &[ReportFormat];
}

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>;
    async fn invalidate(&self, pattern: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

pub trait ConfigService: Send + Sync {
    fn get_analysis_config(&self) -> &AnalysisConfig;
    fn get_database_config(&self) -> &DatabaseConfig;
    fn get_ai_config(&self) -> Option<&AiConfig>;
    fn validate_config(&self) -> Result<()>;
}

pub trait LoggingService: Send + Sync {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn debug(&self, message: &str);
}

#[cfg(feature = "ai-integration")]
#[async_trait]
pub trait AiService: Send + Sync {
    async fn analyze_with_ai(&self, analysis: &AnalysisResult) -> Result<AiInsights>;
    async fn get_recommendations(&self, findings: &[Finding]) -> Result<Vec<Recommendation>>;
    fn available_models(&self) -> &[String];
}
```

### 3. Implement Concrete Services

#### A. Analysis Service Implementation:
```rust
// src/services/analysis_service.rs
use crate::services::traits::*;
use crate::engine::AnalysisEngine;

pub struct DefaultAnalysisService {
    engine: Arc<AnalysisEngine>,
    cache: Arc<dyn CacheService>,
    config: Arc<dyn ConfigService>,
}

impl DefaultAnalysisService {
    pub fn new(
        engine: Arc<AnalysisEngine>,
        cache: Arc<dyn CacheService>,
        config: Arc<dyn ConfigService>,
    ) -> Self {
        Self { engine, cache, config }
    }
}

#[async_trait]
impl AnalysisService for DefaultAnalysisService {
    async fn analyze_project(&self, config: &AnalysisConfig) -> Result<AnalysisResult> {
        // Check cache first
        let cache_key = format!("analysis::{}", config.cache_key());
        if let Some(cached) = self.cache.get::<AnalysisResult>(&cache_key).await? {
            return Ok(cached);
        }

        // Perform analysis
        let result = self.engine.analyze(config).await?;

        // Cache result
        let ttl = Duration::from_secs(3600); // 1 hour
        self.cache.set(&cache_key, &result, Some(ttl)).await?;

        Ok(result)
    }

    async fn analyze_file(&self, file_path: &Path) -> Result<FileAnalysis> {
        self.engine.analyze_file(file_path).await
    }

    fn supported_languages(&self) -> &[Language] {
        self.engine.supported_languages()
    }
}
```

#### B. Database Service Implementation:
```rust
// src/services/database_service.rs
pub struct SqliteDatabaseService {
    pool: Arc<SqlitePool>,
    config: Arc<dyn ConfigService>,
}

impl SqliteDatabaseService {
    pub async fn new(config: Arc<dyn ConfigService>) -> Result<Self> {
        let db_config = config.get_database_config();
        let pool = SqlitePool::connect(&db_config.connection_string).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool: Arc::new(pool), config })
    }
}

#[async_trait]
impl DatabaseService for SqliteDatabaseService {
    async fn save_analysis(&self, analysis: &AnalysisResult) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let serialized = serde_json::to_string(analysis)?;

        sqlx::query!(
            "INSERT INTO analyses (id, data, created_at) VALUES (?, ?, ?)",
            id,
            serialized,
            chrono::Utc::now()
        )
        .execute(&*self.pool)
        .await?;

        Ok(id)
    }

    async fn load_analysis(&self, id: &str) -> Result<AnalysisResult> {
        let row = sqlx::query!(
            "SELECT data FROM analyses WHERE id = ?",
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        let analysis: AnalysisResult = serde_json::from_str(&row.data)?;
        Ok(analysis)
    }

    // ... other methods
}
```

### 4. Create Service Registration

#### A. Service Builder:
```rust
// src/container/builder.rs
pub struct ServiceContainerBuilder {
    container: ServiceContainer,
}

impl ServiceContainerBuilder {
    pub fn new() -> Self {
        Self {
            container: ServiceContainer::new(),
        }
    }

    pub async fn build(mut self) -> Result<ServiceContainer> {
        // Register services in dependency order
        self.register_config_service()?;
        self.register_logging_service()?;
        self.register_cache_service().await?;
        self.register_database_service().await?;
        self.register_analysis_service().await?;
        self.register_report_service().await?;

        #[cfg(feature = "ai-integration")]
        self.register_ai_service().await?;

        Ok(self.container)
    }

    fn register_config_service(&mut self) -> Result<()> {
        let config = DefaultConfigService::from_env()?;
        config.validate_config()?;
        self.container.register::<dyn ConfigService>(config);
        Ok(())
    }

    fn register_logging_service(&mut self) -> Result<()> {
        let config = self.container.resolve::<dyn ConfigService>();
        let logging = DefaultLoggingService::new(config.clone());
        self.container.register::<dyn LoggingService>(logging);
        Ok(())
    }

    async fn register_cache_service(&mut self) -> Result<()> {
        let config = self.container.resolve::<dyn ConfigService>();
        let cache = RedisCacheService::new(config.clone()).await?;
        self.container.register::<dyn CacheService>(cache);
        Ok(())
    }

    async fn register_database_service(&mut self) -> Result<()> {
        let config = self.container.resolve::<dyn ConfigService>();
        let database = SqliteDatabaseService::new(config.clone()).await?;
        self.container.register::<dyn DatabaseService>(database);
        Ok(())
    }

    async fn register_analysis_service(&mut self) -> Result<()> {
        let config = self.container.resolve::<dyn ConfigService>();
        let cache = self.container.resolve::<dyn CacheService>();

        let engine = AnalysisEngine::new(config.clone())?;
        let analysis = DefaultAnalysisService::new(
            Arc::new(engine),
            cache,
            config
        );

        self.container.register::<dyn AnalysisService>(analysis);
        Ok(())
    }

    async fn register_report_service(&mut self) -> Result<()> {
        let config = self.container.resolve::<dyn ConfigService>();
        let report = DefaultReportService::new(config.clone())?;
        self.container.register::<dyn ReportService>(report);
        Ok(())
    }

    #[cfg(feature = "ai-integration")]
    async fn register_ai_service(&mut self) -> Result<()> {
        let config = self.container.resolve::<dyn ConfigService>();
        if let Some(ai_config) = config.get_ai_config() {
            let ai = OllamaAiService::new(ai_config.clone()).await?;
            self.container.register::<dyn AiService>(ai);
        }
        Ok(())
    }
}
```

### 5. Update Application Layer

#### A. Refactor Application Orchestrator:
```rust
// src/application/orchestrator.rs
pub struct ApplicationOrchestrator {
    analysis: Arc<dyn AnalysisService>,
    database: Arc<dyn DatabaseService>,
    report: Arc<dyn ReportService>,
    logging: Arc<dyn LoggingService>,

    #[cfg(feature = "ai-integration")]
    ai: Option<Arc<dyn AiService>>,
}

impl ApplicationOrchestrator {
    pub fn new(container: &ServiceContainer) -> Self {
        Self {
            analysis: container.resolve::<dyn AnalysisService>(),
            database: container.resolve::<dyn DatabaseService>(),
            report: container.resolve::<dyn ReportService>(),
            logging: container.resolve::<dyn LoggingService>(),

            #[cfg(feature = "ai-integration")]
            ai: container.get::<dyn AiService>(),
        }
    }

    pub async fn run_analysis(&self, config: AnalysisConfig) -> Result<String> {
        self.logging.info("Starting analysis...");

        // Run analysis
        let result = self.analysis.analyze_project(&config).await?;

        // Enhance with AI if available
        #[cfg(feature = "ai-integration")]
        let enhanced_result = if let Some(ai) = &self.ai {
            let insights = ai.analyze_with_ai(&result).await?;
            result.with_ai_insights(insights)
        } else {
            result
        };

        #[cfg(not(feature = "ai-integration"))]
        let enhanced_result = result;

        // Save to database
        let analysis_id = self.database.save_analysis(&enhanced_result).await?;

        // Generate report
        let report = self.report.generate_report(&enhanced_result, config.output_format).await?;

        // Export if path specified
        if let Some(output_path) = config.output_path {
            self.report.export_report(&report, &output_path).await?;
        }

        self.logging.info(&format!("Analysis completed: {}", analysis_id));
        Ok(analysis_id)
    }
}
```

### 6. Create Testing Support

#### A. Mock Services:
```rust
// src/services/mocks.rs
#[cfg(test)]
pub struct MockAnalysisService {
    pub results: Arc<RwLock<HashMap<String, AnalysisResult>>>,
}

#[cfg(test)]
impl MockAnalysisService {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_result(mut self, config_key: &str, result: AnalysisResult) -> Self {
        self.results.write().unwrap().insert(config_key.to_string(), result);
        self
    }
}

#[cfg(test)]
#[async_trait]
impl AnalysisService for MockAnalysisService {
    async fn analyze_project(&self, config: &AnalysisConfig) -> Result<AnalysisResult> {
        let key = config.cache_key();
        self.results
            .read()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No mock result for config: {}", key))
    }

    async fn analyze_file(&self, _file_path: &Path) -> Result<FileAnalysis> {
        Ok(FileAnalysis::default())
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::Rust, Language::Python]
    }
}
```

#### B. Test Container Builder:
```rust
// src/container/test_builder.rs
#[cfg(test)]
pub struct TestContainerBuilder {
    container: ServiceContainer,
}

#[cfg(test)]
impl TestContainerBuilder {
    pub fn new() -> Self {
        Self {
            container: ServiceContainer::new(),
        }
    }

    pub fn with_mock_analysis(mut self, service: MockAnalysisService) -> Self {
        self.container.register::<dyn AnalysisService>(service);
        self
    }

    pub fn with_mock_database(mut self, service: MockDatabaseService) -> Self {
        self.container.register::<dyn DatabaseService>(service);
        self
    }

    pub fn build(self) -> ServiceContainer {
        self.container
    }
}
```

### 7. Update Main Application Entry Point

#### A. Update main.rs:
```rust
// src/main.rs
use crate::container::ServiceContainerBuilder;
use crate::application::ApplicationOrchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    // Build service container
    let container = ServiceContainerBuilder::new()
        .build()
        .await?;

    // Create orchestrator with injected dependencies
    let orchestrator = ApplicationOrchestrator::new(&container);

    // Parse CLI arguments
    let config = parse_cli_args()?;

    // Run analysis
    let result_id = orchestrator.run_analysis(config).await?;
    println!("Analysis completed: {}", result_id);

    Ok(())
}
```

### 8. Create Configuration Service

#### A. Environment-based Configuration:
```rust
// src/services/config_service.rs
pub struct DefaultConfigService {
    analysis: AnalysisConfig,
    database: DatabaseConfig,
    ai: Option<AiConfig>,
}

impl DefaultConfigService {
    pub fn from_env() -> Result<Self> {
        let analysis = AnalysisConfig {
            thread_pool_size: env::var("UVEDDI_THREADS")
                .unwrap_or_else(|_| num_cpus::get().to_string())
                .parse()?,
            max_file_size: env::var("UVEDDI_MAX_FILE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                .parse()?,
            // ... other config
        };

        let database = DatabaseConfig {
            connection_string: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:uveddi.db".to_string()),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
        };

        let ai = if env::var("ENABLE_AI").unwrap_or_default() == "true" {
            Some(AiConfig {
                provider: env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string()),
                api_url: env::var("AI_API_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
                model: env::var("AI_MODEL").unwrap_or_else(|_| "deepseek-coder:6.7b".to_string()),
            })
        } else {
            None
        };

        Ok(Self { analysis, database, ai })
    }
}

impl ConfigService for DefaultConfigService {
    fn get_analysis_config(&self) -> &AnalysisConfig {
        &self.analysis
    }

    fn get_database_config(&self) -> &DatabaseConfig {
        &self.database
    }

    fn get_ai_config(&self) -> Option<&AiConfig> {
        self.ai.as_ref()
    }

    fn validate_config(&self) -> Result<()> {
        // Validate configuration consistency
        if self.analysis.thread_pool_size == 0 {
            return Err(anyhow::anyhow!("Thread pool size must be > 0"));
        }

        // Validate database connection
        if self.database.connection_string.is_empty() {
            return Err(anyhow::anyhow!("Database connection string required"));
        }

        Ok(())
    }
}
```

### 9. Update Tests

#### A. Integration Tests with DI:
```rust
// tests/integration_tests.rs
use uveddi::container::TestContainerBuilder;
use uveddi::services::mocks::*;

#[tokio::test]
async fn test_analysis_workflow_with_mocks() {
    // Setup mock services
    let mock_analysis = MockAnalysisService::new()
        .with_result("test-config", create_test_analysis_result());

    let mock_database = MockDatabaseService::new();
    let mock_report = MockReportService::new();

    // Build test container
    let container = TestContainerBuilder::new()
        .with_mock_analysis(mock_analysis)
        .with_mock_database(mock_database)
        .with_mock_report(mock_report)
        .build();

    // Test orchestrator with mocked dependencies
    let orchestrator = ApplicationOrchestrator::new(&container);

    let config = AnalysisConfig::default();
    let result = orchestrator.run_analysis(config).await;

    assert!(result.is_ok());
}
```

### 10. Performance and Error Handling

#### A. Service Health Checks:
```rust
// src/services/health.rs
pub struct HealthCheckService {
    container: Arc<ServiceContainer>,
}

impl HealthCheckService {
    pub async fn check_all(&self) -> HashMap<String, HealthStatus> {
        let mut results = HashMap::new();

        // Check database health
        if let Some(db) = self.container.get::<dyn DatabaseService>() {
            results.insert("database".to_string(), self.check_database(&*db).await);
        }

        // Check cache health
        if let Some(cache) = self.container.get::<dyn CacheService>() {
            results.insert("cache".to_string(), self.check_cache(&*cache).await);
        }

        results
    }

    async fn check_database(&self, db: &dyn DatabaseService) -> HealthStatus {
        // Implement database health check
        match db.list_analyses(&AnalysisFilter::default()).await {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy(e.to_string()),
        }
    }
}
```

## Success Criteria
- [ ] All services implement dependency injection
- [ ] No direct instantiation of dependencies in business logic
- [ ] Components easily testable with mock implementations
- [ ] Clear service interfaces defined
- [ ] Configuration externalized and injectable
- [ ] Health checking system implemented
- [ ] All tests pass with new DI system

## Performance Considerations
- Service resolution overhead should be minimal
- Container initialization time acceptable
- Memory usage of service container reasonable

## Verification Commands
```bash
# Test with dependency injection
cargo test --all-features

# Test mock services
cargo test --test integration_tests

# Check service registration
cargo run -- --health-check

# Performance benchmarking
cargo bench service_container
```

## Completion Notes
_To be filled by AI developer:_
- Services implemented: ___
- DI container complexity: ___
- Test coverage with mocks: ___
- Performance impact: ___
- Configuration externalization: ___