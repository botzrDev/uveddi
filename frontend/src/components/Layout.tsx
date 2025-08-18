import type { ThemeMode } from '@/utils/theme';
import {
    AssessmentOutlined,
    DarkModeOutlined,
    HomeOutlined,
    LightModeOutlined,
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
        sx={{ 
          backgroundColor: 'var(--uveddi-primary-600)',
          borderBottom: '1px solid var(--uveddi-primary-700)',
        }}
      >
        <Toolbar sx={{ minHeight: '72px' }}>
          {/* Logo/Brand */}
          <Box sx={{ display: 'flex', alignItems: 'center', flexGrow: 1 }}>
            <Box
              component="img"
              src="/logo.png"
              alt="Uveddi Logo"
              sx={{
                height: 32,
                width: 32,
                mr: 2,
                cursor: 'pointer',
                filter: 'brightness(0) invert(1)', // Make logo white
                '&:hover': { opacity: 0.8 },
              }}
              onClick={handleHomeClick}
            />
            <Box>
              <Typography
                variant="h6"
                component="div"
                sx={{
                  fontWeight: 700,
                  fontFamily: "'JetBrains Mono', monospace",
                  cursor: 'pointer',
                  fontSize: '1.375rem',
                  letterSpacing: '-0.025em',
                  '&:hover': { opacity: 0.8 },
                }}
                onClick={handleHomeClick}
              >
                Uveddi
              </Typography>
              <Typography
                variant="caption"
                sx={{
                  color: 'rgba(255, 255, 255, 0.7)',
                  fontSize: '0.75rem',
                  fontWeight: 500,
                  lineHeight: 1,
                  mt: -0.5,
                  display: 'block',
                }}
              >
                Code Analysis Platform
              </Typography>
            </Box>
            
            {/* Version Badge */}
            <Chip
              label="Alpha v0.9.0"
              size="small"
              sx={{
                ml: 2,
                backgroundColor: 'var(--uveddi-action-600)',
                color: 'white',
                fontWeight: 600,
                fontSize: '0.7rem',
                height: '22px',
                '&:hover': {
                  backgroundColor: 'var(--uveddi-action-700)',
                },
              }}
            />
          </Box>

          {/* Navigation */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Tooltip title="Dashboard" arrow>
              <IconButton
                color="inherit"
                onClick={handleHomeClick}
                sx={{
                  backgroundColor: location.pathname.includes('/dashboard') 
                    ? 'var(--uveddi-primary-700)' 
                    : 'transparent',
                  borderRadius: 2,
                  '&:hover': {
                    backgroundColor: 'var(--uveddi-primary-700)',
                  },
                }}
              >
                <HomeOutlined />
              </IconButton>
            </Tooltip>

            <Tooltip title="All Reports" arrow>
              <IconButton
                color="inherit"
                onClick={handleReportsClick}
                sx={{
                  backgroundColor: location.pathname === '/reports' 
                    ? 'var(--uveddi-primary-700)' 
                    : 'transparent',
                  borderRadius: 2,
                  '&:hover': {
                    backgroundColor: 'var(--uveddi-primary-700)',
                  },
                }}
              >
                <AssessmentOutlined />
              </IconButton>
            </Tooltip>

            <Box sx={{ width: 1, height: 32, backgroundColor: 'rgba(255, 255, 255, 0.2)', mx: 1 }} />

            <Tooltip title={`Switch to ${themeMode === 'light' ? 'dark' : 'light'} mode`} arrow>
              <IconButton 
                color="inherit" 
                onClick={onToggleTheme}
                sx={{
                  borderRadius: 2,
                  '&:hover': {
                    backgroundColor: 'var(--uveddi-primary-700)',
                  },
                }}
              >
                {themeMode === 'light' ? <DarkModeOutlined /> : <LightModeOutlined />}
              </IconButton>
            </Tooltip>
          </Box>
        </Toolbar>
      </AppBar>

      {/* Main Content */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          backgroundColor: 'var(--uveddi-bg-primary)',
          minHeight: 'calc(100vh - 72px)', // Account for updated AppBar height
        }}
      >
        <Container maxWidth="xl" sx={{ py: 4 }}>
          {children}
        </Container>
      </Box>
    </Box>
  );
}

export default Layout;