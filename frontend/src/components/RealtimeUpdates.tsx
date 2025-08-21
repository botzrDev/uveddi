import NotificationsActiveIcon from '@mui/icons-material/NotificationsActive';
import RefreshIcon from '@mui/icons-material/Refresh';
import {
    Alert,
    AlertTitle,
    Box,
    Button,
    Slide,
    Snackbar,
    Typography
} from '@mui/material';
import { useEffect, useState } from 'react';

// WebSocket connection for real-time updates
const WEBSOCKET_URL = window.location.hostname === 'localhost' 
  ? `ws://${window.location.hostname}:8888`
  : `ws://${window.location.host}`;

interface RealtimeUpdatesProps {
  onNewAnalysis?: (data: any) => void;
}

interface NotificationData {
  type: string;
  timestamp: string;
  metadata: {
    files_analyzed: number;
    issues_found: number;
    analysis_duration_ms: number;
    ai_enhanced: boolean;
    format: string;
  };
}

export function RealtimeUpdates({ onNewAnalysis }: RealtimeUpdatesProps) {
  const [notification, setNotification] = useState<NotificationData | null>(null);
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  // Check if WebSocket is disabled
  const isWebSocketDisabled = import.meta.env.VITE_DISABLE_WEBSOCKET === 'true';

  // Initialize WebSocket connection
  useEffect(() => {
    if (isWebSocketDisabled) {
      console.log('WebSocket disabled via environment variable');
      return;
    }
    const ws = new WebSocket(WEBSOCKET_URL);
    
    ws.onopen = () => {
      console.log('WebSocket connection established');
      setIsConnected(true);
      setSocket(ws);
    };
    
    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        console.log('WebSocket message received:', message);
        
        if (message.type === 'notification' && message.data.type === 'analysis_complete') {
          // Show notification for new analysis results
          setNotification(message.data);
        }
      } catch (error) {
        console.error('Error processing WebSocket message:', error);
      }
    };
    
    ws.onclose = () => {
      console.log('WebSocket connection closed');
      setIsConnected(false);
      setSocket(null);
      
      // Attempt to reconnect after 5 seconds
      setTimeout(() => {
        console.log('Attempting to reconnect WebSocket...');
      }, 5000);
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
    
    // Cleanup on component unmount
    return () => {
      if (ws) {
        ws.close();
      }
    };
  }, []);
  
  // Handle refresh request when user clicks the notification
  const handleRefresh = () => {
    if (onNewAnalysis && notification) {
      onNewAnalysis(notification);
    }
    setNotification(null);
  };
  
  // Close notification without refreshing
  const handleClose = () => {
    setNotification(null);
  };
  
  return (
    <>
      {/* Connection status indicator */}
      {!isConnected && !isWebSocketDisabled && (
        <Box sx={{ position: 'fixed', bottom: 10, left: 10, zIndex: 1000 }}>
          <Alert severity="warning" variant="filled" sx={{ width: '100%' }}>
            <AlertTitle>Disconnected</AlertTitle>
            Real-time updates are currently unavailable
          </Alert>
        </Box>
      )}
      
      {/* New analysis notification */}
      <Snackbar
        anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
        open={!!notification}
        TransitionComponent={Slide}
      >
        <Alert
          severity="info"
          variant="filled"
          icon={<NotificationsActiveIcon />}
          action={
            <Button 
              color="inherit" 
              size="small"
              startIcon={<RefreshIcon />}
              onClick={handleRefresh}
            >
              Refresh
            </Button>
          }
          onClose={handleClose}
          sx={{ width: '100%' }}
        >
          <AlertTitle>New Analysis Results</AlertTitle>
          <Typography variant="body2">
            {notification?.metadata?.files_analyzed || 0} files analyzed, 
            {notification?.metadata?.issues_found || 0} issues found
          </Typography>
        </Alert>
      </Snackbar>
    </>
  );
}

export default RealtimeUpdates;
