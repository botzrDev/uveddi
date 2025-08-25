# JavaScript Project Analysis Guide

## Overview

This guide covers Uveddi analysis for JavaScript projects, including Node.js applications, React, Vue, Angular, and library projects.

## Key Analysis Areas

### 1. Dead Code Detection

JavaScript's dynamic nature presents unique challenges:

- **Unused functions and variables**
- **Unreachable code** after returns
- **Dynamic imports** and code splitting
- **Event handlers** that appear unused
- **Polyfills** and compatibility code

**Configuration Tips:**
```toml
[dead_code]
confidence_threshold = 0.6  # Lower due to dynamic nature
library_mode = false        # Usually false for applications
ignore_patterns = [
    "node_modules/**",
    "dist/**",
    "build/**",
    "test/**",
    "tests/**",
    "*.test.js",
    "*.spec.js",
    "coverage/**",
    ".next/**"              # Next.js build directory
]
keep_alive_patterns = [
    "main",
    "index",
    "app",
    "default",              # Default exports
    "module.exports",       # CommonJS exports
    "export"                # ES6 exports
]
```

### 2. Large Classes Detection

JavaScript-specific considerations:
- **Classes** and constructor functions
- **Object literals** with many methods
- **React components** with too many props/methods
- **Module objects** with many exports

**JavaScript-Specific Thresholds:**
- Max logical LOC: 250
- Max methods: 25
- Max properties: 15 (for objects/classes)

### 3. Architectural Anti-Patterns

**Common JavaScript Anti-Patterns:**
- **Callback hell** (nested callbacks)
- **Global variables** pollution
- **Monolithic modules** (everything in one file)
- **Import spaghetti** (complex dependency chains)
- **Magic values** in configuration
- **Prototype pollution**

### 4. JavaScript-Specific Analysis

**Module System Analysis:**
- CommonJS vs ES6 modules mixing
- Dynamic imports usage
- Tree-shaking opportunities
- Bundle size analysis

**Async Code Patterns:**
- Promise chain complexity
- async/await usage
- Error handling in async code
- Memory leaks in async operations

## Framework-Specific Guidelines

### React Projects

```toml
[large_classes]
# React components should stay small
max_logical_loc = 150
max_methods = 15

[dead_code]
keep_alive_patterns = [
    "Component",
    "render",
    "componentDidMount",
    "useEffect",
    "useState",
    "props",
    "default"               # Default exports for components
]

ignore_patterns = [
    "public/**",
    "build/**",
    "*.test.jsx",
    "*.stories.js",         # Storybook stories
    "__tests__/**"
]
```

**React-Specific Issues:**
- **Huge components** - Break into smaller components
- **Props drilling** - Use Context or state management
- **Unused props** - Clean up component interfaces
- **Effect cleanup** - Proper useEffect cleanup

### Node.js Projects

```toml
[dead_code]
keep_alive_patterns = [
    "main",
    "module.exports",
    "exports",
    "app.listen",
    "server.listen",
    "middleware"
]
```

**Node.js-Specific Issues:**
- **Callback hell** - Use Promises/async-await
- **Memory leaks** - Proper event listener cleanup
- **Middleware bloat** - Organize middleware properly
- **Global state** - Use proper dependency injection

### Vue.js Projects

```toml
[dead_code]
keep_alive_patterns = [
    "components",
    "data",
    "methods",
    "computed",
    "watch",
    "mounted",
    "created"
]
```

### Angular Projects

```toml
[dead_code]
keep_alive_patterns = [
    "Component",
    "Service",
    "NgModule",
    "Injectable",
    "OnInit",
    "OnDestroy",
    "constructor"
]

ignore_patterns = [
    "dist/**",
    "src/environments/**",
    "*.spec.ts"
]
```

## Best Practices

### Module Organization
```javascript
// Good: Clear module structure
src/
├── components/
│   ├── common/
│   │   ├── Button/
│   │   │   ├── index.js
│   │   │   ├── Button.js
│   │   │   └── Button.css
│   │   └── Modal/
│   └── pages/
├── services/
├── utils/
└── hooks/

// Bad: Everything in one directory
src/
├── Button.js
├── Modal.js
├── UserPage.js
├── AdminPage.js
├── api.js
├── utils.js
└── ... 50 more files
```

### Function and Class Design
```javascript
// Before: God function
function processUser(userData) {
    // Validation (50 lines)
    // Database operations (100 lines)
    // Email sending (30 lines)
    // Logging (20 lines)
    // Response formatting (40 lines)
}

// After: Separated concerns
function validateUser(userData) { /* ... */ }
function saveUser(userData) { /* ... */ }
function sendWelcomeEmail(user) { /* ... */ }
function logUserCreation(user) { /* ... */ }
function formatUserResponse(user) { /* ... */ }

async function processUser(userData) {
    const validUser = validateUser(userData);
    const savedUser = await saveUser(validUser);
    await sendWelcomeEmail(savedUser);
    logUserCreation(savedUser);
    return formatUserResponse(savedUser);
}
```

### Async Patterns
```javascript
// Before: Callback hell
function getUser(id, callback) {
    db.findUser(id, (err, user) => {
        if (err) return callback(err);
        getPermissions(user.id, (err, permissions) => {
            if (err) return callback(err);
            getPreferences(user.id, (err, preferences) => {
                if (err) return callback(err);
                callback(null, { user, permissions, preferences });
            });
        });
    });
}

// After: Async/await
async function getUser(id) {
    try {
        const user = await db.findUser(id);
        const [permissions, preferences] = await Promise.all([
            getPermissions(user.id),
            getPreferences(user.id)
        ]);
        return { user, permissions, preferences };
    } catch (error) {
        logger.error('Failed to get user:', error);
        throw error;
    }
}
```

