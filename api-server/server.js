/**
 * Uveddi Interactive Reports API Server
 * 
 * This is the main Express.js server that provides API endpoints for the Uveddi
 * dashboard frontend. It serves analysis reports, handles real-time updates via
 * WebSockets, and manages the interactive report viewing experience.
 * 
 * Key features:
 * - RESTful API endpoints for analysis reports and metadata
 * - WebSocket server for real-time analysis progress updates
 * - Security middleware with helmet and CORS protection
 * - Static file serving for generated reports and assets
 * - Data model conversion between CLI output and dashboard format
 * - Markdown processing for report documentation
 * 
 * The server integrates with the Rust analysis engine and serves as the bridge
 * between the command-line tool and the web-based dashboard interface.
 * 
 * @example Starting the server
 * ```javascript
 * // Default configuration
 * npm start
 * 
 * // Custom port
 * PORT=3000 npm start
 * 
 * // Development mode with auto-restart
 * npm run dev
 * ```
 * 
 * @example API Usage
 * ```javascript
 * // Fetch available reports
 * const reports = await fetch('/api/reports').then(r => r.json());
 * 
 * // Get specific report data
 * const report = await fetch(`/api/reports/${reportId}`).then(r => r.json());
 * 
 * // WebSocket connection for real-time updates
 * const ws = new WebSocket('ws://localhost:8080');
 * ```
 */

const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const morgan = require('morgan');
const fs = require('fs-extra');
const path = require('path');
const marked = require('marked');
const matter = require('gray-matter');
const http = require('http');
const { setupWebSocketServer } = require('./websocket');
const { convertCliOutputToDashboardFormat } = require('./utils/dataModelConverter');

const app = express();
const server = http.createServer(app);
const PORT = process.env.PORT || 8000;

// Initialize WebSocket server
setupWebSocketServer(server);

// Security and middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'", "'unsafe-inline'", "'unsafe-eval'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      imgSrc: ["'self'", "data:", "blob:"],
      connectSrc: ["'self'", "ws:", "wss:"],
      fontSrc: ["'self'"],
      objectSrc: ["'none'"],
      mediaSrc: ["'self'"],
      frameSrc: ["'none'"],
    },
  }
}));
app.use(cors({
  origin: [
    'http://localhost:8001', 'http://127.0.0.1:8001',
    'http://localhost:8002', 'http://127.0.0.1:8002',
    'http://localhost:8003', 'http://127.0.0.1:8003',
    'http://localhost:8080', 'http://127.0.0.1:8080',
    'http://localhost:8082', 'http://127.0.0.1:8082'
  ], // Allow frontend ports in 8000 range
  methods: ['GET', 'POST'],
  allowedHeaders: ['Content-Type', 'Authorization']
}));
app.use(morgan('combined'));
app.use(express.json());

// Documentation root path
const DOCS_PATH = path.join(__dirname, '..', 'docs');

// Helper function to read and parse markdown files
async function readMarkdownFile(filePath) {
  try {
    const fullPath = path.join(DOCS_PATH, filePath);
    const fileContent = await fs.readFile(fullPath, 'utf8');
    const { data, content } = matter(fileContent);
    const html = marked.parse(content);
    
    return {
      frontmatter: data,
      content: content,
      html: html,
      path: filePath
    };
  } catch (error) {
    console.error(`Error reading markdown file ${filePath}:`, error);
    return null;
  }
}

// Helper function to get directory structure
async function getDirectoryStructure(dirPath, basePath = '') {
  try {
    const fullPath = path.join(DOCS_PATH, dirPath);
    const items = await fs.readdir(fullPath, { withFileTypes: true });
    
    const structure = [];
    for (const item of items) {
      const itemPath = path.join(basePath, item.name);
      
      if (item.isDirectory()) {
        const children = await getDirectoryStructure(path.join(dirPath, item.name), itemPath);
        structure.push({
          name: item.name,
          type: 'directory',
          path: itemPath,
          children: children
        });
      } else if (item.name.endsWith('.md')) {
        structure.push({
          name: item.name,
          type: 'file',
          path: itemPath,
          title: item.name.replace('.md', '').replace(/[-_]/g, ' ')
        });
      }
    }
    
    return structure;
  } catch (error) {
    console.error(`Error reading directory ${dirPath}:`, error);
    return [];
  }
}

