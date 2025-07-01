# Uveddi Frontend E2E Testing Guide

## Overview

This document provides comprehensive guidance for running and maintaining end-to-end tests for the Uveddi frontend application. Our test suite covers all critical user journeys, accessibility standards, and performance requirements.

## Test Architecture

### Test Framework
- **Cypress**: Modern E2E testing framework with excellent debugging capabilities
- **TypeScript**: Type-safe test development
- **Real Events**: `cypress-real-events` for authentic user interactions
- **Testing Library**: `@testing-library/cypress` for semantic queries

### Test Structure
```
cypress/
├── e2e/                     # End-to-end test specs
│   ├── landing-page.cy.ts   # Landing page functionality
│   ├── authentication.cy.ts # Login/register/logout flows
│   ├── dashboard.cy.ts      # Dashboard features
│   ├── analysis-details.cy.ts # Analysis viewing & reports
│   ├── user-journey.cy.ts   # Complete user workflows
│   ├── performance.cy.ts    # Performance benchmarks
│   └── accessibility.cy.ts  # A11y compliance tests
├── fixtures/                # Test data
│   ├── users.json          # User account data
│   └── analyses.json       # Sample analysis data
├── support/                 # Test utilities
│   ├── commands.ts         # Custom Cypress commands
│   ├── e2e.ts             # Global test setup
│   └── component.ts       # Component test setup
└── scripts/
    └── run-tests.sh        # Test execution script
```

## Quick Start

### Prerequisites
1. Node.js 18+ installed
2. Frontend dependencies installed (`npm install`)
3. Frontend dev server running (`npm run dev`)

### Running Tests

#### Option 1: Using the Test Script (Recommended)
```bash
# Run all tests headlessly
./cypress/scripts/run-tests.sh

# Run tests with visible browser
./cypress/scripts/run-tests.sh --headed

# Open interactive Cypress UI
./cypress/scripts/run-tests.sh --interactive

# Run specific test file
./cypress/scripts/run-tests.sh --spec landing-page.cy.ts

# Use different browser
./cypress/scripts/run-tests.sh --browser firefox
```

#### Option 2: Direct NPM Commands
```bash
# Run all tests headlessly
npm run cy:run

# Run tests with dev server
npm run test:e2e

# Open Cypress UI
npm run cy:open

# Run with specific browser
npm run cy:run:chrome
npm run cy:run:firefox
```

#### Option 3: Direct Cypress Commands
```bash
# Run all tests
npx cypress run

# Run specific test
npx cypress run --spec "cypress/e2e/landing-page.cy.ts"

# Run with headed browser
npx cypress run --headed

# Open interactive mode
npx cypress open
```

## Test Categories

### 1. Landing Page Tests (`landing-page.cy.ts`)
**Purpose**: Verify the main entry point of the application

**Coverage**:
- Hero section display and content
- Animated terminal functionality
- Navigation links and routing
- Feature cards and statistics
- Responsive design (mobile/tablet/desktop)
- Accessibility attributes
- Page load performance

**Key Test Cases**:
```typescript
// Example test structure
describe('Landing Page', () => {
  it('should display the main hero section', () => {
    cy.visit('/')
    cy.get('[data-testid="hero-section"]').should('be.visible')
    cy.get('h1').should('contain.text', 'AI-Powered Code Architecture Analysis')
  })
})
```

### 2. Authentication Tests (`authentication.cy.ts`)
**Purpose**: Ensure secure user authentication flows

**Coverage**:
- User registration with validation
- User login with error handling
- Logout functionality
- Protected route access
- Form validation and error states
- API error handling

**Key Features Tested**:
- Email/password validation
- Registration success/failure scenarios
- Login with valid/invalid credentials
- Session management
- Automatic redirects

### 3. Dashboard Tests (`dashboard.cy.ts`)
**Purpose**: Verify main application functionality after login

