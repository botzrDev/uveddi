import FindingsList from '@/components/FindingsList';
import MermaidDiagram from '@/components/MermaidDiagram';
import UveddiLoader from '@/components/UveddiLoader';
import { useDemoReport, useReport } from '@/hooks/useReport';
import { apiService } from '@/services/api';
import {
  BugReportOutlined,
  DownloadOutlined,
  ShareOutlined,
  TrendingUpOutlined,
} from '@mui/icons-material';
import {
  Alert,
  Box,
  Button,
  Chip,
  Grid,
  Paper,
  Typography,
  useTheme,
} from '@mui/material';
import { useParams } from 'react-router-dom';

function DashboardPage() {
  const { reportId } = useParams<{ reportId: string }>();
  const theme = useTheme();
  
  // Use demo report for demo route, otherwise fetch by ID
  const isDemoReport = reportId === 'demo';
  const reportQuery = useReport(reportId || '');
  const demoQuery = useDemoReport();

  const handleExportReport = async () => {
    console.log('🚀 Export button clicked!', { isDemoReport, reportId });
    try {
      let blob: Blob;
      
      console.log('📊 About to call export service...');
      if (isDemoReport) {
        console.log('📋 Calling exportDemoReport...');
        blob = await apiService.exportDemoReport('markdown');
      } else if (reportId) {
        console.log('📄 Calling exportReport with ID:', reportId);
        blob = await apiService.exportReport(reportId, 'markdown');
      } else {
        console.error('❌ No report ID available for export');
        return;
      }
      
      console.log('✅ Got blob response:', blob.size, 'bytes');
      
      // Create download link
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.style.display = 'none';
      a.href = url;
      a.download = `uveddi-analysis-${isDemoReport ? 'demo' : reportId}.md`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
      
      console.log('🎉 Report exported successfully');
    } catch (error) {
      console.error('💥 Failed to export report:', error);
    }
  };
  
  const query = isDemoReport ? demoQuery : reportQuery;
  const { data: report, isLoading, error } = query;

  if (isLoading) {
    return (
      <UveddiLoader 
        message="Loading analysis report..." 
        size="large" 
      />
    );
  }

  if (error) {
    return (
      <Alert 
        severity="error" 
        sx={{ 
          maxWidth: 600, 
          mx: 'auto', 
          mt: 4,
          '& .MuiAlert-message': { 
            display: 'flex', 
            flexDirection: 'column', 
            width: '100%' 
          } 
        }}
      >
        <Box component="div">
          <Typography variant="h6" component="div" gutterBottom>
            Failed to load report
          </Typography>
          <Typography variant="body2" component="div">
            {error.message}
          </Typography>
        </Box>
      </Alert>
    );
  }

  if (!report) {
    return (
      <Alert 
        severity="info" 
        sx={{ 
          maxWidth: 600, 
          mx: 'auto', 
          mt: 4,
          '& .MuiAlert-message': { 
            display: 'flex', 
            flexDirection: 'column', 
            width: '100%' 
          } 
        }}
      >
        <Box component="div">
          <Typography variant="h6" component="div" gutterBottom>
            Report not found
          </Typography>
          <Typography variant="body2" component="div">
            The requested report could not be found.
          </Typography>
        </Box>
      </Alert>
    );
  }

  // Additional safety check for required properties
  if (!report.project || !report.summary) {
    return (
      <Alert 
        severity="error" 
        sx={{ 
          maxWidth: 600, 
          mx: 'auto', 
          mt: 4,
          '& .MuiAlert-message': { 
            display: 'flex', 
            flexDirection: 'column', 
            width: '100%' 
          } 
        }}
      >
        <Box component="div">
          <Typography variant="h6" component="div" gutterBottom>
            Invalid report data
          </Typography>
          <Typography variant="body2" component="div">
            The report data is incomplete or corrupted.
          </Typography>
        </Box>
      </Alert>
    );
  }

  return (
    <Box>
      {/* Enhanced Header */}
      <Paper 
        sx={{ 
          p: 4, 
          mb: 4,
          background: (theme) => theme.palette.mode === 'light' 
            ? 'linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)'
            : 'linear-gradient(135deg, #1a1f2e 0%, #242b3d 100%)',
          border: (theme) => `1px solid ${theme.palette.divider}`,
          boxShadow: (theme) => theme.palette.mode === 'light'
            ? '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)'
            : '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2)',
        }}
      >
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 3 }}>
          <Box sx={{ flex: 1 }}>
            <Typography 
              variant="h3" 
              component="h1" 
              sx={{ 
                fontWeight: 700,
                color: (theme) => theme.palette.text.primary,
                mb: 1,
                textShadow: (theme) => theme.palette.mode === 'light' 
                  ? '0 1px 2px rgba(0, 0, 0, 0.1)'
                  : '0 1px 2px rgba(0, 0, 0, 0.3)',
              }}
            >
              {report.project?.name || 'Unknown Project'}
            </Typography>
            <Typography 
              variant="body1" 
              sx={{ 
                color: (theme) => theme.palette.text.secondary,
                mb: 2,
                fontWeight: 500,
              }}
            >
              Analysis completed on{' '}
              {new Date(report.summary?.timeGenerated || Date.now()).toLocaleDateString()} at{' '}
              {new Date(report.summary?.timeGenerated || Date.now()).toLocaleTimeString()}
            </Typography>
          </Box>
          
          {/* Action Buttons */}
          <Box sx={{ display: 'flex', gap: 1, flexShrink: 0 }}>
            <Button
              variant="outlined"
              startIcon={<ShareOutlined />}
              color="primary"
            >
              Share
            </Button>
            <Button
              variant="contained"
              startIcon={<DownloadOutlined />}
              color="primary"
              onClick={handleExportReport}
            >
              Export Report
            </Button>
          </Box>
        </Box>
        
        {/* Tags and Badges */}
        <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
          {(report.project?.languages || []).map((lang: string) => (
            <Chip 
              key={lang} 
              label={lang} 
              size="small" 
              sx={{
                backgroundColor: (theme) => theme.palette.mode === 'dark' 
                  ? theme.palette.primary.dark 
                  : theme.palette.primary.light,
                color: (theme) => theme.palette.mode === 'dark'
                  ? theme.palette.common.white
                  : theme.palette.primary.dark,
                fontWeight: 600,
                border: (theme) => `1px solid ${theme.palette.primary.main}`,
              }}
            />
          ))}
          {isDemoReport && (
            <Chip 
              label="Demo Data" 
              size="small"
              sx={{
                backgroundColor: (theme) => theme.palette.mode === 'dark'
                  ? theme.palette.warning.dark
                  : theme.palette.warning.light,
                color: (theme) => theme.palette.mode === 'dark'
                  ? theme.palette.common.white
                  : theme.palette.warning.dark,
                fontWeight: 600,
                border: (theme) => `1px solid ${theme.palette.warning.main}`,
              }}
            />
          )}
          <Chip
            icon={<BugReportOutlined />}
            label="AI-Enhanced Analysis"
            size="small"
            sx={{
              backgroundColor: (theme) => theme.palette.mode === 'dark'
                ? theme.palette.success.dark
                : theme.palette.success.light,
              color: (theme) => theme.palette.mode === 'dark'
                ? theme.palette.common.white
                : theme.palette.success.dark,
              fontWeight: 600,
              border: (theme) => `1px solid ${theme.palette.success.main}`,
            }}
          />
        </Box>
      </Paper>

      {/* Summary Cards */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Code Quality"
            value={`${Math.round(report.summary?.coverage || 0)}%`}
            subtitle="Overall Score"
            color="primary"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Total Issues"
            value={(report.summary?.issuesTotal || 0).toString()}
            subtitle={`${report.summary?.filesAnalyzed || 0} files analyzed`}
            color="error"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Components"
            value={(report.summary?.componentsAnalyzed || 0).toString()}
            subtitle="Architectural components"
            color="info"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Analysis Time"
            value={`${((report.summary?.analysisDurationMs || 0) / 1000).toFixed(1)}s`}
            subtitle="Processing time"
            color="success"
          />
        </Grid>
      </Grid>

      {/* Issues by Severity */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h5" gutterBottom>
              Issues by Severity
            </Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
              {Object.entries(report.summary?.issuesBySeverity || {}).map(([severity, count]) => (
                <Box
                  key={severity}
                  sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    p: 2,
                    border: 1,
                    borderColor: 'divider',
                    borderRadius: 1,
                    bgcolor: getSeverityColor(severity, theme),
                  }}
                >
                  <Typography variant="body1" sx={{ textTransform: 'capitalize' }}>
                    {severity}
                  </Typography>
                  <Typography variant="h6" fontWeight="bold">
                    {count}
                  </Typography>
                </Box>
              ))}
            </Box>
          </Paper>
        </Grid>

        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h5" gutterBottom>
              Issues by Category
            </Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
              {Object.entries(report.summary?.issuesByCategory || {}).map(([category, count]) => (
                <Box
                  key={category}
                  sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    p: 2,
                    border: 1,
                    borderColor: 'divider',
                    borderRadius: 1,
                    bgcolor: 'action.hover',
                  }}
                >
                  <Typography variant="body1" sx={{ textTransform: 'capitalize' }}>
                    {category}
                  </Typography>
                  <Typography variant="h6" fontWeight="bold">
                    {count}
                  </Typography>
                </Box>
              ))}
            </Box>
          </Paper>
        </Grid>
      </Grid>

      {/* Footer */}
      <Box
        component="footer"
        sx={{
          backgroundColor: (theme) => theme.palette.background.paper,
          borderTop: (theme) => `1px solid ${theme.palette.divider}`,
          borderRadius: 2,
          py: 3,
          mb: 4,
          mt: 2,
        }}
      >
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Typography 
            variant="body2" 
            sx={{ 
              color: (theme) => theme.palette.text.secondary,
              display: 'flex',
              alignItems: 'center',
              gap: 1,
            }}
          >
            <BugReportOutlined sx={{ fontSize: 16 }} />
            Powered by Uveddi Analysis Engine
          </Typography>
          <Typography 
            variant="caption" 
            sx={{ 
              color: (theme) => theme.palette.text.secondary,
              fontFamily: "'JetBrains Mono', monospace",
              opacity: 0.7,
            }}
          >
            Build: {new Date().toISOString().split('T')[0]} • v0.9.0-alpha
          </Typography>
        </Box>
      </Box>
      
      {/* Architectural Diagrams Section */}
      {report.diagrams && report.diagrams.length > 0 && (
        <Box sx={{ mb: 4 }}>
          <Typography variant="h4" component="h2" gutterBottom>
            Architectural Diagrams
          </Typography>
          <Grid container spacing={3}>
            {report.diagrams.map((diagram: any, index: number) => (
              <Grid item xs={12} key={diagram.id || index}>
                <MermaidDiagram 
                  definition={diagram.source}
                  title={diagram.title}
                />
              </Grid>
            ))}
          </Grid>
        </Box>
      )}

      {/* Detailed Findings List */}
      <FindingsList 
        findings={report.findings || []} 
        loading={isLoading}
      />
    </Box>
  );
}

