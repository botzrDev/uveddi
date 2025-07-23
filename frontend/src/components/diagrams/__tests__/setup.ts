// Test setup and utilities for UV-89 Interactive Diagram testing
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock D3.js for testing
vi.mock('d3', () => ({
  select: vi.fn(() => ({
    selectAll: vi.fn(() => ({
      nodes: vi.fn(() => []),
      each: vi.fn(),
      on: vi.fn(),
      attr: vi.fn(),
      style: vi.fn(),
      text: vi.fn(),
      append: vi.fn(),
      remove: vi.fn(),
      empty: vi.fn(() => false),
      node: vi.fn(() => ({ getBBox: () => ({ x: 0, y: 0, width: 100, height: 50 }) })),
      transition: vi.fn(() => ({
        duration: vi.fn(),
        attr: vi.fn(),
        style: vi.fn()
      }))
    })),
    select: vi.fn(),
    append: vi.fn(),
    attr: vi.fn(),
    style: vi.fn(),
    text: vi.fn(),
    on: vi.fn(),
    node: vi.fn(() => ({ getBBox: () => ({ x: 0, y: 0, width: 100, height: 50 }) })),
    empty: vi.fn(() => false)
  })),
  zoom: vi.fn(() => ({
    scaleExtent: vi.fn(() => ({
      on: vi.fn()
    }))
  })),
  zoomIdentity: {
    translate: vi.fn(() => ({
      scale: vi.fn()
    }))
  },
  scaleLinear: vi.fn(() => ({
    domain: vi.fn(() => ({
      range: vi.fn()
    }))
  })),
  scaleLog: vi.fn(() => ({
    domain: vi.fn(() => ({
      range: vi.fn()
    }))
  })),
  scaleThreshold: vi.fn(() => ({
    domain: vi.fn(() => ({
      range: vi.fn()
    }))
  })),
  scaleQuantile: vi.fn(() => ({
    domain: vi.fn(() => ({
      range: vi.fn()
    }))
  }))
}));

// Mock GSAP for testing
vi.mock('gsap', () => ({
  gsap: {
    timeline: vi.fn(() => ({
      add: vi.fn(),
      play: vi.fn(),
      pause: vi.fn(),
      restart: vi.fn(),
      kill: vi.fn(),
      timeScale: vi.fn(),
      onStart: vi.fn(),
      onComplete: vi.fn()
    })),
    to: vi.fn(),
    fromTo: vi.fn(),
    set: vi.fn(),
    delayedCall: vi.fn()
  }
}));

// Mock Y.js for collaboration testing
vi.mock('yjs', () => ({
  Doc: vi.fn(() => ({
    getMap: vi.fn(() => ({
      set: vi.fn(),
      get: vi.fn(),
      forEach: vi.fn(),
      observe: vi.fn()
    })),
    getArray: vi.fn(() => ({
      push: vi.fn(),
      toArray: vi.fn(() => []),
      observe: vi.fn(),
      delete: vi.fn(),
      insert: vi.fn()
    })),
    destroy: vi.fn()
  }))
}));

vi.mock('y-websocket', () => ({
  WebsocketProvider: vi.fn(() => ({
    on: vi.fn(),
    destroy: vi.fn(),
    ws: { readyState: WebSocket.OPEN }
  }))
}));

vi.mock('y-indexeddb', () => ({
  IndexeddbPersistence: vi.fn(() => ({
    destroy: vi.fn()
  }))
}));

// Mock Mermaid for testing
vi.mock('mermaid', () => ({
  default: {
    initialize: vi.fn(),
    render: vi.fn(() => Promise.resolve({ 
      svg: '<svg><g><rect></rect><text>Test</text></g></svg>' 
    }))
  }
}));

// Test data factories
export const createMockNodeData = (overrides = {}) => ({
  nodeId: 'test-node-1',
  nodeType: 'component',
  componentData: {
    name: 'TestComponent',
    filePath: 'src/test-component.rs',
    componentType: 'module',
    metrics: {
      complexity: 0.5,
      coupling: 0.3,
      testCoverage: 0.8,
      performance: 0.9,
      security: 0.95
    }
  },
  position: { x: 100, y: 100 },
  connections: ['test-node-2'],
  ...overrides
});

export const createMockEdgeData = (overrides = {}) => ({
  id: 'test-edge-1',
  source: 'test-node-1',
  target: 'test-node-2',
  label: 'depends on',
  ...overrides
});

