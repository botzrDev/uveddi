import React, { useState, useCallback, useMemo } from 'react';
import {
  Box,
  Grid,
  Typography,
  Paper,
  Tabs,
  Tab,
  Button,
  Chip,
  Alert,
  AlertTitle,
  Fade,
  useTheme,
} from '@mui/material';
import {
  DashboardOutlined,
  AssessmentOutlined,
  TrendingUpOutlined,
  AutoAwesomeOutlined,
  BuildOutlined,
} from '@mui/icons-material';

import type { InteractiveReport, Finding } from '../../types/api';
import type { 
  PriorityMatrixItem, 
  QuadrantData, 
  QualityMetric,
  SelectionEvent,
  ComponentEvent 
} from '../../types/dashboard';

import { PriorityMatrix, QualityScoreCard } from './index';
import { dashboardTheme } from '../../utils/dashboardTheme';

interface EnhancedDashboardDemoProps {
  report: InteractiveReport;
  onFindingSelect?: (findings: Finding[]) => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

const TabPanel: React.FC<TabPanelProps> = ({ children, value, index, ...other }) => {
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`dashboard-tabpanel-${index}`}
      aria-labelledby={`dashboard-tab-${index}`}
      {...other}
    >
      {value === index && (
        <Fade in={true} timeout={300}>
          <Box>{children}</Box>
        </Fade>
      )}
    </div>
  );
};