// Helper component for summary cards
interface SummaryCardProps {
  title: string;
  value: string;
  subtitle: string;
  color: 'primary' | 'error' | 'info' | 'success';
}

function SummaryCard({ title, value, subtitle, color }: SummaryCardProps) {
  const theme = useTheme();
  
  const getColorStyles = (color: string) => {
    const isDark = theme.palette.mode === 'dark';
    
    switch (color) {
      case 'primary':
        return {
          borderColor: theme.palette.primary.main,
          valueColor: theme.palette.primary.main,
          bgGradient: isDark 
            ? 'linear-gradient(135deg, rgba(25, 118, 210, 0.1) 0%, rgba(25, 118, 210, 0.05) 100%)'
            : 'linear-gradient(135deg, rgba(25, 118, 210, 0.1) 0%, rgba(25, 118, 210, 0.05) 100%)',
          icon: <TrendingUpOutlined />,
        };
      case 'error':
        return {
          borderColor: theme.palette.error.main,
          valueColor: theme.palette.error.main,
          bgGradient: isDark
            ? 'linear-gradient(135deg, rgba(239, 83, 80, 0.1) 0%, rgba(229, 115, 115, 0.05) 100%)'
            : 'linear-gradient(135deg, #ffebee 0%, #fce4ec 100%)',
          icon: <BugReportOutlined />,
        };
      case 'success':
        return {
          borderColor: theme.palette.success.main,
          valueColor: theme.palette.success.main,
          bgGradient: isDark
            ? 'linear-gradient(135deg, rgba(129, 199, 132, 0.1) 0%, rgba(165, 214, 167, 0.05) 100%)'
            : 'linear-gradient(135deg, #e8f5e9 0%, #f1f8e9 100%)',
          icon: <TrendingUpOutlined />,
        };
      default:
        return {
          borderColor: theme.palette.info.main,
          valueColor: theme.palette.info.main,
          bgGradient: isDark
            ? 'linear-gradient(135deg, rgba(79, 195, 247, 0.1) 0%, rgba(129, 212, 250, 0.05) 100%)'
            : 'linear-gradient(135deg, #e3f2fd 0%, #f0f4ff 100%)',
          icon: <BugReportOutlined />,
        };
    }
  };

  const styles = getColorStyles(color);

  return (
    <Paper 
      sx={{ 
        p: 3, 
        textAlign: 'center',
        border: (theme) => `1px solid ${theme.palette.divider}`,
        borderTop: `4px solid ${styles.borderColor}`,
        background: styles.bgGradient,
        position: 'relative',
        overflow: 'hidden',
        '&:hover': {
          transform: 'translateY(-2px)',
          boxShadow: (theme) => theme.palette.mode === 'light'
            ? '0 8px 25px rgba(79, 70, 229, 0.15)'
            : '0 8px 25px rgba(0, 0, 0, 0.3)',
        },
        transition: 'all 0.2s ease-in-out',
      }}
    >
      {/* Background Icon */}
      <Box
        sx={{
          position: 'absolute',
          top: 16,
          right: 16,
          opacity: 0.1,
          fontSize: 48,
          color: styles.valueColor,
        }}
      >
        {styles.icon}
      </Box>
      
      <Typography 
        variant="h6" 
        sx={{ 
          color: 'var(--uveddi-text-secondary)',
          fontWeight: 600,
          mb: 2,
        }}
      >
        {title}
      </Typography>
      <Typography 
        variant="h3" 
        component="div" 
        sx={{ 
          color: styles.valueColor,
          fontWeight: 700,
          fontFamily: "'JetBrains Mono', monospace",
          mb: 1,
        }}
      >
        {value}
      </Typography>
      <Typography 
        variant="body2" 
        sx={{ 
          color: 'var(--uveddi-text-muted)',
          fontWeight: 500,
        }}
      >
        {subtitle}
      </Typography>
    </Paper>
  );
}

// Helper function to get severity-based background color
function getSeverityColor(severity: string, theme: any) {
  const alpha = theme.palette.mode === 'light' ? 0.1 : 0.2;
  
  switch (severity.toLowerCase()) {
    case 'critical':
      return theme.palette.mode === 'light' 
        ? 'rgba(244, 67, 54, 0.1)' 
        : 'rgba(239, 83, 80, 0.2)'; // Red
    case 'high':
      return theme.palette.mode === 'light' 
        ? 'rgba(255, 152, 0, 0.1)' 
        : 'rgba(255, 183, 77, 0.2)'; // Orange
    case 'medium':
      return theme.palette.mode === 'light' 
        ? 'rgba(255, 193, 7, 0.1)' 
        : 'rgba(255, 213, 79, 0.2)'; // Yellow
    case 'low':
      return theme.palette.mode === 'light' 
        ? 'rgba(76, 175, 80, 0.1)' 
        : 'rgba(129, 199, 132, 0.2)'; // Green
    default:
      return theme.palette.mode === 'light' 
        ? 'rgba(0, 0, 0, 0.05)' 
        : 'rgba(255, 255, 255, 0.05)'; // Default
  }
}

export default DashboardPage;