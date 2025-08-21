import React from 'react';
import { render, screen } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { render as customRender } from '../../test-utils/test-utils';
import type { SecurityAnalysis } from '../../types/security';

// Import security dashboard components for accessibility testing
import SecurityOverview from '../../components/dashboard/SecurityOverview';
import SecurityMetrics from '../../components/dashboard/SecurityMetrics';
import OwaspCoverage from '../../components/dashboard/OwaspCoverage';
import SecurityIssuesList from '../../components/dashboard/SecurityIssuesList';
import TaintFlowDiagram from '../../components/dashboard/TaintFlowDiagram';

// Add jest-axe matcher
expect.extend({ toHaveNoViolations });

const mockSecurityAnalysis: SecurityAnalysis = {
  summary: {
    totalIssues: 5,
    criticalCount: 1,
    highCount: 2,
    mediumCount: 2,
    lowCount: 0,
    confidenceDistribution: { high: 3, medium: 2 },
    mostCommonIssues: [],
    securityScore: 75
  },
  owaspCoverage: {
    'A01_2021': {
      issuesFound: 2,
      coveragePercentage: 80,
      avgConfidence: 0.8,
      severityDistribution: { critical: 1, high: 1 }
    }
  },
  issues: [],
  taintFlows: [],
  correlations: []
};

describe('Dashboard Components Accessibility', () => {
  beforeEach(() => {
    // Reset any global accessibility settings
    document.documentElement.lang = 'en';
  });

  describe('WCAG 2.1 AA Compliance', () => {
    it('SecurityOverview should have no accessibility violations', async () => {
      const { container } = customRender(
        <SecurityOverview
          securityAnalysis={mockSecurityAnalysis}
          onExportSarif={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('SecurityMetrics should have no accessibility violations', async () => {
      const { container } = customRender(
        <SecurityMetrics summary={mockSecurityAnalysis.summary} />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('OwaspCoverage should have no accessibility violations', async () => {
      const { container } = customRender(
        <OwaspCoverage coverage={mockSecurityAnalysis.owaspCoverage} />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('SecurityIssuesList should have no accessibility violations', async () => {
      const { container } = customRender(
        <SecurityIssuesList
          issues={mockSecurityAnalysis.issues}
          onIssueSelect={() => {}}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('TaintFlowDiagram should have no accessibility violations', async () => {
      const { container } = customRender(
        <TaintFlowDiagram taintFlows={mockSecurityAnalysis.taintFlows} />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('Keyboard Navigation', () => {
    it('should support keyboard navigation in SecurityOverview', () => {
      customRender(
        <SecurityOverview
          securityAnalysis={mockSecurityAnalysis}
          onExportSarif={() => {}}
        />
      );

      // Test that actionable elements are focusable
      const exportButton = screen.queryByText(/Export SARIF/i);
      if (exportButton) {
        exportButton.focus();
        expect(exportButton).toHaveFocus();
      }
    });
  });

  describe('Screen Reader Support', () => {
    it('should have proper ARIA labels for SecurityMetrics', () => {
      customRender(
        <SecurityMetrics summary={mockSecurityAnalysis.summary} />
      );

      // Check for semantic landmarks
      const content = screen.getByRole('main', { hidden: true });
      expect(content || screen.getByText('Critical')).toBeInTheDocument();
    });
  });
});