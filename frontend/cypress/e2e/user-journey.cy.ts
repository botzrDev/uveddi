describe('End-to-End User Journey', () => {
  beforeEach(() => {
    cy.clearLocalStorage()
    cy.clearCookies()
  })

  it('should complete full user journey from registration to analysis', () => {
    cy.fixture('users').then((users) => {
      const testUser = users.testUser
      
      // Step 1: Visit landing page
      cy.visit('/')
      cy.get('[data-testid="hero-section"]').should('be.visible')
      
      // Step 2: Navigate to registration
      cy.get('[data-testid="get-started-button"]').click()
      cy.url().should('include', '/register')
      
      // Step 3: Register new user
      cy.mockApiResponse('POST', '/api/auth/register', {
        statusCode: 201,
        body: {
          success: true,
          user: { id: 1, email: testUser.email, name: testUser.name },
          token: 'mock-jwt-token'
        }
      })
      
      cy.get('[data-testid="name-input"]').type(testUser.name)
      cy.get('[data-testid="email-input"]').type(testUser.email)
      cy.get('[data-testid="password-input"]').type(testUser.password)
      cy.get('[data-testid="confirm-password-input"]').type(testUser.password)
      cy.get('[data-testid="register-button"]').click()
      
      // Step 4: Should be redirected to dashboard
      cy.url().should('include', '/dashboard')
      cy.get('[data-testid="dashboard"]').should('be.visible')
      
      // Step 5: Start new analysis
      cy.mockApiResponse('POST', '/api/analyses', {
        statusCode: 201,
        body: { id: 1, status: 'queued' }
      })
      
      cy.get('[data-testid="new-analysis-button"]').click()
      cy.get('[data-testid="new-analysis-modal"]').should('be.visible')
      
      cy.get('[data-testid="project-path-input"]').type('/path/to/test/project')
      cy.get('[data-testid="language-select"]').select('rust')
      cy.get('[data-testid="analysis-level-select"]').select('high')
      cy.get('[data-testid="start-analysis-button"]').click()
      
      // Step 6: Mock analysis completion and navigate to results
      cy.fixture('analyses').then((analysesData) => {
        cy.mockApiResponse('GET', '/api/analyses', {
          statusCode: 200,
          body: [analysesData.singleAnalysis]
        })
        
        cy.mockApiResponse('GET', '/api/analyses/1', {
          statusCode: 200,
          body: analysesData.singleAnalysis
        })
      })
      
      cy.get('[data-testid="success-message"]').should('be.visible')
      cy.get('[data-testid="view-results-button"]').click()
      
      // Step 7: View analysis results
      cy.url().should('include', '/analysis/1')
      cy.get('[data-testid="analysis-details"]').should('be.visible')
      cy.get('[data-testid="summary-section"]').should('be.visible')
      cy.get('[data-testid="anti-patterns-section"]').should('be.visible')
      
      // Step 8: Export results
      cy.get('[data-testid="export-button"]').click()
      cy.get('[data-testid="export-pdf"]').click()
      cy.get('[data-testid="export-success"]').should('be.visible')
      
      // Step 9: Logout
      cy.get('[data-testid="user-menu"]').click()
      cy.get('[data-testid="logout-button"]').click()
      cy.url().should('include', '/login')
    })
  })

  it('should handle user journey with existing account', () => {
    cy.fixture('users').then((users) => {
      const testUser = users.testUser
      
      // Step 1: Direct login
      cy.visit('/login')
      
      cy.mockApiResponse('POST', '/api/auth/login', {
        statusCode: 200,
        body: {
          success: true,
          user: { id: 1, email: testUser.email, name: testUser.name },
          token: 'mock-jwt-token'
        }
      })
      
      cy.get('[data-testid="email-input"]').type(testUser.email)
      cy.get('[data-testid="password-input"]').type(testUser.password)
      cy.get('[data-testid="login-button"]').click()
      
      // Step 2: View existing analyses
      cy.fixture('analyses').then((analysesData) => {
        cy.mockApiResponse('GET', '/api/analyses', {
          statusCode: 200,
          body: analysesData.analyses
        })
      })
      
      cy.url().should('include', '/dashboard')
      cy.get('[data-testid="analysis-card"]').should('have.length.at.least', 1)
      
      // Step 3: View analysis details
      cy.fixture('analyses').then((analysesData) => {
        cy.mockApiResponse('GET', '/api/analyses/1', {
          statusCode: 200,
          body: analysesData.singleAnalysis
        })
      })
      
      cy.get('[data-testid="analysis-card"]').first().click()
      cy.get('[data-testid="analysis-details"]').should('be.visible')
    })
  })

  it('should handle error scenarios gracefully', () => {
    // Test network failures
    cy.fixture('users').then((users) => {
      const testUser = users.testUser
      
      // Step 1: Failed login due to network error
      cy.visit('/login')
      
      cy.mockApiResponse('POST', '/api/auth/login', {
        statusCode: 500,
        body: { error: 'Internal server error' }
      })
      
      cy.get('[data-testid="email-input"]').type(testUser.email)
      cy.get('[data-testid="password-input"]').type(testUser.password)
      cy.get('[data-testid="login-button"]').click()
      
      cy.get('[data-testid="error-message"]').should('contain.text', 'server error')
      
      // Step 2: Retry with successful login
      cy.mockApiResponse('POST', '/api/auth/login', {
        statusCode: 200,
        body: {
          success: true,
          user: { id: 1, email: testUser.email, name: testUser.name },
          token: 'mock-jwt-token'
        }
      })
      
      cy.get('[data-testid="login-button"]').click()
      cy.url().should('include', '/dashboard')
    })
  })

  it('should maintain session across page refreshes', () => {
    cy.fixture('users').then((users) => {
      // Login first
      cy.login(users.testUser.email, users.testUser.password)
      
      // Navigate to dashboard
      cy.visit('/dashboard')
      cy.get('[data-testid="dashboard"]').should('be.visible')
      
      // Refresh page
      cy.reload()
      
      // Should still be logged in
      cy.url().should('include', '/dashboard')
      cy.get('[data-testid="dashboard"]').should('be.visible')
    })
  })

  it('should handle session expiry', () => {
    cy.fixture('users').then((users) => {
      // Login first
      cy.login(users.testUser.email, users.testUser.password)
      
      // Mock session expiry
      cy.mockApiResponse('GET', '/api/analyses', {
        statusCode: 401,
        body: { error: 'Token expired' }
      })
      
      cy.visit('/dashboard')
      
      // Should redirect to login
      cy.url().should('include', '/login')
      cy.get('[data-testid="session-expired-message"]').should('be.visible')
    })
  })

  it('should work offline with cached data', () => {
    cy.fixture('users').then((users) => {
      // Login and load data first
      cy.login(users.testUser.email, users.testUser.password)
      
      cy.fixture('analyses').then((analysesData) => {
        cy.mockApiResponse('GET', '/api/analyses', {
          statusCode: 200,
          body: analysesData.analyses
        })
      })
      
      cy.visit('/dashboard')
      cy.get('[data-testid="analysis-card"]').should('be.visible')
      
      // Simulate offline by intercepting all API calls
      cy.intercept('**/api/**', { forceNetworkError: true })
      
      cy.reload()
      
      // Should show cached data with offline indicator
      cy.get('[data-testid="offline-indicator"]').should('be.visible')
      cy.get('[data-testid="analysis-card"]').should('be.visible') // From cache
    })
  })

  it('should handle responsive design across devices', () => {
    cy.fixture('users').then((users) => {
      // Test on mobile
      cy.viewport(375, 667)
      cy.login(users.testUser.email, users.testUser.password)
      cy.visit('/dashboard')
      
      cy.get('[data-testid="mobile-dashboard"]').should('be.visible')
      cy.get('[data-testid="mobile-menu-button"]').should('be.visible')
      
      // Test on tablet
      cy.viewport(768, 1024)
      cy.reload()
      cy.get('[data-testid="tablet-layout"]').should('be.visible')
      
      // Test on desktop
      cy.viewport(1920, 1080)
      cy.reload()
      cy.get('[data-testid="desktop-layout"]').should('be.visible')
    })
  })
})
