const express = require('express');
const cors = require('cors');
const rateLimit = require('express-rate-limit');
const helmet = require('helmet');
const compression = require('compression');

// Enhanced error handling middleware
class APIError extends Error {
  constructor(message, statusCode = 500, code = 'INTERNAL_ERROR', isOperational = true) {
    super(message);
    this.statusCode = statusCode;
    this.code = code;
    this.isOperational = isOperational;
    this.timestamp = new Date().toISOString();
    
    Error.captureStackTrace(this, this.constructor);
  }
}

// Circuit breaker for external services
class CircuitBreaker {
  constructor(options = {}) {
    this.failureThreshold = options.failureThreshold || 5;
    this.recoveryTimeout = options.recoveryTimeout || 30000;
    this.monitoringWindow = options.monitoringWindow || 60000;
    
    this.reset();
  }

  reset() {
    this.state = 'CLOSED';
    this.failureCount = 0;
    this.successCount = 0;
    this.lastFailureTime = null;
    this.lastSuccessTime = null;
  }

  async execute(operation) {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime >= this.recoveryTimeout) {
        this.state = 'HALF_OPEN';
        console.log('Circuit breaker attempting recovery');
      } else {
        throw new APIError(
          'Service temporarily unavailable',
          503,
          'CIRCUIT_BREAKER_OPEN'
        );
      }
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  onSuccess() {
    this.successCount++;
    this.lastSuccessTime = Date.now();
    
    if (this.state === 'HALF_OPEN' && this.successCount >= 2) {
      this.reset();
      console.log('Circuit breaker recovered');
    }
  }

  onFailure() {
    this.failureCount++;
    this.lastFailureTime = Date.now();
    
    if (this.failureCount >= this.failureThreshold) {
      this.state = 'OPEN';
      console.log('Circuit breaker opened due to failures');
    } else if (this.state === 'HALF_OPEN') {
      this.state = 'OPEN';
    }
  }

  getStatus() {
    return {
      state: this.state,
      failureCount: this.failureCount,
      successCount: this.successCount,
      lastFailureTime: this.lastFailureTime,
      lastSuccessTime: this.lastSuccessTime
    };
  }
}

// Request timeout middleware
function timeoutMiddleware(timeout = 30000) {
  return (req, res, next) => {
    req.setTimeout(timeout, () => {
      if (!res.headersSent) {
        res.status(408).json({
          error: 'Request timeout',
          code: 'TIMEOUT',
          timestamp: new Date().toISOString()
        });
      }
    });
    next();
  };
}

// Graceful shutdown handler
class GracefulShutdown {
  constructor(server) {
    this.server = server;
    this.connections = new Set();
    this.isShuttingDown = false;
    
    // Track connections
    this.server.on('connection', (connection) => {
      this.connections.add(connection);
      connection.on('close', () => {
        this.connections.delete(connection);
      });
    });
    
    // Handle shutdown signals
    process.on('SIGTERM', () => this.shutdown('SIGTERM'));
    process.on('SIGINT', () => this.shutdown('SIGINT'));
    process.on('uncaughtException', (error) => {
      console.error('Uncaught Exception:', error);
      this.shutdown('uncaughtException');
    });
    process.on('unhandledRejection', (reason, promise) => {
      console.error('Unhandled Rejection at:', promise, 'reason:', reason);
      this.shutdown('unhandledRejection');
    });
  }

  async shutdown(signal) {
    if (this.isShuttingDown) return;
    
    console.log(`\nReceived ${signal}. Starting graceful shutdown...`);
    this.isShuttingDown = true;

    // Stop accepting new requests
    this.server.close(() => {
      console.log('HTTP server closed');
    });

    // Close existing connections
    for (const connection of this.connections) {
      connection.destroy();
    }

    // Cleanup and exit
    setTimeout(() => {
      console.log('Graceful shutdown completed');
      process.exit(0);
    }, 5000);
  }
}

// Health check middleware
function createHealthCheck(dependencies = {}) {
  return async (req, res) => {
    const healthStatus = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      version: process.env.npm_package_version || '1.0.0',
      dependencies: {}
    };

    // Check dependencies
    for (const [name, checkFn] of Object.entries(dependencies)) {
      try {
        const result = await checkFn();
        healthStatus.dependencies[name] = {
          status: 'healthy',
          ...result
        };
      } catch (error) {
        healthStatus.dependencies[name] = {
          status: 'unhealthy',
          error: error.message
        };
        healthStatus.status = 'degraded';
      }
    }

    const statusCode = healthStatus.status === 'healthy' ? 200 : 503;
    res.status(statusCode).json(healthStatus);
  };
}

// Error handling middleware
function errorHandler(error, req, res, next) {
  // Log error
  console.error(`[${new Date().toISOString()}] Error:`, {
    message: error.message,
    stack: error.stack,
    url: req.url,
    method: req.method,
    userAgent: req.get('User-Agent'),
    ip: req.ip
  });

  // Handle operational errors
  if (error.isOperational) {
    return res.status(error.statusCode || 500).json({
      error: error.message,
      code: error.code,
      timestamp: error.timestamp
    });
  }

  // Handle validation errors
  if (error.name === 'ValidationError') {
    return res.status(400).json({
      error: 'Validation failed',
      code: 'VALIDATION_ERROR',
      details: error.details,
      timestamp: new Date().toISOString()
    });
  }

  // Handle file system errors
  if (error.code === 'ENOENT') {
    return res.status(404).json({
      error: 'Requested resource not found',
      code: 'RESOURCE_NOT_FOUND',
      timestamp: new Date().toISOString()
    });
  }

  // Generic error response
  res.status(500).json({
    error: 'Internal server error',
    code: 'INTERNAL_ERROR',
    timestamp: new Date().toISOString(),
    ...(process.env.NODE_ENV === 'development' && { 
      stack: error.stack,
      details: error.message 
    })
  });
}

// Request logging middleware
function requestLogger(req, res, next) {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    const log = {
      timestamp: new Date().toISOString(),
      method: req.method,
      url: req.url,
      status: res.statusCode,
      duration: `${duration}ms`,
      userAgent: req.get('User-Agent'),
      ip: req.ip
    };
    
    // Log based on status code
    if (res.statusCode >= 500) {
      console.error('Request error:', log);
    } else if (res.statusCode >= 400) {
      console.warn('Request warning:', log);
    } else {
      console.log('Request:', log);
    }
  });
  
  next();
}

module.exports = {
  APIError,
  CircuitBreaker,
  timeoutMiddleware,
  GracefulShutdown,
  createHealthCheck,
  errorHandler,
  requestLogger
};
