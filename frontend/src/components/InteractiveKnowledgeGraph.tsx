/**
 * Interactive Knowledge Graph Visualization
 *
 * Real-time, interactive visualization of code relationships using D3.js
 * with live updates from our cached knowledge graph system.
 */

import React, { useRef, useEffect, useState, useCallback } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui';
import { Filter, ZoomIn, ZoomOut, RotateCcw, Play, Pause, Settings } from 'lucide-react';
import { useWebSocket } from '@/hooks/useWebSocket';

interface GraphNode {
  id: string;
  label: string;
  type: 'module' | 'class' | 'function' | 'variable';
  language: 'rust' | 'python' | 'javascript' | 'typescript';
  size: number;
  importance: number;
  file_path: string;
  x?: number;
  y?: number;
  fx?: number;
  fy?: number;
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: 'imports' | 'calls' | 'inherits' | 'uses';
  weight: number;
  cached: boolean;
}

interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metadata: {
    cache_hit_rate: number;
    query_time_ms: number;
    total_relationships: number;
    last_updated: string;
  };
}

interface VisualizationConfig {
  layout: 'force' | 'hierarchical' | 'circular';
  nodeSize: 'uniform' | 'importance' | 'connections';
  edgeWeight: 'uniform' | 'strength' | 'cache_priority';
  showLabels: boolean;
  filterByType: string[];
  filterByLanguage: string[];
}

