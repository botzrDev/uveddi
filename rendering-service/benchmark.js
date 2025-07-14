#!/usr/bin/env node

/**
 * Performance Benchmark Suite for UV-78 Rendering Service
 * Tests the <100ms SVG rendering target and caching performance
 */

const axios = require('axios');
const fs = require('fs').promises;
const path = require('path');
const os = require('os');

const BASE_URL = process.env.RENDERING_SERVICE_URL || 'http://localhost:3001';
const BENCHMARK_ITERATIONS = 50;
const CONCURRENT_REQUESTS = 10;

// Test diagrams of varying complexity
const TEST_DIAGRAMS = {
  simple: `
    graph TD
      A[Start] --> B[Process]
      B --> C[End]
  `,
  medium: `
    graph TD
      A[User Request] --> B{Authentication}
      B -->|Valid| C[Load Dashboard]
      B -->|Invalid| D[Show Login]
      C --> E[Fetch Data]
      E --> F[Render Charts]
      F --> G[Display Results]
      D --> H[Validate Credentials]
      H -->|Success| C
      H -->|Failure| I[Show Error]
  `,
  complex: `
    graph TD
      subgraph "Frontend Layer"
        A[React App] --> B[Redux Store]
        B --> C[API Client]
      end
      
      subgraph "API Gateway"
        C --> D[Load Balancer]
        D --> E[Auth Service]
        D --> F[User Service]
        D --> G[Data Service]
      end
      
      subgraph "Backend Services"
        E --> H[(Auth DB)]
        F --> I[(User DB)]
        G --> J[(Analytics DB)]
        G --> K[Cache Layer]
        K --> L[(Redis)]
      end
      
      subgraph "External Services"
        G --> M[Payment API]
        G --> N[Email Service]
        F --> O[File Storage]
      end
  `,
  sequence: `
    sequenceDiagram
      participant U as User
      participant F as Frontend
      participant A as API Gateway
      participant S as Service
      participant D as Database
      
      U->>F: Login Request
      F->>A: POST /auth/login
      A->>S: Validate Credentials
      S->>D: Query User
      D-->>S: User Data
      S-->>A: JWT Token
      A-->>F: Auth Response
      F-->>U: Dashboard
  `
};

const LOAD_SCENARIOS = [
  { name: 'light', concurrent: 5, iterations: 10 },
  { name: 'normal', concurrent: 10, iterations: 50 },
  { name: 'peak', concurrent: 50, iterations: 200 },
  { name: 'stress', concurrent: 200, iterations: 500 }
];

class BenchmarkRunner {
  constructor() {
    this.results = {
      timestamp: new Date().toISOString(),
      service_url: BASE_URL,
      iterations: BENCHMARK_ITERATIONS,
      concurrent_requests: CONCURRENT_REQUESTS,
      tests: {}
    };
  }

  async runBenchmarks() {
    console.log('🚀 Starting UV-78 Performance Benchmark Suite');
    console.log(`Target: <100ms SVG rendering performance`);
    console.log(`Service: ${BASE_URL}`);
    console.log(`Iterations: ${BENCHMARK_ITERATIONS} per test`);
    console.log('=' * 60);

    // Check service health
    await this.checkServiceHealth();

    // Clear cache for clean testing
    await this.clearCache();

    // Run individual diagram tests
    for (const [name, diagram] of Object.entries(TEST_DIAGRAMS)) {
      console.log(`\n📊 Testing ${name} diagram...`);
      await this.testDiagram(name, diagram);
    }

    // Run cache performance tests
    console.log(`\n🗄️  Testing cache performance...`);
    await this.testCachePerformance();

    // Run concurrent load tests
    console.log(`\n⚡ Testing concurrent load (${CONCURRENT_REQUESTS} requests)...`);
    await this.testConcurrentLoad();

    // Scenario-based load tests
    for (const scenario of LOAD_SCENARIOS) {
      console.log(`\n🧪 Scenario: ${scenario.name} (${scenario.concurrent} concurrent, ${scenario.iterations} iterations)`);
      await this.runScenarioLoadTest(scenario);
    }

    // Generate report
    await this.generateReport();
  }

  async checkServiceHealth() {
    try {
      const response = await axios.get(`${BASE_URL}/health`);
      console.log('✅ Service health check passed');
      console.log(`   Workers: ${response.data.workers.total_workers} total, ${response.data.workers.available_workers} available`);
      console.log(`   Cache: ${response.data.cache.hitRate} hit rate`);
    } catch (error) {
      console.error('❌ Service health check failed:', error.message);
      process.exit(1);
    }
  }

  async clearCache() {
    try {
      await axios.delete(`${BASE_URL}/cache`);
      console.log('🗑️  Cache cleared for clean testing');
    } catch (error) {
      console.warn('⚠️  Could not clear cache:', error.message);
    }
  }

