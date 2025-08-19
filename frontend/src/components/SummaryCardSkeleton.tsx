import { Box, Paper, Skeleton } from '@mui/material';
import { useTheme } from '@mui/material/styles';

interface SummaryCardSkeletonProps {
  color?: 'primary' | 'error' | 'info' | 'success';
}

function SummaryCardSkeleton({ color = 'primary' }: SummaryCardSkeletonProps) {
  const theme = useTheme();
  
  const getSkeletonColor = (color: string) => {
    const isDark = theme.palette.mode === 'dark';
    
    switch (color) {
      case 'primary':
        return isDark 
          ? 'rgba(100, 181, 246, 0.15)'
          : 'rgba(25, 118, 210, 0.12)';
      case 'error':
        return isDark 
          ? 'rgba(255, 204, 203, 0.15)'
          : 'rgba(255, 153, 153, 0.12)';
      case 'success':
        return isDark 
          ? 'rgba(129, 199, 132, 0.15)'
          : 'rgba(76, 175, 80, 0.12)';
      default: // info
        return isDark 
          ? 'rgba(79, 195, 247, 0.15)'
          : 'rgba(2, 136, 209, 0.12)';
    }
  };

  return (
    <Paper 
      className="uveddi-summary-card-skeleton"
      sx={{ 
        p: 4,
        textAlign: 'center',
        borderRadius: 3,
        position: 'relative',
        overflow: 'hidden',
        background: getSkeletonColor(color),
        boxShadow: (theme) => theme.palette.mode === 'light'
          ? '0 4px 20px rgba(0, 0, 0, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.4)'
          : '0 4px 20px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.1)',
        animation: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        '@keyframes pulse': {
          '0%, 100%': {
            opacity: 1,
          },
          '50%': {
            opacity: 0.7,
          },
        },
      }}
    >
      {/* Background Icon Skeleton */}
      <Box
        sx={{
          position: 'absolute',
          top: 20,
          right: 20,
          opacity: 0.08,
        }}
      >
        <Skeleton variant="circular" width={60} height={60} />
      </Box>
      
      {/* Title Skeleton */}
      <Skeleton 
        variant="text" 
        sx={{ 
          fontSize: '1rem',
          mb: 3,
          width: '70%',
          mx: 'auto',
        }}
      />
      
      {/* Value Skeleton */}
      <Skeleton 
        variant="text" 
        sx={{ 
          fontSize: '2.5rem',
          mb: 2,
          width: '50%',
          mx: 'auto',
          height: 60,
        }}
      />
      
      {/* Subtitle Skeleton */}
      <Skeleton 
        variant="text" 
        sx={{ 
          fontSize: '0.9rem',
          width: '60%',
          mx: 'auto',
        }}
      />
    </Paper>
  );
}

export default SummaryCardSkeleton;
