/**
 * Data model conversion utilities
 * 
 * This module handles conversions between different data representations
 * to ensure consistency between CLI outputs and dashboard visualization.
 */

/**
 * Convert CLI analysis output to dashboard-compatible format
 */
function convertCliOutputToDashboardFormat(cliOutput) {
  try {
    // Base structure for dashboard format
    const dashboardFormat = {
      summary: {
        totalFiles: cliOutput.metadata?.files_analyzed || 0,
        totalIssues: cliOutput.metadata?.issues_found || 0,
        analysisTime: cliOutput.metadata?.analysis_duration_ms || 0,
        aiEnhanced: cliOutput.metadata?.ai_enhanced || false,
      },
      issues: [],
      categories: {},
      metrics: {
        quality: calculateQualityScore(cliOutput),
        complexity: calculateComplexityMetrics(cliOutput),
        maintainability: calculateMaintainabilityIndex(cliOutput),
        technicalDebt: estimateTechnicalDebt(cliOutput),
      }
    };
    
    // Convert issues format
    if (Array.isArray(cliOutput.issues)) {
      dashboardFormat.issues = cliOutput.issues.map(issue => ({
        id: issue.id || `issue-${Math.random().toString(36).substr(2, 9)}`,
        title: issue.title || issue.name || 'Unnamed Issue',
        description: issue.description || '',
        severity: mapSeverity(issue.severity),
        location: {
          file: issue.file_path || '',
          line: issue.line_number || 0,
          column: issue.column || 0,
        },
        category: issue.category || mapCategoryFromType(issue.type),
        type: issue.type || '',
        ai_explanation: issue.ai_explanation || '',
        code_snippet: issue.code_snippet || '',
        suggested_fix: issue.suggested_fix || '',
      }));
    }
    
    // Group issues by category for quick access
    dashboardFormat.issues.forEach(issue => {
      if (!dashboardFormat.categories[issue.category]) {
        dashboardFormat.categories[issue.category] = [];
      }
      dashboardFormat.categories[issue.category].push(issue);
    });
    
    return dashboardFormat;
  } catch (error) {
    console.error('Error converting CLI output to dashboard format:', error);
    throw new Error(`Data conversion failed: ${error.message}`);
  }
}

/**
 * Calculate a quality score from issues
 */
function calculateQualityScore(cliOutput) {
  // Start with a perfect score and subtract based on issues
  let baseScore = 100;
  
  if (!cliOutput.issues || !Array.isArray(cliOutput.issues)) {
    return baseScore;
  }
  
  // Count issues by severity
  const counts = {
    critical: 0,
    high: 0,
    medium: 0,
    low: 0,
    info: 0
  };
  
  cliOutput.issues.forEach(issue => {
    const severity = (issue.severity || '').toLowerCase();
    if (counts[severity] !== undefined) {
      counts[severity]++;
    } else {
      counts.medium++; // Default category if unknown
    }
  });
  
  // Subtract from base score according to severity
  baseScore -= counts.critical * 10;
  baseScore -= counts.high * 5;
  baseScore -= counts.medium * 2;
  baseScore -= counts.low * 0.5;
  
  // Ensure score is between 0 and 100
  return Math.max(0, Math.min(100, baseScore));
}

/**
 * Map issue severity to standardized format
 */
function mapSeverity(severity) {
  if (!severity) return 'medium';
  
  const s = severity.toLowerCase();
  if (s.includes('critical')) return 'critical';
  if (s.includes('high')) return 'high';
  if (s.includes('medium') || s.includes('moderate')) return 'medium';
  if (s.includes('low')) return 'low';
  if (s.includes('info')) return 'info';
  
  return 'medium';
}

/**
 * Map issue type to a category
 */
function mapCategoryFromType(type) {
  if (!type) return 'general';
  
  const t = type.toLowerCase();
  if (t.includes('dead_code')) return 'code-quality';
  if (t.includes('large_class')) return 'maintainability';
  if (t.includes('complexity')) return 'complexity';
  if (t.includes('security')) return 'security';
  if (t.includes('performance')) return 'performance';
  
  return 'general';
}

/**
 * Calculate complexity metrics
 */
function calculateComplexityMetrics(cliOutput) {
  // Default values
  return {
    overall: 0,
    cyclomatic: 0,
    cognitive: 0
    // Additional metrics could be added here
  };
}

/**
 * Calculate maintainability index
 */
function calculateMaintainabilityIndex(cliOutput) {
  // Default values
  return {
    overall: 0
    // Additional metrics could be added here
  };
}

/**
 * Estimate technical debt metrics
 */
function estimateTechnicalDebt(cliOutput) {
  // Default values
  return {
    hours: 0,
    storyPoints: 0
    // Additional metrics could be added here
  };
}

module.exports = {
  convertCliOutputToDashboardFormat
};
