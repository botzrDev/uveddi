import * as d3 from 'd3';
import type { DiagramTheme } from './ThemeManager';
import type { NodeEventData } from './types';

export interface ExportOptions {
  format: 'html' | 'svg' | 'png' | 'pdf' | 'plantuml' | 'graphviz' | 'json';
  includeInteractivity?: boolean;
  includeStyles?: boolean;
  includeAnimations?: boolean;
  quality?: number; // For image formats
  width?: number;
  height?: number;
  filename?: string;
  backgroundColor?: string;
  preserveViewBox?: boolean;
}

export interface PlantUMLOptions {
  diagramType: 'component' | 'class' | 'sequence' | 'activity' | 'deployment';
  includeColors?: boolean;
  includeNotes?: boolean;
  theme?: 'default' | 'plain' | 'sketchy' | 'dark';
}

export interface GraphvizOptions {
  engine: 'dot' | 'neato' | 'fdp' | 'sfdp' | 'circo' | 'twopi';
  rankdir?: 'TB' | 'BT' | 'LR' | 'RL';
  splines?: 'polyline' | 'curved' | 'ortho' | 'none';
  includeColors?: boolean;
  nodeShape?: 'box' | 'circle' | 'ellipse' | 'diamond';
}

export interface HTMLExportOptions extends ExportOptions {
  includeInteractivity: true;
  standalone?: boolean;
  includeLibraries?: boolean;
  customCSS?: string;
  customJS?: string;
  embedAssets?: boolean;
}

export class ExportManager {
  private svgElement: SVGElement;
  private nodeData: NodeEventData[];
  private theme?: DiagramTheme;
  private mermaidCode?: string;

  constructor(
    svgElement: SVGElement,
    nodeData: NodeEventData[],
    theme?: DiagramTheme,
    mermaidCode?: string
  ) {
    this.svgElement = svgElement;
    this.nodeData = nodeData;
    this.theme = theme;
    this.mermaidCode = mermaidCode;
  }

  public async exportDiagram(options: ExportOptions): Promise<Blob | string> {
    switch (options.format) {
      case 'html':
        return this.exportToHTML(options as HTMLExportOptions);
      case 'svg':
        return this.exportToSVG(options);
      case 'png':
        return this.exportToPNG(options);
      case 'pdf':
        return this.exportToPDF(options);
      case 'plantuml':
        return this.exportToPlantUML(options);
      case 'graphviz':
        return this.exportToGraphviz(options);
      case 'json':
        return this.exportToJSON(options);
      default:
        throw new Error(`Unsupported export format: ${options.format}`);
    }
  }

  private async exportToHTML(options: HTMLExportOptions): Promise<string> {
    const svgClone = this.cloneSVGWithStyles();
    const svgString = new XMLSerializer().serializeToString(svgClone);
    
    const html = this.generateHTMLTemplate({
      svgContent: svgString,
      nodeData: this.nodeData,
      theme: this.theme,
      options
    });

    return html;
  }

  private generateHTMLTemplate({
    svgContent,
    nodeData,
    theme,
    options
  }: {
    svgContent: string;
    nodeData: NodeEventData[];
    theme?: DiagramTheme;
    options: HTMLExportOptions;
  }): string {
    const d3Scripts = options.includeLibraries ? `
      <script src="https://d3js.org/d3.v7.min.js"></script>
      <script src="https://unpkg.com/gsap@3.12.2/dist/gsap.min.js"></script>
    ` : '';

    const interactivityScript = options.includeInteractivity ? this.generateInteractivityScript(nodeData) : '';
    const customCSS = options.customCSS || '';
    const customJS = options.customJS || '';
    const themeCSS = theme ? this.generateThemeCSS(theme) : '';

    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Interactive Diagram Export</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            font-family: system-ui, -apple-system, sans-serif;
            background-color: ${options.backgroundColor || '#ffffff'};
        }
        
        .diagram-container {
            width: 100%;
            height: 100vh;
            position: relative;
            border: 1px solid #e2e8f0;
            border-radius: 8px;
            overflow: hidden;
            background: white;
        }
        
        svg {
            width: 100%;
            height: 100%;
            display: block;
        }
        
        .node {
            cursor: pointer;
            transition: all 0.2s ease;
        }
        
        .node:hover {
            filter: brightness(1.1);
        }
        
        .tooltip {
            position: absolute;
            background: rgba(0, 0, 0, 0.8);
            color: white;
            padding: 8px 12px;
            border-radius: 4px;
            font-size: 12px;
            pointer-events: none;
            z-index: 1000;
            opacity: 0;
            transition: opacity 0.2s ease;
        }
        
