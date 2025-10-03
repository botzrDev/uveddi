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
  origin: true, // Allow all origins in development (GitHub Codespaces, local, etc.)
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With']
}));
app.use(morgan('combined'));
app.use(express.json());

// Mount routes
app.use(routes);

// Routes
















// Catch-all for undefined routes
app.use('*', (req, res) => {
  res.status(404).json({ 
    error: 'Route not found',
    availableEndpoints: [
      'GET /health',
      'GET /api/v1/health',
      'GET /api/v1/metrics',
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