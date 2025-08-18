import { Alert, Box, Button, Paper, Typography } from '@mui/material';
import mermaid from 'mermaid';
import { useEffect, useState } from 'react';

const TEST_DIAGRAM = `flowchart TD
    A[Start Process] --> B{Decision Point}
    B -->|Option A| C[Process A]
    B -->|Option B| D[Process B]
    B -->|Option C| E[Process C]
    
    C --> F[Validation Step A]
    D --> G[Validation Step B]
    E --> H[Validation Step C]
    
    F --> I[Final Processing]
    G --> I
    H --> I
    
    I --> J[Generate Output]
    J --> K[Quality Check]
    K -->|Pass| L[Success]
    K -->|Fail| M[Error Handling]
    M --> N[Retry Logic]
    N --> B
    
    L --> End([Complete])
    
    style A fill:#e1f5fe,stroke:#01579b,stroke-width:3px
    style End fill:#e8f5e8,stroke:#2e7d32,stroke-width:3px
    style B fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    style M fill:#ffebee,stroke:#c62828,stroke-width:2px`;

export default function MermaidRenderingTest() {
  const [method1Result, setMethod1Result] = useState<string>('');
  const [method2Result, setMethod2Result] = useState<string>('');
  const [method3Result, setMethod3Result] = useState<string>('');
  const [logs, setLogs] = useState<string[]>([]);

  const addLog = (message: string) => {
    setLogs(prev => [...prev, `${new Date().toLocaleTimeString()}: ${message}`]);
    console.log(message);
  };

  useEffect(() => {
    // Initialize mermaid without constraints
    try {
      mermaid.initialize({
        startOnLoad: false,
        theme: 'default',
        useMaxWidth: false,
        flowchart: {
          useMaxWidth: false,
          htmlLabels: true,
          padding: 20,
        },
        securityLevel: 'loose'
      });
      addLog('Mermaid initialized successfully');
    } catch (error) {
      addLog(`Mermaid initialization failed: ${error}`);
    }
  }, []);

  const testMethod1 = async () => {
    addLog('Testing Method 1: Standard render API');
    try {
      const result = await mermaid.render('test-method1', TEST_DIAGRAM);
      const svg = result.svg || result;
      setMethod1Result(svg);
      addLog(`Method 1 success: SVG length ${svg.length}`);
      
      // Analyze SVG
      const viewBoxMatch = svg.match(/viewBox="([^"]+)"/);
      const widthMatch = svg.match(/width="([^"]+)"/);
      const heightMatch = svg.match(/height="([^"]+)"/);
      addLog(`Method 1 - ViewBox: ${viewBoxMatch?.[1] || 'NONE'}, Width: ${widthMatch?.[1] || 'NONE'}, Height: ${heightMatch?.[1] || 'NONE'}`);
    } catch (error) {
      addLog(`Method 1 failed: ${error}`);
      setMethod1Result('');
    }
  };

  const testMethod2 = async () => {
    addLog('Testing Method 2: Large container rendering');
    try {
      // Create large container
      const container = document.createElement('div');
      container.style.position = 'absolute';
      container.style.top = '-10000px';
      container.style.width = '5000px';
      container.style.height = '5000px';
      container.style.overflow = 'visible';
      document.body.appendChild(container);

      try {
        const result = await mermaid.render('test-method2', TEST_DIAGRAM);
        const svg = result.svg || result;
        
        // Render in container and measure
        container.innerHTML = svg;
        const svgElement = container.querySelector('svg');
        
        if (svgElement) {
          const bbox = svgElement.getBBox?.();
          if (bbox) {
            const padding = 30;
            const newViewBox = `${bbox.x - padding} ${bbox.y - padding} ${bbox.width + 2*padding} ${bbox.height + 2*padding}`;
            svgElement.setAttribute('viewBox', newViewBox);
            addLog(`Method 2 - Updated viewBox: ${newViewBox}`);
          }
          const finalSvg = svgElement.outerHTML;
          setMethod2Result(finalSvg);
          addLog(`Method 2 success: SVG length ${finalSvg.length}`);
        } else {
          setMethod2Result(svg);
          addLog(`Method 2 success (no post-processing): SVG length ${svg.length}`);
        }
      } finally {
        document.body.removeChild(container);
      }
    } catch (error) {
      addLog(`Method 2 failed: ${error}`);
      setMethod2Result('');
    }
  };

  const testMethod3 = async () => {
    addLog('Testing Method 3: DOM-based rendering');
    try {
      // Create container and use mermaid.run
      const container = document.createElement('div');
      container.style.position = 'absolute';
      container.style.top = '-20000px';
      container.style.width = '6000px';
      container.style.height = '6000px';
      document.body.appendChild(container);

      try {
        container.innerHTML = `<div class="mermaid">${TEST_DIAGRAM}</div>`;
        await mermaid.run({ nodes: container.querySelectorAll('.mermaid') });
        
        const svgElement = container.querySelector('svg');
        if (svgElement) {
          const svg = svgElement.outerHTML;
          setMethod3Result(svg);
          addLog(`Method 3 success: SVG length ${svg.length}`);
          
          // Analyze SVG
          const viewBoxMatch = svg.match(/viewBox="([^"]+)"/);
          const widthMatch = svg.match(/width="([^"]+)"/);
          const heightMatch = svg.match(/height="([^"]+)"/);
          addLog(`Method 3 - ViewBox: ${viewBoxMatch?.[1] || 'NONE'}, Width: ${widthMatch?.[1] || 'NONE'}, Height: ${heightMatch?.[1] || 'NONE'}`);
        } else {
          throw new Error('No SVG generated');
        }
      } finally {
        document.body.removeChild(container);
      }
    } catch (error) {
      addLog(`Method 3 failed: ${error}`);
      setMethod3Result('');
    }
  };

  const downloadSVG = (svg: string, method: string) => {
    if (!svg) return;
    
    const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `mermaid-test-${method}-${Date.now()}.svg`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="h4" gutterBottom>
        Mermaid Rendering Method Comparison
      </Typography>
      
      <Alert severity="info" sx={{ mb: 3 }}>
        This component tests different methods of rendering Mermaid diagrams to identify which approach prevents SVG cutoff.
      </Alert>

      <Box sx={{ mb: 3 }}>
        <Button variant="contained" onClick={testMethod1} sx={{ mr: 1 }}>
          Test Method 1
        </Button>
        <Button variant="contained" onClick={testMethod2} sx={{ mr: 1 }}>
          Test Method 2
        </Button>
        <Button variant="contained" onClick={testMethod3}>
          Test Method 3
        </Button>
      </Box>

      <Box sx={{ display: 'flex', gap: 2, mb: 3 }}>
        <Paper sx={{ flex: 1, p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Method 1: Standard API
          </Typography>
          <Box sx={{ mb: 2 }}>
            <Button size="small" onClick={() => downloadSVG(method1Result, 'method1')} disabled={!method1Result}>
              Download SVG
            </Button>
          </Box>
          <Box 
            sx={{ 
              border: '1px solid #ccc', 
              minHeight: '200px', 
              overflow: 'auto',
              '& svg': { width: '100%', height: 'auto' }
            }}
            dangerouslySetInnerHTML={{ __html: method1Result }}
          />
        </Paper>

        <Paper sx={{ flex: 1, p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Method 2: Large Container
          </Typography>
          <Box sx={{ mb: 2 }}>
            <Button size="small" onClick={() => downloadSVG(method2Result, 'method2')} disabled={!method2Result}>
              Download SVG
            </Button>
          </Box>
          <Box 
            sx={{ 
              border: '1px solid #ccc', 
              minHeight: '200px', 
              overflow: 'auto',
              '& svg': { width: '100%', height: 'auto' }
            }}
            dangerouslySetInnerHTML={{ __html: method2Result }}
          />
        </Paper>

        <Paper sx={{ flex: 1, p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Method 3: DOM-based
          </Typography>
          <Box sx={{ mb: 2 }}>
            <Button size="small" onClick={() => downloadSVG(method3Result, 'method3')} disabled={!method3Result}>
              Download SVG
            </Button>
          </Box>
          <Box 
            sx={{ 
              border: '1px solid #ccc', 
              minHeight: '200px', 
              overflow: 'auto',
              '& svg': { width: '100%', height: 'auto' }
            }}
            dangerouslySetInnerHTML={{ __html: method3Result }}
          />
        </Paper>
      </Box>

      <Paper sx={{ p: 2 }}>
        <Typography variant="h6" gutterBottom>
          Debug Logs
        </Typography>
        <Box 
          sx={{ 
            maxHeight: '200px', 
            overflow: 'auto', 
            fontFamily: 'monospace', 
            fontSize: '0.8rem',
            backgroundColor: '#f5f5f5',
            p: 1,
            borderRadius: 1
          }}
        >
          {logs.map((log, index) => (
            <div key={index}>{log}</div>
          ))}
        </Box>
        <Button 
          size="small" 
          onClick={() => setLogs([])} 
          sx={{ mt: 1 }}
        >
          Clear Logs
        </Button>
      </Paper>
    </Box>
  );
}
