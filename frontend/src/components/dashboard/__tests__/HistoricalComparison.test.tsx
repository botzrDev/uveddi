import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render, waitForChartRender } from '../../../test-utils/test-utils';
import { HistoricalComparison } from '../HistoricalComparison';
import { mockHistoricalData } from '../../../test-utils/mocks/mockData';

describe('HistoricalComparison', () => {
  const mockProps = {
    historicalData: mockHistoricalData.snapshots,
    currentSnapshot: mockHistoricalData.snapshots[mockHistoricalData.snapshots.length - 1],
    onSnapshotSelect: jest.fn(),
    onComparisonGenerate: jest.fn(),
    onTrendAnalysis: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders historical comparison component', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    expect(screen.getByText('Historical Analysis')).toBeInTheDocument();
    expect(screen.getByText('Timeline & Comparisons')).toBeInTheDocument();
  });

  it('displays timeline chart', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    await waitForChartRender();
    
    expect(screen.getByText(/Quality Score Timeline/)).toBeInTheDocument();
  });

  it('shows snapshot selector', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    expect(screen.getByText(/Compare Snapshots/)).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /baseline snapshot/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /target snapshot/i })).toBeInTheDocument();
  });

  it('handles snapshot selection', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const baselineSelect = screen.getByRole('combobox', { name: /baseline snapshot/i });
    fireEvent.mouseDown(baselineSelect);
    
    const option = screen.getByText(/January 2024/);
    fireEvent.click(option);
    
    await waitFor(() => {
      expect(mockProps.onSnapshotSelect).toHaveBeenCalledWith(
        'baseline',
        expect.stringContaining('snapshot-1')
      );
    });
  });

  it('displays comparison metrics', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const compareButton = screen.getByRole('button', { name: /generate comparison/i });
    fireEvent.click(compareButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Quality Score Change/)).toBeInTheDocument();
      expect(screen.getByText(/Issue Count Change/)).toBeInTheDocument();
      expect(screen.getByText(/Technical Debt Change/)).toBeInTheDocument();
    });
  });

  it('shows trend indicators', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    // Look for trend arrows or indicators
    const trendIndicators = screen.getAllByTestId(/trend-arrow/);
    expect(trendIndicators.length).toBeGreaterThan(0);
  });

  it('displays time range selector', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    expect(screen.getByRole('button', { name: /last 3 months/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /last 6 months/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /last year/i })).toBeInTheDocument();
  });

  it('handles time range changes', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const sixMonthsButton = screen.getByRole('button', { name: /last 6 months/i });
    fireEvent.click(sixMonthsButton);
    
    await waitForChartRender();
    
    // Chart should update to show 6 months of data
    expect(screen.getByText(/Quality Score Timeline/)).toBeInTheDocument();
  });

  it('shows key insights', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    expect(screen.getByText(/Key Insights/)).toBeInTheDocument();
    expect(screen.getByText(/improvement/i)).toBeInTheDocument();
  });

  it('displays regression analysis', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const trendButton = screen.getByRole('button', { name: /trend analysis/i });
    fireEvent.click(trendButton);
    
    await waitFor(() => {
      expect(mockProps.onTrendAnalysis).toHaveBeenCalled();
      expect(screen.getByText(/Regression Analysis/)).toBeInTheDocument();
    });
  });

  it('shows predicted future trends', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const predictionsToggle = screen.getByRole('switch', { name: /show predictions/i });
    fireEvent.click(predictionsToggle);
    
    await waitForChartRender();
    
    expect(screen.getByText(/Predicted Trend/)).toBeInTheDocument();
  });

  it('displays milestone markers', async () => {
    const propsWithMilestones = {
      ...mockProps,
      milestones: [
        {
          id: 'v1.0',
          date: '2024-06-01T00:00:00Z',
          label: 'Version 1.0 Release',
          type: 'release'
        }
      ]
    };
    
    render(<HistoricalComparison {...propsWithMilestones} />);
    
    await waitForChartRender();
    
    expect(screen.getByText('Version 1.0 Release')).toBeInTheDocument();
  });

  it('handles detailed comparison view', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const detailedButton = screen.getByRole('button', { name: /detailed comparison/i });
    fireEvent.click(detailedButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Side-by-side Comparison/)).toBeInTheDocument();
      expect(screen.getByText(/Before/)).toBeInTheDocument();
      expect(screen.getByText(/After/)).toBeInTheDocument();
    });
  });

  it('shows export timeline functionality', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const exportButton = screen.getByRole('button', { name: /export timeline/i });
    fireEvent.click(exportButton);
    
    await waitFor(() => {
      expect(screen.getByText(/PNG/)).toBeInTheDocument();
      expect(screen.getByText(/CSV/)).toBeInTheDocument();
    });
  });

  it('displays velocity metrics', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    expect(screen.getByText(/Development Velocity/)).toBeInTheDocument();
    expect(screen.getByText(/issues per week/)).toBeInTheDocument();
  });

  it('handles empty historical data', () => {
    render(<HistoricalComparison {...mockProps} historicalData={[]} />);
    
    expect(screen.getByText('Historical Analysis')).toBeInTheDocument();
    expect(screen.getByText(/No historical data available/)).toBeInTheDocument();
  });

  it('shows correlation analysis', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const correlationTab = screen.getByRole('tab', { name: /correlations/i });
    fireEvent.click(correlationTab);
    
    await waitFor(() => {
      expect(screen.getByText(/Quality Score vs Test Coverage/)).toBeInTheDocument();
      expect(screen.getByText(/Correlation coefficient/)).toBeInTheDocument();
    });
  });

  it('displays benchmark comparisons', async () => {
    const propsWithBenchmarks = {
      ...mockProps,
      benchmarkData: {
        industry_average: { qualityScore: 75, technicalDebt: 20000 },
        similar_projects: { qualityScore: 80, technicalDebt: 15000 }
      }
    };
    
    render(<HistoricalComparison {...propsWithBenchmarks} />);
    
    await waitFor(() => {
      expect(screen.getByText(/vs Industry Average/)).toBeInTheDocument();
      expect(screen.getByText(/vs Similar Projects/)).toBeInTheDocument();
    });
  });

  it('handles chart zoom and pan', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    await waitForChartRender();
    
    const chartContainer = screen.getByTestId('timeline-chart');
    
    // Simulate mouse wheel zoom
    fireEvent.wheel(chartContainer, { deltaY: -100 });
    
    // Chart should handle zoom interaction
    expect(chartContainer).toBeInTheDocument();
  });

  it('shows snapshot comparison diff', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const diffButton = screen.getByRole('button', { name: /show diff/i });
    fireEvent.click(diffButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Added Issues/)).toBeInTheDocument();
      expect(screen.getByText(/Resolved Issues/)).toBeInTheDocument();
      expect(screen.getByText(/Modified Issues/)).toBeInTheDocument();
    });
  });

  it('has proper accessibility attributes', () => {
    render(<HistoricalComparison {...mockProps} />);
    
    const timeline = screen.getByRole('img', { name: /timeline chart/i });
    expect(timeline).toHaveAttribute('aria-label');
    
    const tabList = screen.getByRole('tablist');
    expect(tabList).toHaveAttribute('aria-label');
  });

  it('maintains chart responsive behavior', async () => {
    render(<HistoricalComparison {...mockProps} />);
    
    await waitForChartRender();
    
    // Simulate window resize
    global.dispatchEvent(new Event('resize'));
    
    // Chart should maintain responsiveness
    expect(screen.getByText(/Quality Score Timeline/)).toBeInTheDocument();
  });
});