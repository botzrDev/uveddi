import React, { useMemo, useCallback, useState, useRef, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  TextField,
  InputAdornment,
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
  FormControl,
  FormLabel,
  Select,
  MenuItem,
  Checkbox,
  FormControlLabel,
  FormGroup,
  Slider,
  Autocomplete,
  Badge,
  Avatar,
  Divider,
  useTheme,
  Tabs,
  Tab,
  Switch,
} from '@mui/material';
import {
  SearchOutlined,
  FilterListOutlined,
  ClearOutlined,
  TuneOutlined,
  BookmarkOutlined,
  BookmarkBorderOutlined,
  HistoryOutlined,
  TrendingUpOutlined,
  ErrorOutlined,
  WarningOutlined,
  InfoOutlined,
  BugReportOutlined,
  SecurityOutlined,
  SpeedOutlined,
  BuildOutlined,
  ExpandMoreOutlined,
  ExpandLessOutlined,
  SaveOutlined,
  SmartToyOutlined,
  LocalOfferOutlined,
  DateRangeOutlined,
  FolderOutlined,
} from '@mui/icons-material';
import Fuse from 'fuse.js';
import { format, subDays, parseISO } from 'date-fns';

import type { Finding } from '../../types/api';
import type {
  SearchQuery,
  SearchFilters,
  SearchSort,
  SearchResult,
  ResponsiveComponentProps,
  FilterEvent,
} from '../../types/dashboard';

import { dataTransformers } from '../../utils/dataTransformers';
import { dashboardTheme } from '../../utils/dashboardTheme';

interface SavedSearch {
  id: string;
  name: string;
  query: SearchQuery;
  createdAt: string;
  isDefault?: boolean;
}

interface SmartSearchPanelProps extends ResponsiveComponentProps {
  findings: Finding[];
  onResultsChange?: (results: SearchResult[]) => void;
  onFilterChange?: (event: FilterEvent) => void;
  enableSemanticSearch?: boolean;
  enableSavedSearches?: boolean;
  maxResults?: number;
  placeholder?: string;
  height?: number;
}

// Mock semantic search function (in real implementation, this would call an AI service)
const performSemanticSearch = async (query: string, findings: Finding[]): Promise<SearchResult[]> => {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 500));
  
  // Simple semantic matching based on keywords
  const semanticKeywords = query.toLowerCase().split(' ');
  const results: SearchResult[] = [];
  
  findings.forEach(finding => {
    let relevanceScore = 0;
    const matchedFields: string[] = [];
    const highlights: Record<string, string> = {};
    
    // Semantic matching logic (simplified)
    const content = `${finding.title} ${finding.message} ${finding.tags.join(' ')}`.toLowerCase();
    
    semanticKeywords.forEach(keyword => {
      if (content.includes(keyword)) {
        relevanceScore += 10;
      }
      
      // Semantic similarity (simplified)
      if (keyword === 'bug' && (content.includes('error') || content.includes('issue'))) {
        relevanceScore += 5;
      }
      if (keyword === 'performance' && content.includes('slow')) {
        relevanceScore += 5;
      }
      if (keyword === 'security' && (content.includes('vulnerability') || content.includes('safe'))) {
        relevanceScore += 5;
      }
    });
    
    if (relevanceScore > 0) {
      if (finding.title.toLowerCase().includes(query.toLowerCase())) {
        matchedFields.push('title');
        highlights.title = finding.title;
      }
      
      results.push({
        finding,
        relevanceScore,
        matchedFields,
        highlights,
      });
    }
  });
  
  return results.sort((a, b) => b.relevanceScore - a.relevanceScore);
};

