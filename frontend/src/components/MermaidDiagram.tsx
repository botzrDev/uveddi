import React, { useEffect, useRef, useState } from 'react';
import { Box, Paper, Typography, Alert, CircularProgress } from '@mui/material';
import mermaid from 'mermaid';

interface MermaidDiagramProps {
  definition: string;
  title?: string;
  className?: string;
}

export default function MermaidDiagram({ definition, title, className }: MermaidDiagramProps) {
  const [elementRef, setElementRef] = useState<HTMLDivElement | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [diagramId] = useState(() => `mermaid-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`);
  const [mermaidReady, setMermaidReady] = useState(false);

  // Callback ref to track when element is mounted
  const refCallback = (element: HTMLDivElement | null) => {
    console.log('MermaidDiagram: refCallback called with element:', element ? 'YES' : 'NO');
    setElementRef(element);
  };

  // Initialize mermaid for this component
  useEffect(() => {
    console.log('MermaidDiagram: Initializing mermaid for component');
    try {
      mermaid.initialize({
        startOnLoad: false,
        theme: 'default',
        themeVariables: {
          primaryColor: '#1976d2',
          primaryTextColor: '#000',
          primaryBorderColor: '#1976d2',
          lineColor: '#666',
          sectionBkgColor: '#f5f5f5',
          altSectionBkgColor: '#fff',
          gridColor: '#e0e0e0',
          secondaryColor: '#006100',
          tertiaryColor: '#fff'
        },
        fontFamily: '"Roboto", "Helvetica", "Arial", sans-serif',
        fontSize: 14,
        flowchart: {
          useMaxWidth: true,
          htmlLabels: true,
          curve: 'basis'
        },
        class: {
          useMaxWidth: true,
        },
        gitGraph: {
          useMaxWidth: true,
        },
        securityLevel: 'loose'
      });
      setMermaidReady(true);
      console.log('MermaidDiagram: Mermaid initialized successfully');
    } catch (initError) {
      console.error('Mermaid initialization failed:', initError);
      setError('Failed to initialize Mermaid');
      setLoading(false);
    }
  }, []);

  // Render diagram when definition changes and element is available
  useEffect(() => {
    console.log('MermaidDiagram: useEffect triggered with definition:', definition ? 'YES' : 'NO');
    console.log('MermaidDiagram: mermaidReady:', mermaidReady);
    console.log('MermaidDiagram: elementRef:', elementRef ? 'YES' : 'NO');
    
    if (!definition || !definition.trim()) {
      console.log('MermaidDiagram: Empty definition provided');
      setError('No diagram definition provided');
      setLoading(false);
      return;
    }

    // Don't render if mermaid isn't ready yet or element isn't available
    if (!mermaidReady || !elementRef) {
      console.log('MermaidDiagram: Waiting for mermaid and element to be ready...');
      return;
    }

    let isCancelled = false;

    const renderDiagram = async () => {
      if (!elementRef || isCancelled) {
        console.log('MermaidDiagram: No element ref or cancelled');
        return;
      }

      console.log('MermaidDiagram: Starting render process for ID:', diagramId);
      console.log('MermaidDiagram: Full definition:', definition);
      
      setLoading(true);
      setError(null);

      try {
        // Clear previous content
        elementRef.innerHTML = '';

        // Clean up the definition - handle escaped newlines
        const cleanDefinition = definition.replace(/\\n/g, '\n').trim();
        console.log('MermaidDiagram: Cleaned definition:', cleanDefinition);

        // Validate the definition first
        try {
          console.log('MermaidDiagram: Parsing definition...');
          await mermaid.parse(cleanDefinition);
          console.log('MermaidDiagram: Parse successful');
        } catch (parseError) {
          console.error('MermaidDiagram: Parse error:', parseError);
          const errorMessage = parseError instanceof Error ? parseError.message : 'Unknown syntax error';
          throw new Error(`Mermaid syntax error: ${errorMessage}`);
        }

        // Render the diagram
        console.log('MermaidDiagram: Rendering with ID:', diagramId);
        const { svg } = await mermaid.render(diagramId, cleanDefinition);
        console.log('MermaidDiagram: Render result SVG length:', svg ? svg.length : 'NULL');
        
        if (isCancelled || !elementRef) {
          console.log('MermaidDiagram: Cancelled or no element during render');
          return;
        }
        
        if (!svg || svg.trim().length === 0) {
          throw new Error('Mermaid rendered empty SVG');
        }
        
        elementRef.innerHTML = svg;
        
        // Make diagram responsive
        const svgElement = elementRef.querySelector('svg');
        if (svgElement) {
          svgElement.setAttribute('width', '100%');
          svgElement.setAttribute('height', 'auto');
          svgElement.style.maxWidth = '100%';
          svgElement.style.height = 'auto';
          svgElement.style.display = 'block';
          svgElement.style.margin = '0 auto';
        }
        console.log('MermaidDiagram: Successfully rendered diagram:', diagramId);
        setLoading(false);
      } catch (err) {
        if (!isCancelled) {
          console.error('MermaidDiagram: Rendering error for', diagramId, ':', err);
          console.error('MermaidDiagram: Failed diagram definition:', definition);
          setError(err instanceof Error ? err.message : 'Failed to render diagram');
          setLoading(false);
        }
      }
    };

    // Use requestAnimationFrame to ensure DOM is ready, then render
    requestAnimationFrame(() => {
      if (!isCancelled) {
        console.log('MermaidDiagram: requestAnimationFrame triggered, calling renderDiagram');
        renderDiagram();
      }
    });

    return () => {
      isCancelled = true;
    };
  }, [definition, diagramId, mermaidReady, elementRef]);

  return (
    <Paper sx={{ p: 2, overflow: 'auto' }} className={className}>
      {title && (
        <Typography variant="h6" gutterBottom sx={{ mb: 2 }}>
          {title}
        </Typography>
      )}
      
      {loading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', gap: 2, mb: 2 }}>
          <CircularProgress size={24} />
          <Typography variant="body2" color="text.secondary">
            Rendering diagram...
          </Typography>
        </Box>
      )}
      
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          <Typography variant="body2">
            Failed to render diagram: {error}
          </Typography>
        </Alert>
      )}
      
      <Box
        ref={refCallback}
        sx={{
          textAlign: 'center',
          minHeight: loading ? '200px' : 'auto',
          '& svg': {
            maxWidth: '100%',
            height: 'auto',
            display: loading ? 'none' : 'block',
          },
        }}
      />
    </Paper>
  );
}

