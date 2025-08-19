// API endpoint for receiving analysis notifications
app.post('/api/notifications/new-analysis', (req, res) => {
  try {
    const notification = req.body;
    
    // Log the notification
    console.log(`Received new analysis notification: ${JSON.stringify(notification)}`);
    
    // Store notification in memory for dashboard to retrieve
    if (!global.analysisNotifications) {
      global.analysisNotifications = [];
    }
    
    global.analysisNotifications.push({
      ...notification,
      received: new Date().toISOString()
    });
    
    // Keep only the latest 10 notifications
    if (global.analysisNotifications.length > 10) {
      global.analysisNotifications = global.analysisNotifications.slice(-10);
    }
    
    // Broadcast to any connected WebSocket clients
    if (global.wss) {
      global.wss.clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
          client.send(JSON.stringify({
            type: 'notification',
            data: notification
          }));
        }
      });
    }
    
    res.status(200).json({ status: 'success' });
  } catch (error) {
    console.error('Error processing analysis notification:', error);
    res.status(500).json({ status: 'error', message: error.message });
  }
});
