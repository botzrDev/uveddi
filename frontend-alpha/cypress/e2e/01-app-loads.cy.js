describe('Alpha Frontend - Application Loading', () => {
  beforeEach(() => {
    cy.waitForServices()
    cy.visit('/')
  })

  it('should load the main application', () => {
    cy.get('body').should('be.visible')
    cy.title().should('contain', 'Uveddi Alpha')
  })

  it('should display alpha branding', () => {
    cy.validateAlphaFeature('Main App')
    cy.get('body').should('contain.text', 'Alpha')
  })

  it('should have working navigation', () => {
    // Test navigation elements are present
    cy.get('nav').should('be.visible')
  })

  it('should load within performance threshold', () => {
    cy.validatePerformance()
  })

  it('should be responsive across viewports', () => {
    cy.checkResponsiveDesign()
  })

  it('should handle network errors gracefully', () => {
    cy.intercept('GET', '**/api/**', { forceNetworkError: true }).as('networkError')
    cy.reload()
    cy.get('body').should('be.visible') // App should still render even with API errors
  })
})