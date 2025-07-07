const workerPool = require('./worker-pool');

/**
 * Renders a Mermaid diagram to SVG or PNG format
 * @param {Object} options - Rendering options
 * @param {string} options.mermaidCode - The Mermaid.js code to render
 * @param {string} options.format - Output format ('svg' or 'png')
 * @param {number} options.width - Viewport width
 * @param {number} options.height - Viewport height
 * @returns {Object} Rendered diagram data and metadata
 */
async function renderDiagram({ mermaidCode, format = 'svg', width = 1200, height = 800 }) {
  const worker = await workerPool.getWorker();
  
  try {
    // Set viewport size
    await worker.page.setViewportSize({ width, height });
    
    // Render the Mermaid diagram
    const success = await worker.page.evaluate(async (code) => {
      return await window.renderMermaid(code);
    }, mermaidCode);
    
    if (!success) {
      throw new Error('Mermaid diagram compilation failed');
    }
    
    // Wait for rendering to complete
    await worker.page.waitForTimeout(100);
    
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
        return svg ? svg.outerHTML : null;
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
    
    return {
      format,
      data,
      dimensions: actualDimensions || { width, height }
    };
    
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
    version: '1.0.0'
  };
}

module.exports = {
  renderDiagram,
  validateDiagram,
  getCapabilities
};
