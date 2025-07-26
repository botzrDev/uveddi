// Keyboard Controller for Interactive Diagrams - UV-89 Phase 1
// Provides keyboard navigation and accessibility support

import type { NodeEventData, KeyboardNavigationState } from './types';

export class KeyboardController {
  private nodes: NodeEventData[];
  private onNodeSelect: (nodeId: string) => void;
  private state: KeyboardNavigationState;
  // private containerElement: Element | null = null; // Unused for now

  constructor(nodes: NodeEventData[], onNodeSelect: (nodeId: string) => void) {
    this.nodes = nodes;
    this.onNodeSelect = onNodeSelect;
    this.state = {
      selectedNodeIndex: -1,
      nodes: nodes,
      focusVisible: false,
    };
    
    this.initializeKeyboardListeners();
    this.addKeyboardStyles();
  }

  private initializeKeyboardListeners(): void {
    document.addEventListener('keydown', this.handleKeyDown.bind(this));
    document.addEventListener('focusin', this.handleFocusIn.bind(this));
    document.addEventListener('focusout', this.handleFocusOut.bind(this));
  }

  private addKeyboardStyles(): void {
    if (document.querySelector('#keyboard-navigation-styles')) return;

    const style = document.createElement('style');
    style.id = 'keyboard-navigation-styles';
    style.textContent = `
      .keyboard-focus-visible {
        outline: 2px solid #007acc !important;
        outline-offset: 2px;
        box-shadow: 0 0 0 4px rgba(0, 122, 204, 0.2) !important;
      }
      
      .keyboard-navigation-hint {
        position: fixed;
        bottom: 20px;
        left: 20px;
        background: rgba(0, 0, 0, 0.8);
        color: white;
        padding: 8px 12px;
        border-radius: 4px;
        font-size: 12px;
        font-family: monospace;
        z-index: 9999;
        transition: opacity 0.2s ease;
      }
      
      .keyboard-navigation-hint.hidden {
        opacity: 0;
        pointer-events: none;
      }
      
      .node-keyboard-selected {
        stroke: #007acc !important;
        stroke-width: 3px !important;
        filter: drop-shadow(0 0 8px rgba(0, 122, 204, 0.6)) !important;
      }
    `;
    
    document.head.appendChild(style);
  }

  private handleKeyDown(event: KeyboardEvent): void {
    // Only handle keyboard navigation if a diagram container is focused
    if (!this.isContainerFocused()) return;

    const { key, ctrlKey, metaKey, shiftKey } = event;

    switch (key) {
      case 'ArrowUp':
      case 'ArrowDown':
      case 'ArrowLeft':
      case 'ArrowRight':
        event.preventDefault();
        this.navigateToAdjacentNode(key, shiftKey);
        break;
        
      case 'Enter':
      case ' ':
        event.preventDefault();
        this.activateSelectedNode();
        break;
        
      case 'Escape':
        event.preventDefault();
        this.clearSelection();
        break;
        
      case 'Tab':
        if (!shiftKey) {
          event.preventDefault();
          this.selectNextNode();
        } else {
          event.preventDefault();
          this.selectPreviousNode();
        }
        break;
        
      case 'Home':
        event.preventDefault();
        this.selectFirstNode();
        break;
        
      case 'End':
        event.preventDefault();
        this.selectLastNode();
        break;
        
      case 'f':
      case 'F':
        if (ctrlKey || metaKey) {
          // Don't interfere with browser find
          break;
        }
        event.preventDefault();
        this.focusOnSelectedNode();
        break;
        
      case '?':
        event.preventDefault();
        this.toggleKeyboardHelp();
        break;
        
      default:
        // Handle alphanumeric keys for quick selection
        if (this.isAlphaNumeric(key)) {
          event.preventDefault();
          this.quickSelectNode(key.toLowerCase());
        }
    }
  }

  private handleFocusIn(event: FocusEvent): void {
    const target = event.target as Element;
    if (this.isDiagramContainer(target)) {
      this.state.focusVisible = true;
      this.showKeyboardHint();
      
      // If no node is selected, select the first one
      if (this.state.selectedNodeIndex === -1 && this.nodes.length > 0) {
        this.selectNodeByIndex(0);
      }
    }
  }

