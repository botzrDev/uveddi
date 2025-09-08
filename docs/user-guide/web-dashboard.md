# Web Dashboard User Guide

## Overview

Uveddi provides a powerful web-based dashboard for interactive code analysis and visualization. The dashboard runs at `http://localhost:8888` and provides real-time insights into your codebase's architecture, dependencies, and quality metrics.

## Getting Started

### Starting the Dashboard

Launch the web dashboard with all required services:

```bash
# Start with default ports
uveddi serve

# Custom port configuration
uveddi serve --port 8888 --rendering-port 3333

# Development mode with hot-reload
uveddi serve --port 8888 --rendering-port 3333 --frontend-port 3000 --development

# With custom database path
uveddi serve --database-path ./uveddi.db
```

### Service URLs

Once started, the following services are available:

- **Dashboard**: `http://localhost:8888` - Main interactive dashboard
- **API**: `http://localhost:8888/api/v1` - REST API endpoints
- **Health Check**: `http://localhost:8888/health` - Service health status
- **Rendering Service**: `http://localhost:3333` - Diagram rendering engine
- **Frontend Dev Server**: `http://localhost:3000` - (Development mode only)

## Dashboard Features

### 1. Real-Time Analysis View

The main dashboard provides an interactive overview of your analysis results:

- **Issue Summary**: Categorized view of detected issues by severity
- **Code Metrics**: Lines of code, complexity scores, and quality indicators
- **Dependency Graph**: Interactive visualization of module dependencies
- **Anti-Pattern Detection**: Visual representation of detected anti-patterns

### 2. Interactive Issue Explorer

Navigate through detected issues with:

- **Filtering**: Filter by severity, type, or file path
- **Sorting**: Sort by impact, file, or detection confidence
- **Details View**: Click any issue for detailed explanation and remediation suggestions
- **Code Preview**: View affected code snippets with syntax highlighting

### 3. Dependency Visualization

Explore your codebase's dependency structure:

- **Interactive Graph**: Zoom, pan, and click nodes for details
- **Dependency Metrics**: Coupling scores, cycle detection, and impact analysis
- **Module Explorer**: Navigate through module hierarchy
- **Export Options**: Export dependency graphs as SVG or PNG

### 4. Analysis Reports

Generate and view comprehensive reports:

- **Report Types**: HTML, JSON, Markdown formats
- **Custom Templates**: Apply different report templates
- **Export Functions**: Download or share reports
- **Report History**: Access previous analysis runs

### 5. Real-Time Monitoring

Monitor analysis progress in real-time:

- **Progress Indicators**: Track analysis stages
- **Live Updates**: See issues as they're detected
- **Performance Metrics**: Monitor resource usage
- **WebSocket Streaming**: Real-time data updates

## Navigation Guide

### Main Menu

The dashboard navigation menu includes:

- **Dashboard**: Main overview and metrics
- **Analysis**: Run new analysis or view results
- **Issues**: Detailed issue explorer
- **Dependencies**: Dependency graph and metrics
- **Reports**: Report generation and history
- **Settings**: Configuration options
- **Help**: Documentation and support

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open command palette |
| `Ctrl+/` | Toggle help overlay |
| `Ctrl+F` | Search/filter current view |
| `Ctrl+R` | Refresh data |
| `Ctrl+E` | Export current view |
| `Esc` | Close modal/overlay |

## Configuration Options

### Dashboard Settings

Configure the dashboard through the Settings panel:

```toml
[dashboard]
theme = "dark"  # light, dark, auto
refresh_interval = 5  # seconds
auto_analyze = false
show_notifications = true
```

### Analysis Configuration

Set analysis parameters directly from the dashboard:

- **Language Selection**: Choose which languages to analyze
- **Detector Settings**: Enable/disable specific detectors
- **Threshold Configuration**: Adjust detection thresholds
- **AI Integration**: Configure AI-powered insights

## Working with Analysis Results

### Starting a New Analysis

1. Click "New Analysis" in the dashboard
2. Select target directory or paste path
3. Configure analysis options:
   - Enable/disable detectors
   - Set thresholds
   - Choose output formats
