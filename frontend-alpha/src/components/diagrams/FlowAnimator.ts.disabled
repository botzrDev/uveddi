import { gsap } from 'gsap';
import * as d3 from 'd3';

export interface FlowPath {
  id: string;
  sourceNodeId: string;
  targetNodeId: string;
  pathElement: SVGPathElement;
  dataType?: string;
  priority?: number;
}

export interface FlowParticle {
  id: string;
  element: SVGCircleElement;
  pathId: string;
  speed: number;
  size: number;
  color: string;
}

export interface FlowAnimationConfig {
  particleCount: number;
  particleSize: number;
  animationDuration: number;
  staggerDelay: number;
  colors: {
    data: string;
    control: string;
    error: string;
    success: string;
  };
  effects: {
    glow: boolean;
    trail: boolean;
    pulse: boolean;
  };
}

export class FlowAnimator {
  private svgElement: SVGSVGElement;
  private containerGroup: SVGGElement;
  private flows: FlowPath[] = [];
  private particles: FlowParticle[] = [];
  private animationTimeline: gsap.core.Timeline | null = null;
  private isAnimating = false;

  constructor(
    svgElement: SVGSVGElement,
    containerGroup: SVGGElement,
    private config: FlowAnimationConfig = {
      particleCount: 3,
      particleSize: 4,
      animationDuration: 2000,
      staggerDelay: 200,
      colors: {
        data: '#4ecdc4',
        control: '#45b7d1',
        error: '#ff6b6b',
        success: '#96ceb4'
      },
      effects: {
        glow: true,
        trail: false,
        pulse: true
      }
    }
  ) {
    this.svgElement = svgElement;
    this.containerGroup = containerGroup;
    this.initializeFlowSystem();
  }

  private initializeFlowSystem(): void {
    // Create particle layer
    const particleLayer = d3.select(this.containerGroup)
      .append('g')
      .attr('class', 'particle-layer')
      .style('pointer-events', 'none');

    // Add glow filter if enabled
    if (this.config.effects.glow) {
      this.createGlowFilter();
    }
  }

  private createGlowFilter(): void {
    const defs = d3.select(this.svgElement)
      .select('defs')
      .empty() ? 
      d3.select(this.svgElement).append('defs') : 
      d3.select(this.svgElement).select('defs');

    const filter = defs.append('filter')
      .attr('id', 'particle-glow')
      .attr('x', '-50%')
      .attr('y', '-50%')
      .attr('width', '200%')
      .attr('height', '200%');

    filter.append('feGaussianBlur')
      .attr('stdDeviation', '3')
      .attr('result', 'coloredBlur');

    const feMerge = filter.append('feMerge');
    feMerge.append('feMergeNode').attr('in', 'coloredBlur');
    feMerge.append('feMergeNode').attr('in', 'SourceGraphic');
  }

  public identifyFlowPaths(): void {
    const edges = d3.select(this.containerGroup).selectAll('.edge');
    
    this.flows = [];
    edges.each((d: any, i: number, nodes: any[]) => {
      const pathElement = nodes[i].querySelector('path') as SVGPathElement;
      if (pathElement) {
        const flow: FlowPath = {
          id: `flow-${i}`,
          sourceNodeId: d.source || `node-${i}-source`,
          targetNodeId: d.target || `node-${i}-target`,
          pathElement,
          dataType: this.inferDataType(d),
          priority: d.priority || 1
        };
        this.flows.push(flow);
      }
    });
  }

  private inferDataType(edgeData: any): string {
    // Infer data type from edge properties
    if (edgeData.label) {
      const label = edgeData.label.toLowerCase();
      if (label.includes('error') || label.includes('exception')) return 'error';
      if (label.includes('success') || label.includes('complete')) return 'success';
      if (label.includes('control') || label.includes('signal')) return 'control';
    }
    return 'data';
  }

