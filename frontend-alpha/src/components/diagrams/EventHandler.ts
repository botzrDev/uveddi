// Event Handler for Interactive Diagrams - UV-89 Phase 1
// Manages click, double-click, hover, and context menu events

import * as d3 from 'd3';
import type { NodeEventData } from './types';

interface EventHandlerCallbacks {
  onNodeClick?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeDoubleClick?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeHover?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeContextMenu?: (nodeId: string, nodeData: NodeEventData, event: MouseEvent) => void;
}

export class EventHandler {
  private svgElement: SVGElement;
  private d3Selection: d3.Selection<SVGElement, unknown, null, undefined>;
  private callbacks: EventHandlerCallbacks;
  private clickTimeout: number | null = null;
  private readonly doubleClickDelay = 300; // ms

  constructor(svgElement: SVGElement, callbacks: EventHandlerCallbacks) {
    this.svgElement = svgElement;
    this.d3Selection = d3.select(svgElement);
    this.callbacks = callbacks;
    this.initializeEventListeners();
  }

  private initializeEventListeners(): void {
    // Add CSS classes for interactive elements
    this.addInteractiveStyles();

    // Set up event listeners on node elements
    this.setupNodeEventListeners();

    // Prevent default context menu on SVG
    this.d3Selection.on('contextmenu', (event) => {
      event.preventDefault();
    });
  }

  private addInteractiveStyles(): void {
    // Add CSS styles for interactive elements
    const style = document.createElement('style');
    style.textContent = `
      .interactive-diagram .node {
        cursor: pointer;
        transition: opacity 0.2s ease, transform 0.2s ease;
      }
      
      .interactive-diagram .node:hover {
        opacity: 0.8;
        transform: scale(1.05);
      }
      
      .interactive-diagram .node-selected {
        stroke: #007acc !important;
        stroke-width: 2px !important;
        filter: drop-shadow(0 0 4px #007acc);
      }
      
      .interactive-diagram .node-hovered {
        filter: drop-shadow(0 0 6px rgba(0, 122, 204, 0.5));
      }
      
      .interactive-diagram .edge {
        transition: stroke-width 0.2s ease, opacity 0.2s ease;
      }
      
      .interactive-diagram .edge-highlighted {
        stroke: #007acc !important;
        stroke-width: 2px !important;
        opacity: 1 !important;
      }
      
      .interactive-diagram .node-keyboard-focus {
        outline: 2px solid #007acc;
        outline-offset: 2px;
      }
    `;
    
    if (!document.querySelector('#interactive-diagram-styles')) {
      style.id = 'interactive-diagram-styles';
      document.head.appendChild(style);
    }
  }

  private setupNodeEventListeners(): void {
    // Find all node elements - Mermaid can use various selectors
    const nodeSelectors = [
      '.node',
      '.nodeLabel', 
      '[id^="flowchart-"]',
      'g[class*="node"]',
      'rect[class*="node"]',
      'circle[class*="node"]'
    ];

    nodeSelectors.forEach(selector => {
      this.d3Selection.selectAll(selector)
        .on('click', this.handleNodeClick.bind(this))
        .on('contextmenu', this.handleNodeContextMenu.bind(this))
        .on('mouseenter', this.handleNodeMouseEnter.bind(this))
        .on('mouseleave', this.handleNodeMouseLeave.bind(this));
    });
  }

  private handleNodeClick(event: MouseEvent, _d?: any): void {
    event.stopPropagation();
    
    const nodeElement = event.currentTarget as Element;
    const nodeData = this.extractNodeData(nodeElement);

    // Handle double-click detection
    if (this.clickTimeout) {
      // This is a double-click
      clearTimeout(this.clickTimeout);
      this.clickTimeout = null;
      this.handleNodeDoubleClick(event, _d);
      return;
    }

    // Set timeout for single click
    this.clickTimeout = window.setTimeout(() => {
      this.clickTimeout = null;
      
      // Clear previous selections
      this.clearSelections();
      
      // Add selection styling
      d3.select(nodeElement).classed('node-selected', true);
      
      // Highlight connected nodes and edges
      this.highlightConnections(nodeData.nodeId);
      
      // Trigger callback
      this.callbacks.onNodeClick?.(nodeData.nodeId, nodeData);
    }, this.doubleClickDelay);
  }

  private handleNodeDoubleClick(event: MouseEvent, _d?: any): void {
    event.stopPropagation();
    
    const nodeElement = event.currentTarget as Element;
    const nodeData = this.extractNodeData(nodeElement);
    
    // Trigger callback
    this.callbacks.onNodeDoubleClick?.(nodeData.nodeId, nodeData);
  }

  private handleNodeContextMenu(event: MouseEvent, _d?: any): void {
    event.preventDefault();
    event.stopPropagation();
    
    const nodeElement = event.currentTarget as Element;
    const nodeData = this.extractNodeData(nodeElement);
    
    // Trigger callback
    this.callbacks.onNodeContextMenu?.(nodeData.nodeId, nodeData, event);
  }

  private handleNodeMouseEnter(event: MouseEvent, _d?: any): void {
    const nodeElement = event.currentTarget as Element;
    const nodeData = this.extractNodeData(nodeElement);
    
    // Add hover styling
    d3.select(nodeElement).classed('node-hovered', true);
    
    // Trigger callback
    this.callbacks.onNodeHover?.(nodeData.nodeId, nodeData);
  }

