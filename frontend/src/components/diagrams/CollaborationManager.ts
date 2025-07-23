import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { IndexeddbPersistence } from 'y-indexeddb';
import * as d3 from 'd3';
import type { NodeEventData } from './types';

export interface CollaborationConfig {
  websocketUrl: string;
  roomId: string;
  userId: string;
  userName: string;
  userColor: string;
  enablePersistence: boolean;
  enableUserPresence: boolean;
  enableComments: boolean;
  enableLiveEditing: boolean;
}

export interface User {
  id: string;
  name: string;
  color: string;
  cursor?: { x: number; y: number };
  activeElement?: string;
  lastSeen: number;
  isOnline: boolean;
}

export interface Comment {
  id: string;
  userId: string;
  userName: string;
  content: string;
  position: { x: number; y: number };
  nodeId?: string;
  timestamp: number;
  resolved: boolean;
  replies: CommentReply[];
}

export interface CommentReply {
  id: string;
  userId: string;
  userName: string;
  content: string;
  timestamp: number;
}

export interface DiagramChange {
  type: 'node-move' | 'node-add' | 'node-remove' | 'node-edit' | 'style-change';
  nodeId?: string;
  data: any;
  userId: string;
  timestamp: number;
}

export interface CursorPosition {
  userId: string;
  x: number;
  y: number;
  timestamp: number;
}

export class CollaborationManager {
  private ydoc: Y.Doc;
  private websocketProvider: WebsocketProvider | null = null;
  private indexeddbProvider: IndexeddbPersistence | null = null;
  private svgElement: SVGElement;
  private config: CollaborationConfig;
  
  // Y.js shared types
  private sharedNodes: Y.Map<any>;
  private sharedComments: Y.Array<Comment>;
  private sharedUsers: Y.Map<User>;
  private sharedCursors: Y.Map<CursorPosition>;
  private sharedChanges: Y.Array<DiagramChange>;
  
  // Local state
  private currentUser: User;
  private onlineUsers: Map<string, User> = new Map();
  private comments: Comment[] = [];
  private cursors: Map<string, CursorPosition> = new Map();
  
  // Event handlers
  private onUserJoined?: (user: User) => void;
  private onUserLeft?: (userId: string) => void;
  private onCommentAdded?: (comment: Comment) => void;
  private onCommentUpdated?: (comment: Comment) => void;
  private onDiagramChanged?: (change: DiagramChange) => void;
  private onCursorMoved?: (cursor: CursorPosition) => void;

  constructor(svgElement: SVGElement, config: CollaborationConfig) {
    this.svgElement = svgElement;
    this.config = config;
    this.ydoc = new Y.Doc();
    
    // Initialize shared types
    this.sharedNodes = this.ydoc.getMap('nodes');
    this.sharedComments = this.ydoc.getArray('comments');
    this.sharedUsers = this.ydoc.getMap('users');
    this.sharedCursors = this.ydoc.getMap('cursors');
    this.sharedChanges = this.ydoc.getArray('changes');
    
    // Initialize current user
    this.currentUser = {
      id: config.userId,
      name: config.userName,
      color: config.userColor,
      lastSeen: Date.now(),
      isOnline: true
    };
    
    this.initializeProviders();
    this.setupEventListeners();
    this.initializeUI();
  }

  private initializeProviders(): void {
    // WebSocket provider for real-time sync
    this.websocketProvider = new WebsocketProvider(
      this.config.websocketUrl,
      this.config.roomId,
      this.ydoc
    );

    // IndexedDB provider for offline persistence
    if (this.config.enablePersistence) {
      this.indexeddbProvider = new IndexeddbPersistence(
        this.config.roomId,
        this.ydoc
      );
    }

    // Handle connection status
    this.websocketProvider.on('status', (event: any) => {
      console.log('Collaboration status:', event.status);
      this.updateConnectionStatus(event.status === 'connected');
    });
  }

