# API Endpoint Status

> **Version**: 0.9.0-alpha  
> **Last Updated**: January 2025  
> **Base URL**: `http://localhost:8080` (when service is running)

## ⚠️ IMPORTANT: API Reality Check

The API server exists but is **highly unstable** and many documented endpoints are **not implemented**. The web dashboard that would consume these APIs is **completely non-functional**.

## Starting the API Server

```bash
# Requires production build with web features
cargo build --release --features=production

# Start services (often crashes or hangs)
uveddi serve --port 8080 --rendering-port 3001

# Expected issues:
# - Port binding failures
# - Service health check timeouts  
# - Rendering service crashes
# - Missing Playwright dependencies
```

## Endpoint Status Overview

### ✅ Working Endpoints

These endpoints actually exist and mostly work:

#### `GET /health`
**Status**: Working  
**Purpose**: Service health check  
**Response**:
```json
{
  "status": "healthy",
  "version": "0.9.0-alpha",
  "timestamp": "2025-01-28T10:00:00Z"
}
```

#### `POST /api/v1/analyze`
**Status**: Partially Working  
**Purpose**: Trigger code analysis  
**Issues**: 
- Requires proper build flags for anti-pattern detection
- May timeout on large codebases
- Response format inconsistent

**Request**:
```json
{
  "path": "/path/to/code",
  "output_format": "json",
  "enable_ai": false
}
```

#### `GET /api/v1/reports/{id}`
**Status**: Partially Working  
**Purpose**: Retrieve analysis reports  
**Issues**:
- Report IDs not always generated correctly
- HTML reports may have broken styling
- Large reports may cause memory issues

---

### ⚠️ Alpha/Experimental Endpoints

These endpoints exist but are unstable:

#### `GET /api/v1/status`
**Status**: Experimental  
**Purpose**: Get analysis status  
**Issues**: Often returns incorrect status

#### `POST /api/v1/config`
**Status**: Experimental  
**Purpose**: Update configuration  
**Issues**: Changes may not persist

#### `GET /api/v1/metrics`
**Status**: Experimental  
**Purpose**: Performance metrics  
**Issues**: Incomplete data, Prometheus integration broken

#### `POST /api/v1/plugins/install`
**Status**: Very Unstable  
**Purpose**: Install WASM plugins  
**Issues**: 
- Frequently crashes server
- Plugin validation fails
- Requires `wasm-plugins` feature

#### `GET /api/v1/plugins`
**Status**: Experimental  
**Purpose**: List installed plugins  
**Issues**: May return empty list even with plugins installed

---

### ❌ Non-Functional/Planned Endpoints

These endpoints are documented but **DO NOT EXIST**:

#### Authentication & Authorization
- `POST /api/v1/auth/login` - ❌ Not implemented
- `POST /api/v1/auth/logout` - ❌ Not implemented  
- `POST /api/v1/auth/refresh` - ❌ Not implemented
- `GET /api/v1/auth/user` - ❌ Not implemented
- `POST /api/v1/auth/register` - ❌ Not implemented

#### Project Management
- `GET /api/v1/projects` - ❌ Not implemented
- `POST /api/v1/projects` - ❌ Not implemented
- `GET /api/v1/projects/{id}` - ❌ Not implemented
- `DELETE /api/v1/projects/{id}` - ❌ Not implemented

#### Dashboard & UI
- `GET /api/v1/dashboard` - ❌ Not implemented
- `GET /api/v1/dashboard/stats` - ❌ Not implemented
- `GET /api/v1/dashboard/trends` - ❌ Not implemented
- `WebSocket /api/v1/ws` - ❌ Not implemented

#### CI/CD Integration
- `POST /api/v1/webhooks/github` - ❌ Not implemented
- `POST /api/v1/webhooks/gitlab` - ❌ Not implemented
- `GET /api/v1/ci/status` - ❌ Not implemented
- `POST /api/v1/ci/trigger` - ❌ Not implemented

