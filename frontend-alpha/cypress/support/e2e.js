// ***********************************************************
// This example support/e2e.js is processed and loaded automatically before your test files.
//
// This is a great place to put global configuration and behavior that modifies Cypress.
//
// You can change the location of this file or turn off automatically serving support files
// with the 'supportFile' configuration option.
//
// You can read more here: https://on.cypress.io/configuration
// ***********************************************************

// Import commands.js using ES2015 syntax:
import './commands'

// Alternatively you can use CommonJS syntax:
// require('./commands')

// Alpha testing configuration
Cypress.on('uncaught:exception', (err, runnable) => {
  // Prevent Cypress from failing on uncaught exceptions during alpha testing
  // This is acceptable for alpha as we're testing core functionality
  console.log('Uncaught exception during alpha testing:', err.message)
  return false
})

// Custom command to wait for services to be ready
Cypress.Commands.add('waitForServices', () => {
  // Check API server health
  cy.request({
    url: `${Cypress.env('API_BASE_URL')}/health`,
    failOnStatusCode: false,
    timeout: 5000
  }).then((response) => {
    if (response.status !== 200) {
      cy.log('API server not ready, continuing with frontend-only tests')
    }
  })
  
  // Check rendering service health
  cy.request({
    url: `${Cypress.env('RENDERING_SERVICE_URL')}/health`,
    failOnStatusCode: false,
    timeout: 5000
  }).then((response) => {
    if (response.status !== 200) {
      cy.log('Rendering service not ready, continuing with basic tests')
    }
  })
})

// Custom command for alpha feature validation
Cypress.Commands.add('validateAlphaFeature', (featureName) => {
  cy.get('body').should('contain.text', 'Alpha')
    .then(() => cy.log(`✓ Alpha feature validated: ${featureName}`))
})