// Utility functions to generate diagrams based on finding type

export function generateDiagramForFinding(finding: any): string {
  const typeKey = (finding.type || '').toString().toLowerCase().replace(/_/g, ' ').trim();
  console.log('Generating diagram for finding type:', typeKey, 'from original:', finding.type);
  
  try {
    switch (typeKey) {
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
        console.log('Using generic diagram for type:', typeKey);
        return generateGenericDiagram(finding);
    }
  } catch (error) {
    console.error('Error generating diagram for finding:', error);
    return generateSimpleFallbackDiagram(finding);
  }
}

function generateSimpleFallbackDiagram(finding: any): string {
  const fileName = finding.file?.split('/').pop() || 'file.rs';
  const issueType = finding.type || 'Issue';
  
  return `flowchart TD
    File["📄 ${fileName}"]
    Issue["⚠️ ${issueType} Detected"]
    
    File --> Issue
    Issue --> Fix["🔧 Needs Attention"]
    
    style File fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style Issue fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style Fix fill:#e8f5e8,stroke:#4caf50,stroke-width:2px`;
}

function sanitizeLabel(input: any): string {
  const s = String(input ?? '');
  return s
    .replace(/"/g, "'")
    .replace(/`/g, "'")
    .replace(/[{}<>]/g, ' ')
    .replace(/\n+/g, '\\n')
    .replace(/[^\x20-\x7E]/g, '');
}

function generateGodObjectDiagram(finding: any): string {
  // Extract real information from the finding
  const rawFileName = finding.file?.split('/').pop()?.replace('.rs', '') || 'GodObject';
  const fileName = sanitizeLabel(rawFileName);
  const filePath = sanitizeLabel(finding.file || 'unknown/path');
  const lineNumber = finding.startLine || '?';
  
  // Create a clean class name for Mermaid
  let className = fileName
    .replace(/[^a-zA-Z0-9_]/g, '_')
    .replace(/^[0-9]/, 'Struct_$&')
    .substring(0, 20);
  
  if (!className || !/^[a-zA-Z]/.test(className)) {
    className = 'GodObject';
  }
  
  console.log('Generating God Object diagram for real class:', className);
  
  // Use flowchart instead of classDiagram to avoid syntax issues
  const diagram = `flowchart TD
    Main["🏗️ ${fileName}.rs<br/>Line: ${lineNumber}"]
    GodObj["⚠️ ${className}<br/>God Object Detected"]
    
    Main --> GodObj
    
    subgraph responsibilities["📋 Too Many Responsibilities"]
        Data["💾 Data Management<br/>Fields: config, state, data"]
        Network["🌐 Network Operations<br/>Methods: handle_request, send"]
        Business["⚖️ Business Logic<br/>Methods: process, validate"]
        IO["📁 I/O Operations<br/>Methods: save_to_db, log"]
    end
    
    GodObj --> Data
    GodObj --> Network
    GodObj --> Business
    GodObj --> IO
    
    subgraph solution["✅ Recommended Split"]
        DataHandler["DataHandler<br/>struct"]
        NetworkService["NetworkService<br/>struct"]
        BusinessLogic["BusinessLogic<br/>struct"]
        IOManager["IOManager<br/>struct"]
    end
    
    Data -.->|"refactor to"| DataHandler
    Network -.->|"refactor to"| NetworkService
    Business -.->|"refactor to"| BusinessLogic
    IO -.->|"refactor to"| IOManager
    
    style Main fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style GodObj fill:#ffebee,stroke:#d32f2f,stroke-width:3px
    style Data fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style Network fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style Business fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style IO fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style DataHandler fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    style NetworkService fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    style BusinessLogic fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    style IOManager fill:#e8f5e8,stroke:#4caf50,stroke-width:2px`;
  
  console.log('Generated God Object diagram:', diagram.substring(0, 100) + '...');
  return diagram;
}

function generateDeadCodeDiagram(finding: any): string {
  // Extract real information from the finding
  const fileName = finding.file?.split('/').pop()?.replace('.rs', '') || 'unknown_file';
  const codeSnippet = finding.codeSnippet || finding.title || 'unused_code';
  const filePath = finding.file || 'unknown/path';
  const startLine = finding.startLine || finding.line || '?';
  const endLine = finding.endLine || '';
  const confidence = finding.confidence || (finding.description?.match(/confidence: ([\d.]+)%/) ? 
    parseFloat(finding.description.match(/confidence: ([\d.]+)%/)[1]) : null);
  
  // Parse the type of dead code from the description or code snippet
  let codeType = 'code';
  let codeName = codeSnippet;
  
  if (finding.description?.includes("function '")) {
    codeType = 'function';
    const match = finding.description.match(/function '([^']+)'/);
    if (match) codeName = match[1];
  } else if (finding.description?.includes("struct '")) {
    codeType = 'struct';
    const match = finding.description.match(/struct '([^']+)'/);
    if (match) codeName = match[1];
  } else if (finding.description?.includes("variable '")) {
    codeType = 'variable';
    const match = finding.description.match(/variable '([^']+)'/);
    if (match) codeName = match[1];
  }
  
  // Create clean names for Mermaid
  const cleanFileName = sanitizeLabel(fileName).replace(/[^a-zA-Z0-9_]/g, '_').substring(0, 20);
  const cleanCodeName = sanitizeLabel(codeName).replace(/[^a-zA-Z0-9_]/g, '_').substring(0, 25);
  const lineRange = endLine && endLine !== startLine ? `${startLine}-${endLine}` : `${startLine}`;
  const confidenceText = confidence ? `${confidence}% confidence` : '';
  
  // Determine icon based on code type
  const getIcon = (type: string) => {
    switch (type) {
      case 'function': return '🔧';
      case 'struct': return '📦';
      case 'variable': return '📊';
      default: return '💀';
    }
  };
  
  console.log('Generating Dead Code diagram for:', codeName, 'type:', codeType, 'in file:', fileName);
  
  return `flowchart TB
    File["📄 ${fileName}.rs<br/>Line: ${lineRange}"]
    
    subgraph analysis["🔍 Analysis Result"]
        DeadCode["${getIcon(codeType)} ${codeName}<br/>❌ DEAD CODE"]
        Details["🔍 Never referenced<br/>${confidence ? confidenceText : 'Not called anywhere'}"]
    end
    
    subgraph actions["🛠️ Actions"]
        direction TB
        Remove["🗑️ Safe to Remove"]
        ${codeType === 'function' ? 'Export["📤 Make Public"]' : ''}
        Archive["📦 Archive First"]
    end
    
    File --> analysis
    DeadCode --> Details
    analysis -.-> actions
    
    style File fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style DeadCode fill:#ffebee,stroke:#d32f2f,stroke-width:3px,color:#000
    style Details fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style Remove fill:#ffcccb,stroke:#d32f2f,stroke-width:2px
    ${codeType === 'function' ? 'style Export fill:#e8f5e8,stroke:#4caf50,stroke-width:2px' : ''}
    style Archive fill:#e3f2fd,stroke:#1976d2,stroke-width:2px`;
}

