// ***********************************************************
// This example support/component.js is processed and loaded automatically before your test files.
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

// Import global styles that component tests might need
import '../../src/index.css'

// Example component test helpers
import { mount } from 'cypress/react18'

// Extend Cypress namespace with mount command
declare global {
  namespace Cypress {
    interface Chainable {
      mount: typeof mount
    }
  }
}

Cypress.Commands.add('mount', mount)

// Configure component testing environment
Cypress.on('uncaught:exception', (err, runnable) => {
  // Prevent Cypress from failing on uncaught exceptions during component testing
  console.log('Component test uncaught exception:', err.message)
  return false
})