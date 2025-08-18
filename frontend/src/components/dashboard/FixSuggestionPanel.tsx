import React, { useMemo, useCallback, useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  Stepper,
  Step,
  StepLabel,
  StepContent,
  Chip,
  Paper,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Collapse,
  Alert,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Tabs,
  Tab,
  useTheme,
  Divider,
  Rating,
  LinearProgress,
  CodeIcon,
  Avatar,
} from '@mui/material';
import {
  ExpandMoreOutlined,
  PlayArrowOutlined,
  CheckCircleOutlined,
  RadioButtonUncheckedOutlined,
  InfoOutlined,
  WarningOutlined,
  ErrorOutlined,
  LightbulbOutlined,
  CodeOutlined,
  SettingsOutlined,
  BugReportOutlined,
  SecurityOutlined,
  SpeedOutlined,
  BuildOutlined,
  DocumentScannerOutlined,
  ContentCopyOutlined,
  OpenInNewOutlined,
  AccessTimeOutlined,
  TrendingUpOutlined,
  SchoolOutlined,
} from '@mui/icons-material';

import type { Finding } from '../../types/api';
import type {
  FixSuggestion,
  FixStep,
  CodeChange,
  FixAlternative,
  ResponsiveComponentProps,
  ExportableComponentProps,
} from '../../types/dashboard';

import { dashboardTheme } from '../../utils/dashboardTheme';

interface FixSuggestionPanelProps extends ResponsiveComponentProps, ExportableComponentProps {
  findings: Finding[];
  selectedFindingId?: string;
  onSuggestionApply?: (suggestion: FixSuggestion) => void;
  onStepComplete?: (suggestionId: string, stepIndex: number) => void;
  showAlternatives?: boolean;
  showCodeChanges?: boolean;
  interactive?: boolean;
}

