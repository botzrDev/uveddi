# API Documentation - Modernized Endpoints

## Overview

The Uveddi API has been completely modernized to expose the high-performance cached analysis engine through clean REST endpoints and real-time WebSocket streams. This API provides 2-30x performance improvements over traditional analysis tools.

## Base URL
- **REST API**: `http://localhost:3000/api/v1`
- **WebSocket**: `ws://localhost:3001/api/v1/stream`

## Authentication
Currently using bearer token authentication:
```http
Authorization: Bearer <your-api-token>
```

## Analysis APIs

### Start Analysis
Initiate a new analysis job with comprehensive cache configuration.

```http
POST /api/v1/analysis/start
```

**Request Body:**
```json
{
    "target": {
        "type": "path",
        "path": "/path/to/codebase"
    },
    "cache_config": {
        "enabled": true,
        "ast_cache_size_mb": 1024,
        "analysis_cache_size_mb": 2048,
        "graph_cache_size_mb": 512,
        "enable_metrics": true,
        "enable_file_watcher": true
    },
    "analysis_config": {
        "detectors": ["security", "performance", "maintainability", "complexity"],
        "include_patterns": ["**/*.rs", "**/*.py", "**/*.js", "**/*.ts"],
        "exclude_patterns": ["**/target/**", "**/node_modules/**", "**/.git/**"],
        "enable_knowledge_graph": true,
        "max_file_size_mb": 10
    },
    "output_config": {
        "format": "detailed_json",
        "include_source_snippets": true,
        "include_metrics": true,
        "include_graph_data": true
    }
}
```

**Response:**
```json
{
    "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "started",
    "estimated_duration_seconds": 120,
    "cache_enabled": true,
    "expected_speedup": "5-15x",
    "stream_url": "ws://localhost:3001/api/v1/stream/analysis/550e8400-e29b-41d4-a716-446655440000",
    "status_url": "/api/v1/analysis/550e8400-e29b-41d4-a716-446655440000/status",
    "created_at": "2025-09-25T10:30:00Z"
}
```

### Get Analysis Status
Check the real-time status of an analysis job with detailed cache performance metrics.

```http
GET /api/v1/analysis/{analysis_id}/status
```

**Response:**
```json
{
    "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "running",
    "progress": {
        "files_processed": 1250,
        "total_files": 2000,
        "percent_complete": 62.5,
        "current_phase": "detector_analysis",
        "phase_details": {
            "current_detector": "security_detector",
            "detectors_completed": ["complexity", "maintainability"],
            "detectors_remaining": ["security", "performance"]
        }
    },
    "performance": {
        "cache_hit_rate": 0.87,
        "current_speedup": 12.5,
        "baseline_estimated_time": 1800,
        "cached_estimated_time": 144,
        "time_saved_seconds": 1656,
        "files_per_second": 45.2,
        "estimated_time_remaining_seconds": 48
    },
    "cache_stats": {
        "ast_cache": {
            "hits": 1100,
            "misses": 150,
            "hit_rate": 0.88,
            "memory_usage_mb": 234
        },
        "analysis_cache": {
            "hits": 950,
            "misses": 300,
            "hit_rate": 0.76,
            "memory_usage_mb": 456
        },
        "graph_cache": {
            "hits": 45,
            "misses": 12,
            "hit_rate": 0.79,
            "memory_usage_mb": 89
        },
        "total_memory_usage_mb": 779
    },
    "current_activity": {
        "active_threads": 4,
        "queue_size": 23,
        "last_activity": "2025-09-25T10:35:42Z"
    }
}
```

### Get Analysis Results
Retrieve the complete analysis results with performance metadata.

```http
GET /api/v1/analysis/{analysis_id}/results
```

**Query Parameters:**
- `format`: `json` (default), `detailed_json`, `markdown`, `html`
- `include_source`: `true`/`false` - Include source code snippets
- `include_metrics`: `true`/`false` - Include performance metrics
- `include_graph`: `true`/`false` - Include knowledge graph data