  public startDataFlowAnimation(options: {
    sequential?: boolean;
    loopCount?: number;
    highlightPaths?: boolean;
  } = {}): Promise<void> {
    const { sequential = false, loopCount = 1, highlightPaths = true } = options;

    return new Promise((resolve) => {
      if (this.isAnimating) {
        this.stopAnimation();
      }

      this.isAnimating = true;
      this.animationTimeline = gsap.timeline({
        repeat: loopCount - 1,
        onComplete: () => {
          this.isAnimating = false;
          resolve();
        }
      });

      if (highlightPaths) {
        this.highlightFlowPaths();
      }

      if (sequential) {
        this.animateFlowsSequentially();
      } else {
        this.animateFlowsParallel();
      }
    });
  }

  private highlightFlowPaths(): void {
    this.flows.forEach((flow, index) => {
      gsap.to(flow.pathElement, {
        stroke: this.config.colors[flow.dataType as keyof typeof this.config.colors] || this.config.colors.data,
        strokeWidth: 3,
        duration: 0.3,
        delay: index * 0.1,
        ease: 'power2.out'
      });
    });
  }

  private animateFlowsSequentially(): void {
    this.flows.forEach((flow, index) => {
      this.animationTimeline?.add(
        this.createFlowAnimation(flow),
        index * (this.config.staggerDelay / 1000)
      );
    });
  }

  private animateFlowsParallel(): void {
    this.flows.forEach((flow, index) => {
      this.animationTimeline?.add(
        this.createFlowAnimation(flow),
        index * 0.1 // Small stagger for visual clarity
      );
    });
  }

  private createFlowAnimation(flow: FlowPath): gsap.core.Timeline {
    const flowTimeline = gsap.timeline();
    const particles = this.createParticlesForFlow(flow);

    particles.forEach((particle, index) => {
      const particleAnimation = this.createParticleAnimation(particle, flow);
      flowTimeline.add(particleAnimation, index * 0.2);
    });

    return flowTimeline;
  }

  private createParticlesForFlow(flow: FlowPath): FlowParticle[] {
    const particles: FlowParticle[] = [];
    const particleLayer = d3.select(this.containerGroup).select('.particle-layer');

    for (let i = 0; i < this.config.particleCount; i++) {
      const particleId = `${flow.id}-particle-${i}`;
      const color = this.config.colors[flow.dataType as keyof typeof this.config.colors] || this.config.colors.data;

      const particleElement = particleLayer
        .append('circle')
        .attr('id', particleId)
        .attr('r', this.config.particleSize)
        .attr('fill', color)
        .attr('opacity', 0)
        .style('filter', this.config.effects.glow ? 'url(#particle-glow)' : 'none')
        .node() as SVGCircleElement;

      const particle: FlowParticle = {
        id: particleId,
        element: particleElement,
        pathId: flow.id,
        speed: 1 + (Math.random() * 0.5), // Slight speed variation
        size: this.config.particleSize,
        color
      };

      particles.push(particle);
      this.particles.push(particle);
    }

    return particles;
  }

  private createParticleAnimation(particle: FlowParticle, flow: FlowPath): gsap.core.Timeline {
    const timeline = gsap.timeline();
    const path = flow.pathElement;
    const duration = this.config.animationDuration / 1000 * particle.speed;

    // Fade in
    timeline.to(particle.element, {
      opacity: 1,
      duration: 0.2,
      ease: 'power2.out'
    });

    // Move along path
    timeline.to(particle.element, {
      motionPath: {
        path: path,
        autoRotate: false,
        alignOrigin: [0.5, 0.5]
      },
      duration: duration,
      ease: 'none'
    });

    // Add pulse effect if enabled
    if (this.config.effects.pulse) {
      timeline.to(particle.element, {
        attr: { r: this.config.particleSize * 1.5 },
        duration: 0.1,
        ease: 'power2.out',
        yoyo: true,
        repeat: 1
      }, duration * 0.5);
    }

    // Fade out
    timeline.to(particle.element, {
      opacity: 0,
      duration: 0.2,
      ease: 'power2.in',
      onComplete: () => {
        particle.element.remove();
        this.particles = this.particles.filter(p => p.id !== particle.id);
      }
    });

    return timeline;
  }

