// ***********************************************************
// This example support/e2e.ts is processed and
// loaded automatically before your test files.
//
// This is a great place to put global configuration and
// behavior that modifies Cypress.
//
// You can change the location of this file or turn off
// automatically serving support files with the
// 'supportFile' configuration option.
//
// You can read more here:
// https://on.cypress.io/configuration
// ***********************************************************

// Import commands.js using ES2015 syntax:
import './commands';

// Alternatively you can use CommonJS syntax:
// require('./commands')

import '@testing-library/cypress/add-commands';
import 'cypress-real-events';

// Hide fetch/XHR requests from Cypress command log
const app = window.top;
if (app && !app.document.head.querySelector('[data-hide-command-log-request]')) {
  const style = app.document.createElement('style');
  style.innerHTML = '.command-name-request, .command-name-xhr { display: none }';
  style.setAttribute('data-hide-command-log-request', '');
  app.document.head.appendChild(style);
}

// Global before hook for all tests
beforeEach(() => {
  // Clear localStorage and sessionStorage before each test
  cy.clearLocalStorage()
  cy.clearCookies()
  
  // Mock window.matchMedia for responsive tests
  cy.window().then((win) => {
    win.matchMedia = win.matchMedia || function() {
      return {
        matches: false,
        addListener: function() {},
        removeListener: function() {}
      }
    }
  })
})
