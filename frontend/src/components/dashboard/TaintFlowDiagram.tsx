import React, { useMemo, useState } from 'react';
import { TaintFlow, FlowNode } from '../../types/security';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/Card';
import { Badge } from '../ui/Badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/Select';
import { GitBranch, ArrowRight, Shield, AlertTriangle, Filter } from 'lucide-react';

interface TaintFlowDiagramProps {
  taintFlows: TaintFlow[];
  className?: string;
}

export const TaintFlowDiagram: React.FC<TaintFlowDiagramProps> = ({
  taintFlows,
  className = '',
}) => {
  const [selectedFlow, setSelectedFlow] = useState<string>('all');
  const [vulnerabilityFilter, setVulnerabilityFilter] = useState<string>('all');

  const filteredFlows = useMemo(() => {
    let flows = taintFlows;

    if (selectedFlow !== 'all') {
      flows = flows.filter(flow => flow.id === selectedFlow);
    }

    if (vulnerabilityFilter !== 'all') {
      flows = flows.filter(flow => flow.vulnerabilityType === vulnerabilityFilter);
    }

    return flows.sort((a, b) => b.confidence - a.confidence);
  }, [taintFlows, selectedFlow, vulnerabilityFilter]);

  const vulnerabilityTypes = useMemo(() => {
    const types = new Set(taintFlows.map(flow => flow.vulnerabilityType));
    return Array.from(types);
  }, [taintFlows]);

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.9) return 'text-red-600';
    if (confidence >= 0.7) return 'text-orange-600';
    if (confidence >= 0.5) return 'text-yellow-600';
    return 'text-gray-600';
  };

  const getConfidenceBadgeVariant = (confidence: number) => {
    if (confidence >= 0.9) return 'destructive';
    if (confidence >= 0.7) return 'destructive';
    if (confidence >= 0.5) return 'secondary';
    return 'default';
  };

  const getNodeIcon = (nodeType: string) => {
    switch (nodeType.toLowerCase()) {
      case 'source':
        return '🔴';
      case 'sink':
        return '🎯';
      case 'sanitizer':
        return '🛡️';
      default:
        return '⚪';
    }
  };

  const renderFlowNode = (node: FlowNode, isLast: boolean = false) => (
    <div key={`${node.location}-${node.lineNumber}`} className="flex items-center">
      <div className="flex items-center bg-white border rounded-lg p-3 shadow-sm min-w-0 flex-1">
        <div className="text-lg mr-3">{getNodeIcon(node.nodeType)}</div>
        <div className="min-w-0 flex-1">
          <div className="font-medium text-gray-900 truncate">{node.name}</div>
          <div className="text-sm text-gray-600 font-mono truncate">{node.location}:{node.lineNumber}</div>
          <Badge variant="outline" className="text-xs mt-1">
            {node.nodeType}
          </Badge>
        </div>
      </div>
      {!isLast && (
        <ArrowRight className="h-5 w-5 text-gray-400 mx-2 flex-shrink-0" />
      )}
    </div>
  );

  return (
    <div className={className}>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <GitBranch className="h-5 w-5 text-blue-600" />
            Data Flow Analysis ({taintFlows.length} flows)
          </CardTitle>
        </CardHeader>
        <CardContent>
          {/* Filters */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
            <Select value={selectedFlow} onValueChange={setSelectedFlow}>
              <SelectTrigger>
                <SelectValue placeholder="Select specific flow" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Flows</SelectItem>
                {taintFlows.map((flow, index) => (
                  <SelectItem key={flow.id} value={flow.id}>
                    Flow #{index + 1} - {flow.vulnerabilityType} ({Math.round(flow.confidence * 100)}%)
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={vulnerabilityFilter} onValueChange={setVulnerabilityFilter}>
              <SelectTrigger>
                <SelectValue placeholder="Filter by vulnerability type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Vulnerability Types</SelectItem>
                {vulnerabilityTypes.map(type => (
                  <SelectItem key={type} value={type}>
                    {type}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Flow Diagrams */}
          <div className="space-y-6">
            {filteredFlows.map((flow, flowIndex) => (
              <div key={flow.id} className="border rounded-lg p-4 bg-gray-50">
                {/* Flow Header */}
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-3">
                    <h4 className="font-semibold text-gray-900">
                      Flow #{flowIndex + 1}: {flow.vulnerabilityType}
                    </h4>
                    <Badge variant={getConfidenceBadgeVariant(flow.confidence)}>
                      {Math.round(flow.confidence * 100)}% confidence
                    </Badge>
                  </div>
                  {flow.sanitizers.length > 0 && (
                    <div className="flex items-center gap-2 text-sm text-green-600">
                      <Shield className="h-4 w-4" />
                      {flow.sanitizers.length} sanitizer(s)
                    </div>
                  )}
                </div>

                {/* Flow Path */}
                <div className="bg-white rounded-lg p-4 border-2 border-dashed border-gray-200">
                  <div className="text-sm font-medium text-gray-700 mb-3">Data Flow Path:</div>
                  
                  {/* Source to Sink Flow */}
                  <div className="flex flex-wrap items-center gap-2 mb-4">
                    {renderFlowNode(flow.source)}
                    
                    {/* Intermediate path nodes */}
                    {flow.path.map((node, index) => (
                      <React.Fragment key={`path-${index}`}>
                        {renderFlowNode(node)}
                      </React.Fragment>
                    ))}
                    
                    {renderFlowNode(flow.sink, true)}
                  </div>

                  {/* Sanitizers */}
                  {flow.sanitizers.length > 0 && (
                    <div className="mt-4 pt-4 border-t">
                      <div className="text-sm font-medium text-gray-700 mb-2 flex items-center gap-2">
                        <Shield className="h-4 w-4 text-green-600" />
                        Sanitizers in Path:
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                        {flow.sanitizers.map((sanitizer, index) => (
                          <div key={index} className="bg-green-50 border border-green-200 rounded-lg p-3">
                            <div className="font-medium text-green-900">{sanitizer.name}</div>
                            <div className="text-sm text-green-700 font-mono">
                              {sanitizer.location}:{sanitizer.lineNumber}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>

                {/* Flow Analysis */}
                <div className="mt-4 grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                  <div className="bg-blue-50 rounded-lg p-3">
                    <div className="font-medium text-blue-900">Source Analysis</div>
                    <div className="text-blue-800">
                      Type: {flow.source.nodeType}<br />
                      Location: {flow.source.location.split('/').pop()}
                    </div>
                  </div>
                  <div className="bg-red-50 rounded-lg p-3">
                    <div className="font-medium text-red-900">Sink Analysis</div>
                    <div className="text-red-800">
                      Type: {flow.sink.nodeType}<br />
                      Vulnerability: {flow.vulnerabilityType}
                    </div>
                  </div>
                  <div className="bg-purple-50 rounded-lg p-3">
                    <div className="font-medium text-purple-900">Risk Assessment</div>
                    <div className="text-purple-800">
                      Confidence: {Math.round(flow.confidence * 100)}%<br />
                      Sanitized: {flow.sanitizers.length > 0 ? 'Yes' : 'No'}
                    </div>
                  </div>
                </div>
              </div>
            ))}

            {filteredFlows.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                <Filter className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <h3 className="text-lg font-medium mb-2">No flows found</h3>
                <p>No data flows match your current filters.</p>
              </div>
            )}
          </div>

          {/* Flow Summary */}
          {taintFlows.length > 0 && (
            <div className="mt-6 pt-6 border-t">
              <h4 className="font-medium text-gray-900 mb-3">Flow Analysis Summary</h4>
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div className="text-center bg-gray-50 rounded-lg p-3">
                  <div className="text-2xl font-bold text-gray-900">{taintFlows.length}</div>
                  <div className="text-sm text-gray-600">Total Flows</div>
                </div>
                <div className="text-center bg-red-50 rounded-lg p-3">
                  <div className="text-2xl font-bold text-red-600">
                    {taintFlows.filter(f => f.confidence >= 0.8).length}
                  </div>
                  <div className="text-sm text-gray-600">High Confidence</div>
                </div>
                <div className="text-center bg-green-50 rounded-lg p-3">
                  <div className="text-2xl font-bold text-green-600">
                    {taintFlows.filter(f => f.sanitizers.length > 0).length}
                  </div>
                  <div className="text-sm text-gray-600">With Sanitizers</div>
                </div>
                <div className="text-center bg-blue-50 rounded-lg p-3">
                  <div className="text-2xl font-bold text-blue-600">{vulnerabilityTypes.length}</div>
                  <div className="text-sm text-gray-600">Vulnerability Types</div>
                </div>
              </div>
            </div>
          )}

          {/* Legend */}
          <div className="mt-6 pt-6 border-t">
            <h4 className="font-medium text-gray-900 mb-3">Legend</h4>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div className="flex items-center gap-2">
                <span className="text-lg">🔴</span>
                <span>Source (Input)</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-lg">🎯</span>
                <span>Sink (Output)</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-lg">🛡️</span>
                <span>Sanitizer</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-lg">⚪</span>
                <span>Intermediate</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default TaintFlowDiagram;