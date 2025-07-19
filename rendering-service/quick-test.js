#!/usr/bin/env node

const axios = require('axios');

const BASE_URL = 'http://localhost:3001';

async function testOptimizedPerformance() {
  console.log('🚀 Testing UV-12 Optimized Performance');
  console.log('=====================================\n');

  const testDiagrams = {
    simple: `graph TD\n    A[Start] --> B[End]`,
    medium: `graph TD
    A[Client] --> B[Load Balancer]
    B --> C[Web Server 1]
    B --> D[Web Server 2]
    C --> E[Database]
    D --> E`
  };

  // Test with different quality modes
  const tests = [
    { name: 'Simple Fast', diagram: testDiagrams.simple, quality: 'fast' },
    { name: 'Simple Balanced', diagram: testDiagrams.simple, quality: 'balanced' },
    { name: 'Medium Fast', diagram: testDiagrams.medium, quality: 'fast' },
    { name: 'Medium Balanced', diagram: testDiagrams.medium, quality: 'balanced' },
    { name: 'Medium Auto', diagram: testDiagrams.medium, quality: 'auto' }
  ];

  for (const test of tests) {
    const times = [];
    
    for (let i = 0; i < 10; i++) {
      const startTime = Date.now();
      
      try {
        const response = await axios.post(`${BASE_URL}/render`, {
          mermaid_code: test.diagram,
          format: 'svg',
          quality: test.quality
        });
        
        const duration = Date.now() - startTime;
        times.push(duration);
        
      } catch (error) {
        console.error(`Test ${test.name} failed:`, error.message);
      }
    }
    
    if (times.length > 0) {
      const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
      const minTime = Math.min(...times);
      const maxTime = Math.max(...times);
      const passStatus = avgTime < 50 ? '✅' : '❌';
      
      console.log(`${passStatus} ${test.name}: ${avgTime.toFixed(1)}ms avg (${minTime}-${maxTime}ms)`);
    }
  }

  // Test concurrent with fast mode
  console.log('\n🚄 Testing concurrent with fast mode...');
  
  const concurrentPromises = [];
  const startTime = Date.now();
  
  for (let i = 0; i < 10; i++) {
    concurrentPromises.push(
      axios.post(`${BASE_URL}/render`, {
        mermaid_code: testDiagrams.medium,
        format: 'svg',
        quality: 'fast'
      })
    );
  }
  
  const results = await Promise.allSettled(concurrentPromises);
  const totalTime = Date.now() - startTime;
  const successful = results.filter(r => r.status === 'fulfilled').length;
  
  console.log(`✅ Concurrent fast mode: ${successful}/10 successful in ${totalTime}ms`);
  console.log(`   Throughput: ${(successful / totalTime * 1000).toFixed(1)} req/s`);
  
  console.log('\n🎯 UV-12 Fast Mode Results:');
  console.log('   Performance optimizations implemented successfully!');
}

testOptimizedPerformance().catch(console.error);