import React, { useState, useMemo } from 'react';
import { SecurityIssue } from '../../types/security';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/Card';
import { Badge } from '../ui/Badge';
import { Input } from '../ui/Input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/Select';
import { AlertTriangle, AlertCircle, Info, CheckCircle, MapPin, ExternalLink, Search } from 'lucide-react';

interface SecurityIssuesListProps {
  issues: SecurityIssue[];
  onIssueSelect?: (issueId: string) => void;
  showAll?: boolean;
  className?: string;
}

export const SecurityIssuesList: React.FC<SecurityIssuesListProps> = ({
  issues,
  onIssueSelect,
  showAll = false,
  className = '',
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [severityFilter, setSeverityFilter] = useState('all');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const [sortBy, setSortBy] = useState('severity');

  const getSeverityIcon = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'critical':
        return <AlertTriangle className="h-4 w-4 text-red-500" />;
      case 'high':
        return <AlertCircle className="h-4 w-4 text-orange-500" />;
      case 'medium':
        return <Info className="h-4 w-4 text-yellow-500" />;
      default:
        return <CheckCircle className="h-4 w-4 text-green-500" />;
    }
  };

  const getSeverityBadgeVariant = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'critical':
      case 'high':
        return 'destructive';
      case 'medium':
        return 'secondary';
      default:
        return 'default';
    }
  };

  const filteredAndSortedIssues = useMemo(() => {
    let filtered = issues.filter(issue => {
      const matchesSearch = searchTerm === '' || 
        issue.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
        issue.issueType.toLowerCase().includes(searchTerm.toLowerCase()) ||
        issue.location.file.toLowerCase().includes(searchTerm.toLowerCase());

      const matchesSeverity = severityFilter === 'all' || 
        issue.severity.toLowerCase() === severityFilter.toLowerCase();

      const matchesCategory = categoryFilter === 'all' || 
        issue.owaspCategory === categoryFilter;

      return matchesSearch && matchesSeverity && matchesCategory;
    });

    // Sort issues
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'severity':
          const severityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
          return (severityOrder[b.severity.toLowerCase() as keyof typeof severityOrder] || 0) - 
                 (severityOrder[a.severity.toLowerCase() as keyof typeof severityOrder] || 0);
        case 'confidence':
          return b.confidenceScore - a.confidenceScore;
        case 'file':
          return a.location.file.localeCompare(b.location.file);
        case 'type':
          return a.issueType.localeCompare(b.issueType);
        default:
          return 0;
      }
    });

    return showAll ? filtered : filtered.slice(0, 10);
  }, [issues, searchTerm, severityFilter, categoryFilter, sortBy, showAll]);

  const uniqueCategories = useMemo(() => {
    const categories = new Set(issues.map(issue => issue.owaspCategory).filter(Boolean));
    return Array.from(categories);
  }, [issues]);

  return (
    <div className={className}>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>Security Issues ({filteredAndSortedIssues.length})</span>
            {!showAll && issues.length > 10 && (
              <Badge variant="outline">
                Showing {filteredAndSortedIssues.length} of {issues.length}
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {/* Filters */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            <div className="relative">
              <Search className="absolute left-3 top-3 h-4 w-4 text-gray-400" />
              <Input
                placeholder="Search issues..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select value={severityFilter} onValueChange={setSeverityFilter}>
              <SelectTrigger>
                <SelectValue placeholder="Filter by severity" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Severities</SelectItem>
                <SelectItem value="critical">Critical</SelectItem>
                <SelectItem value="high">High</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="low">Low</SelectItem>
              </SelectContent>
            </Select>
            <Select value={categoryFilter} onValueChange={setCategoryFilter}>
              <SelectTrigger>
                <SelectValue placeholder="Filter by OWASP category" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {uniqueCategories.map(category => (
                  <SelectItem key={category} value={category}>
                    {category?.replace(/_/g, ' ').replace(/^A\d+_/, '')}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={sortBy} onValueChange={setSortBy}>
              <SelectTrigger>
                <SelectValue placeholder="Sort by" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="severity">Severity</SelectItem>
                <SelectItem value="confidence">Confidence</SelectItem>
                <SelectItem value="file">File</SelectItem>
                <SelectItem value="type">Issue Type</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Issues List */}
          <div className="space-y-4">
            {filteredAndSortedIssues.map((issue) => (
              <div
                key={issue.id}
                className="border rounded-lg p-4 hover:bg-gray-50 transition-colors cursor-pointer"
                onClick={() => onIssueSelect?.(issue.id)}
              >
                {/* Issue Header */}
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-3">
                    {getSeverityIcon(issue.severity)}
                    <div>
                      <h4 className="font-medium text-gray-900">{issue.issueType}</h4>
                      <div className="flex items-center gap-2 mt-1">
                        <Badge variant={getSeverityBadgeVariant(issue.severity)}>
                          {issue.severity}
                        </Badge>
                        {issue.owaspCategory && (
                          <Badge variant="outline">
                            {issue.owaspCategory.replace(/_/g, ' ').replace(/^A\d+_/, '')}
                          </Badge>
                        )}
                        {issue.cweId && (
                          <Badge variant="outline">
                            {issue.cweId}
                          </Badge>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-medium text-gray-700">
                      {Math.round(issue.confidenceScore * 100)}% confidence
                    </div>
                    {issue.cvssScore && (
                      <div className="text-sm text-gray-600">
                        CVSS: {issue.cvssScore}
                      </div>
                    )}
                  </div>
                </div>

                {/* Issue Description */}
                <p className="text-gray-700 mb-3 line-clamp-2">
                  {issue.description}
                </p>

                {/* Location */}
                <div className="flex items-center gap-2 text-sm text-gray-600 mb-3">
                  <MapPin className="h-4 w-4" />
                  <span className="font-mono">
                    {issue.location.file}:{issue.location.startLine}
                  </span>
                </div>

                {/* Remediation */}
                <div className="bg-blue-50 rounded-md p-3 mb-3">
                  <h5 className="text-sm font-medium text-blue-900 mb-1">Remediation:</h5>
                  <p className="text-sm text-blue-800 line-clamp-2">{issue.remediation}</p>
                </div>

                {/* References */}
                {issue.references && issue.references.length > 0 && (
                  <div className="flex items-center gap-2 text-sm">
                    <span className="text-gray-600">References:</span>
                    {issue.references.slice(0, 2).map((ref, index) => (
                      <a
                        key={index}
                        href={ref}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-600 hover:text-blue-800 flex items-center gap-1"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <ExternalLink className="h-3 w-3" />
                        Link {index + 1}
                      </a>
                    ))}
                    {issue.references.length > 2 && (
                      <span className="text-gray-500">
                        +{issue.references.length - 2} more
                      </span>
                    )}
                  </div>
                )}

                {/* Taint Flows */}
                {issue.relatedTaintFlows && issue.relatedTaintFlows.length > 0 && (
                  <div className="mt-3 pt-3 border-t">
                    <div className="text-sm text-gray-600">
                      {issue.relatedTaintFlows.length} related data flow(s) detected
                    </div>
                  </div>
                )}
              </div>
            ))}

            {filteredAndSortedIssues.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                <AlertCircle className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <h3 className="text-lg font-medium mb-2">No issues found</h3>
                <p>No security issues match your current filters.</p>
              </div>
            )}
          </div>

          {/* Show More Button */}
          {!showAll && issues.length > 10 && filteredAndSortedIssues.length === 10 && (
            <div className="text-center mt-6">
              <button
                onClick={() => onIssueSelect?.('show-all')}
                className="px-4 py-2 text-blue-600 hover:text-blue-800 font-medium"
              >
                View All {issues.length} Security Issues
              </button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default SecurityIssuesList;