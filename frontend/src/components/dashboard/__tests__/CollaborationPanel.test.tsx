import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render } from '../../../test-utils/test-utils';
import { CollaborationPanel } from '../CollaborationPanel';
import { mockTeamMembers } from '../../../test-utils/mocks/mockData';

describe('CollaborationPanel', () => {
  const mockProps = {
    teamMembers: mockTeamMembers,
    discussions: [
      {
        id: 'disc1',
        issueId: 'issue1',
        title: 'Refactoring UserService',
        participants: ['alice', 'bob'],
        messageCount: 5,
        lastActivity: '2024-01-15T10:30:00Z',
        status: 'active'
      }
    ],
    codeReviews: [
      {
        id: 'review1',
        issueId: 'issue2',
        title: 'Fix circular dependency',
        reviewer: 'carol',
        status: 'pending',
        createdAt: '2024-01-14T15:20:00Z',
        priority: 'high'
      }
    ],
    onStartDiscussion: jest.fn(),
    onRequestReview: jest.fn(),
    onAssignIssue: jest.fn(),
    onAddComment: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders collaboration panel', () => {
    render(<CollaborationPanel {...mockProps} />);
    
    expect(screen.getByText('Team Collaboration')).toBeInTheDocument();
    expect(screen.getByText('Discussions, Reviews & Assignments')).toBeInTheDocument();
  });

  it('displays team members', () => {
    render(<CollaborationPanel {...mockProps} />);
    
    expect(screen.getByText('Alice Johnson')).toBeInTheDocument();
    expect(screen.getByText('Senior Developer')).toBeInTheDocument();
    expect(screen.getByText('Bob Smith')).toBeInTheDocument();
    expect(screen.getByText('Technical Lead')).toBeInTheDocument();
    expect(screen.getByText('Carol Davis')).toBeInTheDocument();
  });

  it('shows active discussions', () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const discussionsTab = screen.getByRole('tab', { name: /discussions/i });
    fireEvent.click(discussionsTab);
    
    expect(screen.getByText('Refactoring UserService')).toBeInTheDocument();
    expect(screen.getByText('5 messages')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('displays pending code reviews', async () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const reviewsTab = screen.getByRole('tab', { name: /reviews/i });
    fireEvent.click(reviewsTab);
    
    await waitFor(() => {
      expect(screen.getByText('Fix circular dependency')).toBeInTheDocument();
      expect(screen.getByText('Pending')).toBeInTheDocument();
      expect(screen.getByText('High Priority')).toBeInTheDocument();
    });
  });

  it('handles starting new discussion', async () => {
    const user = userEvent.setup();
    render(<CollaborationPanel {...mockProps} />);
    
    const startDiscussionButton = screen.getByRole('button', { name: /start discussion/i });
    fireEvent.click(startDiscussionButton);
    
    const titleInput = screen.getByPlaceholderText(/Discussion title/);
    await user.type(titleInput, 'New Code Review Discussion');
    
    const messageInput = screen.getByPlaceholderText(/Your message/);
    await user.type(messageInput, 'Let\'s discuss this refactoring approach');
    
    const createButton = screen.getByRole('button', { name: /create discussion/i });
    fireEvent.click(createButton);
    
    await waitFor(() => {
      expect(mockProps.onStartDiscussion).toHaveBeenCalledWith(
        'New Code Review Discussion',
        'Let\'s discuss this refactoring approach'
      );
    });
  });

  it('handles requesting code review', async () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const requestReviewButton = screen.getByRole('button', { name: /request review/i });
    fireEvent.click(requestReviewButton);
    
    // Select reviewer
    const reviewerSelect = screen.getByRole('combobox', { name: /select reviewer/i });
    fireEvent.mouseDown(reviewerSelect);
    
    const reviewer = screen.getByText('Carol Davis');
    fireEvent.click(reviewer);
    
    const submitButton = screen.getByRole('button', { name: /submit request/i });
    fireEvent.click(submitButton);
    
    await waitFor(() => {
      expect(mockProps.onRequestReview).toHaveBeenCalledWith(
        expect.objectContaining({ reviewer: 'carol' })
      );
    });
  });

  it('shows team member availability status', () => {
    const propsWithStatus = {
      ...mockProps,
      teamMembers: mockTeamMembers.map(member => ({
        ...member,
        status: member.id === '1' ? 'online' : 'offline',
        lastSeen: member.id === '1' ? null : '2 hours ago'
      }))
    };
    
    render(<CollaborationPanel {...propsWithStatus} />);
    
    expect(screen.getByText(/Online/)).toBeInTheDocument();
    expect(screen.getByText(/2 hours ago/)).toBeInTheDocument();
  });

  it('handles issue assignment', async () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const assignButton = screen.getByRole('button', { name: /assign issue/i });
    fireEvent.click(assignButton);
    
    const assigneeSelect = screen.getByRole('combobox', { name: /select assignee/i });
    fireEvent.mouseDown(assigneeSelect);
    
    const assignee = screen.getByText('Alice Johnson');
    fireEvent.click(assignee);
    
    const confirmButton = screen.getByRole('button', { name: /assign/i });
    fireEvent.click(confirmButton);
    
    await waitFor(() => {
      expect(mockProps.onAssignIssue).toHaveBeenCalledWith(
        expect.any(String),
        '1'
      );
    });
  });

  it('displays discussion messages', async () => {
    const propsWithMessages = {
      ...mockProps,
      discussions: [{
        ...mockProps.discussions[0],
        messages: [
          {
            id: 'msg1',
            author: 'alice',
            content: 'I think we should extract this method',
            timestamp: '2024-01-15T10:30:00Z'
          },
          {
            id: 'msg2',
            author: 'bob',
            content: 'Agreed, it\'s getting too complex',
            timestamp: '2024-01-15T10:35:00Z'
          }
        ]
      }]
    };
    
    render(<CollaborationPanel {...propsWithMessages} />);
    
    const discussionItem = screen.getByText('Refactoring UserService');
    fireEvent.click(discussionItem);
    
    await waitFor(() => {
      expect(screen.getByText('I think we should extract this method')).toBeInTheDocument();
      expect(screen.getByText('Agreed, it\'s getting too complex')).toBeInTheDocument();
    });
  });

  it('handles adding comments to discussions', async () => {
    const user = userEvent.setup();
    render(<CollaborationPanel {...mockProps} />);
    
    const discussionItem = screen.getByText('Refactoring UserService');
    fireEvent.click(discussionItem);
    
    const commentInput = screen.getByPlaceholderText(/Add a comment/);
    await user.type(commentInput, 'This looks good to me!');
    
    const addCommentButton = screen.getByRole('button', { name: /add comment/i });
    fireEvent.click(addCommentButton);
    
    await waitFor(() => {
      expect(mockProps.onAddComment).toHaveBeenCalledWith(
        'disc1',
        'This looks good to me!'
      );
    });
  });

  it('shows review status and priority indicators', async () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const reviewsTab = screen.getByRole('tab', { name: /reviews/i });
    fireEvent.click(reviewsTab);
    
    await waitFor(() => {
      const pendingBadge = screen.getByText('Pending');
      expect(pendingBadge).toHaveClass('MuiChip-colorWarning');
      
      const highPriorityBadge = screen.getByText('High Priority');
      expect(highPriorityBadge).toHaveClass('MuiChip-colorError');
    });
  });

  it('displays team member workload', () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const workloadTab = screen.getByRole('tab', { name: /workload/i });
    fireEvent.click(workloadTab);
    
    expect(screen.getByText(/Assigned Issues/)).toBeInTheDocument();
    expect(screen.getByText(/Review Queue/)).toBeInTheDocument();
  });

  it('handles real-time updates', async () => {
    const { rerender } = render(<CollaborationPanel {...mockProps} />);
    
    const updatedProps = {
      ...mockProps,
      discussions: [
        ...mockProps.discussions,
        {
          id: 'disc2',
          issueId: 'issue3',
          title: 'New Discussion',
          participants: ['carol'],
          messageCount: 1,
          lastActivity: '2024-01-15T11:00:00Z',
          status: 'active'
        }
      ]
    };
    
    rerender(<CollaborationPanel {...updatedProps} />);
    
    await waitFor(() => {
      expect(screen.getByText('New Discussion')).toBeInTheDocument();
    });
  });

  it('shows notification badges for new activity', () => {
    const propsWithNotifications = {
      ...mockProps,
      notifications: {
        newDiscussions: 2,
        pendingReviews: 1,
        mentions: 3
      }
    };
    
    render(<CollaborationPanel {...propsWithNotifications} />);
    
    const discussionsTab = screen.getByRole('tab', { name: /discussions/i });
    expect(discussionsTab).toHaveTextContent('2'); // Notification badge
    
    const reviewsTab = screen.getByRole('tab', { name: /reviews/i });
    expect(reviewsTab).toHaveTextContent('1'); // Notification badge
  });

  it('filters discussions and reviews by status', async () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const statusFilter = screen.getByRole('combobox', { name: /filter by status/i });
    fireEvent.mouseDown(statusFilter);
    
    const activeOption = screen.getByText('Active Only');
    fireEvent.click(activeOption);
    
    await waitFor(() => {
      // Should only show active discussions
      expect(screen.getByText('Refactoring UserService')).toBeInTheDocument();
    });
  });

  it('has proper accessibility attributes', () => {
    render(<CollaborationPanel {...mockProps} />);
    
    const tabList = screen.getByRole('tablist');
    expect(tabList).toHaveAttribute('aria-label');
    
    const tabs = screen.getAllByRole('tab');
    tabs.forEach(tab => {
      expect(tab).toHaveAttribute('aria-controls');
      expect(tab).toHaveAttribute('id');
    });
    
    const teamMemberList = screen.getByRole('list');
    expect(teamMemberList).toHaveAttribute('aria-label');
  });

  it('handles empty states gracefully', () => {
    const emptyProps = {
      ...mockProps,
      teamMembers: [],
      discussions: [],
      codeReviews: []
    };
    
    render(<CollaborationPanel {...emptyProps} />);
    
    expect(screen.getByText(/No team members/)).toBeInTheDocument();
    expect(screen.getByText(/No active discussions/)).toBeInTheDocument();
  });

  it('shows timestamps in relative format', () => {
    render(<CollaborationPanel {...mockProps} />);
    
    // Should show relative times like "2 hours ago"
    expect(screen.getByText(/ago/)).toBeInTheDocument();
  });
});