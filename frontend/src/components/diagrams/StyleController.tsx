import React, { useState, useRef, useEffect } from 'react';
import { ChevronDown, Palette, Settings, Eye, EyeOff } from 'lucide-react';
import { ThemeManager, type DiagramTheme } from './ThemeManager';
import { ComponentStylingEngine, type ConditionalStyle, type MetricVisualization } from './ComponentStylingEngine';
import type { NodeEventData } from './types';

export interface StyleControllerProps {
  svgRef: React.RefObject<SVGSVGElement>;
  nodeData: NodeEventData[];
  onThemeChange?: (theme: DiagramTheme) => void;
  onStyleUpdate?: () => void;
}

interface ThemePreset {
  id: string;
  name: string;
  description: string;
  theme: DiagramTheme;
}

export const StyleController: React.FC<StyleControllerProps> = ({
  svgRef,
  nodeData,
  onThemeChange,
  onStyleUpdate
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'themes' | 'conditions' | 'metrics'>('themes');
  const [themeManager, setThemeManager] = useState<ThemeManager | null>(null);
  const [stylingEngine, setStylingEngine] = useState<ComponentStylingEngine | null>(null);
  const [conditionalStyles, setConditionalStyles] = useState<ConditionalStyle[]>([]);
  const [metricVisualizations, setMetricVisualizations] = useState<MetricVisualization[]>([]);
  const [selectedTheme, setSelectedTheme] = useState<string>('default');

  // Initialize managers when SVG is available
  useEffect(() => {
    if (svgRef.current) {
      const svg = svgRef.current;
      
      // Initialize theme manager
      const newThemeManager = new ThemeManager(svg);
      setThemeManager(newThemeManager);
      
      // Initialize styling engine
      const newStylingEngine = new ComponentStylingEngine(svg, newThemeManager.getCurrentTheme());
      setStylingEngine(newStylingEngine);
      
      // Load initial conditional styles
      setConditionalStyles(newStylingEngine.getConditionalStyles());
      
      // Load initial metric visualizations
      setMetricVisualizations(newStylingEngine.getMetricVisualizations());

      return () => {
        newThemeManager.dispose();
        newStylingEngine.dispose();
      };
    }
  }, [svgRef.current]);

  // Theme presets
  const themePresets: ThemePreset[] = [
    {
      id: 'default',
      name: 'Default',
      description: 'Clean, professional theme',
      theme: themeManager?.getCurrentTheme() || getDefaultTheme()
    },
    {
      id: 'dark',
      name: 'Dark Mode',
      description: 'Dark theme for low-light environments',
      theme: getDarkTheme()
    },
    {
      id: 'colorful',
      name: 'Colorful',
      description: 'Vibrant colors for presentations',
      theme: getColorfulTheme()
    },
    {
      id: 'minimal',
      name: 'Minimal',
      description: 'Clean and minimal design',
      theme: getMinimalTheme()
    },
    {
      id: 'high-contrast',
      name: 'High Contrast',
      description: 'High contrast for accessibility',
      theme: getHighContrastTheme()
    }
  ];

  const handleThemeChange = (themeId: string) => {
    const preset = themePresets.find(p => p.id === themeId);
    if (preset && themeManager) {
      themeManager.setTheme(preset.theme);
      stylingEngine?.updateTheme(preset.theme);
      setSelectedTheme(themeId);
      onThemeChange?.(preset.theme);
      onStyleUpdate?.();
    }
  };

  const handleConditionalStyleToggle = (styleId: string, enabled: boolean) => {
    if (stylingEngine) {
      stylingEngine.enableConditionalStyle(styleId, enabled);
      stylingEngine.applyComponentStyling(nodeData);
      
      // Update local state
      setConditionalStyles(prev => 
        prev.map(style => 
          style.id === styleId ? { ...style, enabled } : style
        )
      );
      
      onStyleUpdate?.();
    }
  };

  const addMetricVisualization = (visualization: MetricVisualization) => {
    if (stylingEngine) {
      stylingEngine.addMetricVisualization(visualization);
      stylingEngine.applyComponentStyling(nodeData);
      setMetricVisualizations(prev => [...prev, visualization]);
      onStyleUpdate?.();
    }
  };

  const removeMetricVisualization = (metric: string) => {
    if (stylingEngine) {
      stylingEngine.removeMetricVisualization(metric as any);
      stylingEngine.applyComponentStyling(nodeData);
      setMetricVisualizations(prev => prev.filter(v => v.metric !== metric));
      onStyleUpdate?.();
    }
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="absolute top-4 right-4 bg-white/90 backdrop-blur-sm rounded-lg p-3 shadow-lg hover:bg-white transition-colors"
        title="Style Controls"
      >
        <Palette size={20} className="text-gray-700" />
      </button>
    );
  }

  return (
    <div className="absolute top-4 right-4 bg-white/95 backdrop-blur-sm rounded-lg shadow-lg border border-gray-200 w-80 max-h-96 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <h3 className="text-lg font-semibold text-gray-800 flex items-center gap-2">
          <Palette size={20} />
          Style Controls
        </h3>
        <button
          onClick={() => setIsOpen(false)}
          className="text-gray-500 hover:text-gray-700 transition-colors"
        >
          ✕
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200">
        {[
          { id: 'themes', label: 'Themes', icon: Palette },
          { id: 'conditions', label: 'Conditions', icon: Settings },
          { id: 'metrics', label: 'Metrics', icon: Eye }
        ].map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as any)}
            className={`flex-1 px-3 py-2 text-sm font-medium transition-colors flex items-center justify-center gap-1 ${
              activeTab === tab.id
                ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                : 'text-gray-600 hover:text-gray-800'
            }`}
          >
            <tab.icon size={16} />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="p-4 max-h-64 overflow-y-auto">
        {activeTab === 'themes' && (
          <ThemesTab
            themes={themePresets}
            selectedTheme={selectedTheme}
            onThemeChange={handleThemeChange}
          />
        )}

        {activeTab === 'conditions' && (
          <ConditionalStylesTab
            styles={conditionalStyles}
            onToggle={handleConditionalStyleToggle}
          />
        )}

        {activeTab === 'metrics' && (
          <MetricsTab
            visualizations={metricVisualizations}
            onAdd={addMetricVisualization}
            onRemove={removeMetricVisualization}
          />
        )}
      </div>
    </div>
  );
};

// Theme Tab Component
const ThemesTab: React.FC<{
  themes: ThemePreset[];
  selectedTheme: string;
  onThemeChange: (themeId: string) => void;
}> = ({ themes, selectedTheme, onThemeChange }) => (
  <div className="space-y-2">
    <h4 className="font-medium text-gray-700 mb-3">Choose Theme</h4>
    {themes.map(theme => (
      <div
        key={theme.id}
        className={`p-3 rounded-lg border-2 cursor-pointer transition-all ${
          selectedTheme === theme.id
            ? 'border-blue-500 bg-blue-50'
            : 'border-gray-200 hover:border-gray-300'
        }`}
        onClick={() => onThemeChange(theme.id)}
      >
        <div className="flex items-center justify-between">
          <div>
            <h5 className="font-medium text-gray-800">{theme.name}</h5>
            <p className="text-sm text-gray-600">{theme.description}</p>
          </div>
          {selectedTheme === theme.id && (
            <div className="w-4 h-4 bg-blue-500 rounded-full flex items-center justify-center">
              <div className="w-2 h-2 bg-white rounded-full"></div>
            </div>
          )}
        </div>
        
        {/* Color palette preview */}
        <div className="flex gap-1 mt-2">
          {Object.entries(theme.theme.colors).slice(0, 6).map(([key, color]) => (
            <div
              key={key}
              className="w-4 h-4 rounded border border-gray-300"
              style={{ backgroundColor: color }}
              title={key}
            />
          ))}
        </div>
      </div>
    ))}
  </div>
);

// Conditional Styles Tab Component
const ConditionalStylesTab: React.FC<{
  styles: ConditionalStyle[];
  onToggle: (styleId: string, enabled: boolean) => void;
}> = ({ styles, onToggle }) => (
  <div className="space-y-2">
    <h4 className="font-medium text-gray-700 mb-3">Conditional Styling</h4>
    {styles.map(style => (
      <div
        key={style.id}
        className="flex items-start justify-between p-3 rounded-lg border border-gray-200"
      >
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <h5 className="font-medium text-gray-800">{style.name}</h5>
            <span className={`px-2 py-1 text-xs rounded-full ${
              style.priority >= 90 ? 'bg-red-100 text-red-600' :
              style.priority >= 80 ? 'bg-yellow-100 text-yellow-600' :
              'bg-blue-100 text-blue-600'
            }`}>
              P{style.priority}
            </span>
          </div>
          <p className="text-sm text-gray-600 mt-1">{style.description}</p>
        </div>
        <button
          onClick={() => onToggle(style.id, !style.enabled)}
          className={`ml-3 p-1 rounded transition-colors ${
            style.enabled
              ? 'text-green-600 hover:text-green-700'
              : 'text-gray-400 hover:text-gray-600'
          }`}
        >
          {style.enabled ? <Eye size={16} /> : <EyeOff size={16} />}
        </button>
      </div>
    ))}
  </div>
);

// Metrics Tab Component
const MetricsTab: React.FC<{
  visualizations: MetricVisualization[];
  onAdd: (visualization: MetricVisualization) => void;
  onRemove: (metric: string) => void;
}> = ({ visualizations, onAdd, onRemove }) => {
  const [showAddForm, setShowAddForm] = useState(false);

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between mb-3">
        <h4 className="font-medium text-gray-700">Metric Visualizations</h4>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          className="text-sm bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 transition-colors"
        >
          {showAddForm ? 'Cancel' : 'Add'}
        </button>
      </div>

      {showAddForm && (
        <AddMetricForm
          onAdd={(viz) => {
            onAdd(viz);
            setShowAddForm(false);
          }}
          onCancel={() => setShowAddForm(false)}
        />
      )}

      {visualizations.map(viz => (
        <div
          key={viz.metric}
          className="flex items-center justify-between p-3 rounded-lg border border-gray-200"
        >
          <div>
            <h5 className="font-medium text-gray-800 capitalize">
              {viz.metric.replace(/([A-Z])/g, ' $1').trim()}
            </h5>
            <p className="text-sm text-gray-600">
              {viz.visualization} · {viz.scale.type} scale
            </p>
          </div>
          <button
            onClick={() => onRemove(viz.metric)}
            className="text-red-500 hover:text-red-700 transition-colors"
          >
            ✕
          </button>
        </div>
      ))}

      {visualizations.length === 0 && !showAddForm && (
        <p className="text-gray-500 text-center py-4">
          No metric visualizations configured
        </p>
      )}
    </div>
  );
};

