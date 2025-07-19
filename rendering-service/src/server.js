const express = require('express');
const cors = require('cors');
const morgan = require('morgan');
const helmet = require('helmet');
const pino = require('pino');
const renderer = require('./renderer');
const workerPool = require('./worker-pool');

// Initialize structured logger
const logger = pino({
  level: process.env.LOG_LEVEL || 'info',
  formatters: {
    level: (label) => ({ level: label })
  },
  timestamp: () => `,"timestamp":"${new Date().toISOString()}"`
});

// Pass logger to renderer and worker pool
renderer.setLogger(logger);
workerPool.setLogger(logger);

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(helmet());
app.use(cors());
app.use(morgan('combined'));
app.use(express.json({ limit: '10mb' }));

// Health check endpoint with cache stats
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    workers: workerPool.getStatus(),
    cache: renderer.getCacheStats(),
    capabilities: renderer.getCapabilities()
  });
});

// Cache statistics endpoint for monitoring
app.get('/cache/stats', (req, res) => {
  res.json({
    timestamp: new Date().toISOString(),
    cache: renderer.getCacheStats()
  });
});

// Cache management endpoint (for maintenance)
app.delete('/cache', async (req, res) => {
  try {
    await renderer.clearCache();
    res.json({
      success: true,
      message: 'Cache cleared successfully',
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    logger.error({
      msg: 'Cache clear error',
      error: error.message,
      stack: error.stack,
      severity: error.severity || 'Error',
      category: error.category || 'CacheFailure'
    }, 'Failed to clear cache');
    res.status(500).json({
      error: 'Failed to clear cache',
      message: error.message,
      severity: error.severity || 'Error',
      category: error.category || 'CacheFailure'
    });
  }
});

// Main rendering endpoint
app.post('/render', async (req, res) => {
  try {
    const { mermaid_code, format = 'svg', width = 1200, height = 800, quality = 'auto' } = req.body;
    
    if (!mermaid_code) {
      return res.status(400).json({
        error: 'Missing required field: mermaid_code'
      });
    }

    if (!['svg', 'png'].includes(format.toLowerCase())) {
      return res.status(400).json({
        error: 'Invalid format. Supported formats: svg, png'
      });
    }

    const startTime = Date.now();
    
    const result = await renderer.renderDiagram({
      mermaidCode: mermaid_code,
      format: format.toLowerCase(),
      width: parseInt(width),
      height: parseInt(height),
      quality
    });

    const renderTime = Date.now() - startTime;
    
    // Set cache headers for Layer 2 caching (CDN/Browser)
    if (result.metadata && result.metadata.cache_status === 'hit') {
      // Cached content - set aggressive caching headers
      res.set({
        'Cache-Control': 'public, max-age=31536000, immutable',
        'ETag': result.metadata.cache_key,
        'X-Cache-Status': 'hit'
      });
    } else {
      // Fresh content - still cacheable but shorter duration
      res.set({
        'Cache-Control': 'public, max-age=86400', // 24 hours
        'ETag': result.metadata ? result.metadata.cache_key : undefined,
        'X-Cache-Status': 'miss'
      });
    }

    res.json({
      success: true,
      format: result.format,
      data: result.data,
      metadata: {
        render_time_ms: renderTime,
        size_bytes: result.data.length,
        dimensions: result.dimensions,
        cache_status: result.metadata ? result.metadata.cache_status : 'unknown',
        cache_key: result.metadata ? result.metadata.cache_key?.substring(0, 8) + '...' : undefined
      }
    });

  } catch (error) {
    logger.error({
      msg: 'Rendering error',
      error: error.message,
      stack: error.stack,
      severity: error.severity || 'Error',
      category: error.category || 'RenderingFailure'
    }, 'Rendering failed');
    res.status(500).json({
      error: 'Rendering failed',
      message: error.message,
      severity: error.severity || 'Error',
      category: error.category || 'RenderingFailure'
    });
  }
});

// Batch rendering endpoint for multiple diagrams
app.post('/render/batch', async (req, res) => {
  try {
    const { diagrams, format = 'svg' } = req.body;
    
    if (!Array.isArray(diagrams) || diagrams.length === 0) {
      return res.status(400).json({
        error: 'diagrams must be a non-empty array'
      });
    }

    if (diagrams.length > 10) {
      return res.status(400).json({
        error: 'Maximum 10 diagrams per batch request'
      });
    }

    const startTime = Date.now();
    const results = await Promise.all(
      diagrams.map(async (diagram, index) => {
        try {
          const result = await renderer.renderDiagram({
            mermaidCode: diagram.mermaid_code,
            format: format.toLowerCase(),
            width: diagram.width || 1200,
            height: diagram.height || 800
          });
          
          return {
            index,
            success: true,
            ...result
          };
        } catch (error) {
          return {
            index,
            success: false,
            error: error.message
          };
        }
      })
    );

    const totalTime = Date.now() - startTime;
    
    res.json({
      success: true,
      results,
      metadata: {
        total_render_time_ms: totalTime,
        diagram_count: diagrams.length,
        success_count: results.filter(r => r.success).length
      }
    });

  } catch (error) {
    logger.error({
      msg: 'Batch rendering error',
      error: error.message,
      stack: error.stack,
      severity: error.severity || 'Error',
      category: error.category || 'BatchRenderingFailure'
    }, 'Batch rendering failed');
    res.status(500).json({
      error: 'Batch rendering failed',
      message: error.message,
      severity: error.severity || 'Error',
      category: error.category || 'BatchRenderingFailure'
    });
  }
});

// Error handling middleware
app.use((err, req, res, next) => {
  logger.error({
    msg: 'Unhandled error',
    error: err.message,
    stack: err.stack,
    severity: err.severity || 'Critical',
    category: err.category || 'UnhandledError'
  }, 'Internal server error');
  res.status(500).json({
    error: 'Internal server error',
    message: err.message,
    severity: err.severity || 'Critical',
    category: err.category || 'UnhandledError'
  });
});

// Graceful shutdown
process.on('SIGTERM', async () => {
  logger.info('SIGTERM received, shutting down gracefully...');
  await workerPool.shutdown();
  process.exit(0);
});

process.on('SIGINT', async () => {
  logger.info('SIGINT received, shutting down gracefully...');
  await workerPool.shutdown();
  process.exit(0);
});

// Initialize worker pool and start server
async function startServer() {
  try {
    await workerPool.initialize();
    
    app.listen(PORT, () => {
    logger.info(`Uveddi Rendering Service listening on port ${PORT}`);
    logger.info(`Health check: http://localhost:${PORT}/health`);
    });
  } catch (error) {
    logger.fatal({
      msg: 'Failed to start server',
      error: error.message,
      stack: error.stack,
      severity: 'Critical',
      category: 'ServerStartupFailure'
    }, 'Server startup failed');
    process.exit(1);
  }
}

startServer();
