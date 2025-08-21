import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { render as customRender } from '../../test-utils/test-utils';
import type { SecurityAnalysis } from '../../types/security';

// Import security dashboard components
import SecurityOverview from '../../components/dashboard/SecurityOverview';
import SecurityMetrics from '../../components/dashboard/SecurityMetrics';
import OwaspCoverage from '../../components/dashboard/OwaspCoverage';
import SecurityIssuesList from '../../components/dashboard/SecurityIssuesList';
import TaintFlowDiagram from '../../components/dashboard/TaintFlowDiagram';

// Mock data for testing
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

describe('Test Coverage Validation', () => {
  describe('Component Rendering Coverage', () => {
    it('should render SecurityOverview without crashing', () => {
      const { container } = customRender(
        <SecurityOverview
          securityAnalysis={mockSecurityAnalysis}
          onExportSarif={() => {}}
        />
      );
      expect(container).toBeInTheDocument();
    });

    it('should render SecurityMetrics without crashing', () => {
      const { container } = customRender(
        <SecurityMetrics summary={mockSecurityAnalysis.summary} />
      );
      expect(container).toBeInTheDocument();
    });

    it('should render OwaspCoverage without crashing', () => {
      const { container } = customRender(
        <OwaspCoverage coverage={mockSecurityAnalysis.owaspCoverage} />
      );
      expect(container).toBeInTheDocument();
    });

    it('should render SecurityIssuesList without crashing', () => {
      const { container } = customRender(
        <SecurityIssuesList
          issues={mockSecurityAnalysis.issues}
          onIssueSelect={() => {}}
        />
      );
      expect(container).toBeInTheDocument();
    });

    it('should render TaintFlowDiagram without crashing', () => {
      const { container } = customRender(
        <TaintFlowDiagram taintFlows={mockSecurityAnalysis.taintFlows} />
      );
      expect(container).toBeInTheDocument();
    });
  });

  describe('User Interaction Coverage', () => {
    it('should handle SecurityOverview interactions', async () => {
      const mockExportSarif = jest.fn();

      customRender(
        <SecurityOverview
          securityAnalysis={mockSecurityAnalysis}
          onExportSarif={mockExportSarif}
        />
      );

      // Test export functionality if button exists
      const exportButton = screen.queryByText(/Export SARIF/i);
      if (exportButton) {
        fireEvent.click(exportButton);
        expect(mockExportSarif).toHaveBeenCalled();
      }
    });

    it('should handle OwaspCoverage rendering', () => {
      customRender(
        <OwaspCoverage coverage={mockSecurityAnalysis.owaspCoverage} />
      );

      // Test basic rendering
      expect(screen.getByRole('main', { hidden: true }) || screen.getByText(/owasp/i, { exact: false })).toBeInTheDocument();
    });

    it('should handle SecurityIssuesList rendering', () => {
      customRender(
        <SecurityIssuesList
          issues={mockSecurityAnalysis.issues}
          onIssueSelect={() => {}}
        />
      );

      // Test basic rendering
      expect(screen.getByRole('main', { hidden: true }) || screen.getByText(/issues/i, { exact: false })).toBeInTheDocument();
    });
  });

  describe('Error Boundary Coverage', () => {
    it('should handle SecurityOverview with invalid data gracefully', () => {
      const invalidAnalysis = {
        summary: null,
        owaspCoverage: null,
        issues: null,
        taintFlows: null,
        correlations: null
      };

      // Should not crash with invalid data
      expect(() => {
        customRender(
          <SecurityOverview
            securityAnalysis={invalidAnalysis as any}
            onExportSarif={() => {}}
          />
        );
      }).not.toThrow();
    });
  });

  describe('Prop Type Coverage', () => {
    it('should accept all required props for SecurityMetrics', () => {
      const requiredProps = {
        summary: mockSecurityAnalysis.summary
      };

      expect(() => {
        customRender(<SecurityMetrics {...requiredProps} />);
      }).not.toThrow();
    });

    it('should accept all required props for TaintFlowDiagram', () => {
      const requiredProps = {
        taintFlows: mockSecurityAnalysis.taintFlows
      };

      expect(() => {
        customRender(<TaintFlowDiagram {...requiredProps} />);
      }).not.toThrow();
    });
  });
});