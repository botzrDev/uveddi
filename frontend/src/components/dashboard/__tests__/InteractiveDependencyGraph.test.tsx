import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render } from '../../../test-utils/test-utils';
import { InteractiveDependencyGraph } from '../InteractiveDependencyGraph';
import { mockDependencies } from '../../../test-utils/mocks/mockData';

// Mock Cytoscape
const mockCytoscape = {
  add: jest.fn(),
  remove: jest.fn(),
  layout: jest.fn(() => ({ run: jest.fn() })),
  fit: jest.fn(),
  destroy: jest.fn(),
  on: jest.fn(),
  off: jest.fn(),
  elements: jest.fn(() => ({ length: mockDependencies.length })),
  nodes: jest.fn(() => ({
    forEach: jest.fn(),
    select: jest.fn(() => ({ addClass: jest.fn(), removeClass: jest.fn() }))
  })),
  edges: jest.fn(() => ({
    forEach: jest.fn()
  })),
  getElementById: jest.fn(() => ({
    addClass: jest.fn(),
    removeClass: jest.fn(),
    connectedEdges: jest.fn(() => ({ addClass: jest.fn(), removeClass: jest.fn() }))
  }))
};

jest.mock('cytoscape', () => () => mockCytoscape);

describe('InteractiveDependencyGraph', () => {
  const mockProps = {
    dependencies: mockDependencies,
    onNodeSelect: jest.fn(),
    onClusterAnalysis: jest.fn(),
    selectedNodes: [],
    highlightCycles: true,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the dependency graph component', () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    expect(screen.getByText('Dependency Graph')).toBeInTheDocument();
    expect(screen.getByText('Interactive Dependency Analysis')).toBeInTheDocument();
  });

  it('displays layout control buttons', () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    expect(screen.getByRole('button', { name: /hierarchical/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /circular/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /force/i })).toBeInTheDocument();
  });

  it('shows dependency statistics', () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    expect(screen.getByText(`${mockDependencies.length} nodes`)).toBeInTheDocument();
    expect(screen.getByText(/dependencies/)).toBeInTheDocument();
  });

  it('handles layout change', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const circularButton = screen.getByRole('button', { name: /circular/i });
    fireEvent.click(circularButton);
    
    await waitFor(() => {
      expect(mockCytoscape.layout).toHaveBeenCalledWith(
        expect.objectContaining({ name: 'circle' })
      );
    });
  });

  it('enables cluster analysis mode', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const clusterButton = screen.getByRole('button', { name: /cluster analysis/i });
    fireEvent.click(clusterButton);
    
    await waitFor(() => {
      expect(mockProps.onClusterAnalysis).toHaveBeenCalled();
    });
  });

  it('highlights circular dependencies when enabled', () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    expect(screen.getByText(/Circular Dependencies/)).toBeInTheDocument();
    // Check that circular dependency highlighting is active
  });

  it('shows node details on selection', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    // Simulate node selection
    const nodeDetailsPanel = screen.getByTestId('node-details-panel');
    expect(nodeDetailsPanel).toBeInTheDocument();
  });

  it('displays filtering controls', () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    expect(screen.getByRole('textbox', { name: /filter nodes/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /services/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /components/i })).toBeInTheDocument();
  });

  it('filters nodes by type', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const servicesFilter = screen.getByRole('button', { name: /services/i });
    fireEvent.click(servicesFilter);
    
    await waitFor(() => {
      // Verify filtering logic is applied
      expect(mockCytoscape.elements).toHaveBeenCalled();
    });
  });

  it('handles search functionality', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const searchInput = screen.getByRole('textbox', { name: /filter nodes/i });
    fireEvent.change(searchInput, { target: { value: 'UserService' } });
    
    await waitFor(() => {
      expect(mockCytoscape.getElementById).toHaveBeenCalledWith('UserService');
    });
  });

  it('shows complexity metrics overlay', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const metricsToggle = screen.getByRole('button', { name: /show metrics/i });
    fireEvent.click(metricsToggle);
    
    await waitFor(() => {
      expect(screen.getByText(/Complexity/)).toBeInTheDocument();
      expect(screen.getByText(/Coupling/)).toBeInTheDocument();
    });
  });

  it('handles node selection events', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    // Mock cytoscape node selection
    const selectHandler = mockCytoscape.on.mock.calls.find(call => call[0] === 'select')[1];
    if (selectHandler) {
      selectHandler({ target: { id: () => 'UserService' } });
    }
    
    await waitFor(() => {
      expect(mockProps.onNodeSelect).toHaveBeenCalledWith('UserService');
    });
  });

  it('updates graph when dependencies change', () => {
    const { rerender } = render(<InteractiveDependencyGraph {...mockProps} />);
    
    const newDependencies = mockDependencies.slice(0, 2);
    rerender(<InteractiveDependencyGraph {...mockProps} dependencies={newDependencies} />);
    
    expect(mockCytoscape.add).toHaveBeenCalled();
  });

  it('handles empty dependencies gracefully', () => {
    render(<InteractiveDependencyGraph {...mockProps} dependencies={[]} />);
    
    expect(screen.getByText('Dependency Graph')).toBeInTheDocument();
    expect(screen.getByText('0 nodes')).toBeInTheDocument();
  });

  it('provides export functionality', async () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const exportButton = screen.getByRole('button', { name: /export/i });
    fireEvent.click(exportButton);
    
    // Check export options
    expect(screen.getByText(/PNG/)).toBeInTheDocument();
    expect(screen.getByText(/SVG/)).toBeInTheDocument();
  });

  it('shows performance metrics for large graphs', () => {
    const largeDependencies = Array(100).fill(null).map((_, i) => ({
      ...mockDependencies[0],
      id: `node-${i}`,
      name: `Node ${i}`
    }));
    
    render(<InteractiveDependencyGraph {...mockProps} dependencies={largeDependencies} />);
    
    expect(screen.getByText(/Performance Mode/)).toBeInTheDocument();
  });

  it('has proper accessibility attributes', () => {
    render(<InteractiveDependencyGraph {...mockProps} />);
    
    const graphContainer = screen.getByRole('img', { name: /dependency graph/i });
    expect(graphContainer).toHaveAttribute('aria-label');
    expect(graphContainer).toHaveAttribute('tabindex', '0');
  });

  it('destroys cytoscape instance on unmount', () => {
    const { unmount } = render(<InteractiveDependencyGraph {...mockProps} />);
    
    unmount();
    
    expect(mockCytoscape.destroy).toHaveBeenCalled();
  });
});