// Mock function to generate fix suggestions from findings
const generateFixSuggestions = (findings: Finding[]): FixSuggestion[] => {
  const suggestions: FixSuggestion[] = [];

  findings.forEach(finding => {
    // Generate suggestion based on finding type
    let suggestion: FixSuggestion;

    switch (finding.detector) {
      case 'god_object':
        suggestion = {
          id: `fix_${finding.id}`,
          findingId: finding.id,
          title: 'Extract Smaller Classes',
          description: 'Break down this large class into smaller, more focused components using the Single Responsibility Principle.',
          confidence: 85,
          complexity: 'complex',
          estimatedTime: '4-8 hours',
          steps: [
            {
              order: 1,
              title: 'Identify Responsibilities',
              description: 'Analyze the class to identify distinct responsibilities that can be separated.',
              type: 'code_change',
              validation: 'Each responsibility should have a clear, single purpose.',
            },
            {
              order: 2,
              title: 'Create New Classes',
              description: 'Extract identified responsibilities into separate classes.',
              type: 'refactoring',
              codeSnippet: `// Extract user validation logic
class UserValidator {
  validate(user: User): ValidationResult {
    // Move validation logic here
  }
}

// Extract user persistence logic
class UserRepository {
  save(user: User): Promise<void> {
    // Move database logic here
  }
}`,
              validation: 'New classes should have cohesive, single responsibilities.',
            },
            {
              order: 3,
              title: 'Update Dependencies',
              description: 'Inject the new classes as dependencies into the original class.',
              type: 'refactoring',
              validation: 'Original class should delegate to injected dependencies.',
            },
            {
              order: 4,
              title: 'Update Tests',
              description: 'Modify existing tests and create new tests for the extracted classes.',
              type: 'testing',
              validation: 'All tests pass and coverage is maintained.',
            },
          ],
          codeChanges: [
            {
              filePath: finding.file,
              startLine: finding.line || 1,
              endLine: (finding.line || 1) + 20,
              oldCode: '// Large class with multiple responsibilities',
              newCode: '// Refactored class with single responsibility',
              explanation: 'Extract validation logic into separate validator class',
            },
          ],
          alternatives: [
            {
              title: 'Use Composition Pattern',
              description: 'Instead of inheritance, use composition to combine smaller components.',
              pros: ['More flexible', 'Easier to test', 'Better separation of concerns'],
              cons: ['More initial setup', 'Requires more interfaces'],
              effort: 6,
            },
            {
              title: 'Apply Strategy Pattern',
              description: 'Extract varying algorithms into strategy classes.',
              pros: ['Runtime algorithm switching', 'Easy to extend', 'Clear separation'],
              cons: ['More classes to maintain', 'Overhead for simple cases'],
              effort: 5,
            },
          ],
          prerequisites: [
            'Comprehensive test coverage',
            'Clear understanding of class responsibilities',
            'Agreement on new architecture',
          ],
          risks: [
            'Breaking changes to existing API',
            'Potential for introducing bugs during refactoring',
            'Increased complexity initially',
          ],
          references: [
            'https://refactoring.guru/extract-class',
            'SOLID Principles - Single Responsibility Principle',
          ],
        };
        break;

      case 'circular_dependency':
        suggestion = {
          id: `fix_${finding.id}`,
          findingId: finding.id,
          title: 'Break Circular Dependencies',
          description: 'Resolve circular dependencies using dependency inversion and interface segregation.',
          confidence: 90,
          complexity: 'moderate',
          estimatedTime: '2-4 hours',
          steps: [
            {
              order: 1,
              title: 'Identify Dependency Chain',
              description: 'Map out the circular dependency chain to understand the relationships.',
              type: 'code_change',
              validation: 'Clear visualization of circular dependencies.',
            },
            {
              order: 2,
              title: 'Introduce Interface',
              description: 'Create an interface to break the direct dependency.',
              type: 'refactoring',
              codeSnippet: `// Create interface to break dependency
interface IUserService {
  getUser(id: string): Promise<User>;
}

// Implement interface
class UserService implements IUserService {
  // Implementation
}`,
            },
            {
              order: 3,
              title: 'Apply Dependency Injection',
              description: 'Inject dependencies through constructor or method parameters.',
              type: 'refactoring',
              validation: 'No direct imports between previously circular classes.',
            },
          ],
          alternatives: [
            {
              title: 'Event-Driven Architecture',
              description: 'Use events to decouple components instead of direct calls.',
              pros: ['Complete decoupling', 'Scalable', 'Flexible'],
              cons: ['Complexity', 'Debugging difficulty', 'Event management'],
              effort: 7,
            },
          ],
          prerequisites: ['Understanding of dependency injection', 'Clear interface contracts'],
          risks: ['Runtime errors if dependencies not properly injected'],
          references: ['Dependency Inversion Principle', 'Interface Segregation Principle'],
        };
        break;

      case 'magic_values':
        suggestion = {
          id: `fix_${finding.id}`,
          findingId: finding.id,
          title: 'Extract Constants',
          description: 'Replace magic numbers and strings with named constants to improve code readability.',
          confidence: 95,
          complexity: 'simple',
          estimatedTime: '15-30 minutes',
          steps: [
            {
              order: 1,
              title: 'Identify Magic Values',
              description: 'Find all magic numbers and strings in the code.',
              type: 'code_change',
              validation: 'All hardcoded values are identified.',
            },
            {
              order: 2,
              title: 'Create Constants',
              description: 'Define meaningful constant names for the magic values.',
              type: 'code_change',
              codeSnippet: `// Before
if (user.age >= 18) {
  // logic
}

// After
const LEGAL_AGE = 18;
if (user.age >= LEGAL_AGE) {
  // logic
}`,
            },
            {
              order: 3,
              title: 'Replace Usage',
              description: 'Replace all occurrences of magic values with the constants.',
              type: 'code_change',
              validation: 'No magic values remain in the code.',
            },
          ],
          alternatives: [
            {
              title: 'Configuration File',
              description: 'Move constants to external configuration file.',
              pros: ['Runtime configuration', 'Environment-specific values'],
              cons: ['External dependency', 'More complex deployment'],
              effort: 3,
            },
          ],
          prerequisites: ['Clear understanding of value meanings'],
          risks: ['Minimal - very safe refactoring'],
          references: ['Clean Code - Meaningful Names'],
        };
        break;

      default:
        suggestion = {
          id: `fix_${finding.id}`,
          findingId: finding.id,
          title: 'Generic Fix Approach',
          description: 'Apply best practices to resolve this code quality issue.',
          confidence: 70,
          complexity: 'moderate',
          estimatedTime: '1-2 hours',
          steps: [
            {
              order: 1,
              title: 'Analyze Issue',
              description: 'Understand the root cause of the issue.',
              type: 'code_change',
              validation: 'Issue is clearly understood.',
            },
            {
              order: 2,
              title: 'Apply Fix',
              description: 'Implement the appropriate solution.',
              type: 'refactoring',
              validation: 'Issue is resolved without breaking existing functionality.',
            },
          ],
          alternatives: [],
          prerequisites: ['Code understanding'],
          risks: ['May require testing'],
          references: [],
        };
        break;
    }

    suggestions.push(suggestion);
  });

  return suggestions;
};

