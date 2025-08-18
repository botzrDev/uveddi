import FindingsList from '@/components/FindingsList';
import MermaidDiagram from '@/components/MermaidDiagram';
import MermaidRenderingTest from '@/components/MermaidRenderingTest';
import SimpleMermaidTest from '@/components/SimpleMermaidTest';
import UveddiLoader from '@/components/UveddiLoader';
import { useDemoReport, useReport } from '@/hooks/useReport';
import {
  Alert,
  Box,
  Chip,
  Grid,
  Paper,
  Typography,
  Button,
} from '@mui/material';
import {
  DownloadOutlined,
  ShareOutlined,
  BugReportOutlined,
  TrendingUpOutlined,
} from '@mui/icons-material';
import { useParams } from 'react-router-dom';

function DashboardPage() {
  const { reportId } = useParams<{ reportId: string }>();
  
  // Use demo report for demo route, otherwise fetch by ID
  const isDemoReport = reportId === 'demo';
  const reportQuery = useReport(reportId || '');
  const demoQuery = useDemoReport();
  
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
        sx={{ maxWidth: 600, mx: 'auto', mt: 4 }}
      >
        <Typography variant="h6" gutterBottom>
          Failed to load report
        </Typography>
        <Typography variant="body2">
          {error.message}
        </Typography>
      </Alert>
    );
  }

  if (!report) {
    return (
      <Alert 
        severity="info" 
        sx={{ maxWidth: 600, mx: 'auto', mt: 4 }}
      >
        <Typography variant="h6" gutterBottom>
          Report not found
        </Typography>
        <Typography variant="body2">
          The requested report could not be found.
        </Typography>
      </Alert>
    );
  }

  // Additional safety check for required properties
  if (!report.project || !report.summary) {
    return (
      <Alert 
        severity="error" 
        sx={{ maxWidth: 600, mx: 'auto', mt: 4 }}
      >
        <Typography variant="h6" gutterBottom>
          Invalid report data
        </Typography>
        <Typography variant="body2">
          The report data is incomplete or corrupted.
        </Typography>
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
          background: 'linear-gradient(135deg, var(--uveddi-primary-50) 0%, var(--uveddi-secondary-50) 100%)',
          border: '1px solid var(--uveddi-border)',
        }}
      >
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 3 }}>
          <Box sx={{ flex: 1 }}>
            <Typography 
              variant="h3" 
              component="h1" 
              sx={{ 
                fontWeight: 700,
                color: 'var(--uveddi-text-primary)',
                mb: 1,
              }}
            >
              {report.project?.name || 'Unknown Project'}
            </Typography>
            <Typography 
              variant="body1" 
              sx={{ 
                color: 'var(--uveddi-text-secondary)',
                mb: 2,
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
              sx={{
                borderColor: 'var(--uveddi-primary-600)',
                color: 'var(--uveddi-primary-600)',
                '&:hover': {
                  backgroundColor: 'var(--uveddi-primary-50)',
                  borderColor: 'var(--uveddi-primary-700)',
                },
              }}
            >
              Share
            </Button>
            <Button
              variant="contained"
              startIcon={<DownloadOutlined />}
              sx={{
                backgroundColor: 'var(--uveddi-primary-600)',
                '&:hover': {
                  backgroundColor: 'var(--uveddi-primary-700)',
                },
              }}
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
                backgroundColor: 'var(--uveddi-primary-100)',
                color: 'var(--uveddi-primary-800)',
                fontWeight: 600,
              }}
            />
          ))}
          {isDemoReport && (
            <Chip 
              label="Demo Data" 
              size="small"
              sx={{
                backgroundColor: 'var(--uveddi-action-100)',
                color: 'var(--uveddi-action-800)',
                fontWeight: 600,
              }}
            />
          )}
          <Chip
            icon={<BugReportOutlined />}
            label="AI-Enhanced Analysis"
            size="small"
            sx={{
              backgroundColor: 'var(--uveddi-success-100)',
              color: 'var(--uveddi-success-800)',
              fontWeight: 600,
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
                    bgcolor: getSeverityColor(severity),
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

      {/* Simple Mermaid Test */}
      <SimpleMermaidTest />
      
      {/* Mermaid Rendering Method Comparison */}
      <MermaidRenderingTest />
      
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
  const getColorStyles = (color: string) => {
    switch (color) {
      case 'primary':
        return {
          borderColor: 'var(--uveddi-primary-600)',
          valueColor: 'var(--uveddi-primary-600)',
          bgGradient: 'linear-gradient(135deg, var(--uveddi-primary-50) 0%, var(--uveddi-primary-25) 100%)',
          icon: <TrendingUpOutlined />,
        };
      case 'error':
        return {
          borderColor: 'var(--uveddi-error-500)',
          valueColor: 'var(--uveddi-error-500)',
          bgGradient: 'linear-gradient(135deg, #ffebee 0%, #fce4ec 100%)',
          icon: <BugReportOutlined />,
        };
      case 'success':
        return {
          borderColor: 'var(--uveddi-success-600)',
          valueColor: 'var(--uveddi-success-600)',
          bgGradient: 'linear-gradient(135deg, var(--uveddi-success-50) 0%, #e8f5e8 100%)',
          icon: <TrendingUpOutlined />,
        };
      default:
        return {
          borderColor: 'var(--uveddi-info-500)',
          valueColor: 'var(--uveddi-info-500)',
          bgGradient: 'linear-gradient(135deg, #e3f2fd 0%, #f0f4ff 100%)',
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
        border: '1px solid var(--uveddi-border)',
        borderTop: `4px solid ${styles.borderColor}`,
        background: styles.bgGradient,
        position: 'relative',
        overflow: 'hidden',
        '&:hover': {
          transform: 'translateY(-2px)',
          boxShadow: '0 8px 25px rgba(79, 70, 229, 0.15)',
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
function getSeverityColor(severity: string) {
  switch (severity.toLowerCase()) {
    case 'critical':
      return 'rgba(244, 67, 54, 0.1)'; // Red
    case 'high':
      return 'rgba(255, 152, 0, 0.1)'; // Orange
    case 'medium':
      return 'rgba(255, 193, 7, 0.1)'; // Yellow
    case 'low':
      return 'rgba(76, 175, 80, 0.1)'; // Green
    default:
      return 'rgba(0, 0, 0, 0.05)'; // Default
  }
}

export default DashboardPage;