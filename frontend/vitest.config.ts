import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    // Test environment
    environment: 'jsdom',
    
    // Setup files
    setupFiles: ['./src/test-utils/setup.ts'],
    
    // Global test configuration
    globals: true,
    
    // Coverage configuration
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      reportsDirectory: './coverage',
      exclude: [
        'node_modules/',
        'src/test-utils/',
        '**/*.d.ts',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        'src/**/__tests__/**',
        'src/**/*.stories.{ts,tsx}',
      ],
      include: [
        'src/components/dashboard/**/*.{ts,tsx}',
        'src/utils/**/*.{ts,tsx}',
        'src/types/**/*.{ts,tsx}',
      ],
      thresholds: {
        global: {
          branches: 85,
          functions: 90,
          lines: 90,
          statements: 90,
        },
      },
    },
    
    // Test timeout
    testTimeout: 30000,
    
    // Include patterns
    include: [
      'src/**/*.{test,spec}.{ts,tsx}',
      'src/__tests__/**/*.{test,spec}.{ts,tsx}',
    ],
    
    // Exclude patterns  
    exclude: [
      'node_modules/',
      'build/',
      'dist/',
      'coverage/',
    ],
    
    // Mock configuration
    clearMocks: true,
    restoreMocks: true,
    
    // Reporter configuration
    reporter: ['verbose', 'junit'],
    outputFile: {
      junit: './test-results/junit.xml',
    },
    
    // Workspace configuration for different test types
    workspace: [
      {
        test: {
          name: 'unit',
          include: ['src/components/**/*.test.{ts,tsx}'],
          environment: 'jsdom',
        },
      },
      {
        test: {
          name: 'integration', 
          include: ['src/__tests__/integration/**/*.test.{ts,tsx}'],
          environment: 'jsdom',
        },
      },
      {
        test: {
          name: 'performance',
          include: ['src/__tests__/performance/**/*.test.{ts,tsx}'],
          environment: 'jsdom',
          testTimeout: 60000,
        },
      },
      {
        test: {
          name: 'accessibility',
          include: ['src/__tests__/accessibility/**/*.test.{ts,tsx}'],
          environment: 'jsdom',
        },
      },
      {
        test: {
          name: 'coverage',
          include: ['src/__tests__/coverage/**/*.test.{ts,tsx}'],
          environment: 'jsdom',
        },
      },
    ],
  },
  
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});