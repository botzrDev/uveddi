// Tooltip Manager for Interactive Diagrams - UV-89 Phase 1
// Manages tooltip display and positioning for diagram nodes

import * as d3 from 'd3';
import type { NodeEventData } from './types';

export class TooltipManager {
  private svgElement: SVGElement;
  private nodes: NodeEventData[];
  private tooltip: d3.Selection<HTMLDivElement, unknown, HTMLElement, unknown> | null = null;
  private tooltipTimeout: number | null = null;
  private readonly showDelay = 500; // ms
  private readonly hideDelay = 100; // ms

  constructor(svgElement: SVGElement, nodes: NodeEventData[]) {
    this.svgElement = svgElement;
    this.nodes = nodes;
    this.initializeTooltip();
    this.setupEventListeners();
  }

  private initializeTooltip(): void {
    // Create tooltip element
    this.tooltip = d3.select('body')
      .append('div')
      .attr('class', 'diagram-tooltip')
      .style('position', 'absolute')
      .style('visibility', 'hidden')
      .style('background-color', '#333')
      .style('color', 'white')
      .style('padding', '8px 12px')
      .style('border-radius', '4px')
      .style('font-size', '12px')
      .style('font-family', 'system-ui, -apple-system, sans-serif')
      .style('box-shadow', '0 2px 8px rgba(0,0,0,0.2)')
      .style('pointer-events', 'none')
      .style('z-index', '10000')
      .style('max-width', '300px')
      .style('word-wrap', 'break-word');

    // Add tooltip styles to document if not already present
    this.addTooltipStyles();
  }

  private addTooltipStyles(): void {
    if (document.querySelector('#diagram-tooltip-styles')) return;

    const style = document.createElement('style');
    style.id = 'diagram-tooltip-styles';
    style.textContent = `
      .diagram-tooltip {
        transition: opacity 0.2s ease, visibility 0.2s ease;
        opacity: 0;
      }
      
      .diagram-tooltip.visible {
        opacity: 1;
        visibility: visible !important;
      }
      
      .diagram-tooltip .tooltip-header {
        font-weight: bold;
        margin-bottom: 4px;
        color: #ffffff;
      }
      
      .diagram-tooltip .tooltip-content {
        color: #cccccc;
        line-height: 1.4;
      }
      
      .diagram-tooltip .tooltip-metadata {
        margin-top: 8px;
        padding-top: 8px;
        border-top: 1px solid #555;
        font-size: 11px;
        color: #aaaaaa;
      }
      
      .diagram-tooltip .tooltip-connection {
        margin-top: 4px;
        font-size: 11px;
        color: #888888;
      }
      
      .diagram-tooltip .tooltip-metrics {
        margin-top: 6px;
        font-size: 11px;
        color: #aaaaaa;
      }
    `;
    
    document.head.appendChild(style);
  }

  private setupEventListeners(): void {
    // Find all node elements and add tooltip event listeners
    const nodeSelectors = [
      '.node',
      '.nodeLabel', 
      '[id^="flowchart-"]',
      'g[class*="node"]',
      'rect[class*="node"]',
      'circle[class*="node"]'
    ];

    nodeSelectors.forEach(selector => {
      d3.select(this.svgElement).selectAll(selector)
        .on('mouseenter.tooltip', this.handleMouseEnter.bind(this))
        .on('mouseleave.tooltip', this.handleMouseLeave.bind(this))
        .on('mousemove.tooltip', this.handleMouseMove.bind(this));
    });
  }

  private handleMouseEnter(event: MouseEvent): void {
    // Clear any existing timeout
    if (this.tooltipTimeout) {
      clearTimeout(this.tooltipTimeout);
    }

    // Set timeout to show tooltip
    this.tooltipTimeout = window.setTimeout(() => {
      const nodeData = this.extractNodeData(event.currentTarget as Element);
      this.showTooltip(event, nodeData);
    }, this.showDelay);
  }

  private handleMouseLeave(_event: MouseEvent): void {
    // Clear show timeout
    if (this.tooltipTimeout) {
      clearTimeout(this.tooltipTimeout);
      this.tooltipTimeout = null;
    }

    // Set timeout to hide tooltip
    this.tooltipTimeout = window.setTimeout(() => {
      this.hideTooltip();
    }, this.hideDelay);
  }

  private handleMouseMove(event: MouseEvent): void {
    if (this.tooltip && this.tooltip.classed('visible')) {
      this.updateTooltipPosition(event);
    }
  }

  private extractNodeData(nodeElement: Element): NodeEventData {
    const nodeId = nodeElement.id || `node-${Math.random().toString(36).substr(2, 9)}`;
    
    // Try to find existing node data first
    const existingNode = this.nodes.find(node => node.nodeId === nodeId);
    if (existingNode) {
      return existingNode;
    }

    // Extract data from element if not found
    const textContent = this.getNodeText(nodeElement);
    const position = this.getNodePosition(nodeElement);
    
    return {
      nodeId,
      nodeType: 'component',
      componentData: {
        name: textContent,
        filePath: `src/components/${textContent.toLowerCase().replace(/[^a-z0-9]/g, '_')}.rs`,
        componentType: 'module',
      },
      position,
      connections: [],
    };
  }