// Routes

// Health check
app.get('/health', (req, res) => {
  res.json({ 
    status: 'ok', 
    service: 'uveddi-api-server-alpha',
    version: '0.9.0-alpha',
    timestamp: new Date().toISOString()
  });
});

// Get documentation structure
app.get('/api/docs/structure', async (req, res) => {
  try {
    const structure = await getDirectoryStructure('');
    res.json({ structure });
  } catch (error) {
    res.status(500).json({ error: 'Failed to read documentation structure' });
  }
});

// Get specific documentation file
app.get('/api/docs/file/*', async (req, res) => {
  try {
    const filePath = req.params[0]; // Everything after /api/docs/file/
    const doc = await readMarkdownFile(filePath);
    
    if (!doc) {
      return res.status(404).json({ error: 'Documentation file not found' });
    }
    
    res.json(doc);
  } catch (error) {
    res.status(500).json({ error: 'Failed to read documentation file' });
  }
});

// Search documentation
app.get('/api/docs/search', async (req, res) => {
  const query = req.query.q?.toLowerCase();
  if (!query) {
    return res.status(400).json({ error: 'Search query required' });
  }
  
  try {
    // Simple search implementation
    const results = [];
    const structure = await getDirectoryStructure('');
    
    async function searchInStructure(items, basePath = '') {
      for (const item of items) {
        if (item.type === 'file' && item.name.endsWith('.md')) {
          const doc = await readMarkdownFile(item.path);
          if (doc && (
            doc.content.toLowerCase().includes(query) ||
            item.title.toLowerCase().includes(query)
          )) {
            results.push({
              title: item.title,
              path: item.path,
              snippet: doc.content.substring(0, 200) + '...'
            });
          }
        } else if (item.type === 'directory' && item.children) {
          await searchInStructure(item.children, item.path);
        }
      }
    }
    
    await searchInStructure(structure);
    res.json({ results, query });
  } catch (error) {
    res.status(500).json({ error: 'Search failed' });
  }
});

// Get project information
app.get('/api/project/info', (req, res) => {
  res.json({
    name: 'Uveddi',
    version: '0.9.0-alpha',
    description: 'AI-Powered Code Analysis Tool',
    repository: 'https://github.com/botzrDev/uveddi',
    documentation: 'https://botzrdev.github.io/uveddi/',
    features: [
      'Multi-language analysis',
      'AI-powered insights', 
      'Privacy-focused local analysis',
      'Extensible plugin architecture',
      'Terminal User Interface (TUI)',
      'Comprehensive reporting'
    ],
    environment: 'alpha'
  });
});

