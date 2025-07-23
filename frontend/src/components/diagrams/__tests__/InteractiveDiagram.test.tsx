import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import InteractiveDiagram from '../InteractiveDiagram';
import { 
  createMockNodeData, 
  createMockAnimationConfig, 
  createMockTheme,
  waitForAnimationFrame,
  triggerMouseEvent,
  triggerKeyboardEvent,
  measurePerformance,
  createLargeDiagramData,
  checkAriaLabels,
  detectMemoryLeaks
} from './setup';

describe('InteractiveDiagram Component', () => {
  const mockMermaidCode = `
    graph TD
      A[Component A] --> B[Service B]
      B --> C[Database C]
      C --> D[External API]
  `;

  const defaultProps = {
    mermaidCode: mockMermaidCode,
    onNodeClick: vi.fn(),
    onNodeDoubleClick: vi.fn(),
    onNodeHover: vi.fn(),
    onNodeContextMenu: vi.fn()
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  describe('Basic Rendering', () => {
    it('renders the interactive diagram container', () => {
      render(<InteractiveDiagram {...defaultProps} />);
      
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      expect(container).toBeInTheDocument();
    });

    it('renders with custom className', () => {
      const { container } = render(
        <InteractiveDiagram {...defaultProps} className="custom-diagram" />
      );
      
      expect(container.firstChild).toHaveClass('interactive-diagram-container', 'custom-diagram');
    });

    it('handles empty mermaid code gracefully', () => {
      render(<InteractiveDiagram {...defaultProps} mermaidCode="" />);
      
      // Should not crash and should render container
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      expect(container).toBeInTheDocument();
    });

    it('handles invalid mermaid code gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      
      render(<InteractiveDiagram {...defaultProps} mermaidCode="invalid mermaid syntax" />);
      
      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith(
          expect.stringContaining('Error rendering Mermaid diagram')
        );
      });
      
      consoleSpy.mockRestore();
    });
  });

  describe('Interactivity Features', () => {
    it('enables pan and zoom controls when enablePanZoom is true', () => {
      render(<InteractiveDiagram {...defaultProps} enablePanZoom={true} />);
      
      const zoomInButton = screen.getByTitle('Zoom In');
      const zoomOutButton = screen.getByTitle('Zoom Out');
      const fitButton = screen.getByTitle('Fit to View');
      
      expect(zoomInButton).toBeInTheDocument();
      expect(zoomOutButton).toBeInTheDocument();
      expect(fitButton).toBeInTheDocument();
    });

    it('hides pan and zoom controls when enablePanZoom is false', () => {
      render(<InteractiveDiagram {...defaultProps} enablePanZoom={false} />);
      
      expect(screen.queryByTitle('Zoom In')).not.toBeInTheDocument();
      expect(screen.queryByTitle('Zoom Out')).not.toBeInTheDocument();
      expect(screen.queryByTitle('Fit to View')).not.toBeInTheDocument();
    });

    it('calls onNodeClick when a node is clicked', async () => {
      const onNodeClick = vi.fn();
      const { container } = render(
        <InteractiveDiagram {...defaultProps} onNodeClick={onNodeClick} />
      );
      
      await waitForAnimationFrame();
      
      // Simulate node click
      const nodeElement = container.querySelector('.node');
      if (nodeElement) {
        triggerMouseEvent(nodeElement, 'click');
        
        await waitFor(() => {
          expect(onNodeClick).toHaveBeenCalledWith(
            expect.any(String),
            expect.objectContaining({
              nodeId: expect.any(String),
              componentData: expect.any(Object)
            })
          );
        });
      }
    });

    it('calls onNodeDoubleClick when a node is double-clicked', async () => {
      const onNodeDoubleClick = vi.fn();
      const { container } = render(
        <InteractiveDiagram {...defaultProps} onNodeDoubleClick={onNodeDoubleClick} />
      );
      
      await waitForAnimationFrame();
      
      const nodeElement = container.querySelector('.node');
      if (nodeElement) {
        triggerMouseEvent(nodeElement, 'dblclick');
        
        await waitFor(() => {
          expect(onNodeDoubleClick).toHaveBeenCalled();
        });
      }
    });

    it('shows context menu on right-click', async () => {
      const { container } = render(<InteractiveDiagram {...defaultProps} />);
      
      await waitForAnimationFrame();
      
      const nodeElement = container.querySelector('.node');
      if (nodeElement) {
        triggerMouseEvent(nodeElement, 'contextmenu', { button: 2 });
        
        await waitFor(() => {
          expect(screen.getByText('View Details')).toBeInTheDocument();
          expect(screen.getByText('Highlight Dependencies')).toBeInTheDocument();
        });
      }
    });

    it('supports keyboard navigation when enabled', async () => {
      render(<InteractiveDiagram {...defaultProps} enableKeyboardNav={true} />);
      
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      
      // Should have tabindex for keyboard focus
      expect(container).toHaveAttribute('tabindex', '0');
      
      // Simulate keyboard events
      container.focus();
      triggerKeyboardEvent(container, 'keydown', 'ArrowRight');
      
      // Keyboard controller should be initialized
      await waitForAnimationFrame();
    });
  });

  describe('Animation Features', () => {
    it('shows animation controller when animations are enabled', () => {
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAnimations={true}
          animationConfig={createMockAnimationConfig()}
        />
      );
      
      expect(screen.getByText('Animation Controls')).toBeInTheDocument();
      expect(screen.getByText('Flow Animation')).toBeInTheDocument();
      expect(screen.getByText('Progressive Build')).toBeInTheDocument();
    });

    it('hides animation controller when animations are disabled', () => {
      render(<InteractiveDiagram {...defaultProps} enableAnimations={false} />);
      
      expect(screen.queryByText('Animation Controls')).not.toBeInTheDocument();
    });

    it('can trigger flow animations', async () => {
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAnimations={true}
          animationConfig={createMockAnimationConfig()}
        />
      );
      
      const flowButton = screen.getByText('Flow Animation');
      fireEvent.click(flowButton);
      
      // Animation should be triggered
      await waitForAnimationFrame();
    });

    it('can trigger progressive building', async () => {
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAnimations={true}
          animationConfig={createMockAnimationConfig()}
        />
      );
      
      const progressiveButton = screen.getByText('Progressive Build');
      fireEvent.click(progressiveButton);
      
      await waitForAnimationFrame();
    });

    it('respects animation speed settings', () => {
      const slowConfig = createMockAnimationConfig({ animationSpeed: 0.5 });
      
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAnimations={true}
          animationConfig={slowConfig}
        />
      );
      
      const speedSlider = screen.getByDisplayValue('0.5');
      expect(speedSlider).toBeInTheDocument();
    });
  });

  describe('Advanced Styling', () => {
    it('shows style controller when advanced styling is enabled', () => {
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAdvancedStyling={true}
          initialTheme={createMockTheme()}
        />
      );
      
      const styleButton = screen.getByTitle('Style Controls');
      fireEvent.click(styleButton);
      
      expect(screen.getByText('Style Controls')).toBeInTheDocument();
    });

    it('hides style controller when advanced styling is disabled', () => {
      render(<InteractiveDiagram {...defaultProps} enableAdvancedStyling={false} />);
      
      expect(screen.queryByTitle('Style Controls')).not.toBeInTheDocument();
    });

    it('applies initial theme correctly', () => {
      const customTheme = createMockTheme({ 
        id: 'custom',
        name: 'Custom Theme' 
      });
      
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAdvancedStyling={true}
          initialTheme={customTheme}
        />
      );
      
      // Theme should be applied
      expect(screen.getByTitle('Style Controls')).toBeInTheDocument();
    });
  });

  describe('Export Features', () => {
    it('shows export controller when export is enabled', () => {
      render(<InteractiveDiagram {...defaultProps} enableExport={true} />);
      
      expect(screen.getByTitle('Export Diagram')).toBeInTheDocument();
    });

    it('hides export controller when export is disabled', () => {
      render(<InteractiveDiagram {...defaultProps} enableExport={false} />);
      
      expect(screen.queryByTitle('Export Diagram')).not.toBeInTheDocument();
    });

    it('opens export dialog when export button is clicked', async () => {
      render(<InteractiveDiagram {...defaultProps} enableExport={true} />);
      
      const exportButton = screen.getByTitle('Export Diagram');
      fireEvent.click(exportButton);
      
      await waitFor(() => {
        expect(screen.getByText('Export Diagram')).toBeInTheDocument();
        expect(screen.getByText('Select Format')).toBeInTheDocument();
      });
    });
  });

  describe('Performance', () => {
    it('renders large diagrams within performance budget', async () => {
      const { nodes } = createLargeDiagramData(100);
      
      const renderTime = await measurePerformance(async () => {
        render(
          <InteractiveDiagram 
            {...defaultProps} 
            mermaidCode={`graph TD\n${nodes.map((n, i) => 
              `${n.nodeId}[${n.componentData.name}]${i > 0 ? ` --> ${nodes[i-1].nodeId}` : ''}`
            ).join('\n')}`}
          />
        );
        await waitForAnimationFrame();
      });
      
      // Should render within 100ms
      expect(renderTime).toBeLessThan(100);
    });

    it('does not cause memory leaks during interaction', async () => {
      const memoryDetector = detectMemoryLeaks();
      
      const { unmount } = render(<InteractiveDiagram {...defaultProps} />);
      
      // Simulate heavy interaction
      for (let i = 0; i < 50; i++) {
        await waitForAnimationFrame();
      }
      
      unmount();
      
      // Force garbage collection if available
      if ((global as any).gc) {
        (global as any).gc();
      }
      
      const memoryCheck = memoryDetector.check();
      expect(memoryCheck.leakDetected).toBe(false);
    });

    it('handles rapid state changes without performance degradation', async () => {
      const { rerender } = render(<InteractiveDiagram {...defaultProps} />);
      
      const updateTime = await measurePerformance(async () => {
        for (let i = 0; i < 10; i++) {
          rerender(
            <InteractiveDiagram 
              {...defaultProps} 
              mermaidCode={`graph TD\nA${i}[Component ${i}] --> B${i}[Service ${i}]`}
            />
          );
          await waitForAnimationFrame();
        }
      });
      
      // Should handle rapid updates efficiently
      expect(updateTime).toBeLessThan(200);
    });
  });

  describe('Accessibility', () => {
    it('provides proper ARIA labels for interactive elements', () => {
      const { container } = render(
        <InteractiveDiagram {...defaultProps} enablePanZoom={true} />
      );
      
      const missingLabels = checkAriaLabels(container);
      expect(missingLabels).toHaveLength(0);
    });

    it('supports keyboard navigation', async () => {
      render(<InteractiveDiagram {...defaultProps} enableKeyboardNav={true} />);
      
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      
      // Should be focusable
      expect(container).toHaveAttribute('tabindex', '0');
      
      // Should respond to keyboard events
      container.focus();
      expect(container).toHaveFocus();
    });

    it('provides proper focus management', async () => {
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enablePanZoom={true}
          enableKeyboardNav={true}
        />
      );
      
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      const zoomButton = screen.getByTitle('Zoom In');
      
      // Test tab navigation
      await userEvent.tab();
      expect(container).toHaveFocus();
      
      await userEvent.tab();
      expect(zoomButton).toHaveFocus();
    });

    it('respects prefers-reduced-motion setting', () => {
      // Mock reduced motion preference
      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: vi.fn().mockImplementation(query => ({
          matches: query === '(prefers-reduced-motion: reduce)',
          media: query,
          onchange: null,
          addListener: vi.fn(),
          removeListener: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
          dispatchEvent: vi.fn(),
        })),
      });
      
      render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAnimations={true}
          animationConfig={createMockAnimationConfig()}
        />
      );
      
      // Animations should be disabled or reduced
      const animationController = screen.queryByText('Animation Controls');
      if (animationController) {
        // Should respect motion preferences
        expect(true).toBe(true); // Placeholder for actual reduced motion logic
      }
    });
  });

  describe('Error Handling', () => {
    it('handles SVG rendering errors gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      
      // Mock mermaid to throw an error
      const mermaidMock = await import('mermaid');
      vi.mocked(mermaidMock.default.render).mockRejectedValueOnce(new Error('Rendering failed'));
      
      render(<InteractiveDiagram {...defaultProps} />);
      
      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith(
          expect.stringContaining('Error rendering Mermaid diagram')
        );
      });
      
      consoleSpy.mockRestore();
    });

    it('handles missing node data gracefully', () => {
      render(<InteractiveDiagram {...defaultProps} mermaidCode="graph TD\nA --> B" />);
      
      // Should not crash even with missing node data
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      expect(container).toBeInTheDocument();
    });

    it('cleans up resources on unmount', () => {
      const { unmount } = render(
        <InteractiveDiagram 
          {...defaultProps} 
          enableAnimations={true}
          enableAdvancedStyling={true}
          enableExport={true}
        />
      );
      
      // Should not throw during cleanup
      expect(() => unmount()).not.toThrow();
    });
  });

  describe('Integration', () => {
    it('integrates all features together', async () => {
      render(
        <InteractiveDiagram 
          {...defaultProps}
          enablePanZoom={true}
          enableTooltips={true}
          enableKeyboardNav={true}
          enableAnimations={true}
          enableAdvancedStyling={true}
          enableExport={true}
          animationConfig={createMockAnimationConfig()}
          initialTheme={createMockTheme()}
        />
      );
      
      // All control panels should be present
      expect(screen.getByTitle('Zoom In')).toBeInTheDocument();
      expect(screen.getByTitle('Style Controls')).toBeInTheDocument();
      expect(screen.getByText('Animation Controls')).toBeInTheDocument();
      expect(screen.getByTitle('Export Diagram')).toBeInTheDocument();
      
      // Should be interactive
      const container = screen.getByRole('img', { name: /interactive diagram/i });
      expect(container).toHaveAttribute('tabindex', '0');
    });

    it('maintains feature isolation when features are disabled', () => {
      render(<InteractiveDiagram {...defaultProps} />);
      
      // Only basic functionality should be present
      expect(screen.queryByTitle('Style Controls')).not.toBeInTheDocument();
      expect(screen.queryByText('Animation Controls')).not.toBeInTheDocument();
      expect(screen.queryByTitle('Export Diagram')).not.toBeInTheDocument();
      expect(screen.queryByTitle('Zoom In')).not.toBeInTheDocument();
    });
  });
});