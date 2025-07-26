describe('MDBook Integration Tests', () => {
  beforeEach(() => {
    cy.visit('/dashboard');
    cy.wait(2000); // Wait for iframe to load
  });

  it('should load the dashboard with mdbook iframe', () => {
    // Check that the dashboard loads
    cy.contains('Welcome to Uveddi Alpha Testing').should('be.visible');
    
    // Check that the iframe exists
    cy.get('iframe[title="Uveddi Documentation"]').should('exist');
    
    // Check that the Discord link is present
    cy.contains('Join Discord Server').should('be.visible');
  });

  it('should load mdbook content in iframe', () => {
    // Get the iframe and check its content
    cy.get('iframe[title="Uveddi Documentation"]')
      .should('be.visible')
      .then(($iframe) => {
        const iframe = $iframe[0];
        const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
        
        // Wait for iframe content to load
        cy.wrap(iframeDoc).should('exist');
        
        // Check for mdbook specific elements
        cy.wrap(iframeDoc.body)
          .should('contain.text', 'Uveddi') // Should contain project name
          .should('not.be.empty'); // Should have content
      });
  });

  it('should have working navigation in mdbook', () => {
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      // Check for navigation elements
      cy.wrap(iframeDoc).within(() => {
        // Look for mdbook navigation elements
        cy.get('.sidebar-scrollbox', { timeout: 10000 }).should('exist');
        cy.get('.chapter', { timeout: 10000 }).should('have.length.greaterThan', 0);
      });
    });
  });

  it('should have content on each documentation page', () => {
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Get all chapter links
        cy.get('.chapter a').then(($links) => {
          const links = Array.from($links).map(link => link.href);
          
          // Test each link
          links.slice(0, 10).forEach((link, index) => { // Test first 10 to avoid timeout
            if (link && !link.includes('#')) { // Skip anchor links
              cy.visit(link.replace(window.location.origin, ''));
              cy.wait(1000);
              
              // Check that the page has content
              cy.get('main', { timeout: 10000 }).should('exist');
              cy.get('main').should('not.be.empty');
              cy.get('main').should('contain.text', /\w+/); // Should contain words
            }
          });
        });
      });
    });
  });

  it('should have working search functionality', () => {
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Look for search input
        cy.get('#searchbar', { timeout: 10000 }).should('exist');
        
        // Test search functionality
        cy.get('#searchbar').type('installation');
        cy.wait(1000);
        
        // Check for search results
        cy.get('.search-results', { timeout: 5000 }).should('exist');
      });
    });
  });

  it('should have responsive navigation', () => {
    // Test mobile view
    cy.viewport(375, 667);
    cy.get('iframe[title="Uveddi Documentation"]').should('be.visible');
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Check for mobile menu button
        cy.get('.mobile-nav-toggle', { timeout: 10000 }).should('exist');
      });
    });
    
    // Test desktop view
    cy.viewport(1280, 720);
    cy.get('iframe[title="Uveddi Documentation"]').should('be.visible');
  });

  it('should load all CSS and JS resources', () => {
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      // Check for CSS files
      cy.wrap(iframeDoc).within(() => {
        cy.get('link[rel="stylesheet"]').should('have.length.greaterThan', 0);
        cy.get('script').should('have.length.greaterThan', 0);
      });
    });
  });
});