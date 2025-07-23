import * as d3 from 'd3';
import type { NodeEventData } from './types';
import type { DiagramTheme, NodeThemeStyle, ComponentStylingRule, DataDrivenStyling } from './ThemeManager';

export interface ComponentAnalysisData {
  complexity: number;
  coupling: number;
  cohesion: number;
  testCoverage: number;
  performance: number;
  security: number;
  maintainability: number;
  technicalDebt: number;
  errorRate: number;
  dependencies: number;
  size: number; // lines of code, file size, etc.
}

export interface StylingContext {
  nodeData: NodeEventData;
  analysisData?: ComponentAnalysisData;
  theme: DiagramTheme;
  globalMetrics?: {
    averageComplexity: number;
    maxComplexity: number;
    averageCoupling: number;
    maxCoupling: number;
  };
}

export interface ConditionalStyle {
  id: string;
  name: string;
  description: string;
  condition: (context: StylingContext) => boolean;
  style: Partial<NodeThemeStyle>;
  priority: number;
  enabled: boolean;
}

export interface MetricVisualization {
  metric: keyof ComponentAnalysisData;
  visualization: 'color' | 'size' | 'border' | 'opacity' | 'pattern' | 'icon';
  scale: {
    type: 'linear' | 'logarithmic' | 'threshold' | 'quantile';
    domain: [number, number];
    range: string[] | number[];
  };
  legend?: {
    title: string;
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
    format: (value: number) => string;
  };
}

export class ComponentStylingEngine {
  private svgElement: SVGElement;
  private theme: DiagramTheme;
  private conditionalStyles: ConditionalStyle[] = [];
  private metricVisualizations: MetricVisualization[] = [];
  private componentData: Map<string, ComponentAnalysisData> = new Map();
  private globalMetrics?: ComponentAnalysisData;

  constructor(svgElement: SVGElement, theme: DiagramTheme) {
    this.svgElement = svgElement;
    this.theme = theme;
    this.initializeDefaultStyles();
  }

  private initializeDefaultStyles(): void {
    // High complexity components
    this.addConditionalStyle({
      id: 'high-complexity',
      name: 'High Complexity',
      description: 'Components with complexity above 80th percentile',
      condition: (context) => {
        const complexity = context.analysisData?.complexity || 0;
        const avgComplexity = context.globalMetrics?.averageComplexity || 0;
        return complexity > avgComplexity * 1.5;
      },
      style: {
        stroke: '#ef4444',
        strokeWidth: 3,
        fill: '#fee2e2'
      },
      priority: 80,
      enabled: true
    });

    // High coupling components
    this.addConditionalStyle({
      id: 'high-coupling',
      name: 'High Coupling',
      description: 'Components with high coupling to other components',
      condition: (context) => {
        const coupling = context.analysisData?.coupling || 0;
        return coupling > 0.8;
      },
      style: {
        stroke: '#f59e0b',
        strokeWidth: 3,
        fill: '#fef3c7'
      },
      priority: 75,
      enabled: true
    });

    // Low test coverage
    this.addConditionalStyle({
      id: 'low-test-coverage',
      name: 'Low Test Coverage',
      description: 'Components with test coverage below 60%',
      condition: (context) => {
        const coverage = context.analysisData?.testCoverage || 0;
        return coverage < 0.6;
      },
      style: {
        stroke: '#dc2626',
        strokeWidth: 2,
        strokeDasharray: '5,5'
      },
      priority: 70,
      enabled: true
    });

    // Performance issues
    this.addConditionalStyle({
      id: 'performance-issues',
      name: 'Performance Issues',
      description: 'Components with performance scores below threshold',
      condition: (context) => {
        const performance = context.analysisData?.performance || 1;
        return performance < 0.5;
      },
      style: {
        stroke: '#7c2d12',
        fill: '#fed7aa',
        strokeWidth: 3
      },
      priority: 85,
      enabled: true
    });

    // Security vulnerabilities
    this.addConditionalStyle({
      id: 'security-vulnerabilities',
      name: 'Security Vulnerabilities',
      description: 'Components with identified security issues',
      condition: (context) => {
        const security = context.analysisData?.security || 1;
        return security < 0.7;
      },
      style: {
        stroke: '#991b1b',
        fill: '#fecaca',
        strokeWidth: 4
      },
      priority: 95,
      enabled: true
    });

    // Well-architected components
    this.addConditionalStyle({
      id: 'well-architected',
      name: 'Well Architected',
      description: 'Components meeting all quality thresholds',
      condition: (context) => {
        const data = context.analysisData;
        if (!data) return false;
        
        return data.complexity < 0.7 &&
               data.coupling < 0.6 &&
               data.testCoverage > 0.8 &&
               data.performance > 0.8 &&
               data.security > 0.9;
      },
      style: {
        stroke: '#059669',
        fill: '#d1fae5',
        strokeWidth: 2
      },
      priority: 60,
      enabled: true
    });
  }

