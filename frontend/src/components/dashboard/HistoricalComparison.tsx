import React, { useMemo, useCallback, useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  ButtonGroup,
  Chip,
  Paper,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Select,
  MenuItem,
  FormControl,
  FormLabel,
  Alert,
  LinearProgress,
  Avatar,
  Divider,
  useTheme,
  Tabs,
  Tab,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Collapse,
} from '@mui/material';
import {
  TrendingUpOutlined,
  TrendingDownOutlined,
  TrendingFlatOutlined,
  CompareArrowsOutlined,
  HistoryOutlined,
  CalendarTodayOutlined,
  AssessmentOutlined,
  ExpandMoreOutlined,
  ExpandLessOutlined,
  InfoOutlined,
  ErrorOutlined,
  WarningOutlined,
  CheckCircleOutlined,
  TimelineOutlined,
  InsightsOutlined,
  DownloadOutlined,
  GitCommitOutlined,
  VersionsOutlined,
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
  TimeScale,
} from 'chart.js';
import { Line, Bar } from 'react-chartjs-2';
import { format, parseISO, subDays, differenceInDays } from 'date-fns';
import 'chartjs-adapter-date-fns';

import type { Finding, InteractiveReport } from '../../types/api';
import type {
  HistoricalSnapshot,
  ComparisonResult,
  ChangeMetric,
  TrendAnalysis,
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
  TimeScale
);

interface HistoricalComparisonProps extends ResponsiveComponentProps, ExportableComponentProps {
  currentReport: InteractiveReport;
  historicalSnapshots?: HistoricalSnapshot[];
  timeRange?: '7d' | '30d' | '90d' | '1y';
  comparisonMode?: 'timeline' | 'comparison';
  showPredictions?: boolean;
  height?: number;
}

// Generate mock historical data
const generateHistoricalSnapshots = (
  currentReport: InteractiveReport,
  timeRange: '7d' | '30d' | '90d' | '1y'
): HistoricalSnapshot[] => {
  const days = timeRange === '7d' ? 7 : timeRange === '30d' ? 30 : timeRange === '90d' ? 90 : 365;
  const today = new Date();
  const snapshots: HistoricalSnapshot[] = [];

  // Generate snapshots at irregular intervals to simulate real data
  const intervals = timeRange === '7d' ? [1, 2, 3, 5, 7] : 
                   timeRange === '30d' ? [3, 7, 14, 21, 28, 30] :
                   timeRange === '90d' ? [7, 21, 45, 60, 75, 90] :
                   [30, 90, 180, 270, 365];

  intervals.forEach((daysAgo, index) => {
    const date = subDays(today, daysAgo);
    const currentSummary = currentReport.summary;
    
    // Simulate gradual improvement or degradation
    const trendFactor = 1 + (Math.random() - 0.7) * 0.3; // Slight bias toward improvement
    const daysFactor = 1 - (daysAgo / (days * 2)); // Older data is generally worse
    
    snapshots.unshift({
      id: `snapshot_${daysAgo}d_ago`,
      timestamp: date.toISOString(),
      projectVersion: `v1.${Math.max(0, 10 - Math.floor(daysAgo / 30))}.0`,
      commit: `abc${(1234 + index * 11).toString(16)}`,
      summary: {
        totalFindings: Math.max(0, Math.round((currentSummary?.issuesTotal || 50) * trendFactor * (1 + daysFactor * 0.5))),
        qualityScore: Math.min(100, Math.max(0, Math.round((currentSummary?.coverage || 75) * (0.8 + daysFactor * 0.2)))),
        debtScore: Math.min(100, Math.max(0, Math.round(
          dataTransformers.technicalDebt.calculateScore(currentReport.findings || []) * trendFactor
        ))),
        coverage: Math.min(100, Math.max(0, Math.round((currentSummary?.coverage || 75) * (0.7 + daysFactor * 0.3)))),
      },
      categoryBreakdown: Object.fromEntries(
        Object.entries(currentSummary?.issuesByCategory || {}).map(([key, value]) => [
          key,
          Math.max(0, Math.round(value * trendFactor * (1 + daysFactor * 0.3)))
        ])
      ),
      severityBreakdown: Object.fromEntries(
        Object.entries(currentSummary?.issuesBySeverity || {}).map(([key, value]) => [
          key,
          Math.max(0, Math.round(value * trendFactor * (1 + daysFactor * 0.4)))
        ])
      ),
    });
  });

  return snapshots;
};

