import React from 'react';
import { Box, CircularProgress, Typography, keyframes } from '@mui/material';
import { BugReportOutlined } from '@mui/icons-material';

interface UveddiLoaderProps {
  message?: string;
  size?: 'small' | 'medium' | 'large';
  showIcon?: boolean;
}

// Custom pulse animation for the icon
const pulseAnimation = keyframes`
  0% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.1);
    opacity: 1;
  }
  100% {
    transform: scale(1);
    opacity: 0.8;
  }
`;

// Glow animation for the progress circle
const glowAnimation = keyframes`
  0% {
    filter: drop-shadow(0 0 5px var(--uveddi-glow-primary));
  }
  50% {
    filter: drop-shadow(0 0 15px var(--uveddi-glow-primary));
  }
  100% {
    filter: drop-shadow(0 0 5px var(--uveddi-glow-primary));
  }
`;

export default function UveddiLoader({ 
  message = 'Loading...', 
  size = 'medium',
  showIcon = true 
}: UveddiLoaderProps) {
  const getSizeConfig = (size: string) => {
    switch (size) {
      case 'small':
        return { progressSize: 32, iconSize: 20, textVariant: 'body2' as const };
      case 'large':
        return { progressSize: 64, iconSize: 32, textVariant: 'h6' as const };
      default:
        return { progressSize: 48, iconSize: 24, textVariant: 'body1' as const };
    }
  };

  const { progressSize, iconSize, textVariant } = getSizeConfig(size);

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 2,
        p: 3,
        minHeight: size === 'large' ? '300px' : size === 'small' ? '100px' : '200px',
      }}
    >
      {/* Loading Icon with Animation */}
      {showIcon && (
        <Box
          sx={{
            position: 'relative',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          {/* Circular Progress with Glow */}
          <CircularProgress
            size={progressSize}
            thickness={3}
            sx={{
              color: 'var(--uveddi-primary-600)',
              animation: `${glowAnimation} 2s ease-in-out infinite`,
            }}
          />
          
          {/* Centered Icon */}
          <Box
            sx={{
              position: 'absolute',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              animation: `${pulseAnimation} 2s ease-in-out infinite`,
            }}
          >
            <BugReportOutlined
              sx={{
                fontSize: iconSize,
                color: 'var(--uveddi-primary-600)',
              }}
            />
          </Box>
        </Box>
      )}

      {/* Loading Message */}
      <Box sx={{ textAlign: 'center' }}>
        <Typography
          variant={textVariant}
          sx={{
            color: 'var(--uveddi-text-secondary)',
            fontWeight: 500,
            mb: 0.5,
          }}
        >
          {message}
        </Typography>
        
        {/* Animated dots */}
        <Typography
          variant="caption"
          sx={{
            color: 'var(--uveddi-text-muted)',
            fontFamily: "'JetBrains Mono', monospace",
            letterSpacing: '0.1em',
          }}
        >
          Analyzing code architecture
          <Box
            component="span"
            sx={{
              '&::after': {
                content: '"..."',
                animation: 'loading-dots 1.5s steps(4, end) infinite',
              },
              '@keyframes loading-dots': {
                '0%': { content: '""' },
                '25%': { content: '"."' },
                '50%': { content: '".."' },
                '75%': { content: '"..."' },
                '100%': { content: '""' },
              },
            }}
          />
        </Typography>
      </Box>

      {/* Brand Signature */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 1,
          mt: 1,
          opacity: 0.6,
        }}
      >
        <Box
          component="img"
          src="/logo.png"
          alt="Uveddi"
          sx={{
            height: 16,
            width: 16,
            filter: 'brightness(0) saturate(100%) invert(45%) sepia(8%) saturate(629%) hue-rotate(202deg) brightness(95%) contrast(86%)',
          }}
        />
        <Typography
          variant="caption"
          sx={{
            color: 'var(--uveddi-text-muted)',
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: '0.7rem',
            fontWeight: 600,
          }}
        >
          UVEDDI
        </Typography>
      </Box>
    </Box>
  );
}

// Alternative minimal loader for inline use
export function UveddiSpinner({ size = 24 }: { size?: number }) {
  return (
    <CircularProgress
      size={size}
      thickness={3}
      sx={{
        color: 'var(--uveddi-primary-600)',
        animation: `${glowAnimation} 2s ease-in-out infinite`,
      }}
    />
  );
}

// Loading overlay for full-screen loading
export function UveddiLoadingOverlay({ message }: { message?: string }) {
  return (
    <Box
      sx={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(248, 250, 252, 0.95)', // Secondary-50 with transparency
        backdropFilter: 'blur(4px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 9999,
      }}
    >
      <Box
        sx={{
          backgroundColor: 'white',
          borderRadius: 3,
          p: 4,
          boxShadow: '0 20px 25px -5px rgba(79, 70, 229, 0.1), 0 10px 10px -5px rgba(79, 70, 229, 0.04)',
          border: '1px solid var(--uveddi-border)',
        }}
      >
        <UveddiLoader message={message} size="large" />
      </Box>
    </Box>
  );
}