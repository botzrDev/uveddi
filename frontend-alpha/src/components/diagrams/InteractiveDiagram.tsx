// Minimal stub for InteractiveDiagram
import React from 'react';
import type { NodeEventData } from './types';

export interface InteractiveDiagramProps {
  nodeData?: NodeEventData[];
  mermaidCode?: string;
  onNodeClick?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeDoubleClick?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeHover?: (nodeData: NodeEventData | null) => void;
  onNodeContextMenu?: (nodeId: string, nodeData: NodeEventData, event: MouseEvent) => void;
  enablePanZoom?: boolean;
  enableTooltips?: boolean;
  enableKeyboardNav?: boolean;
  className?: string;
}

export const InteractiveDiagram: React.FC<InteractiveDiagramProps> = ({ className }) => {
  return (
    <div className={className}>
      <div className="bg-gray-100 rounded-lg p-4 text-center">
        <p className="text-gray-600">Interactive Diagram (Alpha Version)</p>
        <p className="text-sm text-gray-500 mt-2">
          Advanced diagram features are coming soon in the beta release.
        </p>
      </div>
    </div>
  );
};