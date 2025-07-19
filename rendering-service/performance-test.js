#!/usr/bin/env node

/**
 * UV-12 Performance Validation Test Suite
 * Comprehensive testing for rendering service optimization
 */

const axios = require('axios');
const crypto = require('crypto');

const BASE_URL = 'http://localhost:3001';
const CONCURRENT_REQUESTS = 20;
const WARMUP_REQUESTS = 10;

// Test diagrams of varying complexity
const TEST_DIAGRAMS = {
  simple: `graph TD
    A[Start] --> B[End]`,
    
  medium: `graph TD
    A[Client] --> B[Load Balancer]
    B --> C[Web Server 1]
    B --> D[Web Server 2]
    C --> E[Database]
    D --> E
    E --> F[Cache]
    F --> G[Redis]
    G --> H[Monitoring]`,
    
  complex: `graph TD
    A[User] --> B[Authentication Service]
    B --> C{Valid User?}
    C -->|Yes| D[Authorization Service]
    C -->|No| E[Login Failed]
    D --> F{Has Permission?}
    F -->|Yes| G[API Gateway]
    F -->|No| H[Access Denied]
    G --> I[Microservice 1]
    G --> J[Microservice 2]
    G --> K[Microservice 3]
    I --> L[Database 1]
    J --> M[Database 2]
    K --> N[Database 3]
    L --> O[Cache Layer]
    M --> O
    N --> O
    O --> P[Response Aggregator]
    P --> Q[Rate Limiter]
    Q --> R[Response to User]`,
    
  sequence: `sequenceDiagram
    participant U as User
    participant A as Auth Service
    participant G as Gateway
    participant M as Microservice
    participant D as Database
    participant C as Cache
    
    U->>A: Login Request
    A->>A: Validate Credentials
    A-->>U: JWT Token
    U->>G: API Request + Token
    G->>A: Validate Token
    A-->>G: Token Valid
    G->>M: Forward Request
    M->>D: Query Data
    D-->>M: Return Data
    M->>C: Cache Result
    M-->>G: Response
    G-->>U: Final Response`
};

class PerformanceTestSuite {
  constructor() {
    this.results = {
      warmup: [],
      singleRequests: [],
      concurrentRequests: [],
      cacheTests: [],
      stressTests: []
    };
    this.startTime = Date.now();
  }

  async runAllTests() {
    console.log('🚀 Starting UV-12 Performance Validation Tests');
    console.log('================================================\n');

    try {
      // Check service health
      await this.checkServiceHealth();
      
      // Warmup phase
      await this.runWarmupTests();
      
      // Single request performance
      await this.runSingleRequestTests();
      
      // Concurrent load tests
      await this.runConcurrentTests();
      
      // Cache efficiency tests
      await this.runCacheTests();
      
      // Stress testing
      await this.runStressTests();
      
      // Generate final report
      await this.generateReport();
      
    } catch (error) {
      console.error('❌ Test suite failed:', error.message);
      process.exit(1);
    }
  }

  async checkServiceHealth() {
    console.log('🔍 Checking service health...');
    
    try {
      const response = await axios.get(`${BASE_URL}/health`);
      const health = response.data;
      
      console.log(`✅ Service is healthy`);
      console.log(`   Workers: ${health.workers.available_workers}/${health.workers.total_workers}`);
      console.log(`   Cache hit rate: ${health.cache.hitRate}`);
      console.log(`   Total requests: ${health.cache.totalRequests}`);
      console.log('');
      
    } catch (error) {
      throw new Error(`Service health check failed: ${error.message}`);
    }
  }

  async runWarmupTests() {
    console.log('🔥 Running warmup tests...');
    
    for (let i = 0; i < WARMUP_REQUESTS; i++) {
      try {
        const startTime = Date.now();
        await this.renderDiagram(TEST_DIAGRAMS.simple, 'svg');
        const duration = Date.now() - startTime;
        this.results.warmup.push(duration);
        
        process.stdout.write(`   Warmup ${i + 1}/${WARMUP_REQUESTS} - ${duration}ms\r`);
      } catch (error) {
        console.error(`Warmup ${i + 1} failed:`, error.message);
      }
    }
    
    const avgWarmup = this.results.warmup.reduce((a, b) => a + b, 0) / this.results.warmup.length;
    console.log(`\n✅ Warmup complete - Average: ${avgWarmup.toFixed(1)}ms\n`);
  }