**Response:**
```json
{
    "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "results": {
        "summary": {
            "files_analyzed": 2000,
            "total_issues": 156,
            "critical_issues": 12,
            "high_issues": 34,
            "medium_issues": 67,
            "low_issues": 43,
            "analysis_duration_seconds": 144,
            "baseline_duration_seconds": 1800,
            "speedup_achieved": 12.5
        },
        "issues_by_category": {
            "security": 45,
            "performance": 38,
            "maintainability": 52,
            "complexity": 21
        },
        "issues": [
            {
                "id": "SEC-001",
                "severity": "critical",
                "category": "security",
                "title": "SQL Injection Vulnerability",
                "description": "User input directly concatenated into SQL query",
                "file": "src/database/queries.rs",
                "line": 42,
                "column": 15,
                "source_snippet": "let query = format!(\"SELECT * FROM users WHERE id = {}\", user_input);",
                "recommendation": "Use parameterized queries to prevent SQL injection",
                "detector": "security_detector",
                "confidence": 0.95,
                "cache_hit": true
            }
        ],
        "knowledge_graph": {
            "nodes": 2450,
            "edges": 3876,
            "components": 23,
            "max_depth": 8,
            "cyclic_dependencies": 3,
            "visualization_data": {
                "layout": "force_directed",
                "clusters": [...],
                "critical_paths": [...]
            }
        }
    },
    "performance_metrics": {
        "cache_effectiveness": {
            "overall_hit_rate": 0.89,
            "time_saved_total": 1656,
            "time_saved_per_file": 0.83,
            "cost_savings_estimate": "$2,340"
        },
        "detailed_breakdown": {
            "parsing_time_saved": 1200,
            "analysis_time_saved": 456,
            "graph_construction_time_saved": 0
        }
    },
    "completed_at": "2025-09-25T10:37:24Z"
}
```

### Cancel Analysis
Stop a running analysis job gracefully.

```http
DELETE /api/v1/analysis/{analysis_id}
```

**Response:**
```json
{
    "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "cancelled",
    "cancellation_reason": "user_request",
    "partial_results_available": true,
    "cancelled_at": "2025-09-25T10:35:00Z"
}
```

## Cache Management APIs

### Get Cache Statistics
Comprehensive cache performance and usage statistics.

```http
GET /api/v1/cache/stats
```

**Query Parameters:**
- `detailed`: `true`/`false` - Include detailed breakdown
- `period`: `1h`, `24h`, `7d`, `30d` - Statistics period

**Response:**
```json
{
    "timestamp": "2025-09-25T10:30:00Z",
    "overall": {
        "total_memory_mb": 1456,
        "hit_rate": 0.89,
        "total_operations": 125000,
        "operations_per_second": 450.2,
        "average_response_time_ms": 0.8,
        "uptime_seconds": 86400
    },
    "by_type": {
        "ast_cache": {
            "entries": 8500,
            "memory_mb": 612,
            "hit_rate": 0.92,
            "hits": 78000,
            "misses": 7000,
            "evictions": 450,
            "average_get_time_ms": 0.3,
            "average_put_time_ms": 1.2
        },
        "analysis_cache": {
            "entries": 6200,
            "memory_mb": 745,
            "hit_rate": 0.85,
            "hits": 34000,
            "misses": 6000,
            "evictions": 234,
            "average_get_time_ms": 0.5,
            "average_put_time_ms": 2.1
        },
        "graph_cache": {
            "entries": 450,
            "memory_mb": 99,
            "hit_rate": 0.78,
            "hits": 1200,
            "misses": 340,
            "evictions": 12,
            "average_get_time_ms": 0.7,
            "average_put_time_ms": 3.5
        }
    },
    "file_watcher": {
        "active": true,
        "files_watched": 15670,
        "changes_detected": 45,
        "invalidations_triggered": 23,
        "last_check": "2025-09-25T10:29:55Z"
    },
    "recommendations": [
        "AST cache performing excellently (92% hit rate)",
        "Consider increasing analysis cache size by 25% for optimal performance",
        "File watcher successfully maintaining cache freshness",
        "Current configuration optimal for workload"
    ],
    "trends": {
        "hit_rate_trend": "stable",
        "memory_usage_trend": "increasing_slowly", 
        "performance_trend": "improving"
    }
}
```

### Cache Control Operations
Perform cache management operations.

```http
POST /api/v1/cache/control
```

**Request Body:**
```json
{
    "operation": "clear",
    "target": "analysis_cache",
    "reason": "configuration_change",
    "confirm": true
}
```

**Supported Operations:**
- `clear` - Clear all entries from specified cache
- `invalidate_path` - Invalidate specific file/directory
- `invalidate_detector` - Invalidate results for specific detector
- `warmup` - Pre-populate cache with common patterns
- `compact` - Compact cache storage
- `backup` - Create cache backup

