const express = require('express');
const cors = require('cors');
const morgan = require('morgan');
const helmet = require('helmet');
const renderer = require('./renderer');
const workerPool = require('./worker-pool');

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(helmet());
app.use(cors());
app.use(morgan('combined'));
app.use(express.json({ limit: '10mb' }));

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    workers: workerPool.getStatus()
  });
});

// Main rendering endpoint
app.post('/render', async (req, res) => {
  try {
    const { mermaid_code, format = 'svg', width = 1200, height = 800 } = req.body;
    
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
      height: parseInt(height)
    });

    const renderTime = Date.now() - startTime;
    
    res.json({
      success: true,
      format: result.format,
      data: result.data,
      metadata: {
        render_time_ms: renderTime,
        size_bytes: result.data.length,
        dimensions: result.dimensions
      }
    });

  } catch (error) {
    console.error('Rendering error:', error);
    res.status(500).json({
      error: 'Rendering failed',
      message: error.message
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
    console.error('Batch rendering error:', error);
    res.status(500).json({
      error: 'Batch rendering failed',
      message: error.message
    });
  }
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error('Unhandled error:', err);
  res.status(500).json({
    error: 'Internal server error',
    message: err.message
  });
});

// Graceful shutdown
process.on('SIGTERM', async () => {
  console.log('SIGTERM received, shutting down gracefully...');
  await workerPool.shutdown();
  process.exit(0);
});

process.on('SIGINT', async () => {
  console.log('SIGINT received, shutting down gracefully...');
  await workerPool.shutdown();
  process.exit(0);
});

// Initialize worker pool and start server
async function startServer() {
  try {
    await workerPool.initialize();
    
    app.listen(PORT, () => {
      console.log(`Uveddi Rendering Service listening on port ${PORT}`);
      console.log(`Health check: http://localhost:${PORT}/health`);
    });
  } catch (error) {
    console.error('Failed to start server:', error);
    process.exit(1);
  }
}

startServer();
