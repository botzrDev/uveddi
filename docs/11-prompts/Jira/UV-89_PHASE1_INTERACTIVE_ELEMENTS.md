# 🎯 UV-89 Phase 1: Interactive Elements Implementation

## 📋 **Phase Overview**

**Jira Issue**: UV-89 Phase 1 - Interactive Elements  
**Duration**: 3 days  
**Priority**: Foundation phase - all other phases depend on this  
**Focus**: Transform static Mermaid diagrams into interactive, clickable experiences

## 🎯 **Phase 1 Objectives**

Transform the current static diagram system into an interactive experience by implementing:
- ✅ **Clickable Nodes**: Click and double-click event handling
- ✅ **Hover Effects**: Visual feedback and tooltips
- ✅ **Pan & Zoom**: Smooth navigation controls
- ✅ **Context Menus**: Right-click actions for nodes
- ✅ **Keyboard Navigation**: Accessibility support

## 🔍 **Current System Analysis**

### **Existing Implementation**
```rust
// Current: src/analysis/mermaid_generator.rs
pub struct MermaidGenerator {
    template_engine: Tera,
    diagram_specs: HashMap<VizDiagramType, DiagramSpec>,
    cache_engine: Option<Arc<RwLock<DiagramCacheEngine>>>,
}

// Current output: Static Mermaid.js code
pub struct DiagramResult {
    pub diagram_type: DiagramType,
    pub mermaid_src: String,  // Static Mermaid code
    pub components: Vec<Uuid>,
}
```

### **Current Frontend** (Limited)
```typescript
// Current: Basic React components without interactivity
frontend/src/components/ui/  // Basic UI components
frontend/src/pages/          // Static pages
```

### **Rendering Service** (Node.js)
```javascript
// Current: rendering-service/src/renderer.js
async function renderDiagram({ mermaidCode, format = 'svg' }) {
  // Renders static SVG/PNG only
}
```

## 🏗️ **Technical Architecture for Phase 1**

### **Target Architecture**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Mermaid.js    │───▶│   D3.js Layer   │───▶│ Interactive SVG │
│  (Static Code)  │    │ (Event Handler) │    │  (DOM Events)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                        ┌─────────────────┐
                        │ React Component │
                        │   (UI Layer)    │
                        └─────────────────┘
```

### **Implementation Strategy**
1. **Enhance Frontend**: Create interactive diagram component
2. **Extend Rendering**: Add interactive SVG generation
3. **Event System**: Implement comprehensive event handling
4. **Integration**: Connect with existing Rust backend

## 🛠️ **Implementation Requirements**

### **1. Interactive Diagram Component** (Day 1)

Create new React component with D3.js integration:

```typescript
// frontend/src/components/diagrams/InteractiveDiagram.tsx
interface InteractiveDiagramProps {
  mermaidCode: string;
  onNodeClick?: (nodeId: string, nodeData: any) => void;
  onNodeDoubleClick?: (nodeId: string, nodeData: any) => void;
  onNodeHover?: (nodeId: string, nodeData: any) => void;
  onNodeContextMenu?: (nodeId: string, nodeData: any, event: MouseEvent) => void;
  enablePanZoom?: boolean;
  enableTooltips?: boolean;
  enableKeyboardNav?: boolean;
}

export const InteractiveDiagram: React.FC<InteractiveDiagramProps> = ({
  mermaidCode,
  onNodeClick,
  onNodeDoubleClick,
  onNodeHover,
  onNodeContextMenu,
  enablePanZoom = true,
  enableTooltips = true,
  enableKeyboardNav = true
}) => {
  // Implementation here
};
```

### **2. Event Handling System** (Day 1-2)

```typescript
// frontend/src/components/diagrams/EventHandler.ts
interface NodeEventData {
  nodeId: string;
  nodeType: string;
  componentData: {
    name: string;
    filePath: string;
    componentType: string;
    metrics?: ComponentMetrics;
  };
  position: { x: number; y: number };
  connections: string[];
}

class DiagramEventHandler {
  private svgElement: SVGElement;
  private d3Selection: d3.Selection<SVGElement, unknown, null, undefined>;
  
  constructor(svgElement: SVGElement) {
    this.svgElement = svgElement;
    this.d3Selection = d3.select(svgElement);
    this.initializeEventListeners();
  }

