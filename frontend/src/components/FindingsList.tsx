import type { Finding } from '@/types/api';
import {
    BugReport,
    Error as ErrorIcon,
    ExpandLess,
    ExpandMore,
    Info,
    Warning,
} from '@mui/icons-material';
import {
    Alert,
    Box,
    Chip,
    Collapse,
    Divider,
    FormControl,
    Grid,
    IconButton,
    InputLabel,
    List,
    ListItem,
    ListItemButton,
    MenuItem,
    Paper,
    Select,
    TextField,
    Typography,
    useTheme
} from '@mui/material';
import React, { useState } from 'react';
import FindingDetail from './FindingDetail';

interface FindingsListProps {
  findings: Finding[];
  loading?: boolean;
}

export default function FindingsList({ findings, loading }: FindingsListProps) {
  const theme = useTheme();
  const [selectedFinding, setSelectedFinding] = useState<Finding | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [severityFilter, setSeverityFilter] = useState<string>('all');
  const [typeFilter, setTypeFilter] = useState<string>('all');

  // Filter findings based on search and filters
  const filteredFindings = findings.filter(finding => {
    const matchesSearch = 
      finding.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
      finding.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
      finding.file.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesSeverity = severityFilter === 'all' || finding.severity === severityFilter;
    const matchesType = typeFilter === 'all' || finding.type === typeFilter;
    
    return matchesSearch && matchesSeverity && matchesType;
  });

  // Get unique types for filter dropdown
  const uniqueTypes = Array.from(new Set(findings.map(f => f.type)));

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return <ErrorIcon color="error" />;
      case 'high':
        return <Warning color="warning" />;
      case 'medium':
        return <Info color="info" />;
      case 'low':
        return <BugReport color="action" />;
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

  const getSeverityChipStyle = (severity: string) => {
    const isDark = theme.palette.mode === 'dark';
    
    switch (severity) {
      case 'critical':
        return {
          backgroundColor: isDark ? '#d32f2f' : '#ffebee',
          color: isDark ? '#ffffff' : '#c62828',
        };
      case 'high':
        return {
          backgroundColor: isDark ? '#f57c00' : '#fff8e1',
          color: isDark ? '#ffffff' : '#ef6c00',
        };
      case 'medium':
        return {
          backgroundColor: isDark ? '#1976d2' : '#e3f2fd',
          color: isDark ? '#ffffff' : '#1565c0',
        };
      case 'low':
        return {
          backgroundColor: isDark ? '#388e3c' : '#e8f5e8',
          color: isDark ? '#ffffff' : '#2e7d32',
        };
      default:
        return {
          backgroundColor: theme.palette.action.hover,
          color: theme.palette.text.primary,
        };
    }
  };

  if (loading) {
    return (
      <Paper sx={{ p: 3 }}>
        <Typography>Loading findings...</Typography>
      </Paper>
    );
  }

  if (findings.length === 0) {
    return (
      <Paper sx={{ p: 3 }}>
        <Alert 
          severity="success"
          sx={{ 
            '& .MuiAlert-message': { 
              display: 'flex', 
              flexDirection: 'column', 
              width: '100%' 
            } 
          }}
        >
          <Box component="div">
            <Typography variant="h6" component="div">No Issues Found</Typography>
            <Typography variant="body2" component="div">
              Great! No architectural issues were detected in this codebase.
            </Typography>
          </Box>
        </Alert>
      </Paper>
    );
  }

  return (
    <Box>
      {/* Filters Section */}
      <Paper 
        sx={{ 
          p: 3, 
          mb: 3,
          backgroundColor: (theme) => theme.palette.background.paper,
          border: (theme) => `1px solid ${theme.palette.divider}`,
        }}
      >
        <Typography variant="h5" gutterBottom>
          Analysis Findings ({filteredFindings.length} of {findings.length})
        </Typography>
        
        <Grid container spacing={2} sx={{ mb: 2 }}>
          <Grid item xs={12} md={6}>
            <TextField
              fullWidth
              label="Search findings"
              variant="outlined"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search by title, message, or file name..."
            />
          </Grid>
          
          <Grid item xs={12} md={3}>
            <FormControl fullWidth>
              <InputLabel>Severity</InputLabel>
              <Select
                value={severityFilter}
                label="Severity"
                onChange={(e) => setSeverityFilter(e.target.value)}
              >
                <MenuItem value="all">All Severities</MenuItem>
                <MenuItem value="critical">Critical</MenuItem>
                <MenuItem value="high">High</MenuItem>
                <MenuItem value="medium">Medium</MenuItem>
                <MenuItem value="low">Low</MenuItem>
              </Select>
            </FormControl>
          </Grid>
          
          <Grid item xs={12} md={3}>
            <FormControl fullWidth>
              <InputLabel>Type</InputLabel>
              <Select
                value={typeFilter}
                label="Type"
                onChange={(e) => setTypeFilter(e.target.value)}
              >
                <MenuItem value="all">All Types</MenuItem>
                {uniqueTypes.map(type => (
                  <MenuItem key={type} value={type}>{type}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
        </Grid>
      </Paper>

      {/* Findings List */}
      <Paper 
        sx={{ 
          mb: 3,
          backgroundColor: (theme) => theme.palette.background.paper,
          border: (theme) => `1px solid ${theme.palette.divider}`,
        }}
      >
        <List sx={{ p: 2 }}>
          {filteredFindings.map((finding, index) => (
            <React.Fragment key={finding.id}>
              <ListItem disablePadding>
                <ListItemButton
                  onClick={() => setSelectedFinding(
                    selectedFinding?.id === finding.id ? null : finding
                  )}
                  sx={{ 
                    p: 2,
                    backgroundColor: (theme) => 
                      selectedFinding?.id === finding.id 
                        ? theme.palette.action.selected 
                        : 'transparent',
                    '&:hover': {
                      backgroundColor: (theme) => theme.palette.action.hover,
                    },
                    borderRadius: 1,
                    border: (theme) => `1px solid ${theme.palette.divider}`,
                    mb: 1,
                  }}
                >
                  <Box sx={{ display: 'flex', alignItems: 'center', mr: 2 }}>
                    {getSeverityIcon(finding.severity)}
                  </Box>
                  
                  <Box sx={{ flex: 1 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                      <Typography variant="h6" component="div">
                        {finding.title}
                      </Typography>
                      <Chip 
                        label={finding.severity} 
                        size="small" 
                        sx={getSeverityChipStyle(finding.severity)}
                      />
                      <Chip 
                        label={finding.type} 
                        size="small" 
                        variant="outlined"
                        sx={{
                          borderColor: theme.palette.divider,
                          color: theme.palette.text.secondary,
                        }}
                      />
                    </Box>
                    <Box>
                      <Typography variant="body2" color="text.secondary" component="div" gutterBottom>
                        {finding.message}
                      </Typography>
                      <Typography variant="caption" color="text.secondary" component="div">
                        📁 {finding.file}
                        {finding.startLine && ` (Line ${finding.startLine}${finding.endLine ? `-${finding.endLine}` : ''})`}
                        {finding.confidence && ` • Confidence: ${Math.round(finding.confidence * 100)}%`}
                      </Typography>
                    </Box>
                  </Box>
                  
                  <IconButton edge="end">
                    {selectedFinding?.id === finding.id ? <ExpandLess /> : <ExpandMore />}
                  </IconButton>
                </ListItemButton>
              </ListItem>
              
              <Collapse in={selectedFinding?.id === finding.id} timeout="auto" unmountOnExit>
                <Box sx={{ px: 2, pb: 2 }}>
                  {selectedFinding && <FindingDetail finding={selectedFinding} />}
                </Box>
              </Collapse>
              
              {index < filteredFindings.length - 1 && <Divider />}
            </React.Fragment>
          ))}
        </List>
      </Paper>

      {filteredFindings.length === 0 && searchTerm && (
        <Paper sx={{ p: 3 }}>
          <Alert 
            severity="info"
            sx={{ 
              '& .MuiAlert-message': { 
                display: 'flex', 
                flexDirection: 'column', 
                width: '100%' 
              } 
            }}
          >
            <Box component="div">
              <Typography variant="h6" component="div">No findings match your search</Typography>
              <Typography variant="body2" component="div">
                Try adjusting your search terms or filters to see more results.
              </Typography>
            </Box>
          </Alert>
        </Paper>
      )}
    </Box>
  );
}