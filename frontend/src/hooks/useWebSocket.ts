/**
 * Custom WebSocket Hook for Real-time Streaming
 *
 * Provides robust WebSocket connection management with automatic reconnection,
 * optimized for our real-time analysis progress and cache monitoring.
 */

import { useEffect, useRef, useState, useCallback } from 'react';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export interface WebSocketOptions {
  onMessage?: (event: MessageEvent) => void;
  onOpen?: (event: Event) => void;
  onClose?: (event: CloseEvent) => void;
  onError?: (event: Event) => void;
  shouldReconnect?: (closeEvent: CloseEvent) => boolean;
  reconnectInterval?: number;
  reconnectAttempts?: number;
  protocols?: string | string[];
}

export interface WebSocketHook {
  sendMessage: (message: string) => void;
  lastMessage: MessageEvent | null;
  connectionState: ConnectionState;
  reconnectCount: number;
  isConnected: boolean;
}

export const useWebSocket = (url: string, options: WebSocketOptions = {}): WebSocketHook => {
  const {
    onMessage,
    onOpen,
    onClose,
    onError,
    shouldReconnect = () => true,
    reconnectInterval = 3000,
    reconnectAttempts = 5,
    protocols
  } = options;

  const [lastMessage, setLastMessage] = useState<MessageEvent | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>('connecting');
  const [reconnectCount, setReconnectCount] = useState(0);

  const websocketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const messageQueueRef = useRef<string[]>([]);

  // Connection management
  const connect = useCallback(() => {
    if (websocketRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      setConnectionState('connecting');

      const ws = new WebSocket(url, protocols);
      websocketRef.current = ws;

      ws.onopen = (event) => {
        setConnectionState('connected');
        setReconnectCount(0);

        // Send queued messages
        while (messageQueueRef.current.length > 0) {
          const message = messageQueueRef.current.shift();
          if (message) {
            ws.send(message);
          }
        }

        onOpen?.(event);
      };

      ws.onmessage = (event) => {
        setLastMessage(event);
        onMessage?.(event);
      };

      ws.onclose = (event) => {
        setConnectionState('disconnected');
        websocketRef.current = null;

        onClose?.(event);

        // Attempt reconnection if appropriate
        if (shouldReconnect(event) && reconnectCount < reconnectAttempts) {
          const timeout = reconnectInterval * Math.pow(1.5, reconnectCount); // Exponential backoff

          reconnectTimeoutRef.current = setTimeout(() => {
            setReconnectCount(prev => prev + 1);
            connect();
          }, timeout);
        }
      };

      ws.onerror = (event) => {
        setConnectionState('error');
        onError?.(event);
      };

    } catch (error) {
      setConnectionState('error');
      console.error('WebSocket connection failed:', error);
    }
  }, [url, protocols, onOpen, onMessage, onClose, onError, shouldReconnect, reconnectInterval, reconnectAttempts, reconnectCount]);

  // Send message with queuing for disconnected state
  const sendMessage = useCallback((message: string) => {
    const ws = websocketRef.current;

    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(message);
    } else {
      // Queue message for when connection is restored
      messageQueueRef.current.push(message);

      // Attempt to reconnect if not already trying
      if (ws?.readyState === WebSocket.CLOSED || ws?.readyState === undefined) {
        connect();
      }
    }
  }, [connect]);

  // Initialize connection
  useEffect(() => {
    connect();

    return () => {
      // Cleanup on unmount
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }

      if (websocketRef.current) {
        websocketRef.current.close();
      }
    };
  }, [connect]);

  // Cleanup reconnect timeout
  useEffect(() => {
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
    };
  }, []);

  return {
    sendMessage,
    lastMessage,
    connectionState,
    reconnectCount,
    isConnected: connectionState === 'connected'
  };
};