// Authentication endpoints for alpha testing
app.post('/auth/register', async (req, res) => {
  try {
    const { email, username, password } = req.body;
    
    // Basic validation
    if (!email || !username || !password) {
      return res.status(400).json({
        detail: 'Email, username, and password are required'
      });
    }
    
    if (password.length < 6) {
      return res.status(400).json({
        detail: 'Password must be at least 6 characters'
      });
    }
    
    // For alpha: simulate successful registration
    const mockUser = {
      id: Date.now(),
      email: email,
      username: username,
      full_name: username,
      is_active: true,
      created_at: new Date().toISOString(),
      role: 'alpha_tester'
    };
    
    const mockToken = `alpha_token_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    res.status(201).json({
      user: mockUser,
      access_token: mockToken,
      token_type: 'bearer',
      message: 'Alpha account created successfully'
    });
    
  } catch (error) {
    res.status(500).json({
      detail: 'Registration failed',
      error: error.message
    });
  }
});

app.post('/auth/login', async (req, res) => {
  try {
    const { email, password } = req.body;
    
    // Basic validation
    if (!email || !password) {
      return res.status(400).json({
        detail: 'Email and password are required'
      });
    }
    
    // For alpha: simulate successful login for any valid email/password
    if (password.length < 6) {
      return res.status(401).json({
        detail: 'Invalid credentials'
      });
    }
    
    const mockUser = {
      id: Date.now(),
      email: email,
      username: email.split('@')[0],
      full_name: email.split('@')[0],
      is_active: true,
      created_at: new Date().toISOString(),
      role: 'alpha_tester'
    };
    
    const mockToken = `alpha_token_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    res.json({
      user: mockUser,
      access_token: mockToken,
      token_type: 'bearer',
      message: 'Alpha login successful'
    });
    
  } catch (error) {
    res.status(500).json({
      detail: 'Login failed',
      error: error.message
    });
  }
});

app.post('/auth/logout', (req, res) => {
  // For alpha: simple logout acknowledgment
  res.json({
    message: 'Logged out successfully'
  });
});

// User profile endpoint
app.get('/users/me', (req, res) => {
  const authHeader = req.headers.authorization;
  
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({
      detail: 'Authentication required'
    });
  }
  
  // For alpha: return mock user data
  const mockUser = {
    id: 1,
    email: 'alpha.tester@uveddi.dev',
    username: 'alpha_tester',
    full_name: 'Alpha Tester',
    is_active: true,
    created_at: new Date().toISOString(),
    role: 'alpha_tester',
    preferences: {
      theme: 'system',
      notifications: true
    }
  };
  
  res.json(mockUser);
});

// Report API endpoints for React dashboard
app.get('/api/v1/reports/latest', async (req, res) => {
  try {
    // Load the most recent analysis data
    const analysisFiles = [
      'ripgrep_analysis.json',
      'bat_analysis.json', 
      'httpie_analysis.json'
    ];
    
    let reportData = null;
    let analysisSource = null;
    
    // Try to find the most recent analysis file
    for (const filename of analysisFiles) {
      const reportPath = path.join(__dirname, '..', filename);
      if (await fs.pathExists(reportPath)) {
        reportData = await fs.readJson(reportPath);
        analysisSource = filename.replace('_analysis.json', '');
        break;
      }
    }
    
    if (!reportData) {
      return res.status(404).json({ error: 'No analysis data available' });
    }
    
    // Transform to expected format for React dashboard
    const transformedReport = {
      schemaVersion: "1.0.0",
      project: {
        id: `analysis-${analysisSource}`,
        name: `Uveddi Analysis - ${analysisSource}`,
        languages: ["rust"],
        path: analysisSource,
        commit: "latest",
        branch: "main"
      },
      summary: {
        timeGenerated: reportData.metadata?.timestamp || new Date().toISOString(),
        coverage: 85.0,
        issuesTotal: reportData.issues?.length || 0,
        filesAnalyzed: getUniqueFiles(reportData.issues).length,
        componentsAnalyzed: getUniqueFiles(reportData.issues).length,
        analysisDurationMs: reportData.metadata?.durationSeconds ? reportData.metadata.durationSeconds * 1000 : 0,
        issuesBySeverity: calculateIssuesBySeverity(reportData.issues || []),
        issuesByCategory: calculateIssuesByCategory(reportData.issues || [])
      },
      findings: (reportData.issues || []).map((issue, index) => ({
        id: `f-${index + 1}`,
        type: issue.antiPatternType,
        severity: issue.severity.toLowerCase(),
        title: `${issue.antiPatternType} in ${issue.filePath}`,
        message: issue.description,
        file: issue.filePath,
        startLine: issue.startLine || 0,
        endLine: issue.endLine || 0,
        codeSnippet: issue.codeSnippet || "",
        tags: [issue.antiPatternType.toLowerCase().replace(/\s+/g, '-')],
        detector: issue.detectorName || 'uveddi',
        confidence: issue.confidence || 0.8,
        aiExplanation: issue.aiExplanation,
        relatedFindings: []
      })),
      dependencyGraph: {
        nodes: [],
        edges: []
      },
      diagrams: [],
      metadata: {
        generatedAt: reportData.metadata?.timestamp || new Date().toISOString(),
        analysisId: `analysis-${analysisSource}-${Date.now()}`
      }
    };
    
    res.json(transformedReport);
  } catch (error) {
    console.error('Error loading analysis report:', error);
    res.status(500).json({ error: 'Failed to load analysis report' });
  }
});

