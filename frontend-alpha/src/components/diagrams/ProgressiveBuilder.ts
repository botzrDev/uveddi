// Minimal stub for ProgressiveBuilder
export interface ProgressiveBuildConfig {
  buildDirection: string;
  animationDuration: number;
  stepDelay: number;
  showLabels: boolean;
}

export class ProgressiveBuilder {
  constructor(_svgElement: SVGSVGElement, _containerGroup: SVGGElement, _config?: ProgressiveBuildConfig) {}
  
  startBuild(): Promise<void> {
    return Promise.resolve();
  }
}