  async runSingleRequestTests() {
    console.log('⚡ Testing single request performance...');
    
    const tests = [
      { name: 'Simple SVG', diagram: TEST_DIAGRAMS.simple, format: 'svg' },
      { name: 'Medium SVG', diagram: TEST_DIAGRAMS.medium, format: 'svg' },
      { name: 'Complex SVG', diagram: TEST_DIAGRAMS.complex, format: 'svg' },
      { name: 'Sequence SVG', diagram: TEST_DIAGRAMS.sequence, format: 'svg' },
      { name: 'Simple PNG', diagram: TEST_DIAGRAMS.simple, format: 'png' },
      { name: 'Medium PNG', diagram: TEST_DIAGRAMS.medium, format: 'png' }
    ];

    for (const test of tests) {
      const times = [];
      
      for (let i = 0; i < 5; i++) {
        const startTime = Date.now();
        const result = await this.renderDiagram(test.diagram, test.format);
        const duration = Date.now() - startTime;
        times.push({
          duration,
          cacheStatus: result.metadata.cache_status,
          size: result.metadata.size_bytes
        });
      }
      
      const avgTime = times.reduce((a, b) => a + b.duration, 0) / times.length;
      const avgSize = times.reduce((a, b) => a + b.size, 0) / times.length;
      
      this.results.singleRequests.push({
        name: test.name,
        avgTime,
        avgSize,
        times
      });
      
      console.log(`   ${test.name}: ${avgTime.toFixed(1)}ms (${(avgSize/1024).toFixed(1)}KB)`);
    }
    
    console.log('');
  }

  async runConcurrentTests() {
    console.log(`🚄 Testing concurrent load (${CONCURRENT_REQUESTS} requests)...`);
    
    const promises = [];
    const startTime = Date.now();
    
    for (let i = 0; i < CONCURRENT_REQUESTS; i++) {
      const diagram = i % 2 === 0 ? TEST_DIAGRAMS.medium : TEST_DIAGRAMS.complex;
      promises.push(this.timedRequest(diagram, 'svg', i));
    }
    
    const results = await Promise.allSettled(promises);
    const totalTime = Date.now() - startTime;
    
    const successful = results.filter(r => r.status === 'fulfilled').map(r => r.value);
    const failed = results.filter(r => r.status === 'rejected');
    
    const avgTime = successful.reduce((a, b) => a + b.duration, 0) / successful.length;
    const p95Time = this.calculatePercentile(successful.map(r => r.duration), 95);
    const p99Time = this.calculatePercentile(successful.map(r => r.duration), 99);
    
    this.results.concurrentRequests = {
      totalRequests: CONCURRENT_REQUESTS,
      successful: successful.length,
      failed: failed.length,
      totalTime,
      avgTime,
      p95Time,
      p99Time,
      throughput: (successful.length / totalTime * 1000).toFixed(2)
    };
    
    console.log(`✅ Concurrent test results:`);
    console.log(`   Success rate: ${(successful.length/CONCURRENT_REQUESTS*100).toFixed(1)}%`);
    console.log(`   Average time: ${avgTime.toFixed(1)}ms`);
    console.log(`   P95 time: ${p95Time.toFixed(1)}ms`);
    console.log(`   P99 time: ${p99Time.toFixed(1)}ms`);
    console.log(`   Throughput: ${this.results.concurrentRequests.throughput} req/s`);
    console.log('');
  }

  async runCacheTests() {
    console.log('💾 Testing cache efficiency...');
    
    // Test cache hit for same diagram
    const testDiagram = TEST_DIAGRAMS.medium;
    const cacheResults = [];
    
    for (let i = 0; i < 10; i++) {
      const startTime = Date.now();
      const result = await this.renderDiagram(testDiagram, 'svg');
      const duration = Date.now() - startTime;
      
      cacheResults.push({
        request: i + 1,
        duration,
        cacheStatus: result.metadata.cache_status
      });
    }
    
    const cacheHits = cacheResults.filter(r => r.cacheStatus === 'hit').length;
    const cacheMisses = cacheResults.filter(r => r.cacheStatus === 'miss').length;
    const avgCacheHitTime = cacheResults
      .filter(r => r.cacheStatus === 'hit')
      .reduce((a, b) => a + b.duration, 0) / cacheHits || 0;
    
    this.results.cacheTests = {
      totalRequests: cacheResults.length,
      hits: cacheHits,
      misses: cacheMisses,
      hitRate: (cacheHits / cacheResults.length * 100).toFixed(1),
      avgCacheHitTime
    };
    
    console.log(`✅ Cache test results:`);
    console.log(`   Hit rate: ${this.results.cacheTests.hitRate}% (${cacheHits}/${cacheResults.length})`);
    console.log(`   Avg cache hit time: ${avgCacheHitTime.toFixed(1)}ms`);
    console.log('');
  }

