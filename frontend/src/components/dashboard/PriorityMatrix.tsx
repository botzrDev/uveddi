import React, { useMemo, useCallback, useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Chip,
  IconButton,
  Tooltip,
  Button,
  Grid,
  Paper,
  useTheme,
} from '@mui/material';
import {
  TuneOutlined,
  ZoomInOutlined,
  DownloadOutlined,
  InfoOutlined,
} from '@mui/icons-material';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  Title,
  Tooltip as ChartTooltip,
  Legend,
  ScatterController,
  TooltipItem,
} from 'chart.js';
import { Scatter } from 'react-chartjs-2';

import type { Finding } from '../../types/api';
import type {
  PriorityMatrixItem,
  QuadrantData,
  ResponsiveComponentProps,
  ExportableComponentProps,
  SelectionEvent,
} from '../../types/dashboard';

import { dataTransformers } from '../../utils/dataTransformers';
import { dashboardTheme } from '../../utils/dashboardTheme';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  Title,
  ChartTooltip,
  Legend,
  ScatterController
);

interface PriorityMatrixProps extends ResponsiveComponentProps, ExportableComponentProps {
  findings: Finding[];
  onItemSelect?: (items: PriorityMatrixItem[]) => void;
  onQuadrantFilter?: (quadrant: keyof QuadrantData) => void;
  onSelectionChange?: (event: SelectionEvent) => void;
  interactive?: boolean;
  showQuadrantLabels?: boolean;
  showLegend?: boolean;
  height?: number;
}

