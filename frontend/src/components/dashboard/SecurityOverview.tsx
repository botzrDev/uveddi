import React, { useMemo } from 'react';
import { SecurityAnalysis } from '../../types/api';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  CardHeader,
  Chip,
  Typography,
} from '@mui/material';
import {
  Security as ShieldIcon,
  Warning as WarningIcon,
  CheckCircle as CheckIcon,
  Error as ErrorIcon,
} from '@mui/icons-material';
// import { SecurityMetrics } from './SecurityMetrics';
// import { OwaspCoverage } from './OwaspCoverage';
// import { SecurityIssuesList } from './SecurityIssuesList';
// import { TaintFlowDiagram } from './TaintFlowDiagram';

interface ExportableComponentProps {
  exportOptions?: {
    onExport?: () => void;
  };
}

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
        return <ErrorIcon color="error" />;
      case 'high':
        return <WarningIcon color="warning" />;
      case 'medium':
        return <WarningIcon color="warning" />;
      default:
        return <CheckIcon color="success" />;
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
    <Box className={className} sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
      {/* Security Status Header */}
      <Card>
        <CardHeader>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            {getStatusIcon()}
            <Typography variant="h5" component="h2">
              Security Overview
            </Typography>
            <Chip 
              label={`Score: ${securityScore}/100`}
              color={securityStatus === 'critical' || securityStatus === 'high' ? 'error' : 
                     securityStatus === 'medium' ? 'warning' : 'success'}
              sx={{ ml: 'auto' }}
            />
          </Box>
        </CardHeader>
        <CardContent>
          <Alert 
            severity={securityStatus === 'critical' || securityStatus === 'high' ? 'error' : 
                     securityStatus === 'medium' ? 'warning' : 'success'}
          >
            {getStatusMessage()}
          </Alert>
        </CardContent>
      </Card>

      {/* Security Metrics - Placeholder */}
      <Card>
        <CardHeader>
          <Typography variant="h6">Security Metrics</Typography>
        </CardHeader>
        <CardContent>
          <Typography>Security metrics will be displayed here.</Typography>
        </CardContent>
      </Card>

      {/* Export Actions */}
      <Card>
        <CardHeader>
          <Typography variant="h6">Export Security Results</Typography>
        </CardHeader>
        <CardContent>
          <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
            <Button
              variant="contained"
              onClick={onExportSarif}
              sx={{ minWidth: 120 }}
            >
              Export SARIF
            </Button>
            {exportOptions?.onExport && (
              <Button
                variant="outlined"
                onClick={() => exportOptions.onExport?.()}
                sx={{ minWidth: 120 }}
              >
                Export Report
              </Button>
            )}
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};

export default SecurityOverview;