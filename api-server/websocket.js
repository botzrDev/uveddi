// WebSocket server for real-time dashboard updates
const WebSocket = require('ws');

function setupWebSocketServer(server) {
  const wss = new WebSocket.Server({ server });
  
  // Store WebSocket server in global scope for other modules to access
  global.wss = wss;
  
  wss.on('connection', function connection(ws) {
    console.log('WebSocket client connected');
    
    // Send initial state
    ws.send(JSON.stringify({ 
      type: 'connected',
      timestamp: new Date().toISOString()
    }));
    
    // Send any existing notifications to new client
    if (global.analysisNotifications && global.analysisNotifications.length > 0) {
      ws.send(JSON.stringify({
        type: 'notifications',
        data: global.analysisNotifications
      }));
    }
    
    ws.on('message', function incoming(message) {
      try {
        const data = JSON.stringify(message);
        console.log('WebSocket message received:', data);
        
        // Handle client messages if needed
        if (data.type === 'request_update') {
          // Send latest data
        }
      } catch (error) {
        console.error('Error processing WebSocket message:', error);
      }
    });
    
    ws.on('close', function() {
      console.log('WebSocket client disconnected');
    });
  });
  
  return wss;
}

module.exports = { setupWebSocketServer };
