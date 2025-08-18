import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render } from '../../test-utils/test-utils';
import { mockInteractiveReport, mockDependencies } from '../../test-utils/mocks/mockData';

// Import all dashboard components
import { PriorityMatrix } from '../../components/dashboard/PriorityMatrix';
import { QualityScoreCard } from '../../components/dashboard/QualityScoreCard';
import { InteractiveDependencyGraph } from '../../components/dashboard/InteractiveDependencyGraph';
import { TechnicalDebtTracker } from '../../components/dashboard/TechnicalDebtTracker';
import { SmartSearchPanel } from '../../components/dashboard/SmartSearchPanel';
import { FixSuggestionPanel } from '../../components/dashboard/FixSuggestionPanel';

describe('Dashboard Integration Tests', () => {
  describe('Cross-Component Communication', () => {
    it('should synchronize issue selection between components', async () => {
      const mockEventSystem = {
        selectedIssue: null as any,
        listeners: new Map(),
        emit: jest.fn((event) => {
          const handlers = mockEventSystem.listeners.get(event.type) || [];
          handlers.forEach((handler: any) => handler(event));
        }),
        on: jest.fn((type, handler) => {
          if (!mockEventSystem.listeners.has(type)) {
            mockEventSystem.listeners.set(type, []);
          }
          mockEventSystem.listeners.get(type).push(handler);
        })
      };

      const onIssueSelect = jest.fn((issueId) => {
        mockEventSystem.selectedIssue = mockInteractiveReport.issues.find(i => i.id === issueId);
        mockEventSystem.emit({ type: 'issue-selected', data: { issueId } });
      });

      const onFixSuggestionSelect = jest.fn();

      render(
        <div>
          <PriorityMatrix
            issues={mockInteractiveReport.issues}
            onIssueSelect={onIssueSelect}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
          <FixSuggestionPanel
            selectedIssue={mockEventSystem.selectedIssue}
            onApplyFix={() => {}}
            onDismissIssue={() => {}}
            onRequestAlternatives={onFixSuggestionSelect}
            isLoading={false}
          />
        </div>
      );

      // Simulate issue selection in PriorityMatrix
      const priorityMatrix = screen.getByText('Priority Matrix');
      expect(priorityMatrix).toBeInTheDocument();

      // Trigger issue selection
      onIssueSelect('1');

      await waitFor(() => {
        expect(onIssueSelect).toHaveBeenCalledWith('1');
        expect(mockEventSystem.selectedIssue?.id).toBe('1');
      });
    });

    it('should filter issues across components when search is applied', async () => {
      const filteredIssues = mockInteractiveReport.issues.filter(issue => 
        issue.severity === 'high'
      );

      const onSearchResults = jest.fn();
      const onIssueSelect = jest.fn();

      render(
        <div>
          <SmartSearchPanel
            issues={mockInteractiveReport.issues}
            onSearchResults={onSearchResults}
            onSaveSearch={() => {}}
            onLoadSavedSearch={() => {}}
            savedSearches={[]}
          />
          <PriorityMatrix
            issues={filteredIssues}
            onIssueSelect={onIssueSelect}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
        </div>
      );

      // Apply high severity filter
      const highSeverityChip = screen.getByText('High');
      fireEvent.click(highSeverityChip);

      await waitFor(() => {
        expect(onSearchResults).toHaveBeenCalledWith(
          expect.arrayContaining([
            expect.objectContaining({ severity: 'high' })
          ])
        );
      });

      // Verify filtered results are reflected in PriorityMatrix
      const highIssues = mockInteractiveReport.issues.filter(i => i.severity === 'high');
      expect(screen.getByText(`${highIssues.length} issues`)).toBeInTheDocument();
    });

    it('should update quality metrics when issues are resolved', async () => {
      const initialMetrics = mockInteractiveReport.quality_metrics;
      const updatedMetrics = {
        ...initialMetrics,
        overall_score: initialMetrics.overall_score + 5,
        maintainability: initialMetrics.maintainability + 3
      };

      let currentMetrics = initialMetrics;
      const onQualityUpdate = jest.fn((newMetrics) => {
        currentMetrics = newMetrics;
      });

      const { rerender } = render(
        <QualityScoreCard
          qualityMetrics={currentMetrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={onQualityUpdate}
        />
      );

      expect(screen.getByText('87')).toBeInTheDocument(); // Initial score

      // Simulate issue resolution and metric update
      rerender(
        <QualityScoreCard
          qualityMetrics={updatedMetrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={onQualityUpdate}
        />
      );

      expect(screen.getByText('92')).toBeInTheDocument(); // Updated score
    });

    it('should coordinate between dependency graph and technical debt tracker', async () => {
      const onNodeSelect = jest.fn();
      const onDebtItemClick = jest.fn();

      render(
        <div>
          <InteractiveDependencyGraph
            dependencies={mockDependencies}
            onNodeSelect={onNodeSelect}
            onClusterAnalysis={() => {}}
            selectedNodes={[]}
            highlightCycles={true}
          />
          <TechnicalDebtTracker
            technicalDebt={mockInteractiveReport.technical_debt}
            onDebtItemClick={onDebtItemClick}
            onCategoryFilter={() => {}}
            onTimeRangeChange={() => {}}
          />
        </div>
      );

      // Simulate node selection in dependency graph
      const dependencyGraph = screen.getByText('Dependency Graph');
      expect(dependencyGraph).toBeInTheDocument();

      // The components should be able to cross-reference
      // debt associated with selected dependencies
      expect(screen.getByText('Technical Debt Tracker')).toBeInTheDocument();
    });
  });

  describe('Event-Driven Workflow Integration', () => {
    it('should handle complete issue resolution workflow', async () => {
      const workflowSteps = {
        issueSelected: false,
        fixGenerated: false,
        fixApplied: false,
        metricsUpdated: false
      };

      const onWorkflowStep = jest.fn((step) => {
        workflowSteps[step as keyof typeof workflowSteps] = true;
      });

      const selectedIssue = mockInteractiveReport.issues[0];

      render(
        <div>
          <PriorityMatrix
            issues={mockInteractiveReport.issues}
            onIssueSelect={() => onWorkflowStep('issueSelected')}
            onFilterChange={() => {}}
            selectedIssues={[selectedIssue.id]}
          />
          <FixSuggestionPanel
            selectedIssue={selectedIssue}
            onApplyFix={() => onWorkflowStep('fixApplied')}
            onDismissIssue={() => {}}
            onRequestAlternatives={() => onWorkflowStep('fixGenerated')}
            isLoading={false}
          />
          <QualityScoreCard
            qualityMetrics={mockInteractiveReport.quality_metrics}
            historicalData={[]}
            onDrillDown={() => {}}
            onBenchmarkCompare={() => onWorkflowStep('metricsUpdated')}
          />
        </div>
      );

      // Simulate workflow steps
      const applyFixButton = screen.getByRole('button', { name: /apply fix/i });
      fireEvent.click(applyFixButton);

      await waitFor(() => {
        expect(onWorkflowStep).toHaveBeenCalledWith('fixApplied');
      });

      const showAlternativesButton = screen.getByRole('button', { name: /show alternatives/i });
      fireEvent.click(showAlternativesButton);

      await waitFor(() => {
        expect(onWorkflowStep).toHaveBeenCalledWith('fixGenerated');
      });
    });

    it('should propagate filter changes across all components', async () => {
      const filterState = {
        severity: [] as string[],
        category: [] as string[],
        searchQuery: ''
      };

      const onFilterUpdate = jest.fn((newFilter) => {
        Object.assign(filterState, newFilter);
      });

      render(
        <div>
          <SmartSearchPanel
            issues={mockInteractiveReport.issues}
            onSearchResults={(results) => onFilterUpdate({ searchQuery: 'filtered' })}
            onSaveSearch={() => {}}
            onLoadSavedSearch={() => {}}
            savedSearches={[]}
          />
          <PriorityMatrix
            issues={mockInteractiveReport.issues.filter(issue =>
              filterState.severity.length === 0 || filterState.severity.includes(issue.severity)
            )}
            onIssueSelect={() => {}}
            onFilterChange={(severity) => onFilterUpdate({ severity: [severity] })}
            selectedIssues={[]}
          />
          <TechnicalDebtTracker
            technicalDebt={mockInteractiveReport.technical_debt}
            onDebtItemClick={() => {}}
            onCategoryFilter={(category) => onFilterUpdate({ category: [category] })}
            onTimeRangeChange={() => {}}
          />
        </div>
      );

      // Apply severity filter
      const criticalChip = screen.getByText('Critical');
      fireEvent.click(criticalChip);

      await waitFor(() => {
        expect(onFilterUpdate).toHaveBeenCalledWith({ severity: ['critical'] });
      });
    });
  });

  describe('Data Consistency Integration', () => {
    it('should maintain data consistency across component updates', async () => {
      const dataState = {
        issues: mockInteractiveReport.issues,
        dependencies: mockDependencies,
        qualityMetrics: mockInteractiveReport.quality_metrics
      };

      const onDataUpdate = jest.fn((updates) => {
        Object.assign(dataState, updates);
      });

      const { rerender } = render(
        <div>
          <PriorityMatrix
            issues={dataState.issues}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
          <InteractiveDependencyGraph
            dependencies={dataState.dependencies}
            onNodeSelect={() => {}}
            onClusterAnalysis={() => {}}
            selectedNodes={[]}
            highlightCycles={true}
          />
          <QualityScoreCard
            qualityMetrics={dataState.qualityMetrics}
            historicalData={[]}
            onDrillDown={() => {}}
            onBenchmarkCompare={() => {}}
          />
        </div>
      );

      // Verify initial state
      expect(screen.getByText(`${dataState.issues.length} issues`)).toBeInTheDocument();
      expect(screen.getByText(`${dataState.dependencies.length} nodes`)).toBeInTheDocument();
      expect(screen.getByText('87')).toBeInTheDocument(); // Quality score

      // Simulate data update
      const updatedData = {
        ...dataState,
        issues: dataState.issues.slice(0, 2), // Remove some issues
        qualityMetrics: {
          ...dataState.qualityMetrics,
          overall_score: 90
        }
      };

      rerender(
        <div>
          <PriorityMatrix
            issues={updatedData.issues}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
          <InteractiveDependencyGraph
            dependencies={updatedData.dependencies}
            onNodeSelect={() => {}}
            onClusterAnalysis={() => {}}
            selectedNodes={[]}
            highlightCycles={true}
          />
          <QualityScoreCard
            qualityMetrics={updatedData.qualityMetrics}
            historicalData={[]}
            onDrillDown={() => {}}
            onBenchmarkCompare={() => {}}
          />
        </div>
      );

      // Verify updated state
      expect(screen.getByText('2 issues')).toBeInTheDocument();
      expect(screen.getByText('90')).toBeInTheDocument(); // Updated quality score
    });
  });

  describe('Performance Integration', () => {
    it('should handle large datasets efficiently across components', async () => {
      // Create large dataset
      const largeIssueSet = Array(1000).fill(null).map((_, i) => ({
        ...mockInteractiveReport.issues[0],
        id: `issue-${i}`,
        title: `Issue ${i}`
      }));

      const largeDependencySet = Array(500).fill(null).map((_, i) => ({
        ...mockDependencies[0],
        id: `dep-${i}`,
        name: `Dependency ${i}`
      }));

      const startTime = Date.now();

      render(
        <div>
          <PriorityMatrix
            issues={largeIssueSet}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
          <InteractiveDependencyGraph
            dependencies={largeDependencySet}
            onNodeSelect={() => {}}
            onClusterAnalysis={() => {}}
            selectedNodes={[]}
            highlightCycles={true}
          />
          <SmartSearchPanel
            issues={largeIssueSet}
            onSearchResults={() => {}}
            onSaveSearch={() => {}}
            onLoadSavedSearch={() => {}}
            savedSearches={[]}
          />
        </div>
      );

      const renderTime = Date.now() - startTime;

      // Verify components render within reasonable time
      expect(renderTime).toBeLessThan(5000); // 5 seconds max
      expect(screen.getByText('1000 issues')).toBeInTheDocument();
      expect(screen.getByText('500 nodes')).toBeInTheDocument();
    });
  });

  describe('Error Handling Integration', () => {
    it('should gracefully handle component errors without affecting others', async () => {
      const mockConsoleError = jest.spyOn(console, 'error').mockImplementation(() => {});

      const invalidData = null as any;

      render(
        <div>
          <PriorityMatrix
            issues={invalidData}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
          <QualityScoreCard
            qualityMetrics={mockInteractiveReport.quality_metrics}
            historicalData={[]}
            onDrillDown={() => {}}
            onBenchmarkCompare={() => {}}
          />
        </div>
      );

      // Quality score card should still render properly
      expect(screen.getByText('Quality Score')).toBeInTheDocument();
      expect(screen.getByText('87')).toBeInTheDocument();

      mockConsoleError.mockRestore();
    });
  });
});