/**
 * Uveddi Documentation Interactive Enhancements
 * Adds interactivity and improved UX to the documentation
 */

(function() {
    'use strict';

    // Wait for DOM to be ready
    document.addEventListener('DOMContentLoaded', function() {
        initializeEnhancements();
    });

    function initializeEnhancements() {
        addCopyCodeButtons();
        enhanceCallouts();
        addSmoothScrolling();
        addProgressIndicator();
        enhanceNavigation();
        // addKeyboardShortcuts(); // Disabled to remove dead link
        initializeMermaidTheme();
        hideKeyboardShortcutsLink();
    }

    // Add copy buttons to code blocks
    function addCopyCodeButtons() {
        const codeBlocks = document.querySelectorAll('pre code');
        
        codeBlocks.forEach(function(codeBlock) {
            const pre = codeBlock.parentElement;
            const button = document.createElement('button');
            
            button.className = 'copy-code-button';
            button.innerHTML = '📋 Copy';
            button.setAttribute('aria-label', 'Copy code to clipboard');
            
            button.addEventListener('click', function() {
                navigator.clipboard.writeText(codeBlock.textContent).then(function() {
                    button.innerHTML = '✅ Copied!';
                    button.style.background = 'var(--uveddi-success-500)';
                    
                    setTimeout(function() {
                        button.innerHTML = '📋 Copy';
                        button.style.background = '';
                    }, 2000);
                }).catch(function() {
                    button.innerHTML = '❌ Failed';
                    setTimeout(function() {
                        button.innerHTML = '📋 Copy';
                    }, 2000);
                });
            });
            
            // Style the button
            button.style.cssText = `
                position: absolute;
                top: 8px;
                right: 8px;
                background: var(--uveddi-primary);
                color: white;
                border: none;
                border-radius: 4px;
                padding: 4px 8px;
                font-size: 12px;
                cursor: pointer;
                opacity: 0;
                z-index: 10;
            `;
            
            pre.style.position = 'relative';
            pre.appendChild(button);
            
            pre.addEventListener('mouseenter', function() {
                button.style.opacity = '1';
            });
            
            pre.addEventListener('mouseleave', function() {
                button.style.opacity = '0';
            });
        });
    }

    // Enhance callouts with icons and better styling
    function enhanceCallouts() {
        // Convert blockquotes to callouts based on content
        const blockquotes = document.querySelectorAll('blockquote');
        
        blockquotes.forEach(function(blockquote) {
            const text = blockquote.textContent.toLowerCase();
            let type = 'info';
            let icon = 'ℹ️';
            
            if (text.includes('warning') || text.includes('caution') || text.includes('⚠️')) {
                type = 'warning';
                icon = '⚠️';
            } else if (text.includes('note') || text.includes('tip') || text.includes('💡')) {
                type = 'info';
                icon = '💡';
            } else if (text.includes('success') || text.includes('✅')) {
                type = 'success';
                icon = '✅';
            } else if (text.includes('error') || text.includes('danger') || text.includes('❌')) {
                type = 'error';
                icon = '❌';
            }
            
            blockquote.className = `callout ${type}`;
            
            // Add icon if not already present
            if (!blockquote.textContent.match(/[📝💡⚠️✅❌ℹ️]/)) {
                const iconSpan = document.createElement('span');
                iconSpan.textContent = icon + ' ';
                iconSpan.style.marginRight = '0.5rem';
                blockquote.insertBefore(iconSpan, blockquote.firstChild);
            }
        });
    }

    // Add smooth scrolling for anchor links
    function addSmoothScrolling() {
        const links = document.querySelectorAll('a[href^="#"]');
        
        links.forEach(function(link) {
            link.addEventListener('click', function(e) {
                const targetId = this.getAttribute('href').substring(1);
                const targetElement = document.getElementById(targetId);
                
                if (targetElement) {
                    e.preventDefault();
                    targetElement.scrollIntoView({
                        behavior: 'smooth',
                        block: 'start'
                    });
                    
                    // Update URL without jumping
                    history.pushState(null, null, '#' + targetId);
                }
            });
        });
    }

    // Add reading progress indicator
    function addProgressIndicator() {
        const progressBar = document.createElement('div');
        progressBar.id = 'reading-progress';
        progressBar.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            width: 0%;
            height: 3px;
            background: linear-gradient(90deg, var(--uveddi-primary), var(--uveddi-action-500));
            z-index: 1000;
            transition: width 0.1s ease;
        `;
        
        document.body.appendChild(progressBar);
        
        window.addEventListener('scroll', function() {
            const winScroll = document.body.scrollTop || document.documentElement.scrollTop;
            const height = document.documentElement.scrollHeight - document.documentElement.clientHeight;
            const scrolled = (winScroll / height) * 100;
            progressBar.style.width = scrolled + '%';
        });
    }

    // Enhance navigation with active states
    function enhanceNavigation() {
        const navLinks = document.querySelectorAll('.chapter-item a');
        const currentPath = window.location.pathname;
        
        navLinks.forEach(function(link) {
            if (link.getAttribute('href') === currentPath || 
                currentPath.includes(link.getAttribute('href'))) {
                link.parentElement.classList.add('active');
            }
        });
    }

    // Add keyboard shortcuts
    function addKeyboardShortcuts() {
        document.addEventListener('keydown', function(e) {
            // Ctrl/Cmd + K for search
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                const searchInput = document.getElementById('searchbar');
                if (searchInput) {
                    searchInput.focus();
                }
            }
            
            // Escape to close search
            if (e.key === 'Escape') {
                const searchInput = document.getElementById('searchbar');
                if (searchInput && document.activeElement === searchInput) {
                    searchInput.blur();
                }
            }
        });
    }

    // Initialize Mermaid with Uveddi theme
    function initializeMermaidTheme() {
        if (typeof mermaid !== 'undefined') {
            mermaid.initialize({
                theme: 'base',
                themeVariables: {
                    primaryColor: '#4f46e5',
                    primaryTextColor: '#1e293b',
                    primaryBorderColor: '#3730a3',
                    lineColor: '#64748b',
                    secondaryColor: '#f1f5f9',
                    tertiaryColor: '#e2e8f0',
                    background: '#ffffff',
                    mainBkg: '#ffffff',
                    secondBkg: '#f8fafc',
                    tertiaryBkg: '#f1f5f9'
                },
                flowchart: {
                    useMaxWidth: true,
                    htmlLabels: true
                },
                sequence: {
                    useMaxWidth: true,
                    wrap: true
                }
            });
        }
    }

    // Add table of contents generator for long pages
    function generateTableOfContents() {
        const headings = document.querySelectorAll('h2, h3, h4');
        if (headings.length < 3) return; // Don't generate TOC for short pages
        
        const toc = document.createElement('div');
        toc.className = 'table-of-contents';
        toc.innerHTML = '<h3>📚 Table of Contents</h3>';
        
        const list = document.createElement('ul');
        
        headings.forEach(function(heading, index) {
            if (!heading.id) {
                heading.id = 'heading-' + index;
            }
            
            const listItem = document.createElement('li');
            const link = document.createElement('a');
            link.href = '#' + heading.id;
            link.textContent = heading.textContent;
            link.className = 'toc-link toc-' + heading.tagName.toLowerCase();
            
            listItem.appendChild(link);
            list.appendChild(listItem);
        });
        
        toc.appendChild(list);
        
        // Insert TOC after the first paragraph or heading
        const firstParagraph = document.querySelector('.content p');
        if (firstParagraph) {
            firstParagraph.parentNode.insertBefore(toc, firstParagraph.nextSibling);
        }
    }

    // Hide keyboard shortcuts link (dead link)
    function hideKeyboardShortcutsLink() {
        // Wait for mdbook to load completely
        setTimeout(function() {
            const keyboardLinks = document.querySelectorAll('a[href*="keyboard"], a[title*="keyboard"], a[title*="Keyboard"]');
            keyboardLinks.forEach(function(link) {
                if (link.textContent.toLowerCase().includes('keyboard') || 
                    link.title.toLowerCase().includes('keyboard')) {
                    link.style.display = 'none';
                }
            });
            
            // Also check for any menu items or buttons
            const menuItems = document.querySelectorAll('.menu-bar button, .menu-bar a, .sidebar a');
            menuItems.forEach(function(item) {
                if (item.textContent.toLowerCase().includes('keyboard shortcuts') ||
                    item.title.toLowerCase().includes('keyboard shortcuts')) {
                    item.style.display = 'none';
                }
            });
        }, 1000);
    }

    // Initialize additional features after a short delay
    setTimeout(function() {
        generateTableOfContents();
    }, 500);

})();