  async runStressTests() {
    console.log('🔥 Running stress tests...');
    
    const stressResults = [];
    const batchSize = 5;
    const batches = 4;
    
    for (let batch = 0; batch < batches; batch++) {
      const promises = [];
      const batchStart = Date.now();
      
      for (let i = 0; i < batchSize; i++) {
        const diagram = TEST_DIAGRAMS.complex;
        promises.push(this.timedRequest(diagram, 'svg', `batch${batch}-${i}`));
      }
      
      const results = await Promise.allSettled(promises);
      const batchTime = Date.now() - batchStart;
      const successful = results.filter(r => r.status === 'fulfilled').length;
      
      stressResults.push({
        batch: batch + 1,
        successful,
        total: batchSize,
        time: batchTime
      });
      
      console.log(`   Batch ${batch + 1}: ${successful}/${batchSize} successful in ${batchTime}ms`);
      
      // Brief pause between batches
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    this.results.stressTests = stressResults;
    console.log('');
  }

  async timedRequest(diagram, format, id) {
    const startTime = Date.now();
    try {
      const result = await this.renderDiagram(diagram, format);
      const duration = Date.now() - startTime;
      return {
        id,
        duration,
        success: true,
        cacheStatus: result.metadata.cache_status,
        size: result.metadata.size_bytes
      };
    } catch (error) {
      return {
        id,
        duration: Date.now() - startTime,
        success: false,
        error: error.message
      };
    }
  }

  async renderDiagram(mermaidCode, format = 'svg', width = 1200, height = 800) {
    const response = await axios.post(`${BASE_URL}/render`, {
      mermaid_code: mermaidCode,
      format,
      width,
      height
    }, {
      timeout: 30000
    });
    
    return response.data;
  }

  calculatePercentile(values, percentile) {
    const sorted = values.sort((a, b) => a - b);
    const index = Math.ceil(sorted.length * percentile / 100) - 1;
    return sorted[index];
  }

  async generateReport() {
    const totalTime = Date.now() - this.startTime;
    
    console.log('📊 PERFORMANCE VALIDATION REPORT');
    console.log('=================================\n');
    
    // Summary
    console.log('🎯 UV-12 SUCCESS CRITERIA VALIDATION:');
    
    // Single render: <50ms average
    const avgSingleRender = this.results.singleRequests
      .reduce((sum, test) => sum + test.avgTime, 0) / this.results.singleRequests.length;
    const singleRenderPass = avgSingleRender < 50;
    console.log(`   Single render <50ms avg: ${singleRenderPass ? '✅' : '❌'} (${avgSingleRender.toFixed(1)}ms)`);
    
    // Concurrent load: >80% success rate
    const concurrentSuccess = this.results.concurrentRequests.successful / this.results.concurrentRequests.totalRequests;
    const concurrentPass = concurrentSuccess >= 0.8;
    console.log(`   Concurrent >80% success: ${concurrentPass ? '✅' : '❌'} (${(concurrentSuccess*100).toFixed(1)}%)`);
    
    // Cache efficiency: >80% hit rate
    const cacheEfficiencyPass = parseFloat(this.results.cacheTests.hitRate) >= 80;
    console.log(`   Cache >80% hit rate: ${cacheEfficiencyPass ? '✅' : '❌'} (${this.results.cacheTests.hitRate}%)`);
    
    // P99 performance: <100ms
    const p99Pass = this.results.concurrentRequests.p99Time < 100;
    console.log(`   P99 <100ms: ${p99Pass ? '✅' : '❌'} (${this.results.concurrentRequests.p99Time.toFixed(1)}ms)`);
    
    console.log('\n📈 DETAILED METRICS:');
    console.log(`   Total test duration: ${(totalTime/1000).toFixed(1)}s`);
    console.log(`   Throughput: ${this.results.concurrentRequests.throughput} req/s`);
    console.log(`   Cache hit time: ${this.results.cacheTests.avgCacheHitTime.toFixed(1)}ms`);
    
    const overallPass = singleRenderPass && concurrentPass && cacheEfficiencyPass && p99Pass;
    
    console.log('\n🏆 OVERALL RESULT:');
    if (overallPass) {
      console.log('✅ UV-12 PERFORMANCE OPTIMIZATION: COMPLETE');
      console.log('   All success criteria met! 🎉');
    } else {
      console.log('❌ UV-12 PERFORMANCE OPTIMIZATION: NEEDS WORK');
      console.log('   Some criteria not met. See details above.');
    }
    
    console.log('\n📋 NEXT STEPS:');
    if (overallPass) {
      console.log('   • Deploy to production');
      console.log('   • Monitor performance metrics');
      console.log('   • Consider additional optimizations');
    } else {
      console.log('   • Investigate failing criteria');
      console.log('   • Optimize bottlenecks');
      console.log('   • Re-run performance tests');
    }
  }
}

// Run tests if called directly
if (require.main === module) {
  const testSuite = new PerformanceTestSuite();
  testSuite.runAllTests().catch(console.error);
}

module.exports = PerformanceTestSuite;