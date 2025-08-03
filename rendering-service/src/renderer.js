const workerPool = require('./worker-pool');
const AdvancedCache = require('./cache');
const { getQualityMode, getMermaidConfig, getViewportConfig, autoSelectQualityMode } = require('./quality-modes');

// Logger will be passed in from server.js
let logger = console;
function setLogger(newLogger) {
  logger = newLogger;
}

// Initialize advanced cache
const cache = new AdvancedCache({
  cacheDir: process.env.CACHE_DIR || '/tmp/uveddi-cache',
  maxCacheSize: parseInt(process.env.MAX_CACHE_SIZE) || 1024 * 1024 * 1024, // 1GB
  maxAge: parseInt(process.env.CACHE_MAX_AGE) || 7 * 24 * 60 * 60 * 1000 // 7 days
});

/**
 * Renders a Mermaid diagram to SVG or PNG format with content-addressable caching
 * @param {Object} options - Rendering options
 * @param {string} options.mermaidCode - The Mermaid.js code to render
 * @param {string} options.format - Output format ('svg' or 'png')
 * @param {number} options.width - Viewport width
 * @param {number} options.height - Viewport height
 * @returns {Object} Rendered diagram data and metadata
 */
async function renderDiagram({ mermaidCode, format = 'svg', width = 1200, height = 800, quality = 'auto' }) {
  const startTime = Date.now();
  
  // Generate content-addressable cache key
  const cacheKey = cache.generateCacheKey(mermaidCode, format, width, height);
  
  // Check cache first (Layer 1: Hot Cache)
  const cachedResult = await cache.get(cacheKey);
  if (cachedResult) {
    logger.info({
      msg: 'Cache hit',
      cacheKey: cacheKey.substring(0, 8),
      durationMs: Date.now() - startTime
    }, `Cache HIT for key: ${cacheKey.substring(0, 8)}...`);
    return {
      ...cachedResult,
      metadata: {
        ...cachedResult.metadata,
        render_time_ms: Date.now() - startTime,
        cache_status: 'hit'
      }
    };
  }
  
  logger.info({
    msg: 'Cache miss',
    cacheKey: cacheKey.substring(0, 8)
  }, `Cache MISS for key: ${cacheKey.substring(0, 8)}... - rendering...`);
  
  // Cache miss - perform actual rendering
  // Determine quality mode
  const qualityMode = quality === 'auto' ? autoSelectQualityMode(mermaidCode) : quality;
  const qualityConfig = getQualityMode(qualityMode);
  const viewportConfig = getViewportConfig(qualityMode, width, height);
  
  const worker = await workerPool.getWorker();
  
  try {
    // Set viewport size with quality-aware configuration
    await worker.page.setViewportSize(viewportConfig);
    
    // Update Mermaid configuration for this quality mode
    const mermaidConfig = getMermaidConfig(qualityMode);
    await worker.page.evaluate((config) => {
      mermaid.initialize(config);
    }, mermaidConfig);
    
    // Render the Mermaid diagram with unique ID to prevent CSS conflicts
    const diagramId = `diagram-${cacheKey.substring(0, 12)}`;
    const success = await worker.page.evaluate(async ({code, id}) => {
      return await window.renderMermaid(code, id);
    }, {code: mermaidCode, id: diagramId});
    
    if (!success) {
      throw new Error('Mermaid diagram compilation failed');
    }
    
    // Wait for rendering to complete (optimized timeout)
    const waitTime = qualityConfig.complexity === 'low' ? 25 : 50;
    await worker.page.waitForTimeout(waitTime);
    
    let data;
    let actualDimensions;
    
    if (format === 'svg') {
      // Get SVG content directly
      const svgElement = await worker.page.$('svg');
      if (!svgElement) {
        throw new Error('No SVG element found after rendering');
      }
      
      data = await worker.page.evaluate(() => {
        const svg = document.querySelector('svg');
        let svgContent = svg ? svg.outerHTML : null;

        // Regex to find Font Awesome-like codes (e.g., \f542)
        // These are often unrendered icons when the font is not available.
        // We remove the entire <text> element containing these codes.
        if (svgContent) {
          const regex = /<text[^>]*>\\f[0-9a-fA-F]{3,4}<\/text>/g;
          svgContent = svgContent.replace(regex, '');
        }
        return svgContent;
      });
      
      if (!data) {
        throw new Error('Failed to extract SVG content');
      }
      
      // Get actual SVG dimensions
      actualDimensions = await worker.page.evaluate(() => {
        const svg = document.querySelector('svg');
        if (!svg) return null;
        
        const viewBox = svg.getAttribute('viewBox');
        if (viewBox) {
          const [x, y, w, h] = viewBox.split(' ').map(Number);
          return { width: w, height: h };
        }
        
        return {
          width: parseInt(svg.getAttribute('width')) || 800,
          height: parseInt(svg.getAttribute('height')) || 600
        };
      });
      
    } else if (format === 'png') {
      // Get PNG screenshot
      const diagramElement = await worker.page.$('#diagram');
      if (!diagramElement) {
        throw new Error('No diagram element found for PNG rendering');
      }
      
      const buffer = await diagramElement.screenshot({
        type: 'png',
        omitBackground: false
      });
      
      data = buffer.toString('base64');
      
      // Get element dimensions
      actualDimensions = await diagramElement.boundingBox();
    } else {
      throw new Error(`Unsupported format: ${format}`);
    }
    
    const result = {
      format,
      data,
      dimensions: actualDimensions || { width, height },
      metadata: {
        render_time_ms: Date.now() - startTime,
        cache_status: 'miss',
        cache_key: cacheKey
      }
    };
    
    // Store in cache asynchronously (don't wait for completion)
    cache.set(cacheKey, result).catch(error => {
      logger.error({
        msg: 'Cache storage error',
        error: error.message,
        stack: error.stack,
        severity: 'Warning',
        category: 'CacheStorage'
      }, 'Failed to store in cache');
    });
    
    logger.info({
      msg: 'Rendering completed',
      cacheKey: cacheKey.substring(0, 8),
      renderTimeMs: result.metadata.render_time_ms,
      format: result.format,
      dimensions: result.dimensions
    }, `Rendered and cached key: ${cacheKey.substring(0, 8)}...`);
    
    return result;
    
  } finally {
    workerPool.releaseWorker(worker);
  }
}

