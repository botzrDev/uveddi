// Minimal stub for ThemeManager
export interface DiagramTheme {
  id: string;
  name: string;
}

export interface NodeThemeStyle {
  fill: string;
  stroke: string;
  strokeWidth: number;
  strokeDasharray?: string;
  opacity: number;
  textColor: string;
  fontSize: string;
  fontWeight: string;
  borderRadius: number;
}

export class ThemeManager {
  constructor() {}
  
  applyTheme(_theme: DiagramTheme): void {}
  getCurrentTheme(): DiagramTheme {
    return { id: 'default', name: 'Default' };
  }
}