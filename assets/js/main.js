/**
 * Uveddi Report Interactive JavaScript
 * Self-contained, no external dependencies
 */

// Theme Management
function toggleTheme() {
    const html = document.documentElement;
    const currentTheme = html.getAttribute('data-theme') || 'light';
    const newTheme = currentTheme === 'light' ? 'dark' : 'light';
    
    html.setAttribute('data-theme', newTheme);
    localStorage.setItem('uveddi-theme', newTheme);
    
    // Animate the transition
    document.body.style.transition = 'background-color 0.3s ease, color 0.3s ease';
}

// Initialize theme from localStorage
function initTheme() {
    const savedTheme = localStorage.getItem('uveddi-theme') || 'light';
    document.documentElement.setAttribute('data-theme', savedTheme);
}

// Smooth scrolling for navigation links
function initSmoothScrolling() {
    document.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const targetId = link.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);
            
            if (targetElement) {
                targetElement.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
                
                // Update active nav state
                document.querySelectorAll('.nav-link').forEach(l => l.classList.remove('active'));
                link.classList.add('active');
            }
        });
    });
}

// Code snippet toggle functionality
function toggleSnippet(button) {
    const snippet = button.closest('.code-snippet');
    const content = snippet.querySelector('.code-content');
    const icon = button.querySelector('i');
    
    if (content.style.display === 'none' || !content.style.display) {
        content.style.display = 'block';
        icon.className = 'fas fa-chevron-up';
        snippet.classList.add('expanded');
    } else {
        content.style.display = 'none';
        icon.className = 'fas fa-chevron-down';
        snippet.classList.remove('expanded');
    }
}

// Issue filtering functionality
function filterIssues() {
    const severityFilter = document.getElementById('severity-filter')?.value || 'all';
    const typeFilter = document.getElementById('type-filter')?.value || 'all';
    const issues = document.querySelectorAll('.issue-card');
    
    let visibleCount = 0;
    
    issues.forEach(issue => {
        let show = true;
        
        // Filter by severity
        if (severityFilter !== 'all') {
            const severityClasses = ['severity-critical', 'severity-major', 'severity-moderate', 'severity-minor'];
            const hasSeverity = severityClasses.some(cls => issue.classList.contains(cls));
            show = show && issue.classList.contains(`severity-${severityFilter}`);
        }
        
        // Filter by type (simplified - in real implementation would check data attributes)
        if (typeFilter !== 'all') {
            // This would be implemented based on actual data structure
            // show = show && issue.dataset.typeId === typeFilter;
        }
        
        if (show) {
            issue.style.display = 'block';
            issue.style.animation = 'fadeIn 0.3s ease';
            visibleCount++;
        } else {
            issue.style.display = 'none';
        }
    });
    
    // Update filter results counter
    updateFilterCounter(visibleCount);
}

function searchIssues() {
    const searchTerm = document.getElementById('search-filter')?.value.toLowerCase() || '';
    const issues = document.querySelectorAll('.issue-card');
    
    let visibleCount = 0;
    
    issues.forEach(issue => {
        const text = issue.textContent.toLowerCase();
        const matches = text.includes(searchTerm);
        
        if (matches || searchTerm === '') {
            issue.style.display = 'block';
            issue.style.animation = 'fadeIn 0.3s ease';
            visibleCount++;
        } else {
            issue.style.display = 'none';
        }
    });
    
    updateFilterCounter(visibleCount);
}

function updateFilterCounter(count) {
    const counter = document.getElementById('filter-counter');
    if (counter) {
        counter.textContent = `${count} issue${count !== 1 ? 's' : ''} shown`;
    }
}

// Issue interaction functions
function expandIssue(issueId) {
    const issue = document.querySelector(`[data-issue-id="${issueId}"]`);
    if (issue) {
        issue.classList.toggle('expanded');
        
        // Toggle button text
        const expandBtn = issue.querySelector('.expand-btn');
        if (expandBtn) {
            const isExpanded = issue.classList.contains('expanded');
            expandBtn.innerHTML = isExpanded 
                ? '<i class="fas fa-compress-alt"></i> Collapse'
                : '<i class="fas fa-expand-alt"></i> View Details';
        }
    }
}

function dismissIssue(issueId) {
    const issue = document.querySelector(`[data-issue-id="${issueId}"]`);
    if (issue && confirm('Are you sure you want to dismiss this issue?')) {
        issue.style.transition = 'opacity 0.3s ease';
        issue.style.opacity = '0.5';
        issue.classList.add('dismissed');
        
        // Update dismiss button
        const dismissBtn = issue.querySelector('.dismiss-btn');
        if (dismissBtn) {
            dismissBtn.innerHTML = '<i class="fas fa-undo"></i> Undismiss';
            dismissBtn.onclick = () => undismissIssue(issueId);
        }
    }
}

