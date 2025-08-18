import { InteractiveReport, Issue, QualityMetrics, DependencyNode } from '../../types/dashboard';

export const mockIssues: Issue[] = [
  {
    id: '1',
    title: 'Large Method in UserService',
    description: 'Method getUserProfile has 150 lines and violates single responsibility principle',
    severity: 'high',
    category: 'code-smell',
    file: 'src/services/UserService.ts',
    line: 45,
    column: 12,
    impact: 8,
    effort: 6,
    tags: ['maintainability', 'complexity'],
    suggested_fix: 'Extract method into smaller, focused functions',
    confidence: 0.9,
    created_at: '2024-01-15T10:30:00Z',
    updated_at: '2024-01-15T10:30:00Z'
  },
  {
    id: '2', 
    title: 'Circular Dependency Detected',
    description: 'Circular dependency between UserService and AuthService',
    severity: 'critical',
    category: 'architecture',
    file: 'src/services/AuthService.ts',
    line: 23,
    column: 1,
    impact: 9,
    effort: 8,
    tags: ['architecture', 'dependency'],
    suggested_fix: 'Introduce dependency injection or event-driven pattern',
    confidence: 0.95,
    created_at: '2024-01-14T14:20:00Z',
    updated_at: '2024-01-14T14:20:00Z'
  },
  {
    id: '3',
    title: 'Dead Code Found',
    description: 'Unused function calculateOldMetrics has been unused for 6 months',
    severity: 'medium',
    category: 'dead-code',
    file: 'src/utils/metrics.ts',
    line: 78,
    column: 1,
    impact: 3,
    effort: 2,
    tags: ['cleanup', 'dead-code'],
    suggested_fix: 'Remove unused function and associated tests',
    confidence: 0.85,
    created_at: '2024-01-13T09:15:00Z',
    updated_at: '2024-01-13T09:15:00Z'
  },
  {
    id: '4',
    title: 'Magic Number Usage',
    description: 'Hardcoded number 42 used without explanation in calculation',
    severity: 'low',
    category: 'code-smell',
    file: 'src/components/Dashboard.tsx',
    line: 156,
    column: 25,
    impact: 4,
    effort: 3,
    tags: ['maintainability', 'magic-numbers'],
    suggested_fix: 'Extract magic number to named constant',
    confidence: 0.8,
    created_at: '2024-01-12T16:45:00Z',
    updated_at: '2024-01-12T16:45:00Z'
  }
];

export const mockQualityMetrics: QualityMetrics = {
  overall_score: 87,
  maintainability: 85,
  reliability: 90,
  security: 82,
  performance: 88,
  test_coverage: 78,
  complexity_score: 6.2,
  debt_ratio: 0.12,
  duplication_percentage: 3.5,
  trends: {
    overall_score: [82, 84, 85, 87, 87],
    maintainability: [80, 82, 84, 85, 85],
    reliability: [88, 89, 89, 90, 90],
    security: [78, 80, 81, 82, 82],
    performance: [85, 86, 87, 88, 88],
    test_coverage: [75, 76, 77, 78, 78]
  }
};

export const mockDependencies: DependencyNode[] = [
  {
    id: 'UserService',
    name: 'UserService',
    type: 'service',
    file: 'src/services/UserService.ts',
    dependencies: ['AuthService', 'DatabaseService'],
    dependents: ['UserController', 'ProfileComponent'],
    metrics: {
      complexity: 8,
      coupling: 6,
      cohesion: 7,
      lines_of_code: 245
    }
  },
  {
    id: 'AuthService',
    name: 'AuthService', 
    type: 'service',
    file: 'src/services/AuthService.ts',
    dependencies: ['DatabaseService', 'CryptoUtils'],
    dependents: ['UserService', 'LoginComponent'],
    metrics: {
      complexity: 6,
      coupling: 4,
      cohesion: 8,
      lines_of_code: 180
    }
  },
  {
    id: 'DatabaseService',
    name: 'DatabaseService',
    type: 'service', 
    file: 'src/services/DatabaseService.ts',
    dependencies: [],
    dependents: ['UserService', 'AuthService', 'ReportService'],
    metrics: {
      complexity: 4,
      coupling: 2,
      cohesion: 9,
      lines_of_code: 120
    }
  }
];

export const mockInteractiveReport: InteractiveReport = {
  id: 'report-1',
  project_name: 'Uveddi Dashboard',
  generated_at: '2024-01-15T12:00:00Z',
  version: '1.0.0',
  summary: {
    total_issues: mockIssues.length,
    critical_issues: mockIssues.filter(i => i.severity === 'critical').length,
    high_issues: mockIssues.filter(i => i.severity === 'high').length,
    medium_issues: mockIssues.filter(i => i.severity === 'medium').length,
    low_issues: mockIssues.filter(i => i.severity === 'low').length,
    files_analyzed: 45,
    lines_of_code: 12500,
    test_coverage: 78,
    estimated_fix_time: '24 hours'
  },
  issues: mockIssues,
  quality_metrics: mockQualityMetrics,
  dependencies: mockDependencies,
  technical_debt: {
    total_debt_hours: 48,
    debt_by_category: {
      'code-smell': 20,
      'architecture': 15,
      'security': 8,
      'performance': 5
    },
    debt_trend: [50, 48, 45, 44, 48],
    priority_items: mockIssues.slice(0, 2).map(issue => ({
      issue_id: issue.id,
      debt_hours: issue.severity === 'critical' ? 8 : 4,
      roi_score: issue.impact * 0.1
    }))
  },
  insights: [
    'Code complexity has increased by 5% since last analysis',
    'Security vulnerabilities decreased by 20%',
    'Test coverage improved by 3 percentage points'
  ],
  recommendations: [
    'Focus on reducing cyclomatic complexity in core services',
    'Implement automated security scanning in CI/CD pipeline',
    'Increase unit test coverage for utility functions'
  ]
};

export const mockHistoricalData = {
  snapshots: Array.from({ length: 12 }, (_, i) => ({
    id: `snapshot-${i + 1}`,
    timestamp: new Date(2024, i, 1).toISOString(),
    qualityScore: 80 + Math.random() * 15,
    issueCount: 100 + Math.floor(Math.random() * 50),
    technicalDebt: 10000 + Math.floor(Math.random() * 5000),
    testCoverage: 70 + Math.random() * 20,
    linesOfCode: 10000 + i * 1000
  }))
};

export const mockTeamMembers = [
  {
    id: '1',
    name: 'Alice Johnson',
    email: 'alice@example.com',
    avatar: '/avatars/alice.jpg',
    role: 'Senior Developer',
    contributions: 45
  },
  {
    id: '2',
    name: 'Bob Smith',
    email: 'bob@example.com', 
    avatar: '/avatars/bob.jpg',
    role: 'Technical Lead',
    contributions: 38
  },
  {
    id: '3',
    name: 'Carol Davis',
    email: 'carol@example.com',
    avatar: '/avatars/carol.jpg', 
    role: 'QA Engineer',
    contributions: 22
  }
];