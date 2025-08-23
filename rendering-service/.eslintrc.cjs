/**
 * ESLint configuration for Uveddi Rendering Service
 * 
 * This configuration enforces JSDoc documentation requirements for the Node.js
 * rendering service, ensuring all exported functions and modules are properly documented.
 */

module.exports = {
  root: true,
  env: { 
    node: true, 
    es2020: true,
    jest: true
  },
  extends: [
    'eslint:recommended',
    'plugin:jsdoc/recommended',
  ],
  plugins: ['jsdoc'],
  parserOptions: {
    ecmaVersion: 2020,
    sourceType: 'module',
  },
  ignorePatterns: ['node_modules/', 'dist/', '.eslintrc.cjs'],
  rules: {
    // JSDoc enforcement rules
    'jsdoc/require-jsdoc': [
      'error',
      {
        require: {
          FunctionDeclaration: true,
          FunctionExpression: false,
          ArrowFunctionExpression: false,
          ClassDeclaration: true,
          MethodDefinition: false,
        },
        publicOnly: true,
      },
    ],
    'jsdoc/check-alignment': 'error',
    'jsdoc/check-indentation': 'warn',
    'jsdoc/require-param': 'error',
    'jsdoc/require-returns': 'error',
    'jsdoc/require-description': 'warn',
    'jsdoc/require-file-overview': [
      'error',
      {
        tags: {
          file: {
            initialCommentsOnly: true,
            mustExist: true,
          },
        },
      },
    ],
    'jsdoc/check-tag-names': 'error',
    'jsdoc/check-types': 'error',
    'jsdoc/valid-types': 'error',
  },
};