app.get('/api/v1/reports/:id', async (req, res) => {
  const { id } = req.params;
  
  // Redirect all report requests to latest analysis
  return res.redirect('/api/v1/reports/latest');
});

app.get('/api/v1/reports/:id/graphs/dependency', (req, res) => {
  res.json({
    data: {
      nodes: [],
      edges: []
    }
  });
});

// Convert Uveddi analysis output to InteractiveReport format (same logic as frontend)
function convertAnalysisToReportData(analysisData) {
  // Handle both 'issues' (real analysis data) and 'findings' (mock data) arrays
  const issues = analysisData.issues || analysisData.findings || [];
  console.log('📊 Converting data with', issues.length, 'items');
  
  // Group issues by type for dashboard display
  const issuesByType = issues.reduce((acc, issue) => {
    const type = issue.antiPatternType || issue.type || 'Unknown';
    if (!acc[type]) {
      acc[type] = [];
    }
    acc[type].push(issue);
    return acc;
  }, {});

  // Create summary metrics
  const issuesBySeverity = issues.reduce((acc, issue) => {
    const severity = issue.severity === 'Critical' || issue.severity === 'high' ? 'critical' :
                     issue.severity === 'Medium' || issue.severity === 'medium' ? 'medium' :
                     issue.severity === 'Low' || issue.severity === 'low' ? 'low' : 'info';
    acc[severity] = (acc[severity] || 0) + 1;
    return acc;
  }, {});

  const issuesByCategory = Object.keys(issuesByType).reduce((acc, type) => {
    acc[type] = issuesByType[type].length;
    return acc;
  }, {});

  const summary = {
    coverage: analysisData.summary?.coverage || 85, // Use existing summary or default
    filesAnalyzed: analysisData.summary?.filesAnalyzed || new Set(issues.map(i => i.filePath || i.file)).size,
    issuesTotal: issues.length,
    issuesBySeverity,
    issuesByCategory,
    componentsAnalyzed: analysisData.summary?.componentsAnalyzed || new Set(issues.map(i => i.filePath || i.file)).size,
    analysisDurationMs: analysisData.summary?.analysisDurationMs || 2500,
    timeGenerated: new Date().toISOString(),
  };

  return {
    schemaVersion: '1.0',
    project: analysisData.project || {
      id: 'uveddi-analysis',
      name: 'Uveddi Analysis Engine',
      languages: ['Rust'],
      path: 'src/analysis/detectors/'
    },
    summary,
    findings: issues.map((issue, index) => ({
      id: issue.id || `finding-${index + 1}`,
      type: issue.antiPatternType || issue.type || 'Unknown',
      severity: (issue.severity === 'Critical' || issue.severity === 'high' ? 'critical' :
                 issue.severity === 'Medium' || issue.severity === 'medium' ? 'medium' :
                 issue.severity === 'Low' || issue.severity === 'low' ? 'low' : 'low'),
      title: issue.title || issue.antiPatternDescription || issue.message || 'Architectural Issue',
      message: issue.description || issue.antiPatternDescription || issue.message || 'Issue detected',
      file: issue.filePath || issue.file || 'unknown',
      startLine: issue.lineRange?.start || issue.startLine,
      endLine: issue.lineRange?.end || issue.endLine,
      codeSnippet: issue.codeSnippet || '',
      tags: issue.tags || [issue.antiPatternType || issue.type || 'architectural'],
      detector: issue.detectorName || issue.detector || 'uveddi',
      confidence: issue.confidence || 0.8,
      aiExplanation: issue.aiExplanation || issue.description || issue.message
    })),
    dependencyGraph: analysisData.dependencyGraph || {
      nodes: [],
      edges: [],
      metadata: {
        nodeCount: 0,
        edgeCount: 0,
        hasCycles: false,
        maxDepth: 0
      }
    },
    diagrams: analysisData.diagrams || [],
    metadata: analysisData.metadata || {
      generatedAt: new Date().toISOString(),
      analysisId: "real-analysis"
    }
  };
}

