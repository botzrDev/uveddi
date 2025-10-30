# REST API Reference

## Overview

The Uveddi REST API provides programmatic access to code analysis features, enabling integration with CI/CD pipelines, dashboards, and custom tools. The API follows RESTful principles with JSON-based request/response formats.

## Base URL

```
http://localhost:8888/api/v1
```

## Authentication

> **Note**: Authentication is currently disabled for local development. Production deployments should enable authentication using JWT tokens or API keys.

### Headers

```http
Content-Type: application/json
Accept: application/json
Authorization: Bearer <token>  # Future implementation
```

## Rate Limiting

- **Default**: 100 requests per minute per IP
- **Burst**: Up to 10 concurrent requests
- **Headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

## Error Response Format

All errors follow a consistent JSON structure:

```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Report with ID 'xyz' not found",
    "details": {
      "requested_id": "xyz",
      "timestamp": "2025-01-23T10:30:00Z"
    }
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|------------|-------------|
| `RESOURCE_NOT_FOUND` | 404 | Requested resource does not exist |
| `INVALID_REQUEST` | 400 | Request format or parameters are invalid |
| `UNAUTHORIZED` | 401 | Authentication required or failed |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server-side processing error |

## Endpoints

### Health Check

Check API server health and availability.

#### `GET /health`

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-23T10:30:00Z",
  "api_version": "v1",
  "schema_version": "1.0.0"
}
```

**Status Codes:**
- `200 OK` - Service is healthy
- `503 Service Unavailable` - Service is unhealthy

**Example:**
```bash
curl http://localhost:8888/health
```

---

### Reports Management

#### List Reports

Get a list of all available analysis reports.

##### `GET /api/v1/reports`

**Query Parameters:**
- `limit` (optional, integer): Maximum number of reports to return (default: 10, max: 100)
- `offset` (optional, integer): Number of reports to skip for pagination (default: 0)
- `sort` (optional, string): Sort order - `created_asc`, `created_desc` (default: `created_desc`)

