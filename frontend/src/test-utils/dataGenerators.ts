import { faker } from '@faker-js/faker';
import { 
  Issue, 
  QualityMetrics, 
  DependencyNode, 
  InteractiveReport,
  TechnicalDebt 
} from '../types/dashboard';

// Seed for consistent test data
faker.seed(12345);

export const generateMockIssue = (overrides: Partial<Issue> = {}): Issue => ({
  id: faker.string.uuid(),
  title: faker.lorem.sentence({ min: 3, max: 8 }),
  description: faker.lorem.paragraph(),
  severity: faker.helpers.arrayElement(['critical', 'high', 'medium', 'low']),
  category: faker.helpers.arrayElement(['code-smell', 'architecture', 'security', 'performance', 'dead-code']),
  file: `src/${faker.helpers.arrayElement(['services', 'components', 'utils'])}/${faker.lorem.word()}.${faker.helpers.arrayElement(['ts', 'tsx', 'js'])}`,
  line: faker.number.int({ min: 1, max: 500 }),
  column: faker.number.int({ min: 1, max: 120 }),
  impact: faker.number.int({ min: 1, max: 10 }),
  effort: faker.number.int({ min: 1, max: 10 }),
  tags: faker.helpers.arrayElements(['maintainability', 'complexity', 'security', 'performance', 'testing'], { min: 1, max: 3 }),
  suggested_fix: faker.lorem.sentence(),
  confidence: faker.number.float({ min: 0.5, max: 1, fractionDigits: 2 }),
  created_at: faker.date.recent({ days: 30 }).toISOString(),
  updated_at: faker.date.recent({ days: 7 }).toISOString(),
  ...overrides
});

export const generateMockIssues = (count: number, overrides: Partial<Issue> = {}): Issue[] => 
  Array.from({ length: count }, () => generateMockIssue(overrides));

export const generateMockQualityMetrics = (overrides: Partial<QualityMetrics> = {}): QualityMetrics => {
  const baseScore = faker.number.int({ min: 60, max: 95 });
  return {
    overall_score: baseScore,
    maintainability: faker.number.int({ min: baseScore - 10, max: baseScore + 10 }),
    reliability: faker.number.int({ min: baseScore - 5, max: baseScore + 15 }),
    security: faker.number.int({ min: baseScore - 15, max: baseScore + 5 }),
    performance: faker.number.int({ min: baseScore - 10, max: baseScore + 10 }),
    test_coverage: faker.number.int({ min: 50, max: 95 }),
    complexity_score: faker.number.float({ min: 1, max: 10, fractionDigits: 1 }),
    debt_ratio: faker.number.float({ min: 0.05, max: 0.25, fractionDigits: 2 }),
    duplication_percentage: faker.number.float({ min: 1, max: 10, fractionDigits: 1 }),
    trends: {
      overall_score: Array.from({ length: 12 }, () => faker.number.int({ min: baseScore - 5, max: baseScore + 5 })),
      maintainability: Array.from({ length: 12 }, () => faker.number.int({ min: baseScore - 8, max: baseScore + 8 })),
      reliability: Array.from({ length: 12 }, () => faker.number.int({ min: baseScore - 3, max: baseScore + 12 })),
      security: Array.from({ length: 12 }, () => faker.number.int({ min: baseScore - 12, max: baseScore + 3 })),
      performance: Array.from({ length: 12 }, () => faker.number.int({ min: baseScore - 8, max: baseScore + 8 })),
      test_coverage: Array.from({ length: 12 }, () => faker.number.int({ min: 45, max: 90 }))
    },
    ...overrides
  };
};

export const generateMockDependencyNode = (overrides: Partial<DependencyNode> = {}): DependencyNode => {
  const name = faker.lorem.word();
  return {
    id: faker.string.uuid(),
    name: `${name.charAt(0).toUpperCase()}${name.slice(1)}Service`,
    type: faker.helpers.arrayElement(['service', 'component', 'utility', 'model']),
    file: `src/${faker.helpers.arrayElement(['services', 'components', 'utils', 'models'])}/${name}.ts`,
    dependencies: faker.helpers.arrayElements(
      ['AuthService', 'DatabaseService', 'ValidationService', 'LoggerService', 'ConfigService'],
      { min: 0, max: 3 }
    ),
    dependents: faker.helpers.arrayElements(
      ['UserController', 'AdminPanel', 'ApiRouter', 'BackgroundJob'],
      { min: 0, max: 4 }
    ),
    metrics: {
      complexity: faker.number.int({ min: 1, max: 10 }),
      coupling: faker.number.int({ min: 1, max: 8 }),
      cohesion: faker.number.int({ min: 3, max: 10 }),
      lines_of_code: faker.number.int({ min: 50, max: 500 })
    },
    ...overrides
  };
};