## Common Issues and Solutions

### Issue: Large React Components
**Problem:** Components with 300+ lines and many responsibilities
**Solution:** Component composition and custom hooks

```jsx
// Before: Monolithic component
function UserDashboard({ userId }) {
    // 50+ lines of state management
    // 100+ lines of data fetching
    // 150+ lines of render logic
    return (
        <div>
            {/* 200+ lines of JSX */}
        </div>
    );
}

// After: Composed components with hooks
function useUserData(userId) {
    // Custom hook for data management
}

function UserProfile({ user }) {
    // Focused component for user profile
}

function UserStats({ stats }) {
    // Focused component for statistics
}

function UserDashboard({ userId }) {
    const { user, stats, loading } = useUserData(userId);
    
    if (loading) return <LoadingSpinner />;
    
    return (
        <div>
            <UserProfile user={user} />
            <UserStats stats={stats} />
        </div>
    );
}
```

### Issue: Global Variable Pollution
**Problem:** Too many variables in global scope
**Solution:** Modules and namespacing

```javascript
// Before: Global pollution
var currentUser = null;
var appConfig = {};
var apiEndpoints = {};

function login(user) { /* uses globals */ }
function logout() { /* uses globals */ }

// After: Module pattern
const AuthModule = {
    _currentUser: null,
    _config: {},
    
    login(user) {
        this._currentUser = user;
        return this._currentUser;
    },
    
    logout() {
        this._currentUser = null;
    },
    
    getCurrentUser() {
        return this._currentUser;
    }
};

export default AuthModule;
```

### Issue: Callback Hell
**Problem:** Deeply nested callbacks
**Solution:** Promises and async/await

```javascript
// Before: Callback pyramid
getData(id, (err, data) => {
    if (err) throw err;
    processData(data, (err, processed) => {
        if (err) throw err;
        saveData(processed, (err, result) => {
            if (err) throw err;
            notifyUser(result, (err) => {
                if (err) throw err;
                console.log('Done!');
            });
        });
    });
});

// After: Clean async/await
async function handleData(id) {
    try {
        const data = await getData(id);
        const processed = await processData(data);
        const result = await saveData(processed);
        await notifyUser(result);
        console.log('Done!');
    } catch (error) {
        console.error('Error:', error);
    }
}
```

## Analysis Commands

```bash
# Basic JavaScript analysis
uveddi analyze src/

# Include test files and config files
uveddi analyze . --include-patterns "*.config.js,*.test.js"

# React project analysis
uveddi analyze src/ --framework react --jsx

# Node.js server analysis
uveddi analyze . --node --include-patterns "server/**,api/**"

# Bundle size analysis
uveddi analyze src/ --detectors bundle-size,dependencies

# Generate HTML report with performance insights
uveddi analyze src/ --output-format html --enable-ai --detectors performance
```

## Package Manager Integration

### npm Projects
```bash
# Add analysis script to package.json
npm run analyze

# Or run directly
npx uveddi analyze src/
```

### Yarn Projects
```bash
# With Yarn
yarn analyze

# Or run directly
yarn dlx uveddi analyze src/
```

## CI/CD Integration

```yaml
# .github/workflows/analysis.yml
name: JavaScript Code Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
      - name: Install dependencies
        run: npm ci
      - name: Install Uveddi
        run: npm install -g uveddi
      - name: Run Analysis
        run: uveddi analyze src/ --output-format json --output analysis.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: analysis-results
          path: analysis.json
```

## Integration with JavaScript Tools

### ESLint
```bash
# Run ESLint before analysis
npx eslint src/ --fix
uveddi analyze src/
```

### Prettier
```bash
# Format code before analysis
npx prettier --write src/
uveddi analyze src/
```

### Webpack Bundle Analyzer
```bash
# Analyze bundle then run code analysis
npm run build -- --analyze
uveddi analyze src/ --include-bundle-info
```

## Performance Considerations

- **Large node_modules:** Always exclude with ignore patterns
- **Build directories:** Exclude dist/, build/, .next/ directories
- **Source maps:** Use source maps for better analysis accuracy
- **Memory usage:** Configure `--memory-limit` for large React/Angular apps

## Troubleshooting

### Module Resolution Issues
```bash
# Check Node.js path resolution
node -e "console.log(require.resolve('./src/index.js'))"

# Verify Uveddi can parse your modules
uveddi doctor --parsers --javascript

# Run with verbose module resolution
uveddi analyze src/ --verbose --log-modules
```

### React/JSX Issues
```bash
# Ensure JSX parsing is enabled
uveddi analyze src/ --jsx --typescript

# For React projects with TypeScript
uveddi analyze src/ --jsx --typescript --react
```

### Performance Issues
```bash
# Profile analysis performance
uveddi analyze src/ --profile

# Faster analysis for large projects
uveddi analyze src/ --fast-mode --exclude-patterns "node_modules/**"

# Memory-optimized analysis
uveddi analyze src/ --memory-optimization
```

For more JavaScript best practices, see [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript) and [MDN JavaScript Guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide).