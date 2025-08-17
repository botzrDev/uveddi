import React, { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { Box, Paper, Typography, Alert, CircularProgress } from '@mui/material';
import mermaid from 'mermaid';

interface MermaidDiagramProps {
  definition: string;
  title?: string;
  className?: string;
}

export default function MermaidDiagram({ definition, title, className }: MermaidDiagramProps) {
  const elementRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [diagramId] = useState(() => `mermaid-${Math.random().toString(36).substr(2, 9)}`);
  const [mounted, setMounted] = useState(false);

  // Callback ref to ensure we know when the element is actually mounted
  const refCallback = (node: HTMLDivElement | null) => {
    elementRef.current = node;
    if (node) {
      console.log('MermaidDiagram: Element ref callback - element is now available');
      setMounted(true);
    } else {
      console.log('MermaidDiagram: Element ref callback - element removed');
      setMounted(false);
    }
  };

  useEffect(() => {
    // Initialize mermaid with theme settings
    console.log('MermaidDiagram: Initializing mermaid');
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
      }
    });
  }, []);

  // Render when element is mounted and definition changes
  useEffect(() => {
    if (!mounted || !definition.trim()) {
      if (!definition.trim()) {
        console.log('MermaidDiagram: Empty definition');
        setError('No diagram definition provided');
        setLoading(false);
      }
      return;
    }

    const renderDiagram = async () => {
      console.log('MermaidDiagram: Starting render process, mounted:', mounted);
      
      if (!elementRef.current) {
        console.error('MermaidDiagram: Element ref not available despite mounted state');
        setError('Failed to initialize diagram container');
        setLoading(false);
        return;
      }

      setLoading(true);
      setError(null);

      try {
        // Clear previous content
        elementRef.current.innerHTML = '';

        // Validate and render the diagram
        console.log('MermaidDiagram: Rendering diagram with mermaid.render');
        const { svg } = await mermaid.render(diagramId, definition);
        
        if (elementRef.current) {
          elementRef.current.innerHTML = svg;
          
          // Make diagram responsive
          const svgElement = elementRef.current.querySelector('svg');
          if (svgElement) {
            svgElement.setAttribute('width', '100%');
            svgElement.setAttribute('height', 'auto');
            svgElement.style.maxWidth = '100%';
            svgElement.style.height = 'auto';
          }
          console.log('MermaidDiagram: Successfully rendered diagram');
        }
      } catch (err) {
        console.error('Mermaid rendering error:', err);
        setError(err instanceof Error ? err.message : 'Failed to render diagram');
      } finally {
        setLoading(false);
      }
    };

    // Small delay to ensure DOM is fully ready
    const timeoutId = setTimeout(renderDiagram, 100);

    return () => {
      clearTimeout(timeoutId);
    };
  }, [mounted, definition, diagramId]);

  if (loading) {
    return (
      <Paper sx={{ p: 3, textAlign: 'center' }}>
        {title && (
          <Typography variant="h6" gutterBottom>
            {title}
          </Typography>
        )}
        <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', gap: 2 }}>
          <CircularProgress size={24} />
          <Typography variant="body2" color="text.secondary">
            Rendering diagram...
          </Typography>
        </Box>
      </Paper>
    );
  }

  if (error) {
    return (
      <Paper sx={{ p: 3 }}>
        {title && (
          <Typography variant="h6" gutterBottom>
            {title}
          </Typography>
        )}
        <Alert severity="error">
          <Typography variant="body2">
            Failed to render diagram: {error}
          </Typography>
        </Alert>
      </Paper>
    );
  }

  return (
    <Paper sx={{ p: 2, overflow: 'auto' }} className={className}>
      {title && (
        <Typography variant="h6" gutterBottom sx={{ mb: 2 }}>
          {title}
        </Typography>
      )}
      <Box
        ref={elementRef}
        sx={{
          textAlign: 'center',
          '& svg': {
            maxWidth: '100%',
            height: 'auto',
          },
        }}
      />
    </Paper>
  );
}

