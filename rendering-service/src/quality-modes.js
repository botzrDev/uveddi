/**
 * Quality modes implementation for UV-12 performance optimization
 * Provides fast/balanced/high quality rendering profiles
 */

const QUALITY_MODES = {
  fast: {
    timeout: 5000,
    complexity: 'low',
    deviceScaleFactor: 1,
    width: 800,
    height: 600,
    mermaidTheme: 'base',
    flowchartCurve: 'linear',
    optimizations: {
      skipAnimations: true,
      reduceDetails: true,
      simplifyText: true
    }
  },
  balanced: {
    timeout: 15000,
    complexity: 'medium',
    deviceScaleFactor: 1.5,
    width: 1600, // Increased from 1200 for better diagram rendering
    height: 1200, // Increased from 800 for better diagram rendering
    mermaidTheme: 'default',
    flowchartCurve: 'basis',
    optimizations: {
      skipAnimations: false,
      reduceDetails: false,
      simplifyText: false
    }
  },
  high: {
    timeout: 30000,
    complexity: 'high',
    deviceScaleFactor: 2,
    width: 1600,
    height: 1200,
    mermaidTheme: 'default',
    flowchartCurve: 'cardinal',
    optimizations: {
      skipAnimations: false,
      reduceDetails: false,
      simplifyText: false,
      highDPI: true
    }
  }
};

/**
 * Get quality mode configuration
 * @param {string} mode - Quality mode (fast/balanced/high)
 * @returns {Object} Quality configuration
 */
function getQualityMode(mode = 'balanced') {
  const normalizedMode = mode.toLowerCase();
  if (!QUALITY_MODES[normalizedMode]) {
    throw new Error(`Invalid quality mode: ${mode}. Valid modes: fast, balanced, high`);
  }
  return QUALITY_MODES[normalizedMode];
}

/**
 * Apply quality mode to Mermaid configuration
 * @param {string} mode - Quality mode
 * @returns {Object} Mermaid configuration object
 */
function getMermaidConfig(mode = 'balanced') {
  const qualityConfig = getQualityMode(mode);
  
  return {
    startOnLoad: false,
    theme: qualityConfig.mermaidTheme,
    securityLevel: 'loose',
    fontFamily: 'Helvetica Neue, Helvetica, Arial, sans-serif',
    fontSize: qualityConfig.complexity === 'low' ? 14 : 16,
    flowchart: {
      curve: qualityConfig.flowchartCurve,
      padding: qualityConfig.complexity === 'low' ? 10 : 20,
      nodeSpacing: qualityConfig.complexity === 'low' ? 30 : 50,
      rankSpacing: qualityConfig.complexity === 'low' ? 30 : 50,
      useMaxWidth: false, // Disable width constraints
    },
    sequence: {
      diagramMarginX: qualityConfig.complexity === 'low' ? 30 : 50,
      diagramMarginY: qualityConfig.complexity === 'low' ? 5 : 10,
      actorMargin: qualityConfig.complexity === 'low' ? 30 : 50,
      width: qualityConfig.complexity === 'low' ? 120 : 150,
      height: qualityConfig.complexity === 'low' ? 50 : 65,
      boxMargin: qualityConfig.complexity === 'low' ? 5 : 10,
      boxTextMargin: qualityConfig.complexity === 'low' ? 3 : 5,
      noteMargin: qualityConfig.complexity === 'low' ? 5 : 10,
      messageMargin: qualityConfig.complexity === 'low' ? 25 : 35
    },
    gantt: {
      numberSectionStyles: qualityConfig.complexity === 'high' ? 4 : 2
    }
  };
}

/**
 * Get viewport configuration for quality mode
 * @param {string} mode - Quality mode
 * @param {number} customWidth - Custom width override
 * @param {number} customHeight - Custom height override
 * @returns {Object} Viewport configuration
 */
function getViewportConfig(mode = 'balanced', customWidth, customHeight) {
  const qualityConfig = getQualityMode(mode);
  
  return {
    width: customWidth || qualityConfig.width,
    height: customHeight || qualityConfig.height,
    deviceScaleFactor: qualityConfig.deviceScaleFactor
  };
}

/**
 * Estimate rendering complexity based on diagram content
 * @param {string} mermaidCode - Mermaid diagram code
 * @returns {string} Complexity level (low/medium/high)
 */
function estimateComplexity(mermaidCode) {
  const lines = mermaidCode.split('\n').length;
  const nodes = (mermaidCode.match(/\[.*?\]/g) || []).length;
  const connections = (mermaidCode.match(/-->|->|->/g) || []).length;
  
  const complexity = lines + nodes + connections;
  
  if (complexity < 10) return 'low';
  if (complexity < 50) return 'medium';
  return 'high';
}

/**
 * Auto-select quality mode based on diagram complexity
 * @param {string} mermaidCode - Mermaid diagram code
 * @returns {string} Recommended quality mode
 */
function autoSelectQualityMode(mermaidCode) {
  const complexity = estimateComplexity(mermaidCode);
  
  switch (complexity) {
    case 'low':
      return 'fast';
    case 'medium':
      return 'balanced';
    case 'high':
      return 'balanced'; // Use balanced for high complexity for reliability
    default:
      return 'balanced';
  }
}

module.exports = {
  QUALITY_MODES,
  getQualityMode,
  getMermaidConfig,
  getViewportConfig,
  estimateComplexity,
  autoSelectQualityMode
};