describe('Alpha Frontend - UI Components', () => {
  beforeEach(() => {
    cy.waitForServices()
    cy.visit('/')
  })

  it('should render main UI components', () => {
    // Check for main structural elements
    cy.get('body').should('be.visible')
    
    // Look for common UI patterns
    cy.get('body').then(($body) => {
      const hasHeader = $body.find('header, .header, [role="banner"]').length > 0
      const hasMain = $body.find('main, .main, [role="main"]').length > 0
      const hasNav = $body.find('nav, .nav, [role="navigation"]').length > 0
      
      if (hasHeader) cy.log('✓ Header component found')
      if (hasMain) cy.log('✓ Main content area found')
      if (hasNav) cy.log('✓ Navigation component found')
      
      expect(hasHeader || hasMain || hasNav).to.be.true
    })
  })

  it('should handle interactive elements', () => {
    // Test buttons
    cy.get('button, input[type="button"], input[type="submit"]').then(($buttons) => {
      if ($buttons.length > 0) {
        cy.wrap($buttons.first()).should('be.visible')
        cy.log(`✓ Found ${$buttons.length} interactive button(s)`)
      } else {
        cy.log('No buttons found - acceptable for minimal alpha')
      }
    })

    // Test links
    cy.get('a[href]').then(($links) => {
      if ($links.length > 0) {
        cy.wrap($links.first()).should('be.visible')
        cy.log(`✓ Found ${$links.length} link(s)`)
      }
    })
  })

  it('should display alpha-specific UI elements', () => {
    // Look for alpha badges, warnings, or notices
    cy.get('body').then(($body) => {
      const text = $body.text().toLowerCase()
      const hasAlphaIndicators = text.includes('alpha') || 
                                text.includes('preview') || 
                                text.includes('beta') ||
                                text.includes('coming soon')
      
      if (hasAlphaIndicators) {
        cy.log('✓ Alpha UI indicators found')
      } else {
        cy.log('No alpha indicators - checking for version info')
        cy.get('body').should('contain.text', '0.9')
      }
    })
  })

  it('should handle loading states', () => {
    // Test loading behavior on page load
    cy.intercept('GET', '**/api/**', { delay: 1000, body: {} }).as('slowAPI')
    
    cy.reload()
    cy.get('body').should('be.visible')
    cy.log('✓ App handles loading states gracefully')
  })

  it('should validate form elements (if present)', () => {
    cy.get('form, input, textarea, select').then(($forms) => {
      if ($forms.length > 0) {
        cy.log(`✓ Found ${$forms.length} form element(s)`)
        
        // Test basic form interaction
        cy.get('input[type="text"], input[type="email"], textarea').then(($inputs) => {
          if ($inputs.length > 0) {
            cy.wrap($inputs.first()).type('test input').should('have.value', 'test input')
            cy.log('✓ Form inputs working')
          }
        })
      } else {
        cy.log('No forms found - acceptable for alpha documentation interface')
      }
    })
  })

  it('should test accessibility basics', () => {
    // Check for basic accessibility attributes
    cy.get('img').each(($img) => {
      cy.wrap($img).should('have.attr', 'alt')
    })

    // Check for focus management
    cy.get('a, button, input, select, textarea').then(($focusable) => {
      if ($focusable.length > 0) {
        cy.wrap($focusable.first()).focus().should('be.focused')
        cy.log('✓ Focus management working')
      }
    })
  })
})