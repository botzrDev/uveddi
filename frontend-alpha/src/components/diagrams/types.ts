// Types for Interactive Diagram System - UV-89 Phases 1 & 2

import type { ComponentMetrics } from '../../types/api';
import type { AnimationConfig } from './AnimationController';
import type { DiagramTheme } from './ThemeManager';

export interface NodeEventData {
  nodeId: string;
  nodeType: string;
  componentData: {
    name: string;
    filePath: string;
    componentType: string;
    metrics?: ComponentMetrics;
  };
  position: { x: number; y: number };
  connections: string[];
}

export interface InteractiveDiagramProps {
  mermaidCode: string;
  onNodeClick?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeDoubleClick?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeHover?: (nodeId: string, nodeData: NodeEventData) => void;
  onNodeContextMenu?: (nodeId: string, nodeData: NodeEventData, event: MouseEvent) => void;
  enablePanZoom?: boolean;
  enableTooltips?: boolean;
  enableKeyboardNav?: boolean;
  enableAnimations?: boolean;
  enableAdvancedStyling?: boolean;
  enableExport?: boolean;
  animationConfig?: AnimationConfig;
  initialTheme?: DiagramTheme;
  className?: string;
}

export interface ContextMenuProps {
  nodeId: string;
  nodeData: NodeEventData;
  position: { x: number; y: number };
  onClose: () => void;
  onAction: (action: string, nodeId: string) => void;
}

export interface TooltipData {
  title: string;
  content: string;
  position: { x: number; y: number };
}

export interface InteractionConfig {
  enableDoubleClick: boolean;
  enableContextMenu: boolean;
  enableTooltips: boolean;
  enablePanZoom: boolean;
  enableKeyboardNav: boolean;
  doubleClickDelay: number;
  tooltipDelay: number;
}

export interface DiagramState {
  selectedNodeId: string | null;
  hoveredNodeId: string | null;
  contextMenuVisible: boolean;
  contextMenuPosition: { x: number; y: number } | null;
  tooltipVisible: boolean;
  tooltipData: TooltipData | null;
  zoomLevel: number;
  panPosition: { x: number; y: number };
}

export interface ContextMenuItem {
  id: string;
  label: string;
  icon: string;
  disabled?: boolean;
  separator?: boolean;
}

export interface KeyboardNavigationState {
  selectedNodeIndex: number;
  nodes: NodeEventData[];
  focusVisible: boolean;
}