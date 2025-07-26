import React, { useState, useRef } from 'react';
import { Download, Settings, FileText, Image, File, Code, Database } from 'lucide-react';
import { ExportManager, type ExportOptions, type HTMLExportOptions, type PlantUMLOptions, type GraphvizOptions } from './ExportManager';
import type { DiagramTheme } from './ThemeManager';
import type { NodeEventData } from './types';

export interface ExportControllerProps {
  svgRef: React.RefObject<SVGSVGElement>;
  nodeData: NodeEventData[];
  theme?: DiagramTheme;
  mermaidCode?: string;
  onExportStart?: (format: string) => void;
  onExportComplete?: (format: string, success: boolean) => void;
  onExportError?: (format: string, error: Error) => void;
}

interface ExportFormat {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  fileExtension: string;
  options?: string[];
}

export const ExportController: React.FC<ExportControllerProps> = ({
  svgRef,
  nodeData,
  theme,
  mermaidCode,
  onExportStart,
  onExportComplete,
  onExportError
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedFormat, setSelectedFormat] = useState<string>('svg');
  const [isExporting, setIsExporting] = useState(false);
  const [exportOptions, setExportOptions] = useState<Partial<ExportOptions>>({
    includeInteractivity: true,
    includeStyles: true,
    quality: 0.9,
    backgroundColor: '#ffffff'
  });
  const exportManagerRef = useRef<ExportManager | null>(null);

  // Initialize export manager when SVG is available
  React.useEffect(() => {
    if (svgRef.current) {
      exportManagerRef.current = new ExportManager(
        svgRef.current,
        nodeData,
        theme,
        mermaidCode
      );
    }
  }, [svgRef.current, nodeData, theme, mermaidCode]);

  const exportFormats: ExportFormat[] = [
    {
      id: 'html',
      name: 'Interactive HTML',
      description: 'Standalone HTML file with full interactivity',
      icon: <FileText size={20} />,
      fileExtension: 'html',
      options: ['includeInteractivity', 'standalone', 'embedAssets']
    },
    {
      id: 'svg',
      name: 'SVG Vector',
      description: 'Scalable vector graphics format',
      icon: <Image size={20} />,
      fileExtension: 'svg',
      options: ['includeStyles', 'preserveViewBox']
    },
    {
      id: 'png',
      name: 'PNG Image',
      description: 'High-quality raster image',
      icon: <Image size={20} />,
      fileExtension: 'png',
      options: ['quality', 'width', 'height', 'backgroundColor']
    },
    {
      id: 'pdf',
      name: 'PDF Document',
      description: 'Portable document format',
      icon: <File size={20} />,
      fileExtension: 'pdf',
      options: ['width', 'height', 'backgroundColor']
    },
    {
      id: 'plantuml',
      name: 'PlantUML',
      description: 'PlantUML diagram source code',
      icon: <Code size={20} />,
      fileExtension: 'puml',
      options: ['diagramType', 'includeColors', 'theme']
    },
    {
      id: 'graphviz',
      name: 'Graphviz DOT',
      description: 'Graphviz DOT language format',
      icon: <Database size={20} />,
      fileExtension: 'dot',
      options: ['engine', 'rankdir', 'includeColors']
    },
    {
      id: 'json',
      name: 'JSON Data',
      description: 'Complete diagram data export',
      icon: <Code size={20} />,
      fileExtension: 'json',
      options: []
    }
  ];

  const handleExport = async () => {
    if (!exportManagerRef.current) {
      console.error('Export manager not initialized');
      return;
    }

    const format = selectedFormat as ExportOptions['format'];
    const filename = `diagram.${exportFormats.find(f => f.id === format)?.fileExtension || 'txt'}`;

    try {
      setIsExporting(true);
      onExportStart?.(format);

      const options: ExportOptions = {
        format,
        filename,
        ...exportOptions
      };

      const result = await exportManagerRef.current.exportDiagram(options);
      
      if (typeof result === 'string') {
        await exportManagerRef.current.downloadFile(result, filename);
      } else {
        await exportManagerRef.current.downloadFile(result, filename);
      }

      onExportComplete?.(format, true);
    } catch (error) {
      console.error('Export failed:', error);
      onExportError?.(format, error as Error);
      onExportComplete?.(format, false);
    } finally {
      setIsExporting(false);
    }
  };

  const updateExportOption = (key: string, value: any) => {
    setExportOptions(prev => ({
      ...prev,
      [key]: value
    }));
  };

  const selectedFormatData = exportFormats.find(f => f.id === selectedFormat);

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="absolute bottom-4 right-4 bg-blue-500 text-white rounded-lg p-3 shadow-lg hover:bg-blue-600 transition-colors"
        title="Export Diagram"
      >
        <Download size={20} />
      </button>
    );
  }

  return (
    <div className="absolute bottom-4 right-4 bg-white/95 backdrop-blur-sm rounded-lg shadow-lg border border-gray-200 w-96 max-h-[500px] overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <h3 className="text-lg font-semibold text-gray-800 flex items-center gap-2">
          <Download size={20} />
          Export Diagram
        </h3>
        <button
          onClick={() => setIsOpen(false)}
          className="text-gray-500 hover:text-gray-700 transition-colors"
        >
          ✕
        </button>
      </div>

      {/* Format Selection */}
      <div className="p-4 border-b border-gray-200">
        <h4 className="font-medium text-gray-700 mb-3">Select Format</h4>
        <div className="grid grid-cols-2 gap-2">
          {exportFormats.map(format => (
            <button
              key={format.id}
              onClick={() => setSelectedFormat(format.id)}
              className={`p-3 rounded-lg border-2 text-left transition-all ${
                selectedFormat === format.id
                  ? 'border-blue-500 bg-blue-50'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <div className="flex items-center gap-2 mb-1">
                {format.icon}
                <span className="font-medium text-sm">{format.name}</span>
              </div>
              <p className="text-xs text-gray-600">{format.description}</p>
            </button>
          ))}
        </div>
      </div>

      {/* Format-Specific Options */}
      {selectedFormatData && selectedFormatData.options && selectedFormatData.options.length > 0 && (
        <div className="p-4 border-b border-gray-200 max-h-48 overflow-y-auto">
          <h4 className="font-medium text-gray-700 mb-3 flex items-center gap-2">
            <Settings size={16} />
            {selectedFormatData.name} Options
          </h4>
          <div className="space-y-3">
            {selectedFormatData.options.map(option => (
              <FormatOption
                key={option}
                option={option}
                value={exportOptions[option as keyof ExportOptions]}
                onChange={(value) => updateExportOption(option, value)}
                format={selectedFormat}
              />
            ))}
          </div>
        </div>
      )}

      {/* Export Button */}
      <div className="p-4">
        <button
          onClick={handleExport}
          disabled={isExporting}
          className={`w-full py-3 px-4 rounded-lg font-medium transition-colors flex items-center justify-center gap-2 ${
            isExporting
              ? 'bg-gray-400 text-gray-600 cursor-not-allowed'
              : 'bg-blue-500 text-white hover:bg-blue-600'
          }`}
        >
          {isExporting ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-gray-600"></div>
              Exporting...
            </>
          ) : (
            <>
              <Download size={16} />
              Export as {selectedFormatData?.name}
            </>
          )}
        </button>
      </div>
    </div>
  );
};

// Component for rendering format-specific options
const FormatOption: React.FC<{
  option: string;
  value: any;
  onChange: (value: any) => void;
  format: string;
}> = ({ option, value, onChange, format }) => {
  const renderOptionInput = () => {
    switch (option) {
      case 'includeInteractivity':
      case 'includeStyles':
      case 'includeAnimations':
      case 'standalone':
      case 'embedAssets':
      case 'includeColors':
      case 'includeNotes':
      case 'preserveViewBox':
        return (
          <label className="flex items-center space-x-2">
            <input
              type="checkbox"
              checked={value || false}
              onChange={(e) => onChange(e.target.checked)}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700 capitalize">
              {option.replace(/([A-Z])/g, ' $1').trim()}
            </span>
          </label>
        );

      case 'quality':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Quality: {Math.round((value || 0.9) * 100)}%
            </label>
            <input
              type="range"
              min="0.1"
              max="1"
              step="0.1"
              value={value || 0.9}
              onChange={(e) => onChange(parseFloat(e.target.value))}
              className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
            />
          </div>
        );

      case 'width':
      case 'height':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              {option.charAt(0).toUpperCase() + option.slice(1)} (px)
            </label>
            <input
              type="number"
              value={value || ''}
              onChange={(e) => onChange(e.target.value ? parseInt(e.target.value) : undefined)}
              placeholder="Auto"
              className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
            />
          </div>
        );

      case 'backgroundColor':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Background Color
            </label>
            <div className="flex gap-2">
              <input
                type="color"
                value={value || '#ffffff'}
                onChange={(e) => onChange(e.target.value)}
                className="w-12 h-8 border border-gray-300 rounded cursor-pointer"
              />
              <input
                type="text"
                value={value || '#ffffff'}
                onChange={(e) => onChange(e.target.value)}
                className="flex-1 px-3 py-1 border border-gray-300 rounded text-sm"
              />
            </div>
          </div>
        );

      case 'diagramType':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Diagram Type
            </label>
            <select
              value={value || 'component'}
              onChange={(e) => onChange(e.target.value)}
              className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
            >
              <option value="component">Component</option>
              <option value="class">Class</option>
              <option value="sequence">Sequence</option>
              <option value="activity">Activity</option>
              <option value="deployment">Deployment</option>
            </select>
          </div>
        );

      case 'theme':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Theme
            </label>
            <select
              value={value || 'default'}
              onChange={(e) => onChange(e.target.value)}
              className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
            >
              <option value="default">Default</option>
              <option value="plain">Plain</option>
              <option value="sketchy">Sketchy</option>
              <option value="dark">Dark</option>
            </select>
          </div>
        );

      case 'engine':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Layout Engine
            </label>
            <select
              value={value || 'dot'}
              onChange={(e) => onChange(e.target.value)}
              className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
            >
              <option value="dot">Dot (Hierarchical)</option>
              <option value="neato">Neato (Spring Model)</option>
              <option value="fdp">FDP (Force Directed)</option>
              <option value="sfdp">SFDP (Scalable Force Directed)</option>
              <option value="circo">Circo (Circular)</option>
              <option value="twopi">Twopi (Radial)</option>
            </select>
          </div>
        );

      case 'rankdir':
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Direction
            </label>
            <select
              value={value || 'TB'}
              onChange={(e) => onChange(e.target.value)}
              className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
            >
              <option value="TB">Top to Bottom</option>
              <option value="BT">Bottom to Top</option>
              <option value="LR">Left to Right</option>
              <option value="RL">Right to Left</option>
            </select>
          </div>
        );

      default:
        return (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              {option.charAt(0).toUpperCase() + option.slice(1)}
            </label>
            <input
              type="text"
              value={value || ''}
              onChange={(e) => onChange(e.target.value)}
              className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
            />
          </div>
        );
    }
  };

  return <div>{renderOptionInput()}</div>;
};

export default ExportController;