describe('Landing Page', () => {
  beforeEach(() => {
    cy.visit('/')
  })

  it('should display the main hero section', () => {
    cy.get('[data-testid="hero-section"]').should('be.visible')
    cy.get('h1').should('contain.text', 'AI-Powered Code Architecture Analysis')
    cy.get('[data-testid="hero-description"]').should('be.visible')
  })

  it('should display animated terminal', () => {
    cy.get('[data-testid="animated-terminal"]').should('be.visible')
    cy.get('[data-testid="terminal-content"]').should('be.visible')
  })

  it('should have working navigation links', () => {
    cy.get('[data-testid="get-started-button"]').should('be.visible').click()
    cy.url().should('include', '/register')
    
    cy.go('back')
    
    cy.get('[data-testid="login-link"]').should('be.visible').click()
    cy.url().should('include', '/login')
  })

  it('should display feature cards', () => {
    cy.get('[data-testid="features-section"]').should('be.visible')
    cy.get('[data-testid="feature-card"]').should('have.length.at.least', 3)
    
    // Check specific feature cards
    cy.get('[data-testid="feature-card"]').first().should('contain.text', 'AI-Powered Analysis')
    cy.get('[data-testid="feature-card"]').eq(1).should('contain.text', 'Multi-Language Support')
    cy.get('[data-testid="feature-card"]').eq(2).should('contain.text', 'Real-time Insights')
  })

  it('should display statistics section', () => {
    cy.get('[data-testid="stats-section"]').should('be.visible')
    cy.get('[data-testid="stats-card"]').should('have.length.at.least', 3)
  })

  it('should be responsive on mobile devices', () => {
    cy.viewport(375, 667) // iPhone SE dimensions
    
    cy.get('[data-testid="hero-section"]').should('be.visible')
    cy.get('[data-testid="mobile-menu-button"]').should('be.visible')
    
    // Test mobile navigation
    cy.get('[data-testid="mobile-menu-button"]').click()
    cy.get('[data-testid="mobile-menu"]').should('be.visible')
  })

  it('should have proper accessibility attributes', () => {
    cy.get('h1').should('have.attr', 'aria-label')
    cy.get('[data-testid="get-started-button"]').should('have.attr', 'aria-label')
    cy.get('main').should('have.attr', 'role', 'main')
  })

  it('should handle page load performance', () => {
    cy.visit('/', {
      onBeforeLoad: (win) => {
        win.performance.mark('page-start')
      },
      onLoad: (win) => {
        win.performance.mark('page-end')
        win.performance.measure('page-load', 'page-start', 'page-end')
        const measure = win.performance.getEntriesByName('page-load')[0]
        expect(measure.duration).to.be.lessThan(3000) // Page should load in under 3 seconds
      }
    })
  })
})