  async testDiagram(name, mermaidCode) {
    const times = [];
    const errors = [];

    for (let i = 0; i < BENCHMARK_ITERATIONS; i++) {
      try {
        const startTime = Date.now();
        
        const response = await axios.post(`${BASE_URL}/render`, {
          mermaid_code: mermaidCode,
          format: 'svg',
          width: 1200,
          height: 800
        });

        const totalTime = Date.now() - startTime;
        times.push(totalTime);

        if (i === 0) {
          console.log(`   First render: ${totalTime}ms (${response.data.metadata.cache_status})`);
        }

        // Log progress every 10 iterations
        if ((i + 1) % 10 === 0) {
          process.stdout.write(`   Progress: ${i + 1}/${BENCHMARK_ITERATIONS}\r`);
        }

      } catch (error) {
        errors.push({
          iteration: i,
          error: error.message,
          status: error.response?.status
        });
      }
    }

    const stats = this.calculateStats(times);
    const passRate = ((BENCHMARK_ITERATIONS - errors.length) / BENCHMARK_ITERATIONS * 100).toFixed(2);
    const under100ms = times.filter(t => t < 100).length;
    const under100msRate = (under100ms / times.length * 100).toFixed(2);

    console.log(`\n   Results for ${name}:`);
    console.log(`   ├─ Success rate: ${passRate}%`);
    console.log(`   ├─ <100ms target: ${under100msRate}% (${under100ms}/${times.length})`);
    console.log(`   ├─ Average: ${stats.mean.toFixed(2)}ms`);
    console.log(`   ├─ Median: ${stats.median.toFixed(2)}ms`);
    console.log(`   ├─ P95: ${stats.p95.toFixed(2)}ms`);
    console.log(`   ├─ P99: ${stats.p99.toFixed(2)}ms`);
    console.log(`   └─ Range: ${stats.min}ms - ${stats.max}ms`);

    this.results.tests[name] = {
      diagram_type: name,
      iterations: BENCHMARK_ITERATIONS,
      success_rate: parseFloat(passRate),
      under_100ms_rate: parseFloat(under100msRate),
      times: times,
      stats: stats,
      errors: errors
    };
  }

  async testCachePerformance() {
    const testDiagram = TEST_DIAGRAMS.simple;
    const times = { cold: [], warm: [] };

    // Test cold cache (first request)
    await this.clearCache();
    
    const coldStart = Date.now();
    await axios.post(`${BASE_URL}/render`, {
      mermaid_code: testDiagram,
      format: 'svg'
    });
    times.cold.push(Date.now() - coldStart);

    // Test warm cache (subsequent requests)
    for (let i = 0; i < 20; i++) {
      const warmStart = Date.now();
      await axios.post(`${BASE_URL}/render`, {
        mermaid_code: testDiagram,
        format: 'svg'
      });
      times.warm.push(Date.now() - warmStart);
    }

    const coldStats = this.calculateStats(times.cold);
    const warmStats = this.calculateStats(times.warm);
    const speedup = coldStats.mean / warmStats.mean;

    console.log(`   Cold cache: ${coldStats.mean.toFixed(2)}ms`);
    console.log(`   Warm cache: ${warmStats.mean.toFixed(2)}ms (avg)`);
    console.log(`   Cache speedup: ${speedup.toFixed(2)}x faster`);

    this.results.cache_performance = {
      cold_cache_ms: coldStats.mean,
      warm_cache_ms: warmStats.mean,
      speedup_factor: speedup,
      warm_times: times.warm
    };
  }

  async testConcurrentLoad() {
    const testDiagram = TEST_DIAGRAMS.medium;
    const promises = [];
    const startTime = Date.now();

    // Launch concurrent requests
    for (let i = 0; i < CONCURRENT_REQUESTS; i++) {
      promises.push(
        axios.post(`${BASE_URL}/render`, {
          mermaid_code: testDiagram,
          format: 'svg'
        }).then(response => ({
          success: true,
          time: Date.now() - startTime,
          cache_status: response.data.metadata.cache_status
        })).catch(error => ({
          success: false,
          error: error.message,
          status: error.response?.status
        }))
      );
    }

    const results = await Promise.all(promises);
    const totalTime = Date.now() - startTime;
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);

    console.log(`   Total time: ${totalTime}ms`);
    console.log(`   Successful: ${successful.length}/${CONCURRENT_REQUESTS}`);
    console.log(`   Failed: ${failed.length}/${CONCURRENT_REQUESTS}`);
    console.log(`   Throughput: ${(CONCURRENT_REQUESTS / totalTime * 1000).toFixed(2)} req/sec`);

