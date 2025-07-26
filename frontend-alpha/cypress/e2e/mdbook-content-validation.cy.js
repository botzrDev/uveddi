describe('MDBook Content Validation', () => {
  let documentationPages = [];

  before(() => {
    // First, collect all documentation pages
    cy.visit('/dashboard');
    cy.wait(3000);
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        cy.get('.chapter a').then(($links) => {
          documentationPages = Array.from($links).map(link => ({
            title: link.textContent.trim(),
            href: link.href,
            element: link
          })).filter(page => page.href && !page.href.includes('#'));
        });
      });
    });
  });

  it('should have a table of contents with multiple chapters', () => {
    cy.visit('/dashboard');
    cy.wait(2000);
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        cy.get('.chapter').should('have.length.greaterThan', 5); // Should have multiple chapters
        cy.get('.chapter a').should('have.length.greaterThan', 10); // Should have multiple pages
      });
    });
  });

  it('should validate content on key documentation pages', () => {
    const keyPages = [
      { path: '/index.html', expectedContent: ['Uveddi', 'documentation'] },
      { path: '/01-getting-started/installation.html', expectedContent: ['install', 'setup'] },
      { path: '/02-user-guide/basic-concepts.html', expectedContent: ['concept', 'guide'] },
      { path: '/04-architecture/overview.html', expectedContent: ['architecture', 'system'] },
      { path: '/05-development/contributing.html', expectedContent: ['contribute', 'development'] }
    ];

    keyPages.forEach(page => {
      cy.request({ url: page.path, failOnStatusCode: false }).then((response) => {
        if (response.status === 200) {
          expect(response.body).to.include.oneOf(page.expectedContent);
          expect(response.body.length).to.be.greaterThan(100); // Should have substantial content
        }
      });
    });
  });

  it('should check that all pages have minimum content requirements', () => {
    // Test a sample of pages to ensure they have content
    const pagesToTest = [
      '/index.html',
      '/01-getting-started/installation.html',
      '/01-getting-started/configuration.html',
      '/02-user-guide/basic-concepts.html',
      '/04-architecture/overview.html'
    ];

    pagesToTest.forEach(pagePath => {
      cy.request({ url: pagePath, failOnStatusCode: false }).then((response) => {
        if (response.status === 200) {
          const content = response.body;
          
          // Check minimum content requirements
          expect(content).to.have.length.greaterThan(200); // At least 200 characters
          expect(content).to.match(/<h[1-6]>/); // Should have at least one heading
          expect(content).to.match(/<p>/); // Should have at least one paragraph
          expect(content).not.to.include('TODO'); // Should not have TODO placeholders
          expect(content).not.to.include('Lorem ipsum'); // Should not have placeholder text
        }
      });
    });
  });

  it('should validate navigation structure integrity', () => {
    cy.visit('/dashboard');
    cy.wait(2000);
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Check for proper navigation structure
        cy.get('.sidebar-scrollbox').should('exist');
        cy.get('.chapter').should('have.length.greaterThan', 0);
        
        // Check that navigation items are properly nested
        cy.get('.chapter').each(($chapter) => {
          cy.wrap($chapter).find('a').should('have.attr', 'href');
        });
        
        // Check for proper section organization
        cy.get('.chapter').first().should('contain.text', /getting.started|introduction|overview/i);
      });
    });
  });

  it('should test cross-references and internal links', () => {
    cy.visit('/dashboard');
    cy.wait(2000);
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Click on a few navigation items to test they work
        cy.get('.chapter a').first().click();
        cy.wait(1000);
        cy.get('main').should('not.be.empty');
        
        // Test another navigation item
        cy.get('.chapter a').eq(2).click();
        cy.wait(1000);
        cy.get('main').should('not.be.empty');
      });
    });
  });

  it('should validate that images and assets load correctly', () => {
    cy.visit('/dashboard');
    cy.wait(2000);
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Check for any images in the documentation
        cy.get('img').each(($img) => {
          cy.wrap($img).should('have.attr', 'src');
          cy.wrap($img).should('be.visible');
        });
        
        // Check for CSS files
        cy.get('link[rel="stylesheet"]').each(($link) => {
          const href = $link.attr('href');
          if (href && !href.startsWith('http')) {
            cy.request(href).its('status').should('eq', 200);
          }
        });
      });
    });
  });

  it('should check for accessibility features', () => {
    cy.visit('/dashboard');
    cy.wait(2000);
    
    cy.get('iframe[title="Uveddi Documentation"]').then(($iframe) => {
      const iframe = $iframe[0];
      const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
      
      cy.wrap(iframeDoc).within(() => {
        // Check for proper heading hierarchy
        cy.get('h1').should('exist');
        cy.get('main').should('have.attr', 'role').or('exist');
        
        // Check for keyboard navigation support
        cy.get('.sidebar-scrollbox').should('be.visible');
        cy.get('#searchbar').should('be.visible');
      });
    });
  });
});