  private handleFocusOut(event: FocusEvent): void {
    const target = event.target as Element;
    if (this.isDiagramContainer(target)) {
      this.state.focusVisible = false;
      this.hideKeyboardHint();
    }
  }

  private isContainerFocused(): boolean {
    const activeElement = document.activeElement;
    return activeElement ? this.isDiagramContainer(activeElement) : false;
  }

  private isDiagramContainer(element: Element): boolean {
    return element.classList.contains('interactive-diagram-container') ||
           element.classList.contains('diagram-content') ||
           element.closest('.interactive-diagram-container') !== null;
  }

  private navigateToAdjacentNode(direction: string, includeConnected: boolean): void {
    if (this.state.selectedNodeIndex === -1) {
      this.selectNodeByIndex(0);
      return;
    }

    const currentNode = this.nodes[this.state.selectedNodeIndex];
    let nextNode: NodeEventData | null = null;

    if (includeConnected && currentNode.connections.length > 0) {
      // Navigate to connected nodes
      nextNode = this.findConnectedNode(currentNode, direction);
    } else {
      // Navigate by spatial position
      nextNode = this.findSpatialAdjacentNode(currentNode, direction);
    }

    if (nextNode) {
      const nextIndex = this.nodes.findIndex(node => node.nodeId === nextNode!.nodeId);
      if (nextIndex !== -1) {
        this.selectNodeByIndex(nextIndex);
      }
    }
  }

  private findConnectedNode(currentNode: NodeEventData, direction: string): NodeEventData | null {
    const connectedNodes = currentNode.connections
      .map(connectionId => this.nodes.find(node => node.nodeId === connectionId))
      .filter(node => node !== undefined) as NodeEventData[];

    if (connectedNodes.length === 0) return null;

    // For connected navigation, cycle through connected nodes
    const currentConnectedIndex = connectedNodes.findIndex(node => 
      node.nodeId === currentNode.nodeId
    );
    
    const nextIndex = direction === 'ArrowRight' || direction === 'ArrowDown' 
      ? (currentConnectedIndex + 1) % connectedNodes.length
      : (currentConnectedIndex - 1 + connectedNodes.length) % connectedNodes.length;

    return connectedNodes[nextIndex];
  }

  private findSpatialAdjacentNode(currentNode: NodeEventData, direction: string): NodeEventData | null {
    const currentPos = currentNode.position;
    let bestNode: NodeEventData | null = null;
    let bestDistance = Infinity;

    this.nodes.forEach(node => {
      if (node.nodeId === currentNode.nodeId) return;

      const pos = node.position;
      let isValidDirection = false;
      let distance = 0;

      switch (direction) {
        case 'ArrowUp':
          isValidDirection = pos.y < currentPos.y;
          distance = Math.sqrt(Math.pow(pos.x - currentPos.x, 2) + Math.pow(currentPos.y - pos.y, 2));
          break;
        case 'ArrowDown':
          isValidDirection = pos.y > currentPos.y;
          distance = Math.sqrt(Math.pow(pos.x - currentPos.x, 2) + Math.pow(pos.y - currentPos.y, 2));
          break;
        case 'ArrowLeft':
          isValidDirection = pos.x < currentPos.x;
          distance = Math.sqrt(Math.pow(currentPos.x - pos.x, 2) + Math.pow(pos.y - currentPos.y, 2));
          break;
        case 'ArrowRight':
          isValidDirection = pos.x > currentPos.x;
          distance = Math.sqrt(Math.pow(pos.x - currentPos.x, 2) + Math.pow(pos.y - currentPos.y, 2));
          break;
      }

      if (isValidDirection && distance < bestDistance) {
        bestDistance = distance;
        bestNode = node;
      }
    });

    return bestNode;
  }

  private selectNextNode(): void {
    const nextIndex = (this.state.selectedNodeIndex + 1) % this.nodes.length;
    this.selectNodeByIndex(nextIndex);
  }

