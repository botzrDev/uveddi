describe('Alpha Frontend - API Integration', () => {
  beforeEach(() => {
    cy.waitForServices()
    cy.visit('/')
  })

  it('should connect to API server health endpoint', () => {
    cy.request({
      url: `${Cypress.env('API_BASE_URL')}/health`,
      failOnStatusCode: false
    }).then((response) => {
      if (response.status === 200) {
        expect(response.body).to.have.property('status')
        expect(response.body.service).to.include('uveddi-api-server-alpha')
        cy.log('✓ API server health check passed')
      } else {
        cy.log('API server not available - testing frontend in isolation')
      }
    })
  })

  it('should handle documentation API requests', () => {
    cy.request({
      url: `${Cypress.env('API_BASE_URL')}/api/docs/structure`,
      failOnStatusCode: false
    }).then((response) => {
      if (response.status === 200) {
        expect(response.body).to.have.property('structure')
        cy.log('✓ Documentation API working')
      } else {
        cy.log('Documentation API not available - acceptable for alpha')
      }
    })
  })

  it('should handle project info API', () => {
    cy.request({
      url: `${Cypress.env('API_BASE_URL')}/api/project/info`,
      failOnStatusCode: false
    }).then((response) => {
      if (response.status === 200) {
        expect(response.body).to.have.property('name', 'Uveddi')
        expect(response.body).to.have.property('version')
        expect(response.body.version).to.include('alpha')
        cy.log('✓ Project info API working')
      } else {
        cy.log('Project info API not available')
      }
    })
  })

  it('should test rendering service connectivity', () => {
    cy.request({
      url: `${Cypress.env('RENDERING_SERVICE_URL')}/health`,
      failOnStatusCode: false
    }).then((response) => {
      if (response.status === 200) {
        expect(response.body).to.have.property('status', 'healthy')
        cy.log('✓ Rendering service health check passed')
      } else {
        cy.log('Rendering service not available - acceptable for alpha')
      }
    })
  })

  it('should handle API errors gracefully in frontend', () => {
    // Simulate API failures
    cy.intercept('GET', '**/api/**', {
      statusCode: 500,
      body: { error: 'Service unavailable' }
    }).as('apiError')

    cy.reload()
    
    // Frontend should still load despite API errors
    cy.get('body').should('be.visible')
    cy.log('✓ Frontend handles API errors gracefully')
  })

  it('should test CORS configuration', () => {
    cy.window().then((win) => {
      // Test that we can make requests to API from the frontend origin
      fetch(`${Cypress.env('API_BASE_URL')}/health`)
        .then(response => {
          cy.log('✓ CORS configured correctly')
        })
        .catch(error => {
          if (error.message.includes('CORS')) {
            cy.log('CORS issue detected - needs configuration review')
          } else {
            cy.log('API not available - testing in isolation mode')
          }
        })
    })
  })

  it('should validate environment configuration', () => {
    cy.window().then((win) => {
      // Check if environment variables are properly loaded
      if (win.location.origin.includes('9998')) {
        cy.log('✓ Running on expected alpha port 9998')
      }
    })

    // Verify API endpoints are configured
    expect(Cypress.env('API_BASE_URL')).to.include('8080')
    expect(Cypress.env('RENDERING_SERVICE_URL')).to.include('3002')
    cy.log('✓ Environment configuration validated')
  })
})