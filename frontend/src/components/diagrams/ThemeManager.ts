import * as d3 from 'd3';

export interface DiagramTheme {
  id: string;
  name: string;
  description?: string;
  colors: {
    primary: string;
    secondary: string;
    accent: string;
    background: string;
    surface: string;
    text: string;
    textSecondary: string;
    success: string;
    warning: string;
    error: string;
    info: string;
  };
  nodeStyles: {
    default: NodeThemeStyle;
    component: NodeThemeStyle;
    service: NodeThemeStyle;
    database: NodeThemeStyle;
    external: NodeThemeStyle;
    error: NodeThemeStyle;
  };
  edgeStyles: {
    default: EdgeThemeStyle;
    dependency: EdgeThemeStyle;
    dataFlow: EdgeThemeStyle;
    control: EdgeThemeStyle;
    error: EdgeThemeStyle;
  };
  layout: {
    spacing: number;
    padding: number;
    borderRadius: number;
    strokeWidth: number;
    fontSize: string;
    fontFamily: string;
  };
  effects: {
    shadows: boolean;
    gradients: boolean;
    animations: boolean;
    glow: boolean;
  };
}

export interface NodeThemeStyle {
  fill: string;
  stroke: string;
  strokeWidth: number;
  opacity: number;
  textColor: string;
  fontSize: string;
  fontWeight: string;
  borderRadius: number;
  shadow?: string;
  gradient?: {
    type: 'linear' | 'radial';
    stops: Array<{ offset: string; color: string; opacity?: number }>;
  };
  hover?: Partial<NodeThemeStyle>;
  selected?: Partial<NodeThemeStyle>;
  active?: Partial<NodeThemeStyle>;
}

export interface EdgeThemeStyle {
  stroke: string;
  strokeWidth: number;
  strokeDasharray?: string;
  opacity: number;
  markerEnd?: string;
  markerStart?: string;
  textColor: string;
  fontSize: string;
  hover?: Partial<EdgeThemeStyle>;
  selected?: Partial<EdgeThemeStyle>;
  active?: Partial<EdgeThemeStyle>;
}

export interface ComponentStylingRule {
  selector: string;
  condition?: (nodeData: any) => boolean;
  style: Partial<NodeThemeStyle>;
  priority: number;
}

export interface DataDrivenStyling {
  metric: string;
  thresholds: Array<{
    min?: number;
    max?: number;
    style: Partial<NodeThemeStyle>;
  }>;
  interpolation?: 'discrete' | 'linear' | 'logarithmic';
}

export class ThemeManager {
  private currentTheme: DiagramTheme;
  private customRules: ComponentStylingRule[] = [];
  private dataDrivenRules: DataDrivenStyling[] = [];
  private svgElement: SVGElement;
  private defs: d3.Selection<SVGDefsElement, unknown, null, undefined>;

  constructor(svgElement: SVGElement, initialTheme?: DiagramTheme) {
    this.svgElement = svgElement;
    this.currentTheme = initialTheme || this.getDefaultTheme();
    this.initializeSVGDefinitions();
    this.applyTheme();
  }

  private initializeSVGDefinitions(): void {
    // Ensure defs element exists
    let defsElement = this.svgElement.querySelector('defs');
    if (!defsElement) {
      defsElement = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
      this.svgElement.insertBefore(defsElement, this.svgElement.firstChild);
    }
    
    this.defs = d3.select(defsElement);
  }