  private selectPreviousNode(): void {
    const prevIndex = (this.state.selectedNodeIndex - 1 + this.nodes.length) % this.nodes.length;
    this.selectNodeByIndex(prevIndex);
  }

  private selectFirstNode(): void {
    if (this.nodes.length > 0) {
      this.selectNodeByIndex(0);
    }
  }

  private selectLastNode(): void {
    if (this.nodes.length > 0) {
      this.selectNodeByIndex(this.nodes.length - 1);
    }
  }

  private selectNodeByIndex(index: number): void {
    if (index < 0 || index >= this.nodes.length) return;

    this.clearNodeSelection();
    this.state.selectedNodeIndex = index;
    const selectedNode = this.nodes[index];
    
    this.highlightNode(selectedNode.nodeId);
    this.onNodeSelect(selectedNode.nodeId);
    
    // Scroll node into view if needed
    this.scrollNodeIntoView(selectedNode.nodeId);
  }

  private activateSelectedNode(): void {
    if (this.state.selectedNodeIndex === -1) return;
    
    const selectedNode = this.nodes[this.state.selectedNodeIndex];
    
    // Simulate click event
    const nodeElement = document.querySelector(`#${selectedNode.nodeId}`);
    if (nodeElement) {
      nodeElement.dispatchEvent(new MouseEvent('click', {
        bubbles: true,
        cancelable: true,
      }));
    }
  }

  private clearSelection(): void {
    this.clearNodeSelection();
    this.state.selectedNodeIndex = -1;
  }

  private focusOnSelectedNode(): void {
    if (this.state.selectedNodeIndex === -1) return;
    
    const selectedNode = this.nodes[this.state.selectedNodeIndex];
    
    // Trigger focus event for pan/zoom
    document.dispatchEvent(new CustomEvent('keyboard-focus-node', {
      detail: { nodeId: selectedNode.nodeId }
    }));
  }

  private quickSelectNode(letter: string): void {
    // Find nodes that start with the given letter
    const matchingNodes = this.nodes.filter(node => 
      node.componentData.name.toLowerCase().startsWith(letter)
    );

    if (matchingNodes.length === 0) return;

    // If multiple matches, cycle through them
    const currentMatch = matchingNodes.find(node => 
      node.nodeId === this.nodes[this.state.selectedNodeIndex]?.nodeId
    );

    let nextMatch: NodeEventData;
    if (currentMatch) {
      const currentIndex = matchingNodes.indexOf(currentMatch);
      const nextIndex = (currentIndex + 1) % matchingNodes.length;
      nextMatch = matchingNodes[nextIndex];
    } else {
      nextMatch = matchingNodes[0];
    }

    const nodeIndex = this.nodes.findIndex(node => node.nodeId === nextMatch.nodeId);
    if (nodeIndex !== -1) {
      this.selectNodeByIndex(nodeIndex);
    }
  }

  private highlightNode(nodeId: string): void {
    const nodeElement = document.querySelector(`#${nodeId}`);
    if (nodeElement) {
      nodeElement.classList.add('node-keyboard-selected');
      
      // Add ARIA attributes for accessibility
      nodeElement.setAttribute('aria-selected', 'true');
      nodeElement.setAttribute('tabindex', '0');
    }
  }

  private clearNodeSelection(): void {
    // Remove highlights from all nodes
    document.querySelectorAll('.node-keyboard-selected').forEach(element => {
      element.classList.remove('node-keyboard-selected');
      element.removeAttribute('aria-selected');
      element.setAttribute('tabindex', '-1');
    });
  }

