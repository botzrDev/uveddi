import React, { useMemo, useCallback, useState, useRef } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  FormControl,
  FormLabel,
  Select,
  MenuItem,
  TextField,
  Switch,
  FormControlLabel,
  FormGroup,
  Chip,
  Paper,
  Divider,
  Alert,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Stepper,
  Step,
  StepLabel,
  StepContent,
  LinearProgress,
  Tabs,
  Tab,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  useTheme,
  Avatar,
  Badge,
} from '@mui/material';
import {
  GetAppOutlined,
  PictureAsPdfOutlined,
  CodeOutlined,
  TableViewOutlined,
  DescriptionOutlined,
  EmailOutlined,
  ScheduleOutlined,
  CloudDownloadOutlined,
  ShareOutlined,
  SettingsOutlined,
  PlayArrowOutlined,
  CheckCircleOutlined,
  ErrorOutlined,
  InfoOutlined,
  FileDownloadOutlined,
  ContentCopyOutlined,
  LinkOutlined,
  WebhookOutlined,
  IntegrationInstructionsOutlined,
  ApiOutlined,
  ExpandMoreOutlined,
  BrushOutlined,
  PaletteOutlined,
  BusinessOutlined,
  VisibilityOutlined,
  TuneOutlined,
  TimerOutlined,
  CloudUploadOutlined,
  FolderOutlined,
  SaveOutlined,
  HistoryOutlined,
  NotificationsOutlined,
} from '@mui/icons-material';
import { format } from 'date-fns';
import html2canvas from 'html2canvas';
import jsPDF from 'jspdf';

import type { InteractiveReport } from '../../types/api';
import type {
  ExportConfiguration,
  ExportFormat,
  ExportSection,
  ExportBranding,
  ExportSchedule,
  ResponsiveComponentProps,
} from '../../types/dashboard';

import { dashboardTheme } from '../../utils/dashboardTheme';

// Export format definitions
const EXPORT_FORMATS: Record<ExportFormat, { 
  name: string; 
  description: string; 
  icon: React.ReactNode; 
  category: string;
  supports: string[];
}> = {
  pdf: {
    name: 'PDF Report',
    description: 'Comprehensive PDF report with charts and diagrams',
    icon: <PictureAsPdfOutlined />,
    category: 'Document',
    supports: ['charts', 'diagrams', 'branding', 'sections'],
  },
  html: {
    name: 'HTML Report',
    description: 'Interactive web report with full functionality',
    icon: <DescriptionOutlined />,
    category: 'Web',
    supports: ['interactive', 'charts', 'diagrams', 'branding'],
  },
  json: {
    name: 'JSON Data',
    description: 'Raw data in JSON format for API integration',
    icon: <CodeOutlined />,
    category: 'Data',
    supports: ['raw_data', 'api'],
  },
  csv: {
    name: 'CSV Spreadsheet',
    description: 'Tabular data for analysis in spreadsheet applications',
    icon: <TableViewOutlined />,
    category: 'Data',
    supports: ['tabular'],
  },
  xlsx: {
    name: 'Excel Workbook',
    description: 'Excel file with multiple sheets and formatting',
    icon: <TableViewOutlined />,
    category: 'Document',
    supports: ['tabular', 'charts', 'sheets'],
  },
  markdown: {
    name: 'Markdown Report',
    description: 'Documentation-friendly markdown format',
    icon: <DescriptionOutlined />,
    category: 'Document',
    supports: ['text', 'simple_charts'],
  },
  docx: {
    name: 'Word Document',
    description: 'Microsoft Word compatible document',
    icon: <DescriptionOutlined />,
    category: 'Document',
    supports: ['text', 'images', 'branding'],
  },
};

