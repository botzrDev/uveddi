import React, { useMemo, useCallback, useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  LinearProgress,
  Chip,
  Paper,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Tabs,
  Tab,
  Alert,
  useTheme,
  Divider,
  Avatar,
} from '@mui/material';
import {
  TrendingUpOutlined,
  TrendingDownOutlined,
  TrendingFlatOutlined,
  WarningOutlined,
  ErrorOutlined,
  CheckCircleOutlined,
  InfoOutlined,
  DownloadOutlined,
  AccessTimeOutlined,
  BugReportOutlined,
  SecurityOutlined,
  SpeedOutlined,
  BuildOutlined,
} from '@mui/icons-material';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip as ChartTooltip,
  Legend,
  ArcElement,
} from 'chart.js';
import { Line, Bar, Doughnut } from 'react-chartjs-2';
import { format, subDays, parseISO } from 'date-fns';

import type { InteractiveReport } from '../../types/api';
import type {
  TechnicalDebtSummary,
  TechnicalDebtMetric,
  DebtEvolutionPoint,
  ResponsiveComponentProps,
  ExportableComponentProps,
} from '../../types/dashboard';

import { dataTransformers } from '../../utils/dataTransformers';
import { dashboardTheme } from '../../utils/dashboardTheme';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  ChartTooltip,
  Legend,
  ArcElement
);

interface TechnicalDebtTrackerProps extends ResponsiveComponentProps, ExportableComponentProps {
  report: InteractiveReport;
  historicalData?: DebtEvolutionPoint[];
  showTrends?: boolean;
  showMetrics?: boolean;
  showCategories?: boolean;
  timeRange?: '7d' | '30d' | '90d' | '1y';
  height?: number;
}

