import React from 'react';
import { OwaspCategoryStats } from '../../types/security';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/Card';
import { Badge } from '../ui/Badge';
import { Progress } from '../ui/Progress';
import { ExternalLink, Shield, AlertTriangle } from 'lucide-react';

interface OwaspCoverageProps {
  coverage: Record<string, OwaspCategoryStats>;
  className?: string;
}

export const OwaspCoverage: React.FC<OwaspCoverageProps> = ({
  coverage,
  className = '',
}) => {
  const owaspCategories = [
    { id: 'A01_Broken_Access_Control', name: 'Broken Access Control', url: 'https://owasp.org/Top10/A01_2021-Broken_Access_Control/' },
    { id: 'A02_Cryptographic_Failures', name: 'Cryptographic Failures', url: 'https://owasp.org/Top10/A02_2021-Cryptographic_Failures/' },
    { id: 'A03_Injection', name: 'Injection', url: 'https://owasp.org/Top10/A03_2021-Injection/' },
    { id: 'A04_Insecure_Design', name: 'Insecure Design', url: 'https://owasp.org/Top10/A04_2021-Insecure_Design/' },
    { id: 'A05_Security_Misconfiguration', name: 'Security Misconfiguration', url: 'https://owasp.org/Top10/A05_2021-Security_Misconfiguration/' },
    { id: 'A06_Vulnerable_Components', name: 'Vulnerable and Outdated Components', url: 'https://owasp.org/Top10/A06_2021-Vulnerable_and_Outdated_Components/' },
    { id: 'A07_Authentication_Failures', name: 'Identification and Authentication Failures', url: 'https://owasp.org/Top10/A07_2021-Identification_and_Authentication_Failures/' },
    { id: 'A08_Software_Data_Integrity_Failures', name: 'Software and Data Integrity Failures', url: 'https://owasp.org/Top10/A08_2021-Software_and_Data_Integrity_Failures/' },
    { id: 'A09_Security_Logging_Failures', name: 'Security Logging and Monitoring Failures', url: 'https://owasp.org/Top10/A09_2021-Security_Logging_and_Monitoring_Failures/' },
    { id: 'A10_Server_Side_Request_Forgery', name: 'Server-Side Request Forgery (SSRF)', url: 'https://owasp.org/Top10/A10_2021-Server-Side_Request_Forgery_%28SSRF%29/' },
  ];

  const getCoverageColor = (percentage: number) => {
    if (percentage >= 90) return 'text-green-600';
    if (percentage >= 75) return 'text-lime-600';
    if (percentage >= 50) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getCoverageBadgeVariant = (percentage: number) => {
    if (percentage >= 90) return 'default';
    if (percentage >= 75) return 'secondary';
    if (percentage >= 50) return 'secondary';
    return 'destructive';
  };

  const getRiskLevel = (stats: OwaspCategoryStats) => {
    if (stats.issuesFound === 0) return { level: 'None', color: 'text-green-600' };
    if (stats.issuesFound >= 5) return { level: 'High', color: 'text-red-600' };
    if (stats.issuesFound >= 2) return { level: 'Medium', color: 'text-yellow-600' };
    return { level: 'Low', color: 'text-green-600' };
  };

  return (
    <div className={className}>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-600" />
            OWASP Top 10 2021 Coverage Analysis
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {owaspCategories.map((category) => {
              const stats = coverage[category.id];
              if (!stats) return null;

              const risk = getRiskLevel(stats);

              return (
                <div key={category.id} className="border rounded-lg p-4 hover:bg-gray-50 transition-colors">
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <h4 className="font-medium text-gray-900">{category.name}</h4>
                        <a
                          href={category.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-blue-600 hover:text-blue-800"
                          title="View OWASP documentation"
                        >
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      </div>
                      <div className="text-sm text-gray-600">
                        {category.id}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge 
                        variant={getCoverageBadgeVariant(stats.coveragePercentage)}
                        className="text-xs"
                      >
                        {Math.round(stats.coveragePercentage)}% Coverage
                      </Badge>
                    </div>
                  </div>

                  {/* Coverage Progress Bar */}
                  <div className="mb-3">
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-gray-600">Detection Coverage</span>
                      <span className={getCoverageColor(stats.coveragePercentage)}>
                        {Math.round(stats.coveragePercentage)}%
                      </span>
                    </div>
                    <Progress 
                      value={stats.coveragePercentage} 
                      className="h-2"
                    />
                  </div>

                  {/* Stats Grid */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div className="text-center">
                      <div className="text-lg font-semibold text-gray-900">
                        {stats.issuesFound}
                      </div>
                      <div className="text-gray-600">Issues Found</div>
                    </div>
                    <div className="text-center">
                      <div className={`text-lg font-semibold ${risk.color}`}>
                        {risk.level}
                      </div>
                      <div className="text-gray-600">Risk Level</div>
                    </div>
                    <div className="text-center">
                      <div className="text-lg font-semibold text-gray-900">
                        {Math.round(stats.avgConfidence * 100)}%
                      </div>
                      <div className="text-gray-600">Avg Confidence</div>
                    </div>
                    <div className="text-center">
                      <div className="text-lg font-semibold text-gray-900">
                        {Object.values(stats.severityDistribution || {}).reduce((a, b) => a + b, 0)}
                      </div>
                      <div className="text-gray-600">Total Issues</div>
                    </div>
                  </div>

                  {/* Severity Distribution */}
                  {stats.severityDistribution && Object.keys(stats.severityDistribution).length > 0 && (
                    <div className="mt-3 pt-3 border-t">
                      <div className="text-sm text-gray-600 mb-2">Severity Distribution:</div>
                      <div className="flex gap-2 flex-wrap">
                        {Object.entries(stats.severityDistribution).map(([severity, count]) => (
                          <Badge
                            key={severity}
                            variant={
                              severity === 'Critical' ? 'destructive' :
                              severity === 'High' ? 'destructive' :
                              severity === 'Medium' ? 'secondary' : 'default'
                            }
                            className="text-xs"
                          >
                            {severity}: {count}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          {/* Coverage Summary */}
          <div className="mt-6 pt-6 border-t">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="text-center bg-blue-50 rounded-lg p-4">
                <div className="text-2xl font-bold text-blue-600">
                  {owaspCategories.filter(cat => coverage[cat.id]).length}
                </div>
                <div className="text-sm text-gray-600">Categories Covered</div>
              </div>
              <div className="text-center bg-green-50 rounded-lg p-4">
                <div className="text-2xl font-bold text-green-600">
                  {Object.values(coverage).reduce((acc, stats) => acc + stats.issuesFound, 0)}
                </div>
                <div className="text-sm text-gray-600">Total Issues Found</div>
              </div>
              <div className="text-center bg-purple-50 rounded-lg p-4">
                <div className="text-2xl font-bold text-purple-600">
                  {Math.round(
                    Object.values(coverage).reduce(
                      (acc, stats) => acc + stats.avgConfidence, 0
                    ) / Object.values(coverage).length * 100
                  )}%
                </div>
                <div className="text-sm text-gray-600">Avg Confidence</div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default OwaspCoverage;