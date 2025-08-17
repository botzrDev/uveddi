import React, { useEffect, useRef } from 'react';
import { Paper, Typography } from '@mui/material';
import mermaid from 'mermaid';

export default function TestMermaid() {
  const elementRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const renderTest = async () => {
      try {
        // Initialize mermaid
        mermaid.initialize({
          startOnLoad: false,
          theme: 'default',
          securityLevel: 'loose'
        });

        const testDiagram = `graph TD
    A[Start] --> B[Process]
    B --> C[End]
    
    style A fill:#f9f,stroke:#333,stroke-width:2px
    style C fill:#bbf,stroke:#333,stroke-width:2px`;

        console.log('TestMermaid: Rendering test diagram');
        
        if (elementRef.current) {
          const { svg } = await mermaid.render('test-diagram', testDiagram);
          elementRef.current.innerHTML = svg;
          console.log('TestMermaid: Success!');
        }
      } catch (error) {
        console.error('TestMermaid: Error:', error);
        if (elementRef.current) {
          elementRef.current.innerHTML = `<p style="color: red;">Error: ${error}</p>`;
        }
      }
    };

    renderTest();
  }, []);

  return (
    <Paper sx={{ p: 3, m: 2 }}>
      <Typography variant="h6" gutterBottom>
        Test Mermaid Rendering
      </Typography>
      <div ref={elementRef} style={{ textAlign: 'center' }}>
        Loading test diagram...
      </div>
    </Paper>
  );
}