  private getDefaultTheme(): DiagramTheme {
    return {
      id: 'default',
      name: 'Default',
      description: 'Clean, professional theme for architectural diagrams',
      colors: {
        primary: '#3b82f6',
        secondary: '#6366f1',
        accent: '#8b5cf6',
        background: '#ffffff',
        surface: '#f8fafc',
        text: '#1f2937',
        textSecondary: '#6b7280',
        success: '#10b981',
        warning: '#f59e0b',
        error: '#ef4444',
        info: '#06b6d4'
      },
      nodeStyles: {
        default: {
          fill: '#f8fafc',
          stroke: '#e2e8f0',
          strokeWidth: 2,
          opacity: 1,
          textColor: '#1f2937',
          fontSize: '14px',
          fontWeight: '500',
          borderRadius: 8,
          shadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
          hover: {
            fill: '#f1f5f9',
            stroke: '#3b82f6',
            strokeWidth: 3
          },
          selected: {
            fill: '#dbeafe',
            stroke: '#3b82f6',
            strokeWidth: 3
          }
        },
        component: {
          fill: '#dbeafe',
          stroke: '#3b82f6',
          strokeWidth: 2,
          opacity: 1,
          textColor: '#1e40af',
          fontSize: '14px',
          fontWeight: '600',
          borderRadius: 8
        },
        service: {
          fill: '#dcfce7',
          stroke: '#16a34a',
          strokeWidth: 2,
          opacity: 1,
          textColor: '#166534',
          fontSize: '14px',
          fontWeight: '600',
          borderRadius: 8
        },
        database: {
          fill: '#fef3c7',
          stroke: '#d97706',
          strokeWidth: 2,
          opacity: 1,
          textColor: '#92400e',
          fontSize: '14px',
          fontWeight: '600',
          borderRadius: 8
        },
        external: {
          fill: '#f3e8ff',
          stroke: '#9333ea',
          strokeWidth: 2,
          opacity: 1,
          textColor: '#7c3aed',
          fontSize: '14px',
          fontWeight: '600',
          borderRadius: 8
        },
        error: {
          fill: '#fee2e2',
          stroke: '#dc2626',
          strokeWidth: 2,
          opacity: 1,
          textColor: '#991b1b',
          fontSize: '14px',
          fontWeight: '600',
          borderRadius: 8
        }
      },
      edgeStyles: {
        default: {
          stroke: '#6b7280',
          strokeWidth: 2,
          opacity: 0.8,
          markerEnd: 'url(#arrowhead)',
          textColor: '#374151',
          fontSize: '12px'
        },
        dependency: {
          stroke: '#3b82f6',
          strokeWidth: 2,
          opacity: 0.8,
          markerEnd: 'url(#arrowhead-blue)',
          textColor: '#1e40af',
          fontSize: '12px'
        },
        dataFlow: {
          stroke: '#10b981',
          strokeWidth: 3,
          opacity: 0.9,
          markerEnd: 'url(#arrowhead-green)',
          textColor: '#059669',
          fontSize: '12px'
        },
        control: {
          stroke: '#f59e0b',
          strokeWidth: 2,
          strokeDasharray: '5,5',
          opacity: 0.8,
          markerEnd: 'url(#arrowhead-yellow)',
          textColor: '#d97706',
          fontSize: '12px'
        },
        error: {
          stroke: '#ef4444',
          strokeWidth: 3,
          opacity: 0.9,
          markerEnd: 'url(#arrowhead-red)',
          textColor: '#dc2626',
          fontSize: '12px'
        }
      },
      layout: {
        spacing: 50,
        padding: 20,
        borderRadius: 8,
        strokeWidth: 2,
        fontSize: '14px',
        fontFamily: 'system-ui, -apple-system, sans-serif'
      },
      effects: {
        shadows: true,
        gradients: false,
        animations: true,
        glow: false
      }
    };
  }

  public setTheme(theme: DiagramTheme): void {
    this.currentTheme = theme;
    this.applyTheme();
  }

  public getCurrentTheme(): DiagramTheme {
    return { ...this.currentTheme };
  }

  public addCustomRule(rule: ComponentStylingRule): void {
    this.customRules.push(rule);
    this.customRules.sort((a, b) => b.priority - a.priority);
    this.applyCustomRules();
  }

  public removeCustomRule(selector: string): void {
    this.customRules = this.customRules.filter(rule => rule.selector !== selector);
    this.applyCustomRules();
  }

  public addDataDrivenRule(rule: DataDrivenStyling): void {
    this.dataDrivenRules.push(rule);
    this.applyDataDrivenStyling();
  }

