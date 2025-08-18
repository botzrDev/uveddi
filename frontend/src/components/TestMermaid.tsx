import { Paper, Typography } from '@mui/material';
import mermaid from 'mermaid';
import { useEffect, useRef } from 'react';

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

        const testDiagram = `flowchart TD
    Start([🚀 Start Analysis]) --> Config{Configure Settings}
    Config -->|Default| QuickScan[Quick Scan Mode]
    Config -->|Custom| DetailedScan[Detailed Scan Mode]
    
    QuickScan --> ScanFiles[📁 Scan Source Files]
    DetailedScan --> ScanFiles
    
    ScanFiles --> ParseAST[🌳 Parse AST]
    ParseAST --> DetectIssues[🔍 Detect Issues]
    
    DetectIssues --> GodObject[👹 God Object Detection]
    DetectIssues --> DeadCode[💀 Dead Code Detection]  
    DetectIssues --> Coupling[🔗 Tight Coupling Detection]
    DetectIssues --> Duplication[📋 Code Duplication Detection]
    
    GodObject --> Severity{Assess Severity}
    DeadCode --> Severity
    Coupling --> Severity
    Duplication --> Severity
    
    Severity -->|Critical| HighPriority[🔴 High Priority Issues]
    Severity -->|Medium| MediumPriority[🟡 Medium Priority Issues] 
    Severity -->|Low| LowPriority[🟢 Low Priority Issues]
    
    HighPriority --> GenerateReport[📊 Generate Report]
    MediumPriority --> GenerateReport
    LowPriority --> GenerateReport
    
    GenerateReport --> HTMLReport[📄 HTML Report]
    GenerateReport --> JSONReport[📋 JSON Report]
    GenerateReport --> InteractiveReport[🖥️ Interactive Dashboard]
    
    HTMLReport --> Complete([✅ Analysis Complete])
    JSONReport --> Complete
    InteractiveReport --> Complete
    
    style Start fill:#e1f5fe,stroke:#01579b,stroke-width:3px
    style Complete fill:#e8f5e8,stroke:#2e7d32,stroke-width:3px
    style HighPriority fill:#ffebee,stroke:#c62828,stroke-width:2px
    style MediumPriority fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    style LowPriority fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    style Config fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    style GenerateReport fill:#e3f2fd,stroke:#0d47a1,stroke-width:2px`;

        console.log('TestMermaid: Rendering test diagram');
        
        if (elementRef.current) {
          const result = await mermaid.render('test-diagram-complex', testDiagram);
          const svg = (result as any).svg || result;
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