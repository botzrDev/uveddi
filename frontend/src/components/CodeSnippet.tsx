import {
  ContentCopy,
  Visibility,
  VisibilityOff,
} from '@mui/icons-material';
import {
  Alert,
  Box,
  IconButton,
  Paper,
  Snackbar,
  Tooltip,
  Typography,
  useTheme,
} from '@mui/material';
import { useState } from 'react';

interface CodeSnippetProps {
  code: string;
  language?: string;
  title?: string;
  startLine?: number;
  endLine?: number;
  highlightLines?: number[];
  maxHeight?: number;
  showLineNumbers?: boolean;
}

export default function CodeSnippet({
  code,
  language = 'text',
  title,
  startLine = 1,
  endLine,
  highlightLines = [],
  maxHeight = 400,
  showLineNumbers = true,
}: CodeSnippetProps) {
  const theme = useTheme();
  const [expanded, setExpanded] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  };

  const closeCopiedSnackbar = () => {
    setCopied(false);
  };

  // Split code into lines
  const lines = code.split('\n');
  const displayedEndLine = endLine || (startLine + lines.length - 1);

  // Get language-specific styling
  const getLanguageColor = (lang: string) => {
    switch (lang.toLowerCase()) {
      case 'rust':
        return '#dea584';
      case 'python':
        return '#3776ab';
      case 'javascript':
      case 'typescript':
        return '#f7df1e';
      case 'java':
        return '#f89820';
      default:
        return '#6c757d';
    }
  };

  // Simple syntax highlighting for common patterns
  const highlightSyntax = (line: string, language: string): string => {
    let highlighted = line;
    
    // Escape HTML first
    highlighted = highlighted
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    switch (language.toLowerCase()) {
      case 'rust':
        // Keywords
        highlighted = highlighted.replace(
          /\b(fn|let|mut|if|else|match|struct|impl|pub|use|mod|const|static|trait|enum|where|for|while|loop|break|continue|return|self|Self)\b/g,
          '<span style="color: #569cd6;">$1</span>'
        );
        // Strings
        highlighted = highlighted.replace(
          /"([^"\\]|\\.)*"/g,
          '<span style="color: #ce9178;">$&</span>'
        );
        // Comments
        highlighted = highlighted.replace(
          /\/\/.*$/g,
          '<span style="color: #6a9955;">$&</span>'
        );
        break;
        
      case 'python':
        // Keywords
        highlighted = highlighted.replace(
          /\b(def|class|if|else|elif|for|while|try|except|finally|import|from|as|return|yield|lambda|pass|break|continue|global|nonlocal|and|or|not|in|is|True|False|None)\b/g,
          '<span style="color: #569cd6;">$1</span>'
        );
        // Strings
        highlighted = highlighted.replace(
          /(['"])((?:(?!\1)[^\\]|\\.)*)(\1)/g,
          '<span style="color: #ce9178;">$&</span>'
        );
        // Comments
        highlighted = highlighted.replace(
          /#.*$/g,
          '<span style="color: #6a9955;">$&</span>'
        );
        break;
        
      case 'javascript':
      case 'typescript':
        // Keywords
        highlighted = highlighted.replace(
          /\b(function|const|let|var|if|else|for|while|do|switch|case|default|try|catch|finally|return|break|continue|class|extends|interface|type|async|await|import|export|from|as)\b/g,
          '<span style="color: #569cd6;">$1</span>'
        );
        // Strings
        highlighted = highlighted.replace(
          /(['"`])((?:(?!\1)[^\\]|\\.)*)(\1)/g,
          '<span style="color: #ce9178;">$&</span>'
        );
        // Comments
        highlighted = highlighted.replace(
          /\/\/.*$/g,
          '<span style="color: #6a9955;">$&</span>'
        );
        break;
    }
    
    return highlighted;
  };

  const containerHeight = expanded ? 'auto' : maxHeight;

  return (
    <Box>
      <Paper 
        sx={{ 
          border: 1, 
          borderColor: 'divider',
          borderRadius: 1,
          overflow: 'hidden',
        }}
      >
        {/* Header */}
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            p: 1,
            bgcolor: (theme) => theme.palette.mode === 'dark' 
              ? theme.palette.grey[800] 
              : theme.palette.grey[50],
            borderBottom: 1,
            borderColor: 'divider',
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {language && (
              <Box
                sx={{
                  px: 1,
                  py: 0.5,
                  bgcolor: getLanguageColor(language),
                  color: 'white',
                  borderRadius: 0.5,
                  fontSize: '0.75rem',
                  fontWeight: 'bold',
                  textTransform: 'uppercase',
                }}
              >
                {language}
              </Box>
            )}
            {title && (
              <Typography variant="body2" fontWeight="medium">
                {title}
              </Typography>
            )}
            <Typography variant="caption" color="text.secondary">
              Lines {startLine}-{displayedEndLine}
            </Typography>
          </Box>
          
          <Box>
            {lines.length > 10 && (
              <Tooltip title={expanded ? "Collapse" : "Expand"}>
                <IconButton
                  size="small"
                  onClick={() => setExpanded(!expanded)}
                >
                  {expanded ? <VisibilityOff /> : <Visibility />}
                </IconButton>
              </Tooltip>
            )}
            <Tooltip title="Copy to clipboard">
              <IconButton size="small" onClick={handleCopy}>
                <ContentCopy />
              </IconButton>
            </Tooltip>
          </Box>
        </Box>

        {/* Code Content */}
        <Box
          sx={{
            maxHeight: containerHeight,
            overflow: 'auto',
            bgcolor: (theme) => theme.palette.mode === 'dark' 
              ? '#1a1a1a' 
              : '#fafafa',
            color: (theme) => theme.palette.mode === 'dark' 
              ? '#d4d4d4' 
              : '#333333',
            fontFamily: '"Fira Code", "Consolas", "Monaco", monospace',
            fontSize: '0.875rem',
            lineHeight: 1.5,
          }}
        >
          <Box component="pre" sx={{ m: 0, p: 2 }}>
            {lines.map((line, index) => {
              const lineNumber = startLine + index;
              const isHighlighted = highlightLines.includes(lineNumber);
              
              return (
                <Box
                  key={lineNumber}
                  sx={{
                    display: 'flex',
                    bgcolor: isHighlighted ? 'rgba(255, 255, 0, 0.1)' : 'transparent',
                    borderLeft: isHighlighted ? '3px solid #ffd93d' : 'none',
                    pl: isHighlighted ? 1 : 0,
                    '&:hover': {
                      bgcolor: (theme) => theme.palette.mode === 'dark' 
                        ? 'rgba(255, 255, 255, 0.05)' 
                        : 'rgba(0, 0, 0, 0.05)',
                    },
                  }}
                >
                  {showLineNumbers && (
                    <Box
                      component="span"
                      sx={{
                        display: 'inline-block',
                        width: '3rem',
                        textAlign: 'right',
                        mr: 2,
                        color: (theme) => theme.palette.mode === 'dark' 
                          ? '#858585' 
                          : '#666666',
                        flexShrink: 0,
                        userSelect: 'none',
                      }}
                    >
                      {lineNumber}
                    </Box>
                  )}
                  <Box
                    component="span"
                    sx={{ 
                      flex: 1,
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-word',
                    }}
                    dangerouslySetInnerHTML={{
                      __html: highlightSyntax(line, language),
                    }}
                  />
                </Box>
              );
            })}
          </Box>
        </Box>

        {/* Show truncation indicator */}
        {!expanded && lines.length > 10 && (
          <Box
            sx={{
              p: 1,
              textAlign: 'center',
              bgcolor: (theme) => theme.palette.mode === 'dark' 
                ? theme.palette.grey[800] 
                : theme.palette.grey[50],
              borderTop: 1,
              borderColor: 'divider',
            }}
          >
            <Typography variant="caption" color="text.secondary">
              Code truncated. Click expand to view all {lines.length} lines.
            </Typography>
          </Box>
        )}
      </Paper>

      {/* Copy notification */}
      <Snackbar
        open={copied}
        autoHideDuration={2000}
        onClose={closeCopiedSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={closeCopiedSnackbar}
          severity="success"
          sx={{ width: '100%' }}
        >
          Code copied to clipboard!
        </Alert>
      </Snackbar>
    </Box>
  );
}