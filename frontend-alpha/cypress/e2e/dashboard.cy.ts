describe('Dashboard', () => {
  beforeEach(() => {
    // Login before each test
    cy.fixture('users').then((users) => {
      cy.login(users.testUser.email, users.testUser.password)
    })
    
    // Mock analyses API
    cy.fixture('analyses').then((analysesData) => {
      cy.mockApiResponse('GET', '/api/analyses', {
        statusCode: 200,
        body: analysesData.analyses
      })
    })
  })

  it('should display dashboard correctly', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="dashboard"]').should('be.visible')
    cy.get('[data-testid="dashboard-header"]').should('contain.text', 'Dashboard')
    cy.get('[data-testid="user-welcome"]').should('be.visible')
  })

  it('should display analyses list', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="analyses-section"]').should('be.visible')
    cy.get('[data-testid="analysis-card"]').should('have.length.at.least', 1)
    
    // Check first analysis card content
    cy.get('[data-testid="analysis-card"]').first().within(() => {
      cy.get('[data-testid="project-name"]').should('contain.text', 'Sample Project')
      cy.get('[data-testid="analysis-status"]').should('contain.text', 'completed')
      cy.get('[data-testid="language-tag"]').should('contain.text', 'rust')
    })
  })

  it('should display analysis statistics', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="stats-overview"]').should('be.visible')
    cy.get('[data-testid="total-analyses"]').should('be.visible')
    cy.get('[data-testid="critical-issues"]').should('be.visible')
    cy.get('[data-testid="recent-activity"]').should('be.visible')
  })

  it('should have working search functionality', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="search-input"]').should('be.visible')
    cy.get('[data-testid="search-input"]').type('Sample Project')
    
    // Should filter results
    cy.get('[data-testid="analysis-card"]').should('have.length', 1)
    cy.get('[data-testid="analysis-card"]').first().should('contain.text', 'Sample Project')
  })

  it('should filter analyses by status', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="status-filter"]').select('completed')
    cy.get('[data-testid="analysis-card"]').each(($card) => {
      cy.wrap($card).find('[data-testid="analysis-status"]').should('contain.text', 'completed')
    })
  })

  it('should sort analyses by different criteria', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="sort-selector"]').select('date-desc')
    // Verify sorting order (most recent first)
    cy.get('[data-testid="analysis-card"]').first()
      .find('[data-testid="created-date"]')
      .should('contain.text', '2025-07-01')
  })

  it('should navigate to analysis details', () => {
    cy.visit('/dashboard')
    
    cy.fixture('analyses').then((analysesData) => {
      // Mock single analysis API
      cy.mockApiResponse('GET', '/api/analyses/1', {
        statusCode: 200,
        body: analysesData.singleAnalysis
      })
    })
    
    cy.get('[data-testid="analysis-card"]').first().click()
    cy.url().should('include', '/analysis/')
    cy.get('[data-testid="analysis-details"]').should('be.visible')
  })

  it('should start new analysis', () => {
    cy.visit('/dashboard')
    
    cy.get('[data-testid="new-analysis-button"]').click()
    cy.get('[data-testid="new-analysis-modal"]').should('be.visible')
    
    // Fill in analysis form
    cy.get('[data-testid="project-path-input"]').type('/path/to/new/project')
    cy.get('[data-testid="language-select"]').select('rust')
    cy.get('[data-testid="analysis-level-select"]').select('high')
    
    // Mock analysis creation API
    cy.mockApiResponse('POST', '/api/analyses', {
      statusCode: 201,
      body: { id: 3, status: 'queued' }
    })
    
    cy.get('[data-testid="start-analysis-button"]').click()
    
    // Should show success message
    cy.get('[data-testid="success-message"]').should('contain.text', 'Analysis started')
  })

  it('should handle empty state when no analyses exist', () => {
    // Mock empty analyses response
    cy.mockApiResponse('GET', '/api/analyses', {
      statusCode: 200,
      body: []
    })
    
    cy.visit('/dashboard')
    
    cy.get('[data-testid="empty-state"]').should('be.visible')
    cy.get('[data-testid="empty-state-message"]').should('contain.text', 'No analyses found')
    cy.get('[data-testid="create-first-analysis-button"]').should('be.visible')
  })

  it('should handle API errors gracefully', () => {
    // Mock API error
    cy.mockApiResponse('GET', '/api/analyses', {
      statusCode: 500,
      body: { error: 'Internal server error' }
    })
    
    cy.visit('/dashboard')
    
    cy.get('[data-testid="error-state"]').should('be.visible')
    cy.get('[data-testid="error-message"]').should('contain.text', 'Failed to load analyses')
    cy.get('[data-testid="retry-button"]').should('be.visible')
  })

  it('should show loading states', () => {
    // Add delay to API response
    cy.mockApiResponse('GET', '/api/analyses', {
      statusCode: 200,
      body: [],
      delay: 2000
    })
    
    cy.visit('/dashboard')
    
    cy.get('[data-testid="loading-spinner"]').should('be.visible')
    cy.get('[data-testid="analyses-section"]').should('not.exist')
    
    // Wait for loading to complete
    cy.get('[data-testid="loading-spinner"]', { timeout: 3000 }).should('not.exist')
  })

  it('should be responsive on mobile', () => {
    cy.viewport(375, 667)
    cy.visit('/dashboard')
    
    // Check mobile layout
    cy.get('[data-testid="mobile-dashboard"]').should('be.visible')
    cy.get('[data-testid="analysis-card"]').should('have.css', 'flex-direction', 'column')
  })

  it('should support keyboard navigation', () => {
    cy.visit('/dashboard')
    
    // Tab through interactive elements
    cy.get('body').type('{Tab}')
    cy.focused().should('have.attr', 'data-testid', 'search-input')
    
    cy.focused().type('{Tab}')
    cy.focused().should('have.attr', 'data-testid', 'new-analysis-button')
  })
})
