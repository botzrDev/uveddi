import { setupServer } from 'msw/node';
import { rest } from 'msw';
import { mockInteractiveReport } from './mockData';

export const handlers = [
  // Mock API endpoints
  rest.get('/api/reports/:id', (req, res, ctx) => {
    return res(ctx.json(mockInteractiveReport));
  }),

  rest.get('/api/reports/:id/history', (req, res, ctx) => {
    return res(ctx.json({
      snapshots: [
        {
          id: '1',
          timestamp: '2024-01-01T00:00:00Z',
          qualityScore: 85,
          issueCount: 120,
          technicalDebt: 15000
        },
        {
          id: '2', 
          timestamp: '2024-02-01T00:00:00Z',
          qualityScore: 88,
          issueCount: 100,
          technicalDebt: 12000
        }
      ]
    }));
  }),

  rest.post('/api/reports/:id/export', (req, res, ctx) => {
    return res(ctx.json({ 
      success: true, 
      downloadUrl: '/api/downloads/report-export.pdf' 
    }));
  }),

  rest.get('/api/suggestions/:issueId', (req, res, ctx) => {
    return res(ctx.json({
      fixes: [
        {
          id: '1',
          title: 'Extract Method',
          description: 'Break down this large method into smaller, focused methods',
          difficulty: 'medium',
          estimatedTime: '2 hours',
          codeChanges: [
            {
              file: 'src/utils/helper.ts',
              oldCode: 'function largeMethod() { /* ... */ }',
              newCode: 'function extractedMethod() { /* ... */ }'
            }
          ]
        }
      ]
    }));
  }),

  rest.get('/api/search', (req, res, ctx) => {
    const query = req.url.searchParams.get('q');
    return res(ctx.json({
      results: [
        {
          id: '1',
          type: 'issue',
          title: `Search result for: ${query}`,
          description: 'Mock search result',
          severity: 'medium'
        }
      ]
    }));
  })
];

export const server = setupServer(...handlers);