/**
 * Severity threshold utilities for large-scale analysis reports
 * Adjusts thresholds based on total issue count to provide meaningful severity assessment
 */

interface SeverityBreakdown {
  critical: number;
  high: number;
  medium: number;
  low: number;
}

interface ThresholdConfig {
  critical: { min: number; ratio: number };
  high: { min: number; ratio: number };
  medium: { min: number; ratio: number };
}

/**
 * Dynamic thresholds based on dataset size
 * Larger codebases naturally have more issues, so thresholds scale accordingly
 */
const getThresholdConfig = (totalIssues: number): ThresholdConfig => {
  if (totalIssues > 5000) {
    // Large codebase (>5k issues) - expect more issues, be more tolerant
    return {
      critical: { min: 100, ratio: 0.05 }, // 5% or min 100
      high: { min: 500, ratio: 0.25 },     // 25% or min 500
      medium: { min: 200, ratio: 0.15 },   // 15% or min 200
    };
  } else if (totalIssues > 1000) {
    // Medium codebase (1k-5k issues)
    return {
      critical: { min: 50, ratio: 0.08 },  // 8% or min 50
      high: { min: 200, ratio: 0.30 },     // 30% or min 200
      medium: { min: 100, ratio: 0.20 },   // 20% or min 100
    };
  } else {
    // Small codebase (<1k issues) - more strict thresholds
    return {
      critical: { min: 10, ratio: 0.10 },  // 10% or min 10
      high: { min: 50, ratio: 0.35 },      // 35% or min 50
      medium: { min: 30, ratio: 0.25 },    // 25% or min 30
    };
  }
};

/**
 * Calculate risk level based on severity breakdown and dataset size
 */
export const calculateRiskLevel = (severity: SeverityBreakdown, totalIssues: number): 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL' => {
  const thresholds = getThresholdConfig(totalIssues);

  const criticalThreshold = Math.max(thresholds.critical.min, totalIssues * thresholds.critical.ratio);
  const highThreshold = Math.max(thresholds.high.min, totalIssues * thresholds.high.ratio);

  if (severity.critical >= criticalThreshold) {
    return 'CRITICAL';
  } else if (severity.critical > criticalThreshold * 0.5 || severity.high >= highThreshold) {
    return 'HIGH';
  } else if (severity.high > highThreshold * 0.5 || severity.medium > totalIssues * 0.1) {
    return 'MEDIUM';
  } else {
    return 'LOW';
  }
};

/**
 * Get quality score based on issue distribution (0-100 scale)
 */
export const calculateQualityScore = (severity: SeverityBreakdown, totalIssues: number, filesAnalyzed: number): number => {
  if (totalIssues === 0) return 100;

  // Base score starts at 100
  let score = 100;

  // Issues per file - penalize high density
  const issuesPerFile = totalIssues / Math.max(filesAnalyzed, 1);

  // Severity-weighted penalties (scaled for large datasets)
  const criticalWeight = totalIssues > 5000 ? 0.8 : 1.0; // Be more lenient for large codebases
  const highWeight = totalIssues > 5000 ? 0.6 : 0.8;
  const mediumWeight = 0.4;
  const lowWeight = 0.1;

  // Calculate penalties based on severity
  const criticalPenalty = Math.min(40, (severity.critical / Math.max(filesAnalyzed, 1)) * criticalWeight * 10);
  const highPenalty = Math.min(30, (severity.high / Math.max(filesAnalyzed, 1)) * highWeight * 5);
  const mediumPenalty = Math.min(20, (severity.medium / Math.max(filesAnalyzed, 1)) * mediumWeight * 3);
  const lowPenalty = Math.min(10, (severity.low / Math.max(filesAnalyzed, 1)) * lowWeight * 1);

  score = score - criticalPenalty - highPenalty - mediumPenalty - lowPenalty;

  // Additional penalty for very high issue density
  if (issuesPerFile > 100) {
    score -= Math.min(20, (issuesPerFile - 100) / 10);
  }

  return Math.max(0, Math.round(score));
};

/**
 * Get severity color with better contrast for large numbers
 */
export const getSeverityColor = (severity: string, theme: any, alpha: number = 0.1): string => {
  switch (severity.toLowerCase()) {
    case 'critical':
      return theme.palette.mode === 'light'
        ? `rgba(211, 47, 47, ${alpha})` // Darker red for better visibility
        : `rgba(244, 67, 54, ${alpha})`;
    case 'high':
      return theme.palette.mode === 'light'
        ? `rgba(230, 81, 0, ${alpha})` // Strong orange
        : `rgba(255, 152, 0, ${alpha})`;
    case 'medium':
      return theme.palette.mode === 'light'
        ? `rgba(255, 193, 7, ${alpha})` // Amber
        : `rgba(255, 213, 79, ${alpha})`;
    case 'low':
      return theme.palette.mode === 'light'
        ? `rgba(56, 142, 60, ${alpha})` // Green
        : `rgba(76, 175, 80, ${alpha})`;
    default:
      return theme.palette.mode === 'light'
        ? `rgba(117, 117, 117, ${alpha})` // Gray
        : `rgba(158, 158, 158, ${alpha})`;
  }
};

/**
 * Format large numbers for better readability
 */
export const formatIssueCount = (count: number): string => {
  if (count >= 10000) {
    return `${(count / 1000).toFixed(1)}k`;
  } else if (count >= 1000) {
    return `${(count / 1000).toFixed(0)}k`;
  }
  return count.toString();
};

/**
 * Get severity level description appropriate for the dataset size
 */
export const getSeverityLevelDescription = (level: string, totalIssues: number): string => {
  const scale = totalIssues > 5000 ? 'large-scale' : totalIssues > 1000 ? 'medium-scale' : 'small-scale';

  switch (level.toLowerCase()) {
    case 'critical':
      return `Critical issues require immediate attention across this ${scale} codebase`;
    case 'high':
      return `Significant issues present - consider prioritizing fixes for this ${scale} project`;
    case 'medium':
      return `Moderate issue levels - typical for a ${scale} codebase, plan systematic improvements`;
    case 'low':
      return `Low issue density - good code quality for a ${scale} project`;
    default:
      return `Issue assessment for ${scale} codebase`;
  }
};