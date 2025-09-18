# Assignment 07: Refactor API Server God Object

## Priority: HIGH
## Estimated Time: 4-5 hours
## File: `/api-server/server.js` (33,693 lines - Critical God Object)

## Objective
Break down the massive Node.js API server into manageable, focused modules.

## Current Problems
- Single file contains 33,693 lines (extreme God Object)
- Mixes routing, business logic, WebSocket handling, and middleware
- Impossible to maintain or test individual components
- High coupling between unrelated features

## Tasks

### 1. Create API Server Module Structure
```
api-server/
├── server.js (<100 lines - main entry point)
├── src/
│   ├── routes/
│   │   ├── index.js
│   │   ├── analysis.js
│   │   ├── projects.js
│   │   ├── auth.js
│   │   └── admin.js
│   ├── controllers/
│   │   ├── analysisController.js
│   │   ├── projectController.js
│   │   ├── authController.js
│   │   └── adminController.js
│   ├── services/
│   │   ├── analysisService.js
│   │   ├── projectService.js
│   │   ├── authService.js
│   │   └── cacheService.js
│   ├── middleware/
│   │   ├── auth.js
│   │   ├── cors.js
│   │   ├── validation.js
│   │   └── errorHandler.js
│   ├── websocket/
│   │   ├── index.js
│   │   ├── handlers/
│   │   └── events/
│   ├── config/
│   │   ├── database.js
│   │   ├── auth.js
│   │   └── app.js
│   └── utils/
│       ├── logger.js
│       ├── response.js
│       └── validation.js
├── tests/
│   ├── routes/
│   ├── controllers/
│   └── services/
└── docs/
    └── api.md
```

### 2. Extract Route Definitions
Break routes into logical groups:
- `routes/analysis.js`: All analysis-related endpoints (<200 lines)
- `routes/projects.js`: Project management endpoints (<150 lines)
- `routes/auth.js`: Authentication endpoints (<100 lines)
- `routes/admin.js`: Admin functionality (<150 lines)

### 3. Create Controllers
Implement controller pattern:
```javascript
// controllers/analysisController.js
class AnalysisController {
    async createAnalysis(req, res) {
        // Handle analysis creation
    }

    async getAnalysis(req, res) {
        // Handle analysis retrieval
    }

    async listAnalyses(req, res) {
        // Handle analysis listing
    }
}
```
Target: <250 lines per controller

### 4. Extract Services
Create business logic services:
- `services/analysisService.js`: Analysis business logic (<300 lines)
- `services/projectService.js`: Project management logic (<200 lines)
- `services/authService.js`: Authentication logic (<200 lines)
- `services/cacheService.js`: Caching logic (<150 lines)

### 5. Extract Middleware
- `middleware/auth.js`: Authentication middleware (<100 lines)
- `middleware/cors.js`: CORS configuration (<50 lines)
- `middleware/validation.js`: Request validation (<150 lines)
- `middleware/errorHandler.js`: Error handling (<100 lines)

### 6. Refactor WebSocket Handling
```
websocket/
├── index.js (WebSocket server setup)
├── handlers/
│   ├── analysisHandler.js
│   ├── projectHandler.js
│   └── notificationHandler.js
└── events/
    ├── analysisEvents.js
    └── systemEvents.js
```

### 7. Extract Configuration
- `config/database.js`: Database configuration
- `config/auth.js`: Authentication configuration
- `config/app.js`: Application configuration

### 8. Create Utilities
- `utils/logger.js`: Logging utilities
- `utils/response.js`: Response formatting
- `utils/validation.js`: Validation helpers

### 9. Update Package Structure
- Move to proper module structure
- Update package.json dependencies
- Create proper TypeScript support (optional but recommended)

### 10. Add Comprehensive Testing
- Unit tests for controllers
- Integration tests for routes
- WebSocket testing
- API documentation updates

## TypeScript Migration (Optional Enhancement)
Consider migrating to TypeScript for better maintainability:
- Add TypeScript configuration
- Create type definitions
- Gradually migrate modules

## Success Criteria
- [ ] Original file reduced from 33,693 to <100 lines
- [ ] All new modules <300 lines
- [ ] Clear separation of concerns
- [ ] All existing functionality preserved
- [ ] Tests pass
- [ ] API documentation updated
- [ ] WebSocket functionality working

## Breaking Changes
- File structure changes (but API endpoints remain same)
- Some internal module interfaces may change
- Environment variable configuration may change

## Verification Commands
```bash
# Check file sizes
find api-server/src -name "*.js" -exec wc -l {} + | sort -nr

# Run tests
npm test

# Start server and test endpoints
npm start &
curl http://localhost:3000/api/health

# Test WebSocket functionality
node test/websocket-test.js
```

## Performance Considerations
- Ensure startup time not significantly impacted
- WebSocket performance maintained
- Memory usage optimized
- Request handling performance preserved

## Completion Notes
_To be filled by AI developer:_
- Files created: ___
- Lines reduced from 33,693 to: ___
- Modules extracted: ___
- TypeScript migration status: ___
- Performance impact: ___
- Breaking changes documented: ___