  private initializeEventListeners(): void {
    // Click events
    this.d3Selection.selectAll('.node')
      .on('click', this.handleNodeClick.bind(this))
      .on('dblclick', this.handleNodeDoubleClick.bind(this))
      .on('contextmenu', this.handleNodeContextMenu.bind(this));

    // Hover events
    this.d3Selection.selectAll('.node')
      .on('mouseenter', this.handleNodeMouseEnter.bind(this))
      .on('mouseleave', this.handleNodeMouseLeave.bind(this));
  }

  private handleNodeClick(event: MouseEvent, data: NodeEventData): void {
    // Prevent event bubbling
    event.stopPropagation();
    
    // Highlight connected nodes
    this.highlightConnections(data.nodeId);
    
    // Emit click event
    this.onNodeClick?.(data.nodeId, data);
  }

  private handleNodeMouseEnter(event: MouseEvent, data: NodeEventData): void {
    // Show tooltip
    this.showTooltip(event, data);
    
    // Add hover styling
    d3.select(event.currentTarget as Element)
      .classed('node-hovered', true);
  }

  private highlightConnections(nodeId: string): void {
    // Highlight connected edges
    this.d3Selection.selectAll('.edge')
      .classed('highlighted', (d: any) => 
        d.source === nodeId || d.target === nodeId
      );
  }

  private showTooltip(event: MouseEvent, data: NodeEventData): void {
    // Create and position tooltip
    const tooltip = d3.select('body')
      .append('div')
      .attr('class', 'diagram-tooltip')
      .style('position', 'absolute')
      .style('left', `${event.pageX + 10}px`)
      .style('top', `${event.pageY - 10}px`)
      .html(this.generateTooltipContent(data));
  }
}
```

### **3. Pan & Zoom Implementation** (Day 2)

```typescript
// frontend/src/components/diagrams/PanZoomController.ts
import * as d3 from 'd3';

class PanZoomController {
  private zoom: d3.ZoomBehavior<SVGElement, unknown>;
  private svgElement: SVGElement;
  private containerGroup: SVGGElement;

  constructor(svgElement: SVGElement, containerGroup: SVGGElement) {
    this.svgElement = svgElement;
    this.containerGroup = containerGroup;
    this.initializePanZoom();
  }

  private initializePanZoom(): void {
    this.zoom = d3.zoom<SVGElement, unknown>()
      .scaleExtent([0.1, 10])
      .on('zoom', this.handleZoom.bind(this));

    d3.select(this.svgElement)
      .call(this.zoom);
  }

  private handleZoom(event: d3.D3ZoomEvent<SVGElement, unknown>): void {
    const { transform } = event;
    
    d3.select(this.containerGroup)
      .attr('transform', transform.toString());
  }

  public zoomToFit(): void {
    const bounds = this.containerGroup.getBBox();
    const parent = this.svgElement.getBoundingClientRect();
    
    const fullWidth = parent.width;
    const fullHeight = parent.height;
    const width = bounds.width;
    const height = bounds.height;
    
    const midX = bounds.x + width / 2;
    const midY = bounds.y + height / 2;
    
    const scale = Math.min(fullWidth / width, fullHeight / height) * 0.9;
    
    const translate = [
      fullWidth / 2 - scale * midX,
      fullHeight / 2 - scale * midY
    ];

    d3.select(this.svgElement)
      .transition()
      .duration(750)
      .call(
        this.zoom.transform,
        d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale)
      );
  }
}
```

### **4. Context Menu System** (Day 2-3)

```typescript
// frontend/src/components/diagrams/ContextMenu.tsx
interface ContextMenuProps {
  nodeId: string;
  nodeData: NodeEventData;
  position: { x: number; y: number };
  onClose: () => void;
  onAction: (action: string, nodeId: string) => void;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  nodeId,
  nodeData,
  position,
  onClose,
  onAction
}) => {
  const menuItems = [
    { id: 'view-details', label: 'View Details', icon: '👁️' },
    { id: 'highlight-dependencies', label: 'Highlight Dependencies', icon: '🔗' },
    { id: 'focus-component', label: 'Focus on Component', icon: '🎯' },
    { id: 'export-subgraph', label: 'Export Subgraph', icon: '📤' },
    { id: 'add-comment', label: 'Add Comment', icon: '💬' }
  ];

  return (
    <div 
      className="context-menu"
      style={{
        position: 'fixed',
        left: position.x,
        top: position.y,
        zIndex: 1000
      }}
    >
      {menuItems.map(item => (
        <button
          key={item.id}
          className="context-menu-item"
          onClick={() => onAction(item.id, nodeId)}
        >
          <span className="icon">{item.icon}</span>
          <span className="label">{item.label}</span>
        </button>
      ))}
    </div>
  );
};
```

### **5. Keyboard Navigation** (Day 3)

```typescript
// frontend/src/components/diagrams/KeyboardController.ts
class KeyboardController {
  private selectedNodeId: string | null = null;
  private nodes: NodeEventData[] = [];
  private onNodeSelect: (nodeId: string) => void;

