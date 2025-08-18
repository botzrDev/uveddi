import React, { useMemo, useCallback, useState, useRef, useEffect } from 'react';
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
  Slider,
  FormControl,
  FormLabel,
  Select,
  MenuItem,
  Switch,
  FormControlLabel,
  Collapse,
  Alert,
  useTheme,
} from '@mui/material';
import {
  AccountTreeOutlined,
  ZoomInOutlined,
  ZoomOutOutlined,
  CenterFocusStrongOutlined,
  SettingsOutlined,
  DownloadOutlined,
  InfoOutlined,
  ExpandMoreOutlined,
  ExpandLessOutlined,
  FilterListOutlined,
} from '@mui/icons-material';
import CytoscapeComponent from 'react-cytoscapejs';
import type { Core, EventObject, NodeSingular, EdgeSingular } from 'cytoscape';

import type { DependencyGraph, GraphNode, GraphEdge } from '../../types/api';
import type {
  EnhancedGraphNode,
  EnhancedGraphEdge,
  GraphAnalytics,
  GraphCluster,
  ResponsiveComponentProps,
  ExportableComponentProps,
  SelectionEvent,
  FilterEvent,
} from '../../types/dashboard';

import { dataTransformers } from '../../utils/dataTransformers';
import { dashboardTheme } from '../../utils/dashboardTheme';

// Cytoscape layout algorithms
const LAYOUT_ALGORITHMS = [
  { value: 'cola', label: 'Cola (Force-Directed)', description: 'Good for general graphs' },
  { value: 'cose', label: 'COSE (Compound Spring)', description: 'Good for clustered graphs' },
  { value: 'dagre', label: 'Dagre (Hierarchical)', description: 'Good for directed graphs' },
  { value: 'circle', label: 'Circle', description: 'Circular layout' },
  { value: 'grid', label: 'Grid', description: 'Grid layout' },
  { value: 'breadthfirst', label: 'Breadth First', description: 'Tree-like layout' },
];

interface InteractiveDependencyGraphProps extends ResponsiveComponentProps, ExportableComponentProps {
  dependencyGraph: DependencyGraph;
  onNodeSelect?: (nodes: EnhancedGraphNode[]) => void;
  onEdgeSelect?: (edges: EnhancedGraphEdge[]) => void;
  onSelectionChange?: (event: SelectionEvent) => void;
  onFilterChange?: (event: FilterEvent) => void;
  showControls?: boolean;
  showClusters?: boolean;
  showMetrics?: boolean;
  enablePhysics?: boolean;
  height?: number;
  highlightCriticalPaths?: boolean;
}