  public setComponentData(nodeId: string, data: ComponentAnalysisData): void {
    this.componentData.set(nodeId, data);
  }

  public setGlobalMetrics(metrics: ComponentAnalysisData): void {
    this.globalMetrics = metrics;
  }

  public addConditionalStyle(style: ConditionalStyle): void {
    // Remove existing style with same ID
    this.conditionalStyles = this.conditionalStyles.filter(s => s.id !== style.id);
    
    // Add new style
    this.conditionalStyles.push(style);
    
    // Sort by priority (higher priority first)
    this.conditionalStyles.sort((a, b) => b.priority - a.priority);
  }

  public removeConditionalStyle(styleId: string): void {
    this.conditionalStyles = this.conditionalStyles.filter(s => s.id !== styleId);
  }

  public enableConditionalStyle(styleId: string, enabled: boolean): void {
    const style = this.conditionalStyles.find(s => s.id === styleId);
    if (style) {
      style.enabled = enabled;
    }
  }

  public addMetricVisualization(visualization: MetricVisualization): void {
    // Remove existing visualization for same metric
    this.metricVisualizations = this.metricVisualizations.filter(
      v => v.metric !== visualization.metric
    );
    
    this.metricVisualizations.push(visualization);
  }

  public removeMetricVisualization(metric: keyof ComponentAnalysisData): void {
    this.metricVisualizations = this.metricVisualizations.filter(
      v => v.metric !== metric
    );
  }

  public applyComponentStyling(nodes: NodeEventData[]): void {
    const svg = d3.select(this.svgElement);
    
    nodes.forEach(nodeData => {
      const nodeElement = svg.select(`[data-node-id="${nodeData.nodeId}"]`);
      if (!nodeElement.empty()) {
        this.styleComponent(nodeElement, nodeData);
      }
    });

    // Apply metric visualizations
    this.applyMetricVisualizations(nodes);
    
    // Create legends
    this.createLegends();
  }

  private styleComponent(
    element: d3.Selection<any, any, any, any>,
    nodeData: NodeEventData
  ): void {
    const analysisData = this.componentData.get(nodeData.nodeId);
    const context: StylingContext = {
      nodeData,
      analysisData,
      theme: this.theme,
      globalMetrics: this.globalMetrics
    };

    // Apply base component type styling
    const baseStyle = this.getBaseComponentStyle(nodeData);
    this.applyNodeStyle(element, baseStyle);

    // Apply conditional styles in priority order
    for (const conditionalStyle of this.conditionalStyles) {
      if (conditionalStyle.enabled && conditionalStyle.condition(context)) {
        this.applyNodeStyle(element, conditionalStyle.style);
        break; // Apply only the highest priority matching style
      }
    }
  }

  private getBaseComponentStyle(nodeData: NodeEventData): Partial<NodeThemeStyle> {
    const componentType = nodeData.componentData.componentType;
    
    switch (componentType) {
      case 'service':
        return this.theme.nodeStyles.service;
      case 'database':
        return this.theme.nodeStyles.database;
      case 'external':
        return this.theme.nodeStyles.external;
      case 'component':
      default:
        return this.theme.nodeStyles.component;
    }
  }

  private applyNodeStyle(
    element: d3.Selection<any, any, any, any>,
    style: Partial<NodeThemeStyle>
  ): void {
    const shape = element.select('rect, circle, ellipse, path');
    const text = element.selectAll('text');

    // Apply shape styles
    if (style.fill) shape.attr('fill', style.fill);
    if (style.stroke) shape.attr('stroke', style.stroke);
    if (style.strokeWidth) shape.attr('stroke-width', style.strokeWidth);
    if (style.strokeDasharray) shape.attr('stroke-dasharray', style.strokeDasharray);
    if (style.opacity !== undefined) shape.attr('opacity', style.opacity);
    if (style.borderRadius) {
      shape.attr('rx', style.borderRadius).attr('ry', style.borderRadius);
    }

    // Apply text styles
    if (style.textColor) text.attr('fill', style.textColor);
    if (style.fontSize) text.attr('font-size', style.fontSize);
    if (style.fontWeight) text.attr('font-weight', style.fontWeight);
  }

  private applyMetricVisualizations(nodes: NodeEventData[]): void {
    this.metricVisualizations.forEach(visualization => {
      this.applyMetricVisualization(visualization, nodes);
    });
  }