  private handleNodeMouseLeave(event: MouseEvent, _d?: any): void {
    const nodeElement = event.currentTarget as Element;
    
    // Remove hover styling
    d3.select(nodeElement).classed('node-hovered', false);
  }

  private extractNodeData(nodeElement: Element): NodeEventData {
    const nodeId = nodeElement.id || `node-${Math.random().toString(36).substr(2, 9)}`;
    const textContent = this.getNodeText(nodeElement);
    
    // Get position from element
    const position = this.getNodePosition(nodeElement);
    
    // Extract connections (simplified - would need more sophisticated analysis)
    const connections = this.findConnectedNodes(nodeId);
    
    return {
      nodeId,
      nodeType: this.determineNodeType(nodeElement),
      componentData: {
        name: textContent,
        filePath: this.inferFilePath(textContent),
        componentType: this.inferComponentType(nodeElement),
      },
      position,
      connections,
    };
  }

  private getNodeText(nodeElement: Element): string {
    // Try to find text content in various ways
    const textElement = nodeElement.querySelector('text, .nodeLabel, tspan');
    if (textElement) {
      return textElement.textContent?.trim() || '';
    }
    
    // Fallback to element text content
    return nodeElement.textContent?.trim() || nodeElement.id || 'Unknown';
  }

  private getNodePosition(nodeElement: Element): { x: number; y: number } {
    try {
      if (nodeElement instanceof SVGGraphicsElement) {
        const bbox = nodeElement.getBBox();
        return { x: bbox.x + bbox.width / 2, y: bbox.y + bbox.height / 2 };
      }
    } catch (error) {
      console.warn('Could not get bounding box for node:', error);
    }
    
    return { x: 0, y: 0 };
  }

  private findConnectedNodes(nodeId: string): string[] {
    const connections: string[] = [];
    
    // Find edges connected to this node
    const edges = this.svgElement.querySelectorAll('.edge, path[class*="edge"], line[class*="edge"]');
    
    edges.forEach(edge => {
      const edgeId = edge.id;
      // Simple heuristic: if edge ID contains node ID, consider it connected
      // More sophisticated logic would analyze the actual edge connections
      if (edgeId.includes(nodeId)) {
        // Extract other node IDs from edge (simplified)
        const parts = edgeId.split('-');
        parts.forEach(part => {
          if (part !== nodeId && part.startsWith('node')) {
            connections.push(part);
          }
        });
      }
    });
    
    return connections;
  }

  private determineNodeType(nodeElement: Element): string {
    const classList = (nodeElement as any).className?.baseVal || nodeElement.className;
    
    if (typeof classList === 'string') {
      if (classList.includes('decision')) return 'decision';
      if (classList.includes('process')) return 'process';
      if (classList.includes('start')) return 'start';
      if (classList.includes('end')) return 'end';
    }
    
    return 'component';
  }

  private inferFilePath(nodeName: string): string {
    // Simple heuristic for generating file paths
    const cleanName = nodeName.toLowerCase().replace(/[^a-z0-9]/g, '_');
    return `src/components/${cleanName}.rs`;
  }

  private inferComponentType(nodeElement: Element): string {
    const shape = this.getNodeShape(nodeElement);
    
    switch (shape) {
      case 'rect':
      case 'rectangle': return 'module';
      case 'circle': return 'function';
      case 'diamond': return 'decision';
      default: return 'component';
    }
  }

  private getNodeShape(nodeElement: Element): string {
    // Determine shape based on child elements
    if (nodeElement.querySelector('rect')) return 'rect';
    if (nodeElement.querySelector('circle')) return 'circle';
    if (nodeElement.querySelector('polygon')) return 'diamond';
    
    return 'rect'; // default
  }

  private highlightConnections(nodeId: string): void {
    // Clear previous highlights
    this.d3Selection.selectAll('.edge-highlighted').classed('edge-highlighted', false);
    
    // Find and highlight connected edges
    const connections = this.findConnectedNodes(nodeId);
    
    connections.forEach(connectedNodeId => {
      this.d3Selection.selectAll('.edge, path[class*="edge"]')
        .filter((_d, i, nodes) => {
          const edge = nodes[i] as Element;
          const edgeId = edge.id;
          return edgeId.includes(nodeId) && edgeId.includes(connectedNodeId);
        })
        .classed('edge-highlighted', true);
    });
  }

  private clearSelections(): void {
    this.d3Selection.selectAll('.node-selected').classed('node-selected', false);
    this.d3Selection.selectAll('.edge-highlighted').classed('edge-highlighted', false);
  }

  public selectNode(nodeId: string): void {
    this.clearSelections();
    
    this.d3Selection.selectAll('.node, .nodeLabel, [id^="flowchart-"]')
      .filter((_d, i, nodes) => {
        const node = nodes[i] as Element;
        return node.id === nodeId;
      })
      .classed('node-selected', true);
      
    this.highlightConnections(nodeId);
  }

  public clearAllSelections(): void {
    this.clearSelections();
  }

  public destroy(): void {
    // Clear timeouts
    if (this.clickTimeout) {
      clearTimeout(this.clickTimeout);
      this.clickTimeout = null;
    }
    
    // Remove event listeners
    this.d3Selection.selectAll('.node, .nodeLabel, [id^="flowchart-"]')
      .on('click', null)
      .on('contextmenu', null)
      .on('mouseenter', null)
      .on('mouseleave', null);
    
    this.d3Selection.on('contextmenu', null);
  }
}