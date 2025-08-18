import React, { useMemo, useState, useCallback } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  LinearProgress,
  Chip,
  IconButton,
  Tooltip,
  Collapse,
  Paper,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Divider,
  useTheme,
  alpha,
} from '@mui/material';
import {
  TrendingUpOutlined,
  TrendingDownOutlined,
  TrendingFlatOutlined,
  ExpandMoreOutlined,
  ExpandLessOutlined,
  InfoOutlined,
  StarOutlined,
  WarningOutlined,
  CheckCircleOutlined,
  ErrorOutlined,
} from '@mui/icons-material';
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, CartesianGrid } from 'recharts';

import type { InteractiveReport } from '../../types/api';
import type {
  QualityScoreBreakdown,
  QualityMetric,
  ResponsiveComponentProps,
  ExportableComponentProps,
} from '../../types/dashboard';

import { dataTransformers } from '../../utils/dataTransformers';
import { dashboardTheme } from '../../utils/dashboardTheme';

interface QualityScoreCardProps extends ResponsiveComponentProps, ExportableComponentProps {
  report: InteractiveReport;
  showTrends?: boolean;
  showBenchmarks?: boolean;
  showBreakdown?: boolean;
  onMetricClick?: (metric: QualityMetric) => void;
  detailed?: boolean;
}

