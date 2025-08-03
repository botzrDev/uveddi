// Interactive Diagram Component - UV-89 Phase 1
// Transforms static Mermaid diagrams into interactive, clickable experiences

import React, { useEffect, useRef, useState, useCallback } from 'react';
import mermaid from 'mermaid';
import type { InteractiveDiagramProps, DiagramState, NodeEventData } from './types';
import { EventHandler } from './EventHandler';
import { PanZoomController } from './PanZoomController';
import { TooltipManager } from './TooltipManager';
import { KeyboardController } from './KeyboardController';
import { ContextMenu } from './ContextMenu';
import { AnimationController, type AnimationConfig } from './AnimationController';
import { FlowAnimator, type FlowAnimationConfig } from './FlowAnimator';
import { ProgressiveBuilder, type ProgressiveBuildConfig } from './ProgressiveBuilder';
import { StyleController } from './StyleController';
import { ThemeManager, type DiagramTheme } from './ThemeManager';
import { ComponentStylingEngine } from './ComponentStylingEngine';
import { ExportController } from './ExportController';

// Initialize Mermaid
mermaid.initialize({
  startOnLoad: false,
  theme: 'default',
  securityLevel: 'loose',
  htmlLabels: true,
  flowchart: {
    useMaxWidth: false,
    htmlLabels: true,
  },
});