function undismissIssue(issueId) {
    const issue = document.querySelector(`[data-issue-id="${issueId}"]`);
    if (issue) {
        issue.style.opacity = '1';
        issue.classList.remove('dismissed');
        
        const dismissBtn = issue.querySelector('.dismiss-btn');
        if (dismissBtn) {
            dismissBtn.innerHTML = '<i class="fas fa-times"></i> Dismiss';
            dismissBtn.onclick = () => dismissIssue(issueId);
        }
    }
}

// Diagram interaction functions
function fullscreenDiagram(diagramId) {
    const diagram = document.getElementById(diagramId);
    if (diagram) {
        if (diagram.requestFullscreen) {
            diagram.requestFullscreen();
        } else if (diagram.webkitRequestFullscreen) {
            diagram.webkitRequestFullscreen();
        } else if (diagram.mozRequestFullScreen) {
            diagram.mozRequestFullScreen();
        } else if (diagram.msRequestFullscreen) {
            diagram.msRequestFullscreen();
        }
    }
}

function exportDiagram(diagramId) {
    const diagram = document.getElementById(diagramId);
    const svg = diagram?.querySelector('svg');
    
    if (svg) {
        try {
            const serializer = new XMLSerializer();
            const svgString = serializer.serializeToString(svg);
            const blob = new Blob([svgString], { type: 'image/svg+xml' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `${diagramId}.svg`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            
            URL.revokeObjectURL(url);
        } catch (error) {
            console.error('Failed to export diagram:', error);
            alert('Failed to export diagram. Please try again.');
        }
    } else {
        alert('No diagram content available for export.');
    }
}

// Report export functionality
function exportReport() {
    // Create a clean version for export
    const originalControls = document.querySelectorAll('.header-controls, .issues-filters, .action-btn');
    originalControls.forEach(el => el.style.display = 'none');
    
    // Print the page
    window.print();
    
    // Restore controls after printing
    setTimeout(() => {
        originalControls.forEach(el => el.style.display = '');
    }, 1000);
}

// Native chart rendering using Canvas API
function renderSeverityChart() {
    const canvas = document.getElementById('severityChart');
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    const rect = canvas.getBoundingClientRect();
    
    // Set canvas size
    canvas.width = rect.width * window.devicePixelRatio;
    canvas.height = rect.height * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    
    // Get severity data from the page
    const severityData = extractSeverityData();
    if (!severityData || Object.keys(severityData).length === 0) return;
    
    // Chart configuration
    const colors = {
        'CRITICAL': '#dc2626',
        'MAJOR': '#ea580c',
        'MODERATE': '#d97706',
        'MINOR': '#65a30d'
    };
    
    const centerX = rect.width / 2;
    const centerY = rect.height / 2;
    const radius = Math.min(centerX, centerY) - 40;
    
    let total = 0;
    Object.values(severityData).forEach(count => total += count);
    
    if (total === 0) return;
    
    // Draw pie chart
    let currentAngle = -Math.PI / 2; // Start from top
    
    for (const [severity, count] of Object.entries(severityData)) {
        if (count === 0) continue;
        
        const sliceAngle = (count / total) * 2 * Math.PI;
        
        // Draw slice
        ctx.beginPath();
        ctx.moveTo(centerX, centerY);
        ctx.arc(centerX, centerY, radius, currentAngle, currentAngle + sliceAngle);
        ctx.closePath();
        ctx.fillStyle = colors[severity] || '#6b7280';
        ctx.fill();
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 2;
        ctx.stroke();
        
        // Draw label
        const labelAngle = currentAngle + sliceAngle / 2;
        const labelX = centerX + Math.cos(labelAngle) * (radius * 0.7);
        const labelY = centerY + Math.sin(labelAngle) * (radius * 0.7);
        
        ctx.fillStyle = '#ffffff';
        ctx.font = 'bold 12px Arial';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(count.toString(), labelX, labelY);
        
        currentAngle += sliceAngle;
    }
    
    // Draw center circle
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius * 0.4, 0, 2 * Math.PI);
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--bg-primary').trim();
    ctx.fill();
    ctx.strokeStyle = getComputedStyle(document.documentElement).getPropertyValue('--border-color').trim();
    ctx.lineWidth = 2;
    ctx.stroke();
    
    // Draw total in center
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--text-primary').trim();
    ctx.font = 'bold 18px Arial';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(total.toString(), centerX, centerY - 5);
    ctx.font = '12px Arial';
    ctx.fillText('Total Issues', centerX, centerY + 15);
}

function extractSeverityData() {
    const severityData = {};
    const severityBars = document.querySelectorAll('.severity-item');
    
    severityBars.forEach(item => {
        const nameElement = item.querySelector('.severity-name');
        const countElement = item.querySelector('.severity-count');
        
        if (nameElement && countElement) {
            const name = nameElement.textContent.trim();
            const count = parseInt(countElement.textContent.trim(), 10) || 0;
            severityData[name] = count;
        }
    });
    
    return severityData;
}

// Scroll-based navigation highlighting
function initScrollSpy() {
    const sections = document.querySelectorAll('section[id]');
    const navLinks = document.querySelectorAll('.nav-link');
    
    function updateActiveNav() {
        let currentSection = '';
        const scrollPos = window.scrollY + 100;
        
        sections.forEach(section => {
            const sectionTop = section.offsetTop;
            const sectionHeight = section.offsetHeight;
            
            if (scrollPos >= sectionTop && scrollPos < sectionTop + sectionHeight) {
                currentSection = section.id;
            }
        });
        
        navLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('href') === `#${currentSection}`) {
                link.classList.add('active');
            }
        });
    }
    
    window.addEventListener('scroll', updateActiveNav);
    updateActiveNav(); // Initial call
}

