import React, { useMemo, useCallback, useState, useRef } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Paper,
  Grid,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  FormControl,
  FormLabel,
  Select,
  MenuItem,
  Chip,
  Avatar,
  Switch,
  FormControlLabel,
  Slider,
  Alert,
  Tabs,
  Tab,
  useTheme,
  Divider,
  Menu,
  Fab,
  Collapse,
} from '@mui/material';
import {
  DashboardOutlined,
  WidgetsOutlined,
  SettingsOutlined,
  SaveOutlined,
  RestoreOutlined,
  AddOutlined,
  DeleteOutlined,
  EditOutlined,
  DragIndicatorOutlined,
  VisibilityOutlined,
  VisibilityOffOutlined,
  ContentCopyOutlined,
  ShareOutlined,
  GridViewOutlined,
  ViewListOutlined,
  TuneOutlined,
  PaletteOutlined,
  AspectRatioOutlined,
  FilterListOutlined,
  TimelineOutlined,
  AssessmentOutlined,
  BugReportOutlined,
  SecurityOutlined,
  TrendingUpOutlined,
  SearchOutlined,
  GroupOutlined,
  GetAppOutlined,
  ExpandMoreOutlined,
  ExpandLessOutlined,
} from '@mui/icons-material';
import { Responsive, WidthProvider, Layout } from 'react-grid-layout';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';

import type { InteractiveReport } from '../../types/api';
import type {
  DashboardLayout as DashboardLayoutType,
  DashboardWidget,
  WidgetType,
  WidgetConfiguration,
  User,
  ResponsiveComponentProps,
  ExportableComponentProps,
} from '../../types/dashboard';

import { dashboardTheme } from '../../utils/dashboardTheme';

// Import dashboard components
import PriorityMatrix from './PriorityMatrix';
import QualityScoreCard from './QualityScoreCard';
import TechnicalDebtTracker from './TechnicalDebtTracker';
import InteractiveDependencyGraph from './InteractiveDependencyGraph';
import SmartSearchPanel from './SmartSearchPanel';
import FixSuggestionPanel from './FixSuggestionPanel';
import HistoricalComparison from './HistoricalComparison';
import CollaborationPanel from './CollaborationPanel';

const ResponsiveGridLayout = WidthProvider(Responsive);

// Widget type definitions with metadata
const WIDGET_TYPES = {
  priority_matrix: {
    name: 'Priority Matrix',
    description: 'Impact vs Effort analysis for prioritizing fixes',
    icon: <GridViewOutlined />,
    category: 'Analysis',
    minSize: { w: 6, h: 4 },
    defaultSize: { w: 8, h: 6 },
    maxSize: { w: 12, h: 8 },
    configurable: true,
  },
  quality_scorecard: {
    name: 'Quality Score Card',
    description: 'Comprehensive quality scoring with category breakdowns',
    icon: <AssessmentOutlined />,
    category: 'Metrics',
    minSize: { w: 4, h: 3 },
    defaultSize: { w: 6, h: 4 },
    maxSize: { w: 8, h: 6 },
    configurable: true,
  },
  debt_tracker: {
    name: 'Technical Debt Tracker',
    description: 'Track technical debt accumulation and resolution',
    icon: <TrendingUpOutlined />,
    category: 'Metrics',
    minSize: { w: 6, h: 4 },
    defaultSize: { w: 8, h: 6 },
    maxSize: { w: 12, h: 8 },
    configurable: true,
  },
  dependency_graph: {
    name: 'Dependency Graph',
    description: 'Interactive visualization of code dependencies',
    icon: <TimelineOutlined />,
    category: 'Visualization',
    minSize: { w: 8, h: 6 },
    defaultSize: { w: 12, h: 8 },
    maxSize: { w: 12, h: 12 },
    configurable: true,
  },
  search_panel: {
    name: 'Smart Search',
    description: 'Advanced search with AI-powered filtering',
    icon: <SearchOutlined />,
    category: 'Tools',
    minSize: { w: 6, h: 4 },
    defaultSize: { w: 8, h: 6 },
    maxSize: { w: 12, h: 8 },
    configurable: true,
  },
  fix_suggestions: {
    name: 'Fix Suggestions',
    description: 'Step-by-step remediation guides',
    icon: <BugReportOutlined />,
    category: 'Tools',
    minSize: { w: 6, h: 6 },
    defaultSize: { w: 10, h: 8 },
    maxSize: { w: 12, h: 10 },
    configurable: true,
  },
  historical_comparison: {
    name: 'Historical Analysis',
    description: 'Trend analysis and historical comparison',
    icon: <TimelineOutlined />,
    category: 'Analysis',
    minSize: { w: 8, h: 6 },
    defaultSize: { w: 12, h: 8 },
    maxSize: { w: 12, h: 10 },
    configurable: true,
  },
  collaboration: {
    name: 'Team Collaboration',
    description: 'Team discussions and code reviews',
    icon: <GroupOutlined />,
    category: 'Collaboration',
    minSize: { w: 6, h: 6 },
    defaultSize: { w: 8, h: 8 },
    maxSize: { w: 12, h: 10 },
    configurable: true,
  },
};