// Utility function to generate diagrams based on finding type
export function generateDiagramForFinding(finding: any): string {
  const type = finding.type?.toLowerCase();
  console.log('Generating diagram for finding type:', type, 'from original:', finding.type);
  
  switch (type) {
    case 'god object':
      return generateGodObjectDiagram(finding);
    case 'dead code':
      return generateDeadCodeDiagram(finding);
    case 'code duplication':
      return generateCodeDuplicationDiagram(finding);
    case 'circular dependency':
      return generateCircularDependencyDiagram(finding);
    case 'tight coupling':
      return generateTightCouplingDiagram(finding);
    default:
      console.log('Using generic diagram for type:', type);
      return generateGenericDiagram(finding);
  }
}

function generateGodObjectDiagram(finding: any): string {
  // Create a clean class name for Mermaid (must start with letter, only letters/numbers/underscores)
  let className = (finding.title || 'GodObject')
    .replace(/\s+/g, '')
    .replace(/[^a-zA-Z0-9_]/g, '_')
    .replace(/^[0-9]/, '_$&')
    .replace(/_+/g, '_') // Replace multiple underscores with single
    .replace(/^_+|_+$/g, ''); // Remove leading/trailing underscores
  
  // Ensure it starts with a letter and has reasonable length
  if (!className || !/^[a-zA-Z]/.test(className)) {
    className = 'GodObject';
  }
  
  // Limit length to avoid issues
  if (className.length > 30) {
    className = className.substring(0, 30);
  }
  
  console.log('Generating God Object diagram for sanitized class name:', className);
  
  const diagram = `classDiagram
    class ${className} {
        +field1: String
        +field2: String
        +field3: Number
        +field4: Boolean
        +field5: Array
        +massiveMethod()
        +anotherLargeMethod()
        +duplicateCode1()
        +duplicateCode2()
        +unusedMethod()
        +complexCalculation()
        +dataProcessing()
        +businessLogic()
    }
    
    note for ${className} "God Object Detected\\n\\n• 20+ fields/methods\\n• Multiple responsibilities\\n• High complexity\\n• Violation of SRP"
    
    ${className} --> DataLayer : manages
    ${className} --> UILayer : controls
    ${className} --> BusinessLogic : implements
    ${className} --> Validation : handles
    
    class DataLayer {
        +save()
        +load()
    }
    
    class UILayer {
        +render()
        +update()
    }
    
    class BusinessLogic {
        +calculate()
        +process()
    }
    
    class Validation {
        +validate()
        +sanitize()
    }`;
  
  console.log('Generated God Object diagram:', diagram.substring(0, 100) + '...');
  return diagram;
}

function generateDeadCodeDiagram(finding: any): string {
  // Create a clean method name for Mermaid
  let methodName = (finding.title || 'unused_method')
    .replace(/\s+/g, '_')
    .replace(/[^a-zA-Z0-9_]/g, '_')
    .replace(/^[0-9]/, '_$&')
    .replace(/_+/g, '_') // Replace multiple underscores with single
    .replace(/^_+|_+$/g, ''); // Remove leading/trailing underscores
  
  // Ensure it starts with a letter and has reasonable length
  if (!methodName || !/^[a-zA-Z]/.test(methodName)) {
    methodName = 'unused_method';
  }
  
  // Limit length to avoid issues
  if (methodName.length > 30) {
    methodName = methodName.substring(0, 30);
  }
  
  return `flowchart TD
    A[Main Application] --> B[Active Module]
    B --> C[Used Method 1]
    B --> D[Used Method 2]
    B --> E[Used Method 3]
    
    B -.-> F[${methodName}]
    B -.-> G[Another Unused Method]
    B -.-> H[Legacy Function]
    
    F --> I[Unreachable Code Block]
    G --> J[Deprecated Logic]
    H --> K[Old Implementation]
    
    style F fill:#ff6b6b,stroke:#d63384,stroke-width:2px
    style G fill:#ff6b6b,stroke:#d63384,stroke-width:2px
    style H fill:#ff6b6b,stroke:#d63384,stroke-width:2px
    style I fill:#ffcccc,stroke:#d63384,stroke-width:1px
    style J fill:#ffcccc,stroke:#d63384,stroke-width:1px
    style K fill:#ffcccc,stroke:#d63384,stroke-width:1px
    
    classDef deadCode fill:#ff6b6b,stroke:#d63384,stroke-width:2px,color:#fff
    classDef unreachable fill:#ffcccc,stroke:#d63384,stroke-width:1px
    classDef active fill:#6bcf7f,stroke:#28a745,stroke-width:2px`;
}