**Response:**
```json
{
  "reports": [
    {
      "id": "demo",
      "title": "Demo Analysis Report",
      "created_at": "2025-01-23T10:30:00Z",
      "project_name": "Demo Project",
      "file_count": 42,
      "issue_count": 7,
      "is_demo": true
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

**Example:**
```bash
curl "http://localhost:8888/api/v1/reports?limit=5&sort=created_desc"
```

---

#### Get Report

Retrieve a specific analysis report by ID.

##### `GET /api/v1/reports/{id}`

**Path Parameters:**
- `id` (required, string): Report identifier

**Response:**
```json
{
  "schema_version": "1.0.0",
  "project": {
    "id": "demo-project",
    "name": "Demo Project",
    "commit": "abc123",
    "branch": "main",
    "repo_url": "https://github.com/example/demo",
    "path": "./demo",
    "languages": ["rust"]
  },
  "summary": {
    "coverage": 82.5,
    "issues_total": 7,
    "issues_by_severity": {
      "critical": 1,
      "high": 3,
      "medium": 3,
      "low": 0
    },
    "issues_by_category": {
      "anti-patterns": 4,
      "code-quality": 2,
      "security": 1
    },
    "files_analyzed": 42,
    "components_analyzed": 15,
    "analysis_duration_ms": 1250,
    "time_generated": "2025-01-23T10:30:00Z"
  },
  "findings": [
    {
      "id": "demo-001",
      "finding_type": "GodObject",
      "severity": "critical",
      "title": "Large class with too many responsibilities",
      "message": "The UserManager class has grown too large...",
      "file": "src/user_manager.rs",
      "start_line": 45,
      "end_line": 287,
      "confidence": 0.89,
      "ai_explanation": "This class violates the Single Responsibility Principle...",
      "recommendation": "Extract authentication logic into AuthService..."
    }
  ],
  "dependency_graph": {},
  "diagrams": [],
  "ai_insights": {},
  "metadata": {}
}
```

**Status Codes:**
- `200 OK` - Report found and returned
- `404 Not Found` - Report does not exist

**Example:**
```bash
curl http://localhost:8888/api/v1/reports/demo
```

---

#### Get Demo Report

Retrieve the demo report for testing and exploration.

##### `GET /api/v1/reports/demo`

**Response:** Same as Get Report endpoint

**Example:**
```bash
curl http://localhost:8888/api/v1/reports/demo
```

---

### Dependency Analysis

#### Get Dependency Graph

Retrieve the dependency graph for a specific report.

##### `GET /api/v1/reports/{id}/graphs/dependency`

**Path Parameters:**
- `id` (required, string): Report identifier

**Query Parameters:**
- `format` (optional, string): Graph format - `cytoscape` (default), `d3`, `raw`
- `depth` (optional, integer): Maximum depth to traverse (default: unlimited)
- `filter` (optional, string): Filter nodes by type - `all` (default), `modules`, `functions`, `classes`

**Response:**
```json
{
  "nodes": [
    {
      "id": "main",
      "label": "main.rs",
      "path": "src/main.rs",
      "node_type": "module",
      "metrics": {
        "loc": 150,
        "complexity": 5.0,
        "dependencies": 3,
        "dependents": 0
      },
      "group": "core",
      "properties": {}
    }
  ],
  "edges": [
    {
      "source": "main",
      "target": "user_manager",
      "edge_type": "imports",
      "weight": 3.0,
      "properties": {}
    }
  ],
  "metadata": {
    "node_count": 2,
    "edge_count": 1,
    "has_cycles": false,
    "max_depth": 2,
    "suggested_layout": "dagre",
    "cycles": []
  }
}
```

**Example:**
```bash
curl "http://localhost:8888/api/v1/reports/demo/graphs/dependency?format=cytoscape&depth=3"
```

---

### Security Analysis

#### List Security Issues

Get all security issues from the latest analysis.

##### `GET /api/v1/security/issues`

**Query Parameters:**
- `severity` (optional, string): Filter by severity - `critical`, `high`, `medium`, `low`
- `category` (optional, string): Filter by OWASP category
- `confidence_min` (optional, float): Minimum confidence score (0.0-1.0)

**Response:**
```json
[
  {
    "id": "sec_001",
    "issue_type": "Injection",
    "severity": "Critical",
    "confidence_score": 0.95,
    "location": {
      "file": "src/auth.py",
      "start_line": 42,
      "end_line": 42,
      "start_column": 15,
      "end_column": 35
    },
    "description": "SQL injection vulnerability detected in user authentication",
    "remediation": "Use parameterized queries to prevent SQL injection",
    "owasp_category": "A03_Injection",
    "cwe_id": "CWE-89",
    "cvss_score": 9.8,
    "references": [
      "https://owasp.org/www-project-top-ten/2017/A1_2017-Injection"
    ],
    "taint_flows": []
  }
]
```

**Example:**
```bash
curl "http://localhost:8888/api/v1/security/issues?severity=critical&confidence_min=0.8"
```

---

#### Get Security Issue

Retrieve details of a specific security issue.

##### `GET /api/v1/security/issues/{id}`

**Path Parameters:**
- `id` (required, string): Security issue identifier

**Response:** Same structure as individual issue in List Security Issues

**Example:**
```bash
curl http://localhost:8888/api/v1/security/issues/sec_001
```

---

#### Get Security Summary

Get aggregate security analysis statistics.

##### `GET /api/v1/security/summary`

**Response:**
```json
{
  "total_issues": 42,
  "critical_count": 3,
  "high_count": 8,
  "medium_count": 20,
  "low_count": 11,
  "owasp_coverage": {
    "A01_Broken_Access_Control": {
      "issues_found": 5,
      "coverage_percentage": 90.0,
      "avg_confidence": 0.85
    },
    "A03_Injection": {
      "issues_found": 8,
      "coverage_percentage": 95.0,
      "avg_confidence": 0.90
    }
  },
  "confidence_distribution": {
    "high": 25,
    "medium": 12,
    "low": 5
  },
  "most_common_issues": [
    {
      "issue_type": "Injection",
      "count": 8,
      "avg_severity": "High"
    }
  ]
}
```

**Example:**
```bash
curl http://localhost:8888/api/v1/security/summary
```

---

#### Get OWASP Coverage

Get OWASP Top 10 detection coverage statistics.

##### `GET /api/v1/security/owasp-coverage`

**Response:**
```json
{
  "A01_Broken_Access_Control": {
    "issues_found": 5,
    "coverage_percentage": 90.0,
    "avg_confidence": 0.85
  },
  "A02_Cryptographic_Failures": {
    "issues_found": 2,
    "coverage_percentage": 70.0,
    "avg_confidence": 0.95
  },
  "A03_Injection": {
    "issues_found": 8,
    "coverage_percentage": 95.0,
    "avg_confidence": 0.90
  }
}
```

**Example:**
```bash
curl http://localhost:8888/api/v1/security/owasp-coverage
```

---

#### Get Taint Flows

Get taint flow analysis showing data flow from sources to sinks.

##### `GET /api/v1/security/taint-flows`

**Query Parameters:**
- `vulnerability_type` (optional, string): Filter by vulnerability type
- `confidence_min` (optional, float): Minimum confidence score

**Response:**
```json
[
  {
    "source": {
      "name": "user_input",
      "location": "src/auth.py:35",
      "node_type": "UserInput"
    },
    "sink": {
      "name": "sql_execute",
      "location": "src/auth.py:42",
      "node_type": "SqlQuery"
    },
    "confidence": 0.95,
    "sanitizers": []
  }
]
```

**Example:**
```bash
curl "http://localhost:8888/api/v1/security/taint-flows?vulnerability_type=injection"
```

---

#### Export SARIF Report

Export security findings in SARIF (Static Analysis Results Interchange Format) format.

##### `GET /api/v1/security/sarif`

**Query Parameters:**
- `run_id` (optional, string): Analysis run ID (defaults to latest)

**Response Headers:**
```http
Content-Type: application/json
Content-Disposition: attachment; filename="security-analysis.sarif"
```

**Response:**
```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "Uveddi Security Detector",
          "version": "0.9.0",
          "informationUri": "https://github.com/uveddi/uveddi",
          "rules": []
        }
      },
      "results": []
    }
  ]
}
```

**Example:**
```bash
curl -O http://localhost:8888/api/v1/security/sarif
```

---

## WebSocket API

The WebSocket API provides real-time updates during analysis operations.

### Connection

```javascript
const ws = new WebSocket('ws://localhost:8888/ws');
```

### Message Format

All WebSocket messages use JSON format:

```json
{
  "type": "analysis_progress",
  "data": {
    "files_analyzed": 42,
    "total_files": 100,
    "current_file": "src/main.rs",
    "percentage": 42.0
  },
  "timestamp": "2025-01-23T10:30:00Z"
}
```

### Message Types

| Type | Description | Data Fields |
|------|-------------|------------|
| `connection_established` | Connection confirmed | `session_id`, `version` |
| `analysis_started` | Analysis began | `run_id`, `project_name` |
| `analysis_progress` | Progress update | `files_analyzed`, `total_files`, `percentage` |
| `issue_detected` | New issue found | `issue_id`, `type`, `severity`, `file` |
| `analysis_completed` | Analysis finished | `run_id`, `total_issues`, `duration_ms` |
| `error` | Error occurred | `code`, `message`, `details` |

### Client Example

```javascript
const ws = new WebSocket('ws://localhost:8888/ws');