export const InteractiveDiagram: React.FC<InteractiveDiagramProps> = ({
  mermaidCode,
  onNodeClick,
  onNodeDoubleClick,
  onNodeHover,
  onNodeContextMenu,
  enablePanZoom = true,
  enableTooltips = true,
  enableKeyboardNav = true,
  enableAnimations = false,
  enableAdvancedStyling = false,
  enableExport = false,
  animationConfig = {
    enableFlowAnimations: true,
    enableProgressiveBuilding: true,
    enableTransitions: true,
    animationSpeed: 1,
    autoPlay: false
  },
  initialTheme,
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGElement | null>(null);
  const [diagramState, setDiagramState] = useState<DiagramState>({
    selectedNodeId: null,
    hoveredNodeId: null,
    contextMenuVisible: false,
    contextMenuPosition: null,
    tooltipVisible: false,
    tooltipData: null,
    zoomLevel: 1,
    panPosition: { x: 0, y: 0 },
  });

  const [eventHandler, setEventHandler] = useState<EventHandler | null>(null);
  const [panZoomController, setPanZoomController] = useState<PanZoomController | null>(null);
  const [tooltipManager, setTooltipManager] = useState<TooltipManager | null>(null);
  const [keyboardController, setKeyboardController] = useState<KeyboardController | null>(null);
  const [flowAnimator, setFlowAnimator] = useState<FlowAnimator | null>(null);
  const [progressiveBuilder, setProgressiveBuilder] = useState<ProgressiveBuilder | null>(null);
  const [themeManager, setThemeManager] = useState<ThemeManager | null>(null);
  const [stylingEngine, setStylingEngine] = useState<ComponentStylingEngine | null>(null);
  const [nodes, setNodes] = useState<NodeEventData[]>([]);
  const [edges, setEdges] = useState<any[]>([]);

  // Render the Mermaid diagram
  const renderDiagram = useCallback(async () => {
    if (!containerRef.current || !mermaidCode.trim()) return;

    try {
      // Generate unique ID for this diagram
      const diagramId = `mermaid-${Date.now()}`;
      
      // Render the diagram
      const { svg } = await mermaid.render(diagramId, mermaidCode);
      
      // Insert SVG into container
      containerRef.current.innerHTML = svg;
      
      // Get reference to the SVG element
      const svgElement = containerRef.current.querySelector('svg');
      if (!svgElement) {
        console.error('Failed to find SVG element after rendering');
        return;
      }

      svgRef.current = svgElement;

      // Add CSS classes for styling
      svgElement.classList.add('interactive-diagram');
      svgElement.style.width = '100%';
      svgElement.style.height = '100%';

      // Extract node and edge data from the rendered SVG
      const extractedNodes = extractNodeData(svgElement);
      const extractedEdges = extractEdgeData(svgElement);
      setNodes(extractedNodes);
      setEdges(extractedEdges);

      // Initialize interaction systems
      await initializeInteractionSystems(svgElement, extractedNodes, extractedEdges);

    } catch (error) {
      console.error('Error rendering Mermaid diagram:', error);
      if (containerRef.current) {
        containerRef.current.innerHTML = `<div class="error-message">Failed to render diagram: ${error}</div>`;
      }
    }
  }, [mermaidCode]);

  // Extract node data from rendered SVG
  const extractNodeData = (svgElement: SVGElement): NodeEventData[] => {
    const nodes: NodeEventData[] = [];
    
    // Find all node elements (Mermaid typically uses .node class)
    const nodeElements = svgElement.querySelectorAll('.node, .nodeLabel, [id^="flowchart-"]');
    
    nodeElements.forEach((element, index) => {
      const nodeId = element.id || `node-${index}`;
      const textContent = element.textContent?.trim() || `Node ${index}`;
      
      // Get bounding box for position
      const bbox = (element as SVGGraphicsElement).getBBox?.() || { x: 0, y: 0 };
      
      // Extract node information (this would be enhanced with real data)
      const nodeData: NodeEventData = {
        nodeId,
        nodeType: 'component', // Default type
        componentData: {
          name: textContent,
          filePath: `src/components/${textContent}.rs`, // Example path
          componentType: 'module',
        },
        position: { x: bbox.x, y: bbox.y },
        connections: [], // Would be populated from diagram analysis
      };

      nodes.push(nodeData);
    });

    return nodes;
  };

  // Extract edge data from rendered SVG
  const extractEdgeData = (svgElement: SVGElement): any[] => {
    const edges: any[] = [];
    
    // Find all edge elements (Mermaid typically uses .edge class)
    const edgeElements = svgElement.querySelectorAll('.edge, .edgePath, [id^="L-"]');
    
    edgeElements.forEach((element, index) => {
      const edgeId = element.id || `edge-${index}`;
      const pathElement = element.querySelector('path') || element;
      
      edges.push({
        id: edgeId,
        element: element,
        pathElement: pathElement,
        source: null, // Would be determined from diagram analysis
        target: null,
        label: element.textContent?.trim() || ''
      });
    });

    return edges;
  };

  // Initialize all interaction systems
  const initializeInteractionSystems = async (svgElement: SVGElement, nodeData: NodeEventData[], edgeData: any[]) => {
    // Clean up previous instances
    eventHandler?.destroy();
    panZoomController?.destroy();
    tooltipManager?.destroy();
    keyboardController?.destroy();
    flowAnimator?.dispose();
    progressiveBuilder?.dispose();
    themeManager?.dispose();
    stylingEngine?.dispose();

    // Create event handler
    const newEventHandler = new EventHandler(svgElement, {
      onNodeClick: (nodeId: string, data: NodeEventData) => {
        setDiagramState(prev => ({ ...prev, selectedNodeId: nodeId }));
        onNodeClick?.(nodeId, data);
      },
      onNodeDoubleClick: (nodeId: string, data: NodeEventData) => {
        onNodeDoubleClick?.(nodeId, data);
      },
      onNodeHover: (nodeId: string, data: NodeEventData) => {
        setDiagramState(prev => ({ ...prev, hoveredNodeId: nodeId }));
        onNodeHover?.(nodeId, data);
      },
      onNodeContextMenu: (nodeId: string, data: NodeEventData, event: MouseEvent) => {
        setDiagramState(prev => ({
          ...prev,
          contextMenuVisible: true,
          contextMenuPosition: { x: event.clientX, y: event.clientY },
        }));
        onNodeContextMenu?.(nodeId, data, event);
      },
    });

    setEventHandler(newEventHandler);

    // Create pan/zoom controller if enabled
    if (enablePanZoom) {
      const containerGroup = svgElement.querySelector('g') as SVGGElement;
      if (containerGroup) {
        const newPanZoomController = new PanZoomController(svgElement, containerGroup);
        setPanZoomController(newPanZoomController);
      }
    }

    // Create tooltip manager if enabled
    if (enableTooltips) {
      const newTooltipManager = new TooltipManager(svgElement, nodeData);
      setTooltipManager(newTooltipManager);
    }

    // Create keyboard controller if enabled
    if (enableKeyboardNav) {
      const newKeyboardController = new KeyboardController(
        nodeData,
        (nodeId: string) => {
          setDiagramState(prev => ({ ...prev, selectedNodeId: nodeId }));
          // Optionally trigger click event
          const nodeData = nodes.find(n => n.nodeId === nodeId);
          if (nodeData) {
            onNodeClick?.(nodeId, nodeData);
          }
        }
      );
      setKeyboardController(newKeyboardController);
    }

    // Initialize animation systems if enabled
    if (enableAnimations) {
      const containerGroup = svgElement.querySelector('g') as SVGGElement;
      if (containerGroup) {
        // Initialize Flow Animator
        const newFlowAnimator = new FlowAnimator(svgElement, containerGroup);
        newFlowAnimator.identifyFlowPaths();
        setFlowAnimator(newFlowAnimator);

        // Initialize Progressive Builder
        const newProgressiveBuilder = new ProgressiveBuilder(svgElement, containerGroup);
        newProgressiveBuilder.analyzeDiagramStructure();
        setProgressiveBuilder(newProgressiveBuilder);
      }
    }

    // Initialize advanced styling systems if enabled
    if (enableAdvancedStyling) {
      // Initialize Theme Manager
      const newThemeManager = new ThemeManager(svgElement, initialTheme);
      setThemeManager(newThemeManager);

      // Initialize Component Styling Engine
      const newStylingEngine = new ComponentStylingEngine(svgElement, newThemeManager.getCurrentTheme());
      setStylingEngine(newStylingEngine);

      // Apply initial styling to components
      newStylingEngine.applyComponentStyling(nodeData);
    }
  };

  // Handle context menu actions
  const handleContextMenuAction = useCallback((action: string, nodeId: string) => {
    console.log(`Context menu action: ${action} for node: ${nodeId}`);
    
    switch (action) {
      case 'view-details':
        // Handle view details action
        break;
      case 'highlight-dependencies':
        // Handle highlight dependencies action
        break;
      case 'focus-component':
        // Handle focus component action
        panZoomController?.focusOnNode(nodeId);
        break;
      case 'export-subgraph':
        // Handle export subgraph action
        break;
      case 'add-comment':
        // Handle add comment action
        break;
    }

    // Close context menu
    setDiagramState(prev => ({
      ...prev,
      contextMenuVisible: false,
      contextMenuPosition: null,
    }));
  }, [panZoomController]);

  // Close context menu when clicking outside
  const handleContainerClick = useCallback((_event: React.MouseEvent) => {
    if (diagramState.contextMenuVisible) {
      setDiagramState(prev => ({
        ...prev,
        contextMenuVisible: false,
        contextMenuPosition: null,
      }));
    }
  }, [diagramState.contextMenuVisible]);

  // Effect to render diagram when mermaidCode changes
  useEffect(() => {
    renderDiagram();
  }, [renderDiagram]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      eventHandler?.destroy();
      panZoomController?.destroy();
      tooltipManager?.destroy();
      keyboardController?.destroy();
      flowAnimator?.dispose();
      progressiveBuilder?.dispose();
      themeManager?.dispose();
      stylingEngine?.dispose();
    };
  }, [eventHandler, panZoomController, tooltipManager, keyboardController, flowAnimator, progressiveBuilder, themeManager, stylingEngine]);

  return (
    <div 
      className={`interactive-diagram-container ${className}`}
      onClick={handleContainerClick}
      style={{ position: 'relative', width: '100%', height: '100%' }}
    >
      <div 
        ref={containerRef}
        className="diagram-content"
        style={{ width: '100%', height: '100%' }}
        role="img"
        aria-label="Interactive diagram"
        tabIndex={enableKeyboardNav ? 0 : -1}
      />

      {/* Context Menu */}
      {diagramState.contextMenuVisible && diagramState.contextMenuPosition && (
        <ContextMenu
          nodeId={diagramState.selectedNodeId || ''}
          nodeData={nodes.find(n => n.nodeId === diagramState.selectedNodeId) || nodes[0]}
          position={diagramState.contextMenuPosition}
          onClose={() => setDiagramState(prev => ({
            ...prev,
            contextMenuVisible: false,
            contextMenuPosition: null,
          }))}
          onAction={handleContextMenuAction}
        />
      )}

      {/* Toolbar for zoom controls (optional) */}
      {enablePanZoom && (
        <div className="diagram-toolbar" style={{
          position: 'absolute',
          top: '10px',
          right: '10px',
          background: 'rgba(255, 255, 255, 0.9)',
          border: '1px solid #ccc',
          borderRadius: '4px',
          padding: '4px',
          display: 'flex',
          gap: '4px',
        }}>
          <button
            onClick={() => panZoomController?.zoomIn()}
            title="Zoom In"
            style={{ padding: '4px 8px', fontSize: '12px' }}
          >
            +
          </button>
          <button
            onClick={() => panZoomController?.zoomOut()}
            title="Zoom Out"
            style={{ padding: '4px 8px', fontSize: '12px' }}
          >
            -
          </button>
          <button
            onClick={() => panZoomController?.zoomToFit()}
            title="Fit to View"
            style={{ padding: '4px 8px', fontSize: '12px' }}
          >
            ⊡
          </button>
        </div>
      )}

      {/* Animation Controller */}
      {enableAnimations && svgRef.current && (
        <AnimationController
          svgRef={svgRef}
          nodeData={nodes}
          edgeData={edges}
          config={animationConfig}
          onAnimationStart={(type) => console.log(`Animation started: ${type}`)}
          onAnimationComplete={(type) => console.log(`Animation completed: ${type}`)}
        />
      )}

      {/* Style Controller */}
      {enableAdvancedStyling && (
        <StyleController
          svgRef={svgRef}
          nodeData={nodes}
          onThemeChange={(theme) => {
            console.log('Theme changed:', theme.name);
            stylingEngine?.updateTheme(theme);
          }}
          onStyleUpdate={() => {
            console.log('Styles updated');
          }}
        />
      )}

      {/* Export Controller */}
      {enableExport && (
        <ExportController
          svgRef={svgRef}
          nodeData={nodes}
          theme={themeManager?.getCurrentTheme()}
          mermaidCode={mermaidCode}
          onExportStart={(format) => console.log(`Export started: ${format}`)}
          onExportComplete={(format, success) => 
            console.log(`Export ${success ? 'completed' : 'failed'}: ${format}`)
          }
          onExportError={(format, error) => 
            console.error(`Export error for ${format}:`, error)
          }
        />
      )}
    </div>
  );
};

export default InteractiveDiagram;