  private getNodeText(nodeElement: Element): string {
    const textElement = nodeElement.querySelector('text, .nodeLabel, tspan');
    if (textElement) {
      return textElement.textContent?.trim() || '';
    }
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

  private showTooltip(event: MouseEvent, nodeData: NodeEventData): void {
    if (!this.tooltip) return;

    const content = this.generateTooltipContent(nodeData);
    
    this.tooltip
      .html(content)
      .classed('visible', true);

    this.updateTooltipPosition(event);
  }

  private hideTooltip(): void {
    if (!this.tooltip) return;

    this.tooltip
      .classed('visible', false);
  }

  private updateTooltipPosition(event: MouseEvent): void {
    if (!this.tooltip) return;

    const tooltipNode = this.tooltip.node();
    if (!tooltipNode) return;

    const tooltipRect = tooltipNode.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    
    let left = event.pageX + 10;
    let top = event.pageY - 10;

    // Adjust position to keep tooltip in viewport
    if (left + tooltipRect.width > viewportWidth) {
      left = event.pageX - tooltipRect.width - 10;
    }

    if (top + tooltipRect.height > viewportHeight) {
      top = event.pageY - tooltipRect.height - 10;
    }

    // Ensure tooltip doesn't go off-screen
    left = Math.max(5, Math.min(left, viewportWidth - tooltipRect.width - 5));
    top = Math.max(5, Math.min(top, viewportHeight - tooltipRect.height - 5));

    this.tooltip
      .style('left', `${left}px`)
      .style('top', `${top}px`);
  }

  private generateTooltipContent(nodeData: NodeEventData): string {
    const { componentData, connections } = nodeData;
    
    let content = `
      <div class="tooltip-header">${componentData.name}</div>
      <div class="tooltip-content">
        <div><strong>Type:</strong> ${componentData.componentType}</div>
        <div><strong>Path:</strong> ${componentData.filePath}</div>
      </div>
    `;

    // Add metrics if available
    if (componentData.metrics) {
      content += `
        <div class="tooltip-metrics">
          <div><strong>Metrics:</strong></div>
          ${this.formatMetrics(componentData.metrics)}
        </div>
      `;
    }

    // Add connections info
    if (connections.length > 0) {
      content += `
        <div class="tooltip-connection">
          <strong>Connected to:</strong> ${connections.length} node${connections.length === 1 ? '' : 's'}
        </div>
      `;
    }

    // Add metadata
    content += `
      <div class="tooltip-metadata">
        <div>ID: ${nodeData.nodeId}</div>
        <div>Position: (${Math.round(nodeData.position.x)}, ${Math.round(nodeData.position.y)})</div>
      </div>
    `;

    return content;
  }

  private formatMetrics(metrics: any): string {
    // Format metrics object into readable string
    const metricEntries = Object.entries(metrics);
    if (metricEntries.length === 0) {
      return '<div>No metrics available</div>';
    }

    return metricEntries
      .map(([key, value]) => {
        const formattedKey = key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase());
        return `<div>${formattedKey}: ${value}</div>`;
      })
      .join('');
  }

  public showTooltipForNode(nodeId: string, position: { x: number; y: number }): void {
    const nodeData = this.nodes.find(node => node.nodeId === nodeId);
    if (!nodeData || !this.tooltip) return;

    const content = this.generateTooltipContent(nodeData);
    
    this.tooltip
      .html(content)
      .style('left', `${position.x + 10}px`)
      .style('top', `${position.y - 10}px`)
      .classed('visible', true);
  }

  public hideTooltipForNode(): void {
    this.hideTooltip();
  }

  public updateNodeData(nodes: NodeEventData[]): void {
    this.nodes = nodes;
  }

  public setShowDelay(delay: number): void {
    // Allow customization of show delay
    (this as any).showDelay = Math.max(0, delay);
  }

  public setHideDelay(delay: number): void {
    // Allow customization of hide delay
    (this as any).hideDelay = Math.max(0, delay);
  }

  public destroy(): void {
    // Clear any pending timeouts
    if (this.tooltipTimeout) {
      clearTimeout(this.tooltipTimeout);
      this.tooltipTimeout = null;
    }

    // Remove event listeners
    const nodeSelectors = [
      '.node',
      '.nodeLabel', 
      '[id^="flowchart-"]',
      'g[class*="node"]',
      'rect[class*="node"]',
      'circle[class*="node"]'
    ];

    nodeSelectors.forEach(selector => {
      d3.select(this.svgElement).selectAll(selector)
        .on('mouseenter.tooltip', null)
        .on('mouseleave.tooltip', null)
        .on('mousemove.tooltip', null);
    });

    // Remove tooltip from DOM
    if (this.tooltip) {
      this.tooltip.remove();
      this.tooltip = null;
    }

    // Remove styles (optional - might be used by other instances)
    const styleElement = document.querySelector('#diagram-tooltip-styles');
    if (styleElement) {
      styleElement.remove();
    }
  }
}