  private applyMetricVisualization(
    visualization: MetricVisualization,
    nodes: NodeEventData[]
  ): void {
    const svg = d3.select(this.svgElement);
    
    // Create scale based on visualization configuration
    const scale = this.createScale(visualization, nodes);
    
    nodes.forEach(nodeData => {
      const nodeElement = svg.select(`[data-node-id="${nodeData.nodeId}"]`);
      if (!nodeElement.empty()) {
        const analysisData = this.componentData.get(nodeData.nodeId);
        if (analysisData) {
          const metricValue = analysisData[visualization.metric];
          this.applyVisualizationToNode(
            nodeElement, 
            visualization, 
            scale, 
            metricValue
          );
        }
      }
    });
  }

  private createScale(
    visualization: MetricVisualization,
    nodes: NodeEventData[]
  ): d3.ScaleLinear<any, any> | d3.ScaleThreshold<any, any> | d3.ScaleQuantile<any, any> {
    const values = nodes
      .map(node => this.componentData.get(node.nodeId)?.[visualization.metric])
      .filter(value => value !== undefined) as number[];

    const domain = visualization.scale.domain;
    const range = visualization.scale.range;

    switch (visualization.scale.type) {
      case 'linear':
        return d3.scaleLinear()
          .domain(domain)
          .range(range);
      case 'logarithmic':
        return d3.scaleLog()
          .domain(domain)
          .range(range);
      case 'threshold':
        return d3.scaleThreshold<number, any>()
          .domain(values.slice(0, -1)) // All but last value
          .range(range);
      case 'quantile':
        return d3.scaleQuantile<any>()
          .domain(values)
          .range(range);
      default:
        return d3.scaleLinear()
          .domain(domain)
          .range(range);
    }
  }

  private applyVisualizationToNode(
    element: d3.Selection<any, any, any, any>,
    visualization: MetricVisualization,
    scale: any,
    value: number
  ): void {
    const scaledValue = scale(value);
    
    switch (visualization.visualization) {
      case 'color':
        element.select('rect, circle, ellipse, path')
          .attr('fill', scaledValue);
        break;
      case 'size':
        this.applyNodeSizeVisualization(element, scaledValue);
        break;
      case 'border':
        element.select('rect, circle, ellipse, path')
          .attr('stroke', scaledValue);
        break;
      case 'opacity':
        element.select('rect, circle, ellipse, path')
          .attr('opacity', scaledValue);
        break;
      case 'pattern':
        this.applyPatternVisualization(element, scaledValue);
        break;
      case 'icon':
        this.applyIconVisualization(element, scaledValue);
        break;
    }
  }

  private applyNodeSizeVisualization(
    element: d3.Selection<any, any, any, any>,
    size: number
  ): void {
    const shape = element.select('rect, circle, ellipse');
    
    if (!shape.empty()) {
      const currentWidth = parseFloat(shape.attr('width') || '100');
      const currentHeight = parseFloat(shape.attr('height') || '50');
      
      const newWidth = currentWidth * size;
      const newHeight = currentHeight * size;
      
      shape
        .attr('width', newWidth)
        .attr('height', newHeight);
      
      // Adjust text position if needed
      element.selectAll('text')
        .attr('x', newWidth / 2)
        .attr('y', newHeight / 2);
    }
  }

  private applyPatternVisualization(
    element: d3.Selection<any, any, any, any>,
    pattern: string
  ): void {
    // Create pattern definition if it doesn't exist
    const defs = d3.select(this.svgElement).select('defs');
    const patternId = `pattern-${pattern}`;
    
    if (defs.select(`#${patternId}`).empty()) {
      this.createPatternDefinition(defs, patternId, pattern);
    }
    
    element.select('rect, circle, ellipse, path')
      .attr('fill', `url(#${patternId})`);
  }

  private createPatternDefinition(
    defs: d3.Selection<SVGDefsElement, unknown, null, undefined>,
    patternId: string,
    patternType: string
  ): void {
    const pattern = defs.append('pattern')
      .attr('id', patternId)
      .attr('patternUnits', 'userSpaceOnUse')
      .attr('width', 8)
      .attr('height', 8);

    switch (patternType) {
      case 'diagonal-lines':
        pattern.append('path')
          .attr('d', 'M0,8 L8,0')
          .attr('stroke', '#666')
          .attr('stroke-width', 1);
        break;
      case 'dots':
        pattern.append('circle')
          .attr('cx', 4)
          .attr('cy', 4)
          .attr('r', 1)
          .attr('fill', '#666');
        break;
      case 'cross-hatch':
        pattern.append('path')
          .attr('d', 'M0,0 L8,8 M0,8 L8,0')
          .attr('stroke', '#666')
          .attr('stroke-width', 0.5);
        break;
    }
  }

  private applyIconVisualization(
    element: d3.Selection<any, any, any, any>,
    iconType: string
  ): void {
    // Add icon overlay to the node
    const iconSize = 16;
    const bbox = (element.node() as SVGGraphicsElement).getBBox();
    
    const icon = element.append('g')
      .attr('class', 'metric-icon')
      .attr('transform', `translate(${bbox.x + bbox.width - iconSize - 2}, ${bbox.y + 2})`);

    this.createIcon(icon, iconType, iconSize);
  }

