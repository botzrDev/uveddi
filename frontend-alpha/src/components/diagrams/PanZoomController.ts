// Pan & Zoom Controller for Interactive Diagrams - UV-89 Phase 1
// Provides smooth navigation controls with D3.js zoom behavior

import * as d3 from 'd3';

export class PanZoomController {
  private zoom!: d3.ZoomBehavior<SVGElement, unknown>;
  private svgElement: SVGElement;
  private containerGroup: SVGGElement;
  private currentTransform: d3.ZoomTransform;

  constructor(svgElement: SVGElement, containerGroup: SVGGElement) {
    this.svgElement = svgElement;
    this.containerGroup = containerGroup;
    this.currentTransform = d3.zoomIdentity;
    this.initializePanZoom();
  }

  private initializePanZoom(): void {
    // Configure zoom behavior
    this.zoom = d3.zoom<SVGElement, unknown>()
      .scaleExtent([0.1, 10]) // Allow 10% to 1000% zoom
      .on('zoom', this.handleZoom.bind(this))
      .on('start', this.handleZoomStart.bind(this))
      .on('end', this.handleZoomEnd.bind(this));

    // Apply zoom behavior to SVG
    d3.select(this.svgElement)
      .call(this.zoom!)
      .on('dblclick.zoom', null); // Disable default double-click zoom

    // Set initial transform if needed
    this.applyTransform(d3.zoomIdentity);

    // Add wheel event listener for better scroll handling
    this.svgElement.addEventListener('wheel', this.handleWheel.bind(this), { passive: false });
  }

  private handleZoom(event: d3.D3ZoomEvent<SVGElement, unknown>): void {
    const { transform } = event;
    this.currentTransform = transform;
    
    // Apply transform to container group
    d3.select(this.containerGroup)
      .attr('transform', transform.toString());

    // Emit zoom event for other components
    this.svgElement.dispatchEvent(new CustomEvent('diagram-zoom', {
      detail: {
        scale: transform.k,
        translate: [transform.x, transform.y]
      }
    }));
  }

  private handleZoomStart(_event: d3.D3ZoomEvent<SVGElement, unknown>): void {
    // Add visual feedback during zoom/pan
    d3.select(this.svgElement).style('cursor', 'grabbing');
  }

  private handleZoomEnd(_event: d3.D3ZoomEvent<SVGElement, unknown>): void {
    // Remove visual feedback
    d3.select(this.svgElement).style('cursor', 'default');
  }

