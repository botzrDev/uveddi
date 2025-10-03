# Uveddi API Server

**Version:** 1.0.0
**Port:** 8000 (default)
**Technology:** Node.js + Express

## Overview

The Uveddi API Server provides RESTful endpoints for the web dashboard and external integrations. It serves analysis reports, manages configurations, and provides real-time updates via optional WebSocket support.

## Quick Start

### Installation

```bash
cd api-server
npm install
```

### Configuration

The server uses the following default configuration:

| Variable    | Default     | Description                |
|-------------|-------------|----------------------------|
| PORT        | 8000        | HTTP server port           |
| NODE_ENV    | development | Environment mode           |
| REPORTS_DIR | ../reports  | Analysis reports directory |

### Running

```bash
# Development (with auto-reload)
npm run dev

# Production
npm start

# With custom port
PORT=3000 npm start
```

### Testing

```bash
# Health check
curl http://localhost:8000/health

# Get latest report
curl http://localhost:8000/api/v1/reports/latest

# List all reports
curl http://localhost:8000/api/v1/reports
```

## API Endpoints

### Health & Status

**GET /health**

Returns server health status and version information.

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2025-10-03T12:00:00.000Z",
  "uptime": 123.45
}
```

### Reports

**GET /api/v1/reports/latest**

Retrieves the most recent analysis report.

**Response:**
```json
{
  "id": "report-2025-10-03-120000",
  "timestamp": "2025-10-03T12:00:00.000Z",
  "summary": {
    "total_files": 150,
    "total_issues": 42,
    "critical_issues": 5
  },
  "data": { ... }
}
```

**GET /api/v1/reports/:id**

Retrieves a specific report by ID.

**Parameters:**
- `id` (string): Report identifier

**Response:** Same as `/latest`

**GET /api/v1/reports**

Lists all available reports with pagination.

**Query Parameters:**
- `page` (number, default: 1): Page number
- `limit` (number, default: 20): Items per page

**Response:**
```json
{
  "reports": [
    {
      "id": "report-2025-10-03-120000",
      "timestamp": "2025-10-03T12:00:00.000Z",
      "summary": { ... }
    }
  ],
  "total": 100,
  "page": 1,
  "pages": 5
}
```

**GET /api/v1/reports/:id/export**

Exports a report in the specified format.

**Query Parameters:**
- `format` (string): Export format (json, html, markdown, sarif)

**Response:** Report data in requested format

### Documentation

**GET /api/docs/structure**

Returns API documentation structure.

**Response:**
```json
{
  "endpoints": [ ... ],
  "version": "1.0.0",
  "openapi": "3.0.0"
}
```

## WebSocket Support

WebSocket support is available but disabled by default. To enable:

1. Set `ENABLE_WEBSOCKET=true` in environment
2. Frontend connects via `ws://localhost:8000`

**Events:**
- `analysis:started` - Analysis has begun
- `analysis:progress` - Progress update with percentage
- `analysis:completed` - Analysis finished with results
- `analysis:error` - Analysis encountered an error

**Note:** WebSocket is experimental and may be unstable.

## CORS Configuration

The server accepts all origins in development mode. For production:

```javascript
// File: server.js
app.use(cors({
  origin: ['https://yourdomain.com'],
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
  allowedHeaders: ['Content-Type', 'Authorization']
}));
```

## Error Handling

All endpoints return consistent error responses:

```json
{
  "error": {
    "code": "REPORT_NOT_FOUND",
    "message": "Report with ID 'xyz' not found",
    "timestamp": "2025-10-03T12:00:00.000Z"
  }
}
```

**Common Error Codes:**
- `REPORT_NOT_FOUND` (404): Report doesn't exist
- `INVALID_FORMAT` (400): Invalid export format
- `SERVER_ERROR` (500): Internal server error

## Troubleshooting

### Port Already in Use

```bash
# Find process using port 8000
lsof -ti:8000

# Kill the process
lsof -ti:8000 | xargs kill -9

# Or use a different port
PORT=8001 npm start
```

### Reports Not Loading

- Verify `REPORTS_DIR` points to correct directory
- Check file permissions
- Ensure reports are valid JSON format

## Development

### Adding New Endpoints

1. Create route file in `routes/`
2. Implement controller in `src/controllers/`
3. Register route in `routes/index.js`

**Example:**
```javascript
// routes/newFeature.js
const express = require('express');
const router = express.Router();

router.get('/api/v1/newfeature', (req, res) => {
  res.json({ message: 'New feature' });
});

module.exports = router;
```

### Testing with Frontend

The Vite dev server proxies `/api` requests to this server:

```typescript
// frontend/vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8000',
      changeOrigin: true
    }
  }
}
```

## Production Deployment

### Docker

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
EXPOSE 8000
CMD ["node", "server.js"]
```

### PM2

```bash
npm install -g pm2
pm2 start server.js --name uveddi-api
pm2 save
pm2 startup
```

### Environment Variables

**Production Checklist:**
- Set `NODE_ENV=production`
- Configure `REPORTS_DIR` absolute path
- Set appropriate `PORT`
- Configure CORS origins
- Enable rate limiting
- Set up SSL/TLS certificates
- Configure logging

## Security

### Rate Limiting

```javascript
const rateLimit = require('express-rate-limit');

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100 // limit each IP to 100 requests per windowMs
});

app.use('/api/', limiter);
```

### Authentication (Optional)

```javascript
const jwt = require('jsonwebtoken');

function authenticateToken(req, res, next) {
  const token = req.headers['authorization'];
  if (!token) return res.sendStatus(401);

  jwt.verify(token, process.env.JWT_SECRET, (err, user) => {
    if (err) return res.sendStatus(403);
    req.user = user;
    next();
  });
}

app.use('/api/v1/reports', authenticateToken);
```

## License

MIT License - See main repository for details