/**
 * Validates Mermaid syntax without rendering
 * @param {string} mermaidCode - The Mermaid.js code to validate
 * @returns {Object} Validation result
 */
async function validateDiagram(mermaidCode) {
  const worker = await workerPool.getWorker();
  
  try {
    const result = await worker.page.evaluate(async (code) => {
      try {
        await mermaid.parse(code);
        return { valid: true };
      } catch (error) {
        return { 
          valid: false, 
          error: error.message 
        };
      }
    }, mermaidCode);
    
    return result;
    
  } finally {
    workerPool.releaseWorker(worker);
  }
}

/**
 * UV-8: Expose predictive cache warming for repeated renders
 * @param {Array<string>} diagramKeys - List of cache keys to warm
 */
async function warmCache(diagramKeys) {
  return cache.predictiveCacheWarm(diagramKeys);
}

/**
 * Get cache statistics for monitoring
 * @returns {Object} Cache performance metrics
 */
function getCacheStats() {
  return cache.getStats();
}

/**
 * Clear cache (for maintenance/testing)
 * @returns {Promise<void>}
 */
async function clearCache() {
  return cache.clear();
}

/**
 * Get supported diagram types and features
 * @returns {Object} Capabilities information
 */
function getCapabilities() {
  return {
    supported_formats: ['svg', 'png'],
    supported_diagram_types: [
      'flowchart',
      'sequence',
      'class',
      'state',
      'gantt',
      'pie',
      'journey',
      'git',
      'c4',
      'mindmap',
      'timeline',
      'architecture'
    ],
    max_viewport_width: 4096,
    max_viewport_height: 4096,
    max_diagram_complexity: 1000,
    version: '1.0.0',
    caching: {
      enabled: true,
      strategy: 'content-addressable',
      hash_algorithm: 'sha256'
    }
  };
}

module.exports = {
  renderDiagram,
  validateDiagram,
  getCapabilities,
  getCacheStats,
  clearCache,
  setLogger,
  warmCache // UV-8: Export cache warming
};
