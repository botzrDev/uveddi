// Material-UI theme configuration for Uveddi Interactive Reports
import { createTheme, ThemeOptions } from '@mui/material/styles';

const commonTheme: ThemeOptions = {
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    h1: {
      fontSize: '2.5rem',
      fontWeight: 600,
      lineHeight: 1.2,
    },
    h2: {
      fontSize: '2rem',
      fontWeight: 600,
      lineHeight: 1.3,
    },
    h3: {
      fontSize: '1.5rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    h4: {
      fontSize: '1.25rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    h5: {
      fontSize: '1.125rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    h6: {
      fontSize: '1rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    body1: {
      fontSize: '1rem',
      lineHeight: 1.6,
    },
    body2: {
      fontSize: '0.875rem',
      lineHeight: 1.6,
    },
    button: {
      textTransform: 'none',
      fontWeight: 500,
    },
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          textTransform: 'none',
          fontWeight: 500,
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: 12,
          boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)',
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          borderRadius: 16,
        },
      },
    },
  },
};

export const lightTheme = createTheme({
  ...commonTheme,
  palette: {
    mode: 'light',
    primary: {
      main: '#1976d2',
      light: '#42a5f5',
      dark: '#1565c0',
    },
    secondary: {
      main: '#9c27b0',
      light: '#ba68c8',
      dark: '#7b1fa2',
    },
    error: {
      main: '#d32f2f',
      light: '#ef5350',
      dark: '#c62828',
    },
    warning: {
      main: '#ed6c02',
      light: '#ff9800',
      dark: '#e65100',
    },
    info: {
      main: '#0288d1',
      light: '#03a9f4',
      dark: '#01579b',
    },
    success: {
      main: '#2e7d32',
      light: '#4caf50',
      dark: '#1b5e20',
    },
    background: {
      default: '#fafafa',
      paper: '#ffffff',
    },
    text: {
      primary: 'rgba(0, 0, 0, 0.87)',
      secondary: 'rgba(0, 0, 0, 0.6)',
    },
  },
});

export const darkTheme = createTheme({
  ...commonTheme,
  palette: {
    mode: 'dark',
    primary: {
      main: '#64b5f6',
      light: '#90caf9',
      dark: '#1976d2',
    },
    secondary: {
      main: '#ba68c8',
      light: '#ce93d8',
      dark: '#8e24aa',
    },
    error: {
      main: '#ef5350',
      light: '#e57373',
      dark: '#c62828',
    },
    warning: {
      main: '#ffb74d',
      light: '#ffd54f',
      dark: '#f57c00',
    },
    info: {
      main: '#4fc3f7',
      light: '#81d4fa',
      dark: '#0288d1',
    },
    success: {
      main: '#81c784',
      light: '#a5d6a7',
      dark: '#388e3c',
    },
    background: {
      default: '#0a0e1a',
      paper: '#1a1f2e',
    },
    surface: '#242b3d',
    text: {
      primary: '#e4e6ea',
      secondary: 'rgba(228, 230, 234, 0.7)',
    },
    divider: 'rgba(255, 255, 255, 0.12)',
    action: {
      active: 'rgba(255, 255, 255, 0.56)',
      hover: 'rgba(255, 255, 255, 0.08)',
      selected: 'rgba(255, 255, 255, 0.12)',
      disabled: 'rgba(255, 255, 255, 0.26)',
      disabledBackground: 'rgba(255, 255, 255, 0.12)',
    },
  },
});

// Severity-specific colors
export const severityColors = {
  critical: {
    light: '#ffebee',
    main: '#f44336',
    dark: '#c62828',
    contrastText: '#ffffff',
  },
  high: {
    light: '#fff3e0',
    main: '#ff9800',
    dark: '#f57c00',
    contrastText: '#000000',
  },
  medium: {
    light: '#fff8e1',
    main: '#ffc107',
    dark: '#f9a825',
    contrastText: '#000000',
  },
  low: {
    light: '#f1f8e9',
    main: '#8bc34a',
    dark: '#689f38',
    contrastText: '#000000',
  },
};

// Chart color palettes
export const chartColors = {
  primary: ['#1976d2', '#42a5f5', '#64b5f6', '#90caf9', '#bbdefb'],
  severity: [
    severityColors.critical.main,
    severityColors.high.main,
    severityColors.medium.main,
    severityColors.low.main,
  ],
  categorical: [
    '#1976d2', // Blue
    '#388e3c', // Green  
    '#f57c00', // Orange
    '#7b1fa2', // Purple
    '#d32f2f', // Red
    '#0288d1', // Light Blue
    '#689f38', // Light Green
    '#f9a825', // Amber
  ],
};

export type ThemeMode = 'light' | 'dark';

export function getTheme(mode: ThemeMode) {
  return mode === 'light' ? lightTheme : darkTheme;
}