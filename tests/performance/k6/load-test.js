import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics for UV-82 load testing
const analysisSuccessRate = new Rate('analysis_success_rate');
const renderingSuccessRate = new Rate('rendering_success_rate');
const analysisLatency = new Trend('analysis_latency');
const renderingLatency = new Trend('rendering_latency');

// Load test data from files (will be provided by initContainer)
const testFiles = new SharedArray('test-files', function () {
  try {
    return JSON.parse(open('/test-data/test-files.json')).files;
  } catch (e) {
    // Fallback test data
    return [
      { name: 'small.rs', size: 'small', content: 'fn main() { println!("Hello, world!"); }' },
      { name: 'medium.rs', size: 'medium', content: 'fn main() {\n'.repeat(50) + '}' },
      { name: 'large.rs', size: 'large', content: 'fn main() {\n'.repeat(500) + '}' }
    ];
  }
});

const users = new SharedArray('users', function () {
  try {
    return JSON.parse(open('/test-data/users.json')).users;
  } catch (e) {
    // Fallback test users
    return [
      { username: 'testuser1', password: 'password123' },
      { username: 'testuser2', password: 'password123' },
      { username: 'testuser3', password: 'password123' }
    ];
  }
});

// Load test configuration based on UV-82 requirements
export const options = {
  scenarios: {
    // 80% of load: Full analysis workflow (1000+ concurrent users)
    full_analysis_workflow: {
      executor: 'ramping-vus',
      exec: 'analysisJourney',
      stages: [
        { duration: '2m', target: 100 },   // Warm-up
        { duration: '3m', target: 500 },   // Ramp to half capacity
        { duration: '5m', target: 1000 },  // Target load
        { duration: '10m', target: 1000 }, // Sustained load
        { duration: '2m', target: 1500 },  // Spike test
        { duration: '1m', target: 1000 },  // Back to sustained
        { duration: '3m', target: 0 },     // Ramp down
      ],
    },
    // 20% of load: Diagram rendering only
    diagram_rendering_workflow: {
      executor: 'ramping-vus',
      exec: 'renderingJourney',
      startTime: '1m',
      stages: [
        { duration: '4m', target: 200 },
        { duration: '10m', target: 200 },
        { duration: '2m', target: 400 },   // Spike
        { duration: '1m', target: 200 },
        { duration: '3m', target: 0 },
      ],
    },
  },
  thresholds: {
    // UV-82 performance targets
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_duration{scenario:full_analysis_workflow}': ['p(95)<500'],
    'http_req_duration{scenario:diagram_rendering_workflow}': ['p(99)<50'],
    'http_req_failed': ['rate<0.01'],
    'analysis_success_rate': ['rate>0.99'],
    'rendering_success_rate': ['rate>0.995'],
    'analysis_latency': ['p(95)<500'],
    'rendering_latency': ['p(99)<50'],
  },
};

// Global setup: Authenticate once
export function setup() {
  const baseUrl = __ENV.API_HOST || 'http://localhost:8080';
  
  const loginRes = http.post(`${baseUrl}/auth/login`, {
    username: 'loadtest',
    password: 'loadtest123',
  });
  
  check(loginRes, { 
    'login successful': (r) => r.status === 200 
  });
  
  return { 
    token: loginRes.json('access_token'),
    baseUrl: baseUrl
  };
}

