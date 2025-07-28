# Security API Reference

## Overview

This document provides comprehensive API reference for the Uveddi Security and RBAC system (UV-247). The security system provides enterprise-grade authentication, authorization, audit logging, and rate limiting capabilities.

## Table of Contents

1. [Authentication API](#authentication-api)
2. [Authorization API](#authorization-api)
3. [User Management API](#user-management-api)
4. [Audit API](#audit-api)
5. [Rate Limiting API](#rate-limiting-api)
6. [Configuration API](#configuration-api)
7. [Health and Monitoring API](#health-and-monitoring-api)
8. [Error Responses](#error-responses)

## Base URL

```
Production: https://api.uveddi.com/v1
Development: https://dev-api.uveddi.com/v1
```

## Authentication

All API requests (except authentication endpoints) require authentication via one of the following methods:

### JWT Bearer Token
```http
Authorization: Bearer <jwt_token>
```

### API Key
```http
Authorization: ApiKey <api_key>
```

## Authentication API

### POST /auth/login

Authenticate user with email and password.

#### Request
```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 28800,
  "refresh_token": "refresh_token_here",
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "display_name": "John Doe",
    "roles": ["Developer"]
  }
}
```

#### Error Responses
- `400 Bad Request`: Invalid request format
- `401 Unauthorized`: Invalid credentials
- `429 Too Many Requests`: Rate limit exceeded

### POST /auth/refresh

Refresh an expired JWT token.

#### Request
```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "refresh_token_here"
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "access_token": "new_jwt_token_here",
  "token_type": "Bearer",
  "expires_in": 28800
}
```

### POST /auth/logout

Logout user and invalidate tokens.

#### Request
```http
POST /auth/logout
Authorization: Bearer <jwt_token>
```

#### Response
```http
HTTP/1.1 204 No Content
```

### GET /auth/me

Get current authenticated user information.

#### Request
```http
GET /auth/me
Authorization: Bearer <jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "email": "user@example.com",
  "display_name": "John Doe",
  "roles": ["Developer"],
  "permissions": [
    {
      "resource": "projects",
      "actions": ["read", "write"],
      "scope": "own"
    }
  ],
  "last_login": "2024-01-15T10:30:00Z",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### POST /auth/oauth/{provider}

Initiate OAuth authentication flow.

#### Request
```http
POST /auth/oauth/google
Content-Type: application/json

{
  "redirect_uri": "https://your-app.com/auth/callback"
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "authorization_url": "https://accounts.google.com/oauth/authorize?client_id=...",
  "state": "random_state_value"
}
```

### POST /auth/oauth/{provider}/callback

Complete OAuth authentication flow.

#### Request
```http
POST /auth/oauth/google/callback
Content-Type: application/json

{
  "code": "authorization_code_from_provider",
  "state": "random_state_value"
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "access_token": "jwt_token_here",
  "token_type": "Bearer",
  "expires_in": 28800,
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "display_name": "John Doe",
    "roles": ["Developer"]
  }
}
```

## Authorization API

### POST /auth/check-permission

Check if user has specific permission.

#### Request
```http
POST /auth/check-permission
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "resource": "projects",
  "action": "write",
  "context": {
    "project_id": "proj_123",
    "team_id": "team_456"
  }
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "allowed": true,
  "reason": "User has Developer role with write access to own projects",
  "policy_matched": "Developer:projects:write:own"
}
```

### GET /auth/permissions

Get all permissions for current user.

#### Request
```http
GET /auth/permissions
Authorization: Bearer <jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "permissions": [
    {
      "resource": "projects",
      "actions": ["read", "write"],
      "scope": "own"
    },
    {
      "resource": "reports",
      "actions": ["read"],
      "scope": "team"
    }
  ],
  "roles": ["Developer"],
  "effective_permissions": {
    "projects": {
      "read": true,
      "write": true,
      "delete": false
    },
    "reports": {
      "read": true,
      "write": false,
      "delete": false
    }
  }
}
```

## User Management API

### GET /users

List users (Admin only).

#### Request
```http
GET /users?page=1&limit=20&role=Developer
Authorization: Bearer <admin_jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "users": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "display_name": "John Doe",
      "roles": ["Developer"],
      "is_active": true,
      "last_login": "2024-01-15T10:30:00Z",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150,
    "total_pages": 8
  }
}
```

### POST /users

Create new user (Admin only).

#### Request
```http
POST /users
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json

{
  "email": "newuser@example.com",
  "display_name": "Jane Smith",
  "roles": ["QA"],
  "send_invitation": true
}
```

#### Response
```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "id": "456e7890-e89b-12d3-a456-426614174001",
  "email": "newuser@example.com",
  "display_name": "Jane Smith",
  "roles": ["QA"],
  "is_active": true,
  "created_at": "2024-01-15T10:30:00Z",
  "invitation_sent": true
}
```

### PUT /users/{user_id}

Update user (Admin only or own profile).

#### Request
```http
PUT /users/123e4567-e89b-12d3-a456-426614174000
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "display_name": "John Smith",
  "roles": ["Developer", "QA"]
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "email": "user@example.com",
  "display_name": "John Smith",
  "roles": ["Developer", "QA"],
  "is_active": true,
  "updated_at": "2024-01-15T10:30:00Z"
}
```

### DELETE /users/{user_id}

Deactivate user (Admin only).

#### Request
```http
DELETE /users/123e4567-e89b-12d3-a456-426614174000
Authorization: Bearer <admin_jwt_token>
```

#### Response
```http
HTTP/1.1 204 No Content
```

### POST /users/{user_id}/api-keys

Generate API key for user.

#### Request
```http
POST /users/123e4567-e89b-12d3-a456-426614174000/api-keys
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "name": "CI/CD Integration",
  "expires_in_days": 90,
  "scopes": ["projects:read", "reports:read"]
}
```

#### Response
```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "id": "api_key_id_here",
  "name": "CI/CD Integration",
  "key": "ak_1234567890abcdef...",
  "scopes": ["projects:read", "reports:read"],
  "expires_at": "2024-04-15T10:30:00Z",
  "created_at": "2024-01-15T10:30:00Z"
}
```

### GET /users/{user_id}/api-keys

List API keys for user.

#### Request
```http
GET /users/123e4567-e89b-12d3-a456-426614174000/api-keys
Authorization: Bearer <jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "api_keys": [
    {
      "id": "api_key_id_here",
      "name": "CI/CD Integration",
      "scopes": ["projects:read", "reports:read"],
      "expires_at": "2024-04-15T10:30:00Z",
      "last_used": "2024-01-14T15:20:00Z",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

### DELETE /users/{user_id}/api-keys/{key_id}

Revoke API key.

#### Request
```http
DELETE /users/123e4567-e89b-12d3-a456-426614174000/api-keys/api_key_id_here
Authorization: Bearer <jwt_token>
```

#### Response
```http
HTTP/1.1 204 No Content
```

## Audit API

### GET /audit/events

Get audit events (Admin only).

#### Request
```http
GET /audit/events?start_time=2024-01-01T00:00:00Z&end_time=2024-01-15T23:59:59Z&event_type=Authentication&user_id=123e4567-e89b-12d3-a456-426614174000&page=1&limit=50
Authorization: Bearer <admin_jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "events": [
    {
      "id": "audit_event_id",
      "timestamp": "2024-01-15T10:30:00Z",
      "event_type": "Authentication",
      "user_id": "123e4567-e89b-12d3-a456-426614174000",
      "session_id": "session_id_here",
      "resource": "login",
      "action": "jwt_authentication",
      "outcome": "Success",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "additional_data": {
        "login_method": "password",
        "user_agent": "web_browser"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 1250,
    "total_pages": 25
  }
}
```

### GET /audit/statistics

Get audit statistics (Admin only).

#### Request
```http
GET /audit/statistics?start_time=2024-01-01T00:00:00Z&end_time=2024-01-15T23:59:59Z
Authorization: Bearer <admin_jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "total_events": 15420,
  "events_by_type": {
    "Authentication": 5240,
    "Authorization": 8930,
    "DataAccess": 1100,
    "SecurityViolation": 150
  },
  "events_by_outcome": {
    "Success": 14850,
    "Failure": 420,
    "Denied": 150
  },
  "events_by_day": [
    {
      "date": "2024-01-15",
      "count": 1250
    }
  ],
  "top_users": [
    {
      "user_id": "123e4567-e89b-12d3-a456-426614174000",
      "event_count": 450
    }
  ],
  "security_violations": {
    "failed_logins": 120,
    "unauthorized_access": 30
  }
}
```

### POST /audit/export

Export audit events (Admin only).

#### Request
```http
POST /audit/export
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json

{
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": "2024-01-15T23:59:59Z",
  "format": "csv",
  "event_types": ["Authentication", "Authorization"],
  "include_sensitive_data": false
}
```

#### Response
```http
HTTP/1.1 202 Accepted
Content-Type: application/json

{
  "export_id": "export_123456",
  "status": "processing",
  "estimated_completion": "2024-01-15T10:35:00Z"
}
```

### GET /audit/export/{export_id}

Get export status and download link.

#### Request
```http
GET /audit/export/export_123456
Authorization: Bearer <admin_jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "export_id": "export_123456",
  "status": "completed",
  "download_url": "https://api.uveddi.com/v1/audit/export/export_123456/download",
  "expires_at": "2024-01-16T10:30:00Z",
  "file_size": 2048576,
  "record_count": 15420
}
```

## Rate Limiting API

### GET /rate-limits/status

Get current rate limit status for authenticated user.

#### Request
```http
GET /rate-limits/status
Authorization: Bearer <jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "limits": [
    {
      "endpoint": "/api/projects",
      "requests_made": 45,
      "limit": 100,
      "window_seconds": 60,
      "reset_time": "2024-01-15T10:31:00Z",
      "remaining": 55
    },
    {
      "endpoint": "/api/auth/login",
      "requests_made": 2,
      "limit": 5,
      "window_seconds": 300,
      "reset_time": "2024-01-15T10:35:00Z",
      "remaining": 3
    }
  ],
  "global_limit": {
    "requests_made": 450,
    "limit": 1000,
    "window_seconds": 3600,
    "reset_time": "2024-01-15T11:00:00Z",
    "remaining": 550
  }
}
```

### POST /rate-limits/reset (Admin only)

Reset rate limits for a user.

#### Request
```http
POST /rate-limits/reset
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json

{
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "endpoint": "/api/auth/login"
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "message": "Rate limits reset successfully",
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "endpoint": "/api/auth/login",
  "reset_at": "2024-01-15T10:30:00Z"
}
```

## Configuration API

### GET /config/security (Admin only)

Get current security configuration.

#### Request
```http
GET /config/security
Authorization: Bearer <admin_jwt_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "authentication": {
    "jwt_expiry_hours": 8,
    "session_timeout_hours": 24,
    "oauth_providers": ["google", "azure"],
    "password_policy": {
      "min_length": 8,
      "require_uppercase": true,
      "require_lowercase": true,
      "require_numbers": true,
      "require_symbols": true
    }
  },
  "authorization": {
    "cache_enabled": true,
    "cache_ttl_seconds": 300
  },
  "rate_limiting": {
    "enabled": true,
    "default_rate_limit": 1000,
    "window_seconds": 60
  },
  "audit": {
    "enabled": true,
    "retention_days": 2555,
    "integrity_verification": true
  }
}
```

### PUT /config/security (Admin only)

Update security configuration.

#### Request
```http
PUT /config/security
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json

{
  "authentication": {
    "jwt_expiry_hours": 12,
    "session_timeout_hours": 48
  },
  "rate_limiting": {
    "default_rate_limit": 1500
  }
}
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "message": "Security configuration updated successfully",
  "updated_at": "2024-01-15T10:30:00Z",
  "changes": [
    "authentication.jwt_expiry_hours: 8 -> 12",
    "authentication.session_timeout_hours: 24 -> 48",
    "rate_limiting.default_rate_limit: 1000 -> 1500"
  ]
}
```

## Health and Monitoring API

### GET /health

Get service health status.

#### Request
```http
GET /health
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "components": {
    "database": {
      "status": "healthy",
      "response_time_ms": 5,
      "connections_active": 8,
      "connections_max": 20
    },
    "redis": {
      "status": "healthy",
      "response_time_ms": 2,
      "memory_usage_mb": 45,
      "connected_clients": 12
    },
    "authentication": {
      "status": "healthy",
      "jwt_validation_avg_ms": 3
    },
    "authorization": {
      "status": "healthy",
      "permission_check_avg_ms": 2,
      "cache_hit_rate": 0.85
    }
  }
}
```

### GET /metrics

Get Prometheus metrics.

#### Request
```http
GET /metrics
Authorization: Bearer <monitoring_token>
```

#### Response
```http
HTTP/1.1 200 OK
Content-Type: text/plain

# HELP uveddi_auth_requests_total Total authentication requests
# TYPE uveddi_auth_requests_total counter
uveddi_auth_requests_total{method="jwt",outcome="success"} 15420
uveddi_auth_requests_total{method="jwt",outcome="failure"} 245

# HELP uveddi_authz_checks_total Total authorization checks
# TYPE uveddi_authz_checks_total counter
uveddi_authz_checks_total{outcome="allowed"} 89340
uveddi_authz_checks_total{outcome="denied"} 1250

# HELP uveddi_rate_limit_exceeded_total Rate limit exceeded events
# TYPE uveddi_rate_limit_exceeded_total counter
uveddi_rate_limit_exceeded_total{endpoint="/api/auth/login"} 45

# HELP uveddi_audit_events_total Total audit events logged
# TYPE uveddi_audit_events_total counter
uveddi_audit_events_total{event_type="Authentication"} 5240
uveddi_audit_events_total{event_type="Authorization"} 8930
```

## Error Responses

### Standard Error Format

All error responses follow this format:

```json
{
  "error": {
    "code": "AUTHENTICATION_FAILED",
    "message": "Invalid credentials provided",
    "details": "The email or password is incorrect",
    "timestamp": "2024-01-15T10:30:00Z",
    "request_id": "req_123456789"
  }
}
```

### Error Codes

#### Authentication Errors (401)
- `AUTHENTICATION_FAILED`: Invalid credentials
- `TOKEN_EXPIRED`: JWT token has expired
- `TOKEN_INVALID`: JWT token is malformed or invalid
- `SESSION_EXPIRED`: User session has expired

#### Authorization Errors (403)
- `PERMISSION_DENIED`: User lacks required permissions
- `ROLE_REQUIRED`: Specific role required for this action
- `RESOURCE_ACCESS_DENIED`: Access to specific resource denied

#### Rate Limiting Errors (429)
- `RATE_LIMIT_EXCEEDED`: Request rate limit exceeded
- `QUOTA_EXCEEDED`: API quota exceeded

#### Validation Errors (400)
- `INVALID_REQUEST`: Request format is invalid
- `MISSING_PARAMETER`: Required parameter is missing
- `INVALID_PARAMETER`: Parameter value is invalid

#### Server Errors (500)
- `INTERNAL_ERROR`: Internal server error
- `DATABASE_ERROR`: Database connection or query error
- `SERVICE_UNAVAILABLE`: Service temporarily unavailable

### Rate Limiting Headers

When rate limits are enforced, responses include these headers:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1642248600
X-RateLimit-Window: 60
```

### Security Headers

All responses include security headers:

```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
```

---

This API reference provides comprehensive documentation for integrating with the Uveddi Security and RBAC system. For additional examples and SDKs, refer to the developer documentation.