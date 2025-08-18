import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render } from '../../../test-utils/test-utils';
import { SmartSearchPanel } from '../SmartSearchPanel';
import { mockIssues } from '../../../test-utils/mocks/mockData';

describe('SmartSearchPanel', () => {
  const mockProps = {
    issues: mockIssues,
    onSearchResults: jest.fn(),
    onSaveSearch: jest.fn(),
    onLoadSavedSearch: jest.fn(),
    savedSearches: [
      {
        id: 'search1',
        name: 'High Impact Issues',
        query: 'impact:high',
        filters: { severity: ['high', 'critical'] }
      }
    ],
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders smart search panel', () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    expect(screen.getByText('Smart Search')).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Search issues, files, or descriptions/)).toBeInTheDocument();
  });

  it('handles basic text search', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'UserService');
    
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({ file: 'src/services/UserService.ts' })
        ])
      );
    });
  });

  it('displays search suggestions', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'large');
    
    await waitFor(() => {
      expect(screen.getByText(/Large Method/)).toBeInTheDocument();
    });
  });

  it('shows severity filter options', () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    expect(screen.getByText('Critical')).toBeInTheDocument();
    expect(screen.getByText('High')).toBeInTheDocument();
    expect(screen.getByText('Medium')).toBeInTheDocument();
    expect(screen.getByText('Low')).toBeInTheDocument();
  });

  it('filters by severity', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const criticalChip = screen.getByText('Critical');
    fireEvent.click(criticalChip);
    
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalledWith(
        mockIssues.filter(issue => issue.severity === 'critical')
      );
    });
  });

  it('shows category filter options', () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    expect(screen.getByText('Code Smell')).toBeInTheDocument();
    expect(screen.getByText('Architecture')).toBeInTheDocument();
    expect(screen.getByText('Security')).toBeInTheDocument();
    expect(screen.getByText('Performance')).toBeInTheDocument();
  });

  it('applies multiple filters simultaneously', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const highChip = screen.getByText('High');
    const codeSmellChip = screen.getByText('Code Smell');
    
    fireEvent.click(highChip);
    fireEvent.click(codeSmellChip);
    
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalledWith(
        mockIssues.filter(issue => 
          issue.severity === 'high' && issue.category === 'code-smell'
        )
      );
    });
  });

  it('displays search results count', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'method');
    
    await waitFor(() => {
      expect(screen.getByText(/1 result/)).toBeInTheDocument();
    });
  });

  it('shows advanced search options', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const advancedButton = screen.getByRole('button', { name: /advanced/i });
    fireEvent.click(advancedButton);
    
    await waitFor(() => {
      expect(screen.getByText(/File Path/)).toBeInTheDocument();
      expect(screen.getByText(/Date Range/)).toBeInTheDocument();
      expect(screen.getByText(/Impact Range/)).toBeInTheDocument();
    });
  });

  it('handles semantic search queries', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'complex methods that are hard to maintain');
    
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({ title: 'Large Method in UserService' })
        ])
      );
    });
  });

  it('saves search queries', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'high impact issues');
    
    const saveButton = screen.getByRole('button', { name: /save search/i });
    fireEvent.click(saveButton);
    
    // Enter search name
    const nameInput = screen.getByPlaceholderText(/Search name/);
    await user.type(nameInput, 'My High Impact Search');
    
    const confirmSaveButton = screen.getByRole('button', { name: /save/i });
    fireEvent.click(confirmSaveButton);
    
    await waitFor(() => {
      expect(mockProps.onSaveSearch).toHaveBeenCalledWith(
        'My High Impact Search',
        'high impact issues',
        expect.any(Object)
      );
    });
  });

  it('loads saved searches', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const savedSearchItem = screen.getByText('High Impact Issues');
    fireEvent.click(savedSearchItem);
    
    await waitFor(() => {
      expect(mockProps.onLoadSavedSearch).toHaveBeenCalledWith('search1');
    });
  });

  it('deletes saved searches', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const deleteButton = screen.getByRole('button', { name: /delete search/i });
    fireEvent.click(deleteButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Are you sure/)).toBeInTheDocument();
    });
    
    const confirmDelete = screen.getByRole('button', { name: /confirm/i });
    fireEvent.click(confirmDelete);
  });

  it('shows search history', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const historyButton = screen.getByRole('button', { name: /history/i });
    fireEvent.click(historyButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Recent Searches/)).toBeInTheDocument();
    });
  });

  it('applies quick search templates', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const templatesButton = screen.getByRole('button', { name: /templates/i });
    fireEvent.click(templatesButton);
    
    const securityTemplate = screen.getByText('Security Issues');
    fireEvent.click(securityTemplate);
    
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalledWith(
        mockIssues.filter(issue => issue.category === 'security')
      );
    });
  });

  it('handles empty search gracefully', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'nonexistent issue');
    
    await waitFor(() => {
      expect(screen.getByText(/No results found/)).toBeInTheDocument();
    });
  });

  it('shows search syntax help', async () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const helpButton = screen.getByRole('button', { name: /help/i });
    fireEvent.click(helpButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Search Syntax/)).toBeInTheDocument();
      expect(screen.getByText(/severity:high/)).toBeInTheDocument();
      expect(screen.getByText(/file:*.ts/)).toBeInTheDocument();
    });
  });

  it('supports keyboard navigation', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    await user.type(searchInput, 'method');
    
    // Navigate through suggestions with arrow keys
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{Enter}');
    
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalled();
    });
  });

  it('handles real-time search updates', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    
    // Type characters one by one
    await user.type(searchInput, 'u');
    await user.type(searchInput, 's');
    await user.type(searchInput, 'e');
    
    // Should update results as user types
    await waitFor(() => {
      expect(mockProps.onSearchResults).toHaveBeenCalled();
    });
  });

  it('has proper accessibility attributes', () => {
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByRole('combobox');
    expect(searchInput).toHaveAttribute('aria-label');
    expect(searchInput).toHaveAttribute('aria-describedby');
    
    const filterGroup = screen.getByRole('group', { name: /severity filters/i });
    expect(filterGroup).toBeInTheDocument();
  });

  it('maintains focus management', async () => {
    const user = userEvent.setup();
    render(<SmartSearchPanel {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search issues, files, or descriptions/);
    
    await user.tab();
    expect(searchInput).toHaveFocus();
    
    await user.tab();
    expect(screen.getByText('Critical')).toHaveFocus();
  });
});