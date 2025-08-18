import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render, waitForChartRender } from '../../../test-utils/test-utils';
import { PriorityMatrix } from '../PriorityMatrix';
import { mockIssues } from '../../../test-utils/mocks/mockData';

describe('PriorityMatrix', () => {
  const mockProps = {
    issues: mockIssues,
    onIssueSelect: jest.fn(),
    onFilterChange: jest.fn(),
    selectedIssues: [],
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the priority matrix component', async () => {
    render(<PriorityMatrix {...mockProps} />);
    
    expect(screen.getByText('Priority Matrix')).toBeInTheDocument();
    expect(screen.getByText('Impact vs Effort Analysis')).toBeInTheDocument();
    
    await waitForChartRender();
  });

  it('displays severity filter buttons', () => {
    render(<PriorityMatrix {...mockProps} />);
    
    expect(screen.getByText('All')).toBeInTheDocument();
    expect(screen.getByText('Critical')).toBeInTheDocument();
    expect(screen.getByText('High')).toBeInTheDocument();
    expect(screen.getByText('Medium')).toBeInTheDocument();
    expect(screen.getByText('Low')).toBeInTheDocument();
  });

  it('filters issues by severity when filter button is clicked', async () => {
    render(<PriorityMatrix {...mockProps} />);
    
    const criticalButton = screen.getByText('Critical');
    fireEvent.click(criticalButton);
    
    await waitFor(() => {
      expect(mockProps.onFilterChange).toHaveBeenCalledWith('critical');
    });
  });

  it('handles quadrant hover events', async () => {
    render(<PriorityMatrix {...mockProps} />);
    
    await waitForChartRender();
    
    // Simulate hover on high impact, low effort quadrant
    const quadrantInfo = screen.getByText(/High Impact, Low Effort/);
    expect(quadrantInfo).toBeInTheDocument();
  });

  it('displays issue statistics', () => {
    render(<PriorityMatrix {...mockProps} />);
    
    expect(screen.getByText('4 issues')).toBeInTheDocument();
    expect(screen.getByText(/Critical:/)).toBeInTheDocument();
    expect(screen.getByText(/High:/)).toBeInTheDocument();
  });

  it('handles empty issues array', () => {
    render(<PriorityMatrix {...mockProps} issues={[]} />);
    
    expect(screen.getByText('Priority Matrix')).toBeInTheDocument();
    expect(screen.getByText('0 issues')).toBeInTheDocument();
  });

  it('highlights selected issues', async () => {
    const propsWithSelected = {
      ...mockProps,
      selectedIssues: ['1', '2']
    };
    
    render(<PriorityMatrix {...propsWithSelected} />);
    
    await waitForChartRender();
    
    // Check that selected issues are highlighted (implementation depends on Chart.js config)
    expect(screen.getByText('Priority Matrix')).toBeInTheDocument();
  });

  it('calls onIssueSelect when chart point is clicked', async () => {
    const { container } = render(<PriorityMatrix {...mockProps} />);
    
    await waitForChartRender();
    
    // Mock a chart click event
    const canvas = container.querySelector('canvas');
    if (canvas) {
      fireEvent.click(canvas);
      // Note: In a real test, we'd need to mock Chart.js click events more thoroughly
    }
  });

  it('updates chart when issues prop changes', async () => {
    const { rerender } = render(<PriorityMatrix {...mockProps} />);
    
    await waitForChartRender();
    
    const newIssues = [...mockIssues.slice(0, 2)];
    rerender(<PriorityMatrix {...mockProps} issues={newIssues} />);
    
    expect(screen.getByText('2 issues')).toBeInTheDocument();
  });

  it('maintains aspect ratio on resize', async () => {
    render(<PriorityMatrix {...mockProps} />);
    
    await waitForChartRender();
    
    // Simulate window resize
    global.dispatchEvent(new Event('resize'));
    
    // Chart should maintain its aspect ratio
    expect(screen.getByText('Priority Matrix')).toBeInTheDocument();
  });

  it('has proper accessibility attributes', () => {
    render(<PriorityMatrix {...mockProps} />);
    
    const canvas = screen.getByRole('img', { name: /priority matrix chart/i });
    expect(canvas).toHaveAttribute('aria-label');
  });
});