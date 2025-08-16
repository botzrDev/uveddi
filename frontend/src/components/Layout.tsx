import React, { ReactNode } from 'react';
import {
  AppBar,
  Toolbar,
  Typography,
  Box,
  IconButton,
  Container,
  Tooltip,
} from '@mui/material';
import {
  DarkModeOutlined,
  LightModeOutlined,
  HomeOutlined,
  AssessmentOutlined,
} from '@mui/icons-material';
import { useNavigate, useLocation } from 'react-router-dom';
import type { ThemeMode } from '@/utils/theme';

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
      <AppBar position="static" elevation={1}>
        <Toolbar>
          {/* Logo/Brand */}
          <Box sx={{ display: 'flex', alignItems: 'center', flexGrow: 1 }}>
            <Typography
              variant="h6"
              component="div"
              sx={{
                fontWeight: 600,
                cursor: 'pointer',
                '&:hover': { opacity: 0.8 },
              }}
              onClick={handleHomeClick}
            >
              Uveddi Reports
            </Typography>
          </Box>

          {/* Navigation */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Tooltip title="Dashboard">
              <IconButton
                color="inherit"
                onClick={handleHomeClick}
                sx={{
                  bgcolor: location.pathname.includes('/dashboard') 
                    ? 'rgba(255, 255, 255, 0.1)' 
                    : 'transparent',
                }}
              >
                <HomeOutlined />
              </IconButton>
            </Tooltip>

            <Tooltip title="All Reports">
              <IconButton
                color="inherit"
                onClick={handleReportsClick}
                sx={{
                  bgcolor: location.pathname === '/reports' 
                    ? 'rgba(255, 255, 255, 0.1)' 
                    : 'transparent',
                }}
              >
                <AssessmentOutlined />
              </IconButton>
            </Tooltip>

            <Tooltip title={`Switch to ${themeMode === 'light' ? 'dark' : 'light'} mode`}>
              <IconButton color="inherit" onClick={onToggleTheme}>
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
          bgcolor: 'background.default',
          minHeight: 'calc(100vh - 64px)', // Account for AppBar height
        }}
      >
        <Container maxWidth="xl" sx={{ py: 3 }}>
          {children}
        </Container>
      </Box>
    </Box>
  );
}

export default Layout;