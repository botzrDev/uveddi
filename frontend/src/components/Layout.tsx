import type { ThemeMode } from '@/utils/theme';
import {
    DarkModeOutlined,
    LightModeOutlined
} from '@mui/icons-material';
import {
    AppBar,
    Box,
    Chip,
    Container,
    IconButton,
    Toolbar,
    Tooltip,
    Typography,
} from '@mui/material';
import { ReactNode } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

interface LayoutProps {
  children: ReactNode;
  themeMode: ThemeMode;
  onToggleTheme: () => void;
}

function Layout({ children, themeMode, onToggleTheme }: LayoutProps) {
  const navigate = useNavigate();
  const location = useLocation();

  const handleHomeClick = () => {
    navigate('/dashboard/demo');
  };

  const handleReportsClick = () => {
    navigate('/reports');
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <AppBar
        position="static"
        elevation={0}
        component="header"
        role="banner"
        sx={{
          backgroundColor: (theme) => theme.palette.mode === 'light' 
            ? '#1976d2 !important' // Force dark blue in light mode
            : '#0a1929 !important', // Force very dark in dark mode
          borderBottom: (theme) => theme.palette.mode === 'light'
            ? '1px solid #1565c0'
            : '1px solid rgba(111, 102, 255, 0.2)',
          '& .MuiToolbar-root': {
            backgroundColor: 'transparent !important',
          },
        }}
      >
        <Toolbar sx={{ minHeight: '72px' }}>
          {/* Logo/Brand */}
          <Box sx={{ display: 'flex', alignItems: 'center', flexGrow: 1 }}>
            <Box
              component="img"
              src="/logo.png"
              alt="Uveddi - Code Analysis Dashboard Logo"
              role="button"
              tabIndex={0}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  handleHomeClick();
                }
              }}
              onClick={handleHomeClick}
              sx={{
                height: 36,
                width: 36,
                mr: 2,
                cursor: 'pointer',
                objectFit: 'contain',
                '&:hover': { opacity: 0.9 },
                '&:focus': {
                  outline: '2px solid',
                  outlineColor: 'primary.main',
                  outlineOffset: '2px',
                  borderRadius: '4px'
                },
              }}
            />
            <Box>
              <Typography
                variant="h6"
                component="h1"
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    handleHomeClick();
                  }
                }}
                sx={{
                  fontWeight: 700,
                  fontFamily: "'JetBrains Mono', monospace",
                  cursor: 'pointer',
                  fontSize: '1.375rem',
                  letterSpacing: '-0.025em',
                  color: (theme) => theme.palette.mode === 'dark' ? '#ffffff' : '#1a237e',
                  '&:hover': { opacity: 0.8 },
                  '&:focus': {
                    outline: '2px solid',
                    outlineColor: 'primary.main',
                    outlineOffset: '2px',
                    borderRadius: '4px'
                  },
                }}
                onClick={handleHomeClick}
              >
                Uveddi
              </Typography>
              <Typography
                variant="caption"
                sx={{
                  color: (theme) => theme.palette.mode === 'dark' ? '#ffffff' : '#1a237e',
                  fontSize: '0.75rem',
                  fontWeight: 500,
                  lineHeight: 1,
                  mt: -0.5,
                  display: 'block',
                  opacity: 0.8,
                }}
              >
                Code Analysis
              </Typography>
            </Box>
            
            {/* Version Badge */}
            <Chip
              label="v1.0 Community Core"
              size="small"
              sx={{
                ml: 2,
                backgroundImage: 'var(--uveddi-badge-bg, linear-gradient(90deg,#ffb84d,#ffd66b))',
                backgroundClip: 'padding-box',
                color: 'var(--uveddi-badge-color, #1b1226)',
                fontWeight: 700,
                fontSize: '0.75rem',
                height: '24px',
                borderRadius: '12px',
                px: 1.5,
                boxShadow: '0 1px 0 rgba(0,0,0,0.35)',
                border: '1px solid rgba(255,255,255,0.06)'
              }}
            />
          </Box>

          {/* Navigation */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Tooltip title={`Switch to ${themeMode === 'light' ? 'dark' : 'light'} mode`} arrow>
              <IconButton
                onClick={onToggleTheme}
                aria-label={`Switch to ${themeMode === 'light' ? 'dark' : 'light'} theme mode`}
                sx={{
                  borderRadius: 2,
                  backgroundColor: 'transparent',
                  color: themeMode === 'light' ? 'rgba(0, 0, 0, 0.7)' : 'rgba(255, 255, 255, 0.9)',
                  '&:hover': {
                    backgroundColor: themeMode === 'light' 
                      ? 'rgba(0, 0, 0, 0.1)' 
                      : 'var(--uveddi-primary-700)',
                  },
                  '&:focus': {
                    outline: '2px solid',
                    outlineColor: 'primary.main',
                    outlineOffset: '2px'
                  }
                }}
              >
                {themeMode === 'light' ? 
                  <DarkModeOutlined aria-hidden="true" /> : 
                  <LightModeOutlined aria-hidden="true" />
                }
              </IconButton>
            </Tooltip>
          </Box>
        </Toolbar>
      </AppBar>

      {/* Main Content */}
      <Box
        component="main"
        role="main"
        aria-label="Dashboard content"
        sx={{
          flexGrow: 1,
          backgroundColor: 'var(--uveddi-bg-primary)',
          minHeight: 'calc(100vh - 72px)', // Account for updated AppBar height
        }}
      >
        <Container maxWidth="xl" sx={{ py: 4 }} id="main-content">
          {children}
        </Container>
      </Box>
    </Box>
  );
}

export default Layout;