// Integration options
const INTEGRATIONS = [
  {
    id: 'slack',
    name: 'Slack',
    description: 'Send reports to Slack channels',
    icon: '💬',
    category: 'Communication',
    setup: 'webhook',
  },
  {
    id: 'teams',
    name: 'Microsoft Teams',
    description: 'Share reports in Teams channels',
    icon: '📞',
    category: 'Communication',
    setup: 'webhook',
  },
  {
    id: 'jira',
    name: 'Jira',
    description: 'Create issues from critical findings',
    icon: '🎯',
    category: 'Issue Tracking',
    setup: 'api_key',
  },
  {
    id: 'github',
    name: 'GitHub',
    description: 'Create issues and PR comments',
    icon: '🐙',
    category: 'Version Control',
    setup: 'oauth',
  },
  {
    id: 'email',
    name: 'Email',
    description: 'Send reports via email',
    icon: '📧',
    category: 'Communication',
    setup: 'smtp',
  },
  {
    id: 's3',
    name: 'AWS S3',
    description: 'Store reports in cloud storage',
    icon: '☁️',
    category: 'Storage',
    setup: 'api_key',
  },
];

interface ExportHubProps extends ResponsiveComponentProps {
  report: InteractiveReport;
  onExport?: (config: ExportConfiguration) => Promise<void>;
  onScheduleExport?: (config: ExportConfiguration) => Promise<void>;
  onIntegrationSetup?: (integration: string, config: any) => Promise<void>;
  height?: number;
}

