/**
 * Comprehensive Integration Tests for Uveddi API Server
 *
 * This test suite provides extensive coverage for all API endpoints including:
 * - Health and status endpoints
 * - Report management endpoints
 * - Documentation endpoints
 * - WebSocket real-time updates
 * - Authentication and authorization
 * - Error handling and edge cases
 *
 * Target coverage: 90%+
 */

const request = require('supertest');
const WebSocket = require('ws');
const path = require('path');
const fs = require('fs-extra');

// Configure test environment
const API_BASE_URL = process.env.API_URL || 'http://localhost:8000';
const WS_URL = process.env.WS_URL || 'ws://localhost:8000';

// Test data fixtures
const TEST_REPORT = {
  id: 'test-report-001',
  timestamp: new Date().toISOString(),
  summary: {
    filesAnalyzed: 150,
    issuesFound: 25,
    criticalIssues: 3,
    majorIssues: 8,
    minorIssues: 14
  },
  findings: [
    {
      type: 'god_object',
      severity: 'critical',
      file: 'src/main.rs',
      line: 45,
      description: 'Class has too many responsibilities'
    }
  ]
};

describe('API Integration Tests', () => {
  let app;

  beforeAll(async () => {
    // Setup test environment
    // In production, you'd start the actual server or use a test instance
  });

  afterAll(async () => {
    // Cleanup
  });

  describe('Health and Status Endpoints', () => {
    test('GET /health should return service status', async () => {
      const response = await request(API_BASE_URL)
        .get('/health')
        .expect(200);

      expect(response.body).toMatchObject({
        status: 'ok',
        service: 'uveddi-api-server-alpha',
        version: expect.stringMatching(/\d+\.\d+\.\d+/),
        timestamp: expect.any(String)
      });
    });

    test('GET /health should respond quickly', async () => {
      const startTime = Date.now();
      await request(API_BASE_URL)
        .get('/health')
        .expect(200);
      const endTime = Date.now();

      expect(endTime - startTime).toBeLessThan(100); // Should respond within 100ms
    });
  });

  describe('Project Information Endpoints', () => {
    test('GET /api/project/info should return project details', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/project/info')
        .expect(200);

      expect(response.body).toMatchObject({
        name: 'Uveddi',
        version: expect.any(String),
        description: expect.any(String),
        repository: expect.stringContaining('github.com'),
        features: expect.arrayContaining([
          expect.stringContaining('analysis'),
          expect.stringContaining('AI')
        ])
      });
    });
  });

  describe('Documentation Endpoints', () => {
    test('GET /api/docs/structure should return documentation tree', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/docs/structure')
        .expect(200);

      expect(response.body).toHaveProperty('structure');
      expect(Array.isArray(response.body.structure)).toBe(true);
    });

    test('GET /api/docs/file/:path should return markdown content', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/docs/file/README.md')
        .expect(200);

      expect(response.body).toMatchObject({
        content: expect.any(String),
        html: expect.any(String),
        path: 'README.md'
      });
    });

    test('GET /api/docs/file/:path should handle non-existent files', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/docs/file/non-existent-file.md')
        .expect(404);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('not found')
      });
    });

    test('GET /api/docs/search should search documentation', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/docs/search')
        .query({ q: 'analysis' })
        .expect(200);

      expect(response.body).toMatchObject({
        query: 'analysis',
        results: expect.any(Array)
      });

      if (response.body.results.length > 0) {
        expect(response.body.results[0]).toMatchObject({
          title: expect.any(String),
          path: expect.any(String),
          snippet: expect.any(String)
        });
      }
    });

    test('GET /api/docs/search should require query parameter', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/docs/search')
        .expect(400);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('query required')
      });
    });
  });

  describe('Report Endpoints', () => {
    test('GET /api/reports should list available reports', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports')
        .expect(200);

      expect(response.body).toMatchObject({
        reports: expect.any(Array),
        total: expect.any(Number)
      });
    });

    test('GET /api/reports/:id should return specific report', async () => {
      // First, get list of reports
      const listResponse = await request(API_BASE_URL)
        .get('/api/reports')
        .expect(200);

      if (listResponse.body.reports.length > 0) {
        const reportId = listResponse.body.reports[0].id;

        const response = await request(API_BASE_URL)
          .get(`/api/reports/${reportId}`)
          .expect(200);

        expect(response.body).toMatchObject({
          id: reportId,
          timestamp: expect.any(String),
          summary: expect.objectContaining({
            filesAnalyzed: expect.any(Number),
            issuesFound: expect.any(Number)
          })
        });
      }
    });

    test('GET /api/reports/:id should handle non-existent report', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports/non-existent-id')
        .expect(404);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('not found')
      });
    });

    test('GET /api/reports/latest should return most recent report', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports/latest')
        .expect(200);

      if (response.body && response.body.id) {
        expect(response.body).toMatchObject({
          id: expect.any(String),
          timestamp: expect.any(String),
          summary: expect.any(Object)
        });
      }
    });

    test('POST /api/reports should create new report', async () => {
      const response = await request(API_BASE_URL)
        .post('/api/reports')
        .send(TEST_REPORT)
        .expect(201);

      expect(response.body).toMatchObject({
        id: expect.any(String),
        message: expect.stringContaining('created')
      });
    });

    test('POST /api/reports should validate report data', async () => {
      const invalidReport = { invalid: 'data' };

      const response = await request(API_BASE_URL)
        .post('/api/reports')
        .send(invalidReport)
        .expect(400);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('validation')
      });
    });

    test('DELETE /api/reports/:id should remove report', async () => {
      // First create a report
      const createResponse = await request(API_BASE_URL)
        .post('/api/reports')
        .send(TEST_REPORT)
        .expect(201);

      const reportId = createResponse.body.id;

      // Then delete it
      const deleteResponse = await request(API_BASE_URL)
        .delete(`/api/reports/${reportId}`)
        .expect(200);

      expect(deleteResponse.body).toMatchObject({
        message: expect.stringContaining('deleted')
      });

      // Verify it's gone
      await request(API_BASE_URL)
        .get(`/api/reports/${reportId}`)
        .expect(404);
    });
  });

  describe('Export Endpoints', () => {
    test('GET /api/reports/:id/export?format=json should export as JSON', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports/latest/export')
        .query({ format: 'json' })
        .expect(200);

      expect(response.headers['content-type']).toContain('application/json');
    });

    test('GET /api/reports/:id/export?format=markdown should export as Markdown', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports/latest/export')
        .query({ format: 'markdown' })
        .expect(200);

      expect(response.headers['content-type']).toContain('text/markdown');
      expect(response.text).toContain('#'); // Markdown headers
    });

    test('GET /api/reports/:id/export?format=html should export as HTML', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports/latest/export')
        .query({ format: 'html' })
        .expect(200);

      expect(response.headers['content-type']).toContain('text/html');
      expect(response.text).toContain('<html>');
    });

    test('GET /api/reports/:id/export should handle invalid format', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/reports/latest/export')
        .query({ format: 'invalid' })
        .expect(400);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('format')
      });
    });
  });

  describe('Analysis Endpoints', () => {
    test('POST /api/analyze should trigger new analysis', async () => {
      const response = await request(API_BASE_URL)
        .post('/api/analyze')
        .send({
          path: '/test/path',
          options: {
            language: 'rust',
            depth: 'full'
          }
        })
        .expect(202); // Accepted

      expect(response.body).toMatchObject({
        jobId: expect.any(String),
        status: 'queued'
      });
    });

    test('GET /api/analyze/:jobId/status should return job status', async () => {
      // First start an analysis
      const analyzeResponse = await request(API_BASE_URL)
        .post('/api/analyze')
        .send({ path: '/test/path' })
        .expect(202);

      const jobId = analyzeResponse.body.jobId;

      // Check status
      const statusResponse = await request(API_BASE_URL)
        .get(`/api/analyze/${jobId}/status`)
        .expect(200);

      expect(statusResponse.body).toMatchObject({
        jobId: jobId,
        status: expect.stringMatching(/queued|running|completed|failed/),
        progress: expect.any(Number)
      });
    });
  });

  describe('WebSocket Real-time Updates', () => {
    test('WebSocket should connect and receive updates', (done) => {
      const ws = new WebSocket(WS_URL);

      ws.on('open', () => {
        expect(ws.readyState).toBe(WebSocket.OPEN);
        ws.send(JSON.stringify({ type: 'subscribe', channel: 'analysis' }));
      });

      ws.on('message', (data) => {
        const message = JSON.parse(data);
        expect(message).toHaveProperty('type');
        ws.close();
        done();
      });

      ws.on('error', (error) => {
        done(error);
      });
    });

    test('WebSocket should handle invalid messages', (done) => {
      const ws = new WebSocket(WS_URL);

      ws.on('open', () => {
        ws.send('invalid json');
      });

      ws.on('message', (data) => {
        const message = JSON.parse(data);
        expect(message.type).toBe('error');
        ws.close();
        done();
      });
    });
  });

  describe('Authentication Endpoints', () => {
    test('POST /auth/register should create new user', async () => {
      const response = await request(API_BASE_URL)
        .post('/auth/register')
        .send({
          username: 'testuser',
          email: 'test@example.com',
          password: 'TestPassword123!'
        })
        .expect(201);

      expect(response.body).toMatchObject({
        userId: expect.any(String),
        message: expect.stringContaining('registered')
      });
    });

    test('POST /auth/login should authenticate user', async () => {
      const response = await request(API_BASE_URL)
        .post('/auth/login')
        .send({
          email: 'test@example.com',
          password: 'TestPassword123!'
        })
        .expect(200);

      expect(response.body).toMatchObject({
        token: expect.any(String),
        expiresIn: expect.any(Number)
      });
    });

    test('POST /auth/login should reject invalid credentials', async () => {
      const response = await request(API_BASE_URL)
        .post('/auth/login')
        .send({
          email: 'test@example.com',
          password: 'WrongPassword'
        })
        .expect(401);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('Invalid')
      });
    });

    test('Protected endpoints should require authentication', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/user/profile')
        .expect(401);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('authentication')
      });
    });

    test('Protected endpoints should accept valid token', async () => {
      // First login
      const loginResponse = await request(API_BASE_URL)
        .post('/auth/login')
        .send({
          email: 'test@example.com',
          password: 'TestPassword123!'
        })
        .expect(200);

      const token = loginResponse.body.token;

      // Access protected endpoint
      const response = await request(API_BASE_URL)
        .get('/api/user/profile')
        .set('Authorization', `Bearer ${token}`)
        .expect(200);

      expect(response.body).toHaveProperty('userId');
    });
  });

  describe('Error Handling', () => {
    test('Should handle 404 for unknown routes', async () => {
      const response = await request(API_BASE_URL)
        .get('/api/unknown/endpoint')
        .expect(404);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('not found')
      });
    });

    test('Should handle malformed JSON', async () => {
      const response = await request(API_BASE_URL)
        .post('/api/reports')
        .set('Content-Type', 'application/json')
        .send('{ invalid json }')
        .expect(400);

      expect(response.body).toMatchObject({
        error: expect.stringContaining('JSON')
      });
    });

    test('Should handle server errors gracefully', async () => {
      // This would test error handling by triggering a server error
      // In a real test, you might mock a database failure
    });
  });

  describe('Performance Tests', () => {
    test('API should handle concurrent requests', async () => {
      const requests = Array(10).fill(null).map(() =>
        request(API_BASE_URL).get('/health')
      );

      const responses = await Promise.all(requests);
      responses.forEach(response => {
        expect(response.status).toBe(200);
      });
    });

    test('Large report should be handled efficiently', async () => {
      const largeReport = {
        ...TEST_REPORT,
        findings: Array(1000).fill(TEST_REPORT.findings[0])
      };

      const startTime = Date.now();
      const response = await request(API_BASE_URL)
        .post('/api/reports')
        .send(largeReport)
        .expect(201);
      const endTime = Date.now();

      expect(endTime - startTime).toBeLessThan(5000); // Should complete within 5 seconds
    });
  });

  describe('CORS and Security', () => {
    test('Should include security headers', async () => {
      const response = await request(API_BASE_URL)
        .get('/health')
        .expect(200);

      expect(response.headers).toHaveProperty('x-content-type-options');
      expect(response.headers).toHaveProperty('x-frame-options');
    });

    test('Should handle CORS for allowed origins', async () => {
      const response = await request(API_BASE_URL)
        .get('/health')
        .set('Origin', 'http://localhost:8001')
        .expect(200);

      expect(response.headers['access-control-allow-origin']).toBe('http://localhost:8001');
    });

    test('Should reject CORS for disallowed origins', async () => {
      const response = await request(API_BASE_URL)
        .get('/health')
        .set('Origin', 'http://evil.com')
        .expect(200); // Request succeeds but without CORS headers

      expect(response.headers['access-control-allow-origin']).toBeUndefined();
    });
  });
});

// Load testing helper
async function loadTest(endpoint, concurrency = 100, duration = 10000) {
  const results = {
    successful: 0,
    failed: 0,
    totalTime: 0,
    minTime: Infinity,
    maxTime: 0
  };

  const startTime = Date.now();
  const promises = [];

  while (Date.now() - startTime < duration) {
    for (let i = 0; i < concurrency; i++) {
      const requestStart = Date.now();
      promises.push(
        request(API_BASE_URL)
          .get(endpoint)
          .then(response => {
            const requestTime = Date.now() - requestStart;
            results.successful++;
            results.totalTime += requestTime;
            results.minTime = Math.min(results.minTime, requestTime);
            results.maxTime = Math.max(results.maxTime, requestTime);
          })
          .catch(() => {
            results.failed++;
          })
      );
    }
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  await Promise.all(promises);

  return {
    ...results,
    avgTime: results.totalTime / results.successful,
    requestsPerSecond: (results.successful / (duration / 1000))
  };
}

// Export for use in other test files
module.exports = {
  API_BASE_URL,
  WS_URL,
  TEST_REPORT,
  loadTest
};