const TechnicalDebtTracker: React.FC<TechnicalDebtTrackerProps> = ({
  report,
  historicalData = [],
  showTrends = true,
  showMetrics = true,
  showCategories = true,
  timeRange = '30d',
  height = 400,
  loading = false,
  error,
  className,
  exportable = false,
  onExport,
  testId,
}) => {
  const theme = useTheme();
  const [activeTab, setActiveTab] = useState(0);

  // Transform report data to debt summary
  const debtSummary = useMemo(() => {
    return dataTransformers.technicalDebt.transformSummary(report, historicalData);
  }, [report, historicalData]);

  // Generate mock historical data if none provided
  const evolutionData = useMemo(() => {
    if (historicalData.length > 0) {
      return historicalData;
    }

    // Generate sample evolution data
    const days = timeRange === '7d' ? 7 : timeRange === '30d' ? 30 : timeRange === '90d' ? 90 : 365;
    const today = new Date();
    
    return Array.from({ length: days }, (_, i) => {
      const date = subDays(today, days - i - 1);
      const baseDebt = debtSummary.totalDebtScore;
      const variance = (Math.random() - 0.5) * 20;
      
      return {
        timestamp: date.toISOString(),
        totalDebt: Math.max(0, baseDebt + variance + (Math.random() - 0.5) * 10),
        newDebt: Math.max(0, Math.random() * 5),
        resolvedDebt: Math.max(0, Math.random() * 3),
        categories: debtSummary.topCategories.reduce((acc, cat) => {
          acc[cat.name] = Math.max(0, cat.value + (Math.random() - 0.5) * 5);
          return acc;
        }, {} as Record<string, number>),
      } as DebtEvolutionPoint;
    });
  }, [historicalData, timeRange, debtSummary]);

  // Chart data for debt evolution
  const evolutionChartData = useMemo(() => {
    const labels = evolutionData.map(point => 
      format(parseISO(point.timestamp), timeRange === '7d' ? 'MMM dd' : 'MMM dd')
    );

    return {
      labels,
      datasets: [
        {
          label: 'Total Debt',
          data: evolutionData.map(point => point.totalDebt),
          borderColor: dashboardTheme.colors.debt.trend.worsening,
          backgroundColor: dashboardTheme.colorUtils.withAlpha(
            dashboardTheme.colors.debt.trend.worsening, 0.1
          ),
          fill: true,
          tension: 0.4,
        },
        {
          label: 'New Debt',
          data: evolutionData.map(point => point.newDebt),
          borderColor: dashboardTheme.colors.warning[1],
          backgroundColor: dashboardTheme.colorUtils.withAlpha(
            dashboardTheme.colors.warning[1], 0.1
          ),
          fill: false,
          tension: 0.4,
        },
        {
          label: 'Resolved Debt',
          data: evolutionData.map(point => point.resolvedDebt),
          borderColor: dashboardTheme.colors.debt.trend.improving,
          backgroundColor: dashboardTheme.colorUtils.withAlpha(
            dashboardTheme.colors.debt.trend.improving, 0.1
          ),
          fill: false,
          tension: 0.4,
        },
      ],
    };
  }, [evolutionData, timeRange]);

  // Chart data for category breakdown
  const categoryChartData = useMemo(() => {
    const colors = dashboardTheme.colorUtils.generatePalette(
      debtSummary.topCategories.length, 
      'categorical'
    );

    return {
      labels: debtSummary.topCategories.map(cat => cat.name),
      datasets: [
        {
          data: debtSummary.topCategories.map(cat => cat.value),
          backgroundColor: colors,
          borderColor: colors.map(color => 
            dashboardTheme.colorUtils.withAlpha(color, 0.8)
          ),
          borderWidth: 2,
        },
      ],
    };
  }, [debtSummary.topCategories]);

  // Chart options
  const evolutionChartOptions = useMemo(() => ({
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      title: {
        display: false,
      },
      legend: {
        position: 'top' as const,
        labels: {
          usePointStyle: true,
          padding: 20,
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 12,
            weight: '500',
          },
        },
      },
      tooltip: {
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        titleColor: '#fff',
        bodyColor: '#fff',
        borderColor: dashboardTheme.colors.primary[0],
        borderWidth: 1,
        cornerRadius: 8,
        padding: 12,
        callbacks: {
          title: (tooltipItems: any[]) => {
            return `Date: ${tooltipItems[0].label}`;
          },
          label: (tooltipItem: any) => {
            return `${tooltipItem.dataset.label}: ${tooltipItem.parsed.y.toFixed(1)}`;
          },
        },
      },
    },
    scales: {
      x: {
        grid: {
          color: theme.palette.divider,
          lineWidth: 1,
        },
        ticks: {
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 11,
          },
          color: theme.palette.text.secondary,
        },
      },
      y: {
        beginAtZero: true,
        grid: {
          color: theme.palette.divider,
          lineWidth: 1,
        },
        ticks: {
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 11,
          },
          color: theme.palette.text.secondary,
        },
      },
    },
  }), [theme]);

  const categoryChartOptions = useMemo(() => ({
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      title: {
        display: false,
      },
      legend: {
        position: 'right' as const,
        labels: {
          usePointStyle: true,
          padding: 15,
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 12,
            weight: '500',
          },
        },
      },
      tooltip: {
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        titleColor: '#fff',
        bodyColor: '#fff',
        borderColor: dashboardTheme.colors.primary[0],
        borderWidth: 1,
        cornerRadius: 8,
        padding: 12,
        callbacks: {
          label: (tooltipItem: any) => {
            const total = debtSummary.topCategories.reduce((sum, cat) => sum + cat.value, 0);
            const percentage = ((tooltipItem.parsed / total) * 100).toFixed(1);
            return `${tooltipItem.label}: ${tooltipItem.parsed} (${percentage}%)`;
          },
        },
      },
    },
  }), [debtSummary.topCategories]);

  // Get trend icon and color
  const getTrendIcon = useCallback((trend: string) => {
    switch (trend) {
      case 'improving':
        return <TrendingDownOutlined color="success" />;
      case 'worsening':
        return <TrendingUpOutlined color="error" />;
      default:
        return <TrendingFlatOutlined color="action" />;
    }
  }, []);

  const getTrendColor = useCallback((trend: string) => {
    return dashboardTheme.colorUtils.getTrendColor(trend as 'improving' | 'stable' | 'worsening');
  }, []);

  // Get category icon
  const getCategoryIcon = useCallback((category: string) => {
    const iconMap: Record<string, React.ReactNode> = {
      'code_quality': <BugReportOutlined />,
      'architecture': <BuildOutlined />,
      'performance': <SpeedOutlined />,
      'security': <SecurityOutlined />,
      'maintainability': <BuildOutlined />,
    };
    return iconMap[category] || <InfoOutlined />;
  }, []);

  // Get metric priority color
  const getPriorityColor = useCallback((priority: string) => {
    switch (priority) {
      case 'high':
        return 'error';
      case 'medium':
        return 'warning';
      case 'low':
        return 'success';
      default:
        return 'default';
    }
  }, []);

  // Handle export
  const handleExport = useCallback(() => {
    if (onExport) {
      onExport('json'); // Default export format
    }
  }, [onExport]);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading technical debt analysis...
            </Typography>
          </Box>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Alert severity="error" sx={{ maxWidth: 400 }}>
              <Typography variant="body2">
                Error loading debt tracker: {error}
              </Typography>
            </Alert>
          </Box>
        </CardContent>
      </Card>
    );
  }

  return (
    <Box className={className} data-testid={testId}>
      {/* Header */}
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box>
          <Typography variant="h5" component="h2" fontWeight="600" color="text.primary">
            Technical Debt Tracker
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Debt accumulation and resolution analysis
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          {exportable && (
            <Tooltip title="Export debt data" arrow>
              <IconButton onClick={handleExport}>
                <DownloadOutlined />
              </IconButton>
            </Tooltip>
          )}
          
          <Tooltip title="Track technical debt over time and by category" arrow>
            <IconButton>
              <InfoOutlined />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Debt overview cards */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        {/* Total debt score */}
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" gap={2}>
                <Avatar
                  sx={{
                    bgcolor: dashboardTheme.colorUtils.getQualityColor(100 - debtSummary.totalDebtScore),
                    width: 48,
                    height: 48,
                  }}
                >
                  <WarningOutlined />
                </Avatar>
                <Box flex={1}>
                  <Typography variant="h4" fontWeight="700" color="text.primary">
                    {debtSummary.totalDebtScore}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Debt Score
                  </Typography>
                  <Box display="flex" alignItems="center" gap={0.5} mt={0.5}>
                    {getTrendIcon(debtSummary.debtTrend)}
                    <Typography 
                      variant="caption" 
                      color={getTrendColor(debtSummary.debtTrend)}
                      fontWeight="600"
                    >
                      {debtSummary.debtTrend}
                    </Typography>
                  </Box>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Critical issues */}
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" gap={2}>
                <Avatar sx={{ bgcolor: 'error.main', width: 48, height: 48 }}>
                  <ErrorOutlined />
                </Avatar>
                <Box flex={1}>
                  <Typography variant="h4" fontWeight="700" color="text.primary">
                    {debtSummary.criticalIssues}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Critical Issues
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    Require immediate attention
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Estimated resolution time */}
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" gap={2}>
                <Avatar sx={{ bgcolor: 'info.main', width: 48, height: 48 }}>
                  <AccessTimeOutlined />
                </Avatar>
                <Box flex={1}>
                  <Typography variant="h4" fontWeight="700" color="text.primary">
                    {debtSummary.estimatedResolutionHours}h
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Est. Resolution
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {Math.round(debtSummary.estimatedResolutionHours / 8)} working days
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Top category */}
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" gap={2}>
                <Avatar sx={{ bgcolor: 'warning.main', width: 48, height: 48 }}>
                  {getCategoryIcon(debtSummary.topCategories[0]?.name || 'default')}
                </Avatar>
                <Box flex={1}>
                  <Typography variant="h4" fontWeight="700" color="text.primary">
                    {debtSummary.topCategories[0]?.value || 0}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    {debtSummary.topCategories[0]?.name || 'No Data'}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {debtSummary.topCategories[0]?.percentage || 0}% of total debt
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Main content tabs */}
      <Card>
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs 
            value={activeTab} 
            onChange={(_, newValue) => setActiveTab(newValue)}
            variant="scrollable"
            scrollButtons="auto"
          >
            {showTrends && <Tab label="Trends" />}
            {showCategories && <Tab label="Categories" />}
            {showMetrics && <Tab label="Metrics" />}
          </Tabs>
        </Box>

        <CardContent>
          {/* Trends tab */}
          {activeTab === 0 && showTrends && (
            <Box>
              <Typography variant="h6" fontWeight="600" mb={2}>
                Debt Evolution Over Time
              </Typography>
              <Box height={height - 100}>
                <Line data={evolutionChartData} options={evolutionChartOptions} />
              </Box>
            </Box>
          )}

          {/* Categories tab */}
          {activeTab === (showTrends ? 1 : 0) && showCategories && (
            <Grid container spacing={3}>
              {/* Category breakdown chart */}
              <Grid item xs={12} md={7}>
                <Typography variant="h6" fontWeight="600" mb={2}>
                  Debt by Category
                </Typography>
                <Box height={height - 100}>
                  <Doughnut data={categoryChartData} options={categoryChartOptions} />
                </Box>
              </Grid>

              {/* Category details */}
              <Grid item xs={12} md={5}>
                <Typography variant="h6" fontWeight="600" mb={2}>
                  Category Details
                </Typography>
                <List dense>
                  {debtSummary.topCategories.map((category, index) => (
                    <ListItem key={category.name} sx={{ px: 0 }}>
                      <ListItemIcon>
                        <Avatar
                          sx={{
                            bgcolor: dashboardTheme.colorUtils.getCategoryColor(index),
                            width: 32,
                            height: 32,
                          }}
                        >
                          {getCategoryIcon(category.name)}
                        </Avatar>
                      </ListItemIcon>
                      <ListItemText
                        primary={
                          <Box display="flex" justifyContent="space-between" alignItems="center">
                            <Typography variant="body2" fontWeight="600">
                              {category.name}
                            </Typography>
                            <Chip 
                              label={`${category.percentage}%`}
                              size="small"
                              color="primary"
                              variant="outlined"
                            />
                          </Box>
                        }
                        secondary={
                          <Box>
                            <Typography variant="caption" color="text.secondary">
                              {category.value} issues
                            </Typography>
                            <LinearProgress
                              variant="determinate"
                              value={category.percentage}
                              sx={{ 
                                mt: 0.5,
                                height: 4,
                                borderRadius: 2,
                                backgroundColor: 'action.hover',
                                '& .MuiLinearProgress-bar': {
                                  backgroundColor: dashboardTheme.colorUtils.getCategoryColor(index),
                                },
                              }}
                            />
                          </Box>
                        }
                      />
                    </ListItem>
                  ))}
                </List>
              </Grid>
            </Grid>
          )}

          {/* Metrics tab */}
          {activeTab === (showTrends && showCategories ? 2 : showTrends || showCategories ? 1 : 0) && showMetrics && (
            <Box>
              <Typography variant="h6" fontWeight="600" mb={2}>
                Debt Metrics
              </Typography>
              
              <Grid container spacing={2}>
                {debtSummary.metrics.map((metric) => (
                  <Grid item xs={12} md={6} key={metric.id}>
                    <Paper
                      sx={{
                        p: 2,
                        border: '1px solid',
                        borderColor: 'divider',
                        borderLeft: `4px solid ${dashboardTheme.colorUtils.getCategoryColorByName(metric.category)}`,
                      }}
                    >
                      <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={1}>
                        <Typography variant="subtitle2" fontWeight="600">
                          {metric.name}
                        </Typography>
                        <Chip
                          label={metric.priority}
                          size="small"
                          color={getPriorityColor(metric.priority) as any}
                          variant="outlined"
                        />
                      </Box>
                      
                      <Box display="flex" alignItems="center" gap={2} mb={2}>
                        <Typography variant="h5" fontWeight="700" color="text.primary">
                          {metric.currentValue}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {metric.unit}
                        </Typography>
                        {getTrendIcon(metric.trend)}
                      </Box>

                      <Box display="flex" justifyContent="space-between" mb={1}>
                        <Typography variant="caption" color="text.secondary">
                          Current
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          Target: {metric.targetValue} {metric.unit}
                        </Typography>
                      </Box>

                      <LinearProgress
                        variant="determinate"
                        value={Math.min((metric.currentValue / metric.targetValue) * 100, 100)}
                        sx={{
                          height: 6,
                          borderRadius: 3,
                          backgroundColor: 'action.hover',
                          '& .MuiLinearProgress-bar': {
                            backgroundColor: dashboardTheme.colorUtils.getCategoryColorByName(metric.category),
                          },
                        }}
                      />
                      
                      <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                        {metric.category.replace('_', ' ')} metric
                      </Typography>
                    </Paper>
                  </Grid>
                ))}
              </Grid>

              {/* Additional insights */}
              {debtSummary.metrics.length > 0 && (
                <Box mt={3}>
                  <Divider sx={{ mb: 2 }} />
                  <Typography variant="h6" fontWeight="600" mb={2}>
                    Key Insights
                  </Typography>
                  
                  <Grid container spacing={2}>
                    <Grid item xs={12} md={6}>
                      <Alert severity="info" sx={{ mb: 2 }}>
                        <Typography variant="body2" fontWeight="600" mb={0.5}>
                          Improvement Opportunities
                        </Typography>
                        <Typography variant="body2">
                          Focus on {debtSummary.topCategories[0]?.name || 'code quality'} issues first - 
                          they represent {debtSummary.topCategories[0]?.percentage || 0}% of your technical debt.
                        </Typography>
                      </Alert>
                    </Grid>
                    
                    <Grid item xs={12} md={6}>
                      <Alert 
                        severity={debtSummary.criticalIssues > 5 ? 'error' : debtSummary.criticalIssues > 2 ? 'warning' : 'success'}
                      >
                        <Typography variant="body2" fontWeight="600" mb={0.5}>
                          Priority Assessment
                        </Typography>
                        <Typography variant="body2">
                          {debtSummary.criticalIssues > 5 
                            ? 'High number of critical issues requiring immediate attention'
                            : debtSummary.criticalIssues > 2
                            ? 'Moderate technical debt levels - plan regular improvement sprints'
                            : 'Good technical debt management - maintain current practices'
                          }
                        </Typography>
                      </Alert>
                    </Grid>
                  </Grid>
                </Box>
              )}
            </Box>
          )}
        </CardContent>
      </Card>
    </Box>
  );
};

export default TechnicalDebtTracker;