ws.onopen = () => {
  console.log('Connected to Uveddi WebSocket');
  ws.send(JSON.stringify({
    type: 'subscribe',
    channels: ['analysis_progress', 'issue_detected']
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  switch(message.type) {
    case 'analysis_progress':
      updateProgressBar(message.data.percentage);
      break;
    case 'issue_detected':
      addIssueToList(message.data);
      break;
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('WebSocket connection closed');
};
```

---

## Code Examples

### JavaScript/TypeScript

```javascript
// Using fetch API
async function getReport(reportId) {
  const response = await fetch(`http://localhost:8888/api/v1/reports/${reportId}`);
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  return await response.json();
}

// Using axios
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8888/api/v1',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  }
});

async function listReports() {
  const { data } = await api.get('/reports');
  return data.reports;
}
```

### Python

```python
import requests
import json

BASE_URL = "http://localhost:8888/api/v1"

def get_security_summary():
    """Get security analysis summary"""
    response = requests.get(f"{BASE_URL}/security/summary")
    response.raise_for_status()
    return response.json()

def export_sarif_report(output_file="security.sarif"):
    """Export security findings in SARIF format"""
    response = requests.get(f"{BASE_URL}/security/sarif")
    response.raise_for_status()
    
    with open(output_file, 'w') as f:
        json.dump(response.json(), f, indent=2)
    
    return output_file

# WebSocket example using websocket-client
import websocket
import json

def on_message(ws, message):
    data = json.loads(message)
    print(f"Received: {data['type']}")

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("WebSocket closed")

def on_open(ws):
    ws.send(json.dumps({
        "type": "subscribe",
        "channels": ["analysis_progress"]
    }))

ws = websocket.WebSocketApp("ws://localhost:8888/ws",
                            on_open=on_open,
                            on_message=on_message,
                            on_error=on_error,
                            on_close=on_close)

ws.run_forever()
```

### cURL

```bash
# List all reports
curl -X GET http://localhost:8888/api/v1/reports

# Get specific report
curl -X GET http://localhost:8888/api/v1/reports/demo

# Get security issues with filters
curl -X GET "http://localhost:8888/api/v1/security/issues?severity=critical"

# Export SARIF report
curl -X GET http://localhost:8888/api/v1/security/sarif \
  -H "Accept: application/json" \
  -o security-report.sarif

# Pretty print JSON response
curl -X GET http://localhost:8888/api/v1/security/summary | jq '.'
```

### CI/CD Integration

```yaml
# GitHub Actions example
name: Security Analysis

on: [push, pull_request]

jobs:
  uveddi-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Start Uveddi Server
        run: |
          docker run -d -p 8888:8888 uveddi/uveddi:latest serve
          sleep 5  # Wait for server to start
      
      - name: Run Analysis
        run: |
          curl -X POST http://localhost:8888/api/v1/analyze \
            -H "Content-Type: application/json" \
            -d '{"path": "./src", "enable_ai": true}'
      
      - name: Check Security Issues
        run: |
          CRITICAL_COUNT=$(curl -s http://localhost:8888/api/v1/security/summary | jq '.critical_count')
          if [ "$CRITICAL_COUNT" -gt 0 ]; then
            echo "Critical security issues found!"
            exit 1
          fi
      
      - name: Export SARIF Report
        run: |
          curl -s http://localhost:8888/api/v1/security/sarif > security.sarif
      
      - name: Upload SARIF to GitHub
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: security.sarif
```

---

## Response Headers

All API responses include standard headers:

```http
X-Request-ID: uuid-v4
X-Response-Time: 123ms
X-API-Version: v1
Cache-Control: no-cache, no-store, must-revalidate
Content-Type: application/json; charset=utf-8
```

---

## Pagination

Endpoints that return lists support pagination:

```http
GET /api/v1/reports?limit=10&offset=20
```

Response includes pagination metadata:

```json
{
  "data": [...],
  "pagination": {
    "total": 100,
    "limit": 10,
    "offset": 20,
    "has_more": true,
    "next_offset": 30,
    "prev_offset": 10
  }
}
```

---

## Filtering and Sorting

Many endpoints support filtering and sorting:

```http
GET /api/v1/security/issues?severity=critical&sort=confidence_desc
```

Common filter parameters:
- `severity`: Filter by severity level
- `category`: Filter by category
- `confidence_min`: Minimum confidence score
- `date_from`: Start date (ISO 8601)
- `date_to`: End date (ISO 8601)

Common sort options:
- `created_asc`: Oldest first
- `created_desc`: Newest first
- `severity_asc`: Lowest severity first
- `severity_desc`: Highest severity first
- `confidence_asc`: Lowest confidence first
- `confidence_desc`: Highest confidence first

---

## API Versioning

The API uses URL-based versioning:

- Current version: `v1`
- Base URL: `/api/v1`
- Version header: `X-API-Version: v1`

Future versions will maintain backward compatibility for at least 6 months after deprecation notice.

---

## Status Codes Summary

| Code | Description | Usage |
|------|-------------|-------|
| 200 | OK | Successful GET request |
| 201 | Created | Resource successfully created |
| 204 | No Content | Successful DELETE request |
| 400 | Bad Request | Invalid request format or parameters |
| 401 | Unauthorized | Authentication required or failed |
| 403 | Forbidden | Access denied to resource |
| 404 | Not Found | Resource does not exist |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | Service temporarily unavailable |

---

## Support

For API support and bug reports:
- GitHub Issues: https://github.com/botzrDev/uveddi/issues
- Documentation: https://docs.uveddi.io
- Community Discord: https://discord.gg/uveddi