// Export latest analysis report (must come before the general /:id/export route)
app.get('/api/v1/reports/latest/export', async (req, res) => {
  const format = req.query.format || 'markdown';

  try {
    let rawData;
    let analysisSource;
    
    // Try to load real analysis data first
    const analysisFiles = [
      'ripgrep_analysis.json',
      'bat_analysis.json', 
      'httpie_analysis.json'
    ];
    
    for (const filename of analysisFiles) {
      const analysisPath = path.join(__dirname, '..', filename);
      if (await fs.pathExists(analysisPath)) {
        rawData = await fs.readJSON(analysisPath);
        analysisSource = filename.replace('_analysis.json', '');
        console.log('🔍 Loaded real analysis data for export:', rawData.issues?.length || 0, 'issues from', analysisSource);
        break;
      }
    }
    
    if (!rawData) {
      return res.status(404).json({ error: 'No analysis data available for export' });
    }

    // Transform raw analysis data to report format (same as frontend)
    const reportData = convertAnalysisToReportData(rawData);
    console.log('🔄 Converted to report format:', reportData.findings?.length || 0, 'findings');

    if (format === 'markdown') {
      const markdown = generateMarkdownReport(reportData);
      
      res.setHeader('Content-Type', 'text/markdown');
      res.setHeader('Content-Disposition', `attachment; filename="uveddi-analysis-${analysisSource}.md"`);
      res.send(markdown);
    } else {
      res.status(400).json({ error: 'Unsupported format. Only markdown is currently supported.' });
    }
  } catch (error) {
    console.error('Analysis export error:', error);
    res.status(500).json({ error: 'Failed to export analysis report' });
  }
});

// Export report as markdown (general route)
app.get('/api/v1/reports/:id/export', async (req, res) => {
  const { id } = req.params;
  const format = req.query.format || 'markdown';

  try {
    let rawData;
    
    // Get the report data
    if (id === 'demo') {
      const demoDataPath = path.join(__dirname, '..', 'real-analysis-detectors.json');
      if (await fs.pathExists(demoDataPath)) {
        rawData = await fs.readJSON(demoDataPath);
      } else {
        const fallbackPath = path.join(__dirname, '..', 'frontend', 'public', 'mock-data', 'demo-report.json');
        rawData = await fs.readJSON(fallbackPath);
      }
    } else {
      return res.status(404).json({ error: 'Report not found' });
    }

    // Transform raw analysis data to report format (same as frontend)
    const reportData = convertAnalysisToReportData(rawData);
    console.log('🔄 General export - converted to report format:', reportData.findings?.length || 0, 'findings');

    if (format === 'markdown') {
      const markdown = generateMarkdownReport(reportData);
      
      res.setHeader('Content-Type', 'text/markdown');
      res.setHeader('Content-Disposition', `attachment; filename="uveddi-analysis-${id}.md"`);
      res.send(markdown);
    } else {
      res.status(400).json({ error: 'Unsupported format. Only markdown is currently supported.' });
    }
  } catch (error) {
    console.error('Export error:', error);
    res.status(500).json({ error: 'Failed to export report' });
  }
});

