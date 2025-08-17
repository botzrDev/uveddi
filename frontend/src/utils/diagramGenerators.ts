// Utility functions to generate diagrams based on finding type

export function generateDiagramForFinding(finding: any): string {
  const type = finding.type?.toLowerCase();
  console.log('Generating diagram for finding type:', type, 'from original:', finding.type);
  
  try {
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

function generateGodObjectDiagram(finding: any): string {
  // Extract real information from the finding
  const fileName = finding.file?.split('/').pop()?.replace('.rs', '') || 'GodObject';
  const filePath = finding.file || 'unknown/path';
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
  
  // Create clean names for Mermaid
  const cleanFileName = fileName
    .replace(/[^a-zA-Z0-9_]/g, '_')
    .replace(/^[0-9]/, 'file_$&')
    .substring(0, 20);
  
  const cleanCodeName = codeSnippet
    .replace(/\s+/g, '_')
    .replace(/[^a-zA-Z0-9_]/g, '_')
    .replace(/^[0-9]/, '_$&')
    .substring(0, 25);

  return `flowchart TD
    File["📄 ${fileName}<br/>${filePath}"]
    Active["✅ Active Code<br/>Used Functions"]
    Dead["💀 Dead Code<br/>${cleanCodeName}"]
    
    File --> Active
    File -.-> Dead
    
    Active --> Method1["fn used_function_1()"]
    Active --> Method2["fn used_function_2()"]
    Active --> Method3["fn active_logic()"]
    
    Dead --> UnusedCode["❌ ${cleanCodeName}<br/>Line ${finding.startLine || '?'}"]
    Dead --> DeadLogic["❌ Unreachable Code"]
    Dead --> LegacyCode["❌ Legacy Function"]
    
    style File fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style Active fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    style Dead fill:#ffebee,stroke:#f44336,stroke-width:3px
    style UnusedCode fill:#ffcccb,stroke:#d32f2f,stroke-width:2px
    style DeadLogic fill:#ffcccb,stroke:#d32f2f,stroke-width:2px
    style LegacyCode fill:#ffcccb,stroke:#d32f2f,stroke-width:2px
    
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