// Analyze comparison between two snapshots
const analyzeComparison = (
  current: HistoricalSnapshot,
  previous: HistoricalSnapshot,
  currentFindings?: Finding[]
): ComparisonResult => {
  const calculateChange = (current: number, previous: number): ChangeMetric => {
    const absolute = current - previous;
    const relative = previous !== 0 ? (absolute / previous) * 100 : 0;
    const direction = absolute > 5 ? 'degraded' : absolute < -5 ? 'improved' : 'stable';
    
    return { absolute, relative, direction };
  };

  const changes = {
    totalFindings: calculateChange(current.summary.totalFindings, previous.summary.totalFindings),
    qualityScore: calculateChange(current.summary.qualityScore, previous.summary.qualityScore),
    debtScore: calculateChange(current.summary.debtScore, previous.summary.debtScore),
    coverage: calculateChange(current.summary.coverage, previous.summary.coverage),
    newIssues: [], // Would be populated with actual new findings
    resolvedIssues: [], // Would be populated with resolved findings
    categoryChanges: Object.fromEntries(
      Object.keys({ ...current.categoryBreakdown, ...previous.categoryBreakdown }).map(category => [
        category,
        calculateChange(
          current.categoryBreakdown[category] || 0,
          previous.categoryBreakdown[category] || 0
        )
      ])
    ),
  };

  // Generate trend analysis
  const trends: TrendAnalysis[] = [
    {
      metric: 'Quality Score',
      trend: changes.qualityScore.direction === 'improved' ? 'improving' : 
             changes.qualityScore.direction === 'degraded' ? 'worsening' : 'stable',
      confidence: 0.85,
      prediction: current.summary.qualityScore + (changes.qualityScore.absolute * 0.5),
      timeframe: '30 days',
    },
    {
      metric: 'Technical Debt',
      trend: changes.debtScore.direction === 'improved' ? 'improving' : 
             changes.debtScore.direction === 'degraded' ? 'worsening' : 'stable',
      confidence: 0.78,
      prediction: current.summary.debtScore + (changes.debtScore.absolute * 0.3),
      timeframe: '30 days',
    },
  ];

  return {
    current,
    previous,
    changes,
    trends,
  };
};

