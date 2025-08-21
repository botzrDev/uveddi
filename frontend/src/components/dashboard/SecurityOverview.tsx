import React, { useMemo } from 'react';
import { SecurityAnalysis } from '../../types/security';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/Card';
import { Badge } from '../ui/Badge';
import { Alert, AlertDescription } from '../ui/Alert';
import { SecurityMetrics } from './SecurityMetrics';
import { OwaspCoverage } from './OwaspCoverage';
import { SecurityIssuesList } from './SecurityIssuesList';
import { TaintFlowDiagram } from './TaintFlowDiagram';
import { ExportableComponentProps } from '../../types/dashboard';
import { ShieldAlert, ShieldCheck, AlertTriangle, Info } from 'lucide-react';

interface SecurityOverviewProps extends ExportableComponentProps {
  securityAnalysis: SecurityAnalysis;
  className?: string;
  showDetails?: boolean;
  onIssueSelect?: (issueId: string) => void;
  onExportSarif?: () => void;
}

export const SecurityOverview: React.FC<SecurityOverviewProps> = ({
  securityAnalysis,
  className = '',
  showDetails = true,
  onIssueSelect,
  onExportSarif,
  exportOptions,
}) => {
  const securityScore = useMemo(() => {
    const { summary } = securityAnalysis;
    if (summary.totalIssues === 0) return 100;
    
    // Calculate weighted score based on severity
    const weighted = (
      summary.criticalCount * 4 +
      summary.highCount * 3 +
      summary.mediumCount * 2 +
      summary.lowCount * 1
    );
    const max = summary.totalIssues * 4;
    return Math.max(0, Math.round(100 - (weighted / max) * 100));
  }, [securityAnalysis.summary]);

  const securityStatus = useMemo(() => {
    if (securityAnalysis.summary.criticalCount > 0) return 'critical';
    if (securityAnalysis.summary.highCount > 0) return 'high';
    if (securityAnalysis.summary.mediumCount > 0) return 'medium';
    return 'low';
  }, [securityAnalysis.summary]);

  const getStatusIcon = () => {
    switch (securityStatus) {
      case 'critical':
        return <ShieldAlert className="h-6 w-6 text-red-500" />;
      case 'high':
        return <AlertTriangle className="h-6 w-6 text-orange-500" />;
      case 'medium':
        return <Info className="h-6 w-6 text-yellow-500" />;
      default:
        return <ShieldCheck className="h-6 w-6 text-green-500" />;
    }
  };

  const getStatusMessage = () => {
    switch (securityStatus) {
      case 'critical':
        return 'Critical security vulnerabilities detected that require immediate attention.';
      case 'high':
        return 'High-severity security issues found that should be addressed soon.';
      case 'medium':
        return 'Medium-severity security issues identified for review.';
      default:
        return 'Good security posture with only low-severity issues or no issues found.';
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Security Status Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-3">
            {getStatusIcon()}
            <span>Security Overview</span>
            <Badge 
              variant={securityStatus === 'critical' ? 'destructive' : 
                     securityStatus === 'high' ? 'destructive' :
                     securityStatus === 'medium' ? 'secondary' : 'default'}
              className="ml-auto"
            >
              Score: {securityScore}/100
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Alert>
            <AlertDescription>
              {getStatusMessage()}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Security Metrics */}
      <SecurityMetrics 
        summary={securityAnalysis.summary}
        showDetails={showDetails}
        className="mb-6"
      />

      {/* OWASP Top 10 Coverage */}
      {securityAnalysis.owaspCoverage && (
        <OwaspCoverage 
          coverage={securityAnalysis.owaspCoverage}
          className="mb-6"
        />
      )}

      {/* Security Issues List */}
      {securityAnalysis.issues && securityAnalysis.issues.length > 0 && (
        <SecurityIssuesList
          issues={securityAnalysis.issues}
          onIssueSelect={onIssueSelect}
          showAll={showDetails}
          className="mb-6"
        />
      )}

      {/* Taint Flow Analysis */}
      {securityAnalysis.taintFlows && securityAnalysis.taintFlows.length > 0 && showDetails && (
        <TaintFlowDiagram
          taintFlows={securityAnalysis.taintFlows}
          className="mb-6"
        />
      )}

      {/* Export Actions */}
      <Card>
        <CardHeader>
          <CardTitle>Export Security Results</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4 flex-wrap">
            <button
              onClick={onExportSarif}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors flex items-center gap-2"
            >
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              Export SARIF
            </button>
            {exportOptions?.onExport && (
              <button
                onClick={() => exportOptions.onExport?.()}
                className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 transition-colors flex items-center gap-2"
              >
                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
                </svg>
                Export Report
              </button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default SecurityOverview;