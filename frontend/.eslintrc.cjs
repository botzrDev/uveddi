module.exports = {
  root: true,
  env: { browser: true, es2020: true },
  extends: [
    'eslint:recommended',
    '@typescript-eslint/recommended',
    'plugin:react-hooks/recommended',
    'plugin:jsdoc/recommended-typescript',
  ],
  ignorePatterns: ['dist', '.eslintrc.cjs'],
  parser: '@typescript-eslint/parser',
  plugins: ['react-refresh', 'jsdoc'],
  rules: {
    'react-refresh/only-export-components': [
      'warn',
      { allowConstantExport: true },
    ],
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    '@typescript-eslint/explicit-function-return-type': 'off',
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/no-explicit-any': 'warn',
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
        contexts: [
          'ExportDefaultDeclaration',
          'ExportNamedDeclaration',
          'TSInterfaceDeclaration',
          'TSTypeAliasDeclaration',
        ],
        publicOnly: true,
      },
    ],
    'jsdoc/check-alignment': 'error',
    'jsdoc/check-indentation': 'warn',
    'jsdoc/require-param': 'error',
    'jsdoc/require-returns': ['error', { forceRequireReturn: false }],
    'jsdoc/require-description': ['warn', { contexts: ['any'] }],
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
    // Disable some noisy JSDoc rules for React components
    'jsdoc/require-returns': 'off', // React components don't always need return docs
    'jsdoc/require-param': 'off', // Props are self-documenting in TypeScript
  },
};