const HistoricalComparison: React.FC<HistoricalComparisonProps> = ({
  currentReport,
  historicalSnapshots,
  timeRange = '30d',
  comparisonMode = 'timeline',
  showPredictions = true,
  height = 400,
  loading = false,
  error,
  className,
  exportable = false,
  onExport,
  testId,
}) => {
  const theme = useTheme();
  const [selectedTimeRange, setSelectedTimeRange] = useState(timeRange);
  const [selectedMode, setSelectedMode] = useState(comparisonMode);
  const [activeTab, setActiveTab] = useState(0);
  const [selectedSnapshot, setSelectedSnapshot] = useState<string | null>(null);
  const [expandedCategories, setExpandedCategories] = useState<string[]>([]);

  // Generate or use provided historical data
  const snapshots = useMemo(() => {
    if (historicalSnapshots && historicalSnapshots.length > 0) {
      return historicalSnapshots.sort((a, b) => 
        parseISO(a.timestamp).getTime() - parseISO(b.timestamp).getTime()
      );
    }
    return generateHistoricalSnapshots(currentReport, selectedTimeRange);
  }, [historicalSnapshots, currentReport, selectedTimeRange]);

  // Create current snapshot
  const currentSnapshot = useMemo(() => {
    return dataTransformers.historical.createSnapshot(currentReport);
  }, [currentReport]);

  // Prepare timeline data
  const timelineData = useMemo(() => {
    const allSnapshots = [...snapshots, currentSnapshot].sort((a, b) => 
      parseISO(a.timestamp).getTime() - parseISO(b.timestamp).getTime()
    );

    const labels = allSnapshots.map(snapshot => parseISO(snapshot.timestamp));
    
    return {
      labels,
      datasets: [
        {
          label: 'Quality Score',
          data: allSnapshots.map(s => ({ x: parseISO(s.timestamp), y: s.summary.qualityScore })),
          borderColor: dashboardTheme.colors.success[1],
          backgroundColor: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.success[1], 0.1),
          fill: false,
          tension: 0.4,
          yAxisID: 'y',
        },
        {
          label: 'Total Findings',
          data: allSnapshots.map(s => ({ x: parseISO(s.timestamp), y: s.summary.totalFindings })),
          borderColor: dashboardTheme.colors.error[1],
          backgroundColor: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.error[1], 0.1),
          fill: false,
          tension: 0.4,
          yAxisID: 'y1',
        },
        {
          label: 'Debt Score',
          data: allSnapshots.map(s => ({ x: parseISO(s.timestamp), y: s.summary.debtScore })),
          borderColor: dashboardTheme.colors.warning[1],
          backgroundColor: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.warning[1], 0.1),
          fill: false,
          tension: 0.4,
          yAxisID: 'y',
        },
      ],
    };
  }, [snapshots, currentSnapshot]);

  // Chart options for timeline
  const timelineOptions = useMemo(() => ({
    responsive: true,
    maintainAspectRatio: false,
    interaction: {
      mode: 'index' as const,
      intersect: false,
    },
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
            return format(tooltipItems[0].parsed.x, 'MMM dd, yyyy');
          },
        },
      },
    },
    scales: {
      x: {
        type: 'time' as const,
        time: {
          displayFormats: {
            day: 'MMM dd',
            week: 'MMM dd',
            month: 'MMM yyyy',
          },
        },
        title: {
          display: true,
          text: 'Date',
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 12,
            weight: '600',
          },
          color: theme.palette.text.primary,
        },
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
        type: 'linear' as const,
        display: true,
        position: 'left' as const,
        title: {
          display: true,
          text: 'Score',
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 12,
            weight: '600',
          },
          color: theme.palette.text.primary,
        },
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
      y1: {
        type: 'linear' as const,
        display: true,
        position: 'right' as const,
        title: {
          display: true,
          text: 'Findings Count',
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 12,
            weight: '600',
          },
          color: theme.palette.text.primary,
        },
        grid: {
          drawOnChartArea: false,
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

  // Comparison analysis
  const comparison = useMemo(() => {
    if (snapshots.length < 2) return null;
    
    const previous = selectedSnapshot 
      ? snapshots.find(s => s.id === selectedSnapshot) || snapshots[snapshots.length - 2]
      : snapshots[snapshots.length - 2];
      
    return analyzeComparison(currentSnapshot, previous, currentReport.findings);
  }, [snapshots, currentSnapshot, selectedSnapshot, currentReport.findings]);

  // Get trend icon and color
  const getTrendIcon = useCallback((trend: string, size: 'small' | 'medium' | 'large' = 'medium') => {
    const fontSize = size === 'small' ? 16 : size === 'medium' ? 24 : 32;
    
    switch (trend) {
      case 'improving':
        return <TrendingUpOutlined sx={{ fontSize, color: 'success.main' }} />;
      case 'worsening':
        return <TrendingDownOutlined sx={{ fontSize, color: 'error.main' }} />;
      default:
        return <TrendingFlatOutlined sx={{ fontSize, color: 'action.active' }} />;
    }
  }, []);

  const getTrendColor = useCallback((trend: string) => {
    return dashboardTheme.colorUtils.getTrendColor(trend as 'improving' | 'stable' | 'worsening');
  }, []);

  // Handle category expansion
  const handleCategoryToggle = useCallback((category: string) => {
    setExpandedCategories(prev => 
      prev.includes(category) 
        ? prev.filter(c => c !== category)
        : [...prev, category]
    );
  }, []);

  // Handle export
  const handleExport = useCallback(() => {
    if (onExport) {
      onExport('json');
    }
  }, [onExport]);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading historical comparison...
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
                Error loading comparison: {error}
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
            Historical Comparison
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Trend analysis and progress tracking over time
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          {/* Time range selector */}
          <FormControl size="small" sx={{ minWidth: 80 }}>
            <Select
              value={selectedTimeRange}
              onChange={(e) => setSelectedTimeRange(e.target.value as any)}
            >
              <MenuItem value="7d">7d</MenuItem>
              <MenuItem value="30d">30d</MenuItem>
              <MenuItem value="90d">90d</MenuItem>
              <MenuItem value="1y">1y</MenuItem>
            </Select>
          </FormControl>

          {/* Mode selector */}
          <ButtonGroup size="small" variant="outlined">
            <Button
              variant={selectedMode === 'timeline' ? 'contained' : 'outlined'}
              onClick={() => setSelectedMode('timeline')}
              startIcon={<TimelineOutlined />}
            >
              Timeline
            </Button>
            <Button
              variant={selectedMode === 'comparison' ? 'contained' : 'outlined'}
              onClick={() => setSelectedMode('comparison')}
              startIcon={<CompareArrowsOutlined />}
            >
              Compare
            </Button>
          </ButtonGroup>

          {exportable && (
            <Tooltip title="Export comparison data" arrow>
              <IconButton onClick={handleExport}>
                <DownloadOutlined />
              </IconButton>
            </Tooltip>
          )}

          <Tooltip title="View historical trends and compare snapshots" arrow>
            <IconButton>
              <InfoOutlined />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {snapshots.length === 0 ? (
        <Card>
          <CardContent>
            <Box display="flex" flexDirection="column" alignItems="center" py={4}>
              <HistoryOutlined sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
              <Typography variant="h6" color="text.secondary" mb={1}>
                No Historical Data Available
              </Typography>
              <Typography variant="body2" color="text.secondary" textAlign="center">
                Historical snapshots will appear here as you run analyses over time.
              </Typography>
            </Box>
          </CardContent>
        </Card>
      ) : (
        <>
          {selectedMode === 'timeline' && (
            <Card>
              <CardContent>
                <Typography variant="h6" fontWeight="600" mb={2}>
                  Metrics Over Time
                </Typography>
                <Box height={height}>
                  <Line data={timelineData} options={timelineOptions} />
                </Box>
              </CardContent>
            </Card>
          )}

          {selectedMode === 'comparison' && comparison && (
            <Grid container spacing={2}>
              {/* Snapshot selector */}
              <Grid item xs={12}>
                <Paper sx={{ p: 2, mb: 2, bgcolor: 'action.hover' }}>
                  <Box display="flex" alignItems="center" gap={2} mb={2}>
                    <Typography variant="subtitle2" fontWeight="600">
                      Compare Against:
                    </Typography>
                    <FormControl size="small" sx={{ minWidth: 200 }}>
                      <Select
                        value={selectedSnapshot || snapshots[snapshots.length - 2]?.id || ''}
                        onChange={(e) => setSelectedSnapshot(e.target.value)}
                      >
                        {snapshots.map(snapshot => (
                          <MenuItem key={snapshot.id} value={snapshot.id}>
                            {format(parseISO(snapshot.timestamp), 'MMM dd, yyyy')} 
                            {snapshot.projectVersion && ` (${snapshot.projectVersion})`}
                          </MenuItem>
                        ))}
                      </Select>
                    </FormControl>
                  </Box>

                  <Box display="flex" alignItems="center" gap={4}>
                    <Box display="flex" alignItems="center" gap={1}>
                      <Avatar sx={{ bgcolor: 'primary.light', width: 24, height: 24 }}>
                        <CalendarTodayOutlined fontSize="small" />
                      </Avatar>
                      <Typography variant="body2">
                        <strong>Current:</strong> {format(parseISO(comparison.current.timestamp), 'MMM dd, yyyy')}
                      </Typography>
                    </Box>
                    <Box display="flex" alignItems="center" gap={1}>
                      <Avatar sx={{ bgcolor: 'secondary.light', width: 24, height: 24 }}>
                        <HistoryOutlined fontSize="small" />
                      </Avatar>
                      <Typography variant="body2">
                        <strong>Previous:</strong> {format(parseISO(comparison.previous.timestamp), 'MMM dd, yyyy')}
                      </Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary">
                      ({differenceInDays(parseISO(comparison.current.timestamp), parseISO(comparison.previous.timestamp))} days difference)
                    </Typography>
                  </Box>
                </Paper>
              </Grid>

              {/* Key metrics comparison */}
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" fontWeight="600" mb={2}>
                      Key Metrics Comparison
                    </Typography>
                    
                    <Grid container spacing={2}>
                      {/* Quality Score */}
                      <Grid item xs={12} sm={6} md={3}>
                        <Paper sx={{ p: 2, textAlign: 'center', border: '1px solid', borderColor: 'divider' }}>
                          <Typography variant="caption" color="text.secondary" mb={1} display="block">
                            Quality Score
                          </Typography>
                          <Box display="flex" alignItems="center" justifyContent="center" gap={1} mb={1}>
                            <Typography variant="h5" fontWeight="700">
                              {comparison.current.summary.qualityScore}
                            </Typography>
                            {getTrendIcon(comparison.changes.qualityScore.direction, 'small')}
                          </Box>
                          <Typography 
                            variant="body2" 
                            color={getTrendColor(comparison.changes.qualityScore.direction)}
                            fontWeight="600"
                          >
                            {comparison.changes.qualityScore.absolute > 0 ? '+' : ''}{comparison.changes.qualityScore.absolute.toFixed(1)}
                            {' '}({comparison.changes.qualityScore.relative.toFixed(1)}%)
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            vs {comparison.previous.summary.qualityScore}
                          </Typography>
                        </Paper>
                      </Grid>

                      {/* Total Findings */}
                      <Grid item xs={12} sm={6} md={3}>
                        <Paper sx={{ p: 2, textAlign: 'center', border: '1px solid', borderColor: 'divider' }}>
                          <Typography variant="caption" color="text.secondary" mb={1} display="block">
                            Total Findings
                          </Typography>
                          <Box display="flex" alignItems="center" justifyContent="center" gap={1} mb={1}>
                            <Typography variant="h5" fontWeight="700">
                              {comparison.current.summary.totalFindings}
                            </Typography>
                            {getTrendIcon(comparison.changes.totalFindings.direction === 'improved' ? 'worsening' : 
                                         comparison.changes.totalFindings.direction === 'degraded' ? 'improving' : 'stable', 'small')}
                          </Box>
                          <Typography 
                            variant="body2" 
                            color={getTrendColor(comparison.changes.totalFindings.direction === 'improved' ? 'worsening' : 
                                               comparison.changes.totalFindings.direction === 'degraded' ? 'improving' : 'stable')}
                            fontWeight="600"
                          >
                            {comparison.changes.totalFindings.absolute > 0 ? '+' : ''}{comparison.changes.totalFindings.absolute}
                            {' '}({comparison.changes.totalFindings.relative.toFixed(1)}%)
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            vs {comparison.previous.summary.totalFindings}
                          </Typography>
                        </Paper>
                      </Grid>

                      {/* Debt Score */}
                      <Grid item xs={12} sm={6} md={3}>
                        <Paper sx={{ p: 2, textAlign: 'center', border: '1px solid', borderColor: 'divider' }}>
                          <Typography variant="caption" color="text.secondary" mb={1} display="block">
                            Technical Debt
                          </Typography>
                          <Box display="flex" alignItems="center" justifyContent="center" gap={1} mb={1}>
                            <Typography variant="h5" fontWeight="700">
                              {comparison.current.summary.debtScore}
                            </Typography>
                            {getTrendIcon(comparison.changes.debtScore.direction === 'improved' ? 'worsening' : 
                                         comparison.changes.debtScore.direction === 'degraded' ? 'improving' : 'stable', 'small')}
                          </Box>
                          <Typography 
                            variant="body2" 
                            color={getTrendColor(comparison.changes.debtScore.direction === 'improved' ? 'worsening' : 
                                               comparison.changes.debtScore.direction === 'degraded' ? 'improving' : 'stable')}
                            fontWeight="600"
                          >
                            {comparison.changes.debtScore.absolute > 0 ? '+' : ''}{comparison.changes.debtScore.absolute.toFixed(1)}
                            {' '}({comparison.changes.debtScore.relative.toFixed(1)}%)
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            vs {comparison.previous.summary.debtScore}
                          </Typography>
                        </Paper>
                      </Grid>

                      {/* Coverage */}
                      <Grid item xs={12} sm={6} md={3}>
                        <Paper sx={{ p: 2, textAlign: 'center', border: '1px solid', borderColor: 'divider' }}>
                          <Typography variant="caption" color="text.secondary" mb={1} display="block">
                            Coverage
                          </Typography>
                          <Box display="flex" alignItems="center" justifyContent="center" gap={1} mb={1}>
                            <Typography variant="h5" fontWeight="700">
                              {comparison.current.summary.coverage}%
                            </Typography>
                            {getTrendIcon(comparison.changes.coverage.direction, 'small')}
                          </Box>
                          <Typography 
                            variant="body2" 
                            color={getTrendColor(comparison.changes.coverage.direction)}
                            fontWeight="600"
                          >
                            {comparison.changes.coverage.absolute > 0 ? '+' : ''}{comparison.changes.coverage.absolute.toFixed(1)}%
                            {' '}({comparison.changes.coverage.relative.toFixed(1)}%)
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            vs {comparison.previous.summary.coverage}%
                          </Typography>
                        </Paper>
                      </Grid>
                    </Grid>
                  </CardContent>
                </Card>
              </Grid>

              {/* Category breakdown comparison */}
              <Grid item xs={12} md={8}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" fontWeight="600" mb={2}>
                      Category Changes
                    </Typography>

                    <List dense>
                      {Object.entries(comparison.changes.categoryChanges).map(([category, change]) => (
                        <ListItem key={category} sx={{ px: 0 }}>
                          <ListItemIcon>
                            <Avatar
                              sx={{
                                bgcolor: dashboardTheme.colorUtils.getCategoryColorByName(category),
                                width: 32,
                                height: 32,
                              }}
                            >
                              {change.direction === 'improved' ? <CheckCircleOutlined /> :
                               change.direction === 'degraded' ? <ErrorOutlined /> : <InfoOutlined />}
                            </Avatar>
                          </ListItemIcon>
                          <ListItemText
                            primary={
                              <Box display="flex" justifyContent="space-between" alignItems="center">
                                <Typography variant="body2" fontWeight="600">
                                  {category.replace('_', ' ')}
                                </Typography>
                                <Box display="flex" alignItems="center" gap={1}>
                                  <Typography 
                                    variant="body2" 
                                    color={getTrendColor(change.direction)}
                                    fontWeight="600"
                                  >
                                    {change.absolute > 0 ? '+' : ''}{change.absolute}
                                  </Typography>
                                  {getTrendIcon(change.direction === 'improved' ? 'worsening' : 
                                               change.direction === 'degraded' ? 'improving' : 'stable', 'small')}
                                </Box>
                              </Box>
                            }
                            secondary={
                              <Box>
                                <Typography variant="caption" color="text.secondary">
                                  Current: {comparison.current.categoryBreakdown[category] || 0} • 
                                  Previous: {comparison.previous.categoryBreakdown[category] || 0}
                                </Typography>
                                <LinearProgress
                                  variant="determinate"
                                  value={Math.min(100, Math.abs(change.relative))}
                                  sx={{ 
                                    mt: 0.5,
                                    height: 4,
                                    borderRadius: 2,
                                    backgroundColor: 'action.hover',
                                    '& .MuiLinearProgress-bar': {
                                      backgroundColor: getTrendColor(change.direction === 'improved' ? 'worsening' : 
                                                                   change.direction === 'degraded' ? 'improving' : 'stable'),
                                    },
                                  }}
                                />
                              </Box>
                            }
                          />
                        </ListItem>
                      ))}
                    </List>
                  </CardContent>
                </Card>
              </Grid>

              {/* Trends and predictions */}
              <Grid item xs={12} md={4}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" fontWeight="600" mb={2}>
                      Trend Analysis
                    </Typography>

                    {comparison.trends.map((trend, index) => (
                      <Paper
                        key={trend.metric}
                        sx={{
                          p: 2,
                          mb: 2,
                          border: '1px solid',
                          borderColor: 'divider',
                          borderLeft: `4px solid ${getTrendColor(trend.trend)}`,
                        }}
                      >
                        <Box display="flex" alignItems="center" justifyContent="between" mb={1}>
                          <Typography variant="subtitle2" fontWeight="600">
                            {trend.metric}
                          </Typography>
                          {getTrendIcon(trend.trend, 'small')}
                        </Box>

                        <Typography 
                          variant="body2" 
                          color={getTrendColor(trend.trend)}
                          fontWeight="600"
                          mb={0.5}
                        >
                          {trend.trend.charAt(0).toUpperCase() + trend.trend.slice(1)}
                        </Typography>

                        <Typography variant="caption" color="text.secondary" display="block" mb={1}>
                          Confidence: {(trend.confidence * 100).toFixed(0)}% • {trend.timeframe}
                        </Typography>

                        {showPredictions && trend.prediction && (
                          <Alert severity="info" sx={{ mt: 1 }}>
                            <Typography variant="caption">
                              <strong>Prediction:</strong> {trend.prediction.toFixed(1)} in {trend.timeframe}
                            </Typography>
                          </Alert>
                        )}
                      </Paper>
                    ))}

                    {/* Overall assessment */}
                    <Divider sx={{ my: 2 }} />
                    <Box textAlign="center">
                      <Typography variant="subtitle2" fontWeight="600" mb={1}>
                        Overall Progress
                      </Typography>
                      {comparison.changes.qualityScore.direction === 'improved' ? (
                        <Alert severity="success">
                          <Typography variant="body2">
                            Quality improvements detected! Keep up the good work.
                          </Typography>
                        </Alert>
                      ) : comparison.changes.qualityScore.direction === 'degraded' ? (
                        <Alert severity="warning">
                          <Typography variant="body2">
                            Quality decline detected. Consider reviewing recent changes.
                          </Typography>
                        </Alert>
                      ) : (
                        <Alert severity="info">
                          <Typography variant="body2">
                            Quality remains stable with minor variations.
                          </Typography>
                        </Alert>
                      )}
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          )}

          {/* Snapshot history table */}
          <Box mt={3}>
            <Card>
              <CardContent>
                <Typography variant="h6" fontWeight="600" mb={2}>
                  Snapshot History
                </Typography>
                
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Date</TableCell>
                      <TableCell>Version</TableCell>
                      <TableCell align="right">Quality</TableCell>
                      <TableCell align="right">Findings</TableCell>
                      <TableCell align="right">Debt</TableCell>
                      <TableCell align="right">Coverage</TableCell>
                      <TableCell>Trend</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {[currentSnapshot, ...snapshots].reverse().map((snapshot, index) => {
                      const isLatest = index === 0;
                      const previousSnapshot = index < snapshots.length ? 
                        (index === 0 ? snapshots[snapshots.length - 1] : snapshots[snapshots.length - index - 1]) : null;
                      
                      const qualityChange = previousSnapshot ? 
                        dataTransformers.utils.calculateTrend(snapshot.summary.qualityScore, previousSnapshot.summary.qualityScore) : 'stable';

                      return (
                        <TableRow 
                          key={snapshot.id}
                          sx={{ 
                            bgcolor: isLatest ? 'action.hover' : 'transparent',
                            fontWeight: isLatest ? 600 : 400,
                          }}
                        >
                          <TableCell>
                            <Box display="flex" alignItems="center" gap={1}>
                              {isLatest && <Chip label="current" size="small" color="primary" />}
                              <Typography variant="body2" fontWeight={isLatest ? 600 : 400}>
                                {format(parseISO(snapshot.timestamp), 'MMM dd, yyyy')}
                              </Typography>
                            </Box>
                          </TableCell>
                          <TableCell>
                            <Box display="flex" alignItems="center" gap={0.5}>
                              {snapshot.commit && <GitCommitOutlined fontSize="small" color="action" />}
                              <Typography variant="body2">
                                {snapshot.projectVersion || 'N/A'}
                              </Typography>
                            </Box>
                          </TableCell>
                          <TableCell align="right">
                            <Typography variant="body2" fontWeight={isLatest ? 600 : 400}>
                              {snapshot.summary.qualityScore}
                            </Typography>
                          </TableCell>
                          <TableCell align="right">
                            <Typography variant="body2" fontWeight={isLatest ? 600 : 400}>
                              {snapshot.summary.totalFindings}
                            </Typography>
                          </TableCell>
                          <TableCell align="right">
                            <Typography variant="body2" fontWeight={isLatest ? 600 : 400}>
                              {snapshot.summary.debtScore}
                            </Typography>
                          </TableCell>
                          <TableCell align="right">
                            <Typography variant="body2" fontWeight={isLatest ? 600 : 400}>
                              {snapshot.summary.coverage}%
                            </Typography>
                          </TableCell>
                          <TableCell>
                            {!isLatest && getTrendIcon(qualityChange, 'small')}
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </Box>
        </>
      )}
    </Box>
  );
};

export default HistoricalComparison;