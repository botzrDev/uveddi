// Context Menu Component for Interactive Diagrams - UV-89 Phase 1
// Provides right-click actions for nodes

import React, { useEffect, useRef } from 'react';
import type { ContextMenuProps, ContextMenuItem } from './types';

export const ContextMenu: React.FC<ContextMenuProps> = ({
  nodeId,
  nodeData,
  position,
  onClose,
  onAction
}) => {
  const menuRef = useRef<HTMLDivElement>(null);

  // Define context menu items
  const menuItems: ContextMenuItem[] = [
    { id: 'view-details', label: 'View Details', icon: '👁️' },
    { id: 'highlight-dependencies', label: 'Highlight Dependencies', icon: '🔗' },
    { id: 'focus-component', label: 'Focus on Component', icon: '🎯' },
    { separator: true, id: 'sep1', label: '', icon: '' },
    { id: 'export-subgraph', label: 'Export Subgraph', icon: '📤' },
    { id: 'add-comment', label: 'Add Comment', icon: '💬' },
    { separator: true, id: 'sep2', label: '', icon: '' },
    { id: 'copy-node-id', label: 'Copy Node ID', icon: '📋' },
    { id: 'copy-file-path', label: 'Copy File Path', icon: '📁' },
  ];

  // Handle click outside to close menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscapeKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscapeKey);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscapeKey);
    };
  }, [onClose]);

  // Position menu to stay within viewport
  useEffect(() => {
    if (menuRef.current) {
      const menu = menuRef.current;
      const rect = menu.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      let adjustedX = position.x;
      let adjustedY = position.y;

      // Adjust horizontal position if menu would go off-screen
      if (position.x + rect.width > viewportWidth) {
        adjustedX = viewportWidth - rect.width - 10;
      }

      // Adjust vertical position if menu would go off-screen
      if (position.y + rect.height > viewportHeight) {
        adjustedY = viewportHeight - rect.height - 10;
      }

      // Ensure menu doesn't go negative
      adjustedX = Math.max(10, adjustedX);
      adjustedY = Math.max(10, adjustedY);

      menu.style.left = `${adjustedX}px`;
      menu.style.top = `${adjustedY}px`;
    }
  }, [position]);

  const handleItemClick = (itemId: string) => {
    switch (itemId) {
      case 'copy-node-id':
        copyToClipboard(nodeId);
        break;
      case 'copy-file-path':
        copyToClipboard(nodeData.componentData.filePath);
        break;
      default:
        onAction(itemId, nodeId);
    }
    onClose();
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      console.log('Copied to clipboard:', text);
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      // Fallback for older browsers
      const textArea = document.createElement('textarea');
      textArea.value = text;
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
    }
  };

  return (
    <div
      ref={menuRef}
      className="context-menu"
      style={{
        position: 'fixed',
        left: position.x,
        top: position.y,
        zIndex: 10000,
        backgroundColor: '#ffffff',
        border: '1px solid #e0e0e0',
        borderRadius: '6px',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        padding: '4px 0',
        minWidth: '180px',
        fontSize: '14px',
        fontFamily: 'system-ui, -apple-system, sans-serif',
      }}
    >
      {/* Menu Header */}
      <div
        style={{
          padding: '8px 16px',
          borderBottom: '1px solid #e0e0e0',
          fontSize: '12px',
          color: '#666',
          fontWeight: '500',
        }}
      >
        {nodeData.componentData.name}
      </div>

      {/* Menu Items */}
      {menuItems.map((item) => {
        if (item.separator) {
          return (
            <div
              key={item.id}
              style={{
                height: '1px',
                backgroundColor: '#e0e0e0',
                margin: '4px 0',
              }}
            />
          );
        }

        return (
          <button
            key={item.id}
            className="context-menu-item"
            onClick={() => handleItemClick(item.id)}
            disabled={item.disabled}
            style={{
              width: '100%',
              padding: '8px 16px',
              border: 'none',
              backgroundColor: 'transparent',
              cursor: item.disabled ? 'not-allowed' : 'pointer',
              textAlign: 'left',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              fontSize: '14px',
              color: item.disabled ? '#999' : '#333',
              transition: 'background-color 0.1s ease',
            }}
            onMouseEnter={(e) => {
              if (!item.disabled) {
                e.currentTarget.style.backgroundColor = '#f5f5f5';
              }
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            <span className="icon" style={{ fontSize: '16px', width: '16px' }}>
              {item.icon}
            </span>
            <span className="label">{item.label}</span>
            {item.id === 'view-details' && (
              <span style={{ marginLeft: 'auto', fontSize: '12px', color: '#999' }}>
                Enter
              </span>
            )}
            {item.id === 'focus-component' && (
              <span style={{ marginLeft: 'auto', fontSize: '12px', color: '#999' }}>
                F
              </span>
            )}
          </button>
        );
      })}

      {/* Menu Footer with Node Info */}
      <div
        style={{
          padding: '8px 16px',
          borderTop: '1px solid #e0e0e0',
          fontSize: '11px',
          color: '#888',
        }}
      >
        <div>Type: {nodeData.componentData.componentType}</div>
        <div>ID: {nodeId}</div>
        {nodeData.connections.length > 0 && (
          <div>Connections: {nodeData.connections.length}</div>
        )}
      </div>
    </div>
  );
};

export default ContextMenu;