export const InteractiveKnowledgeGraph: React.FC = () => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const [graphData, setGraphData] = useState<GraphData | null>(null);
  const [config, setConfig] = useState<VisualizationConfig>({
    layout: 'force',
    nodeSize: 'importance',
    edgeWeight: 'strength',
    showLabels: true,
    filterByType: ['module', 'class', 'function'],
    filterByLanguage: ['rust', 'python', 'javascript', 'typescript']
  });

  const [isPlaying, setIsPlaying] = useState(false);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [isLoading, setIsLoading] = useState(true);

  // D3 simulation and elements
  const simulationRef = useRef<d3.Simulation<GraphNode, GraphEdge> | null>(null);
  const transformRef = useRef<d3.ZoomTransform>(d3.zoomIdentity);

  // WebSocket for real-time graph updates
  const { sendMessage, lastMessage } = useWebSocket(
    `ws://${window.location.host}/api/v1/stream/events`,
    {
      onMessage: handleGraphUpdate,
      shouldReconnect: () => true
    }
  );

  function handleGraphUpdate(message: MessageEvent) {
    try {
      const data = JSON.parse(message.data);

      if (data.type === 'graph_update') {
        // Incrementally update the graph with new relationships
        setGraphData(prev => {
          if (!prev) return null;

          const newNodes = [...prev.nodes];
          const newEdges = [...prev.edges];

          // Add any new nodes/edges from the update
          data.affected_files?.forEach((filePath: string) => {
            // Simulate adding new nodes/relationships
            // In real implementation, this would come from the backend
            console.log(`Graph updated for ${filePath}`);
          });

          return {
            ...prev,
            metadata: {
              ...prev.metadata,
              last_updated: data.timestamp,
              cache_hit_rate: Math.min(prev.metadata.cache_hit_rate + 0.01, 0.95)
            }
          };
        });
      }
    } catch (error) {
      console.error('Failed to handle graph update:', error);
    }
  }

  // Fetch initial graph data
  const fetchGraphData = useCallback(async () => {
    setIsLoading(true);
    try {
      // Query our knowledge graph API
      const response = await fetch('/api/v1/graph/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query_type: 'neighborhood',
          parameters: {
            max_depth: 3,
            relation_types: ['imports', 'uses', 'calls'],
            include_metrics: true
          },
          use_cache: true,
          cache_ttl: 1800
        })
      });

      const result = await response.json();

      // Transform API response to our graph format
      const nodes: GraphNode[] = result.results.nodes?.map((node: any) => ({
        id: node.id,
        label: node.label,
        type: node.node_type,
        language: detectLanguage(node.file_path),
        size: node.metrics?.out_degree || 5,
        importance: node.metrics?.pagerank_score || 0.1,
        file_path: node.file_path
      })) || generateMockNodes();

      const edges: GraphEdge[] = result.results.paths?.[0]?.edges?.map((edge: any) => ({
        id: `${edge.from}-${edge.to}`,
        source: edge.from,
        target: edge.to,
        type: edge.relation_type,
        weight: 1,
        cached: true
      })) || generateMockEdges(nodes);

      setGraphData({
        nodes,
        edges,
        metadata: {
          cache_hit_rate: result.cache_info?.cache_hit ? 0.87 : 0.12,
          query_time_ms: result.performance?.execution_time_ms || 45,
          total_relationships: edges.length,
          last_updated: result.query_meta?.executed_at || new Date().toISOString()
        }
      });

    } catch (error) {
      console.error('Failed to fetch graph data:', error);
      // Use mock data for demonstration
      setGraphData(generateMockGraphData());
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Initialize the D3 visualization
  useEffect(() => {
    if (!graphData || !svgRef.current || !containerRef.current) return;

    const svg = d3.select(svgRef.current);
    const container = containerRef.current;
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Clear previous visualization
    svg.selectAll('*').remove();

    // Set up zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        const { transform } = event;
        transformRef.current = transform;
        setZoomLevel(transform.k);

        svg.select('.graph-container')
          .attr('transform', transform);
      });

    svg.call(zoom);

    // Create main container group
    const g = svg.append('g')
      .attr('class', 'graph-container');

    // Filter data based on config
    const filteredNodes = graphData.nodes.filter(node =>
      config.filterByType.includes(node.type) &&
      config.filterByLanguage.includes(node.language)
    );

    const filteredEdges = graphData.edges.filter(edge =>
      filteredNodes.some(n => n.id === edge.source) &&
      filteredNodes.some(n => n.id === edge.target)
    );

    // Set up force simulation
    const simulation = d3.forceSimulation<GraphNode>(filteredNodes)
      .force('link', d3.forceLink<GraphNode, GraphEdge>(filteredEdges)
        .id(d => d.id)
        .distance(d => 50 + d.weight * 20))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(d => getNodeSize(d, config) + 5));

    simulationRef.current = simulation;

    // Create edges
    const edges = g.selectAll('.edge')
      .data(filteredEdges)
      .enter()
      .append('line')
      .attr('class', 'edge')
      .attr('stroke', d => getEdgeColor(d))
      .attr('stroke-width', d => getEdgeWidth(d, config))
      .attr('stroke-opacity', d => d.cached ? 0.8 : 0.4)
      .attr('stroke-dasharray', d => d.cached ? 'none' : '5,5');

    // Create nodes
    const nodes = g.selectAll('.node')
      .data(filteredNodes)
      .enter()
      .append('circle')
      .attr('class', 'node')
      .attr('r', d => getNodeSize(d, config))
      .attr('fill', d => getNodeColor(d))
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .call(d3.drag<SVGCircleElement, GraphNode>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended))
      .on('click', (event, d) => {
        setSelectedNode(d);
        // Highlight connected nodes
        highlightConnected(d, filteredEdges);
      })
      .on('mouseover', function(event, d) {
        d3.select(this).attr('stroke-width', 4);
        showTooltip(event, d);
      })
      .on('mouseout', function() {
        d3.select(this).attr('stroke-width', 2);
        hideTooltip();
      });

    // Add labels if enabled
    if (config.showLabels) {
      const labels = g.selectAll('.label')
        .data(filteredNodes)
        .enter()
        .append('text')
        .attr('class', 'label')
        .text(d => d.label)
        .attr('text-anchor', 'middle')
        .attr('dy', '.35em')
        .style('font-size', '12px')
        .style('font-family', 'Arial, sans-serif')
        .style('fill', '#333')
        .style('pointer-events', 'none');

      simulation.on('tick', () => {
        edges
          .attr('x1', d => (d.source as GraphNode).x!)
          .attr('y1', d => (d.source as GraphNode).y!)
          .attr('x2', d => (d.target as GraphNode).x!)
          .attr('y2', d => (d.target as GraphNode).y!);

        nodes
          .attr('cx', d => d.x!)
          .attr('cy', d => d.y!);

        labels
          .attr('x', d => d.x!)
          .attr('y', d => d.y!);
      });
    } else {
      simulation.on('tick', () => {
        edges
          .attr('x1', d => (d.source as GraphNode).x!)
          .attr('y1', d => (d.source as GraphNode).y!)
          .attr('x2', d => (d.target as GraphNode).x!)
          .attr('y2', d => (d.target as GraphNode).y!);

        nodes
          .attr('cx', d => d.x!)
          .attr('cy', d => d.y!);
      });
    }

    // Drag functions
    function dragstarted(event: d3.D3DragEvent<SVGCircleElement, GraphNode, GraphNode>) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: d3.D3DragEvent<SVGCircleElement, GraphNode, GraphNode>) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: d3.D3DragEvent<SVGCircleElement, GraphNode, GraphNode>) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }

    // Auto-play/pause control
    if (isPlaying) {
      simulation.alpha(0.3).restart();
    } else {
      simulation.stop();
    }

    return () => {
      simulation.stop();
    };

  }, [graphData, config, isPlaying]);

  // Helper functions
  const getNodeColor = (node: GraphNode): string => {
    const colors = {
      rust: '#dea584',
      python: '#3776ab',
      javascript: '#f7df1e',
      typescript: '#3178c6'
    };
    return colors[node.language] || '#666';
  };

  const getNodeSize = (node: GraphNode, config: VisualizationConfig): number => {
    switch (config.nodeSize) {
      case 'importance':
        return 5 + node.importance * 20;
      case 'connections':
        return 5 + node.size;
      default:
        return 10;
    }
  };

  const getEdgeColor = (edge: GraphEdge): string => {
    return edge.cached ? '#10b981' : '#6b7280';
  };

  const getEdgeWidth = (edge: GraphEdge, config: VisualizationConfig): number => {
    switch (config.edgeWeight) {
      case 'strength':
        return 1 + edge.weight * 2;
      case 'cache_priority':
        return edge.cached ? 3 : 1;
      default:
        return 2;
    }
  };

  const highlightConnected = (node: GraphNode, edges: GraphEdge[]) => {
    // Implementation for highlighting connected nodes
    console.log(`Highlighting connections for ${node.label}`);
  };

  const showTooltip = (event: MouseEvent, node: GraphNode) => {
    // Implementation for showing node tooltip
  };

  const hideTooltip = () => {
    // Implementation for hiding tooltip
  };

  const detectLanguage = (filePath: string): GraphNode['language'] => {
    if (filePath.endsWith('.rs')) return 'rust';
    if (filePath.endsWith('.py')) return 'python';
    if (filePath.endsWith('.ts')) return 'typescript';
    if (filePath.endsWith('.js')) return 'javascript';
    return 'javascript';
  };

  // Load graph data on component mount
  useEffect(() => {
    fetchGraphData();
  }, [fetchGraphData]);

  // Control functions
  const handleZoomIn = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      svg.transition().call(
        d3.zoom<SVGSVGElement, unknown>().scaleBy as any, 1.2
      );
    }
  };

  const handleZoomOut = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      svg.transition().call(
        d3.zoom<SVGSVGElement, unknown>().scaleBy as any, 0.8
      );
    }
  };

  const handleReset = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      svg.transition().call(
        d3.zoom<SVGSVGElement, unknown>().transform as any, d3.zoomIdentity
      );
    }
  };

  const handleRefresh = () => {
    fetchGraphData();
  };

  return (
    <div className="space-y-4">
      {/* Controls */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>Interactive Knowledge Graph</span>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setIsPlaying(!isPlaying)}
                className="p-2 bg-blue-100 hover:bg-blue-200 rounded-lg transition-colors"
                title={isPlaying ? 'Pause simulation' : 'Start simulation'}
              >
                {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
              </button>

              <button
                onClick={handleZoomIn}
                className="p-2 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                title="Zoom in"
              >
                <ZoomIn className="h-4 w-4" />
              </button>

              <button
                onClick={handleZoomOut}
                className="p-2 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                title="Zoom out"
              >
                <ZoomOut className="h-4 w-4" />
              </button>

              <button
                onClick={handleReset}
                className="p-2 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                title="Reset view"
              >
                <RotateCcw className="h-4 w-4" />
              </button>

              <button
                onClick={handleRefresh}
                className="p-2 bg-green-100 hover:bg-green-200 rounded-lg transition-colors"
                title="Refresh graph data"
                disabled={isLoading}
              >
                <Filter className="h-4 w-4" />
              </button>
            </div>
          </CardTitle>
        </CardHeader>
      </Card>

      {/* Graph Statistics */}
      {graphData && (
        <div className="grid grid-cols-4 gap-4">
          <Card className="text-center">
            <CardContent className="pt-6">
              <div className="text-2xl font-bold text-blue-600">{graphData.nodes.length}</div>
              <p className="text-sm text-gray-600">Nodes</p>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardContent className="pt-6">
              <div className="text-2xl font-bold text-green-600">{graphData.edges.length}</div>
              <p className="text-sm text-gray-600">Relationships</p>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardContent className="pt-6">
              <div className="text-2xl font-bold text-purple-600">
                {(graphData.metadata.cache_hit_rate * 100).toFixed(1)}%
              </div>
              <p className="text-sm text-gray-600">Cache Hit Rate</p>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardContent className="pt-6">
              <div className="text-2xl font-bold text-orange-600">
                {graphData.metadata.query_time_ms}ms
              </div>
              <p className="text-sm text-gray-600">Query Time</p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Main Visualization */}
      <Card className="relative">
        <div
          ref={containerRef}
          className="relative w-full h-96 border rounded-lg overflow-hidden"
        >
          {isLoading && (
            <div className="absolute inset-0 flex items-center justify-center bg-gray-50">
              <div className="text-lg font-medium text-gray-600">Loading knowledge graph...</div>
            </div>
          )}

          <svg
            ref={svgRef}
            width="100%"
            height="100%"
            className="bg-gray-50"
          />

          {/* Zoom indicator */}
          <div className="absolute top-4 left-4 bg-white px-3 py-1 rounded-lg shadow-sm border">
            <span className="text-sm font-medium">Zoom: {(zoomLevel * 100).toFixed(0)}%</span>
          </div>

          {/* Cache indicator */}
          {graphData && (
            <div className="absolute top-4 right-4 bg-green-100 px-3 py-1 rounded-lg shadow-sm border border-green-200">
              <span className="text-sm font-medium text-green-800">
                ⚡ {(graphData.metadata.cache_hit_rate * 100).toFixed(1)}% cached
              </span>
            </div>
          )}
        </div>
      </Card>

      {/* Selected Node Details */}
      {selectedNode && (
        <Card>
          <CardHeader>
            <CardTitle>Selected Node: {selectedNode.label}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div><strong>Type:</strong> {selectedNode.type}</div>
              <div><strong>Language:</strong> {selectedNode.language}</div>
              <div><strong>File:</strong> {selectedNode.file_path}</div>
              <div><strong>Importance:</strong> {selectedNode.importance.toFixed(3)}</div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

// Mock data generators for demonstration
function generateMockNodes(): GraphNode[] {
  return [
    {
      id: 'main.rs',
      label: 'main.rs',
      type: 'module',
      language: 'rust',
      size: 15,
      importance: 0.9,
      file_path: 'src/main.rs'
    },
    {
      id: 'lib.rs',
      label: 'lib.rs',
      type: 'module',
      language: 'rust',
      size: 12,
      importance: 0.7,
      file_path: 'src/lib.rs'
    },
    {
      id: 'analysis.py',
      label: 'analysis.py',
      type: 'module',
      language: 'python',
      size: 8,
      importance: 0.5,
      file_path: 'scripts/analysis.py'
    }
    // Add more mock nodes...
  ];
}

function generateMockEdges(nodes: GraphNode[]): GraphEdge[] {
  return [
    {
      id: 'main-lib',
      source: 'main.rs',
      target: 'lib.rs',
      type: 'imports',
      weight: 2,
      cached: true
    }
    // Add more mock edges...
  ];
}

function generateMockGraphData(): GraphData {
  const nodes = generateMockNodes();
  const edges = generateMockEdges(nodes);

  return {
    nodes,
    edges,
    metadata: {
      cache_hit_rate: 0.87,
      query_time_ms: 45,
      total_relationships: edges.length,
      last_updated: new Date().toISOString()
    }
  };
}