**Response:**
```json
{
    "operation": "clear",
    "target": "analysis_cache",
    "status": "completed",
    "affected_entries": 6200,
    "memory_freed_mb": 745,
    "operation_duration_ms": 234,
    "timestamp": "2025-09-25T10:30:00Z"
}
```

### Cache Warmup
Pre-populate caches for improved performance.

```http
POST /api/v1/cache/warmup
```

**Request Body:**
```json
{
    "target": {
        "type": "path",
        "path": "/path/to/codebase"
    },
    "strategy": "common_patterns",
    "options": {
        "include_patterns": ["**/*.rs", "**/*.py"],
        "priority_files": ["src/main.rs", "src/lib.rs"],
        "max_files": 1000,
        "max_time_seconds": 300
    }
}
```

**Response:**
```json
{
    "warmup_id": "warmup_123",
    "status": "started",
    "estimated_duration_seconds": 180,
    "files_to_process": 856,
    "progress_url": "/api/v1/cache/warmup/warmup_123/status"
}
```

### Cache Health Check
Validate cache system health and configuration.

```http
GET /api/v1/cache/health
```

**Response:**
```json
{
    "overall_health": "healthy",
    "timestamp": "2025-09-25T10:30:00Z",
    "components": {
        "ast_cache": {
            "status": "healthy",
            "hit_rate_health": "excellent",
            "memory_health": "good", 
            "performance_health": "excellent"
        },
        "analysis_cache": {
            "status": "healthy",
            "hit_rate_health": "good",
            "memory_health": "warning",
            "performance_health": "good"
        },
        "file_watcher": {
            "status": "healthy",
            "responsiveness": "excellent",
            "error_rate": "none"
        }
    },
    "alerts": [
        {
            "level": "warning",
            "component": "analysis_cache",
            "message": "Memory usage approaching configured limit",
            "recommendation": "Consider increasing memory limit or enabling more aggressive eviction"
        }
    ],
    "performance_score": 92,
    "efficiency_score": 89
}
```

## Knowledge Graph APIs

### Query Graph
Execute high-performance graph queries with caching.

```http
POST /api/v1/graph/query
```

**Request Body:**
```json
{
    "query_type": "dependencies",
    "target": "src/main.rs",
    "options": {
        "depth": 3,
        "direction": "both",
        "include_types": ["imports", "function_calls", "data_flow", "inheritance"],
        "exclude_types": ["comments", "formatting"],
        "max_nodes": 500,
        "include_metadata": true
    },
    "cache_config": {
        "use_cache": true,
        "cache_ttl_seconds": 300,
        "force_refresh": false
    }
}
```

**Response:**
```json
{
    "query_id": "graph_query_456",
    "query_hash": "sha256:abc123...",
    "cache_hit": true,
    "nodes": [
        {
            "id": "src/main.rs",
            "type": "file",
            "name": "main.rs",
            "path": "src/main.rs",
            "metadata": {
                "lines_of_code": 245,
                "complexity_score": 3.2,
                "last_modified": "2025-09-25T09:15:00Z"
            }
        },
        {
            "id": "src/lib.rs::parse_config",
            "type": "function",
            "name": "parse_config", 
            "parent": "src/lib.rs",
            "metadata": {
                "visibility": "public",
                "parameters": 1,
                "return_type": "Result<Config, ConfigError>"
            }
        }
    ],
    "edges": [
        {
            "source": "src/main.rs",
            "target": "src/lib.rs::parse_config",
            "type": "function_call",
            "weight": 1.0,
            "metadata": {
                "call_count": 1,
                "line_number": 23
            }
        }
    ],
    "metadata": {
        "total_nodes": 45,
        "total_edges": 67,
        "depth_reached": 3,
        "generation_time_ms": 8,
        "cache_hit": true,
        "query_complexity": "medium"
    }
}
```

### Graph Visualization Data
Get optimized data for graph visualization with multiple layout options.

```http
POST /api/v1/graph/visualization
```

**Request Body:**
```json
{
    "source_query": {
        "query_type": "component_overview",
        "target": "/path/to/codebase"
    },
    "layout": {
        "algorithm": "force_directed",
        "options": {
            "iterations": 1000,
            "spring_length": 100,
            "spring_strength": 0.8,
            "damping": 0.9
        }
    },
    "rendering": {
        "max_nodes": 500,
        "min_edge_weight": 0.1,
        "cluster_threshold": 0.7,
        "include_labels": true,
        "color_scheme": "category",
        "size_metric": "complexity"
    },
    "cache_config": {
        "use_cache": true,
        "cache_ttl_seconds": 600
    }
}
```

