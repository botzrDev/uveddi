// Interactive Diagram Demo Page - UV-89 Phase 1
// Demonstrates the interactive diagram functionality

import React, { useState } from 'react';
import { InteractiveDiagram } from '../components/diagrams/InteractiveDiagram';
import type { NodeEventData } from '../components/diagrams/types';

const InteractiveDiagramDemo: React.FC = () => {
  const [selectedNodeInfo, setSelectedNodeInfo] = useState<NodeEventData | null>(null);
  const [eventLog, setEventLog] = useState<string[]>([]);
  const [currentDiagram, setCurrentDiagram] = useState('component');

  const logEvent = (event: string) => {
    setEventLog(prev => [
      `${new Date().toLocaleTimeString()}: ${event}`,
      ...prev.slice(0, 9) // Keep only last 10 events
    ]);
  };

  const handleNodeClick = (nodeId: string, nodeData: NodeEventData) => {
    setSelectedNodeInfo(nodeData);
    logEvent(`Clicked node: ${nodeData.componentData.name} (${nodeId})`);
  };

  const handleNodeDoubleClick = (nodeId: string, nodeData: NodeEventData) => {
    logEvent(`Double-clicked node: ${nodeData.componentData.name} (${nodeId})`);
    // Could trigger navigation to component details
  };

  const handleNodeHover = (nodeData: NodeEventData | null) => {
    if (nodeData) {
      logEvent(`Hovered node: ${nodeData.componentData.name}`);
    }
  };

  const handleNodeContextMenu = (nodeId: string, nodeData: NodeEventData, _event: MouseEvent) => {
    logEvent(`Right-clicked node: ${nodeData.componentData.name} (${nodeId})`);
  };

  const diagrams = {
    component: {
      title: 'Component Architecture',
      code: `
        graph TD
          A[Authentication Service] --> B[User Management]
          A --> C[Session Handler]
          B --> D[Profile Service]
          B --> E[Permissions]
          C --> F[Token Validator]
          D --> G[Database Layer]
          E --> G
          F --> H[Cache Service]
          G --> I[PostgreSQL]
          H --> J[Redis]
      `
    },
    analysis: {
      title: 'Analysis Flow',
      code: `
        graph LR
          A[Source Code] --> B[Parser]
          B --> C[AST Builder]
          C --> D[Analysis Engine]
          D --> E[Pattern Detector]
          D --> F[Metrics Collector]
          E --> G[Anti-pattern Reports]
          F --> H[Quality Metrics]
          G --> I[Report Generator]
          H --> I
          I --> J[Visualization]
      `
    },
    dependency: {
      title: 'Dependency Graph',
      code: `
        graph TD
          CLI[CLI Interface] --> Engine[Analysis Engine]
          Engine --> Parser[Code Parser]
          Engine --> Detector[Pattern Detector]
          Engine --> AI[AI Service]
          Parser --> TreeSitter[Tree-sitter]
          Detector --> Rules[Detection Rules]
          AI --> Ollama[Ollama Provider]
          Engine --> Report[Report Generator]
          Report --> Mermaid[Mermaid Renderer]
          Report --> Export[Export Service]
      `
    },
    sequence: {
      title: 'Analysis Sequence',
      code: `
        sequenceDiagram
          participant U as User
          participant C as CLI
          participant E as Engine
          participant P as Parser
          participant D as Detector
          participant R as Reporter
          
          U->>C: analyze command
          C->>E: start analysis
          E->>P: parse files
          P-->>E: AST nodes
          E->>D: detect patterns
          D-->>E: findings
          E->>R: generate report
          R-->>C: report data
          C-->>U: display results
      `
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Interactive Diagram Demo
          </h1>
          <p className="text-gray-600">
            UV-89 Phase 1: Interactive Elements Implementation
          </p>
        </div>

        {/* Controls */}
        <div className="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 className="text-xl font-semibold mb-4">Select Diagram</h2>
          <div className="flex flex-wrap gap-2">
            {(Object.entries(diagrams) as Array<[keyof typeof diagrams, typeof diagrams[keyof typeof diagrams]]>).map(([key, diagram]) => (
              <button
                key={key}
                onClick={() => setCurrentDiagram(key)}
                className={`px-4 py-2 rounded-md font-medium transition-colors ${
                  currentDiagram === key
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
                }`}
              >
                {diagram.title}
              </button>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* Main Diagram */}
          <div className="lg:col-span-3">
            <div className="bg-white rounded-lg shadow-md">
              <div className="p-4 border-b border-gray-200">
                <h2 className="text-xl font-semibold">
                  {diagrams[currentDiagram as keyof typeof diagrams].title}
                </h2>
                <p className="text-sm text-gray-600 mt-1">
                  Click nodes, right-click for context menu, use arrow keys to navigate
                </p>
              </div>
              <div className="h-96 lg:h-[600px]">
                <InteractiveDiagram
                  mermaidCode={diagrams[currentDiagram as keyof typeof diagrams].code}
                  onNodeClick={handleNodeClick}
                  onNodeDoubleClick={handleNodeDoubleClick}
                  onNodeHover={handleNodeHover}
                  onNodeContextMenu={handleNodeContextMenu}
                  enablePanZoom={true}
                  enableTooltips={true}
                  enableKeyboardNav={true}
                  className="w-full h-full"
                />
              </div>
            </div>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Selected Node Info */}
            <div className="bg-white rounded-lg shadow-md p-4">
              <h3 className="text-lg font-semibold mb-3">Selected Node</h3>
              {selectedNodeInfo ? (
                <div className="space-y-2 text-sm">
                  <div>
                    <span className="font-medium">Name:</span>{' '}
                    {selectedNodeInfo.componentData.name}
                  </div>
                  <div>
                    <span className="font-medium">Type:</span>{' '}
                    {selectedNodeInfo.componentData.componentType}
                  </div>
                  <div>
                    <span className="font-medium">Path:</span>{' '}
                    <code className="text-xs bg-gray-100 px-1 rounded">
                      {selectedNodeInfo.componentData.filePath}
                    </code>
                  </div>
                  <div>
                    <span className="font-medium">Connections:</span>{' '}
                    {selectedNodeInfo.connections.length}
                  </div>
                  <div>
                    <span className="font-medium">Position:</span>{' '}
                    ({Math.round(selectedNodeInfo.position.x)}, {Math.round(selectedNodeInfo.position.y)})
                  </div>
                </div>
              ) : (
                <p className="text-gray-500 text-sm">
                  Click on a node to see its details
                </p>
              )}
            </div>

            {/* Event Log */}
            <div className="bg-white rounded-lg shadow-md p-4">
              <h3 className="text-lg font-semibold mb-3">Event Log</h3>
              <div className="space-y-1 text-xs font-mono max-h-48 overflow-y-auto">
                {eventLog.length > 0 ? (
                  eventLog.map((event, index) => (
                    <div 
                      key={index} 
                      className={`p-2 rounded ${
                        index === 0 ? 'bg-blue-50 text-blue-800' : 'text-gray-600'
                      }`}
                    >
                      {event}
                    </div>
                  ))
                ) : (
                  <p className="text-gray-500">
                    Interact with the diagram to see events
                  </p>
                )}
              </div>
              <button
                onClick={() => setEventLog([])}
                className="mt-2 text-xs text-blue-600 hover:text-blue-800"
              >
                Clear Log
              </button>
            </div>

            {/* Features */}
            <div className="bg-white rounded-lg shadow-md p-4">
              <h3 className="text-lg font-semibold mb-3">Features</h3>
              <div className="space-y-2 text-sm">
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                  <span>Click nodes</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                  <span>Double-click activation</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                  <span>Hover tooltips</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                  <span>Right-click context menu</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                  <span>Pan & zoom</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                  <span>Keyboard navigation</span>
                </div>
              </div>
            </div>

            {/* Keyboard Shortcuts */}
            <div className="bg-white rounded-lg shadow-md p-4">
              <h3 className="text-lg font-semibold mb-3">Keyboard Shortcuts</h3>
              <div className="space-y-1 text-xs">
                <div className="flex justify-between">
                  <span>Arrow Keys</span>
                  <span className="text-gray-500">Navigate</span>
                </div>
                <div className="flex justify-between">
                  <span>Enter/Space</span>
                  <span className="text-gray-500">Activate</span>
                </div>
                <div className="flex justify-between">
                  <span>Tab</span>
                  <span className="text-gray-500">Next node</span>
                </div>
                <div className="flex justify-between">
                  <span>F</span>
                  <span className="text-gray-500">Focus</span>
                </div>
                <div className="flex justify-between">
                  <span>?</span>
                  <span className="text-gray-500">Help</span>
                </div>
                <div className="flex justify-between">
                  <span>Esc</span>
                  <span className="text-gray-500">Clear</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Technical Details */}
        <div className="mt-8 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">Technical Implementation</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-sm">
            <div>
              <h3 className="font-semibold text-blue-600 mb-2">Event Handling</h3>
              <ul className="space-y-1 text-gray-600">
                <li>• D3.js event system integration</li>
                <li>• Click vs double-click detection</li>
                <li>• Hover state management</li>
                <li>• Context menu positioning</li>
              </ul>
            </div>
            <div>
              <h3 className="font-semibold text-blue-600 mb-2">Navigation</h3>
              <ul className="space-y-1 text-gray-600">
                <li>• Pan & zoom with d3-zoom</li>
                <li>• Smooth transitions</li>
                <li>• Viewport constraints</li>
                <li>• Focus on node functionality</li>
              </ul>
            </div>
            <div>
              <h3 className="font-semibold text-blue-600 mb-2">Accessibility</h3>
              <ul className="space-y-1 text-gray-600">
                <li>• Full keyboard navigation</li>
                <li>• ARIA labels and roles</li>
                <li>• Screen reader support</li>
                <li>• Focus management</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default InteractiveDiagramDemo;