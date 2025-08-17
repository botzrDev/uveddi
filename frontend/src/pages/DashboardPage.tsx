import React from 'react';
import { useParams } from 'react-router-dom';
import {
  Box,
  Typography,
  CircularProgress,
  Alert,
  Grid,
  Paper,
  Chip,
} from '@mui/material';
import { useReport, useDemoReport } from '@/hooks/useReport';
import type { InteractiveReport } from '@/types/api';
import FindingsList from '@/components/FindingsList';
import MermaidDiagram from '@/components/MermaidDiagram';
import SimpleMermaidTest from '@/components/SimpleMermaidTest';

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
      <Box 
        display="flex" 
        justifyContent="center" 
        alignItems="center" 
        minHeight="400px"
      >
        <CircularProgress size={60} />
        <Typography variant="h6" sx={{ ml: 2 }}>
          Loading report...
        </Typography>
      </Box>
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

  return (
    <Box>
      {/* Header */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h3" component="h1" gutterBottom>
          {report.project.name}
        </Typography>
        <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mb: 2 }}>
          {report.project.languages.map((lang) => (
            <Chip key={lang} label={lang} size="small" variant="outlined" />
          ))}
          {isDemoReport && (
            <Chip label="Demo Data" color="primary" size="small" />
          )}
        </Box>
        <Typography variant="body1" color="text.secondary">
          Analysis completed on{' '}
          {new Date(report.summary.timeGenerated).toLocaleDateString()} at{' '}
          {new Date(report.summary.timeGenerated).toLocaleTimeString()}
        </Typography>
      </Box>

      {/* Summary Cards */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Code Quality"
            value={`${Math.round(report.summary.coverage)}%`}
            subtitle="Overall Score"
            color="primary"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Total Issues"
            value={report.summary.issuesTotal.toString()}
            subtitle={`${report.summary.filesAnalyzed} files analyzed`}
            color="error"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Components"
            value={report.summary.componentsAnalyzed.toString()}
            subtitle="Architectural components"
            color="info"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <SummaryCard
            title="Analysis Time"
            value={`${(report.summary.analysisDurationMs / 1000).toFixed(1)}s`}
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
              {Object.entries(report.summary.issuesBySeverity).map(([severity, count]) => (
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
              {Object.entries(report.summary.issuesByCategory).map(([category, count]) => (
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
      
      {/* Architectural Diagrams Section */}
      {report.diagrams && report.diagrams.length > 0 && (
        <Box sx={{ mb: 4 }}>
          <Typography variant="h4" component="h2" gutterBottom>
            Architectural Diagrams
          </Typography>
          <Grid container spacing={3}>
            {report.diagrams.map((diagram, index) => (
              <Grid item xs={12} lg={6} key={diagram.id || index}>
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
  return (
    <Paper 
      sx={{ 
        p: 3, 
        textAlign: 'center',
        borderTop: 4,
        borderTopColor: `${color}.main`,
      }}
    >
      <Typography variant="h6" color="text.secondary" gutterBottom>
        {title}
      </Typography>
      <Typography variant="h3" component="div" color={`${color}.main`} fontWeight="bold">
        {value}
      </Typography>
      <Typography variant="body2" color="text.secondary">
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