  public highlightDataFlow(sourceNodeId: string, targetNodeId: string): void {
    const relevantFlows = this.flows.filter(flow => 
      flow.sourceNodeId === sourceNodeId && flow.targetNodeId === targetNodeId
    );

    relevantFlows.forEach(flow => {
      // Highlight the path
      gsap.to(flow.pathElement, {
        stroke: '#ff6b6b',
        strokeWidth: 4,
        duration: 0.3,
        ease: 'power2.out'
      });

      // Create emphasis particles
      const emphasisParticles = this.createParticlesForFlow(flow);
      emphasisParticles.forEach((particle, index) => {
        particle.size = this.config.particleSize * 1.5;
        particle.element.setAttribute('r', particle.size.toString());
        
        const animation = this.createParticleAnimation(particle, flow);
        animation.delay(index * 0.1);
      });
    });
  }

  public showSequentialExecution(executionOrder: string[]): Promise<void> {
    return new Promise((resolve) => {
      const timeline = gsap.timeline({
        onComplete: resolve
      });

      executionOrder.forEach((nodeId, index) => {
        // Highlight node
        const nodeElement = d3.select(this.containerGroup)
          .select(`[data-node-id="${nodeId}"]`)
          .node();

        if (nodeElement) {
          timeline.to(nodeElement, {
            fill: '#ffd93d',
            duration: 0.5,
            ease: 'power2.out'
          }, index * 0.7);

          timeline.to(nodeElement, {
            fill: '',
            duration: 0.3,
            ease: 'power2.in'
          }, (index * 0.7) + 0.4);
        }

        // Animate outgoing flows
        const outgoingFlows = this.flows.filter(flow => 
          flow.sourceNodeId === nodeId
        );

        outgoingFlows.forEach(flow => {
          const flowAnimation = this.createFlowAnimation(flow);
          timeline.add(flowAnimation, (index * 0.7) + 0.2);
        });
      });
    });
  }

  public createFlowTrail(flow: FlowPath): void {
    if (!this.config.effects.trail) return;

    const path = flow.pathElement;
    const pathLength = path.getTotalLength();
    
    // Create trail effect using stroke-dasharray animation
    gsap.fromTo(path, 
      {
        strokeDasharray: `0 ${pathLength}`,
        strokeOpacity: 0.8
      },
      {
        strokeDasharray: `${pathLength} 0`,
        duration: 1.5,
        ease: 'power2.inOut',
        onComplete: () => {
          gsap.to(path, {
            strokeDasharray: 'none',
            duration: 0.3
          });
        }
      }
    );
  }

  public stopAnimation(): void {
    if (this.animationTimeline) {
      this.animationTimeline.kill();
      this.animationTimeline = null;
    }

    // Clean up particles
    this.particles.forEach(particle => {
      particle.element.remove();
    });
    this.particles = [];

    // Reset flow paths
    this.flows.forEach(flow => {
      gsap.set(flow.pathElement, {
        stroke: '',
        strokeWidth: '',
        strokeDasharray: 'none'
      });
    });

    this.isAnimating = false;
  }

  public pauseAnimation(): void {
    if (this.animationTimeline) {
      this.animationTimeline.pause();
    }
  }

  public resumeAnimation(): void {
    if (this.animationTimeline) {
      this.animationTimeline.resume();
    }
  }

  public updateConfig(newConfig: Partial<FlowAnimationConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }

  public getFlowPaths(): FlowPath[] {
    return [...this.flows];
  }

  public getActiveParticles(): FlowParticle[] {
    return [...this.particles];
  }

  public dispose(): void {
    this.stopAnimation();
    
    // Remove particle layer
    d3.select(this.containerGroup).select('.particle-layer').remove();
    
    // Remove glow filter
    d3.select(this.svgElement).select('#particle-glow').remove();
  }
}

export default FlowAnimator;