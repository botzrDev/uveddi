import '@testing-library/jest-dom';
import { configure } from '@testing-library/react';
import { vi, beforeAll, afterEach, afterAll } from 'vitest';
import { server } from './mocks/server';
import { toHaveNoViolations } from 'jest-axe';

// Configure testing library
configure({ testIdAttribute: 'data-testid' });

// Add jest-axe matcher for accessibility testing
expect.extend(toHaveNoViolations);

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  observe() {}
  unobserve() {}
  disconnect() {}
  takeRecords() { return []; }
  get root() { return null; }
  get rootMargin() { return '0px'; }
  get thresholds() { return [0]; }
} as any;

// Mock HTMLCanvasElement.getContext
HTMLCanvasElement.prototype.getContext = vi.fn(() => ({
  fillRect: vi.fn(),
  clearRect: vi.fn(),
  getImageData: vi.fn(() => ({ data: new Array(4) })),
  putImageData: vi.fn(),
  createImageData: vi.fn(() => []),
  setTransform: vi.fn(),
  drawImage: vi.fn(),
  save: vi.fn(),
  fillText: vi.fn(),
  restore: vi.fn(),
  beginPath: vi.fn(),
  moveTo: vi.fn(),
  lineTo: vi.fn(),
  closePath: vi.fn(),
  stroke: vi.fn(),
  translate: vi.fn(),
  scale: vi.fn(),
  rotate: vi.fn(),
  arc: vi.fn(),
  fill: vi.fn(),
  measureText: vi.fn(() => ({ width: 0 })),
  transform: vi.fn(),
  rect: vi.fn(),
  clip: vi.fn(),
})) as any;

// Mock Chart.js
vi.mock('chart.js', () => ({
  Chart: vi.fn(() => ({
    destroy: vi.fn(),
    update: vi.fn(),
    resize: vi.fn(),
  })),
  registerables: [],
}));

// Mock Cytoscape
vi.mock('cytoscape', () => ({
  __esModule: true,
  default: vi.fn(() => ({
    add: vi.fn(),
    remove: vi.fn(),
    layout: vi.fn(() => ({ run: vi.fn() })),
    fit: vi.fn(),
    destroy: vi.fn(),
    on: vi.fn(),
    off: vi.fn(),
    elements: vi.fn(() => ({ length: 0 })),
  })),
}));

// Mock React Grid Layout
vi.mock('react-grid-layout', () => ({
  Responsive: vi.fn(({ children }) => children),
  WidthProvider: vi.fn((component) => component),
}));

// Setup MSW
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());