export const generateMockDependencies = (count: number, overrides: Partial<DependencyNode> = {}): DependencyNode[] =>
  Array.from({ length: count }, () => generateMockDependencyNode(overrides));

export const generateMockTechnicalDebt = (overrides: Partial<TechnicalDebt> = {}): TechnicalDebt => {
  const totalHours = faker.number.int({ min: 20, max: 200 });
  const categories = {
    'code-smell': faker.number.int({ min: 5, max: Math.floor(totalHours * 0.4) }),
    'architecture': faker.number.int({ min: 3, max: Math.floor(totalHours * 0.3) }),
    'security': faker.number.int({ min: 2, max: Math.floor(totalHours * 0.2) }),
    'performance': faker.number.int({ min: 1, max: Math.floor(totalHours * 0.2) })
  };
  
  // Adjust to match total
  const categorySum = Object.values(categories).reduce((a, b) => a + b, 0);
  const adjustment = totalHours - categorySum;
  categories['code-smell'] += adjustment;

  return {
    total_debt_hours: totalHours,
    debt_by_category: categories,
    debt_trend: Array.from({ length: 6 }, () => faker.number.int({ min: totalHours - 20, max: totalHours + 20 })),
    priority_items: Array.from({ length: faker.number.int({ min: 3, max: 8 }) }, () => ({
      issue_id: faker.string.uuid(),
      debt_hours: faker.number.int({ min: 1, max: 12 }),
      roi_score: faker.number.float({ min: 0.1, max: 0.9, fractionDigits: 2 })
    })),
    ...overrides
  };
};

export const generateMockInteractiveReport = (overrides: Partial<InteractiveReport> = {}): InteractiveReport => {
  const issues = generateMockIssues(faker.number.int({ min: 10, max: 50 }));
  const dependencies = generateMockDependencies(faker.number.int({ min: 5, max: 20 }));
  const qualityMetrics = generateMockQualityMetrics();
  const technicalDebt = generateMockTechnicalDebt();

  return {
    id: faker.string.uuid(),
    project_name: faker.company.name(),
    generated_at: faker.date.recent({ days: 1 }).toISOString(),
    version: faker.system.semver(),
    summary: {
      total_issues: issues.length,
      critical_issues: issues.filter(i => i.severity === 'critical').length,
      high_issues: issues.filter(i => i.severity === 'high').length,
      medium_issues: issues.filter(i => i.severity === 'medium').length,
      low_issues: issues.filter(i => i.severity === 'low').length,
      files_analyzed: faker.number.int({ min: 20, max: 200 }),
      lines_of_code: faker.number.int({ min: 5000, max: 50000 }),
      test_coverage: qualityMetrics.test_coverage,
      estimated_fix_time: `${faker.number.int({ min: 8, max: 120 })} hours`
    },
    issues,
    quality_metrics: qualityMetrics,
    dependencies,
    technical_debt: technicalDebt,
    insights: Array.from({ length: faker.number.int({ min: 3, max: 7 }) }, () => 
      faker.lorem.sentence({ min: 8, max: 15 })
    ),
    recommendations: Array.from({ length: faker.number.int({ min: 2, max: 5 }) }, () =>
      faker.lorem.sentence({ min: 10, max: 20 })
    ),
    ...overrides
  };
};

export const generateHistoricalSnapshots = (count: number) =>
  Array.from({ length: count }, (_, i) => {
    const date = faker.date.past({ years: 1 });
    const baseScore = faker.number.int({ min: 70, max: 90 });
    
    return {
      id: faker.string.uuid(),
      timestamp: new Date(date.getTime() + i * 24 * 60 * 60 * 1000).toISOString(),
      qualityScore: baseScore + faker.number.int({ min: -10, max: 10 }),
      issueCount: faker.number.int({ min: 50, max: 200 }),
      technicalDebt: faker.number.int({ min: 10000, max: 50000 }),
      testCoverage: faker.number.int({ min: 60, max: 90 }),
      linesOfCode: faker.number.int({ min: 8000, max: 25000 })
    };
  });