        .controls {
            position: absolute;
            top: 10px;
            right: 10px;
            background: rgba(255, 255, 255, 0.9);
            padding: 10px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }
        
        .controls button {
            margin: 0 5px;
            padding: 5px 10px;
            border: 1px solid #ccc;
            background: white;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
        }
        
        .controls button:hover {
            background: #f0f0f0;
        }
        
        ${themeCSS}
        ${customCSS}
    </style>
</head>
<body>
    <div class="diagram-container">
        ${svgContent}
        ${options.includeInteractivity ? `
        <div class="tooltip" id="tooltip"></div>
        <div class="controls">
            <button onclick="zoomIn()">+</button>
            <button onclick="zoomOut()">-</button>
            <button onclick="resetZoom()">Reset</button>
            <button onclick="downloadSVG()">Download SVG</button>
        </div>
        ` : ''}
    </div>

    ${d3Scripts}
    
    <script>
        const nodeData = ${JSON.stringify(nodeData)};
        const theme = ${JSON.stringify(theme)};
        
        ${interactivityScript}
        ${customJS}
    </script>
</body>
</html>`;
  }

  private generateInteractivityScript(nodeData: NodeEventData[]): string {
    return `
        // Interactive functionality for exported diagram
        let currentZoom = 1;
        const svg = document.querySelector('svg');
        const container = svg.querySelector('g');
        
        // Zoom functionality
        function zoomIn() {
            currentZoom *= 1.2;
            updateZoom();
        }
        
        function zoomOut() {
            currentZoom /= 1.2;
            updateZoom();
        }
        
        function resetZoom() {
            currentZoom = 1;
            updateZoom();
        }
        