// Main analysis journey - simulates complete user workflow
export function analysisJourney(data) {
  const user = users[__VU % users.length];
  const testFile = testFiles[Math.floor(Math.random() * testFiles.length)];
  
  const headers = { 
    'Authorization': `Bearer ${data.token}`,
    'Content-Type': 'application/json',
  };

  // Step 1: Upload and analyze code file
  const analysisStart = Date.now();
  const analysisPayload = {
    project_name: `project-${__VU}-${__ITER}`,
    content: testFile.content,
    file_name: testFile.name,
    language: 'rust',
  };

  const analysisRes = http.post(
    `${data.baseUrl}/api/analysis`, 
    JSON.stringify(analysisPayload), 
    { headers }
  );
  
  const analysisSuccess = check(analysisRes, {
    'analysis request successful': (r) => r.status === 202 || r.status === 200,
    'analysis returns valid id': (r) => r.json('analysis_id') !== undefined,
  });
  
  analysisSuccessRate.add(analysisSuccess);
  
  if (!analysisSuccess) {
    console.error(`Analysis failed for VU ${__VU}: ${analysisRes.status} ${analysisRes.body}`);
    return;
  }

  const analysisId = analysisRes.json('analysis_id');
  
  // Realistic think time
  sleep(Math.random() * 2 + 1);

  // Step 2: Poll for analysis completion
  let status = '';
  let pollAttempts = 0;
  const maxPollAttempts = 30; // 30 seconds max wait
  
  while (status !== 'completed' && pollAttempts < maxPollAttempts) {
    const statusRes = http.get(
      `${data.baseUrl}/api/analysis/${analysisId}/status`, 
      { headers }
    );
    
    if (check(statusRes, { 'status check successful': (r) => r.status === 200 })) {
      status = statusRes.json('status');
      
      if (status === 'failed') {
        console.error(`Analysis failed for VU ${__VU}: ${statusRes.body}`);
        return;
      }
    }
    
    pollAttempts++;
    sleep(1);
  }
  
  const analysisLatencyMs = Date.now() - analysisStart;
  analysisLatency.add(analysisLatencyMs);
  
  check(status, { 'analysis completed': (s) => s === 'completed' });

  // Step 3: Fetch analysis results
  const resultsRes = http.get(
    `${data.baseUrl}/api/analysis/${analysisId}/results`, 
    { headers }
  );
  
  check(resultsRes, {
    'results fetch successful': (r) => r.status === 200,
    'results contain data': (r) => r.json('components') !== undefined,
  });

  // Realistic think time before requesting diagram
  sleep(Math.random() * 3 + 2);

  // Step 4: Request diagram rendering
  const renderingStart = Date.now();
  const renderingRes = http.post(
    `${data.baseUrl}/api/render/diagram`,
    JSON.stringify({
      analysis_id: analysisId,
      format: 'svg',
      width: 1200,
      height: 800,
    }),
    { headers }
  );
  
  const renderingLatencyMs = Date.now() - renderingStart;
  renderingLatency.add(renderingLatencyMs);
  
  const renderingSuccess = check(renderingRes, {
    'rendering successful': (r) => r.status === 200,
    'rendering returns svg': (r) => r.body.includes('<svg'),
    'rendering latency acceptable': (r) => renderingLatencyMs < 100,
  });
  
  renderingSuccessRate.add(renderingSuccess);

  // Final think time
  sleep(Math.random() * 2 + 1);
}

// Rendering-only journey - for users just viewing diagrams
export function renderingJourney(data) {
  const headers = { 
    'Authorization': `Bearer ${data.token}`,
    'Content-Type': 'application/json',
  };

  // Simulate requesting an existing diagram
  const renderingStart = Date.now();
  const renderingRes = http.post(
    `${data.baseUrl}/api/render/diagram`,
    JSON.stringify({
      mermaid_code: `graph TD
        A[User Request] --> B{Analysis Engine}
        B --> C[Component Extraction]
        B --> D[Dependency Analysis]
        C --> E[Mermaid Generation]
        D --> E
        E --> F[SVG Rendering]
        F --> G[Response]`,
      format: 'svg',
      width: 800,
      height: 600,
    }),
    { headers }
  );
  
  const renderingLatencyMs = Date.now() - renderingStart;
  renderingLatency.add(renderingLatencyMs);
  
  const renderingSuccess = check(renderingRes, {
    'direct rendering successful': (r) => r.status === 200,
    'direct rendering returns svg': (r) => r.body.includes('<svg'),
    'direct rendering fast': (r) => renderingLatencyMs < 50,
  });
  
  renderingSuccessRate.add(renderingSuccess);

  // Short think time for rendering-only users
  sleep(Math.random() * 1 + 0.5);
}

// Teardown: Clean up resources
export function teardown(data) {
  // Log final metrics
  console.log('Load test completed');
  console.log(`Final analysis success rate: ${analysisSuccessRate.rate}`);
  console.log(`Final rendering success rate: ${renderingSuccessRate.rate}`);
}