**Response:**
```json
{
    "visualization_id": "viz_789",
    "layout": "force_directed",
    "cache_hit": false,
    "nodes": [
        {
            "id": "src/main.rs",
            "x": 150.5,
            "y": 200.3,
            "size": 12,
            "color": "#3498db",
            "label": "main.rs",
            "cluster": "core"
        }
    ],
    "edges": [
        {
            "source": "src/main.rs",
            "target": "src/lib.rs",
            "weight": 0.8,
            "color": "#95a5a6"
        }
    ],
    "clusters": [
        {
            "id": "core",
            "name": "Core Components",
            "color": "#3498db",
            "node_count": 12,
            "centroid": {"x": 200, "y": 300}
        }
    ],
    "metadata": {
        "total_nodes": 127,
        "total_edges": 203,
        "clusters": 8,
        "layout_time_ms": 45,
        "cache_hit": false,
        "complexity_score": 6.7
    }
}
```

### Graph Analytics
Get comprehensive analytics about code structure and relationships.

```http
GET /api/v1/graph/analytics
```

**Query Parameters:**
- `target`: Path to analyze
- `metrics`: Comma-separated list of metrics to include
- `format`: `json`, `dashboard`, `summary`

**Response:**
```json
{
    "target": "/path/to/codebase",
    "generated_at": "2025-09-25T10:30:00Z",
    "cache_hit": true,
    "overview": {
        "total_files": 2000,
        "total_functions": 8500,
        "total_classes": 450,
        "total_interfaces": 123,
        "total_dependencies": 1200
    },
    "complexity_analysis": {
        "average_cyclomatic_complexity": 3.2,
        "highest_complexity_file": "src/parser/complex_parser.rs",
        "highest_complexity_score": 12.5,
        "complexity_distribution": {
            "low": 1650,
            "medium": 300,
            "high": 45,
            "very_high": 5
        }
    },
    "dependency_analysis": {
        "total_external_dependencies": 45,
        "total_internal_dependencies": 1155,
        "circular_dependencies": 3,
        "dependency_depth": {
            "average": 4.2,
            "maximum": 12,
            "deepest_chain": ["src/main.rs", "src/lib.rs", "..."]
        }
    },
    "hotspot_analysis": {
        "most_connected_files": [
            {"file": "src/lib.rs", "connections": 45},
            {"file": "src/core/engine.rs", "connections": 38}
        ],
        "most_modified_files": [
            {"file": "src/api/handlers.rs", "change_frequency": 0.85}
        ]
    },
    "recommendations": [
        "Consider refactoring src/parser/complex_parser.rs (complexity: 12.5)",
        "Review circular dependencies in authentication module",
        "src/lib.rs has high coupling - consider splitting"
    ]
}
```

### Incremental Graph Update
Update graph data for specific files without full reconstruction.

```http
POST /api/v1/graph/update/{file_path}
```

**Request Body:**
```json
{
    "update_type": "file_modified",
    "change_metadata": {
        "modified_at": "2025-09-25T10:28:00Z",
        "content_hash": "sha256:def456...",
        "change_type": "content_modified"
    },
    "invalidation_strategy": "cascading",
    "rebuild_dependencies": true
}
```

**Response:**
```json
{
    "update_id": "update_321",
    "status": "completed",
    "affected_nodes": 12,
    "affected_edges": 23,
    "propagation_depth": 3,
    "update_time_ms": 156,
    "cache_invalidations": 8,
    "cache_updates": 15
}
```

## Real-time Streaming APIs

### Analysis Progress Stream
Real-time updates during analysis execution.

```
ws://localhost:3001/api/v1/stream/analysis/{analysis_id}
```

**Connection Parameters:**
- `update_frequency`: `realtime`, `1s`, `5s`, `30s`
- `include_cache_stats`: `true`/`false`
- `include_performance_metrics`: `true`/`false`

**Message Types:**

#### Progress Update
```json
{
    "type": "progress",
    "timestamp": "2025-09-25T10:30:15Z",
    "data": {
        "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
        "files_processed": 1500,
        "total_files": 2000,
        "percent_complete": 75.0,
        "current_phase": "detector_analysis",
        "current_file": "src/complex_module.rs",
        "phase_progress": {
            "detectors_completed": 3,
            "detectors_total": 4,
            "current_detector": "performance_detector"
        }
    }
}
```

