import { Close, Code, Download, Fullscreen, OpenInNew } from '@mui/icons-material';
import { Alert, Box, Button, CircularProgress, Dialog, DialogActions, DialogContent, DialogTitle, IconButton, Paper, Tooltip, Typography } from '@mui/material';
import mermaid from 'mermaid';
import { useEffect, useState } from 'react';

interface MermaidDiagramProps {
  definition: string;
  title?: string;
  className?: string;
  previewHeight?: string;
}

export default function MermaidDiagram({ definition, title, className, previewHeight = '300px' }: MermaidDiagramProps) {
  const [elementRef, setElementRef] = useState<HTMLDivElement | null>(null);
  const [fullscreenElementRef, setFullscreenElementRef] = useState<HTMLDivElement | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [diagramId] = useState(() => `mermaid-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`);
  const [fullscreenDiagramId] = useState(() => `mermaid-fullscreen-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`);
  const [mermaidReady, setMermaidReady] = useState(false);
  const [isFullscreenOpen, setIsFullscreenOpen] = useState(false);
  const [fullscreenLoading, setFullscreenLoading] = useState(false);

  // Callback refs to track when elements are mounted
  const refCallback = (element: HTMLDivElement | null) => {
    console.log('MermaidDiagram: refCallback called with element:', element ? 'YES' : 'NO');
    setElementRef(element);
  };

  const fullscreenRefCallback = (element: HTMLDivElement | null) => {
    console.log('MermaidDiagram: fullscreenRefCallback called with element:', element ? 'YES' : 'NO');
    setFullscreenElementRef(element);
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
        fontSize: 16,
        // Global settings to prevent constraints
        useMaxWidth: false,
        wrap: false,
        flowchart: {
          useMaxWidth: false, 
          htmlLabels: true,
          curve: 'basis',
          nodeSpacing: 60,
          rankSpacing: 100,
          padding: 20,
          // Explicitly set large dimensions
          width: undefined,
          height: undefined,
        },
        class: {
          useMaxWidth: false,
        },
        gitGraph: {
          useMaxWidth: false, 
        },
        sequence: {
          useMaxWidth: false, 
          diagramMarginX: 50,
          diagramMarginY: 20,
          width: undefined,
          height: undefined,
        },
        gantt: {
          useMaxWidth: false,
        },
        journey: {
          useMaxWidth: false,
        },
        timeline: {
          useMaxWidth: false,
        },
        mindmap: {
          useMaxWidth: false,
        },
        securityLevel: 'loose',
        maxTextSize: 90000,
        // Try to disable any automatic sizing
        deterministicIds: false,
        deterministicIDSeed: undefined,
      });
      setMermaidReady(true);
      console.log('MermaidDiagram: Mermaid initialized successfully');
    } catch (initError) {
      console.error('Mermaid initialization failed:', initError);
      setError('Failed to initialize Mermaid');
      setLoading(false);
    }
  }, []);

  // Function to render a diagram in a specific element
  const renderDiagramInElement = async (
    element: HTMLDivElement, 
    diagramIdToUse: string, 
    isFullscreen: boolean = false
  ) => {
    try {
      // Clear previous content
      element.innerHTML = '';

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

      // Create a temporary, large rendering container to avoid viewport constraints
      const tempContainer = document.createElement('div');
      tempContainer.style.position = 'absolute';
      tempContainer.style.top = '-10000px';
      tempContainer.style.left = '-10000px';
      tempContainer.style.width = '4000px'; // Very large width
      tempContainer.style.height = '4000px'; // Very large height
      tempContainer.style.overflow = 'visible';
      tempContainer.style.visibility = 'hidden';
      document.body.appendChild(tempContainer);

      try {
        // Render the diagram in the unconstrained container
        console.log('MermaidDiagram: Rendering with ID:', diagramIdToUse, 'in unconstrained container');
        
        // Method 1: Try the modern render API
        let svg;
        try {
          const result = await mermaid.render(diagramIdToUse, cleanDefinition);
          svg = result.svg || result;
        } catch (renderError) {
          console.warn('Modern render API failed, trying alternative approach:', renderError);
          
          // Method 2: Alternative - render directly in the temp container
          tempContainer.innerHTML = `<div class="mermaid">${cleanDefinition}</div>`;
          await mermaid.run({ nodes: tempContainer.querySelectorAll('.mermaid') });
          
          const renderedSvg = tempContainer.querySelector('svg');
          if (renderedSvg) {
            svg = renderedSvg.outerHTML;
          } else {
            throw new Error('No SVG generated from alternative rendering method');
          }
        }
        
        console.log('MermaidDiagram: Render result SVG length:', svg ? svg.length : 'NULL');
        
        if (!svg || svg.trim().length === 0) {
          throw new Error('Mermaid rendered empty SVG');
        }
        
        // Debug: Log SVG content before any processing
        console.log('=== ORIGINAL SVG DEBUG ===');
        console.log('Raw SVG length:', svg.length);
        console.log('First 300 chars:', svg.substring(0, 300));
        
        // Check if SVG contains viewBox and dimensions in the raw output
        const viewBoxMatch = svg.match(/viewBox="([^"]+)"/);
        const widthMatch = svg.match(/width="([^"]+)"/);
        const heightMatch = svg.match(/height="([^"]+)"/);
        console.log('Original viewBox:', viewBoxMatch ? viewBoxMatch[1] : 'NONE');
        console.log('Original width:', widthMatch ? widthMatch[1] : 'NONE');
        console.log('Original height:', heightMatch ? heightMatch[1] : 'NONE');
        
        // Count SVG elements to see if content is being cut off
        const gElementCount = (svg.match(/<g[^>]*>/g) || []).length;
        const textElementCount = (svg.match(/<text[^>]*>/g) || []).length;
        const pathElementCount = (svg.match(/<path[^>]*>/g) || []).length;
        console.log('SVG contains:', gElementCount, 'groups,', textElementCount, 'texts,', pathElementCount, 'paths');
        console.log('=== END ORIGINAL SVG DEBUG ===');
        
        // Put the SVG in the temporary container first to measure it properly
        tempContainer.innerHTML = svg;
        const tempSvg = tempContainer.querySelector('svg');
        
        if (tempSvg) {
          // Get the actual bounding box of the SVG content
          const bbox = tempSvg.getBBox ? tempSvg.getBBox() : null;
          console.log('SVG BBox:', bbox);
          
          // If we have a bounding box, ensure the viewBox encompasses all content
          if (bbox && bbox.width > 0 && bbox.height > 0) {
            const padding = 20;
            const newViewBox = `${bbox.x - padding} ${bbox.y - padding} ${bbox.width + 2*padding} ${bbox.height + 2*padding}`;
            tempSvg.setAttribute('viewBox', newViewBox);
            console.log('Updated viewBox to:', newViewBox);
          }
          
          // Get the final SVG with proper viewBox
          const finalSvg = tempSvg.outerHTML;
          element.innerHTML = finalSvg;
        } else {
          element.innerHTML = svg;
        }
        
      } finally {
        // Clean up temporary container
        document.body.removeChild(tempContainer);
      }
      
      // Make diagram responsive
      const svgElement = element.querySelector('svg');
      if (svgElement) {
        // Remove any hardcoded width/height attributes
        svgElement.removeAttribute('width');
        svgElement.removeAttribute('height');
        
        // Ensure viewBox exists
        if (!svgElement.getAttribute('viewBox')) {
          const wAttr = svgElement.getAttribute('width');
          const hAttr = svgElement.getAttribute('height');
          const w = wAttr ? parseFloat(wAttr) : 0;
          const h = hAttr ? parseFloat(hAttr) : 0;
          if (w && h) {
            svgElement.setAttribute('viewBox', `0 0 ${w} ${h}`);
          }
        }
        svgElement.setAttribute('preserveAspectRatio', 'xMidYMin meet');
        
        // Set responsive styling based on mode
        if (!isFullscreen) {
          // Preview mode: maintain aspect ratio, allow vertical scrolling if needed
          svgElement.style.width = '100%';
          svgElement.style.height = 'auto';
          svgElement.style.maxWidth = '100%';
          svgElement.style.minHeight = '200px';
          // Remove maxHeight constraint that was cutting off diagrams
          svgElement.style.cursor = 'pointer';
        } else {
          // Fullscreen mode: use available space without cutting off content
          svgElement.style.width = 'auto';
          svgElement.style.height = 'auto';
          svgElement.style.maxWidth = '100%';
          // Allow natural height, container will handle scrolling
          svgElement.style.minWidth = '300px';
        }
        
        svgElement.style.display = 'block';
        svgElement.style.margin = '0 auto';

        // Improved scaling logic that preserves readability
        const vb = (svgElement.getAttribute('viewBox') || '').split(' ');
        const vbW = parseFloat(vb[2] || '0');
        const vbH = parseFloat(vb[3] || '0');
        
        if (vbW && vbH) {
          const containerWidth = element.clientWidth;
          const containerHeight = element.clientHeight;
          
          // Only scale down if diagram is significantly larger than container
          if (containerWidth > 0 && vbW > containerWidth * 1.1) {
            const scaleX = containerWidth / vbW;
            
            // In preview mode, also consider height but be more lenient
            if (!isFullscreen && containerHeight > 0) {
              const maxPreviewHeight = parseInt(previewHeight) || 400;
              const scaleY = Math.min(maxPreviewHeight / vbH, 1);
              const scale = Math.min(scaleX, scaleY, 1);
              
              if (scale < 0.9) {  // Only scale if significant size reduction
                svgElement.style.transformOrigin = 'top center';
                svgElement.style.transform = `scale(${scale})`;
                // Adjust container to fit scaled content
                element.style.height = `${vbH * scale}px`;
              }
            } else if (!isFullscreen) {
              // Scale to fit width in preview
              svgElement.style.transformOrigin = 'top center';
              svgElement.style.transform = `scale(${Math.min(scaleX, 1)})`;
            }
          } else {
            svgElement.style.transform = '';
            element.style.height = 'auto';
          }
        }
      }
      console.log('MermaidDiagram: Successfully rendered diagram:', diagramIdToUse);
      
    } catch (err) {
      console.error('MermaidDiagram: Rendering error for', diagramIdToUse, ':', err);
      console.error('MermaidDiagram: Failed diagram definition:', definition);
      throw err;
    }
  };

  // Render preview diagram when definition changes and element is available
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

    const renderPreviewDiagram = async () => {
      if (!elementRef || isCancelled) {
        console.log('MermaidDiagram: No element ref or cancelled');
        return;
      }

      console.log('MermaidDiagram: Starting render process for preview ID:', diagramId);
      setLoading(true);
      setError(null);

      try {
        await renderDiagramInElement(elementRef, diagramId, false);
        if (!isCancelled) {
          setLoading(false);
        }
      } catch (err) {
        if (!isCancelled) {
          setError(err instanceof Error ? err.message : 'Failed to render diagram');
          setLoading(false);
        }
      }
    };

    // Use requestAnimationFrame to ensure DOM is ready, then render
    requestAnimationFrame(() => {
      if (!isCancelled) {
        console.log('MermaidDiagram: requestAnimationFrame triggered, calling renderPreviewDiagram');
        renderPreviewDiagram();
      }
    });

    return () => {
      isCancelled = true;
    };
  }, [definition, diagramId, mermaidReady, elementRef, previewHeight]);

  // Render fullscreen diagram when modal opens
  useEffect(() => {
    if (!isFullscreenOpen || !fullscreenElementRef || !mermaidReady || !definition) {
      return;
    }

    let isCancelled = false;

    const renderFullscreenDiagram = async () => {
      if (!fullscreenElementRef || isCancelled) {
        return;
      }

      setFullscreenLoading(true);

      try {
        await renderDiagramInElement(fullscreenElementRef, fullscreenDiagramId, true);
        if (!isCancelled) {
          setFullscreenLoading(false);
        }
      } catch (err) {
        if (!isCancelled) {
          console.error('Failed to render fullscreen diagram:', err);
          setFullscreenLoading(false);
        }
      }
    };

    requestAnimationFrame(() => {
      if (!isCancelled) {
        renderFullscreenDiagram();
      }
    });

    return () => {
      isCancelled = true;
    };
  }, [isFullscreenOpen, fullscreenElementRef, mermaidReady, definition, fullscreenDiagramId]);

  const handleOpenFullscreen = () => {
    setIsFullscreenOpen(true);
  };

  const handleDownloadSVG = () => {
    try {
      // Get the current SVG from the preview element
      const svgElement = elementRef?.querySelector('svg');
      if (!svgElement) {
        console.warn('No SVG element found to download');
        return;
      }

      // Log SVG dimensions and content for debugging
      const viewBox = svgElement.getAttribute('viewBox');
      const width = svgElement.getAttribute('width');
      const height = svgElement.getAttribute('height');
      const svgRect = svgElement.getBoundingClientRect();
      
      console.log('=== SVG DEBUG INFO ===');
      console.log('ViewBox:', viewBox);
      console.log('Width attr:', width);
      console.log('Height attr:', height);
      console.log('Client rect:', svgRect);
      console.log('SVG content length:', svgElement.outerHTML.length);
      console.log('First 200 chars:', svgElement.outerHTML.substring(0, 200));
      
      // Create a clean SVG string
      const serializer = new XMLSerializer();
      let svgString = serializer.serializeToString(svgElement);
      
      // Add XML declaration if not present
      if (!svgString.match(/^<\?xml/)) {
        svgString = '<?xml version="1.0" standalone="no"?>\r\n' + svgString;
      }
      
      // Log the final SVG string length
      console.log('Final SVG string length:', svgString.length);
      console.log('=== END DEBUG INFO ===');
      
      // Create blob and download
      const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${title || 'diagram'}-${Date.now()}.svg`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to download SVG:', error);
    }
  };

  const handleViewDefinition = () => {
    // Open a new window to show the raw Mermaid definition
    const win = window.open('', '_blank');
    if (!win) return;
    
    const cleanDefinition = definition.replace(/\\n/g, '\n').trim();
    
    const html = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8"/>
  <title>Mermaid Definition - ${title || 'Diagram'}</title>
  <style>
    body { 
      font-family: monospace; 
      margin: 20px; 
      background: #f5f5f5; 
      color: #333;
    }
    pre { 
      background: white; 
      padding: 20px; 
      border-radius: 8px; 
      border: 1px solid #ddd;
      overflow: auto;
      white-space: pre-wrap;
    }
    .info {
      background: #e3f2fd;
      padding: 15px;
      border-radius: 8px;
      margin-bottom: 20px;
      border-left: 4px solid #1976d2;
    }
    button {
      padding: 8px 16px;
      background: #1976d2;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      margin-right: 10px;
    }
    button:hover { background: #1565c0; }
  </style>
</head>
<body>
  <h1>Mermaid Definition</h1>
  <div class="info">
    <strong>Diagram:</strong> ${title || 'Untitled'}<br>
    <strong>Length:</strong> ${cleanDefinition.length} characters<br>
    <strong>Lines:</strong> ${cleanDefinition.split('\n').length}
  </div>
  <button onclick="copyToClipboard()">Copy to Clipboard</button>
  <button onclick="downloadDefinition()">Download Definition</button>
  <pre id="definition">${cleanDefinition}</pre>
  
  <script>
    function copyToClipboard() {
      const text = document.getElementById('definition').textContent;
      navigator.clipboard.writeText(text).then(() => {
        alert('Definition copied to clipboard!');
      }).catch(err => {
        console.error('Failed to copy: ', err);
        // Fallback
        const textArea = document.createElement('textarea');
        textArea.value = text;
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
        alert('Definition copied to clipboard!');
      });
    }
    
    function downloadDefinition() {
      const text = document.getElementById('definition').textContent;
      const blob = new Blob([text], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = '${title || 'diagram'}-definition.mmd';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  </script>
</body>
</html>`;
    
    win.document.write(html);
    win.document.close();
  };

  const handleOpenStandalone = () => {
    try {
      const cleanDefinition = definition.replace(/\n/g, '\n').trim();
      // Render once to get SVG and open in new window
      mermaid.render(`standalone-${Date.now()}`, cleanDefinition).then((result: any) => {
        const svg = result.svg || result;
        const win = window.open('', '_blank');
        if (!win) return;
        const html = `<!doctype html>
<html>
<head>
  <meta charset=\"utf-8\"/>
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>
  <title>${title || 'Diagram'}</title>
  <script>
    function downloadSVG() {
      const svg = document.querySelector('svg');
      if (!svg) return;
      const serializer = new XMLSerializer();
      let source = serializer.serializeToString(svg);
      if (!source.match(/^<\?xml/)) {
        source = '<?xml version="1.0" standalone="no"?>\r\n' + source;
      }
      const blob = new Blob([source], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = (document.title || 'diagram') + '.svg';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }
  </script>
  <style>
    html, body { height: 100%; margin: 0; }
    body { background: #111; color: #fff; overflow: auto; }
    .wrap { display: flex; align-items: flex-start; justify-content: center; min-height: 100vh; overflow: auto; padding: 16px; box-sizing: border-box; }
    svg { 
      display: block; 
      width: auto; 
      height: auto; 
      max-width: 90vw; 
      min-width: 300px;
      /* Remove max-height constraint to prevent cutoff */
    }
    .controls {
      position: fixed; 
      top: 8px; 
      right: 8px; 
      z-index: 10;
      display: flex;
      gap: 8px;
    }
    .btn {
      padding: 8px 12px; 
      font-family: sans-serif; 
      font-size: 14px; 
      border: none; 
      border-radius: 4px; 
      background: #1976d2; 
      color: #fff; 
      cursor: pointer; 
      box-shadow: 0 2px 6px rgba(0,0,0,.25);
    }
    .btn:hover { background: #1565c0; }
  </style>
</head>
<body>
  <div class=\"wrap\">
    <div class=\"controls\">
      <button class=\"btn\" onclick=\"downloadSVG()\">Download SVG</button>
      <button class=\"btn\" onclick=\"window.print()\">Print</button>
    </div>
    ${svg}
  </div>
</body>
</html>`;
        win.document.write(html);
        win.document.close();
        // Normalize the SVG in the new window
        const svgEl = win.document.querySelector('svg') as SVGElement | null;
        if (svgEl) {
          // Read original width/height before removing
          const origW = parseFloat(svgEl.getAttribute('width') || '0');
          const origH = parseFloat(svgEl.getAttribute('height') || '0');
          // Remove any hardcoded attributes so CSS can scale
          svgEl.removeAttribute('width');
          svgEl.removeAttribute('height');
          // Ensure viewBox
          if (!svgEl.getAttribute('viewBox')) {
            if (origW && origH) {
              svgEl.setAttribute('viewBox', `0 0 ${origW} ${origH}`);
            } else {
              // Try to compute from content bbox
              try {
                const bbox = (svgEl as any).getBBox?.();
                if (bbox && bbox.width && bbox.height) {
                  svgEl.setAttribute('viewBox', `${bbox.x} ${bbox.y} ${bbox.width} ${bbox.height}`);
                }
              } catch {}
            }
          }
          svgEl.setAttribute('preserveAspectRatio', 'xMinYMin meet');
        }
      });
    } catch (e) {
      console.error('Failed to open standalone diagram', e);
    }
  };

  const handleCloseFullscreen = () => {
    setIsFullscreenOpen(false);
  };

  return (
    <>
      <Paper sx={{ p: 2, width: '100%', maxWidth: '100%', position: 'relative' }} className={className}>
        {title && (
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
            <Typography variant="h6">
              {title}
            </Typography>
            <Box>
              <Tooltip title="View definition source">
                <IconButton onClick={handleViewDefinition} size="small" sx={{ mr: 1 }}>
                  <Code />
                </IconButton>
              </Tooltip>
              <Tooltip title="Download SVG">
                <IconButton onClick={handleDownloadSVG} size="small" sx={{ mr: 1 }}>
                  <Download />
                </IconButton>
              </Tooltip>
              <Tooltip title="Open standalone window">
                <IconButton onClick={handleOpenStandalone} size="small" sx={{ mr: 1 }}>
                  <OpenInNew />
                </IconButton>
              </Tooltip>
              <Tooltip title="View fullscreen">
                <IconButton onClick={handleOpenFullscreen} size="small">
                  <Fullscreen />
                </IconButton>
              </Tooltip>
            </Box>
          </Box>
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
          onClick={handleOpenFullscreen}
          sx={{
            width: '100%',
            maxWidth: '100%',
            minHeight: loading ? previewHeight : 'auto',
            textAlign: 'center',
            overflow: 'visible', // Changed from 'hidden' to allow content to show
            cursor: 'pointer',
            position: 'relative',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            padding: '8px', // Add padding to prevent cutoff
            '& svg': {
              width: 'auto !important',
              height: 'auto !important',
              maxWidth: '100%',
              // Remove maxHeight constraint in preview mode
              display: loading ? 'none' : 'block',
              margin: '0 auto',
              // Allow SVG to grow beyond container if needed
              minHeight: '200px',
            },
            '&:hover': {
              backgroundColor: 'action.hover',
            },
          }}
        />
        
        {!loading && !error && (
          <Box sx={{ 
            position: 'absolute', 
            bottom: 8, 
            right: 8, 
            backgroundColor: 'rgba(0,0,0,0.7)', 
            borderRadius: 1,
            p: 0.5
          }}>
            <Typography variant="caption" sx={{ color: 'white', fontSize: '0.7rem' }}>
              Click to expand
            </Typography>
          </Box>
        )}
      </Paper>

      {/* Fullscreen Modal */}
      <Dialog
        open={isFullscreenOpen}
        onClose={handleCloseFullscreen}
        maxWidth={false}
        fullWidth
        PaperProps={{
          sx: {
            width: '95vw',
            height: '95vh',
            maxWidth: 'none',
            maxHeight: 'none',
            m: 1,
          }
        }}
      >
        <DialogTitle sx={{ 
          display: 'flex', 
          justifyContent: 'space-between', 
          alignItems: 'center',
          pb: 1
        }}>
          {title || 'Diagram'}
          <IconButton onClick={handleCloseFullscreen}>
            <Close />
          </IconButton>
        </DialogTitle>
        
        <DialogContent sx={{ 
          p: 2, 
          overflow: 'auto',
          flex: 1,
          minHeight: 'calc(95vh - 140px)',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
        }}>
          {fullscreenLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', gap: 2, flex: 1 }}>
              <CircularProgress />
              <Typography variant="body1">Rendering fullscreen diagram...</Typography>
            </Box>
          )}
          
          <Box
            ref={fullscreenRefCallback}
            sx={{
              width: '100%',
              textAlign: 'center',
              flex: 1,
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'flex-start', // Changed from center to flex-start
              overflow: 'auto',
              padding: '16px',
              '& svg': {
                width: 'auto !important',
                height: 'auto !important',
                maxWidth: '100%',
                // Remove maxHeight constraints in fullscreen
                display: fullscreenLoading ? 'none' : 'block',
                margin: '0 auto',
                minWidth: '300px',
              },
            }}
          />
        </DialogContent>
        
        <DialogActions sx={{ pt: 1 }}>
          <Button onClick={handleCloseFullscreen} variant="contained">Close</Button>
        </DialogActions>
      </Dialog>
    </>
  );
}