// Helper functions
function generateMarkdownReport(reportData) {
  const timestamp = new Date().toISOString().split('T')[0];
  
  // Handle transformed report format with 'findings' array
  const findings = reportData.findings || [];
  
  // Log the data structure for debugging
  console.log('Report data structure for markdown:', {
    hasFindings: !!reportData.findings,
    findingsLength: findings.length,
    sampleFinding: findings[0] ? Object.keys(findings[0]) : 'none'
  });
  
  const findingsBySeverity = calculateFindingsBySeverity(findings);
  const findingsByCategory = calculateFindingsByCategory(findings);

  let markdown = `# Uveddi Analysis Report

**Generated:** ${timestamp}  
**Analysis Engine:** Uveddi v0.9.0-alpha  
**Project:** ${reportData.project?.name || 'Code Analysis'}  

---

## Executive Summary

This report presents a comprehensive architectural analysis of your codebase, identifying potential issues and areas for improvement.

### Key Findings

- **Total Issues:** ${findings.length}
- **Critical Issues:** ${findingsBySeverity.critical}
- **High Priority Issues:** ${findingsBySeverity.high}
- **Medium Priority Issues:** ${findingsBySeverity.medium}
- **Low Priority Issues:** ${findingsBySeverity.low}

### Severity Distribution

| Severity | Count | Percentage |
|----------|-------|------------|
| Critical | ${findingsBySeverity.critical} | ${findings.length > 0 ? ((findingsBySeverity.critical / findings.length) * 100).toFixed(1) : 0}% |
| High     | ${findingsBySeverity.high} | ${findings.length > 0 ? ((findingsBySeverity.high / findings.length) * 100).toFixed(1) : 0}% |
| Medium   | ${findingsBySeverity.medium} | ${findings.length > 0 ? ((findingsBySeverity.medium / findings.length) * 100).toFixed(1) : 0}% |
| Low      | ${findingsBySeverity.low} | ${findings.length > 0 ? ((findingsBySeverity.low / findings.length) * 100).toFixed(1) : 0}% |

## Issue Categories

`;

  // Add category breakdown
  Object.entries(findingsByCategory).forEach(([category, count]) => {
    const percentage = findings.length > 0 ? ((count / findings.length) * 100).toFixed(1) : 0;
    markdown += `- **${category.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}:** ${count} (${percentage}%)\n`;
  });

  markdown += `

## Detailed Findings

`;

  if (findings.length === 0) {
    markdown += `No issues were found in the analysis. Great job maintaining clean code!

`;
  } else {
    // Group findings by severity for better organization
    const severityOrder = ['critical', 'high', 'medium', 'low'];
    
    severityOrder.forEach(severity => {
      const severityFindings = findings.filter(finding => 
        finding.severity && finding.severity.toLowerCase() === severity.toLowerCase()
      );
      
      if (severityFindings.length > 0) {
        markdown += `### ${severity.charAt(0).toUpperCase() + severity.slice(1)} Severity Issues

`;
        
        severityFindings.forEach((finding, index) => {
          markdown += `#### ${index + 1}. ${finding.title || finding.message || 'Untitled Issue'}

**Type:** ${finding.type || 'Unknown'}  
**File:** \`${finding.file || 'Unknown'}\`  
**Lines:** ${finding.startLine || 'N/A'}-${finding.endLine || 'N/A'}  
**Confidence:** ${finding.confidence ? Math.round(finding.confidence * 100) + '%' : 'N/A'}

${finding.message || 'No description available.'}

${finding.codeSnippet ? '```' + (finding.file?.endsWith('.rs') ? 'rust' : 'text') + '\n' + finding.codeSnippet + '\n```' : ''}

${finding.aiExplanation && finding.aiExplanation !== finding.message ? '**AI Analysis:** ' + finding.aiExplanation + '\n' : ''}

---

`;
        });
      }
    });
  }

  // Add analysis summary
  markdown += `## Analysis Summary

**Files Analyzed:** ${reportData.summary?.filesAnalyzed || 'Unknown'}  
**Components Analyzed:** ${reportData.summary?.componentsAnalyzed || 'Unknown'}  
**Analysis Duration:** ${reportData.summary?.analysisDurationMs ? (reportData.summary.analysisDurationMs / 1000).toFixed(2) + 's' : 'Unknown'}  
**Generated At:** ${reportData.metadata?.generatedAt || new Date().toISOString()}

---

*Report generated by Uveddi Analysis Engine v0.9.0-alpha*  
*For more information, visit: https://github.com/your-org/uveddi*
`;

  return markdown;
}



