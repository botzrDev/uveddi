import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render } from '../../../test-utils/test-utils';
import { DashboardBuilder } from '../DashboardBuilder';

// Mock React Grid Layout
const mockLayoutProps = {
  onLayoutChange: jest.fn(),
  onDragStart: jest.fn(),
  onDragStop: jest.fn(),
  onResizeStart: jest.fn(),
  onResizeStop: jest.fn()
};

jest.mock('react-grid-layout', () => {
  const ResponsiveGridLayout = ({ children, onLayoutChange, ...props }: any) => (
    <div data-testid="grid-layout" {...mockLayoutProps}>
      {children}
    </div>
  );
  
  ResponsiveGridLayout.utils = {
    synchronizeLayoutWithChildren: jest.fn()
  };
  
  return {
    Responsive: ResponsiveGridLayout,
    WidthProvider: (component: any) => component
  };
});

describe('DashboardBuilder', () => {
  const mockProps = {
    availableWidgets: [
      {
        id: 'priority-matrix',
        name: 'Priority Matrix',
        description: 'Impact vs Effort Analysis',
        category: 'analysis',
        icon: 'scatter_plot'
      },
      {
        id: 'quality-score',
        name: 'Quality Score',
        description: 'Overall code quality metrics',
        category: 'metrics',
        icon: 'assessment'
      },
      {
        id: 'dependency-graph',
        name: 'Dependency Graph',
        description: 'Interactive dependency visualization',
        category: 'visualization',
        icon: 'account_tree'
      }
    ],
    currentLayout: [
      {
        i: 'priority-matrix',
        x: 0,
        y: 0,
        w: 6,
        h: 4,
        minW: 4,
        minH: 3
      }
    ],
    onLayoutChange: jest.fn(),
    onWidgetAdd: jest.fn(),
    onWidgetRemove: jest.fn(),
    onWidgetConfigure: jest.fn(),
    onLayoutSave: jest.fn(),
    onLayoutLoad: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders dashboard builder', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    expect(screen.getByText('Dashboard Builder')).toBeInTheDocument();
    expect(screen.getByText('Customize Your Analysis Dashboard')).toBeInTheDocument();
  });

  it('displays widget palette', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    expect(screen.getByText('Widget Palette')).toBeInTheDocument();
    expect(screen.getByText('Priority Matrix')).toBeInTheDocument();
    expect(screen.getByText('Quality Score')).toBeInTheDocument();
    expect(screen.getByText('Dependency Graph')).toBeInTheDocument();
  });

  it('shows widget categories', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    expect(screen.getByText('Analysis')).toBeInTheDocument();
    expect(screen.getByText('Metrics')).toBeInTheDocument();
    expect(screen.getByText('Visualization')).toBeInTheDocument();
  });

  it('filters widgets by category', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const metricsCategory = screen.getByRole('button', { name: /metrics/i });
    fireEvent.click(metricsCategory);
    
    await waitFor(() => {
      expect(screen.getByText('Quality Score')).toBeInTheDocument();
      // Priority Matrix should be filtered out
      expect(screen.queryByText('Priority Matrix')).not.toBeInTheDocument();
    });
  });

  it('handles widget addition via drag and drop', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const qualityScoreWidget = screen.getByText('Quality Score');
    const gridLayout = screen.getByTestId('grid-layout');
    
    // Simulate drag and drop
    fireEvent.dragStart(qualityScoreWidget);
    fireEvent.dragEnter(gridLayout);
    fireEvent.drop(gridLayout);
    
    await waitFor(() => {
      expect(mockProps.onWidgetAdd).toHaveBeenCalledWith('quality-score');
    });
  });

  it('shows widget configuration options', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const configButton = screen.getByRole('button', { name: /configure/i });
    fireEvent.click(configButton);
    
    await waitFor(() => {
      expect(screen.getByText('Widget Configuration')).toBeInTheDocument();
      expect(screen.getByText('Display Options')).toBeInTheDocument();
    });
  });

  it('handles widget removal', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const removeButton = screen.getByRole('button', { name: /remove widget/i });
    fireEvent.click(removeButton);
    
    // Confirm removal
    const confirmButton = screen.getByRole('button', { name: /confirm/i });
    fireEvent.click(confirmButton);
    
    await waitFor(() => {
      expect(mockProps.onWidgetRemove).toHaveBeenCalledWith('priority-matrix');
    });
  });

  it('displays layout presets', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const presetsButton = screen.getByRole('button', { name: /layout presets/i });
    fireEvent.click(presetsButton);
    
    expect(screen.getByText('Default Layout')).toBeInTheDocument();
    expect(screen.getByText('Analysis Focus')).toBeInTheDocument();
    expect(screen.getByText('Developer Dashboard')).toBeInTheDocument();
  });

  it('saves custom layout', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const saveButton = screen.getByRole('button', { name: /save layout/i });
    fireEvent.click(saveButton);
    
    const nameInput = screen.getByPlaceholderText(/Layout name/);
    fireEvent.change(nameInput, { target: { value: 'My Custom Layout' } });
    
    const confirmSaveButton = screen.getByRole('button', { name: /save/i });
    fireEvent.click(confirmSaveButton);
    
    await waitFor(() => {
      expect(mockProps.onLayoutSave).toHaveBeenCalledWith('My Custom Layout', mockProps.currentLayout);
    });
  });

  it('loads saved layouts', async () => {
    const propsWithSaved = {
      ...mockProps,
      savedLayouts: [
        {
          id: 'layout1',
          name: 'My Saved Layout',
          layout: mockProps.currentLayout,
          createdAt: '2024-01-15T10:00:00Z'
        }
      ]
    };
    
    render(<DashboardBuilder {...propsWithSaved} />);
    
    const loadButton = screen.getByRole('button', { name: /load layout/i });
    fireEvent.click(loadButton);
    
    const savedLayout = screen.getByText('My Saved Layout');
    fireEvent.click(savedLayout);
    
    await waitFor(() => {
      expect(mockProps.onLayoutLoad).toHaveBeenCalledWith('layout1');
    });
  });

  it('shows responsive breakpoint controls', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    expect(screen.getByRole('button', { name: /desktop/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /tablet/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /mobile/i })).toBeInTheDocument();
  });

  it('handles breakpoint switching', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const tabletButton = screen.getByRole('button', { name: /tablet/i });
    fireEvent.click(tabletButton);
    
    await waitFor(() => {
      // Grid should adapt to tablet view
      expect(screen.getByTestId('grid-layout')).toBeInTheDocument();
    });
  });

  it('displays widget search functionality', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const searchInput = screen.getByPlaceholderText(/Search widgets/);
    fireEvent.change(searchInput, { target: { value: 'quality' } });
    
    await waitFor(() => {
      expect(screen.getByText('Quality Score')).toBeInTheDocument();
      expect(screen.queryByText('Priority Matrix')).not.toBeInTheDocument();
    });
  });

  it('shows widget size and position controls', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const widgetSettings = screen.getByRole('button', { name: /widget settings/i });
    fireEvent.click(widgetSettings);
    
    await waitFor(() => {
      expect(screen.getByText('Width')).toBeInTheDocument();
      expect(screen.getByText('Height')).toBeInTheDocument();
      expect(screen.getByText('Position')).toBeInTheDocument();
    });
  });

  it('handles layout reset', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const resetButton = screen.getByRole('button', { name: /reset layout/i });
    fireEvent.click(resetButton);
    
    // Confirm reset
    const confirmButton = screen.getByRole('button', { name: /confirm reset/i });
    fireEvent.click(confirmButton);
    
    await waitFor(() => {
      expect(mockProps.onLayoutChange).toHaveBeenCalled();
    });
  });

  it('displays layout validation errors', () => {
    const propsWithErrors = {
      ...mockProps,
      layoutErrors: [
        'Widget overlaps detected',
        'Minimum size constraint violated'
      ]
    };
    
    render(<DashboardBuilder {...propsWithErrors} />);
    
    expect(screen.getByText('Layout Validation')).toBeInTheDocument();
    expect(screen.getByText('Widget overlaps detected')).toBeInTheDocument();
    expect(screen.getByText('Minimum size constraint violated')).toBeInTheDocument();
  });

  it('shows undo/redo functionality', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const undoButton = screen.getByRole('button', { name: /undo/i });
    const redoButton = screen.getByRole('button', { name: /redo/i });
    
    expect(undoButton).toBeInTheDocument();
    expect(redoButton).toBeInTheDocument();
    
    fireEvent.click(undoButton);
    // Should trigger undo action
  });

  it('handles export dashboard configuration', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const exportButton = screen.getByRole('button', { name: /export config/i });
    fireEvent.click(exportButton);
    
    await waitFor(() => {
      expect(screen.getByText('JSON')).toBeInTheDocument();
      expect(screen.getByText('YAML')).toBeInTheDocument();
    });
  });

  it('displays widget preview on hover', async () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const qualityWidget = screen.getByText('Quality Score');
    fireEvent.mouseEnter(qualityWidget);
    
    await waitFor(() => {
      expect(screen.getByText('Preview')).toBeInTheDocument();
    });
    
    fireEvent.mouseLeave(qualityWidget);
    
    await waitFor(() => {
      expect(screen.queryByText('Preview')).not.toBeInTheDocument();
    });
  });

  it('has proper accessibility attributes', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const gridLayout = screen.getByTestId('grid-layout');
    expect(gridLayout).toHaveAttribute('role', 'grid');
    
    const widgetPalette = screen.getByRole('list', { name: /widget palette/i });
    expect(widgetPalette).toBeInTheDocument();
    
    const widgets = screen.getAllByRole('listitem');
    widgets.forEach(widget => {
      expect(widget).toHaveAttribute('draggable');
    });
  });

  it('maintains layout state during edit mode', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    const editModeToggle = screen.getByRole('switch', { name: /edit mode/i });
    fireEvent.click(editModeToggle);
    
    expect(editModeToggle).toBeChecked();
    expect(screen.getByText('Dashboard Builder')).toBeInTheDocument();
  });

  it('shows layout statistics', () => {
    render(<DashboardBuilder {...mockProps} />);
    
    expect(screen.getByText(/1 widget/)).toBeInTheDocument();
    expect(screen.getByText(/Grid utilization/)).toBeInTheDocument();
  });
});