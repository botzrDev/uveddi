# Uveddi Dashboard User Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Dashboard Overview](#dashboard-overview)
3. [Analysis Management](#analysis-management)
4. [Real-Time Monitoring](#real-time-monitoring)
5. [AI-Powered Insights](#ai-powered-insights)
6. [Interactive Visualizations](#interactive-visualizations)
7. [Report Generation](#report-generation)
8. [Configuration Management](#configuration-management)
9. [Troubleshooting](#troubleshooting)
10. [Advanced Features](#advanced-features)

## Getting Started

The Uveddi dashboard provides a comprehensive web-based interface for managing code analysis, monitoring system performance, and exploring AI-powered insights. This guide will help you navigate and utilize all dashboard features effectively.

### Accessing the Dashboard

1. **Local Development**
   ```bash
   # Start the Uveddi server
   cargo run --features tui
   
   # Or use the dedicated frontend
   cd frontend && npm start
   ```
   Access: http://localhost:3000

2. **Production Environment**
   - Navigate to your deployed Uveddi instance
   - Default port: 8080 (configurable)
   - Example: https://uveddi.yourcompany.com

### Authentication

The dashboard supports multiple authentication methods:

- **JWT Token Authentication** (recommended for production)
- **API Key Authentication** (for programmatic access)
- **OAuth2 Integration** (GitHub, Google, etc.)
- **Local Development Mode** (no authentication required)

#### Logging In

1. Navigate to the login page
2. Enter your credentials or select OAuth provider
3. Upon successful authentication, you'll be redirected to the main dashboard

## Dashboard Overview

### Main Navigation

The dashboard is organized into several key sections:

#### 🏠 Home Dashboard
- **Analysis Overview**: High-level metrics and health indicators
- **Recent Activity**: Latest analysis runs and results
- **Quick Actions**: Start new analysis, view reports, configure settings
- **System Status**: Real-time system health and performance

#### 🔍 Analysis Center
- **New Analysis**: Configure and launch code analysis
- **Analysis History**: Browse past analysis results
- **Batch Operations**: Manage multiple analysis jobs
- **Scheduled Analysis**: Set up recurring analysis tasks

#### 📊 Monitoring Hub
- **Real-Time Metrics**: Live system performance data
- **Performance Trends**: Historical performance analysis
- **Alert Management**: Configure and view system alerts
- **Resource Usage**: CPU, memory, and storage monitoring

#### 🤖 AI Insights
- **Analysis Explanations**: AI-powered issue explanations
- **Refactoring Suggestions**: Intelligent code improvement recommendations  
- **Pattern Recognition**: Detected architectural patterns
- **Best Practices**: AI-generated best practice recommendations

#### 📈 Reports & Visualization
- **Interactive Reports**: Explore analysis results with rich visualizations
- **Dependency Graphs**: Visual representation of code dependencies
- **Trend Analysis**: Track code quality metrics over time
- **Export Options**: Download reports in multiple formats

#### ⚙️ Configuration
- **Analysis Settings**: Configure detectors and analysis parameters
- **System Configuration**: Manage system-wide settings
- **User Management**: Manage users and permissions (admin only)
- **Integration Settings**: Configure external integrations

### Dashboard Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Uveddi Logo    [Navigation Menu]         [User] [Settings] │
├─────────────────────────────────────────────────────────────┤
│ [Sidebar]              [Main Content Area]                 │
│                                                             │
│ • Analysis                                                  │
│ • Monitoring           ┌─────────────────────────────────┐  │
│ • AI Insights          │                                 │  │
│ • Reports              │      Primary Content            │  │
│ • Configuration        │                                 │  │
│                        │                                 │  │
│                        └─────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│ Status Bar: Real-time metrics | Last update: 2 min ago     │
└─────────────────────────────────────────────────────────────┘
```

## Analysis Management

### Starting a New Analysis

#### Basic Analysis Setup

1. **Navigate to Analysis Center** → **New Analysis**
2. **Configure Basic Settings**:
   - **Project Path**: Select or enter the path to your project
   - **Languages**: Choose programming languages to analyze
   - **Analysis Type**: Quick scan, comprehensive, or custom
   
3. **Select Detectors**:
   ```
   ☑️ God Object Detection
   ☑️ Dead Code Detection  
   ☑️ Tight Coupling Detection
   ☑️ Large Classes Detection
   ☑️ Cyclic Dependencies
   ☑️ Magic Values Detection
   ☐ Custom Detectors (Plugin-based)
   ```

4. **Advanced Options**:
   - **AI Enhancement**: Enable AI-powered explanations
   - **Performance Profiling**: Monitor analysis performance
   - **Incremental Analysis**: Only analyze changed files
   - **Output Format**: JSON, Markdown, HTML

#### Analysis Configuration Examples

**Quick Scan (Recommended for Development)**
```yaml
Analysis Type: Quick
Duration: ~30 seconds
Detectors: Core anti-patterns only
AI Enhancement: Disabled
Memory Usage: Low
```

**Comprehensive Analysis (Recommended for CI/CD)**
```yaml
Analysis Type: Comprehensive  
Duration: ~5-15 minutes
Detectors: All available detectors
AI Enhancement: Enabled
Memory Usage: Medium
Reports: Full HTML + JSON
```

**Custom Analysis (Advanced Users)**
```yaml
Analysis Type: Custom
Custom Detectors: User-defined plugins
AI Provider: Ollama/OpenAI
Performance Monitoring: Enabled
Batch Processing: Multiple projects
```

### Monitoring Analysis Progress

#### Real-Time Progress Tracking

During analysis execution, the dashboard provides live updates:

1. **Progress Indicator**
   ```
   Analysis Progress: ████████░░ 80%
   Phase: AI Processing
   Files Processed: 245/306
   Issues Found: 23
   Estimated Completion: 2 min 15 sec
   ```

2. **Live Metrics**
   - **CPU Usage**: Real-time CPU consumption
   - **Memory Usage**: Current memory allocation
   - **Throughput**: Files processed per second
   - **Issue Detection Rate**: Issues found per file

3. **Phase Breakdown**
   ```
   ✅ File Discovery (2.1s)
   ✅ AST Parsing (15.3s) 
   ✅ Anti-Pattern Detection (42.1s)
   🔄 AI Analysis (current)
   ⏳ Report Generation
   ```

#### Analysis Queue Management

For multiple concurrent analyses:

1. **Queue View**
   ```
   Active: Project A (80% complete)
   Queued: Project B (waiting)
   Queued: Project C (waiting)
   ```

2. **Priority Management**
   - Drag and drop to reorder queue
   - Set analysis priority levels
   - Cancel or pause running analyses

### Analysis Results

#### Results Overview

Once analysis completes, you'll see a comprehensive summary:

```
┌─────────────────────────────────────────┐
│ Analysis Results - Project XYZ          │
├─────────────────────────────────────────┤
│ Status: ✅ Completed                     │
│ Duration: 3m 42s                        │
│ Files Analyzed: 306                     │
│ Issues Found: 23 (3 Critical, 20 Warn)  │
│ Code Quality Score: 8.2/10              │
│ AI Insights: 15 suggestions             │
└─────────────────────────────────────────┘
```

#### Issue Breakdown

Issues are categorized by type and severity:

**By Type**
- 🏢 God Objects: 3 issues
- ☠️ Dead Code: 8 issues  
- 🔗 Tight Coupling: 7 issues
- 📏 Large Classes: 3 issues
- 🔄 Cyclic Dependencies: 2 issues

**By Severity**
- 🔴 Critical: 3 issues (requires immediate attention)
- 🟡 Warning: 15 issues (should be addressed)
- 🔵 Info: 5 issues (suggestions for improvement)

#### Detailed Issue View

Click on any issue to see detailed information:

```
┌──────────────────────────────────────────────────────────────┐
│ Issue #1: God Object Detected                               │
├──────────────────────────────────────────────────────────────┤
│ File: src/analysis/engine.rs                                │
│ Lines: 45-892 (847 lines)                                   │
│ Severity: Critical                                           │
│ Confidence: 95%                                              │
├──────────────────────────────────────────────────────────────┤
│ Description:                                                 │
│ The class 'AnalysisEngine' has grown too large with 847     │
│ lines of code, exceeding the recommended limit of 500.      │
├──────────────────────────────────────────────────────────────┤
│ AI Explanation:                                              │
│ This class appears to handle multiple responsibilities       │
│ including file parsing, analysis coordination, and result   │
│ aggregation. Consider applying the Single Responsibility    │
│ Principle by extracting separate classes for each concern.   │
├──────────────────────────────────────────────────────────────┤
│ Suggested Refactoring:                                       │
│ 1. Create a FileParsingService for parsing operations       │
│ 2. Extract AnalysisCoordinator for orchestration            │
│ 3. Move result handling to ResultAggregationService         │
├──────────────────────────────────────────────────────────────┤
│ Related Issues: #3, #7, #12                                 │
│ [View Code] [Mark as False Positive] [Create Issue]         │
└──────────────────────────────────────────────────────────────┘
```

## Real-Time Monitoring

### System Performance Dashboard

The monitoring hub provides comprehensive real-time insights:

#### Key Performance Indicators (KPIs)

**System Health**
```
┌────────────────┬────────────────┬────────────────┐
│ CPU Usage      │ Memory Usage   │ Disk Usage     │
│ 25.3% ▓▓▓░░░░  │ 1.2GB ▓▓▓▓░░░ │ 45% ▓▓▓▓▓░░░  │
│ Normal         │ Normal         │ Normal         │
└────────────────┴────────────────┴────────────────┘
```

**Analysis Performance**
```
┌────────────────┬────────────────┬────────────────┐
│ Active Analyses│ Queue Length   │ Avg Duration   │
│ 2              │ 3              │ 4m 23s         │
└────────────────┴────────────────┴────────────────┘
```

**Throughput Metrics**
```
┌────────────────┬────────────────┬────────────────┐
│ Files/Second   │ Issues/Minute  │ API Requests/s │
│ 125.3          │ 45.2          │ 892            │
└────────────────┴────────────────┴────────────────┘
```

#### WebSocket Live Updates

The dashboard uses WebSocket connections for real-time updates:

- **Refresh Rate**: Configurable (default: 1 second)
- **Capacity**: Supports 4.3M+ metrics/second
- **Connection Status**: Shows live connection health
- **Auto-Reconnect**: Handles connection drops gracefully

#### Performance Trend Analysis

**CPU Usage Over Time**
```
CPU % │
100   │                                    ╭─╮
 80   │                                ╭───╯ ╰─╮
 60   │                        ╭───────╯       ╰──╮
 40   │               ╭────────╯                  ╰─╮
 20   │ ──────────────╯                            ╰──
  0   └─┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──
       09:00   09:30   10:00   10:30   11:00   11:30
```

**Memory Usage Pattern**
```
Memory│
 2GB  │ ████████████████████████████████████████████
1.5GB │ ██████████████░░░░██████████████░░░░████████
 1GB  │ ██████░░░░░░░░░░░░██████░░░░░░░░░░░░██████░░
500MB │ ████░░░░░░░░░░░░░░████░░░░░░░░░░░░░░████░░░░
  0   └─┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──
```

#### Alert System

**Alert Configuration**
```
┌─────────────────────────────────────────────────────────┐
│ Alert Thresholds                                        │
├─────────────────────────────────────────────────────────┤
│ CPU Usage:           > 80%    🔴 Critical               │
│ Memory Usage:        > 85%    🔴 Critical               │
│ Disk Usage:          > 90%    🟡 Warning                │
│ Error Rate:          > 5%     🔴 Critical               │
│ Response Time:       > 2s     🟡 Warning                │
│ Analysis Queue:      > 10     🔵 Info                   │
└─────────────────────────────────────────────────────────┘
```

**Active Alerts**
```
🔴 High Memory Usage: 87% (threshold: 85%)
   Since: 11:23 AM (5 minutes ago)
   Actions: [View Details] [Acknowledge] [Snooze]

🟡 Slow Response Time: 2.3s avg (threshold: 2s)  
   Since: 11:15 AM (13 minutes ago)
   Actions: [View Details] [Acknowledge] [Snooze]
```

## AI-Powered Insights

### AI Analysis Features

The dashboard integrates AI capabilities throughout the analysis workflow:

#### Smart Issue Explanations

For each detected issue, AI provides contextual explanations:

**Example: God Object Explanation**
```
┌──────────────────────────────────────────────────────────┐
│ 🤖 AI Analysis                                           │
├──────────────────────────────────────────────────────────┤
│ The AnalysisEngine class exhibits characteristics of a   │
│ God Object anti-pattern. It has grown to handle multiple │
│ concerns including:                                      │
│                                                          │
│ 1. File system operations (25% of methods)              │
│ 2. AST parsing coordination (30% of methods)            │
│ 3. Result aggregation (20% of methods)                  │
│ 4. Plugin management (15% of methods)                   │
│ 5. Configuration handling (10% of methods)              │
│                                                          │
│ This violates the Single Responsibility Principle and   │
│ makes the class difficult to maintain and test.         │
└──────────────────────────────────────────────────────────┘
```

#### Intelligent Refactoring Suggestions

```
┌──────────────────────────────────────────────────────────┐
│ 💡 Refactoring Suggestions                               │
├──────────────────────────────────────────────────────────┤
│ Priority: High                                           │
│ Estimated Effort: 4-6 hours                            │
│ Risk Level: Medium                                       │
├──────────────────────────────────────────────────────────┤
│ Recommended Steps:                                       │
│                                                          │
│ 1. Extract FileSystemService (Priority: High)           │
│    - Move file I/O methods to dedicated service         │
│    - Estimated impact: 15% reduction in class size      │
│                                                          │
│ 2. Create AnalysisOrchestrator (Priority: High)         │
│    - Extract coordination logic                          │
│    - Improved testability and separation of concerns    │
│                                                          │
│ 3. Implement ResultProcessor (Priority: Medium)         │
│    - Dedicated result handling and aggregation          │
│    - Better error handling and performance              │
├──────────────────────────────────────────────────────────┤
│ Benefits:                                                │
│ • Improved maintainability (estimated 40% reduction     │
│   in complexity)                                        │
│ • Better testability (isolated components)              │
│ • Enhanced performance (specialized services)           │
│ • Easier debugging (clear responsibility boundaries)    │
└──────────────────────────────────────────────────────────┘
```

#### Pattern Recognition

AI identifies architectural patterns in your codebase:

```
┌──────────────────────────────────────────────────────────┐
│ 🔍 Detected Patterns                                     │
├──────────────────────────────────────────────────────────┤
│ ✅ Builder Pattern (AnalysisEngineBuilder)               │
│    Confidence: 98%                                       │
│    Usage: Proper implementation, well-structured         │
│                                                          │
│ ⚠️  Singleton Pattern (ConfigManager)                    │
│    Confidence: 85%                                       │
│    Issue: Not thread-safe, consider dependency injection │
│                                                          │
│ ❌ Service Locator (PluginRegistry)                      │
│    Confidence: 92%                                       │
│    Recommendation: Replace with dependency injection     │
│    for better testability                               │
└──────────────────────────────────────────────────────────┘
```

#### AI Configuration

**AI Provider Settings**
```
┌─────────────────────────────────────────────────────────┐
│ AI Configuration                                        │
├─────────────────────────────────────────────────────────┤
│ Provider: Ollama (Local)          [Change Provider]    │
│ Model: llama3.2:latest            [Select Model]       │
│ Temperature: 0.3                  [────░──────]        │
│ Max Tokens: 2048                  [────────░──]        │
│ Enable Caching: ☑️                                      │
│ Context Window: 8192 tokens       [──────────░]        │
├─────────────────────────────────────────────────────────┤
│ Features:                                               │
│ ☑️ Issue Explanations                                   │
│ ☑️ Refactoring Suggestions                              │
│ ☑️ Pattern Recognition                                  │
│ ☑️ Best Practice Recommendations                        │
│ ☐ Code Generation (Beta)                               │
└─────────────────────────────────────────────────────────┘
```

## Interactive Visualizations

### Dependency Graph Visualization

The dashboard provides rich, interactive dependency graphs:

#### Graph Navigation

```
┌─────────────────────────────────────────────────────────────┐
│ Dependency Graph - Project Overview                        │
├─────────────────────────────────────────────────────────────┤
│ [🔍 Zoom In] [🔍 Zoom Out] [📐 Fit to Screen] [⬇️ Export]   │
│                                                             │
│         ┌─────────────┐                                     │
│         │ AnalysisApp │                                     │
│         └──────┬──────┘                                     │
│                │                                            │
│    ┌───────────┼───────────┐                               │
│    │           │           │                               │
│    ▼           ▼           ▼                               │
│ ┌─────┐   ┌─────────┐  ┌────────┐                          │
│ │ CLI │   │ TUI App │  │ Web UI │                          │
│ └─────┘   └────┬────┘  └───┬────┘                          │
│                │           │                               │
│                ▼           ▼                               │
│           ┌──────────────────────┐                         │
│           │   AnalysisEngine     │ ← High coupling detected │
│           └─────────┬────────────┘                         │
│                     │                                      │
│              ┌──────┼──────┐                               │
│              ▼      ▼      ▼                               │
│         ┌─────┐ ┌─────┐ ┌─────┐                            │
│         │ AST │ │ Det │ │ Rep │                            │
│         └─────┘ └─────┘ └─────┘                            │
└─────────────────────────────────────────────────────────────┘
```

#### Interactive Features

1. **Node Interaction**
   - **Hover**: Show detailed information
   - **Click**: Navigate to source code
   - **Double-click**: Expand/collapse dependencies

2. **Graph Controls**
   - **Zoom**: Mouse wheel or controls
   - **Pan**: Click and drag
   - **Filter**: By file type, dependency strength, or issue type
   - **Layout**: Tree, force-directed, circular, hierarchical

3. **Dependency Analysis**
   ```
   Node: AnalysisEngine
   Dependencies: 15 incoming, 8 outgoing
   Coupling Strength: High (0.85)
   Issues: 3 related to this component
   
   [View Source] [Show Issues] [Analyze Dependencies]
   ```

### Code Metrics Visualization

#### Complexity Heatmap

```
File Complexity (Lines of Code)
┌─────────────────────────────────────────────────────────────┐
│ src/                                                        │
│ ├── analysis/                                               │
│ │   ├── engine.rs        ████████████░░ 892 lines (High)    │
│ │   ├── detectors/       ██████░░░░░░░░ 456 lines (Med)     │
│ │   └── cache/           ████░░░░░░░░░░ 234 lines (Low)     │
│ ├── ai/                                                     │
│ │   ├── ollama.rs        ████████░░░░░░ 567 lines (Med)     │
│ │   └── prompts/         ██░░░░░░░░░░░░ 123 lines (Low)     │
│ └── tui/                                                    │
│     ├── app.rs           █████████░░░░░ 678 lines (High)    │
│     └── terminal.rs      ████░░░░░░░░░░ 234 lines (Low)     │
└─────────────────────────────────────────────────────────────┘

Legend: █ High (>500) ▓ Medium (200-500) ░ Low (<200)
```

#### Issue Distribution Chart

```
Issues by Category
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│ God Objects     ████████░░ 8 issues (35%)                  │
│ Dead Code       ██████░░░░ 6 issues (26%)                  │
│ Tight Coupling  █████░░░░░ 5 issues (22%)                  │  
│ Large Classes   ███░░░░░░░ 3 issues (13%)                  │
│ Cyclic Deps     █░░░░░░░░░ 1 issue  (4%)                   │
│                                                             │
│ Total: 23 issues                                            │
└─────────────────────────────────────────────────────────────┘
```

#### Trend Analysis

```
Code Quality Trends (Last 30 Days)
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│ Quality Score                                               │
│  10 ┤                                             ╭─╮       │
│   9 ┤                                     ╭───────╯ ╰───╮   │
│   8 ┤                             ╭───────╯             ╰─╮ │
│   7 ┤                     ╭───────╯                       ╰─│
│   6 ┤             ╭───────╯                                 │
│   5 ┤     ╭───────╯                                         │
│   4 └─────╯                                                 │
│     ├─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬───────┤
│    Day 1  Day 5 Day 10 Day 15 Day 20 Day 25 Day 30      Today│
│                                                             │
│ Issues Count                                                │
│  50 ┤ ╭─╮                                                   │
│  40 ┤ ╯ ╰─╮                                                 │
│  30 ┤     ╰─╮                                               │
│  20 ┤       ╰───╮                                           │
│  10 ┤           ╰─────╮                                     │
│   0 ┤                 ╰─────────────────────────────────────│
└─────────────────────────────────────────────────────────────┘

Trend: ↗️ Improving (Quality Score +2.3, Issues -31)
```

## Report Generation

### Report Types

#### Executive Summary Report

Perfect for stakeholders and management:

```
┌──────────────────────────────────────────────────────────────┐
│              UVEDDI ANALYSIS EXECUTIVE SUMMARY              │
├──────────────────────────────────────────────────────────────┤
│ Project: E-Commerce Platform                                │
│ Analysis Date: 2024-07-21                                   │
│ Duration: 8 minutes 32 seconds                              │
│ Analyzed: 1,247 files (125,432 lines of code)              │
├──────────────────────────────────────────────────────────────┤
│ OVERALL HEALTH SCORE: 7.8/10 (Good)                        │
├──────────────────────────────────────────────────────────────┤
│ KEY FINDINGS:                                                │
│ • 31 issues identified (3 critical, 15 high, 13 medium)    │
│ • Technical debt estimated at 2.3 weeks of development      │
│ • Code maintainability: Above average                       │
│ • Test coverage gaps in payment module                      │
├──────────────────────────────────────────────────────────────┤
│ PRIORITY RECOMMENDATIONS:                                    │
│ 1. Refactor PaymentProcessor (God Object - Critical)        │
│ 2. Remove unused authentication code (Dead Code)            │
│ 3. Reduce coupling in order management system               │
├──────────────────────────────────────────────────────────────┤
│ BUSINESS IMPACT:                                             │
│ • Reduced maintenance costs: ~15% with recommended fixes    │
│ • Improved developer productivity: ~20% time savings        │
│ • Enhanced system reliability: Lower bug rates expected     │
└──────────────────────────────────────────────────────────────┘
```

#### Technical Detailed Report

For developers and technical teams:

```
┌──────────────────────────────────────────────────────────────┐
│                  TECHNICAL ANALYSIS REPORT                  │
├──────────────────────────────────────────────────────────────┤
│ ANTI-PATTERN ANALYSIS                                        │
├──────────────────────────────────────────────────────────────┤
│ God Objects (3 detected):                                   │
│ ┌─────────────────────────┬──────────┬─────────┬────────────┐ │
│ │ File                    │ Lines    │ Methods │ Complexity │ │
│ ├─────────────────────────┼──────────┼─────────┼────────────┤ │
│ │ PaymentProcessor.py     │ 1,247    │ 42      │ 23.5       │ │
│ │ OrderManager.rs         │ 892      │ 35      │ 18.2       │ │
│ │ UserController.js       │ 756      │ 28      │ 15.8       │ │
│ └─────────────────────────┴──────────┴─────────┴────────────┘ │
├──────────────────────────────────────────────────────────────┤
│ Dead Code Analysis (8 locations):                           │
│ • Unused functions: 5                                       │
│ • Unreachable code blocks: 2                               │
│ • Deprecated methods still referenced: 1                    │
│ • Total lines that can be removed: 234                     │
├──────────────────────────────────────────────────────────────┤
│ DEPENDENCY ANALYSIS                                          │
│ • Circular dependencies: 2 detected                         │
│ • High coupling components: 5                               │
│ • Architectural violations: 3                               │
│ • Suggested refactoring opportunities: 12                   │
├──────────────────────────────────────────────────────────────┤
│ PERFORMANCE METRICS                                          │
│ • Analysis throughput: 145.6 files/minute                   │
│ • Memory usage: 423 MB peak                                 │
│ • AI processing time: 2m 15s (26% of total)                │
│ • Cache hit rate: 78%                                       │
└──────────────────────────────────────────────────────────────┘
```

#### AI Insights Report

Leveraging AI analysis for deeper insights:

```
┌──────────────────────────────────────────────────────────────┐
│                    AI-POWERED INSIGHTS                      │
├──────────────────────────────────────────────────────────────┤
│ ARCHITECTURAL PATTERNS DETECTED:                             │
│                                                              │
│ ✅ Repository Pattern - Well implemented                     │
│    Files: UserRepository.py, OrderRepository.py             │
│    Quality: High adherence to pattern principles            │
│                                                              │
│ ⚠️  Observer Pattern - Partially implemented                 │
│    Files: EventManager.js, NotificationService.js           │
│    Issue: Missing proper decoupling in some observers       │
│                                                              │
│ ❌ Singleton Pattern - Anti-pattern detected                 │
│    Files: DatabaseConnection.py, ConfigManager.rs           │
│    Problem: Global state causing testing difficulties       │
├──────────────────────────────────────────────────────────────┤
│ CODE SMELL ANALYSIS:                                         │
│                                                              │
│ 🔴 Feature Envy (High Priority)                             │
│    Location: OrderProcessor.calculateTotalWithTax()         │
│    Issue: Method extensively uses TaxCalculator fields      │
│    Suggestion: Move method to TaxCalculator class           │
│                                                              │
│ 🟡 Data Class (Medium Priority)                             │
│    Location: UserProfile.py                                 │
│    Issue: Class only holds data without behavior            │
│    Suggestion: Add validation and business logic methods    │
├──────────────────────────────────────────────────────────────┤
│ REFACTORING RECOMMENDATIONS:                                 │
│                                                              │
│ 1. Extract Service Layer (Estimated effort: 8 hours)        │
│    • Move business logic from controllers                   │
│    • Create dedicated service classes                       │
│    • Benefits: Better testability, cleaner separation       │
│                                                              │
│ 2. Implement Dependency Injection (Estimated effort: 16h)   │
│    • Replace singleton patterns                             │
│    • Use IoC container for dependency management            │
│    • Benefits: Improved testability and flexibility         │
│                                                              │
│ 3. Add Abstract Factory for Payment Processing (6 hours)    │
│    • Create factory for different payment providers         │
│    • Reduce coupling between order and payment systems      │
│    • Benefits: Easier to add new payment methods            │
└──────────────────────────────────────────────────────────────┘
```

### Report Export Options

#### Export Formats

1. **JSON** - Machine-readable data
   ```json
   {
     "analysis_id": "analysis_2024_07_21_001",
     "timestamp": "2024-07-21T14:30:00Z",
     "summary": {
       "total_files": 1247,
       "issues_found": 31,
       "quality_score": 7.8
     },
     "issues": [...],
     "ai_insights": [...],
     "performance_metrics": {...}
   }
   ```

2. **Markdown** - Human-readable documentation
   - Structured headings and sections
   - Code examples with syntax highlighting
   - Links and cross-references
   - Compatible with documentation systems

3. **HTML** - Rich interactive reports
   - Interactive charts and graphs
   - Collapsible sections
   - Search and filtering capabilities
   - Print-friendly stylesheets

4. **PDF** - Professional documents
   - Executive-ready formatting
   - Charts and visualizations
   - Table of contents
   - Suitable for presentations

#### Automated Report Distribution

Configure automatic report delivery:

```
┌─────────────────────────────────────────────────────────────┐
│ Report Distribution Settings                                │
├─────────────────────────────────────────────────────────────┤
│ Email Recipients:                                           │
│ • tech-leads@company.com (Technical Report)                │
│ • management@company.com (Executive Summary)               │
│ • dev-team@company.com (Detailed Issues)                   │
├─────────────────────────────────────────────────────────────┤
│ Slack Integration:                                          │
│ • Channel: #code-quality                                   │
│ • Trigger: Critical issues found                           │
│ • Format: Summary with link to full report                 │
├─────────────────────────────────────────────────────────────┤
│ Schedule:                                                   │
│ • Daily: Quick analysis summary                            │
│ • Weekly: Comprehensive trend report                       │
│ • Monthly: Executive dashboard update                      │
│ • On-demand: Full analysis completion                      │
└─────────────────────────────────────────────────────────────┘
```

## Configuration Management

### Analysis Configuration

#### Global Settings

```
┌─────────────────────────────────────────────────────────────┐
│ Global Analysis Configuration                               │
├─────────────────────────────────────────────────────────────┤
│ Performance Settings:                                       │
│ • Max concurrent analyses: [5    ] (1-20)                  │
│ • Default timeout (minutes): [30 ] (5-180)                 │
│ • Memory limit (MB): [2048] (512-8192)                     │
│ • Enable parallel processing: ☑️                            │
├─────────────────────────────────────────────────────────────┤
│ Detection Thresholds:                                       │
│ • God Object - Max lines: [500 ] (100-2000)               │
│ • God Object - Max methods: [20  ] (5-50)                  │
│ • Large Class - Threshold: [300 ] (100-1000)              │
│ • Cyclic Dependency - Max depth: [5   ] (2-15)            │
├─────────────────────────────────────────────────────────────┤
│ Language-Specific Settings:                                 │
│ Python:                                                     │
│ • Line length limit: [88  ] characters                     │
│ • Complexity threshold: [10  ] (1-20)                      │
│ • Enable type checking: ☑️                                  │
│                                                             │
│ Rust:                                                       │
│ • Max function length: [100] lines                         │
│ • Clippy warnings as errors: ☑️                            │
│ • Check unsafe code blocks: ☑️                             │
│                                                             │
│ JavaScript/TypeScript:                                      │
│ • ESLint integration: ☑️                                    │
│ • Complexity threshold: [15  ] (1-25)                      │
│ • Check for unused variables: ☑️                           │
└─────────────────────────────────────────────────────────────┘
```

#### Project-Specific Configuration

Create custom configurations for different projects:

```
┌─────────────────────────────────────────────────────────────┐
│ Project Configuration: E-Commerce Platform                 │
├─────────────────────────────────────────────────────────────┤
│ Project Path: /projects/ecommerce-platform                 │
│ Configuration File: .uveddi/config.toml                    │
├─────────────────────────────────────────────────────────────┤
│ Exclude Patterns:                                           │
│ • node_modules/**                                           │
│ • target/**                                                 │
│ • *.test.js                                                 │
│ • migrations/**                                             │
│ • docs/**                                                   │
├─────────────────────────────────────────────────────────────┤
│ Custom Detectors:                                           │
│ ☑️ Payment Security Checker (custom plugin)                │
│ ☑️ Database Query Validator                                 │
│ ☐ API Rate Limiting Checker                                │
├─────────────────────────────────────────────────────────────┤
│ Integration Settings:                                       │
│ • Git hook integration: ☑️ (pre-commit)                    │
│ • CI/CD pipeline: ☑️ (GitHub Actions)                      │
│ • IDE integration: ☑️ (VS Code extension)                  │
└─────────────────────────────────────────────────────────────┘
```

### User Management (Admin)

For enterprise deployments with multiple users:

```
┌─────────────────────────────────────────────────────────────┐
│ User Management Dashboard                                   │
├─────────────────────────────────────────────────────────────┤
│ Active Users (24):                                          │
│ ┌─────────────────────┬─────────────┬──────────┬──────────┐ │
│ │ User                │ Role        │ Last Seen│ Projects │ │
│ ├─────────────────────┼─────────────┼──────────┼──────────┤ │
│ │ alice@company.com   │ Admin       │ 2 min ago│ 15       │ │
│ │ bob@company.com     │ Developer   │ 1 hr ago │ 8        │ │
│ │ charlie@company.com │ Viewer      │ 3 hrs ago│ 3        │ │
│ │ diana@company.com   │ Developer   │ 1 day ago│ 12       │ │
│ └─────────────────────┴─────────────┴──────────┴──────────┘ │
├─────────────────────────────────────────────────────────────┤
│ Role Permissions:                                           │
│ Admin:    Full access, user management, system config      │
│ Developer: Run analysis, view reports, configure projects  │
│ Viewer:   View reports, access dashboard (read-only)       │
├─────────────────────────────────────────────────────────────┤
│ Actions:                                                    │
│ [Add User] [Import from LDAP] [Export User List]           │
└─────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Common Issues

#### Analysis Performance Issues

**Problem**: Analysis taking too long or consuming excessive memory

**Diagnostic Steps**:
1. Check system resources in the monitoring hub
2. Review analysis configuration settings
3. Examine project size and complexity

**Solutions**:
```
Performance Optimization Checklist:
☑️ Enable incremental analysis for large projects
☑️ Adjust memory limits based on system capacity
☑️ Use exclude patterns to skip unnecessary files
☑️ Enable parallel processing if CPU cores available
☑️ Consider using batch mode for very large codebases
```

#### WebSocket Connection Issues

**Problem**: Real-time updates not working, connection drops

**Diagnostic Panel**:
```
┌─────────────────────────────────────────────────────────────┐
│ WebSocket Diagnostics                                       │
├─────────────────────────────────────────────────────────────┤
│ Connection Status: ❌ Disconnected                          │
│ Last Successful Connection: 5 minutes ago                   │
│ Reconnection Attempts: 3/5                                  │
│ Error: Connection timeout after 30 seconds                 │
├─────────────────────────────────────────────────────────────┤
│ Network Information:                                        │
│ • Client IP: 192.168.1.100                                 │
│ • Server Port: 8080                                         │
│ • Protocol: WSS (Secure WebSocket)                         │
│ • Proxy: Detected (may cause issues)                       │
├─────────────────────────────────────────────────────────────┤
│ Troubleshooting Actions:                                    │
│ [Test Connection] [Clear Cache] [Download Logs]            │
│ [Check Firewall] [Verify Certificates]                     │
└─────────────────────────────────────────────────────────────┘
```

#### Authentication Problems

**Problem**: Unable to log in or frequent session timeouts

**Debug Information**:
```
Authentication Status: Failed
Error Code: JWT_EXPIRED
Token Expiry: 2024-07-21 10:30:00 UTC (2 hours ago)
Last Refresh Attempt: Failed at 12:45:00 UTC
Server Time: 2024-07-21 14:45:00 UTC
Client Time: 2024-07-21 14:45:00 UTC (✓ Synchronized)

Suggested Actions:
1. Clear browser cache and cookies
2. Check system time synchronization
3. Verify JWT configuration
4. Contact administrator if issues persist
```

### System Health Dashboard

```
┌─────────────────────────────────────────────────────────────┐
│ System Health Check                                         │
├─────────────────────────────────────────────────────────────┤
│ ✅ API Server: Operational                                  │
│ ✅ Database: Connected (PostgreSQL 13.4)                   │
│ ✅ WebSocket Service: Running (24 active connections)      │
│ ✅ AI Service: Available (Ollama - llama3.2:latest)        │
│ ⚠️  Cache Service: High memory usage (85%)                 │
│ ❌ Plugin System: 2 plugins failed to load                 │
├─────────────────────────────────────────────────────────────┤
│ Recent Errors (Last 24h):                                  │
│ • 14:23 - Plugin load failure: custom_detector.wasm        │
│ • 14:15 - High memory usage warning                        │
│ • 13:45 - WebSocket connection timeout (resolved)          │
├─────────────────────────────────────────────────────────────┤
│ System Resources:                                           │
│ • CPU: 45% (Normal)                                         │
│ • Memory: 2.1GB / 4GB (High)                               │
│ • Disk: 120GB / 500GB (Normal)                             │
│ • Network: 15 Mbps in/out (Normal)                         │
└─────────────────────────────────────────────────────────────┘
```

## Advanced Features

### API Integration

The dashboard can be integrated with external tools and systems:

#### REST API Usage

```javascript
// Example: Triggering analysis from external systems
const analysisRequest = {
  project_path: "/path/to/project",
  languages: ["rust", "python"],
  detectors: ["god_object", "dead_code"],
  enable_ai: true
};

fetch('/api/v1/analysis', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer YOUR_JWT_TOKEN'
  },
  body: JSON.stringify(analysisRequest)
})
.then(response => response.json())
.then(data => {
  console.log('Analysis started:', data.analysis_id);
});
```

#### WebSocket Integration

```javascript
// Example: Real-time monitoring integration
const ws = new WebSocket('ws://localhost:8080/ws/monitoring');

ws.onmessage = function(event) {
  const data = JSON.parse(event.data);
  
  switch(data.type) {
    case 'metrics':
      updateDashboard(data.payload);
      break;
    case 'analysis_progress':
      updateProgressBar(data.payload);
      break;
    case 'alerts':
      showAlert(data.payload);
      break;
  }
};
```

### Custom Dashboards

Create personalized dashboard views:

```
┌─────────────────────────────────────────────────────────────┐
│ Custom Dashboard: Team Lead View                            │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┬─────────────────────────────────────┐ │
│ │ Active Projects (6) │ Team Performance                    │ │
│ │ ├─ Project Alpha     │ ┌─────────────────────────────────┐ │ │
│ │ ├─ Project Beta      │ │ Issues Resolved: 45 this week  │ │ │
│ │ ├─ Project Gamma     │ │ Code Quality: ↗️ +2.3 points   │ │ │
│ │ ├─- Project Delta    │ │ Technical Debt: ↘️ -15%        │ │ │
│ │ ├─ Project Epsilon   │ └─────────────────────────────────┘ │ │
│ │ └─ Project Zeta      │                                     │ │
│ └─────────────────────┴─────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Critical Issues Requiring Attention                        │ │
│ │ • PaymentProcessor refactoring (Project Alpha) - 3 days    │ │
│ │ • Security vulnerability in auth module (Beta) - 1 day     │ │
│ │ • Performance regression in search (Gamma) - 2 days        │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────┬─────────────────────────────────────┐ │
│ │ Weekly Trends       │ Resource Allocation                 │ │
│ │ [Quality Graph]     │ • Analysis Queue: 3 pending        │ │
│ │                     │ • CPU Usage: 34% average           │ │
│ │                     │ • Developer Hours: 156/week        │ │
│ │                     │ • AI Analysis: 78% adoption        │ │
│ └─────────────────────┴─────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Integration with Development Tools

#### IDE Extensions

The dashboard can be integrated with popular IDEs:

- **VS Code Extension**: Real-time issue highlighting
- **IntelliJ Plugin**: Integrated analysis results
- **Vim Plugin**: Command-line integration
- **Emacs Package**: Buffer-level analysis

#### CI/CD Pipeline Integration

```yaml
# Example GitHub Actions integration
name: Code Quality Analysis
on: [push, pull_request]

jobs:
  analysis:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Run Uveddi Analysis
      uses: uveddi/github-action@v1
      with:
        project-path: '.'
        enable-ai: true
        fail-on-critical: true
        upload-results: true
    - name: Upload Results to Dashboard
      run: |
        curl -X POST "$UVEDDI_DASHBOARD_URL/api/v1/analysis" \
          -H "Authorization: Bearer $UVEDDI_TOKEN" \
          -H "Content-Type: application/json" \
          -d @analysis-results.json
```

This comprehensive dashboard guide covers all aspects of using Uveddi's web interface effectively, from basic navigation to advanced integrations and troubleshooting. The dashboard provides a powerful, user-friendly way to manage code analysis, monitor system performance, and leverage AI insights for improved code quality.