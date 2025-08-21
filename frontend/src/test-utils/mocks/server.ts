import { setupServer } from 'msw/node';
import { http } from 'msw';
import { mockInteractiveReport } from './mockData';

export const handlers = [
  // Mock API endpoints
  http.get('/api/reports/:id', () => {
    return Response.json(mockInteractiveReport);
  }),

  http.get('/api/reports/:id/history', () => {
    return Response.json({
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
    });
  }),

  http.post('/api/reports/:id/export', () => {
    return Response.json({ 
      success: true, 
      downloadUrl: '/api/downloads/report-export.pdf' 
    });
  }),

  http.get('/api/suggestions/:issueId', () => {
    return Response.json({
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
    });
  }),

  http.get('/api/search', ({ request }) => {
    const url = new URL(request.url);
    const query = url.searchParams.get('q');
    return Response.json({
      results: [
        {
          id: '1',
          type: 'issue',
          title: `Search result for: ${query}`,
          description: 'Mock search result',
          severity: 'medium'
        }
      ]
    });
  })
];

export const server = setupServer(...handlers);