  constructor(nodes: NodeEventData[], onNodeSelect: (nodeId: string) => void) {
    this.nodes = nodes;
    this.onNodeSelect = onNodeSelect;
    this.initializeKeyboardListeners();
  }

  private initializeKeyboardListeners(): void {
    document.addEventListener('keydown', this.handleKeyDown.bind(this));
  }

  private handleKeyDown(event: KeyboardEvent): void {
    if (!this.selectedNodeId) return;

    switch (event.key) {
      case 'ArrowUp':
      case 'ArrowDown':
      case 'ArrowLeft':
      case 'ArrowRight':
        event.preventDefault();
        this.navigateToAdjacentNode(event.key);
        break;
      case 'Enter':
        event.preventDefault();
        this.activateSelectedNode();
        break;
      case 'Escape':
        event.preventDefault();
        this.clearSelection();
        break;
    }
  }

  private navigateToAdjacentNode(direction: string): void {
    // Find adjacent node based on direction
    const currentNode = this.nodes.find(n => n.nodeId === this.selectedNodeId);
    if (!currentNode) return;

    // Implementation for finding adjacent nodes based on position
    const adjacentNode = this.findAdjacentNode(currentNode, direction);
    if (adjacentNode) {
      this.selectNode(adjacentNode.nodeId);
    }
  }
}
```

## 🔧 **Integration Points**

### **Backend API Extensions**
```rust
// src/server/mod.rs - Add new endpoint for interactive diagram data
#[derive(Serialize, Deserialize)]
pub struct InteractiveDiagramRequest {
    pub mermaid_code: String,
    pub enable_interactions: bool,
    pub node_metadata: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InteractiveDiagramResponse {
    pub svg_content: String,
    pub node_metadata: HashMap<String, NodeMetadata>,
    pub interaction_config: InteractionConfig,
}

pub async fn generate_interactive_diagram(
    request: InteractiveDiagramRequest
) -> Result<InteractiveDiagramResponse, ApiError> {
    // Implementation
}
```

### **Enhanced Rendering Service**
```javascript
// rendering-service/src/interactive-renderer.js
async function renderInteractiveDiagram({ mermaidCode, options }) {
  const page = await getWorkerPage();
  
  // Inject D3.js and interaction scripts
  await page.addScriptTag({ path: './node_modules/d3/dist/d3.min.js' });
  await page.addScriptTag({ content: interactionScript });
  
  // Render with interaction capabilities
  const result = await page.evaluate(async (code, opts) => {
    // Generate interactive SVG with embedded event handlers
    const svg = await mermaid.render('diagram', code);
    
    // Add interaction metadata
    const metadata = extractNodeMetadata(svg);
    
    return {
      svg: svg.svg,
      metadata: metadata,
      interactionConfig: opts
    };
  }, mermaidCode, options);
  
  return result;
}
```

## 🧪 **Testing Strategy**

### **Unit Tests**
```typescript
// frontend/src/components/diagrams/__tests__/InteractiveDiagram.test.tsx
describe('InteractiveDiagram', () => {
  test('renders mermaid diagram correctly', () => {
    const mermaidCode = 'graph TD; A-->B; B-->C;';
    render(<InteractiveDiagram mermaidCode={mermaidCode} />);
    expect(screen.getByRole('img')).toBeInTheDocument();
  });

  test('handles node click events', () => {
    const onNodeClick = jest.fn();
    const { container } = render(
      <InteractiveDiagram 
        mermaidCode="graph TD; A-->B;" 
        onNodeClick={onNodeClick} 
      />
    );
    
    const nodeElement = container.querySelector('.node');
    fireEvent.click(nodeElement!);
    
    expect(onNodeClick).toHaveBeenCalledWith('A', expect.any(Object));
  });

  test('shows tooltips on hover', async () => {
    const { container } = render(
      <InteractiveDiagram 
        mermaidCode="graph TD; A-->B;" 
        enableTooltips={true} 
      />
    );
    
    const nodeElement = container.querySelector('.node');
    fireEvent.mouseEnter(nodeElement!);
    
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });
  });
});
```

### **Integration Tests**
```typescript
// frontend/src/components/diagrams/__tests__/integration.test.tsx
describe('Diagram Integration', () => {
  test('pan and zoom functionality', () => {
    const { container } = render(
      <InteractiveDiagram 
        mermaidCode="graph TD; A-->B; B-->C; C-->D;" 
        enablePanZoom={true} 
      />
    );
    
    const svgElement = container.querySelector('svg');
    
    // Simulate zoom
    fireEvent.wheel(svgElement!, { deltaY: -100 });
    
    // Check transform applied
    const groupElement = container.querySelector('g');
    expect(groupElement).toHaveAttribute('transform');
  });
});
```

## 📋 **Implementation Checklist**

### **Day 1: Foundation**
- [ ] Create `InteractiveDiagram` React component
- [ ] Implement basic D3.js integration
- [ ] Add click event handling for nodes
- [ ] Create tooltip system
- [ ] Set up development environment with D3.js

### **Day 2: Navigation & Context**
- [ ] Implement pan & zoom with d3-zoom
- [ ] Add hover effects and visual feedback
- [ ] Create context menu component
- [ ] Implement connection highlighting
- [ ] Add zoom-to-fit functionality

### **Day 3: Polish & Integration**
- [ ] Implement keyboard navigation
- [ ] Add accessibility features (ARIA labels)
- [ ] Integrate with existing backend API
- [ ] Create comprehensive test suite
- [ ] Document component API and usage

## 🎯 **Success Criteria**

### **Functional Requirements**
- [ ] All nodes are clickable with proper event handling
- [ ] Hover effects work smoothly with tooltips
- [ ] Pan and zoom controls are responsive
- [ ] Context menus appear on right-click
- [ ] Keyboard navigation is fully functional

### **Performance Requirements**
- [ ] Event handling responds within 16ms (60fps)
- [ ] Smooth pan/zoom without lag
- [ ] Tooltips appear within 100ms of hover
- [ ] No memory leaks during interaction

### **User Experience**
- [ ] Intuitive interaction patterns
- [ ] Visual feedback for all interactions
- [ ] Accessible via keyboard
- [ ] Responsive across different screen sizes

## 📚 **Dependencies & Setup**

### **Frontend Dependencies**
```json
{
  "dependencies": {
    "d3": "^7.8.5",
    "d3-zoom": "^3.0.0",
    "react": "^18.2.0",
    "typescript": "^5.0.0"
  },
  "devDependencies": {
    "@testing-library/react": "^13.4.0",
    "@testing-library/jest-dom": "^5.16.5",
    "jest": "^29.5.0"
  }
}
```

### **File Structure**
```
frontend/src/components/diagrams/
├── InteractiveDiagram.tsx
├── EventHandler.ts
├── PanZoomController.ts
├── ContextMenu.tsx
├── KeyboardController.ts
├── TooltipManager.ts
├── types.ts
└── __tests__/
    ├── InteractiveDiagram.test.tsx
    ├── EventHandler.test.ts
    └── integration.test.tsx
```

## 🚀 **Getting Started**

1. **Install Dependencies**: Add D3.js and related packages
2. **Create Component Structure**: Set up the file structure above
3. **Start with Basic Interactivity**: Implement click events first
4. **Add Visual Feedback**: Implement hover effects and tooltips
5. **Integrate Pan/Zoom**: Add navigation controls
6. **Test Thoroughly**: Ensure all interactions work smoothly

This phase creates the foundation for all future interactive features. Once complete, you'll have a fully interactive diagram system ready for animations, collaboration, and advanced features.

**Ready to start Phase 1?** 🚀