const SmartSearchPanel: React.FC<SmartSearchPanelProps> = ({
  findings,
  onResultsChange,
  onFilterChange,
  enableSemanticSearch = true,
  enableSavedSearches = true,
  maxResults = 100,
  placeholder = 'Search findings...',
  height = 600,
  loading = false,
  error,
  className,
  testId,
}) => {
  const theme = useTheme();
  const searchInputRef = useRef<HTMLInputElement>(null);
  
  // Search state
  const [searchText, setSearchText] = useState('');
  const [filters, setFilters] = useState<SearchFilters>({
    severity: [],
    categories: [],
    files: [],
    confidence: { min: 0, max: 100 },
    hasAiInsights: false,
    hasFixSuggestions: false,
    tags: [],
  });
  const [sortConfig, setSortConfig] = useState<SearchSort>({
    field: 'relevance',
    direction: 'desc',
  });
  const [useSemanticSearch, setUseSemanticSearch] = useState(false);
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(false);
  const [activeTab, setActiveTab] = useState(0);
  const [isSearching, setIsSearching] = useState(false);
  
  // Saved searches
  const [savedSearches, setSavedSearches] = useState<SavedSearch[]>([
    {
      id: '1',
      name: 'Critical Security Issues',
      query: {
        text: 'security',
        filters: { severity: ['critical'], categories: ['security'] },
        sort: { field: 'severity', direction: 'desc' },
        semantic: false,
      },
      createdAt: new Date().toISOString(),
      isDefault: true,
    },
    {
      id: '2', 
      name: 'Performance Problems',
      query: {
        text: 'performance slow',
        filters: { categories: ['performance'] },
        sort: { field: 'confidence', direction: 'desc' },
        semantic: true,
      },
      createdAt: new Date().toISOString(),
    },
  ]);
  const [searchHistory, setSearchHistory] = useState<string[]>([]);

  // Available filter options derived from findings
  const availableOptions = useMemo(() => {
    const severities = [...new Set(findings.map(f => f.severity))];
    const categories = [...new Set(findings.map(f => f.detector))];
    const files = [...new Set(findings.map(f => f.file))];
    const tags = [...new Set(findings.flatMap(f => f.tags))];
    
    return { severities, categories, files, tags };
  }, [findings]);

  // Fuse.js search instance for fuzzy search
  const fuseInstance = useMemo(() => {
    return new Fuse(findings, {
      keys: [
        { name: 'title', weight: 0.4 },
        { name: 'message', weight: 0.3 },
        { name: 'file', weight: 0.1 },
        { name: 'tags', weight: 0.2 },
      ],
      threshold: 0.4,
      includeScore: true,
      includeMatches: true,
    });
  }, [findings]);

  // Perform search
  const searchResults = useMemo(async () => {
    if (!searchText.trim() && Object.values(filters).every(v => 
      Array.isArray(v) ? v.length === 0 : 
      typeof v === 'object' && v !== null ? (v as any).min === 0 && (v as any).max === 100 :
      !v
    )) {
      return findings.map(finding => ({
        finding,
        relevanceScore: 1,
        matchedFields: [],
        highlights: {},
      })) as SearchResult[];
    }

    let results: SearchResult[];

    if (useSemanticSearch && enableSemanticSearch && searchText.trim()) {
      setIsSearching(true);
      try {
        results = await performSemanticSearch(searchText, findings);
      } catch (err) {
        console.error('Semantic search failed:', err);
        results = dataTransformers.search.searchFindings(findings, searchText, filters);
      } finally {
        setIsSearching(false);
      }
    } else {
      if (searchText.trim()) {
        // Use Fuse.js for fuzzy search
        const fuseResults = fuseInstance.search(searchText);
        results = fuseResults.map(result => ({
          finding: result.item,
          relevanceScore: (1 - (result.score || 0)) * 100,
          matchedFields: result.matches?.map(m => m.key) || [],
          highlights: result.matches?.reduce((acc, match) => {
            if (match.key && match.value) {
              acc[match.key] = match.value;
            }
            return acc;
          }, {} as Record<string, string>) || {},
        }));
      } else {
        results = findings.map(finding => ({
          finding,
          relevanceScore: 1,
          matchedFields: [],
          highlights: {},
        }));
      }
    }

    // Apply filters
    results = results.filter(result => {
      const { finding } = result;

      // Severity filter
      if (filters.severity && filters.severity.length > 0) {
        if (!filters.severity.includes(finding.severity)) return false;
      }

      // Category filter
      if (filters.categories && filters.categories.length > 0) {
        if (!filters.categories.includes(finding.detector)) return false;
      }

      // File filter
      if (filters.files && filters.files.length > 0) {
        if (!filters.files.some(f => finding.file.includes(f))) return false;
      }

      // Confidence filter
      if (filters.confidence) {
        if (finding.confidence < filters.confidence.min || finding.confidence > filters.confidence.max) {
          return false;
        }
      }

      // AI insights filter
      if (filters.hasAiInsights && !finding.aiInsight) {
        return false;
      }

      // Fix suggestions filter (simplified check)
      if (filters.hasFixSuggestions && !finding.recommendation) {
        return false;
      }

      // Tags filter
      if (filters.tags && filters.tags.length > 0) {
        if (!filters.tags.some(tag => finding.tags.includes(tag))) return false;
      }

      return true;
    });

    // Apply sorting
    results.sort((a, b) => {
      let comparison = 0;
      
      switch (sortConfig.field) {
        case 'relevance':
          comparison = b.relevanceScore - a.relevanceScore;
          break;
        case 'severity':
          const severityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
          comparison = severityOrder[a.finding.severity] - severityOrder[b.finding.severity];
          break;
        case 'confidence':
          comparison = b.finding.confidence - a.finding.confidence;
          break;
        case 'file':
          comparison = a.finding.file.localeCompare(b.finding.file);
          break;
        default:
          comparison = 0;
      }

      return sortConfig.direction === 'asc' ? comparison : -comparison;
    });

    // Limit results
    return results.slice(0, maxResults);
  }, [searchText, filters, sortConfig, useSemanticSearch, enableSemanticSearch, findings, fuseInstance, maxResults]);

  // Update parent component with results
  useEffect(() => {
    const runSearch = async () => {
      const results = await searchResults;
      if (onResultsChange) {
        onResultsChange(results);
      }
    };
    runSearch();
  }, [searchResults, onResultsChange]);

  // Handle search text change
  const handleSearchTextChange = useCallback((value: string) => {
    setSearchText(value);
    
    // Add to search history if not empty and not already present
    if (value.trim() && !searchHistory.includes(value)) {
      setSearchHistory(prev => [value, ...prev.slice(0, 9)]); // Keep last 10 searches
    }
  }, [searchHistory]);

  // Handle filter change
  const handleFilterChange = useCallback((key: keyof SearchFilters, value: any) => {
    const newFilters = { ...filters, [key]: value };
    setFilters(newFilters);
    
    if (onFilterChange) {
      onFilterChange({
        type: 'filters_changed',
        payload: newFilters,
        timestamp: new Date().toISOString(),
        source: 'smart_search',
      });
    }
  }, [filters, onFilterChange]);

  // Clear all filters
  const handleClearFilters = useCallback(() => {
    const clearedFilters = {
      severity: [],
      categories: [],
      files: [],
      confidence: { min: 0, max: 100 },
      hasAiInsights: false,
      hasFixSuggestions: false,
      tags: [],
    };
    setFilters(clearedFilters);
    setSearchText('');
  }, []);

  // Save current search
  const handleSaveSearch = useCallback(() => {
    const name = prompt('Enter a name for this search:');
    if (name) {
      const newSavedSearch: SavedSearch = {
        id: Date.now().toString(),
        name,
        query: {
          text: searchText,
          filters,
          sort: sortConfig,
          semantic: useSemanticSearch,
        },
        createdAt: new Date().toISOString(),
      };
      setSavedSearches(prev => [newSavedSearch, ...prev]);
    }
  }, [searchText, filters, sortConfig, useSemanticSearch]);

  // Load saved search
  const handleLoadSavedSearch = useCallback((savedSearch: SavedSearch) => {
    setSearchText(savedSearch.query.text);
    setFilters(savedSearch.query.filters);
    setSortConfig(savedSearch.query.sort);
    setUseSemanticSearch(savedSearch.query.semantic || false);
    setActiveTab(0); // Switch to search tab
  }, []);

  // Get severity icon and color
  const getSeverityInfo = useCallback((severity: Finding['severity']) => {
    switch (severity) {
      case 'critical':
        return { icon: <ErrorOutlined />, color: 'error' };
      case 'high':
        return { icon: <WarningOutlined />, color: 'warning' };
      case 'medium':
        return { icon: <InfoOutlined />, color: 'info' };
      case 'low':
        return { icon: <InfoOutlined />, color: 'success' };
      default:
        return { icon: <InfoOutlined />, color: 'default' };
    }
  }, []);

  // Get category icon
  const getCategoryIcon = useCallback((category: string) => {
    const iconMap: Record<string, React.ReactNode> = {
      'god_object': <BuildOutlined />,
      'circular_dependency': <TrendingUpOutlined />,
      'tight_coupling': <BuildOutlined />,
      'magic_values': <SpeedOutlined />,
      'dead_code': <BugReportOutlined />,
      'security': <SecurityOutlined />,
      'performance': <SpeedOutlined />,
    };
    return iconMap[category] || <BugReportOutlined />;
  }, []);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading search panel...
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
                Error loading search: {error}
              </Typography>
            </Alert>
          </Box>
        </CardContent>
      </Card>
    );
  }

  const activeFiltersCount = [
    filters.severity?.length || 0,
    filters.categories?.length || 0,
    filters.files?.length || 0,
    filters.tags?.length || 0,
    filters.hasAiInsights ? 1 : 0,
    filters.hasFixSuggestions ? 1 : 0,
    filters.confidence?.min !== 0 || filters.confidence?.max !== 100 ? 1 : 0,
  ].reduce((sum, count) => sum + count, 0);

  return (
    <Box className={className} data-testid={testId}>
      {/* Header */}
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box>
          <Typography variant="h5" component="h2" fontWeight="600" color="text.primary">
            Smart Search
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Advanced search with AI-powered semantic matching
          </Typography>
        </Box>
      </Box>

      <Card>
        {/* Search input */}
        <CardContent sx={{ pb: 1 }}>
          <TextField
            ref={searchInputRef}
            fullWidth
            value={searchText}
            onChange={(e) => handleSearchTextChange(e.target.value)}
            placeholder={placeholder}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  {isSearching ? (
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <SearchOutlined />
                      <SmartToyOutlined color="primary" />
                    </Box>
                  ) : (
                    <SearchOutlined />
                  )}
                </InputAdornment>
              ),
              endAdornment: (
                <InputAdornment position="end">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    {enableSemanticSearch && (
                      <FormControlLabel
                        control={
                          <Switch
                            size="small"
                            checked={useSemanticSearch}
                            onChange={(e) => setUseSemanticSearch(e.target.checked)}
                          />
                        }
                        label="AI"
                        componentsProps={{ 
                          typography: { variant: 'caption', color: 'text.secondary' }
                        }}
                        sx={{ mr: 1 }}
                      />
                    )}
                    
                    <Badge badgeContent={activeFiltersCount} color="primary">
                      <IconButton
                        size="small"
                        onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
                        color={showAdvancedFilters ? 'primary' : 'default'}
                      >
                        <FilterListOutlined />
                      </IconButton>
                    </Badge>
                    
                    {(searchText || activeFiltersCount > 0) && (
                      <IconButton size="small" onClick={handleClearFilters}>
                        <ClearOutlined />
                      </IconButton>
                    )}
                  </Box>
                </InputAdornment>
              ),
            }}
            sx={{
              '& .MuiOutlinedInput-root': {
                borderRadius: 2,
              },
            }}
          />

          {/* Quick filters */}
          <Box display="flex" alignItems="center" gap={1} mt={1} flexWrap="wrap">
            {['critical', 'high'].map(severity => (
              <Chip
                key={severity}
                label={severity}
                size="small"
                clickable
                color={filters.severity?.includes(severity as Finding['severity']) ? 'primary' : 'default'}
                onClick={() => {
                  const newSeverities = filters.severity?.includes(severity as Finding['severity'])
                    ? filters.severity.filter(s => s !== severity)
                    : [...(filters.severity || []), severity as Finding['severity']];
                  handleFilterChange('severity', newSeverities);
                }}
                icon={getSeverityInfo(severity as Finding['severity']).icon}
              />
            ))}
            
            <Divider orientation="vertical" flexItem sx={{ mx: 1 }} />
            
            {['god_object', 'circular_dependency', 'security'].map(category => (
              <Chip
                key={category}
                label={category.replace('_', ' ')}
                size="small"
                clickable
                color={filters.categories?.includes(category) ? 'primary' : 'default'}
                onClick={() => {
                  const newCategories = filters.categories?.includes(category)
                    ? filters.categories.filter(c => c !== category)
                    : [...(filters.categories || []), category];
                  handleFilterChange('categories', newCategories);
                }}
                icon={getCategoryIcon(category)}
              />
            ))}

            {enableSavedSearches && (
              <>
                <Divider orientation="vertical" flexItem sx={{ mx: 1 }} />
                <Button
                  size="small"
                  startIcon={<SaveOutlined />}
                  onClick={handleSaveSearch}
                  disabled={!searchText && activeFiltersCount === 0}
                >
                  Save Search
                </Button>
              </>
            )}
          </Box>
        </CardContent>

        {/* Advanced filters */}
        <Collapse in={showAdvancedFilters}>
          <CardContent sx={{ pt: 0, borderTop: '1px solid', borderColor: 'divider' }}>
            <Grid container spacing={2}>
              {/* Severity filter */}
              <Grid item xs={12} sm={6} md={3}>
                <FormControl fullWidth size="small">
                  <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500 }}>
                    Severity
                  </FormLabel>
                  <FormGroup row>
                    {availableOptions.severities.map(severity => (
                      <FormControlLabel
                        key={severity}
                        control={
                          <Checkbox
                            size="small"
                            checked={filters.severity?.includes(severity) || false}
                            onChange={(e) => {
                              const newSeverities = e.target.checked
                                ? [...(filters.severity || []), severity]
                                : filters.severity?.filter(s => s !== severity) || [];
                              handleFilterChange('severity', newSeverities);
                            }}
                          />
                        }
                        label={severity}
                        componentsProps={{ 
                          typography: { variant: 'body2' }
                        }}
                      />
                    ))}
                  </FormGroup>
                </FormControl>
              </Grid>

              {/* Categories filter */}
              <Grid item xs={12} sm={6} md={3}>
                <FormControl fullWidth size="small">
                  <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500 }}>
                    Categories
                  </FormLabel>
                  <Autocomplete
                    multiple
                    size="small"
                    options={availableOptions.categories}
                    value={filters.categories || []}
                    onChange={(_, value) => handleFilterChange('categories', value)}
                    renderTags={(value, getTagProps) =>
                      value.map((option, index) => (
                        <Chip
                          variant="outlined"
                          label={option.replace('_', ' ')}
                          size="small"
                          {...getTagProps({ index })}
                          key={option}
                        />
                      ))
                    }
                    renderInput={(params) => (
                      <TextField {...params} placeholder="Select categories..." />
                    )}
                    sx={{ mt: 0.5 }}
                  />
                </FormControl>
              </Grid>

              {/* Files filter */}
              <Grid item xs={12} sm={6} md={3}>
                <FormControl fullWidth size="small">
                  <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500 }}>
                    Files
                  </FormLabel>
                  <Autocomplete
                    multiple
                    size="small"
                    options={availableOptions.files}
                    value={filters.files || []}
                    onChange={(_, value) => handleFilterChange('files', value)}
                    renderTags={(value, getTagProps) =>
                      value.map((option, index) => (
                        <Chip
                          variant="outlined"
                          label={option.split('/').pop() || option}
                          size="small"
                          {...getTagProps({ index })}
                          key={option}
                        />
                      ))
                    }
                    renderInput={(params) => (
                      <TextField {...params} placeholder="Select files..." />
                    )}
                    sx={{ mt: 0.5 }}
                  />
                </FormControl>
              </Grid>

              {/* Confidence filter */}
              <Grid item xs={12} sm={6} md={3}>
                <FormControl fullWidth size="small">
                  <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500 }}>
                    Confidence Range
                  </FormLabel>
                  <Box sx={{ px: 1, mt: 1 }}>
                    <Slider
                      value={[filters.confidence?.min || 0, filters.confidence?.max || 100]}
                      onChange={(_, value) => {
                        const [min, max] = value as number[];
                        handleFilterChange('confidence', { min, max });
                      }}
                      valueLabelDisplay="auto"
                      min={0}
                      max={100}
                      marks={[
                        { value: 0, label: '0%' },
                        { value: 50, label: '50%' },
                        { value: 100, label: '100%' },
                      ]}
                      size="small"
                    />
                  </Box>
                </FormControl>
              </Grid>

              {/* Tags filter */}
              <Grid item xs={12} sm={6} md={4}>
                <FormControl fullWidth size="small">
                  <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500 }}>
                    Tags
                  </FormLabel>
                  <Autocomplete
                    multiple
                    size="small"
                    options={availableOptions.tags}
                    value={filters.tags || []}
                    onChange={(_, value) => handleFilterChange('tags', value)}
                    renderTags={(value, getTagProps) =>
                      value.map((option, index) => (
                        <Chip
                          variant="outlined"
                          label={option}
                          size="small"
                          icon={<LocalOfferOutlined />}
                          {...getTagProps({ index })}
                          key={option}
                        />
                      ))
                    }
                    renderInput={(params) => (
                      <TextField {...params} placeholder="Select tags..." />
                    )}
                    sx={{ mt: 0.5 }}
                  />
                </FormControl>
              </Grid>

              {/* Additional filters */}
              <Grid item xs={12} sm={6} md={4}>
                <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500, display: 'block' }}>
                  Additional Filters
                </FormLabel>
                <FormGroup>
                  <FormControlLabel
                    control={
                      <Checkbox
                        size="small"
                        checked={filters.hasAiInsights || false}
                        onChange={(e) => handleFilterChange('hasAiInsights', e.target.checked)}
                      />
                    }
                    label="Has AI Insights"
                    componentsProps={{ typography: { variant: 'body2' } }}
                  />
                  <FormControlLabel
                    control={
                      <Checkbox
                        size="small"
                        checked={filters.hasFixSuggestions || false}
                        onChange={(e) => handleFilterChange('hasFixSuggestions', e.target.checked)}
                      />
                    }
                    label="Has Fix Suggestions"
                    componentsProps={{ typography: { variant: 'body2' } }}
                  />
                </FormGroup>
              </Grid>

              {/* Sort options */}
              <Grid item xs={12} sm={6} md={4}>
                <FormControl fullWidth size="small">
                  <FormLabel sx={{ mb: 1, fontSize: '0.875rem', fontWeight: 500 }}>
                    Sort By
                  </FormLabel>
                  <Box display="flex" gap={1} mt={0.5}>
                    <Select
                      value={sortConfig.field}
                      onChange={(e) => setSortConfig(prev => ({ ...prev, field: e.target.value as any }))}
                      size="small"
                      sx={{ flex: 2 }}
                    >
                      <MenuItem value="relevance">Relevance</MenuItem>
                      <MenuItem value="severity">Severity</MenuItem>
                      <MenuItem value="confidence">Confidence</MenuItem>
                      <MenuItem value="file">File</MenuItem>
                    </Select>
                    <Select
                      value={sortConfig.direction}
                      onChange={(e) => setSortConfig(prev => ({ ...prev, direction: e.target.value as 'asc' | 'desc' }))}
                      size="small"
                      sx={{ flex: 1 }}
                    >
                      <MenuItem value="desc">↓</MenuItem>
                      <MenuItem value="asc">↑</MenuItem>
                    </Select>
                  </Box>
                </FormControl>
              </Grid>
            </Grid>
          </CardContent>
        </Collapse>

        {/* Tabs for search history and saved searches */}
        {enableSavedSearches && (
          <>
            <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
              <Tabs value={activeTab} onChange={(_, newValue) => setActiveTab(newValue)} variant="fullWidth">
                <Tab label="Current Search" />
                <Tab 
                  label={
                    <Box display="flex" alignItems="center" gap={0.5}>
                      <BookmarkOutlined fontSize="small" />
                      Saved ({savedSearches.length})
                    </Box>
                  }
                />
                <Tab 
                  label={
                    <Box display="flex" alignItems="center" gap={0.5}>
                      <HistoryOutlined fontSize="small" />
                      History ({searchHistory.length})
                    </Box>
                  }
                />
              </Tabs>
            </Box>

            {/* Saved searches tab */}
            {activeTab === 1 && (
              <CardContent sx={{ maxHeight: 300, overflow: 'auto' }}>
                {savedSearches.length === 0 ? (
                  <Box display="flex" flexDirection="column" alignItems="center" py={3}>
                    <BookmarkBorderOutlined sx={{ fontSize: 48, color: 'text.secondary', mb: 1 }} />
                    <Typography variant="body2" color="text.secondary">
                      No saved searches yet
                    </Typography>
                  </Box>
                ) : (
                  <List dense>
                    {savedSearches.map(savedSearch => (
                      <ListItemButton
                        key={savedSearch.id}
                        onClick={() => handleLoadSavedSearch(savedSearch)}
                        sx={{ borderRadius: 1, mb: 0.5 }}
                      >
                        <ListItemIcon>
                          <Avatar sx={{ width: 32, height: 32, bgcolor: 'primary.light' }}>
                            <BookmarkOutlined fontSize="small" />
                          </Avatar>
                        </ListItemIcon>
                        <ListItemText
                          primary={
                            <Box display="flex" alignItems="center" gap={1}>
                              <Typography variant="subtitle2" fontWeight="600">
                                {savedSearch.name}
                              </Typography>
                              {savedSearch.isDefault && (
                                <Chip label="default" size="small" color="primary" variant="outlined" />
                              )}
                            </Box>
                          }
                          secondary={
                            <Box>
                              <Typography variant="caption" color="text.secondary" display="block">
                                "{savedSearch.query.text}" • {format(parseISO(savedSearch.createdAt), 'MMM dd, yyyy')}
                              </Typography>
                              <Box display="flex" gap={0.5} mt={0.5}>
                                {savedSearch.query.filters.severity?.length > 0 && (
                                  <Chip label={`${savedSearch.query.filters.severity.length} severities`} size="small" variant="outlined" />
                                )}
                                {savedSearch.query.filters.categories?.length > 0 && (
                                  <Chip label={`${savedSearch.query.filters.categories.length} categories`} size="small" variant="outlined" />
                                )}
                                {savedSearch.query.semantic && (
                                  <Chip label="AI" size="small" color="primary" variant="outlined" />
                                )}
                              </Box>
                            </Box>
                          }
                        />
                      </ListItemButton>
                    ))}
                  </List>
                )}
              </CardContent>
            )}

            {/* Search history tab */}
            {activeTab === 2 && (
              <CardContent sx={{ maxHeight: 300, overflow: 'auto' }}>
                {searchHistory.length === 0 ? (
                  <Box display="flex" flexDirection="column" alignItems="center" py={3}>
                    <HistoryOutlined sx={{ fontSize: 48, color: 'text.secondary', mb: 1 }} />
                    <Typography variant="body2" color="text.secondary">
                      No search history yet
                    </Typography>
                  </Box>
                ) : (
                  <List dense>
                    {searchHistory.map((query, index) => (
                      <ListItemButton
                        key={index}
                        onClick={() => handleSearchTextChange(query)}
                        sx={{ borderRadius: 1, mb: 0.5 }}
                      >
                        <ListItemIcon>
                          <HistoryOutlined color="action" />
                        </ListItemIcon>
                        <ListItemText
                          primary={query}
                          secondary={`Search ${index + 1}`}
                        />
                      </ListItemButton>
                    ))}
                  </List>
                )}
              </CardContent>
            )}
          </>
        )}
      </Card>

      {/* Search tips */}
      {searchText.length === 0 && activeFiltersCount === 0 && (
        <Alert severity="info" sx={{ mt: 2 }}>
          <Typography variant="body2" fontWeight="600" mb={0.5}>
            Search Tips
          </Typography>
          <Typography variant="body2">
            • Use natural language for semantic search with AI enabled<br />
            • Search by severity, category, file, or content<br />
            • Combine filters to narrow down results<br />
            • Save frequently used searches for quick access
          </Typography>
        </Alert>
      )}

      {/* Search results summary (if needed) */}
      {(searchText || activeFiltersCount > 0) && (
        <Box mt={2}>
          <Paper sx={{ p: 2, bgcolor: 'action.hover' }}>
            <Typography variant="body2" color="text.secondary">
              {isSearching ? (
                <Box display="flex" alignItems="center" gap={1}>
                  <SmartToyOutlined color="primary" />
                  Searching with AI...
                </Box>
              ) : (
                `Search active with ${activeFiltersCount} filters applied`
              )}
            </Typography>
          </Paper>
        </Box>
      )}
    </Box>
  );
};

export default SmartSearchPanel;