interface DashboardBuilderProps extends ResponsiveComponentProps, ExportableComponentProps {
  report: InteractiveReport;
  currentUser?: User;
  initialLayout?: DashboardLayoutType;
  onLayoutSave?: (layout: DashboardLayoutType) => void;
  onLayoutLoad?: (layoutId: string) => void;
  readOnly?: boolean;
  height?: number;
}

const DashboardBuilder: React.FC<DashboardBuilderProps> = ({
  report,
  currentUser,
  initialLayout,
  onLayoutSave,
  onLayoutLoad,
  readOnly = false,
  height = 800,
  loading = false,
  error,
  className,
  exportable = false,
  onExport,
  testId,
}) => {
  const theme = useTheme();
  
  // Layout state
  const [dashboardLayout, setDashboardLayout] = useState<DashboardLayoutType>(
    initialLayout || {
      id: `layout_${Date.now()}`,
      name: 'My Dashboard',
      widgets: [
        {
          id: 'widget_1',
          type: 'priority_matrix',
          title: 'Priority Matrix',
          configuration: {},
          position: { x: 0, y: 0 },
          size: { width: 8, height: 6 },
          dataSource: 'main',
        },
        {
          id: 'widget_2', 
          type: 'quality_scorecard',
          title: 'Quality Overview',
          configuration: {},
          position: { x: 8, y: 0 },
          size: { width: 4, height: 6 },
          dataSource: 'main',
        },
      ],
      layout: {
        columns: 12,
        rowHeight: 60,
        margin: [10, 10],
        compactType: 'vertical',
      },
      createdBy: currentUser,
      createdAt: new Date().toISOString(),
      lastModified: new Date().toISOString(),
    }
  );

  // UI state
  const [widgetPanelOpen, setWidgetPanelOpen] = useState(false);
  const [editMode, setEditMode] = useState(!readOnly);
  const [selectedWidget, setSelectedWidget] = useState<string | null>(null);
  const [configDialogOpen, setConfigDialogOpen] = useState(false);
  const [saveDialogOpen, setSaveDialogOpen] = useState(false);
  const [previewMode, setPreviewMode] = useState(false);
  const [activeTab, setActiveTab] = useState(0);
  const [widgetMenu, setWidgetMenu] = useState<{ anchorEl: HTMLElement | null; widgetId: string | null }>({
    anchorEl: null,
    widgetId: null,
  });

  // Save dialog state
  const [layoutName, setLayoutName] = useState(dashboardLayout.name);
  const [layoutDescription, setLayoutDescription] = useState(dashboardLayout.description || '');

  // Widget configuration state
  const [widgetConfig, setWidgetConfig] = useState<WidgetConfiguration>({});

  // Saved layouts (mock data)
  const [savedLayouts] = useState<DashboardLayoutType[]>([
    {
      id: 'layout_default',
      name: 'Default Dashboard',
      description: 'Standard dashboard with essential widgets',
      isDefault: true,
      widgets: [],
      layout: dashboardLayout.layout,
      createdBy: currentUser,
      createdAt: new Date().toISOString(),
      lastModified: new Date().toISOString(),
    },
    {
      id: 'layout_dev',
      name: 'Developer Focus',
      description: 'Dashboard optimized for developers',
      widgets: [],
      layout: dashboardLayout.layout,
      createdBy: currentUser,
      createdAt: new Date().toISOString(),
      lastModified: new Date().toISOString(),
    },
  ]);

  // Convert dashboard layout to react-grid-layout format
  const gridLayout = useMemo(() => {
    const breakpoints = { lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 };
    const cols = { lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 };
    
    return dashboardLayout.widgets.map(widget => ({
      i: widget.id,
      x: widget.position.x,
      y: widget.position.y,
      w: widget.size.width,
      h: widget.size.height,
      minW: WIDGET_TYPES[widget.type]?.minSize.w || 2,
      minH: WIDGET_TYPES[widget.type]?.minSize.h || 2,
      maxW: WIDGET_TYPES[widget.type]?.maxSize.w || 12,
      maxH: WIDGET_TYPES[widget.type]?.maxSize.h || 12,
      static: !editMode,
    }));
  }, [dashboardLayout.widgets, editMode]);

  // Render widget content
  const renderWidget = useCallback((widget: DashboardWidget) => {
    const commonProps = {
      loading,
      className: 'dashboard-widget',
      testId: `widget-${widget.id}`,
    };

    switch (widget.type) {
      case 'priority_matrix':
        return (
          <PriorityMatrix
            findings={report.findings || []}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'quality_scorecard':
        return (
          <QualityScoreCard
            report={report}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'debt_tracker':
        return (
          <TechnicalDebtTracker
            report={report}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'dependency_graph':
        return (
          <InteractiveDependencyGraph
            dependencyGraph={report.dependencyGraph || { nodes: [], edges: [] }}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'search_panel':
        return (
          <SmartSearchPanel
            findings={report.findings || []}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'fix_suggestions':
        return (
          <FixSuggestionPanel
            findings={report.findings || []}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'historical_comparison':
        return (
          <HistoricalComparison
            currentReport={report}
            {...commonProps}
            {...widget.configuration}
          />
        );
      case 'collaboration':
        return (
          <CollaborationPanel
            findings={report.findings || []}
            currentUser={currentUser}
            {...commonProps}
            {...widget.configuration}
          />
        );
      default:
        return (
          <Box
            display="flex"
            alignItems="center"
            justifyContent="center"
            height="100%"
            bgcolor="action.hover"
            borderRadius={1}
          >
            <Typography variant="h6" color="text.secondary">
              Unknown Widget Type
            </Typography>
          </Box>
        );
    }
  }, [report, currentUser, loading]);

  // Handle layout change
  const handleLayoutChange = useCallback((layout: Layout[]) => {
    setDashboardLayout(prev => ({
      ...prev,
      widgets: prev.widgets.map(widget => {
        const layoutItem = layout.find(l => l.i === widget.id);
        return layoutItem ? {
          ...widget,
          position: { x: layoutItem.x, y: layoutItem.y },
          size: { width: layoutItem.w, height: layoutItem.h },
        } : widget;
      }),
      lastModified: new Date().toISOString(),
    }));
  }, []);

  // Add widget
  const handleAddWidget = useCallback((type: WidgetType) => {
    const widgetType = WIDGET_TYPES[type];
    if (!widgetType) return;

    const newWidget: DashboardWidget = {
      id: `widget_${Date.now()}`,
      type,
      title: widgetType.name,
      configuration: {},
      position: { x: 0, y: 0 }, // Grid will auto-position
      size: { 
        width: widgetType.defaultSize.w, 
        height: widgetType.defaultSize.h 
      },
      dataSource: 'main',
    };

    setDashboardLayout(prev => ({
      ...prev,
      widgets: [...prev.widgets, newWidget],
      lastModified: new Date().toISOString(),
    }));

    setWidgetPanelOpen(false);
  }, []);

  // Remove widget
  const handleRemoveWidget = useCallback((widgetId: string) => {
    setDashboardLayout(prev => ({
      ...prev,
      widgets: prev.widgets.filter(w => w.id !== widgetId),
      lastModified: new Date().toISOString(),
    }));
    setWidgetMenu({ anchorEl: null, widgetId: null });
  }, []);

  // Configure widget
  const handleConfigureWidget = useCallback((widgetId: string) => {
    const widget = dashboardLayout.widgets.find(w => w.id === widgetId);
    if (!widget) return;

    setSelectedWidget(widgetId);
    setWidgetConfig(widget.configuration);
    setConfigDialogOpen(true);
    setWidgetMenu({ anchorEl: null, widgetId: null });
  }, [dashboardLayout.widgets]);

  // Save widget configuration
  const handleSaveWidgetConfig = useCallback(() => {
    if (!selectedWidget) return;

    setDashboardLayout(prev => ({
      ...prev,
      widgets: prev.widgets.map(widget =>
        widget.id === selectedWidget
          ? { ...widget, configuration: widgetConfig }
          : widget
      ),
      lastModified: new Date().toISOString(),
    }));

    setConfigDialogOpen(false);
    setSelectedWidget(null);
    setWidgetConfig({});
  }, [selectedWidget, widgetConfig]);

  // Duplicate widget
  const handleDuplicateWidget = useCallback((widgetId: string) => {
    const widget = dashboardLayout.widgets.find(w => w.id === widgetId);
    if (!widget) return;

    const duplicatedWidget: DashboardWidget = {
      ...widget,
      id: `widget_${Date.now()}`,
      title: `${widget.title} (Copy)`,
      position: { x: widget.position.x + 1, y: widget.position.y + 1 },
    };

    setDashboardLayout(prev => ({
      ...prev,
      widgets: [...prev.widgets, duplicatedWidget],
      lastModified: new Date().toISOString(),
    }));

    setWidgetMenu({ anchorEl: null, widgetId: null });
  }, [dashboardLayout.widgets]);

  // Save layout
  const handleSaveLayout = useCallback(() => {
    const savedLayout: DashboardLayoutType = {
      ...dashboardLayout,
      name: layoutName,
      description: layoutDescription,
      lastModified: new Date().toISOString(),
    };

    if (onLayoutSave) {
      onLayoutSave(savedLayout);
    }

    setSaveDialogOpen(false);
  }, [dashboardLayout, layoutName, layoutDescription, onLayoutSave]);

  // Load layout
  const handleLoadLayout = useCallback((layout: DashboardLayoutType) => {
    setDashboardLayout(layout);
    if (onLayoutLoad) {
      onLayoutLoad(layout.id);
    }
  }, [onLayoutLoad]);

  // Handle export
  const handleExport = useCallback(() => {
    if (onExport) {
      onExport('json');
    }
  }, [onExport]);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading dashboard builder...
            </Typography>
          </Box>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Alert severity="error" sx={{ maxWidth: 400 }}>
              <Typography variant="body2">
                Error loading dashboard builder: {error}
              </Typography>
            </Alert>
          </Box>
        </CardContent>
      </Card>
    );
  }

  return (
    <Box className={className} data-testid={testId}>
      {/* Header */}
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box>
          <Typography 
            variant="h5" 
            component="h1" 
            fontWeight="600" 
            color="text.primary"
            id="dashboard-title"
          >
            Dashboard Builder
          </Typography>
          <Typography variant="body2" color="text.secondary" id="dashboard-summary">
            {dashboardLayout.name} • {dashboardLayout.widgets.length} widgets
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          {!readOnly && (
            <>
              <FormControlLabel
                control={
                  <Switch
                    checked={editMode}
                    onChange={(e) => setEditMode(e.target.checked)}
                    inputProps={{
                      'aria-label': 'Toggle dashboard edit mode',
                      'aria-describedby': 'edit-mode-description'
                    }}
                  />
                }
                label="Edit Mode"
                componentsProps={{ typography: { variant: 'body2' } }}
              />
              <Typography 
                id="edit-mode-description" 
                variant="caption" 
                color="text.secondary"
                sx={{ display: 'none' }}
              >
                Enable edit mode to add, remove, or modify dashboard widgets
              </Typography>

              <Button
                variant="outlined"
                startIcon={<AddOutlined aria-hidden="true" />}
                onClick={() => setWidgetPanelOpen(true)}
                disabled={!editMode}
                aria-label="Add new widget to dashboard"
                aria-describedby={!editMode ? 'add-widget-disabled' : undefined}
              >
                Add Widget
              </Button>
              {!editMode && (
                <Typography 
                  id="add-widget-disabled" 
                  variant="caption" 
                  color="text.secondary"
                  sx={{ display: 'none' }}
                >
                  Enable edit mode to add widgets
                </Typography>
              )}

              <Button
                variant="outlined"
                startIcon={<SaveOutlined aria-hidden="true" />}
                onClick={() => setSaveDialogOpen(true)}
                aria-label="Save current dashboard layout"
              >
                Save Layout
              </Button>
            </>
          )}

          <FormControlLabel
            control={
              <Switch
                checked={previewMode}
                onChange={(e) => setPreviewMode(e.target.checked)}
                inputProps={{
                  'aria-label': 'Toggle preview mode',
                  'aria-describedby': 'preview-mode-description'
                }}
              />
            }
            label="Preview"
            componentsProps={{ typography: { variant: 'body2' } }}
          />
          <Typography 
            id="preview-mode-description" 
            variant="caption" 
            color="text.secondary"
            sx={{ display: 'none' }}
          >
            Preview mode hides editing controls and shows the dashboard as users will see it
          </Typography>

          {exportable && (
            <Tooltip title="Export dashboard" arrow>
              <IconButton 
                onClick={handleExport}
                aria-label="Export dashboard data"
              >
                <GetAppOutlined aria-hidden="true" />
              </IconButton>
            </Tooltip>
          )}
        </Box>
      </Box>

      {/* Dashboard grid */}
      <Box
        sx={{
          height: height - 80,
          bgcolor: previewMode ? 'background.default' : 'action.hover',
          borderRadius: 1,
          p: previewMode ? 0 : 1,
          position: 'relative',
          overflow: 'auto',
          '& .react-grid-item.react-grid-placeholder': {
            backgroundColor: theme.palette.primary.main,
            opacity: 0.3,
            borderRadius: 1,
          },
          '& .react-grid-item:not(.react-grid-placeholder)': {
            backgroundColor: 'background.paper',
            borderRadius: 1,
            boxShadow: previewMode ? 'none' : 1,
            overflow: 'hidden',
          },
          '& .react-resizable-handle': {
            display: editMode ? 'block' : 'none',
          },
        }}
      >
        {dashboardLayout.widgets.length === 0 ? (
          <Box
            display="flex"
            flexDirection="column"
            alignItems="center"
            justifyContent="center"
            height="100%"
            color="text.secondary"
          >
            <DashboardOutlined sx={{ fontSize: 64, mb: 2 }} />
            <Typography variant="h6" mb={1} component="h2">
              Empty Dashboard
            </Typography>
            <Typography variant="body2" textAlign="center" mb={3}>
              Add widgets to start building your custom dashboard
            </Typography>
            {!readOnly && (
              <Button
                variant="contained"
                startIcon={<AddOutlined aria-hidden="true" />}
                onClick={() => setWidgetPanelOpen(true)}
                aria-label="Add your first widget to the dashboard"
              >
                Add Your First Widget
              </Button>
            )}
          </Box>
        ) : (
          <ResponsiveGridLayout
            className="dashboard-grid"
            layouts={{ lg: gridLayout }}
            breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
            cols={{ lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 }}
            rowHeight={dashboardLayout.layout.rowHeight}
            margin={dashboardLayout.layout.margin}
            compactType={dashboardLayout.layout.compactType}
            preventCollision={dashboardLayout.layout.preventCollision}
            isDraggable={editMode && !previewMode}
            isResizable={editMode && !previewMode}
            onLayoutChange={handleLayoutChange}
            useCSSTransforms
          >
            {dashboardLayout.widgets.map(widget => (
              <Card
                key={widget.id}
                sx={{
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  cursor: editMode ? 'move' : 'default',
                  position: 'relative',
                  overflow: 'hidden',
                }}
              >
                {/* Widget header */}
                {!previewMode && (
                  <Box
                    sx={{
                      p: 1,
                      borderBottom: '1px solid',
                      borderColor: 'divider',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'between',
                      bgcolor: 'action.hover',
                      minHeight: 48,
                    }}
                  >
                    <Box display="flex" alignItems="center" gap={1}>
                      <Avatar
                        sx={{
                          bgcolor: 'primary.light',
                          width: 24,
                          height: 24,
                        }}
                      >
                        {WIDGET_TYPES[widget.type]?.icon}
                      </Avatar>
                      <Typography variant="subtitle2" fontWeight="600">
                        {widget.title}
                      </Typography>
                    </Box>

                    {editMode && (
                      <Box display="flex" alignItems="center">
                        <Tooltip title="Drag to move" arrow>
                          <IconButton size="small" className="drag-handle">
                            <DragIndicatorOutlined />
                          </IconButton>
                        </Tooltip>
                        <IconButton
                          size="small"
                          onClick={(e) => setWidgetMenu({ anchorEl: e.currentTarget, widgetId: widget.id })}
                        >
                          <TuneOutlined />
                        </IconButton>
                      </Box>
                    )}
                  </Box>
                )}

                {/* Widget content */}
                <CardContent sx={{ 
                  flex: 1, 
                  overflow: 'auto',
                  p: previewMode ? 0 : 2,
                  '&:last-child': { pb: previewMode ? 0 : 2 }
                }}>
                  {renderWidget(widget)}
                </CardContent>
              </Card>
            ))}
          </ResponsiveGridLayout>
        )}
      </Box>

      {/* Floating action button for quick add */}
      {editMode && !readOnly && (
        <Fab
          color="primary"
          sx={{
            position: 'fixed',
            bottom: 24,
            right: 24,
            zIndex: 1000,
          }}
          onClick={() => setWidgetPanelOpen(true)}
        >
          <AddOutlined />
        </Fab>
      )}

      {/* Widget panel drawer */}
      <Drawer
        anchor="right"
        open={widgetPanelOpen}
        onClose={() => setWidgetPanelOpen(false)}
        PaperProps={{ 
          sx: { width: 360 },
          'aria-label': 'Widget selection panel'
        }}
        ModalProps={{
          keepMounted: false,
          'aria-labelledby': 'widget-drawer-title',
          'aria-describedby': 'widget-drawer-description'
        }}
      >
        <Box sx={{ p: 2 }}>
          <Box display="flex" alignItems="center" justifyContent="between" mb={2}>
            <Typography 
              variant="h6" 
              fontWeight="600"
              id="widget-drawer-title"
              component="h2"
            >
              Add Widget
            </Typography>
            <Typography 
              id="widget-drawer-description" 
              variant="caption" 
              color="text.secondary"
              sx={{ display: 'none' }}
            >
              Choose from available widgets to add to your dashboard
            </Typography>
            <IconButton 
              onClick={() => setWidgetPanelOpen(false)}
              aria-label="Close widget panel"
            >
              <ExpandLessOutlined aria-hidden="true" />
            </IconButton>
          </Box>

          <Tabs 
            value={activeTab} 
            onChange={(_, newValue) => setActiveTab(newValue)} 
            sx={{ mb: 2 }}
            aria-label="Widget selection tabs"
          >
            <Tab 
              label="By Category" 
              id="tab-category"
              aria-controls="tabpanel-category"
            />
            <Tab 
              label="All Widgets" 
              id="tab-all"
              aria-controls="tabpanel-all"
            />
            <Tab 
              label="Templates" 
              id="tab-templates"
              aria-controls="tabpanel-templates"
            />
          </Tabs>

          {/* Widgets by category */}
          {activeTab === 0 && (
            <Box>
              {['Analysis', 'Metrics', 'Visualization', 'Tools', 'Collaboration'].map(category => (
                <Box key={category} mb={2}>
                  <Typography variant="subtitle2" fontWeight="600" mb={1} color="text.secondary">
                    {category}
                  </Typography>
                  <Grid container spacing={1}>
                    {Object.entries(WIDGET_TYPES)
                      .filter(([, widgetType]) => widgetType.category === category)
                      .map(([type, widgetType]) => (
                        <Grid item xs={12} key={type}>
                          <Paper
                            component="button"
                            sx={{
                              p: 2,
                              cursor: 'pointer',
                              border: '1px solid',
                              borderColor: 'divider',
                              backgroundColor: 'transparent',
                              width: '100%',
                              textAlign: 'left',
                              '&:hover': {
                                bgcolor: 'action.hover',
                                borderColor: 'primary.main',
                              },
                              '&:focus': {
                                outline: '2px solid',
                                outlineColor: 'primary.main',
                                outlineOffset: '2px'
                              },
                            }}
                            onClick={() => handleAddWidget(type as WidgetType)}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                handleAddWidget(type as WidgetType);
                              }
                            }}
                            aria-label={`Add ${widgetType.name} widget: ${widgetType.description}`}
                          >
                            <Box display="flex" alignItems="center" gap={2}>
                              <Avatar
                                sx={{
                                  bgcolor: 'primary.light',
                                  width: 32,
                                  height: 32,
                                }}
                              >
                                {widgetType.icon}
                              </Avatar>
                              <Box>
                                <Typography variant="body2" fontWeight="600">
                                  {widgetType.name}
                                </Typography>
                                <Typography variant="caption" color="text.secondary">
                                  {widgetType.description}
                                </Typography>
                              </Box>
                            </Box>
                          </Paper>
                        </Grid>
                      ))}
                  </Grid>
                </Box>
              ))}
            </Box>
          )}

          {/* All widgets */}
          {activeTab === 1 && (
            <List>
              {Object.entries(WIDGET_TYPES).map(([type, widgetType]) => (
                <ListItemButton
                  key={type}
                  onClick={() => handleAddWidget(type as WidgetType)}
                  sx={{ borderRadius: 1, mb: 0.5 }}
                  aria-label={`Add ${widgetType.name} widget: ${widgetType.description}`}
                >
                  <ListItemIcon>
                    <Avatar
                      sx={{
                        bgcolor: 'primary.light',
                        width: 32,
                        height: 32,
                      }}
                    >
                      {widgetType.icon}
                    </Avatar>
                  </ListItemIcon>
                  <ListItemText
                    primary={widgetType.name}
                    secondary={widgetType.description}
                  />
                </ListItemButton>
              ))}
            </List>
          )}

          {/* Template layouts */}
          {activeTab === 2 && (
            <Box>
              <Typography variant="body2" color="text.secondary" mb={2}>
                Saved layouts and templates
              </Typography>
              
              <List>
                {savedLayouts.map(layout => (
                  <ListItemButton
                    key={layout.id}
                    onClick={() => handleLoadLayout(layout)}
                    sx={{ borderRadius: 1, mb: 0.5 }}
                    aria-label={`Load ${layout.name} dashboard layout${layout.description ? ': ' + layout.description : ''}`}
                  >
                    <ListItemIcon>
                      <Avatar
                        sx={{
                          bgcolor: layout.isDefault ? 'primary.light' : 'secondary.light',
                          width: 32,
                          height: 32,
                        }}
                      >
                        <DashboardOutlined />
                      </Avatar>
                    </ListItemIcon>
                    <ListItemText
                      primary={
                        <Box display="flex" alignItems="center" gap={1}>
                          <Typography variant="body2" fontWeight="600">
                            {layout.name}
                          </Typography>
                          {layout.isDefault && (
                            <Chip label="default" size="small" color="primary" variant="outlined" />
                          )}
                        </Box>
                      }
                      secondary={layout.description}
                    />
                  </ListItemButton>
                ))}
              </List>
            </Box>
          )}
        </Box>
      </Drawer>

      {/* Widget context menu */}
      <Menu
        anchorEl={widgetMenu.anchorEl}
        open={Boolean(widgetMenu.anchorEl)}
        onClose={() => setWidgetMenu({ anchorEl: null, widgetId: null })}
      >
        <MenuItem onClick={() => widgetMenu.widgetId && handleConfigureWidget(widgetMenu.widgetId)}>
          <ListItemIcon>
            <TuneOutlined fontSize="small" />
          </ListItemIcon>
          Configure
        </MenuItem>
        <MenuItem onClick={() => widgetMenu.widgetId && handleDuplicateWidget(widgetMenu.widgetId)}>
          <ListItemIcon>
            <ContentCopyOutlined fontSize="small" />
          </ListItemIcon>
          Duplicate
        </MenuItem>
        <Divider />
        <MenuItem 
          onClick={() => widgetMenu.widgetId && handleRemoveWidget(widgetMenu.widgetId)}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon>
            <DeleteOutlined fontSize="small" color="error" />
          </ListItemIcon>
          Remove
        </MenuItem>
      </Menu>

      {/* Widget configuration dialog */}
      <Dialog
        open={configDialogOpen}
        onClose={() => setConfigDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          Configure Widget
          {selectedWidget && (
            <Typography variant="subtitle2" color="text.secondary">
              {dashboardLayout.widgets.find(w => w.id === selectedWidget)?.title}
            </Typography>
          )}
        </DialogTitle>
        <DialogContent>
          <Box display="flex" flexDirection="column" gap={2} pt={1}>
            <Alert severity="info">
              <Typography variant="body2">
                Widget-specific configuration options would appear here. 
                Each widget type would have its own configuration form.
              </Typography>
            </Alert>

            {/* Mock configuration options */}
            <FormControlLabel
              control={
                <Switch
                  checked={widgetConfig.interactive !== false}
                  onChange={(e) => setWidgetConfig(prev => ({ ...prev, interactive: e.target.checked }))}
                />
              }
              label="Interactive Mode"
            />

            <FormControlLabel
              control={
                <Switch
                  checked={widgetConfig.showLegend !== false}
                  onChange={(e) => setWidgetConfig(prev => ({ ...prev, showLegend: e.target.checked }))}
                />
              }
              label="Show Legend"
            />

            <Box>
              <FormLabel>Height</FormLabel>
              <Slider
                value={widgetConfig.height || 400}
                onChange={(_, value) => setWidgetConfig(prev => ({ ...prev, height: value as number }))}
                min={200}
                max={800}
                step={50}
                valueLabelDisplay="auto"
                marks={[
                  { value: 200, label: '200px' },
                  { value: 400, label: '400px' },
                  { value: 600, label: '600px' },
                  { value: 800, label: '800px' },
                ]}
              />
            </Box>

            <TextField
              fullWidth
              label="Custom Title"
              value={widgetConfig.title || ''}
              onChange={(e) => setWidgetConfig(prev => ({ ...prev, title: e.target.value }))}
              placeholder="Enter a custom title for this widget..."
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfigDialogOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleSaveWidgetConfig} variant="contained">
            Save Configuration
          </Button>
        </DialogActions>
      </Dialog>

      {/* Save layout dialog */}
      <Dialog
        open={saveDialogOpen}
        onClose={() => setSaveDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Save Dashboard Layout</DialogTitle>
        <DialogContent>
          <Box display="flex" flexDirection="column" gap={2} pt={1}>
            <TextField
              fullWidth
              label="Layout Name"
              value={layoutName}
              onChange={(e) => setLayoutName(e.target.value)}
              placeholder="Enter a name for this layout..."
            />

            <TextField
              fullWidth
              multiline
              rows={3}
              label="Description (Optional)"
              value={layoutDescription}
              onChange={(e) => setLayoutDescription(e.target.value)}
              placeholder="Describe this dashboard layout..."
            />

            <Alert severity="info">
              <Typography variant="body2">
                This layout contains {dashboardLayout.widgets.length} widgets and will be saved to your profile.
              </Typography>
            </Alert>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSaveDialogOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleSaveLayout}
            variant="contained"
            startIcon={<SaveOutlined />}
            disabled={!layoutName.trim()}
          >
            Save Layout
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default DashboardBuilder;