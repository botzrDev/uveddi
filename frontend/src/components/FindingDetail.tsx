import React, { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Tabs,
  Tab,
  Chip,
  Alert,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Grid,
  Button,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  Code,
  Psychology,
  AccountTree,
  Lightbulb,
  Info,
  Warning,
  Error as ErrorIcon,
  CheckCircle,
  FileCopy,
  OpenInNew,
} from '@mui/icons-material';
import type { Finding } from '@/types/api';
import CodeSnippet from './CodeSnippet';
import MermaidDiagram, { generateDiagramForFinding } from './MermaidDiagram';

interface FindingDetailProps {
  finding: Finding;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel({ children, value, index, ...other }: TabPanelProps) {
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`finding-tabpanel-${index}`}
      aria-labelledby={`finding-tab-${index}`}
      {...other}
    >
      {value === index && (
        <Box sx={{ py: 3 }}>
          {children}
        </Box>
      )}
    </div>
  );
}

export default function FindingDetail({ finding }: FindingDetailProps) {
  const [activeTab, setActiveTab] = useState(0);

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return <ErrorIcon color="error" />;
      case 'high':
        return <Warning color="warning" />;
      case 'medium':
        return <Info color="info" />;
      case 'low':
        return <CheckCircle color="success" />;
      default:
        return <Info color="action" />;
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'error' as const;
      case 'high':
        return 'warning' as const;
      case 'medium':
        return 'info' as const;
      case 'low':
        return 'success' as const;
      default:
        return 'default' as const;
    }
  };

  const getLanguageFromFile = (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'rs':
        return 'rust';
      case 'py':
        return 'python';
      case 'js':
        return 'javascript';
      case 'ts':
        return 'typescript';
      case 'java':
        return 'java';
      case 'go':
        return 'go';
      case 'cpp':
      case 'cc':
      case 'cxx':
        return 'cpp';
      case 'c':
        return 'c';
      default:
        return 'text';
    }
  };

  // For now, let's use a simple test diagram to verify Mermaid is working
  const testDiagram = `graph TD
    A[${finding.type}] --> B[${finding.severity}]
    B --> C[${finding.file}]
    
    style A fill:#f9f,stroke:#333,stroke-width:4px
    style B fill:#bbf,stroke:#333,stroke-width:2px`;
  
  // Generate a diagram based on the finding type
  const diagramDefinition = generateDiagramForFinding(finding);
  console.log('FindingDetail: Generated diagram definition for finding type', finding.type, ':', diagramDefinition ? diagramDefinition.substring(0, 100) + '...' : 'EMPTY OR NULL');
  
  const finalDiagram = diagramDefinition && diagramDefinition.trim() ? diagramDefinition : testDiagram;

  return (
    <Paper sx={{ mt: 2, overflow: 'hidden' }}>
      {/* Header with finding summary */}
      <Box sx={{ p: 3, bgcolor: 'grey.50', borderBottom: 1, borderColor: 'divider' }}>
        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} md={8}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
              {getSeverityIcon(finding.severity)}
              <Typography variant="h5" component="h3">
                {finding.title}
              </Typography>
            </Box>
            
            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mb: 2 }}>
              <Chip 
                label={finding.severity} 
                size="small" 
                color={getSeverityColor(finding.severity)}
              />
              <Chip 
                label={finding.type} 
                size="small" 
                variant="outlined"
              />
              <Chip 
                label={finding.detector} 
                size="small" 
                variant="outlined"
                color="secondary"
              />
              {finding.confidence && (
                <Chip 
                  label={`${Math.round(finding.confidence * 100)}% confidence`} 
                  size="small" 
                  variant="outlined"
                />
              )}
            </Box>
            
            <Typography variant="body1" color="text.secondary">
              {finding.message}
            </Typography>
          </Grid>
          
          <Grid item xs={12} md={4}>
            <Box sx={{ textAlign: { xs: 'left', md: 'right' } }}>
              <Typography variant="body2" color="text.secondary" gutterBottom>
                <strong>File:</strong> {finding.file}
              </Typography>
              {finding.startLine && (
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  <strong>Lines:</strong> {finding.startLine}
                  {finding.endLine && finding.endLine !== finding.startLine && `-${finding.endLine}`}
                </Typography>
              )}
              {finding.column && (
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  <strong>Column:</strong> {finding.column}
                </Typography>
              )}
              
              <Box sx={{ mt: 2, display: 'flex', gap: 1, justifyContent: { xs: 'flex-start', md: 'flex-end' } }}>
                <Tooltip title="Copy file path">
                  <IconButton 
                    size="small"
                    onClick={() => navigator.clipboard.writeText(finding.file)}
                  >
                    <FileCopy />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Open in editor">
                  <IconButton size="small">
                    <OpenInNew />
                  </IconButton>
                </Tooltip>
              </Box>
            </Box>
          </Grid>
        </Grid>
      </Box>

      {/* Tabs for different views */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={activeTab} onChange={handleTabChange} aria-label="finding detail tabs">
          <Tab 
            icon={<Code />} 
            label="Code" 
            id="finding-tab-0"
            aria-controls="finding-tabpanel-0"
          />
          <Tab 
            icon={<AccountTree />} 
            label="Diagram" 
            id="finding-tab-1"
            aria-controls="finding-tabpanel-1"
          />
          {finding.aiExplanation && (
            <Tab 
              icon={<Psychology />} 
              label="AI Analysis" 
              id="finding-tab-2"
              aria-controls="finding-tabpanel-2"
            />
          )}
          {finding.recommendation && (
            <Tab 
              icon={<Lightbulb />} 
              label="Recommendations" 
              id="finding-tab-3"
              aria-controls="finding-tabpanel-3"
            />
          )}
        </Tabs>
      </Box>

      {/* Tab Panels */}
      <Box sx={{ p: 3 }}>
        {/* Code Tab */}
        <TabPanel value={activeTab} index={0}>
          <Typography variant="h6" gutterBottom>
            Code Snippet
          </Typography>
          
          {finding.codeSnippet ? (
            <CodeSnippet
              code={finding.codeSnippet}
              language={getLanguageFromFile(finding.file)}
              title={`${finding.file}`}
              startLine={finding.startLine}
              endLine={finding.endLine}
              highlightLines={finding.startLine && finding.endLine ? 
                Array.from(
                  { length: finding.endLine - finding.startLine + 1 }, 
                  (_, i) => finding.startLine! + i
                ) : []
              }
            />
          ) : (
            <Alert severity="info">
              <Typography variant="body2">
                No code snippet available for this finding.
              </Typography>
            </Alert>
          )}
          
          {/* Additional context */}
          {finding.tags && finding.tags.length > 0 && (
            <Box sx={{ mt: 3 }}>
              <Typography variant="h6" gutterBottom>
                Tags
              </Typography>
              <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
                {finding.tags.map((tag, index) => (
                  <Chip 
                    key={index} 
                    label={tag} 
                    size="small" 
                    variant="outlined" 
                  />
                ))}
              </Box>
            </Box>
          )}
        </TabPanel>

        {/* Diagram Tab */}
        <TabPanel value={activeTab} index={1}>
          <Typography variant="h6" gutterBottom>
            Architectural Visualization
          </Typography>
          
          <MermaidDiagram 
            definition={finalDiagram}
            title={`${finding.type} Analysis`}
          />
          
          <Box sx={{ mt: 2 }}>
            <Alert severity="info">
              <Typography variant="body2">
                This diagram shows the architectural context and impact of the detected issue.
                The visualization helps understand the relationships and dependencies involved.
              </Typography>
            </Alert>
          </Box>
        </TabPanel>

        {/* AI Analysis Tab */}
        {finding.aiExplanation && (
          <TabPanel value={activeTab} index={2}>
            <Typography variant="h6" gutterBottom>
              AI-Powered Analysis
            </Typography>
            
            <Paper sx={{ p: 3, bgcolor: 'grey.50' }}>
              <Typography variant="body1" sx={{ whiteSpace: 'pre-wrap' }}>
                {finding.aiExplanation}
              </Typography>
            </Paper>
            
            <Box sx={{ mt: 2 }}>
              <Alert severity="info">
                <Typography variant="body2">
                  This analysis was generated by AI and provides context about the issue,
                  its potential impact, and suggested approaches for resolution.
                </Typography>
              </Alert>
            </Box>
          </TabPanel>
        )}

        {/* Recommendations Tab */}
        {finding.recommendation && (
          <TabPanel value={activeTab} index={3}>
            <Typography variant="h6" gutterBottom>
              Remediation Recommendations
            </Typography>
            
            <Paper sx={{ p: 3, bgcolor: 'success.light', color: 'success.contrastText' }}>
              <Typography variant="body1" sx={{ whiteSpace: 'pre-wrap' }}>
                {finding.recommendation}
              </Typography>
            </Paper>
            
            {/* Related findings */}
            {finding.relatedFindings && finding.relatedFindings.length > 0 && (
              <Box sx={{ mt: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Related Findings
                </Typography>
                <List>
                  {finding.relatedFindings.map((relatedId, index) => (
                    <ListItem key={index}>
                      <ListItemIcon>
                        <Info color="info" />
                      </ListItemIcon>
                      <ListItemText
                        primary={`Finding ID: ${relatedId}`}
                        secondary="Click to view related issue"
                      />
                      <Button 
                        size="small" 
                        variant="outlined"
                        onClick={() => {
                          // TODO: Navigate to related finding
                          console.log('Navigate to finding:', relatedId);
                        }}
                      >
                        View
                      </Button>
                    </ListItem>
                  ))}
                </List>
              </Box>
            )}
          </TabPanel>
        )}
      </Box>
    </Paper>
  );
}