export const generateTeamMembers = (count: number) =>
  Array.from({ length: count }, () => ({
    id: faker.string.uuid(),
    name: faker.person.fullName(),
    email: faker.internet.email(),
    avatar: faker.image.avatar(),
    role: faker.helpers.arrayElement(['Senior Developer', 'Technical Lead', 'QA Engineer', 'DevOps Engineer']),
    contributions: faker.number.int({ min: 10, max: 100 }),
    status: faker.helpers.arrayElement(['online', 'offline', 'away']),
    lastSeen: faker.helpers.arrayElement([null, '2 hours ago', '1 day ago', 'Last week'])
  }));

export const generateDiscussions = (count: number) =>
  Array.from({ length: count }, () => ({
    id: faker.string.uuid(),
    issueId: faker.string.uuid(),
    title: faker.lorem.sentence({ min: 4, max: 8 }),
    participants: faker.helpers.arrayElements(['alice', 'bob', 'carol', 'david'], { min: 2, max: 4 }),
    messageCount: faker.number.int({ min: 1, max: 20 }),
    lastActivity: faker.date.recent({ days: 7 }).toISOString(),
    status: faker.helpers.arrayElement(['active', 'resolved', 'pending'])
  }));

export const generateCodeReviews = (count: number) =>
  Array.from({ length: count }, () => ({
    id: faker.string.uuid(),
    issueId: faker.string.uuid(),
    title: faker.lorem.sentence({ min: 3, max: 10 }),
    reviewer: faker.helpers.arrayElement(['alice', 'bob', 'carol', 'david']),
    status: faker.helpers.arrayElement(['pending', 'approved', 'changes-requested']),
    createdAt: faker.date.recent({ days: 14 }).toISOString(),
    priority: faker.helpers.arrayElement(['low', 'medium', 'high'])
  }));

// Performance test data generators
export const generateLargeDataset = () => ({
  issues: generateMockIssues(10000),
  dependencies: generateMockDependencies(5000),
  historicalData: generateHistoricalSnapshots(365), // 1 year of daily snapshots
  teamMembers: generateTeamMembers(50),
  discussions: generateDiscussions(500),
  codeReviews: generateCodeReviews(200)
});

// Stress test data generators  
export const generateStressTestData = () => ({
  issues: generateMockIssues(50000),
  dependencies: generateMockDependencies(20000),
  historicalData: generateHistoricalSnapshots(1000),
  complexMetrics: {
    ...generateMockQualityMetrics(),
    trends: {
      overall_score: Array.from({ length: 1000 }, () => faker.number.int({ min: 50, max: 100 })),
      maintainability: Array.from({ length: 1000 }, () => faker.number.int({ min: 40, max: 95 })),
      reliability: Array.from({ length: 1000 }, () => faker.number.int({ min: 60, max: 98 })),
      security: Array.from({ length: 1000 }, () => faker.number.int({ min: 30, max: 90 })),
      performance: Array.from({ length: 1000 }, () => faker.number.int({ min: 45, max: 95 })),
      test_coverage: Array.from({ length: 1000 }, () => faker.number.int({ min: 40, max: 90 }))
    }
  }
});

// Edge case data generators
export const generateEdgeCaseData = () => ({
  emptyReport: {
    ...generateMockInteractiveReport(),
    issues: [],
    dependencies: [],
    technical_debt: {
      total_debt_hours: 0,
      debt_by_category: {},
      debt_trend: [],
      priority_items: []
    }
  },
  singleIssueReport: {
    ...generateMockInteractiveReport(),
    issues: [generateMockIssue()]
  },
  extremeQualityScores: {
    perfect: generateMockQualityMetrics({ 
      overall_score: 100,
      maintainability: 100,
      reliability: 100,
      security: 100,
      performance: 100,
      test_coverage: 100
    }),
    worst: generateMockQualityMetrics({
      overall_score: 1,
      maintainability: 1,
      reliability: 1,
      security: 1,
      performance: 1,
      test_coverage: 0
    })
  },
  longTextData: generateMockIssue({
    title: faker.lorem.sentence({ min: 50, max: 100 }),
    description: faker.lorem.paragraphs(10, '\n\n'),
    file: `src/very/deeply/nested/folder/structure/with/many/levels/and/long/names/${Array.isArray(faker.lorem.words(5)) ? faker.lorem.words(5).join('-') : faker.lorem.words(5)}.tsx`
  }),
  specialCharacterData: generateMockIssue({
    title: 'Issue with "quotes", <brackets>, & special chars: éñ中文',
    description: 'Description with special chars: @#$%^&*(){}[]|\\:";\'<>?,./',
    file: 'src/components/weird-file-name!@#$%.tsx'
  })
});