#### Cache Statistics Update  
```json
{
    "type": "cache_stats",
    "timestamp": "2025-09-25T10:30:15Z",
    "data": {
        "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
        "overall_hit_rate": 0.89,
        "ast_hit_rate": 0.93,
        "analysis_hit_rate": 0.86,
        "graph_hit_rate": 0.81,
        "memory_usage_mb": 1234,
        "operations_per_second": 445.2,
        "current_speedup": 15.2,
        "time_saved_seconds": 1456
    }
}
```

#### Performance Metrics
```json
{
    "type": "performance",
    "timestamp": "2025-09-25T10:30:15Z", 
    "data": {
        "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
        "files_per_second": 48.3,
        "average_file_time_ms": 20.7,
        "baseline_estimate_remaining": 450,
        "cached_estimate_remaining": 32,
        "speedup_factor": 14.1,
        "efficiency_score": 0.94
    }
}
```

#### Issue Discovery
```json
{
    "type": "issue_found",
    "timestamp": "2025-09-25T10:30:15Z",
    "data": {
        "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
        "issue": {
            "id": "SEC-042",
            "severity": "high",
            "category": "security",
            "file": "src/auth/validator.rs",
            "line": 78,
            "title": "Weak Password Validation",
            "cache_hit": true
        },
        "running_totals": {
            "critical": 2,
            "high": 8,
            "medium": 23,
            "low": 45
        }
    }
}
```

#### Completion
```json
{
    "type": "completed",
    "timestamp": "2025-09-25T10:32:24Z",
    "data": {
        "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
        "status": "completed",
        "total_time_seconds": 144,
        "baseline_time_seconds": 1800,
        "speedup_achieved": 12.5,
        "files_analyzed": 2000,
        "total_issues": 156,
        "cache_effectiveness": {
            "overall_hit_rate": 0.89,
            "time_saved": 1656,
            "cost_savings": "$2,340"
        },
        "results_url": "/api/v1/analysis/550e8400-e29b-41d4-a716-446655440000/results"
    }
}
```

### Cache Statistics Stream
Real-time cache performance monitoring.

```
ws://localhost:3001/api/v1/stream/cache
```

**Message Types:**

#### Cache Metrics Update
```json
{
    "type": "metrics_update",
    "timestamp": "2025-09-25T10:30:00Z",
    "data": {
        "overall": {
            "hit_rate": 0.89,
            "memory_usage_mb": 1456,
            "operations_per_second": 428.5
        },
        "by_cache": {
            "ast": {"hit_rate": 0.93, "memory_mb": 612},
            "analysis": {"hit_rate": 0.86, "memory_mb": 745}, 
            "graph": {"hit_rate": 0.81, "memory_mb": 99}
        }
    }
}
```

#### File Watcher Events
```json
{
    "type": "file_watcher",
    "timestamp": "2025-09-25T10:30:05Z",
    "data": {
        "event": "files_changed",
        "files": ["src/main.rs", "src/lib.rs"],
        "invalidations_triggered": 8,
        "caches_affected": ["ast", "analysis"],
        "cleanup_time_ms": 23
    }
}
```

#### Performance Alerts
```json
{
    "type": "alert",
    "timestamp": "2025-09-25T10:30:10Z",
    "data": {
        "level": "warning",
        "category": "performance",
        "message": "Analysis cache hit rate dropped below 80%",
        "current_value": 0.78,
        "threshold": 0.80,
        "recommendation": "Consider increasing cache size or reviewing invalidation patterns"
    }
}
```

## Error Handling

### Standard Error Response
All endpoints return errors in a consistent format:

```json
{
    "error": {
        "code": "CACHE_UNAVAILABLE",
        "message": "Cache service is temporarily unavailable",
        "details": "Connection to cache backend failed after 3 retries",
        "timestamp": "2025-09-25T10:30:00Z",
        "request_id": "req_12345",
        "retry_after_seconds": 30
    }
}
```

### Common Error Codes

#### Analysis Errors
- `ANALYSIS_NOT_FOUND`: Analysis ID doesn't exist
- `ANALYSIS_IN_PROGRESS`: Operation not allowed while analysis running
- `INVALID_TARGET`: Target path/configuration invalid
- `PERMISSION_DENIED`: Insufficient permissions for target
- `RESOURCE_EXHAUSTED`: System resources insufficient

