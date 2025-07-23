import React, { useEffect, useRef, useState } from 'react';
import { gsap } from 'gsap';
import * as d3 from 'd3';

export interface AnimationConfig {
  enableFlowAnimations: boolean;
  enableProgressiveBuilding: boolean;
  enableTransitions: boolean;
  animationSpeed: number; // 0.5x to 2x speed
  autoPlay: boolean;
}

export interface FlowAnimationOptions {
  dataFlowVisualization: boolean;
  pathHighlighting: boolean;
  sequentialExecution: boolean;
  particleEffect: boolean;
}

export interface ProgressiveBuildingOptions {
  nodeByNodeReveal: boolean;
  layeredConstruction: boolean;
  storyboardMode: boolean;
  buildDirection: 'top-down' | 'left-right' | 'center-out';
}

export interface TransitionOptions {
  smoothLayoutChanges: boolean;
  morphingDiagrams: boolean;
  fadeInOut: boolean;
  duration: number;
}

export interface AnimationControllerProps {
  svgRef: React.RefObject<SVGSVGElement>;
  nodeData: any[];
  edgeData: any[];
  config: AnimationConfig;
  flowOptions?: FlowAnimationOptions;
  progressiveOptions?: ProgressiveBuildingOptions;
  transitionOptions?: TransitionOptions;
  onAnimationComplete?: (type: string) => void;
  onAnimationStart?: (type: string) => void;
}