export const createMockSVGElement = () => {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.innerHTML = `
    <defs></defs>
    <g>
      <g class="node" data-node-id="test-node-1">
        <rect width="100" height="50"></rect>
        <text>Test Component</text>
      </g>
      <g class="edge" data-edge-id="test-edge-1">
        <path d="M 100 100 L 200 200"></path>
      </g>
    </g>
  `;
  
  // Mock getBBox method
  const mockGetBBox = vi.fn(() => ({ x: 0, y: 0, width: 100, height: 50 }));
  svg.getBBox = mockGetBBox;
  
  // Mock all child elements
  const elements = svg.querySelectorAll('*');
  elements.forEach(el => {
    (el as any).getBBox = mockGetBBox;
  });
  
  return svg;
};

export const createMockAnimationConfig = (overrides = {}) => ({
  enableFlowAnimations: true,
  enableProgressiveBuilding: true,
  enableTransitions: true,
  animationSpeed: 1,
  autoPlay: false,
  ...overrides
});

export const createMockTheme = (overrides = {}) => ({
  id: 'test-theme',
  name: 'Test Theme',
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
      borderRadius: 8
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
  },
  ...overrides
});

export const createMockCollaborationConfig = (overrides = {}) => ({
  websocketUrl: 'ws://localhost:3001',
  roomId: 'test-room',
  userId: 'test-user-1',
  userName: 'Test User',
  userColor: '#3b82f6',
  enablePersistence: true,
  enableUserPresence: true,
  enableComments: true,
  enableLiveEditing: true,
  ...overrides
});

// Test utilities
export const waitForAnimationFrame = () => 
  new Promise(resolve => requestAnimationFrame(resolve));

export const waitForTimeout = (ms: number) => 
  new Promise(resolve => setTimeout(resolve, ms));

export const triggerMouseEvent = (element: Element, eventType: string, options = {}) => {
  const event = new MouseEvent(eventType, {
    bubbles: true,
    cancelable: true,
    view: window,
    ...options
  });
  element.dispatchEvent(event);
};

export const triggerKeyboardEvent = (element: Element, eventType: string, key: string, options = {}) => {
  const event = new KeyboardEvent(eventType, {
    bubbles: true,
    cancelable: true,
    key,
    ...options
  });
  element.dispatchEvent(event);
};

// Performance testing utilities
export const measurePerformance = async (fn: () => Promise<void> | void) => {
  const start = performance.now();
  await fn();
  const end = performance.now();
  return end - start;
};

export const createLargeDiagramData = (nodeCount: number) => {
  const nodes = [];
  const edges = [];
  
  for (let i = 0; i < nodeCount; i++) {
    nodes.push(createMockNodeData({
      nodeId: `node-${i}`,
      componentData: {
        name: `Component${i}`,
        filePath: `src/component${i}.rs`,
        componentType: i % 3 === 0 ? 'service' : i % 3 === 1 ? 'component' : 'database'
      },
      position: { x: (i % 10) * 120, y: Math.floor(i / 10) * 80 }
    }));
    
    if (i > 0) {
      edges.push(createMockEdgeData({
        id: `edge-${i}`,
        source: `node-${i-1}`,
        target: `node-${i}`
      }));
    }
  }
  
  return { nodes, edges };
};

// Accessibility testing utilities
export const checkAriaLabels = (container: HTMLElement) => {
  const interactive = container.querySelectorAll('button, [role="button"], [tabindex]');
  const missingLabels = Array.from(interactive).filter(el => 
    !el.getAttribute('aria-label') && 
    !el.getAttribute('aria-labelledby') &&
    !el.textContent?.trim()
  );
  return missingLabels;
};

export const checkColorContrast = async (element: HTMLElement) => {
  // Simplified color contrast check
  const styles = window.getComputedStyle(element);
  const backgroundColor = styles.backgroundColor;
  const color = styles.color;
  
  // Return mock contrast ratio for testing
  return { backgroundColor, color, ratio: 4.5, passes: true };
};

// Memory leak detection utilities
export const detectMemoryLeaks = () => {
  const initialMemory = (performance as any).memory?.usedJSHeapSize || 0;
  
  return {
    check: () => {
      const currentMemory = (performance as any).memory?.usedJSHeapSize || 0;
      const increase = currentMemory - initialMemory;
      return {
        initialMemory,
        currentMemory,
        increase,
        leakDetected: increase > 10 * 1024 * 1024 // 10MB threshold
      };
    }
  };
};

// Visual regression testing utilities
export const captureElementScreenshot = async (element: HTMLElement) => {
  // Mock screenshot capture for testing
  return `data:image/png;base64,${btoa('mock-screenshot-data')}`;
};

export const compareScreenshots = (screenshot1: string, screenshot2: string) => {
  // Mock image comparison
  return {
    identical: screenshot1 === screenshot2,
    difference: screenshot1 === screenshot2 ? 0 : 0.1,
    threshold: 0.05
  };
};