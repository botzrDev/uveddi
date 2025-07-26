// ***********************************************
// This example commands.js shows you how to create various custom commands and overwrite existing commands.
//
// For more comprehensive examples of custom commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************

// Custom commands for Uveddi Alpha testing

Cypress.Commands.add('loginAsAlphaTester', () => {
  // Mock alpha tester login - no real auth in alpha
  cy.window().then((win) => {
    win.localStorage.setItem('alphaUser', JSON.stringify({
      name: 'Alpha Tester',
      role: 'tester',
      session: Date.now()
    }))
  })
})

Cypress.Commands.add('navigateToSection', (section) => {
  cy.get(`[data-testid="nav-${section}"]`).click()
  cy.url().should('include', section)
})

Cypress.Commands.add('validateDocumentationLoad', () => {
  cy.intercept('GET', '**/api/docs/**').as('docsRequest')
  cy.wait('@docsRequest', { timeout: 10000 })
  cy.get('[data-testid="documentation-content"]').should('be.visible')
})

Cypress.Commands.add('checkResponsiveDesign', () => {
  const viewports = [
    { width: 375, height: 667 },   // Mobile
    { width: 768, height: 1024 },  // Tablet
    { width: 1280, height: 720 },  // Desktop
    { width: 1920, height: 1080 }  // Large Desktop
  ]
  
  viewports.forEach(viewport => {
    cy.viewport(viewport.width, viewport.height)
    cy.get('body').should('be.visible')
    cy.wait(500) // Allow layout to settle
  })
})

Cypress.Commands.add('validatePerformance', () => {
  cy.window().then((win) => {
    // Check if performance API is available
    if (win.performance && win.performance.getEntriesByType) {
      const navigationEntries = win.performance.getEntriesByType('navigation')
      if (navigationEntries.length > 0) {
        const loadTime = navigationEntries[0].loadEventEnd - navigationEntries[0].loadEventStart
        expect(loadTime).to.be.lessThan(5000) // Page should load in under 5 seconds
      }
    }
  })
})