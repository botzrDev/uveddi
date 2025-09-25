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
const http = require('http');
const { setupWebSocketServer } = require('./websocket');
const { convertCliOutputToDashboardFormat } = require('./utils/dataModelConverter');
const routes = require('./src/routes');

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

// Mount routes
app.use(routes);

// Routes








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

// Specific demo endpoint (must come before the generic :id route)
app.get('/api/v1/reports/demo', async (req, res) => {
  try {
    // Return demo report data - same as latest for now
    const demoData = await getDemoReportData();
    res.json(demoData);
  } catch (error) {
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'Failed to load demo report',
      timestamp: new Date().toISOString()
    });
  }
});

app.get('/api/v1/reports/:id', async (req, res) => {
  const { id } = req.params;
  
  // For now, redirect non-demo requests to latest analysis
  // In production, this would load the specific report by ID
  if (id === 'latest') {
    return res.redirect('/api/v1/reports/latest');
  } else {
    res.status(404).json({
      error: 'Report Not Found',
      message: `Report with ID '${id}' not found`,
      timestamp: new Date().toISOString()
    });
  }
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

// Get demo report data
async function getDemoReportData() {
  // Return a simplified demo report
  return {
    schemaVersion: '1.0',
    project: {
      id: 'demo-project',
      name: 'Demo Analysis Project',
      languages: ['Rust', 'JavaScript'],
      path: 'demo-project/'
    },
    summary: {
      timeGenerated: new Date().toISOString(),
      coverage: 88,
      issuesTotal: 15,
      filesAnalyzed: 8,
      componentsAnalyzed: 12,
      analysisDurationMs: 1250,
      issuesBySeverity: {
        critical: 1,
        high: 3,
        medium: 6,
        low: 5
      },
      issuesByCategory: {
        'god-object': 1,
        'code-duplication': 3,
        'dead-code': 2,
        'long-methods': 4,
        'magic-values': 5
      }
    },
    findings: [
      {
        id: 'demo-1',
        type: 'God Object',
        severity: 'critical',
        title: 'Class has too many responsibilities',
        message: 'The UserService class handles authentication, database operations, and email notifications.',
        file: 'src/user_service.rs',
        startLine: 15,
        endLine: 180,
        tags: ['maintainability', 'architecture'],
        detector: 'uveddi',
        confidence: 0.95
      },
      {
        id: 'demo-2',
        type: 'Code Duplication',
        severity: 'medium',
        title: 'Duplicated validation logic',
        message: 'Email validation logic is duplicated across multiple modules.',
        file: 'src/validators.js',
        startLine: 42,
        endLine: 58,
        tags: ['duplication', 'maintainability'],
        detector: 'uveddi',
        confidence: 0.87
      }
    ],
    dependencyGraph: {
      nodes: [],
      edges: [],
      metadata: {
        nodeCount: 8,
        edgeCount: 12,
        hasCycles: false,
        maxDepth: 4
      }
    },
    metadata: {
      generatedAt: new Date().toISOString(),
      uveddiVersion: '0.9.0',
      configuration: {
        'analysis.depth': 'full',
        'detectors.enabled': 'all'
      }
    }
  };
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

// API v1 endpoints that frontend expects


app.get('/api/v1/reports', (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 10;
    const sort = req.query.sort || 'generatedAt';
    const order = req.query.order || 'desc';

    // Mock data - in real implementation this would come from database
    const reports = [
      {
        id: 'latest',
        name: 'Latest Analysis',
        path: 'src/',
        generatedAt: new Date().toISOString(),
        issuesTotal: 42,
        status: 'completed',
        filesAnalyzed: 156
      },
      {
        id: 'demo',
        name: 'Demo Analysis Report',
        path: 'test-project/',
        generatedAt: new Date(Date.now() - 86400000).toISOString(),
        issuesTotal: 23,
        status: 'completed',
        filesAnalyzed: 89
      }
    ];

    const total = reports.length;
    const pages = Math.ceil(total / limit);
    const startIndex = (page - 1) * limit;
    const endIndex = startIndex + limit;
    const paginatedReports = reports.slice(startIndex, endIndex);

    res.json({
      data: {
        reports: paginatedReports,
        pagination: {
          page: page,
          limit: limit,
          total: total,
          pages: pages
        }
      }
    });
  } catch (error) {
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'Failed to fetch reports',
      timestamp: new Date().toISOString()
    });
  }
});

app.get('/api/v1/security/sarif', (req, res) => {
  try {
    const sarifReport = {
      version: '2.1.0',
      $schema: 'https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json',
      runs: [
        {
          tool: {
            driver: {
              name: 'Uveddi',
              version: '0.9.0',
              informationUri: 'https://github.com/your-org/uveddi'
            }
          },
          results: [
            {
              ruleId: 'god-object',
              level: 'warning',
              message: {
                text: 'God Object detected: excessive responsibilities'
              },
              locations: [
                {
                  physicalLocation: {
                    artifactLocation: {
                      uri: 'src/example.rs'
                    },
                    region: {
                      startLine: 10,
                      endLine: 50
                    }
                  }
                }
              ]
            }
          ]
        }
      ]
    };

    res.setHeader('Content-Type', 'application/json');
    res.setHeader('Content-Disposition', 'attachment; filename="uveddi-security-report.sarif"');
    res.json(sarifReport);
  } catch (error) {
    res.status(500).json({
      error: 'SARIF Export Error',
      message: 'Failed to generate SARIF report',
      timestamp: new Date().toISOString()
    });
  }
});

// Catch-all for undefined routes
app.use('*', (req, res) => {
  res.status(404).json({ 
    error: 'Route not found',
    availableEndpoints: [
      'GET /health',
      'GET /api/v1/health',
      'GET /api/v1/metrics',
      'GET /api/v1/reports',
      'GET /api/v1/reports/latest',
      'GET /api/v1/reports/:id',
      'GET /api/v1/reports/:id/graphs/dependency',
      'GET /api/v1/reports/:id/export?format={format}',
      'GET /api/v1/reports/latest/export?format={format}',
      'GET /api/v1/security/sarif',
      'GET /api/docs/structure',
      'GET /api/docs/file/{path}',
      'GET /api/docs/search?q={query}',
      'GET /api/project/info',
      'GET /api/analysis/status',
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