import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render } from '../../../test-utils/test-utils';
import { QualityScoreCard } from '../QualityScoreCard';
import { mockQualityMetrics, mockHistoricalData } from '../../../test-utils/mocks/mockData';

describe('QualityScoreCard', () => {
  const mockProps = {
    qualityMetrics: mockQualityMetrics,
    historicalData: mockHistoricalData.snapshots,
    onDrillDown: jest.fn(),
    onBenchmarkCompare: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders quality score card with overall score', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    expect(screen.getByText('Quality Score')).toBeInTheDocument();
    expect(screen.getByText('87')).toBeInTheDocument(); // Overall score
    expect(screen.getByText('Overall Quality')).toBeInTheDocument();
  });

  it('displays all quality metrics', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    expect(screen.getByText('Maintainability')).toBeInTheDocument();
    expect(screen.getByText('85')).toBeInTheDocument();
    expect(screen.getByText('Reliability')).toBeInTheDocument();
    expect(screen.getByText('90')).toBeInTheDocument();
    expect(screen.getByText('Security')).toBeInTheDocument();
    expect(screen.getByText('82')).toBeInTheDocument();
    expect(screen.getByText('Performance')).toBeInTheDocument();
    expect(screen.getByText('88')).toBeInTheDocument();
  });

  it('shows trend indicators', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    // Look for trend indicators (arrows or trend text)
    const trendElements = screen.getAllByTestId(/trend-indicator/);
    expect(trendElements.length).toBeGreaterThan(0);
  });

  it('displays test coverage with progress bar', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    expect(screen.getByText('Test Coverage')).toBeInTheDocument();
    expect(screen.getByText('78%')).toBeInTheDocument();
    
    const progressBar = screen.getByRole('progressbar');
    expect(progressBar).toHaveAttribute('aria-valuenow', '78');
  });

  it('shows complexity and debt metrics', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    expect(screen.getByText(/Complexity Score/)).toBeInTheDocument();
    expect(screen.getByText('6.2')).toBeInTheDocument();
    expect(screen.getByText(/Debt Ratio/)).toBeInTheDocument();
    expect(screen.getByText('12%')).toBeInTheDocument();
  });

  it('handles drill down action on metric click', async () => {
    render(<QualityScoreCard {...mockProps} />);
    
    const maintainabilityButton = screen.getByRole('button', { name: /maintainability/i });
    fireEvent.click(maintainabilityButton);
    
    await waitFor(() => {
      expect(mockProps.onDrillDown).toHaveBeenCalledWith('maintainability');
    });
  });

  it('shows benchmark comparison when available', () => {
    const propsWithBenchmark = {
      ...mockProps,
      benchmarkData: {
        industry_average: 80,
        similar_projects: 85,
        top_percentile: 95
      }
    };
    
    render(<QualityScoreCard {...propsWithBenchmark} />);
    
    expect(screen.getByText(/Industry Average/)).toBeInTheDocument();
    expect(screen.getByText('80')).toBeInTheDocument();
  });

  it('displays score change indicators', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    // Check for score change indicators (up/down arrows, colors)
    const changeIndicators = screen.getAllByTestId(/score-change/);
    expect(changeIndicators.length).toBeGreaterThan(0);
  });

  it('handles missing historical data gracefully', () => {
    const propsWithoutHistory = {
      ...mockProps,
      historicalData: []
    };
    
    render(<QualityScoreCard {...propsWithoutHistory} />);
    
    expect(screen.getByText('Quality Score')).toBeInTheDocument();
    expect(screen.getByText('87')).toBeInTheDocument();
  });

  it('shows detailed metrics in expanded view', async () => {
    render(<QualityScoreCard {...mockProps} />);
    
    const expandButton = screen.getByRole('button', { name: /show details/i });
    fireEvent.click(expandButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Duplication/)).toBeInTheDocument();
      expect(screen.getByText('3.5%')).toBeInTheDocument();
    });
  });

  it('renders circular progress indicators correctly', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    const progressElements = screen.getAllByRole('progressbar');
    expect(progressElements.length).toBeGreaterThan(1);
    
    progressElements.forEach(progress => {
      expect(progress).toHaveAttribute('aria-valuenow');
      expect(progress).toHaveAttribute('aria-valuemin', '0');
      expect(progress).toHaveAttribute('aria-valuemax', '100');
    });
  });

  it('applies correct color coding based on score ranges', () => {
    const lowScoreMetrics = {
      ...mockQualityMetrics,
      overall_score: 45,
      maintainability: 40,
      security: 35
    };
    
    render(<QualityScoreCard {...mockProps} qualityMetrics={lowScoreMetrics} />);
    
    expect(screen.getByText('45')).toBeInTheDocument();
    // Check for error/warning styling on low scores
  });

  it('handles benchmark comparison action', async () => {
    render(<QualityScoreCard {...mockProps} />);
    
    const benchmarkButton = screen.getByRole('button', { name: /compare with benchmark/i });
    fireEvent.click(benchmarkButton);
    
    await waitFor(() => {
      expect(mockProps.onBenchmarkCompare).toHaveBeenCalledWith(mockQualityMetrics);
    });
  });

  it('has proper accessibility labels', () => {
    render(<QualityScoreCard {...mockProps} />);
    
    const overallScoreElement = screen.getByLabelText(/overall quality score/i);
    expect(overallScoreElement).toBeInTheDocument();
    
    const maintainabilityScore = screen.getByLabelText(/maintainability score/i);
    expect(maintainabilityScore).toBeInTheDocument();
  });
});