# Developer Guide - Uveddi High-Performance Analysis Engine

## Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Development Setup](#development-setup)
3. [Core Components](#core-components)
4. [Cache System Development](#cache-system-development)
5. [API Development](#api-development)
6. [Testing Guidelines](#testing-guidelines)
7. [Performance Profiling](#performance-profiling)
8. [Contributing](#contributing)
9. [Release Process](#release-process)
10. [Extension Development](#extension-development)

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Uveddi Engine Architecture               │
├─────────────────────┬─────────────────────┬─────────────────┤
│   User Interface    │      API Layer      │    Management   │
│                     │                     │                 │
│ ┌─────────────────┐ │ ┌─────────────────┐ │ ┌─────────────┐ │
│ │ CLI Interface   │ │ │ REST API        │ │ │ Monitoring  │ │
│ │ TUI Interface   │ │ │ WebSocket API   │ │ │ Metrics     │ │
│ │ Web Dashboard   │ │ │ SDK Endpoints   │ │ │ Health      │ │
│ └─────────────────┘ │ └─────────────────┘ │ └─────────────┘ │
├─────────────────────┼─────────────────────┼─────────────────┤
│                     Analysis Engine Core                     │
├──────────────────────────────────────────────────────────────┤
│  Multi-Layer Intelligent Cache System                       │
│                                                              │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│ │ AST Cache   │ │Analysis Cache│ │ Graph Cache │ │File     │ │
│ │             │ │             │ │             │ │Watcher  │ │
│ │ • Parsing   │ │ • Results   │ │ • Relations │ │• Change │ │
│ │ • Tree-sit  │ │ • Metadata  │ │ • Queries   │ │  Events │ │
│ │ • Syntax    │ │ • Findings  │ │ • Indexes   │ │• Update │ │
│ └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├──────────────────────────────────────────────────────────────┤
│                    Foundation Layer                          │
│                                                              │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│ │ Tree-sitter │ │ Database    │ │ Knowledge   │ │Config   │ │
│ │ Parsers     │ │ Engine      │ │ Graph       │ │System   │ │
│ │             │ │             │ │             │ │         │ │
│ │ • Multi-lang│ │ • SQLite    │ │ • Relations │ │• TOML   │ │
│ │ • AST Gen   │ │ • Indexes   │ │ • Traversal │ │• Schema │ │
│ │ • Incremen. │ │ • Migration │ │ • Analytics │ │• Valid. │ │
│ └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### Core Design Principles

#### 1. **Cache-First Architecture**
Every operation in Uveddi is designed around intelligent caching:
- **AST Cache**: Parsed syntax trees with TTL-based eviction
- **Analysis Cache**: Computed analysis results with adaptive policies
- **Graph Cache**: Knowledge graph queries and relationships
- **File Watcher**: Incremental updates minimize re-analysis

#### 2. **Performance-Oriented Design**
- **Zero-copy operations** where possible
- **Memory-mapped files** for large datasets
- **Lock-free data structures** in hot paths
- **Adaptive algorithms** that learn from usage patterns

#### 3. **Modular and Extensible**
- **Trait-based abstractions** for easy extension
- **Plugin system** for custom analyzers
- **Configurable pipelines** for different use cases
- **Language-agnostic parsing** via Tree-sitter

### Technology Stack

#### Core Language
- **Rust 1.70+**: Systems programming language for performance and safety
- **Edition 2021**: Latest language features and optimizations

#### Key Dependencies
```toml
[dependencies]
# Web Framework & API
axum = "0.7"           # Modern async web framework
tower = "0.4"          # Service abstractions
tokio = { version = "1.0", features = ["full"] }

# Serialization & Data
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Database & Storage
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-native-tls"] }
rusqlite = "0.29"

# Caching & Memory
dashmap = "5.5"        # Concurrent hashmap
lru = "0.12"           # LRU cache implementation
moka = "0.12"          # High-performance cache

# Parsing & Analysis
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-python = "0.20"
tree-sitter-javascript = "0.20"

# CLI & TUI
clap = { version = "4.4", features = ["derive"] }
ratatui = "0.24"      # Terminal UI framework

# Async & Concurrency
futures = "0.3"
async-trait = "0.1"
parking_lot = "0.12"   # Fast mutex implementations

# Monitoring & Observability
tracing = "0.1"
tracing-subscriber = "0.3"
prometheus = "0.13"
```

## Development Setup

### Prerequisites

#### System Requirements
```bash
# Rust toolchain (required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Development tools
cargo install cargo-watch
cargo install cargo-nextest
cargo install cargo-audit
cargo install cargo-expand
cargo install flamegraph

# Platform-specific requirements
## Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev clang lld

## macOS
xcode-select --install
brew install llvm

## Additional tools
npm install -g @mermaid-js/mermaid-cli  # For diagram generation
```

### Repository Setup

```bash
# Clone repository
git clone https://github.com/example/uveddi.git
cd uveddi

# Configure development environment
cp .env.example .env
mkdir -p ~/.uveddi/{cache,logs,config}

# Install git hooks
cp scripts/git-hooks/* .git/hooks/
chmod +x .git/hooks/*

# Build development version
cargo build --features="dev-full,analysis-cache,ast-cache"

# Run tests to verify setup
cargo nextest run
```

### IDE Configuration

#### VS Code Setup
Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.features": [
        "dev-full",
        "analysis-cache",
        "ast-cache"
    ],
    "rust-analyzer.cargo.target": "x86_64-unknown-linux-gnu",
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.extraArgs": [
        "--", "-W", "clippy::all", "-W", "clippy::nursery"
    ],
    "editor.formatOnSave": true,
    "rust-analyzer.hover.actions.enable": true,
    "rust-analyzer.lens.enable": true
}
```

Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Uveddi",
            "cargo": {
                "args": [
                    "build",
                    "--bin=uveddi",
                    "--features=dev-full,analysis-cache,ast-cache"
                ],
                "filter": {
                    "name": "uveddi",
                    "kind": "bin"
                }
            },
            "args": ["serve", "--config", "configs/development.toml"],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug",
                "RUST_BACKTRACE": "1"
            }
        }
    ]
}
```

### Feature Flags

Uveddi uses feature flags for modular compilation:

#### Development Features
```bash
# Minimal development build
cargo build --features="dev-minimal"

# Core development features
cargo build --features="dev-core,analysis-cache"

# Full development build
cargo build --features="dev-full,analysis-cache,ast-cache,tui"
```

#### Production Features
```bash
# Production build
cargo build --release --features="production"

# Production with specific caches
cargo build --release --features="production,analysis-cache,graph-cache"
```

#### Feature Definitions
```toml
[features]
default = ["analysis-cache"]

# Development features
dev-minimal = []
dev-core = ["analysis-cache", "ast-cache"] 
dev-full = ["dev-core", "tui", "dashboard", "plugin-system"]

# Production features
production = ["analysis-cache", "ast-cache", "graph-cache", "optimization"]

# Cache implementations
analysis-cache = []
ast-cache = []
graph-cache = []

# Interface features
tui = ["ratatui", "crossterm"]
dashboard = ["axum", "tower", "tokio/full"]

# System features
plugin-system = ["wasmtime", "wit-bindgen"]
optimization = ["lto", "codegen-units"]
```

## Core Components

### Cache System Architecture

The cache system is the heart of Uveddi's performance, implementing a sophisticated multi-layer architecture:

#### Cache Layer Abstractions

```rust
// src/cache/mod.rs
use async_trait::async_trait;
use std::hash::Hash;

#[async_trait]
pub trait Cache<K, V>: Send + Sync 
where 
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Get value from cache
    async fn get(&self, key: &K) -> Option<V>;
    
    /// Insert value into cache
    async fn insert(&self, key: K, value: V);
    
    /// Remove value from cache
    async fn remove(&self, key: &K) -> Option<V>;
    
    /// Clear all entries
    async fn clear(&self);
    
    /// Get cache statistics
    async fn stats(&self) -> CacheStats;
    
    /// Get cache health information
    async fn health(&self) -> CacheHealth;
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_count: u64,
    pub miss_count: u64,
    pub entry_count: usize,
    pub memory_usage: usize,
    pub hit_rate: f64,
}

#[derive(Debug, Clone)]
pub struct CacheHealth {
    pub is_healthy: bool,
    pub memory_pressure: f64,
    pub last_eviction: Option<std::time::Instant>,
    pub error_count: u64,
}
```

#### AST Cache Implementation

```rust
// src/cache/ast_cache.rs
use super::{Cache, CacheStats, CacheHealth};
use crate::parsing::ast::AstNode;
use moka::future::Cache as MokaCache;
use std::path::PathBuf;

pub struct AstCache {
    cache: MokaCache<PathBuf, AstNode>,
    config: AstCacheConfig,
    metrics: Arc<AstCacheMetrics>,
}

#[derive(Debug, Clone)]
pub struct AstCacheConfig {
    pub max_entries: u64,
    pub max_memory_mb: u64,
    pub ttl_seconds: u64,
    pub eviction_policy: EvictionPolicy,
}

impl AstCache {
    pub fn new(config: AstCacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .weigher(|_key, value: &AstNode| value.memory_size() as u32)
            .eviction_listener(|key, value, cause| {
                tracing::debug!(
                    "AST evicted: path={:?}, size={}, cause={:?}",
                    key, value.memory_size(), cause
                );
            })
            .build();
            
        Self {
            cache,
            config,
            metrics: Arc::new(AstCacheMetrics::new()),
        }
    }
    
    pub async fn get_or_parse<F, Fut>(&self, path: PathBuf, parser: F) -> Result<AstNode, ParseError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<AstNode, ParseError>>,
    {
        if let Some(ast) = self.get(&path).await {
            self.metrics.record_hit();
            return Ok(ast);
        }
        
        self.metrics.record_miss();
        let ast = parser().await?;
        self.insert(path, ast.clone()).await;
        Ok(ast)
    }
}

#[async_trait]
impl Cache<PathBuf, AstNode> for AstCache {
    async fn get(&self, key: &PathBuf) -> Option<AstNode> {
        self.cache.get(key).await
    }
    
    async fn insert(&self, key: PathBuf, value: AstNode) {
        self.cache.insert(key, value).await
    }
    
    async fn remove(&self, key: &PathBuf) -> Option<AstNode> {
        self.cache.remove(key).await
    }
    
    async fn clear(&self) {
        self.cache.invalidate_all();
    }
    
    async fn stats(&self) -> CacheStats {
        let hit_count = self.metrics.hit_count();
        let miss_count = self.metrics.miss_count();
        let total_requests = hit_count + miss_count;
        
        CacheStats {
            hit_count,
            miss_count,
            entry_count: self.cache.entry_count() as usize,
            memory_usage: self.cache.weighted_size() as usize,
            hit_rate: if total_requests > 0 {
                hit_count as f64 / total_requests as f64
            } else {
                0.0
            },
        }
    }
    
    async fn health(&self) -> CacheHealth {
        let stats = self.stats().await;
        
        CacheHealth {
            is_healthy: stats.hit_rate > 0.7, // Healthy if >70% hit rate
            memory_pressure: stats.memory_usage as f64 / 
                (self.config.max_memory_mb * 1024 * 1024) as f64,
            last_eviction: None, // Could be tracked if needed
            error_count: 0,
        }
    }
}
```

#### Analysis Cache with Adaptive Eviction

```rust
// src/cache/analysis_cache.rs
use super::Cache;
use crate::analysis::{AnalysisResult, AnalysisRequest};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct AnalysisCache {
    entries: Arc<RwLock<HashMap<AnalysisRequest, CacheEntry>>>,
    config: AnalysisCacheConfig,
    eviction_strategy: Box<dyn EvictionStrategy>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    result: AnalysisResult,
    last_accessed: Instant,
    access_count: u64,
    priority_score: f64,
}

pub trait EvictionStrategy: Send + Sync {
    fn should_evict(&self, entry: &CacheEntry, config: &AnalysisCacheConfig) -> bool;
    fn calculate_priority(&self, entry: &CacheEntry) -> f64;
}

pub struct AdaptiveEvictionStrategy {
    access_weight: f64,
    recency_weight: f64,
    size_weight: f64,
}

impl EvictionStrategy for AdaptiveEvictionStrategy {
    fn should_evict(&self, entry: &CacheEntry, _config: &AnalysisCacheConfig) -> bool {
        // Adaptive algorithm considers access patterns
        let age_seconds = entry.last_accessed.elapsed().as_secs() as f64;
        let access_frequency = entry.access_count as f64 / (age_seconds / 3600.0); // per hour
        
        // Keep frequently accessed recent entries
        access_frequency < 0.1 && age_seconds > 3600.0
    }
    
    fn calculate_priority(&self, entry: &CacheEntry) -> f64 {
        let age_seconds = entry.last_accessed.elapsed().as_secs() as f64;
        let size_factor = entry.result.memory_size() as f64;
        
        // Higher priority = less likely to evict
        (self.access_weight * entry.access_count as f64) +
        (self.recency_weight * (1.0 / (age_seconds + 1.0))) -
        (self.size_weight * (size_factor / 1024.0)) // Size in KB
    }
}

impl AnalysisCache {
    pub fn new(config: AnalysisCacheConfig) -> Self {
        let eviction_strategy: Box<dyn EvictionStrategy> = match config.eviction_policy {
            EvictionPolicy::Adaptive => Box::new(AdaptiveEvictionStrategy {
                access_weight: 2.0,
                recency_weight: 1.5,
                size_weight: 0.5,
            }),
            EvictionPolicy::LRU => Box::new(LruEvictionStrategy::new()),
            EvictionPolicy::TTL => Box::new(TtlEvictionStrategy::new(config.ttl_seconds)),
        };
        
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            eviction_strategy,
        }
    }
    
    async fn maybe_evict(&self) {
        let entries = self.entries.read().await;
        let current_memory = entries.values()
            .map(|e| e.result.memory_size())
            .sum::<usize>();
            
        if current_memory > self.config.max_memory_mb * 1024 * 1024 {
            drop(entries); // Release read lock
            
            let mut entries = self.entries.write().await;
            
            // Calculate priorities for all entries
            let mut priorities: Vec<_> = entries.iter()
                .map(|(key, entry)| (key.clone(), self.eviction_strategy.calculate_priority(entry)))
                .collect();
                
            // Sort by priority (lowest first = evict first)
            priorities.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Evict lowest priority entries until memory is acceptable
            let target_memory = (self.config.max_memory_mb * 1024 * 1024 * 80) / 100; // 80% of max
            let mut current_memory = entries.values().map(|e| e.result.memory_size()).sum::<usize>();
            
            for (key, _priority) in priorities {
                if current_memory <= target_memory {
                    break;
                }
                
                if let Some(entry) = entries.remove(&key) {
                    current_memory -= entry.result.memory_size();
                    tracing::debug!("Evicted analysis cache entry: {:?}", key);
                }
            }
        }
    }
}

#[async_trait]
impl Cache<AnalysisRequest, AnalysisResult> for AnalysisCache {
    async fn get(&self, key: &AnalysisRequest) -> Option<AnalysisResult> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            Some(entry.result.clone())
        } else {
            None
        }
    }
    
    async fn insert(&self, key: AnalysisRequest, value: AnalysisResult) {
        let entry = CacheEntry {
            result: value,
            last_accessed: Instant::now(),
            access_count: 1,
            priority_score: 0.0,
        };
        
        {
            let mut entries = self.entries.write().await;
            entries.insert(key, entry);
        }
        
        // Check if eviction is needed
        self.maybe_evict().await;
    }
    
    async fn remove(&self, key: &AnalysisRequest) -> Option<AnalysisResult> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|entry| entry.result)
    }
    
    async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
    
    async fn stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        
        let entry_count = entries.len();
        let memory_usage = entries.values()
            .map(|e| e.result.memory_size())
            .sum::<usize>();
            
        // These would be tracked by metrics in a real implementation
        CacheStats {
            hit_count: 0, // Would be tracked separately
            miss_count: 0,
            entry_count,
            memory_usage,
            hit_rate: 0.0, // Would be calculated from hit/miss counters
        }
    }
    
    async fn health(&self) -> CacheHealth {
        let stats = self.stats().await;
        let max_memory = self.config.max_memory_mb * 1024 * 1024;
        
        CacheHealth {
            is_healthy: stats.memory_usage < max_memory,
            memory_pressure: stats.memory_usage as f64 / max_memory as f64,
            last_eviction: None,
            error_count: 0,
        }
    }
}
```

### File Watcher Integration

```rust
// src/cache/file_watcher.rs
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    event_tx: mpsc::UnboundedSender<FileEvent>,
    config: FileWatcherConfig,
}

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed(PathBuf), // From -> To path
}

impl FileWatcher {
    pub fn new(
        config: FileWatcherConfig,
        cache_manager: Arc<CacheManager>,
    ) -> Result<Self, FileWatcherError> {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let event_tx_clone = event_tx.clone();
        
        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        for path in event.paths {
                            let file_event = FileEvent {
                                path: path.clone(),
                                event_type: match event.kind {
                                    notify::EventKind::Create(_) => FileEventType::Created,
                                    notify::EventKind::Modify(_) => FileEventType::Modified,
                                    notify::EventKind::Remove(_) => FileEventType::Deleted,
                                    _ => FileEventType::Modified,
                                },
                                timestamp: Instant::now(),
                            };
                            
                            let _ = event_tx_clone.send(file_event);
                        }
                    }
                    Err(e) => tracing::error!("File watcher error: {:?}", e),
                }
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_millis(config.poll_interval_ms)),
        )?;
        
        // Spawn event processor
        let cache_manager_clone = cache_manager.clone();
        let debounce_ms = config.debounce_ms;
        
        tokio::spawn(async move {
            let mut debounce_map: HashMap<PathBuf, Instant> = HashMap::new();
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        debounce_map.insert(event.path.clone(), event.timestamp);
                    }
                    _ = interval.tick() => {
                        let now = Instant::now();
                        let mut to_process = Vec::new();
                        
                        // Find events that have been stable for debounce period
                        debounce_map.retain(|path, timestamp| {
                            if now.duration_since(*timestamp).as_millis() > debounce_ms as u128 {
                                to_process.push(path.clone());
                                false // Remove from map
                            } else {
                                true // Keep in map
                            }
                        });
                        
                        // Process stable events
                        for path in to_process {
                            cache_manager_clone.invalidate_path(&path).await;
                            tracing::debug!("Cache invalidated for path: {:?}", path);
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            watcher,
            event_tx,
            config,
        })
    }
    
    pub fn watch_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), FileWatcherError> {
        self.watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
        Ok(())
    }
}
```

### Cache Manager Orchestration

```rust
// src/cache/manager.rs
use super::{Cache, AstCache, AnalysisCache, GraphCache, FileWatcher};
use std::sync::Arc;

pub struct CacheManager {
    ast_cache: Arc<dyn Cache<PathBuf, AstNode>>,
    analysis_cache: Arc<dyn Cache<AnalysisRequest, AnalysisResult>>,
    graph_cache: Arc<dyn Cache<GraphQuery, GraphResult>>,
    file_watcher: Option<FileWatcher>,
    config: CacheManagerConfig,
    metrics: Arc<CacheManagerMetrics>,
}

impl CacheManager {
    pub async fn new(config: CacheManagerConfig) -> Result<Self, CacheError> {
        let ast_cache = Arc::new(AstCache::new(config.ast_cache_config.clone()));
        let analysis_cache = Arc::new(AnalysisCache::new(config.analysis_cache_config.clone()));
        let graph_cache = Arc::new(GraphCache::new(config.graph_cache_config.clone()));
        
        let metrics = Arc::new(CacheManagerMetrics::new());
        
        let file_watcher = if config.enable_file_watcher {
            let cache_manager_weak = Arc::downgrade(&Arc::new(Self {
                ast_cache: ast_cache.clone(),
                analysis_cache: analysis_cache.clone(),
                graph_cache: graph_cache.clone(),
                file_watcher: None,
                config: config.clone(),
                metrics: metrics.clone(),
            }));
            
            Some(FileWatcher::new(config.file_watcher_config, cache_manager_weak)?)
        } else {
            None
        };
        
        Ok(Self {
            ast_cache,
            analysis_cache,
            graph_cache,
            file_watcher,
            config,
            metrics,
        })
    }
    
    pub async fn invalidate_path(&self, path: &Path) {
        // Invalidate AST cache
        self.ast_cache.remove(&path.to_path_buf()).await;
        
        // Invalidate related analysis results
        // This would need to track which analyses depend on which files
        self.invalidate_related_analyses(path).await;
        
        self.metrics.record_invalidation();
        
        tracing::info!("Invalidated cache entries for path: {:?}", path);
    }
    
    async fn invalidate_related_analyses(&self, path: &Path) {
        // Implementation would track dependencies and invalidate
        // analysis results that depend on the changed file
    }
    
    pub async fn get_global_stats(&self) -> GlobalCacheStats {
        let ast_stats = self.ast_cache.stats().await;
        let analysis_stats = self.analysis_cache.stats().await;
        let graph_stats = self.graph_cache.stats().await;
        
        GlobalCacheStats {
            ast_cache: ast_stats,
            analysis_cache: analysis_stats,
            graph_cache: graph_stats,
            total_memory_mb: (ast_stats.memory_usage + 
                            analysis_stats.memory_usage + 
                            graph_stats.memory_usage) / (1024 * 1024),
            overall_hit_rate: {
                let total_hits = ast_stats.hit_count + 
                               analysis_stats.hit_count + 
                               graph_stats.hit_count;
                let total_requests = total_hits + 
                                   ast_stats.miss_count + 
                                   analysis_stats.miss_count + 
                                   graph_stats.miss_count;
                
                if total_requests > 0 {
                    total_hits as f64 / total_requests as f64
                } else {
                    0.0
                }
            },
        }
    }
    
    pub async fn health_check(&self) -> CacheManagerHealth {
        let ast_health = self.ast_cache.health().await;
        let analysis_health = self.analysis_cache.health().await;
        let graph_health = self.graph_cache.health().await;
        
        CacheManagerHealth {
            overall_healthy: ast_health.is_healthy && 
                           analysis_health.is_healthy && 
                           graph_health.is_healthy,
            ast_cache: ast_health,
            analysis_cache: analysis_health,
            graph_cache: graph_health,
            file_watcher_active: self.file_watcher.is_some(),
        }
    }
}
```

## API Development

### REST API Framework

The REST API is built with Axum for high performance and modern async handling:

```rust
// src/api/mod.rs
use axum::{
    routing::{get, post},
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
    Router,
};
use std::sync::Arc;

pub async fn create_app(state: AppState) -> Router {
    Router::new()
        // Health and metrics
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics_handler))
        
        // Analysis endpoints
        .route("/api/v1/analysis", post(start_analysis))
        .route("/api/v1/analysis/:id", get(get_analysis))
        .route("/api/v1/analysis/:id/status", get(get_analysis_status))
        .route("/api/v1/analysis/:id/results", get(get_analysis_results))
        
        // Cache management
        .route("/api/v1/cache/stats", get(cache_stats))
        .route("/api/v1/cache/health", get(cache_health))
        .route("/api/v1/cache/control", post(cache_control))
        .route("/api/v1/cache/warmup", post(cache_warmup))
        
        // Real-time features
        .route("/ws/analysis", get(analysis_websocket))
        .route("/ws/metrics", get(metrics_websocket))
        
        .with_state(Arc::new(state))
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(CompressionLayer::new())
                .layer(RequestIdLayer::new())
        )
}

#[derive(Clone)]
pub struct AppState {
    pub analysis_engine: Arc<AnalysisEngine>,
    pub cache_manager: Arc<CacheManager>,
    pub config: Arc<ServerConfig>,
    pub metrics: Arc<MetricsCollector>,
}
```

### Analysis API Implementation

```rust
// src/api/analysis.rs
use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AnalysisRequest {
    pub target: AnalysisTarget,
    pub options: AnalysisOptions,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AnalysisTarget {
    #[serde(rename = "path")]
    Path { path: String },
    #[serde(rename = "git")]
    Git { url: String, branch: Option<String> },
    #[serde(rename = "content")]
    Content { content: String, language: String },
}

#[derive(Debug, Deserialize)]
pub struct AnalysisOptions {
    pub language: Option<String>,
    pub analyzers: Option<Vec<String>>,
    pub cache_strategy: Option<CacheStrategy>,
    pub depth: Option<AnalysisDepth>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    pub id: Uuid,
    pub status: AnalysisStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn start_analysis(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AnalysisRequest>,
) -> Result<Json<AnalysisResponse>, StatusCode> {
    let analysis_id = Uuid::new_v4();
    
    // Validate request
    if !is_valid_analysis_request(&request) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Start analysis task
    let analysis_task = state.analysis_engine.start_analysis(analysis_id, request).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Record metrics
    state.metrics.record_analysis_started(&analysis_task);
    
    let response = AnalysisResponse {
        id: analysis_id,
        status: AnalysisStatus::Queued,
        created_at: chrono::Utc::now(),
        estimated_completion: analysis_task.estimated_completion(),
    };
    
    Ok(Json(response))
}

pub async fn get_analysis_results(
    State(state): State<Arc<AppState>>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Json<AnalysisResultResponse>, StatusCode> {
    let analysis = state.analysis_engine.get_analysis(analysis_id).await
        .ok_or(StatusCode::NOT_FOUND)?;
    
    match analysis.status {
        AnalysisStatus::Completed => {
            let results = analysis.results.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(AnalysisResultResponse {
                id: analysis_id,
                status: AnalysisStatus::Completed,
                results: Some(results),
                performance: analysis.performance_metrics,
                cache_stats: Some(state.cache_manager.get_analysis_cache_stats(analysis_id).await),
            }))
        },
        AnalysisStatus::Failed => Err(StatusCode::INTERNAL_SERVER_ERROR),
        _ => Ok(Json(AnalysisResultResponse {
            id: analysis_id,
            status: analysis.status,
            results: None,
            performance: analysis.performance_metrics,
            cache_stats: None,
        }))
    }
}
```

### WebSocket Implementation for Real-Time Updates

```rust
// src/api/websocket.rs
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;

pub async fn analysis_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_analysis_websocket(socket, state))
}

async fn handle_analysis_websocket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut analysis_updates = state.analysis_engine.subscribe_to_updates();
    let mut cache_updates = state.cache_manager.subscribe_to_metrics();
    
    // Handle incoming subscription requests
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Analysis progress updates
                update = analysis_updates.recv() => {
                    match update {
                        Ok(update) => {
                            let message = WebSocketMessage::AnalysisProgress {
                                analysis_id: update.analysis_id,
                                percentage: update.percentage,
                                current_phase: update.phase,
                                files_processed: update.files_processed,
                                cache_hit_rate: update.cache_hit_rate,
                            };
                            
                            if sender.send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&message).unwrap()
                            )).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                
                // Cache performance updates
                cache_update = cache_updates.recv() => {
                    match cache_update {
                        Ok(update) => {
                            let message = WebSocketMessage::CacheMetrics {
                                timestamp: chrono::Utc::now(),
                                ast_hit_rate: update.ast_hit_rate,
                                analysis_hit_rate: update.analysis_hit_rate,
                                memory_usage_mb: update.total_memory_mb,
                                speedup_factor: update.current_speedup,
                            };
                            
                            if sender.send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&message).unwrap()
                            )).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });
    
    // Handle incoming messages for subscriptions
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    if let Ok(subscription) = serde_json::from_str::<SubscriptionRequest>(&text) {
                        // Handle subscription logic
                        tracing::debug!("WebSocket subscription: {:?}", subscription);
                    }
                }
                Ok(axum::extract::ws::Message::Close(_)) => break,
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "analysis_progress")]
    AnalysisProgress {
        analysis_id: Uuid,
        percentage: f64,
        current_phase: String,
        files_processed: usize,
        cache_hit_rate: f64,
    },
    
    #[serde(rename = "cache_metrics")]
    CacheMetrics {
        timestamp: chrono::DateTime<chrono::Utc>,
        ast_hit_rate: f64,
        analysis_hit_rate: f64,
        memory_usage_mb: usize,
        speedup_factor: f64,
    },
    
    #[serde(rename = "analysis_complete")]
    AnalysisComplete {
        analysis_id: Uuid,
        duration_ms: u64,
        final_speedup: f64,
        results_summary: AnalysisResultsSummary,
    },
}
```

## Testing Guidelines

### Test Architecture

Uveddi follows a comprehensive testing strategy with different levels of testing:

```rust
// tests/integration/cache_integration.rs
use uveddi::{cache::CacheManager, config::CacheManagerConfig};
use tempfile::TempDir;
use std::time::Duration;

#[tokio::test]
async fn test_multi_layer_cache_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheManagerConfig {
        enable_file_watcher: true,
        cache_directory: temp_dir.path().to_path_buf(),
        ast_cache_config: AstCacheConfig {
            max_entries: 1000,
            max_memory_mb: 100,
            ttl_seconds: 300,
            eviction_policy: EvictionPolicy::LRU,
        },
        analysis_cache_config: AnalysisCacheConfig {
            max_entries: 500,
            max_memory_mb: 200,
            eviction_policy: EvictionPolicy::Adaptive,
        },
        ..Default::default()
    };
    
    let cache_manager = CacheManager::new(config).await.unwrap();
    
    // Test cache warming
    let test_project_path = create_test_project(&temp_dir);
    cache_manager.warm_cache(&test_project_path).await.unwrap();
    
    // Verify cache population
    let stats = cache_manager.get_global_stats().await;
    assert!(stats.ast_cache.entry_count > 0);
    
    // Test cache hit rates
    let analysis_request = create_test_analysis_request(&test_project_path);
    let start_time = Instant::now();
    let result1 = cache_manager.get_or_analyze(analysis_request.clone()).await.unwrap();
    let first_duration = start_time.elapsed();
    
    let start_time = Instant::now();
    let result2 = cache_manager.get_or_analyze(analysis_request.clone()).await.unwrap();
    let second_duration = start_time.elapsed();
    
    // Second call should be significantly faster due to cache
    assert!(second_duration < first_duration / 2);
    
    // Results should be identical
    assert_eq!(result1.analysis_id, result2.analysis_id);
    
    // Test file invalidation
    modify_test_file(&test_project_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await; // Wait for file watcher
    
    let start_time = Instant::now();
    let result3 = cache_manager.get_or_analyze(analysis_request.clone()).await.unwrap();
    let third_duration = start_time.elapsed();
    
    // Third call should be slower as cache was invalidated
    assert!(third_duration > second_duration);
    assert_ne!(result1.analysis_id, result3.analysis_id); // New analysis
}

#[tokio::test]
async fn test_cache_pressure_and_eviction() {
    let config = create_memory_constrained_config();
    let cache_manager = CacheManager::new(config).await.unwrap();
    
    // Fill cache beyond memory limit
    for i in 0..1000 {
        let request = create_large_analysis_request(i);
        cache_manager.get_or_analyze(request).await.unwrap();
    }
    
    // Verify eviction occurred
    let stats = cache_manager.get_global_stats().await;
    assert!(stats.total_memory_mb < 500); // Should be under limit
    
    // Verify health check passes
    let health = cache_manager.health_check().await;
    assert!(health.overall_healthy);
}

#[tokio::test]
async fn test_adaptive_eviction_strategy() {
    let cache_manager = create_cache_with_adaptive_eviction().await;
    
    // Create different access patterns
    let frequently_accessed = create_test_analysis_request("frequent");
    let rarely_accessed = create_test_analysis_request("rare");
    
    // Access frequently_accessed multiple times
    for _ in 0..10 {
        cache_manager.get_or_analyze(frequently_accessed.clone()).await.unwrap();
    }
    
    // Access rarely_accessed once
    cache_manager.get_or_analyze(rarely_accessed.clone()).await.unwrap();
    
    // Fill cache to trigger eviction
    fill_cache_to_capacity(&cache_manager).await;
    
    // Frequently accessed should still be in cache
    let start_time = Instant::now();
    cache_manager.get_or_analyze(frequently_accessed.clone()).await.unwrap();
    let frequent_duration = start_time.elapsed();
    
    // Rarely accessed should have been evicted
    let start_time = Instant::now();
    cache_manager.get_or_analyze(rarely_accessed.clone()).await.unwrap();
    let rare_duration = start_time.elapsed();
    
    assert!(frequent_duration < rare_duration);
}
```

### Performance Testing

```rust
// tests/performance/cache_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uveddi::cache::CacheManager;

fn benchmark_cache_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache_manager = rt.block_on(async {
        CacheManager::new(create_performance_test_config()).await.unwrap()
    });
    
    let mut group = c.benchmark_group("cache_operations");
    
    // Benchmark cache hits
    group.bench_function("cache_hit", |b| {
        let request = create_test_analysis_request("benchmark");
        rt.block_on(async {
            // Prime the cache
            cache_manager.get_or_analyze(request.clone()).await.unwrap();
        });
        
        b.iter(|| {
            rt.block_on(async {
                black_box(cache_manager.get_or_analyze(request.clone()).await.unwrap());
            })
        })
    });
    
    // Benchmark cache misses
    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            let request = create_unique_analysis_request();
            rt.block_on(async {
                black_box(cache_manager.get_or_analyze(request).await.unwrap());
            })
        })
    });
    
    // Benchmark eviction performance
    group.bench_function("eviction", |b| {
        b.iter(|| {
            rt.block_on(async {
                fill_cache_beyond_capacity(&cache_manager).await;
                black_box(cache_manager.force_eviction().await);
            })
        })
    });
    
    group.finish();
}

fn benchmark_speedup_measurement(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("speedup_comparison", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Measure analysis without cache
                let no_cache_manager = create_no_cache_manager().await;
                let start = Instant::now();
                no_cache_manager.analyze_project("test_project").await.unwrap();
                let no_cache_duration = start.elapsed();
                
                // Measure analysis with cache (second run)
                let cache_manager = create_full_cache_manager().await;
                cache_manager.analyze_project("test_project").await.unwrap(); // Prime cache
                let start = Instant::now();
                cache_manager.analyze_project("test_project").await.unwrap();
                let cached_duration = start.elapsed();
                
                let speedup = no_cache_duration.as_nanos() as f64 / cached_duration.as_nanos() as f64;
                black_box(speedup);
            })
        })
    });
}

criterion_group!(
    cache_benches,
    benchmark_cache_operations,
    benchmark_speedup_measurement
);
criterion_main!(cache_benches);
```

### Property-Based Testing

```rust
// tests/property/cache_properties.rs
use proptest::prelude::*;
use uveddi::{cache::CacheManager, analysis::AnalysisRequest};

proptest! {
    #[test]
    fn cache_consistency_property(
        requests in prop::collection::vec(arbitrary_analysis_request(), 1..100)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache_manager = CacheManager::new(create_test_config()).await.unwrap();
            
            for request in requests {
                // First analysis
                let result1 = cache_manager.get_or_analyze(request.clone()).await.unwrap();
                
                // Second analysis (should hit cache)
                let result2 = cache_manager.get_or_analyze(request.clone()).await.unwrap();
                
                // Results should be identical
                prop_assert_eq!(result1.checksum, result2.checksum);
                prop_assert_eq!(result1.findings.len(), result2.findings.len());
            }
        });
    }
    
    #[test]
    fn cache_memory_bounds_property(
        operations in prop::collection::vec(cache_operation(), 1..1000)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = CacheManagerConfig {
                max_total_memory_mb: 100, // Strict limit
                ..create_test_config()
            };
            let cache_manager = CacheManager::new(config).await.unwrap();
            
            for operation in operations {
                match operation {
                    CacheOperation::Insert(request) => {
                        cache_manager.get_or_analyze(request).await.unwrap();
                    }
                    CacheOperation::Clear => {
                        cache_manager.clear_all().await;
                    }
                }
                
                // Memory should never exceed limit
                let stats = cache_manager.get_global_stats().await;
                prop_assert!(stats.total_memory_mb <= 100);
            }
        });
    }
    
    #[test]
    fn cache_hit_rate_improvement_property(
        requests in prop::collection::vec(arbitrary_analysis_request(), 10..50)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache_manager = CacheManager::new(create_test_config()).await.unwrap();
            
            // First round - cold cache
            let start_stats = cache_manager.get_global_stats().await;
            for request in &requests {
                cache_manager.get_or_analyze(request.clone()).await.unwrap();
            }
            let first_round_stats = cache_manager.get_global_stats().await;
            
            // Second round - warm cache
            for request in &requests {
                cache_manager.get_or_analyze(request.clone()).await.unwrap();
            }
            let second_round_stats = cache_manager.get_global_stats().await;
            
            // Hit rate should improve significantly
            prop_assert!(second_round_stats.overall_hit_rate > 
                        first_round_stats.overall_hit_rate + 0.3);
        });
    }
}

fn arbitrary_analysis_request() -> impl Strategy<Value = AnalysisRequest> {
    (
        prop::string::string_regex(r"[a-zA-Z0-9_/]{1,100}").unwrap(),
        prop::option::of(prop::string::string_regex(r"rust|python|javascript").unwrap()),
        prop::collection::vec(prop::string::string_regex(r"[a-z_]{1,20}").unwrap(), 0..5)
    ).prop_map(|(path, language, analyzers)| {
        AnalysisRequest {
            target: AnalysisTarget::Path { path },
            options: AnalysisOptions {
                language,
                analyzers: if analyzers.is_empty() { None } else { Some(analyzers) },
                cache_strategy: Some(CacheStrategy::Adaptive),
                depth: Some(AnalysisDepth::Full),
                timeout_seconds: Some(300),
            }
        }
    })
}
```

## Performance Profiling

### Built-in Profiling Tools

```rust
// src/profiling/mod.rs
use std::time::{Duration, Instant};
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct PerformanceProfiler {
    measurements: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    active_timers: Arc<RwLock<HashMap<String, Instant>>>,
    enabled: bool,
}

impl PerformanceProfiler {
    pub fn new(enabled: bool) -> Self {
        Self {
            measurements: Arc::new(RwLock::new(HashMap::new())),
            active_timers: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }
    
    pub fn start_timing(&self, operation: &str) {
        if !self.enabled {
            return;
        }
        
        let mut timers = self.active_timers.write();
        timers.insert(operation.to_string(), Instant::now());
    }
    
    pub fn end_timing(&self, operation: &str) {
        if !self.enabled {
            return;
        }
        
        let mut timers = self.active_timers.write();
        if let Some(start_time) = timers.remove(operation) {
            let duration = start_time.elapsed();
            
            let mut measurements = self.measurements.write();
            measurements.entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }
    
    pub fn get_stats(&self, operation: &str) -> Option<PerformanceStats> {
        let measurements = self.measurements.read();
        let durations = measurements.get(operation)?;
        
        if durations.is_empty() {
            return None;
        }
        
        let total: Duration = durations.iter().sum();
        let count = durations.len();
        let average = total / count as u32;
        
        let mut sorted_durations = durations.clone();
        sorted_durations.sort();
        
        let median = sorted_durations[count / 2];
        let p95 = sorted_durations[(count * 95) / 100];
        let min = sorted_durations[0];
        let max = sorted_durations[count - 1];
        
        Some(PerformanceStats {
            operation: operation.to_string(),
            count,
            total_duration: total,
            average_duration: average,
            median_duration: median,
            p95_duration: p95,
            min_duration: min,
            max_duration: max,
        })
    }
    
    pub fn generate_report(&self) -> PerformanceReport {
        let measurements = self.measurements.read();
        let mut operations = Vec::new();
        
        for operation in measurements.keys() {
            if let Some(stats) = self.get_stats(operation) {
                operations.push(stats);
            }
        }
        
        operations.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        
        PerformanceReport {
            timestamp: chrono::Utc::now(),
            operations,
        }
    }
}

// Profiling macros for easy integration
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $operation:expr, $code:block) => {
        $profiler.start_timing($operation);
        let result = $code;
        $profiler.end_timing($operation);
        result
    };
}

// Integration with cache operations
impl CacheManager {
    pub async fn get_or_analyze_with_profiling(
        &self,
        request: AnalysisRequest,
        profiler: &PerformanceProfiler,
    ) -> Result<AnalysisResult, AnalysisError> {
        profile!(profiler, "cache_lookup", {
            if let Some(cached) = self.analysis_cache.get(&request).await {
                profiler.start_timing("cache_hit");
                let result = Ok(cached);
                profiler.end_timing("cache_hit");
                return result;
            }
        });
        
        profile!(profiler, "analysis_execution", {
            let result = self.execute_analysis(request.clone()).await?;
            
            profile!(profiler, "cache_store", {
                self.analysis_cache.insert(request, result.clone()).await;
            });
            
            Ok(result)
        })
    }
}
```

### Flame Graph Generation

```rust
// src/profiling/flamegraph.rs
use std::collections::HashMap;

pub struct FlameGraphProfiler {
    call_stack: Vec<String>,
    measurements: HashMap<Vec<String>, Duration>,
    start_times: HashMap<Vec<String>, Instant>,
}

impl FlameGraphProfiler {
    pub fn enter(&mut self, function: &str) {
        self.call_stack.push(function.to_string());
        self.start_times.insert(self.call_stack.clone(), Instant::now());
    }
    
    pub fn exit(&mut self, function: &str) {
        if let Some(expected) = self.call_stack.last() {
            assert_eq!(expected, function, "Mismatched function exit");
        }
        
        if let Some(start_time) = self.start_times.remove(&self.call_stack) {
            let duration = start_time.elapsed();
            *self.measurements.entry(self.call_stack.clone()).or_default() += duration;
        }
        
        self.call_stack.pop();
    }
    
    pub fn generate_flame_graph(&self) -> String {
        let mut output = String::new();
        
        for (stack, duration) in &self.measurements {
            let stack_str = stack.join(";");
            let duration_us = duration.as_micros();
            output.push_str(&format!("{} {}\n", stack_str, duration_us));
        }
        
        output
    }
}

// Macro for easy flame graph profiling
#[macro_export]
macro_rules! flame_profile {
    ($profiler:expr, $function:expr, $code:block) => {
        $profiler.enter($function);
        let result = $code;
        $profiler.exit($function);
        result
    };
}
```

This comprehensive developer guide provides the foundation for working with Uveddi's high-performance architecture. The modular design, comprehensive testing strategies, and built-in profiling tools ensure that the engine maintains its performance characteristics while remaining maintainable and extensible.