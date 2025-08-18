module.exports = {
  // Test environment
  testEnvironment: 'jsdom',
  
  // Setup files
  setupFilesAfterEnv: ['<rootDir>/src/test-utils/setup.ts'],
  
  // Module name mapping for imports
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
  },
  
  // File extensions to consider
  moduleFileExtensions: [
    'ts',
    'tsx', 
    'js',
    'jsx',
    'json',
    'node'
  ],
  
  // Transform configuration
  transform: {
    '^.+\\.(ts|tsx)$': ['ts-jest', {
      tsconfig: {
        jsx: 'react-jsx',
      },
    }],
    '^.+\\.(js|jsx)$': ['babel-jest', {
      presets: [
        ['@babel/preset-env', { targets: { node: 'current' } }],
        ['@babel/preset-react', { runtime: 'automatic' }],
      ],
    }],
  },
  
  // Test match patterns
  testMatch: [
    '<rootDir>/src/**/__tests__/**/*.(ts|tsx|js)',
    '<rootDir>/src/**/*.(test|spec).(ts|tsx|js)',
  ],
  
  // Files to ignore
  testPathIgnorePatterns: [
    '<rootDir>/node_modules/',
    '<rootDir>/build/',
    '<rootDir>/dist/',
  ],
  
  // Transform ignore patterns
  transformIgnorePatterns: [
    'node_modules/(?!(react-grid-layout|cytoscape|chart.js|fuse.js)/)',
  ],
  
  // Coverage configuration
  collectCoverage: true,
  collectCoverageFrom: [
    'src/components/dashboard/**/*.{ts,tsx}',
    'src/utils/**/*.{ts,tsx}',
    'src/types/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/**/*.stories.{ts,tsx}',
    '!src/test-utils/**/*',
    '!src/**/__tests__/**/*',
  ],
  
  coverageThreshold: {
    global: {
      branches: 85,
      functions: 90,
      lines: 90,
      statements: 90,
    },
    './src/components/dashboard/': {
      branches: 80,
      functions: 85,
      lines: 85,
      statements: 85,
    },
    './src/utils/': {
      branches: 90,
      functions: 95,
      lines: 95,
      statements: 95,
    },
  },
  
  coverageReporters: [
    'text',
    'text-summary',
    'html',
    'lcov',
    'json-summary',
  ],
  
  coverageDirectory: '<rootDir>/coverage',
  
  // Global setup and teardown
  globalSetup: undefined,
  globalTeardown: undefined,
  
  // Test timeout
  testTimeout: 30000,
  
  // Clear mocks between tests
  clearMocks: true,
  restoreMocks: true,
  resetMocks: true,
  
  // Verbose output
  verbose: true,
  
  // Error handling
  errorOnDeprecated: true,
  
  // Watch plugins for better development experience
  watchPlugins: [
    'jest-watch-typeahead/filename',
    'jest-watch-typeahead/testname',
  ],
  
  // Additional Jest configuration for specific test types
  projects: [
    {
      displayName: 'unit',
      testMatch: ['<rootDir>/src/components/**/*.test.(ts|tsx)'],
      setupFilesAfterEnv: ['<rootDir>/src/test-utils/setup.ts'],
    },
    {
      displayName: 'integration',
      testMatch: ['<rootDir>/src/__tests__/integration/**/*.test.(ts|tsx)'],
      setupFilesAfterEnv: ['<rootDir>/src/test-utils/setup.ts'],
    },
    {
      displayName: 'performance',
      testMatch: ['<rootDir>/src/__tests__/performance/**/*.test.(ts|tsx)'],
      setupFilesAfterEnv: ['<rootDir>/src/test-utils/setup.ts'],
      testTimeout: 60000, // Longer timeout for performance tests
    },
    {
      displayName: 'accessibility',
      testMatch: ['<rootDir>/src/__tests__/accessibility/**/*.test.(ts|tsx)'],
      setupFilesAfterEnv: ['<rootDir>/src/test-utils/setup.ts'],
    },
    {
      displayName: 'coverage',
      testMatch: ['<rootDir>/src/__tests__/coverage/**/*.test.(ts|tsx)'],
      setupFilesAfterEnv: ['<rootDir>/src/test-utils/setup.ts'],
    },
  ],
  
  // Reporter configuration for CI/CD
  reporters: [
    'default',
    ['jest-junit', {
      outputDirectory: 'test-results',
      outputName: 'junit.xml',
      classNameTemplate: '{classname}',
      titleTemplate: '{title}',
      ancestorSeparator: ' › ',
      usePathForSuiteName: true,
    }],
  ],
  
  // Mock configuration
  modulePathIgnorePatterns: ['<rootDir>/build/'],
  
  // Additional environment variables for tests
  testEnvironmentOptions: {
    url: 'http://localhost:3000',
  },
  
  // Custom resolver for handling complex module resolution
  resolver: undefined,
  
  // Snapshot serializer configuration
  snapshotSerializers: ['@emotion/jest/serializer'],
  
  // Dependency extraction configuration
  dependencyExtractor: undefined,
  
  // Handle CSS modules and static assets
  moduleNameMapping: {
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
    '\\.(jpg|jpeg|png|gif|eot|otf|webp|svg|ttf|woff|woff2|mp4|webm|wav|mp3|m4a|aac|oga)$':
      '<rootDir>/src/test-utils/mocks/fileMock.js',
  },
};