import React, { useEffect, useRef, useState } from 'react';
import { Paper, Typography, Alert } from '@mui/material';
import mermaid from 'mermaid';

export default function SimpleMermaidTest() {
  const elementRef = useRef<HTMLDivElement | null>(null);
  const [status, setStatus] = useState<string>('Starting...');

  useEffect(() => {
    const runTest = async () => {
      try {
        setStatus('Initializing mermaid...');
        
        // Initialize mermaid
        mermaid.initialize({
          startOnLoad: false,
          theme: 'default',
          securityLevel: 'loose'
        });
        
        setStatus('Creating simple diagram...');
        
        const simpleDiagram = `graph TD
    A[Start] --> B[Process]
    B --> C[End]`;

        console.log('SimpleMermaidTest: About to render diagram');
        
        const { svg } = await mermaid.render('simple-test', simpleDiagram);
        
        console.log('SimpleMermaidTest: Got SVG:', svg ? 'SUCCESS' : 'FAILED');
        
        if (elementRef.current && svg) {
          elementRef.current.innerHTML = svg;
          setStatus('SUCCESS: Diagram rendered!');
        } else {
          setStatus('FAILED: No SVG or no element');
        }
        
      } catch (error) {
        console.error('SimpleMermaidTest error:', error);
        setStatus(`ERROR: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    };

    runTest();
  }, []);

  return (
    <Paper sx={{ p: 3, mb: 2 }}>
      <Typography variant="h6" gutterBottom>
        Simple Mermaid Test
      </Typography>
      <Alert severity="info" sx={{ mb: 2 }}>
        Status: {status}
      </Alert>
      <div 
        ref={elementRef}
        style={{ 
          border: '1px solid #ccc', 
          padding: '10px', 
          minHeight: '100px',
          textAlign: 'center'
        }}
      >
        Waiting for diagram...
      </div>
    </Paper>
  );
}