const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const morgan = require('morgan');
const fs = require('fs-extra');
const path = require('path');
const marked = require('marked');
const matter = require('gray-matter');

const app = express();
const PORT = process.env.PORT || 8080;

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
  origin: ['http://localhost:3000', 'http://localhost:3001', 'http://localhost:3002', 'http://localhost:3003', 'http://localhost:9998', 'http://localhost:9999'], // Allow frontend ports
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
app.get('/api/v1/reports/demo', async (req, res) => {
  try {
    // Load the actual analysis data
    const reportPath = path.join(__dirname, '..', 'reports', 'dashboard_data.json');
    const reportData = await fs.readJson(reportPath);
    
    // Transform to expected format for React dashboard
    const transformedReport = {
      schemaVersion: "1.0.0",
      project: {
        id: "demo-project",
        name: "Test Analysis Demo",
        languages: ["rust"],
        path: "test_with_issues.rs",
        commit: "demo",
        branch: "demo"
      },
      summary: {
        timeGenerated: reportData.metadata.timestamp,
        coverage: 85.0,
        issuesTotal: reportData.issues.length,
        filesAnalyzed: 1,
        componentsAnalyzed: 1,
        analysisDurationMs: reportData.metadata.durationSeconds * 1000,
        issuesBySeverity: calculateIssuesBySeverity(reportData.issues),
        issuesByCategory: calculateIssuesByCategory(reportData.issues)
      },
      findings: reportData.issues.map((issue, index) => ({
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
        generatedAt: reportData.metadata.timestamp,
        analysisId: "demo-analysis"
      }
    };
    
    res.json({
      data: transformedReport,
      timestamp: new Date().toISOString(),
      schemaVersion: "1.0.0"
    });
  } catch (error) {
    console.error('Error loading demo report:', error);
    res.status(500).json({ error: 'Failed to load demo report' });
  }
});

app.get('/api/v1/reports/:id', async (req, res) => {
  const { id } = req.params;
  
  // For demo, just redirect to demo report
  if (id === 'demo' || id === '1') {
    return res.redirect('/api/v1/reports/demo');
  }
  
  res.status(404).json({ error: 'Report not found' });
});

app.get('/api/v1/reports/:id/graphs/dependency', (req, res) => {
  res.json({
    data: {
      nodes: [],
      edges: []
    }
  });
});

// Helper functions
function calculateIssuesBySeverity(issues) {
  const counts = { critical: 0, high: 0, medium: 0, low: 0 };
  issues.forEach(issue => {
    const severity = issue.severity.toLowerCase();
    if (counts.hasOwnProperty(severity)) {
      counts[severity]++;
    }
  });
  return counts;
}

function calculateIssuesByCategory(issues) {
  const counts = {};
  issues.forEach(issue => {
    const category = issue.antiPatternType.toLowerCase().replace(/\s+/g, '-');
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
app.listen(PORT, () => {
  console.log(`🚀 Uveddi Alpha API Server running on http://localhost:${PORT}`);
  console.log(`📚 Documentation API: http://localhost:${PORT}/api/docs/structure`);
  console.log(`🔍 Health check: http://localhost:${PORT}/health`);
});

module.exports = app;