  private setupEventListeners(): void {
    // Listen for shared data changes
    this.sharedUsers.observe(this.handleUsersChange.bind(this));
    this.sharedComments.observe(this.handleCommentsChange.bind(this));
    this.sharedCursors.observe(this.handleCursorsChange.bind(this));
    this.sharedChanges.observe(this.handleDiagramChanges.bind(this));
    this.sharedNodes.observe(this.handleNodesChange.bind(this));

    // Add current user to shared users
    this.sharedUsers.set(this.currentUser.id, this.currentUser);

    // Track mouse movements for cursor sharing
    if (this.config.enableUserPresence) {
      this.setupCursorTracking();
    }

    // Handle window visibility changes
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        this.setUserOffline();
      } else {
        this.setUserOnline();
      }
    });

    // Handle beforeunload to clean up
    window.addEventListener('beforeunload', () => {
      this.disconnect();
    });
  }

  private initializeUI(): void {
    // Create collaboration UI elements
    this.createCollaborationPanel();
    this.createUserPresenceIndicators();
    
    if (this.config.enableComments) {
      this.setupCommentSystem();
    }
  }

  private createCollaborationPanel(): void {
    const panel = d3.select(this.svgElement.parentElement)
      .append('div')
      .attr('class', 'collaboration-panel')
      .style('position', 'absolute')
      .style('top', '10px')
      .style('left', '10px')
      .style('background', 'rgba(255, 255, 255, 0.95)')
      .style('border-radius', '8px')
      .style('padding', '12px')
      .style('box-shadow', '0 4px 12px rgba(0, 0, 0, 0.15)')
      .style('font-family', 'system-ui, sans-serif')
      .style('font-size', '14px')
      .style('min-width', '200px');

    // Connection status
    panel.append('div')
      .attr('class', 'connection-status')
      .style('display', 'flex')
      .style('align-items', 'center')
      .style('gap', '8px')
      .style('margin-bottom', '8px');

    // Online users list
    panel.append('div')
      .attr('class', 'online-users')
      .style('border-top', '1px solid #e5e7eb')
      .style('padding-top', '8px');

    this.updateCollaborationPanel();
  }

  private createUserPresenceIndicators(): void {
    // Create container for user cursors
    d3.select(this.svgElement.parentElement)
      .append('div')
      .attr('class', 'user-cursors')
      .style('position', 'absolute')
      .style('top', '0')
      .style('left', '0')
      .style('width', '100%')
      .style('height', '100%')
      .style('pointer-events', 'none')
      .style('z-index', '10');
  }

  private setupCommentSystem(): void {
    // Add double-click listener for creating comments
    d3.select(this.svgElement).on('dblclick', (event) => {
      const rect = this.svgElement.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      
      this.showCommentDialog(x, y);
    });

    // Create comments container
    d3.select(this.svgElement.parentElement)
      .append('div')
      .attr('class', 'comments-container')
      .style('position', 'absolute')
      .style('top', '0')
      .style('left', '0')
      .style('width', '100%')
      .style('height', '100%')
      .style('pointer-events', 'none')
      .style('z-index', '5');
  }

  private setupCursorTracking(): void {
    let throttleTimer: number | null = null;
    
    d3.select(this.svgElement).on('mousemove', (event) => {
      if (throttleTimer) return;
      
      throttleTimer = window.setTimeout(() => {
        const rect = this.svgElement.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        
        this.updateCursorPosition(x, y);
        throttleTimer = null;
      }, 50); // Throttle to 20fps
    });
  }

  private updateCursorPosition(x: number, y: number): void {
    const cursor: CursorPosition = {
      userId: this.currentUser.id,
      x,
      y,
      timestamp: Date.now()
    };
    
    this.sharedCursors.set(this.currentUser.id, cursor);
  }

  private handleUsersChange(): void {
    this.onlineUsers.clear();
    
    this.sharedUsers.forEach((user, userId) => {
      this.onlineUsers.set(userId, user);
      
      if (userId !== this.currentUser.id) {
        if (user.isOnline && !this.onlineUsers.has(userId)) {
          this.onUserJoined?.(user);
        }
      }
    });
    
    this.updateCollaborationPanel();
    this.updateUserPresenceIndicators();
  }

  private handleCommentsChange(): void {
    this.comments = this.sharedComments.toArray();
    this.renderComments();
    
    // Notify about new comments
    const latestComment = this.comments[this.comments.length - 1];
    if (latestComment && latestComment.userId !== this.currentUser.id) {
      this.onCommentAdded?.(latestComment);
    }
  }

  private handleCursorsChange(): void {
    this.cursors.clear();
    
    this.sharedCursors.forEach((cursor, userId) => {
      if (userId !== this.currentUser.id) {
        this.cursors.set(userId, cursor);
        this.onCursorMoved?.(cursor);
      }
    });
    
    this.renderUserCursors();
  }

  private handleDiagramChanges(): void {
    const changes = this.sharedChanges.toArray();
    const latestChange = changes[changes.length - 1];
    
    if (latestChange && latestChange.userId !== this.currentUser.id) {
      this.onDiagramChanged?.(latestChange);
      this.applyDiagramChange(latestChange);
    }
  }

  private handleNodesChange(): void {
    // Handle shared node updates
    this.sharedNodes.forEach((nodeData, nodeId) => {
      this.updateNodeFromSharedData(nodeId, nodeData);
    });
  }

  private applyDiagramChange(change: DiagramChange): void {
    switch (change.type) {
      case 'node-move':
        this.moveNode(change.nodeId!, change.data.x, change.data.y);
        break;
      case 'node-edit':
        this.updateNodeData(change.nodeId!, change.data);
        break;
      case 'style-change':
        this.applyStyleChange(change.data);
        break;
    }
  }

  private moveNode(nodeId: string, x: number, y: number): void {
    const nodeElement = d3.select(this.svgElement)
      .select(`[data-node-id="${nodeId}"]`);
    
    if (!nodeElement.empty()) {
      nodeElement.attr('transform', `translate(${x}, ${y})`);
    }
  }

  private updateNodeData(nodeId: string, data: any): void {
    // Update node with new data
    const nodeElement = d3.select(this.svgElement)
      .select(`[data-node-id="${nodeId}"]`);
    
    if (!nodeElement.empty()) {
      // Apply data updates to the node
      nodeElement.selectAll('text').text(data.label || '');
    }
  }

  private updateNodeFromSharedData(nodeId: string, data: any): void {
    const nodeElement = d3.select(this.svgElement)
      .select(`[data-node-id="${nodeId}"]`);
    
    if (!nodeElement.empty()) {
      if (data.position) {
        this.moveNode(nodeId, data.position.x, data.position.y);
      }
      if (data.label) {
        nodeElement.selectAll('text').text(data.label);
      }
    }
  }

  private applyStyleChange(styleData: any): void {
    // Apply style changes to the diagram
    if (styleData.selector && styleData.styles) {
      d3.select(this.svgElement)
        .selectAll(styleData.selector)
        .each(function() {
          const element = d3.select(this);
          Object.entries(styleData.styles).forEach(([prop, value]) => {
            element.style(prop, value as string);
          });
        });
    }
  }

  private updateCollaborationPanel(): void {
    const panel = d3.select('.collaboration-panel');
    if (panel.empty()) return;
    
    // Update connection status
    const statusDiv = panel.select('.connection-status');
    statusDiv.selectAll('*').remove();
    
    const isConnected = this.websocketProvider?.ws?.readyState === WebSocket.OPEN;
    
    statusDiv.append('div')
      .style('width', '8px')
      .style('height', '8px')
      .style('border-radius', '50%')
      .style('background', isConnected ? '#10b981' : '#ef4444');
    
    statusDiv.append('span')
      .text(isConnected ? 'Connected' : 'Disconnected')
      .style('color', isConnected ? '#059669' : '#dc2626')
      .style('font-weight', '500');
    
    // Update online users
    const usersDiv = panel.select('.online-users');
    usersDiv.selectAll('*').remove();
    
    usersDiv.append('div')
      .text(`Online (${this.onlineUsers.size})`)
      .style('font-weight', '600')
      .style('margin-bottom', '4px');
    
    this.onlineUsers.forEach((user) => {
      const userDiv = usersDiv.append('div')
        .style('display', 'flex')
        .style('align-items', 'center')
        .style('gap', '6px')
        .style('margin', '2px 0');
      
      userDiv.append('div')
        .style('width', '12px')
        .style('height', '12px')
        .style('border-radius', '50%')
        .style('background', user.color);
      
      userDiv.append('span')
        .text(user.name)
        .style('font-size', '13px')
        .style('color', user.id === this.currentUser.id ? '#059669' : '#374151');
    });
  }

  private updateUserPresenceIndicators(): void {
    const cursorsContainer = d3.select('.user-cursors');
    if (cursorsContainer.empty()) return;
    
    // Remove old cursors
    cursorsContainer.selectAll('.user-cursor').remove();
    
    // Add current cursors
    this.cursors.forEach((cursor, userId) => {
      const user = this.onlineUsers.get(userId);
      if (!user) return;
      
      const cursorDiv = cursorsContainer.append('div')
        .attr('class', 'user-cursor')
        .style('position', 'absolute')
        .style('left', `${cursor.x}px`)
        .style('top', `${cursor.y}px`)
        .style('transform', 'translate(-2px, -2px)')
        .style('pointer-events', 'none')
        .style('z-index', '15');
      
      // Cursor pointer
      cursorDiv.append('div')
        .style('width', '0')
        .style('height', '0')
        .style('border-left', '8px solid transparent')
        .style('border-right', '8px solid transparent')
        .style('border-bottom', `12px solid ${user.color}`);
      
      // User name label
      cursorDiv.append('div')
        .text(user.name)
        .style('background', user.color)
        .style('color', 'white')
        .style('padding', '2px 6px')
        .style('border-radius', '4px')
        .style('font-size', '11px')
        .style('font-weight', '500')
        .style('margin-top', '2px')
        .style('white-space', 'nowrap');
    });
  }

  private renderUserCursors(): void {
    this.updateUserPresenceIndicators();
  }

  private renderComments(): void {
    const commentsContainer = d3.select('.comments-container');
    if (commentsContainer.empty()) return;
    
    // Remove old comments
    commentsContainer.selectAll('.comment').remove();
    
    // Render current comments
    this.comments.forEach(comment => {
      if (comment.resolved) return;
      
      const commentDiv = commentsContainer.append('div')
        .attr('class', 'comment')
        .style('position', 'absolute')
        .style('left', `${comment.position.x}px`)
        .style('top', `${comment.position.y}px`)
        .style('background', 'white')
        .style('border', '2px solid #3b82f6')
        .style('border-radius', '8px')
        .style('padding', '8px')
        .style('max-width', '250px')
        .style('box-shadow', '0 4px 12px rgba(0, 0, 0, 0.15)')
        .style('font-family', 'system-ui, sans-serif')
        .style('font-size', '13px')
        .style('pointer-events', 'auto')
        .style('z-index', '20');
      
      // Comment header
      const header = commentDiv.append('div')
        .style('display', 'flex')
        .style('justify-content', 'space-between')
        .style('align-items', 'center')
        .style('margin-bottom', '6px');
      
      header.append('span')
        .text(comment.userName)
        .style('font-weight', '600')
        .style('color', '#374151');
      
      header.append('button')
        .text('✕')
        .style('background', 'none')
        .style('border', 'none')
        .style('cursor', 'pointer')
        .style('color', '#6b7280')
        .on('click', () => this.resolveComment(comment.id));
      
      // Comment content
      commentDiv.append('div')
        .text(comment.content)
        .style('color', '#4b5563')
        .style('line-height', '1.4');
      
      // Comment timestamp
      commentDiv.append('div')
        .text(new Date(comment.timestamp).toLocaleTimeString())
        .style('color', '#9ca3af')
        .style('font-size', '11px')
        .style('margin-top', '4px');
    });
  }

  private showCommentDialog(x: number, y: number): void {
    const dialog = d3.select(this.svgElement.parentElement)
      .append('div')
      .attr('class', 'comment-dialog')
      .style('position', 'absolute')
      .style('left', `${x}px`)
      .style('top', `${y}px`)
      .style('background', 'white')
      .style('border', '1px solid #d1d5db')
      .style('border-radius', '8px')
      .style('padding', '12px')
      .style('box-shadow', '0 4px 12px rgba(0, 0, 0, 0.15)')
      .style('z-index', '30')
      .style('min-width', '200px');
    
    dialog.append('textarea')
      .attr('placeholder', 'Add a comment...')
      .style('width', '100%')
      .style('height', '60px')
      .style('border', '1px solid #d1d5db')
      .style('border-radius', '4px')
      .style('padding', '6px')
      .style('font-family', 'system-ui, sans-serif')
      .style('font-size', '13px')
      .style('resize', 'none')
      .style('outline', 'none');
    
    const buttonRow = dialog.append('div')
      .style('display', 'flex')
      .style('gap', '8px')
      .style('margin-top', '8px');
    
    buttonRow.append('button')
      .text('Add')
      .style('background', '#3b82f6')
      .style('color', 'white')
      .style('border', 'none')
      .style('padding', '4px 12px')
      .style('border-radius', '4px')
      .style('cursor', 'pointer')
      .style('font-size', '12px')
      .on('click', () => {
        const textarea = dialog.select('textarea').node() as HTMLTextAreaElement;
        const content = textarea.value.trim();
        
        if (content) {
          this.addComment(content, x, y);
        }
        
        dialog.remove();
      });
    
    buttonRow.append('button')
      .text('Cancel')
      .style('background', '#f3f4f6')
      .style('color', '#374151')
      .style('border', 'none')
      .style('padding', '4px 12px')
      .style('border-radius', '4px')
      .style('cursor', 'pointer')
      .style('font-size', '12px')
      .on('click', () => dialog.remove());
    
    // Focus the textarea
    (dialog.select('textarea').node() as HTMLTextAreaElement).focus();
  }

  // Public methods
  public addComment(content: string, x: number, y: number, nodeId?: string): void {
    const comment: Comment = {
      id: `comment-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      userId: this.currentUser.id,
      userName: this.currentUser.name,
      content,
      position: { x, y },
      nodeId,
      timestamp: Date.now(),
      resolved: false,
      replies: []
    };
    
    this.sharedComments.push([comment]);
  }

  public resolveComment(commentId: string): void {
    const commentIndex = this.comments.findIndex(c => c.id === commentId);
    if (commentIndex >= 0) {
      const comment = { ...this.comments[commentIndex], resolved: true };
      this.sharedComments.delete(commentIndex, 1);
      this.sharedComments.insert(commentIndex, [comment]);
    }
  }

  public broadcastDiagramChange(change: Omit<DiagramChange, 'userId' | 'timestamp'>): void {
    const diagramChange: DiagramChange = {
      ...change,
      userId: this.currentUser.id,
      timestamp: Date.now()
    };
    
    this.sharedChanges.push([diagramChange]);
  }

  public updateSharedNode(nodeId: string, data: any): void {
    this.sharedNodes.set(nodeId, {
      ...this.sharedNodes.get(nodeId),
      ...data,
      lastModifiedBy: this.currentUser.id,
      lastModified: Date.now()
    });
  }

  private updateConnectionStatus(connected: boolean): void {
    this.updateCollaborationPanel();
  }

  private setUserOnline(): void {
    this.currentUser.isOnline = true;
    this.currentUser.lastSeen = Date.now();
    this.sharedUsers.set(this.currentUser.id, this.currentUser);
  }

  private setUserOffline(): void {
    this.currentUser.isOnline = false;
    this.currentUser.lastSeen = Date.now();
    this.sharedUsers.set(this.currentUser.id, this.currentUser);
  }

  public setEventHandlers(handlers: {
    onUserJoined?: (user: User) => void;
    onUserLeft?: (userId: string) => void;
    onCommentAdded?: (comment: Comment) => void;
    onCommentUpdated?: (comment: Comment) => void;
    onDiagramChanged?: (change: DiagramChange) => void;
    onCursorMoved?: (cursor: CursorPosition) => void;
  }): void {
    this.onUserJoined = handlers.onUserJoined;
    this.onUserLeft = handlers.onUserLeft;
    this.onCommentAdded = handlers.onCommentAdded;
    this.onCommentUpdated = handlers.onCommentUpdated;
    this.onDiagramChanged = handlers.onDiagramChanged;
    this.onCursorMoved = handlers.onCursorMoved;
  }

  public getOnlineUsers(): User[] {
    return Array.from(this.onlineUsers.values());
  }

  public getComments(): Comment[] {
    return this.comments.filter(c => !c.resolved);
  }

  public getCurrentUser(): User {
    return this.currentUser;
  }

  public disconnect(): void {
    this.setUserOffline();
    
    if (this.websocketProvider) {
      this.websocketProvider.destroy();
    }
    
    if (this.indexeddbProvider) {
      this.indexeddbProvider.destroy();
    }
    
    this.ydoc.destroy();
  }

  public dispose(): void {
    this.disconnect();
    
    // Clean up UI elements
    d3.select('.collaboration-panel').remove();
    d3.select('.user-cursors').remove();
    d3.select('.comments-container').remove();
    d3.select('.comment-dialog').remove();
  }
}

export default CollaborationManager;