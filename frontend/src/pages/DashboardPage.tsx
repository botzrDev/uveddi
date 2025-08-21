import SeverityBarChart from '@/components/charts/SeverityBarChart';
import Sparkline from '@/components/charts/Sparkline';
import FindingsList from '@/components/FindingsList';
import MermaidDiagram from '@/components/MermaidDiagram';
import UveddiLoader from '@/components/UveddiLoader';
// import SecurityOverview from '@/components/dashboard/SecurityOverview';
import { useDemoReport, useReport } from '@/hooks/useReport';
import { apiService } from '@/services/api';
import {
    AccountTreeOutlined,
    AssessmentOutlined,
    BugReportOutlined,
    CheckCircleOutline,
    DownloadOutlined,
    HourglassEmpty,
    SecurityOutlined,
    TimerOutlined
} from '@mui/icons-material';
import {
    Alert,
    Box,
    Button,
    Chip,
    Grid,
    Paper,
    Tab,
    Tabs,
    Typography,
    useTheme,
} from '@mui/material';
import { useEffect, useState } from 'react';
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
    setExportStatus('exporting');
    
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
        setExportStatus('error');
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
      setExportStatus('success');
      
      // Reset to idle after success animation
      setTimeout(() => setExportStatus('idle'), 3000);
    } catch (error) {
      console.error('💥 Failed to export report:', error);
      setExportStatus('error');
      
      // Reset to idle after error display
      setTimeout(() => setExportStatus('idle'), 3000);
    }
  };
  
  const query = isDemoReport ? demoQuery : reportQuery;
  const { data: report, isLoading, error } = query;
  const [selectedSeverity, setSelectedSeverity] = useState<string | null>(null);
  const [exportStatus, setExportStatus] = useState<'idle' | 'exporting' | 'success' | 'error'>('idle');
  const [activeTab, setActiveTab] = useState<'overview' | 'security' | 'findings'>('overview');

  

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
          
          {/* Action Buttons with Enhanced Status Feedback */}
          <Box sx={{ display: 'flex', gap: 1, flexShrink: 0 }}>
            <Button
              variant="contained"
              startIcon={
                exportStatus === 'exporting' ? <HourglassEmpty /> :
                exportStatus === 'success' ? <CheckCircleOutline /> :
                <DownloadOutlined />
              }
              onClick={handleExportReport}
              disabled={exportStatus === 'exporting'}
              sx={{
                background: (theme) => {
                  if (exportStatus === 'success') {
                    return theme.palette.mode === 'light'
                      ? 'linear-gradient(135deg, #4caf50 0%, #66bb6a 50%, #81c784 100%)'
                      : 'linear-gradient(135deg, #81c784 0%, #66bb6a 50%, #4caf50 100%)';
                  } else if (exportStatus === 'error') {
                    return theme.palette.mode === 'light'
                      ? 'linear-gradient(135deg, #f44336 0%, #ef5350 50%, #e57373 100%)'
                      : 'linear-gradient(135deg, #e57373 0%, #ef5350 50%, #f44336 100%)';
                  } else {
                    return theme.palette.mode === 'light'
                      ? 'linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #42a5f5 100%)'
                      : 'linear-gradient(135deg, #42a5f5 0%, #1976d2 50%, #1565c0 100%)';
                  }
                },
                color: 'white',
                fontWeight: 600,
                fontSize: '0.95rem',
                px: 3,
                py: 1.5,
                borderRadius: 2,
                textTransform: 'none',
                boxShadow: (theme) => {
                  const baseColor = exportStatus === 'success' ? '76, 175, 80' :
                                   exportStatus === 'error' ? '244, 67, 54' :
                                   theme.palette.mode === 'light' ? '25, 118, 210' : '100, 181, 246';
                  return `0 4px 15px rgba(${baseColor}, 0.3)`;
                },
                border: 'none',
                position: 'relative',
                overflow: 'hidden',
                opacity: exportStatus === 'exporting' ? 0.8 : 1,
                transform: exportStatus === 'success' ? 'scale(1.05)' : 'scale(1)',
                '&::before': {
                  content: '""',
                  position: 'absolute',
                  top: 0,
                  left: exportStatus === 'exporting' ? '-100%' : '-100%',
                  width: '100%',
                  height: '100%',
                  background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent)',
                  transition: 'left 0.5s',
                  ...(exportStatus === 'exporting' && {
                    animation: 'shimmer 1.5s infinite linear',
                    '@keyframes shimmer': {
                      '0%': { left: '-100%' },
                      '100%': { left: '100%' },
                    },
                  }),
                },
                '&:hover:not(:disabled)': {
                  background: (theme) => {
                    if (exportStatus === 'success') {
                      return theme.palette.mode === 'light'
                        ? 'linear-gradient(135deg, #388e3c 0%, #4caf50 50%, #66bb6a 100%)'
                        : 'linear-gradient(135deg, #a5d6a7 0%, #81c784 50%, #66bb6a 100%)';
                    } else if (exportStatus === 'error') {
                      return theme.palette.mode === 'light'
                        ? 'linear-gradient(135deg, #d32f2f 0%, #f44336 50%, #ef5350 100%)'
                        : 'linear-gradient(135deg, #ffcdd2 0%, #e57373 50%, #ef5350 100%)';
                    } else {
                      return theme.palette.mode === 'light'
                        ? 'linear-gradient(135deg, #0d47a1 0%, #1565c0 50%, #1976d2 100%)'
                        : 'linear-gradient(135deg, #64b5f6 0%, #42a5f5 50%, #1976d2 100%)';
                    }
                  },
                  transform: exportStatus === 'success' ? 'translateY(-2px) scale(1.05)' : 'translateY(-2px)',
                  boxShadow: (theme) => {
                    const baseColor = exportStatus === 'success' ? '76, 175, 80' :
                                     exportStatus === 'error' ? '244, 67, 54' :
                                     theme.palette.mode === 'light' ? '25, 118, 210' : '100, 181, 246';
                    return `0 6px 20px rgba(${baseColor}, 0.4)`;
                  },
                  '&::before': {
                    left: '100%',
                  },
                },
                '&:active:not(:disabled)': {
                  transform: exportStatus === 'success' ? 'translateY(0px) scale(1.05)' : 'translateY(0px)',
                  boxShadow: (theme) => {
                    const baseColor = exportStatus === 'success' ? '76, 175, 80' :
                                     exportStatus === 'error' ? '244, 67, 54' :
                                     theme.palette.mode === 'light' ? '25, 118, 210' : '100, 181, 246';
                    return `0 2px 10px rgba(${baseColor}, 0.3)`;
                  },
                },
                '&:disabled': {
                  background: (theme) => theme.palette.mode === 'light'
                    ? 'rgba(0, 0, 0, 0.12)'
                    : 'rgba(255, 255, 255, 0.12)',
                  color: (theme) => theme.palette.mode === 'light'
                    ? 'rgba(0, 0, 0, 0.26)'
                    : 'rgba(255, 255, 255, 0.3)',
                },
                transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
              }}
            >
              {exportStatus === 'exporting' ? 'Exporting...' :
               exportStatus === 'success' ? 'Exported!' :
               exportStatus === 'error' ? 'Export Failed' :
               'Export Report'}
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

      {/* Summary Cards - Enhanced Responsive Layout */}
      <Grid container spacing={{ xs: 2, sm: 3 }} sx={{ mb: 4 }}>
        <Grid item xs={12} sm={6} lg={3}>
          <SummaryCard
            title="Code Quality"
            value={`${Math.round(report.summary?.coverage || 0)}%`}
            sparkData={[(report.summary?.coverage || 0) - 2, (report.summary?.coverage || 0) - 1, report.summary?.coverage || 0]}
            subtitle="Overall Score"
            color="primary"
          />
        </Grid>
        <Grid item xs={12} sm={6} lg={3}>
          <SummaryCard
            title="Total Issues"
            value={(report.summary?.issuesTotal || 0).toString()}
            sparkData={[Math.max(0, (report.summary?.issuesTotal || 0) - 5), Math.max(0, (report.summary?.issuesTotal || 0) - 2), report.summary?.issuesTotal || 0]}
            subtitle={`${report.summary?.filesAnalyzed || 0} files analyzed`}
            color="error"
          />
        </Grid>
        <Grid item xs={12} sm={6} lg={3}>
          <SummaryCard
            title="Components"
            value={(report.summary?.componentsAnalyzed || 0).toString()}
            sparkData={[(report.summary?.componentsAnalyzed || 0) - 1, (report.summary?.componentsAnalyzed || 0), (report.summary?.componentsAnalyzed || 0) + 1]}
            subtitle="Architectural components"
            color="info"
          />
        </Grid>
        <Grid item xs={12} sm={6} lg={3}>
          <SummaryCard
            title="Analysis Time"
            value={`${((report.summary?.analysisDurationMs || 0) / 1000).toFixed(1)}s`}
            sparkData={[((report.summary?.analysisDurationMs || 0) / 1000) - 1, ((report.summary?.analysisDurationMs || 0) / 1000) - 0.5, ((report.summary?.analysisDurationMs || 0) / 1000)]}
            subtitle="Processing time"
            color="success"
          />
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
            Build: {new Date().toISOString().split('T')[0]} • v1.0 community core
          </Typography>
        </Box>
      </Box>
      {/* Enhanced Architectural Diagrams Section */}
      {report.diagrams && report.diagrams.length > 0 && (
        <Paper 
          sx={{ 
            p: { xs: 3, sm: 4 }, 
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
          <Box sx={{ mb: 4 }}>
            <Typography 
              variant="h4" 
              component="h2" 
              sx={{
                fontWeight: 700,
                color: (theme) => theme.palette.text.primary,
                mb: 2,
                display: 'flex',
                alignItems: 'center',
                gap: 2,
                fontSize: { xs: '1.75rem', sm: '2.125rem' },
              }}
            >
              🏗️ Architectural Diagrams
            </Typography>
            <Typography 
              variant="body1" 
              sx={{ 
                color: (theme) => theme.palette.text.secondary,
                mb: 3,
                fontSize: { xs: '0.9rem', sm: '1rem' },
                lineHeight: 1.6,
                maxWidth: '800px',
              }}
            >
              Interactive system architecture visualizations showing component relationships, 
              data flow, and structural dependencies. Click any diagram to view in full screen.
            </Typography>
          </Box>
          
          <Grid container spacing={{ xs: 3, sm: 4 }}>
            {report.diagrams.map((diagram: any, index: number) => (
              <Grid item xs={12} key={diagram.id || index}>
                <Box
                  sx={{
                    border: (theme) => `2px solid ${theme.palette.divider}`,
                    borderRadius: 3,
                    overflow: 'hidden',
                    background: (theme) => theme.palette.background.paper,
                    transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                    '&:hover': {
                      borderColor: (theme) => theme.palette.primary.main,
                      boxShadow: (theme) => theme.palette.mode === 'light'
                        ? `0 12px 24px rgba(25, 118, 210, 0.15)`
                        : `0 12px 24px rgba(100, 181, 246, 0.25)`,
                      transform: 'translateY(-2px)',
                    }
                  }}
                >
                  {/* Diagram Header */}
                  <Box 
                    sx={{ 
                      p: 3,
                      borderBottom: (theme) => `1px solid ${theme.palette.divider}`,
                      background: (theme) => theme.palette.mode === 'light'
                        ? 'rgba(25, 118, 210, 0.04)'
                        : 'rgba(100, 181, 246, 0.08)',
                    }}
                  >
                    <Typography 
                      variant="h6" 
                      sx={{ 
                        fontWeight: 600,
                        color: (theme) => theme.palette.text.primary,
                        fontSize: { xs: '1.1rem', sm: '1.25rem' },
                      }}
                    >
                      {diagram.title || `Architecture Overview ${index + 1}`}
                    </Typography>
                    <Typography 
                      variant="body2" 
                      sx={{ 
                        color: (theme) => theme.palette.text.secondary,
                        mt: 1,
                        fontSize: '0.875rem',
                      }}
                    >
                      System Architecture Overview • Interactive Diagram
                    </Typography>
                  </Box>
                  
                  {/* Enhanced Diagram Container */}
                  <Box sx={{ p: 0 }}>
                    <MermaidDiagram 
                      definition={diagram.source}
                      title={diagram.title}
                      previewHeight="600px" // Increased from default 300px
                    />
                  </Box>
                </Box>
              </Grid>
            ))}
          </Grid>
        </Paper>
      )}

      {/* Main Content Tabs */}
      <Paper sx={{ mb: 4 }}>
        <Tabs 
          value={activeTab} 
          onChange={(_, newValue) => setActiveTab(newValue)}
          variant="fullWidth"
          sx={{
            borderBottom: 1,
            borderColor: 'divider',
            '& .MuiTab-root': {
              textTransform: 'none',
              fontWeight: 600,
              fontSize: '1rem',
            }
          }}
        >
          <Tab 
            label="Overview" 
            value="overview" 
            icon={<AssessmentOutlined />} 
            iconPosition="start"
          />
          {report.securityAnalysis && (
            <Tab 
              label={`Security (${report.securityAnalysis.summary?.totalIssues || 0} issues)`}
              value="security" 
              icon={<SecurityOutlined />} 
              iconPosition="start"
            />
          )}
          <Tab 
            label={`Findings (${report.findings?.length || 0})`}
            value="findings" 
            icon={<BugReportOutlined />} 
            iconPosition="start"
          />
        </Tabs>
        
        {/* Tab Content */}
        <Box sx={{ p: 3 }}>
          {activeTab === 'overview' && (
            <>
              {/* Issues by Severity (interactive) */}
              <Grid container spacing={3} sx={{ mb: 4 }}>
                <Grid item xs={12} md={7}>
                  <Paper sx={{ p: 3 }}>
                    <Typography variant="h5" gutterBottom>
                      Issues by Severity
                    </Typography>
                    <SeverityBarChart
                      data={Object.entries(report.summary?.issuesBySeverity || {}).map(([severity, count]) => ({ severity, count: Number(count) }))}
                      onBarClick={(p) => {
                        const sev = String(p.severity).toLowerCase();
                        setSelectedSeverity(prev => prev === sev ? null : sev);
                      }}
                      selectedSeverity={selectedSeverity}
                    />
                  </Paper>
                </Grid>

                <Grid item xs={12} md={5}>
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
            </>
          )}
          
          {activeTab === 'security' && report.securityAnalysis && (
            <Box>
              <Typography variant="h5" gutterBottom>
                Security Analysis
              </Typography>
              
              {/* Security Summary */}
              <Grid container spacing={3} sx={{ mb: 4 }}>
                <Grid item xs={12} sm={6} md={3}>
                  <Paper sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="h6" color="error">
                      {report.securityAnalysis.summary?.criticalCount || 0}
                    </Typography>
                    <Typography variant="body2">Critical Issues</Typography>
                  </Paper>
                </Grid>
                <Grid item xs={12} sm={6} md={3}>
                  <Paper sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="h6" color="warning.main">
                      {report.securityAnalysis.summary?.highCount || 0}
                    </Typography>
                    <Typography variant="body2">High Issues</Typography>
                  </Paper>
                </Grid>
                <Grid item xs={12} sm={6} md={3}>
                  <Paper sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="h6" color="info.main">
                      {report.securityAnalysis.summary?.mediumCount || 0}
                    </Typography>
                    <Typography variant="body2">Medium Issues</Typography>
                  </Paper>
                </Grid>
                <Grid item xs={12} sm={6} md={3}>
                  <Paper sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="h6" color="success.main">
                      {report.securityAnalysis.summary?.securityScore || 0}/100
                    </Typography>
                    <Typography variant="body2">Security Score</Typography>
                  </Paper>
                </Grid>
              </Grid>
              
              {/* Security Issues List */}
              <Paper sx={{ p: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Security Issues ({report.securityAnalysis.summary?.totalIssues || 0})
                </Typography>
                {report.securityAnalysis.issues && report.securityAnalysis.issues.length > 0 ? (
                  <Box sx={{ mt: 2 }}>
                    {report.securityAnalysis.issues.slice(0, 10).map((issue, index) => (
                      <Box key={index} sx={{ 
                        p: 2, 
                        mb: 2, 
                        border: 1, 
                        borderColor: 'divider', 
                        borderRadius: 1,
                        bgcolor: issue.severity === 'Critical' ? 'error.light' :
                                issue.severity === 'High' ? 'warning.light' :
                                issue.severity === 'Medium' ? 'info.light' : 'grey.100'
                      }}>
                        <Typography variant="h6" gutterBottom>
                          {issue.issueType}
                        </Typography>
                        <Typography variant="body2" sx={{ mb: 1 }}>
                          <strong>File:</strong> {issue.location?.file}
                        </Typography>
                        <Typography variant="body2" sx={{ mb: 1 }}>
                          <strong>Line:</strong> {issue.location?.startLine}
                        </Typography>
                        <Typography variant="body2" sx={{ mb: 1 }}>
                          <strong>Severity:</strong> {issue.severity}
                        </Typography>
                        <Typography variant="body2">
                          {issue.description}
                        </Typography>
                      </Box>
                    ))}
                  </Box>
                ) : (
                  <Typography>No security issues found.</Typography>
                )}
                
                {/* Export Button */}
                <Box sx={{ mt: 3 }}>
                  <Button
                    variant="contained"
                    onClick={async () => {
                      try {
                        const blob = await apiService.exportSarif();
                        const url = window.URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = 'security-analysis.sarif';
                        a.click();
                        window.URL.revokeObjectURL(url);
                      } catch (error) {
                        console.error('Failed to export SARIF:', error);
                      }
                    }}
                  >
                    Export SARIF
                  </Button>
                </Box>
              </Paper>
            </Box>
          )}
          
          {activeTab === 'findings' && (
            <FindingsList 
              findings={report.findings || []} 
              loading={isLoading}
              severityOverride={selectedSeverity}
            />
          )}
        </Box>
      </Paper>
    </Box>
  );
}

// Snackbar for share feedback is rendered near the root of the page
 

// Helper component for summary cards
interface SummaryCardProps {
  title: string;
  value: string;
  subtitle: string;
  color: 'primary' | 'error' | 'info' | 'success';
  sparkData?: number[];
}

function SummaryCard({ title, value, subtitle, color, sparkData }: SummaryCardProps) {
  const theme = useTheme();
  const [displayValue, setDisplayValue] = useState<string>(value);
  
  const getColorStyles = (color: string) => {
    const isDark = theme.palette.mode === 'dark';
    
    switch (color) {
      case 'primary':
        return {
          borderColor: theme.palette.primary.main,
          valueColor: theme.palette.primary.main,
          bgGradient: isDark 
            ? 'linear-gradient(135deg, rgba(100, 181, 246, 0.15) 0%, rgba(25, 118, 210, 0.08) 100%)'
            : 'linear-gradient(135deg, rgba(25, 118, 210, 0.12) 0%, rgba(66, 165, 245, 0.06) 100%)',
          icon: <AssessmentOutlined />,
        };
      case 'error':
        // Much lighter, more pleasant red - 50% lighter
        const errorColor = isDark ? '#ffcccb' : '#ff9999'; // Much lighter coral/salmon colors
        return {
          borderColor: errorColor,
          valueColor: errorColor,
          bgGradient: isDark 
            ? 'linear-gradient(135deg, rgba(255, 204, 203, 0.15) 0%, rgba(255, 153, 153, 0.08) 100%)'
            : 'linear-gradient(135deg, rgba(255, 153, 153, 0.12) 0%, rgba(255, 204, 203, 0.06) 100%)',
          icon: <BugReportOutlined />,
        };
      case 'success':
        return {
          borderColor: theme.palette.success.main,
          valueColor: theme.palette.success.main,
          bgGradient: isDark 
            ? 'linear-gradient(135deg, rgba(129, 199, 132, 0.15) 0%, rgba(76, 175, 80, 0.08) 100%)'
            : 'linear-gradient(135deg, rgba(76, 175, 80, 0.12) 0%, rgba(129, 199, 132, 0.06) 100%)',
          icon: <TimerOutlined />,
        };
      default: // info
        return {
          borderColor: theme.palette.info.main,
          valueColor: theme.palette.info.main,
          bgGradient: isDark 
            ? 'linear-gradient(135deg, rgba(79, 195, 247, 0.15) 0%, rgba(2, 136, 209, 0.08) 100%)'
            : 'linear-gradient(135deg, rgba(2, 136, 209, 0.12) 0%, rgba(79, 195, 247, 0.06) 100%)',
          icon: <AccountTreeOutlined />,
        };
    }
  };

  const styles = getColorStyles(color);

  // Animate numeric values (simple count-up) when numeric present
  useEffect(() => {
    const raw = String(value || '');
    const numericMatch = raw.match(/-?\d+(?:[\.,]\d+)?/);
    if (!numericMatch) {
      setDisplayValue(raw);
      return;
    }

    const numStr = numericMatch[0].replace(',', '.');
    const target = Number(numStr);
    if (isNaN(target)) {
      setDisplayValue(raw);
      return;
    }

    const suffix = raw.replace(numericMatch[0], '').trim();
    const duration = 700;
    const start = performance.now();
    const from = 0;

    const step = (now: number) => {
      const t = Math.min(1, (now - start) / duration);
      const current = from + (target - from) * t;
      const formatted = (Math.abs(target) >= 100 ? Math.round(current) : Math.round(current * 10) / 10);
      setDisplayValue(`${formatted}${suffix}`);
      if (t < 1) requestAnimationFrame(step);
    };

    requestAnimationFrame(step);
  }, [value]);

  return (
    <Paper 
      className={`uveddi-summary-card card-${color}`}
      sx={{ 
        p: { xs: 3, sm: 4 }, // Responsive padding
        textAlign: 'center',
        border: 'none',
        borderRadius: 3,
        position: 'relative',
        overflow: 'hidden',
        background: styles.bgGradient,
        boxShadow: (theme) => theme.palette.mode === 'light'
          ? `0 4px 20px rgba(0, 0, 0, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.4)`
          : `0 4px 20px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.1)`,
        '&:hover': {
          transform: { xs: 'translateY(-2px)', sm: 'translateY(-6px) scale(1.02)' }, // Less dramatic on mobile
          boxShadow: (theme) => theme.palette.mode === 'light'
            ? `0 12px 40px ${styles.borderColor}25, 0 4px 20px rgba(0, 0, 0, 0.12)`
            : `0 12px 40px ${styles.borderColor}35, 0 4px 20px rgba(0, 0, 0, 0.5)`,
          '& .card-icon': {
            opacity: 0.25,
            transform: 'rotate(10deg) scale(1.2)',
          },
          '& .card-value': {
            transform: { xs: 'scale(1.02)', sm: 'scale(1.05)' }, // Less scaling on mobile
            color: styles.borderColor,
          }
        },
        transition: 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
        '&::after': {
          content: '""',
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: `radial-gradient(circle at top, ${styles.borderColor}10, transparent 70%)`,
          pointerEvents: 'none',
        }
      }}
    >
      {/* Background Icon */}
      <Box
        className="card-icon"
        sx={{
          position: 'absolute',
          top: { xs: 16, sm: 20 }, // Responsive positioning
          right: { xs: 16, sm: 20 },
          opacity: 0.12,
          fontSize: { xs: 48, sm: 60 }, // Responsive icon size
          color: styles.valueColor,
          transition: 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
          transform: 'rotate(-10deg)',
        }}
      >
        {styles.icon}
      </Box>
      
      <Typography 
        variant="h6" 
        sx={{ 
          color: 'var(--uveddi-text-secondary)',
          fontWeight: 700,
          mb: { xs: 2, sm: 3 }, // Responsive spacing
          fontSize: { xs: '0.9rem', sm: '1rem' }, // Responsive font size
          letterSpacing: '1px',
          textTransform: 'uppercase',
          opacity: 0.8,
        }}
      >
        {title}
      </Typography>
      <Typography 
        variant="h2"
        component="div" 
        className="card-value"
        sx={{ 
          color: styles.valueColor,
          fontWeight: 800,
          fontFamily: "'JetBrains Mono', monospace",
          mb: { xs: 1.5, sm: 2 }, // Responsive spacing
          fontSize: { xs: '1.8rem', sm: '2.5rem' }, // Responsive font size
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 1,
          transition: 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
          textShadow: (theme) => theme.palette.mode === 'light'
            ? `0 2px 4px ${styles.borderColor}20`
            : `0 2px 4px rgba(0, 0, 0, 0.5)`,
        }}
      >
        {displayValue}
        {sparkData && (
          <Box sx={{ ml: 1, display: { xs: 'none', sm: 'inline-flex' } }}>
            <Sparkline data={sparkData} color={styles.valueColor as string} />
          </Box>
        )}
      </Typography>
      <Typography 
        variant="body1"
        sx={{ 
          color: 'var(--uveddi-text-muted)',
          fontWeight: 600,
          fontSize: { xs: '0.8rem', sm: '0.9rem' }, // Responsive font size
          opacity: 0.7,
          letterSpacing: '0.3px',
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