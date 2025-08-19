import { Close, Code, Download, Fullscreen, OpenInNew } from '@mui/icons-material';
import { Alert, Box, Button, CircularProgress, Dialog, DialogActions, DialogContent, DialogTitle, IconButton, Paper, Tooltip, Typography, useTheme } from '@mui/material';
import mermaid from 'mermaid';
import { useEffect, useState } from 'react';

interface MermaidDiagramProps {
  definition: string;
  title?: string;
  className?: string;
  previewHeight?: string;
}

export default function MermaidDiagram({ definition, title, className, previewHeight = '600px' }: MermaidDiagramProps) {
  const theme = useTheme();
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
    const isDarkMode = theme.palette.mode === 'dark';
    
    try {
      mermaid.initialize({
        startOnLoad: false,
        theme: isDarkMode ? 'dark' : 'default',
        themeVariables: {
          primaryColor: isDarkMode ? '#64b5f6' : '#1976d2',
          primaryTextColor: isDarkMode ? '#e4e6ea' : '#000',
          primaryBorderColor: isDarkMode ? '#64b5f6' : '#1976d2',
          lineColor: isDarkMode ? '#888' : '#666',
          sectionBkgColor: isDarkMode ? '#2a3441' : '#f5f5f5',
          altSectionBkgColor: isDarkMode ? '#1a1f2e' : '#fff',
          gridColor: isDarkMode ? '#444' : '#e0e0e0',
          secondaryColor: isDarkMode ? '#81c784' : '#006100',
          tertiaryColor: isDarkMode ? '#1a1f2e' : '#fff',
          // Enhanced text readability
          textColor: isDarkMode ? '#e4e6ea' : '#000',
          mainBkg: isDarkMode ? '#2a3441' : '#ffffff',
          secondBkg: isDarkMode ? '#1a1f2e' : '#f8f9fa',
          // Node styling for better visibility
          nodeBkg: isDarkMode ? '#3a4551' : '#ffffff',
          nodeBorder: isDarkMode ? '#64b5f6' : '#1976d2',
        },
        fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
        fontSize: isDarkMode ? 14 : 16, // Slightly smaller font in dark mode for better contrast
        // Global settings to prevent constraints
        useMaxWidth: false,
        wrap: false,
        flowchart: {
          useMaxWidth: false, 
          htmlLabels: true,
          curve: 'basis',
          nodeSpacing: 70, // Increased spacing for better readability
          rankSpacing: 120, // Increased spacing
          padding: 30,
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
      console.log('MermaidDiagram: Mermaid initialized successfully with theme:', isDarkMode ? 'dark' : 'light');
    } catch (initError) {
      console.error('Mermaid initialization failed:', initError);
      setError('Failed to initialize Mermaid');
      setLoading(false);
    }
  }, [theme.palette.mode]); // Re-initialize when theme changes

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
          // Enhanced preview mode: larger size with better readability
          svgElement.style.width = '100%';
          svgElement.style.height = 'auto';
          svgElement.style.maxWidth = '100%';
          svgElement.style.minHeight = '400px'; // Increased minimum height
          // Allow diagrams to be larger for better readability
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

        // Enhanced scaling logic for better readability
        const vb = (svgElement.getAttribute('viewBox') || '').split(' ');
        const vbW = parseFloat(vb[2] || '0');
        const vbH = parseFloat(vb[3] || '0');
        
        if (vbW && vbH) {
          const containerWidth = element.clientWidth;
          const containerHeight = element.clientHeight;
          
          // More generous scaling for better readability
          if (containerWidth > 0 && vbW > containerWidth * 1.2) {
            const scaleX = containerWidth / vbW;
            
            // In preview mode, allow for larger height with increased preview size
            if (!isFullscreen && containerHeight > 0) {
              const maxPreviewHeight = parseInt(previewHeight) || 600; // Increased default
              const scaleY = Math.min(maxPreviewHeight / vbH, 1.2); // Allow slight upscaling
              const scale = Math.min(scaleX, scaleY, 1.1); // Allow slight upscaling for better readability
              
              if (scale < 0.8) {  // Only scale down if significant size reduction needed
                svgElement.style.transformOrigin = 'top center';
                svgElement.style.transform = `scale(${scale})`;
                // Adjust container to fit scaled content with padding
                element.style.height = `${Math.max(vbH * scale, 400)}px`;
              } else {
                // Ensure minimum readable size
                element.style.minHeight = '400px';
              }
            } else if (!isFullscreen) {
              // Scale to fit width in preview but maintain readability
              const finalScale = Math.max(Math.min(scaleX, 1), 0.7); // Don't scale below 70%
              svgElement.style.transformOrigin = 'top center';
              svgElement.style.transform = `scale(${finalScale})`;
            }
          } else {
            svgElement.style.transform = '';
            element.style.height = 'auto';
            element.style.minHeight = isFullscreen ? 'auto' : '400px';
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
            minHeight: loading ? previewHeight : '400px', // Increased minimum height
            textAlign: 'center',
            overflow: 'visible', // Allow content to show fully
            cursor: 'pointer',
            position: 'relative',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            padding: { xs: 2, sm: 3 }, // Responsive padding to prevent cutoff
            backgroundColor: (theme) => theme.palette.mode === 'light' 
              ? 'rgba(248, 250, 252, 0.3)' 
              : 'rgba(26, 31, 46, 0.3)',
            borderRadius: 2,
            border: (theme) => `1px dashed ${theme.palette.divider}`,
            transition: 'all 0.3s ease',
            '& svg': {
              width: 'auto !important',
              height: 'auto !important',
              maxWidth: '100%',
              // Allow full height for better readability
              display: loading ? 'none' : 'block',
              margin: '0 auto',
              // Enhanced minimum size for readability
              minHeight: '300px',
              // Ensure text is readable in both themes
              '& text': {
                fill: (theme) => theme.palette.text.primary,
                fontSize: '14px !important',
                fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif !important',
              },
              // Style diagram elements for better theme compatibility
              '& .node rect, & .node circle, & .node polygon': {
                fill: (theme) => theme.palette.mode === 'light' 
                  ? '#ffffff' 
                  : '#3a4551',
                stroke: (theme) => theme.palette.mode === 'light' 
                  ? '#1976d2' 
                  : '#64b5f6',
                strokeWidth: '2px',
              },
              '& .edgePath path': {
                stroke: (theme) => theme.palette.mode === 'light' 
                  ? '#666' 
                  : '#888',
                strokeWidth: '2px',
              },
            },
            '&:hover': {
              backgroundColor: (theme) => theme.palette.mode === 'light' 
                ? 'rgba(25, 118, 210, 0.04)' 
                : 'rgba(100, 181, 246, 0.08)',
              borderColor: (theme) => theme.palette.primary.main,
              transform: 'scale(1.01)', // Subtle scale effect
            },
          }}
        />
        
        {!loading && !error && (
          <Box sx={{ 
            position: 'absolute', 
            bottom: 12, 
            right: 12, 
            backgroundColor: (theme) => theme.palette.mode === 'light'
              ? 'rgba(0, 0, 0, 0.8)'
              : 'rgba(255, 255, 255, 0.9)',
            borderRadius: 2,
            px: 2,
            py: 1,
            transition: 'all 0.3s ease',
            '&:hover': {
              backgroundColor: (theme) => theme.palette.mode === 'light'
                ? 'rgba(25, 118, 210, 0.9)'
                : 'rgba(100, 181, 246, 0.9)',
            }
          }}>
            <Typography 
              variant="caption" 
              sx={{ 
                color: (theme) => theme.palette.mode === 'light'
                  ? 'white'
                  : 'black',
                fontSize: '0.75rem',
                fontWeight: 600,
                display: 'flex',
                alignItems: 'center',
                gap: 0.5,
              }}
            >
              🔍 Click to expand
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