**Coverage**:
- Analysis list display
- Search and filtering
- Pagination and sorting
- New analysis creation
- Loading states and error handling
- Empty states
- Mobile responsiveness

**Key Features Tested**:
- Analysis cards rendering
- Search functionality
- Filter by status/language
- Create new analysis modal
- Performance with large datasets

### 4. Analysis Details Tests (`analysis-details.cy.ts`)
**Purpose**: Test detailed analysis viewing and reporting

**Coverage**:
- Analysis overview display
- Anti-pattern listings
- Code snippet viewing
- Report generation and export
- Sharing functionality
- Chart and visualization rendering

**Key Features Tested**:
- Detailed metrics display
- Interactive anti-pattern exploration
- Markdown report rendering
- Export to PDF/JSON
- Social sharing features

### 5. User Journey Tests (`user-journey.cy.ts`)
**Purpose**: Test complete end-to-end user workflows

**Coverage**:
- Full registration → analysis → results flow
- Error recovery scenarios
- Session persistence
- Offline functionality
- Cross-device compatibility

**Key Scenarios**:
- New user onboarding
- Returning user workflow
- Error handling and recovery
- Multi-device usage patterns

### 6. Performance Tests (`performance.cy.ts`)
**Purpose**: Ensure application meets performance standards

**Coverage**:
- Page load times (< 2-3 seconds)
- API response times (< 1 second)
- Large dataset handling
- Memory usage monitoring
- Bundle size validation
- Animation frame rates

**Performance Budgets**:
- Landing page: < 2 seconds
- Dashboard: < 3 seconds
- API responses: < 1 second
- Search/filter: < 500ms
- Memory usage: < 50MB

### 7. Accessibility Tests (`accessibility.cy.ts`)
**Purpose**: Ensure WCAG 2.1 AA compliance

**Coverage**:
- Semantic HTML structure
- ARIA labels and roles
- Keyboard navigation
- Screen reader compatibility
- Color contrast ratios
- Focus management
- Motion preferences

**Accessibility Standards**:
- Proper heading hierarchy (h1-h6)
- Alt text for images
- Form labels and associations
- Focus indicators
- Skip links
- Error announcements

## Custom Commands

Our test suite includes custom Cypress commands for common operations:

```typescript
// Authentication
cy.login(email, password)           // Login user
cy.register(name, email, password)  // Register new user
cy.logout()                         // Logout current user

// UI Interactions
cy.waitForElement(selector, timeout) // Wait for element
cy.mockApiResponse(method, url, response) // Mock API calls

// Data Management
cy.seedDatabase()    // Seed test data
cy.cleanDatabase()   // Clean test data
```

## Test Data Management

### Fixtures
Test data is stored in JSON fixtures:

```json
// cypress/fixtures/users.json
{
  "testUser": {
    "name": "Test User",
    "email": "test@uveddi.com", 
    "password": "testpassword123"
  }
}

// cypress/fixtures/analyses.json
{
  "analyses": [
    {
      "id": "1",
      "projectName": "Sample Project",
      "status": "completed",
      // ... more fields
    }
  ]
}
```

### API Mocking
We use Cypress intercepts to mock API responses:

```typescript
// Mock successful API response
cy.mockApiResponse('GET', '/api/analyses', {
  statusCode: 200,
  body: analyses
})

// Mock error response
cy.mockApiResponse('POST', '/api/auth/login', {
  statusCode: 401,
  body: { error: 'Invalid credentials' }
})
```

## Environment Configuration

### Test Environments
- **Local Development**: `http://localhost:9999`
- **Staging**: Configure via `baseUrl` in cypress.config.ts
- **CI/CD**: Automated testing in pipeline

### Environment Variables
```bash
# cypress.config.ts environment settings
env: {
  apiUrl: 'http://localhost:8000',
  coverage: true
}
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: E2E Tests
on: [push, pull_request]

jobs:
  cypress-run:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: cypress-io/github-action@v5
        with:
          working-directory: frontend
          start: npm run dev
          wait-on: 'http://localhost:9999'
          browser: chrome
```

