module.exports = {
  extends: [
    'eslint:recommended',
    '@typescript-eslint/recommended'
  ],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  rules: {
    // Relaxed rules for alpha development
    '@typescript-eslint/no-unused-vars': ['warn', { 
      varsIgnorePattern: '^_',
      argsIgnorePattern: '^_' 
    }],
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-unused-expressions': 'warn',
    '@typescript-eslint/no-namespace': 'off', // Allow for Cypress types
    'prefer-const': 'warn',
    'react-hooks/exhaustive-deps': 'warn'
  },
  ignorePatterns: [
    'cypress/**/*.js', // Ignore plain JS Cypress files
    'dist/',
    'node_modules/'
  ]
}