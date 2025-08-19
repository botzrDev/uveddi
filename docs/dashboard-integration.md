# Dashboard Integration Guide

This document explains how the Uveddi analysis engine integrates with the dashboard visualization system.

## Overview

Uveddi now features a seamless integration between the core CLI analysis tool and the dashboard visualization system. Analysis results are automatically shared with the dashboard, enabling real-time updates and consistent visualization of code quality metrics.

## Key Components

### 1. CLI Analysis Engine

The core analysis engine (`uveddi analyze`) performs static code analysis and detects issues. When analysis completes, it:

- Writes results to the database
- Creates a report file in the specified format (markdown, JSON, HTML)
- Notifies the dashboard of new results

### 2. API Server

The API server provides a REST interface for the dashboard and handles:

- Serving analysis results from the database
- Converting data models between CLI output and dashboard format
- Real-time notifications via WebSockets

### 3. Dashboard UI

The React-based dashboard visualizes analysis results with:

- Summary cards showing key metrics
- Interactive issue lists and filters
- Data visualizations and trend analysis
- Real-time update notifications

## How to Use

### Basic Usage

The simplest way to use the dashboard is:

```bash
uveddi analyze ./your-project --open-dashboard
```

This will:
1. Run analysis on your project
2. Start the dashboard server
3. Open the dashboard in your browser

### Manual Approach

You can also run the analysis and dashboard separately:

```bash
# First run analysis
uveddi analyze ./your-project

# Then start the dashboard server
uveddi serve
```

## Data Flow

1. CLI analyzes code → Writes to database
2. Dashboard reads from database through API server
3. Real-time updates via WebSockets

## Troubleshooting

If you experience issues with dashboard integration:

1. Ensure the database file exists at `./.uveddi/database.db`
2. Check if the dashboard server is running (`uveddi serve`)
3. Verify WebSocket connectivity in browser developer tools
4. Try running with `--verbose` flag for more detailed logs

## Additional Resources

- [Uveddi Dashboard Architecture](./dashboard-architecture.md)
- [API Documentation](./api-docs.md)
- [Configuration Guide](./configuration.md)