4. Click "Start Analysis"
5. Monitor progress in real-time

### Viewing Results

Analysis results are displayed in multiple views:

- **Summary View**: High-level metrics and charts
- **Table View**: Sortable, filterable issue list
- **Tree View**: File hierarchy with issue indicators
- **Graph View**: Visual representation of relationships

### Exporting Results

Export analysis results in various formats:

```bash
# From dashboard UI
Click "Export" → Select format → Download

# Available formats:
- HTML Report (interactive)
- JSON (machine-readable)
- Markdown (documentation)
- CSV (spreadsheet)
```

## Integration Features

### CI/CD Integration

The dashboard supports CI/CD pipeline integration:

```yaml
# GitHub Actions example
- name: Run Uveddi Analysis
  run: |
    uveddi serve --port 8888 &
    sleep 5
    curl -X POST http://localhost:8888/api/v1/analysis/start \
      -H "Content-Type: application/json" \
      -d '{"path": "./src", "output_format": "json"}'
```

### API Access

Programmatically interact with the dashboard:

```javascript
// Start analysis via API
const response = await fetch('http://localhost:8888/api/v1/analysis/start', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    path: './src',
    enable_ai: true,
    output_format: 'json'
  })
});

// Get results
const results = await fetch(`http://localhost:8888/api/v1/analysis/${analysisId}/results`);
```

### Webhook Notifications

Configure webhooks for analysis events:

```json
{
  "webhook_url": "https://your-server.com/webhook",
  "events": ["analysis.started", "analysis.completed", "issue.detected"],
  "secret": "your-webhook-secret"
}
```

## Advanced Features

### Custom Plugins

The dashboard supports custom plugin integration:

- Upload plugins through the UI
- Configure plugin settings
- View plugin-detected issues
- Access plugin-specific visualizations

### Team Collaboration

Share analysis results with your team:

- **Shareable Links**: Generate links to specific views
- **Comments**: Add comments to issues
- **Annotations**: Mark issues as resolved/ignored
- **Export Reports**: Share comprehensive reports

### Performance Optimization

Optimize dashboard performance:

- **Lazy Loading**: Large datasets load on-demand
- **Caching**: Results cached for faster access
- **Pagination**: Navigate large result sets efficiently
- **Virtual Scrolling**: Smooth scrolling for long lists

## Troubleshooting

### Common Issues

#### Dashboard won't start
```bash
# Check if ports are available
lsof -i :8888
lsof -i :3333

# Start with different ports
uveddi serve --port 9000 --rendering-port 4000
```

#### Rendering service errors
```bash
# Install Playwright dependencies
npx playwright install
npx playwright install-deps

# Check rendering service health
curl http://localhost:3333/health
```

#### Slow performance
- Reduce refresh interval in settings
- Enable pagination for large result sets
- Clear browser cache
- Check system resources

### Debug Mode

Enable debug mode for detailed logging:

```bash
RUST_LOG=debug uveddi serve --port 8888
```

## Best Practices

1. **Regular Analysis**: Schedule periodic analysis runs
2. **Threshold Tuning**: Adjust thresholds based on your codebase
3. **Report Archival**: Save important reports for comparison
4. **Team Standards**: Configure shared analysis profiles
5. **Integration First**: Integrate with CI/CD early

## Security Considerations

The dashboard includes security features:

- **Local-only by default**: Binds to localhost
- **CORS protection**: Configurable CORS headers
- **Rate limiting**: API request throttling
- **Input validation**: Sanitized user inputs

To expose the dashboard externally:

```bash
# WARNING: Only for trusted networks
uveddi serve --port 8888 --bind 0.0.0.0
```

## Next Steps

- [API Reference](../08-api/rest-api-reference.md) - Detailed API documentation
- [Plugin Development](../04-development/plugin-development.md) - Create custom plugins
- [Configuration Guide](configuration-options.md) - Advanced configuration
- [Troubleshooting](troubleshooting.md) - Detailed troubleshooting guide