// Add Metric Form Component
const AddMetricForm: React.FC<{
  onAdd: (visualization: MetricVisualization) => void;
  onCancel: () => void;
}> = ({ onAdd, onCancel }) => {
  const [metric, setMetric] = useState('complexity');
  const [visualization, setVisualization] = useState('color');
  const [scaleType, setScaleType] = useState('linear');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const newVisualization: MetricVisualization = {
      metric: metric as any,
      visualization: visualization as any,
      scale: {
        type: scaleType as any,
        domain: [0, 1],
        range: visualization === 'color' 
          ? ['#10b981', '#f59e0b', '#ef4444']
          : [0.5, 1, 1.5]
      }
    };

    onAdd(newVisualization);
  };

  return (
    <form onSubmit={handleSubmit} className="p-3 bg-gray-50 rounded-lg space-y-3">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Metric
        </label>
        <select
          value={metric}
          onChange={(e) => setMetric(e.target.value)}
          className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
        >
          <option value="complexity">Complexity</option>
          <option value="coupling">Coupling</option>
          <option value="testCoverage">Test Coverage</option>
          <option value="performance">Performance</option>
          <option value="security">Security</option>
          <option value="maintainability">Maintainability</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Visualization
        </label>
        <select
          value={visualization}
          onChange={(e) => setVisualization(e.target.value)}
          className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
        >
          <option value="color">Color</option>
          <option value="size">Size</option>
          <option value="border">Border</option>
          <option value="opacity">Opacity</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Scale Type
        </label>
        <select
          value={scaleType}
          onChange={(e) => setScaleType(e.target.value)}
          className="w-full px-3 py-1 border border-gray-300 rounded text-sm"
        >
          <option value="linear">Linear</option>
          <option value="logarithmic">Logarithmic</option>
          <option value="threshold">Threshold</option>
        </select>
      </div>

      <div className="flex gap-2">
        <button
          type="submit"
          className="flex-1 bg-blue-500 text-white px-3 py-1 rounded text-sm hover:bg-blue-600 transition-colors"
        >
          Add
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="flex-1 bg-gray-300 text-gray-700 px-3 py-1 rounded text-sm hover:bg-gray-400 transition-colors"
        >
          Cancel
        </button>
      </div>
    </form>
  );
};

// Theme factories
function getDefaultTheme(): DiagramTheme {
  // This would return the default theme - simplified for brevity
  return {
    id: 'default',
    name: 'Default',
    colors: {
      primary: '#3b82f6',
      secondary: '#6366f1',
      accent: '#8b5cf6',
      background: '#ffffff',
      surface: '#f8fafc',
      text: '#1f2937',
      textSecondary: '#6b7280',
      success: '#10b981',
      warning: '#f59e0b',
      error: '#ef4444',
      info: '#06b6d4'
    },
    nodeStyles: {} as any,
    edgeStyles: {} as any,
    layout: {} as any,
    effects: {} as any
  };
}

function getDarkTheme(): DiagramTheme {
  const defaultTheme = getDefaultTheme();
  return {
    ...defaultTheme,
    id: 'dark',
    name: 'Dark',
    colors: {
      ...defaultTheme.colors,
      background: '#1f2937',
      surface: '#374151',
      text: '#f9fafb',
      textSecondary: '#d1d5db'
    }
  };
}

function getColorfulTheme(): DiagramTheme {
  const defaultTheme = getDefaultTheme();
  return {
    ...defaultTheme,
    id: 'colorful',
    name: 'Colorful',
    colors: {
      ...defaultTheme.colors,
      primary: '#8b5cf6',
      secondary: '#06b6d4',
      accent: '#f59e0b'
    }
  };
}

function getMinimalTheme(): DiagramTheme {
  const defaultTheme = getDefaultTheme();
  return {
    ...defaultTheme,
    id: 'minimal',
    name: 'Minimal',
    colors: {
      ...defaultTheme.colors,
      primary: '#6b7280',
      secondary: '#9ca3af',
      accent: '#374151'
    }
  };
}

function getHighContrastTheme(): DiagramTheme {
  const defaultTheme = getDefaultTheme();
  return {
    ...defaultTheme,
    id: 'high-contrast',
    name: 'High Contrast',
    colors: {
      ...defaultTheme.colors,
      primary: '#000000',
      secondary: '#ffffff',
      text: '#000000',
      background: '#ffffff'
    }
  };
}

export default StyleController;