### Test Reports
- **Screenshots**: Automatic on test failure
- **Videos**: Complete test run recordings  
- **Mochawesome Reports**: HTML test reports
- **Coverage Reports**: Code coverage metrics

## Debugging and Troubleshooting

### Common Issues

1. **Test Timeouts**
   ```typescript
   // Increase timeout for slow operations
   cy.get('[data-testid="element"]', { timeout: 10000 })
   ```

2. **Flaky Tests**
   ```typescript
   // Add explicit waits
   cy.wait('@apiCall')
   cy.waitForElement('[data-testid="element"]')
   ```

3. **API Integration Issues**
   ```typescript
   // Verify API mocking
   cy.intercept('GET', '/api/**').as('apiCall')
   cy.wait('@apiCall').then((interception) => {
     expect(interception.response.statusCode).to.eq(200)
   })
   ```

### Debug Mode
```bash
# Run tests with debug output
DEBUG=cypress:* npm run cy:run

# Open dev tools in headed mode  
npm run cy:run -- --headed --browser chrome
```

### Test Isolation
Each test is isolated with:
- Fresh browser context
- Cleared localStorage/cookies
- Reset application state
- Clean database state

## Best Practices

### Test Writing
1. **Use data-testid attributes** for reliable element selection
2. **Test user behavior**, not implementation details
3. **Keep tests focused** and single-purpose
4. **Use descriptive test names** that explain the scenario
5. **Mock external dependencies** consistently

### Selectors
```typescript
// ✅ Good - semantic and stable
cy.get('[data-testid="login-button"]')
cy.get('[aria-label="Search analyses"]')

// ❌ Avoid - brittle and implementation-dependent
cy.get('.btn-primary')
cy.get('#login-form > div:nth-child(3)')
```

### Assertions
```typescript
// ✅ Good - clear and specific
cy.get('[data-testid="error-message"]')
  .should('be.visible')
  .should('contain.text', 'Invalid email format')

// ❌ Avoid - vague and unreliable
cy.get('.error').should('exist')
```

### Test Organization
```typescript
describe('Feature Group', () => {
  beforeEach(() => {
    // Common setup
  })

  describe('Specific Functionality', () => {
    it('should handle specific scenario', () => {
      // Test implementation
    })
  })
})
```

## Maintenance

### Regular Tasks
1. **Update fixtures** when API changes
2. **Review and update selectors** for UI changes
3. **Monitor test performance** and optimize slow tests
4. **Update accessibility standards** as requirements evolve
5. **Review test coverage** and add tests for new features

### Test Health Monitoring
- Monitor test execution times
- Track flaky test patterns
- Review failure rates and common issues
- Update browser versions and dependencies

## Performance Monitoring

### Metrics Tracked
- Page load times
- Time to first contentful paint
- Time to interactive
- Bundle sizes
- Memory usage
- API response times

### Performance Budgets
Our tests enforce strict performance budgets:
- Landing page load: < 2 seconds
- Dashboard load: < 3 seconds  
- Search response: < 500ms
- Filter response: < 300ms
- Bundle size: < 1MB (main), < 2MB (vendor)

## Accessibility Standards

### WCAG 2.1 AA Compliance
Our tests verify:
- ✅ Keyboard navigation support
- ✅ Screen reader compatibility
- ✅ Sufficient color contrast (4.5:1)
- ✅ Proper heading hierarchy
- ✅ Form labels and error announcements
- ✅ Focus management in modals
- ✅ Skip links for navigation
- ✅ Reduced motion respect

### Testing Tools Integration
- axe-core accessibility testing
- Color contrast validation
- Keyboard navigation verification
- Screen reader simulation

---

This comprehensive test suite ensures Uveddi's frontend meets the highest standards of functionality, performance, and accessibility. Regular execution of these tests provides confidence in code quality and user experience across all supported devices and use cases.