#### Cache Errors  
- `CACHE_UNAVAILABLE`: Cache service not available
- `CACHE_FULL`: Cache storage limit exceeded
- `CACHE_CORRUPTION`: Cache data integrity check failed
- `INVALID_CACHE_CONFIG`: Cache configuration invalid

#### Graph Errors
- `GRAPH_NOT_BUILT`: Knowledge graph not available
- `QUERY_TOO_COMPLEX`: Graph query exceeds complexity limits
- `VISUALIZATION_FAILED`: Graph layout generation failed

### Rate Limiting
Rate limits are enforced per API key:

```http
HTTP 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0  
X-RateLimit-Reset: 1632567890

{
    "error": {
        "code": "RATE_LIMIT_EXCEEDED",
        "message": "API rate limit exceeded",
        "retry_after_seconds": 60
    }
}
```

## SDK Examples

### JavaScript/Node.js
```javascript
const UveddiClient = require('@uveddi/client');

const client = new UveddiClient({
    apiUrl: 'http://localhost:3000/api/v1',
    websocketUrl: 'ws://localhost:3001/api/v1/stream',
    apiKey: process.env.UVEDDI_API_KEY
});

// Start analysis with caching
const analysis = await client.analysis.start({
    target: { type: 'path', path: './src' },
    cache_config: { enabled: true },
    analysis_config: { 
        detectors: ['security', 'performance'],
        enable_knowledge_graph: true
    }
});

// Stream progress updates
const stream = client.analysis.stream(analysis.analysis_id);
stream.on('progress', (data) => {
    console.log(`Progress: ${data.percent_complete}%`);
    console.log(`Speedup: ${data.current_speedup}x`);
});

stream.on('completed', async (data) => {
    const results = await client.analysis.getResults(analysis.analysis_id);
    console.log(`Analysis completed in ${data.total_time_seconds}s`);
    console.log(`Speedup achieved: ${data.speedup_achieved}x`);
});
```

### Python
```python
import asyncio
from uveddi_client import UveddiClient

client = UveddiClient(
    api_url='http://localhost:3000/api/v1',
    websocket_url='ws://localhost:3001/api/v1/stream',
    api_key=os.getenv('UVEDDI_API_KEY')
)

async def analyze_codebase():
    # Start analysis
    analysis = await client.analysis.start({
        'target': {'type': 'path', 'path': './src'},
        'cache_config': {'enabled': True},
        'analysis_config': {
            'detectors': ['security', 'performance'],
            'enable_knowledge_graph': True
        }
    })
    
    # Stream updates
    async for update in client.analysis.stream(analysis['analysis_id']):
        if update['type'] == 'progress':
            print(f"Progress: {update['data']['percent_complete']}%")
            print(f"Cache Hit Rate: {update['data']['cache_hit_rate'] * 100}%")
        elif update['type'] == 'completed':
            results = await client.analysis.get_results(analysis['analysis_id'])
            print(f"Speedup: {update['data']['speedup_achieved']}x")
            break

asyncio.run(analyze_codebase())
```

### Rust
```rust
use uveddi_client::{UveddiClient, AnalysisConfig, CacheConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = UveddiClient::new(
        "http://localhost:3000/api/v1",
        "ws://localhost:3001/api/v1/stream",
        std::env::var("UVEDDI_API_KEY")?
    ).await?;

    let analysis = client.analysis().start(AnalysisConfig {
        target: Target::Path("./src".into()),
        cache_config: CacheConfig { enabled: true, ..Default::default() },
        detectors: vec!["security".into(), "performance".into()],
        enable_knowledge_graph: true,
        ..Default::default()
    }).await?;

    let mut stream = client.analysis().stream(&analysis.analysis_id).await?;
    while let Some(update) = stream.next().await {
        match update.message_type.as_str() {
            "progress" => {
                println!("Progress: {}%", update.data["percent_complete"]);
                println!("Speedup: {}x", update.data["current_speedup"]);
            }
            "completed" => {
                let results = client.analysis().get_results(&analysis.analysis_id).await?;
                println!("Analysis completed with {}x speedup", 
                         update.data["speedup_achieved"]);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

---

This API provides comprehensive access to Uveddi's high-performance cached analysis engine with real-time streaming capabilities, detailed performance metrics, and intelligent cache management. The 2-30x performance improvements are exposed through clear metrics and cost savings calculations, making the value proposition evident to users.