function calculateIssuesBySeverity(issues) {
  const counts = { critical: 0, high: 0, medium: 0, low: 0 };
  issues.forEach(issue => {
    const severity = (issue.severity || '').toLowerCase();
    if (counts.hasOwnProperty(severity)) {
      counts[severity]++;
    }
  });
  return counts;
}

function calculateIssuesByCategory(issues) {
  const counts = {};
  issues.forEach(issue => {
    // Handle both 'type' and 'antiPatternType' fields
    const category = (issue.type || issue.antiPatternType || 'unknown')
      .toLowerCase().replace(/\s+/g, '-');
    counts[category] = (counts[category] || 0) + 1;
  });
  return counts;
}

function getUniqueFiles(issues) {
  const files = new Set();
  issues.forEach(issue => {
    if (issue.filePath) {
      files.add(issue.filePath);
    }
  });
  return Array.from(files);
}

function calculateFindingsBySeverity(findings) {
  const counts = { critical: 0, high: 0, medium: 0, low: 0 };
  findings.forEach(finding => {
    const severity = (finding.severity || '').toLowerCase();
    if (counts.hasOwnProperty(severity)) {
      counts[severity]++;
    }
  });
  return counts;
}

function calculateFindingsByCategory(findings) {
  const counts = {};
  findings.forEach(finding => {
    const category = (finding.type || 'unknown')
      .toLowerCase().replace(/\s+/g, '-');
    counts[category] = (counts[category] || 0) + 1;
  });
  return counts;
}

// Mock API endpoints that the frontend might expect
app.get('/api/analysis/status', (req, res) => {
  res.json({
    status: 'ready',
    version: '0.9.0-alpha',
    capabilities: ['code-analysis', 'documentation', 'basic-reporting']
  });
});

app.get('/api/analysis/history', (req, res) => {
  res.json({
    analyses: [],
    message: 'Analysis history will be available in the beta release'
  });
});

// Catch-all for undefined routes
app.use('*', (req, res) => {
  res.status(404).json({ 
    error: 'Route not found',
    availableEndpoints: [
      'GET /health',
      'GET /api/docs/structure',
      'GET /api/docs/file/{path}',
      'GET /api/docs/search?q={query}',
      'GET /api/project/info',
      'GET /api/analysis/status',
      'GET /api/v1/reports/demo',
      'GET /api/v1/reports/:id',
      'GET /api/v1/reports/:id/graphs/dependency',
      'GET /api/v1/reports/:id/export?format={format}',
      'GET /api/v1/reports/demo/export?format={format}',
      'POST /auth/register',
      'POST /auth/login',
      'POST /auth/logout',
      'GET /users/me'
    ]
  });
});

// Error handler
app.use((error, req, res, next) => {
  console.error('Server error:', error);
  res.status(500).json({ error: 'Internal server error' });
});

// Start server
server.listen(PORT, () => {
  console.log(`🚀 Uveddi v1.0 Community Core API Server running on http://localhost:${PORT}`);
  console.log(`📚 Documentation API: http://localhost:${PORT}/api/docs/structure`);
  console.log(`🔍 Health check: http://localhost:${PORT}/health`);
});

module.exports = app;