    this.results.concurrent_load = {
      concurrent_requests: CONCURRENT_REQUESTS,
      total_time_ms: totalTime,
      successful_requests: successful.length,
      failed_requests: failed.length,
      throughput_req_per_sec: CONCURRENT_REQUESTS / totalTime * 1000,
      results: results
    };
  }

  async runScenarioLoadTest(scenario) {
    const times = [];
    const errors = [];
    const promises = [];
    const startTime = Date.now();
    for (let i = 0; i < scenario.concurrent; i++) {
      promises.push((async () => {
        for (let j = 0; j < scenario.iterations; j++) {
          try {
            const reqStart = Date.now();
            await axios.post(`${BASE_URL}/render`, {
              mermaid_code: TEST_DIAGRAMS.medium,
              format: 'svg'
            });
            times.push(Date.now() - reqStart);
          } catch (error) {
            errors.push({ iteration: j, error: error.message, status: error.response?.status });
          }
        }
      })());
    }
    await Promise.all(promises);
    const totalTime = Date.now() - startTime;
    const stats = this.calculateStats(times);

    // Resource monitoring
    const resourceStats = {
      cpu: os.loadavg(),
      memory: process.memoryUsage(),
      free_mem: os.freemem(),
      total_mem: os.totalmem(),
      timestamp: new Date().toISOString()
    };
    console.log(`   Scenario ${scenario.name}: ${times.length} requests, ${errors.length} errors, avg ${stats.mean.toFixed(2)}ms, total ${totalTime}ms`);
    this.results[`scenario_${scenario.name}`] = {
      scenario,
      times,
      errors,
      stats,
      total_time_ms: totalTime,
      resource_stats: resourceStats
    };
  }

  calculateStats(times) {
    if (times.length === 0) return null;

    const sorted = [...times].sort((a, b) => a - b);
    const sum = times.reduce((a, b) => a + b, 0);

    return {
      count: times.length,
      min: Math.min(...times),
      max: Math.max(...times),
      mean: sum / times.length,
      median: sorted[Math.floor(sorted.length / 2)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      p99: sorted[Math.floor(sorted.length * 0.99)]
    };
  }

  async generateReport() {
    const reportPath = path.join(__dirname, 'benchmark-report.json');
    
    // Add summary
    this.results.summary = {
      total_tests: Object.keys(this.results.tests).length,
      overall_success_rate: Object.values(this.results.tests)
        .reduce((sum, test) => sum + test.success_rate, 0) / Object.keys(this.results.tests).length,
      uv78_target_compliance: Object.values(this.results.tests)
        .reduce((sum, test) => sum + test.under_100ms_rate, 0) / Object.keys(this.results.tests).length,
      cache_enabled: true
    };
    
    // Calculate performance grade after uv78_target_compliance is set
    this.results.summary.performance_grade = this.calculatePerformanceGrade();

    await fs.writeFile(reportPath, JSON.stringify(this.results, null, 2));

    console.log('\n' + '=' * 60);
    console.log('📋 BENCHMARK SUMMARY');
    console.log('=' * 60);
    console.log(`Overall Success Rate: ${this.results.summary.overall_success_rate.toFixed(2)}%`);
    console.log(`UV-78 Target Compliance: ${this.results.summary.uv78_target_compliance.toFixed(2)}%`);
    console.log(`Performance Grade: ${this.results.summary.performance_grade}`);
    console.log(`Cache Speedup: ${this.results.cache_performance?.speedup_factor.toFixed(2)}x`);
    console.log(`Report saved: ${reportPath}`);

    // UV-78 specific validation
    if (this.results.summary.uv78_target_compliance >= 95) {
      console.log('✅ UV-78 PERFORMANCE TARGET ACHIEVED (<100ms SVG rendering)');
    } else {
      console.log('❌ UV-78 PERFORMANCE TARGET NOT MET');
    }
  }

  calculatePerformanceGrade() {
    const compliance = this.results.summary.uv78_target_compliance;
    if (compliance >= 95) return 'A';
    if (compliance >= 90) return 'B';
    if (compliance >= 80) return 'C';
    if (compliance >= 70) return 'D';
    return 'F';
  }
}

class PerformanceValidator {
  // UV-8: Validate memory, cache, and rendering optimizations
  async validateOptimizations() {
    // Run regression tests, collect metrics, compare to baseline
    // Optionally invoke renderer.getCacheStats(), workerPool.getStatus(), etc.
    // This is a stub for future automated validation
    console.log('Validating optimizations (UV-8)...');
  }
}

// Run benchmarks if called directly
if (require.main === module) {
  const runner = new BenchmarkRunner();
  runner.runBenchmarks().catch(error => {
    console.error('Benchmark failed:', error);
    process.exit(1);
  });
}

module.exports = BenchmarkRunner;