function generateCodeDuplicationDiagram(finding: any): string {
  return `flowchart LR
    A[Method A] --> D[Duplicated Logic Block]
    B[Method B] --> D
    C[Method C] --> D
    
    D --> E[Common Functionality]
    E --> F[Refactoring Opportunity]
    
    F --> G[Extract Method]
    F --> H[Create Utility Class]
    F --> I[Use Inheritance]
    
    style D fill:#ffd93d,stroke:#ffc107,stroke-width:3px
    style F fill:#6bcf7f,stroke:#28a745,stroke-width:2px
    style G fill:#b3d9ff,stroke:#007bff,stroke-width:2px
    style H fill:#b3d9ff,stroke:#007bff,stroke-width:2px
    style I fill:#b3d9ff,stroke:#007bff,stroke-width:2px
    
    subgraph "Current State"
        A
        B
        C
        D
    end
    
    subgraph "Refactored State"
        G
        H
        I
    end`;
}

function generateCircularDependencyDiagram(_finding: any): string {
  return `flowchart LR
    A[Module A] --> B[Module B]
    B --> C[Module C]
    C --> D[Module D]
    D --> A
    
    A -.-> E[External Dependency]
    B -.-> F[Shared Utility]
    
    style A fill:#ff6b6b,stroke:#d63384,stroke-width:3px
    style B fill:#ff6b6b,stroke:#d63384,stroke-width:3px
    style C fill:#ff6b6b,stroke:#d63384,stroke-width:3px
    style D fill:#ff6b6b,stroke:#d63384,stroke-width:3px
    
    classDef cyclic fill:#ff6b6b,stroke:#d63384,stroke-width:3px,color:#fff
    classDef normal fill:#f8f9fa,stroke:#6c757d,stroke-width:1px`;
}

function generateTightCouplingDiagram(_finding: any): string {
  return `classDiagram
    class ComponentA {
        -componentB: ComponentB
        -componentC: ComponentC
        +methodA()
        +dependsOnB()
        +dependsOnC()
    }
    
    class ComponentB {
        -componentA: ComponentA
        -componentC: ComponentC
        +methodB()
        +dependsOnA()
        +dependsOnC()
    }
    
    class ComponentC {
        -componentA: ComponentA
        -componentB: ComponentB
        +methodC()
        +dependsOnA()
        +dependsOnB()
    }
    
    ComponentA --> ComponentB : tightly coupled
    ComponentA --> ComponentC : tightly coupled
    ComponentB --> ComponentA : tightly coupled
    ComponentB --> ComponentC : tightly coupled
    ComponentC --> ComponentA : tightly coupled
    ComponentC --> ComponentB : tightly coupled
    
    note for ComponentA "High Coupling Detected\\n\\n• Direct dependencies\\n• Hard to test\\n• Difficult to modify"`;
}

function generateGenericDiagram(finding: any): string {
  const type = finding.type || 'Issue';
  
  return `flowchart TD
    A[Code Analysis] --> B{${type} Detected}
    B --> C[File: ${finding.file || 'unknown'}]
    C --> D[Line: ${finding.startLine || 'N/A'}]
    D --> E[Severity: ${finding.severity || 'unknown'}]
    E --> F[Confidence: ${Math.round((finding.confidence || 0) * 100)}%]
    
    F --> G[Recommendation]
    G --> H[Review Code]
    G --> I[Apply Fix]
    G --> J[Run Tests]
    
    style B fill:#ffd93d,stroke:#ffc107,stroke-width:2px
    style G fill:#6bcf7f,stroke:#28a745,stroke-width:2px`;
}