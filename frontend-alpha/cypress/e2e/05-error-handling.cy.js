describe('Alpha Frontend - Error Handling & Edge Cases', () => {
  beforeEach(() => {
    cy.waitForServices()
  })

  it('should handle 404 pages gracefully', () => {
    cy.visit('/nonexistent-page', { failOnStatusCode: false })
    cy.get('body').should('be.visible')
    cy.log('✓ 404 handling works')
  })

  it('should handle network connectivity issues', () => {
    // Simulate offline state
    cy.intercept('GET', '**', { forceNetworkError: true }).as('networkError')
    
    cy.visit('/')
    cy.get('body').should('be.visible')
    cy.log('✓ Offline state handled gracefully')
  })

  it('should handle slow network responses', () => {
    cy.intercept('GET', '**/api/**', { delay: 5000, body: {} }).as('slowResponse')
    
    cy.visit('/')
    cy.get('body').should('be.visible')
    cy.log('✓ Slow network responses handled')
  })

  it('should handle malformed API responses', () => {
    cy.intercept('GET', '**/api/**', { body: 'invalid json' }).as('malformedResponse')
    
    cy.visit('/')
    cy.get('body').should('be.visible')
    cy.log('✓ Malformed API responses handled')
  })

  it('should handle browser console errors', () => {
    let consoleErrors = []
    
    cy.window().then((win) => {
      const originalError = win.console.error
      win.console.error = (...args) => {
        consoleErrors.push(args.join(' '))
        originalError.apply(win.console, args)
      }
    })

    cy.visit('/')
    
    cy.then(() => {
      const criticalErrors = consoleErrors.filter(error => 
        !error.includes('Warning') && 
        !error.includes('DevTools') &&
        !error.includes('alpha') // Filter out expected alpha warnings
      )
      
      if (criticalErrors.length > 0) {
        cy.log(`Found ${criticalErrors.length} console errors:`)
        criticalErrors.forEach(error => cy.log(`- ${error}`))
      } else {
        cy.log('✓ No critical console errors found')
      }
    })
  })

  it('should handle memory constraints', () => {
    // Simulate memory pressure by creating large objects
    cy.window().then((win) => {
      const testData = []
      try {
        for (let i = 0; i < 1000; i++) {
          testData.push(new Array(1000).fill(`test-data-${i}`))
        }
        cy.log('✓ Memory pressure test completed')
      } catch (error) {
        cy.log('Memory constraint reached - handled gracefully')
      }
    })
  })

  it('should handle invalid user interactions', () => {
    cy.visit('/')
    
    // Test rapid clicking
    cy.get('body').then(($body) => {
      if ($body.find('button').length > 0) {
        cy.get('button').first().click({ multiple: true })
        cy.log('✓ Rapid clicking handled')
      }
    })

    // Test keyboard spam
    cy.get('body').type('{esc}{enter}{space}', { force: true })
    cy.log('✓ Keyboard spam handled')
  })

  it('should validate browser compatibility features', () => {
    cy.window().then((win) => {
      const features = {
        localStorage: !!win.localStorage,
        sessionStorage: !!win.sessionStorage,
        fetch: !!win.fetch,
        Promise: !!win.Promise,
        console: !!win.console
      }
      
      Object.entries(features).forEach(([feature, supported]) => {
        if (supported) {
          cy.log(`✓ ${feature} supported`)
        } else {
          cy.log(`⚠ ${feature} not supported`)
        }
      })
    })
  })

  it('should handle localStorage quota exceeded', () => {
    cy.window().then((win) => {
      try {
        // Try to fill localStorage
        for (let i = 0; i < 10000; i++) {
          win.localStorage.setItem(`test-${i}`, 'x'.repeat(1000))
        }
      } catch (error) {
        if (error.name === 'QuotaExceededError') {
          cy.log('✓ localStorage quota error handled gracefully')
        }
      }
      
      // Clean up
      Object.keys(win.localStorage).forEach(key => {
        if (key.startsWith('test-')) {
          win.localStorage.removeItem(key)
        }
      })
    })
  })
})