function generateCodeDuplicationDiagram(finding: any): string {
  const filePath = sanitizeLabel(finding.file || 'unknown.rs');
  const fileName = sanitizeLabel((finding.file || 'unknown.rs').split('/').pop() || 'unknown.rs');
  const codeName = sanitizeLabel(finding.codeSnippet || finding.title || 'duplicated_code');
  const similarityMatch = (finding.message || finding.description || '').match(/(\d{2,3})%/);
  const similarity = similarityMatch ? `${similarityMatch[1]}%` : 'high';

  return `flowchart LR
    A["📄 ${fileName}"] --> D["📋 Duplicated: ${codeName}"]
    B[Other Location] --> D
    
    D --> E[Similarity: ${similarity}]
    E --> F[Refactoring Opportunity]
    
    F --> G[Extract Function]
    F --> H[Create Utility]
    F --> I[Deduplicate Shared Logic]
    
    style D fill:#ffd93d,stroke:#ffc107,stroke-width:3px
    style F fill:#6bcf7f,stroke:#28a745,stroke-width:2px
    style G fill:#b3d9ff,stroke:#007bff,stroke-width:2px
    style H fill:#b3d9ff,stroke:#007bff,stroke-width:2px
    style I fill:#b3d9ff,stroke:#007bff,stroke-width:2px`;
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