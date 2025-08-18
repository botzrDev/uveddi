// Enhanced theme utilities for advanced dashboard components
import { createTheme, ThemeOptions, alpha } from '@mui/material/styles';
import { lightTheme, darkTheme } from './theme';

// Extended color palette for data visualizations
export const dashboardColors = {
  // Primary data visualization palette
  primary: [
    '#4f46e5', // Uveddi primary
    '#6366f1', // Lighter primary
    '#818cf8', // Light primary
    '#a5b4fc', // Very light primary
    '#c7d2fe', // Pale primary
  ],
  
  // Secondary palette for categories
  secondary: [
    '#475569', // Charcoal
    '#64748b', // Medium gray
    '#94a3b8', // Light gray
    '#cbd5e1', // Very light gray
    '#e2e8f0', // Pale gray
  ],
  
  // Success/Error gradients
  success: [
    '#16a34a', // Dark green
    '#22c55e', // Green
    '#4ade80', // Light green
    '#86efac', // Very light green
    '#bbf7d0', // Pale green
  ],
  
  error: [
    '#dc2626', // Dark red
    '#ef4444', // Red
    '#f87171', // Light red
    '#fca5a5', // Very light red
    '#fecaca', // Pale red
  ],
  
  warning: [
    '#d97706', // Dark amber
    '#f59e0b', // Amber
    '#fbbf24', // Light amber
    '#fcd34d', // Very light amber
    '#fde68a', // Pale amber
  ],
  
  // Quality score colors
  qualityScore: {
    excellent: '#22c55e', // 90-100
    good: '#84cc16',      // 75-89
    fair: '#eab308',      // 60-74
    poor: '#f97316',      // 40-59
    critical: '#ef4444',  // 0-39
  },
  
  // Severity colors (enhanced)
  severity: {
    critical: '#dc2626',
    high: '#ea580c',
    medium: '#d97706',
    low: '#16a34a',
  },
  
  // Technical debt colors
  debt: {
    high: '#dc2626',
    medium: '#f59e0b',
    low: '#22c55e',
    trend: {
      improving: '#22c55e',
      stable: '#6b7280',
      worsening: '#dc2626',
    },
  },
  
  // Category-based colors for consistent visualization
  categories: [
    '#4f46e5', // Architecture
    '#06b6d4', // Performance
    '#10b981', // Security
    '#f59e0b', // Quality
    '#8b5cf6', // Maintainability
    '#ef4444', // Bugs
    '#84cc16', // Tests
    '#f97316', // Documentation
  ],
};

// Chart-specific theme extensions
export const chartTheme = {
  fonts: {
    base: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
    mono: "'JetBrains Mono', 'Consolas', monospace",
  },
  
  // Chart.js theme configuration
  chartjs: {
    plugins: {
      legend: {
        labels: {
          usePointStyle: true,
          padding: 20,
          font: {
            family: "'Inter', sans-serif",
            size: 12,
            weight: '500',
          },
        },
      },
      tooltip: {
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        titleFont: {
          family: "'Inter', sans-serif",
          size: 13,
          weight: '600',
        },
        bodyFont: {
          family: "'Inter', sans-serif",
          size: 12,
        },
        cornerRadius: 8,
        padding: 12,
      },
    },
    scales: {
      x: {
        grid: {
          color: 'rgba(0, 0, 0, 0.05)',
          lineWidth: 1,
        },
        ticks: {
          font: {
            family: "'Inter', sans-serif",
            size: 11,
          },
        },
      },
      y: {
        grid: {
          color: 'rgba(0, 0, 0, 0.05)',
          lineWidth: 1,
        },
        ticks: {
          font: {
            family: "'Inter', sans-serif",
            size: 11,
          },
        },
      },
    },
  },
  
  // D3.js theme configuration
  d3: {
    node: {
      fill: '#4f46e5',
      stroke: '#1e293b',
      strokeWidth: 2,
      radius: 6,
    },
    edge: {
      stroke: '#64748b',
      strokeWidth: 1.5,
      opacity: 0.6,
    },
    text: {
      fill: '#1e293b',
      fontSize: '12px',
      fontFamily: "'Inter', sans-serif",
    },
  },
  
  // Cytoscape.js theme configuration
  cytoscape: {
    node: {
      'background-color': '#4f46e5',
      'border-color': '#1e293b',
      'border-width': 2,
      'label': 'data(label)',
      'font-family': "'Inter', sans-serif",
      'font-size': '12px',
      'text-valign': 'center',
      'text-halign': 'center',
      'color': '#1e293b',
    },
    edge: {
      'line-color': '#64748b',
      'target-arrow-color': '#64748b',
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier',
      'width': 2,
    },
  },
};