const QualityScoreCard: React.FC<QualityScoreCardProps> = ({
  report,
  showTrends = true,
  showBenchmarks = true,
  showBreakdown = false,
  onMetricClick,
  detailed = false,
  loading = false,
  error,
  className,
  exportable = false,
  onExport,
  testId,
}) => {
  const theme = useTheme();
  const [expanded, setExpanded] = useState(showBreakdown);

  // Transform report data to quality breakdown
  const qualityBreakdown = useMemo(() => {
    return dataTransformers.qualityScore.transformBreakdown(report);
  }, [report]);

  // Prepare chart data for category breakdown
  const categoryChartData = useMemo(() => {
    return Object.entries(qualityBreakdown.categories).map(([category, score]) => ({
      name: category.charAt(0).toUpperCase() + category.slice(1),
      score,
      color: dashboardTheme.colorUtils.getCategoryColorByName(category),
    }));
  }, [qualityBreakdown.categories]);

  // Prepare pie chart data for score distribution
  const pieChartData = useMemo(() => {
    const excellent = Object.values(qualityBreakdown.categories).filter(s => s >= 90).length;
    const good = Object.values(qualityBreakdown.categories).filter(s => s >= 75 && s < 90).length;
    const fair = Object.values(qualityBreakdown.categories).filter(s => s >= 60 && s < 75).length;
    const poor = Object.values(qualityBreakdown.categories).filter(s => s >= 40 && s < 60).length;
    const critical = Object.values(qualityBreakdown.categories).filter(s => s < 40).length;

    return [
      { name: 'Excellent (90-100)', value: excellent, color: dashboardTheme.colors.qualityScore.excellent },
      { name: 'Good (75-89)', value: good, color: dashboardTheme.colors.qualityScore.good },
      { name: 'Fair (60-74)', value: fair, color: dashboardTheme.colors.qualityScore.fair },
      { name: 'Poor (40-59)', value: poor, color: dashboardTheme.colors.qualityScore.poor },
      { name: 'Critical (0-39)', value: critical, color: dashboardTheme.colors.qualityScore.critical },
    ].filter(item => item.value > 0);
  }, [qualityBreakdown.categories]);

  // Get overall score color and status
  const getScoreStatus = useCallback((score: number) => {
    if (score >= 90) return { 
      color: dashboardTheme.colors.qualityScore.excellent, 
      label: 'Excellent', 
      icon: <CheckCircleOutlined /> 
    };
    if (score >= 75) return { 
      color: dashboardTheme.colors.qualityScore.good, 
      label: 'Good', 
      icon: <StarOutlined /> 
    };
    if (score >= 60) return { 
      color: dashboardTheme.colors.qualityScore.fair, 
      label: 'Fair', 
      icon: <WarningOutlined /> 
    };
    if (score >= 40) return { 
      color: dashboardTheme.colors.qualityScore.poor, 
      label: 'Poor', 
      icon: <ErrorOutlined /> 
    };
    return { 
      color: dashboardTheme.colors.qualityScore.critical, 
      label: 'Critical', 
      icon: <ErrorOutlined /> 
    };
  }, []);

  const scoreStatus = getScoreStatus(qualityBreakdown.overall);

  // Handle metric click
  const handleMetricClick = useCallback((metric: QualityMetric) => {
    if (onMetricClick) {
      onMetricClick(metric);
    }
  }, [onMetricClick]);

  // Handle export
  const handleExport = useCallback(() => {
    if (onExport) {
      onExport('png');
    }
  }, [onExport]);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Typography variant="body2" color="text.secondary" align="center">
            Loading quality metrics...
          </Typography>
          <Box mt={2}>
            <LinearProgress />
          </Box>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Typography variant="body2" color="error" align="center">
            Error: {error}
          </Typography>
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
            Quality Score
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Overall code quality assessment
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          <Tooltip title="Quality score based on multiple metrics including maintainability, reliability, and security" arrow>
            <IconButton size="small">
              <InfoOutlined />
            </IconButton>
          </Tooltip>
          
          <Tooltip title={expanded ? "Show less details" : "Show more details"} arrow>
            <IconButton size="small" onClick={() => setExpanded(!expanded)}>
              {expanded ? <ExpandLessOutlined /> : <ExpandMoreOutlined />}
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Main Score Card */}
      <Card sx={{ mb: 2, border: `2px solid ${scoreStatus.color}` }}>
        <CardContent>
          <Grid container spacing={3} alignItems="center">
            {/* Overall Score */}
            <Grid item xs={12} md={4}>
              <Box textAlign="center">
                <Box display="flex" alignItems="center" justifyContent="center" mb={1}>
                  <Box sx={{ color: scoreStatus.color, mr: 1 }}>
                    {scoreStatus.icon}
                  </Box>
                  <Chip 
                    label={scoreStatus.label}
                    sx={{ 
                      backgroundColor: alpha(scoreStatus.color, 0.1),
                      color: scoreStatus.color,
                      fontWeight: 600,
                    }}
                  />
                </Box>
                
                <Typography 
                  variant="h2" 
                  component="div" 
                  sx={{ 
                    color: scoreStatus.color,
                    fontWeight: 700,
                    fontFamily: dashboardTheme.typography.dashboard.metric.fontFamily,
                    mb: 1,
                  }}
                >
                  {qualityBreakdown.overall}
                </Typography>
                
                <Typography variant="body2" color="text.secondary">
                  Overall Quality Score
                </Typography>
                
                <Box mt={1}>
                  <LinearProgress 
                    variant="determinate" 
                    value={qualityBreakdown.overall}
                    sx={{
                      height: 8,
                      borderRadius: 4,
                      backgroundColor: alpha(scoreStatus.color, 0.1),
                      '& .MuiLinearProgress-bar': {
                        backgroundColor: scoreStatus.color,
                        borderRadius: 4,
                      },
                    }}
                  />
                </Box>
              </Box>
            </Grid>

            {/* Category Scores */}
            <Grid item xs={12} md={5}>
              <Typography variant="subtitle2" fontWeight="600" mb={2} color="text.primary">
                Category Breakdown
              </Typography>
              
              <Box display="flex" flexDirection="column" gap={1.5}>
                {qualityBreakdown.metrics.slice(0, 5).map((metric) => {
                  const metricStatus = getScoreStatus(metric.value);
                  const percentage = (metric.value / metric.maxValue) * 100;
                  
                  return (
                    <Box 
                      key={metric.id}
                      sx={{
                        cursor: onMetricClick ? 'pointer' : 'default',
                        p: 1,
                        borderRadius: 1,
                        '&:hover': onMetricClick ? {
                          backgroundColor: 'action.hover',
                        } : {},
                      }}
                      onClick={() => handleMetricClick(metric)}
                    >
                      <Box display="flex" alignItems="center" justifyContent="space-between" mb={0.5}>
                        <Typography variant="body2" fontWeight="500" color="text.primary">
                          {metric.name}
                        </Typography>
                        <Box display="flex" alignItems="center" gap={0.5}>
                          <Typography 
                            variant="body2" 
                            fontWeight="600" 
                            sx={{ 
                              color: metricStatus.color,
                              fontFamily: dashboardTheme.typography.dashboard.metric.fontFamily,
                              fontSize: '0.875rem',
                            }}
                          >
                            {metric.value}
                          </Typography>
                          {showTrends && metric.trend && (
                            <Box sx={{ color: dashboardTheme.colorUtils.getTrendColor(
                              metric.trend === 'up' ? 'improving' : 
                              metric.trend === 'down' ? 'worsening' : 'stable'
                            )}}>
                              {metric.trend === 'up' && <TrendingUpOutlined sx={{ fontSize: 16 }} />}
                              {metric.trend === 'down' && <TrendingDownOutlined sx={{ fontSize: 16 }} />}
                              {metric.trend === 'stable' && <TrendingFlatOutlined sx={{ fontSize: 16 }} />}
                            </Box>
                          )}
                        </Box>
                      </Box>
                      
                      <LinearProgress 
                        variant="determinate" 
                        value={percentage}
                        sx={{
                          height: 6,
                          borderRadius: 3,
                          backgroundColor: alpha(metricStatus.color, 0.1),
                          '& .MuiLinearProgress-bar': {
                            backgroundColor: metricStatus.color,
                            borderRadius: 3,
                          },
                        }}
                      />
                      
                      {showBenchmarks && metric.benchmark && (
                        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5 }}>
                          Benchmark: {metric.benchmark}%
                        </Typography>
                      )}
                    </Box>
                  );
                })}
              </Box>
            </Grid>

            {/* Score Distribution Chart */}
            <Grid item xs={12} md={3}>
              <Typography variant="subtitle2" fontWeight="600" mb={1} color="text.primary" align="center">
                Score Distribution
              </Typography>
              
              <Box height={200}>
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={pieChartData}
                      cx="50%"
                      cy="50%"
                      innerRadius={40}
                      outerRadius={80}
                      paddingAngle={2}
                      dataKey="value"
                    >
                      {pieChartData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Pie>
                  </PieChart>
                </ResponsiveContainer>
              </Box>
              
              <Box mt={1}>
                {pieChartData.slice(0, 3).map((entry, index) => (
                  <Box key={index} display="flex" alignItems="center" justifyContent="space-between" mb={0.5}>
                    <Box display="flex" alignItems="center">
                      <Box 
                        sx={{ 
                          width: 12, 
                          height: 12, 
                          backgroundColor: entry.color,
                          borderRadius: '50%',
                          mr: 1,
                        }} 
                      />
                      <Typography variant="caption" color="text.secondary">
                        {entry.name.split(' ')[0]}
                      </Typography>
                    </Box>
                    <Typography variant="caption" fontWeight="500" color="text.primary">
                      {entry.value}
                    </Typography>
                  </Box>
                ))}
              </Box>
            </Grid>
          </Grid>
        </CardContent>
      </Card>

      {/* Detailed Breakdown */}
      <Collapse in={expanded}>
        <Grid container spacing={2}>
          {/* Category Bar Chart */}
          <Grid item xs={12} md={8}>
            <Card>
              <CardContent>
                <Typography variant="h6" fontWeight="600" mb={2} color="text.primary">
                  Category Performance
                </Typography>
                
                <Box height={300}>
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={categoryChartData} margin={{ top: 20, right: 30, left: 20, bottom: 5 }}>
                      <CartesianGrid strokeDasharray="3 3" stroke={theme.palette.divider} />
                      <XAxis 
                        dataKey="name" 
                        tick={{ fontSize: 12, fill: theme.palette.text.secondary }}
                        stroke={theme.palette.text.secondary}
                      />
                      <YAxis 
                        domain={[0, 100]}
                        tick={{ fontSize: 12, fill: theme.palette.text.secondary }}
                        stroke={theme.palette.text.secondary}
                      />
                      <Bar 
                        dataKey="score" 
                        radius={[4, 4, 0, 0]}
                        fill={dashboardTheme.colors.primary[0]}
                      >
                        {categoryChartData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={entry.color} />
                        ))}
                      </Bar>
                    </BarChart>
                  </ResponsiveContainer>
                </Box>
              </CardContent>
            </Card>
          </Grid>

          {/* Recommendations */}
          <Grid item xs={12} md={4}>
            <Card>
              <CardContent>
                <Typography variant="h6" fontWeight="600" mb={2} color="text.primary">
                  Top Recommendations
                </Typography>
                
                <List dense>
                  {qualityBreakdown.recommendations.slice(0, 4).map((recommendation, index) => (
                    <React.Fragment key={recommendation.id}>
                      <ListItem alignItems="flex-start" sx={{ px: 0 }}>
                        <ListItemIcon sx={{ mt: 0.5, minWidth: 32 }}>
                          <Box
                            sx={{
                              width: 20,
                              height: 20,
                              borderRadius: '50%',
                              backgroundColor: 
                                recommendation.priority === 'high' ? dashboardTheme.colors.error[0] :
                                recommendation.priority === 'medium' ? dashboardTheme.colors.warning[0] :
                                dashboardTheme.colors.success[0],
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: 'center',
                              color: 'white',
                              fontSize: '12px',
                              fontWeight: 600,
                            }}
                          >
                            {index + 1}
                          </Box>
                        </ListItemIcon>
                        <ListItemText
                          primary={
                            <Typography variant="body2" fontWeight="500" color="text.primary">
                              {recommendation.title}
                            </Typography>
                          }
                          secondary={
                            <Typography variant="caption" color="text.secondary">
                              {recommendation.description}
                            </Typography>
                          }
                        />
                      </ListItem>
                      {index < qualityBreakdown.recommendations.slice(0, 4).length - 1 && (
                        <Divider component="li" />
                      )}
                    </React.Fragment>
                  ))}
                </List>
                
                {qualityBreakdown.recommendations.length > 4 && (
                  <Box mt={1} textAlign="center">
                    <Typography variant="caption" color="text.secondary">
                      +{qualityBreakdown.recommendations.length - 4} more recommendations
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Collapse>
    </Box>
  );
};

export default QualityScoreCard;