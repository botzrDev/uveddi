import '@testing-library/jest-dom';
import { configure } from '@testing-library/react';
import { server } from './mocks/server';

// Configure testing library
configure({ testIdAttribute: 'data-testid' });

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
};

// Mock HTMLCanvasElement.getContext
HTMLCanvasElement.prototype.getContext = jest.fn(() => ({
  fillRect: jest.fn(),
  clearRect: jest.fn(),
  getImageData: jest.fn(() => ({ data: new Array(4) })),
  putImageData: jest.fn(),
  createImageData: jest.fn(() => []),
  setTransform: jest.fn(),
  drawImage: jest.fn(),
  save: jest.fn(),
  fillText: jest.fn(),
  restore: jest.fn(),
  beginPath: jest.fn(),
  moveTo: jest.fn(),
  lineTo: jest.fn(),
  closePath: jest.fn(),
  stroke: jest.fn(),
  translate: jest.fn(),
  scale: jest.fn(),
  rotate: jest.fn(),
  arc: jest.fn(),
  fill: jest.fn(),
  measureText: jest.fn(() => ({ width: 0 })),
  transform: jest.fn(),
  rect: jest.fn(),
  clip: jest.fn(),
})) as any;

// Mock Chart.js
jest.mock('chart.js', () => ({
  Chart: jest.fn(() => ({
    destroy: jest.fn(),
    update: jest.fn(),
    resize: jest.fn(),
  })),
  registerables: [],
}));

// Mock Cytoscape
jest.mock('cytoscape', () => ({
  __esModule: true,
  default: jest.fn(() => ({
    add: jest.fn(),
    remove: jest.fn(),
    layout: jest.fn(() => ({ run: jest.fn() })),
    fit: jest.fn(),
    destroy: jest.fn(),
    on: jest.fn(),
    off: jest.fn(),
    elements: jest.fn(() => ({ length: 0 })),
  })),
}));

// Mock React Grid Layout
jest.mock('react-grid-layout', () => ({
  Responsive: jest.fn(({ children }) => children),
  WidthProvider: jest.fn((component) => component),
}));

// Setup MSW
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());