// Responsive breakpoints for dashboard components
export const breakpoints = {
  mobile: 320,
  tablet: 768,
  desktop: 1024,
  wide: 1440,
  ultrawide: 1920,
};

// Dashboard-specific Material-UI theme extensions
export const createDashboardTheme = (mode: 'light' | 'dark') => {
  const baseTheme = mode === 'light' ? lightTheme : darkTheme;
  
  return createTheme({
    ...baseTheme,
    
    // Enhanced component overrides for dashboard
    components: {
      ...baseTheme.components,
      
      // Dashboard cards
      MuiCard: {
        styleOverrides: {
          root: {
            borderRadius: 12,
            border: '1px solid var(--uveddi-border)',
            boxShadow: '0 2px 8px rgba(79, 70, 229, 0.08)',
            transition: 'box-shadow 0.2s ease-in-out, transform 0.2s ease-in-out',
            '&:hover': {
              boxShadow: '0 4px 16px rgba(79, 70, 229, 0.12)',
              transform: 'translateY(-2px)',
            },
          },
        },
      },
      
      // Dashboard buttons
      MuiButton: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            textTransform: 'none',
            fontWeight: 500,
            padding: '8px 16px',
          },
          contained: {
            backgroundColor: 'var(--uveddi-primary-600)',
            '&:hover': {
              backgroundColor: 'var(--uveddi-primary-700)',
            },
          },
          outlined: {
            borderColor: 'var(--uveddi-border)',
            '&:hover': {
              backgroundColor: 'var(--uveddi-primary-50)',
              borderColor: 'var(--uveddi-primary-600)',
            },
          },
        },
      },
      
      // Enhanced chips for tags and categories
      MuiChip: {
        styleOverrides: {
          root: {
            borderRadius: 16,
            fontWeight: 500,
            '&.MuiChip-colorPrimary': {
              backgroundColor: 'var(--uveddi-primary-100)',
              color: 'var(--uveddi-primary-800)',
            },
            '&.MuiChip-colorSecondary': {
              backgroundColor: 'var(--uveddi-secondary-100)',
              color: 'var(--uveddi-secondary-800)',
            },
          },
          small: {
            height: 24,
            fontSize: '0.75rem',
          },
        },
      },
      
      // Dashboard-specific tooltips
      MuiTooltip: {
        styleOverrides: {
          tooltip: {
            backgroundColor: 'rgba(0, 0, 0, 0.9)',
            color: '#ffffff',
            fontSize: '0.75rem',
            fontWeight: 500,
            borderRadius: 6,
            padding: '8px 12px',
            maxWidth: 300,
          },
        },
      },
      
      // Form controls for dashboard filters
      MuiTextField: {
        styleOverrides: {
          root: {
            '& .MuiOutlinedInput-root': {
              borderRadius: 8,
              '&:hover .MuiOutlinedInput-notchedOutline': {
                borderColor: 'var(--uveddi-primary-600)',
              },
              '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                borderColor: 'var(--uveddi-primary-600)',
                borderWidth: 2,
              },
            },
          },
        },
      },
      
      // Dashboard data tables
      MuiTableCell: {
        styleOverrides: {
          head: {
            backgroundColor: 'var(--uveddi-secondary-50)',
            color: 'var(--uveddi-text-primary)',
            fontWeight: 600,
            fontSize: '0.875rem',
          },
          body: {
            fontSize: '0.875rem',
          },
        },
      },
      
      // Progress indicators
      MuiLinearProgress: {
        styleOverrides: {
          root: {
            borderRadius: 4,
            height: 8,
            backgroundColor: 'var(--uveddi-secondary-200)',
          },
          bar: {
            borderRadius: 4,
          },
        },
      },
      
      MuiCircularProgress: {
        styleOverrides: {
          root: {
            color: 'var(--uveddi-primary-600)',
          },
        },
      },
    },
    
    // Dashboard-specific palette extensions
    palette: {
      ...baseTheme.palette,
      
      // Add custom dashboard colors to palette
      dashboard: {
        qualityScore: dashboardColors.qualityScore,
        severity: dashboardColors.severity,
        debt: dashboardColors.debt,
        categories: dashboardColors.categories,
      },
    },
  });
};