#### Advanced Analysis
- `POST /api/v1/analyze/incremental` - ❌ Not implemented
- `POST /api/v1/analyze/compare` - ❌ Not implemented
- `GET /api/v1/analyze/history` - ❌ Not implemented
- `POST /api/v1/analyze/schedule` - ❌ Not implemented

#### Team Collaboration
- `GET /api/v1/teams` - ❌ Not implemented
- `POST /api/v1/comments` - ❌ Not implemented
- `GET /api/v1/notifications` - ❌ Not implemented

---

## Common API Issues

### 1. CORS Errors
**Symptom**: Browser console shows CORS policy errors  
**Cause**: CORS headers not properly configured  
**Workaround**: Use API directly, not from browser

### 2. Connection Refused
**Symptom**: `ECONNREFUSED` errors  
**Cause**: API server not running or crashed  
**Solution**: 
```bash
# Check if service is running
lsof -i :8080

# Restart service
uveddi serve --port 8080
```

### 3. Timeout Errors
**Symptom**: Requests timeout after 30s  
**Cause**: Analysis taking too long  
**Solution**: Analyze smaller codebases or increase timeout

### 4. Invalid JSON Response
**Symptom**: Parse errors on response  
**Cause**: Server returning HTML error pages  
**Solution**: Check server logs for actual error

### 5. 404 Not Found
**Symptom**: Endpoint returns 404  
**Cause**: Endpoint not implemented despite being documented  
**Solution**: Check this document for actual endpoint status

---

## Example API Usage

### Working Example: Basic Health Check
```bash
# This should work
curl http://localhost:8080/health
```

### Working Example: Trigger Analysis
```bash
# This works with proper build
curl -X POST http://localhost:8080/api/v1/analyze \
  -H "Content-Type: application/json" \
  -d '{"path": "./src", "output_format": "json"}'
```

### Non-Working Example: Authentication
```bash
# This WILL NOT WORK - not implemented
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "user", "password": "pass"}'
# Returns: 404 Not Found
```

---

## API Development Status

### Current State (v0.9.0-alpha)
- Basic REST structure implemented
- Health and analysis endpoints partially working
- No authentication or authorization
- No real-time features
- Minimal error handling

### Planned for Beta (v0.9.5)
- Stabilize existing endpoints
- Add basic authentication
- Improve error responses
- Add request validation

### Planned for v1.0
- Full REST API implementation
- GraphQL support
- WebSocket real-time updates
- OpenAPI documentation
- Rate limiting and caching

---

## Testing the API

### Using curl
```bash
# Test health endpoint
curl -i http://localhost:8080/health

# Test analysis (ensure you have production build)
curl -X POST http://localhost:8080/api/v1/analyze \
  -H "Content-Type: application/json" \
  -d '{"path": ".", "output_format": "json"}' \
  -o response.json
```

### Using httpie
```bash
# Install httpie
pip install httpie

# Test endpoints
http GET localhost:8080/health
http POST localhost:8080/api/v1/analyze path=. output_format=json
```

### Expected Responses

#### Successful Analysis
```json
{
  "status": "success",
  "report_id": "abc123",
  "issues_found": 42,
  "files_analyzed": 100
}
```

#### Common Error Response
```json
{
  "error": "Analysis failed",
  "message": "Failed to parse TypeScript file",
  "code": "PARSE_ERROR"
}
```

---

## Important Notes

1. **Documentation vs Reality**: Many endpoints in other docs don't exist
2. **Stability**: Even "working" endpoints may crash the server
3. **Data Persistence**: Analysis results may not persist between restarts
4. **Performance**: Large requests will timeout or crash
5. **Security**: NO authentication or authorization implemented

---

## Getting Help

- Check server logs: `RUST_LOG=debug uveddi serve 2> server.log`
- Report API issues: https://github.com/org/uveddi/issues
- Known Issues: [docs/known-issues.md](./known-issues.md)

---

*This document reflects the ACTUAL API implementation as of January 2025, not the planned or documented features.*