  public removeDataDrivenRule(metric: string): void {
    this.dataDrivenRules = this.dataDrivenRules.filter(rule => rule.metric !== metric);
    this.applyDataDrivenStyling();
  }

  private applyTheme(): void {
    this.createSVGDefinitions();
    this.applyGlobalStyles();
    this.applyNodeStyles();
    this.applyEdgeStyles();
    this.applyShadowsAndEffects();
  }

  private createSVGDefinitions(): void {
    // Clear existing definitions
    this.defs.selectAll('*').remove();

    // Create gradients if enabled
    if (this.currentTheme.effects.gradients) {
      this.createGradientDefinitions();
    }

    // Create arrow markers
    this.createArrowMarkers();

    // Create filters for effects
    if (this.currentTheme.effects.shadows) {
      this.createShadowFilter();
    }

    if (this.currentTheme.effects.glow) {
      this.createGlowFilter();
    }
  }

  private createGradientDefinitions(): void {
    const gradients = [
      {
        id: 'node-gradient-primary',
        colors: [this.currentTheme.colors.primary, this.currentTheme.colors.secondary]
      },
      {
        id: 'node-gradient-accent',
        colors: [this.currentTheme.colors.accent, this.currentTheme.colors.primary]
      },
      {
        id: 'node-gradient-success',
        colors: [this.currentTheme.colors.success, '#34d399']
      },
      {
        id: 'node-gradient-warning',
        colors: [this.currentTheme.colors.warning, '#fbbf24']
      },
      {
        id: 'node-gradient-error',
        colors: [this.currentTheme.colors.error, '#f87171']
      }
    ];

    gradients.forEach(gradient => {
      const linearGradient = this.defs.append('linearGradient')
        .attr('id', gradient.id)
        .attr('x1', '0%')
        .attr('y1', '0%')
        .attr('x2', '100%')
        .attr('y2', '100%');

      linearGradient.append('stop')
        .attr('offset', '0%')
        .attr('stop-color', gradient.colors[0])
        .attr('stop-opacity', 0.9);

      linearGradient.append('stop')
        .attr('offset', '100%')
        .attr('stop-color', gradient.colors[1])
        .attr('stop-opacity', 0.7);
    });
  }

  private createArrowMarkers(): void {
    const markers = [
      { id: 'arrowhead', color: this.currentTheme.edgeStyles.default.stroke },
      { id: 'arrowhead-blue', color: this.currentTheme.colors.primary },
      { id: 'arrowhead-green', color: this.currentTheme.colors.success },
      { id: 'arrowhead-yellow', color: this.currentTheme.colors.warning },
      { id: 'arrowhead-red', color: this.currentTheme.colors.error }
    ];

    markers.forEach(marker => {
      const markerElement = this.defs.append('marker')
        .attr('id', marker.id)
        .attr('viewBox', '0 0 10 10')
        .attr('refX', 8)
        .attr('refY', 3)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto');

      markerElement.append('path')
        .attr('d', 'M0,0 L0,6 L9,3 z')
        .attr('fill', marker.color);
    });
  }

  private createShadowFilter(): void {
    const filter = this.defs.append('filter')
      .attr('id', 'node-shadow')
      .attr('x', '-50%')
      .attr('y', '-50%')
      .attr('width', '200%')
      .attr('height', '200%');

    filter.append('feDropShadow')
      .attr('dx', 2)
      .attr('dy', 2)
      .attr('stdDeviation', 3)
      .attr('flood-opacity', 0.3);
  }

  private createGlowFilter(): void {
    const filter = this.defs.append('filter')
      .attr('id', 'node-glow')
      .attr('x', '-50%')
      .attr('y', '-50%')
      .attr('width', '200%')
      .attr('height', '200%');

    filter.append('feGaussianBlur')
      .attr('stdDeviation', 3)
      .attr('result', 'coloredBlur');

    const feMerge = filter.append('feMerge');
    feMerge.append('feMergeNode').attr('in', 'coloredBlur');
    feMerge.append('feMergeNode').attr('in', 'SourceGraphic');
  }

