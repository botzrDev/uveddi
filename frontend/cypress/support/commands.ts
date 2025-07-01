// ***********************************************
// This example commands.ts shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************

/// <reference types="cypress" />

declare namespace Cypress {
  interface Chainable {
    /**
     * Custom command to login a user
     * @example cy.login('test@example.com', 'password123')
     */
    login(email: string, password: string): Chainable<Element>
    
    /**
     * Custom command to register a new user
     * @example cy.register('John Doe', 'test@example.com', 'password123')
     */
    register(name: string, email: string, password: string): Chainable<Element>
    
    /**
     * Custom command to logout current user
     * @example cy.logout()
     */
    logout(): Chainable<Element>
    
    /**
     * Custom command to wait for element to be visible and stable
     * @example cy.waitForElement('[data-testid="dashboard"]')
     */
    waitForElement(selector: string, timeout?: number): Chainable<JQuery<HTMLElement>>
    
    /**
     * Custom command to mock API responses
     * @example cy.mockApiResponse('GET', '/api/analyses', { fixture: 'analyses.json' })
     */
    mockApiResponse(method: string, url: string, response: any): Chainable<Element>
    
    /**
     * Custom command to seed test data
     * @example cy.seedDatabase()
     */
    seedDatabase(): Chainable<Element>
    
    /**
     * Custom command to clean test data
     * @example cy.cleanDatabase()
     */
    cleanDatabase(): Chainable<Element>
  }
}

// Login command
Cypress.Commands.add('login', (email: string, password: string) => {
  cy.visit('/login')
  cy.get('[data-testid="email-input"]').type(email)
  cy.get('[data-testid="password-input"]').type(password)
  cy.get('[data-testid="login-button"]').click()
  cy.url().should('include', '/dashboard')
  cy.get('[data-testid="dashboard"]').should('be.visible')
})

// Register command
Cypress.Commands.add('register', (name: string, email: string, password: string) => {
  cy.visit('/register')
  cy.get('[data-testid="name-input"]').type(name)
  cy.get('[data-testid="email-input"]').type(email)
  cy.get('[data-testid="password-input"]').type(password)
  cy.get('[data-testid="confirm-password-input"]').type(password)
  cy.get('[data-testid="register-button"]').click()
  cy.url().should('include', '/dashboard')
})

// Logout command
Cypress.Commands.add('logout', () => {
  cy.get('[data-testid="user-menu"]').click()
  cy.get('[data-testid="logout-button"]').click()
  cy.url().should('include', '/login')
})

// Wait for element command
Cypress.Commands.add('waitForElement', (selector: string, timeout = 10000) => {
  return cy.get(selector, { timeout })
    .should('be.visible')
    .should('not.be.disabled')
})

// Mock API response command
Cypress.Commands.add('mockApiResponse', (method: string, url: string, response: any) => {
  cy.intercept(method as any, url, response).as(`${method.toLowerCase()}${url.replace(/\//g, '_')}`)
})

// Seed database command
Cypress.Commands.add('seedDatabase', () => {
  cy.task('log', 'Seeding test database...')
  // Implementation depends on your backend setup
  cy.request('POST', `${Cypress.env('apiUrl')}/test/seed`)
})

// Clean database command
Cypress.Commands.add('cleanDatabase', () => {
  cy.task('log', 'Cleaning test database...')
  // Implementation depends on your backend setup
  cy.request('DELETE', `${Cypress.env('apiUrl')}/test/clean`)
})