  private scrollNodeIntoView(nodeId: string): void {
    const nodeElement = document.querySelector(`#${nodeId}`);
    if (nodeElement && nodeElement instanceof Element) {
      nodeElement.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
        inline: 'center'
      });
    }
  }

  private showKeyboardHint(): void {
    let hint = document.querySelector('.keyboard-navigation-hint') as HTMLElement;
    
    if (!hint) {
      hint = document.createElement('div');
      hint.className = 'keyboard-navigation-hint';
      hint.innerHTML = `
        <div><strong>Keyboard Navigation:</strong></div>
        <div>↑↓←→ Navigate | Tab Next | Enter Select</div>
        <div>Shift+↑↓←→ Follow connections | ? Help</div>
      `;
      document.body.appendChild(hint);
    }
    
    hint.classList.remove('hidden');
    
    // Auto-hide after 3 seconds
    setTimeout(() => {
      hint.classList.add('hidden');
    }, 3000);
  }

  private hideKeyboardHint(): void {
    const hint = document.querySelector('.keyboard-navigation-hint');
    if (hint) {
      hint.classList.add('hidden');
    }
  }

  private toggleKeyboardHelp(): void {
    // Show/hide detailed keyboard help
    let helpModal = document.querySelector('.keyboard-help-modal') as HTMLElement;
    
    if (helpModal) {
      helpModal.remove();
      return;
    }

    helpModal = document.createElement('div');
    helpModal.className = 'keyboard-help-modal';
    helpModal.style.cssText = `
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: white;
      border: 1px solid #ccc;
      border-radius: 8px;
      padding: 20px;
      box-shadow: 0 4px 20px rgba(0,0,0,0.3);
      z-index: 10001;
      max-width: 400px;
      font-family: system-ui;
    `;

    helpModal.innerHTML = `
      <h3 style="margin-top: 0;">Keyboard Shortcuts</h3>
      <div style="display: grid; grid-template-columns: 1fr 2fr; gap: 8px; font-size: 14px;">
        <strong>↑↓←→</strong><span>Navigate between nodes</span>
        <strong>Shift + ↑↓←→</strong><span>Follow connections</span>
        <strong>Tab / Shift+Tab</strong><span>Next/previous node</span>
        <strong>Enter / Space</strong><span>Activate selected node</span>
        <strong>Home / End</strong><span>First/last node</span>
        <strong>F</strong><span>Focus on selected node</span>
        <strong>A-Z</strong><span>Quick select by name</span>
        <strong>Escape</strong><span>Clear selection</span>
        <strong>?</strong><span>Toggle this help</span>
      </div>
      <button style="margin-top: 16px; padding: 8px 16px; float: right;" onclick="this.parentElement.remove()">Close</button>
    `;

    document.body.appendChild(helpModal);

    // Close on escape or click outside
    const closeHandler = (e: KeyboardEvent | MouseEvent) => {
      if (e instanceof KeyboardEvent && e.key === 'Escape') {
        helpModal.remove();
        document.removeEventListener('keydown', closeHandler);
      } else if (e instanceof MouseEvent && !helpModal.contains(e.target as Node)) {
        helpModal.remove();
        document.removeEventListener('click', closeHandler);
      }
    };

    document.addEventListener('keydown', closeHandler);
    document.addEventListener('click', closeHandler);
  }

  private isAlphaNumeric(key: string): boolean {
    return /^[a-zA-Z0-9]$/.test(key);
  }

  public updateNodes(nodes: NodeEventData[]): void {
    this.nodes = nodes;
    this.state.nodes = nodes;
    
    // Reset selection if current selection is no longer valid
    if (this.state.selectedNodeIndex >= nodes.length) {
      this.state.selectedNodeIndex = -1;
    }
  }

  public selectNode(nodeId: string): void {
    const index = this.nodes.findIndex(node => node.nodeId === nodeId);
    if (index !== -1) {
      this.selectNodeByIndex(index);
    }
  }

  public getSelectedNode(): NodeEventData | null {
    if (this.state.selectedNodeIndex === -1) return null;
    return this.nodes[this.state.selectedNodeIndex] || null;
  }

  public destroy(): void {
    // Remove event listeners
    document.removeEventListener('keydown', this.handleKeyDown);
    document.removeEventListener('focusin', this.handleFocusIn);
    document.removeEventListener('focusout', this.handleFocusOut);

    // Clear selection
    this.clearNodeSelection();

    // Remove UI elements
    const hint = document.querySelector('.keyboard-navigation-hint');
    if (hint) hint.remove();

    const help = document.querySelector('.keyboard-help-modal');
    if (help) help.remove();

    const styles = document.querySelector('#keyboard-navigation-styles');
    if (styles) styles.remove();
  }
}