  private applyGlobalStyles(): void {
    // Apply global font styles
    d3.select(this.svgElement)
      .style('font-family', this.currentTheme.layout.fontFamily)
      .style('font-size', this.currentTheme.layout.fontSize);

    // Apply background
    const bgRect = d3.select(this.svgElement).select('.background');
    if (!bgRect.empty()) {
      bgRect.attr('fill', this.currentTheme.colors.background);
    }
  }

  private applyNodeStyles(): void {
    const svg = d3.select(this.svgElement);
    
    Object.entries(this.currentTheme.nodeStyles).forEach(([type, style]) => {
      const selector = type === 'default' ? '.node' : `.node.${type}`;
      const nodes = svg.selectAll(selector);

      this.applyNodeStyle(nodes, style);
    });
  }

  private applyNodeStyle(
    selection: d3.Selection<any, any, any, any>, 
    style: NodeThemeStyle
  ): void {
    // Apply basic styles
    selection
      .select('rect, circle, ellipse, path')
      .attr('fill', style.gradient ? `url(#${this.getGradientId(style)})` : style.fill)
      .attr('stroke', style.stroke)
      .attr('stroke-width', style.strokeWidth)
      .attr('opacity', style.opacity)
      .attr('rx', style.borderRadius)
      .attr('ry', style.borderRadius);

    // Apply text styles
    selection
      .selectAll('text')
      .attr('fill', style.textColor)
      .attr('font-size', style.fontSize)
      .attr('font-weight', style.fontWeight);

    // Apply effects
    if (this.currentTheme.effects.shadows && style.shadow) {
      selection.style('filter', 'url(#node-shadow)');
    }

    if (this.currentTheme.effects.glow) {
      selection.style('filter', 'url(#node-glow)');
    }

    // Set up hover and selection states
    this.setupInteractionStates(selection, style);
  }

  private setupInteractionStates(
    selection: d3.Selection<any, any, any, any>,
    style: NodeThemeStyle
  ): void {
    if (style.hover) {
      selection.on('mouseenter', function() {
        const element = d3.select(this);
        const shape = element.select('rect, circle, ellipse, path');
        const text = element.selectAll('text');

        shape
          .transition()
          .duration(200)
          .attr('fill', style.hover!.fill || style.fill)
          .attr('stroke', style.hover!.stroke || style.stroke)
          .attr('stroke-width', style.hover!.strokeWidth || style.strokeWidth);

        text
          .transition()
          .duration(200)
          .attr('fill', style.hover!.textColor || style.textColor);
      });

      selection.on('mouseleave', function() {
        const element = d3.select(this);
        const shape = element.select('rect, circle, ellipse, path');
        const text = element.selectAll('text');

        shape
          .transition()
          .duration(200)
          .attr('fill', style.fill)
          .attr('stroke', style.stroke)
          .attr('stroke-width', style.strokeWidth);

        text
          .transition()
          .duration(200)
          .attr('fill', style.textColor);
      });
    }
  }

  private applyEdgeStyles(): void {
    const svg = d3.select(this.svgElement);
    
    Object.entries(this.currentTheme.edgeStyles).forEach(([type, style]) => {
      const selector = type === 'default' ? '.edge' : `.edge.${type}`;
      const edges = svg.selectAll(selector);

      this.applyEdgeStyle(edges, style);
    });
  }

  private applyEdgeStyle(
    selection: d3.Selection<any, any, any, any>,
    style: EdgeThemeStyle
  ): void {
    // Apply path styles
    selection
      .select('path')
      .attr('stroke', style.stroke)
      .attr('stroke-width', style.strokeWidth)
      .attr('stroke-dasharray', style.strokeDasharray || 'none')
      .attr('opacity', style.opacity)
      .attr('marker-end', style.markerEnd)
      .attr('marker-start', style.markerStart);

    // Apply text styles
    selection
      .selectAll('text')
      .attr('fill', style.textColor)
      .attr('font-size', style.fontSize);
  }

