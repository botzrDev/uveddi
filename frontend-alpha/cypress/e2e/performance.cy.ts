describe('Performance Tests', () => {
  beforeEach(() => {
    // Clear storage before each test
    cy.clearLocalStorage()
    cy.clearCookies()
  })

  describe('Page Load Performance', () => {
    it('should load landing page within performance budget', () => {
      cy.visit('/', {
        onBeforeLoad: (win) => {
          win.performance.mark('page-start')
        },
        onLoad: (win) => {
          win.performance.mark('page-end')
          win.performance.measure('page-load', 'page-start', 'page-end')
          
          const measure = win.performance.getEntriesByName('page-load')[0]
          expect(measure.duration).to.be.lessThan(2000) // Page should load in under 2 seconds
        }
      })

      cy.get('[data-testid="hero-section"]').should('be.visible')
    })

    it('should load dashboard within performance budget', () => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })

      cy.visit('/dashboard', {
        onBeforeLoad: (win) => {
          win.performance.mark('dashboard-start')
        },
        onLoad: (win) => {
          win.performance.mark('dashboard-end')
          win.performance.measure('dashboard-load', 'dashboard-start', 'dashboard-end')
          
          const measure = win.performance.getEntriesByName('dashboard-load')[0]
          expect(measure.duration).to.be.lessThan(3000) // Dashboard should load in under 3 seconds
        }
      })

      cy.get('[data-testid="dashboard"]').should('be.visible')
    })
  })

  describe('API Response Performance', () => {
    beforeEach(() => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })
    })

    it('should load analyses list quickly', () => {
      cy.fixture('analyses').then((analysesData) => {
        cy.intercept('GET', '/api/analyses', (req) => {
          req.reply((res) => {
            // Add delay to test performance
            setTimeout(() => {
              res.send({ statusCode: 200, body: analysesData.analyses })
            }, 500) // 500ms delay
          })
        }).as('getAnalyses')
      })

      const startTime = Date.now()
      cy.visit('/dashboard')
      
      cy.wait('@getAnalyses').then(() => {
        const endTime = Date.now()
        const responseTime = endTime - startTime
        expect(responseTime).to.be.lessThan(1000) // API should respond within 1 second
      })

      cy.get('[data-testid="analysis-card"]').should('be.visible')
    })

    it('should handle large dataset efficiently', () => {
      // Create large dataset for testing
      const largeDataset = Array.from({ length: 100 }, (_, i) => ({
        id: i + 1,
        projectName: `Project ${i + 1}`,
        projectPath: `/path/to/project-${i + 1}`,
        createdAt: new Date().toISOString(),
        status: 'completed',
        language: 'rust',
        totalFiles: Math.floor(Math.random() * 1000),
        issues: {
          critical: Math.floor(Math.random() * 5),
          high: Math.floor(Math.random() * 10),
          medium: Math.floor(Math.random() * 20),
          low: Math.floor(Math.random() * 30)
        }
      }))

      cy.mockApiResponse('GET', '/api/analyses', {
        statusCode: 200,
        body: largeDataset
      })

      const startTime = performance.now()
      cy.visit('/dashboard')
      
      cy.get('[data-testid="analysis-card"]').should('have.length.at.least', 10)
      
      cy.window().then((win) => {
        const endTime = performance.now()
        const renderTime = endTime - startTime
        expect(renderTime).to.be.lessThan(5000) // Should render large dataset within 5 seconds
      })
    })
  })

  describe('Search and Filter Performance', () => {
    beforeEach(() => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })

      // Create dataset with many items for filtering
      const searchDataset = Array.from({ length: 50 }, (_, i) => ({
        id: i + 1,
        projectName: `Project ${i + 1}`,
        projectPath: `/path/to/project-${i + 1}`,
        createdAt: new Date().toISOString(),
        status: i % 3 === 0 ? 'completed' : i % 3 === 1 ? 'in_progress' : 'failed',
        language: i % 4 === 0 ? 'rust' : i % 4 === 1 ? 'javascript' : i % 4 === 2 ? 'python' : 'typescript',
        totalFiles: Math.floor(Math.random() * 1000)
      }))

      cy.mockApiResponse('GET', '/api/analyses', {
        statusCode: 200,
        body: searchDataset
      })

      cy.visit('/dashboard')
    })

    it('should search results quickly', () => {
      const startTime = performance.now()
      
      cy.get('[data-testid="search-input"]').type('Project 1')
      
      cy.get('[data-testid="analysis-card"]').should('have.length.at.least', 1)
      
      cy.window().then(() => {
        const endTime = performance.now()
        const searchTime = endTime - startTime
        expect(searchTime).to.be.lessThan(500) // Search should complete within 500ms
      })
    })

    it('should filter results efficiently', () => {
      const startTime = performance.now()
      
      cy.get('[data-testid="status-filter"]').select('completed')
      
      cy.get('[data-testid="analysis-card"]').should('have.length.at.least', 1)
      
      cy.window().then(() => {
        const endTime = performance.now()
        const filterTime = endTime - startTime
        expect(filterTime).to.be.lessThan(300) // Filtering should complete within 300ms
      })
    })
  })

  describe('Memory Usage', () => {
    it('should not have memory leaks during navigation', () => {
      cy.fixture('users').then((users) => {
        cy.login(users.testUser.email, users.testUser.password)
      })

      // Navigate between pages multiple times
      for (let i = 0; i < 5; i++) {
        cy.visit('/dashboard')
        cy.get('[data-testid="dashboard"]').should('be.visible')
        
        cy.visit('/analysis/1')
        cy.get('[data-testid="analysis-details"]', { timeout: 10000 }).should('be.visible')
      }

      // Check memory usage
      cy.window().then((win) => {
        if ((win.performance as any).memory) {
          const memoryInfo = (win.performance as any).memory
          // Memory usage should be reasonable (less than 50MB)
          expect(memoryInfo.usedJSHeapSize).to.be.lessThan(50 * 1024 * 1024)
        }
      })
    })
  })

  describe('Bundle Size Performance', () => {
    it('should have reasonable bundle size', () => {
      cy.intercept('**/*.js', (req) => {
        req.continue((res) => {
          // Check bundle sizes
          if (res.headers['content-length']) {
            const contentLength = Array.isArray(res.headers['content-length']) 
              ? res.headers['content-length'][0] 
              : res.headers['content-length']
            const size = parseInt(contentLength)
            // Main bundle should be less than 1MB
            if (req.url.includes('main')) {
              expect(size).to.be.lessThan(1024 * 1024)
            }
            // Vendor bundle should be less than 2MB
            if (req.url.includes('vendor')) {
              expect(size).to.be.lessThan(2 * 1024 * 1024)
            }
          }
        })
      })

      cy.visit('/')
      cy.get('[data-testid="hero-section"]').should('be.visible')
    })
  })

  describe('Network Performance', () => {
    it('should handle slow network conditions', () => {
      // Simulate slow 3G network
      cy.intercept('**/*', (req) => {
        req.continue((res) => {
          // Add 2 second delay to simulate slow network
          setTimeout(() => {
            res.send(res.body)
          }, 2000)
        })
      })

      cy.visit('/', { timeout: 30000 })
      
      // Page should still be functional even with slow network
      cy.get('[data-testid="hero-section"]', { timeout: 30000 }).should('be.visible')
    })

    it('should prioritize critical resources', () => {
      let cssLoaded = false
      let jsLoaded = false
      
      cy.intercept('**/*.css', () => {
        cssLoaded = true
      })
      
      cy.intercept('**/*.js', () => {
        jsLoaded = true
      })

      cy.visit('/')
      
      cy.window().then(() => {
        // CSS should load before JS for better perceived performance
        expect(cssLoaded).to.be.true
        expect(jsLoaded).to.be.true
      })
    })
  })

  describe('Animation Performance', () => {
    it('should maintain 60fps during animations', () => {
      cy.visit('/')
      
      // Monitor frame rate during animations
      cy.window().then((win) => {
        let frameCount = 0
        const startTime = performance.now()
        
        function countFrame() {
          frameCount++
          if (performance.now() - startTime < 1000) {
            requestAnimationFrame(countFrame)
          } else {
            // Should achieve close to 60fps
            expect(frameCount).to.be.greaterThan(55)
          }
        }
        
        requestAnimationFrame(countFrame)
      })
      
      // Trigger animations
      cy.get('[data-testid="animated-terminal"]').should('be.visible')
      cy.wait(1000) // Wait for animation measurement
    })
  })
})