        function updateZoom() {
            if (container) {
                container.style.transform = \`scale(\${currentZoom})\`;
            }
        }
        
        // Pan functionality
        let isPanning = false;
        let startX, startY, translateX = 0, translateY = 0;
        
        svg.addEventListener('mousedown', (e) => {
            isPanning = true;
            startX = e.clientX - translateX;
            startY = e.clientY - translateY;
            svg.style.cursor = 'grabbing';
        });
        
        svg.addEventListener('mousemove', (e) => {
            if (!isPanning) return;
            translateX = e.clientX - startX;
            translateY = e.clientY - startY;
            updateTransform();
        });
        
        svg.addEventListener('mouseup', () => {
            isPanning = false;
            svg.style.cursor = 'grab';
        });
        
        function updateTransform() {
            if (container) {
                container.style.transform = \`translate(\${translateX}px, \${translateY}px) scale(\${currentZoom})\`;
            }
        }
        
        // Tooltip functionality
        const tooltip = document.getElementById('tooltip');
        
        document.querySelectorAll('.node').forEach((node, index) => {
            node.addEventListener('mouseenter', (e) => {
                const data = nodeData[index] || {};
                tooltip.innerHTML = \`
                    <strong>\${data.componentData?.name || 'Component'}</strong><br>
                    Type: \${data.componentData?.componentType || 'N/A'}<br>
                    Path: \${data.componentData?.filePath || 'N/A'}
                \`;
                tooltip.style.left = e.pageX + 10 + 'px';
                tooltip.style.top = e.pageY + 10 + 'px';
                tooltip.style.opacity = '1';
            });
            
            node.addEventListener('mouseleave', () => {
                tooltip.style.opacity = '0';
            });
            
            node.addEventListener('click', () => {
                console.log('Node clicked:', nodeData[index]);
                // Custom click behavior can be added here
            });
        });
        
        // Download functionality
        function downloadSVG() {
            const svgData = new XMLSerializer().serializeToString(svg);
            const svgBlob = new Blob([svgData], {type: 'image/svg+xml;charset=utf-8'});
            const svgUrl = URL.createObjectURL(svgBlob);
            const downloadLink = document.createElement('a');
            downloadLink.href = svgUrl;
            downloadLink.download = 'diagram.svg';
            document.body.appendChild(downloadLink);
            downloadLink.click();
            document.body.removeChild(downloadLink);
        }
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            switch(e.key) {
                case '+':
                case '=':
                    e.preventDefault();
                    zoomIn();
                    break;
                case '-':
                    e.preventDefault();
                    zoomOut();
                    break;
                case '0':
                    e.preventDefault();
                    resetZoom();
                    break;
            }
        });
        
        // Initialize
        svg.style.cursor = 'grab';
    `;
  }

  private generateThemeCSS(theme: DiagramTheme): string {
    return `
        /* Theme-specific styles */
        .node.component rect {
            fill: ${theme.nodeStyles.component?.fill || '#dbeafe'};
            stroke: ${theme.nodeStyles.component?.stroke || '#3b82f6'};
            stroke-width: ${theme.nodeStyles.component?.strokeWidth || 2};
        }
        
        .node.service rect {
            fill: ${theme.nodeStyles.service?.fill || '#dcfce7'};
            stroke: ${theme.nodeStyles.service?.stroke || '#16a34a'};
            stroke-width: ${theme.nodeStyles.service?.strokeWidth || 2};
        }
        
        .node.database rect {
            fill: ${theme.nodeStyles.database?.fill || '#fef3c7'};
            stroke: ${theme.nodeStyles.database?.stroke || '#d97706'};
            stroke-width: ${theme.nodeStyles.database?.strokeWidth || 2};
        }
        
        .edge path {
            stroke: ${theme.edgeStyles.default?.stroke || '#6b7280'};
            stroke-width: ${theme.edgeStyles.default?.strokeWidth || 2};
        }
    `;
  }

  private async exportToSVG(options: ExportOptions): Promise<Blob> {
    const svgClone = this.cloneSVGWithStyles();
    
    if (options.width && options.height) {
      svgClone.setAttribute('width', options.width.toString());
      svgClone.setAttribute('height', options.height.toString());
    }
    
    if (options.backgroundColor) {
      const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      rect.setAttribute('width', '100%');
      rect.setAttribute('height', '100%');
      rect.setAttribute('fill', options.backgroundColor);
      svgClone.insertBefore(rect, svgClone.firstChild);
    }
    
    const svgString = new XMLSerializer().serializeToString(svgClone);
    return new Blob([svgString], { type: 'image/svg+xml' });
  }

  private async exportToPNG(options: ExportOptions): Promise<Blob> {
    const svgBlob = await this.exportToSVG(options);
    const svgUrl = URL.createObjectURL(svgBlob);
    
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d')!;
        
        canvas.width = options.width || img.width;
        canvas.height = options.height || img.height;
        
        if (options.backgroundColor) {
          ctx.fillStyle = options.backgroundColor;
          ctx.fillRect(0, 0, canvas.width, canvas.height);
        }
        
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        
        canvas.toBlob((blob) => {
          URL.revokeObjectURL(svgUrl);
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error('Failed to create PNG blob'));
          }
        }, 'image/png', options.quality || 0.9);
      };
      
      img.onerror = () => {
        URL.revokeObjectURL(svgUrl);
        reject(new Error('Failed to load SVG for PNG conversion'));
      };
      
      img.src = svgUrl;
    });
  }

  private async exportToPDF(options: ExportOptions): Promise<Blob> {
    // For PDF export, we'd typically use a library like jsPDF
    // This is a simplified implementation
    const pngBlob = await this.exportToPNG(options);
    
    // In a real implementation, you would use jsPDF:
    // const pdf = new jsPDF();
    // pdf.addImage(pngData, 'PNG', 0, 0, width, height);
    // return pdf.output('blob');
    
    // For now, return the PNG blob with PDF mime type as placeholder
    return new Blob([pngBlob], { type: 'application/pdf' });
  }

  private exportToPlantUML(options: ExportOptions): string {
    const plantUmlOptions = options as any as PlantUMLOptions;
    
    let plantuml = '@startuml\n';
    
    // Add theme
    if (plantUmlOptions.theme && plantUmlOptions.theme !== 'default') {
      plantuml += `!theme ${plantUmlOptions.theme}\n`;
    }
    
    // Add title
    plantuml += 'title Component Architecture Diagram\n\n';
    
    // Add components
    this.nodeData.forEach(node => {
      const name = this.sanitizePlantUMLName(node.componentData.name);
      const type = node.componentData.componentType;
      
      switch (plantUmlOptions.diagramType) {
        case 'component':
          plantuml += `component "${node.componentData.name}" as ${name}\n`;
          break;
        case 'class':
          plantuml += `class ${name} {\n}\n`;
          break;
        case 'deployment':
          plantuml += `node "${node.componentData.name}" as ${name}\n`;
          break;
        default:
          plantuml += `rectangle "${node.componentData.name}" as ${name}\n`;
      }
      
      if (plantUmlOptions.includeNotes && node.componentData.filePath) {
        plantuml += `note right of ${name} : ${node.componentData.filePath}\n`;
      }
    });
    
    plantuml += '\n';
    
    // Add relationships (simplified)
    this.nodeData.forEach(node => {
      node.connections.forEach(targetId => {
        const sourceName = this.sanitizePlantUMLName(node.componentData.name);
        const targetNode = this.nodeData.find(n => n.nodeId === targetId);
        if (targetNode) {
          const targetName = this.sanitizePlantUMLName(targetNode.componentData.name);
          plantuml += `${sourceName} --> ${targetName}\n`;
        }
      });
    });
    
    plantuml += '@enduml';
    
    return plantuml;
  }

  private exportToGraphviz(options: ExportOptions): string {
    const graphvizOptions = options as any as GraphvizOptions;
    
    let dot = `digraph G {\n`;
    dot += `  rankdir=${graphvizOptions.rankdir || 'TB'};\n`;
    dot += `  splines=${graphvizOptions.splines || 'polyline'};\n`;
    dot += `  node [shape=${graphvizOptions.nodeShape || 'box'}];\n\n`;
    
    // Add nodes
    this.nodeData.forEach(node => {
      const name = this.sanitizeGraphvizName(node.componentData.name);
      const label = node.componentData.name;
      const type = node.componentData.componentType;
      
      let nodeStyle = `  ${name} [label="${label}"`;
      
      if (graphvizOptions.includeColors) {
        const color = this.getNodeColor(type);
        nodeStyle += `, fillcolor="${color}", style=filled`;
      }
      
      nodeStyle += `];\n`;
      dot += nodeStyle;
    });
    
    dot += '\n';
    
    // Add edges
    this.nodeData.forEach(node => {
      const sourceName = this.sanitizeGraphvizName(node.componentData.name);
      node.connections.forEach(targetId => {
        const targetNode = this.nodeData.find(n => n.nodeId === targetId);
        if (targetNode) {
          const targetName = this.sanitizeGraphvizName(targetNode.componentData.name);
          dot += `  ${sourceName} -> ${targetName};\n`;
        }
      });
    });
    
    dot += '}\n';
    
    return dot;
  }

  private exportToJSON(options: ExportOptions): string {
    const exportData = {
      metadata: {
        exportDate: new Date().toISOString(),
        format: 'uveddi-diagram-json',
        version: '1.0.0',
        options
      },
      theme: this.theme,
      mermaidCode: this.mermaidCode,
      nodes: this.nodeData,
      svg: new XMLSerializer().serializeToString(this.svgElement)
    };
    
    return JSON.stringify(exportData, null, 2);
  }

  private cloneSVGWithStyles(): SVGElement {
    const svgClone = this.svgElement.cloneNode(true) as SVGElement;
    
    // Apply computed styles to the clone
    this.applyComputedStyles(this.svgElement, svgClone);
    
    return svgClone;
  }

  private applyComputedStyles(source: Element, target: Element): void {
    const sourceStyles = window.getComputedStyle(source);
    
    // Copy important SVG styles
    ['fill', 'stroke', 'stroke-width', 'font-family', 'font-size', 'font-weight'].forEach(prop => {
      const value = sourceStyles.getPropertyValue(prop);
      if (value && value !== 'none') {
        (target as any).style[prop] = value;
      }
    });
    
    // Recursively apply to children
    for (let i = 0; i < source.children.length; i++) {
      this.applyComputedStyles(source.children[i], target.children[i]);
    }
  }

  private sanitizePlantUMLName(name: string): string {
    return name.replace(/[^a-zA-Z0-9_]/g, '_').replace(/^[0-9]/, '_$&');
  }

  private sanitizeGraphvizName(name: string): string {
    return `"${name.replace(/"/g, '\\"')}"`;
  }

  private getNodeColor(type: string): string {
    const colorMap: Record<string, string> = {
      component: '#dbeafe',
      service: '#dcfce7',
      database: '#fef3c7',
      external: '#f3e8ff',
      default: '#f8fafc'
    };
    
    return colorMap[type] || colorMap.default;
  }

  public async downloadFile(blob: Blob | string, filename: string): Promise<void> {
    const url = typeof blob === 'string' 
      ? `data:text/plain;charset=utf-8,${encodeURIComponent(blob)}`
      : URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    
    if (typeof blob !== 'string') {
      URL.revokeObjectURL(url);
    }
  }

  public getSupportedFormats(): string[] {
    return ['html', 'svg', 'png', 'pdf', 'plantuml', 'graphviz', 'json'];
  }

  public updateNodeData(nodeData: NodeEventData[]): void {
    this.nodeData = nodeData;
  }

  public updateTheme(theme: DiagramTheme): void {
    this.theme = theme;
  }

  public updateMermaidCode(mermaidCode: string): void {
    this.mermaidCode = mermaidCode;
  }
}

export default ExportManager;