export const AnimationController: React.FC<AnimationControllerProps> = ({
  svgRef,
  nodeData,
  edgeData,
  config,
  flowOptions = {
    dataFlowVisualization: true,
    pathHighlighting: true,
    sequentialExecution: false,
    particleEffect: true
  },
  progressiveOptions = {
    nodeByNodeReveal: true,
    layeredConstruction: false,
    storyboardMode: false,
    buildDirection: 'top-down'
  },
  transitionOptions = {
    smoothLayoutChanges: true,
    morphingDiagrams: false,
    fadeInOut: true,
    duration: 1000
  },
  onAnimationComplete,
  onAnimationStart
}) => {
  const [isPlaying, setIsPlaying] = useState(config.autoPlay);
  const [currentAnimation, setCurrentAnimation] = useState<string | null>(null);
  const timelineRef = useRef<gsap.core.Timeline | null>(null);
  const animationStateRef = useRef({
    currentStep: 0,
    totalSteps: 0,
    isInitialized: false
  });

  // Initialize GSAP timeline
  useEffect(() => {
    if (!svgRef.current) return;

    timelineRef.current = gsap.timeline({
      paused: !config.autoPlay,
      onStart: () => {
        setCurrentAnimation('timeline');
        onAnimationStart?.('timeline');
      },
      onComplete: () => {
        setCurrentAnimation(null);
        onAnimationComplete?.('timeline');
      }
    });

    return () => {
      if (timelineRef.current) {
        timelineRef.current.kill();
      }
    };
  }, [svgRef.current, config.autoPlay]);

  // Flow Animation Implementation
  const startFlowAnimation = async () => {
    if (!svgRef.current || !flowOptions.dataFlowVisualization) return;

    setCurrentAnimation('flow');
    onAnimationStart?.('flow');

    const svg = d3.select(svgRef.current);
    
    if (flowOptions.pathHighlighting) {
      // Highlight paths sequentially
      const edges = svg.selectAll('.edge').nodes();
      
      for (let i = 0; i < edges.length; i++) {
        const edge = edges[i];
        
        gsap.to(edge, {
          stroke: '#ff6b6b',
          strokeWidth: 3,
          duration: 0.3 * config.animationSpeed,
          ease: 'power2.inOut'
        });

        if (flowOptions.sequentialExecution) {
          await new Promise(resolve => setTimeout(resolve, 200 * config.animationSpeed));
        }
      }
    }

    if (flowOptions.particleEffect) {
      await animateDataParticles();
    }

    setCurrentAnimation(null);
    onAnimationComplete?.('flow');
  };

  // Particle animation along edges
  const animateDataParticles = async () => {
    if (!svgRef.current) return;

    const svg = d3.select(svgRef.current);
    const edges = svg.selectAll('.edge').nodes();

    edges.forEach((edge: any) => {
      const pathLength = edge.getTotalLength();
      
      // Create particle
      const particle = svg.append('circle')
        .attr('r', 3)
        .attr('fill', '#4ecdc4')
        .attr('opacity', 0);

      // Animate particle along path
      gsap.timeline()
        .to(particle.node(), {
          opacity: 1,
          duration: 0.2 * config.animationSpeed
        })
        .to(particle.node(), {
          motionPath: {
            path: edge,
            autoRotate: false
          },
          duration: 2 * config.animationSpeed,
          ease: 'none'
        })
        .to(particle.node(), {
          opacity: 0,
          duration: 0.2 * config.animationSpeed,
          onComplete: () => particle.remove()
        });
    });
  };

  // Progressive Building Animation
  const startProgressiveBuilding = async () => {
    if (!svgRef.current || !progressiveOptions.nodeByNodeReveal) return;

    setCurrentAnimation('progressive');
    onAnimationStart?.('progressive');

    const svg = d3.select(svgRef.current);
    const nodes = svg.selectAll('.node').nodes();
    const edges = svg.selectAll('.edge').nodes();

    // Hide all elements initially
    gsap.set([...nodes, ...edges], { opacity: 0, scale: 0 });

    if (progressiveOptions.layeredConstruction) {
      await animateByLayers(nodes, edges);
    } else if (progressiveOptions.nodeByNodeReveal) {
      await animateNodeByNode(nodes, edges);
    }

    setCurrentAnimation(null);
    onAnimationComplete?.('progressive');
  };

  // Animate nodes layer by layer
  const animateByLayers = async (nodes: any[], edges: any[]) => {
    // Group nodes by their hierarchical level
    const layers = groupNodesByLevel(nodes);
    
    for (let i = 0; i < layers.length; i++) {
      const layerNodes = layers[i];
      
      // Animate layer nodes simultaneously
      gsap.to(layerNodes, {
        opacity: 1,
        scale: 1,
        duration: 0.5 * config.animationSpeed,
        stagger: 0.1 * config.animationSpeed,
        ease: 'back.out(1.7)'
      });

      // Add connecting edges after nodes appear
      if (i > 0) {
        const relevantEdges = getEdgesForLayer(edges, layerNodes);
        gsap.to(relevantEdges, {
          opacity: 1,
          duration: 0.3 * config.animationSpeed,
          delay: 0.2 * config.animationSpeed
        });
      }

      await new Promise(resolve => 
        setTimeout(resolve, 800 * config.animationSpeed)
      );
    }
  };

  // Animate nodes one by one
  const animateNodeByNode = async (nodes: any[], edges: any[]) => {
    const orderedNodes = orderNodesForReveal(nodes);
    
    for (let i = 0; i < orderedNodes.length; i++) {
      const node = orderedNodes[i];
      
      // Animate node appearance
      gsap.to(node, {
        opacity: 1,
        scale: 1,
        duration: 0.4 * config.animationSpeed,
        ease: 'back.out(1.7)'
      });

      // Animate connected edges
      const connectedEdges = getConnectedEdges(edges, node);
      gsap.to(connectedEdges, {
        opacity: 1,
        duration: 0.2 * config.animationSpeed,
        delay: 0.1 * config.animationSpeed
      });

      await new Promise(resolve => 
        setTimeout(resolve, 300 * config.animationSpeed)
      );
    }
  };

  // Smooth Transition Animations
  const startTransition = (fromLayout: any, toLayout: any) => {
    if (!svgRef.current || !transitionOptions.smoothLayoutChanges) return;

    setCurrentAnimation('transition');
    onAnimationStart?.('transition');

    const svg = d3.select(svgRef.current);
    const nodes = svg.selectAll('.node');

    if (transitionOptions.morphingDiagrams) {
      morphDiagram(nodes, fromLayout, toLayout);
    } else if (transitionOptions.fadeInOut) {
      fadeTransition(nodes, toLayout);
    }
  };

  // Morph diagram smoothly between layouts
  const morphDiagram = (nodes: any, fromLayout: any, toLayout: any) => {
    nodes.each(function(d: any) {
      const node = d3.select(this);
      const newPosition = toLayout[d.id];
      
      if (newPosition) {
        gsap.to(this, {
          x: newPosition.x,
          y: newPosition.y,
          duration: transitionOptions.duration / 1000 * config.animationSpeed,
          ease: 'power2.inOut'
        });
      }
    });

    setTimeout(() => {
      setCurrentAnimation(null);
      onAnimationComplete?.('transition');
    }, transitionOptions.duration * config.animationSpeed);
  };

  // Fade out old, fade in new
  const fadeTransition = (nodes: any, newLayout: any) => {
    gsap.timeline()
      .to(nodes.nodes(), {
        opacity: 0,
        duration: 0.3 * config.animationSpeed
      })
      .call(() => {
        // Update positions
        nodes.each(function(d: any) {
          const newPosition = newLayout[d.id];
          if (newPosition) {
            d3.select(this)
              .attr('transform', `translate(${newPosition.x}, ${newPosition.y})`);
          }
        });
      })
      .to(nodes.nodes(), {
        opacity: 1,
        duration: 0.3 * config.animationSpeed,
        onComplete: () => {
          setCurrentAnimation(null);
          onAnimationComplete?.('transition');
        }
      });
  };

  // Helper functions
  const groupNodesByLevel = (nodes: any[]): any[][] => {
    // Implementation to group nodes by hierarchical level
    // This would analyze the graph structure to determine layers
    const levels: any[][] = [];
    // Simplified implementation - in real case, would use graph analysis
    const nodesPerLevel = Math.ceil(nodes.length / 3);
    for (let i = 0; i < nodes.length; i += nodesPerLevel) {
      levels.push(nodes.slice(i, i + nodesPerLevel));
    }
    return levels;
  };

  const getEdgesForLayer = (edges: any[], layerNodes: any[]): any[] => {
    // Implementation to find edges connecting to current layer
    return edges.filter((edge: any) => {
      // Simplified - would check if edge connects to any node in layer
      return Math.random() > 0.5; // Placeholder
    });
  };

  const orderNodesForReveal = (nodes: any[]): any[] => {
    // Order nodes based on build direction
    switch (progressiveOptions.buildDirection) {
      case 'top-down':
        return nodes.sort((a: any, b: any) => a.y - b.y);
      case 'left-right':
        return nodes.sort((a: any, b: any) => a.x - b.x);
      case 'center-out':
        // Calculate distance from center and sort
        const centerX = nodes.reduce((sum, n) => sum + n.x, 0) / nodes.length;
        const centerY = nodes.reduce((sum, n) => sum + n.y, 0) / nodes.length;
        return nodes.sort((a: any, b: any) => {
          const distA = Math.sqrt((a.x - centerX) ** 2 + (a.y - centerY) ** 2);
          const distB = Math.sqrt((b.x - centerX) ** 2 + (b.y - centerY) ** 2);
          return distA - distB;
        });
      default:
        return nodes;
    }
  };

  const getConnectedEdges = (edges: any[], node: any): any[] => {
    // Find edges connected to the given node
    return edges.filter((edge: any) => {
      // Simplified - would check actual edge connections
      return Math.random() > 0.7; // Placeholder
    });
  };

  // Control functions
  const playAnimation = () => {
    if (timelineRef.current) {
      timelineRef.current.play();
      setIsPlaying(true);
    }
  };

  const pauseAnimation = () => {
    if (timelineRef.current) {
      timelineRef.current.pause();
      setIsPlaying(false);
    }
  };

  const resetAnimation = () => {
    if (timelineRef.current) {
      timelineRef.current.restart();
      setIsPlaying(config.autoPlay);
    }
  };

  const setAnimationSpeed = (speed: number) => {
    if (timelineRef.current) {
      timelineRef.current.timeScale(speed);
    }
  };

  // Animation Control UI
  return (
    <div className="animation-controller absolute top-4 left-4 bg-white/90 backdrop-blur-sm rounded-lg p-3 shadow-lg">
      <div className="flex items-center gap-2 mb-2">
        <h3 className="text-sm font-semibold text-gray-700">Animation Controls</h3>
        {currentAnimation && (
          <span className="text-xs bg-blue-100 text-blue-600 px-2 py-1 rounded">
            {currentAnimation}
          </span>
        )}
      </div>
      
      <div className="flex gap-2 mb-3">
        <button
          onClick={isPlaying ? pauseAnimation : playAnimation}
          className="px-3 py-1 bg-blue-500 text-white rounded text-xs hover:bg-blue-600"
          disabled={!!currentAnimation}
        >
          {isPlaying ? '⏸️' : '▶️'}
        </button>
        
        <button
          onClick={resetAnimation}
          className="px-3 py-1 bg-gray-500 text-white rounded text-xs hover:bg-gray-600"
          disabled={!!currentAnimation}
        >
          🔄
        </button>
      </div>

      <div className="space-y-2">
        <button
          onClick={startFlowAnimation}
          disabled={!!currentAnimation || !config.enableFlowAnimations}
          className="w-full px-2 py-1 bg-green-500 text-white rounded text-xs hover:bg-green-600 disabled:opacity-50"
        >
          Flow Animation
        </button>
        
        <button
          onClick={startProgressiveBuilding}
          disabled={!!currentAnimation || !config.enableProgressiveBuilding}
          className="w-full px-2 py-1 bg-purple-500 text-white rounded text-xs hover:bg-purple-600 disabled:opacity-50"
        >
          Progressive Build
        </button>
      </div>

      <div className="mt-3">
        <label className="text-xs text-gray-600">Speed: {config.animationSpeed}x</label>
        <input
          type="range"
          min="0.5"
          max="2"
          step="0.1"
          value={config.animationSpeed}
          onChange={(e) => setAnimationSpeed(Number(e.target.value))}
          className="w-full h-1 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
      </div>
    </div>
  );
};

export default AnimationController;