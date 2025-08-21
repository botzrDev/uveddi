import React from 'react';
import { SecuritySummary } from '../../types/security';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/Card';
import { Badge } from '../ui/Badge';
import { AlertTriangle, AlertCircle, Info, CheckCircle } from 'lucide-react';

interface SecurityMetricsProps {
  summary: SecuritySummary;
  showDetails?: boolean;
  className?: string;
}

export const SecurityMetrics: React.FC<SecurityMetricsProps> = ({
  summary,
  showDetails = true,
  className = '',
}) => {
  const metrics = [
    {
      label: 'Critical',
      value: summary.criticalCount,
      icon: AlertTriangle,
      color: 'text-red-500',
      bgColor: 'bg-red-50',
      borderColor: 'border-red-200',
    },
    {
      label: 'High',
      value: summary.highCount,
      icon: AlertCircle,
      color: 'text-orange-500',
      bgColor: 'bg-orange-50',
      borderColor: 'border-orange-200',
    },
    {
      label: 'Medium',
      value: summary.mediumCount,
      icon: Info,
      color: 'text-yellow-500',
      bgColor: 'bg-yellow-50',
      borderColor: 'border-yellow-200',
    },
    {
      label: 'Low',
      value: summary.lowCount,
      icon: CheckCircle,
      color: 'text-green-500',
      bgColor: 'bg-green-50',
      borderColor: 'border-green-200',
    },
  ];

  return (
    <div className={`space-y-4 ${className}`}>
      <Card>
        <CardHeader>
          <CardTitle>Security Issues Summary</CardTitle>
        </CardHeader>
        <CardContent>
          {/* Main Metrics Grid */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            {metrics.map((metric) => (
              <div
                key={metric.label}
                className={`p-4 rounded-lg border-2 ${metric.bgColor} ${metric.borderColor} transition-all hover:shadow-md`}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <div className={`flex items-center gap-2 ${metric.color} mb-1`}>
                      <metric.icon className="h-5 w-5" />
                      <span className="text-sm font-medium text-gray-600">
                        {metric.label}
                      </span>
                    </div>
                    <div className="text-2xl font-bold text-gray-900">
                      {metric.value}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Total Issues */}
          <div className="bg-gray-50 rounded-lg p-4 mb-4">
            <div className="flex items-center justify-between">
              <div className="text-sm font-medium text-gray-600">Total Issues</div>
              <Badge variant="outline" className="text-lg px-3 py-1">
                {summary.totalIssues}
              </Badge>
            </div>
          </div>

          {/* Security Score */}
          <div className="bg-blue-50 rounded-lg p-4 mb-4">
            <div className="flex items-center justify-between">
              <div className="text-sm font-medium text-gray-600">Security Score</div>
              <Badge 
                variant={summary.securityScore >= 80 ? 'default' : 
                        summary.securityScore >= 60 ? 'secondary' : 'destructive'}
                className="text-lg px-3 py-1"
              >
                {Math.round(summary.securityScore)}/100
              </Badge>
            </div>
          </div>

          {/* Confidence Distribution */}
          {showDetails && summary.confidenceDistribution && (
            <div className="mt-6">
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Detection Confidence Distribution
              </h4>
              <div className="grid grid-cols-3 gap-4">
                {Object.entries(summary.confidenceDistribution).map(([level, count]) => (
                  <div key={level} className="bg-gray-50 rounded-lg p-3 text-center">
                    <div className="text-lg font-semibold text-gray-900">{count}</div>
                    <div className="text-sm text-gray-600 capitalize">{level} Confidence</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Most Common Issues */}
          {showDetails && summary.mostCommonIssues && summary.mostCommonIssues.length > 0 && (
            <div className="mt-6">
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Most Common Issue Types
              </h4>
              <div className="space-y-2">
                {summary.mostCommonIssues.slice(0, 5).map((issue, index) => (
                  <div key={issue.issueType} className="flex items-center justify-between bg-gray-50 rounded-lg p-3">
                    <div className="flex items-center gap-3">
                      <Badge variant="outline" className="text-xs">
                        #{index + 1}
                      </Badge>
                      <span className="font-medium text-gray-900">{issue.issueType}</span>
                    </div>
                    <div className="flex items-center gap-3">
                      <Badge variant="secondary">{issue.count} issues</Badge>
                      <Badge 
                        variant={issue.avgSeverity === 'Critical' ? 'destructive' :
                                issue.avgSeverity === 'High' ? 'destructive' :
                                issue.avgSeverity === 'Medium' ? 'secondary' : 'default'}
                      >
                        {issue.avgSeverity}
                      </Badge>
                      <span className="text-sm text-gray-600">
                        {Math.round(issue.avgConfidence * 100)}% conf.
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default SecurityMetrics;