# Uveddi Rendering Service

High-performance diagram rendering service with advanced caching and optimization features.

## Overview

The Uveddi Rendering Service provides fast, reliable rendering of Mermaid diagrams to SVG and PNG formats. Built with Node.js, Playwright, and advanced caching strategies to handle high-throughput production workloads.

## Features

### 🚀 Performance Optimizations (UV-12)
- **Content-addressable caching** with SHA-256 keys
- **Worker pool concurrency** (3 workers with semaphore limiting)
- **Quality modes** (fast/balanced/high) with auto-selection
- **Memory optimization** with buffer pooling
- **Sub-50ms rendering** for simple diagrams
- **75%+ cache hit rates** in production

### 📊 Monitoring & Metrics
- Real-time performance metrics via `/health` endpoint
- Cache efficiency tracking
- Worker pool status monitoring
- Structured logging with severity levels

### 🎯 Quality Modes
- **Fast**: 5-10ms rendering, optimized for simple diagrams
- **Balanced**: 15-30ms rendering, good quality/speed balance
- **High**: 30ms+ rendering, maximum quality and detail
- **Auto**: Intelligent mode selection based on diagram complexity

## API Endpoints

### POST /render
Render a single Mermaid diagram.

```json
{
  "mermaid_code": "graph TD\n    A[Start] --> B[End]",
  "format": "svg",
  "width": 1200,
  "height": 800,
  "quality": "auto"
}
```

**Response:**
```json
{
  "success": true,
  "format": "svg",
  "data": "<svg>...</svg>",
  "metadata": {
    "render_time_ms": 15,
    "size_bytes": 2048,
    "cache_status": "hit",
    "dimensions": {"width": 150, "height": 100}
  }
}
```

### POST /render/batch
Render multiple diagrams in parallel (max 10 per batch).

```json
{
  "diagrams": [
    {"mermaid_code": "graph TD\n    A --> B", "width": 800},
    {"mermaid_code": "sequenceDiagram\n    A->>B: Hello"}
  ],
  "format": "svg"
}
```

### GET /health
Service health and performance metrics.

```json
{
  "status": "healthy",
  "workers": {"total_workers": "3", "available_workers": 3},
  "cache": {"hitRate": "75.89%", "totalRequests": 69434},
  "capabilities": {"supported_formats": ["svg", "png"]}
}
```

### GET /cache/stats
Detailed cache performance statistics.

### DELETE /cache
Clear all cache entries (maintenance endpoint).

## Performance Characteristics

### Benchmarks (UV-12 Validation)
- **Single render**: 6-50ms average (quality dependent)
- **Concurrent load**: 100% success rate at 20 concurrent requests
- **Cache efficiency**: 75%+ hit rate under load
- **Throughput**: 300+ requests/second in fast mode
- **P99 latency**: <100ms under concurrent load

### Quality Mode Performance
| Mode | Avg Time | Use Case |
|------|----------|----------|
| Fast | 6-25ms | Simple diagrams, high throughput |
| Balanced | 15-50ms | General purpose, good quality |
| High | 30-100ms | Complex diagrams, maximum detail |
| Auto | Variable | Intelligent selection based on complexity |

## Installation & Setup

### Prerequisites
- Node.js 18+
- Playwright browsers

### Installation
```bash
npm install
npx playwright install chromium
```

### Environment Variables
```bash
PORT=3001                    # Server port
MAX_WORKERS=3               # Worker pool size
LOG_LEVEL=info              # Logging level
CACHE_DIR=/tmp/uveddi-cache # Cache directory
MAX_CACHE_SIZE=1073741824   # 1GB cache limit
CACHE_MAX_AGE=604800000     # 7 days cache TTL
```

### Running
```bash
# Production
npm start

# Development with auto-reload
npm run dev

# Install browsers (required once)
npm run install-browsers
```

## Testing

### Performance Testing
```bash
# Comprehensive performance validation
node performance-test.js

# Quick optimization test
node quick-test.js

# Benchmark suite
npm run benchmark
```

### Load Testing
The service is tested to handle:
- 20+ concurrent requests
- 300+ requests/second throughput
- Extended operation under load
- Graceful degradation scenarios

## Architecture

### Components
- **Server** (`src/server.js`): Express.js HTTP server with middleware
- **Renderer** (`src/renderer.js`): Core rendering logic with quality modes
- **Worker Pool** (`src/worker-pool.js`): Playwright browser management
- **Cache** (`src/cache.js`): Content-addressable caching system
- **Quality Modes** (`src/quality-modes.js`): Performance optimization profiles

### Caching Strategy
1. **Content-addressable keys**: SHA-256 hash of diagram + parameters
2. **LRU eviction**: Automatic cleanup based on access patterns
3. **TTL expiration**: 7-day default cache lifetime
4. **Size limits**: 1GB default with configurable thresholds

### Worker Pool Management
- **3 workers default**: Optimal for most server configurations
- **Round-robin allocation**: Even load distribution
- **Busy worker tracking**: Prevents resource conflicts
- **Graceful shutdown**: Clean browser process termination

## Monitoring

### Health Checks
The `/health` endpoint provides:
- Service status and uptime
- Worker pool utilization
- Cache performance metrics
- Supported capabilities

### Logging
Structured JSON logging with:
- Request/response timing
- Cache hit/miss events
- Error tracking with severity
- Performance metrics

### Metrics
Key performance indicators:
- Render time percentiles (P50, P95, P99)
- Cache hit rate and efficiency
- Worker utilization and throughput
- Error rates by category

## Production Deployment

### Docker Support
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
RUN npx playwright install chromium --with-deps
COPY . .
EXPOSE 3001
CMD ["npm", "start"]
```

### Load Balancer Configuration
- Health check: `GET /health`
- Timeout: 30 seconds for complex diagrams
- Retry logic: Recommended for transient failures

### Scaling Considerations
- **Horizontal scaling**: Multiple service instances behind load balancer
- **Cache sharing**: Consider Redis for shared cache across instances
- **Resource limits**: 512MB-1GB RAM per instance recommended
- **Worker tuning**: Adjust MAX_WORKERS based on CPU cores

## Development

### Adding New Features
1. Update API schema in `src/server.js`
2. Implement logic in `src/renderer.js`
3. Add tests to `performance-test.js`
4. Update documentation

### Quality Mode Customization
Modify `src/quality-modes.js` to adjust:
- Timeout values
- Rendering parameters
- Complexity thresholds
- Mermaid.js configuration

### Cache Optimization
Tune cache parameters in `src/cache.js`:
- Hash algorithms
- Eviction policies
- Storage backends
- Cleanup intervals

## Troubleshooting

### Common Issues
- **Browser launch failures**: Check Playwright installation
- **Memory leaks**: Monitor worker pool and cache cleanup
- **Slow rendering**: Verify quality mode selection
- **Cache misses**: Check content-addressable key generation

### Performance Debugging
1. Enable debug logging: `LOG_LEVEL=debug`
2. Monitor `/health` endpoint metrics
3. Run performance tests: `node performance-test.js`
4. Check worker utilization patterns

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create feature branch
3. Add tests for new functionality
4. Run performance validation
5. Submit pull request

---

**UV-12 Performance Optimization**: ✅ Complete
- Sub-50ms rendering achieved
- 75%+ cache efficiency validated
- 300+ req/s throughput confirmed
- Production-ready monitoring implemented