// Utility functions for color manipulation
export const colorUtils = {
  /**
   * Get color by quality score
   */
  getQualityColor: (score: number): string => {
    if (score >= 90) return dashboardColors.qualityScore.excellent;
    if (score >= 75) return dashboardColors.qualityScore.good;
    if (score >= 60) return dashboardColors.qualityScore.fair;
    if (score >= 40) return dashboardColors.qualityScore.poor;
    return dashboardColors.qualityScore.critical;
  },
  
  /**
   * Get color by severity
   */
  getSeverityColor: (severity: 'critical' | 'high' | 'medium' | 'low'): string => {
    return dashboardColors.severity[severity] || dashboardColors.severity.low;
  },
  
  /**
   * Get color by category index
   */
  getCategoryColor: (index: number): string => {
    return dashboardColors.categories[index % dashboardColors.categories.length];
  },
  
  /**
   * Get color by category name
   */
  getCategoryColorByName: (category: string): string => {
    const categoryMap: Record<string, string> = {
      architecture: dashboardColors.categories[0],
      performance: dashboardColors.categories[1],
      security: dashboardColors.categories[2],
      quality: dashboardColors.categories[3],
      maintainability: dashboardColors.categories[4],
      bugs: dashboardColors.categories[5],
      tests: dashboardColors.categories[6],
      documentation: dashboardColors.categories[7],
    };
    
    return categoryMap[category.toLowerCase()] || dashboardColors.categories[0];
  },
  
  /**
   * Generate color palette for data series
   */
  generatePalette: (count: number, type: 'primary' | 'categorical' = 'primary'): string[] => {
    const base = type === 'primary' ? dashboardColors.primary : dashboardColors.categories;
    const palette: string[] = [];
    
    for (let i = 0; i < count; i++) {
      palette.push(base[i % base.length]);
    }
    
    return palette;
  },
  
  /**
   * Create alpha variant of color
   */
  withAlpha: (color: string, alpha: number): string => {
    // Convert hex to rgba
    const hex = color.replace('#', '');
    const r = parseInt(hex.substr(0, 2), 16);
    const g = parseInt(hex.substr(2, 2), 16);
    const b = parseInt(hex.substr(4, 2), 16);
    
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  },
  
  /**
   * Get trend color
   */
  getTrendColor: (trend: 'improving' | 'stable' | 'worsening'): string => {
    return dashboardColors.debt.trend[trend];
  },
};

// Animation configurations for dashboard components
export const animations = {
  // Standard transition durations
  duration: {
    fast: 150,
    normal: 250,
    slow: 400,
  },
  
  // Easing functions
  easing: {
    standard: 'cubic-bezier(0.4, 0, 0.2, 1)',
    decelerated: 'cubic-bezier(0.0, 0, 0.2, 1)',
    accelerated: 'cubic-bezier(0.4, 0, 1, 1)',
  },
  
  // Chart animation configurations
  chart: {
    duration: 750,
    easing: 'easeInOutCubic',
  },
  
  // Loading animations
  loading: {
    skeleton: {
      animation: 'wave',
      backgroundColor: 'rgba(79, 70, 229, 0.1)',
    },
  },
};

// Typography scale for dashboard components
export const typography = {
  // Dashboard-specific typography
  dashboard: {
    hero: {
      fontSize: '2.5rem',
      fontWeight: 700,
      lineHeight: 1.2,
      color: 'var(--uveddi-text-primary)',
    },
    title: {
      fontSize: '1.5rem',
      fontWeight: 600,
      lineHeight: 1.3,
      color: 'var(--uveddi-text-primary)',
    },
    subtitle: {
      fontSize: '1.125rem',
      fontWeight: 500,
      lineHeight: 1.4,
      color: 'var(--uveddi-text-secondary)',
    },
    metric: {
      fontSize: '2rem',
      fontWeight: 700,
      lineHeight: 1,
      fontFamily: "'JetBrains Mono', monospace",
      color: 'var(--uveddi-primary-600)',
    },
    label: {
      fontSize: '0.875rem',
      fontWeight: 500,
      lineHeight: 1.4,
      color: 'var(--uveddi-text-secondary)',
    },
    caption: {
      fontSize: '0.75rem',
      fontWeight: 400,
      lineHeight: 1.4,
      color: 'var(--uveddi-text-muted)',
    },
  },
};

// Spacing system for consistent layouts
export const spacing = {
  // Standard spacing scale
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
  xxl: 48,
  
  // Component-specific spacing
  component: {
    padding: {
      small: 12,
      medium: 16,
      large: 24,
    },
    margin: {
      small: 8,
      medium: 16,
      large: 24,
    },
    gap: {
      small: 8,
      medium: 12,
      large: 16,
    },
  },
};

// Shadow system for depth
export const shadows = {
  subtle: '0 1px 3px rgba(79, 70, 229, 0.1)',
  medium: '0 4px 6px rgba(79, 70, 229, 0.1)',
  strong: '0 10px 15px rgba(79, 70, 229, 0.1)',
  glow: '0 0 20px rgba(79, 70, 229, 0.2)',
};

// Export the complete dashboard theme system
export const dashboardTheme = {
  colors: dashboardColors,
  chart: chartTheme,
  breakpoints,
  colorUtils,
  animations,
  typography,
  spacing,
  shadows,
  createTheme: createDashboardTheme,
};

export default dashboardTheme;