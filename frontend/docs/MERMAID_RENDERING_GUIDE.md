# Mermaid Diagram Rendering Guide

## Overview

The Uveddi frontend uses Mermaid.js for rendering interactive diagrams in the dashboard. This guide documents the implementation and troubleshooting steps for the diagram rendering system.

## Architecture

### Components

1. **MermaidDiagram** (`src/components/MermaidDiagram.tsx`)
   - Main component for rendering individual Mermaid diagrams
   - Handles initialization, rendering, and error states
   - Uses callback ref pattern for reliable DOM element access

2. **SimpleMermaidTest** (`src/components/SimpleMermaidTest.tsx`)
   - Test component for verifying Mermaid functionality
   - Useful for debugging rendering issues

3. **DashboardPage** (`src/pages/DashboardPage.tsx`)
   - Integrates MermaidDiagram components into the main dashboard
   - Maps diagram data from report JSON to component props

## Implementation Details

### Key Technical Solutions

#### Callback Ref Pattern
The MermaidDiagram component uses a callback ref instead of useRef to ensure reliable DOM element access:

```tsx
const refCallback = (element: HTMLDivElement | null) => {
  console.log('MermaidDiagram: refCallback called with element:', element ? 'YES' : 'NO');
  setElementRef(element);
};

// In JSX:
<Box ref={refCallback} />
```

#### Timing and Race Condition Prevention
- Uses `requestAnimationFrame` instead of `setTimeout` for better DOM timing
- Component always renders the container Box element to ensure ref callback fires
- Conditional rendering for loading states and error messages

#### Mermaid Configuration
```tsx
mermaid.initialize({
  startOnLoad: false,
  theme: 'default',
  themeVariables: {
    primaryColor: '#1976d2',
    primaryTextColor: '#000',
    // ... theme customization
  },
  fontFamily: '"Roboto", "Helvetica", "Arial", sans-serif',
  fontSize: 14,
  flowchart: {
    useMaxWidth: true,
    htmlLabels: true,
    curve: 'basis'
  },
  securityLevel: 'loose'
});
```

### Rendering Process

1. **Initialization**: Mermaid is initialized once per component instance
2. **Element Mounting**: Callback ref ensures DOM element is available
3. **Definition Validation**: Diagram syntax is validated before rendering
4. **SVG Generation**: Mermaid renders the diagram to SVG
5. **DOM Insertion**: SVG is inserted into the container element
6. **Responsive Styling**: SVG attributes are updated for responsiveness

## Data Format

Diagrams are defined in the report JSON under the `diagrams` array:

```json
{
  "diagrams": [
    {
      "id": "god-object-diagram",
      "kind": "class",
      "title": "God Object Analysis",
      "source": "classDiagram\n    class GodObject {\n        +field1: String\n        ...",
      "description": "Visualization of the God Object anti-pattern",
      "metadata": {
        "width": 800,
        "height": 600,
        "theme": "default"
      }
    }
  ]
}
```

### Supported Diagram Types

- **Class Diagrams**: `classDiagram`
- **Flowcharts**: `flowchart` or `graph`
- **Dependency Graphs**: `graph TD/LR`
- **Sequence Diagrams**: `sequenceDiagram`

## Troubleshooting

### Common Issues and Solutions

#### "Diagram Rendering..." Stuck State

**Symptoms**: Component shows loading spinner indefinitely
**Cause**: DOM element not available for rendering
**Solution**: 
- Check console for "refCallback called with element: NO"
- Ensure container Box is always rendered (not conditionally)
- Verify callback ref pattern is used

#### "No element ref or cancelled" Error

**Symptoms**: Console error during render attempt
**Cause**: Race condition between render and element availability
**Solution**:
- Use `requestAnimationFrame` for timing
- Check element availability before rendering
- Implement proper cleanup in useEffect

#### SVG Not Displaying

**Symptoms**: Component shows success but no visual diagram
**Cause**: SVG styling or size issues
**Solution**:
- Check SVG element has proper width/height attributes
- Verify CSS doesn't hide SVG (`display: none`)
- Ensure container has minimum height

#### Mermaid Syntax Errors

**Symptoms**: Parse errors in console
**Cause**: Invalid Mermaid diagram syntax
**Solution**:
- Validate diagram syntax with `mermaid.parse()`
- Check for escaped newlines (`\\n` vs `\n`)
- Test diagram in online Mermaid editor

### Debug Console Messages

Enable detailed logging by checking these console messages:

```
MermaidDiagram: refCallback called with element: YES/NO
MermaidDiagram: Mermaid initialized successfully
MermaidDiagram: Parse successful
MermaidDiagram: Render result SVG length: [number]
MermaidDiagram: Successfully rendered diagram: [id]
```

### Testing Approach

1. **Verify SimpleMermaidTest**: Should show "SUCCESS: Diagram rendered!"
2. **Check Console Logs**: Look for "refCallback called with element: YES"
3. **Validate Diagram Data**: Ensure JSON contains valid Mermaid syntax
4. **Test Different Diagram Types**: Verify each diagram type renders correctly

## Best Practices

### Component Usage

```tsx
// Correct usage
<MermaidDiagram 
  definition={diagram.source}
  title={diagram.title}
/>

// With error handling
{report.diagrams?.map((diagram, index) => (
  <Grid item xs={12} lg={6} key={diagram.id || index}>
    <MermaidDiagram 
      definition={diagram.source}
      title={diagram.title}
    />
  </Grid>
))}
```

### Performance Considerations

- Each component initializes its own Mermaid instance
- Diagram IDs are auto-generated to prevent conflicts
- SVG content is cached by browser
- Use `React.memo` for expensive diagram components if needed

### Error Handling

- Always validate diagram syntax before rendering
- Provide fallback content for failed renders
- Show meaningful error messages to users
- Log detailed errors for debugging

## Dependencies

- **mermaid**: ^10.x (core rendering library)
- **@mui/material**: UI components and styling
- **react**: ^18.x (hooks and component lifecycle)

## Future Improvements

- Implement diagram export functionality
- Add theme switching for diagrams
- Support for custom Mermaid plugins
- Diagram interaction and zooming capabilities