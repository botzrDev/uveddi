//! HTML Templates and Static Content
//!
//! This module contains HTML, CSS, and JavaScript templates used by the report generator.

/// Main CSS styles for the HTML report
pub const HTML_STYLES: &str = r#"
        /* Modern CSS Variables for theming */
        :root {
            --primary-color: #2563eb;
            --secondary-color: #64748b;
            --success-color: #059669;
            --warning-color: #d97706;
            --error-color: #dc2626;
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #e2e8f0;
            --text-primary: #0f172a;
            --text-secondary: #475569;
            --border-color: #e2e8f0;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            --radius: 8px;
        }

        [data-theme="dark"] {
            --primary-color: #3b82f6;
            --secondary-color: #94a3b8;
            --success-color: #10b981;
            --warning-color: #f59e0b;
            --error-color: #ef4444;
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-tertiary: #334155;
            --text-primary: #f1f5f9;
            --text-secondary: #cbd5e1;
            --border-color: #475569;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: var(--bg-secondary);
            color: var(--text-primary);
            line-height: 1.6;
            transition: background-color 0.3s ease, color 0.3s ease;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }

        /* Header Styles */
        .header {
            background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
            color: white;
            padding: 2rem;
            border-radius: var(--radius);
            margin-bottom: 2rem;
            position: relative;
            overflow: hidden;
        }

        .header::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: linear-gradient(45deg, transparent 49%, rgba(255,255,255,0.1) 50%, transparent 51%);
            animation: shimmer 3s infinite;
        }

        @keyframes shimmer {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(100%); }
        }

        .header h1 {
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            position: relative;
            z-index: 1;
        }

        .header .subtitle {
            font-size: 1.1rem;
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }

        /* Navigation Styles */
        .nav-tabs {
            display: flex;
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 0.5rem;
            margin-bottom: 2rem;
            box-shadow: var(--shadow);
        }

        .nav-tab {
            flex: 1;
            padding: 0.75rem 1rem;
            background: transparent;
            border: none;
            border-radius: calc(var(--radius) - 4px);
            cursor: pointer;
            font-weight: 500;
            color: var(--text-secondary);
            transition: all 0.2s ease;
        }

        .nav-tab.active,
        .nav-tab:hover {
            background: var(--primary-color);
            color: white;
            transform: translateY(-1px);
        }

        /* Card Styles */
        .card {
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            box-shadow: var(--shadow);
            border: 1px solid var(--border-color);
        }

        .card h2 {
            color: var(--primary-color);
            margin-bottom: 1rem;
            font-size: 1.5rem;
            font-weight: 600;
        }

        /* Severity Badge Styles */
        .severity-badge {
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        .severity-critical {
            background: #fef2f2;
            color: #991b1b;
            border: 1px solid #fca5a5;
        }

        .severity-high {
            background: #fef3c7;
            color: #92400e;
            border: 1px solid #fcd34d;
        }

        .severity-medium {
            background: #ecfdf5;
            color: #065f46;
            border: 1px solid #86efac;
        }

        .severity-low {
            background: #f0f9ff;
            color: #0c4a6e;
            border: 1px solid #7dd3fc;
        }

        [data-theme="dark"] .severity-critical {
            background: #450a0a;
            color: #fca5a5;
        }

        [data-theme="dark"] .severity-high {
            background: #451a03;
            color: #fcd34d;
        }

        [data-theme="dark"] .severity-medium {
            background: #052e16;
            color: #86efac;
        }

        [data-theme="dark"] .severity-low {
            background: #0c2d48;
            color: #7dd3fc;
        }

        /* Theme Toggle */
        .theme-toggle {
            position: fixed;
            top: 20px;
            right: 20px;
            background: var(--primary-color);
            color: white;
            border: none;
            border-radius: 50%;
            width: 50px;
            height: 50px;
            font-size: 1.2rem;
            cursor: pointer;
            box-shadow: var(--shadow);
            transition: transform 0.2s ease;
            z-index: 1000;
        }

        .theme-toggle:hover {
            transform: scale(1.1);
        }

        /* Responsive Design */
        @media (max-width: 768px) {
            .container {
                padding: 10px;
            }

            .header h1 {
                font-size: 2rem;
            }

            .nav-tabs {
                flex-direction: column;
            }
        }
"#;

/// Basic navigation JavaScript
pub const BASIC_NAVIGATION_JS: &str = r#"
        function showSection(sectionId) {
            // Hide all sections
            document.getElementById('summary').style.display = 'none';
            document.getElementById('severity').style.display = 'none';
            document.getElementById('detailed').style.display = 'none';

            // Show selected section
            document.getElementById(sectionId).style.display = 'block';

            // Update tab states
            const tabs = document.querySelectorAll('.nav-tab');
            tabs.forEach(tab => tab.classList.remove('active'));
            event.target.classList.add('active');
        }
"#;

/// Theme toggle JavaScript
pub const THEME_TOGGLE_JS: &str = r#"
        // Theme toggle functionality
        function toggleTheme() {
            const body = document.body;
            const currentTheme = body.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            body.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
        }

        // Load saved theme
        document.addEventListener('DOMContentLoaded', function() {
            const savedTheme = localStorage.getItem('theme') || 'light';
            document.body.setAttribute('data-theme', savedTheme);
        });
"#;

/// HTML head template
pub const HTML_HEAD_TEMPLATE: &str = r#"
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Uveddi Architectural Analysis Report</title>

    <!-- Mermaid.js for diagram rendering -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mermaid/10.6.1/mermaid.min.js"></script>

    <!-- Font Awesome for icons -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">

    <style>
        {styles}
    </style>
"#;

/// Navigation tabs template
pub const NAVIGATION_TEMPLATE: &str = r#"
    <div class="nav-tabs">
        <button class="nav-tab active" onclick="showSection('summary')">
            <i class="fas fa-chart-pie"></i> Executive Summary
        </button>
        <button class="nav-tab" onclick="showSection('severity')">
            <i class="fas fa-exclamation-triangle"></i> Issues Dashboard
        </button>
        <button class="nav-tab" onclick="showSection('detailed')">
            <i class="fas fa-list-ul"></i> Detailed Analysis
        </button>
    </div>
"#;