  private applyShadowsAndEffects(): void {
    if (!this.currentTheme.effects.shadows) return;

    const svg = d3.select(this.svgElement);
    svg.selectAll('.node')
      .style('filter', 'url(#node-shadow)');
  }

  private applyCustomRules(): void {
    const svg = d3.select(this.svgElement);

    this.customRules.forEach(rule => {
      const elements = svg.selectAll(rule.selector);
      
      elements.each((d: any, i: number, nodes: any[]) => {
        if (!rule.condition || rule.condition(d)) {
          const element = d3.select(nodes[i]);
          this.applyStyleObject(element, rule.style);
        }
      });
    });
  }

  private applyDataDrivenStyling(): void {
    const svg = d3.select(this.svgElement);

    this.dataDrivenRules.forEach(rule => {
      svg.selectAll('.node').each((d: any, i: number, nodes: any[]) => {
        const metricValue = this.getMetricValue(d, rule.metric);
        if (metricValue !== undefined) {
          const style = this.calculateStyleFromThreshold(metricValue, rule);
          if (style) {
            const element = d3.select(nodes[i]);
            this.applyStyleObject(element, style);
          }
        }
      });
    });
  }

  private getMetricValue(nodeData: any, metric: string): number | undefined {
    // Navigate through object path to get metric value
    const keys = metric.split('.');
    let value = nodeData;
    
    for (const key of keys) {
      if (value && typeof value === 'object' && key in value) {
        value = value[key];
      } else {
        return undefined;
      }
    }
    
    return typeof value === 'number' ? value : undefined;
  }

  private calculateStyleFromThreshold(
    value: number, 
    rule: DataDrivenStyling
  ): Partial<NodeThemeStyle> | null {
    for (const threshold of rule.thresholds) {
      const minMet = threshold.min === undefined || value >= threshold.min;
      const maxMet = threshold.max === undefined || value <= threshold.max;
      
      if (minMet && maxMet) {
        return threshold.style;
      }
    }
    
    return null;
  }

  private applyStyleObject(
    element: d3.Selection<any, any, any, any>,
    style: Partial<NodeThemeStyle>
  ): void {
    const shape = element.select('rect, circle, ellipse, path');
    const text = element.selectAll('text');

    if (style.fill) shape.attr('fill', style.fill);
    if (style.stroke) shape.attr('stroke', style.stroke);
    if (style.strokeWidth) shape.attr('stroke-width', style.strokeWidth);
    if (style.opacity) shape.attr('opacity', style.opacity);
    if (style.borderRadius) {
      shape.attr('rx', style.borderRadius).attr('ry', style.borderRadius);
    }

    if (style.textColor) text.attr('fill', style.textColor);
    if (style.fontSize) text.attr('font-size', style.fontSize);
    if (style.fontWeight) text.attr('font-weight', style.fontWeight);
  }

  private getGradientId(style: NodeThemeStyle): string {
    // Return appropriate gradient ID based on style
    return 'node-gradient-primary'; // Simplified for now
  }

  public exportTheme(): DiagramTheme {
    return { ...this.currentTheme };
  }

  public importTheme(theme: DiagramTheme): void {
    this.setTheme(theme);
  }

  public createCustomTheme(baseTheme: DiagramTheme, modifications: Partial<DiagramTheme>): DiagramTheme {
    return {
      ...baseTheme,
      ...modifications,
      colors: { ...baseTheme.colors, ...modifications.colors },
      nodeStyles: { ...baseTheme.nodeStyles, ...modifications.nodeStyles },
      edgeStyles: { ...baseTheme.edgeStyles, ...modifications.edgeStyles },
      layout: { ...baseTheme.layout, ...modifications.layout },
      effects: { ...baseTheme.effects, ...modifications.effects }
    };
  }

  public dispose(): void {
    // Clean up event listeners and resources
    const svg = d3.select(this.svgElement);
    svg.selectAll('.node').on('mouseenter mouseleave', null);
  }
}

export default ThemeManager;