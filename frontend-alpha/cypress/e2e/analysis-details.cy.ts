describe('Analysis Details and Reports', () => {
  beforeEach(() => {
    // Login before each test
    cy.fixture('users').then((users) => {
      cy.login(users.testUser.email, users.testUser.password)
    })
    
    // Mock single analysis API
    cy.fixture('analyses').then((analysesData) => {
      cy.mockApiResponse('GET', '/api/analyses/1', {
        statusCode: 200,
        body: analysesData.singleAnalysis
      })
    })
  })

  it('should display analysis overview correctly', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="analysis-details"]').should('be.visible')
    cy.get('[data-testid="project-name"]').should('contain.text', 'Sample Project')
    cy.get('[data-testid="analysis-status"]').should('contain.text', 'completed')
    cy.get('[data-testid="created-date"]').should('be.visible')
  })

  it('should display analysis summary metrics', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="summary-section"]').should('be.visible')
    cy.get('[data-testid="total-issues"]').should('contain.text', '27')
    cy.get('[data-testid="critical-issues"]').should('contain.text', '2')
    cy.get('[data-testid="lines-of-code"]').should('contain.text', '15420')
    cy.get('[data-testid="technical-debt"]').should('contain.text', 'High')
  })

  it('should display issues breakdown chart', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="issues-chart"]').should('be.visible')
    cy.get('[data-testid="chart-legend"]').should('be.visible')
    
    // Check chart data
    cy.get('[data-testid="critical-bar"]').should('have.length', 1)
    cy.get('[data-testid="high-bar"]').should('have.length', 1)
    cy.get('[data-testid="medium-bar"]').should('have.length', 1)
    cy.get('[data-testid="low-bar"]').should('have.length', 1)
  })

  it('should display anti-patterns list', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="anti-patterns-section"]').should('be.visible')
    cy.get('[data-testid="anti-pattern-item"]').should('have.length.at.least', 2)
    
    // Check first anti-pattern
    cy.get('[data-testid="anti-pattern-item"]').first().within(() => {
      cy.get('[data-testid="pattern-type"]').should('contain.text', 'god_object')
      cy.get('[data-testid="severity-badge"]').should('contain.text', 'high')
      cy.get('[data-testid="file-path"]').should('contain.text', 'src/main.rs')
      cy.get('[data-testid="line-number"]').should('contain.text', '45')
    })
  })

  it('should expand anti-pattern details', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="anti-pattern-item"]').first().click()
    cy.get('[data-testid="pattern-details"]').should('be.visible')
    cy.get('[data-testid="pattern-description"]').should('be.visible')
    cy.get('[data-testid="code-snippet"]').should('be.visible')
    cy.get('[data-testid="ai-suggestions"]').should('be.visible')
  })

  it('should display analysis report with markdown', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="report-tab"]').click()
    cy.get('[data-testid="analysis-report"]').should('be.visible')
    
    // Check markdown rendering
    cy.get('[data-testid="analysis-report"] h1').should('contain.text', 'Analysis Report')
    cy.get('[data-testid="analysis-report"] h2').should('contain.text', 'Summary')
    cy.get('[data-testid="analysis-report"] h3').should('contain.text', 'Critical Issues')
  })

  it('should display code syntax highlighting in snippets', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="anti-pattern-item"]').first().click()
    cy.get('[data-testid="code-snippet"]').should('be.visible')
    
    // Check syntax highlighting classes
    cy.get('[data-testid="code-snippet"] .hljs-keyword').should('exist')
    cy.get('[data-testid="code-snippet"] .hljs-string').should('exist')
  })

  it('should allow filtering issues by severity', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="severity-filter"]').select('critical')
    cy.get('[data-testid="anti-pattern-item"]').should('have.length', 1)
    cy.get('[data-testid="severity-badge"]').should('contain.text', 'critical')
  })

  it('should allow filtering issues by type', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="type-filter"]').select('god_object')
    cy.get('[data-testid="anti-pattern-item"]').should('have.length', 1)
    cy.get('[data-testid="pattern-type"]').should('contain.text', 'god_object')
  })

  it('should export analysis report', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="export-button"]').click()
    cy.get('[data-testid="export-options"]').should('be.visible')
    
    // Test PDF export
    cy.get('[data-testid="export-pdf"]').click()
    cy.get('[data-testid="export-success"]').should('contain.text', 'Report exported successfully')
    
    // Test JSON export
    cy.get('[data-testid="export-button"]').click()
    cy.get('[data-testid="export-json"]').click()
    cy.get('[data-testid="export-success"]').should('contain.text', 'Data exported successfully')
  })

  it('should share analysis results', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="share-button"]').click()
    cy.get('[data-testid="share-modal"]').should('be.visible')
    
    // Test copy link
    cy.get('[data-testid="copy-link-button"]').click()
    cy.get('[data-testid="copy-success"]').should('contain.text', 'Link copied to clipboard')
    
    // Test email sharing
    cy.get('[data-testid="email-share"]').type('colleague@example.com')
    cy.get('[data-testid="send-email-button"]').click()
    cy.get('[data-testid="email-success"]').should('contain.text', 'Email sent successfully')
  })

  it('should navigate back to dashboard', () => {
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="back-to-dashboard"]').click()
    cy.url().should('include', '/dashboard')
  })

  it('should handle analysis not found', () => {
    cy.mockApiResponse('GET', '/api/analyses/999', {
      statusCode: 404,
      body: { error: 'Analysis not found' }
    })
    
    cy.visit('/analysis/999', { failOnStatusCode: false })
    
    cy.get('[data-testid="not-found-message"]').should('contain.text', 'Analysis not found')
    cy.get('[data-testid="back-to-dashboard"]').should('be.visible')
  })

  it('should handle loading states', () => {
    cy.mockApiResponse('GET', '/api/analyses/1', {
      statusCode: 200,
      body: {},
      delay: 2000
    })
    
    cy.visit('/analysis/1')
    
    cy.get('[data-testid="loading-spinner"]').should('be.visible')
    cy.get('[data-testid="analysis-details"]').should('not.exist')
    
    // Wait for loading to complete
    cy.get('[data-testid="loading-spinner"]', { timeout: 3000 }).should('not.exist')
  })

  it('should be responsive on mobile', () => {
    cy.viewport(375, 667)
    cy.visit('/analysis/1')
    
    // Check mobile layout
    cy.get('[data-testid="mobile-analysis-view"]').should('be.visible')
    cy.get('[data-testid="summary-section"]').should('have.css', 'flex-direction', 'column')
  })

  it('should support accessibility features', () => {
    cy.visit('/analysis/1')
    
    // Check ARIA labels and roles
    cy.get('[data-testid="analysis-details"]').should('have.attr', 'role', 'main')
    cy.get('[data-testid="anti-patterns-section"]').should('have.attr', 'aria-label')
    cy.get('[data-testid="severity-filter"]').should('have.attr', 'aria-label')
    
    // Check keyboard navigation
    cy.get('[data-testid="anti-pattern-item"]').first().focus()
    cy.focused().type('{enter}')
    cy.get('[data-testid="pattern-details"]').should('be.visible')
  })
})