const EnhancedDashboardDemo: React.FC<EnhancedDashboardDemoProps> = ({
  report,
  onFindingSelect,
}) => {
  const theme = useTheme();
  const [activeTab, setActiveTab] = useState(0);
  const [selectedFindings, setSelectedFindings] = useState<string[]>([]);
  const [selectedQuadrant, setSelectedQuadrant] = useState<keyof QuadrantData | null>(null);

  // Component event handlers
  const handlePriorityMatrixSelection = useCallback((event: SelectionEvent) => {
    setSelectedFindings(event.payload);
    
    // Find actual findings from selected IDs
    const findings = report.findings?.filter(f => event.payload.includes(f.id)) || [];
    if (onFindingSelect) {
      onFindingSelect(findings);
    }
  }, [report.findings, onFindingSelect]);

  const handleQuadrantFilter = useCallback((quadrant: keyof QuadrantData) => {
    setSelectedQuadrant(quadrant);
    console.log(`Filtered to quadrant: ${quadrant}`);
  }, []);

  const handleQualityMetricClick = useCallback((metric: QualityMetric) => {
    console.log(`Quality metric clicked: ${metric.name}`);
    // Could trigger drill-down view or filter findings by metric
  }, []);

  // Demo statistics
  const demoStats = useMemo(() => {
    const findings = report.findings || [];
    return {
      totalFindings: findings.length,
      criticalFindings: findings.filter(f => f.severity === 'critical').length,
      highFindings: findings.filter(f => f.severity === 'high').length,
      selectedCount: selectedFindings.length,
      categoriesCount: new Set(findings.map(f => f.detector)).size,
      filesAnalyzed: new Set(findings.map(f => f.file)).size,
    };
  }, [report.findings, selectedFindings]);

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  // Simulate export functionality
  const handleExport = useCallback((format: string) => {
    console.log(`Exporting dashboard in ${format} format`);
    // In real implementation, this would trigger actual export
  }, []);

  const tabsData = [
    {
      label: 'Overview',
      icon: <DashboardOutlined />,
      description: 'Priority Matrix & Quality Overview',
    },
    {
      label: 'Quality Deep Dive',
      icon: <AssessmentOutlined />,
      description: 'Detailed Quality Metrics',
    },
    {
      label: 'Coming Soon',
      icon: <BuildOutlined />,
      description: 'Advanced Analytics & Collaboration',
      disabled: true,
    },
  ];

  return (
    <Box sx={{ width: '100%', bgcolor: 'background.default', minHeight: '100vh' }}>
      {/* Enhanced Header */}
      <Paper 
        sx={{ 
          p: 4, 
          mb: 3,
          background: `linear-gradient(135deg, ${dashboardTheme.colors.primary[0]} 0%, ${dashboardTheme.colors.primary[1]} 100%)`,
          color: 'white',
          borderRadius: 2,
        }}
      >
        <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
          <Box>
            <Typography variant="h4" component="h1" fontWeight="700" mb={1}>
              Enhanced Dashboard Demo
            </Typography>
            <Typography variant="h6" sx={{ opacity: 0.9 }}>
              {report.project?.name || 'Code Analysis Project'}
            </Typography>
          </Box>
          
          <Box display="flex" gap={1}>
            <Chip 
              icon={<AutoAwesomeOutlined />}
              label="Phase 1 Implementation"
              sx={{
                backgroundColor: 'rgba(255, 255, 255, 0.2)',
                color: 'white',
                fontWeight: 600,
              }}
            />
          </Box>
        </Box>

        {/* Statistics Bar */}
        <Grid container spacing={2} sx={{ mt: 2 }}>
          <Grid item xs={6} sm={3}>
            <Box textAlign="center">
              <Typography variant="h5" fontWeight="700">
                {demoStats.totalFindings}
              </Typography>
              <Typography variant="body2" sx={{ opacity: 0.8 }}>
                Total Findings
              </Typography>
            </Box>
          </Grid>
          <Grid item xs={6} sm={3}>
            <Box textAlign="center">
              <Typography variant="h5" fontWeight="700" color={dashboardTheme.colors.error[0]}>
                {demoStats.criticalFindings}
              </Typography>
              <Typography variant="body2" sx={{ opacity: 0.8 }}>
                Critical Issues
              </Typography>
            </Box>
          </Grid>
          <Grid item xs={6} sm={3}>
            <Box textAlign="center">
              <Typography variant="h5" fontWeight="700">
                {demoStats.filesAnalyzed}
              </Typography>
              <Typography variant="body2" sx={{ opacity: 0.8 }}>
                Files Analyzed
              </Typography>
            </Box>
          </Grid>
          <Grid item xs={6} sm={3}>
            <Box textAlign="center">
              <Typography variant="h5" fontWeight="700">
                {demoStats.selectedCount}
              </Typography>
              <Typography variant="body2" sx={{ opacity: 0.8 }}>
                Selected Items
              </Typography>
            </Box>
          </Grid>
        </Grid>
      </Paper>

      {/* Phase Implementation Alert */}
      <Alert severity="info" sx={{ mb: 3 }} icon={<TrendingUpOutlined />}>
        <AlertTitle>Implementation Progress</AlertTitle>
        <strong>Phase 1 Complete:</strong> Priority Matrix & Quality Score Card are fully functional.
        <br />
        <strong>Coming in Phase 2:</strong> Interactive Dependency Graph & Technical Debt Tracker
        <br />
        <strong>Future Phases:</strong> Smart Search, Fix Suggestions, Historical Comparison, Collaboration Tools
      </Alert>

      {/* Navigation Tabs */}
      <Paper sx={{ mb: 3 }}>
        <Tabs 
          value={activeTab} 
          onChange={handleTabChange}
          variant="fullWidth"
          sx={{
            '& .MuiTab-root': {
              minHeight: 80,
              flexDirection: 'column',
              gap: 1,
            },
          }}
        >
          {tabsData.map((tab, index) => (
            <Tab
              key={index}
              icon={tab.icon}
              label={
                <Box>
                  <Typography variant="subtitle2" fontWeight="600">
                    {tab.label}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {tab.description}
                  </Typography>
                </Box>
              }
              disabled={tab.disabled}
            />
          ))}
        </Tabs>
      </Paper>

      {/* Tab Content */}
      <TabPanel value={activeTab} index={0}>
        <Grid container spacing={3}>
          {/* Priority Matrix */}
          <Grid item xs={12} lg={8}>
            <PriorityMatrix
              findings={report.findings || []}
              onSelectionChange={handlePriorityMatrixSelection}
              onQuadrantFilter={handleQuadrantFilter}
              interactive={true}
              showQuadrantLabels={true}
              showLegend={true}
              exportable={true}
              onExport={handleExport}
              height={500}
            />
          </Grid>

          {/* Quality Score Card */}
          <Grid item xs={12} lg={4}>
            <QualityScoreCard
              report={report}
              showTrends={true}
              showBenchmarks={true}
              showBreakdown={false}
              onMetricClick={handleQualityMetricClick}
              exportable={true}
              onExport={handleExport}
            />
          </Grid>
        </Grid>
      </TabPanel>

      <TabPanel value={activeTab} index={1}>
        <Grid container spacing={3}>
          {/* Expanded Quality Score Card */}
          <Grid item xs={12}>
            <QualityScoreCard
              report={report}
              showTrends={true}
              showBenchmarks={true}
              showBreakdown={true}
              detailed={true}
              onMetricClick={handleQualityMetricClick}
              exportable={true}
              onExport={handleExport}
            />
          </Grid>
        </Grid>
      </TabPanel>

      <TabPanel value={activeTab} index={2}>
        <Box textAlign="center" py={6}>
          <Box sx={{ 
            fontSize: 64, 
            mb: 2,
            color: theme.palette.text.secondary,
            opacity: 0.5,
          }}>
            🚧
          </Box>
          <Typography variant="h4" color="text.secondary" gutterBottom>
            Advanced Features Coming Soon
          </Typography>
          <Typography variant="body1" color="text.secondary" mb={4}>
            Phase 2-5 components are in development and will include:
          </Typography>
          
          <Grid container spacing={2} sx={{ maxWidth: 800, mx: 'auto' }}>
            {[
              'Interactive Dependency Graph with clustering',
              'Technical Debt Tracker with trend analysis', 
              'AI-Powered Fix Suggestions',
              'Smart Search with NLP',
              'Historical Comparison & Trends',
              'Team Collaboration Features',
              'Drag-and-Drop Dashboard Builder',
              'Comprehensive Export Hub'
            ].map((feature, index) => (
              <Grid item xs={12} sm={6} key={index}>
                <Paper 
                  sx={{ 
                    p: 2, 
                    textAlign: 'left',
                    backgroundColor: 'action.hover',
                    border: `1px solid ${theme.palette.divider}`,
                  }}
                >
                  <Typography variant="body2">
                    {feature}
                  </Typography>
                </Paper>
              </Grid>
            ))}
          </Grid>

          <Button 
            variant="contained" 
            sx={{ mt: 4 }}
            disabled
          >
            Stay Tuned for Updates
          </Button>
        </Box>
      </TabPanel>

      {/* Selection Summary */}
      {selectedFindings.length > 0 && (
        <Paper sx={{ mt: 3, p: 2, backgroundColor: 'action.hover' }}>
          <Typography variant="subtitle1" fontWeight="600" mb={1}>
            Selection Summary
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {selectedFindings.length} findings selected
            {selectedQuadrant && ` from ${selectedQuadrant} quadrant`}
            {' • '}Click items in the Priority Matrix to see cross-component communication in action
          </Typography>
        </Paper>
      )}
    </Box>
  );
};

export default EnhancedDashboardDemo;