const FixSuggestionPanel: React.FC<FixSuggestionPanelProps> = ({
  findings,
  selectedFindingId,
  onSuggestionApply,
  onStepComplete,
  showAlternatives = true,
  showCodeChanges = true,
  interactive = true,
  loading = false,
  error,
  className,
  testId,
}) => {
  const theme = useTheme();
  const [selectedSuggestionId, setSelectedSuggestionId] = useState<string | null>(null);
  const [completedSteps, setCompletedSteps] = useState<Record<string, number[]>>({});
  const [activeTab, setActiveTab] = useState(0);
  const [expandedAccordion, setExpandedAccordion] = useState<string | false>(false);
  const [codeDialogOpen, setCodeDialogOpen] = useState(false);
  const [selectedCodeChange, setSelectedCodeChange] = useState<CodeChange | null>(null);

  // Generate fix suggestions
  const fixSuggestions = useMemo(() => {
    return generateFixSuggestions(findings);
  }, [findings]);

  // Filter suggestions based on selected finding
  const filteredSuggestions = useMemo(() => {
    if (!selectedFindingId) return fixSuggestions;
    return fixSuggestions.filter(suggestion => suggestion.findingId === selectedFindingId);
  }, [fixSuggestions, selectedFindingId]);

  // Get selected suggestion
  const selectedSuggestion = useMemo(() => {
    return fixSuggestions.find(suggestion => suggestion.id === selectedSuggestionId);
  }, [fixSuggestions, selectedSuggestionId]);

  // Get complexity color and icon
  const getComplexityInfo = useCallback((complexity: string) => {
    switch (complexity) {
      case 'simple':
        return { 
          color: 'success', 
          icon: <CheckCircleOutlined />, 
          label: 'Simple',
          description: 'Quick fix, low risk'
        };
      case 'moderate':
        return { 
          color: 'warning', 
          icon: <WarningOutlined />, 
          label: 'Moderate',
          description: 'Requires some planning'
        };
      case 'complex':
        return { 
          color: 'error', 
          icon: <ErrorOutlined />, 
          label: 'Complex',
          description: 'Significant refactoring needed'
        };
      default:
        return { 
          color: 'default', 
          icon: <InfoOutlined />, 
          label: 'Unknown',
          description: 'Complexity not determined'
        };
    }
  }, []);

  // Handle step completion
  const handleStepComplete = useCallback((suggestionId: string, stepIndex: number) => {
    setCompletedSteps(prev => {
      const current = prev[suggestionId] || [];
      const updated = current.includes(stepIndex)
        ? current.filter(i => i !== stepIndex)
        : [...current, stepIndex].sort((a, b) => a - b);
      
      return { ...prev, [suggestionId]: updated };
    });

    if (onStepComplete) {
      onStepComplete(suggestionId, stepIndex);
    }
  }, [onStepComplete]);

  // Handle suggestion apply
  const handleSuggestionApply = useCallback((suggestion: FixSuggestion) => {
    if (onSuggestionApply) {
      onSuggestionApply(suggestion);
    }
  }, [onSuggestionApply]);

  // Handle code change view
  const handleViewCodeChange = useCallback((codeChange: CodeChange) => {
    setSelectedCodeChange(codeChange);
    setCodeDialogOpen(true);
  }, []);

  // Copy code to clipboard
  const handleCopyCode = useCallback(async (code: string) => {
    try {
      await navigator.clipboard.writeText(code);
      // Could show a toast notification here
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  }, []);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={300}>
            <Typography variant="body2" color="text.secondary">
              Loading fix suggestions...
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
          <Box display="flex" alignItems="center" justifyContent="center" height={300}>
            <Alert severity="error" sx={{ maxWidth: 400 }}>
              <Typography variant="body2">
                Error loading suggestions: {error}
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
            Fix Suggestions
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {filteredSuggestions.length} actionable recommendations
          </Typography>
        </Box>
        
        <Tooltip title="AI-powered fix suggestions with step-by-step guidance" arrow>
          <IconButton>
            <InfoOutlined />
          </IconButton>
        </Tooltip>
      </Box>

      {filteredSuggestions.length === 0 ? (
        <Card>
          <CardContent>
            <Box display="flex" flexDirection="column" alignItems="center" py={4}>
              <LightbulbOutlined sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
              <Typography variant="h6" color="text.secondary" mb={1}>
                No Fix Suggestions Available
              </Typography>
              <Typography variant="body2" color="text.secondary" textAlign="center">
                {selectedFindingId 
                  ? 'No suggestions found for the selected finding.'
                  : 'Select a finding to view available fix suggestions.'
                }
              </Typography>
            </Box>
          </CardContent>
        </Card>
      ) : (
        <Grid container spacing={2}>
          {/* Suggestions list */}
          <Grid item xs={12} lg={4}>
            <Card>
              <CardContent sx={{ p: 1 }}>
                <List dense>
                  {filteredSuggestions.map((suggestion, index) => {
                    const complexityInfo = getComplexityInfo(suggestion.complexity);
                    const isSelected = suggestion.id === selectedSuggestionId;
                    const completedCount = completedSteps[suggestion.id]?.length || 0;
                    const totalSteps = suggestion.steps.length;
                    
                    return (
                      <ListItemButton
                        key={suggestion.id}
                        selected={isSelected}
                        onClick={() => setSelectedSuggestionId(suggestion.id)}
                        sx={{
                          borderRadius: 1,
                          mb: 0.5,
                          border: isSelected ? `2px solid ${theme.palette.primary.main}` : '1px solid transparent',
                        }}
                      >
                        <ListItemIcon>
                          <Avatar
                            sx={{
                              bgcolor: complexityInfo.color === 'success' ? 'success.light' : 
                                      complexityInfo.color === 'warning' ? 'warning.light' : 'error.light',
                              width: 32,
                              height: 32,
                            }}
                          >
                            {complexityInfo.icon}
                          </Avatar>
                        </ListItemIcon>
                        <ListItemText
                          primary={
                            <Box>
                              <Typography variant="subtitle2" fontWeight="600">
                                {suggestion.title}
                              </Typography>
                              <Box display="flex" alignItems="center" gap={0.5} mt={0.5}>
                                <Chip
                                  label={complexityInfo.label}
                                  size="small"
                                  color={complexityInfo.color as any}
                                  variant="outlined"
                                />
                                <Chip
                                  label={suggestion.estimatedTime}
                                  size="small"
                                  icon={<AccessTimeOutlined />}
                                  variant="outlined"
                                />
                              </Box>
                            </Box>
                          }
                          secondary={
                            <Box mt={1}>
                              <Box display="flex" alignItems="center" gap={0.5} mb={0.5}>
                                <Rating
                                  value={suggestion.confidence / 20}
                                  readOnly
                                  size="small"
                                  precision={0.1}
                                />
                                <Typography variant="caption" color="text.secondary">
                                  {suggestion.confidence}% confidence
                                </Typography>
                              </Box>
                              
                              <LinearProgress
                                variant="determinate"
                                value={(completedCount / totalSteps) * 100}
                                sx={{
                                  height: 4,
                                  borderRadius: 2,
                                  backgroundColor: 'action.hover',
                                }}
                              />
                              <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
                                {completedCount}/{totalSteps} steps completed
                              </Typography>
                            </Box>
                          }
                        />
                      </ListItemButton>
                    );
                  })}
                </List>
              </CardContent>
            </Card>
          </Grid>

          {/* Suggestion details */}
          <Grid item xs={12} lg={8}>
            {selectedSuggestion ? (
              <Card>
                <CardContent>
                  {/* Suggestion header */}
                  <Box display="flex" alignItems="flex-start" justifyContent="space-between" mb={3}>
                    <Box flex={1}>
                      <Typography variant="h6" fontWeight="600" mb={1}>
                        {selectedSuggestion.title}
                      </Typography>
                      <Typography variant="body2" color="text.secondary" mb={2}>
                        {selectedSuggestion.description}
                      </Typography>
                      
                      <Box display="flex" flexWrap="wrap" gap={1} mb={2}>
                        {getComplexityInfo(selectedSuggestion.complexity).color && (
                          <Chip
                            icon={getComplexityInfo(selectedSuggestion.complexity).icon}
                            label={getComplexityInfo(selectedSuggestion.complexity).label}
                            color={getComplexityInfo(selectedSuggestion.complexity).color as any}
                            variant="outlined"
                          />
                        )}
                        <Chip
                          icon={<AccessTimeOutlined />}
                          label={selectedSuggestion.estimatedTime}
                          variant="outlined"
                        />
                        <Chip
                          icon={<TrendingUpOutlined />}
                          label={`${selectedSuggestion.confidence}% confidence`}
                          variant="outlined"
                        />
                      </Box>
                    </Box>

                    {interactive && (
                      <Button
                        variant="contained"
                        startIcon={<PlayArrowOutlined />}
                        onClick={() => handleSuggestionApply(selectedSuggestion)}
                        sx={{ ml: 2 }}
                      >
                        Apply Fix
                      </Button>
                    )}
                  </Box>

                  {/* Tabs for different sections */}
                  <Tabs value={activeTab} onChange={(_, newValue) => setActiveTab(newValue)} sx={{ mb: 2 }}>
                    <Tab label="Steps" />
                    {showCodeChanges && selectedSuggestion.codeChanges && (
                      <Tab label={`Code Changes (${selectedSuggestion.codeChanges.length})`} />
                    )}
                    {showAlternatives && selectedSuggestion.alternatives && selectedSuggestion.alternatives.length > 0 && (
                      <Tab label={`Alternatives (${selectedSuggestion.alternatives.length})`} />
                    )}
                    <Tab label="Details" />
                  </Tabs>

                  {/* Steps tab */}
                  {activeTab === 0 && (
                    <Box>
                      <Stepper orientation="vertical" nonLinear={interactive}>
                        {selectedSuggestion.steps.map((step, index) => {
                          const isCompleted = completedSteps[selectedSuggestion.id]?.includes(index) || false;
                          
                          return (
                            <Step key={step.order} completed={isCompleted}>
                              <StepLabel
                                optional={
                                  <Typography variant="caption" color="text.secondary">
                                    {step.type.replace('_', ' ')}
                                  </Typography>
                                }
                                StepIconComponent={({ active, completed }) => (
                                  <Box
                                    sx={{
                                      width: 24,
                                      height: 24,
                                      borderRadius: '50%',
                                      display: 'flex',
                                      alignItems: 'center',
                                      justifyContent: 'center',
                                      bgcolor: completed ? 'success.main' : active ? 'primary.main' : 'action.disabled',
                                      color: 'white',
                                      cursor: interactive ? 'pointer' : 'default',
                                    }}
                                    onClick={interactive ? () => handleStepComplete(selectedSuggestion.id, index) : undefined}
                                  >
                                    {completed ? (
                                      <CheckCircleOutlined sx={{ fontSize: 16 }} />
                                    ) : (
                                      <Typography variant="caption" fontWeight="600">
                                        {step.order}
                                      </Typography>
                                    )}
                                  </Box>
                                )}
                              >
                                <Typography variant="subtitle2" fontWeight="600">
                                  {step.title}
                                </Typography>
                              </StepLabel>
                              <StepContent>
                                <Typography variant="body2" color="text.secondary" mb={2}>
                                  {step.description}
                                </Typography>

                                {step.codeSnippet && (
                                  <Paper
                                    sx={{
                                      p: 2,
                                      bgcolor: 'grey.50',
                                      border: '1px solid',
                                      borderColor: 'divider',
                                      fontFamily: 'monospace',
                                      fontSize: '0.875rem',
                                      mb: 2,
                                      position: 'relative',
                                    }}
                                  >
                                    <Box
                                      position="absolute"
                                      top={8}
                                      right={8}
                                      display="flex"
                                      gap={0.5}
                                    >
                                      <IconButton
                                        size="small"
                                        onClick={() => handleCopyCode(step.codeSnippet!)}
                                        title="Copy code"
                                      >
                                        <ContentCopyOutlined fontSize="small" />
                                      </IconButton>
                                    </Box>
                                    <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
                                      {step.codeSnippet}
                                    </pre>
                                  </Paper>
                                )}

                                {step.validation && (
                                  <Alert severity="info" sx={{ mb: 2 }}>
                                    <Typography variant="body2">
                                      <strong>Validation:</strong> {step.validation}
                                    </Typography>
                                  </Alert>
                                )}

                                {interactive && (
                                  <Button
                                    variant={isCompleted ? 'outlined' : 'contained'}
                                    size="small"
                                    startIcon={isCompleted ? <RadioButtonUncheckedOutlined /> : <CheckCircleOutlined />}
                                    onClick={() => handleStepComplete(selectedSuggestion.id, index)}
                                    sx={{ mt: 1 }}
                                  >
                                    {isCompleted ? 'Mark Incomplete' : 'Mark Complete'}
                                  </Button>
                                )}
                              </StepContent>
                            </Step>
                          );
                        })}
                      </Stepper>
                    </Box>
                  )}

                  {/* Code changes tab */}
                  {activeTab === 1 && showCodeChanges && selectedSuggestion.codeChanges && (
                    <Box>
                      {selectedSuggestion.codeChanges.map((change, index) => (
                        <Paper
                          key={index}
                          sx={{ p: 2, mb: 2, border: '1px solid', borderColor: 'divider' }}
                        >
                          <Box display="flex" alignItems="center" justifyContent="between" mb={2}>
                            <Box>
                              <Typography variant="subtitle2" fontWeight="600">
                                {change.filePath}
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                Lines {change.startLine}-{change.endLine}
                              </Typography>
                            </Box>
                            <Button
                              variant="outlined"
                              size="small"
                              startIcon={<OpenInNewOutlined />}
                              onClick={() => handleViewCodeChange(change)}
                            >
                              View Details
                            </Button>
                          </Box>
                          
                          <Typography variant="body2" color="text.secondary" mb={2}>
                            {change.explanation}
                          </Typography>

                          <Grid container spacing={2}>
                            <Grid item xs={12} md={6}>
                              <Typography variant="caption" color="error" fontWeight="600">
                                BEFORE
                              </Typography>
                              <Paper sx={{ p: 1, bgcolor: 'error.light', fontFamily: 'monospace', fontSize: '0.75rem' }}>
                                <pre style={{ margin: 0 }}>{change.oldCode}</pre>
                              </Paper>
                            </Grid>
                            <Grid item xs={12} md={6}>
                              <Typography variant="caption" color="success" fontWeight="600">
                                AFTER
                              </Typography>
                              <Paper sx={{ p: 1, bgcolor: 'success.light', fontFamily: 'monospace', fontSize: '0.75rem' }}>
                                <pre style={{ margin: 0 }}>{change.newCode}</pre>
                              </Paper>
                            </Grid>
                          </Grid>
                        </Paper>
                      ))}
                    </Box>
                  )}

                  {/* Alternatives tab */}
                  {activeTab === (showCodeChanges && selectedSuggestion.codeChanges ? 2 : 1) && 
                   showAlternatives && selectedSuggestion.alternatives && selectedSuggestion.alternatives.length > 0 && (
                    <Box>
                      {selectedSuggestion.alternatives.map((alternative, index) => (
                        <Accordion
                          key={index}
                          expanded={expandedAccordion === `alt_${index}`}
                          onChange={(_, isExpanded) => setExpandedAccordion(isExpanded ? `alt_${index}` : false)}
                        >
                          <AccordionSummary expandIcon={<ExpandMoreOutlined />}>
                            <Box display="flex" alignItems="center" gap={2} width="100%">
                              <Typography variant="subtitle2" fontWeight="600">
                                {alternative.title}
                              </Typography>
                              <Chip
                                label={`${alternative.effort}h effort`}
                                size="small"
                                variant="outlined"
                                icon={<AccessTimeOutlined />}
                              />
                            </Box>
                          </AccordionSummary>
                          <AccordionDetails>
                            <Typography variant="body2" color="text.secondary" mb={2}>
                              {alternative.description}
                            </Typography>

                            <Grid container spacing={2}>
                              <Grid item xs={12} md={6}>
                                <Typography variant="subtitle2" fontWeight="600" color="success.main" mb={1}>
                                  Pros
                                </Typography>
                                <List dense>
                                  {alternative.pros.map((pro, proIndex) => (
                                    <ListItem key={proIndex} sx={{ py: 0 }}>
                                      <ListItemIcon sx={{ minWidth: 24 }}>
                                        <CheckCircleOutlined color="success" fontSize="small" />
                                      </ListItemIcon>
                                      <ListItemText>
                                        <Typography variant="body2">{pro}</Typography>
                                      </ListItemText>
                                    </ListItem>
                                  ))}
                                </List>
                              </Grid>

                              <Grid item xs={12} md={6}>
                                <Typography variant="subtitle2" fontWeight="600" color="error.main" mb={1}>
                                  Cons
                                </Typography>
                                <List dense>
                                  {alternative.cons.map((con, conIndex) => (
                                    <ListItem key={conIndex} sx={{ py: 0 }}>
                                      <ListItemIcon sx={{ minWidth: 24 }}>
                                        <ErrorOutlined color="error" fontSize="small" />
                                      </ListItemIcon>
                                      <ListItemText>
                                        <Typography variant="body2">{con}</Typography>
                                      </ListItemText>
                                    </ListItem>
                                  ))}
                                </List>
                              </Grid>
                            </Grid>
                          </AccordionDetails>
                        </Accordion>
                      ))}
                    </Box>
                  )}

                  {/* Details tab */}
                  {activeTab === (
                    showCodeChanges && selectedSuggestion.codeChanges ? 
                      (showAlternatives && selectedSuggestion.alternatives && selectedSuggestion.alternatives.length > 0 ? 3 : 2) :
                      (showAlternatives && selectedSuggestion.alternatives && selectedSuggestion.alternatives.length > 0 ? 2 : 1)
                  ) && (
                    <Grid container spacing={2}>
                      {/* Prerequisites */}
                      {selectedSuggestion.prerequisites && selectedSuggestion.prerequisites.length > 0 && (
                        <Grid item xs={12} md={6}>
                          <Paper sx={{ p: 2, border: '1px solid', borderColor: 'info.main', bgcolor: 'info.light' }}>
                            <Typography variant="subtitle2" fontWeight="600" mb={1}>
                              <SchoolOutlined sx={{ mr: 1, verticalAlign: 'middle' }} />
                              Prerequisites
                            </Typography>
                            <List dense>
                              {selectedSuggestion.prerequisites.map((prerequisite, index) => (
                                <ListItem key={index} sx={{ py: 0 }}>
                                  <ListItemIcon sx={{ minWidth: 24 }}>
                                    <InfoOutlined fontSize="small" />
                                  </ListItemIcon>
                                  <ListItemText>
                                    <Typography variant="body2">{prerequisite}</Typography>
                                  </ListItemText>
                                </ListItem>
                              ))}
                            </List>
                          </Paper>
                        </Grid>
                      )}

                      {/* Risks */}
                      {selectedSuggestion.risks && selectedSuggestion.risks.length > 0 && (
                        <Grid item xs={12} md={6}>
                          <Paper sx={{ p: 2, border: '1px solid', borderColor: 'warning.main', bgcolor: 'warning.light' }}>
                            <Typography variant="subtitle2" fontWeight="600" mb={1}>
                              <WarningOutlined sx={{ mr: 1, verticalAlign: 'middle' }} />
                              Risks & Considerations
                            </Typography>
                            <List dense>
                              {selectedSuggestion.risks.map((risk, index) => (
                                <ListItem key={index} sx={{ py: 0 }}>
                                  <ListItemIcon sx={{ minWidth: 24 }}>
                                    <WarningOutlined fontSize="small" />
                                  </ListItemIcon>
                                  <ListItemText>
                                    <Typography variant="body2">{risk}</Typography>
                                  </ListItemText>
                                </ListItem>
                              ))}
                            </List>
                          </Paper>
                        </Grid>
                      )}

                      {/* References */}
                      {selectedSuggestion.references && selectedSuggestion.references.length > 0 && (
                        <Grid item xs={12}>
                          <Paper sx={{ p: 2, border: '1px solid', borderColor: 'divider' }}>
                            <Typography variant="subtitle2" fontWeight="600" mb={1}>
                              <DocumentScannerOutlined sx={{ mr: 1, verticalAlign: 'middle' }} />
                              References & Further Reading
                            </Typography>
                            <List dense>
                              {selectedSuggestion.references.map((reference, index) => (
                                <ListItem key={index} sx={{ py: 0 }}>
                                  <ListItemIcon sx={{ minWidth: 24 }}>
                                    <OpenInNewOutlined fontSize="small" />
                                  </ListItemIcon>
                                  <ListItemText>
                                    <Typography 
                                      variant="body2" 
                                      color="primary" 
                                      sx={{ cursor: 'pointer', textDecoration: 'underline' }}
                                      onClick={() => {
                                        if (reference.startsWith('http')) {
                                          window.open(reference, '_blank');
                                        }
                                      }}
                                    >
                                      {reference}
                                    </Typography>
                                  </ListItemText>
                                </ListItem>
                              ))}
                            </List>
                          </Paper>
                        </Grid>
                      )}
                    </Grid>
                  )}
                </CardContent>
              </Card>
            ) : (
              <Card>
                <CardContent>
                  <Box display="flex" flexDirection="column" alignItems="center" py={4}>
                    <LightbulbOutlined sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                    <Typography variant="h6" color="text.secondary" mb={1}>
                      Select a Suggestion
                    </Typography>
                    <Typography variant="body2" color="text.secondary" textAlign="center">
                      Choose a fix suggestion from the list to view detailed implementation steps.
                    </Typography>
                  </Box>
                </CardContent>
              </Card>
            )}
          </Grid>
        </Grid>
      )}

      {/* Code change details dialog */}
      <Dialog
        open={codeDialogOpen}
        onClose={() => setCodeDialogOpen(false)}
        maxWidth="lg"
        fullWidth
      >
        <DialogTitle>
          Code Change Details
          {selectedCodeChange && (
            <Typography variant="subtitle2" color="text.secondary">
              {selectedCodeChange.filePath} (lines {selectedCodeChange.startLine}-{selectedCodeChange.endLine})
            </Typography>
          )}
        </DialogTitle>
        <DialogContent>
          {selectedCodeChange && (
            <Box>
              <Typography variant="body2" color="text.secondary" mb={2}>
                {selectedCodeChange.explanation}
              </Typography>

              <Grid container spacing={2}>
                <Grid item xs={12} md={6}>
                  <Typography variant="subtitle2" color="error" fontWeight="600" mb={1}>
                    Before (Old Code)
                  </Typography>
                  <Paper
                    sx={{
                      p: 2,
                      bgcolor: 'grey.50',
                      fontFamily: 'monospace',
                      fontSize: '0.875rem',
                      maxHeight: 400,
                      overflow: 'auto',
                      border: '1px solid',
                      borderColor: 'error.light',
                    }}
                  >
                    <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
                      {selectedCodeChange.oldCode}
                    </pre>
                  </Paper>
                </Grid>

                <Grid item xs={12} md={6}>
                  <Box display="flex" alignItems="center" justifyContent="between" mb={1}>
                    <Typography variant="subtitle2" color="success" fontWeight="600">
                      After (New Code)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => handleCopyCode(selectedCodeChange.newCode)}
                      title="Copy new code"
                    >
                      <ContentCopyOutlined fontSize="small" />
                    </IconButton>
                  </Box>
                  <Paper
                    sx={{
                      p: 2,
                      bgcolor: 'grey.50',
                      fontFamily: 'monospace',
                      fontSize: '0.875rem',
                      maxHeight: 400,
                      overflow: 'auto',
                      border: '1px solid',
                      borderColor: 'success.light',
                    }}
                  >
                    <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
                      {selectedCodeChange.newCode}
                    </pre>
                  </Paper>
                </Grid>
              </Grid>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCodeDialogOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default FixSuggestionPanel;