// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="assets/interactive-examples.html">Interactive Examples</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="01-getting-started/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="01-getting-started/first-steps.html"><strong aria-hidden="true">2.</strong> First Steps</a></li><li class="chapter-item expanded "><a href="01-getting-started/configuration.html"><strong aria-hidden="true">3.</strong> Configuration</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="02-user-guide/basic-concepts.html"><strong aria-hidden="true">4.</strong> Basic Concepts</a></li><li class="chapter-item expanded "><a href="02-user-guide/common-use-cases.html"><strong aria-hidden="true">5.</strong> Common Use Cases</a></li><li class="chapter-item expanded "><a href="02-user-guide/troubleshooting.html"><strong aria-hidden="true">6.</strong> Troubleshooting</a></li><li class="chapter-item expanded "><a href="02-user-guide/comprehensive-manual.html"><strong aria-hidden="true">7.</strong> Comprehensive Manual</a></li><li class="chapter-item expanded "><a href="02-user-guide/feature-flags.html"><strong aria-hidden="true">8.</strong> Feature Flags</a></li><li class="chapter-item expanded "><a href="02-user-guide/title-standardization-guide.html"><strong aria-hidden="true">9.</strong> Title Standardization Guide</a></li><li class="chapter-item expanded "><a href="02-user-guide/dashboard-guide.html"><strong aria-hidden="true">10.</strong> Dashboard Guide</a></li><li class="chapter-item expanded "><a href="02-user-guide/dependency-injection-guide.html"><strong aria-hidden="true">11.</strong> Dependency Injection Guide</a></li><li class="chapter-item expanded "><a href="02-user-guide/tui-interface.html"><strong aria-hidden="true">12.</strong> Terminal User Interface (TUI)</a></li><li class="chapter-item expanded affix "><li class="part-title">API Reference</li><li class="chapter-item expanded "><a href="api/openapi.html"><strong aria-hidden="true">13.</strong> OpenAPI Specification</a></li><li class="chapter-item expanded "><a href="api/rust-documentation.html"><strong aria-hidden="true">14.</strong> Rust Documentation</a></li><li class="chapter-item expanded affix "><li class="part-title">Development</li><li class="chapter-item expanded "><a href="05-development/DEVELOPER_GUIDE.html"><strong aria-hidden="true">15.</strong> Developer Guide</a></li><li class="chapter-item expanded "><a href="05-development/contributing.html"><strong aria-hidden="true">16.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="05-development/detector_development_guide.html"><strong aria-hidden="true">17.</strong> Detector Development</a></li><li class="chapter-item expanded affix "><li class="part-title">Architecture</li><li class="chapter-item expanded "><a href="04-architecture/overview.html"><strong aria-hidden="true">18.</strong> Overview</a></li><li class="chapter-item expanded "><a href="04-architecture/design-principles.html"><strong aria-hidden="true">19.</strong> Design Principles</a></li><li class="chapter-item expanded "><a href="04-architecture/component-model.html"><strong aria-hidden="true">20.</strong> Component Model</a></li><li class="chapter-item expanded "><a href="04-architecture/data-flow.html"><strong aria-hidden="true">21.</strong> Data Flow</a></li><li class="chapter-item expanded "><a href="04-architecture/extensibility.html"><strong aria-hidden="true">22.</strong> Extensibility</a></li><li class="chapter-item expanded "><a href="04-architecture/performance.html"><strong aria-hidden="true">23.</strong> Performance</a></li><li class="chapter-item expanded "><a href="04-architecture/security-model.html"><strong aria-hidden="true">24.</strong> Security Model</a></li><li class="chapter-item expanded "><a href="04-architecture/docker_local_ai_integration.html"><strong aria-hidden="true">25.</strong> Docker Local AI Integration</a></li><li class="chapter-item expanded "><a href="04-architecture/analysis-engine.html"><strong aria-hidden="true">26.</strong> Analysis Engine</a></li><li class="chapter-item expanded "><a href="04-architecture/EntRelDiag.html"><strong aria-hidden="true">27.</strong> Entity Relationship Diagram</a></li><li class="chapter-item expanded "><a href="04-architecture/C4_ARCHITECTURE.html"><strong aria-hidden="true">28.</strong> C4 Architecture</a></li><li class="chapter-item expanded "><a href="04-architecture/database-guide.html"><strong aria-hidden="true">29.</strong> Database Guide</a></li><li class="chapter-item expanded affix "><li class="part-title">Operations</li><li class="chapter-item expanded "><a href="operations/deployment-guide.html"><strong aria-hidden="true">30.</strong> Deployment Guide</a></li><li class="chapter-item expanded affix "><li class="part-title">Security</li><li class="chapter-item expanded "><a href="security/SECURITY_HARDENING_GUIDE.html"><strong aria-hidden="true">31.</strong> Security Hardening Guide</a></li><li class="chapter-item expanded "><a href="security/threat-model.html"><strong aria-hidden="true">32.</strong> Threat Model</a></li><li class="chapter-item expanded "><a href="security/vulnerability-disclosure.html"><strong aria-hidden="true">33.</strong> Vulnerability Disclosure</a></li><li class="chapter-item expanded "><a href="security/INCIDENT_RESPONSE_RUNBOOKS.html"><strong aria-hidden="true">34.</strong> Incident Response Runbooks</a></li><li class="chapter-item expanded "><a href="security/SECURITY_COMPLIANCE_CHECKLIST.html"><strong aria-hidden="true">35.</strong> Security Compliance Checklist</a></li><li class="chapter-item expanded affix "><li class="part-title">Community</li><li class="chapter-item expanded "><a href="09-community/GUIDELINES.html"><strong aria-hidden="true">36.</strong> Guidelines</a></li><li class="chapter-item expanded "><a href="09-community/CONTRIBUTORS.html"><strong aria-hidden="true">37.</strong> Contributors</a></li><li class="chapter-item expanded "><a href="09-community/support.html"><strong aria-hidden="true">38.</strong> Support</a></li><li class="chapter-item expanded "><a href="09-community/faq.html"><strong aria-hidden="true">39.</strong> FAQ</a></li><li class="chapter-item expanded "><a href="09-community/CHANGELOG.html"><strong aria-hidden="true">40.</strong> Changelog</a></li><li class="chapter-item expanded "><a href="09-community/TASK_COMPLEXITY_SCORING.html"><strong aria-hidden="true">41.</strong> Task Complexity Scoring</a></li><li class="chapter-item expanded "><a href="09-community/GOOD_FIRST_ISSUES.html"><strong aria-hidden="true">42.</strong> Good First Issues</a></li><li class="chapter-item expanded "><a href="09-community/Technical_and_Value_Proposition.html"><strong aria-hidden="true">43.</strong> Technical and Value Proposition</a></li><li class="chapter-item expanded "><a href="09-community/MENTORSHIP_SYSTEM.html"><strong aria-hidden="true">44.</strong> Mentorship System</a></li><li class="chapter-item expanded "><a href="09-community/CONTRIBUTING.html"><strong aria-hidden="true">45.</strong> Contributing (Community)</a></li><li class="chapter-item expanded "><a href="09-community/glossary.html"><strong aria-hidden="true">46.</strong> Glossary</a></li><li class="chapter-item expanded "><a href="09-community/SECURITY.html"><strong aria-hidden="true">47.</strong> Security (Community)</a></li><li class="chapter-item expanded "><a href="09-community/CELEBRATION_SYSTEM.html"><strong aria-hidden="true">48.</strong> Celebration System</a></li><li class="chapter-item expanded "><a href="09-community/resources.html"><strong aria-hidden="true">49.</strong> Resources</a></li><li class="chapter-item expanded "><a href="09-community/CODE_OF_CONDUCT.html"><strong aria-hidden="true">50.</strong> Code of Conduct</a></li><li class="chapter-item expanded "><a href="09-community/ISSUE_PREPARATION_TEMPLATE.html"><strong aria-hidden="true">51.</strong> Issue Preparation Template</a></li><li class="chapter-item expanded "><a href="09-community/PRD.html"><strong aria-hidden="true">52.</strong> PRD</a></li><li class="chapter-item expanded "><a href="09-community/CONTRIBUTION_MATRIX.html"><strong aria-hidden="true">53.</strong> Contribution Matrix</a></li><li class="chapter-item expanded "><a href="09-community/teamwork.html"><strong aria-hidden="true">54.</strong> Teamwork</a></li><li class="chapter-item expanded "><a href="09-community/ISSUE_AUDIT_RESULTS.html"><strong aria-hidden="true">55.</strong> Issue Audit Results</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples</li><li class="chapter-item expanded "><a href="08-examples/basic-example.html"><strong aria-hidden="true">56.</strong> Basic Example</a></li><li class="chapter-item expanded "><a href="08-examples/advanced-example.html"><strong aria-hidden="true">57.</strong> Advanced Example</a></li><li class="chapter-item expanded "><a href="08-examples/performance-optimization.html"><strong aria-hidden="true">58.</strong> Performance Optimization</a></li><li class="chapter-item expanded "><a href="08-examples/integration-example.html"><strong aria-hidden="true">59.</strong> Integration Example</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="10-reference/error-codes.html"><strong aria-hidden="true">60.</strong> Error Codes</a></li><li class="chapter-item expanded "><a href="10-reference/SAM.html"><strong aria-hidden="true">61.</strong> SAM</a></li><li class="chapter-item expanded "><a href="10-reference/dead_code_detection.html"><strong aria-hidden="true">62.</strong> Dead Code Detection</a></li><li class="chapter-item expanded "><a href="10-reference/Rust_Error_Stratiegy.html"><strong aria-hidden="true">63.</strong> Rust Error Strategy</a></li><li class="chapter-item expanded "><a href="10-reference/security-api-reference.html"><strong aria-hidden="true">64.</strong> Security API Reference</a></li><li class="chapter-item expanded "><a href="10-reference/testing_strategy.html"><strong aria-hidden="true">65.</strong> Testing Strategy</a></li><li class="chapter-item expanded "><a href="10-reference/ci/cd_guide.html"><strong aria-hidden="true">66.</strong> CI/CD Guide</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