const PriorityMatrix: React.FC<PriorityMatrixProps> = ({
  findings,
  onItemSelect,
  onQuadrantFilter,
  onSelectionChange,
  interactive = true,
  showQuadrantLabels = true,
  showLegend = true,
  height = 400,
  loading = false,
  error,
  className,
  exportable = false,
  onExport,
  testId,
}) => {
  const theme = useTheme();
  const [selectedItems, setSelectedItems] = useState<string[]>([]);
  const [hoveredQuadrant, setHoveredQuadrant] = useState<keyof QuadrantData | null>(null);

  // Transform findings to matrix items
  const matrixItems = useMemo(() => {
    return dataTransformers.priorityMatrix.transformFindings(findings);
  }, [findings]);

  // Group items by quadrant
  const quadrantData = useMemo(() => {
    return dataTransformers.priorityMatrix.groupByQuadrant(matrixItems);
  }, [matrixItems]);

  // Prepare chart data
  const chartData = useMemo(() => {
    const datasets = [
      {
        label: 'Critical',
        data: matrixItems
          .filter(item => item.severity === 'critical')
          .map(item => ({
            x: item.effort,
            y: item.impact,
            id: item.id,
            item,
          })),
        backgroundColor: dashboardTheme.colorUtils.getSeverityColor('critical'),
        borderColor: dashboardTheme.colorUtils.getSeverityColor('critical'),
        pointRadius: 8,
        pointHoverRadius: 12,
      },
      {
        label: 'High',
        data: matrixItems
          .filter(item => item.severity === 'high')
          .map(item => ({
            x: item.effort,
            y: item.impact,
            id: item.id,
            item,
          })),
        backgroundColor: dashboardTheme.colorUtils.getSeverityColor('high'),
        borderColor: dashboardTheme.colorUtils.getSeverityColor('high'),
        pointRadius: 6,
        pointHoverRadius: 10,
      },
      {
        label: 'Medium',
        data: matrixItems
          .filter(item => item.severity === 'medium')
          .map(item => ({
            x: item.effort,
            y: item.impact,
            id: item.id,
            item,
          })),
        backgroundColor: dashboardTheme.colorUtils.getSeverityColor('medium'),
        borderColor: dashboardTheme.colorUtils.getSeverityColor('medium'),
        pointRadius: 5,
        pointHoverRadius: 8,
      },
      {
        label: 'Low',
        data: matrixItems
          .filter(item => item.severity === 'low')
          .map(item => ({
            x: item.effort,
            y: item.impact,
            id: item.id,
            item,
          })),
        backgroundColor: dashboardTheme.colorUtils.getSeverityColor('low'),
        borderColor: dashboardTheme.colorUtils.getSeverityColor('low'),
        pointRadius: 4,
        pointHoverRadius: 6,
      },
    ];

    return { datasets };
  }, [matrixItems]);

  // Chart options with quadrant backgrounds
  const chartOptions = useMemo(() => ({
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      title: {
        display: false,
      },
      legend: {
        display: showLegend,
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
          title: (tooltipItems: TooltipItem<'scatter'>[]) => {
            const item = (tooltipItems[0].raw as any).item as PriorityMatrixItem;
            return item.title;
          },
          label: (tooltipItem: TooltipItem<'scatter'>) => {
            const item = (tooltipItem.raw as any).item as PriorityMatrixItem;
            return [
              `Impact: ${item.impact}/10`,
              `Effort: ${item.effort}/10`,
              `Category: ${item.category}`,
              `Severity: ${item.severity}`,
            ];
          },
        },
      },
    },
    scales: {
      x: {
        title: {
          display: true,
          text: 'Effort →',
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 14,
            weight: '600',
          },
          color: theme.palette.text.primary,
        },
        min: 0,
        max: 10,
        ticks: {
          stepSize: 1,
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 11,
          },
          color: theme.palette.text.secondary,
        },
        grid: {
          color: theme.palette.divider,
          lineWidth: 1,
        },
      },
      y: {
        title: {
          display: true,
          text: '← Impact',
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 14,
            weight: '600',
          },
          color: theme.palette.text.primary,
        },
        min: 0,
        max: 10,
        ticks: {
          stepSize: 1,
          font: {
            family: dashboardTheme.chart.fonts.base,
            size: 11,
          },
          color: theme.palette.text.secondary,
        },
        grid: {
          color: theme.palette.divider,
          lineWidth: 1,
        },
      },
    },
    onClick: (event: any, elements: any[]) => {
      if (!interactive || elements.length === 0) return;
      
      const clickedItem = (elements[0].element.$context.raw as any).item as PriorityMatrixItem;
      const newSelection = selectedItems.includes(clickedItem.id)
        ? selectedItems.filter(id => id !== clickedItem.id)
        : [...selectedItems, clickedItem.id];
      
      setSelectedItems(newSelection);
      
      if (onSelectionChange) {
        onSelectionChange({
          type: 'selection_changed',
          payload: newSelection,
          timestamp: new Date().toISOString(),
          source: 'priority_matrix',
        });
      }
    },
    // Add quadrant background plugin
    plugins: [
      {
        id: 'quadrantBackground',
        beforeDraw: (chart: any) => {
          if (!showQuadrantLabels) return;
          
          const ctx = chart.ctx;
          const chartArea = chart.chartArea;
          const { left, top, right, bottom } = chartArea;
          
          const midX = left + (right - left) / 2;
          const midY = top + (bottom - top) / 2;
          
          // Save context
          ctx.save();
          
          // Draw quadrant backgrounds with subtle colors
          const quadrantColors = {
            quickWins: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.success[2], 0.1),
            majorProjects: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.primary[2], 0.1),
            fillIns: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.secondary[2], 0.1),
            thanklessWork: dashboardTheme.colorUtils.withAlpha(dashboardTheme.colors.error[2], 0.1),
          };
          
          // Quick Wins (top-left)
          ctx.fillStyle = quadrantColors.quickWins;
          ctx.fillRect(left, top, midX - left, midY - top);
          
          // Major Projects (top-right)
          ctx.fillStyle = quadrantColors.majorProjects;
          ctx.fillRect(midX, top, right - midX, midY - top);
          
          // Fill-ins (bottom-left)
          ctx.fillStyle = quadrantColors.fillIns;
          ctx.fillRect(left, midY, midX - left, bottom - midY);
          
          // Thankless Work (bottom-right)
          ctx.fillStyle = quadrantColors.thanklessWork;
          ctx.fillRect(midX, midY, right - midX, bottom - midY);
          
          // Draw quadrant labels
          ctx.font = '12px Inter, sans-serif';
          ctx.fillStyle = theme.palette.text.secondary;
          ctx.textAlign = 'center';
          
          const labelOffset = 15;
          ctx.fillText('Quick Wins', left + (midX - left) / 2, top + labelOffset);
          ctx.fillText('Major Projects', midX + (right - midX) / 2, top + labelOffset);
          ctx.fillText('Fill-ins', left + (midX - left) / 2, bottom - labelOffset);
          ctx.fillText('Thankless Work', midX + (right - midX) / 2, bottom - labelOffset);
          
          // Restore context
          ctx.restore();
        },
      },
    ],
  }), [theme, interactive, showLegend, showQuadrantLabels, selectedItems, onSelectionChange]);

  // Handle quadrant selection
  const handleQuadrantSelect = useCallback((quadrant: keyof QuadrantData) => {
    if (onQuadrantFilter) {
      onQuadrantFilter(quadrant);
    }
    
    const quadrantItems = quadrantData[quadrant];
    if (onItemSelect) {
      onItemSelect(quadrantItems);
    }
  }, [quadrantData, onQuadrantFilter, onItemSelect]);

  // Handle export
  const handleExport = useCallback(() => {
    if (onExport) {
      onExport('png'); // Default to PNG, could be configurable
    }
  }, [onExport]);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading priority matrix...
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
            <Typography variant="body2" color="error">
              Error: {error}
            </Typography>
          </Box>
        </CardContent>
      </Card>
    );
  }

  return (
    <Box className={className} data-testid={testId}>
      {/* Header with actions */}
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box>
          <Typography variant="h5" component="h2" fontWeight="600" color="text.primary">
            Priority Matrix
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Impact vs Effort analysis for {matrixItems.length} findings
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          <Tooltip title="Matrix shows impact vs effort to help prioritize fixes" arrow>
            <IconButton size="small">
              <InfoOutlined />
            </IconButton>
          </Tooltip>
          
          {interactive && (
            <Tooltip title="Configure matrix settings" arrow>
              <IconButton size="small">
                <TuneOutlined />
              </IconButton>
            </Tooltip>
          )}
          
          {exportable && (
            <Tooltip title="Export matrix" arrow>
              <IconButton size="small" onClick={handleExport}>
                <DownloadOutlined />
              </IconButton>
            </Tooltip>
          )}
        </Box>
      </Box>

      {/* Quadrant summary cards */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        {(Object.entries(quadrantData) as [keyof QuadrantData, PriorityMatrixItem[]][]).map(
          ([quadrant, items]) => {
            const quadrantInfo = {
              quickWins: { 
                label: 'Quick Wins', 
                description: 'High impact, low effort',
                color: dashboardTheme.colors.success[1],
                icon: '🎯'
              },
              majorProjects: { 
                label: 'Major Projects', 
                description: 'High impact, high effort',
                color: dashboardTheme.colors.primary[1],
                icon: '🏗️'
              },
              fillIns: { 
                label: 'Fill-ins', 
                description: 'Low impact, low effort',
                color: dashboardTheme.colors.secondary[1],
                icon: '🔧'
              },
              thanklessWork: { 
                label: 'Thankless Work', 
                description: 'Low impact, high effort',
                color: dashboardTheme.colors.error[1],
                icon: '⚠️'
              },
            };

            const info = quadrantInfo[quadrant];

            return (
              <Grid item xs={12} sm={6} md={3} key={quadrant}>
                <Paper
                  sx={{
                    p: 2,
                    cursor: interactive ? 'pointer' : 'default',
                    border: '1px solid',
                    borderColor: 'divider',
                    borderLeft: `4px solid ${info.color}`,
                    transition: 'all 0.2s ease-in-out',
                    '&:hover': interactive ? {
                      transform: 'translateY(-2px)',
                      boxShadow: 2,
                      backgroundColor: 'action.hover',
                    } : {},
                  }}
                  onClick={interactive ? () => handleQuadrantSelect(quadrant) : undefined}
                  onMouseEnter={() => setHoveredQuadrant(quadrant)}
                  onMouseLeave={() => setHoveredQuadrant(null)}
                >
                  <Box display="flex" alignItems="center" gap={1} mb={1}>
                    <Typography variant="body2" sx={{ fontSize: '16px' }}>
                      {info.icon}
                    </Typography>
                    <Typography variant="subtitle2" fontWeight="600" color="text.primary">
                      {info.label}
                    </Typography>
                  </Box>
                  
                  <Typography variant="h4" fontWeight="700" color={info.color} mb={0.5}>
                    {items.length}
                  </Typography>
                  
                  <Typography variant="caption" color="text.secondary" display="block">
                    {info.description}
                  </Typography>
                  
                  {items.length > 0 && (
                    <Box mt={1}>
                      <Chip
                        label={`${items.filter(i => i.severity === 'critical').length} critical`}
                        size="small"
                        sx={{
                          backgroundColor: dashboardTheme.colorUtils.withAlpha(
                            dashboardTheme.colors.severity.critical, 0.1
                          ),
                          color: dashboardTheme.colors.severity.critical,
                          mr: 0.5,
                          fontSize: '0.7rem',
                          height: '20px',
                        }}
                      />
                    </Box>
                  )}
                </Paper>
              </Grid>
            );
          }
        )}
      </Grid>

      {/* Matrix chart */}
      <Card>
        <CardContent>
          <Box height={height} position="relative">
            <Scatter data={chartData} options={chartOptions} />
          </Box>
        </CardContent>
      </Card>

      {/* Selected items summary */}
      {selectedItems.length > 0 && (
        <Box mt={2}>
          <Paper sx={{ p: 2, backgroundColor: 'action.hover' }}>
            <Typography variant="subtitle2" fontWeight="600" mb={1}>
              Selected Items ({selectedItems.length})
            </Typography>
            <Box display="flex" flexWrap="wrap" gap={1}>
              {selectedItems.slice(0, 5).map(id => {
                const item = matrixItems.find(i => i.id === id);
                return item ? (
                  <Chip
                    key={id}
                    label={item.title}
                    size="small"
                    onDelete={() => {
                      const newSelection = selectedItems.filter(sid => sid !== id);
                      setSelectedItems(newSelection);
                      if (onSelectionChange) {
                        onSelectionChange({
                          type: 'selection_changed',
                          payload: newSelection,
                          timestamp: new Date().toISOString(),
                          source: 'priority_matrix',
                        });
                      }
                    }}
                    sx={{
                      backgroundColor: dashboardTheme.colorUtils.getSeverityColor(item.severity),
                      color: 'white',
                    }}
                  />
                ) : null;
              })}
              {selectedItems.length > 5 && (
                <Chip
                  label={`+${selectedItems.length - 5} more`}
                  size="small"
                  variant="outlined"
                />
              )}
            </Box>
          </Paper>
        </Box>
      )}
    </Box>
  );
};

export default PriorityMatrix;