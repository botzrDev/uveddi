import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render } from '../../../test-utils/test-utils';
import { FixSuggestionPanel } from '../FixSuggestionPanel';
import { mockIssues } from '../../../test-utils/mocks/mockData';

describe('FixSuggestionPanel', () => {
  const mockProps = {
    selectedIssue: mockIssues[0],
    onApplyFix: jest.fn(),
    onDismissIssue: jest.fn(),
    onRequestAlternatives: jest.fn(),
    isLoading: false,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders fix suggestion panel with issue details', () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    expect(screen.getByText('Fix Suggestions')).toBeInTheDocument();
    expect(screen.getByText('Large Method in UserService')).toBeInTheDocument();
    expect(screen.getByText(/Method getUserProfile has 150 lines/)).toBeInTheDocument();
  });

  it('displays fix suggestion steps', () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    expect(screen.getByText('Suggested Fix')).toBeInTheDocument();
    expect(screen.getByText('Extract method into smaller, focused functions')).toBeInTheDocument();
  });

  it('shows issue metadata', () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    expect(screen.getByText('High')).toBeInTheDocument(); // Severity
    expect(screen.getByText('Code Smell')).toBeInTheDocument(); // Category
    expect(screen.getByText('src/services/UserService.ts:45')).toBeInTheDocument(); // Location
  });

  it('displays impact and effort scores', () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    expect(screen.getByText(/Impact:/)).toBeInTheDocument();
    expect(screen.getByText('8')).toBeInTheDocument();
    expect(screen.getByText(/Effort:/)).toBeInTheDocument();
    expect(screen.getByText('6')).toBeInTheDocument();
  });

  it('shows confidence score', () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    expect(screen.getByText(/Confidence:/)).toBeInTheDocument();
    expect(screen.getByText('90%')).toBeInTheDocument();
  });

  it('handles apply fix action', async () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    const applyButton = screen.getByRole('button', { name: /apply fix/i });
    fireEvent.click(applyButton);
    
    await waitFor(() => {
      expect(mockProps.onApplyFix).toHaveBeenCalledWith(mockIssues[0]);
    });
  });

  it('handles dismiss issue action', async () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    const dismissButton = screen.getByRole('button', { name: /dismiss/i });
    fireEvent.click(dismissButton);
    
    await waitFor(() => {
      expect(mockProps.onDismissIssue).toHaveBeenCalledWith(mockIssues[0].id);
    });
  });

  it('requests alternative solutions', async () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    const alternativesButton = screen.getByRole('button', { name: /show alternatives/i });
    fireEvent.click(alternativesButton);
    
    await waitFor(() => {
      expect(mockProps.onRequestAlternatives).toHaveBeenCalledWith(mockIssues[0].id);
    });
  });

  it('displays loading state', () => {
    render(<FixSuggestionPanel {...mockProps} isLoading={true} />);
    
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
    expect(screen.getByText(/Loading fix suggestions/)).toBeInTheDocument();
  });

  it('shows code preview when available', async () => {
    const issueWithCode = {
      ...mockIssues[0],
      code_preview: {
        before: 'function largeMethod() { /* 150 lines of code */ }',
        after: 'function extractedMethod() { /* refactored code */ }',
        language: 'typescript'
      }
    };
    
    render(<FixSuggestionPanel {...mockProps} selectedIssue={issueWithCode} />);
    
    const codeTab = screen.getByRole('tab', { name: /code preview/i });
    fireEvent.click(codeTab);
    
    await waitFor(() => {
      expect(screen.getByText(/Before/)).toBeInTheDocument();
      expect(screen.getByText(/After/)).toBeInTheDocument();
    });
  });

  it('displays step-by-step fix instructions', () => {
    const issueWithSteps = {
      ...mockIssues[0],
      fix_steps: [
        'Identify the core responsibility of the method',
        'Extract helper methods for secondary concerns',
        'Update method signature and callers',
        'Add unit tests for new methods'
      ]
    };
    
    render(<FixSuggestionPanel {...mockProps} selectedIssue={issueWithSteps} />);
    
    expect(screen.getByText('Step 1')).toBeInTheDocument();
    expect(screen.getByText('Identify the core responsibility')).toBeInTheDocument();
    expect(screen.getByText('Step 4')).toBeInTheDocument();
  });

  it('handles no selected issue', () => {
    render(<FixSuggestionPanel {...mockProps} selectedIssue={null} />);
    
    expect(screen.getByText('No Issue Selected')).toBeInTheDocument();
    expect(screen.getByText(/Select an issue to view fix suggestions/)).toBeInTheDocument();
  });

  it('shows alternative fix suggestions', async () => {
    const issueWithAlternatives = {
      ...mockIssues[0],
      alternative_fixes: [
        {
          id: 'alt1',
          title: 'Refactor using Strategy Pattern',
          description: 'Apply strategy pattern to reduce complexity',
          difficulty: 'hard'
        },
        {
          id: 'alt2', 
          title: 'Split into multiple classes',
          description: 'Break the class into smaller, focused classes',
          difficulty: 'medium'
        }
      ]
    };
    
    render(<FixSuggestionPanel {...mockProps} selectedIssue={issueWithAlternatives} />);
    
    const alternativesTab = screen.getByRole('tab', { name: /alternatives/i });
    fireEvent.click(alternativesTab);
    
    await waitFor(() => {
      expect(screen.getByText('Refactor using Strategy Pattern')).toBeInTheDocument();
      expect(screen.getByText('Split into multiple classes')).toBeInTheDocument();
    });
  });

  it('displays estimated fix time', () => {
    const issueWithTime = {
      ...mockIssues[0],
      estimated_fix_time: '2-4 hours'
    };
    
    render(<FixSuggestionPanel {...mockProps} selectedIssue={issueWithTime} />);
    
    expect(screen.getByText(/Estimated Time:/)).toBeInTheDocument();
    expect(screen.getByText('2-4 hours')).toBeInTheDocument();
  });

  it('shows related issues', () => {
    const issueWithRelated = {
      ...mockIssues[0],
      related_issues: [mockIssues[1].id, mockIssues[2].id]
    };
    
    render(<FixSuggestionPanel {...mockProps} selectedIssue={issueWithRelated} />);
    
    expect(screen.getByText(/Related Issues/)).toBeInTheDocument();
    expect(screen.getByText('2 related issues')).toBeInTheDocument();
  });

  it('handles copy to clipboard functionality', async () => {
    // Mock clipboard API
    Object.assign(navigator, {
      clipboard: {
        writeText: jest.fn().mockImplementation(() => Promise.resolve()),
      },
    });
    
    render(<FixSuggestionPanel {...mockProps} />);
    
    const copyButton = screen.getByRole('button', { name: /copy/i });
    fireEvent.click(copyButton);
    
    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalled();
    });
  });

  it('has proper accessibility attributes', () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    const panel = screen.getByRole('tabpanel');
    expect(panel).toHaveAttribute('aria-labelledby');
    
    const tabs = screen.getAllByRole('tab');
    tabs.forEach(tab => {
      expect(tab).toHaveAttribute('aria-controls');
    });
  });

  it('maintains tab navigation state', async () => {
    render(<FixSuggestionPanel {...mockProps} />);
    
    const detailsTab = screen.getByRole('tab', { name: /details/i });
    fireEvent.click(detailsTab);
    
    await waitFor(() => {
      expect(detailsTab).toHaveAttribute('aria-selected', 'true');
    });
  });

  it('shows warning for high-risk fixes', () => {
    const highRiskIssue = {
      ...mockIssues[0],
      fix_risk: 'high',
      risk_factors: ['Affects multiple files', 'May break existing tests']
    };
    
    render(<FixSuggestionPanel {...mockProps} selectedIssue={highRiskIssue} />);
    
    expect(screen.getByText(/High Risk/)).toBeInTheDocument();
    expect(screen.getByText('Affects multiple files')).toBeInTheDocument();
  });
});