// Keyboard shortcuts
function initKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        // Only activate shortcuts when not typing in input fields
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
        
        switch (e.key) {
            case 't':
                if (e.ctrlKey || e.metaKey) {
                    e.preventDefault();
                    toggleTheme();
                }
                break;
            case 'p':
                if (e.ctrlKey || e.metaKey) {
                    e.preventDefault();
                    window.print();
                }
                break;
            case 'f':
                if (e.ctrlKey || e.metaKey) {
                    e.preventDefault();
                    const searchInput = document.getElementById('search-filter');
                    if (searchInput) {
                        searchInput.focus();
                    }
                }
                break;
        }
    });
}

// Performance monitoring
function initPerformanceMonitoring() {
    // Monitor page load performance
    window.addEventListener('load', () => {
        const perfData = performance.getEntriesByType('navigation')[0];
        if (perfData) {
            console.log(`Page loaded in ${Math.round(perfData.loadEventEnd - perfData.fetchStart)}ms`);
        }
    });
    
    // Monitor chart rendering performance
    const observer = new PerformanceObserver((list) => {
        list.getEntries().forEach((entry) => {
            if (entry.name === 'chart-render') {
                console.log(`Chart rendered in ${Math.round(entry.duration)}ms`);
            }
        });
    });
    
    if ('PerformanceObserver' in window) {
        observer.observe({ entryTypes: ['measure'] });
    }
}

// Animation utilities
function animateValue(element, start, end, duration) {
    if (!element) return;
    
    const startTime = performance.now();
    
    function updateValue(currentTime) {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        const current = start + (end - start) * easeOutCubic(progress);
        element.textContent = Math.round(current);
        
        if (progress < 1) {
            requestAnimationFrame(updateValue);
        }
    }
    
    requestAnimationFrame(updateValue);
}

function easeOutCubic(t) {
    return 1 - Math.pow(1 - t, 3);
}

// Initialize counters animation
function initCounterAnimations() {
    const counters = document.querySelectorAll('.stat-value, .severity-count, .count');
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting && !entry.target.classList.contains('animated')) {
                const target = parseInt(entry.target.textContent) || 0;
                entry.target.classList.add('animated');
                animateValue(entry.target, 0, target, 1000);
            }
        });
    });
    
    counters.forEach(counter => observer.observe(counter));
}

// Add CSS animations
const animationCSS = `
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

@keyframes slideIn {
    from { transform: translateX(-100%); }
    to { transform: translateX(0); }
}

.animated {
    animation: fadeIn 0.5s ease;
}
`;

// Inject animations
function injectAnimations() {
    const style = document.createElement('style');
    style.textContent = animationCSS;
    document.head.appendChild(style);
}

// Main initialization function
function init() {
    console.log('Initializing Uveddi Report Interface...');
    
    // Core functionality
    initTheme();
    initSmoothScrolling();
    initScrollSpy();
    initKeyboardShortcuts();
    
    // Visual enhancements
    injectAnimations();
    initCounterAnimations();
    
    // Chart rendering
    setTimeout(() => {
        performance.mark('chart-render-start');
        renderSeverityChart();
        performance.mark('chart-render-end');
        performance.measure('chart-render', 'chart-render-start', 'chart-render-end');
    }, 100);
    
    // Performance monitoring
    if (typeof console !== 'undefined') {
        initPerformanceMonitoring();
    }
    
    // Initialize code snippet toggles
    document.querySelectorAll('.code-snippet').forEach(snippet => {
        const content = snippet.querySelector('.code-content');
        if (content) {
            content.style.display = 'none';
        }
    });
    
    console.log('Uveddi Report Interface initialized successfully');
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}

// Handle resize events
window.addEventListener('resize', () => {
    // Redraw charts on resize
    setTimeout(renderSeverityChart, 100);
});

// Handle theme changes from system
if (window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addListener((e) => {
        if (!localStorage.getItem('uveddi-theme')) {
            document.documentElement.setAttribute('data-theme', e.matches ? 'dark' : 'light');
        }
    });
}