  private handleWheel(event: WheelEvent): void {
    // Prevent default scrolling behavior when zooming
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
    }
  }

  private applyTransform(transform: d3.ZoomTransform): void {
    d3.select(this.svgElement)
      .transition()
      .duration(300)
      .call(this.zoom.transform, transform);
  }

  public zoomIn(): void {
    const newScale = Math.min(this.currentTransform.k * 1.5, 10);
    const transform = this.currentTransform.scale(newScale / this.currentTransform.k);
    this.applyTransform(transform);
  }

  public zoomOut(): void {
    const newScale = Math.max(this.currentTransform.k / 1.5, 0.1);
    const transform = this.currentTransform.scale(newScale / this.currentTransform.k);
    this.applyTransform(transform);
  }

  public zoomToScale(scale: number): void {
    const clampedScale = Math.max(0.1, Math.min(10, scale));
    const transform = this.currentTransform.scale(clampedScale / this.currentTransform.k);
    this.applyTransform(transform);
  }

  public zoomToFit(): void {
    try {
      // Get the bounding box of all content
      const bounds = this.containerGroup.getBBox();
      const parent = this.svgElement.getBoundingClientRect();
      
      if (bounds.width === 0 || bounds.height === 0) {
        console.warn('Cannot zoom to fit: content has no dimensions');
        return;
      }

      const fullWidth = parent.width;
      const fullHeight = parent.height;
      const width = bounds.width;
      const height = bounds.height;
      
      // Calculate center points
      const midX = bounds.x + width / 2;
      const midY = bounds.y + height / 2;
      
      // Calculate scale to fit with padding
      const scale = Math.min(fullWidth / width, fullHeight / height) * 0.9;
      
      // Calculate translation to center
      const translate: [number, number] = [
        fullWidth / 2 - scale * midX,
        fullHeight / 2 - scale * midY
      ];

      // Apply transform with animation
      const transform = d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale);
      this.applyTransform(transform);
      
    } catch (error) {
      console.error('Error in zoomToFit:', error);
      // Fallback to reset
      this.resetZoom();
    }
  }

  public resetZoom(): void {
    this.applyTransform(d3.zoomIdentity);
  }

  public panTo(x: number, y: number): void {
    const transform = this.currentTransform.translate(
      x - this.currentTransform.x,
      y - this.currentTransform.y
    );
    this.applyTransform(transform);
  }

  public focusOnNode(nodeId: string): void {
    try {
      // Find the node element
      const nodeElement = this.svgElement.querySelector(`#${nodeId}`) as SVGGraphicsElement;
      if (!nodeElement) {
        console.warn(`Node with ID ${nodeId} not found`);
        return;
      }

      // Get node bounding box
      const nodeBounds = nodeElement.getBBox();
      const svgBounds = this.svgElement.getBoundingClientRect();
      
      // Calculate center of node
      const nodeCenterX = nodeBounds.x + nodeBounds.width / 2;
      const nodeCenterY = nodeBounds.y + nodeBounds.height / 2;
      
      // Calculate transform to center the node
      const scale = Math.min(2, this.currentTransform.k); // Limit zoom level
      const translate: [number, number] = [
        svgBounds.width / 2 - scale * nodeCenterX,
        svgBounds.height / 2 - scale * nodeCenterY
      ];

      const transform = d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale);
      this.applyTransform(transform);
      
    } catch (error) {
      console.error('Error focusing on node:', error);
    }
  }

  public getCurrentTransform(): d3.ZoomTransform {
    return this.currentTransform;
  }

  public getZoomLevel(): number {
    return this.currentTransform.k;
  }

  public getPanPosition(): { x: number; y: number } {
    return { x: this.currentTransform.x, y: this.currentTransform.y };
  }

  public enablePan(enabled: boolean): void {
    if (enabled) {
      d3.select(this.svgElement).call(this.zoom);
    } else {
      d3.select(this.svgElement).on('.zoom', null);
    }
  }

  public setScaleExtent(min: number, max: number): void {
    this.zoom?.scaleExtent([Math.max(0.01, min), Math.min(100, max)]);
  }

  public enableDoubleClickZoom(enabled: boolean): void {
    if (enabled) {
      d3.select(this.svgElement).on('dblclick.zoom', () => {
        // Custom double-click zoom behavior
        this.zoomIn();
      });
    } else {
      d3.select(this.svgElement).on('dblclick.zoom', null);
    }
  }

  public constrainToViewport(enabled: boolean): void {
    if (enabled) {
      // Add extent constraint to keep content within viewport
      const svgBounds = this.svgElement.getBoundingClientRect();
      this.zoom?.extent([[0, 0], [svgBounds.width, svgBounds.height]]);
    } else {
      // Remove extent constraint
      this.zoom?.extent([[-Infinity, -Infinity], [Infinity, Infinity]]);
    }
  }

  public animateToTransform(transform: d3.ZoomTransform, duration: number = 750): void {
    d3.select(this.svgElement)
      .transition()
      .duration(duration)
      .ease(d3.easeQuadInOut)
      .call(this.zoom.transform, transform);
  }

  public destroy(): void {
    // Remove event listeners
    d3.select(this.svgElement)
      .on('.zoom', null)
      .on('dblclick.zoom', null);
    
    this.svgElement.removeEventListener('wheel', this.handleWheel);
    
    // Reset cursor
    d3.select(this.svgElement).style('cursor', null);
  }
}