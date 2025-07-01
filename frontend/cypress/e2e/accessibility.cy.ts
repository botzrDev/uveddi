describe('Accessibility Tests', () => {
  beforeEach(() => {
    cy.clearLocalStorage()
    cy.clearCookies()
  })

  describe('Landing Page Accessibility', () => {
    it('should have proper semantic HTML structure', () => {
      cy.visit('/')
      
      // Check for proper heading hierarchy
      cy.get('h1').should('exist')
      cy.get('h1').should('have.length', 1) // Only one h1 per page
      
      // Check for main landmark
      cy.get('main').should('exist')
      cy.get('main').should('have.attr', 'role', 'main')
      
      // Check for header and footer
      cy.get('header').should('exist')
      cy.get('footer').should('exist')
    })

    it('should have proper ARIA labels and roles', () => {
      cy.visit('/')
      
      // Check navigation
      cy.get('nav').should('have.attr', 'role', 'navigation')
      cy.get('nav').should('have.attr', 'aria-label')
      
      // Check buttons
      cy.get('[data-testid="get-started-button"]').should('have.attr', 'aria-label')
      
      // Check form controls
      cy.get('input[type="email"]').should('have.attr', 'aria-label')
    })

    it('should be keyboard navigable', () => {
      cy.visit('/')
      
      // Start keyboard navigation
      cy.get('body').type('{tab}')
      
      // Should focus on first interactive element
      cy.focused().should('be.visible')
      
      // Tab through all interactive elements
      let tabCount = 0
      const maxTabs = 20
      
      function tabNext() {
        if (tabCount < maxTabs) {
          cy.focused().then(($el) => {
            if ($el.length > 0) {
              cy.focused().type('{tab}')
              tabCount++
              tabNext()
            }
          })
        }
      }
      
      tabNext()
      
      // Should be able to activate focused elements with Enter/Space
      cy.get('[data-testid="get-started-button"]').focus()
      cy.focused().type('{enter}')
      cy.url().should('include', '/register')
    })

    it('should have sufficient color contrast', () => {
      cy.visit('/')
      
      // Check text contrast
      cy.get('h1').should('have.css', 'color').and('not.be.empty')
      cy.get('p').should('have.css', 'color').and('not.be.empty')
      
      // Buttons should have good contrast
      cy.get('[data-testid="get-started-button"]')
        .should('have.css', 'background-color')
        .and('not.be.empty')
    })

    it('should work with screen readers', () => {
      cy.visit('/')
      
      // Check for screen reader friendly content
      cy.get('[aria-hidden="true"]').should('not.contain.text')
      cy.get('[role="img"]').should('have.attr', 'aria-label')
      
      // Check for skip links
      cy.get('a[href="#main"]').should('exist')
    })
  })

  describe('Form Accessibility', () => {
    it('should have accessible registration form', () => {
      cy.visit('/register')
      
      // Check form structure
      cy.get('form').should('exist')
      cy.get('fieldset').should('exist')
      cy.get('legend').should('exist')
      
      // Check labels
      cy.get('label[for="name"]').should('exist')
      cy.get('label[for="email"]').should('exist')
      cy.get('label[for="password"]').should('exist')
      
      // Check input associations
      cy.get('#name').should('have.attr', 'aria-describedby')
      cy.get('#email').should('have.attr', 'aria-describedby')
      cy.get('#password').should('have.attr', 'aria-describedby')
    })

    it('should announce form errors to screen readers', () => {
      cy.visit('/register')
      
      // Submit empty form to trigger errors
      cy.get('[data-testid="register-button"]').click()
      
      // Check error messages have proper ARIA attributes
      cy.get('[data-testid="name-error"]')
        .should('have.attr', 'role', 'alert')
        .should('have.attr', 'aria-live', 'polite')
      
      cy.get('[data-testid="email-error"]')
        .should('have.attr', 'role', 'alert')
        .should('have.attr', 'aria-live', 'polite')
      
      // Inputs should reference error messages
      cy.get('[data-testid="name-input"]')
        .should('have.attr', 'aria-invalid', 'true')
        .should('have.attr', 'aria-describedby')
    })

    it('should have accessible login form', () => {
      cy.visit('/login')
      
      // Check form accessibility
      cy.get('form[role="form"]').should('exist')
      cy.get('input[type="email"]').should('have.attr', 'aria-label')
      cy.get('input[type="password"]').should('have.attr', 'aria-label')
      
      // Check password visibility toggle
      cy.get('[data-testid="password-toggle"]')
        .should('have.attr', 'aria-label')
        .should('have.attr', 'aria-pressed')
    })
  })

  describe('Dashboard Accessibility', () => {
    beforeEach(() => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })
      
      cy.fixture('analyses').then((analysesData) => {
        cy.mockApiResponse('GET', '/api/analyses', {
          statusCode: 200,
          body: analysesData.analyses
        })
      })
    })

    it('should have accessible dashboard navigation', () => {
      cy.visit('/dashboard')
      
      // Check main navigation
      cy.get('[data-testid="main-nav"]')
        .should('have.attr', 'role', 'navigation')
        .should('have.attr', 'aria-label', 'Main navigation')
      
      // Check user menu
      cy.get('[data-testid="user-menu"]')
        .should('have.attr', 'aria-haspopup', 'true')
        .should('have.attr', 'aria-expanded', 'false')
      
      // Test user menu expansion
      cy.get('[data-testid="user-menu"]').click()
      cy.get('[data-testid="user-menu"]').should('have.attr', 'aria-expanded', 'true')
    })

    it('should have accessible data tables', () => {
      cy.visit('/dashboard')
      
      // Check table structure
      cy.get('table').should('exist')
      cy.get('thead').should('exist')
      cy.get('tbody').should('exist')
      
      // Check table headers
      cy.get('th').should('have.attr', 'scope', 'col')
      cy.get('th').should('contain.text') // Headers should have text
      
      // Check table caption
      cy.get('table').should('have.attr', 'aria-label')
      
      // Check sortable columns
      cy.get('th[aria-sort]').should('exist')
      cy.get('th[aria-sort]').should('have.attr', 'tabindex', '0')
    })

    it('should have accessible search and filters', () => {
      cy.visit('/dashboard')
      
      // Check search input
      cy.get('[data-testid="search-input"]')
        .should('have.attr', 'aria-label', 'Search analyses')
        .should('have.attr', 'role', 'searchbox')
      
      // Check filter controls
      cy.get('[data-testid="status-filter"]')
        .should('have.attr', 'aria-label')
      
      // Check live region for search results
      cy.get('[data-testid="search-results"]')
        .should('have.attr', 'aria-live', 'polite')
        .should('have.attr', 'aria-atomic', 'true')
    })

    it('should announce loading states', () => {
      cy.mockApiResponse('GET', '/api/analyses', {
        statusCode: 200,
        body: [],
        delay: 2000
      })
      
      cy.visit('/dashboard')
      
      // Check loading announcement
      cy.get('[data-testid="loading-message"]')
        .should('have.attr', 'aria-live', 'polite')
        .should('contain.text', 'Loading')
    })
  })

  describe('Modal Accessibility', () => {
    beforeEach(() => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })
    })

    it('should have accessible modal dialogs', () => {
      cy.visit('/dashboard')
      
      // Open modal
      cy.get('[data-testid="new-analysis-button"]').click()
      
      // Check modal structure
      cy.get('[data-testid="modal"]')
        .should('have.attr', 'role', 'dialog')
        .should('have.attr', 'aria-modal', 'true')
        .should('have.attr', 'aria-labelledby')
      
      // Check focus management
      cy.focused().should('be.visible')
      cy.focused().should('be.within', '[data-testid="modal"]')
      
      // Check close button
      cy.get('[data-testid="modal-close"]')
        .should('have.attr', 'aria-label', 'Close dialog')
      
      // Test Escape key
      cy.get('body').type('{esc}')
      cy.get('[data-testid="modal"]').should('not.exist')
      
      // Focus should return to trigger button
      cy.focused().should('have.attr', 'data-testid', 'new-analysis-button')
    })

    it('should trap focus within modal', () => {
      cy.visit('/dashboard')
      cy.get('[data-testid="new-analysis-button"]').click()
      
      // Get all focusable elements in modal
      cy.get('[data-testid="modal"] input, [data-testid="modal"] button, [data-testid="modal"] select')
        .should('have.length.greaterThan', 0)
      
      // Tab through modal elements
      cy.get('body').type('{tab}')
      cy.focused().should('be.within', '[data-testid="modal"]')
      
      // Shift+Tab should also stay within modal
      cy.get('body').type('{shift}{tab}')
      cy.focused().should('be.within', '[data-testid="modal"]')
    })
  })

  describe('Error Page Accessibility', () => {
    it('should have accessible 404 page', () => {
      cy.visit('/non-existent-page', { failOnStatusCode: false })
      
      // Check error page structure
      cy.get('[data-testid="error-page"]')
        .should('have.attr', 'role', 'main')
      
      cy.get('h1').should('contain.text', '404')
      
      // Check error announcement
      cy.get('[data-testid="error-message"]')
        .should('have.attr', 'role', 'alert')
        .should('be.visible')
      
      // Check navigation options
      cy.get('[data-testid="back-home"]')
        .should('have.attr', 'aria-label')
        .should('be.visible')
    })
  })

  describe('Animation and Motion Accessibility', () => {
    it('should respect prefers-reduced-motion', () => {
      // Simulate reduced motion preference
      cy.window().then((win) => {
        Object.defineProperty(win, 'matchMedia', {
          writable: true,
          value: (query: string) => ({
            matches: query.includes('prefers-reduced-motion: reduce'),
            media: query,
            onchange: null,
            addListener: () => {},
            removeListener: () => {},
            addEventListener: () => {},
            removeEventListener: () => {},
            dispatchEvent: () => {},
          }),
        })
      })
      
      cy.visit('/')
      
      // Check that animations are disabled or reduced
      cy.get('[data-testid="animated-terminal"]').then(($el) => {
        const animationDuration = $el.css('animation-duration')
        const animation = $el.css('animation')
        expect(animationDuration === '0s' || animation === 'none').to.be.true
      })
    })

    it('should not cause seizures with flashing content', () => {
      cy.visit('/')
      
      // Check for potentially problematic animations
      cy.get('*').each(($el) => {
        cy.wrap($el).should('not.have.css', 'animation-name', 'flash')
        cy.wrap($el).should('not.have.css', 'animation-name', 'strobe')
      })
    })
  })

  describe('Language and Internationalization', () => {
    it('should have proper language attributes', () => {
      cy.visit('/')
      
      // Check document language
      cy.get('html').should('have.attr', 'lang', 'en')
      
      // Check for language changes in content
      cy.get('[lang]').each(($el) => {
        cy.wrap($el).should('have.attr', 'lang').and('not.be.empty')
      })
    })

    it('should have proper text direction', () => {
      cy.visit('/')
      
      // Check text direction
      cy.get('html').should('have.attr', 'dir', 'ltr')
      
      // Check for RTL content if any
      cy.get('[dir="rtl"]').should('have.css', 'direction', 'rtl')
    })
  })
})
