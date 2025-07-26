describe('Authentication Flow', () => {
  beforeEach(() => {
    // Clean up any existing user sessions
    cy.clearLocalStorage()
    cy.clearCookies()
  })

  describe('User Registration', () => {
    beforeEach(() => {
      cy.visit('/register')
    })

    it('should display registration form correctly', () => {
      cy.get('[data-testid="register-form"]').should('be.visible')
      cy.get('[data-testid="name-input"]').should('be.visible')
      cy.get('[data-testid="email-input"]').should('be.visible') 
      cy.get('[data-testid="password-input"]').should('be.visible')
      cy.get('[data-testid="confirm-password-input"]').should('be.visible')
      cy.get('[data-testid="register-button"]').should('be.visible')
    })

    it('should show validation errors for empty fields', () => {
      cy.get('[data-testid="register-button"]').click()
      
      cy.get('[data-testid="name-error"]').should('be.visible')
      cy.get('[data-testid="email-error"]').should('be.visible')
      cy.get('[data-testid="password-error"]').should('be.visible')
    })

    it('should show validation error for invalid email', () => {
      cy.get('[data-testid="email-input"]').type('invalid-email')
      cy.get('[data-testid="register-button"]').click()
      
      cy.get('[data-testid="email-error"]').should('contain.text', 'valid email')
    })

    it('should show validation error for mismatched passwords', () => {
      cy.get('[data-testid="password-input"]').type('password123')
      cy.get('[data-testid="confirm-password-input"]').type('different123')
      cy.get('[data-testid="register-button"]').click()
      
      cy.get('[data-testid="password-match-error"]').should('contain.text', 'match')
    })

    it('should register user successfully with valid data', () => {
      cy.fixture('users').then((users) => {
        const testUser = users.testUser
        
        // Mock successful registration API
        cy.mockApiResponse('POST', '/api/auth/register', { 
          statusCode: 201,
          body: { 
            success: true, 
            user: { id: 1, email: testUser.email, name: testUser.name },
            token: 'mock-jwt-token'
          }
        })
        
        cy.get('[data-testid="name-input"]').type(testUser.name)
        cy.get('[data-testid="email-input"]').type(testUser.email)
        cy.get('[data-testid="password-input"]').type(testUser.password)
        cy.get('[data-testid="confirm-password-input"]').type(testUser.password)
        
        cy.get('[data-testid="register-button"]').click()
        
        // Should redirect to dashboard
        cy.url().should('include', '/dashboard')
        cy.get('[data-testid="dashboard"]').should('be.visible')
      })
    })

    it('should handle registration API errors', () => {
      cy.fixture('users').then((users) => {
        const testUser = users.testUser
        
        // Mock failed registration API
        cy.mockApiResponse('POST', '/api/auth/register', {
          statusCode: 400,
          body: { error: 'Email already exists' }
        })
        
        cy.get('[data-testid="name-input"]').type(testUser.name)
        cy.get('[data-testid="email-input"]').type(testUser.email)
        cy.get('[data-testid="password-input"]').type(testUser.password)
        cy.get('[data-testid="confirm-password-input"]').type(testUser.password)
        
        cy.get('[data-testid="register-button"]').click()
        
        cy.get('[data-testid="error-message"]').should('contain.text', 'Email already exists')
      })
    })

    it('should have link to login page', () => {
      cy.get('[data-testid="login-link"]').click()
      cy.url().should('include', '/login')
    })
  })

  describe('User Login', () => {
    beforeEach(() => {
      cy.visit('/login')
    })

    it('should display login form correctly', () => {
      cy.get('[data-testid="login-form"]').should('be.visible')
      cy.get('[data-testid="email-input"]').should('be.visible')
      cy.get('[data-testid="password-input"]').should('be.visible')
      cy.get('[data-testid="login-button"]').should('be.visible')
    })

    it('should show validation errors for empty fields', () => {
      cy.get('[data-testid="login-button"]').click()
      
      cy.get('[data-testid="email-error"]').should('be.visible')
      cy.get('[data-testid="password-error"]').should('be.visible')
    })

    it('should login user successfully with valid credentials', () => {
      cy.fixture('users').then((users) => {
        const testUser = users.testUser
        
        // Mock successful login API
        cy.mockApiResponse('POST', '/api/auth/login', {
          statusCode: 200,
          body: {
            success: true,
            user: { id: 1, email: testUser.email, name: testUser.name },
            token: 'mock-jwt-token'
          }
        })
        
        cy.get('[data-testid="email-input"]').type(testUser.email)
        cy.get('[data-testid="password-input"]').type(testUser.password)
        cy.get('[data-testid="login-button"]').click()
        
        // Should redirect to dashboard
        cy.url().should('include', '/dashboard')
        cy.get('[data-testid="dashboard"]').should('be.visible')
      })
    })

    it('should handle login with invalid credentials', () => {
      cy.fixture('users').then((users) => {
        const invalidUser = users.invalidUser
        
        // Mock failed login API
        cy.mockApiResponse('POST', '/api/auth/login', {
          statusCode: 401,
          body: { error: 'Invalid credentials' }
        })
        
        cy.get('[data-testid="email-input"]').type(invalidUser.email)
        cy.get('[data-testid="password-input"]').type(invalidUser.password)
        cy.get('[data-testid="login-button"]').click()
        
        cy.get('[data-testid="error-message"]').should('contain.text', 'Invalid credentials')
      })
    })

    it('should have link to registration page', () => {
      cy.get('[data-testid="register-link"]').click()
      cy.url().should('include', '/register')
    })

    it('should have forgot password functionality', () => {
      cy.get('[data-testid="forgot-password-link"]').should('be.visible').click()
      // This would navigate to forgot password page when implemented
    })
  })

  describe('User Logout', () => {
    beforeEach(() => {
      // Login first
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })
    })

    it('should logout user successfully', () => {
      cy.get('[data-testid="user-menu"]').click()
      cy.get('[data-testid="logout-button"]').click()
      
      // Should redirect to login page
      cy.url().should('include', '/login')
    })
  })

  describe('Protected Routes', () => {
    it('should redirect unauthenticated users to login', () => {
      cy.visit('/dashboard')
      cy.url().should('include', '/login')
    })

    it('should allow authenticated users to access dashboard', () => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
        cy.visit('/dashboard')
        cy.url().should('include', '/dashboard')
        cy.get('[data-testid="dashboard"]').should('be.visible')
      })
    })
  })
})