  private createIcon(
    container: d3.Selection<SVGGElement, unknown, null, undefined>,
    iconType: string,
    size: number
  ): void {
    const iconBackground = container.append('circle')
      .attr('cx', size / 2)
      .attr('cy', size / 2)
      .attr('r', size / 2)
      .attr('fill', 'white')
      .attr('stroke', '#666')
      .attr('stroke-width', 1);

    switch (iconType) {
      case 'warning':
        container.append('text')
          .attr('x', size / 2)
          .attr('y', size / 2 + 4)
          .attr('text-anchor', 'middle')
          .attr('font-size', '12px')
          .attr('fill', '#f59e0b')
          .text('⚠');
        break;
      case 'error':
        container.append('text')
          .attr('x', size / 2)
          .attr('y', size / 2 + 4)
          .attr('text-anchor', 'middle')
          .attr('font-size', '12px')
          .attr('fill', '#ef4444')
          .text('❌');
        break;
      case 'success':
        container.append('text')
          .attr('x', size / 2)
          .attr('y', size / 2 + 4)
          .attr('text-anchor', 'middle')
          .attr('font-size', '12px')
          .attr('fill', '#10b981')
          .text('✓');
        break;
    }
  }

  private createLegends(): void {
    this.metricVisualizations.forEach(visualization => {
      if (visualization.legend) {
        this.createLegend(visualization);
      }
    });
  }

  private createLegend(visualization: MetricVisualization): void {
    const svg = d3.select(this.svgElement);
    const legend = visualization.legend!;
    
    // Remove existing legend for this metric
    svg.select(`.legend-${visualization.metric}`).remove();
    
    // Create legend container
    const legendContainer = svg.append('g')
      .attr('class', `legend legend-${visualization.metric}`);

    // Position legend based on configuration
    const position = this.getLegendPosition(legend.position);
    legendContainer.attr('transform', `translate(${position.x}, ${position.y})`);

    // Create legend background
    const legendBg = legendContainer.append('rect')
      .attr('fill', 'rgba(255, 255, 255, 0.9)')
      .attr('stroke', '#ccc')
      .attr('rx', 4);

    // Add legend title
    const title = legendContainer.append('text')
      .attr('x', 10)
      .attr('y', 20)
      .attr('font-size', '12px')
      .attr('font-weight', 'bold')
      .text(legend.title);

    // Add legend items based on visualization type
    this.addLegendItems(legendContainer, visualization);

    // Size the background to fit content
    const bbox = legendContainer.node()!.getBBox();
    legendBg
      .attr('width', bbox.width + 20)
      .attr('height', bbox.height + 10);
  }

  private getLegendPosition(position: string): { x: number; y: number } {
    const svgRect = this.svgElement.getBoundingClientRect();
    const margin = 20;

    switch (position) {
      case 'top-left':
        return { x: margin, y: margin };
      case 'top-right':
        return { x: svgRect.width - 200, y: margin };
      case 'bottom-left':
        return { x: margin, y: svgRect.height - 150 };
      case 'bottom-right':
        return { x: svgRect.width - 200, y: svgRect.height - 150 };
      default:
        return { x: margin, y: margin };
    }
  }

  private addLegendItems(
    container: d3.Selection<SVGGElement, unknown, null, undefined>,
    visualization: MetricVisualization
  ): void {
    const scale = visualization.scale;
    const range = scale.range as string[];
    const domain = scale.domain;
    
    range.forEach((value, index) => {
      const y = 40 + index * 25;
      
      // Add color/pattern sample
      container.append('rect')
        .attr('x', 10)
        .attr('y', y - 8)
        .attr('width', 15)
        .attr('height', 15)
        .attr('fill', value)
        .attr('stroke', '#666');
      
      // Add label
      const domainValue = domain[0] + (domain[1] - domain[0]) * (index / (range.length - 1));
      const formattedValue = visualization.legend?.format 
        ? visualization.legend.format(domainValue)
        : domainValue.toFixed(2);
      
      container.append('text')
        .attr('x', 35)
        .attr('y', y + 4)
        .attr('font-size', '11px')
        .text(formattedValue);
    });
  }

  public getConditionalStyles(): ConditionalStyle[] {
    return [...this.conditionalStyles];
  }

  public getMetricVisualizations(): MetricVisualization[] {
    return [...this.metricVisualizations];
  }

  public updateTheme(theme: DiagramTheme): void {
    this.theme = theme;
  }

  public dispose(): void {
    // Clean up legends and event listeners
    const svg = d3.select(this.svgElement);
    svg.selectAll('.legend').remove();
    svg.selectAll('.metric-icon').remove();
  }
}

export default ComponentStylingEngine;