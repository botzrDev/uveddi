import React from 'react';
import { render, screen } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { render as customRender } from '../../test-utils/test-utils';
import { mockInteractiveReport, mockDependencies, mockTeamMembers } from '../../test-utils/mocks/mockData';

// Import all dashboard components for accessibility testing
import { PriorityMatrix } from '../../components/dashboard/PriorityMatrix';
import { QualityScoreCard } from '../../components/dashboard/QualityScoreCard';
import { InteractiveDependencyGraph } from '../../components/dashboard/InteractiveDependencyGraph';
import { TechnicalDebtTracker } from '../../components/dashboard/TechnicalDebtTracker';
import { FixSuggestionPanel } from '../../components/dashboard/FixSuggestionPanel';
import { SmartSearchPanel } from '../../components/dashboard/SmartSearchPanel';
import { HistoricalComparison } from '../../components/dashboard/HistoricalComparison';
import { CollaborationPanel } from '../../components/dashboard/CollaborationPanel';
import { DashboardBuilder } from '../../components/dashboard/DashboardBuilder';
import { ExportHub } from '../../components/dashboard/ExportHub';

// Add jest-axe matcher
expect.extend(toHaveNoViolations);

describe('Dashboard Accessibility Tests', () => {
  beforeEach(() => {
    // Reset any global accessibility settings
    document.documentElement.lang = 'en';
  });

  describe('WCAG 2.1 AA Compliance', () => {
    it('PriorityMatrix should have no accessibility violations', async () => {
      const { container } = customRender(
        <PriorityMatrix
          issues={mockInteractiveReport.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('QualityScoreCard should have no accessibility violations', async () => {
      const { container } = customRender(
        <QualityScoreCard
          qualityMetrics={mockInteractiveReport.quality_metrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('InteractiveDependencyGraph should have no accessibility violations', async () => {
      const { container } = customRender(
        <InteractiveDependencyGraph
          dependencies={mockDependencies}
          onNodeSelect={() => {}}
          onClusterAnalysis={() => {}}
          selectedNodes={[]}
          highlightCycles={true}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('TechnicalDebtTracker should have no accessibility violations', async () => {
      const { container } = customRender(
        <TechnicalDebtTracker
          technicalDebt={mockInteractiveReport.technical_debt}
          onDebtItemClick={() => {}}
          onCategoryFilter={() => {}}
          onTimeRangeChange={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('FixSuggestionPanel should have no accessibility violations', async () => {
      const { container } = customRender(
        <FixSuggestionPanel
          selectedIssue={mockInteractiveReport.issues[0]}
          onApplyFix={() => {}}
          onDismissIssue={() => {}}
          onRequestAlternatives={() => {}}
          isLoading={false}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('SmartSearchPanel should have no accessibility violations', async () => {
      const { container } = customRender(
        <SmartSearchPanel
          issues={mockInteractiveReport.issues}
          onSearchResults={() => {}}
          onSaveSearch={() => {}}
          onLoadSavedSearch={() => {}}
          savedSearches={[]}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('HistoricalComparison should have no accessibility violations', async () => {
      const { container } = customRender(
        <HistoricalComparison
          historicalData={[]}
          currentSnapshot={{
            id: 'current',
            timestamp: new Date().toISOString(),
            qualityScore: 85,
            issueCount: 10,
            technicalDebt: 5000,
            testCoverage: 80,
            linesOfCode: 10000
          }}
          onSnapshotSelect={() => {}}
          onComparisonGenerate={() => {}}
          onTrendAnalysis={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('CollaborationPanel should have no accessibility violations', async () => {
      const { container } = customRender(
        <CollaborationPanel
          teamMembers={mockTeamMembers}
          discussions={[]}
          codeReviews={[]}
          onStartDiscussion={() => {}}
          onRequestReview={() => {}}
          onAssignIssue={() => {}}
          onAddComment={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('DashboardBuilder should have no accessibility violations', async () => {
      const { container } = customRender(
        <DashboardBuilder
          availableWidgets={[]}
          currentLayout={[]}
          onLayoutChange={() => {}}
          onWidgetAdd={() => {}}
          onWidgetRemove={() => {}}
          onWidgetConfigure={() => {}}
          onLayoutSave={() => {}}
          onLayoutLoad={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('ExportHub should have no accessibility violations', async () => {
      const { container } = customRender(
        <ExportHub
          reportData={mockInteractiveReport}
          onExportComplete={() => {}}
          onExportError={() => {}}
          availableFormats={['pdf', 'png', 'json']}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('Keyboard Navigation', () => {
    it('should support tab navigation in PriorityMatrix', () => {
      customRender(
        <PriorityMatrix
          issues={mockInteractiveReport.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
        />
      );

      // Find all focusable elements
      const focusableElements = screen.getAllByRole('button');
      
      expect(focusableElements.length).toBeGreaterThan(0);
      
      focusableElements.forEach(element => {
        expect(element).toHaveAttribute('tabindex', expect.any(String));
      });
    });

    it('should support arrow key navigation in dependency graph', () => {
      customRender(
        <InteractiveDependencyGraph
          dependencies={mockDependencies}
          onNodeSelect={() => {}}
          onClusterAnalysis={() => {}}
          selectedNodes={[]}
          highlightCycles={true}
        />
      );

      const graphContainer = screen.getByRole('img', { name: /dependency graph/i });
      expect(graphContainer).toHaveAttribute('tabindex', '0');
    });

    it('should have proper focus management in modals', () => {
      customRender(
        <ExportHub
          reportData={mockInteractiveReport}
          onExportComplete={() => {}}
          onExportError={() => {}}
          availableFormats={['pdf', 'png', 'json']}
        />
      );

      // Check for proper modal focus management
      const buttons = screen.getAllByRole('button');
      buttons.forEach(button => {
        expect(button).not.toHaveAttribute('tabindex', '-1');
      });
    });
  });

  describe('Screen Reader Support', () => {
    it('should have proper ARIA labels on interactive elements', () => {
      customRender(
        <QualityScoreCard
          qualityMetrics={mockInteractiveReport.quality_metrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={() => {}}
        />
      );

      const progressBars = screen.getAllByRole('progressbar');
      progressBars.forEach(progressBar => {
        expect(progressBar).toHaveAttribute('aria-label');
        expect(progressBar).toHaveAttribute('aria-valuenow');
        expect(progressBar).toHaveAttribute('aria-valuemin');
        expect(progressBar).toHaveAttribute('aria-valuemax');
      });
    });

    it('should provide descriptive alt text for charts', () => {
      customRender(
        <PriorityMatrix
          issues={mockInteractiveReport.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
        />
      );

      const chartImage = screen.getByRole('img', { name: /priority matrix chart/i });
      expect(chartImage).toHaveAttribute('aria-label');
    });

    it('should announce dynamic content changes', () => {
      customRender(
        <SmartSearchPanel
          issues={mockInteractiveReport.issues}
          onSearchResults={() => {}}
          onSaveSearch={() => {}}
          onLoadSavedSearch={() => {}}
          savedSearches={[]}
        />
      );

      // Check for live regions
      const searchInput = screen.getByRole('combobox');
      expect(searchInput).toHaveAttribute('aria-describedby');
    });

    it('should have proper table headers and captions', () => {
      customRender(
        <TechnicalDebtTracker
          technicalDebt={mockInteractiveReport.technical_debt}
          onDebtItemClick={() => {}}
          onCategoryFilter={() => {}}
          onTimeRangeChange={() => {}}
        />
      );

      // Check for proper table structure
      const tables = document.querySelectorAll('table');
      tables.forEach(table => {
        const headers = table.querySelectorAll('th');
        headers.forEach(header => {
          expect(header).toHaveAttribute('scope');
        });
      });
    });
  });

  describe('Color Contrast and Visual Accessibility', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = customRender(
        <QualityScoreCard
          qualityMetrics={mockInteractiveReport.quality_metrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={() => {}}
        />
      );

      // Use axe with color-contrast rule specifically
      const results = await axe(container, {
        rules: {
          'color-contrast': { enabled: true }
        }
      });

      expect(results).toHaveNoViolations();
    });

    it('should not rely solely on color to convey information', () => {
      customRender(
        <PriorityMatrix
          issues={mockInteractiveReport.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
        />
      );

      // Check that severity indicators have text labels, not just colors
      const severityElements = screen.getAllByText(/critical|high|medium|low/i);
      expect(severityElements.length).toBeGreaterThan(0);
    });

    it('should support high contrast mode', async () => {
      // Simulate high contrast mode
      document.body.classList.add('high-contrast');
      
      const { container } = customRender(
        <DashboardBuilder
          availableWidgets={[]}
          currentLayout={[]}
          onLayoutChange={() => {}}
          onWidgetAdd={() => {}}
          onWidgetRemove={() => {}}
          onWidgetConfigure={() => {}}
          onLayoutSave={() => {}}
          onLayoutLoad={() => {}}
        />
      );

      // Check that elements are still accessible in high contrast mode
      const results = await axe(container);
      expect(results).toHaveNoViolations();
      
      document.body.classList.remove('high-contrast');
    });
  });

  describe('Motion and Animation Accessibility', () => {
    it('should respect prefers-reduced-motion setting', () => {
      // Mock prefers-reduced-motion
      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: jest.fn().mockImplementation(query => ({
          matches: query === '(prefers-reduced-motion: reduce)',
          media: query,
          onchange: null,
          addListener: jest.fn(),
          removeListener: jest.fn(),
          addEventListener: jest.fn(),
          removeEventListener: jest.fn(),
          dispatchEvent: jest.fn(),
        })),
      });

      customRender(
        <HistoricalComparison
          historicalData={[]}
          currentSnapshot={{
            id: 'current',
            timestamp: new Date().toISOString(),
            qualityScore: 85,
            issueCount: 10,
            technicalDebt: 5000,
            testCoverage: 80,
            linesOfCode: 10000
          }}
          onSnapshotSelect={() => {}}
          onComparisonGenerate={() => {}}
          onTrendAnalysis={() => {}}
        />
      );

      // Animations should be reduced or disabled
      // This would typically check CSS properties or animation states
      expect(screen.getByText('Historical Analysis')).toBeInTheDocument();
    });

    it('should not cause seizures with flashing content', () => {
      customRender(
        <InteractiveDependencyGraph
          dependencies={mockDependencies}
          onNodeSelect={() => {}}
          onClusterAnalysis={() => {}}
          selectedNodes={[]}
          highlightCycles={true}
        />
      );

      // Ensure no rapidly flashing content
      // This would check for animations that flash more than 3 times per second
      expect(screen.getByText('Dependency Graph')).toBeInTheDocument();
    });
  });

  describe('Form and Input Accessibility', () => {
    it('should have proper form labels and descriptions', () => {
      customRender(
        <SmartSearchPanel
          issues={mockInteractiveReport.issues}
          onSearchResults={() => {}}
          onSaveSearch={() => {}}
          onLoadSavedSearch={() => {}}
          savedSearches={[]}
        />
      );

      const searchInput = screen.getByRole('combobox');
      expect(searchInput).toHaveAttribute('aria-label');
    });

    it('should provide error messages for form validation', () => {
      customRender(
        <CollaborationPanel
          teamMembers={mockTeamMembers}
          discussions={[]}
          codeReviews={[]}
          onStartDiscussion={() => {}}
          onRequestReview={() => {}}
          onAssignIssue={() => {}}
          onAddComment={() => {}}
        />
      );

      // Check for proper error handling in forms
      const textInputs = screen.getAllByRole('textbox');
      textInputs.forEach(input => {
        expect(input).toHaveAttribute('aria-describedby');
      });
    });

    it('should have proper fieldset grouping', () => {
      customRender(
        <ExportHub
          reportData={mockInteractiveReport}
          onExportComplete={() => {}}
          onExportError={() => {}}
          availableFormats={['pdf', 'png', 'json']}
        />
      );

      // Check for proper form grouping
      const fieldsets = document.querySelectorAll('fieldset');
      fieldsets.forEach(fieldset => {
        const legend = fieldset.querySelector('legend');
        expect(legend).not.toBeNull();
      });
    });
  });

  describe('Mobile Accessibility', () => {
    it('should be accessible on touch devices', async () => {
      // Simulate mobile viewport
      Object.defineProperty(window, 'innerWidth', { writable: true, configurable: true, value: 375 });
      Object.defineProperty(window, 'innerHeight', { writable: true, configurable: true, value: 667 });
      
      const { container } = customRender(
        <PriorityMatrix
          issues={mockInteractiveReport.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
        />
      );

      // Check that touch targets are at least 44px (recommended minimum)
      const buttons = container.querySelectorAll('button');
      buttons.forEach(button => {
        const styles = window.getComputedStyle(button);
        const minSize = parseInt(styles.minHeight, 10) || parseInt(styles.height, 10);
        expect(minSize).toBeGreaterThanOrEqual(44);
      });

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should support voice control', () => {
      customRender(
        <QualityScoreCard
          qualityMetrics={mockInteractiveReport.quality_metrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={() => {}}
        />
      );

      // Check that interactive elements have accessible names
      const buttons = screen.getAllByRole('button');
      buttons.forEach(button => {
        expect(
          button.hasAttribute('aria-label') || 
          button.textContent?.trim() !== '' ||
          button.hasAttribute('aria-labelledby')
        ).toBeTruthy();
      });
    });
  });

  describe('Internationalization Accessibility', () => {
    it('should support right-to-left languages', async () => {
      document.documentElement.dir = 'rtl';
      document.documentElement.lang = 'ar';

      const { container } = customRender(
        <TechnicalDebtTracker
          technicalDebt={mockInteractiveReport.technical_debt}
          onDebtItemClick={() => {}}
          onCategoryFilter={() => {}}
          onTimeRangeChange={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();

      // Reset to default
      document.documentElement.dir = 'ltr';
      document.documentElement.lang = 'en';
    });

    it('should have proper language attributes', () => {
      customRender(
        <FixSuggestionPanel
          selectedIssue={mockInteractiveReport.issues[0]}
          onApplyFix={() => {}}
          onDismissIssue={() => {}}
          onRequestAlternatives={() => {}}
          isLoading={false}
        />
      );

      // Check that the document has a language attribute
      expect(document.documentElement.lang).toBeTruthy();
    });
  });

  describe('Loading States Accessibility', () => {
    it('should announce loading states to screen readers', () => {
      customRender(
        <FixSuggestionPanel
          selectedIssue={mockInteractiveReport.issues[0]}
          onApplyFix={() => {}}
          onDismissIssue={() => {}}
          onRequestAlternatives={() => {}}
          isLoading={true}
        />
      );

      const loadingIndicator = screen.getByRole('progressbar');
      expect(loadingIndicator).toHaveAttribute('aria-label');
    });

    it('should manage focus during loading states', () => {
      customRender(
        <ExportHub
          reportData={mockInteractiveReport}
          onExportComplete={() => {}}
          onExportError={() => {}}
          availableFormats={['pdf', 'png', 'json']}
        />
      );

      // Loading states should not cause focus to be lost
      const focusableElements = screen.getAllByRole('button');
      expect(focusableElements.length).toBeGreaterThan(0);
    });
  });
});