const ExportHub: React.FC<ExportHubProps> = ({
  report,
  onExport,
  onScheduleExport,
  onIntegrationSetup,
  height = 600,
  loading = false,
  error,
  className,
  testId,
}) => {
  const theme = useTheme();
  const [activeTab, setActiveTab] = useState(0);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [scheduleDialogOpen, setScheduleDialogOpen] = useState(false);
  const [integrationDialogOpen, setIntegrationDialogOpen] = useState(false);
  
  // Export configuration state
  const [exportConfig, setExportConfig] = useState<ExportConfiguration>({
    format: 'pdf',
    includeCharts: true,
    includeDiagrams: true,
    includeRawData: false,
    customSections: [],
    branding: {
      companyName: '',
      logo: '',
      colors: {
        primary: dashboardTheme.colors.primary[0],
        secondary: dashboardTheme.colors.secondary[0],
        accent: dashboardTheme.colors.success[1],
      },
    },
  });

  // Schedule configuration state
  const [scheduleConfig, setScheduleConfig] = useState<ExportSchedule>({
    enabled: false,
    frequency: 'weekly',
    recipients: [],
    subject: 'Uveddi Analysis Report',
    body: 'Please find the latest code analysis report attached.',
  });

  // UI state
  const [exportProgress, setExportProgress] = useState(0);
  const [isExporting, setIsExporting] = useState(false);
  const [selectedFormat, setSelectedFormat] = useState<ExportFormat>('pdf');
  const [selectedIntegration, setSelectedIntegration] = useState<string | null>(null);
  const [expandedSection, setExpandedSection] = useState<string | false>('format');

  // Export history (mock data)
  const [exportHistory] = useState([
    {
      id: '1',
      format: 'pdf',
      timestamp: '2024-01-15T10:30:00Z',
      status: 'completed',
      size: '2.4 MB',
      downloads: 3,
    },
    {
      id: '2',
      format: 'html',
      timestamp: '2024-01-14T15:20:00Z',
      status: 'completed',
      size: '1.8 MB',
      downloads: 1,
    },
    {
      id: '3',
      format: 'json',
      timestamp: '2024-01-13T09:15:00Z',
      status: 'completed',
      size: '456 KB',
      downloads: 5,
    },
  ]);

  // Generate export preview
  const exportPreview = useMemo(() => {
    const sections = [
      { name: 'Executive Summary', included: true },
      { name: 'Key Metrics', included: true },
      { name: 'Findings Overview', included: true },
      { name: 'Priority Matrix', included: exportConfig.includeCharts },
      { name: 'Quality Scorecard', included: exportConfig.includeCharts },
      { name: 'Dependency Graph', included: exportConfig.includeDiagrams },
      { name: 'Technical Debt Analysis', included: exportConfig.includeCharts },
      { name: 'Raw Data', included: exportConfig.includeRawData },
    ];

    return sections;
  }, [exportConfig]);

  // Handle export
  const handleExport = useCallback(async () => {
    setIsExporting(true);
    setExportProgress(0);

    try {
      // Simulate export progress
      const steps = 5;
      for (let i = 1; i <= steps; i++) {
        await new Promise(resolve => setTimeout(resolve, 500));
        setExportProgress((i / steps) * 100);
      }

      if (onExport) {
        await onExport(exportConfig);
      } else {
        // Mock export functionality
        await mockExport(exportConfig);
      }

      setExportDialogOpen(false);
    } catch (err) {
      console.error('Export failed:', err);
    } finally {
      setIsExporting(false);
      setExportProgress(0);
    }
  }, [exportConfig, onExport]);

  // Mock export function
  const mockExport = useCallback(async (config: ExportConfiguration) => {
    switch (config.format) {
      case 'pdf':
        // Generate PDF using html2canvas and jsPDF
        const element = document.querySelector('.dashboard-grid') as HTMLElement;
        if (element) {
          const canvas = await html2canvas(element);
          const pdf = new jsPDF('p', 'mm', 'a4');
          const imgData = canvas.toDataURL('image/png');
          const pdfWidth = pdf.internal.pageSize.getWidth();
          const pdfHeight = (canvas.height * pdfWidth) / canvas.width;
          
          pdf.addImage(imgData, 'PNG', 0, 0, pdfWidth, pdfHeight);
          pdf.save('uveddi-analysis-report.pdf');
        }
        break;

      case 'json':
        const jsonData = JSON.stringify(report, null, 2);
        const jsonBlob = new Blob([jsonData], { type: 'application/json' });
        const jsonUrl = URL.createObjectURL(jsonBlob);
        const jsonLink = document.createElement('a');
        jsonLink.href = jsonUrl;
        jsonLink.download = 'uveddi-analysis-data.json';
        jsonLink.click();
        URL.revokeObjectURL(jsonUrl);
        break;

      case 'csv':
        const csvData = convertToCsv(report);
        const csvBlob = new Blob([csvData], { type: 'text/csv' });
        const csvUrl = URL.createObjectURL(csvBlob);
        const csvLink = document.createElement('a');
        csvLink.href = csvUrl;
        csvLink.download = 'uveddi-findings.csv';
        csvLink.click();
        URL.revokeObjectURL(csvUrl);
        break;

      default:
        alert(`${config.format} export would be implemented here`);
        break;
    }
  }, [report]);

  // Convert report to CSV
  const convertToCsv = useCallback((report: InteractiveReport): string => {
    if (!report.findings || report.findings.length === 0) {
      return 'No findings data available';
    }

    const headers = ['File', 'Line', 'Severity', 'Detector', 'Title', 'Message', 'Confidence', 'Tags'];
    const rows = report.findings.map(finding => [
      finding.file,
      finding.line?.toString() || '',
      finding.severity,
      finding.detector,
      finding.title,
      finding.message.replace(/,/g, ';'), // Escape commas
      finding.confidence.toString(),
      finding.tags.join('; '),
    ]);

    return [headers, ...rows].map(row => row.map(cell => `"${cell}"`).join(',')).join('\n');
  }, []);

  // Handle schedule setup
  const handleScheduleSetup = useCallback(async () => {
    if (onScheduleExport) {
      await onScheduleExport({ ...exportConfig, scheduling: scheduleConfig });
    } else {
      alert('Schedule export would be set up here');
    }
    setScheduleDialogOpen(false);
  }, [exportConfig, scheduleConfig, onScheduleExport]);

  // Handle integration setup
  const handleIntegrationSetup = useCallback(async (integration: string, config: any) => {
    if (onIntegrationSetup) {
      await onIntegrationSetup(integration, config);
    } else {
      alert(`${integration} integration would be set up here with config: ${JSON.stringify(config)}`);
    }
    setIntegrationDialogOpen(false);
  }, [onIntegrationSetup]);

  // Get format category
  const getFormatsByCategory = useCallback((category: string) => {
    return Object.entries(EXPORT_FORMATS).filter(([, format]) => format.category === category);
  }, []);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading export hub...
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
                Error loading export hub: {error}
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
          <Typography variant="h5" component="h2" fontWeight="600" color="text.primary">
            Export Hub
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Export reports and set up integrations
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          <Button
            variant="contained"
            startIcon={<GetAppOutlined />}
            onClick={() => setExportDialogOpen(true)}
          >
            Quick Export
          </Button>
          
          <Tooltip title="Comprehensive export and integration options" arrow>
            <IconButton>
              <InfoOutlined />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      <Card>
        {/* Tabs */}
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs value={activeTab} onChange={(_, newValue) => setActiveTab(newValue)}>
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <FileDownloadOutlined fontSize="small" />
                  Export Formats
                </Box>
              }
            />
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <IntegrationInstructionsOutlined fontSize="small" />
                  Integrations
                </Box>
              }
            />
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <ScheduleOutlined fontSize="small" />
                  Scheduling
                </Box>
              }
            />
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <HistoryOutlined fontSize="small" />
                  History
                </Box>
              }
            />
          </Tabs>
        </Box>

        <CardContent sx={{ height: height - 120, overflow: 'auto' }}>
          {/* Export Formats Tab */}
          {activeTab === 0 && (
            <Box>
              {/* Quick export buttons */}
              <Box mb={3}>
                <Typography variant="h6" fontWeight="600" mb={2}>
                  Quick Export
                </Typography>
                <Grid container spacing={2}>
                  {['pdf', 'json', 'csv', 'html'].map(format => {
                    const formatInfo = EXPORT_FORMATS[format as ExportFormat];
                    return (
                      <Grid item xs={12} sm={6} md={3} key={format}>
                        <Paper
                          sx={{
                            p: 2,
                            textAlign: 'center',
                            cursor: 'pointer',
                            border: '1px solid',
                            borderColor: 'divider',
                            '&:hover': {
                              bgcolor: 'action.hover',
                              borderColor: 'primary.main',
                            },
                          }}
                          onClick={() => {
                            setSelectedFormat(format as ExportFormat);
                            setExportConfig(prev => ({ ...prev, format: format as ExportFormat }));
                            setExportDialogOpen(true);
                          }}
                        >
                          <Avatar
                            sx={{
                              bgcolor: 'primary.light',
                              width: 48,
                              height: 48,
                              mx: 'auto',
                              mb: 1,
                            }}
                          >
                            {formatInfo.icon}
                          </Avatar>
                          <Typography variant="subtitle2" fontWeight="600" mb={0.5}>
                            {formatInfo.name}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            {formatInfo.description}
                          </Typography>
                        </Paper>
                      </Grid>
                    );
                  })}
                </Grid>
              </Box>

              <Divider sx={{ my: 3 }} />

              {/* Format categories */}
              <Typography variant="h6" fontWeight="600" mb={2}>
                All Export Formats
              </Typography>
              
              {['Document', 'Web', 'Data'].map(category => (
                <Box key={category} mb={3}>
                  <Typography variant="subtitle1" fontWeight="600" mb={2} color="text.secondary">
                    {category} Formats
                  </Typography>
                  <Grid container spacing={2}>
                    {getFormatsByCategory(category).map(([format, formatInfo]) => (
                      <Grid item xs={12} sm={6} md={4} key={format}>
                        <Card
                          sx={{
                            cursor: 'pointer',
                            border: '1px solid',
                            borderColor: selectedFormat === format ? 'primary.main' : 'divider',
                            bgcolor: selectedFormat === format ? 'action.selected' : 'transparent',
                            '&:hover': {
                              borderColor: 'primary.main',
                            },
                          }}
                          onClick={() => setSelectedFormat(format as ExportFormat)}
                        >
                          <CardContent>
                            <Box display="flex" alignItems="center" gap={2} mb={1}>
                              <Avatar
                                sx={{
                                  bgcolor: selectedFormat === format ? 'primary.main' : 'primary.light',
                                  width: 32,
                                  height: 32,
                                }}
                              >
                                {formatInfo.icon}
                              </Avatar>
                              <Typography variant="subtitle2" fontWeight="600">
                                {formatInfo.name}
                              </Typography>
                            </Box>
                            <Typography variant="body2" color="text.secondary" mb={2}>
                              {formatInfo.description}
                            </Typography>
                            <Box display="flex" flexWrap="wrap" gap={0.5}>
                              {formatInfo.supports.map(feature => (
                                <Chip
                                  key={feature}
                                  label={feature.replace('_', ' ')}
                                  size="small"
                                  variant="outlined"
                                  sx={{ fontSize: '0.6rem', height: 18 }}
                                />
                              ))}
                            </Box>
                          </CardContent>
                        </Card>
                      </Grid>
                    ))}
                  </Grid>
                </Box>
              ))}
            </Box>
          )}

          {/* Integrations Tab */}
          {activeTab === 1 && (
            <Box>
              <Typography variant="h6" fontWeight="600" mb={2}>
                Available Integrations
              </Typography>
              
              {['Communication', 'Issue Tracking', 'Version Control', 'Storage'].map(category => (
                <Box key={category} mb={3}>
                  <Typography variant="subtitle1" fontWeight="600" mb={2} color="text.secondary">
                    {category}
                  </Typography>
                  <Grid container spacing={2}>
                    {INTEGRATIONS.filter(integration => integration.category === category).map(integration => (
                      <Grid item xs={12} sm={6} md={4} key={integration.id}>
                        <Card>
                          <CardContent>
                            <Box display="flex" alignItems="center" gap={2} mb={2}>
                              <Box
                                sx={{
                                  width: 32,
                                  height: 32,
                                  display: 'flex',
                                  alignItems: 'center',
                                  justifyContent: 'center',
                                  fontSize: '20px',
                                  bgcolor: 'action.hover',
                                  borderRadius: 1,
                                }}
                              >
                                {integration.icon}
                              </Box>
                              <Typography variant="subtitle1" fontWeight="600">
                                {integration.name}
                              </Typography>
                            </Box>
                            <Typography variant="body2" color="text.secondary" mb={2}>
                              {integration.description}
                            </Typography>
                            <Button
                              fullWidth
                              variant="outlined"
                              startIcon={<SettingsOutlined />}
                              onClick={() => {
                                setSelectedIntegration(integration.id);
                                setIntegrationDialogOpen(true);
                              }}
                            >
                              Setup
                            </Button>
                          </CardContent>
                        </Card>
                      </Grid>
                    ))}
                  </Grid>
                </Box>
              ))}
            </Box>
          )}

          {/* Scheduling Tab */}
          {activeTab === 2 && (
            <Box>
              <Typography variant="h6" fontWeight="600" mb={2}>
                Automated Reports
              </Typography>
              
              <Alert severity="info" sx={{ mb: 3 }}>
                <Typography variant="body2">
                  Set up automated report generation and delivery. Reports will be generated 
                  based on the latest analysis results and sent to specified recipients.
                </Typography>
              </Alert>

              <Card>
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="between" mb={3}>
                    <Box>
                      <Typography variant="subtitle1" fontWeight="600">
                        Weekly Analysis Report
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Automated weekly report delivered every Monday
                      </Typography>
                    </Box>
                    <Switch
                      checked={scheduleConfig.enabled}
                      onChange={(e) => setScheduleConfig(prev => ({ ...prev, enabled: e.target.checked }))}
                    />
                  </Box>

                  <Grid container spacing={2}>
                    <Grid item xs={12} sm={6}>
                      <FormControl fullWidth size="small">
                        <FormLabel sx={{ mb: 1 }}>Frequency</FormLabel>
                        <Select
                          value={scheduleConfig.frequency}
                          onChange={(e) => setScheduleConfig(prev => ({ ...prev, frequency: e.target.value as any }))}
                        >
                          <MenuItem value="daily">Daily</MenuItem>
                          <MenuItem value="weekly">Weekly</MenuItem>
                          <MenuItem value="monthly">Monthly</MenuItem>
                        </Select>
                      </FormControl>
                    </Grid>

                    <Grid item xs={12} sm={6}>
                      <FormControl fullWidth size="small">
                        <FormLabel sx={{ mb: 1 }}>Format</FormLabel>
                        <Select
                          value={exportConfig.format}
                          onChange={(e) => setExportConfig(prev => ({ ...prev, format: e.target.value as ExportFormat }))}
                        >
                          {Object.entries(EXPORT_FORMATS).map(([format, info]) => (
                            <MenuItem key={format} value={format}>
                              <Box display="flex" alignItems="center" gap={1}>
                                {info.icon}
                                {info.name}
                              </Box>
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                    </Grid>

                    <Grid item xs={12}>
                      <TextField
                        fullWidth
                        size="small"
                        label="Recipients (comma-separated emails)"
                        value={scheduleConfig.recipients.join(', ')}
                        onChange={(e) => setScheduleConfig(prev => ({ 
                          ...prev, 
                          recipients: e.target.value.split(',').map(email => email.trim()).filter(Boolean)
                        }))}
                        placeholder="user1@company.com, user2@company.com"
                      />
                    </Grid>
                  </Grid>

                  <Box mt={2}>
                    <Button
                      variant="contained"
                      startIcon={<SaveOutlined />}
                      onClick={() => setScheduleDialogOpen(true)}
                      disabled={!scheduleConfig.enabled || scheduleConfig.recipients.length === 0}
                    >
                      Save Schedule
                    </Button>
                  </Box>
                </CardContent>
              </Card>
            </Box>
          )}

          {/* History Tab */}
          {activeTab === 3 && (
            <Box>
              <Typography variant="h6" fontWeight="600" mb={2}>
                Export History
              </Typography>
              
              <List>
                {exportHistory.map((exportItem, index) => (
                  <React.Fragment key={exportItem.id}>
                    <ListItem sx={{ px: 0 }}>
                      <ListItemIcon>
                        <Avatar
                          sx={{
                            bgcolor: exportItem.status === 'completed' ? 'success.light' : 'warning.light',
                            width: 40,
                            height: 40,
                          }}
                        >
                          {EXPORT_FORMATS[exportItem.format as ExportFormat]?.icon}
                        </Avatar>
                      </ListItemIcon>
                      <ListItemText
                        primary={
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="subtitle2" fontWeight="600">
                              {EXPORT_FORMATS[exportItem.format as ExportFormat]?.name}
                            </Typography>
                            <Chip
                              label={exportItem.status}
                              size="small"
                              color={exportItem.status === 'completed' ? 'success' : 'warning'}
                              variant="outlined"
                            />
                          </Box>
                        }
                        secondary={
                          <Box>
                            <Typography variant="body2" color="text.secondary">
                              {format(new Date(exportItem.timestamp), 'MMM dd, yyyy HH:mm')} • 
                              {exportItem.size} • {exportItem.downloads} downloads
                            </Typography>
                          </Box>
                        }
                      />
                      <Box display="flex" alignItems="center" gap={1}>
                        <Badge badgeContent={exportItem.downloads} color="primary">
                          <Button
                            size="small"
                            startIcon={<CloudDownloadOutlined />}
                            onClick={() => {
                              // Mock download functionality
                              alert(`Download ${exportItem.format} export from ${exportItem.timestamp}`);
                            }}
                          >
                            Download
                          </Button>
                        </Badge>
                        <IconButton
                          size="small"
                          onClick={() => {
                            navigator.clipboard.writeText(`Export ${exportItem.id} link copied`);
                          }}
                        >
                          <ShareOutlined />
                        </IconButton>
                      </Box>
                    </ListItem>
                    {index < exportHistory.length - 1 && <Divider />}
                  </React.Fragment>
                ))}
              </List>

              {exportHistory.length === 0 && (
                <Box display="flex" flexDirection="column" alignItems="center" py={4}>
                  <HistoryOutlined sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                  <Typography variant="h6" color="text.secondary" mb={1}>
                    No Export History
                  </Typography>
                  <Typography variant="body2" color="text.secondary" textAlign="center">
                    Your export history will appear here after you generate reports.
                  </Typography>
                </Box>
              )}
            </Box>
          )}
        </CardContent>
      </Card>

      {/* Export Configuration Dialog */}
      <Dialog
        open={exportDialogOpen}
        onClose={() => setExportDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          Configure Export
          <Typography variant="subtitle2" color="text.secondary">
            {EXPORT_FORMATS[selectedFormat]?.name}
          </Typography>
        </DialogTitle>
        <DialogContent>
          <Box sx={{ width: '100%', mt: 1 }}>
            <Stepper orientation="vertical">
              {/* Format Selection */}
              <Step expanded>
                <StepLabel>Export Format</StepLabel>
                <StepContent>
                  <Grid container spacing={2}>
                    {Object.entries(EXPORT_FORMATS).map(([format, info]) => (
                      <Grid item xs={12} sm={6} key={format}>
                        <Paper
                          sx={{
                            p: 2,
                            cursor: 'pointer',
                            border: '1px solid',
                            borderColor: exportConfig.format === format ? 'primary.main' : 'divider',
                            bgcolor: exportConfig.format === format ? 'action.selected' : 'transparent',
                          }}
                          onClick={() => setExportConfig(prev => ({ ...prev, format: format as ExportFormat }))}
                        >
                          <Box display="flex" alignItems="center" gap={2}>
                            <Avatar sx={{ bgcolor: 'primary.light', width: 32, height: 32 }}>
                              {info.icon}
                            </Avatar>
                            <Box>
                              <Typography variant="body2" fontWeight="600">
                                {info.name}
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                {info.description}
                              </Typography>
                            </Box>
                          </Box>
                        </Paper>
                      </Grid>
                    ))}
                  </Grid>
                </StepContent>
              </Step>

              {/* Content Selection */}
              <Step expanded>
                <StepLabel>Content Options</StepLabel>
                <StepContent>
                  <FormGroup>
                    <FormControlLabel
                      control={
                        <Switch
                          checked={exportConfig.includeCharts}
                          onChange={(e) => setExportConfig(prev => ({ ...prev, includeCharts: e.target.checked }))}
                        />
                      }
                      label="Include Charts and Visualizations"
                    />
                    <FormControlLabel
                      control={
                        <Switch
                          checked={exportConfig.includeDiagrams}
                          onChange={(e) => setExportConfig(prev => ({ ...prev, includeDiagrams: e.target.checked }))}
                        />
                      }
                      label="Include Dependency Diagrams"
                    />
                    <FormControlLabel
                      control={
                        <Switch
                          checked={exportConfig.includeRawData}
                          onChange={(e) => setExportConfig(prev => ({ ...prev, includeRawData: e.target.checked }))}
                        />
                      }
                      label="Include Raw Data Tables"
                    />
                  </FormGroup>

                  <Box mt={2}>
                    <Typography variant="subtitle2" fontWeight="600" mb={1}>
                      Preview Sections
                    </Typography>
                    <List dense>
                      {exportPreview.map((section, index) => (
                        <ListItem key={index} sx={{ py: 0.5 }}>
                          <ListItemIcon>
                            {section.included ? (
                              <CheckCircleOutlined color="success" />
                            ) : (
                              <ErrorOutlined color="disabled" />
                            )}
                          </ListItemIcon>
                          <ListItemText
                            primary={section.name}
                            primaryTypographyProps={{
                              color: section.included ? 'text.primary' : 'text.secondary',
                            }}
                          />
                        </ListItem>
                      ))}
                    </List>
                  </Box>
                </StepContent>
              </Step>

              {/* Branding */}
              <Step expanded>
                <StepLabel>Branding (Optional)</StepLabel>
                <StepContent>
                  <Grid container spacing={2}>
                    <Grid item xs={12} sm={6}>
                      <TextField
                        fullWidth
                        size="small"
                        label="Company Name"
                        value={exportConfig.branding?.companyName || ''}
                        onChange={(e) => setExportConfig(prev => ({
                          ...prev,
                          branding: { ...prev.branding, companyName: e.target.value }
                        }))}
                      />
                    </Grid>
                    <Grid item xs={12} sm={6}>
                      <TextField
                        fullWidth
                        size="small"
                        label="Logo URL"
                        value={exportConfig.branding?.logo || ''}
                        onChange={(e) => setExportConfig(prev => ({
                          ...prev,
                          branding: { ...prev.branding, logo: e.target.value }
                        }))}
                      />
                    </Grid>
                  </Grid>
                </StepContent>
              </Step>
            </Stepper>
          </Box>

          {/* Export progress */}
          {isExporting && (
            <Box mt={3}>
              <Typography variant="body2" mb={1}>
                Generating {EXPORT_FORMATS[exportConfig.format]?.name}...
              </Typography>
              <LinearProgress 
                variant="determinate" 
                value={exportProgress}
                sx={{ height: 8, borderRadius: 4 }}
              />
              <Typography variant="caption" color="text.secondary" mt={0.5}>
                {Math.round(exportProgress)}% complete
              </Typography>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setExportDialogOpen(false)} disabled={isExporting}>
            Cancel
          </Button>
          <Button
            onClick={handleExport}
            variant="contained"
            startIcon={isExporting ? <TimerOutlined /> : <FileDownloadOutlined />}
            disabled={isExporting}
          >
            {isExporting ? 'Exporting...' : `Export ${EXPORT_FORMATS[exportConfig.format]?.name}`}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Schedule Dialog */}
      <Dialog
        open={scheduleDialogOpen}
        onClose={() => setScheduleDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Schedule Automated Reports</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            <Typography variant="body2">
              Reports will be automatically generated and sent based on your configuration.
            </Typography>
          </Alert>
          
          <Box display="flex" flexDirection="column" gap={2} pt={1}>
            <TextField
              fullWidth
              label="Subject Line"
              value={scheduleConfig.subject}
              onChange={(e) => setScheduleConfig(prev => ({ ...prev, subject: e.target.value }))}
            />
            
            <TextField
              fullWidth
              multiline
              rows={3}
              label="Email Body"
              value={scheduleConfig.body}
              onChange={(e) => setScheduleConfig(prev => ({ ...prev, body: e.target.value }))}
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setScheduleDialogOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleScheduleSetup} variant="contained" startIcon={<ScheduleOutlined />}>
            Schedule Reports
          </Button>
        </DialogActions>
      </Dialog>

      {/* Integration Setup Dialog */}
      <Dialog
        open={integrationDialogOpen}
        onClose={() => setIntegrationDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          Setup Integration
          {selectedIntegration && (
            <Typography variant="subtitle2" color="text.secondary">
              {INTEGRATIONS.find(i => i.id === selectedIntegration)?.name}
            </Typography>
          )}
        </DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            <Typography variant="body2">
              Integration setup would provide specific configuration options for each service.
            </Typography>
          </Alert>
          
          {selectedIntegration && (
            <TextField
              fullWidth
              label={`${INTEGRATIONS.find(i => i.id === selectedIntegration)?.name} Configuration`}
              placeholder="Enter API key, webhook URL, or other configuration..."
              multiline
              rows={3}
            />
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIntegrationDialogOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => selectedIntegration && handleIntegrationSetup(selectedIntegration, {})}
            variant="contained"
            startIcon={<IntegrationInstructionsOutlined />}
          >
            Setup Integration
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default ExportHub;