const InteractiveDependencyGraph: React.FC<InteractiveDependencyGraphProps> = ({
  dependencyGraph,
  onNodeSelect,
  onEdgeSelect,
  onSelectionChange,
  onFilterChange,
  showControls = true,
  showClusters = true,
  showMetrics = true,
  enablePhysics = true,
  height = 600,
  highlightCriticalPaths = false,
  loading = false,
  error,
  className,
  exportable = false,
  onExport,
  testId,
}) => {
  const theme = useTheme();
  const cytoscapeRef = useRef<Core | null>(null);
  
  // State management
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);
  const [selectedEdges, setSelectedEdges] = useState<string[]>([]);
  const [layout, setLayout] = useState('cola');
  const [showSettings, setShowSettings] = useState(false);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [filters, setFilters] = useState({
    minCentrality: 0,
    maxCentrality: 100,
    showIsolatedNodes: true,
    riskLevel: 'all' as 'all' | 'high' | 'medium' | 'low',
    clusterFilter: 'all' as string,
  });

  // Enhanced graph data
  const enhancedNodes = useMemo(() => {
    return dataTransformers.dependencyGraph.enhanceNodes(dependencyGraph.nodes || []);
  }, [dependencyGraph.nodes]);

  const enhancedEdges = useMemo(() => {
    return dataTransformers.dependencyGraph.enhanceEdges(dependencyGraph.edges || []);
  }, [dependencyGraph.edges]);

  // Graph analytics
  const analytics = useMemo(() => {
    return dataTransformers.dependencyGraph.analyze(enhancedNodes, enhancedEdges);
  }, [enhancedNodes, enhancedEdges]);

  // Filtered data based on current filters
  const filteredNodes = useMemo(() => {
    return enhancedNodes.filter(node => {
      // Centrality filter
      const centralityInRange = (node.centralityScore || 0) >= filters.minCentrality && 
                                (node.centralityScore || 0) <= filters.maxCentrality;
      
      // Risk level filter
      const riskMatch = filters.riskLevel === 'all' || 
        (node.riskScore || 0) >= (filters.riskLevel === 'high' ? 7 : filters.riskLevel === 'medium' ? 4 : 0) &&
        (node.riskScore || 0) < (filters.riskLevel === 'high' ? 10 : filters.riskLevel === 'medium' ? 7 : 4);
      
      // Cluster filter
      const clusterMatch = filters.clusterFilter === 'all' || node.clusterGroup === filters.clusterFilter;
      
      // Isolated nodes filter
      const isIsolated = !enhancedEdges.some(e => e.source === node.id || e.target === node.id);
      const isolatedMatch = filters.showIsolatedNodes || !isIsolated;
      
      return centralityInRange && riskMatch && clusterMatch && isolatedMatch;
    });
  }, [enhancedNodes, enhancedEdges, filters]);

  const filteredEdges = useMemo(() => {
    const validNodeIds = new Set(filteredNodes.map(n => n.id));
    return enhancedEdges.filter(edge => 
      validNodeIds.has(edge.source) && validNodeIds.has(edge.target)
    );
  }, [filteredNodes, enhancedEdges]);

  // Cytoscape elements format
  const cytoscapeElements = useMemo(() => {
    const nodes = filteredNodes.map(node => ({
      data: {
        id: node.id,
        label: node.name,
        type: node.type,
        centralityScore: node.centralityScore,
        clusterGroup: node.clusterGroup,
        riskScore: node.riskScore,
        technicalDebt: node.technicalDebt,
        size: Math.max(10, Math.min(40, (node.centralityScore || 10) / 2)),
      },
      classes: [
        node.type,
        node.clusterGroup,
        selectedNodes.includes(node.id) ? 'selected' : '',
        (node.riskScore || 0) > 7 ? 'high-risk' : (node.riskScore || 0) > 4 ? 'medium-risk' : 'low-risk',
      ].filter(Boolean).join(' '),
      position: node.position,
    }));

    const edges = filteredEdges.map(edge => ({
      data: {
        id: `${edge.source}-${edge.target}`,
        source: edge.source,
        target: edge.target,
        strength: edge.strength,
        riskLevel: edge.riskLevel,
        type: edge.type,
      },
      classes: [
        edge.type,
        edge.riskLevel,
        selectedEdges.includes(`${edge.source}-${edge.target}`) ? 'selected' : '',
      ].filter(Boolean).join(' '),
    }));

    return [...nodes, ...edges];
  }, [filteredNodes, filteredEdges, selectedNodes, selectedEdges]);

  // Cytoscape stylesheet
  const cytoscapeStylesheet = useMemo(() => [
    // Base node styles
    {
      selector: 'node',
      style: {
        ...dashboardTheme.chart.cytoscape.node,
        'width': 'data(size)',
        'height': 'data(size)',
        'background-color': dashboardTheme.colors.primary[0],
        'border-color': theme.palette.divider,
        'border-width': 2,
        'font-size': '10px',
        'color': theme.palette.text.primary,
        'text-outline-color': theme.palette.background.paper,
        'text-outline-width': 1,
      },
    },
    // Node hover effects
    {
      selector: 'node:hover',
      style: {
        'border-color': dashboardTheme.colors.primary[1],
        'border-width': 3,
        'background-color': dashboardTheme.colors.primary[1],
      },
    },
    // Selected nodes
    {
      selector: 'node.selected',
      style: {
        'border-color': dashboardTheme.colors.success[1],
        'border-width': 4,
        'background-color': dashboardTheme.colors.success[2],
      },
    },
    // Risk level styles
    {
      selector: 'node.high-risk',
      style: {
        'background-color': dashboardTheme.colors.severity.critical,
      },
    },
    {
      selector: 'node.medium-risk',
      style: {
        'background-color': dashboardTheme.colors.severity.medium,
      },
    },
    {
      selector: 'node.low-risk',
      style: {
        'background-color': dashboardTheme.colors.severity.low,
      },
    },
    // Base edge styles
    {
      selector: 'edge',
      style: {
        ...dashboardTheme.chart.cytoscape.edge,
        'line-color': theme.palette.divider,
        'target-arrow-color': theme.palette.divider,
        'width': 'mapData(strength, 0, 1, 1, 4)',
        'opacity': 0.6,
      },
    },
    // Edge hover effects
    {
      selector: 'edge:hover',
      style: {
        'line-color': dashboardTheme.colors.primary[1],
        'target-arrow-color': dashboardTheme.colors.primary[1],
        'opacity': 0.8,
      },
    },
    // Selected edges
    {
      selector: 'edge.selected',
      style: {
        'line-color': dashboardTheme.colors.success[1],
        'target-arrow-color': dashboardTheme.colors.success[1],
        'width': 3,
        'opacity': 1,
      },
    },
    // Edge risk levels
    {
      selector: 'edge.high',
      style: {
        'line-color': dashboardTheme.colors.severity.critical,
        'target-arrow-color': dashboardTheme.colors.severity.critical,
      },
    },
    {
      selector: 'edge.medium',
      style: {
        'line-color': dashboardTheme.colors.severity.medium,
        'target-arrow-color': dashboardTheme.colors.severity.medium,
      },
    },
    {
      selector: 'edge.low',
      style: {
        'line-color': dashboardTheme.colors.severity.low,
        'target-arrow-color': dashboardTheme.colors.severity.low,
      },
    },
  ], [theme]);

  // Layout configuration
  const layoutConfig = useMemo(() => {
    const baseConfig = {
      name: layout,
      animate: true,
      animationDuration: 500,
      fit: true,
      padding: 30,
    };

    switch (layout) {
      case 'cola':
        return {
          ...baseConfig,
          nodeSpacing: 10,
          edgeLengthVal: 100,
          animate: enablePhysics,
          randomize: false,
          maxSimulationTime: 2000,
        };
      case 'cose':
        return {
          ...baseConfig,
          nodeRepulsion: 400000,
          nodeOverlap: 10,
          idealEdgeLength: 100,
          edgeElasticity: 100,
          nestingFactor: 5,
          gravity: 80,
          numIter: 1000,
          initialTemp: 200,
          coolingFactor: 0.95,
          minTemp: 1.0,
        };
      case 'dagre':
        return {
          ...baseConfig,
          rankDir: 'TB',
          align: 'UL',
          rankSep: 75,
          nodeSep: 50,
        };
      default:
        return baseConfig;
    }
  }, [layout, enablePhysics]);

  // Event handlers
  const handleNodeTap = useCallback((event: EventObject) => {
    const node = event.target as NodeSingular;
    const nodeId = node.id();
    
    const newSelection = selectedNodes.includes(nodeId)
      ? selectedNodes.filter(id => id !== nodeId)
      : [...selectedNodes, nodeId];
    
    setSelectedNodes(newSelection);
    
    if (onSelectionChange) {
      onSelectionChange({
        type: 'selection_changed',
        payload: newSelection,
        timestamp: new Date().toISOString(),
        source: 'dependency_graph_nodes',
      });
    }

    if (onNodeSelect) {
      const selectedNodeObjects = enhancedNodes.filter(n => newSelection.includes(n.id));
      onNodeSelect(selectedNodeObjects);
    }
  }, [selectedNodes, onSelectionChange, onNodeSelect, enhancedNodes]);

  const handleEdgeTap = useCallback((event: EventObject) => {
    const edge = event.target as EdgeSingular;
    const edgeId = edge.id();
    
    const newSelection = selectedEdges.includes(edgeId)
      ? selectedEdges.filter(id => id !== edgeId)
      : [...selectedEdges, edgeId];
    
    setSelectedEdges(newSelection);
    
    if (onEdgeSelect) {
      const selectedEdgeObjects = enhancedEdges.filter(e => 
        newSelection.includes(`${e.source}-${e.target}`)
      );
      onEdgeSelect(selectedEdgeObjects);
    }
  }, [selectedEdges, onEdgeSelect, enhancedEdges]);

  const handleZoom = useCallback((event: EventObject) => {
    if (cytoscapeRef.current) {
      setZoomLevel(cytoscapeRef.current.zoom());
    }
  }, []);

  // Control handlers
  const handleZoomIn = useCallback(() => {
    if (cytoscapeRef.current) {
      cytoscapeRef.current.zoom(cytoscapeRef.current.zoom() * 1.2);
      cytoscapeRef.current.center();
    }
  }, []);

  const handleZoomOut = useCallback(() => {
    if (cytoscapeRef.current) {
      cytoscapeRef.current.zoom(cytoscapeRef.current.zoom() * 0.8);
      cytoscapeRef.current.center();
    }
  }, []);

  const handleFitToScreen = useCallback(() => {
    if (cytoscapeRef.current) {
      cytoscapeRef.current.fit(undefined, 50);
    }
  }, []);

  const handleLayoutChange = useCallback((newLayout: string) => {
    setLayout(newLayout);
  }, []);

  const handleFilterChange = useCallback((filterKey: string, value: any) => {
    const newFilters = { ...filters, [filterKey]: value };
    setFilters(newFilters);
    
    if (onFilterChange) {
      onFilterChange({
        type: 'filters_changed',
        payload: newFilters,
        timestamp: new Date().toISOString(),
        source: 'dependency_graph',
      });
    }
  }, [filters, onFilterChange]);

  const handleExport = useCallback(() => {
    if (onExport && cytoscapeRef.current) {
      // Export as PNG by default
      const png = cytoscapeRef.current.png({ 
        scale: 2, 
        bg: theme.palette.background.paper 
      });
      
      // Create download link
      const link = document.createElement('a');
      link.download = 'dependency-graph.png';
      link.href = png;
      link.click();
      
      onExport('png');
    }
  }, [onExport, theme]);

  // Initialize cytoscape reference
  const handleCytoscapeInit = useCallback((cy: Core) => {
    cytoscapeRef.current = cy;
    
    // Add event listeners
    cy.on('tap', 'node', handleNodeTap);
    cy.on('tap', 'edge', handleEdgeTap);
    cy.on('zoom', handleZoom);
    
    // Initial fit
    cy.fit(undefined, 50);
    
    return () => {
      cy.off('tap', 'node', handleNodeTap);
      cy.off('tap', 'edge', handleEdgeTap);
      cy.off('zoom', handleZoom);
    };
  }, [handleNodeTap, handleEdgeTap, handleZoom]);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading dependency graph...
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
                Error loading graph: {error}
              </Typography>
            </Alert>
          </Box>
        </CardContent>
      </Card>
    );
  }

  return (
    <Box className={className} data-testid={testId}>
      {/* Header with controls */}
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box>
          <Typography variant="h5" component="h2" fontWeight="600" color="text.primary">
            Dependency Graph
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {filteredNodes.length} nodes, {filteredEdges.length} edges
          </Typography>
        </Box>
        
        {showControls && (
          <Box display="flex" alignItems="center" gap={1}>
            {/* Layout selector */}
            <FormControl size="small" sx={{ minWidth: 120 }}>
              <Select
                value={layout}
                onChange={(e) => handleLayoutChange(e.target.value)}
                displayEmpty
              >
                {LAYOUT_ALGORITHMS.map(algo => (
                  <MenuItem key={algo.value} value={algo.value}>
                    <Tooltip title={algo.description} placement="left" arrow>
                      <span>{algo.label}</span>
                    </Tooltip>
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            {/* Zoom controls */}
            <ButtonGroup size="small" variant="outlined">
              <Tooltip title="Zoom In" arrow>
                <IconButton onClick={handleZoomIn}>
                  <ZoomInOutlined />
                </IconButton>
              </Tooltip>
              <Tooltip title="Zoom Out" arrow>
                <IconButton onClick={handleZoomOut}>
                  <ZoomOutOutlined />
                </IconButton>
              </Tooltip>
              <Tooltip title="Fit to Screen" arrow>
                <IconButton onClick={handleFitToScreen}>
                  <CenterFocusStrongOutlined />
                </IconButton>
              </Tooltip>
            </ButtonGroup>

            <Tooltip title="Graph Settings" arrow>
              <IconButton onClick={() => setShowSettings(!showSettings)}>
                <SettingsOutlined />
              </IconButton>
            </Tooltip>

            {exportable && (
              <Tooltip title="Export Graph" arrow>
                <IconButton onClick={handleExport}>
                  <DownloadOutlined />
                </IconButton>
              </Tooltip>
            )}

            <Tooltip title="Graph shows component dependencies and relationships" arrow>
              <IconButton>
                <InfoOutlined />
              </IconButton>
            </Tooltip>
          </Box>
        )}
      </Box>

      {/* Settings panel */}
      <Collapse in={showSettings}>
        <Paper sx={{ p: 2, mb: 2, backgroundColor: 'action.hover' }}>
          <Grid container spacing={3}>
            {/* Centrality filter */}
            <Grid item xs={12} md={4}>
              <FormLabel component="legend" sx={{ mb: 1 }}>
                Centrality Score Range
              </FormLabel>
              <Slider
                value={[filters.minCentrality, filters.maxCentrality]}
                onChange={(_, value) => {
                  const [min, max] = value as number[];
                  handleFilterChange('minCentrality', min);
                  handleFilterChange('maxCentrality', max);
                }}
                valueLabelDisplay="auto"
                min={0}
                max={100}
                marks={[
                  { value: 0, label: '0' },
                  { value: 50, label: '50' },
                  { value: 100, label: '100' },
                ]}
              />
            </Grid>

            {/* Risk level filter */}
            <Grid item xs={12} md={3}>
              <FormControl fullWidth size="small">
                <FormLabel sx={{ mb: 1 }}>Risk Level</FormLabel>
                <Select
                  value={filters.riskLevel}
                  onChange={(e) => handleFilterChange('riskLevel', e.target.value)}
                >
                  <MenuItem value="all">All Levels</MenuItem>
                  <MenuItem value="high">High Risk Only</MenuItem>
                  <MenuItem value="medium">Medium Risk Only</MenuItem>
                  <MenuItem value="low">Low Risk Only</MenuItem>
                </Select>
              </FormControl>
            </Grid>

            {/* Cluster filter */}
            <Grid item xs={12} md={3}>
              <FormControl fullWidth size="small">
                <FormLabel sx={{ mb: 1 }}>Cluster</FormLabel>
                <Select
                  value={filters.clusterFilter}
                  onChange={(e) => handleFilterChange('clusterFilter', e.target.value)}
                >
                  <MenuItem value="all">All Clusters</MenuItem>
                  {analytics.clusters.map(cluster => (
                    <MenuItem key={cluster.id} value={cluster.id}>
                      {cluster.purpose || cluster.id}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Grid>

            {/* Options */}
            <Grid item xs={12} md={2}>
              <Box display="flex" flexDirection="column" gap={1}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={filters.showIsolatedNodes}
                      onChange={(e) => handleFilterChange('showIsolatedNodes', e.target.checked)}
                      size="small"
                    />
                  }
                  label="Show Isolated"
                  componentsProps={{ typography: { variant: 'caption' } }}
                />
                <FormControlLabel
                  control={
                    <Switch
                      checked={enablePhysics}
                      onChange={(e) => handleFilterChange('enablePhysics', e.target.checked)}
                      size="small"
                    />
                  }
                  label="Enable Physics"
                  componentsProps={{ typography: { variant: 'caption' } }}
                />
              </Box>
            </Grid>
          </Grid>
        </Paper>
      </Collapse>

      {/* Main graph container */}
      <Grid container spacing={2}>
        {/* Graph visualization */}
        <Grid item xs={12} lg={showMetrics ? 9 : 12}>
          <Card>
            <CardContent sx={{ p: 1, '&:last-child': { pb: 1 } }}>
              <Box
                height={height}
                position="relative"
                sx={{
                  '& > div': {
                    height: '100% !important',
                    width: '100% !important',
                  },
                }}
              >
                <CytoscapeComponent
                  elements={cytoscapeElements}
                  stylesheet={cytoscapeStylesheet}
                  layout={layoutConfig}
                  style={{ width: '100%', height: '100%' }}
                  cy={(cy) => handleCytoscapeInit(cy)}
                  wheelSensitivity={0.1}
                  minZoom={0.1}
                  maxZoom={3}
                  autoungrabify={false}
                  autounselectify={false}
                />

                {/* Zoom level indicator */}
                <Box
                  position="absolute"
                  bottom={16}
                  left={16}
                  bgcolor="background.paper"
                  border={1}
                  borderColor="divider"
                  borderRadius={1}
                  px={1}
                  py={0.5}
                >
                  <Typography variant="caption" color="text.secondary">
                    Zoom: {Math.round(zoomLevel * 100)}%
                  </Typography>
                </Box>

                {/* Selection indicator */}
                {(selectedNodes.length > 0 || selectedEdges.length > 0) && (
                  <Box
                    position="absolute"
                    top={16}
                    left={16}
                    bgcolor="success.light"
                    color="success.contrastText"
                    borderRadius={1}
                    px={2}
                    py={1}
                  >
                    <Typography variant="body2" fontWeight="600">
                      Selected: {selectedNodes.length} nodes, {selectedEdges.length} edges
                    </Typography>
                  </Box>
                )}
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Metrics sidebar */}
        {showMetrics && (
          <Grid item xs={12} lg={3}>
            <Box display="flex" flexDirection="column" gap={2}>
              {/* Graph metrics */}
              <Card>
                <CardContent>
                  <Typography variant="h6" fontWeight="600" mb={2}>
                    Graph Metrics
                  </Typography>
                  
                  <Box display="flex" flexDirection="column" gap={1}>
                    <Box display="flex" justifyContent="space-between">
                      <Typography variant="body2" color="text.secondary">
                        Modularity:
                      </Typography>
                      <Typography variant="body2" fontWeight="600">
                        {(analytics.metrics.modularity * 100).toFixed(1)}%
                      </Typography>
                    </Box>
                    
                    <Box display="flex" justifyContent="space-between">
                      <Typography variant="body2" color="text.secondary">
                        Avg Path Length:
                      </Typography>
                      <Typography variant="body2" fontWeight="600">
                        {analytics.metrics.avgPathLength.toFixed(1)}
                      </Typography>
                    </Box>
                    
                    <Box display="flex" justifyContent="space-between">
                      <Typography variant="body2" color="text.secondary">
                        Clustering:
                      </Typography>
                      <Typography variant="body2" fontWeight="600">
                        {(analytics.metrics.clusteringCoefficient * 100).toFixed(1)}%
                      </Typography>
                    </Box>
                  </Box>
                </CardContent>
              </Card>

              {/* Clusters */}
              {showClusters && analytics.clusters.length > 0 && (
                <Card>
                  <CardContent>
                    <Typography variant="h6" fontWeight="600" mb={2}>
                      Clusters ({analytics.clusters.length})
                    </Typography>
                    
                    <Box display="flex" flexDirection="column" gap={1}>
                      {analytics.clusters.slice(0, 5).map(cluster => (
                        <Box
                          key={cluster.id}
                          p={1}
                          border={1}
                          borderColor="divider"
                          borderRadius={1}
                        >
                          <Typography variant="body2" fontWeight="600" mb={0.5}>
                            {cluster.purpose || cluster.id}
                          </Typography>
                          <Typography variant="caption" color="text.secondary" display="block">
                            {cluster.nodes.length} nodes
                          </Typography>
                          <Box display="flex" gap={0.5} mt={0.5}>
                            <Chip
                              label={`Cohesion: ${(cluster.cohesion * 100).toFixed(0)}%`}
                              size="small"
                              variant="outlined"
                              sx={{ fontSize: '0.6rem', height: 18 }}
                            />
                            <Chip
                              label={`Coupling: ${(cluster.coupling * 100).toFixed(0)}%`}
                              size="small"
                              variant="outlined"
                              sx={{ fontSize: '0.6rem', height: 18 }}
                            />
                          </Box>
                        </Box>
                      ))}
                    </Box>
                  </CardContent>
                </Card>
              )}

              {/* Hotspots */}
              {analytics.hotspots.length > 0 && (
                <Card>
                  <CardContent>
                    <Typography variant="h6" fontWeight="600" mb={2}>
                      Hotspots
                    </Typography>
                    
                    <Box display="flex" flexDirection="column" gap={1}>
                      {analytics.hotspots.slice(0, 5).map(node => (
                        <Box
                          key={node.id}
                          p={1}
                          border={1}
                          borderColor="warning.main"
                          borderRadius={1}
                          bgcolor="warning.light"
                          sx={{ opacity: 0.8 }}
                        >
                          <Typography variant="body2" fontWeight="600" mb={0.5}>
                            {node.name}
                          </Typography>
                          <Typography variant="caption" color="text.secondary" display="block">
                            Centrality: {((node as EnhancedGraphNode).centralityScore || 0).toFixed(1)}
                          </Typography>
                          <Typography variant="caption" color="text.secondary" display="block">
                            Risk: {((node as EnhancedGraphNode).riskScore || 0).toFixed(1)}/10
                          </Typography>
                        </Box>
                      ))}
                    </Box>
                  </CardContent>
                </Card>
              )}
            </Box>
          </Grid>
        )}
      </Grid>

      {/* Selection details */}
      {selectedNodes.length > 0 && (
        <Box mt={2}>
          <Paper sx={{ p: 2, backgroundColor: 'action.hover' }}>
            <Typography variant="subtitle2" fontWeight="600" mb={1}>
              Selected Nodes ({selectedNodes.length})
            </Typography>
            <Box display="flex" flexWrap="wrap" gap={1}>
              {selectedNodes.slice(0, 10).map(nodeId => {
                const node = enhancedNodes.find(n => n.id === nodeId);
                return node ? (
                  <Chip
                    key={nodeId}
                    label={node.name}
                    size="small"
                    color="primary"
                    onDelete={() => {
                      const newSelection = selectedNodes.filter(id => id !== nodeId);
                      setSelectedNodes(newSelection);
                      if (onSelectionChange) {
                        onSelectionChange({
                          type: 'selection_changed',
                          payload: newSelection,
                          timestamp: new Date().toISOString(),
                          source: 'dependency_graph_nodes',
                        });
                      }
                    }}
                  />
                ) : null;
              })}
              {selectedNodes.length > 10 && (
                <Chip
                  label={`+${selectedNodes.length - 10} more`}
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

export default InteractiveDependencyGraph;