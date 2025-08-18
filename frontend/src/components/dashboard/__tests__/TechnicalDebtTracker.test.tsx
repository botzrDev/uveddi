import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render, waitForChartRender } from '../../../test-utils/test-utils';
import { TechnicalDebtTracker } from '../TechnicalDebtTracker';
import { mockInteractiveReport } from '../../../test-utils/mocks/mockData';

describe('TechnicalDebtTracker', () => {
  const mockProps = {
    technicalDebt: mockInteractiveReport.technical_debt,
    onDebtItemClick: jest.fn(),
    onCategoryFilter: jest.fn(),
    onTimeRangeChange: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders technical debt tracker component', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    expect(screen.getByText('Technical Debt Tracker')).toBeInTheDocument();
    expect(screen.getByText('Debt Analysis & Trends')).toBeInTheDocument();
  });

  it('displays total debt hours', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    expect(screen.getByText('48 hours')).toBeInTheDocument();
    expect(screen.getByText(/Total Technical Debt/)).toBeInTheDocument();
  });

  it('shows debt breakdown by category', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    expect(screen.getByText('Code Smell')).toBeInTheDocument();
    expect(screen.getByText('20 hours')).toBeInTheDocument();
    expect(screen.getByText('Architecture')).toBeInTheDocument();
    expect(screen.getByText('15 hours')).toBeInTheDocument();
    expect(screen.getByText('Security')).toBeInTheDocument();
    expect(screen.getByText('8 hours')).toBeInTheDocument();
  });

  it('renders tab navigation', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    expect(screen.getByRole('tab', { name: /overview/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /trends/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /priority items/i })).toBeInTheDocument();
  });

  it('switches between tabs', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const trendsTab = screen.getByRole('tab', { name: /trends/i });
    fireEvent.click(trendsTab);
    
    await waitFor(() => {
      expect(screen.getByText(/Debt Trend Over Time/)).toBeInTheDocument();
    });
  });

  it('displays trend chart in trends tab', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const trendsTab = screen.getByRole('tab', { name: /trends/i });
    fireEvent.click(trendsTab);
    
    await waitForChartRender();
    
    expect(screen.getByText(/Debt Trend Over Time/)).toBeInTheDocument();
  });

  it('shows priority items with ROI scores', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const priorityTab = screen.getByRole('tab', { name: /priority items/i });
    fireEvent.click(priorityTab);
    
    await waitFor(() => {
      expect(screen.getByText(/High Priority Debt Items/)).toBeInTheDocument();
      expect(screen.getByText(/ROI Score/)).toBeInTheDocument();
    });
  });

  it('handles category filter changes', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const codeSmellChip = screen.getByText('Code Smell');
    fireEvent.click(codeSmellChip);
    
    await waitFor(() => {
      expect(mockProps.onCategoryFilter).toHaveBeenCalledWith('code-smell');
    });
  });

  it('displays time range selector', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    expect(screen.getByRole('button', { name: /last 3 months/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /last 6 months/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /last year/i })).toBeInTheDocument();
  });

  it('handles time range changes', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const sixMonthsButton = screen.getByRole('button', { name: /last 6 months/i });
    fireEvent.click(sixMonthsButton);
    
    await waitFor(() => {
      expect(mockProps.onTimeRangeChange).toHaveBeenCalledWith('6m');
    });
  });

  it('shows debt trend indicators', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    // Look for trend indicators (arrows or percentages)
    const trendIndicators = screen.getAllByTestId(/trend-indicator/);
    expect(trendIndicators.length).toBeGreaterThan(0);
  });

  it('handles debt item clicks', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const priorityTab = screen.getByRole('tab', { name: /priority items/i });
    fireEvent.click(priorityTab);
    
    await waitFor(() => {
      const debtItem = screen.getByText(/issue-1/);
      fireEvent.click(debtItem);
      
      expect(mockProps.onDebtItemClick).toHaveBeenCalled();
    });
  });

  it('displays progress bars for category breakdown', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const progressBars = screen.getAllByRole('progressbar');
    expect(progressBars.length).toBeGreaterThan(0);
    
    progressBars.forEach(bar => {
      expect(bar).toHaveAttribute('aria-valuemin', '0');
      expect(bar).toHaveAttribute('aria-valuemax', '100');
    });
  });

  it('shows category percentages', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    // Calculate expected percentages
    const total = 48; // Total debt hours
    const codeSmellPercentage = Math.round((20 / total) * 100);
    
    expect(screen.getByText(`${codeSmellPercentage}%`)).toBeInTheDocument();
  });

  it('handles empty debt data gracefully', () => {
    const emptyDebtProps = {
      ...mockProps,
      technicalDebt: {
        total_debt_hours: 0,
        debt_by_category: {},
        debt_trend: [],
        priority_items: []
      }
    };
    
    render(<TechnicalDebtTracker {...emptyDebtProps} />);
    
    expect(screen.getByText('Technical Debt Tracker')).toBeInTheDocument();
    expect(screen.getByText('0 hours')).toBeInTheDocument();
  });

  it('displays debt velocity metrics', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const trendsTab = screen.getByRole('tab', { name: /trends/i });
    fireEvent.click(trendsTab);
    
    await waitFor(() => {
      expect(screen.getByText(/Debt Velocity/)).toBeInTheDocument();
      expect(screen.getByText(/hours per week/)).toBeInTheDocument();
    });
  });

  it('shows actionable recommendations', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const priorityTab = screen.getByRole('tab', { name: /priority items/i });
    fireEvent.click(priorityTab);
    
    await waitFor(() => {
      expect(screen.getByText(/Recommended Actions/)).toBeInTheDocument();
    });
  });

  it('has proper accessibility attributes', () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const tabList = screen.getByRole('tablist');
    expect(tabList).toHaveAttribute('aria-label');
    
    const tabs = screen.getAllByRole('tab');
    tabs.forEach(tab => {
      expect(tab).toHaveAttribute('aria-controls');
      expect(tab).toHaveAttribute('id');
    });
  });

  it('maintains selected tab state', async () => {
    render(<TechnicalDebtTracker {...mockProps} />);
    
    const trendsTab = screen.getByRole('tab', { name: /trends/i });
    fireEvent.click(trendsTab);
    
    await waitFor(() => {
      expect(trendsTab).toHaveAttribute('aria-selected', 'true');
    });
  });
});