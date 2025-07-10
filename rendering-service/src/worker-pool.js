const { chromium } = require('playwright');
const { bufferPool } = require('./memory_optimizer');

class WorkerPool {
  constructor() {
    this.logger = console;
    this.workers = [];
    this.maxWorkers = process.env.MAX_WORKERS || 3;
    this.currentWorker = 0;
    this.browser = null;
  }

  setLogger(newLogger) {
    this.logger = newLogger;
  }

  async initialize() {
    this.logger.info('Initializing browser worker pool...');
    
    try {
    
    // Launch browser with optimized settings
    this.browser = await chromium.launch({
      headless: true,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
        '--no-first-run',
        '--no-default-browser-check',
        '--disable-default-apps',
        '--disable-extensions',
        '--disable-background-timer-throttling',
        '--disable-renderer-backgrounding',
        '--disable-backgrounding-occluded-windows'
      ]
    });

    // Create worker contexts
    for (let i = 0; i < this.maxWorkers; i++) {
      const context = await this.browser.newContext({
        viewport: { width: 1200, height: 800 },
        deviceScaleFactor: 2
      });
      
      const page = await context.newPage();
      
      // Set up Mermaid.js environment
      await this.setupMermaidEnvironment(page);
      
      this.workers.push({
        id: i,
        context,
        page,
        busy: false,
        requestCount: 0
      });
    }

    this.logger.info({
      msg: 'Worker pool initialized',
      workerCount: this.maxWorkers
    }, `Browser worker pool initialized with ${this.maxWorkers} workers`);
    } catch (error) {
      this.logger.fatal({
        msg: 'Worker pool initialization failed',
        error: error.message,
        stack: error.stack,
        severity: 'Critical',
        category: 'WorkerPoolInitializationFailure'
      }, 'Failed to initialize worker pool');
      throw error;
    }
  }

  async setupMermaidEnvironment(page) {
    // Create HTML template with Mermaid.js
    await page.setContent(`
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="utf-8">
        <script src="https://cdn.jsdelivr.net/npm/mermaid@10.6.1/dist/mermaid.min.js"></script>
        <style>
          body { 
            margin: 0; 
            padding: 20px; 
            font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
            background: white;
          }
          #diagram { 
            display: flex; 
            justify-content: center; 
            align-items: center; 
            min-height: 100vh;
          }
          .mermaid {
            background: white;
          }
        </style>
      </head>
      <body>
        <div id="diagram"></div>
        <script>
          mermaid.initialize({ 
            startOnLoad: false,
            theme: 'default',
            securityLevel: 'loose',
            fontFamily: 'Helvetica Neue, Helvetica, Arial, sans-serif',
            fontSize: 16,
            flowchart: {
              curve: 'basis',
              padding: 20
            },
            sequence: {
              diagramMarginX: 50,
              diagramMarginY: 10,
              actorMargin: 50,
              width: 150,
              height: 65,
              boxMargin: 10,
              boxTextMargin: 5,
              noteMargin: 10,
              messageMargin: 35
            }
          });
          
          window.renderMermaid = async function(code) {
            const element = document.getElementById('diagram');
            element.innerHTML = '';
            
            try {
              const { svg } = await mermaid.render('diagram-svg', code);
              element.innerHTML = svg;
              return true;
            } catch (error) {
              console.error('Mermaid rendering error:', error);
              element.innerHTML = '<div style="color: red; font-family: monospace;">Diagram rendering failed: ' + error.message + '</div>';
              return false;
            }
          };
        </script>
      </body>
      </html>
    `);
  }

  async getWorker() {
    // Simple round-robin selection
    let attempts = 0;
    while (attempts < this.maxWorkers * 2) {
      // UV-8: Optionally pre-allocate rendering buffers from pool for hot path
      const worker = this.workers[this.currentWorker];
      if (!worker.buffer) {
        worker.buffer = bufferPool.acquire();
      }
      this.currentWorker = (this.currentWorker + 1) % this.maxWorkers;
      
      if (!worker.busy) {
        worker.busy = true;
        worker.requestCount++;
        return worker;
      }
      
      attempts++;
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    
    const error = new Error('No available workers');
    this.logger.error({
      msg: 'Worker pool exhausted',
      error: error.message,
      severity: 'Error',
      category: 'WorkerPoolExhausted'
    }, 'No available workers');
    throw error;
  }

  releaseWorker(worker) {
    // UV-8: Release rendering buffer back to pool
    if (worker.buffer) {
      bufferPool.release(worker.buffer);
      worker.buffer = null;
    }
    worker.busy = false;
  }

  getStatus() {
    return {
      total_workers: this.maxWorkers,
      busy_workers: this.workers.filter(w => w.busy).length,
      available_workers: this.workers.filter(w => !w.busy).length,
      total_requests: this.workers.reduce((sum, w) => sum + w.requestCount, 0)
    };
  }

  async shutdown() {
    this.logger.info('Shutting down browser worker pool...');
    
    try {
      if (this.browser) {
        await this.browser.close();
      }
      
      this.workers = [];
      this.logger.info('Browser worker pool shut down');
    } catch (error) {
      this.logger.error({
        msg: 'Worker pool shutdown error',
        error: error.message,
        stack: error.stack,
        severity: 'Error',
        category: 'WorkerPoolShutdownFailure'
      }, 'Failed to shutdown worker pool');
      throw error;
    }
  }
}

const workerPool = new WorkerPool();

// Export singleton instance
module.exports = workerPool;
